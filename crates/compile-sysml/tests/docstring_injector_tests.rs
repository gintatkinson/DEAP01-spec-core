//! Integration test suite for `compile-sysml --inject-safety-docstrings` and `--extract-source` (FEAT-42 & DEFECT-09).
//!
//! /// Realises: [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS], [REQ-DEFECT-09-DOCSTRING-CLI-EXIT-CODES]
//!
//! # Summary
//! Exercises end-to-end 7-section safety docstring synthesis, second-pass bit-for-bit idempotency,
//! duplicate callable docblock disambiguation, Dart 3 AST member coverage, fail-closed `--schema`
//! validation (exit code `1`), and conflicting unforced docstring detection (exit code `2`).
//!
//! # Preconditions
//! - The `compile-sysml` library and binary target (`CARGO_BIN_EXE_compile-sysml`) compile cleanly.
//! - Temporary filesystem directories under `std::env::temp_dir()` are writable by the test process.
//!
//! # Postconditions
//! - Every test cleans up its temporary workspace directory prior to returning `Ok(())`.
//! - Confirms exit codes `0`, `1`, and `2` match FEAT-42 Section 2.2 and DO-178C Level A fail-closed rules.
//!
//! # Invariants
//! - Zero `.unwrap()`, `.expect()`, `panic!()`, direct slice indexing, or `#![allow(...)]` suppressions.
//! - All temporary test fixtures reside exclusively under `std::env::temp_dir()`.
//!
//! # Failure Modes
//! - Any unexpected exit status, I/O error, or docstring synthesis mismatch returns `Err(String)`.
//!
//! # Side Effects
//! - Creates and deletes isolated temporary directories in the OS temp folder and spawns `compile-sysml` subprocesses.
//!
//! # Determinism
//! - Deterministic AST lowering, docstring generation, and SHA-256 digest comparisons across runs.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
//! requirements for `CompileSysml::SafetyDocstringSynthesizer` and `IngestSysml::SourceCodeAstIngestion`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Bit-for-bit idempotency on second pass (`files_modified == 0`, identical SHA-256).
//! - Fail-closed exit code `1` on broken or unreadable `--schema` input.
//! - Fail-closed exit code `2` on conflicting unforced `Realises: [...]` tags against `--schema`.
//!
//! ## 3. Preconditions
//! - Host OS provides writable `std::env::temp_dir()` and compiled `compile-sysml` binary.
//!
//! ## 4. Postconditions
//! - All 5 integration tests pass and remove their temporary directories.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ per test fixture of byte size $N$.
//! - Space Complexity: $O(N)$ for in-memory source and AST buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Propagates all I/O, parse, and subprocess errors via `Result<_, String>`.
//! - State: Each test uses a PID- and nanosecond-qualified directory path.
//! - Action: Never panics or unwraps.
//! - Resource: Removes temporary directories on completion.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS], [REQ-DEFECT-09-DOCSTRING-CLI-EXIT-CODES]

use compile_sysml::docstring::{
    inject_dart_file_docstrings, inject_rust_file_docstrings, run_docstring_injection,
    run_extract_source, DocstringExitCode, DocstringInjectionOptions,
};
use compile_sysml::{count_structural_elements, generate_digest, parse_sysml};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Creates a unique temporary directory under `std::env::temp_dir()` for an integration test.
///
/// /// Realises: [REQ-FEAT-42-UNIQUE-TEST-DIR]
///
/// # Summary
/// Allocates and creates a collision-free temporary directory named `<prefix>_<pid>_<nanos>`.
///
/// # Preconditions
/// - `prefix` is a non-empty ASCII label identifying the calling test.
///
/// # Postconditions
/// - Returns `Ok(PathBuf)` pointing to an existing writable directory on disk.
///
/// # Invariants
/// - Never creates files or directories inside the repository workspace root.
///
/// # Failure Modes
/// - Returns `Err(String)` if `fs::create_dir_all` fails.
///
/// # Side Effects
/// - Creates a directory under `std::env::temp_dir()`.
///
/// # Determinism
/// - Deterministic path structure parameterized by OS process ID and monotonic system clock timestamp.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-330 TQL-4 isolated test fixture management (`unique_test_dir`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Directory path is strictly outside the repository working tree.
///
/// ## 3. Preconditions
/// - `prefix: &str` is a valid filename prefix.
///
/// ## 4. Postconditions
/// - Returns `Ok(PathBuf)` after creating the directory on disk.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) syscall overhead.
/// - Space Complexity: O(P) where P is the path length.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates directory creation errors via `Result<PathBuf, String>`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-UNIQUE-TEST-DIR]
fn unique_test_dir(prefix: &str) -> Result<PathBuf, String> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!("{}_{}_{}", prefix, std::process::id(), nanos));
    fs::create_dir_all(&path)
        .map_err(|e| format!("Failed to create unique temp dir '{}': {}", path.display(), e))?;
    Ok(path)
}

/// Verifies AC-03 and AC-04 Rust and Dart 7-section docstring synthesis and second-pass idempotency.
///
/// /// Realises: [REQ-FEAT-42-AC03-AC04-SYNTHESIS-IDEMPOTENCY]
///
/// # Summary
/// Runs `run_docstring_injection` twice across a mixed Rust and Dart source tree with a `SafetyModel`
/// SysML v2 schema and verifies complete 7-section enrichment, attribute placement, and bit-for-bit idempotency.
///
/// # Preconditions
/// - Temporary directory is writable and `parse_sysml` parses the inline `SafetyModel` schema.
///
/// # Postconditions
/// - First run modifies both `.rs` and `.dart` files and enriches `>= 10` items; second run modifies `0` files.
///
/// # Invariants
/// - Placeholder `"Member documentation."` lines are stripped and `#[derive(...)]` remains below the synthesized docstring.
///
/// # Failure Modes
/// - Returns `Err(String)` if schema parsing, docstring injection, or SHA-256 idempotency fails.
///
/// # Side Effects
/// - Writes and removes temporary `.rs` and `.dart` files under `std::env::temp_dir()`.
///
/// # Determinism
/// - Second pass produces identical file contents and SHA-256 digests.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 AC-03/AC-04 verification (`test_ac03_and_ac04_rust_and_dart_docstring_synthesis_and_idempotency`).
///
/// ## 2. Invariants & Mathematical Properties
/// - `report_2.files_modified == 0` and `rust_digest_1 == rust_digest_2`.
///
/// ## 3. Preconditions
/// - Valid `SafetyModel` SysML v2 fixture.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` after cleaning up the temporary directory.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) in fixture source size.
/// - Space Complexity: O(N) for source and digest strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking error propagation via `Result<(), String>`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-AC03-AC04-SYNTHESIS-IDEMPOTENCY]
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

/// Verifies that duplicate function docstrings within the same file are disambiguated automatically.
///
/// /// Realises: [REQ-FEAT-42-DUPLICATE-DOCSTRING-DISAMBIGUATION]
///
/// # Summary
/// Confirms that when two Rust functions share a verbatim 7-section docstring body, the first is
/// preserved and the second is re-synthesized with function-specific attributes so second pass is idempotent.
///
/// # Preconditions
/// - Input Rust source contains two functions (`first_op`, `second_op`) with identical 7-section docblocks.
///
/// # Postconditions
/// - First pass returns `enriched == 1`; second pass returns `enriched_again == 0`.
///
/// # Invariants
/// - Enforces Check 34 / Check 36 per-file normalized callable docblock uniqueness.
///
/// # Failure Modes
/// - Returns `Err(String)` if injection fails or fails idempotency on the second pass.
///
/// # Side Effects
/// - Pure in-memory string transformation; zero filesystem writes.
///
/// # Determinism
/// - Deterministic in-memory synthesis.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 36 anti-copy-paste enforcement (`test_duplicate_function_docstrings_are_disambiguated`).
///
/// ## 2. Invariants & Mathematical Properties
/// - `enriched == 1` on pass 1 and `enriched_again == 0` on pass 2.
///
/// ## 3. Preconditions
/// - Valid Rust source string with two identical function docblocks.
///
/// ## 4. Postconditions
/// - Returns `Ok(())`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) in source length.
/// - Space Complexity: O(N) for output strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates any error via `Result<(), String>`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-DUPLICATE-DOCSTRING-DISAMBIGUATION]
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
///
/// /// Realises: [REQ-FEAT-42-EXTRACT-SOURCE-AND-CLI]
///
/// # Summary
/// Tests `run_extract_source`, `compile-sysml --inject-safety-docstrings` subprocess execution,
/// missing-path error exit code `1`, and direct Dart class/constructor docstring injection.
///
/// # Preconditions
/// - `CARGO_BIN_EXE_compile-sysml` is available and temporary directory is writable.
///
/// # Postconditions
/// - Extracted `.sysml` and digest files exist and re-parse with `count_structural_elements > 0`.
///
/// # Invariants
/// - Missing target path fails closed with `DocstringExitCode::Error` (`1`).
///
/// # Failure Modes
/// - Returns `Err(String)` on I/O, parse, or subprocess failure.
///
/// # Side Effects
/// - Creates and removes temporary files under `std::env::temp_dir()`.
///
/// # Determinism
/// - Deterministic AST extraction and CLI exit code verification.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 Section 2.2 CLI verification (`test_extract_source_and_cli_flags`).
///
/// ## 2. Invariants & Mathematical Properties
/// - `DocstringExitCode::Error.code() == 1`.
///
/// ## 3. Preconditions
/// - Temporary directory under `std::env::temp_dir()` is accessible.
///
/// ## 4. Postconditions
/// - Cleans up temporary directory and returns `Ok(())`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) in fixture file size.
/// - Space Complexity: O(N) for parsed AST.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates errors via `Result<(), String>`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-EXTRACT-SOURCE-AND-CLI]
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
///
/// /// Realises: [REQ-FEAT-42-DART-MULTILINE-AND-STRING-SLICE-SAFETY]
///
/// # Summary
/// Confirms `inject_dart_file_docstrings` enriches constructors, factory constructors, getters,
/// `@override` methods, and multiple classes per file while preserving string literals with `//` and UTF-8 characters.
///
/// # Preconditions
/// - Input Dart source contains `PrimaryService` and `SecondaryService` with URL strings, UTF-8 bullets/quotes, and string interpolations.
///
/// # Postconditions
/// - First pass enriches `>= 8` items; second pass enriches `0` items and matches byte-for-byte.
///
/// # Invariants
/// - `//` inside string literals is never stripped as a line comment.
///
/// # Failure Modes
/// - Returns `Err(String)` if Dart syntax validation or docstring injection fails.
///
/// # Side Effects
/// - Pure in-memory string transformation; zero disk I/O.
///
/// # Determinism
/// - Byte-for-byte idempotent on second run.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 36 Dart AST coverage (`test_injector_handles_multiline_dart_classes_string_slashes_interpolated_utf8_quotes_and_dart3_items`).
///
/// ## 2. Invariants & Mathematical Properties
/// - `second_count == 0` and `injected == second_injected`.
///
/// ## 3. Preconditions
/// - Valid multi-class Dart source buffer.
///
/// ## 4. Postconditions
/// - Returns `Ok(())`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) in Dart source length.
/// - Space Complexity: O(N) for enriched buffer.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `Err(String)` on any assertion or parser error.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-DART-MULTILINE-AND-STRING-SLICE-SAFETY]
#[test]
fn test_injector_handles_multiline_dart_classes_string_slashes_interpolated_utf8_quotes_and_dart3_items()
-> Result<(), String> {
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

/// Verifies CLI exit code `1` on broken `--schema` and exit code `2` on conflicting unforced docstring (DEFECT-09).
///
/// /// Realises: [REQ-DEFECT-09-DOCSTRING-CLI-EXIT-CODES], [REQ-FEAT-42-DOCSTRING-EXIT-CODE]
///
/// # Summary
/// Invokes the `compile-sysml` binary to verify that:
/// (1) an unreadable or syntactically invalid `--schema` exits with status `1`,
/// (2) an unforced docstring whose `Realises: [REQ-WRONG-999]` conflicts with the schema's
///     `REQ-SCHEMA-001` exits with status `2`, and
/// (3) passing `--force` overwrites `REQ-WRONG-999` with `REQ-SCHEMA-001` and exits with status `0`.
///
/// # Preconditions
/// - `CARGO_BIN_EXE_compile-sysml` binary is built and temporary directory is writable.
///
/// # Postconditions
/// - All three CLI exit statuses (`1`, `2`, `0`) and the forced requirement overwrite are empirically verified.
///
/// # Invariants
/// - `--schema` errors never fail open to `None`.
/// - Conflicting unforced docstrings never silently pass with exit code `0` or map to generic error `1`.
///
/// # Failure Modes
/// - Returns `Err(String)` if any CLI invocation returns an unexpected exit status or fails to overwrite under `--force`.
///
/// # Side Effects
/// - Spawns `compile-sysml` child processes against temporary files in `std::env::temp_dir()`.
///
/// # Determinism
/// - Deterministic process exit status codes (`1`, `2`, `0`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and FEAT-42 Section 2.2 exit code contract
/// (`test_cli_exit_code_1_on_broken_schema_and_exit_code_2_on_conflicting_unforced_docstring`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Invalid `--schema` -> exit code `1`.
/// - Conflicting unforced docstring -> exit code `2`.
/// - Forced overwrite (`--force`) -> exit code `0` with `[REQ-SCHEMA-001]` in the updated source file.
///
/// ## 3. Preconditions
/// - Writable temp directory and `CARGO_BIN_EXE_compile-sysml` binary.
///
/// ## 4. Postconditions
/// - Removes temporary directory and returns `Ok(())`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) across three subprocess invocations on small fixture files.
/// - Space Complexity: O(N) for temporary fixture files.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `Err(String)` on any unexpected exit status or file read/write error.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEFECT-09-DOCSTRING-CLI-EXIT-CODES], [REQ-FEAT-42-DOCSTRING-EXIT-CODE]
#[test]
fn test_cli_exit_code_1_on_broken_schema_and_exit_code_2_on_conflicting_unforced_docstring()
-> Result<(), String> {
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
        .map_err(|e| format!("Failed to run compile-sysml on conflicting docstring: {}", e))?;
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

    let updated_rust =
        fs::read_to_string(&rust_file).map_err(|e| format!("Failed to read autopilot.rs: {}", e))?;
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C traceability and
/// structural completeness requirements (`test_issue_520_multi_req_constraint_aggregation_dart_details_and_rust_unbounded_items`).
///
/// ## 2. Invariants & Mathematical Properties
/// - `resolve_schema_context` aggregates all satisfied requirements (joined with `", "`) and all constraints (joined with `"; "`) within the winning specificity tier.
/// - `inject_dart_file_docstrings` populates `fields_or_variants` for Dart classes/mixins/enums and `invariants` from callable `assert(...)` statements.
/// - `inspect_rust_item_details` extracts all `assert!` invariants (>3), struct fields (>5), enum variants (>6), and trait methods (>6) without arbitrary count caps.
///
/// ## 3. Preconditions
/// - `parse_sysml`, `inject_rust_file_docstrings`, and `inject_dart_file_docstrings` are available in-memory.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` when all three sub-defects of Issue #520 are verified and second-pass idempotency holds.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) in synthetic fixture byte length.
/// - Space Complexity: O(N) for in-memory enriched buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Propagates all parse and synthesis errors via `Result<(), String>`.
/// - State: Pure in-memory evaluation with zero filesystem side effects.
/// - Action: Zero `.unwrap()`, `.expect()`, or panicking operations.
/// - Resource: Bounded in-memory strings.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ISSUE-520-MULTI-REQ-DART-DETAILS-RUST-UNBOUNDED-TEST]
#[test]
fn test_issue_520_multi_req_constraint_aggregation_dart_details_and_rust_unbounded_items()
-> Result<(), String> {
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
    let (dart_enriched, dart_count) = inject_dart_file_docstrings(
        dart_src,
        "lib/beta_controller.dart",
        None,
        false,
    )
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
    let (dart_second, dart_second_count) = inject_dart_file_docstrings(
        &dart_enriched,
        "lib/beta_controller.dart",
        None,
        false,
    )
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
    let (wide_enriched, wide_count) = inject_rust_file_docstrings(
        wide_rust_src,
        "crates/core_module/src/wide.rs",
        None,
        false,
    )
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

