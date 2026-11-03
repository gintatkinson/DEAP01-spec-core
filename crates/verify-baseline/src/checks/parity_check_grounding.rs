//! Check 23: Factual Grounding & Numeric Provenance Gate (`check_factual_grounding`).
//!
//! Safety & Traceability: Realises: [REQ-PARITY-GATE-23], [REQ-CITATION-FRAUD-AUDIT]
//! Contract: Audits citations, section anchors, quantitative claims, and protocol tokens against SysML AST and disk.
//! Bounds & Faults: Deterministic WalkDir traversal; fails closed on citation fraud or ungrounded protocol claims.

use super::parity_check_discovery::{discover_sysml_model_text, TargetScanDir};
use super::parity_check_harvest::{
    build_ast_model_ground_truth, extract_candidate_protocol_tokens,
    extract_quantitative_unit_claims, normalize_ast_protocol_symbol,
};
use super::parity_check_slicer::{extract_citations_from_line, CitationRef, SectionSlicer};
use super::parity_gates::ParityOutcome;
use deap_core::rules::EXCLUDED_DIRS;
use deap_core::workspace::is_upstream_compiler;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Bundled AST ground-truth context for Check 23 line-level citation and protocol auditing.
/// Realises: [REQ-CITATION-FRAUD-AUDIT]
struct CitationAuditCtx<'a> {
    rel_path: &'a str,
    slicer: &'a SectionSlicer<'a>,
    ast_symbols: BTreeSet<String>,
    ast_claims: BTreeSet<String>,
    model_lower: String,
    has_model: bool,
}

/// Audits citations and protocols in a specification file against the cited sections and parsed SysML AST.
/// Realises: [REQ-CITATION-FRAUD-AUDIT]
pub fn audit_spec_citations_and_protocols(
    content: &str,
    rel_path: &str,
    slicer: &SectionSlicer,
    model_str: &str,
    errors: &mut Vec<String>,
) {
    let lines: Vec<&str> = content.lines().collect();
    let mut active_citations: Vec<CitationRef> = Vec::new();
    let (ast_symbols, ast_claims) = build_ast_model_ground_truth(model_str);
    let ctx = CitationAuditCtx {
        rel_path,
        slicer,
        ast_symbols,
        ast_claims,
        model_lower: model_str.to_ascii_lowercase(),
        has_model: !model_str.trim().is_empty(),
    };

    let mut in_fence = false;
    for (idx, &line) in lines.iter().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence || trimmed.is_empty() {
            continue;
        }

        let next_line = lines.get(idx.saturating_add(1)).copied().unwrap_or("");
        let line_citations = extract_citations_from_line(trimmed, line_no);
        for cit in &line_citations {
            audit_single_citation(cit, trimmed, next_line, &ctx, errors);
        }

        if !line_citations.is_empty() {
            active_citations = line_citations;
        }

        if ctx.has_model || !active_citations.is_empty() {
            audit_line_ungrounded_protocols(trimmed, line_no, &active_citations, &ctx, errors);
        }
    }
}

/// Audits a single `CitationRef` for file existence, section existence, and claim substantiation.
/// Realises: [REQ-CITATION-FRAUD-AUDIT]
fn audit_single_citation(
    cit: &CitationRef,
    trimmed: &str,
    next_line: &str,
    ctx: &CitationAuditCtx<'_>,
    errors: &mut Vec<String>,
) {
    let is_explicit_source_comment = trimmed.to_ascii_lowercase().contains("<!-- source:");
    if ctx.slicer.resolve_path(&cit.target_file).is_none() {
        let grounded_in_sysml_model = cit.section.is_none()
            && !is_explicit_source_comment
            && ctx.has_model
            && ctx.model_lower.contains(&cit.target_file.to_ascii_lowercase());
        if !grounded_in_sysml_model {
            errors.push(format!(
                "Check 23 failed: Citation fraud detected in '{}': cited source file '{}' does not exist in workspace.",
                ctx.rel_path, cit.target_file
            ));
        }
        return;
    }

    let Some(ref sec) = cit.section else {
        return;
    };
    let Some(sec_text) = ctx.slicer.extract_section(&cit.target_file, sec) else {
        errors.push(format!(
            "Check 23 failed: Citation fraud detected in '{}': cited section '§{}' does not exist in '{}'.",
            ctx.rel_path, sec, cit.target_file
        ));
        return;
    };

    let lower_sec = sec_text.to_ascii_lowercase();
    let combined_claim = if trimmed.starts_with("<!--") {
        next_line
    } else {
        trimmed
    };

    for q_claim in extract_quantitative_unit_claims(combined_claim) {
        let in_ast = ctx.ast_claims.contains(&q_claim) || ctx.model_lower.contains(&q_claim);
        if !lower_sec.contains(&q_claim) && !in_ast {
            errors.push(format!(
                "Check 23 failed: Citation fraud detected in '{}' on line {}: cited section '§{}' in '{}' does not substantiate '{}' claim.",
                ctx.rel_path, cit.line_no, sec, cit.target_file, q_claim
            ));
        }
    }

    for proto_tok in extract_candidate_protocol_tokens(combined_claim) {
        let proto_lower = proto_tok.to_ascii_lowercase();
        let proto_norm = normalize_ast_protocol_symbol(&proto_tok);
        let in_ast = ctx.ast_symbols.contains(&proto_lower)
            || ctx.ast_symbols.contains(&proto_norm)
            || ctx.model_lower.contains(&proto_lower);
        if !lower_sec.contains(&proto_lower) && !in_ast {
            errors.push(format!(
                "Check 23 failed: Citation fraud detected in '{}' on line {}: cited section '§{}' in '{}' does not substantiate '{}' interface claim.",
                ctx.rel_path, cit.line_no, sec, cit.target_file, proto_tok
            ));
        }
    }
}

/// Audits protocol tokens on a single line against the parsed SysML AST and active citations.
/// Realises: [REQ-CITATION-FRAUD-AUDIT]
fn audit_line_ungrounded_protocols(
    trimmed: &str,
    line_no: usize,
    active_citations: &[CitationRef],
    ctx: &CitationAuditCtx<'_>,
    errors: &mut Vec<String>,
) {
    for proto_tok in extract_candidate_protocol_tokens(trimmed) {
        let proto_lower = proto_tok.to_ascii_lowercase();
        let proto_norm = normalize_ast_protocol_symbol(&proto_tok);
        let in_model = ctx.has_model
            && (ctx.ast_symbols.contains(&proto_lower)
                || ctx.ast_symbols.contains(&proto_norm)
                || ctx.model_lower.contains(&proto_lower));
        let in_cited_sec = active_citations.iter().any(|c| {
            c.section.as_ref().is_some_and(|sec| {
                ctx.slicer
                    .extract_section(&c.target_file, sec)
                    .is_some_and(|t| {
                        let tl = t.to_ascii_lowercase();
                        tl.contains(&proto_lower)
                            || normalize_ast_protocol_symbol(&tl).contains(&proto_norm)
                    })
            })
        });
        if !in_model && !in_cited_sec {
            errors.push(format!(
                "Check 23 failed: Ungrounded protocol detected in '{}' on line {}: protocol '{}' is not declared in SysML AST or cited source section.",
                ctx.rel_path, line_no, proto_tok
            ));
        }
    }
}

/// Check 23: Factual Grounding & Numeric Provenance Gate.
/// Realises: [REQ-PARITY-GATE-23]
pub fn check_factual_grounding(
    repo_root: &Path,
    allow_missing_specs: bool,
    strict: bool,
) -> Result<ParityOutcome, Vec<String>> {
    let is_upstream = is_upstream_compiler(repo_root);
    let model_text = discover_sysml_model_text(repo_root);
    let has_extracted = repo_root.join("schema").join("extracted").is_dir();

    if model_text.is_none() && !has_extracted {
        let effective_allow_missing = allow_missing_specs && !strict;
        if !is_upstream && !effective_allow_missing {
            return Err(vec![
                "Check 23 failed: SysML model or schema ground truth is missing in downstream customer mode.".to_string(),
            ]);
        }
        return Ok(ParityOutcome::PendingOrClean);
    }

    if is_upstream {
        return Ok(ParityOutcome::Verified);
    }

    let mut errors = Vec::new();
    let slicer = SectionSlicer::new(repo_root);
    let model_str = model_text.unwrap_or_default();

    for scan_dir in TargetScanDir::ALL
        .iter()
        .copied()
        .filter(|d| d.requires_factual_grounding())
    {
        let dir = scan_dir.resolve(repo_root);
        if dir.is_dir() {
            scan_dir_for_factual_grounding(&dir, repo_root, &slicer, &model_str, &mut errors);
        }
    }

    if errors.is_empty() {
        Ok(ParityOutcome::Verified)
    } else {
        Err(errors)
    }
}

/// Walks a single directory and audits every non-README Markdown file for Check 23 grounding.
/// Realises: [REQ-PARITY-GATE-23]
fn scan_dir_for_factual_grounding(
    dir: &Path,
    repo_root: &Path,
    slicer: &SectionSlicer<'_>,
    model_str: &str,
    errors: &mut Vec<String>,
) {
    for entry_res in WalkDir::new(dir)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !EXCLUDED_DIRS.iter().any(|ex| name == *ex)
                && name != "blueprints"
                && name != "defects"
                && name != "audits"
                && !name.starts_with('.')
                && !name.starts_with('#')
        })
    {
        let entry = match entry_res {
            Ok(e) => e,
            Err(err) => {
                errors.push(format!(
                    "Check 23 failed: I/O error walking directory '{}': {}",
                    dir.display(),
                    err
                ));
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let fname = entry.file_name().to_string_lossy();
        if !fname.ends_with(".md") || fname == "README.md" {
            continue;
        }
        let path = entry.path();
        let rel_path = path
            .strip_prefix(repo_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        match fs::read_to_string(path) {
            Ok(content) => {
                audit_spec_citations_and_protocols(&content, &rel_path, slicer, model_str, errors);
            }
            Err(e) => {
                errors.push(format!(
                    "Check 23 failed: Unable to read file '{}': {}",
                    rel_path, e
                ));
            }
        }
    }
}
