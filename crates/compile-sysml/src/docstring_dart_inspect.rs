#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! [DOC-TIER: A] Dart AST/CST signature, invariant, field, and enum-variant inspection for docstring synthesis.
//!
//! Safety & Traceability: Realises: [REQ-FEAT-42-INSPECT-DART-ITEM-DETAILS]
//! Contract: Extracts Dart callable signatures, `assert(...)` invariants, fields, and enum variants deterministically via `deap_core::dart_cst`.
//! Bounds & Faults: O(L) time and space over inspected item lines; zero panics and zero unwrapped indexing.

use crate::docstring_cst::{
    classify_dart_member_ast, collect_dart_callable_signature, extract_leading_ast_ident,
    parse_dart_callable_signature, strip_leading_dart_annotations,
    strip_line_comment_preserving_strings, update_ast_delimiter_depths, SourceItemKind,
    SourceParam,
};
use crate::docstring_rust_inspect::extract_balanced_rust_assert_cond;

/// Inspects lines starting at `start_idx` in `raw_lines` to extract Dart callable signatures, `assert(...)` invariants, class/mixin/extension fields, or enum variants.
/// Realises: [REQ-FEAT-42-INSPECT-DART-ITEM-DETAILS]
pub(crate) fn inspect_dart_item_details(
    raw_lines: &[&str],
    start_idx: usize,
    kind: SourceItemKind,
    item_name: &str,
) -> (Vec<SourceParam>, Option<String>, Vec<String>, Vec<String>) {
    if kind.is_callable() {
        let (params, return_type, invariants) =
            inspect_dart_callable_details(raw_lines, start_idx, item_name);
        return (params, return_type, Vec::new(), invariants);
    }
    if kind == SourceItemKind::Enum {
        let variants = inspect_dart_enum_variants(raw_lines, start_idx);
        return (Vec::new(), None, variants, Vec::new());
    }
    if matches!(
        kind,
        SourceItemKind::Class | SourceItemKind::Mixin | SourceItemKind::Extension
    ) {
        let fields = inspect_dart_classifier_fields(raw_lines, start_idx, item_name);
        return (Vec::new(), None, fields, Vec::new());
    }
    (Vec::new(), None, Vec::new(), Vec::new())
}

/// Extracts Dart callable parameters, return type, and body `assert(...)` conditions.
/// Realises: [REQ-FEAT-42-INSPECT-DART-ITEM-DETAILS]
fn inspect_dart_callable_details(
    raw_lines: &[&str],
    start_idx: usize,
    item_name: &str,
) -> (Vec<SourceParam>, Option<String>, Vec<String>) {
    let sig_code = collect_dart_callable_signature(raw_lines, start_idx);
    let (params, return_type) = parse_dart_callable_signature(&sig_code, item_name);
    let mut invariants = Vec::new();
    let mut item_brace_depth: usize = 0;
    let mut item_paren_depth: usize = 0;
    let mut saw_open_brace = false;
    if let Some(window) = raw_lines.get(start_idx..) {
        for &raw_line in window {
            let t = strip_line_comment_preserving_strings(raw_line.trim());
            collect_dart_assert_invariants(t, &mut invariants);
            update_ast_delimiter_depths(t, &mut item_brace_depth, &mut item_paren_depth);
            if item_paren_depth == 0 && (item_brace_depth > 0 || t.contains('}')) {
                saw_open_brace = true;
            }
            if (saw_open_brace && item_brace_depth == 0 && item_paren_depth == 0)
                || (!saw_open_brace && item_paren_depth == 0 && t.ends_with(';'))
            {
                break;
            }
        }
    }
    (params, return_type, invariants)
}

/// Collects balanced `assert(...)` conditions from a comment-stripped Dart source line.
/// Realises: [REQ-FEAT-42-INSPECT-DART-ITEM-DETAILS]
fn collect_dart_assert_invariants(t: &str, invariants: &mut Vec<String>) {
    let mut search_slice = t;
    while let Some(pos) = search_slice.find("assert(") {
        let after_assert = search_slice.get(pos.saturating_add(7)..).unwrap_or("");
        if let Some(cond) = extract_balanced_rust_assert_cond(after_assert) {
            let clean = cond.replace('"', "'");
            if !clean.is_empty() && !invariants.contains(&clean) {
                invariants.push(clean);
            }
        }
        search_slice = after_assert;
    }
}

/// Extracts Dart `enum` variant names from `raw_lines` starting at `start_idx`.
/// Realises: [REQ-FEAT-42-INSPECT-DART-ITEM-DETAILS]
fn inspect_dart_enum_variants(raw_lines: &[&str], start_idx: usize) -> Vec<String> {
    let mut fields_or_variants = Vec::new();
    let mut item_brace_depth: usize = 0;
    let mut item_paren_depth: usize = 0;
    let mut saw_open_brace = false;
    let mut enum_variants_done = false;
    let Some(window) = raw_lines.get(start_idx..) else {
        return fields_or_variants;
    };
    for &raw_line in window {
        let t = strip_line_comment_preserving_strings(raw_line.trim());
        let depth_before = item_brace_depth;
        let paren_before = item_paren_depth;
        if (depth_before == 1 || (depth_before == 0 && t.contains('{'))) && !enum_variants_done {
            let segment = if depth_before == 0 {
                t.split_once('{').map(|(_, a)| a).unwrap_or("")
            } else {
                t
            };
            let cut_idx = match (segment.find(';'), segment.find('}')) {
                (Some(s), Some(b)) => Some(s.min(b)),
                (Some(s), None) => Some(s),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };
            let variant_part = match cut_idx {
                Some(idx) => {
                    enum_variants_done = true;
                    segment.get(..idx).unwrap_or("")
                }
                None => segment,
            };
            collect_dart_enum_variant_tokens(
                variant_part,
                paren_before,
                &mut fields_or_variants,
            );
        }
        update_ast_delimiter_depths(t, &mut item_brace_depth, &mut item_paren_depth);
        if item_brace_depth > 0 || t.contains('{') {
            saw_open_brace = true;
        }
        if (saw_open_brace && item_brace_depth == 0) || (!saw_open_brace && t.ends_with(';')) {
            break;
        }
    }
    fields_or_variants
}

/// Collects comma-separated Dart enum variant identifiers at top-level parenthesis depth.
/// Realises: [REQ-FEAT-42-INSPECT-DART-ITEM-DETAILS]
fn collect_dart_enum_variant_tokens(
    variant_part: &str,
    mut local_paren: usize,
    fields_or_variants: &mut Vec<String>,
) {
    for token in variant_part.split(',') {
        if local_paren == 0 {
            let clean_tok = strip_leading_dart_annotations(token.trim()).trim();
            if !clean_tok.is_empty() && !clean_tok.starts_with("//") {
                if let Some(v) = extract_leading_ast_ident(clean_tok) {
                    if !fields_or_variants.contains(&v) {
                        fields_or_variants.push(v);
                    }
                }
            }
        }
        let mut dummy_brace = 0usize;
        update_ast_delimiter_depths(token, &mut dummy_brace, &mut local_paren);
    }
}

/// Extracts Dart class, mixin, or extension field declarations starting at `start_idx`.
/// Realises: [REQ-FEAT-42-INSPECT-DART-ITEM-DETAILS]
fn inspect_dart_classifier_fields(
    raw_lines: &[&str],
    start_idx: usize,
    item_name: &str,
) -> Vec<String> {
    let mut fields_or_variants = Vec::new();
    let mut item_brace_depth: usize = 0;
    let mut item_paren_depth: usize = 0;
    let mut saw_open_brace = false;
    let Some(window) = raw_lines.get(start_idx..) else {
        return fields_or_variants;
    };
    for &raw_line in window {
        let raw_t = strip_line_comment_preserving_strings(raw_line.trim());
        let t = strip_leading_dart_annotations(raw_t).trim();
        let depth_before = item_brace_depth;
        let paren_before = item_paren_depth;
        update_ast_delimiter_depths(raw_t, &mut item_brace_depth, &mut item_paren_depth);
        if item_brace_depth > 0 || raw_t.contains('{') {
            saw_open_brace = true;
        }
        if depth_before == 1
            && paren_before == 0
            && !t.is_empty()
            && !t.starts_with('}')
            && !t.starts_with("//")
            && t.ends_with(';')
            && classify_dart_member_ast(t, item_name).is_none()
        {
            extract_single_dart_field(t, &mut fields_or_variants);
        }
        if (saw_open_brace && item_brace_depth == 0) || (!saw_open_brace && raw_t.ends_with(';')) {
            break;
        }
    }
    fields_or_variants
}

/// Extracts a single Dart field declaration `name: Type` from a class member line ending in `;`.
/// Realises: [REQ-FEAT-42-INSPECT-DART-ITEM-DETAILS]
fn extract_single_dart_field(t: &str, fields_or_variants: &mut Vec<String>) {
    let without_semi = t.trim_end_matches(';').trim();
    let decl_head = without_semi
        .split_once('=')
        .map(|(b, _)| b.trim())
        .unwrap_or(without_semi);
    if decl_head.is_empty() || decl_head.contains('(') || decl_head.contains('{') {
        return;
    }
    let mut clean_decl = decl_head;
    loop {
        let next = clean_decl
            .strip_prefix("static ")
            .or_else(|| clean_decl.strip_prefix("late "))
            .or_else(|| clean_decl.strip_prefix("final "))
            .or_else(|| clean_decl.strip_prefix("const "))
            .or_else(|| clean_decl.strip_prefix("covariant "))
            .or_else(|| clean_decl.strip_prefix("external "))
            .map(str::trim_start);
        if let Some(stripped) = next {
            clean_decl = stripped;
        } else {
            break;
        }
    }
    if let Some(stripped_var) = clean_decl.strip_prefix("var ") {
        if let Some(fname) = extract_leading_ast_ident(stripped_var.trim()) {
            let entry = format!("{fname}: dynamic");
            if !fields_or_variants.contains(&entry) {
                fields_or_variants.push(entry);
            }
        }
    } else if let Some((ftype_raw, fname_raw)) = clean_decl.rsplit_once(char::is_whitespace) {
        let ftype = ftype_raw.trim();
        if let Some(fname) = extract_leading_ast_ident(fname_raw.trim()) {
            if !ftype.is_empty() {
                let entry = format!("{fname}: {ftype}");
                if !fields_or_variants.contains(&entry) {
                    fields_or_variants.push(entry);
                }
            }
        }
    } else if let Some(fname) = extract_leading_ast_ident(clean_decl) {
        let entry = format!("{fname}: dynamic");
        if !fields_or_variants.contains(&entry) {
            fields_or_variants.push(entry);
        }
    }
}
