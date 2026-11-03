//! Regression and reproduction test suite for `compile-sysml` forward synchronization and AST verification.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
//! objectives for `crates/compile-sysml/tests/reproducer_tests.rs`. Provides deterministic
//! regression entrypoints and shared RAII/Markdown verification utilities for adversarial audit
//! findings in `compile-sysml`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Unallocated forward-synced specification files never synthesize hardcoded arithmetic fallback
//!   tracker issue IDs (`#10`, `#89`, `#95`, `#101`) that could collide with existing historical
//!   GitHub issues; unallocated specifications emit `TBD` while allocated tracker IDs survive re-sync.
//! - All AST-to-safety-suite and AST-to-Markdown forward synchronization projections remain 100%
//!   domain-neutral, free of compiler self-description boilerplate, and bounded in complexity.
//!
//! ## 3. Preconditions
//! - `compile_sysml` public APIs are accessible and `std::env::temp_dir()` is writable for
//!   ephemeral RAII `TempDirGuard` test fixtures.
//!
//! ## 4. Postconditions
//! - All temporary test fixtures are removed deterministically on scope exit and all assertions hold.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ over the AST node count $N$ of synthetic SysML package fixtures.
//! - Space Complexity: $O(N)$ bounded auxiliary memory for generated Markdown specification buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Validates boundary inputs and file paths passed to `compile_sysml` entrypoints.
//! - State: Ensures zero cross-test state contamination via process-, timestamp-, and atomic-sequence-scoped
//!   temporary directories.
//! - Action: Executes deterministically with zero unbounded recursion, panics, or `.unwrap()` calls.
//! - Resource: Cleans up temporary filesystem directories deterministically on drop.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYNC-FORWARD-RESOLVE-ISSUE-ID] | SysML v2: VerificationCaseDef(reproducer_tests)

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

/// RAII temporary directory guard that deterministically removes ephemeral filesystem fixtures on drop.
pub(crate) struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    pub(crate) fn new(prefix: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let seq = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "{}_{}_{}_{}",
            prefix,
            std::process::id(),
            nanos,
            seq
        ));
        if path.exists() {
            fs::remove_dir_all(&path)?;
        }
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        if self.path.exists() {
            if let Err(err) = fs::remove_dir_all(&self.path) {
                eprintln!(
                    "TempDirGuard warning: failed to remove {}: {}",
                    self.path.display(),
                    err
                );
            }
        }
    }
}

pub(crate) fn extract_markdown_h2_section<'a>(
    markdown: &'a str,
    heading: &str,
) -> Result<&'a str, String> {
    let (_before_heading, after_heading) = markdown
        .split_once(heading)
        .ok_or_else(|| format!("Missing heading {:?} in Markdown:\n{}", heading, markdown))?;
    if let Some((section_body, _remaining_sections)) = after_heading.split_once("\n## ") {
        Ok(section_body)
    } else {
        Ok(after_heading)
    }
}

pub(crate) fn assert_mermaid_blocks_free_of_backticks_and_semicolons(
    doc_name: &str,
    markdown: &str,
) -> Result<(), String> {
    let mut block_count: usize = 0;
    for tail in markdown.split("```mermaid").skip(1) {
        block_count += 1;
        let (mermaid_body, _after_close) = tail.split_once("```").ok_or_else(|| {
            format!(
                "{} contains an unclosed ```mermaid fence:\n{}",
                doc_name, markdown
            )
        })?;
        assert!(
            !mermaid_body.contains('`'),
            "{} Mermaid diagram block must contain zero backticks ('`'):\n```mermaid{}```",
            doc_name,
            mermaid_body
        );
        assert!(
            !mermaid_body.contains(';'),
            "{} Mermaid diagram block must contain zero semicolons (';'):\n```mermaid{}```",
            doc_name,
            mermaid_body
        );
    }
    if block_count == 0 {
        return Err(format!(
            "{} must contain at least one ```mermaid diagram block:\n{}",
            doc_name, markdown
        ));
    }
    Ok(())
}

#[test]
fn test_issue_552_forward_sync_does_not_emit_hardcoded_colliding_issue_ids(
) -> Result<(), Box<dyn std::error::Error>> {
    crate::reproducer_b_tests::run_issue_552_checks()
}

#[test]
fn test_issue_546_resolve_schema_targets_falls_back_to_pipeline_schema(
) -> Result<(), Box<dyn std::error::Error>> {
    crate::reproducer_b_tests::run_issue_546_checks()
}

#[test]
fn test_issue_547_run_compilation_gate_compiles_blueprint_triplet_dir_in_place(
) -> Result<(), Box<dyn std::error::Error>> {
    crate::reproducer_b_tests::run_issue_547_checks()
}

#[test]
fn test_issue_693_fmeca_software_ast_eliminates_hardware_templates_fake_ssot_first_truncation_and_score_fallback(
) -> Result<(), Box<dyn std::error::Error>> {
    crate::reproducer_c_tests::run_issue_693_fmeca_software_ast_checks()
}

#[test]
fn test_issue_693_stpa_transpile_scopes_subworkspace_out_dir_and_ingests_failure_mode_registry(
) -> Result<(), Box<dyn std::error::Error>> {
    crate::reproducer_c_tests::run_issue_693_stpa_transpile_subworkspace_checks()
}

#[test]
fn test_issue_695_non_uav_safety_suite_eliminates_hardcoded_uav_physics_proofs_and_sora_tables(
) -> Result<(), Box<dyn std::error::Error>> {
    crate::reproducer_d_tests::run_issue_695_non_uav_safety_suite_checks()
}

#[test]
fn test_issue_695_safety_suite_derives_rta_proofs_sldv_and_regulatory_objectives_purely_from_ast(
) -> Result<(), Box<dyn std::error::Error>> {
    crate::reproducer_d_tests::run_issue_695_ast_derived_rta_sldv_reg_checks()
}

#[test]
fn test_issue_698_stpa_lowers_sysml_operators_to_katex_and_matlab(
) -> Result<(), Box<dyn std::error::Error>> {
    crate::reproducer_e_tests::run_issue_698_katex_and_matlab_lowering_checks()
}

#[test]
fn test_issue_697_stpa_derives_uca_loss_scenarios_and_hazard_log_from_ast_without_pending_parameter_or_deap_compiler_system(
) -> Result<(), Box<dyn std::error::Error>> {
    crate::reproducer_f_tests::run_issue_697_uca_loss_and_hazard_log_checks()
}

#[test]
fn test_issue_697_upstream_sysml_model_transpiles_safety_suite_with_zero_pending_parameter(
) -> Result<(), Box<dyn std::error::Error>> {
    crate::reproducer_f_tests::run_issue_697_upstream_model_zero_pending_checks()
}

#[test]
fn test_issue_696_forward_sync_user_story_derives_ast_clause_3_layer_bdd_and_zero_compiler_boilerplate(
) -> Result<(), Box<dyn std::error::Error>> {
    crate::reproducer_g_tests::run_issue_696_user_story_ast_clause_checks()
}

#[test]
fn test_issue_696_forward_sync_non_forced_evicts_stale_user_story_boilerplate_preserving_issue_id(
) -> Result<(), Box<dyn std::error::Error>> {
    crate::reproducer_h_tests::run_issue_696_non_forced_boilerplate_eviction_checks()
}

#[test]
fn test_issue_694_forward_sync_projects_single_line_bold_bdd_and_evicts_stale_use_case_boilerplate(
) -> Result<(), Box<dyn std::error::Error>> {
    crate::reproducer_h_tests::run_issue_694_single_line_bold_bdd_checks()
}

#[test]
fn test_issue_694_multiline_bdd_actor_extraction_and_diagnostic_only_use_case_projection(
) -> Result<(), Box<dyn std::error::Error>> {
    crate::reproducer_i_tests::run_issue_694_multiline_bdd_and_diag_checks()
}

#[test]
fn test_issue_699_forward_sync_ast_grounded_features_epics_acronyms_and_merge_eviction(
) -> Result<(), Box<dyn std::error::Error>> {
    crate::reproducer_k_tests::run_issue_699_features_epics_acronyms_and_merge_checks()
}
