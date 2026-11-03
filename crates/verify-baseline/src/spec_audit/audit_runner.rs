//! Specification audit runner, SysML symbol indexing, and file discovery.
//!
//! Coordinates SysML schema symbol loading, Markdown file collection, and parallel audit dispatch.

use crate::spec_audit::audit_gates::{
    audit_single_spec_file, run_cross_document_spec_gates, AuditedFileOutcome,
};
use crate::spec_audit::{
    load_behavioral_triggers, ActiveGates, SpecAuditError, SpecAuditFinding, SpecAuditOptions,
    SpecAuditSummary, SysmlSymbolIndex,
};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Reads and concatenates `.sysml` files from `schema/` or `.pipeline/schema.sysml`.
/// Realises: [REQ-SPEC-AUDIT-MOD-READ-COMBINED-SYSML]
fn read_combined_sysml_schema(
    repo_root: &Path,
    sysml_files: &[PathBuf],
) -> Result<String, SpecAuditError> {
    let mut combined_content = String::new();
    for f in sysml_files {
        let text = fs::read_to_string(f).map_err(|e| {
            SpecAuditError::IoError(format!("Failed to read schema file at '{}': {}", f.display(), e))
        })?;
        combined_content.push_str(&text);
        combined_content.push_str("\n\n");
    }
    if combined_content.trim().is_empty() {
        let pipeline_schema = repo_root.join(".pipeline").join("schema.sysml");
        if pipeline_schema.is_file() {
            combined_content = fs::read_to_string(&pipeline_schema).map_err(|e| {
                SpecAuditError::IoError(format!("Failed to read schema file at '{}': {}", pipeline_schema.display(), e))
            })?;
        }
    }
    Ok(combined_content)
}

/// Pre-indexes SysML v2 models from schema files into O(1) symbol lookup tables.
/// Realises: [REQ-SPEC-AUDIT-MOD-LOAD-SYSML-SYMBOL-INDEX]
pub fn load_sysml_symbol_index(repo_root: &Path) -> Result<SysmlSymbolIndex, SpecAuditError> {
    let schema_dir = repo_root.join("schema");
    let mut sysml_files = Vec::new();
    let mut other_schema_files = Vec::new();

    if schema_dir.is_dir() {
        let all_schemas =
            deap_core::sysml_ast::UnifiedSchemaModelExtractor::discover_schema_files(&[
                &schema_dir,
            ]);
        for p in all_schemas {
            if p.extension().is_some_and(|ext| ext == "sysml") {
                sysml_files.push(p);
            } else {
                other_schema_files.push(p);
            }
        }
    }

    let combined_content = read_combined_sysml_schema(repo_root, &sysml_files)?;
    let mut parts = deap_core::sysml_ast::parse_part_defs(&combined_content);
    let ports = deap_core::sysml_ast::parse_port_defs(&combined_content);
    let actions = deap_core::sysml_ast::parse_action_defs(&combined_content);
    let constraints = deap_core::sysml_ast::parse_constraint_defs(&combined_content);
    let requirements = deap_core::sysml_ast::parse_requirement_defs(&combined_content);
    let attributes = deap_core::sysml_ast::parse_attribute_defs(&combined_content);
    let top_states = deap_core::sysml_ast::parse_state_defs(&combined_content);
    let test_cases = deap_core::sysml_ast::parse_test_case_defs(&combined_content);

    if !other_schema_files.is_empty() {
        let non_sysml_parts =
            deap_core::sysml_ast::UnifiedSchemaModelExtractor::extract_subsystem_parts(
                &other_schema_files,
            );
        for nsp in non_sysml_parts {
            if !parts.iter().any(|p| p.name == nsp) {
                parts.push(deap_core::sysml_ast::PartDef {
                    name: nsp,
                    is_def: true,
                    ..Default::default()
                });
            }
        }
    }

    if parts.is_empty()
        && ports.is_empty()
        && actions.is_empty()
        && constraints.is_empty()
        && requirements.is_empty()
        && attributes.is_empty()
        && top_states.is_empty()
        && test_cases.is_empty()
    {
        return Ok(SysmlSymbolIndex::new());
    }

    let model = deap_core::sysml_ast::SysmlModel {
        package_name: None,
        doc: None,
        attributes,
        parts,
        ports,
        actions,
        constraints,
        requirements,
        test_cases,
    };

    let mut index = SysmlSymbolIndex::from_model(&model);
    for st in &top_states {
        index.index_state(st);
    }

    Ok(index)
}

/// Resolves a single requested `--only-file` path against `repo_root` and `target_dirs`.
/// Realises: [REQ-SPEC-AUDIT-MOD-RESOLVE-SINGLE-FILE]
fn resolve_single_spec_file(
    repo_root: &Path,
    single_path: &Path,
    target_dirs: &[PathBuf],
) -> Result<Vec<PathBuf>, SpecAuditError> {
    if single_path.is_file() {
        return Ok(vec![single_path.to_path_buf()]);
    }
    let from_root = if single_path.is_absolute() {
        single_path.to_path_buf()
    } else {
        repo_root.join(single_path)
    };
    if from_root.is_file() {
        return Ok(vec![from_root]);
    }

    let mut matched = Vec::new();
    for dir in target_dirs {
        let candidate = dir.join(single_path);
        if candidate.is_file() {
            matched.push(candidate);
        }
    }
    if !matched.is_empty() {
        return Ok(matched);
    }

    let target_name = single_path.file_name().unwrap_or(single_path.as_os_str());
    for dir in target_dirs {
        if !dir.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && !name.starts_with('#')
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let p = entry.path();
                if p.file_name() == Some(target_name) || p.ends_with(single_path) {
                    matched.push(p.to_path_buf());
                }
            }
        }
    }
    if !matched.is_empty() {
        matched.sort();
        matched.dedup();
        return Ok(matched);
    }

    Err(SpecAuditError::IoError(format!(
        "Target specification file not found at: {}",
        from_root.display()
    )))
}

/// Discovers all specification Markdown files targetable by the audit suite.
/// Realises: [REQ-SPEC-AUDIT-MOD-COLLECT-SPEC-FILES]
pub fn collect_spec_files(
    repo_root: &Path,
    only_file: Option<&Path>,
) -> Result<Vec<PathBuf>, SpecAuditError> {
    let target_dirs = [
        repo_root.join("docs").join("epics"),
        repo_root.join("docs").join("features"),
        repo_root.join("docs").join("user-stories"),
        repo_root.join("docs").join("use-cases"),
    ];

    if let Some(single_path) = only_file {
        return resolve_single_spec_file(repo_root, single_path, &target_dirs);
    }

    let mut files = Vec::new();
    for dir in &target_dirs {
        if !dir.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && !name.starts_with('#')
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let p = entry.path();
                let Some(fname) = p.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if fname.ends_with(".md") && fname != "README.md" && fname != ".gitkeep" {
                    files.push(p.to_path_buf());
                }
            }
        }
    }

    files.sort();
    Ok(files)
}

/// Resolves `rules/behavioral_triggers.json` from `repo_root` or an ancestor workspace directory.
/// Realises: [REQ-SPEC-AUDIT-MOD-RESOLVE-BEHAVIORAL-TRIGGERS-PATH]
pub(crate) fn resolve_behavioral_triggers_path(repo_root: &Path) -> Option<PathBuf> {
    let direct = repo_root.join("rules").join("behavioral_triggers.json");
    if direct.is_file() {
        return Some(direct);
    }
    for ancestor in repo_root.ancestors().skip(1) {
        let candidate = ancestor.join("rules").join("behavioral_triggers.json");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    if let Ok(canon) = fs::canonicalize(repo_root) {
        for ancestor in canon.ancestors().skip(1) {
            let candidate = ancestor.join("rules").join("behavioral_triggers.json");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Prints a formatted summary of specification audit findings and elapsed timing.
/// Realises: [REQ-SPEC-AUDIT-MOD-PRINT-SUMMARY]
fn print_spec_audit_summary(
    all_findings: &[SpecAuditFinding],
    duration_ms: u128,
    total_files_audited: usize,
    passed: bool,
) {
    if !all_findings.is_empty() {
        println!("Specification Audit Findings ({} total):", all_findings.len());
        for finding in all_findings {
            if let Some(line) = finding.line {
                println!(
                    "  {}:{}: [{}] {}",
                    finding.location, line, finding.rule_id, finding.message
                );
            } else {
                println!(
                    "  {}: [{}] {}",
                    finding.location, finding.rule_id, finding.message
                );
            }
        }
    }
    println!(
        "Spec audit completed in {} ms ({} files audited, {} findings, passed: {})",
        duration_ms,
        total_files_audited,
        all_findings.len(),
        passed
    );
}

/// Primary orchestrator for the specification quality and coverage audit suite.
/// Realises: [REQ-SPEC-AUDIT-MOD-RUN-SPEC-AUDIT]
pub fn run_spec_audit(options: &SpecAuditOptions) -> Result<SpecAuditSummary, SpecAuditError> {
    let start_time = Instant::now();
    let active_gates = ActiveGates::parse(options.gate_filter.as_deref())?;
    let symbol_index = load_sysml_symbol_index(&options.repo_root)?;

    let triggers_path = options.repo_root.join("rules").join("behavioral_triggers.json");
    let triggers = if let Some(resolved_triggers_path) = resolve_behavioral_triggers_path(&options.repo_root) {
        load_behavioral_triggers(&resolved_triggers_path)?
    } else if options.allow_missing_specs || !active_gates.behavioral {
        Vec::new()
    } else {
        return Err(SpecAuditError::IoError(format!(
            "Behavioral triggers configuration not found at: {}",
            triggers_path.display()
        )));
    };

    let files = collect_spec_files(&options.repo_root, options.only_file.as_deref())?;
    if files.is_empty() {
        let duration_ms = start_time.elapsed().as_millis();
        if options.allow_missing_specs {
            return Ok(SpecAuditSummary { total_files_audited: 0, total_findings: 0, findings: Vec::new(), duration_ms, passed: true });
        }
        let finding = SpecAuditFinding::new(
            "specification-files-present",
            "No specification markdown files found under docs/{epics,features,user-stories,use-cases}",
            "docs",
            None,
        );
        return Ok(SpecAuditSummary { total_files_audited: 0, total_findings: 1, findings: vec![finding], duration_ms, passed: false });
    }

    let file_outcomes: Result<Vec<AuditedFileOutcome>, SpecAuditError> = files
        .par_iter()
        .map(|file_path| audit_single_spec_file(file_path, &options.repo_root, active_gates, &symbol_index))
        .collect();
    let file_outcomes = file_outcomes?;

    let mut all_findings = Vec::new();
    for outcome in &file_outcomes {
        all_findings.extend(outcome.findings.clone());
    }

    if options.only_file.is_none() {
        all_findings.extend(run_cross_document_spec_gates(options, active_gates, &symbol_index, &triggers, &file_outcomes)?);
    }

    let duration_ms = start_time.elapsed().as_millis();
    let total_files_audited = file_outcomes.len();
    let total_findings = all_findings.len();
    let passed = total_findings == 0;
    print_spec_audit_summary(&all_findings, duration_ms, total_files_audited, passed);

    Ok(SpecAuditSummary { total_files_audited, total_findings, findings: all_findings, duration_ms, passed })
}
