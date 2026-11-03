//! Per-file and cross-document gate execution helpers for `spec_audit::audit_runner`.
//!
//! Evaluates sections, metadata, UML, LUMI, provenance, ConOps, and behavioral trigger gates.

use crate::spec_audit::contract_and_schema_parity::run_extended_cross_doc_gates;
use crate::spec_audit::{
    infer_spec_type, parse_markdown, validate_behavioral_triggers, validate_conops,
    validate_conops_workspace, validate_contract_ab_and_dod, validate_document_uml,
    validate_epic_statechart_heterogeneity, validate_feature_operation_coverage,
    validate_filenames, validate_logical_ui, validate_metadata, validate_parent_epic_alignment,
    validate_provenance, validate_sections, validate_title_uniqueness,
    validate_use_case_anti_boilerplate, validate_use_case_diagram_compliance,
    validate_user_story_test_cases, ActiveGates, AuditedSpec, BehavioralTrigger, MarkdownDoc,
    SpecAuditError, SpecAuditFinding, SpecAuditOptions, SysmlSymbolIndex,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Outcome of auditing a single Markdown specification file.
/// Realises: [REQ-SPEC-AUDIT-MOD-AUDITEDFILEOUTCOME]
pub(crate) struct AuditedFileOutcome {
    pub(crate) rel_path: String,
    pub(crate) spec_type: String,
    pub(crate) content: String,
    pub(crate) doc: MarkdownDoc,
    pub(crate) findings: Vec<SpecAuditFinding>,
}

/// Evaluates UML model integrity, use-case diagram compliance, and feature/story bindings.
/// Realises: [REQ-SPEC-AUDIT-MOD-AUDIT-FILE-UML]
fn audit_file_uml_gates(
    spec_and_path: (&str, &str),
    doc: &MarkdownDoc,
    content: &str,
    symbol_index: &SysmlSymbolIndex,
    findings: &mut Vec<SpecAuditFinding>,
) {
    let (spec_type, rel_path) = spec_and_path;
    if let Err(uml_errs) = validate_document_uml(doc, symbol_index) {
        for err in uml_errs {
            findings.push(SpecAuditFinding::new("uml-model-integrity", err.to_string(), rel_path, None));
        }
    }
    if spec_type == "use_case" {
        for err in validate_use_case_diagram_compliance(doc, rel_path) {
            findings.push(SpecAuditFinding::new("use-case-diagram-compliance", err, rel_path, None));
        }
    }
    if spec_type == "feature" {
        findings.extend(validate_feature_operation_coverage(doc, content, rel_path, symbol_index));
    }
    if spec_type == "user_story" {
        findings.extend(validate_user_story_test_cases(doc, content, rel_path, symbol_index));
    }
}

/// Audits a single Markdown specification file across all enabled per-file gates.
/// Realises: [REQ-SPEC-AUDIT-MOD-AUDIT-SINGLE-FILE]
pub(crate) fn audit_single_spec_file(
    file_path: &Path,
    repo_root: &Path,
    active_gates: ActiveGates,
    symbol_index: &SysmlSymbolIndex,
) -> Result<AuditedFileOutcome, SpecAuditError> {
    let content = fs::read_to_string(file_path).map_err(|e| {
        SpecAuditError::IoError(format!("Failed to read specification file at '{}': {}", file_path.display(), e))
    })?;
    let rel_path = file_path
        .strip_prefix(repo_root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| file_path.to_string_lossy().to_string())
        .replace('\\', "/");

    let doc = parse_markdown(&content)?;
    let spec_type = infer_spec_type(&rel_path).to_string();
    let mut findings = Vec::new();

    if active_gates.sections {
        for err in validate_sections(&spec_type, &doc, &rel_path) {
            findings.push(SpecAuditFinding::new("mandatory-sections-missing", err, &rel_path, None));
        }
        for err in validate_contract_ab_and_dod(&spec_type, &doc, &content, &rel_path) {
            findings.push(SpecAuditFinding::new(
                "contract-ab-and-3layer-dod",
                err,
                &rel_path,
                None,
            ));
        }
        if spec_type == "use_case" {
            for err in validate_use_case_anti_boilerplate(&doc, &content, &rel_path) {
                findings.push(SpecAuditFinding::new("anti-boilerplate-gate", err, &rel_path, None));
            }
        }
    }

    if active_gates.metadata {
        for err in validate_metadata(&spec_type, &doc, &content, &rel_path) {
            let rule_id = if err.contains("stuttering prefix") || err.contains("repeating title prefix") {
                "anti-stutter-gate"
            } else {
                "specification-metadata-integrity"
            };
            findings.push(SpecAuditFinding::new(rule_id, err, &rel_path, None));
        }
        for ph in &doc.unresolved_placeholders {
            findings.push(SpecAuditFinding::new(
                "unresolved-placeholder",
                format!("{}: {}", ph.label, ph.text),
                &rel_path,
                Some(ph.line),
            ));
        }
        findings.extend(
            crate::cmark_gates::spec_audit::validate_file_si_metrology_cmark(&content, &rel_path),
        );
    }

    if active_gates.uml {
        audit_file_uml_gates((&spec_type, &rel_path), &doc, &content, symbol_index, &mut findings);
    }
    if active_gates.logical_ui {
        findings.extend(validate_logical_ui(&doc, &content, &rel_path)?);
    }
    if active_gates.provenance {
        findings.extend(validate_provenance(&doc, &content, &rel_path, Some(symbol_index))?);
    }
    if active_gates.conops {
        findings.extend(validate_conops(&doc, &content, &rel_path)?);
    }

    Ok(AuditedFileOutcome { rel_path, spec_type, content, doc, findings })
}

/// Evaluates cross-document title uniqueness and filename conventions.
/// Realises: [REQ-SPEC-AUDIT-MOD-CROSS-DOC-METADATA]
fn run_cross_doc_metadata_gates(
    file_outcomes: &[AuditedFileOutcome],
    findings: &mut Vec<SpecAuditFinding>,
) {
    let title_tuples: Vec<(String, String, MarkdownDoc)> = file_outcomes
        .iter()
        .map(|o| (o.spec_type.clone(), o.rel_path.clone(), o.doc.clone()))
        .collect();
    for err in validate_title_uniqueness(&title_tuples) {
        findings.push(SpecAuditFinding::new(
            "spec-title-uniqueness",
            err,
            "docs",
            None,
        ));
    }

    let mut grouped_by_dir: BTreeMap<String, (String, Vec<String>)> = BTreeMap::new();
    for o in file_outcomes {
        let dir = o
            .rel_path
            .rfind('/')
            .and_then(|idx| o.rel_path.get(..idx))
            .unwrap_or("docs");
        let fname = o
            .rel_path
            .rfind('/')
            .and_then(|idx| o.rel_path.get(idx + 1..))
            .unwrap_or(&o.rel_path);
        let entry = grouped_by_dir
            .entry(dir.to_string())
            .or_insert_with(|| (o.spec_type.clone(), Vec::new()));
        entry.1.push(fname.to_string());
    }

    for (dir_rel, (spec_type, fnames)) in &grouped_by_dir {
        let fname_slices: Vec<&str> = fnames.iter().map(|s| s.as_str()).collect();
        for err in validate_filenames(spec_type, &fname_slices, dir_rel) {
            let rule_id = if err.contains("stuttering prefix") {
                "anti-stutter-gate"
            } else {
                "spec-filename-convention"
            };
            findings.push(SpecAuditFinding::new(rule_id, err, dir_rel, None));
        }
    }
}

/// Runs cross-document workspace gates across all audited file outcomes.
/// Realises: [REQ-SPEC-AUDIT-MOD-CROSS-DOC-GATES]
pub(crate) fn run_cross_document_spec_gates(
    options: &SpecAuditOptions,
    active_gates: ActiveGates,
    symbol_index: &SysmlSymbolIndex,
    triggers: &[BehavioralTrigger],
    file_outcomes: &[AuditedFileOutcome],
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let mut findings = Vec::new();

    if active_gates.metadata {
        run_cross_doc_metadata_gates(file_outcomes, &mut findings);
    }

    let epics: Vec<(&str, &MarkdownDoc)> = file_outcomes
        .iter()
        .filter(|o| o.spec_type == "epic")
        .map(|o| (o.rel_path.as_str(), &o.doc))
        .collect();
    let features: Vec<(&str, &MarkdownDoc)> = file_outcomes
        .iter()
        .filter(|o| o.spec_type == "feature")
        .map(|o| (o.rel_path.as_str(), &o.doc))
        .collect();
    let stories: Vec<(&str, &MarkdownDoc)> = file_outcomes
        .iter()
        .filter(|o| o.spec_type == "user_story")
        .map(|o| (o.rel_path.as_str(), &o.doc))
        .collect();

    if active_gates.uml {
        for err in validate_epic_statechart_heterogeneity(&epics) {
            findings.push(SpecAuditFinding::new(
                "epic-statechart-heterogeneity",
                err,
                "docs/epics",
                None,
            ));
        }
    }

    if active_gates.metadata || active_gates.sections {
        for err in validate_parent_epic_alignment(&stories, &features, &epics) {
            findings.push(SpecAuditFinding::new(
                "parent-epic-alignment",
                err,
                "docs/user-stories",
                None,
            ));
        }
    }

    if active_gates.behavioral {
        let audited_specs: Vec<AuditedSpec> = file_outcomes
            .iter()
            .map(|o| AuditedSpec {
                rel_path: o.rel_path.clone(),
                spec_type: o.spec_type.clone(),
                doc: o.doc.clone(),
                content: o.content.clone(),
            })
            .collect();
        findings.extend(validate_behavioral_triggers(
            triggers,
            symbol_index,
            &audited_specs,
        )?);
    }

    if active_gates.conops {
        findings.extend(validate_conops_workspace(&options.repo_root)?);
    }
    if options.only_file.is_none()
        && (active_gates.sections || active_gates.metadata || active_gates.uml)
    {
        run_extended_cross_doc_gates(&options.repo_root, &mut findings);
    }

    Ok(findings)
}
