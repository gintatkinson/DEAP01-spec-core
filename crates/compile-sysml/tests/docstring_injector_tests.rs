//! Integration test suite for `compile-sysml --inject-safety-docstrings` and `--extract-source`.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Verifies tiered Rust/Dart docstring synthesis, Tier B/C legacy 7-section compaction, idempotency,
//! `modular_codegen/generated` skip isolation, and CLI exit codes (`0`, `1`, `2`) under DO-178C Level A.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Second-pass docstring injection modifies `0` files and `0` items with identical SHA-256 digests.
//! - Duplicate function docstrings in a single file are disambiguated automatically.
//!
//! ## 3. Preconditions
//! - Host OS temporary directory is writable and `CARGO_BIN_EXE_compile-sysml` is available.
//!
//! ## 4. Postconditions
//! - Temporary test directories are removed deterministically prior to test return.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ over fixture source files; Space Complexity: $O(N)$.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Propagates fixture setup, parse, and subprocess errors via `Result<(), String>`.
//!
//! ## 7. Realization & Traceability
//! Safety & Traceability: Realises: [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS], [REQ-DEFECT-09-DOCSTRING-CLI-EXIT-CODES]
//! Contract: Verifies tiered Rust/Dart docstring synthesis, Tier B/C compaction, idempotency, and CLI exit codes.
//! Bounds & Faults: All filesystem fixtures allocate isolated directories under `std::env::temp_dir()`.

use compile_sysml::docstring::{
    inject_dart_file_docstrings, inject_rust_file_docstrings, run_docstring_injection,
    run_extract_source, DocstringExitCode, DocstringInjectionOptions,
};
use compile_sysml::{count_structural_elements, generate_digest, parse_sysml};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Allocates and creates a collision-free temporary directory under `std::env::temp_dir()`.
/// Realises: [REQ-FEAT-42-UNIQUE-TEST-DIR]
pub(crate) fn unique_test_dir(prefix: &str) -> Result<PathBuf, String> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!("{}_{}_{}", prefix, std::process::id(), nanos));
    fs::create_dir_all(&path)
        .map_err(|e| format!("Failed to create unique temp dir '{}': {}", path.display(), e))?;
    Ok(path)
}

/// Seeds the Rust and Dart source files for AC-03 and AC-04 docstring synthesis testing.
/// Realises: [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS]
fn write_ac03_ac04_fixtures(src_dir: &Path) -> Result<(PathBuf, PathBuf), String> {
    let rust_file = src_dir.join("telemetry.rs");
    let initial_rust = r#"//! Telemetry packet module.

/// Member documentation.
#[derive(Debug, Clone, PartialEq)]
pub struct TelemetryPacket {
    pub sequence_id: u64,
    pub timestamp_ms: f64,
}

pub enum TelemetryStatus { Nominal, Degraded, Faulted }

pub trait PacketValidator {
    fn validate_packet(&self) -> Result<bool, String>;
}

impl TelemetryPacket {
    pub fn new(sequence_id: u64, timestamp_ms: f64) -> Result<Self, String> {
        assert!(timestamp_ms >= 0.0);
        Ok(Self { sequence_id, timestamp_ms })
    }
    pub fn sequence_id(&self) -> u64 { self.sequence_id }
}
"#;
    fs::write(&rust_file, initial_rust).map_err(|e| format!("write telemetry.rs: {e}"))?;

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
enum FlightMode { manual, autonomous }
mixin TelemetryLoggingMixin { void recordSample(int sequenceId) {} }
extension FlightModeExtension on FlightMode {
  bool get isAutonomous => this == FlightMode.autonomous;
}
typedef CommandCallback = void Function(double thrustNewtons);
"#;
    fs::write(&dart_file, initial_dart).map_err(|e| format!("write guidance_controller.dart: {e}"))?;
    Ok((rust_file, dart_file))
}

/// Verifies first-pass Rust and Dart enriched outputs for AC-03 and AC-04.
/// Realises: [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS]
fn assert_ac03_ac04_first_pass(rust_after_1: &str, dart_after_1: &str) -> Result<(), String> {
    assert!(!rust_after_1.contains("Member documentation."));
    assert!(!dart_after_1.contains("Member documentation."));
    for content in [rust_after_1, dart_after_1] {
        for heading in [
            "/// ## 1. Safety Intent & Regulatory Scope",
            "/// ## 2. Invariants & Mathematical Properties",
            "/// ## 3. Preconditions",
            "/// ## 4. Postconditions",
            "/// ## 5. Algorithmic Complexity Bounds",
            "/// ## 6. Failure Modes & Fault Tolerance",
            "/// ## 7. Realization & Traceability",
            "/// Realises: [REQ-",
        ] {
            assert!(content.contains(heading), "missing {heading}");
        }
    }
    assert!(rust_after_1.contains("NonNegativeTimestamp"));
    let derive_pos = rust_after_1
        .find("#[derive(Debug, Clone, PartialEq)]")
        .ok_or_else(|| "Missing #[derive(Debug, Clone, PartialEq)]".to_string())?;
    let struct_pos = rust_after_1
        .find("pub struct TelemetryPacket")
        .ok_or_else(|| "Missing pub struct TelemetryPacket".to_string())?;
    let before_derive = rust_after_1.get(..derive_pos).unwrap_or("");
    let section_7_pos = before_derive
        .rfind("/// ## 7. Realization & Traceability")
        .ok_or_else(|| "Missing Section 7 before #[derive]".to_string())?;
    assert!(section_7_pos < derive_pos && derive_pos < struct_pos);
    Ok(())
}

#[test]
fn test_ac03_and_ac04_rust_and_dart_docstring_synthesis_and_idempotency() -> Result<(), String> {
    let dir = unique_test_dir("feat42_docstring_synth")?;
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
    let schema_model = parse_sysml(schema_sysml).map_err(|e| format!("parse schema: {e}"))?;
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("create src dir: {e}"))?;
    let (rust_file, dart_file) = write_ac03_ac04_fixtures(&src_dir)?;

    let opts = DocstringInjectionOptions {
        target_path: src_dir,
        dry_run: false,
        force: false,
        schema_model: Some(schema_model.clone()),
    };
    let report_1 = run_docstring_injection(&opts).map_err(|(c, m)| format!("run 1 ({c:?}): {m}"))?;
    assert_eq!(report_1.files_scanned, 2);
    assert_eq!(report_1.files_modified, 2);
    assert!(report_1.items_enriched >= 10);

    let rust_after_1 = fs::read_to_string(&rust_file).map_err(|e| e.to_string())?;
    let dart_after_1 = fs::read_to_string(&dart_file).map_err(|e| e.to_string())?;
    assert_ac03_ac04_first_pass(&rust_after_1, &dart_after_1)?;

    let rust_digest_1 = generate_digest(&schema_model, &rust_after_1).sha256;
    let dart_digest_1 = generate_digest(&schema_model, &dart_after_1).sha256;

    let report_2 = run_docstring_injection(&opts).map_err(|(c, m)| format!("run 2 ({c:?}): {m}"))?;
    assert_eq!(report_2.files_scanned, 2);
    assert_eq!(report_2.files_modified, 0);
    assert_eq!(report_2.items_enriched, 0);

    let rust_after_2 = fs::read_to_string(&rust_file).map_err(|e| e.to_string())?;
    let dart_after_2 = fs::read_to_string(&dart_file).map_err(|e| e.to_string())?;
    assert_eq!(rust_after_1, rust_after_2);
    assert_eq!(dart_after_1, dart_after_2);
    assert_eq!(rust_digest_1, generate_digest(&schema_model, &rust_after_2).sha256);
    assert_eq!(dart_digest_1, generate_digest(&schema_model, &dart_after_2).sha256);

    fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(())
}

#[test]
fn test_duplicate_function_docstrings_are_disambiguated() -> Result<(), String> {
    let shared_doc = "/// Shared docstring.\n/// ## 1. Safety Intent & Regulatory Scope\n/// Shared intent.\n/// ## 2. Invariants & Mathematical Properties\n/// Shared invariant.\n/// ## 3. Preconditions\n/// Shared precondition.\n/// ## 4. Postconditions\n/// Shared postcondition.\n/// ## 5. Algorithmic Complexity Bounds\n/// O(1).\n/// ## 6. Failure Modes & Fault Tolerance\n/// None.\n/// ## 7. Realization & Traceability\n/// Realises: [REQ-SHARED-001]";
    let duplicate_rust = format!(
        "{shared_doc}\npub fn first_op(alpha: i32) -> i32 {{\n    alpha + 1\n}}\n\n{shared_doc}\npub fn second_op(beta: i32) -> i32 {{\n    beta + 2\n}}\n"
    );

    let (updated, enriched) =
        inject_rust_file_docstrings(&duplicate_rust, "crates/demo/src/ops.rs", None, false)?;
    assert_eq!(enriched, 1);
    assert!(updated.contains("second_op"));

    let (updated_again, enriched_again) =
        inject_rust_file_docstrings(&updated, "crates/demo/src/ops.rs", None, false)?;
    assert_eq!(enriched_again, 0);
    assert_eq!(updated, updated_again);
    Ok(())
}

#[test]
fn test_extract_source_and_cli_flags() -> Result<(), String> {
    let dir = unique_test_dir("feat42_extract_source")?;
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
    fs::write(
        src_dir.join("controller.rs"),
        "pub struct AttitudeController { pub pitch_deg: f64, pub roll_deg: f64 }\nimpl AttitudeController {\n    pub fn stabilize(&self, target_pitch: f64) -> f64 {\n        assert!(target_pitch.is_finite());\n        target_pitch - self.pitch_deg\n    }\n}\n",
    )
    .map_err(|e| e.to_string())?;

    let out_sysml = dir.join("extracted.sysml");
    let out_digest = dir.join("extracted-digest.json");
    let pkg = run_extract_source(&src_dir, &out_sysml, &out_digest, false)
        .map_err(|(c, m)| format!("extract failed ({c:?}): {m}"))?;
    assert!(count_structural_elements(&pkg) > 0);
    let sysml_text = fs::read_to_string(&out_sysml).map_err(|e| e.to_string())?;
    let reparsed = parse_sysml(&sysml_text).map_err(|e| e.to_string())?;
    assert!(count_structural_elements(&reparsed) > 0);
    assert!(out_digest.exists());

    let bin = env!("CARGO_BIN_EXE_compile-sysml");
    let status = Command::new(bin)
        .arg("--inject-safety-docstrings")
        .arg(&src_dir)
        .status()
        .map_err(|e| e.to_string())?;
    assert!(status.success());

    let err = run_docstring_injection(&DocstringInjectionOptions {
        target_path: dir.join("does_not_exist"),
        dry_run: false,
        force: false,
        schema_model: None,
    })
    .err()
    .ok_or_else(|| "Expected missing path to fail".to_string())?;
    assert_eq!(err.0, DocstringExitCode::Error);
    assert_eq!(err.0.code(), 1);

    let dart_src = "class SensorNode {\n  final int id;\n  SensorNode(this.id);\n}\n";
    let (dart_updated, dart_count) =
        inject_dart_file_docstrings(dart_src, "lib/sensor_node.dart", None, false)?;
    assert_eq!(dart_count, 2);
    assert!(dart_updated.contains("/// ## 1. Safety Intent & Regulatory Scope"));

    fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(())
}

#[test]
fn test_injector_handles_multiline_dart_classes_string_slashes_interpolated_utf8_quotes_and_dart3_items(
) -> Result<(), String> {
    let dart_source = r#"// Leading top comment
import 'dart:async';

class PrimaryService {
  final String endpoint = "https://api.example.com/v1//endpoint";
  final String marker = '•';
  final String quoted = "“nested quotes”";
  PrimaryService();
  factory PrimaryService.fromConfig(Map<String, dynamic> config) { return PrimaryService(); }
  String get serviceName => "Primary";
  @override
  Future<void> execute() async {
    final log = "Executing: ${serviceName.toLowerCase()} with id: ${1 + 1}";
  }
}

class SecondaryService {
  final int code = 42;
  SecondaryService(this.code);
  @visibleForTesting
  @override
  int get calculatedCode => code * 2;
  void reset() { final status = 'reset://done'; }
}
"#;
    let (injected, count) =
        inject_dart_file_docstrings(dart_source, "lib/services.dart", None, false)?;
    assert!(count >= 8, "Expected at least 8 documented items, got {count}");
    for needle in [
        "PrimaryService",
        "PrimaryService.fromConfig",
        "serviceName",
        "SecondaryService",
        "calculatedCode",
    ] {
        assert!(injected.contains(needle));
    }

    let (second_injected, second_count) =
        inject_dart_file_docstrings(&injected, "lib/services.dart", None, false)?;
    assert_eq!(second_count, 0);
    assert_eq!(injected, second_injected);
    Ok(())
}

#[test]
fn test_cli_exit_code_1_on_broken_schema_and_exit_code_2_on_conflicting_unforced_docstring(
) -> Result<(), String> {
    crate::docstring_injector_b_tests::run_cli_exit_code_1_and_2_checks()
}

#[test]
fn test_issue_520_multi_req_constraint_aggregation_dart_details_and_rust_unbounded_items(
) -> Result<(), String> {
    crate::docstring_injector_b_tests::run_issue_520_aggregation_checks()
}

#[test]
fn test_tiered_v2_rust_synthesis_compaction_and_generated_skip() -> Result<(), String> {
    crate::docstring_injector_c_tests::run_tiered_v2_synthesis_and_compaction_checks()
}
