#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! STPA Safety Artifact Suite Transpiler.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! safety transpilation objectives for module `crates/compile-sysml/src/stpa/safety_suite/mod.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Synthesizes the 10-pillar STPA, FMECA, RTA, Hazard Log, and SLDV safety artifact suite
//!   deterministically from SysML v2 `PackageDef` AST models using checked slice accessors and saturating arithmetic.
//! - Propagates `FAILURE_MODE_REGISTRY.md` entries identically across `05_FMECA_MATRIX.md` and `STPA_MATRIX.md`.
//!
//! ## 3. Preconditions
//! - Caller supplies a parsed `PackageDef`, optional `FmecaScoringConfig`, and optional output directory path.
//!
//! ## 4. Postconditions
//! - Emits an ordered `BTreeMap<String, String>` of all 10 safety artifacts and persists them atomically via `write_atomic`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N(pkg) + R) over AST package cardinality and registry row count R.
//! - Space Complexity: O(N(pkg) + R) across rendered Markdown and MATLAB artifact buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Accepts empty `registry_entries` (`&[]`) and `None` scoring configurations cleanly.
//! - State: Uses `write_atomic` per destination file to prevent partial or torn artifact writes on I/O failure.
//! - Action: Enforces bounded expression recursion (`MAX_EXPR_DEPTH = 64`) and non-panicking UTF-8 boundary scanning.
//! - Resource: Bounded memory proportional to rendered safety documentation size.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-SAFETY-SUITE-MODULE] | SysML v2: PackageDef(safety_suite)

pub mod constraint_compiler;
pub mod expr_lowering;
mod expr_lowering_lexer;
mod expr_lowering_matlab;
mod expr_lowering_parser;
mod expr_lowering_render;
mod loss_scenarios;
pub mod pillars;
pub mod pillars_suite;
mod regulatory_objectives;
mod rta_renderer;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_extended;
pub mod tokens;
pub mod topology_uca;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub use constraint_compiler::*;
pub use expr_lowering::*;
use expr_lowering_parser::parse_sysml_expr;
use expr_lowering_render::{
    contains_latex_command_outside_quotes, escape_katex_ident, fallback_tokens_to_katex,
    fallback_tokens_to_matlab, render_ast_katex, render_ast_matlab, sanitize_preauthored_katex,
};
pub use pillars::*;
use pillars::{render_regulatory_objectives_assessment, render_sldv_script};
use pillars_suite::*;
pub use tokens::*;
use tokens::{find_token_ci, find_token_prefix_ci, sanitize_markdown_cell};
use topology_uca::*;

use super::fmeca::{
    generate_fmeca_matrix_with_registry, load_failure_mode_registry_for_out_dir, FmecaRow,
    FmecaScoringConfig, RegistryFailureEntry,
};
use super::uca::{expand_cartesian_stpa, UnsafeControlAction};
use crate::semantic::digest::write_atomic;
use deap_core::sysml_ast::{ConstraintDef, PackageDef, PartDef};

/// Transpiles the SysML v2 AST into the 10-pillar safety artifact suite.
/// Realises: [REQ-STPA-SAFETY-SUITE-TRANSPILE-SAFETY-SUITE]
pub fn transpile_safety_suite(
    pkg: &PackageDef,
    scoring_config: Option<&FmecaScoringConfig>,
) -> BTreeMap<String, String> {
    transpile_safety_suite_with_registry(pkg, scoring_config, &[])
}

/// Transpiles the complete 10-pillar STPA safety artifact suite integrating pre-loaded `FAILURE_MODE_REGISTRY.md` records.
/// Realises: [REQ-STPA-SAFETY-SUITE-TRANSPILE-WITH-REGISTRY]
pub fn transpile_safety_suite_with_registry(
    pkg: &PackageDef,
    scoring_config: Option<&FmecaScoringConfig>,
    registry_entries: &[RegistryFailureEntry],
) -> BTreeMap<String, String> {
    let mut artifacts = BTreeMap::new();
    let tokens = collect_parameter_tokens(pkg);
    let ucas = expand_cartesian_stpa(pkg);
    let constraints = collect_all_constraints(pkg);
    let fmeca_rows = generate_fmeca_matrix_with_registry(pkg, scoring_config, registry_entries);

    artifacts.insert(
        "01_LOSSES_HAZARDS_TOPOLOGY.md".to_string(),
        render_losses_hazards_topology(pkg, &ucas),
    );
    artifacts.insert(
        "02_UCA_COMBINATORIAL_MATRIX.md".to_string(),
        render_uca_matrix(&ucas),
    );
    artifacts.insert(
        "03_LOSS_SCENARIOS.md".to_string(),
        render_loss_scenarios(pkg, &ucas, &constraints),
    );
    artifacts.insert(
        "04_SAFETY_CONSTRAINTS.md".to_string(),
        render_safety_constraints(pkg, &ucas, &constraints),
    );
    artifacts.insert(
        "05_FMECA_MATRIX.md".to_string(),
        render_fmeca_matrix_doc(&fmeca_rows),
    );
    artifacts.insert(
        "06_REGULATORY_OBJECTIVES_ASSESSMENT.md".to_string(),
        render_regulatory_objectives_assessment(pkg, &tokens),
    );
    artifacts.insert(
        "07_RTA_ARCHITECTURE.md".to_string(),
        render_rta_architecture(pkg, &tokens),
    );
    artifacts.insert(
        "STPA_MATRIX.md".to_string(),
        render_unified_stpa_matrix(pkg, &ucas, &constraints, &fmeca_rows, &tokens),
    );
    artifacts.insert(
        "HAZARD_LOG.md".to_string(),
        render_hazard_log(pkg, &ucas, &constraints, &tokens),
    );
    artifacts.insert(
        "SLDV_FORMAL_PROOFS.m".to_string(),
        render_sldv_script(pkg, &constraints),
    );

    artifacts
}

/// Writes all generated safety suite artifacts to `out_dir` atomically.
/// Realises: [REQ-STPA-SAFETY-SUITE-EMIT-SAFETY-SUITE]
pub fn emit_safety_suite(
    pkg: &PackageDef,
    out_dir: &Path,
    scoring_config: Option<&FmecaScoringConfig>,
) -> Result<(), String> {
    fs::create_dir_all(out_dir).map_err(|e| {
        format!(
            "Failed to create output directory '{}': {}",
            out_dir.display(),
            e
        )
    })?;

    let registry_entries = load_failure_mode_registry_for_out_dir(out_dir);
    let artifacts = transpile_safety_suite_with_registry(pkg, scoring_config, &registry_entries);
    for (filename, content) in artifacts {
        let dest = out_dir.join(&filename);
        write_atomic(&dest, &content).map_err(|e| {
            format!(
                "Failed to write safety artifact '{}': {}",
                dest.display(),
                e
            )
        })?;
    }

    Ok(())
}

/// Returns a cycled element reference from `slice` at `idx` using checked remainder and `.get()`.
/// Realises: [REQ-STPA-SAFETY-SUITE-RESOLVE-LOSS-ANCHORS]
pub(super) fn get_cycled<T>(slice: &[T], idx: usize) -> Option<&T> {
    let rem = idx.checked_rem(slice.len())?;
    slice.get(rem)
}
