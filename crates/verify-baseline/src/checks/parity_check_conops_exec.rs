//! Checks 28, 29, and 30: ConOps Completeness, Research Inventory, and Executive Deliverable Traceability Gates.
//!
//! Safety & Traceability: Realises: [REQ-PARITY-GATE-28], [REQ-PARITY-GATE-29], [REQ-PARITY-GATE-30]
//! Contract: Validates ConOps schema/markdown contracts, bibliography source references, and executive report connections.
//! Bounds & Faults: Deterministic sorted file walking; fails closed on malformed schemas, I/O errors, or ungrounded claims.

use super::parity_check_cross_doc::extract_connection_ids;
use super::parity_check_discovery::{discover_sysml_model_text, TargetScanDir};
use super::parity_check_icd_ast::is_metrology_placeholder;
use super::parity_gates::ParityOutcome;
use deap_core::workspace::is_upstream_compiler;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Check 28: ConOps & Mission Intent Completeness Gate (Gate 26).
/// Realises: [REQ-PARITY-GATE-28]
pub fn check_conops_and_mission_intent_completeness(
    repo_root: &Path,
    allow_missing_specs: bool,
    strict: bool,
) -> Result<ParityOutcome, Vec<String>> {
    let schema_file = repo_root
        .join(".pipeline")
        .join("schemas")
        .join("conops_specification_schema.json");
    if schema_file.is_file() {
        match crate::spec_audit::conops_audit::validate_conops_schema_file(&schema_file) {
            Ok(schema_findings) if !schema_findings.is_empty() => {
                return Err(schema_findings.into_iter().map(|f| f.to_string()).collect());
            }
            Err(e) => {
                return Err(vec![format!(
                    "Check 28 failed: Unable to parse or validate schema '{}': {}",
                    schema_file.display(),
                    e
                )]);
            }
            Ok(_) => {}
        }
    }

    if is_upstream_compiler(repo_root) {
        return Ok(ParityOutcome::Verified);
    }

    let conops_dir = repo_root.join("docs").join("conops");
    if !conops_dir.is_dir() {
        return Ok(ParityOutcome::ConopsDirMissing);
    }

    let conops_files = collect_conops_files(&conops_dir)?;
    if conops_files.is_empty() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let model_text = discover_sysml_model_text(repo_root);
    let has_extracted = repo_root.join("schema").join("extracted").is_dir();
    let effective_allow_missing = allow_missing_specs && !strict;

    if effective_allow_missing && model_text.is_none() && !has_extracted {
        return Ok(ParityOutcome::PendingOrClean);
    }

    for path in &conops_files {
        validate_single_conops_file(path, repo_root, strict || !effective_allow_missing)?;
    }

    Ok(ParityOutcome::Verified)
}

/// Collects non-template, non-README Markdown files in `conops_dir`.
/// Realises: [REQ-PARITY-GATE-28]
fn collect_conops_files(conops_dir: &Path) -> Result<Vec<PathBuf>, Vec<String>> {
    let mut conops_files = Vec::new();
    let entries = fs::read_dir(conops_dir).map_err(|e| {
        vec![format!(
            "Check 28 failed: Unable to read conops directory '{}': {}",
            conops_dir.display(),
            e
        )]
    })?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".md")
            && name != "README.md"
            && !name.to_ascii_uppercase().contains("TEMPLATE")
        {
            conops_files.push(entry.path());
        }
    }
    conops_files.sort();
    Ok(conops_files)
}

/// Validates a single ConOps Markdown file against `conops_audit::validate_conops`.
/// Realises: [REQ-PARITY-GATE-28]
fn validate_single_conops_file(
    path: &Path,
    repo_root: &Path,
    enforce_findings: bool,
) -> Result<(), Vec<String>> {
    let content = fs::read_to_string(path).map_err(|e| {
        vec![format!(
            "Check 28 failed: Unable to read ConOps file '{}': {}",
            path.display(),
            e
        )]
    })?;
    let doc = crate::spec_audit::markdown_ast::parse_markdown(&content).map_err(|e| {
        vec![format!(
            "Check 28 failed: Unable to parse markdown in '{}': {}",
            path.display(),
            e
        )]
    })?;
    let rel = path
        .strip_prefix(repo_root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();
    let findings =
        crate::spec_audit::conops_audit::validate_conops(&doc, &content, &rel).map_err(|e| {
            vec![format!(
                "Check 28 failed: Validation error in '{}': {}",
                rel, e
            )]
        })?;
    if !findings.is_empty() && enforce_findings {
        return Err(findings.into_iter().map(|f| f.to_string()).collect());
    }
    Ok(())
}

/// Check 29: Cited Research Inventory & Declared-Total Population Register Gate (Gate 27).
/// Realises: [REQ-PARITY-GATE-29]
pub fn check_research_inventory(
    repo_root: &Path,
    allow_missing_specs: bool,
    strict: bool,
) -> Result<ParityOutcome, Vec<String>> {
    if is_upstream_compiler(repo_root) {
        return Ok(ParityOutcome::Verified);
    }

    let model_text = discover_sysml_model_text(repo_root);
    let has_extracted = repo_root.join("schema").join("extracted").is_dir();
    if allow_missing_specs && !strict && model_text.is_none() && !has_extracted {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let mut errors = Vec::new();
    let mut spec_files = Vec::new();
    let scan_dirs = [
        TargetScanDir::Epics.resolve(repo_root),
        TargetScanDir::Features.resolve(repo_root),
        TargetScanDir::Reports.resolve(repo_root),
    ];

    for dir in &scan_dirs {
        if !dir.is_dir() {
            continue;
        }
        for entry_res in WalkDir::new(dir).sort_by_file_name().into_iter() {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    errors.push(format!(
                        "Check 29 failed: I/O error walking directory '{}': {}",
                        dir.display(),
                        err
                    ));
                    continue;
                }
            };
            if entry.file_type().is_file()
                && entry.path().extension().is_some_and(|ext| ext == "md")
            {
                spec_files.push(entry.path().to_path_buf());
            }
        }
    }
    spec_files.sort();

    if spec_files.is_empty() && errors.is_empty() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    for path in spec_files {
        audit_file_source_references(&path, repo_root, &mut errors);
    }

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Audits `Source References` or `Bibliography` list items in a single specification file.
/// Realises: [REQ-PARITY-GATE-29]
fn audit_file_source_references(path: &Path, repo_root: &Path, errors: &mut Vec<String>) {
    let rel = path
        .strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            errors.push(format!(
                "Check 29 failed: Unable to read file '{}': {}",
                rel, e
            ));
            return;
        }
    };

    let mut in_source_refs = false;
    for (idx, line) in content.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let lower = trimmed.to_ascii_lowercase();
            in_source_refs =
                lower.contains("source references") || lower.contains("bibliography");
            continue;
        }
        if in_source_refs && (trimmed.starts_with("- ") || trimmed.starts_with("* ")) {
            let entry_text = trimmed.trim_start_matches(['-', '*', ' ']);
            if entry_text.is_empty() || is_metrology_placeholder(entry_text) {
                errors.push(format!(
                    "Check 29 failed: Incomplete or placeholder source reference on line {} in '{}': '{}'",
                    line_no, rel, entry_text
                ));
            }
        }
    }
}

/// Check 30: Executive Deliverable Traceability & Completeness Gate (Gate 27B).
/// Realises: [REQ-PARITY-GATE-30]
pub fn check_executive_deliverable_traceability(
    repo_root: &Path,
) -> Result<ParityOutcome, Vec<String>> {
    let reports_dir = TargetScanDir::Reports.resolve(repo_root);
    if !reports_dir.is_dir() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let mut errors = Vec::new();
    let mut report_files = Vec::new();
    for entry_res in WalkDir::new(&reports_dir).sort_by_file_name().into_iter() {
        let entry = match entry_res {
            Ok(e) => e,
            Err(err) => {
                errors.push(format!(
                    "Check 30 failed: I/O error walking directory '{}': {}",
                    reports_dir.display(),
                    err
                ));
                continue;
            }
        };
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "md") {
            report_files.push(entry.path().to_path_buf());
        }
    }
    report_files.sort();

    if report_files.is_empty() && errors.is_empty() {
        return Ok(ParityOutcome::PendingOrClean);
    }

    let ssot_conns = collect_ssot_connections_for_check30(repo_root);
    for path in report_files {
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!(
                    "Check 30 failed: Unable to read file '{}': {}",
                    rel, e
                ));
                continue;
            }
        };
        for conn in extract_connection_ids(&content) {
            let digits_only = conn
                .strip_prefix("CONN-")
                .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()));
            if !digits_only || (!ssot_conns.is_empty() && !ssot_conns.contains(&conn)) {
                errors.push(format!(
                    "Check 30 failed: Report '{}' references ungrounded connection '{}'",
                    rel, conn
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Collects SSOT connection IDs from architecture views, `ICD_01`, and `docs/conops/` for Check 30.
/// Realises: [REQ-PARITY-GATE-30]
fn collect_ssot_connections_for_check30(repo_root: &Path) -> BTreeSet<String> {
    let mut ssot_conns = BTreeSet::new();
    let views_dir = repo_root.join("docs").join("architecture").join("views");
    if views_dir.is_dir() {
        for entry in WalkDir::new(&views_dir)
            .sort_by_file_name()
            .into_iter()
            .flatten()
        {
            if entry.file_type().is_file() {
                if let Ok(vc) = fs::read_to_string(entry.path()) {
                    ssot_conns.extend(extract_connection_ids(&vc));
                }
            }
        }
    }
    let icd_file = TargetScanDir::Interfaces
        .resolve(repo_root)
        .join("ICD_01_SYSTEM_INTERFACE_MATRIX.md");
    if icd_file.is_file() {
        if let Ok(ic) = fs::read_to_string(&icd_file) {
            ssot_conns.extend(extract_connection_ids(&ic));
        }
    }
    let conops_dir = TargetScanDir::Conops.resolve(repo_root);
    if conops_dir.is_dir() {
        for entry in WalkDir::new(&conops_dir)
            .sort_by_file_name()
            .into_iter()
            .flatten()
        {
            if entry.file_type().is_file()
                && entry.path().extension().is_some_and(|ext| ext == "md")
            {
                if let Ok(cc) = fs::read_to_string(entry.path()) {
                    ssot_conns.extend(extract_connection_ids(&cc));
                }
            }
        }
    }
    ssot_conns
}
