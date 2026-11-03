//! Handoff/backlog integrity checks, handover checkpoint detector, and retired script doc gate.
//!
//! Safety & Traceability: Realises: [REQ-CHECK-HANDOFF-BACKLOG-INTEGRITY]
//! Contract: Enforces zero root `HANDOFF.md`, zero `MASTER_HANDOVER_CHECKPOINT*`, and zero retired script doc references.
//! Bounds & Faults: Single-pass directory and file scans; fails closed with sorted diagnostic vectors.

use deap_core::rules::EXCLUDED_DIRS;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Check documentation and backlog integrity (refs #318).
/// Realises: [REQ-CHECK-HANDOFF-BACKLOG-INTEGRITY].
pub fn check_handoff_and_backlog_integrity(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    let root_handoff = repo_root.join("HANDOFF.md");
    if root_handoff.exists() {
        violations.push(
            "Root HANDOFF.md detected: root handoff artifacts are forbidden per commit 69c749c to prevent documentation drift and stale backlog hallucinations."
                .to_string(),
        );
    }

    let solution_doc = repo_root
        .join("docs")
        .join("designs")
        .join("feat-289-290-293-solution.md");
    if solution_doc.is_file() {
        validate_solution_doc(&solution_doc, &mut violations);
    }

    if let Err(errs) = check_no_master_handover_checkpoint(repo_root) {
        violations.extend(errs);
    }
    if let Err(errs) = check_zero_retired_scripts_in_documentation(repo_root) {
        violations.extend(errs);
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Validates `docs/designs/feat-289-290-293-solution.md` against stale attribution and em dash rules.
/// Realises: [REQ-CHECK-HANDOFF-BACKLOG-INTEGRITY]
fn validate_solution_doc(solution_doc: &Path, violations: &mut Vec<String>) {
    match fs::read_to_string(solution_doc) {
        Ok(content) => {
            if !content.contains("318") {
                violations.push(
                    "feat-289-290-293-solution.md must cite Issue #318 resolving stale backlog hallucinations."
                        .to_string(),
                );
            }
            if content.contains("prompt quarantine across all skills") {
                violations.push(
                    "feat-289-290-293-solution.md contains stale attribution error: 402d56c delivered commit message non-closure rule (refs #315), not prompt quarantine."
                        .to_string(),
                );
            }
            if content.contains("Work Package B: Issue #293")
                || content.contains("Work Package C: Issue #289")
                || content.contains("Work Package D: Issue #290")
            {
                violations.push(
                    "feat-289-290-293-solution.md lists resolved issues #289, #290, or #293 as outstanding work packages."
                        .to_string(),
                );
            }
            if content.contains("scripts/bootstrap_tracker_labels.py")
                && !content
                    .contains("skills/spec-orchestrator/scripts/bootstrap_tracker_labels.py")
            {
                violations.push(
                    "feat-289-290-293-solution.md contains invalid path scripts/bootstrap_tracker_labels.py."
                        .to_string(),
                );
            }
            if content.contains('\u{2014}') {
                violations.push(
                    "feat-289-290-293-solution.md contains forbidden Unicode em dash (\\u2014)."
                        .to_string(),
                );
            }
        }
        Err(err) => {
            violations.push(format!(
                "Unable to read {}: {}",
                solution_doc.display(),
                err
            ));
        }
    }
}

/// Check documentation and root artifact cleanliness (refs #321).
/// Realises: [REQ-CHECK-NO-MASTER-HANDOVER-CHECKPOINT].
pub fn check_no_master_handover_checkpoint(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    let root_checkpoint = repo_root.join("MASTER_HANDOVER_CHECKPOINT.md");
    if root_checkpoint.exists() {
        violations.push(
            "Root MASTER_HANDOVER_CHECKPOINT.md detected: ephemeral/spurious handover checkpoint files containing hardcoded domain specifications are strictly forbidden at repository root (refs #321)."
                .to_string(),
        );
    }

    match fs::read_dir(repo_root) {
        Ok(entries) => {
            for entry_res in entries {
                let entry = match entry_res {
                    Ok(e) => e,
                    Err(err) => {
                        violations.push(format!(
                            "Unable to read directory entry in '{}': {}",
                            repo_root.display(),
                            err
                        ));
                        continue;
                    }
                };
                if entry.file_type().is_ok_and(|t| t.is_file()) {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    if fname != "MASTER_HANDOVER_CHECKPOINT.md"
                        && fname
                            .to_ascii_uppercase()
                            .starts_with("MASTER_HANDOVER_CHECKPOINT")
                    {
                        violations.push(format!(
                            "Root checkpoint variant detected: '{}' is strictly forbidden (refs #321).",
                            fname
                        ));
                    }
                }
            }
        }
        Err(err) => {
            violations.push(format!(
                "Unable to read repository root directory '{}': {}",
                repo_root.display(),
                err
            ));
        }
    }

    scan_docs_for_handover_checkpoint(repo_root, &mut violations);

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        violations.dedup();
        Err(violations)
    }
}

/// Scans `docs/` for any `MASTER_HANDOVER_CHECKPOINT*` file.
/// Realises: [REQ-CHECK-NO-MASTER-HANDOVER-CHECKPOINT]
fn scan_docs_for_handover_checkpoint(repo_root: &Path, violations: &mut Vec<String>) {
    let docs_dir = repo_root.join("docs");
    if !docs_dir.is_dir() {
        return;
    }
    for entry_res in WalkDir::new(&docs_dir).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let name = e.file_name().to_string_lossy();
            !EXCLUDED_DIRS.iter().any(|ex| name == *ex)
        } else {
            true
        }
    }) {
        let entry = match entry_res {
            Ok(e) => e,
            Err(err) => {
                violations.push(format!(
                    "Unable to read directory entry under '{}': {}",
                    docs_dir.display(),
                    err
                ));
                continue;
            }
        };
        if entry.file_type().is_file() {
            let fname_upper = entry.file_name().to_string_lossy().to_ascii_uppercase();
            if fname_upper == "MASTER_HANDOVER_CHECKPOINT.MD"
                || fname_upper.starts_with("MASTER_HANDOVER_CHECKPOINT")
            {
                let rel = entry
                    .path()
                    .strip_prefix(repo_root)
                    .unwrap_or(entry.path())
                    .display()
                    .to_string();
                violations.push(format!(
                    "Forbidden handover checkpoint file detected at '{}': ephemeral/spurious handover checkpoints containing domain specifications are strictly forbidden (refs #321).",
                    rel
                ));
            }
        }
    }
}

/// Check documentation for references to retired Python and shell scripts (refs #441).
/// Realises: [REQ-CHECK-ZERO-RETIRED-SCRIPTS-IN-DOCS].
pub fn check_zero_retired_scripts_in_documentation(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    let retired_needles = retired_script_needles();

    for dir in [repo_root.join("docs"), repo_root.join("skills")] {
        if dir.is_dir() {
            scan_doc_dir_for_retired_scripts(repo_root, &dir, &retired_needles, &mut violations);
        }
    }

    for file in [
        repo_root.join("README.md"),
        repo_root.join(".pipeline/constitution.md"),
        repo_root.join(".pipeline/upstream/pipeline-tooling.md"),
    ] {
        if file.is_file() {
            scan_single_file_for_retired_scripts(
                repo_root,
                &file,
                &retired_needles,
                &mut violations,
            );
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        violations.dedup();
        Err(violations)
    }
}

/// Returns the canonical list of retired script tokens forbidden in governed documentation.
/// Realises: [REQ-CHECK-ZERO-RETIRED-SCRIPTS-IN-DOCS]
fn retired_script_needles() -> Vec<String> {
    vec![
        "create_issue.sh".to_string(),
        "verify_downstream_baseline.py".to_string(),
        "install_pipeline.sh".to_string(),
        "generate-wbs-suite".to_string(),
        "compile_yang.py".to_string(),
        "compile_sysml.py".to_string(),
        "assemble_conops.py".to_string(),
        "parity_auditor".to_string(),
        "setup_git_hooks.py".to_string(),
        "file_defect.py".to_string(),
        "verify_commit_messages.py".to_string(),
        "verify_subagent_output.py".to_string(),
        "reconcile_backlog.py".to_string(),
        "spec_validator.py".to_string(),
    ]
}

/// Walks a documentation directory for markdown files referencing retired scripts.
/// Realises: [REQ-CHECK-ZERO-RETIRED-SCRIPTS-IN-DOCS]
fn scan_doc_dir_for_retired_scripts(
    repo_root: &Path,
    dir: &Path,
    retired_needles: &[String],
    violations: &mut Vec<String>,
) {
    for entry_res in WalkDir::new(dir).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let name = e.file_name().to_string_lossy();
            !EXCLUDED_DIRS.iter().any(|ex| name == *ex)
                && !matches!(
                    name.as_ref(),
                    "designs" | "audits" | "blueprints" | "decisions"
                )
        } else {
            true
        }
    }) {
        let entry = match entry_res {
            Ok(e) => e,
            Err(err) => {
                violations.push(format!(
                    "Unable to read directory entry under '{}': {}",
                    dir.display(),
                    err
                ));
                continue;
            }
        };
        let path = entry.path();
        if entry.file_type().is_file() && path.extension().is_some_and(|ext| ext == "md") {
            let rel = path
                .strip_prefix(repo_root)
                .unwrap_or(path)
                .display()
                .to_string();
            if rel.ends_with("_HANDOFF.md") || rel.ends_with("_HANDOVER.md") {
                continue;
            }
            scan_single_file_for_retired_scripts(repo_root, path, retired_needles, violations);
        }
    }
}

/// Scans a single documentation file for retired script tokens.
/// Realises: [REQ-CHECK-ZERO-RETIRED-SCRIPTS-IN-DOCS]
fn scan_single_file_for_retired_scripts(
    repo_root: &Path,
    file: &Path,
    retired_needles: &[String],
    violations: &mut Vec<String>,
) {
    let rel = file
        .strip_prefix(repo_root)
        .unwrap_or(file)
        .display()
        .to_string();
    match fs::read_to_string(file) {
        Ok(content) => {
            for (idx, line) in content.lines().enumerate() {
                for needle in retired_needles {
                    if line.contains(needle) {
                        violations.push(format!(
                            "{}:{}: references retired script '{}'",
                            rel,
                            idx.saturating_add(1),
                            needle
                        ));
                    }
                }
            }
        }
        Err(err) => {
            violations.push(format!(
                "{}: unable to read documentation file: {}",
                rel, err
            ));
        }
    }
}
