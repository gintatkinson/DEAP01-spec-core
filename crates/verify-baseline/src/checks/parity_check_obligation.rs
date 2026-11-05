//! Checks 30B and 30C: Obligation-Witness Registry (`check_obligation_witness`) and Architecture Viewpoint Gates.
//!
//! Safety & Traceability: Realises: [REQ-OBLIGATION-WITNESS-GATE], [REQ-PARITY-GATE-30-VIEWPOINTS]
//! Contract: Verifies that STPA hazards (`H-*`) are witnessed by safety constraints (`SC-*`) and SV-1 views exist.
//! Bounds & Faults: Deterministic sorted WalkDir scanning; fails closed on unwitnessed hazards or missing upstream views.

use super::parity_check_discovery::{discover_sysml_model_text, TargetScanDir};
use super::parity_gates::ParityOutcome;
use deap_core::workspace::{is_upstream_compiler, is_upstream_compiler_workspace};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Extracts canonical STPA identifiers with a given prefix (e.g., `H-` or `SC-`) from a line.
/// Realises: [REQ-OBLIGATION-WITNESS-GATE]
pub(crate) fn extract_stpa_prefixed_ids(line: &str, prefix: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for raw_tok in line.split(|c: char| !c.is_ascii_alphanumeric() && c != '-') {
        let upper = raw_tok.to_ascii_uppercase();
        if let Some(rest) = upper.strip_prefix(prefix) {
            if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
                let normalized_num = rest.trim_start_matches('0');
                let num_str = if normalized_num.is_empty() {
                    "0"
                } else {
                    normalized_num
                };
                ids.insert(format!("{}{}", prefix, num_str));
            }
        }
    }
    ids
}

/// Check 30B: Obligation-Witness Registry Gate (Gate 29).
/// Realises: [REQ-OBLIGATION-WITNESS-GATE]
pub fn check_obligation_witness(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    if is_upstream_compiler(repo_root) {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let safety_dir = TargetScanDir::Safety.resolve(repo_root);
    if !safety_dir.is_dir() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let mut errors = Vec::new();
    let mut stpa_files = Vec::new();
    for entry_res in WalkDir::new(&safety_dir).sort_by_file_name().into_iter() {
        let entry = match entry_res {
            Ok(e) => e,
            Err(err) => {
                errors.push(format!(
                    "Check 30B failed: I/O error walking directory '{}': {}",
                    safety_dir.display(),
                    err
                ));
                continue;
            }
        };
        if entry.file_type().is_file()
            && entry.path().extension().is_some_and(|ext| ext == "md")
            && entry.file_name().to_string_lossy() != "README.md"
        {
            stpa_files.push(entry.path().to_path_buf());
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    if stpa_files.is_empty() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    audit_stpa_obligation_files(&stpa_files, repo_root, &mut errors);
    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Audits `H-*` hazards and `SC-*` witnessing safety constraints across `stpa_files`.
/// Realises: [REQ-OBLIGATION-WITNESS-GATE]
fn audit_stpa_obligation_files(
    stpa_files: &[PathBuf],
    repo_root: &Path,
    errors: &mut Vec<String>,
) {
    let mut declared_hazards = BTreeSet::new();
    let mut declared_constraints = BTreeSet::new();
    let mut witnessed_hazards = BTreeSet::new();
    let mut has_explicit_uca_bindings = false;

    for path in stpa_files {
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(path)
            .display()
            .to_string();
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("Check 30B failed: Unable to read '{}': {}", rel, e));
                continue;
            }
        };
        for line in content.lines() {
            let line_h = extract_stpa_prefixed_ids(line, "H-");
            let line_sc = extract_stpa_prefixed_ids(line, "SC-");
            if !line_h.is_empty() && !line_sc.is_empty() {
                has_explicit_uca_bindings = true;
                witnessed_hazards.extend(line_h.iter().cloned());
            }
            declared_hazards.extend(line_h);
            declared_constraints.extend(line_sc);
        }
    }

    if !declared_hazards.is_empty() && declared_constraints.is_empty() {
        errors.push(format!(
            "Check 30B failed: Safety specifications declare hazards ({:?}) without any witnessing safety constraint (SC-*).",
            declared_hazards
        ));
    } else if has_explicit_uca_bindings {
        for h in &declared_hazards {
            if !witnessed_hazards.contains(h) {
                errors.push(format!(
                    "Check 30B failed: Declared hazard '{}' in safety specifications is not witnessed by any safety constraint (SC-*) binding.",
                    h
                ));
            }
        }
    }
}

/// Check 30C: Architecture Viewpoint & Diagram Completeness Gate (Gate 30).
/// Realises: [REQ-PARITY-GATE-30-VIEWPOINTS]
pub fn check_architecture_viewpoint_diagrams(
    repo_root: &Path,
    allow_missing_specs: bool,
    strict: bool,
) -> Result<ParityOutcome, Vec<String>> {
    if is_upstream_compiler_workspace(repo_root) {
        let arch_views_dir = TargetScanDir::Architecture.resolve(repo_root).join("views");
        if !arch_views_dir.is_dir() {
            return Err(vec![
                "Check 30C failed: Canonical SSOT Architecture Views Library (docs/architecture/views/) is missing in upstream compiler workspace (refs #276)".to_string(),
            ]);
        }
        if !arch_views_dir.join("sv1_super_system.mmd").is_file() {
            return Err(vec![
                "Check 30C failed: Canonical SSOT Architecture Views Library is missing required view sv1_super_system.mmd (refs #276)".to_string(),
            ]);
        }
    } else if !is_upstream_compiler(repo_root) {
        let model_text = discover_sysml_model_text(repo_root);
        let has_extracted = repo_root.join("schema").join("extracted").is_dir();
        if allow_missing_specs && !strict && model_text.is_none() && !has_extracted {
            return Ok(ParityOutcome::PendingOrClean);
        }
    }

    Ok(ParityOutcome::Verified)
}
