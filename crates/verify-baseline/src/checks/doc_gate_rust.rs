#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Rust per-item docstring scanner and tier classifier for Check 36 (`audit_rust_source_docstrings`).
//! Realises: [REQ-CHECK-36-AUDIT-RUST-DOCSTRINGS]

use crate::checks::doc_gate_rust_classify::{
    classify_rust_item_with_tier, scan_pending_rust_owner_line, starts_rust_impl_block,
    PendingRustOwner, RustOwnerKind,
};
use crate::checks::doc_gate_validate::{
    contains_placeholder_docstring, update_line_delimiter_depths,
};
use crate::checks::docstring_gate::{validate_item_docblock, DocTier, ItemDocTarget};
use deap_core::rust_cst::source_bridge::{
    scan_line_with_unified_lex_state, CstLexState, ScopedItemScannerState, SourceLanguage,
};
use std::collections::BTreeSet;

/// Per-file mutable lexical and scope-tracking state used by `audit_rust_source_docstrings`.
/// Realises: [REQ-CHECK-36-RUST-AUDIT-STATE]
#[derive(Default)]
struct RustAuditState {
    current_doc: Vec<String>,
    seen_docblocks: BTreeSet<String>,
    scoped_state: ScopedItemScannerState,
    impl_owner_stack: Vec<(usize, RustOwnerKind)>,
    pending_owner: Option<PendingRustOwner>,
    attr_bracket_depth: usize,
    brace_depth: usize,
    paren_depth: usize,
    documented_items: usize,
    prev_ended_with_type_pos: bool,
}

/// Returns `true` when `file_rel` is an extracted `#[cfg(test)]` unit-test submodule file.
/// Realises: [REQ-CHECK-36-AUDIT-RUST-DOCSTRINGS]
#[must_use]
pub(crate) fn is_unit_test_submodule_file(file_rel: &str) -> bool {
    if file_rel.ends_with("safety_hazards_and_tests.rs") {
        return false;
    }
    let fname = std::path::Path::new(file_rel)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file_rel);
    fname == "tests.rs"
        || fname.ends_with("_tests.rs")
        || (fname.starts_with("tests_") && fname.ends_with(".rs"))
        || (fname.contains("_tests_") && fname.ends_with(".rs"))
}

/// Audits a single Rust source string for tiered per-item docstring completeness, traceability, and uniqueness.
/// Realises: [REQ-CHECK-36-AUDIT-RUST-DOCSTRINGS]
pub fn audit_rust_source_docstrings(source: &str, file_rel: &str) -> Result<usize, Vec<String>> {
    if is_unit_test_submodule_file(file_rel) {
        return Ok(0);
    }
    let mut violations = Vec::new();
    let mut state = RustAuditState::default();
    let mut lex_state = CstLexState {
        language: SourceLanguage::Rust,
        ..Default::default()
    };

    for (idx, raw_line) in source.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = raw_line.trim();
        let started_in_comment_or_string = lex_state.block_comment_depth > 0
            || lex_state.raw_hash_count.is_some()
            || lex_state.active_string.is_some()
            || !lex_state.interp_stack.is_empty();

        if !started_in_comment_or_string {
            if trimmed.starts_with("//!") {
                if contains_placeholder_docstring(trimmed) {
                    violations.push(format!(
                        "{}:{}: placeholder module docstring '{}' detected",
                        file_rel, line_no, trimmed
                    ));
                }
                continue;
            }
            if trimmed.starts_with("///") {
                if state.scoped_state.mod_tests_brace_depth.is_none()
                    && state.scoped_state.cfg_test_block_brace_depth.is_none()
                {
                    if contains_placeholder_docstring(trimmed) {
                        violations.push(format!(
                            "{}:{}: placeholder docstring '{}' detected",
                            file_rel, line_no, trimmed
                        ));
                    }
                    state.current_doc.push(trimmed.to_string());
                }
                continue;
            }
        }

        let scanned = scan_line_with_unified_lex_state(trimmed, &mut lex_state);
        let code = scanned.trim();
        if !code.is_empty() {
            process_rust_code_line(code, file_rel, line_no, &mut state, &mut violations);
        }
    }

    if violations.is_empty() {
        Ok(state.documented_items)
    } else {
        Err(violations)
    }
}

/// Processes a single comment-stripped, non-empty Rust code line during `audit_rust_source_docstrings`.
/// Realises: [REQ-CHECK-36-PROCESS-RUST-CODE-LINE]
fn process_rust_code_line(
    raw_code: &str,
    file_rel: &str,
    line_no: usize,
    state: &mut RustAuditState,
    violations: &mut Vec<String>,
) {
    if let Some(entry_depth) = state
        .scoped_state
        .mod_tests_brace_depth
        .or(state.scoped_state.cfg_test_block_brace_depth)
    {
        update_line_delimiter_depths(raw_code, &mut state.brace_depth, &mut state.paren_depth);
        pop_closed_rust_owners(state.brace_depth, &mut state.impl_owner_stack);
        if state.brace_depth <= entry_depth {
            state.scoped_state.mod_tests_brace_depth = None;
            state.scoped_state.cfg_test_block_brace_depth = None;
        }
        state.current_doc.clear();
        state.scoped_state.cfg_test_pending = false;
        state.pending_owner = None;
        state.prev_ended_with_type_pos = false;
        return;
    }

    if is_mod_tests_header(raw_code) {
        state.scoped_state.cfg_test_pending = true;
    }
    let Some(code) = consume_leading_rust_attributes(
        raw_code,
        &mut state.attr_bracket_depth,
        &mut state.scoped_state.cfg_test_pending,
    ) else {
        return;
    };
    if is_mod_tests_header(code) {
        state.scoped_state.cfg_test_pending = true;
    }
    if state.scoped_state.cfg_test_pending {
        let prev_brace_depth = state.brace_depth;
        let is_mod_tests = code.starts_with("mod tests");
        update_line_delimiter_depths(code, &mut state.brace_depth, &mut state.paren_depth);
        pop_closed_rust_owners(state.brace_depth, &mut state.impl_owner_stack);
        state.current_doc.clear();
        state.pending_owner = None;
        state.prev_ended_with_type_pos = false;
        if state.brace_depth > prev_brace_depth {
            if is_mod_tests {
                state.scoped_state.mod_tests_brace_depth = Some(prev_brace_depth);
            } else {
                state.scoped_state.cfg_test_block_brace_depth = Some(prev_brace_depth);
            }
            state.scoped_state.cfg_test_pending = false;
        } else if state.paren_depth == 0 && (code.ends_with(';') || code.ends_with('}')) {
            state.scoped_state.cfg_test_pending = false;
        }
        return;
    }

    dispatch_rust_declaration_or_owner(code, file_rel, line_no, state, violations);
    update_line_delimiter_depths(code, &mut state.brace_depth, &mut state.paren_depth);
    finalize_pending_rust_owner(code, state);
    pop_closed_rust_owners(state.brace_depth, &mut state.impl_owner_stack);
    state.prev_ended_with_type_pos =
        code.ends_with("->") || code.ends_with('+') || code.ends_with('=') || code.ends_with(':');
}

/// Checks whether a comment-stripped Rust line opens or declares a `mod tests` unit-test module.
/// Realises: [REQ-CHECK-36-IS-MOD-TESTS-HEADER]
fn is_mod_tests_header(code: &str) -> bool {
    code == "mod tests {"
        || code == "mod tests;"
        || code.starts_with("mod tests ")
        || code.starts_with("mod tests{")
}

/// Consumes multi-line or leading `#[...]` Rust attributes while preserving `current_doc` and flagging `cfg(test)`.
/// Realises: [REQ-CHECK-36-CONSUME-RUST-ATTRIBUTES]
fn consume_leading_rust_attributes<'a>(
    code: &'a str,
    attr_bracket_depth: &mut usize,
    cfg_test_pending: &mut bool,
) -> Option<&'a str> {
    if *attr_bracket_depth > 0 {
        if code.contains("cfg(test)") {
            *cfg_test_pending = true;
        }
        for b in code.bytes() {
            if b == b'[' {
                *attr_bracket_depth = attr_bracket_depth.saturating_add(1);
            } else if b == b']' {
                *attr_bracket_depth = attr_bracket_depth.saturating_sub(1);
            }
        }
        return None;
    }
    if !code.starts_with("#[") {
        return Some(code);
    }
    if code.starts_with("#[cfg(test)") || code.contains("cfg(test)") {
        *cfg_test_pending = true;
    }
    let mut rest = code;
    while let Some(after_hash) = rest.strip_prefix("#[") {
        let mut depth: usize = 1;
        let mut end_offset: Option<usize> = None;
        for (idx, b) in after_hash.bytes().enumerate() {
            if b == b'[' {
                depth = depth.saturating_add(1);
            } else if b == b']' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end_offset = Some(idx.saturating_add(1));
                    break;
                }
            }
        }
        if let Some(off) = end_offset {
            rest = after_hash.get(off..).unwrap_or("").trim_start();
        } else {
            *attr_bracket_depth = depth;
            return None;
        }
    }
    (!rest.is_empty()).then_some(rest)
}

/// Dispatches `code` across active `pending_owner` headers, `impl` block openers, or tiered Rust item declarations.
/// Realises: [REQ-CHECK-36-DISPATCH-RUST-DECLARATION]
fn dispatch_rust_declaration_or_owner(
    code: &str,
    file_rel: &str,
    line_no: usize,
    state: &mut RustAuditState,
    violations: &mut Vec<String>,
) {
    if let Some(ref mut pending) = state.pending_owner {
        if pending.kind != RustOwnerKind::TraitDef {
            scan_pending_rust_owner_line(code, pending);
        }
        state.current_doc.clear();
        return;
    }
    if state.paren_depth != 0 {
        return;
    }
    if !state.prev_ended_with_type_pos {
        if let Some(after_impl) = starts_rust_impl_block(code) {
            let mut pending = PendingRustOwner {
                base_brace_depth: state.brace_depth,
                kind: RustOwnerKind::InherentImpl,
                in_where_clause: false,
                angle_depth: 0,
                paren_depth: 0,
                bracket_depth: 0,
            };
            scan_pending_rust_owner_line(after_impl, &mut pending);
            state.pending_owner = Some(pending);
            state.current_doc.clear();
            return;
        }
    }
    let enclosing_owner_kind = state.impl_owner_stack.last().map(|&(_, kind)| kind);
    if let Some((kind, item_name, tier)) =
        classify_rust_item_with_tier(code, enclosing_owner_kind, &state.current_doc)
    {
        let target = ItemDocTarget {
            file_rel,
            line_no,
            kind,
            item_name: &item_name,
            tier,
            check_uniqueness: tier == DocTier::RustTierA && kind == "fn",
        };
        validate_item_docblock(
            &state.current_doc,
            &target,
            &mut state.seen_docblocks,
            violations,
        );
        state.documented_items = state.documented_items.saturating_add(1);
        if kind == "trait" {
            state.pending_owner = Some(PendingRustOwner {
                base_brace_depth: state.brace_depth,
                kind: RustOwnerKind::TraitDef,
                in_where_clause: true,
                angle_depth: 0,
                paren_depth: 0,
                bracket_depth: 0,
            });
        }
    }
    state.current_doc.clear();
}

/// Promotes an active `PendingRustOwner` onto `impl_owner_stack` once `{` opens a block, or clears it on `;`/`}`.
/// Realises: [REQ-CHECK-36-FINALIZE-PENDING-RUST-OWNER]
fn finalize_pending_rust_owner(code: &str, state: &mut RustAuditState) {
    if let Some(pending) = state.pending_owner {
        if state.brace_depth > pending.base_brace_depth {
            state
                .impl_owner_stack
                .push((pending.base_brace_depth, pending.kind));
            state.pending_owner = None;
        } else if state.paren_depth == 0 && (code.ends_with(';') || code.ends_with('}')) {
            state.pending_owner = None;
        }
    }
}

/// Pops all `impl` or `trait` owner entries from `impl_owner_stack` whose brace scope has closed.
/// Realises: [REQ-CHECK-36-POP-CLOSED-RUST-OWNERS]
fn pop_closed_rust_owners(brace_depth: usize, impl_owner_stack: &mut Vec<(usize, RustOwnerKind)>) {
    while let Some(&(open_depth, _)) = impl_owner_stack.last() {
        if brace_depth <= open_depth {
            impl_owner_stack.pop();
        } else {
            break;
        }
    }
}
