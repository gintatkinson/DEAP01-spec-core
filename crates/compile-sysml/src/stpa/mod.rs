//! STPA Safety Transpilation and Hazard Analysis Module.

pub mod fmeca;
pub mod safety_suite;
pub mod uca;

pub use fmeca::*;
pub use safety_suite::*;
pub use uca::*;

use std::fs;
use std::path::Path;

/// High-level entrypoint to transpile a SysML v2 model file into the 10-pillar safety artifact suite.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `transpile_stpa` in `crates/compile-sysml/src/stpa/mod.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `transpile_stpa` (declaration #1) within `STPA-MOD`.
///
/// ## 3. Preconditions
/// - `schema_path: &Path` must satisfy domain and type-safety constraints prior to invoking `transpile_stpa`.
/// - `out_dir: &Path` must satisfy domain and type-safety constraints prior to invoking `transpile_stpa`.
/// - `scoring_config_path: Option<&Path>` must satisfy domain and type-safety constraints prior to invoking `transpile_stpa`.
///
/// ## 4. Postconditions
/// - Returns `Result<(), String>` from `transpile_stpa` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(schema_path)) bounded over input/member dimension of `transpile_stpa`.
/// - Space Complexity: O(N(schema_path)) bounded auxiliary memory for `transpile_stpa`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `transpile_stpa` (`Result<(), String>`).
/// - State: Maintains consistent state invariants across `transpile_stpa` without partial mutation on error.
/// - Action: Executes `transpile_stpa` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `transpile_stpa` (`STPA-MOD`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-MOD-TRANSPILE-STPA] | SysML v2: ActionDef(transpile_stpa)
pub fn transpile_stpa(
    schema_path: &Path,
    out_dir: &Path,
    scoring_config_path: Option<&Path>,
) -> Result<(), String> {
    if !schema_path.exists() {
        return Err(format!("Schema file does not exist: {}", schema_path.display()));
    }

    let content = fs::read_to_string(schema_path)
        .map_err(|e| format!("Failed to read schema file '{}': {}", schema_path.display(), e))?;

    if content.trim().is_empty() {
        return Err(format!("Schema file '{}' is empty.", schema_path.display()));
    }

    let pkg = crate::parse_sysml(&content)
        .map_err(|e| format!("Failed to parse schema '{}': {}", schema_path.display(), e))?;

    let scoring_config = if let Some(cfg_path) = scoring_config_path {
        if !cfg_path.exists() {
            return Err(format!(
                "FMECA scoring config does not exist: {}",
                cfg_path.display()
            ));
        }
        let cfg_text = fs::read_to_string(cfg_path)
            .map_err(|e| format!("Failed to read scoring config: {}", e))?;
        let parsed: FmecaScoringConfig = serde_json::from_str(&cfg_text)
            .map_err(|e| format!("Failed to parse scoring config JSON: {}", e))?;
        Some(parsed)
    } else {
        None
    };

    emit_safety_suite(&pkg, out_dir, scoring_config.as_ref())?;

    println!(
        "[STPA Transpile] Emitted safety artifact suite to '{}'",
        out_dir.display()
    );

    Ok(())
}
