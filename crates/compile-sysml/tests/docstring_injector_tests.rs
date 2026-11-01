//! Integration test suite for `compile-sysml --inject-safety-docstrings` and `--extract-source`.
//!
//! Safety & Traceability: Realises: [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS], [REQ-DEFECT-09-DOCSTRING-CLI-EXIT-CODES]
//! Contract: Verifies tiered Rust/Dart docstring synthesis, Tier B/C legacy 7-section compaction, idempotency, `modular_codegen/generated` skip isolation, and CLI exit codes (`0`, `1`, `2`).
//! Bounds & Faults: All filesystem fixtures allocate isolated directories under `std::env::temp_dir()` and clean up deterministically without panicking.

use compile_sysml::docstring::{
    inject_dart_file_docstrings, inject_rust_file_docstrings, run_docstring_injection,
    run_extract_source, DocstringExitCode, DocstringInjectionOptions,
};
use compile_sysml::{count_structural_elements, generate_digest, parse_sysml};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Allocates and creates a collision-free temporary directory under `std::env::temp_dir()`.
/// Realises: [REQ-FEAT-42-UNIQUE-TEST-DIR]
fn unique_test_dir(prefix: &str) -> Result<PathBuf, String> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!("{}_{}_{}", prefix, std::process::id(), nanos));
    fs::create_dir_all(&path).map_err(|e| {
        format!(
            "Failed to create unique temp dir '{}': {}",
            path.display(),
            e
        )
    })?;
    Ok(path)
}

/// Verifies AC-03 and AC-04 Rust and Dart docstring synthesis and second-pass idempotency.
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
    let schema_model =
        parse_sysml(schema_sysml).map_err(|e| format!("Failed to parse schema_sysml: {}", e))?;

    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("Failed to create src dir: {}", e))?;

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
    fs::write(&rust_file, initial_rust)
        .map_err(|e| format!("Failed to write telemetry.rs: {}", e))?;

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
    fs::write(&dart_file, initial_dart)
        .map_err(|e| format!("Failed to write guidance_controller.dart: {}", e))?;

    let opts = DocstringInjectionOptions {
        target_path: src_dir.clone(),
        dry_run: false,
        force: false,
        schema_model: Some(schema_model.clone()),
    };
    let report_1 = run_docstring_injection(&opts)
        .map_err(|(code, msg)| format!("First run failed ({:?}): {}", code, msg))?;
    assert_eq!(report_1.files_scanned, 2);
    assert_eq!(report_1.files_modified, 2);
    assert!(
        report_1.items_enriched >= 10,
        "Expected at least 10 items enriched across Rust and Dart, got {}",
        report_1.items_enriched
    );

    let rust_after_1 =
        fs::read_to_string(&rust_file).map_err(|e| format!("Read rust after run 1: {}", e))?;
    let dart_after_1 =
        fs::read_to_string(&dart_file).map_err(|e| format!("Read dart after run 1: {}", e))?;

    assert!(!rust_after_1.contains("Member documentation."));
    assert!(!dart_after_1.contains("Member documentation."));

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

    let rust_digest_1 = generate_digest(&schema_model, &rust_after_1).sha256;
    let dart_digest_1 = generate_digest(&schema_model, &dart_after_1).sha256;

    let report_2 = run_docstring_injection(&opts)
        .map_err(|(code, msg)| format!("Second run failed ({:?}): {}", code, msg))?;
    assert_eq!(report_2.files_scanned, 2);
    assert_eq!(
        report_2.files_modified, 0,
        "Second run must modify 0 files for idempotency"
    );
    assert_eq!(report_2.items_enriched, 0);

    let rust_after_2 =
        fs::read_to_string(&rust_file).map_err(|e| format!("Read rust after run 2: {}", e))?;
    let dart_after_2 =
        fs::read_to_string(&dart_file).map_err(|e| format!("Read dart after run 2: {}", e))?;
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

    fs::remove_dir_all(&dir).map_err(|e| format!("Failed to clean up temp dir: {}", e))?;
    Ok(())
}

/// Verifies that duplicate Tier A function docstrings within the same file are disambiguated automatically.
#[test]
fn test_duplicate_function_docstrings_are_disambiguated() -> Result<(), String> {
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
        inject_rust_file_docstrings(duplicate_rust, "crates/demo/src/ops.rs", None, false)?;
    assert_eq!(
        enriched, 1,
        "First compliant function docstring should be kept and second duplicate should be re-synthesized"
    );
    assert!(updated.contains("second_op"));

    let (updated_again, enriched_again) =
        inject_rust_file_docstrings(&updated, "crates/demo/src/ops.rs", None, false)?;
    assert_eq!(enriched_again, 0);
    assert_eq!(updated, updated_again);
    Ok(())
}

/// Verifies `--extract-source` AST extraction and `--inject-safety-docstrings` CLI execution.
#[test]
fn test_extract_source_and_cli_flags() -> Result<(), String> {
    let dir = unique_test_dir("feat42_extract_source")?;
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("Failed to create src: {}", e))?;

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
    .map_err(|e| format!("Failed to write controller.rs: {}", e))?;

    let out_sysml = dir.join("extracted.sysml");
    let out_digest = dir.join("extracted-digest.json");
    let pkg = run_extract_source(&src_dir, &out_sysml, &out_digest, false)
        .map_err(|(code, msg)| format!("run_extract_source failed ({:?}): {}", code, msg))?;
    assert!(count_structural_elements(&pkg) > 0);
    let sysml_text =
        fs::read_to_string(&out_sysml).map_err(|e| format!("Read extracted.sysml: {}", e))?;
    let reparsed =
        parse_sysml(&sysml_text).map_err(|e| format!("Reparse extracted.sysml: {}", e))?;
    assert!(count_structural_elements(&reparsed) > 0);
    assert!(out_digest.exists());

    let bin = env!("CARGO_BIN_EXE_compile-sysml");
    let status = Command::new(bin)
        .arg("--inject-safety-docstrings")
        .arg(&src_dir)
        .status()
        .map_err(|e| format!("Failed to invoke compile-sysml: {}", e))?;
    assert!(status.success());

    let missing = dir.join("does_not_exist");
    let err = match run_docstring_injection(&DocstringInjectionOptions {
        target_path: missing,
        dry_run: false,
        force: false,
        schema_model: None,
    }) {
        Ok(_) => return Err("Expected missing path to fail".to_string()),
        Err(e) => e,
    };
    assert_eq!(err.0, DocstringExitCode::Error);
    assert_eq!(err.0.code(), 1);

    let dart_src = "class SensorNode {\n  final int id;\n  SensorNode(this.id);\n}\n";
    let (dart_updated, dart_count) =
        inject_dart_file_docstrings(dart_src, "lib/sensor_node.dart", None, false)?;
    assert_eq!(dart_count, 2);
    assert!(dart_updated.contains("/// ## 1. Safety Intent & Regulatory Scope"));

    fs::remove_dir_all(&dir).map_err(|e| format!("Failed to cleanup temp dir: {}", e))?;
    Ok(())
}

/// Verifies Dart docstring injection across multiline classes, URL slashes in strings, UTF-8 quotes, and Dart 3 items.
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

  factory PrimaryService.fromConfig(Map<String, dynamic> config) {
    return PrimaryService();
  }

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

  void reset() {
    final status = 'reset://done';
  }
}
"#;

    let (injected, count) =
        inject_dart_file_docstrings(dart_source, "lib/services.dart", None, false)?;

    assert!(
        count >= 8,
        "Expected at least 8 documented items, got {}",
        count
    );
    assert!(injected.contains("PrimaryService"));
    assert!(injected.contains("PrimaryService.fromConfig"));
    assert!(injected.contains("serviceName"));
    assert!(injected.contains("SecondaryService"));
    assert!(injected.contains("calculatedCode"));

    let (second_injected, second_count) =
        inject_dart_file_docstrings(&injected, "lib/services.dart", None, false)?;

    assert_eq!(second_count, 0, "Idempotent run must have count 0");
    assert_eq!(
        injected, second_injected,
        "Idempotent injection must match byte-for-byte"
    );
    Ok(())
}

/// Verifies CLI exit code `1` on broken `--schema` and exit code `2` on conflicting unforced docstring.
#[test]
fn test_cli_exit_code_1_on_broken_schema_and_exit_code_2_on_conflicting_unforced_docstring(
) -> Result<(), String> {
    let dir = unique_test_dir("feat42_cli_exit_codes")?;
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("Failed to create src dir: {}", e))?;

    let broken_schema = dir.join("broken.sysml");
    fs::write(&broken_schema, "package Broken { part def Unclosed ")
        .map_err(|e| format!("Failed to write broken.sysml: {}", e))?;

    let valid_schema = dir.join("valid.sysml");
    let valid_sysml = r#"
package FlightControlPkg {
    requirement def AutopilotAuthorityReq {
        id = "REQ-SCHEMA-001";
        doc /* Autopilot authority requirement */
    }
    part def AutopilotController {
        satisfy requirement AutopilotAuthorityReq;
    }
}
"#;
    fs::write(&valid_schema, valid_sysml)
        .map_err(|e| format!("Failed to write valid.sysml: {}", e))?;

    let rust_file = src_dir.join("autopilot.rs");
    let conflicting_rust = r#"/// Autopilot controller struct.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A safety objectives for struct `AutopilotController`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic control authority invariants.
///
/// ## 3. Preconditions
/// - Caller supplies initialized gain parameters.
///
/// ## 4. Postconditions
/// - Defines the authoritative `AutopilotController` state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1).
/// - Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates inputs.
/// - State: Preserves state.
/// - Action: Deterministic.
/// - Resource: Bounded stack.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-WRONG-999] | SysML v2: PartDef(AutopilotController)
pub struct AutopilotController {
    pub enabled: bool,
}
"#;
    fs::write(&rust_file, conflicting_rust)
        .map_err(|e| format!("Failed to write autopilot.rs: {}", e))?;

    let bin = env!("CARGO_BIN_EXE_compile-sysml");

    // 1a. Nonexistent explicit --schema must exit with code 1
    let nonexistent_schema = dir.join("nonexistent.sysml");
    let out_nonexistent = Command::new(bin)
        .arg("--inject-safety-docstrings")
        .arg(&src_dir)
        .arg("--schema")
        .arg(&nonexistent_schema)
        .output()
        .map_err(|e| format!("Failed to run compile-sysml with nonexistent schema: {}", e))?;
    assert_eq!(
        out_nonexistent.status.code(),
        Some(1),
        "Expected exit code 1 for nonexistent --schema, stderr: {}",
        String::from_utf8_lossy(&out_nonexistent.stderr)
    );

    // 1b. Broken/unparseable explicit --schema must exit with code 1
    let out_broken = Command::new(bin)
        .arg("--inject-safety-docstrings")
        .arg(&src_dir)
        .arg("--schema")
        .arg(&broken_schema)
        .output()
        .map_err(|e| format!("Failed to run compile-sysml with broken schema: {}", e))?;
    assert_eq!(
        out_broken.status.code(),
        Some(1),
        "Expected exit code 1 for broken --schema, stderr: {}",
        String::from_utf8_lossy(&out_broken.stderr)
    );

    // 2. Conflicting unforced docstring against valid --schema must exit with code 2
    let out_conflict = Command::new(bin)
        .arg("--inject-safety-docstrings")
        .arg(&src_dir)
        .arg("--schema")
        .arg(&valid_schema)
        .output()
        .map_err(|e| {
            format!(
                "Failed to run compile-sysml on conflicting docstring: {}",
                e
            )
        })?;
    assert_eq!(
        out_conflict.status.code(),
        Some(2),
        "Expected exit code 2 for conflicting unforced docstring, stderr: {}",
        String::from_utf8_lossy(&out_conflict.stderr)
    );

    // 3. Adding --force must overwrite REQ-WRONG-999 with REQ-SCHEMA-001 and exit with code 0
    let out_forced = Command::new(bin)
        .arg("--inject-safety-docstrings")
        .arg(&src_dir)
        .arg("--schema")
        .arg(&valid_schema)
        .arg("--force")
        .output()
        .map_err(|e| format!("Failed to run compile-sysml with --force: {}", e))?;
    assert_eq!(
        out_forced.status.code(),
        Some(0),
        "Expected exit code 0 when --force is passed, stderr: {}",
        String::from_utf8_lossy(&out_forced.stderr)
    );

    let updated_rust = fs::read_to_string(&rust_file)
        .map_err(|e| format!("Failed to read autopilot.rs: {}", e))?;
    assert!(
        updated_rust.contains("[REQ-SCHEMA-001]"),
        "Expected forced docstring to contain [REQ-SCHEMA-001], got:\n{}",
        updated_rust
    );
    assert!(
        !updated_rust.contains("REQ-WRONG-999"),
        "Expected forced docstring to overwrite REQ-WRONG-999, got:\n{}",
        updated_rust
    );

    fs::remove_dir_all(&dir).map_err(|e| format!("Failed to cleanup temp dir: {}", e))?;
    Ok(())
}

/// Verifies multi-requirement/constraint schema aggregation, Dart AST detail inspection, and unbounded Rust item detail extraction (Issue #520).
#[test]
fn test_issue_520_multi_req_constraint_aggregation_dart_details_and_rust_unbounded_items(
) -> Result<(), String> {
    // Sub-Defect A: Multi-requirement and multi-constraint aggregation + partial Realises conflict detection
    let schema_sysml = r#"
package CoreVerificationPkg {
    requirement def ReqAlphaOne {
        id = "REQ-ALPHA-001";
        doc /* First alpha requirement */
    }
    requirement def ReqAlphaTwo {
        id = "REQ-ALPHA-002";
        doc /* Second alpha requirement */
    }
    requirement def ReqActOne {
        id = "REQ-ACT-001";
        doc /* First action requirement */
    }
    requirement def ReqActTwo {
        id = "REQ-ACT-002";
        doc /* Second action requirement */
    }
    part def AlphaRecord {
        satisfy requirement ReqAlphaOne;
        satisfy requirement ReqAlphaTwo;
        constraint def BoundAlpha {
            val_a >= 0
        }
        constraint def BoundBeta {
            val_b <= 100
        }
        action def evaluate_record {
            doc /* Realises: [REQ-ACT-001], [REQ-ACT-002] */
            in val_a: Integer;
            in val_b: Integer;
        }
        constraint def invariant_evaluate_record_0 {
            val_a > 0
        }
        constraint def invariant_evaluate_record_1 {
            val_b < 100
        }
    }
}
"#;
    let schema_model =
        parse_sysml(schema_sysml).map_err(|e| format!("Failed to parse schema_sysml: {e}"))?;

    let rust_alpha_src = r#"pub struct AlphaRecord {
    pub val_a: i64,
    pub val_b: i64,
}

impl AlphaRecord {
    pub fn evaluate_record(&self, val_a: i64, val_b: i64) -> bool {
        val_a > 0 && val_b < 100
    }
}
"#;
    let (rust_alpha_enriched, alpha_count) = inject_rust_file_docstrings(
        rust_alpha_src,
        "crates/core_module/src/alpha.rs",
        Some(&schema_model),
        false,
    )
    .map_err(|e| e.into_message())?;
    assert_eq!(alpha_count, 2);
    assert!(
        rust_alpha_enriched.contains("Realises: [REQ-ALPHA-001], [REQ-ALPHA-002] | SysML v2: PartDef(AlphaRecord)"),
        "Expected AlphaRecord to aggregate both satisfied requirements, got:\n{rust_alpha_enriched}"
    );
    assert!(
        rust_alpha_enriched.contains("BoundAlpha (val_a >= 0); BoundBeta (val_b <= 100)"),
        "Expected AlphaRecord to aggregate both part constraints, got:\n{rust_alpha_enriched}"
    );
    assert!(
        rust_alpha_enriched.contains("Realises: [REQ-ACT-001], [REQ-ACT-002] | SysML v2: ActionDef(AlphaRecord::evaluate_record)"),
        "Expected evaluate_record to aggregate both action doc requirements with tier precedence over parent part, got:\n{rust_alpha_enriched}"
    );
    assert!(
        rust_alpha_enriched.contains("invariant_evaluate_record_0 (val_a > 0); invariant_evaluate_record_1 (val_b < 100)"),
        "Expected evaluate_record to aggregate both action-matched constraints, got:\n{rust_alpha_enriched}"
    );

    // Partial Realises in unforced mode must be flagged as a conflict because REQ-ALPHA-002 is missing
    let partial_rust = rust_alpha_enriched.replace(
        "Realises: [REQ-ALPHA-001], [REQ-ALPHA-002]",
        "Realises: [REQ-ALPHA-001]",
    );
    let partial_res = inject_rust_file_docstrings(
        &partial_rust,
        "crates/core_module/src/alpha.rs",
        Some(&schema_model),
        false,
    );
    if partial_res.is_ok() {
        return Err(
            "Expected unforced docstring missing second schema requirement REQ-ALPHA-002 to return UnforcedConflict"
                .to_string(),
        );
    }

    // Sub-Defect B: Dart fields, enum variants, and callable assert(...) invariants
    let dart_src = r#"class BetaController {
  final double gainAlpha;
  late int stepCount = 0;

  BetaController(this.gainAlpha);

  double computeStep(double inputVal) {
    assert(inputVal >= 0.0);
    assert(stepCount >= 0, "non-negative step count");
    return inputVal * gainAlpha;
  }
}

enum GammaState {
  idle,
  active,
  completed,
  faulted,
  suspended,
  archived,
  terminated,
}
"#;
    let (dart_enriched, dart_count) =
        inject_dart_file_docstrings(dart_src, "lib/beta_controller.dart", None, false)
            .map_err(|e| e.into_message())?;
    assert!(dart_count >= 4);
    assert!(
        dart_enriched.contains("gainAlpha: double, stepCount: int"),
        "Expected BetaController docstring to interpolate Dart class fields, got:\n{dart_enriched}"
    );
    assert!(
        dart_enriched.contains("inputVal >= 0.0 && stepCount >= 0"),
        "Expected computeStep docstring to interpolate Dart assert(...) conditions, got:\n{dart_enriched}"
    );
    assert!(
        dart_enriched.contains("idle, active, completed, faulted, suspended, archived, terminated"),
        "Expected GammaState docstring to interpolate all 7 Dart enum variants, got:\n{dart_enriched}"
    );
    let (dart_second, dart_second_count) =
        inject_dart_file_docstrings(&dart_enriched, "lib/beta_controller.dart", None, false)
            .map_err(|e| e.into_message())?;
    assert_eq!(dart_second_count, 0);
    assert_eq!(dart_enriched, dart_second);

    // Sub-Defect C: Rust inspect_rust_item_details without arbitrary 3/5/6 element caps
    let wide_rust_src = r#"pub struct WideRecord {
    pub field_1: u32,
    pub field_2: u32,
    pub field_3: u32,
    pub field_4: u32,
    pub field_5: u32,
    pub field_6: u32,
    pub field_7: u32,
}

pub enum WideState {
    State1,
    State2,
    State3,
    State4,
    State5,
    State6,
    State7,
    State8,
}

pub trait WideStageTrait {
    fn stage_1(&self) -> u32;
    fn stage_2(&self) -> u32;
    fn stage_3(&self) -> u32;
    fn stage_4(&self) -> u32;
    fn stage_5(&self) -> u32;
    fn stage_6(&self) -> u32;
    fn stage_7(&self) -> u32;
    fn stage_8(&self) -> u32;
}

pub fn check_five_invariants(val: i32) -> i32 {
    assert!(val > 1);
    assert!(val > 2);
    assert!(val > 3);
    assert!(val > 4);
    assert!(val > 5);
    val
}
"#;
    let (wide_enriched, wide_count) =
        inject_rust_file_docstrings(wide_rust_src, "crates/core_module/src/wide.rs", None, false)
            .map_err(|e| e.into_message())?;
    assert!(wide_count >= 4);
    assert!(
        wide_enriched.contains("field_6: u32, field_7: u32"),
        "Expected WideRecord docstring to include fields beyond 5, got:\n{wide_enriched}"
    );
    assert!(
        wide_enriched.contains("State7, State8"),
        "Expected WideState docstring to include variants beyond 6, got:\n{wide_enriched}"
    );
    assert!(
        wide_enriched.contains("fn stage_7, fn stage_8"),
        "Expected WideStageTrait docstring to include trait methods beyond 6, got:\n{wide_enriched}"
    );
    assert!(
        wide_enriched.contains("val > 4 && val > 5"),
        "Expected check_five_invariants docstring to include assert! invariants beyond 3, got:\n{wide_enriched}"
    );

    Ok(())
}

/// Extracts the contiguous `///` docblock lines immediately preceding `decl_needle` (skipping intervening `#[...]` attributes).
/// Realises: [REQ-FEAT-42-TEST-DOCBLOCK-EXTRACTOR]
fn extract_docblock_before_decl<'a>(
    source: &'a str,
    decl_needle: &str,
) -> Result<Vec<&'a str>, String> {
    let lines: Vec<&'a str> = source.lines().collect();
    let decl_idx = lines
        .iter()
        .position(|line| line.trim().starts_with(decl_needle))
        .ok_or_else(|| format!("Declaration needle '{decl_needle}' not found in:\n{source}"))?;

    let mut cursor = decl_idx;
    while cursor > 0 {
        let prev = lines
            .get(cursor.saturating_sub(1))
            .copied()
            .unwrap_or("")
            .trim();
        if prev.starts_with("#[") {
            cursor = cursor.saturating_sub(1);
        } else {
            break;
        }
    }

    let mut doc_lines = Vec::new();
    while cursor > 0 {
        let prev = lines
            .get(cursor.saturating_sub(1))
            .copied()
            .unwrap_or("")
            .trim();
        if prev.starts_with("///") {
            doc_lines.push(prev);
            cursor = cursor.saturating_sub(1);
        } else {
            break;
        }
    }
    doc_lines.reverse();
    Ok(doc_lines)
}

/// Verifies Tier A 7-section synthesis, Tier B/C concise 2-line synthesis, legacy Tier B/C 7-section compaction, idempotence, and `modular_codegen/generated` skip isolation.
#[test]
fn test_tiered_v2_rust_synthesis_compaction_and_generated_skip() -> Result<(), String> {
    // 1. Undocumented Tier A vs Tier B/C synthesis + #[test] immunity
    let undocumented_tiered_src = r#"pub struct RecordFrame {
    pub slot_id: u32,
}

pub enum StageMode {
    Idle,
    Active,
}

pub trait FrameCodec {
    fn encode_slot(&self) -> u32;
}

impl RecordFrame {
    pub fn new(slot_id: u32) -> Self {
        Self { slot_id }
    }

    pub(crate) fn internal_slot(&self) -> u32 {
        self.slot_id
    }

    fn private_slot(&self) -> u32 {
        self.slot_id
    }
}

impl FrameCodec for RecordFrame {
    fn encode_slot(&self) -> u32 {
        self.slot_id
    }
}

impl Default for RecordFrame {
    fn default() -> Self {
        Self { slot_id: 0 }
    }
}

struct PrivateScratch {
    offset: usize,
}

enum PrivateCursor {
    Head,
    Tail,
}

const MAX_SLOT_WINDOW: usize = 32;

type SlotWindowIndex = usize;

pub fn exported_transform(val: u32) -> u32 {
    val.saturating_add(1)
}

pub(crate) fn crate_transform(val: u32) -> u32 {
    val.saturating_add(2)
}

pub(super) fn super_transform(val: u32) -> u32 {
    val.saturating_add(3)
}

pub(in crate) fn path_transform(val: u32) -> u32 {
    val.saturating_add(4)
}

fn private_transform(val: u32) -> u32 {
    val.saturating_add(5)
}

#[test]
fn standalone_unit_probe() {
    assert_eq!(1 + 1, 2);
}
"#;

    let (synthesized, synth_count) = inject_rust_file_docstrings(
        undocumented_tiered_src,
        "crates/core_module/src/frame_codec.rs",
        None,
        false,
    )
    .map_err(|e| e.into_message())?;

    assert!(
        synth_count >= 16,
        "Expected all undocumented Tier A and Tier B/C items (excluding #[test]) to be enriched, got {synth_count}"
    );

    // Tier A items must still receive full 7-section docstrings
    for tier_a_decl in [
        "pub struct RecordFrame",
        "pub enum StageMode",
        "pub trait FrameCodec",
        "pub fn new(slot_id: u32)",
        "pub fn exported_transform(val: u32)",
    ] {
        let doc = extract_docblock_before_decl(&synthesized, tier_a_decl)?;
        let joined = doc.join("\n");
        assert!(
            joined.contains("## 1. Safety Intent & Regulatory Scope")
                && joined.contains("## 7. Realization & Traceability")
                && joined.contains("Realises: [REQ-"),
            "Expected Tier A item `{tier_a_decl}` to carry full 7-section docstring, got:\n{joined}"
        );
    }

    // Tier B/C items must receive concise 2-line docstrings with zero 7-section headers
    for tier_bc_decl in [
        "fn encode_slot(&self) -> u32;",
        "pub(crate) fn internal_slot(&self)",
        "fn private_slot(&self)",
        "fn encode_slot(&self) -> u32 {",
        "fn default() -> Self",
        "struct PrivateScratch",
        "enum PrivateCursor",
        "const MAX_SLOT_WINDOW: usize = 32;",
        "type SlotWindowIndex = usize;",
        "pub(crate) fn crate_transform(val: u32)",
        "pub(super) fn super_transform(val: u32)",
        "pub(in crate) fn path_transform(val: u32)",
        "fn private_transform(val: u32)",
    ] {
        let doc = extract_docblock_before_decl(&synthesized, tier_bc_decl)?;
        assert_eq!(
            doc.len(),
            2,
            "Expected Tier B/C item `{tier_bc_decl}` to receive concise 2-line docstring, got {} lines:\n{}",
            doc.len(),
            doc.join("\n")
        );
        let joined = doc.join("\n");
        assert!(
            !joined.contains("## 1."),
            "Tier B/C item `{tier_bc_decl}` must not carry 7-section headers:\n{joined}"
        );
        assert!(
            doc.get(1).is_some_and(|line| line.starts_with("/// Realises: [REQ-")),
            "Tier B/C item `{tier_bc_decl}` line 2 must be `/// Realises: [REQ-...]`, got:\n{joined}"
        );
    }

    // #[test] function must NOT receive 28-line 7-section boilerplate
    let test_fn_doc = extract_docblock_before_decl(&synthesized, "fn standalone_unit_probe()")?;
    assert!(
        test_fn_doc.is_empty(),
        "Expected undocumented #[test] function to remain free of 7-section boilerplate, got:\n{}",
        test_fn_doc.join("\n")
    );

    // Second pass on synthesized output must be 100% idempotent
    let (synth_second, synth_second_count) = inject_rust_file_docstrings(
        &synthesized,
        "crates/core_module/src/frame_codec.rs",
        None,
        false,
    )
    .map_err(|e| e.into_message())?;
    assert_eq!(synth_second_count, 0);
    assert_eq!(synthesized, synth_second);

    // 2. Legacy 28-line 7-section boilerplate on Tier B/C items is compacted while Tier A is preserved
    let legacy_mixed_src = r#"/// Public signal router state.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to deterministic routing rules for `SignalRouter`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Maintains bounded channel index.
///
/// ## 3. Preconditions
/// - Initialized channel count.
///
/// ## 4. Postconditions
/// - Holds active routing table state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1).
/// - Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Typed fields.
/// - State: Immutable projection.
/// - Action: Non-blocking.
/// - Resource: Stack-allocated.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ROUTER-PUBLIC-00]
pub struct SignalRouter {
    pub channel_count: usize,
}

/// Clamps raw ratio within normalized unit interval.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A safety objectives for fn `clamp_ratio`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Output remains in `[0.0, 1.0]`.
///
/// ## 3. Preconditions
/// - `raw: f64` is finite.
///
/// ## 4. Postconditions
/// - Returns clamped `f64`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1).
/// - Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Pure scalar input.
/// - State: Stateless.
/// - Action: Total clamp.
/// - Resource: Zero heap allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ROUTER-CLAMP-01] | SysML v2: ActionDef(clamp_ratio)
fn clamp_ratio(raw: f64) -> f64 {
    raw.clamp(0.0, 1.0)
}

/// ## 1. Safety Intent & Regulatory Scope
/// Advances discrete step counter with saturating increment.
///
/// ## 2. Invariants & Mathematical Properties
/// - Monotonic non-overflowing step progression.
///
/// ## 3. Preconditions
/// - `tick: u64` is a valid counter.
///
/// ## 4. Postconditions
/// - Returns incremented tick.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1).
/// - Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Scalar `u64`.
/// - State: Pure function.
/// - Action: Saturates at `u64::MAX`.
/// - Resource: Zero allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ROUTER-STEP-02]
pub(crate) fn step_counter(tick: u64) -> u64 {
    tick.saturating_add(1)
}

impl Default for SignalRouter {
    /// Constructs a zero-channel default `SignalRouter`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A safety objectives for `SignalRouter::default`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Initializes `channel_count` to `0`.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns default `SignalRouter`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1).
    /// - Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible.
    ///
    /// ## 7. Realization & Traceability
    fn default() -> Self {
        Self { channel_count: 0 }
    }
}

/// Internal cursor tracking active router offset.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A safety objectives for struct `RouterCursor`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Tracks bounded `index`.
///
/// ## 3. Preconditions
/// - Valid index.
///
/// ## 4. Postconditions
/// - Stores cursor state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1).
/// - Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Stack value.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ROUTER-CURSOR-03]
struct RouterCursor {
    index: usize,
}
"#;

    let (compacted, compact_count) = inject_rust_file_docstrings(
        legacy_mixed_src,
        "crates/core_module/src/signal_router.rs",
        None,
        false,
    )
    .map_err(|e| e.into_message())?;

    assert_eq!(
        compact_count, 4,
        "Expected all 4 legacy Tier B/C items to be compacted while Tier A struct is preserved, got {compact_count} in:\n{compacted}"
    );
    assert_eq!(
        compacted
            .matches("## 1. Safety Intent & Regulatory Scope")
            .count(),
        1,
        "Only Tier A `pub struct SignalRouter` should retain 7-section headers:\n{compacted}"
    );

    let clamp_doc = extract_docblock_before_decl(&compacted, "fn clamp_ratio(raw: f64)")?;
    assert_eq!(
        clamp_doc,
        vec![
            "/// Clamps raw ratio within normalized unit interval.",
            "/// Realises: [REQ-ROUTER-CLAMP-01]",
        ]
    );

    let step_doc =
        extract_docblock_before_decl(&compacted, "pub(crate) fn step_counter(tick: u64)")?;
    assert_eq!(
        step_doc,
        vec![
            "/// Advances discrete step counter with saturating increment.",
            "/// Realises: [REQ-ROUTER-STEP-02]",
        ]
    );

    let default_doc = extract_docblock_before_decl(&compacted, "fn default() -> Self")?;
    assert_eq!(default_doc.len(), 2);
    assert_eq!(
        default_doc.first().copied(),
        Some("/// Constructs a zero-channel default `SignalRouter`.")
    );
    assert!(
        default_doc
            .get(1)
            .is_some_and(|line| line.starts_with("/// Realises: [REQ-")),
        "Missing fallback Realises tag on compacted Default::default:\n{}",
        default_doc.join("\n")
    );

    let cursor_doc = extract_docblock_before_decl(&compacted, "struct RouterCursor")?;
    assert_eq!(
        cursor_doc,
        vec![
            "/// Internal cursor tracking active router offset.",
            "/// Realises: [REQ-ROUTER-CURSOR-03]",
        ]
    );

    let (compacted_again, compact_again_count) = inject_rust_file_docstrings(
        &compacted,
        "crates/core_module/src/signal_router.rs",
        None,
        false,
    )
    .map_err(|e| e.into_message())?;
    assert_eq!(
        compact_again_count, 0,
        "Second pass after Tier B/C compaction must make 0 modifications"
    );
    assert_eq!(compacted, compacted_again);

    // 3. Files under `modular_codegen/generated/` must be skipped during compaction (both in-memory and directory scan)
    let generated_legacy_src = r#"/// Generated stage helper.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Preserves Stage-1 == Stage-2 metacircular SHA-256 parity.
///
/// ## 2. Invariants & Mathematical Properties
/// - Bit-for-bit stable generated docblock.
///
/// ## 3. Preconditions
/// - Valid input token.
///
/// ## 4. Postconditions
/// - Returns token index.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1).
/// - Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total projection.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-METACIRCULAR-STAGE-01]
fn generated_stage_helper(tok: u32) -> u32 {
    tok
}
"#;

    let (gen_out, gen_count) = inject_rust_file_docstrings(
        generated_legacy_src,
        "crates/compile-sysml/src/modular_codegen/generated/stage_emit.rs",
        None,
        false,
    )
    .map_err(|e| e.into_message())?;
    assert_eq!(
        gen_count, 0,
        "Files under modular_codegen/generated/ must be skipped from Tier B/C compaction"
    );
    assert_eq!(gen_out, generated_legacy_src);

    let dir = unique_test_dir("feat42_tiered_compaction_gen_skip")?;
    let src_dir = dir.join("src");
    let gen_dir = src_dir.join("modular_codegen").join("generated");
    fs::create_dir_all(&gen_dir).map_err(|e| format!("Failed to create gen_dir: {e}"))?;

    let regular_file = src_dir.join("signal_router.rs");
    let generated_file = gen_dir.join("stage_emit.rs");
    fs::write(&regular_file, legacy_mixed_src)
        .map_err(|e| format!("Failed to write signal_router.rs: {e}"))?;
    fs::write(&generated_file, generated_legacy_src)
        .map_err(|e| format!("Failed to write stage_emit.rs: {e}"))?;

    let report = run_docstring_injection(&DocstringInjectionOptions {
        target_path: src_dir,
        dry_run: false,
        force: false,
        schema_model: None,
    })
    .map_err(|(code, msg)| format!("Directory compaction failed ({code:?}): {msg}"))?;
    assert_eq!(report.files_scanned, 1);
    assert_eq!(report.files_modified, 1);

    let generated_after = fs::read_to_string(&generated_file)
        .map_err(|e| format!("Failed to read stage_emit.rs: {e}"))?;
    assert_eq!(
        generated_after, generated_legacy_src,
        "File inside modular_codegen/generated/ must remain byte-for-byte untouched"
    );

    let direct_gen_report = run_docstring_injection(&DocstringInjectionOptions {
        target_path: gen_dir,
        dry_run: false,
        force: false,
        schema_model: None,
    })
    .map_err(|(code, msg)| format!("Direct generated dir scan failed ({code:?}): {msg}"))?;
    assert_eq!(direct_gen_report.files_scanned, 0);
    assert_eq!(direct_gen_report.files_modified, 0);

    fs::remove_dir_all(&dir).map_err(|e| format!("Failed to clean up temp dir: {e}"))?;
    Ok(())
}
