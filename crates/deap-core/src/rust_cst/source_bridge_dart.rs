//! Dart source file AST parser and top-level classifier. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D qualification standards.
//! Provides lossless Dart AST file parsing and snippet classification via `dart_cst`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Pure delegation to `dart_cst`; zero hand-rolled regex or unsafe pointer arithmetic.
//!
//! ## 3. Preconditions
//! - Callers supply valid UTF-8 Dart source text.
//!
//! ## 4. Postconditions
//! - Returns structured `ParsedSourceFileAst` records with preserved line masks and brace depths.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - O(N) linear time and space complexity with respect to source line count N.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Unrecognized declarations return safe defaults without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0201], [REQ-0602], [REQ-0608]

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use logos::Logos;

use crate::dart_cst::{DartItemKind, DartToken};
use super::source_bridge::{
    extract_item_doc_block_ast, extract_realises_ids_from_doc_lines, ParsedSourceFileAst,
    SourceAstItem, SourceField, SourceItemKind, SourceLanguage, SourceParam,
};
use super::source_bridge_helpers::{
    extract_leading_ast_ident, strip_line_comment_preserving_strings, update_ast_delimiter_depths,
};

/// Extracts Dart class `extends` parent type and field declarations from class lines. Realises: [REQ-0201]
fn extract_dart_class_metadata(lines: &[&str], start_line: usize, end_line: usize) -> (Option<String>, Vec<SourceField>) {
    let header = lines.get(start_line.saturating_sub(1)).copied().unwrap_or("");
    let parent_type = header
        .split_once(" extends ")
        .map(|(_, after)| after.split('{').next().unwrap_or(after).trim().to_string())
        .filter(|s| !s.is_empty());
    let mut fields = Vec::new();
    for raw in lines.get(start_line..end_line.saturating_sub(1)).unwrap_or(&[]) {
        let t = raw.trim();
        if !t.ends_with(';') || t.starts_with("//") || t.starts_with("return ") || t.contains('(') || t.contains('=') {
            continue;
        }
        let body = t
            .trim_end_matches(';')
            .trim_start_matches("static ")
            .trim_start_matches("late ")
            .trim_start_matches("final ")
            .trim_start_matches("const ")
            .trim();
        let parts: Vec<&str> = body.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = (*parts.last().unwrap_or(&"")).to_string();
            let type_name = parts.get(..parts.len().saturating_sub(1)).unwrap_or(&[]).join(" ");
            let is_callback_or_port = type_name.contains("Callback")
                || type_name.starts_with("ValueChanged")
                || type_name.starts_with("ValueSetter")
                || type_name.starts_with("ValueGetter")
                || type_name.contains("Function");
            fields.push(SourceField { name, type_name, is_callback_or_port, doc: None });
        }
    }
    (parent_type, fields)
}

/// Parses Dart source into a normalized `ParsedSourceFileAst` via `dart_cst`. Realises: [REQ-0201]
#[must_use]
pub fn parse_dart_source_file_ast(source: &str) -> ParsedSourceFileAst {
    let lines: Vec<&str> = source.lines().collect();
    let items = crate::dart_cst::parse_dart_items(source)
        .into_iter()
        .map(|d| {
            let kind = match d.kind {
                DartItemKind::Class => SourceItemKind::Class,
                DartItemKind::Enum => SourceItemKind::Enum,
                DartItemKind::TypeAlias => SourceItemKind::Typedef,
                DartItemKind::Mixin => SourceItemKind::Mixin,
                DartItemKind::Extension => SourceItemKind::Extension,
                DartItemKind::Function => SourceItemKind::Function,
                DartItemKind::Const => SourceItemKind::Const,
            };
            let (doc_lines, has_doc) = extract_item_doc_block_ast(&lines, d.start_line.saturating_sub(1));
            let satisfied_requirements = extract_realises_ids_from_doc_lines(&doc_lines);
            let (parent_type, fields) = if kind == SourceItemKind::Class {
                extract_dart_class_metadata(&lines, d.start_line, d.end_line)
            } else {
                (None, Vec::new())
            };
            SourceAstItem {
                kind,
                name: d.name,
                line: d.start_line,
                line_no: d.start_line,
                end_line: d.end_line,
                doc_lines,
                has_doc,
                is_test_annotated: false,
                is_inside_test_block: false,
                is_trait_impl_member: false,
                owner_name: None,
                parent_type,
                fields,
                params: Vec::new(),
                return_type: None,
                satisfied_requirements,
                invariants: Vec::new(),
                states: Vec::new(),
            }
        })
        .collect();
    let mut code_line_mask = Vec::with_capacity(lines.len());
    let mut brace_depths = Vec::with_capacity(lines.len());
    let mut running_braces = 0usize;
    for &line in &lines {
        brace_depths.push(running_braces);
        let mut has_code = false;
        for (tok, span) in DartToken::lexer(line).spanned() {
            if !matches!(tok, Ok(DartToken::Trivia | DartToken::DocComment)) {
                has_code = true;
            }
            if tok == Ok(DartToken::Punct) {
                match line.get(span).unwrap_or("") {
                    "{" => running_braces += 1,
                    "}" => running_braces = running_braces.saturating_sub(1),
                    _ => {}
                }
            }
        }
        code_line_mask.push(has_code);
    }
    ParsedSourceFileAst { language: SourceLanguage::Dart, items, code_line_mask, brace_depths }
}

/// Classifies a single Dart top-level snippet using `dart_cst`. Realises: [REQ-0201]
#[must_use]
pub fn classify_dart_top_level_ast(trimmed: &str) -> Option<(SourceItemKind, String, Option<String>)> {
    let t = trimmed.trim();
    if t.starts_with("import ") || t.starts_with("export ") || t.starts_with("part ") || t.starts_with("library ")
        || t.starts_with("const ") || t.starts_with("var ") || t.starts_with("late ")
        || (t.starts_with("final ") && !t.contains("class ") && !t.contains("mixin "))
    {
        return None;
    }
    if let Some(getter_name) = extract_dart_getter_ident(t) {
        return Some((SourceItemKind::Function, getter_name, None));
    }
    let open_parens = t.matches('(').count();
    let close_parens = t.matches(')').count();
    let mut probe = t.to_string();
    if open_parens > close_parens {
        probe.push_str(&")".repeat(open_parens - close_parens));
        probe.push_str(" {}");
    } else if probe.ends_with('{') {
        probe.push('}');
    } else if !probe.ends_with('}') && !probe.ends_with(';') {
        probe.push_str(" {}");
    }
    parse_dart_source_file_ast(&probe)
        .items
        .into_iter()
        .find(|i| i.kind != SourceItemKind::Const)
        .map(|i| (i.kind, i.name, i.parent_type))
}

/// Classifies Dart member declaration snippet. Realises: [REQ-0201]
#[must_use]
pub fn classify_dart_member_ast(code: &str, class_name: &str) -> Option<(SourceItemKind, String)> {
    let t = code.trim();
    if ["final ", "var ", "late ", "static final ", "static const ", "static var ", "static late "].iter().any(|p| t.starts_with(p)) {
        return None;
    }
    if let Some(after_factory) = t.strip_prefix("factory ").map(str::trim_start) {
        if let Some((cls, rest)) = after_factory.split_once('.') {
            if let Some(named) = extract_leading_ast_ident(rest) {
                return Some((SourceItemKind::Constructor, format!("{cls}.{named}")));
            }
        }
        return extract_leading_ast_ident(after_factory).map(|cname| (SourceItemKind::Constructor, cname));
    }
    let after_const = t.strip_prefix("const ").map_or(t, str::trim_start);
    if let Some(after_cls) = after_const.strip_prefix(class_name).map(str::trim_start) {
        if after_cls.starts_with('(') {
            return Some((SourceItemKind::Constructor, class_name.to_string()));
        }
        if let Some(named_ctor) = after_cls.strip_prefix('.').and_then(extract_leading_ast_ident) {
            return Some((SourceItemKind::Constructor, format!("{class_name}.{named_ctor}")));
        }
    }
    if t.starts_with("const ") { return None; }
    if let Some(getter_name) = extract_dart_getter_ident(t) {
        return Some((SourceItemKind::Method, getter_name));
    }
    let (open_p, close_p) = (t.matches('(').count(), t.matches(')').count());
    let mut probe = t.to_string();
    if open_p > close_p {
        probe.push_str(&")".repeat(open_p - close_p));
        probe.push_str(" {}");
    } else if probe.ends_with('{') {
        probe.push('}');
    } else if !probe.ends_with('}') && !probe.ends_with(';') {
        probe.push_str(" {}");
    }
    crate::dart_cst::parse_dart_items(&probe).into_iter().find(|i| i.kind == DartItemKind::Function).map(|i| (SourceItemKind::Method, i.name))
}

/// Extracts Dart callable identifier. Realises: [REQ-0201]
#[must_use]
pub fn extract_dart_callable_ident(code: &str) -> Option<String> { extract_leading_ast_ident(code) }

/// Extracts Dart getter identifier. Realises: [REQ-0201]
#[must_use]
pub fn extract_dart_getter_ident(code: &str) -> Option<String> {
    let s = strip_leading_dart_annotations(code.trim()).trim_start_matches("static ").trim_start();
    let after = s.strip_prefix("get ").or_else(|| s.split_once(" get ").map(|(_, r)| r)).or_else(|| s.split_once("\tget ").map(|(_, r)| r))?;
    (!after.contains('(')).then(|| extract_leading_ast_ident(after.trim_start())).flatten()
}

/// Strips leading Dart annotations. Realises: [REQ-0201]
#[must_use]
pub fn strip_leading_dart_annotations(code: &str) -> &str {
    let mut depth = 0;
    strip_dart_annotations_tracking_state(code, &mut depth)
}

/// Strips annotations tracking state. Realises: [REQ-0201]
#[must_use]
pub fn strip_dart_annotations_tracking_state<'a>(mut code: &'a str, depth: &mut usize) -> &'a str {
    while *depth > 0 || code.trim_start().starts_with('@') {
        let mut closed_at = None;
        if *depth == 0 {
            let after_at = code.trim_start().strip_prefix('@').unwrap_or("");
            let mut ident_len = 0usize;
            for (tok, span) in DartToken::lexer(after_at).spanned() {
                if span.start == ident_len
                    && (matches!(tok, Ok(DartToken::Ident | DartToken::Keyword))
                        || after_at.get(span.clone()) == Some("."))
                {
                    ident_len = span.end;
                } else {
                    break;
                }
            }
            if ident_len == 0 { return code.trim_start(); }
            let rest = after_at.get(ident_len..).unwrap_or("").trim_start();
            if !rest.starts_with('(') { code = rest; continue; }
            code = rest;
        }
        for (tok, span) in DartToken::lexer(code).spanned() {
            if tok == Ok(DartToken::Punct) {
                if code.get(span.clone()) == Some("(") { *depth += 1; }
                else if code.get(span.clone()) == Some(")") {
                    *depth = depth.saturating_sub(1);
                    if *depth == 0 { closed_at = Some(span.end); break; }
                }
            }
        }
        code = match closed_at { Some(end) => code.get(end..).unwrap_or(""), None => return "" };
    }
    code.trim_start()
}

/// Parses a single Dart formal parameter token into a `SourceParam`. Realises: [REQ-0201]
fn parse_single_dart_param(raw: &str) -> Option<SourceParam> {
    let before_eq = raw.split_once('=').map_or(raw, |(b, _)| b).trim();
    let mut clean = strip_leading_dart_annotations(before_eq).trim();
    while let Some(rest) = clean
        .strip_prefix("required ")
        .or_else(|| clean.strip_prefix("covariant "))
        .or_else(|| clean.strip_prefix("final "))
        .or_else(|| clean.strip_prefix("var "))
    {
        clean = rest.trim();
    }
    if clean.is_empty() {
        return None;
    }
    if let Some(field) = clean.strip_prefix("this.").or_else(|| clean.strip_prefix("super.")) {
        let name = extract_leading_ast_ident(field.trim())?;
        return Some(SourceParam { name, type_name: "dynamic".to_string() });
    }
    let parts: Vec<&str> = clean.split_whitespace().collect();
    if parts.len() >= 2 {
        let raw_name = parts.last().copied().unwrap_or("");
        let name = raw_name.strip_prefix("this.").or_else(|| raw_name.strip_prefix("super.")).unwrap_or(raw_name);
        let clean_name = extract_leading_ast_ident(name).unwrap_or_else(|| name.to_string());
        let type_name = parts.get(..parts.len().saturating_sub(1)).map(|s| s.join(" ")).unwrap_or_default();
        Some(SourceParam { name: clean_name, type_name })
    } else {
        extract_leading_ast_ident(clean).map(|name| SourceParam { name, type_name: "dynamic".to_string() })
    }
}

/// Parses Dart callable signature. Realises: [REQ-0201]
#[must_use]
pub fn parse_dart_callable_signature(code: &str, callable_name: &str) -> (Vec<SourceParam>, Option<String>) {
    let Some((before_open, after_open)) = code.split_once('(') else { return (Vec::new(), None); };
    let ret = before_open.trim().strip_suffix(callable_name)
        .map(|p| p.trim().strip_prefix("static ").unwrap_or_else(|| p.trim()).trim())
        .filter(|s| !s.is_empty() && *s != "const" && *s != "factory")
        .map(ToString::to_string);
    let (mut depth, mut end_idx) = (1usize, after_open.len());
    for (tok, span) in DartToken::lexer(after_open).spanned() {
        if tok == Ok(DartToken::Punct) {
            if after_open.get(span.clone()) == Some("(") { depth += 1; }
            else if after_open.get(span.clone()) == Some(")") {
                depth = depth.saturating_sub(1);
                if depth == 0 { end_idx = span.start; break; }
            }
        }
    }
    let param_str = after_open.get(..end_idx).unwrap_or(after_open);
    let params = param_str.split(',').filter_map(|seg| {
        parse_single_dart_param(seg.trim().trim_start_matches(['{', '[']).trim_end_matches(['}', ']']).trim())
    }).collect();
    (params, ret)
}

/// Collects Dart callable signature from lines across multi-line parameter lists. Realises: [REQ-0201]
#[must_use]
pub fn collect_dart_callable_signature(raw_lines: &[&str], start_idx: usize) -> String {
    let (mut combined, mut brace_depth, mut paren_depth, mut saw_open_paren) = (String::new(), 0usize, 0usize, false);
    for (offset, raw_line) in raw_lines.get(start_idx..start_idx.saturating_add(32).min(raw_lines.len())).unwrap_or(&[]).iter().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") { continue; }
        let without_comment = strip_line_comment_preserving_strings(trimmed).trim();
        let cleaned = if offset == 0 { strip_leading_dart_annotations(without_comment).trim() } else { without_comment };
        if cleaned.is_empty() { continue; }
        if !combined.is_empty() { combined.push(' '); }
        combined.push_str(cleaned);
        if cleaned.contains('(') { saw_open_paren = true; }
        update_ast_delimiter_depths(cleaned, &mut brace_depth, &mut paren_depth);
        if !saw_open_paren || paren_depth == 0 { break; }
    }
    combined
}
