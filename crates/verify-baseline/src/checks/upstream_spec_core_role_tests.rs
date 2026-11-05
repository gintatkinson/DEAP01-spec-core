//! Regression tests for upstream spec-core role gating (refs #863, #864).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A and ECSS-E-ST-40C workspace role verification controls.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Upstream compiler workspaces retain all compiler-only structure gates.
//! - Upstream spec-core workspaces pass compiler-only checks without Tier-0 source.
//!
//! ## 3. Preconditions
//! - Isolated temporary directories with deterministic cleanup.
//!
//! ## 4. Postconditions
//! - Every fixture is removed on drop and assertions are deterministic.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - O(N) over the bounded fixture file set.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Returns Result<(), Box<dyn std::error::Error>> without unwrap or panic.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0601], [REQ-CHECK-37-BLUEPRINT-GROUNDING]

use super::domain_cleanliness_gate::check_domain_agnostic_ast_cleanliness;
use super::parity_check_cross_doc::check_cross_document_diagram_parity;
use super::parity_check_obligation::check_architecture_viewpoint_diagrams;
use super::parity_gates::ParityOutcome;
use super::struct_check_grounding::check_blueprint_grounding;
use deap_core::rules::CANONICAL_ARCHITECTURE_BLUEPRINTS;
use std::fs;
use std::path::{Path, PathBuf};

struct TempRepo {
    path: PathBuf,
}

impl TempRepo {
    fn new(suffix: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = std::env::temp_dir().join(format!(
            "deap_spec_core_role_{}_{}",
            std::process::id(),
            suffix
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(path.join(".pipeline").join("upstream"))?;
        Ok(Self { path })
    }

    fn write(&self, rel: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.path.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_canonical_blueprints(repo: &TempRepo) -> Result<(), Box<dyn std::error::Error>> {
    for name in CANONICAL_ARCHITECTURE_BLUEPRINTS.iter() {
        repo.write(&format!("docs/architecture/blueprints/{name}"), "# Master\n")?;
    }
    Ok(())
}

fn write_design_blueprint(
    repo: &TempRepo,
    rel: &str,
    status: &str,
    target: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    repo.write(
        rel,
        &format!("| **Status** | `{status}` |\n\n- Test target: `{target}`\n"),
    )
}

#[test]
fn test_upstream_spec_core_passes_compiler_only_checks(
) -> Result<(), Box<dyn std::error::Error>> {
    let repo = TempRepo::new("spec_core")?;
    write_canonical_blueprints(&repo)?;
    write_design_blueprint(
        &repo,
        "docs/designs/blueprint-01-spec-core.md",
        "Approved",
        "crates/verify-baseline/tests/future_test.rs",
    )?;
    assert!(check_domain_agnostic_ast_cleanliness(repo.path()).is_ok());
    assert_eq!(
        check_cross_document_diagram_parity(repo.path()).map_err(|errors| errors.join("; "))?,
        ParityOutcome::Verified
    );
    assert_eq!(
        check_architecture_viewpoint_diagrams(repo.path(), false, false)
            .map_err(|errors| errors.join("; "))?,
        ParityOutcome::Verified
    );
    assert!(check_blueprint_grounding(repo.path()).is_ok());
    Ok(())
}

#[test]
fn test_upstream_compiler_workspace_retains_structure_gates(
) -> Result<(), Box<dyn std::error::Error>> {
    let repo = TempRepo::new("compiler_workspace")?;
    repo.write("Cargo.toml", "[workspace]\n")?;
    repo.write("crates/deap-core/src/lib.rs", "")?;
    write_canonical_blueprints(&repo)?;
    write_design_blueprint(
        &repo,
        "docs/designs/blueprint-01-compiler.md",
        "Approved",
        "crates/deap-core/src/lib.rs",
    )?;
    assert!(check_domain_agnostic_ast_cleanliness(repo.path()).is_ok());
    assert!(check_cross_document_diagram_parity(repo.path()).is_err());
    assert!(check_architecture_viewpoint_diagrams(repo.path(), false, false).is_err());
    assert!(check_blueprint_grounding(repo.path()).is_ok());
    repo.write("docs/architecture/views/sv1_super_system.mmd", "graph TD\n")?;
    assert_eq!(
        check_cross_document_diagram_parity(repo.path()).map_err(|errors| errors.join("; "))?,
        ParityOutcome::Verified
    );
    assert_eq!(
        check_architecture_viewpoint_diagrams(repo.path(), false, false)
            .map_err(|errors| errors.join("; "))?,
        ParityOutcome::Verified
    );
    Ok(())
}

#[test]
fn test_upstream_compiler_workspace_missing_crates_fails_closed(
) -> Result<(), Box<dyn std::error::Error>> {
    let repo = TempRepo::new("compiler_workspace_missing_crates")?;
    repo.write("Cargo.toml", "[workspace]\n")?;
    write_canonical_blueprints(&repo)?;
    assert!(check_domain_agnostic_ast_cleanliness(repo.path()).is_err());
    Ok(())
}

#[test]
fn test_upstream_compiler_workspace_approved_blueprint_rejects_missing_path(
) -> Result<(), Box<dyn std::error::Error>> {
    let repo = TempRepo::new("compiler_workspace_grounding")?;
    repo.write("Cargo.toml", "[workspace]\n")?;
    repo.write("crates/deap-core/src/lib.rs", "")?;
    write_canonical_blueprints(&repo)?;
    write_design_blueprint(
        &repo,
        "docs/designs/blueprint-01-compiler.md",
        "Approved",
        "crates/deap-core/tests/missing_test.rs",
    )?;
    assert!(check_blueprint_grounding(repo.path()).is_err());
    Ok(())
}
