//! Checks 25 and 26: Operational-to-Resource Allocation and Standards Parameter Metrology Gates.
//!
//! Safety & Traceability: Realises: [REQ-PARITY-GATE-25], [REQ-PARITY-GATE-26]
//! Contract: Validates `[OA-*]`/`[MET-*]` allocation tags and SI dimensional units in specification tables.
//! Bounds & Faults: Deterministic sorted file walking; fails closed on unreadable files or metrology violations.

use super::parity_check_discovery::TargetScanDir;
use super::parity_gates::ParityOutcome;
use deap_core::rules::EXCLUDED_DIRS;
use deap_core::workspace::is_upstream_compiler;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Collects sorted Markdown files across `scan_dirs` while recording WalkDir I/O errors under `check_label`.
/// Realises: [REQ-PARITY-GATE-25], [REQ-PARITY-GATE-26]
fn collect_sorted_md_files(
    scan_dirs: &[PathBuf],
    check_label: &str,
    errors: &mut Vec<String>,
) -> Vec<PathBuf> {
    let mut target_files = Vec::new();
    for dir in scan_dirs {
        if !dir.is_dir() {
            continue;
        }
        for entry_res in WalkDir::new(dir)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    let name = e.file_name().to_string_lossy();
                    !EXCLUDED_DIRS.iter().any(|ex| name == *ex) && name != "blueprints"
                } else {
                    true
                }
            })
        {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    errors.push(format!(
                        "{} failed: I/O error walking directory '{}': {}",
                        check_label,
                        dir.display(),
                        err
                    ));
                    continue;
                }
            };
            if entry.file_type().is_file()
                && entry.path().extension().is_some_and(|ext| ext == "md")
            {
                target_files.push(entry.path().to_path_buf());
            }
        }
    }
    target_files.sort();
    target_files
}

/// Check 25: Operational-to-Resource Allocation Gate (Gate 24).
/// Realises: [REQ-PARITY-GATE-25]
pub fn check_operational_allocation(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    let is_upstream = is_upstream_compiler(repo_root);
    let conops_dir = TargetScanDir::Conops.resolve(repo_root);
    let arch_dir = TargetScanDir::Architecture.resolve(repo_root);

    if is_upstream && !conops_dir.is_dir() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let mut errors = Vec::new();
    let scan_dirs = if is_upstream {
        vec![conops_dir]
    } else {
        vec![conops_dir, arch_dir]
    };
    let target_files = collect_sorted_md_files(&scan_dirs, "Check 25", &mut errors);
    if target_files.is_empty() && errors.is_empty() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    for path in target_files {
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(&path)
            .display()
            .to_string();
        match fs::read_to_string(&path) {
            Ok(content) => {
                for f in crate::spec_audit::conops_audit::validate_operational_allocation_tags(
                    &content, &rel,
                ) {
                    errors.push(f.to_string());
                }
            }
            Err(e) => errors.push(format!(
                "Check 25 failed: Unable to read file '{}': {}",
                rel, e
            )),
        }
    }

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Check 26: Standards & SI 7-Dimensional Parameter Metrology Gate (Gate 25).
/// Realises: [REQ-PARITY-GATE-26]
pub fn check_standards_measurement(
    repo_root: &Path,
    allow_missing_specs: bool,
    strict: bool,
) -> Result<ParityOutcome, Vec<String>> {
    crate::cmark_gates::parity::check_standards_measurement_filtered(
        repo_root,
        allow_missing_specs,
        strict,
        None,
    )
}
