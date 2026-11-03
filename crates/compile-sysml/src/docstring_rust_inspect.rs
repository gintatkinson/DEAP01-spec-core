#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! [DOC-TIER: A] Rust callable signature, struct field, enum variant, and `assert!` invariant inspector.
//! Safety & Traceability: Realises: [REQ-FEAT-42-INSPECT-RUST-ITEM-DETAILS], [REQ-0605]
//! Contract: Extracts parameter lists, return types, struct/enum/trait members, and balanced `assert!` conditions from Rust lines.
//! Bounds & Faults: O(L) time and space in declaration body lines `L`; zero panics or unbounded slices.

use crate::docstring_contract_synth::{
    synthesize_7_section_docstring, synthesize_tier_bc_docstring, ItemSynthesisContext,
};
use crate::docstring_cst::{
    extract_leading_ast_ident, leading_whitespace, parse_rust_callable_signature,
    single_char_literal_byte_len, strip_line_comment_preserving_strings, SourceItemKind,
    SourceLineEdits, SourceParam,
};
use crate::docstring_rust_scan::ClassifiedRustItem;
use deap_core::sysml_cst::PackageDef;
use std::collections::BTreeSet;

/// Scanned single-line context passed between Rust docstring injection helpers.
/// Realises: [REQ-FEAT-42-RUST-LINE-SCAN-CONTEXT]
pub(crate) struct RustLineScanContext<'b> {
    pub raw_line: &'b str,
    pub idx: usize,
    pub code: &'b str,
    pub inline_comment_closed: bool,
}

/// Inspects lines starting at `start_idx` in `raw_lines` to extract Rust callable signatures, struct fields, or enum variants.
/// Realises: [REQ-FEAT-42-INSPECT-RUST-ITEM-DETAILS]
#[must_use]
pub(crate) fn inspect_rust_item_details(
    raw_lines: &[&str],
    start_idx: usize,
    kind: SourceItemKind,
) -> (Vec<SourceParam>, Option<String>, Vec<String>, Vec<String>) {
    if kind.is_callable() {
        let (params, return_type, invariants) = inspect_rust_callable_details(raw_lines, start_idx);
        (params, return_type, Vec::new(), invariants)
    } else if matches!(
        kind,
        SourceItemKind::Struct | SourceItemKind::Enum | SourceItemKind::Trait
    ) {
        let members = inspect_rust_nominal_members(raw_lines, start_idx, kind);
        (Vec::new(), None, members, Vec::new())
    } else {
        (Vec::new(), None, Vec::new(), Vec::new())
    }
}

/// Extracts parameters, return type, and `assert!` conditions from a Rust callable starting at `start_idx`.
/// Realises: [REQ-FEAT-42-INSPECT-RUST-ITEM-DETAILS]
fn inspect_rust_callable_details(
    raw_lines: &[&str],
    start_idx: usize,
) -> (Vec<SourceParam>, Option<String>, Vec<String>) {
    let mut sig_buf = String::new();
    let (mut brace_depth, mut saw_open_brace) = (0i32, false);
    let mut body_start_idx = start_idx.saturating_add(1);
    let mut invariants = Vec::new();
    if let Some(window) = raw_lines.get(start_idx..) {
        for (offset, &line) in window.iter().enumerate() {
            let code = strip_line_comment_preserving_strings(line.trim());
            if !sig_buf.is_empty() {
                sig_buf.push(' ');
            }
            sig_buf.push_str(code);
            let (delta, has_open) = count_rust_line_brace_delta(code);
            if has_open {
                saw_open_brace = true;
                if let Some((_, after_brace)) = code.split_once('{') {
                    push_rust_assert_invariant(after_brace.trim(), &mut invariants);
                }
            }
            brace_depth = brace_depth.saturating_add(delta);
            body_start_idx = start_idx.saturating_add(offset).saturating_add(1);
            if has_open || code.ends_with(';') {
                break;
            }
        }
    }
    let (params, return_type) = parse_rust_callable_signature(&sig_buf);
    if saw_open_brace && brace_depth > 0 {
        if let Some(body_window) = raw_lines.get(body_start_idx..) {
            for &line in body_window {
                let t = strip_line_comment_preserving_strings(line.trim());
                push_rust_assert_invariant(t, &mut invariants);
                let (delta, has_open) = count_rust_line_brace_delta(t);
                saw_open_brace |= has_open;
                brace_depth = brace_depth.saturating_add(delta);
                if saw_open_brace && brace_depth <= 0 {
                    break;
                }
            }
        }
    }
    (params, return_type, invariants)
}

/// Appends a balanced `assert!` or `debug_assert!` condition from `stmt` into `invariants`.
/// Realises: [REQ-FEAT-42-INSPECT-RUST-ITEM-DETAILS]
fn push_rust_assert_invariant(stmt: &str, invariants: &mut Vec<String>) {
    if let Some(rest) = stmt
        .strip_prefix("assert!(")
        .or_else(|| stmt.strip_prefix("debug_assert!("))
    {
        if let Some(clean_cond) = extract_balanced_rust_assert_cond(rest) {
            let formatted_cond = clean_cond.replace('"', "'");
            if !formatted_cond.is_empty() && !invariants.contains(&formatted_cond) {
                invariants.push(formatted_cond);
            }
        }
    }
}

/// Extracts struct fields, enum variants, or trait methods from a Rust nominal declaration.
/// Realises: [REQ-FEAT-42-INSPECT-RUST-ITEM-DETAILS]
fn inspect_rust_nominal_members(
    raw_lines: &[&str],
    start_idx: usize,
    kind: SourceItemKind,
) -> Vec<String> {
    let mut fields_or_variants = Vec::new();
    let (mut brace_depth, mut saw_open_brace) = (0i32, false);
    if let Some(window) = raw_lines.get(start_idx..) {
        for &line in window {
            let t = strip_line_comment_preserving_strings(line.trim());
            let depth_before = brace_depth;
            let (delta, has_open) = count_rust_line_brace_delta(t);
            saw_open_brace |= has_open;
            brace_depth = brace_depth.saturating_add(delta);

            if depth_before == 1
                && !t.is_empty()
                && !t.starts_with('}')
                && !t.starts_with('#')
                && !t.starts_with("///")
                && !t.starts_with("//!")
            {
                collect_single_rust_nominal_member(t, kind, &mut fields_or_variants);
            }
            if (saw_open_brace && brace_depth <= 0) || (!saw_open_brace && t.ends_with(';')) {
                break;
            }
        }
    }
    fields_or_variants
}

/// Parses one struct field, enum variant, or trait method line into `fields_or_variants`.
/// Realises: [REQ-FEAT-42-INSPECT-RUST-ITEM-DETAILS]
fn collect_single_rust_nominal_member(
    t: &str,
    kind: SourceItemKind,
    fields_or_variants: &mut Vec<String>,
) {
    if kind == SourceItemKind::Struct {
        let after_vis = t
            .strip_prefix("pub(crate) ")
            .or_else(|| t.strip_prefix("pub(super) "))
            .or_else(|| t.strip_prefix("pub "))
            .unwrap_or(t)
            .trim_start();
        if let Some((fname, ftype)) = after_vis.split_once(':') {
            if let Some(id) = extract_leading_ast_ident(fname.trim()) {
                let entry = format!("{}: {}", id, ftype.trim().trim_end_matches(',').trim());
                if !fields_or_variants.contains(&entry) {
                    fields_or_variants.push(entry);
                }
            }
        }
    } else if kind == SourceItemKind::Enum {
        if let Some(v) = extract_leading_ast_ident(t) {
            if !fields_or_variants.contains(&v) {
                fields_or_variants.push(v);
            }
        }
    } else if kind == SourceItemKind::Trait {
        if let Some((_, fn_rest)) = t.split_once("fn ") {
            if let Some(m) = extract_leading_ast_ident(fn_rest.trim()) {
                let entry = format!("fn {m}");
                if !fields_or_variants.contains(&entry) {
                    fields_or_variants.push(entry);
                }
            }
        }
    }
}

/// Extracts the first argument expression of a `assert!(...)` or `debug_assert!(...)` invocation.
/// Realises: [REQ-FEAT-42-EXTRACT-BALANCED-RUST-ASSERT-COND]
#[must_use]
pub(crate) fn extract_balanced_rust_assert_cond(after_open_paren: &str) -> Option<&str> {
    let (mut paren_depth, mut bracket_depth, mut brace_depth) = (1i32, 0i32, 0i32);
    let (mut in_string, mut in_char, mut escaped) = (false, false, false);
    let mut offset: usize = 0;
    for ch in after_open_paren.chars() {
        let idx = offset;
        offset = offset.saturating_add(ch.len_utf8());
        if escaped {
            escaped = false;
            continue;
        }
        if in_string {
            escaped = ch == '\\';
            in_string = ch != '"';
            continue;
        }
        if in_char {
            escaped = ch == '\\';
            in_char = ch != '\'';
            continue;
        }
        match ch {
            '"' => in_string = true,
            '\'' => {
                in_char = after_open_paren
                    .get(idx..)
                    .and_then(single_char_literal_byte_len)
                    .is_some();
            }
            '(' => paren_depth = paren_depth.saturating_add(1),
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                if paren_depth == 0 {
                    return after_open_paren.get(..idx).map(str::trim);
                }
            }
            '[' => bracket_depth = bracket_depth.saturating_add(1),
            ']' => bracket_depth = bracket_depth.saturating_sub(1).max(0),
            '{' => brace_depth = brace_depth.saturating_add(1),
            '}' => brace_depth = brace_depth.saturating_sub(1).max(0),
            ',' if paren_depth == 1 && bracket_depth == 0 && brace_depth == 0 => {
                return after_open_paren.get(..idx).map(str::trim);
            }
            _ => {}
        }
    }
    None
}

/// Computes the net `{` minus `}` brace depth delta on a comment-stripped Rust line.
/// Realises: [REQ-FEAT-42-COUNT-RUST-LINE-BRACE-DELTA]
#[must_use]
pub(crate) fn count_rust_line_brace_delta(code: &str) -> (i32, bool) {
    let (mut delta, mut saw_open) = (0i32, false);
    let (mut in_string, mut in_char, mut escaped) = (false, false, false);
    let mut offset: usize = 0;
    for ch in code.chars() {
        let idx = offset;
        offset = offset.saturating_add(ch.len_utf8());
        if escaped {
            escaped = false;
            continue;
        }
        if in_string {
            escaped = ch == '\\';
            in_string = ch != '"';
            continue;
        }
        if in_char {
            escaped = ch == '\\';
            in_char = ch != '\'';
            continue;
        }
        match ch {
            '"' => in_string = true,
            '\'' => {
                in_char = code.get(idx..).and_then(single_char_literal_byte_len).is_some();
            }
            '{' => {
                delta = delta.saturating_add(1);
                saw_open = true;
            }
            '}' => delta = delta.saturating_sub(1),
            _ => {}
        }
    }
    (delta, saw_open)
}

/// Context bundle for synthesizing and recording a Rust item docblock in `SourceLineEdits`.
/// Realises: [REQ-FEAT-42-RUST-INJECTOR-EMIT-DOC]
pub(crate) struct RustEmitDocRequest<'a> {
    pub file_rel: &'a str,
    pub module_tag: &'a str,
    pub owner_name: Option<&'a str>,
    pub item: &'a ClassifiedRustItem,
    pub ordinal_index: usize,
    pub first_attr_idx: Option<usize>,
    pub current_doc: &'a [String],
    pub pending_doc_indices: &'a [usize],
    pub schema_model: Option<&'a PackageDef>,
}

/// Synthesizes a Tier A or Tier B/C Rust docstring and records the surgical line edit in `edits`.
/// Realises: [REQ-FEAT-42-RUST-INJECTOR-EMIT-DOC]
pub(crate) fn emit_synthesized_rust_item_doc(
    raw_lines: &[&str],
    scan: &RustLineScanContext<'_>,
    req: &RustEmitDocRequest<'_>,
    seen_docblocks: &mut BTreeSet<String>,
    edits: &mut SourceLineEdits,
) {
    edits.mark_removed(req.pending_doc_indices);
    let inline_comment_split = if scan.inline_comment_closed && req.first_attr_idx.is_none() {
        scan.raw_line
            .split_once("*/")
            .map(|(prefix, rest)| (format!("{prefix}*/"), rest.trim_start()))
    } else {
        None
    };
    let indent = if inline_comment_split.is_some() {
        ""
    } else {
        leading_whitespace(scan.raw_line)
    };
    let (params, return_type, fields_or_variants, invariants) =
        if req.item.keyword_override.is_some() {
            (Vec::new(), None, Vec::new(), Vec::new())
        } else {
            inspect_rust_item_details(raw_lines, scan.idx, req.item.kind)
        };
    let ctx = ItemSynthesisContext {
        file_rel: req.file_rel,
        module_tag: req.module_tag,
        owner_name: req.owner_name,
        item_kind: req.item.kind,
        item_kw_override: req.item.keyword_override,
        item_name: &req.item.name,
        ordinal_index: req.ordinal_index,
        indent,
        existing_doc: req.current_doc,
        params,
        return_type,
        fields_or_variants,
        invariants,
        schema_model: req.schema_model,
    };
    let new_doc = if req.item.is_tier_a {
        synthesize_7_section_docstring(&ctx, seen_docblocks)
    } else {
        synthesize_tier_bc_docstring(&ctx)
    };
    let target_insert_idx = req.first_attr_idx.unwrap_or(scan.idx);
    if let Some((comment_prefix, decl_tail)) = inline_comment_split {
        let mut combined = Vec::with_capacity(new_doc.len().saturating_add(2));
        combined.push(comment_prefix);
        combined.extend(new_doc);
        combined.push(format!("{indent}{decl_tail}"));
        edits.insert_before(target_insert_idx, combined);
        edits.mark_removed(&[scan.idx]);
    } else {
        edits.insert_before(target_insert_idx, new_doc);
    }
    edits.enriched_count = edits.enriched_count.saturating_add(1);
}
