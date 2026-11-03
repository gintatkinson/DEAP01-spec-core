#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Pillar 5 (`E0605` Semantic Traceability), Mermaid ID sanitization, and workspace scan helpers (`BP-06`).
//! Realises: [REQ-0606-AUDIT-PILLAR-5-TRACEABILITY], [REQ-0606-SCAN-WORKSPACE-DETERMINISTIC]

use crate::audit_code::{AuditFinding, AuditPillar, UmlDiagramKind};
use crate::audit_code_models::{evaluate_source_file_pillars, SourceFileAuditCtx};
use crate::audit_code_scanner::{
    collect_target_source_files, find_brace_block_end, normalize_rel_path, sanitize_slug,
};
use crate::defect_dossier::DossierSeverity;
use deap_core::rust_cst::source_bridge::{
    parse_dart_source_ast, parse_rust_source_ast, validate_source_syntax, SourceLanguage,
};
use std::fs;
use std::path::Path;

/// Determines whether a masked Rust line declares a public `struct`, `enum`, `trait`, or `fn`.
/// Realises: [REQ-0606-IS-PUBLIC-RUST-DECL]
pub(crate) fn is_public_rust_declaration(masked_line: &str) -> bool {
    let trimmed = masked_line.trim_start();
    trimmed.starts_with("pub struct ")
        || trimmed.starts_with("pub enum ")
        || trimmed.starts_with("pub trait ")
        || trimmed.starts_with("pub fn ")
        || trimmed.starts_with("pub const fn ")
        || trimmed.starts_with("pub async fn ")
        || trimmed.starts_with("pub unsafe fn ")
}

/// Checks contiguous preceding Rust doc-comment and attribute lines above `item_line_1based` for `Realises: [`.
/// Realises: [REQ-0606-HAS-PRECEDING-REALISES]
pub(crate) fn has_preceding_realises_doc(raw_lines: &[&str], item_line_1based: usize) -> bool {
    if item_line_1based <= 1 {
        return false;
    }
    let mut cur = item_line_1based.saturating_sub(2);
    while let Some(raw) = raw_lines.get(cur) {
        let trimmed = raw.trim();
        if trimmed.starts_with("///")
            || trimmed.starts_with("/**")
            || trimmed.starts_with('*')
            || trimmed.starts_with("#[doc")
        {
            if trimmed.contains("Realises: [") || trimmed.contains("Realises:[") {
                return true;
            }
        } else if !trimmed.starts_with("#[") {
            break;
        }
        let Some(prev) = cur.checked_sub(1) else {
            break;
        };
        cur = prev;
    }
    false
}

/// Evaluates Pillar 5 (`E0605` Semantic Traceability): public Rust AST items lacking `Realises: [REQ-...]`.
/// Realises: [REQ-0606-AUDIT-PILLAR-5-TRACEABILITY]
pub(crate) fn audit_pillar_semantic_traceability(
    ctx: &SourceFileAuditCtx<'_>,
    out: &mut Vec<AuditFinding>,
) {
    for item in ctx.ast_items {
        let idx0 = item.line_no.saturating_sub(1);
        if ctx.non_test_mask.get(idx0).copied() != Some(true) {
            continue;
        }
        let decl_line = ctx.masked_lines.get(idx0).map(String::as_str).unwrap_or("");
        if !is_public_rust_declaration(decl_line)
            || !item.satisfied_requirements.is_empty()
            || has_preceding_realises_doc(&ctx.raw_lines, item.line_no)
        {
            continue;
        }
        let end_idx = find_brace_block_end(&ctx.masked_lines, idx0);
        let start_line = item.line_no.max(1);
        let end_line = end_idx.saturating_add(1).max(start_line);
        let sym = if item.name.trim().is_empty() {
            "PublicDeclaration".to_string()
        } else {
            item.name.trim().to_string()
        };
        out.push(AuditFinding {
            slug: format!("{}-e0605-trace-l{}", sanitize_slug(ctx.rel_path), start_line),
            file_path: ctx.rel_path.to_string(),
            start_line,
            end_line,
            pillar: AuditPillar::SemanticTraceability,
            severity: DossierSeverity::Important,
            symptom: format!(
                "Public declaration {sym} omits mandatory Realises requirement traceability binding in docstring"
            ),
            test_target: ctx.test_target.clone(),
            whys: vec![
                ("is the public API item unlinked from upstream system requirements".to_string(), format!("public declaration {sym} at line {start_line} carries zero Realises requirement tags")),
                ("did compilation succeed without traceability metadata".to_string(), "standard Rust compiler passes ignore missing requirement tags inside rustdoc comments".to_string()),
                ("do untraced public symbols accumulate during implementation".to_string(), "utility or helper functions are exposed publicly without binding them to an approved requirement ID".to_string()),
                ("does missing traceability break DO-178C bijective verification".to_string(), "untraced public code represents dead or unverified functionality outside the requirement matrix".to_string()),
                ("must static AST traceability checks be enforced".to_string(), "every public struct, enum, trait, and function must declare an explicit Realises requirement tag".to_string()),
            ],
            correctness_trace: format!(
                "At `{}:{start_line}-{end_line}`, public Rust AST item `{sym}` has an empty `satisfied_requirements` set and no `Realises: [REQ-...]` tag in its preceding documentation block, breaking bijective requirement-to-source traceability.",
                ctx.rel_path
            ),
            invariant_violated: "BijectiveRequirementRealisesTag".to_string(),
            uml_kind: UmlDiagramKind::ClassDiagram,
            caller_symbol: sym.clone(),
            callee_symbol: "RequirementTraceabilityMatrix".to_string(),
            affected_callers: vec![
                (sym, format!("Exposes untraced public API surface across lines {start_line}-{end_line}")),
            ],
            language_tag: "rust".to_string(),
            proposed_correction_code: "/// Advances bounded counter by one saturating step.\n///\n/// ## 7. Realization & Traceability\n/// Realises: [REQ-0606-TRACED-COUNTER-STEP]\npub fn traced_counter_step(current: u32) -> u32 {\n    current.saturating_add(1)\n}".to_string(),
        });
    }
}

/// Sanitizes an arbitrary symbol into a pure ASCII PascalCase Mermaid identifier (`[A-Z][A-Za-z0-9]*`).
/// Realises: [REQ-0606-SANITIZE-MERMAID-ID]
pub(crate) fn sanitize_mermaid_identifier(raw: &str, fallback: &str) -> String {
    let mut out = String::with_capacity(raw.len().saturating_add(4));
    let mut cap_next = true;
    for ch in raw.chars() {
        if ch.is_ascii_alphabetic() {
            if cap_next {
                out.push(ch.to_ascii_uppercase());
                cap_next = false;
            } else {
                out.push(ch);
            }
        } else if ch.is_ascii_digit() {
            if !out.is_empty() {
                out.push(ch);
            }
        } else {
            cap_next = true;
        }
    }
    if out.is_empty() {
        fallback.to_string()
    } else {
        out
    }
}

/// Scans `target_path` deterministically, validating balanced syntax and evaluating all requested pillars.
/// Realises: [REQ-0606-SCAN-WORKSPACE-DETERMINISTIC]
pub(crate) fn scan_workspace_path_deterministic(
    workspace_root: &Path,
    target_path: &Path,
    filter: Option<AuditPillar>,
) -> Result<Vec<AuditFinding>, i32> {
    let resolved_target = if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        workspace_root.join(target_path)
    };
    let mut files = Vec::new();
    match fs::metadata(&resolved_target) {
        Ok(_) => collect_target_source_files(&resolved_target, &mut files)?,
        Err(_) => {
            eprintln!(
                "[AUDIT-CODE PATH ERROR] Target path does not exist: {}",
                resolved_target.display()
            );
            return Err(1);
        }
    }
    let mut all_findings = Vec::new();
    for file_path in files {
        let rel_path = normalize_rel_path(workspace_root, &file_path);
        let source_text = fs::read_to_string(&file_path).map_err(|err| {
            eprintln!(
                "[AUDIT-CODE IO ERROR] Unable to read {}: {err}",
                file_path.display()
            );
            1_i32
        })?;
        validate_source_syntax(&source_text, &rel_path).map_err(|msg| {
            eprintln!("[AUDIT-CODE SYNTAX ERROR] {msg}");
            1_i32
        })?;
        let (lang, ast_items) = if rel_path.ends_with(".dart") {
            let parsed = parse_dart_source_ast(&source_text, &rel_path).map_err(|msg| {
                eprintln!("[AUDIT-CODE SYNTAX ERROR] {msg}");
                1_i32
            })?;
            (SourceLanguage::Dart, parsed)
        } else {
            let parsed = parse_rust_source_ast(&source_text, &rel_path).map_err(|msg| {
                eprintln!("[AUDIT-CODE SYNTAX ERROR] {msg}");
                1_i32
            })?;
            (SourceLanguage::Rust, parsed)
        };
        let ctx = SourceFileAuditCtx::new(workspace_root, &rel_path, &source_text, lang, &ast_items);
        all_findings.extend(evaluate_source_file_pillars(&ctx, filter));
    }
    Ok(all_findings)
}

/// Checks whether a `(file_location, pillar)` finding is already persisted under `defects_dir`.
/// Realises: [REQ-0606-IS-ALREADY-RECORDED-LOCALLY]
pub(crate) fn is_already_recorded_locally(
    defects_dir: &Path,
    file_location: &str,
    pillar: AuditPillar,
) -> Result<bool, i32> {
    let entries = match fs::read_dir(defects_dir) {
        Ok(rd) => rd,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(err) => {
            eprintln!(
                "[AUDIT-CODE IO ERROR] Unable to read defects dir {}: {err}",
                defects_dir.display()
            );
            return Err(1);
        }
    };
    let loc_marker = format!("FILE_LOCATION: {file_location}");
    let pillar_marker = format!("Adversarial {} Audit", pillar.as_str());
    for entry_res in entries {
        let entry = entry_res.map_err(|err| {
            eprintln!("[AUDIT-CODE IO ERROR] Defects entry error: {err}");
            1_i32
        })?;
        let path = entry.path();
        let is_md = path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().ends_with(".md"));
        if !path.is_file() || !is_md {
            continue;
        }
        let body = fs::read_to_string(&path).map_err(|err| {
            eprintln!(
                "[AUDIT-CODE IO ERROR] Unable to read dossier {}: {err}",
                path.display()
            );
            1_i32
        })?;
        if body.contains(&loc_marker) && body.contains(&pillar_marker) {
            return Ok(true);
        }
    }
    Ok(false)
}
