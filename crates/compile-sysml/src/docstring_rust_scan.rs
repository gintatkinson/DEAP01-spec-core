#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] Rust CST/AST item classification, attribute scanning, and `impl` header resolution.
//! Safety & Traceability: Realises: [REQ-FEAT-42-CLASSIFY-RUST-ITEM-FOR-INJECTION], [REQ-0605]
//! Contract: Classifies Rust declarations into Tier A vs Tier B/C and resolves `InherentImpl` vs `TraitImpl` targets.
//! Bounds & Faults: O(L) time and space in scanned declaration lines `L`; zero panics or unbounded slices.

use crate::docstring_cst::{
    classify_rust_item_line_ast, extract_leading_ast_ident, extract_rust_impl_header_prefix,
    parse_rust_impl_header, CstLexState, SourceItemKind,
};

/// Classification of an enclosing Rust block on `impl_owner_stack` for Tier A versus Tier B/C resolution.
/// Realises: [REQ-FEAT-42-RUST-OWNER-KIND]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RustOwnerKind {
    InherentImpl,
    TraitImpl,
    TraitDef,
}

/// Pending test-related attribute classification attached to the next Rust declaration.
/// Realises: [REQ-FEAT-42-RUST-PENDING-ATTR]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RustPendingAttr {
    None,
    CfgTest,
    TestFn,
}

/// Classified Rust item declaration with its documentation tier (`is_tier_a`).
/// Realises: [REQ-FEAT-42-CLASSIFIED-RUST-ITEM]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClassifiedRustItem {
    pub kind: SourceItemKind,
    pub keyword_override: Option<&'static str>,
    pub name: String,
    pub is_tier_a: bool,
}

/// Intermediate state for a multi-line Rust `impl` header prior to its opening `{` brace.
/// Realises: [REQ-FEAT-42-PENDING-RUST-IMPL]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingRustImpl {
    pub target: String,
    pub kind: RustOwnerKind,
    pub in_where_clause: bool,
}

/// Checks whether `lex_state` is outside block comments, raw strings, active strings, and interpolations.
/// Realises: [REQ-FEAT-42-IS-LEX-STATE-NORMAL]
#[must_use]
pub(crate) fn is_lex_state_normal(lex_state: &CstLexState) -> bool {
    let outside_strings = lex_state.raw_hash_count.is_none() && lex_state.active_string.is_none();
    lex_state.block_comment_depth == 0 && outside_strings && lex_state.interp_stack.is_empty()
}

/// Checks whether `code` opens an inline `mod tests` or `#[cfg(test)] mod ...` block.
/// Realises: [REQ-FEAT-42-IS-MOD-TESTS-ENTRY]
#[must_use]
pub(crate) fn is_mod_tests_entry(code: &str, pending_attr: RustPendingAttr) -> bool {
    let is_named_tests =
        code == "mod tests {" || code.starts_with("mod tests ") || code.starts_with("mod tests{");
    let is_cfg_test_mod = pending_attr == RustPendingAttr::CfgTest
        && code.starts_with("mod ")
        && !code.ends_with(';');
    is_named_tests || is_cfg_test_mod
}

/// Checks whether a leading `#[...]` attribute line marks a `#[test]` entrypoint function.
/// Realises: [REQ-FEAT-42-IS-RUST-TEST-FN-ATTR]
fn is_rust_test_fn_attr(code: &str) -> bool {
    let Some(after_open) = code.trim_start().strip_prefix("#[") else {
        return false;
    };
    let Some((attr_body, _)) = after_open.split_once(']') else {
        return false;
    };
    let attr = attr_body.trim();
    let base = attr.split_once('(').map_or(attr, |(head, _)| head.trim());
    base == "test" || base.ends_with("::test")
}

/// Updates the pending test attribute state when scanning a Rust `#[...]` attribute line.
/// Realises: [REQ-FEAT-42-CLASSIFY-PENDING-RUST-ATTR]
#[must_use]
pub(crate) fn classify_pending_rust_attr(code: &str, current: RustPendingAttr) -> RustPendingAttr {
    if code.starts_with("#[cfg(test)") || code.contains("cfg(test)") || current == RustPendingAttr::CfgTest {
        RustPendingAttr::CfgTest
    } else if is_rust_test_fn_attr(code) {
        RustPendingAttr::TestFn
    } else {
        current
    }
}

/// Updates `attr_bracket_depth` across `[` and `]` bytes in a multi-line Rust attribute continuation.
/// Realises: [REQ-FEAT-42-UPDATE-ATTR-BRACKET-DEPTH]
pub(crate) fn update_attr_bracket_depth(code: &str, attr_bracket_depth: &mut usize) {
    for b in code.bytes() {
        if b == b'[' {
            *attr_bracket_depth = attr_bracket_depth.saturating_add(1);
        } else if b == b']' {
            *attr_bracket_depth = attr_bracket_depth.saturating_sub(1);
        }
    }
}

/// Consumes leading `#[...]` attributes on `code`, updating `attr_bracket_depth` if an attribute spans lines.
/// Realises: [REQ-FEAT-42-CONSUME-LEADING-RUST-ATTRS]
#[must_use]
pub(crate) fn consume_leading_rust_attrs<'b>(code: &'b str, attr_bracket_depth: &mut usize) -> &'b str {
    let mut rest = code.trim_start();
    while let Some(after_hash) = rest.strip_prefix("#[") {
        let mut depth: usize = 1;
        let mut close_off: Option<usize> = None;
        for (i, b) in after_hash.bytes().enumerate() {
            if b == b'[' {
                depth = depth.saturating_add(1);
            } else if b == b']' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    close_off = Some(i.saturating_add(1));
                    break;
                }
            }
        }
        if let Some(off) = close_off {
            rest = after_hash.get(off..).unwrap_or("").trim_start();
        } else {
            *attr_bracket_depth = depth;
            return "";
        }
    }
    rest
}

/// Extracts the target type identifier and owner kind (`InherentImpl` vs `TraitImpl`) from a Rust `impl` header.
/// Realises: [REQ-0605], [REQ-FEAT-42-EXTRACT-IMPL-TARGET]
#[must_use]
pub(crate) fn extract_rust_impl_target_and_kind(code: &str) -> Option<(String, RustOwnerKind)> {
    extract_rust_impl_header_prefix(code)?;
    let (trait_for_tail, _) = scan_rust_impl_segment(code);
    let kind = if trait_for_tail.is_some() {
        RustOwnerKind::TraitImpl
    } else {
        RustOwnerKind::InherentImpl
    };
    if let Some(target) = parse_rust_impl_header(code) {
        Some((target, kind))
    } else if kind == RustOwnerKind::TraitImpl {
        let fallback_target = trait_for_tail
            .and_then(extract_target_from_trait_for_tail)
            .unwrap_or_default();
        Some((fallback_target, kind))
    } else {
        None
    }
}

/// Scans a Rust `impl` header line segment for a top-level trait-separating `for` keyword and `where` clause.
/// Realises: [REQ-FEAT-42-SCAN-RUST-IMPL-SEGMENT]
#[must_use]
pub(crate) fn scan_rust_impl_segment(segment: &str) -> (Option<&str>, bool) {
    let (mut angle_depth, mut paren_depth, mut bracket_depth) = (0usize, 0usize, 0usize);
    let (mut trait_for_tail, mut saw_where) = (None, false);
    let (mut offset, mut skip_until) = (0usize, 0usize);

    for ch in segment.chars() {
        let idx = offset;
        offset = offset.saturating_add(ch.len_utf8());
        if idx < skip_until {
            continue;
        }
        if ch.is_ascii_alphanumeric() || ch == '_' {
            let rest = segment.get(idx..).unwrap_or("");
            let word_len = rest
                .bytes()
                .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
                .count();
            let end_idx = idx.saturating_add(word_len);
            skip_until = end_idx;
            let is_raw = idx > 0 && segment.as_bytes().get(idx.saturating_sub(1)) == Some(&b'#');
            if !is_raw && angle_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                let word = rest.get(..word_len).unwrap_or("");
                if word == "where" {
                    saw_where = true;
                    break;
                }
                if word == "for" && trait_for_tail.is_none() {
                    let after_for = segment.get(end_idx..).unwrap_or("").trim_start();
                    let is_hrtb = after_for.strip_prefix('<').is_some_and(|after_lt| {
                        let t = after_lt.trim_start();
                        t.starts_with('\'') || t.starts_with('>')
                    });
                    if !is_hrtb {
                        trait_for_tail = Some(after_for);
                    }
                }
            }
        } else {
            match ch {
                '-' if segment.get(offset..).is_some_and(|s| s.starts_with('>')) => {
                    skip_until = offset.saturating_add(1);
                }
                '<' => angle_depth = angle_depth.saturating_add(1),
                '>' => angle_depth = angle_depth.saturating_sub(1),
                '(' => paren_depth = paren_depth.saturating_add(1),
                ')' => paren_depth = paren_depth.saturating_sub(1),
                '[' => bracket_depth = bracket_depth.saturating_add(1),
                ']' => bracket_depth = bracket_depth.saturating_sub(1),
                '{' if angle_depth == 0 && paren_depth == 0 && bracket_depth == 0 => break,
                _ => {}
            }
        }
    }
    (trait_for_tail, saw_where)
}

/// Extracts the leading nominal target type identifier from the slice following `for` in `impl Trait for Type`.
/// Realises: [REQ-FEAT-42-EXTRACT-TARGET-FROM-TRAIT-FOR-TAIL]
#[must_use]
pub(crate) fn extract_target_from_trait_for_tail(after_for: &str) -> Option<String> {
    let before_brace = after_for.split_once('{').map_or(after_for, |(b, _)| b.trim());
    let before_where = before_brace
        .split_once(" where ")
        .map_or(before_brace, |(b, _)| b.trim())
        .trim_end_matches(" where")
        .trim();
    let mut clean = before_where.trim_start_matches('&').trim_start();
    if let Some(after_tick) = clean.strip_prefix('\'') {
        let lt_len = after_tick
            .bytes()
            .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
            .count();
        clean = after_tick.get(lt_len..).unwrap_or("").trim_start();
    }
    extract_leading_ast_ident(clean.strip_prefix("mut ").unwrap_or(clean).trim())
}

/// Advances a multi-line `PendingRustImpl` state across a continuation line and pushes onto `impl_owner_stack` on `{`.
/// Realises: [REQ-FEAT-42-ADVANCE-PENDING-RUST-IMPL]
pub(crate) fn advance_pending_rust_impl(
    code: &str,
    brace_depth: usize,
    pending_impl: &mut Option<PendingRustImpl>,
    impl_owner_stack: &mut Vec<(usize, String, RustOwnerKind)>,
) {
    if let Some(ref mut pending) = pending_impl {
        if !pending.in_where_clause {
            let (trait_for_tail, saw_where) = scan_rust_impl_segment(code);
            if let Some(after_for) = trait_for_tail {
                pending.kind = RustOwnerKind::TraitImpl;
                pending.target = extract_target_from_trait_for_tail(after_for).unwrap_or_default();
            }
            if saw_where {
                pending.in_where_clause = true;
            }
        }
    }
    if code.contains('{') && !code.contains('}') {
        if let Some(pending) = pending_impl.take() {
            impl_owner_stack.push((brace_depth, pending.target, pending.kind));
        }
    } else if code.contains(';') || code.contains('}') {
        *pending_impl = None;
    }
}

/// Strips restricted Rust visibility qualifiers (`pub(crate)`, `pub(super)`, `pub(self)`, `pub(in ...)`) from `code`.
/// Realises: [REQ-FEAT-42-STRIP-RESTRICTED-PUB-VISIBILITY]
fn strip_restricted_pub_visibility(code: &str) -> Option<&str> {
    let after_pub = code.strip_prefix("pub")?.trim_start();
    let after_open = after_pub.strip_prefix('(')?;
    let mut depth: usize = 1;
    for (idx, b) in after_open.bytes().enumerate() {
        if b == b'(' {
            depth = depth.saturating_add(1);
        } else if b == b')' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                let tail = after_open.get(idx.saturating_add(1)..).unwrap_or("").trim_start();
                return (!tail.is_empty()).then_some(tail);
            }
        }
    }
    None
}

/// Classifies a Rust declaration outside function bodies into a tiered `ClassifiedRustItem`.
/// Realises: [REQ-FEAT-42-CLASSIFY-RUST-ITEM-FOR-INJECTION]
#[must_use]
pub(crate) fn classify_rust_item_for_injection(
    code: &str,
    enclosing_owner_kind: Option<RustOwnerKind>,
    in_fn_body: bool,
) -> Option<ClassifiedRustItem> {
    if in_fn_body {
        return None;
    }
    let mut bracket_depth = 0usize;
    let clean = consume_leading_rust_attrs(code, &mut bracket_depth);
    let has_exact_pub = match clean.strip_prefix("pub") {
        Some(after_pub) if after_pub.starts_with(' ') || after_pub.starts_with('\t') => {
            !after_pub.trim_start().starts_with('(')
        }
        _ => false,
    };
    if let Some((kind, name)) = classify_rust_item_line_ast(clean)
        .or_else(|| strip_restricted_pub_visibility(clean).and_then(classify_rust_item_line_ast))
    {
        let is_tier_a = has_exact_pub
            && matches!(enclosing_owner_kind, None | Some(RustOwnerKind::InherentImpl));
        return Some(ClassifiedRustItem { kind, keyword_override: None, name, is_tier_a });
    }
    let (kw, name) = classify_rust_const_or_type_ast(clean, enclosing_owner_kind)?;
    Some(ClassifiedRustItem { kind: SourceItemKind::Struct, keyword_override: Some(kw), name, is_tier_a: false })
}

/// Classifies module-level or inherent-impl Rust `const` and `type` declarations as Tier B/C items.
/// Realises: [REQ-FEAT-42-CLASSIFY-RUST-CONST-OR-TYPE]
fn classify_rust_const_or_type_ast(
    clean: &str,
    enclosing_owner_kind: Option<RustOwnerKind>,
) -> Option<(&'static str, String)> {
    if matches!(enclosing_owner_kind, Some(RustOwnerKind::TraitImpl | RustOwnerKind::TraitDef)) {
        return None;
    }
    let after_vis = strip_restricted_pub_visibility(clean)
        .or_else(|| clean.strip_prefix("pub ").or_else(|| clean.strip_prefix("pub\t")).map(str::trim_start))
        .unwrap_or(clean);

    if let Some(after_const) = after_vis.strip_prefix("const ").or_else(|| after_vis.strip_prefix("const\t")) {
        let body = after_const.trim_start();
        let name = extract_leading_ast_ident(body)?;
        if name == "_" || matches!(name.as_str(), "fn" | "unsafe" | "extern") {
            return None;
        }
        let tail = body.get(name.len()..).unwrap_or("").trim_start();
        return tail.starts_with(':').then_some(("const", name));
    }
    let after_type = after_vis.strip_prefix("type ").or_else(|| after_vis.strip_prefix("type\t"))?;
    let body = after_type.trim_start();
    let name = extract_leading_ast_ident(body)?;
    if name == "_" {
        return None;
    }
    let tail = body.get(name.len()..).unwrap_or("").trim_start();
    let valid_tail = tail.starts_with('=') || tail.starts_with('<') || tail.starts_with(':');
    (valid_tail || tail.starts_with("where")).then_some(("type", name))
}
