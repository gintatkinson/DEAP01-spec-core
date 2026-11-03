//! Check 17: Safety Integrity, STPA 8-Pillars, and Regulatory Objectives Gate.
//! Realises: [REQ-CHECKS-SAFETY-STPA]

use std::fs;
use std::path::{Path, PathBuf};

use deap_core::rules::EXCLUDED_DIRS;
use deap_core::workspace::is_upstream_compiler;
use walkdir::WalkDir;

pub use crate::checks::safety_fmeca::{parse_fmeca_table, FmecaData, FmecaRow};
pub use crate::checks::safety_pillars::{
    check_failure_dimension_coverage, check_uca_categories,
};
use crate::checks::safety_pillars::audit_stpa_pillars_1_to_8;

/// Outcome of Check 17 safety integrity verification.
/// Realises: [REQ-CHECKS-SAFETY-STPA-SAFETYCHECKOUTCOME]
#[derive(Debug, PartialEq, Eq)]
pub enum SafetyCheckOutcome {
    /// Upstream compiler repository clean.
    /// Realises: [REQ-CHECKS-SAFETY-STPA]
    UpstreamClean,
    /// Downstream safety specs allowed missing or clean.
    /// Realises: [REQ-CHECKS-SAFETY-STPA]
    DownstreamPendingOrClean,
    /// Downstream safety specs verified against all 8 STPA pillars.
    /// Realises: [REQ-CHECKS-SAFETY-STPA]
    DownstreamVerified,
}

/// Check 17: Safety Integrity Quality Gate and Regulatory Objectives Completeness Verification.
/// Realises: [REQ-CHECKS-SAFETY-STPA-CHECK-SAFETY-INTEGRITY]
pub fn check_safety_integrity(
    repo_root: &Path,
    allow_missing_specs: bool,
    strict: bool,
) -> Result<SafetyCheckOutcome, Vec<String>> {
    let safety_dir = repo_root.join("docs").join("safety");
    if is_upstream_compiler(repo_root) {
        return check_upstream_safety_landing_zone(repo_root, &safety_dir);
    }

    let effective_allow_missing = allow_missing_specs && !strict;
    if !safety_dir.is_dir() {
        return if effective_allow_missing {
            Ok(SafetyCheckOutcome::DownstreamPendingOrClean)
        } else {
            Err(vec![
                "Check 17 failed: Safety specification directory 'docs/safety/' is missing."
                    .to_string(),
            ])
        };
    }

    let safety_files = collect_downstream_safety_files(&safety_dir)?;
    if safety_files.is_empty() {
        return if effective_allow_missing {
            Ok(SafetyCheckOutcome::DownstreamPendingOrClean)
        } else {
            Err(vec!["Check 17 failed: No safety specifications found in 'docs/safety/'.".to_string()])
        };
    }

    let stpa_matrix_file = safety_dir.join("STPA_MATRIX.md");
    if !stpa_matrix_file.is_file() {
        return Err(vec![
            "Check 17 failed: Unified STPA matrix 'docs/safety/STPA_MATRIX.md' is missing."
                .to_string(),
        ]);
    }
    let stpa_content = fs::read_to_string(&stpa_matrix_file).map_err(|e| {
        vec![format!("Check 17 failed: Failed to read {}: {}", stpa_matrix_file.display(), e)]
    })?;

    let mut combined_content = String::new();
    let mut errors = Vec::new();
    let slicer = crate::checks::parity_gates::SectionSlicer::new(repo_root);
    let model_str =
        crate::checks::parity_gates::discover_sysml_model_text(repo_root).unwrap_or_default();

    for sf in &safety_files {
        let c = fs::read_to_string(sf).map_err(|e| {
            vec![format!("Check 17 failed: Failed to read {}: {}", sf.display(), e)]
        })?;
        combined_content.push_str(&c);
        combined_content.push_str("\n\n---\n\n");
        let rel_path = sf.strip_prefix(repo_root).unwrap_or(sf).to_string_lossy().to_string();
        crate::checks::parity_gates::audit_spec_citations_and_protocols(
            &c, &rel_path, &slicer, &model_str, &mut errors,
        );
    }

    audit_stpa_pillars_1_to_8(repo_root, &stpa_content, &combined_content, &mut errors);

    if errors.is_empty() {
        Ok(SafetyCheckOutcome::DownstreamVerified)
    } else {
        Err(errors)
    }
}

/// Verifies that the upstream compiler `docs/safety/` landing zone contains only `.gitkeep` / `README.md`.
/// Realises: [REQ-CHECKS-SAFETY-STPA-CHECK-SAFETY-INTEGRITY]
fn check_upstream_safety_landing_zone(
    repo_root: &Path,
    safety_dir: &Path,
) -> Result<SafetyCheckOutcome, Vec<String>> {
    if !safety_dir.is_dir() {
        return Ok(SafetyCheckOutcome::UpstreamClean);
    }
    let allowed_files = [".gitkeep", "README.md"];
    let mut violations = Vec::new();
    let mut walk_errors = Vec::new();
    for entry_res in WalkDir::new(safety_dir).into_iter().filter_entry(|e| {
        !e.file_type().is_dir() || !EXCLUDED_DIRS.contains(e.file_name().to_string_lossy().as_ref())
    }) {
        match entry_res {
            Ok(entry) if entry.file_type().is_file() => {
                let fname = entry.file_name().to_string_lossy();
                if !allowed_files.contains(&fname.as_ref()) {
                    let rel = entry.path().strip_prefix(repo_root).unwrap_or(entry.path());
                    violations.push(rel.display().to_string());
                }
            }
            Ok(_) => {}
            Err(err) => {
                walk_errors.push(format!("Check 17 failed: Safety directory walk I/O error: {}", err));
            }
        }
    }
    if !violations.is_empty() {
        walk_errors.push(format!(
            "Check 17 failed: Upstream distribution template safety landing zone contains concrete specification files: {}",
            violations.join(", ")
        ));
    }
    if walk_errors.is_empty() {
        Ok(SafetyCheckOutcome::UpstreamClean)
    } else {
        Err(walk_errors)
    }
}

/// Collects `.md` safety specification files under `docs/safety/` in downstream workspaces.
/// Realises: [REQ-CHECKS-SAFETY-STPA-CHECK-SAFETY-INTEGRITY]
fn collect_downstream_safety_files(safety_dir: &Path) -> Result<Vec<PathBuf>, Vec<String>> {
    let mut safety_files = Vec::new();
    let mut walk_errors = Vec::new();
    for entry_res in WalkDir::new(safety_dir).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let name = e.file_name().to_string_lossy();
            !EXCLUDED_DIRS.iter().any(|ex| name == *ex)
                && !matches!(name.as_ref(), "defects" | "audits" | "decisions")
        } else {
            true
        }
    }) {
        match entry_res {
            Ok(entry) if entry.file_type().is_file() => {
                let fname = entry.file_name().to_string_lossy();
                if fname.ends_with(".md") && fname != "README.md" {
                    safety_files.push(entry.path().to_path_buf());
                }
            }
            Ok(_) => {}
            Err(err) => {
                walk_errors.push(format!("Check 17 failed: Safety directory walk I/O error: {}", err));
            }
        }
    }
    if walk_errors.is_empty() {
        Ok(safety_files)
    } else {
        Err(walk_errors)
    }
}

#[cfg(test)]
#[path = "safety_tests_a.rs"]
pub(crate) mod safety_tests_a;
#[cfg(test)]
#[path = "safety_tests_b.rs"]
mod safety_tests_b;
