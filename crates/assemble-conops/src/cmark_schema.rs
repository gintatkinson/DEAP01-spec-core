//! SysML CST and CommonMark AST schema extraction (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use deap_core::markdown_ast::parse_markdown_ast;
use deap_core::sysml_cst::{tokenize_sysml, LexedSysmlToken, SysmlToken};
use deap_core::{parse_action_defs, UnifiedSchemaModelExtractor};

use super::cmark_aliases::{extract_numeric_substring, parse_f64_opt};
use super::cmark_engine::{SubsystemPart, SubsystemPort};

/// [DOC-TIER: A] Calculates the 1-indexed line number of a byte offset within `content`.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-FIND-LINE-NUMBER] [FLS: fls_conops_cmark]
/// Contract: Counts newline bytes preceding `byte_offset` in `content: &str` and returns `Some(line)` or `None` if out of bounds.
/// Bounds & Faults: O(byte_offset) time and O(1) space; safe slice bounds check via `.get(..byte_offset)`.
pub fn find_line_number(content: &str, byte_offset: usize) -> Option<usize> {
    let prefix = content.as_bytes().get(..byte_offset)?;
    Some(prefix.iter().filter(|&&b| b == b'\n').count() + 1)
}

/// Checks whether a `LexedSysmlToken` is non-trivia (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn is_non_trivia(tok: &LexedSysmlToken<'_>) -> bool {
    !matches!(tok.token, SysmlToken::Whitespace | SysmlToken::LineComment | SysmlToken::BlockComment)
}

/// Finds the first `package <Identifier>` declaration in a SysML source string (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
pub(super) fn find_first_package(text: &str) -> Option<&str> {
    let tokens = tokenize_sysml(text);
    for (idx, tok) in tokens.iter().enumerate() {
        if tok.token == SysmlToken::PackageKw && tok.slice == "package" {
            for next in tokens.get(idx + 1..).unwrap_or_default() {
                match next.token {
                    SysmlToken::Whitespace | SysmlToken::LineComment | SysmlToken::BlockComment | SysmlToken::DefKw => continue,
                    SysmlToken::Ident | SysmlToken::UnrestrictedName => return Some(next.slice.trim_matches('\'')),
                    _ => break,
                }
            }
        }
    }
    None
}

/// Extracts all `attribute <name> [: <Type>] = <value>;` bindings from a SysML block (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
pub(super) fn find_all_attributes(text: &str) -> Vec<(&str, &str)> {
    let tokens = tokenize_sysml(text);
    let non_trivia: Vec<&LexedSysmlToken<'_>> = tokens.iter().filter(|t| is_non_trivia(t)).collect();
    let (mut results, mut idx) = (Vec::new(), 0);
    while let Some(&tok) = non_trivia.get(idx) {
        if !(tok.token == SysmlToken::StructuralKw && tok.slice == "attribute") {
            idx += 1;
            continue;
        }
        let mut cur = idx + 1 + usize::from(non_trivia.get(idx + 1).is_some_and(|t| t.token == SysmlToken::DefKw));
        let Some(&name_tok) = non_trivia.get(cur) else { break };
        cur += 1 + if non_trivia.get(cur + 1).is_some_and(|t| t.token == SysmlToken::Colon) { 2 } else { 0 };
        if let Some(&eq_tok) = non_trivia.get(cur).filter(|t| (t.token == SysmlToken::Operator && t.slice == "=") || t.token == SysmlToken::ColonEq) {
            let mut semi_cur = cur + 1;
            while let Some(&cand) = non_trivia.get(semi_cur) {
                if cand.token == SysmlToken::Semi {
                    if let Some(val_slice) = text.get(eq_tok.span.end..cand.span.start).filter(|s| !s.contains('\n')) {
                        results.push((name_tok.slice, val_slice.trim().trim_matches(['"', '\''])));
                    }
                    cur = semi_cur;
                    break;
                }
                if matches!(cand.token, SysmlToken::LBrace | SysmlToken::RBrace) { break; }
                semi_cur += 1;
            }
        }
        idx = cur.max(idx + 1);
    }
    results
}

/// [DOC-TIER: A] Parses subsystem port declarations from a SysML `part def` block body via `deap_core::sysml_cst::tokenize_sysml`.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-PARSE-SUBSYSTEM-PORTS-FROM-BODY] [FLS: fls_conops_cmark]
/// Contract: Extracts `SubsystemPort` entries with direction (`IN`, `OUT`, `INOUT`), type, and 1-indexed source line from `body: &str`.
/// Bounds & Faults: O(N) time and space over `body.len()`; preserves OEM provenance `[TIER-1: OEM]`.
pub fn parse_subsystem_ports_from_body(content: &str, body: &str, body_offset: usize, filepath: &str) -> Vec<SubsystemPort> {
    let tokens = tokenize_sysml(body);
    let non_trivia: Vec<&LexedSysmlToken<'_>> = tokens.iter().filter(|t| is_non_trivia(t)).collect();
    let (mut ports, mut idx) = (Vec::new(), 0);
    let dir_of = |s: &str| match s { "inout" => Some("INOUT"), "out" => Some("OUT"), "in" => Some("IN"), _ => None };
    while let Some(&tok) = non_trivia.get(idx) {
        if tok.slice != "port" { idx += 1; continue; }
        let (mut direction, stmt_offset) = idx.checked_sub(1).and_then(|p| non_trivia.get(p)).and_then(|prev| dir_of(prev.slice).map(|d| (d, prev.span.start))).unwrap_or(("INOUT", tok.span.start));
        let mut cursor = idx + 1 + usize::from(non_trivia.get(idx + 1).is_some_and(|t| t.slice == "def"));
        if direction == "INOUT" {
            if let Some(dir) = non_trivia.get(cursor).and_then(|t| dir_of(t.slice)) { direction = dir; cursor += 1; }
        }
        let Some(&name_tok) = non_trivia.get(cursor) else { break };
        if matches!(name_tok.token, SysmlToken::Semi | SysmlToken::LBrace | SysmlToken::RBrace | SysmlToken::Colon) { idx += 1; continue; }
        let port_name = name_tok.slice.trim_matches('\'').to_string();
        cursor += 1;
        let mut type_name = "Port".to_string();
        if non_trivia.get(cursor).is_some_and(|t| t.token == SysmlToken::Colon) {
            if let Some(&t_tok) = non_trivia.get(cursor + 1).filter(|t| !matches!(t.token, SysmlToken::Semi | SysmlToken::LBrace | SysmlToken::RBrace)) {
                type_name = t_tok.slice.trim_matches('\'').to_string();
                cursor += 2;
            }
        }
        ports.push(SubsystemPort {
            name: port_name, direction: direction.to_string(), type_name, doc: String::new(),
            source_file: filepath.to_string(), source_line: find_line_number(content, body_offset + stmt_offset),
            epistemic_tier: "[TIER-1: OEM]".to_string(),
        });
        idx = cursor.max(idx + 1);
    }
    ports
}

/// Extracts attributes, mass, power, and action names from a `part def` body (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn extract_part_body_attributes(body: &str) -> (BTreeMap<String, String>, Option<f64>, Option<f64>, Vec<String>) {
    let (mut attributes, mut mass_kg, mut power_w, mut actions) = (BTreeMap::new(), None, None, Vec::new());
    if !body.is_empty() {
        for (attr_name, raw_val) in find_all_attributes(body) {
            attributes.insert(attr_name.to_string(), raw_val.to_string());
            let lower_attr = attr_name.to_ascii_lowercase();
            if let Some(val_f64) = extract_numeric_substring(raw_val).and_then(parse_f64_opt) {
                if (lower_attr.contains("mass") || lower_attr.contains("weight")) && mass_kg.is_none() {
                    mass_kg = Some(val_f64);
                } else if (lower_attr.contains("power") || lower_attr.contains("watt")) && power_w.is_none() {
                    power_w = Some(val_f64);
                }
            }
        }
        for act in parse_action_defs(body) {
            actions.push(act.name);
        }
    }
    (attributes, mass_kg, power_w, actions)
}

/// Resolves the `{ ... }` body slice and next token index starting at `cur` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn resolve_part_body_span<'a>(content: &'a str, tokens: &[LexedSysmlToken<'_>], cur: usize, part_pos: usize, idx: usize) -> Option<(&'a str, usize, usize)> {
    if tokens.get(cur).is_some_and(|t| t.token == SysmlToken::LBrace) {
        let open_end = tokens.get(cur)?.span.end;
        let (mut depth, mut close_idx) = (0usize, None);
        for (k, t) in tokens.iter().enumerate().skip(cur) {
            if t.token == SysmlToken::LBrace { depth += 1; }
            else if t.token == SysmlToken::RBrace {
                depth = depth.saturating_sub(1);
                if depth == 0 { close_idx = Some(k); break; }
            }
        }
        if let Some(r_idx) = close_idx {
            let close_start = tokens.get(r_idx)?.span.start;
            Some((content.get(open_end..close_start).unwrap_or(""), open_end, r_idx + 1))
        } else {
            Some((content.get(open_end..).unwrap_or(""), open_end, tokens.len()))
        }
    } else if tokens.get(cur).is_some_and(|t| t.token == SysmlToken::Semi) {
        Some(("", part_pos, cur + 1))
    } else {
        Some(("", part_pos, idx + 1))
    }
}

/// Parses a single `part` or `part def` declaration starting at token `idx` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn parse_part_from_tokens(content: &str, filepath: &str, tokens: &[LexedSysmlToken<'_>], idx: usize) -> Option<(SubsystemPart, usize)> {
    let part_pos = tokens.get(idx)?.span.start;
    let mut cur = idx + 1;
    while tokens.get(cur).is_some_and(|t| !is_non_trivia(t)) { cur += 1; }
    if tokens.get(cur).is_some_and(|t| t.token == SysmlToken::DefKw) {
        cur += 1;
        while tokens.get(cur).is_some_and(|t| !is_non_trivia(t)) { cur += 1; }
    }
    let name_tok = tokens.get(cur)?;
    if !matches!(name_tok.token, SysmlToken::Ident | SysmlToken::UnrestrictedName) || name_tok.slice == "def" { return None; }
    let name = name_tok.slice.trim_matches('\'').to_string();
    cur += 1;
    while tokens.get(cur).is_some_and(|t| !is_non_trivia(t)) { cur += 1; }
    if tokens.get(cur).is_some_and(|t| matches!(t.token, SysmlToken::Colon | SysmlToken::ColonGt)) {
        for _ in 0..2 {
            cur += 1;
            while tokens.get(cur).is_some_and(|t| !is_non_trivia(t)) { cur += 1; }
        }
    }
    let (body, body_offset, next_idx) = resolve_part_body_span(content, tokens, cur, part_pos, idx)?;
    let doc = tokenize_sysml(body).into_iter().find(|t| t.token == SysmlToken::BlockComment).map(|t| {
        let inner = t.slice.trim().strip_prefix("/*").unwrap_or(t.slice).strip_suffix("*/").unwrap_or(t.slice).trim();
        inner.strip_prefix("doc").unwrap_or(inner).trim().to_string()
    }).unwrap_or_default();
    let ports = if body.is_empty() { Vec::new() } else { parse_subsystem_ports_from_body(content, body, body_offset, filepath) };
    let (attributes, mass_kg, power_w, actions) = extract_part_body_attributes(body);
    Some((
        SubsystemPart {
            name, doc, ports, mass_kg, power_w, actions, attributes, constraints: Vec::new(),
            source_file: filepath.to_string(), source_line: find_line_number(content, part_pos),
            epistemic_tier: "[TIER-1: OEM]".to_string(),
        },
        next_idx.max(idx + 1),
    ))
}

/// [DOC-TIER: A] Extracts `SubsystemPart` definitions with ports, attributes, actions, and source provenance from SysML text.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-SUBSYSTEM-PARTS-FROM-SYSML] [FLS: fls_conops_cmark]
/// Contract: Scans `content: &str` token stream for `part` / `part def` declarations and returns deduplicated `Vec<SubsystemPart>`.
/// Bounds & Faults: O(N) time and space over `content.len()`; records 1-indexed `source_line` and `[TIER-1: OEM]` provenance.
pub fn extract_subsystem_parts_from_sysml(content: &str, filepath: &str) -> Vec<SubsystemPart> {
    let tokens = tokenize_sysml(content);
    let (mut parts, mut seen, mut idx) = (Vec::new(), BTreeSet::new(), 0);
    while let Some(tok) = tokens.get(idx) {
        if tok.token == SysmlToken::StructuralKw && tok.slice == "part" {
            if let Some((part, next_idx)) = parse_part_from_tokens(content, filepath, &tokens, idx) {
                if seen.insert(part.name.clone()) { parts.push(part); }
                idx = next_idx;
                continue;
            }
        }
        idx += 1;
    }
    parts
}

/// Builds `SubsystemPart` records with line-number provenance from extracted part names (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
fn parts_from_names(content: &str, filepath: &str, part_names: BTreeSet<String>) -> Vec<SubsystemPart> {
    part_names.into_iter().map(|name| {
        let line_num = content.lines().enumerate().find(|(_, l)| l.contains(&name)).map(|(idx, _)| idx + 1);
        SubsystemPart { name, source_file: filepath.to_string(), source_line: line_num, ..Default::default() }
    }).collect()
}

/// [DOC-TIER: A] Extracts subsystem parts from a YAML schema document.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-SUBSYSTEM-PARTS-FROM-YAML] [FLS: fls_conops_cmark]
/// Contract: Delegates part name extraction to `UnifiedSchemaModelExtractor` for `.yaml` and attaches source provenance.
/// Bounds & Faults: O(N) time and space over `content.len()`; infallible extraction.
pub fn extract_subsystem_parts_from_yaml(content: &str, filepath: &str) -> Vec<SubsystemPart> {
    parts_from_names(content, filepath, UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".yaml", content))
}

/// [DOC-TIER: A] Extracts subsystem parts from a JSON schema document.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-SUBSYSTEM-PARTS-FROM-JSON] [FLS: fls_conops_cmark]
/// Contract: Delegates part name extraction to `UnifiedSchemaModelExtractor` for `.json` and attaches source provenance.
/// Bounds & Faults: O(N) time and space over `content.len()`; infallible extraction.
pub fn extract_subsystem_parts_from_json(content: &str, filepath: &str) -> Vec<SubsystemPart> {
    parts_from_names(content, filepath, UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".json", content))
}

/// [DOC-TIER: A] Extracts subsystem parts from a Markdown specification using `deap_core::markdown_ast`.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-SUBSYSTEM-PARTS-FROM-MARKDOWN] [FLS: fls_conops_cmark]
/// Contract: Extracts subsystem part names and enriches descriptions/line offsets from `parse_markdown_ast(content)` tables.
/// Bounds & Faults: O(N) time and space over `content.len()`; attaches 1-indexed `source_line` and `[TIER-1: OEM]` tier.
pub fn extract_subsystem_parts_from_markdown(content: &str, filepath: &str) -> Vec<SubsystemPart> {
    let ast = parse_markdown_ast(content);
    let mut parts = parts_from_names(content, filepath, UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".md", content));
    for part in &mut parts {
        for table in &ast.tables {
            for row in &table.rows {
                if row.first().is_some_and(|c0| c0.contains(&part.name)) {
                    if let Some(desc) = row.get(1).filter(|d| part.doc.is_empty() && !d.trim().is_empty()) {
                        part.doc = desc.trim().to_string();
                    }
                    if part.source_line.is_none() { part.source_line = find_line_number(content, table.start_offset); }
                }
            }
        }
    }
    parts
}

/// [DOC-TIER: A] Extracts subsystem parts from an interface schema file (`.proto`, `.idl`, `.arxml`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-SUBSYSTEM-PARTS-FROM-INTERFACE-SCHEMA] [FLS: fls_conops_cmark]
/// Contract: Dispatches by file extension to `UnifiedSchemaModelExtractor` and attaches line-number provenance.
/// Bounds & Faults: O(N) time and space over `content.len()`; infallible extraction.
pub fn extract_subsystem_parts_from_interface_schema(content: &str, filepath: &str) -> Vec<SubsystemPart> {
    let ext = Path::new(filepath).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    parts_from_names(content, filepath, UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(&format!(".{ext}"), content))
}

/// [DOC-TIER: A] Extracts subsystem parts from a schema file or directory tree.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-SUBSYSTEM-PARTS] [FLS: fls_conops_cmark]
/// Contract: Inspects `target: &Path` (single file or directory of schema files) and returns all discovered `Vec<SubsystemPart>`.
/// Bounds & Faults: O(F * N) time and space over discovered schema files; ignores unreadable files gracefully.
pub fn extract_subsystem_parts(target: &Path) -> Vec<SubsystemPart> {
    let mut parts = Vec::new();
    if target.is_file() {
        if let Ok(content) = fs::read_to_string(target) {
            let fpath = target.to_string_lossy().to_string();
            let ext = target.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            match ext.as_str() {
                "sysml" => parts.extend(extract_subsystem_parts_from_sysml(&content, &fpath)),
                "yaml" | "yml" => parts.extend(extract_subsystem_parts_from_yaml(&content, &fpath)),
                "json" => parts.extend(extract_subsystem_parts_from_json(&content, &fpath)),
                "md" | "markdown" => parts.extend(extract_subsystem_parts_from_markdown(&content, &fpath)),
                _ => parts.extend(extract_subsystem_parts_from_interface_schema(&content, &fpath)),
            }
        }
    } else if target.is_dir() {
        for sf in UnifiedSchemaModelExtractor::discover_schema_files(&[target]) {
            parts.extend(extract_subsystem_parts(&sf));
        }
    }
    parts
}
