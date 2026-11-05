//! Structural repository topology, compiler AST source purity (Check 34), and governance integrity gates.
//!
//! Safety & Traceability: Realises: [REQ-CHECK-34-STRUCTURAL-GATES], [DO-330-TQL-4-STRUCTURAL-PURITY]
//! Contract: Enforces deterministic workspace topology, script/handoff cleanliness, manifest lint gates, and tiered Check 34 compiler AST purity invariants across audited Rust sources.
//! Bounds & Faults: O(F * L) bounded file and line traversal with non-panicking slice inspection and fail-closed `Result<(), Vec<String>>` error reporting.

use deap_core::workspace::{
    check_downstream_instructions_exist as core_check_downstream_instructions_exist,
    check_gitignore_exists as core_check_gitignore_exists,
    check_no_ds_store_files as core_check_no_ds_store_files,
    check_no_duplicate_master_blueprints as core_check_no_duplicate_master_blueprints,
    check_reconcile_backlog_tooling_exists as core_check_reconcile_backlog_tooling_exists,
    check_upstream_template_clean_landing_zones as core_check_upstream_template_clean_landing_zones,
    is_upstream_compiler,
};
use std::path::{Path, PathBuf};

pub use super::domain_cleanliness_gate::check_domain_agnostic_ast_cleanliness;
pub use super::struct_check_audit::audit_source_purity;
pub use super::struct_check_clippy::{
    scan_clippy_allow_tokens, ClippySuppressionViolation, RequiredClippyForbidLint,
    WorkspaceManifestForbidGate,
};
pub use super::struct_check_feat42::{
    check_compiler_ast_purity, check_feat42_design_blueprint_compliance,
};
pub use super::struct_check_grounding::check_blueprint_grounding;
pub use super::struct_check_handoff::{
    check_handoff_and_backlog_integrity, check_no_master_handover_checkpoint,
    check_zero_retired_scripts_in_documentation,
};
pub use super::struct_check_purity::check_zero_regex;
#[cfg(test)]
pub(crate) use super::struct_check_purity::{
    contains_bounded_split_token, enforces_strict_io_arms, has_split_token, is_audited_source_path,
    AUDITED_MODULES,
};
pub use super::struct_check_scripts::check_zero_python_scripts;
#[cfg(test)]
pub(crate) use super::struct_check_scripts::is_governed_python_scan_extension;
pub use super::struct_check_upstream::{
    check_zero_shell_scripts, verify_upstream_blueprint_domain_cleanliness,
};

/// Resolves the enclosing upstream compiler root when `repo_root` points to `<root>/.pipeline/upstream`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C configuration management controls.
/// Enables self-hosted dogfood verification of `<root>/.pipeline/upstream` against shared root-level
/// governance and tooling artifacts without weakening fail-closed checks on standalone downstream workspaces.
///
/// ## 2. Invariants & Mathematical Properties
/// Returns `Some(parent_repo)` if and only if `repo_root` has terminal path components `.pipeline/upstream`
/// and `is_upstream_compiler(&parent_repo)` evaluates to `true`. All standalone downstream workspaces
/// evaluate deterministically to `None`.
///
/// ## 3. Preconditions
/// `repo_root` is a valid filesystem path reference.
///
/// ## 4. Postconditions
/// Returns `Some(PathBuf)` pointing to the verified enclosing upstream compiler root, or `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1) bounded path component inspection. Space Complexity: O(P) where P is path length.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Pure path inspection plus `is_upstream_compiler` directory check; returns `None` on any mismatch.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-UPSTREAM-DOGFOOD-ROOT-RESOLUTION].
pub fn enclosing_upstream_compiler_root(repo_root: &Path) -> Option<PathBuf> {
    if repo_root.file_name().and_then(|n| n.to_str()) != Some("upstream") {
        return None;
    }
    let pipeline_dir = repo_root.parent()?;
    if pipeline_dir.file_name().and_then(|n| n.to_str()) != Some(".pipeline") {
        return None;
    }
    let raw_parent = pipeline_dir.parent()?;
    let parent_repo = if raw_parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        raw_parent
    };
    if is_upstream_compiler(parent_repo) {
        Some(parent_repo.to_path_buf())
    } else {
        None
    }
}

/// Check 10: Verify .gitignore exists in repository root.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C configuration management controls.
/// Ensures `.gitignore` is present at the repository root (or enclosing upstream compiler root for `<root>/.pipeline/upstream`).
///
/// ## 2. Invariants & Mathematical Properties
/// `repo_root.join(".gitignore")` (or `parent_repo.join(".gitignore")` for verified `<root>/.pipeline/upstream`) must resolve to an existing regular file.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` if `.gitignore` exists, or `Err(String)` describing the violation.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1). Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed if `.gitignore` is missing or inaccessible.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-10-GITIGNORE-EXISTS].
pub fn check_gitignore_exists(repo_root: &Path) -> Result<(), String> {
    match core_check_gitignore_exists(repo_root) {
        Ok(()) => Ok(()),
        Err(err) => {
            if let Some(parent_repo) = enclosing_upstream_compiler_root(repo_root) {
                core_check_gitignore_exists(&parent_repo).map_err(String::from)
            } else {
                Err(err.into_string())
            }
        }
    }
}

/// Check 11: Verify zero .DS_Store files exist in the working tree.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C workspace hygiene controls.
/// Ensures zero macOS `.DS_Store` metadata artifacts pollute the repository tree.
///
/// ## 2. Invariants & Mathematical Properties
/// Every non-excluded file entry in `repo_root` has filename != `.DS_Store`.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` when zero `.DS_Store` files exist, or `Err(Vec<String>)` listing violations.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) where N is directory entry count. Space Complexity: O(V) where V is violation count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Delegates to `deap_core::workspace::check_no_ds_store_files` which fails closed on I/O errors.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-11-NO-DS-STORE].
pub fn check_no_ds_store_files(repo_root: &Path) -> Result<(), Vec<String>> {
    core_check_no_ds_store_files(repo_root)
}

/// Check 12: Verify downstream repositories do not contain duplicate master core blueprints.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C single-source-of-truth (SSOT) architecture controls.
/// Prevents downstream repositories from duplicating upstream master core blueprints.
///
/// ## 2. Invariants & Mathematical Properties
/// Downstream repositories must contain zero duplicate master blueprint files.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` on compliance, or `Err(Vec<String>)` listing duplicate blueprint paths.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(B) where B is number of master blueprints. Space Complexity: O(V).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed if forbidden duplicate blueprints exist in downstream workspaces.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-12-NO-DUPLICATE-MASTER-BLUEPRINTS].
pub fn check_no_duplicate_master_blueprints(repo_root: &Path) -> Result<(), Vec<String>> {
    core_check_no_duplicate_master_blueprints(repo_root)
}

/// Check 14: Verify presence of README.md, agent instruction entrypoints, and rules/sysml-ssot-completeness.md.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C operational governance controls.
/// Verifies all mandatory instruction entrypoints and SSOT completeness rules exist.
///
/// ## 2. Invariants & Mathematical Properties
/// Required governance files must exist and be non-empty.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` when all instruction entrypoints exist, or `Err(Vec<String>)` on missing files.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1). Space Complexity: O(V).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed when any required instruction file is absent.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-14-DOWNSTREAM-INSTRUCTIONS].
pub fn check_downstream_instructions_exist(repo_root: &Path) -> Result<(), Vec<String>> {
    match core_check_downstream_instructions_exist(repo_root) {
        Ok(()) => Ok(()),
        Err(errs) => {
            if let Some(parent_repo) = enclosing_upstream_compiler_root(repo_root) {
                core_check_downstream_instructions_exist(&parent_repo)
            } else {
                Err(errs)
            }
        }
    }
}

/// Check 15: Verify reconcile-backlog tooling exists and is executable/buildable.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C traceability tooling controls.
/// Ensures backlog reconciliation tooling is present in the workspace (or enclosing upstream compiler root for `<root>/.pipeline/upstream`).
///
/// ## 2. Invariants & Mathematical Properties
/// Either compiled `reconcile-backlog` binary or its Rust crate manifest must exist.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` if tooling is present, or `Err(Vec<String>)` if missing.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1). Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed if neither binary nor crate manifest is found.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-15-RECONCILE-BACKLOG-TOOLING].
pub fn check_reconcile_backlog_tooling_exists(repo_root: &Path) -> Result<(), Vec<String>> {
    match core_check_reconcile_backlog_tooling_exists(repo_root) {
        Ok(()) => Ok(()),
        Err(errs) => {
            if let Some(parent_repo) = enclosing_upstream_compiler_root(repo_root) {
                core_check_reconcile_backlog_tooling_exists(&parent_repo)
            } else {
                Err(errs)
            }
        }
    }
}

/// Check 16: Upstream Template Clean Landing Zone Gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C M2/M1 separation controls.
/// Ensures upstream template landing zones contain only `.gitkeep` sentinel files.
///
/// ## 2. Invariants & Mathematical Properties
/// In upstream repositories, landing zone directories contain zero non-`.gitkeep` files.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` on clean landing zones, or `Err(Vec<String>)` listing contaminating files.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) where N is files in landing zones. Space Complexity: O(V).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on any non-`.gitkeep` file or directory traversal error.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-16-CLEAN-LANDING-ZONES].
pub fn check_upstream_template_clean_landing_zones(repo_root: &Path) -> Result<(), Vec<String>> {
    core_check_upstream_template_clean_landing_zones(repo_root)
}
