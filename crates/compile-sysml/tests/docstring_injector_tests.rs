use compile_sysml::docstring::{
    inject_dart_file_docstrings, inject_rust_file_docstrings, run_docstring_injection,
    run_extract_source, DocstringExitCode, DocstringInjectionOptions,
};
use compile_sysml::{count_structural_elements, generate_digest, parse_sysml};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_test_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!("{}_{}_{}", prefix, std::process::id(), nanos));
    fs::create_dir_all(&path).expect("create unique temp dir");
    path
}

#[test]
fn test_ac03_and_ac04_rust_and_dart_docstring_synthesis_and_idempotency() {
    let dir = unique_test_dir("feat42_docstring_synth");
    let schema_sysml = r#"
package SafetyModel {
    part def TelemetryPacket {
        attribute sequence_id: Integer;
        attribute timestamp_ms: Real;
        constraint def NonNegativeTimestamp {
            doc /* timestamp_ms >= 0.0 */
            timestamp_ms >= 0.0
        }
    }

    part def GuidanceController {
        attribute gain_kp: Real;
        action def compute_command {
            in target_altitude: Real;
            out thrust_newtons: Real;
        }
    }
}
"#;
    let schema_model = parse_sysml(schema_sysml).expect("parse schema_sysml");

    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");

    let rust_file = src_dir.join("telemetry.rs");
    let initial_rust = r#"//! Telemetry packet module.

/// Member documentation.
#[derive(Debug, Clone, PartialEq)]
pub struct TelemetryPacket {
    pub sequence_id: u64,
    pub timestamp_ms: f64,
}

pub enum TelemetryStatus {
    Nominal,
    Degraded,
    Faulted,
}

pub trait PacketValidator {
    fn validate_packet(&self) -> Result<bool, String>;
}

impl TelemetryPacket {
    pub fn new(sequence_id: u64, timestamp_ms: f64) -> Result<Self, String> {
        assert!(timestamp_ms >= 0.0);
        Ok(Self {
            sequence_id,
            timestamp_ms,
        })
    }

    pub fn sequence_id(&self) -> u64 {
        self.sequence_id
    }
}
"#;
    fs::write(&rust_file, initial_rust).expect("write telemetry.rs");

    let dart_file = src_dir.join("guidance_controller.dart");
    let initial_dart = r#"/// Member documentation.
class GuidanceController {
  final double gainKp;

  const GuidanceController({required this.gainKp});

  double computeCommand(double targetAltitude) {
    assert(targetAltitude >= 0.0);
    return targetAltitude * gainKp;
  }
}

enum FlightMode {
  manual,
  autonomous,
}

mixin TelemetryLoggingMixin {
  void recordSample(int sequenceId) {}
}

extension FlightModeExtension on FlightMode {
  bool get isAutonomous => this == FlightMode.autonomous;
}

typedef CommandCallback = void Function(double thrustNewtons);
"#;
    fs::write(&dart_file, initial_dart).expect("write guidance_controller.dart");

    // First run: synthesize and inject 7-section docstrings
    let opts = DocstringInjectionOptions {
        target_path: src_dir.clone(),
        dry_run: false,
        force: false,
        schema_model: Some(schema_model.clone()),
    };
    let report_1 = run_docstring_injection(&opts).expect("first docstring injection run");
    assert_eq!(report_1.files_scanned, 2);
    assert_eq!(report_1.files_modified, 2);
    assert!(
        report_1.items_enriched >= 10,
        "Expected at least 10 items enriched across Rust and Dart, got {}",
        report_1.items_enriched
    );

    let rust_after_1 = fs::read_to_string(&rust_file).expect("read rust after run 1");
    let dart_after_1 = fs::read_to_string(&dart_file).expect("read dart after run 1");

    // Verify zero placeholder strings remain
    assert!(!rust_after_1.contains("Member documentation."));
    assert!(!dart_after_1.contains("Member documentation."));

    // Verify all 7 sections and Realises tags exist in both Rust and Dart outputs
    for content in [&rust_after_1, &dart_after_1] {
        assert!(content.contains("/// ## 1. Safety Intent & Regulatory Scope"));
        assert!(content.contains("/// ## 2. Invariants & Mathematical Properties"));
        assert!(content.contains("/// ## 3. Preconditions"));
        assert!(content.contains("/// ## 4. Postconditions"));
        assert!(content.contains("/// ## 5. Algorithmic Complexity Bounds"));
        assert!(content.contains("/// ## 6. Failure Modes & Fault Tolerance"));
        assert!(content.contains("/// ## 7. Realization & Traceability"));
        assert!(content.contains("/// Realises: [REQ-"));
    }

    // Verify schema constraint propagation into TelemetryPacket docstring
    assert!(rust_after_1.contains("NonNegativeTimestamp"));

    // Verify #[derive(...)] remains immediately above `pub struct TelemetryPacket` (docstring injected before attributes)
    let derive_pos = rust_after_1
        .find("#[derive(Debug, Clone, PartialEq)]")
        .expect("find derive attribute");
    let struct_pos = rust_after_1
        .find("pub struct TelemetryPacket")
        .expect("find TelemetryPacket struct");
    let before_derive = rust_after_1.get(..derive_pos).unwrap_or("");
    let section_7_pos = before_derive
        .rfind("/// ## 7. Realization & Traceability")
        .expect("docstring before derive");
    assert!(section_7_pos < derive_pos && derive_pos < struct_pos);

    let rust_digest_1 = generate_digest(&schema_model, &rust_after_1).sha256;
    let dart_digest_1 = generate_digest(&schema_model, &dart_after_1).sha256;

    // Second run (AC-04): bit-for-bit idempotency with 0 files modified
    let report_2 = run_docstring_injection(&opts).expect("second docstring injection run");
    assert_eq!(report_2.files_scanned, 2);
    assert_eq!(
        report_2.files_modified, 0,
        "Second run must modify 0 files for idempotency"
    );
    assert_eq!(report_2.items_enriched, 0);

    let rust_after_2 = fs::read_to_string(&rust_file).expect("read rust after run 2");
    let dart_after_2 = fs::read_to_string(&dart_file).expect("read dart after run 2");
    assert_eq!(rust_after_1, rust_after_2);
    assert_eq!(dart_after_1, dart_after_2);
    assert_eq!(
        rust_digest_1,
        generate_digest(&schema_model, &rust_after_2).sha256
    );
    assert_eq!(
        dart_digest_1,
        generate_digest(&schema_model, &dart_after_2).sha256
    );

    fs::remove_dir_all(&dir).expect("cleanup temp dir");
}

#[test]
fn test_duplicate_function_docstrings_are_disambiguated() {
    let duplicate_rust = r#"/// Shared docstring.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Shared intent.
///
/// ## 2. Invariants & Mathematical Properties
/// Shared invariant.
///
/// ## 3. Preconditions
/// Shared precondition.
///
/// ## 4. Postconditions
/// Shared postcondition.
///
/// ## 5. Algorithmic Complexity Bounds
/// O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// None.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SHARED-001]
pub fn first_op(alpha: i32) -> i32 {
    alpha + 1
}

/// Shared docstring.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Shared intent.
///
/// ## 2. Invariants & Mathematical Properties
/// Shared invariant.
///
/// ## 3. Preconditions
/// Shared precondition.
///
/// ## 4. Postconditions
/// Shared postcondition.
///
/// ## 5. Algorithmic Complexity Bounds
/// O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// None.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SHARED-001]
pub fn second_op(beta: i32) -> i32 {
    beta + 2
}
"#;

    let (updated, enriched) =
        inject_rust_file_docstrings(duplicate_rust, "crates/demo/src/ops.rs", None, false)
            .expect("inject rust docstrings");
    assert_eq!(
        enriched, 1,
        "First compliant function docstring should be kept and second duplicate should be re-synthesized"
    );
    assert!(updated.contains("second_op"));

    // Second pass on updated must report 0 changes
    let (updated_again, enriched_again) =
        inject_rust_file_docstrings(&updated, "crates/demo/src/ops.rs", None, false)
            .expect("second pass");
    assert_eq!(enriched_again, 0);
    assert_eq!(updated, updated_again);
}

#[test]
fn test_extract_source_and_cli_flags() {
    let dir = unique_test_dir("feat42_extract_source");
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).expect("create src");

    fs::write(
        src_dir.join("controller.rs"),
        r#"
pub struct AttitudeController {
    pub pitch_deg: f64,
    pub roll_deg: f64,
}

impl AttitudeController {
    pub fn stabilize(&self, target_pitch: f64) -> f64 {
        assert!(target_pitch.is_finite());
        target_pitch - self.pitch_deg
    }
}
"#,
    )
    .expect("write controller.rs");

    let out_sysml = dir.join("extracted.sysml");
    let out_digest = dir.join("extracted-digest.json");
    let pkg = run_extract_source(&src_dir, &out_sysml, &out_digest, false)
        .expect("run_extract_source");
    assert!(count_structural_elements(&pkg) > 0);
    let sysml_text = fs::read_to_string(&out_sysml).expect("read extracted.sysml");
    let reparsed = parse_sysml(&sysml_text).expect("reparse extracted.sysml");
    assert!(count_structural_elements(&reparsed) > 0);
    assert!(out_digest.exists());

    // Verify CLI --inject-safety-docstrings and --extract-source
    let bin = env!("CARGO_BIN_EXE_compile-sysml");
    let status = Command::new(bin)
        .arg("--inject-safety-docstrings")
        .arg(&src_dir)
        .status()
        .expect("invoke compile-sysml --inject-safety-docstrings");
    assert!(status.success());

    // Verify exit code 1 on missing path
    let missing = dir.join("does_not_exist");
    let err = run_docstring_injection(&DocstringInjectionOptions {
        target_path: missing,
        dry_run: false,
        force: false,
        schema_model: None,
    })
    .expect_err("missing path should fail");
    assert_eq!(err.0, DocstringExitCode::Error);
    assert_eq!(err.0.code(), 1);

    // Verify Dart direct helper
    let dart_src = "class SensorNode {\n  final int id;\n  SensorNode(this.id);\n}\n";
    let (dart_updated, dart_count) =
        inject_dart_file_docstrings(dart_src, "lib/sensor_node.dart", None, false)
            .expect("inject dart docstrings");
    assert_eq!(dart_count, 2);
    assert!(dart_updated.contains("/// ## 1. Safety Intent & Regulatory Scope"));

    fs::remove_dir_all(&dir).expect("cleanup temp dir");
}
