#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Dart per-item docstring scanner and Rust/Dart item classifiers for Check 36.
//! Realises: [REQ-0605], [REQ-CHECK-36-AUDIT-DART-DOCSTRINGS]

use crate::checks::doc_gate_validate::{
    contains_placeholder_docstring, update_line_delimiter_depths,
};
use crate::checks::docstring_gate::{validate_item_docblock, DocTier, ItemDocTarget};
use deap_core::rust_cst::source_bridge::{
    extract_dart_getter_ident, scan_line_with_unified_lex_state,
    strip_dart_annotations_tracking_state, CstLexState, ScopedItemScannerState, SourceLanguage,
};
use std::collections::BTreeSet;

/// Per-file mutable state for `audit_dart_source_docstrings`.
/// Realises: [REQ-CHECK-36-AUDIT-DART-DOCSTRINGS]
#[derive(Default)]
struct DartAuditState {
    current_doc: Vec<String>,
    seen_docblocks: BTreeSet<String>,
    scoped_state: ScopedItemScannerState,
    brace_depth: usize,
    paren_depth: usize,
    active_class_name: Option<String>,
    documented_items: usize,
}

/// Audits a single Dart source string for 100% per-item 7-section docstring completeness and zero placeholders.
/// Realises: [REQ-0605], [REQ-CHECK-36-AUDIT-DART-DOCSTRINGS]
pub fn audit_dart_source_docstrings(source: &str, file_rel: &str) -> Result<usize, Vec<String>> {
    let mut violations = Vec::new();
    let mut state = DartAuditState::default();
    let mut dart_lex_state = CstLexState {
        language: SourceLanguage::Dart,
        ..Default::default()
    };

    for (idx, raw_line) in source.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = raw_line.trim();
        let started_in_comment_or_string = dart_lex_state.block_comment_depth > 0
            || dart_lex_state.active_string.is_some()
            || !dart_lex_state.interp_stack.is_empty();

        if !started_in_comment_or_string
            && state.scoped_state.dart_annotation_paren_depth == 0
            && trimmed.starts_with("///")
        {
            if contains_placeholder_docstring(trimmed) {
                violations.push(format!(
                    "{}:{}: placeholder docstring '{}' detected",
                    file_rel, line_no, trimmed
                ));
            }
            state.current_doc.push(trimmed.to_string());
            continue;
        }

        let scanned = scan_line_with_unified_lex_state(trimmed, &mut dart_lex_state);
        let code = scanned.trim();
        if !code.is_empty() {
            process_dart_code_line(code, file_rel, line_no, &mut state, &mut violations);
        }
    }

    if violations.is_empty() {
        Ok(state.documented_items)
    } else {
        Err(violations)
    }
}

/// Processes a single comment-stripped Dart code line during `audit_dart_source_docstrings`.
/// Realises: [REQ-CHECK-36-AUDIT-DART-DOCSTRINGS]
fn process_dart_code_line(
    raw_code: &str,
    file_rel: &str,
    line_no: usize,
    state: &mut DartAuditState,
    violations: &mut Vec<String>,
) {
    let stripped_code = strip_dart_annotations_tracking_state(
        raw_code,
        &mut state.scoped_state.dart_annotation_paren_depth,
    );
    let code = stripped_code.trim();
    if code.is_empty() {
        return;
    }

    if state.paren_depth == 0 {
        if state.brace_depth == 0 {
            if let Some((kind, item_name)) = classify_dart_top_level_item(code) {
                let target = ItemDocTarget {
                    file_rel,
                    line_no,
                    kind,
                    item_name: &item_name,
                    tier: DocTier::DartSevenSection,
                    check_uniqueness: kind == "fn",
                };
                validate_item_docblock(
                    &state.current_doc,
                    &target,
                    &mut state.seen_docblocks,
                    violations,
                );
                state.documented_items = state.documented_items.saturating_add(1);
                if matches!(kind, "class" | "mixin" | "extension") {
                    state.active_class_name = Some(item_name);
                }
            }
            state.current_doc.clear();
        } else if state.brace_depth == 1 {
            if let Some(ref class_name) = state.active_class_name {
                if let Some((kind, member_name)) = classify_dart_member_item(code, class_name) {
                    let target = ItemDocTarget {
                        file_rel,
                        line_no,
                        kind,
                        item_name: &member_name,
                        tier: DocTier::DartSevenSection,
                        check_uniqueness: true,
                    };
                    validate_item_docblock(
                        &state.current_doc,
                        &target,
                        &mut state.seen_docblocks,
                        violations,
                    );
                    state.documented_items = state.documented_items.saturating_add(1);
                }
            }
            state.current_doc.clear();
        } else {
            state.current_doc.clear();
        }
    }

    let prev_brace_depth = state.brace_depth;
    update_line_delimiter_depths(code, &mut state.brace_depth, &mut state.paren_depth);
    if (prev_brace_depth > 0 && state.brace_depth == 0)
        || (state.brace_depth == 0 && (code.ends_with(';') || code.ends_with('}')))
    {
        state.active_class_name = None;
    }
}

/// Classifies a top-level (`brace_depth == 0`) Dart line as a `class`, `enum`, `mixin`, `extension`, `typedef`, or `fn`.
/// Realises: [REQ-CHECK-36-CLASSIFY-DART-TOP-LEVEL]
fn classify_dart_top_level_item(code: &str) -> Option<(&'static str, String)> {
    if code.starts_with("import ")
        || code.starts_with("export ")
        || code.starts_with("part ")
        || code.starts_with("library ")
        || code.starts_with("final ")
        || code.starts_with("const ")
        || code.starts_with("var ")
        || code.starts_with("late ")
    {
        return None;
    }
    let mut rest = code;
    for _ in 0..4 {
        if let Some(after) = rest
            .strip_prefix("abstract ")
            .or_else(|| rest.strip_prefix("base "))
            .or_else(|| rest.strip_prefix("final "))
            .or_else(|| rest.strip_prefix("interface "))
            .or_else(|| rest.strip_prefix("sealed "))
        {
            rest = after.trim_start();
        } else {
            break;
        }
    }
    if let Some(after_mc) = rest.strip_prefix("mixin class ") {
        return Some(("class", extract_leading_identifier(after_mc.trim_start())?));
    }
    if let Some(after_c) = rest.strip_prefix("class ") {
        return Some(("class", extract_leading_identifier(after_c.trim_start())?));
    }
    if let Some(after_e) = rest.strip_prefix("enum ") {
        return Some(("enum", extract_leading_identifier(after_e.trim_start())?));
    }
    if let Some(after_m) = rest.strip_prefix("mixin ") {
        return Some(("mixin", extract_leading_identifier(after_m.trim_start())?));
    }
    if let Some(after_et) = rest.strip_prefix("extension type ") {
        return Some(("extension", extract_leading_identifier(after_et.trim_start())?));
    }
    if let Some(after_ext) = rest.strip_prefix("extension ") {
        let name = extract_leading_identifier(after_ext.trim_start())
            .unwrap_or_else(|| "anonymous_extension".to_string());
        return Some(("extension", name));
    }
    if let Some(after_td) = rest.strip_prefix("typedef ") {
        return Some(("typedef", extract_leading_identifier(after_td.trim_start())?));
    }
    if let Some(fn_name) = extract_dart_callable_name(code) {
        return Some(("fn", fn_name));
    }
    extract_dart_getter_ident(code).map(|g| ("fn", g))
}

/// Classifies a class-body (`brace_depth == 1`) Dart line as a constructor or method declaration.
/// Realises: [REQ-CHECK-36-CLASSIFY-DART-MEMBER]
fn classify_dart_member_item(code: &str, class_name: &str) -> Option<(&'static str, String)> {
    if code.starts_with("final ")
        || code.starts_with("var ")
        || code.starts_with("late ")
        || code.starts_with("static final ")
        || code.starts_with("static const ")
        || code.starts_with("static var ")
        || code.starts_with("static late ")
    {
        return None;
    }
    if let Some(after_factory) = code.strip_prefix("factory ") {
        let trimmed_after = after_factory.trim_start();
        if let Some((cls, rest)) = trimmed_after.split_once('.') {
            if let Some(named) = extract_leading_identifier(rest) {
                return Some(("constructor", format!("{cls}.{named}")));
            }
        }
        return Some(("constructor", extract_leading_identifier(trimmed_after)?));
    }
    let after_const = code.strip_prefix("const ").map_or(code, str::trim_start);
    if let Some(after_cls) = after_const.strip_prefix(class_name) {
        let trimmed_after = after_cls.trim_start();
        if trimmed_after.starts_with('(') {
            return Some(("constructor", class_name.to_string()));
        }
        if let Some(after_dot) = trimmed_after.strip_prefix('.') {
            if let Some(named_ctor) = extract_leading_identifier(after_dot) {
                return Some(("constructor", format!("{class_name}.{named_ctor}")));
            }
        }
    }
    if code.starts_with("const ") {
        return None;
    }
    if let Some(getter_name) = extract_dart_getter_ident(code) {
        return Some(("method", getter_name));
    }
    let after_mods = code.strip_prefix("static ").unwrap_or(code).trim_start();
    if let Some(method_name) = extract_dart_callable_name(after_mods) {
        if !matches!(
            method_name.as_str(),
            "if" | "for" | "while" | "switch" | "return" | "assert" | "super"
        ) {
            return Some(("method", method_name));
        }
    }
    None
}

/// Extracts the leading ASCII identifier (`[A-Za-z0-9_]+`) from `input`.
/// Realises: [REQ-CHECK-36-EXTRACT-IDENTIFIER]
pub(crate) fn extract_leading_identifier(input: &str) -> Option<String> {
    let mut chars = input.chars();
    let first = chars.next()?;
    if !first.is_ascii_alphabetic() && first != '_' {
        return None;
    }
    let mut ident = String::new();
    ident.push(first);
    for ch in chars {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            ident.push(ch);
        } else {
            break;
        }
    }
    Some(ident)
}

/// Extracts the function or method name preceding `(` in a Dart callable declaration line.
/// Realises: [REQ-CHECK-36-EXTRACT-DART-CALLABLE]
fn extract_dart_callable_name(code: &str) -> Option<String> {
    let (before_paren, _) = code.split_once('(')?;
    if before_paren.contains('=') || before_paren.contains(';') {
        return None;
    }
    let trimmed_before = before_paren.trim();
    let without_trailing_generics = if trimmed_before.ends_with('>') {
        let mut depth: usize = 0;
        let mut cut_byte_idx: Option<usize> = None;
        let mut byte_idx = trimmed_before.len();
        for ch in trimmed_before.chars().rev() {
            byte_idx = byte_idx.saturating_sub(ch.len_utf8());
            if ch == '>' {
                depth = depth.saturating_add(1);
            } else if ch == '<' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    cut_byte_idx = Some(byte_idx);
                    break;
                }
            }
        }
        cut_byte_idx
            .and_then(|idx| trimmed_before.get(..idx))
            .unwrap_or(trimmed_before)
            .trim_end()
    } else {
        trimmed_before
    };
    let mut rev_chars = without_trailing_generics.chars().rev();
    let mut ident_rev = String::new();
    for ch in &mut rev_chars {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            ident_rev.push(ch);
        } else if ch.is_whitespace() {
            break;
        } else {
            return None;
        }
    }
    let remaining_return_type: String = rev_chars.collect();
    if ident_rev.is_empty() || remaining_return_type.trim().is_empty() {
        return None;
    }
    let ident: String = ident_rev.chars().rev().collect();
    let first = ident.chars().next()?;
    (first.is_ascii_alphabetic() || first == '_').then_some(ident)
}
