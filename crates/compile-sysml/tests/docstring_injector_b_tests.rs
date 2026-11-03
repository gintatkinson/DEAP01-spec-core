//! Integration test suite B for `compile-sysml --inject-safety-docstrings` CLI exit codes and Issue #520.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Verifies DO-178C Level A and DO-330 TQL-4 CLI exit codes (`1` on broken/missing `--schema`,
//! `2` on unforced `Realises` conflict, `0` with `--force`) and Issue #520 multi-requirement/constraint
//! schema aggregation, Dart AST detail inspection, and unbounded Rust item detail extraction.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Nonexistent or syntactically invalid `--schema` paths exit with status code `1`.
//! - Conflicting `Realises` tags without `--force` exit with status code `2`.
//! - Multiple satisfied requirements and constraints aggregate without `.first()` or 3/5/6 item truncation.
//!
//! ## 3. Preconditions
//! - Writable OS temporary directory and `CARGO_BIN_EXE_compile-sysml` binary.
//!
//! ## 4. Postconditions
//! - Temporary test directories are cleaned up prior to function exit.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ over fixture source files; Space Complexity: $O(N)$.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Propagates fixture setup, parse, and subprocess errors via `Result<(), String>`.
//!
//! ## 7. Realization & Traceability
//! Safety & Traceability: Realises: [REQ-DEFECT-09-DOCSTRING-CLI-EXIT-CODES], [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS]
//! Contract: Verifies CLI exit codes (`0`, `1`, `2`) and Issue #520 aggregation invariants.
//! Bounds & Faults: Isolated temporary directories under `std::env::temp_dir()`.

use crate::docstring_injector_tests::unique_test_dir;
use compile_sysml::docstring::{inject_dart_file_docstrings, inject_rust_file_docstrings};
use compile_sysml::parse_sysml;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Writes the broken schema, valid schema, and conflicting Rust source for CLI exit code testing.
/// Realises: [REQ-DEFECT-09-DOCSTRING-CLI-EXIT-CODES]
fn write_cli_exit_code_fixtures(
    dir: &Path,
    src_dir: &Path,
) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let broken_schema = dir.join("broken.sysml");
    fs::write(&broken_schema, "package Broken { part def Unclosed ")
        .map_err(|e| format!("write broken.sysml: {e}"))?;

    let valid_schema = dir.join("valid.sysml");
    let valid_sysml = "\
package FlightControlPkg {
    requirement def AutopilotAuthorityReq {
        id = \"REQ-SCHEMA-001\";
        doc /* Autopilot authority requirement */
    }
    part def AutopilotController {
        satisfy requirement AutopilotAuthorityReq;
    }
}
";
    fs::write(&valid_schema, valid_sysml).map_err(|e| format!("write valid.sysml: {e}"))?;

    let rust_file = src_dir.join("autopilot.rs");
    let conflicting_rust = "\
/// Autopilot controller struct.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A safety objectives for struct `AutopilotController`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic control authority invariants.
/// ## 3. Preconditions
/// - Caller supplies initialized gain parameters.
/// ## 4. Postconditions
/// - Defines the authoritative `AutopilotController` state.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1); Space Complexity: O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates inputs.
/// ## 7. Realization & Traceability
/// Realises: [REQ-WRONG-999] | SysML v2: PartDef(AutopilotController)
pub struct AutopilotController {
    pub enabled: bool,
}
";
    fs::write(&rust_file, conflicting_rust).map_err(|e| format!("write autopilot.rs: {e}"))?;
    Ok((broken_schema, valid_schema, rust_file))
}

/// Executes CLI exit code `1`, `2`, and `0` (`--force`) verification checks.
/// Safety & Traceability: Realises: [REQ-DEFECT-09-DOCSTRING-CLI-EXIT-CODES]
/// Contract: Verifies exit code `1` on missing/broken schema, `2` on conflict, and `0` with `--force`.
/// Bounds & Faults: Cleans up temporary directory before returning.
pub fn run_cli_exit_code_1_and_2_checks() -> Result<(), String> {
    let dir = unique_test_dir("feat42_cli_exit_codes")?;
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("create src dir: {e}"))?;
    let (broken_schema, valid_schema, rust_file) = write_cli_exit_code_fixtures(&dir, &src_dir)?;
    let bin = env!("CARGO_BIN_EXE_compile-sysml");

    let out_nonexistent = Command::new(bin)
        .args(["--inject-safety-docstrings"])
        .arg(&src_dir)
        .arg("--schema")
        .arg(dir.join("nonexistent.sysml"))
        .output()
        .map_err(|e| format!("run nonexistent schema: {e}"))?;
    assert_eq!(out_nonexistent.status.code(), Some(1));

    let out_broken = Command::new(bin)
        .args(["--inject-safety-docstrings"])
        .arg(&src_dir)
        .arg("--schema")
        .arg(&broken_schema)
        .output()
        .map_err(|e| format!("run broken schema: {e}"))?;
    assert_eq!(out_broken.status.code(), Some(1));

    let out_conflict = Command::new(bin)
        .args(["--inject-safety-docstrings"])
        .arg(&src_dir)
        .arg("--schema")
        .arg(&valid_schema)
        .output()
        .map_err(|e| format!("run conflict schema: {e}"))?;
    assert_eq!(out_conflict.status.code(), Some(2));

    let out_forced = Command::new(bin)
        .args(["--inject-safety-docstrings"])
        .arg(&src_dir)
        .arg("--schema")
        .arg(&valid_schema)
        .arg("--force")
        .output()
        .map_err(|e| format!("run forced schema: {e}"))?;
    assert_eq!(out_forced.status.code(), Some(0));

    let updated_rust = fs::read_to_string(&rust_file).map_err(|e| format!("read autopilot.rs: {e}"))?;
    assert!(updated_rust.contains("[REQ-SCHEMA-001]"));
    assert!(!updated_rust.contains("REQ-WRONG-999"));

    fs::remove_dir_all(&dir).map_err(|e| format!("cleanup temp dir: {e}"))?;
    Ok(())
}

/// Verifies Issue #520 Sub-Defect A: multi-requirement and multi-constraint aggregation + partial conflict detection.
/// Realises: [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS]
fn verify_issue_520_subdefect_a() -> Result<(), String> {
    let schema_sysml = "\
package CoreVerificationPkg {
    requirement def ReqAlphaOne { id = \"REQ-ALPHA-001\"; doc /* First alpha requirement */ }
    requirement def ReqAlphaTwo { id = \"REQ-ALPHA-002\"; doc /* Second alpha requirement */ }
    requirement def ReqActOne { id = \"REQ-ACT-001\"; doc /* First action requirement */ }
    requirement def ReqActTwo { id = \"REQ-ACT-002\"; doc /* Second action requirement */ }
    part def AlphaRecord {
        satisfy requirement ReqAlphaOne;
        satisfy requirement ReqAlphaTwo;
        constraint def BoundAlpha { val_a >= 0 }
        constraint def BoundBeta { val_b <= 100 }
        action def evaluate_record {
            doc /* Realises: [REQ-ACT-001], [REQ-ACT-002] */
            in val_a: Integer;
            in val_b: Integer;
        }
        constraint def invariant_evaluate_record_0 { val_a > 0 }
        constraint def invariant_evaluate_record_1 { val_b < 100 }
    }
}
";
    let schema_model = parse_sysml(schema_sysml).map_err(|e| format!("parse schema_sysml: {e}"))?;
    let rust_alpha_src = "pub struct AlphaRecord {\n    pub val_a: i64,\n    pub val_b: i64,\n}\nimpl AlphaRecord {\n    pub fn evaluate_record(&self, val_a: i64, val_b: i64) -> bool {\n        val_a > 0 && val_b < 100\n    }\n}\n";
    let (rust_alpha_enriched, alpha_count) = inject_rust_file_docstrings(
        rust_alpha_src,
        "crates/core_module/src/alpha.rs",
        Some(&schema_model),
        false,
    )
    .map_err(|e| e.into_message())?;
    assert_eq!(alpha_count, 2);
    for needle in [
        "Realises: [REQ-ALPHA-001], [REQ-ALPHA-002] | SysML v2: PartDef(AlphaRecord)",
        "BoundAlpha (val_a >= 0); BoundBeta (val_b <= 100)",
        "Realises: [REQ-ACT-001], [REQ-ACT-002] | SysML v2: ActionDef(AlphaRecord::evaluate_record)",
        "invariant_evaluate_record_0 (val_a > 0); invariant_evaluate_record_1 (val_b < 100)",
    ] {
        assert!(rust_alpha_enriched.contains(needle), "missing {needle}:\n{rust_alpha_enriched}");
    }

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
        return Err("Expected unforced docstring missing second schema requirement to fail".to_string());
    }
    Ok(())
}

/// Verifies Issue #520 Sub-Defects B and C: Dart AST details and unbounded Rust item details.
/// Realises: [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS]
fn verify_issue_520_subdefects_b_and_c() -> Result<(), String> {
    let dart_src = "\
class BetaController {
  final double gainAlpha;
  late int stepCount = 0;
  BetaController(this.gainAlpha);
  double computeStep(double inputVal) {
    assert(inputVal >= 0.0);
    assert(stepCount >= 0, \"non-negative step count\");
    return inputVal * gainAlpha;
  }
}
enum GammaState { idle, active, completed, faulted, suspended, archived, terminated }
";
    let (dart_enriched, dart_count) =
        inject_dart_file_docstrings(dart_src, "lib/beta_controller.dart", None, false)
            .map_err(|e| e.into_message())?;
    assert!(dart_count >= 4);
    for needle in ["gainAlpha: double, stepCount: int", "inputVal >= 0.0 && stepCount >= 0", "idle, active, completed, faulted, suspended, archived, terminated"] {
        assert!(dart_enriched.contains(needle), "missing {needle}:\n{dart_enriched}");
    }
    let (dart_second, dart_second_count) = inject_dart_file_docstrings(&dart_enriched, "lib/beta_controller.dart", None, false).map_err(|e| e.into_message())?;
    assert_eq!(dart_second_count, 0);
    assert_eq!(dart_enriched, dart_second);

    let wide_rust_src = "\
pub struct WideRecord {
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
";
    let (wide_enriched, wide_count) = inject_rust_file_docstrings(wide_rust_src, "crates/core_module/src/wide.rs", None, false).map_err(|e| e.into_message())?;
    assert!(wide_count >= 4);
    for needle in ["field_6: u32, field_7: u32", "State7, State8", "fn stage_7, fn stage_8", "val > 4 && val > 5"] {
        assert!(wide_enriched.contains(needle), "missing {needle}:\n{wide_enriched}");
    }
    Ok(())
}

/// Runs all Issue #520 multi-requirement, Dart AST detail, and wide Rust item detail checks.
/// Safety & Traceability: Realises: [REQ-FEAT-42-DOCSTRING-INJECTOR-INTEGRATION-TESTS]
/// Contract: Executes `verify_issue_520_subdefect_a` and `verify_issue_520_subdefects_b_and_c`.
/// Bounds & Faults: Pure in-memory evaluation; propagates errors via `Result`.
pub fn run_issue_520_aggregation_checks() -> Result<(), String> {
    verify_issue_520_subdefect_a()?;
    verify_issue_520_subdefects_b_and_c()
}
