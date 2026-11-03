#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Rust item and `impl`/`trait` owner header classifiers for Check 36 (`audit_rust_source_docstrings`).
//! Realises: [REQ-CHECK-36-AUDIT-RUST-DOCSTRINGS]

use crate::checks::doc_gate_dart::extract_leading_identifier;
use crate::checks::doc_gate_validate::{has_non_empty_summary_line, has_valid_four_block_contract};
use crate::checks::docstring_gate::{has_all_seven_sections, DocTier};

/// Enclosing Rust `impl` or `trait` block classification tracked across brace scopes in Check 36.
/// Realises: [REQ-CHECK-36-RUST-OWNER-KIND]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RustOwnerKind {
    InherentImpl,
    TraitImpl,
    TraitDef,
}

/// Incremental multi-line header scanner state for an unclosed Rust `impl` or `trait` block header.
/// Realises: [REQ-CHECK-36-PENDING-RUST-OWNER]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PendingRustOwner {
    pub(crate) base_brace_depth: usize,
    pub(crate) kind: RustOwnerKind,
    pub(crate) in_where_clause: bool,
    pub(crate) angle_depth: usize,
    pub(crate) paren_depth: usize,
    pub(crate) bracket_depth: usize,
}

/// Strips restricted Rust visibility qualifiers (`pub(crate)`, `pub(super)`, `pub(self)`, `pub(in ...)`) from `code`.
/// Realises: [REQ-CHECK-36-STRIP-RESTRICTED-PUB-VISIBILITY]
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
                let tail = after_open
                    .get(idx.saturating_add(1)..)
                    .unwrap_or("")
                    .trim_start();
                return (!tail.is_empty()).then_some(tail);
            }
        }
    }
    None
}

/// Classifies a comment-stripped Rust source line as a `struct`, `enum`, `trait`, or `fn` declaration.
/// Realises: [REQ-CHECK-36-CLASSIFY-RUST-ITEM]
pub(crate) fn classify_rust_item_line(code: &str) -> Option<(&'static str, String)> {
    let after_vis = if let Some(rest) = strip_restricted_pub_visibility(code) {
        rest
    } else if let Some(rest) = code.strip_prefix("pub ") {
        rest.trim_start()
    } else if let Some(rest) = code.strip_prefix("pub\t") {
        rest.trim_start()
    } else {
        code
    };

    if let Some(rest) = after_vis.strip_prefix("struct ") {
        return Some(("struct", extract_leading_identifier(rest.trim_start())?));
    }
    if let Some(rest) = after_vis.strip_prefix("enum ") {
        return Some(("enum", extract_leading_identifier(rest.trim_start())?));
    }
    let after_unsafe_trait = after_vis
        .strip_prefix("unsafe ")
        .map(str::trim_start)
        .unwrap_or(after_vis);
    if let Some(rest) = after_unsafe_trait.strip_prefix("trait ") {
        return Some(("trait", extract_leading_identifier(rest.trim_start())?));
    }

    let mut after_qual = after_vis;
    for _ in 0..4 {
        if let Some(rest) = after_qual.strip_prefix("const ") {
            after_qual = rest.trim_start();
        } else if let Some(rest) = after_qual.strip_prefix("async ") {
            after_qual = rest.trim_start();
        } else if let Some(rest) = after_qual.strip_prefix("unsafe ") {
            after_qual = rest.trim_start();
        } else if let Some(rest) = after_qual.strip_prefix("extern") {
            let rest_trimmed = rest.trim_start();
            if let Some(after_quote) = rest_trimmed.strip_prefix('"') {
                if let Some((_, after_abi)) = after_quote.split_once('"') {
                    after_qual = after_abi.trim_start();
                } else {
                    break;
                }
            } else if rest.starts_with(' ') || rest_trimmed.starts_with("fn ") {
                after_qual = rest_trimmed;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    if let Some(rest) = after_qual.strip_prefix("fn ") {
        return Some(("fn", extract_leading_identifier(rest.trim_start())?));
    }
    None
}

/// Detects whether `code` begins a Rust `impl` or `unsafe impl` block header and returns the slice after `impl`.
/// Realises: [REQ-CHECK-36-STARTS-RUST-IMPL-BLOCK]
pub(crate) fn starts_rust_impl_block(code: &str) -> Option<&str> {
    let clean = strip_leading_inline_rust_attrs(code);
    let after_unsafe = clean
        .strip_prefix("unsafe")
        .and_then(|rest| rest.starts_with(char::is_whitespace).then(|| rest.trim_start()))
        .unwrap_or(clean);
    let after_impl = after_unsafe.strip_prefix("impl")?;
    (after_impl.is_empty()
        || after_impl.starts_with('<')
        || after_impl.starts_with(char::is_whitespace))
    .then_some(after_impl)
}

/// Scans a single-line or continuation `impl` header segment to distinguish inherent `impl` from `impl Trait for Type`.
/// Realises: [REQ-CHECK-36-SCAN-PENDING-RUST-OWNER]
pub(crate) fn scan_pending_rust_owner_line(segment: &str, pending: &mut PendingRustOwner) {
    let bytes = segment.as_bytes();
    let mut idx: usize = 0;
    while let Some(&b) = bytes.get(idx) {
        if b.is_ascii_alphanumeric() || b == b'_' {
            let mut end_idx = idx.saturating_add(1);
            while let Some(&next_b) = bytes.get(end_idx) {
                if next_b.is_ascii_alphanumeric() || next_b == b'_' {
                    end_idx = end_idx.saturating_add(1);
                } else {
                    break;
                }
            }
            let is_raw_ident = idx > 0 && bytes.get(idx.saturating_sub(1)) == Some(&b'#');
            let at_top =
                pending.angle_depth == 0 && pending.paren_depth == 0 && pending.bracket_depth == 0;
            if !pending.in_where_clause && !is_raw_ident && at_top {
                let word = segment.get(idx..end_idx).unwrap_or("");
                if word == "where" {
                    pending.in_where_clause = true;
                } else if word == "for" && pending.kind == RustOwnerKind::InherentImpl {
                    let after_for = segment.get(end_idx..).unwrap_or("").trim_start();
                    let is_hrtb = after_for.strip_prefix('<').is_some_and(|after_lt| {
                        let t = after_lt.trim_start();
                        t.starts_with('\'') || t.starts_with('>')
                    });
                    if !is_hrtb {
                        pending.kind = RustOwnerKind::TraitImpl;
                    }
                }
            }
            idx = end_idx;
        } else {
            match b {
                b'-' if bytes.get(idx.saturating_add(1)) == Some(&b'>') => {
                    idx = idx.saturating_add(2);
                    continue;
                }
                b'<' => pending.angle_depth = pending.angle_depth.saturating_add(1),
                b'>' => pending.angle_depth = pending.angle_depth.saturating_sub(1),
                b'(' => pending.paren_depth = pending.paren_depth.saturating_add(1),
                b')' => pending.paren_depth = pending.paren_depth.saturating_sub(1),
                b'[' => pending.bracket_depth = pending.bracket_depth.saturating_add(1),
                b']' => pending.bracket_depth = pending.bracket_depth.saturating_sub(1),
                b'{' if pending.angle_depth == 0
                    && pending.paren_depth == 0
                    && pending.bracket_depth == 0 =>
                {
                    break;
                }
                _ => {}
            }
            idx = idx.saturating_add(1);
        }
    }
}

/// Strips any balanced leading inline `#[...]` attributes preceding a declaration on the same line.
/// Realises: [REQ-CHECK-36-STRIP-INLINE-RUST-ATTRS]
pub(crate) fn strip_leading_inline_rust_attrs(code: &str) -> &str {
    let mut rest = code.trim_start();
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
            break;
        }
    }
    rest
}

/// Classifies a Rust declaration line and computes its normative `DocTier` (`RustTierA` vs `RustTierBC`).
/// Realises: [REQ-CHECK-36-CLASSIFY-RUST-ITEM-WITH-TIER]
pub(crate) fn classify_rust_item_with_tier(
    code: &str,
    _enclosing_owner_kind: Option<RustOwnerKind>,
    current_doc: &[String],
) -> Option<(&'static str, String, DocTier)> {
    let clean = strip_leading_inline_rust_attrs(code);
    let (kind, item_name) = classify_rust_item_line(clean)?;
    let doc_joined = current_doc.join("\n");
    let tier = if super::docstring_tier_rules::has_explicit_tier_a_marker(current_doc)
        || has_valid_four_block_contract(current_doc)
        || (has_all_seven_sections(&doc_joined) && has_non_empty_summary_line(current_doc))
    {
        DocTier::RustTierA
    } else {
        DocTier::RustTierBC
    };
    Some((kind, item_name, tier))
}
