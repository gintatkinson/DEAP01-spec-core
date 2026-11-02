#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Lossless Rust CST item inventory and metric extraction. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A and ISO 26262 ASIL D qualification standards.
//! Provides typed CST item inventory, parameter counts, exec SLOC, and undocumented unsafe audits.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Item ordering follows deterministic CST pre-order traversal.
//! - Parameter count strictly excludes `self` receivers (`self`, `&self`, `&mut self`).
//! - Unsafe block verification flags any `unsafe { .. }` lacking a preceding `// SAFETY:` comment.
//!
//! ## 3. Preconditions
//! - Callers provide a valid `ParsedRustFile` representing UTF-8 Rust source.
//!
//! ## 4. Postconditions
//! - Returns deterministic `Vec<RustCstItem>` without side effects or panics.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear over total CST nodes in the file.
//! - Space Complexity: $O(K)$ proportional to extracted item count $K$.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Malformed items degrade gracefully with fallback names without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0608], [SC-01], [FLS: fls_cst_items]
//! SysML v2: part def RustCstItem

use crate::rust_cst::parse::ParsedRustFile;
use ra_ap_syntax::ast::{self, AstNode, HasAttrs, HasName, HasVisibility};
use ra_ap_syntax::{SyntaxKind, SyntaxNode, TextSize};
use std::collections::BTreeSet;

/// Classification of Rust CST item declarations.
/// Realises: [REQ-0608], [SC-01]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RustItemKind {
    Fn,
    Struct,
    Enum,
    Trait,
    Impl,
    Mod,
    Const,
    Static,
    TypeAlias,
    MacroRules,
}

/// Extracted Rust CST item metadata and DEAP v2.0 metrics.
/// Realises: [REQ-0608], [SC-01], [FLS: fls_cst_items]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustCstItem {
    pub kind: RustItemKind,
    pub name: String,
    pub is_pub: bool,
    pub start_line: usize,
    pub end_line: usize,
    pub fn_exec_lines: Option<usize>,
    pub fn_param_count: Option<usize>,
    pub doc_comment_lines: Vec<String>,
    pub attrs: Vec<String>,
    pub undocumented_unsafe_lines: Vec<usize>,
}

/// Extracts all typed Rust CST items from a parsed Rust source file.
/// Realises: [REQ-0608], [SC-01], [FLS: fls_cst_items]
#[must_use]
pub fn list_rust_items(parsed: &ParsedRustFile) -> Vec<RustCstItem> {
    let mut items = Vec::new();
    for node in parsed.syntax().descendants() {
        let Some(it) = ast::Item::cast(node) else {
            continue;
        };
        let Some(kind) = classify_item_kind(&it) else {
            continue;
        };
        items.push(extract_item(&it, kind, parsed));
    }
    items
}

/// Maps an AST item node to its corresponding `RustItemKind`.
/// Realises: [REQ-0608], [FLS: fls_cst_items]
fn classify_item_kind(item: &ast::Item) -> Option<RustItemKind> {
    match item {
        ast::Item::Fn(_) => Some(RustItemKind::Fn),
        ast::Item::Struct(_) => Some(RustItemKind::Struct),
        ast::Item::Enum(_) => Some(RustItemKind::Enum),
        ast::Item::Trait(_) => Some(RustItemKind::Trait),
        ast::Item::Impl(_) => Some(RustItemKind::Impl),
        ast::Item::Module(_) => Some(RustItemKind::Mod),
        ast::Item::Const(_) => Some(RustItemKind::Const),
        ast::Item::Static(_) => Some(RustItemKind::Static),
        ast::Item::TypeAlias(_) => Some(RustItemKind::TypeAlias),
        ast::Item::MacroRules(_) | ast::Item::MacroDef(_) => Some(RustItemKind::MacroRules),
        _ => None,
    }
}

/// Extracts the identifier name and public visibility status of an AST item.
/// Realises: [REQ-0608], [FLS: fls_cst_items]
fn extract_name_and_pub(it: &ast::Item, attrs: &[String]) -> (String, bool) {
    let (name, is_pub) = match it {
        ast::Item::Fn(f) => (f.name().map(|n| n.to_string()), f.visibility().is_some()),
        ast::Item::Struct(s) => (s.name().map(|n| n.to_string()), s.visibility().is_some()),
        ast::Item::Enum(e) => (e.name().map(|n| n.to_string()), e.visibility().is_some()),
        ast::Item::Trait(t) => (t.name().map(|n| n.to_string()), t.visibility().is_some()),
        ast::Item::Impl(i) => (Some(i.self_ty().map_or_else(|| "impl".to_string(), |t| t.to_string())), false),
        ast::Item::Module(m) => (m.name().map(|n| n.to_string()), m.visibility().is_some()),
        ast::Item::Const(c) => (c.name().map(|n| n.to_string()), c.visibility().is_some()),
        ast::Item::Static(s) => (s.name().map(|n| n.to_string()), s.visibility().is_some()),
        ast::Item::TypeAlias(t) => (t.name().map(|n| n.to_string()), t.visibility().is_some()),
        ast::Item::MacroRules(mr) => (mr.name().map(|n| n.to_string()), false),
        ast::Item::MacroDef(md) => (md.name().map(|n| n.to_string()), md.visibility().is_some()),
        _ => (None, false),
    };
    let is_pub = is_pub || attrs.iter().any(|a| a.contains("macro_export"));
    (name.unwrap_or_else(|| "<unnamed>".to_string()), is_pub)
}

/// Extracts full syntactic metadata for a single AST item.
/// Realises: [REQ-0608], [FLS: fls_cst_items]
fn extract_item(it: &ast::Item, kind: RustItemKind, parsed: &ParsedRustFile) -> RustCstItem {
    let attrs: Vec<String> = it.attrs().map(|a| a.syntax().text().to_string()).collect();
    let (name, is_pub) = extract_name_and_pub(it, &attrs);
    let text_range = it.syntax().text_range();
    let decl_offset = it
        .syntax()
        .children_with_tokens()
        .find(|el| !matches!(el.kind(), SyntaxKind::COMMENT | SyntaxKind::DOC_COMMENT | SyntaxKind::WHITESPACE | SyntaxKind::ATTR))
        .map_or(text_range.start(), |el| el.text_range().start());
    let start_line = parsed.line_col(decl_offset).0;
    let end_offset = if text_range.end() > text_range.start() { text_range.end() - TextSize::from(1) } else { text_range.start() };
    let end_line = parsed.line_col(end_offset).0;
    let (fn_exec_lines, fn_param_count) = match it {
        ast::Item::Fn(f) => (f.body().map(|b| fn_body_exec_lines(&b, parsed)), f.param_list().map(|pl| pl.params().count())),
        _ => (None, None),
    };
    let mut doc_comment_lines = Vec::new();
    for comment in it.doc_comments() {
        for line in comment.syntax().text().to_string().lines() {
            doc_comment_lines.push(line.to_string());
        }
    }
    let undocumented_unsafe_lines = find_undocumented_unsafe(it.syntax(), parsed);
    RustCstItem {
        kind,
        name,
        is_pub,
        start_line,
        end_line,
        fn_exec_lines,
        fn_param_count,
        doc_comment_lines,
        attrs,
        undocumented_unsafe_lines,
    }
}

/// Counts the executable lines within a function body excluding braces.
/// Realises: [REQ-0608], [FLS: fls_cst_items]
fn fn_body_exec_lines(body: &ast::BlockExpr, parsed: &ParsedRustFile) -> usize {
    let mut lines = BTreeSet::new();
    let l_curly = body.stmt_list().and_then(|s| s.l_curly_token());
    let r_curly = body.stmt_list().and_then(|s| s.r_curly_token());
    for el in body.syntax().descendants_with_tokens() {
        let Some(tok) = el.into_token() else { continue };
        if tok.kind() == SyntaxKind::WHITESPACE || tok.kind() == SyntaxKind::COMMENT {
            continue;
        }
        if Some(&tok) == l_curly.as_ref() || Some(&tok) == r_curly.as_ref() {
            continue;
        }
        lines.insert(parsed.line_col(tok.text_range().start()).0);
    }
    lines.len()
}

/// Discovers line numbers of unsafe blocks lacking a preceding SAFETY comment.
/// Realises: [REQ-0608], [FLS: fls_cst_items]
fn find_undocumented_unsafe(node: &SyntaxNode, parsed: &ParsedRustFile) -> Vec<usize> {
    let mut lines = Vec::new();
    for desc in node.descendants() {
        if let Some(block) = ast::BlockExpr::cast(desc) {
            if block.unsafe_token().is_some() && !has_safety_comment(&block) {
                if let Some(tok) = block.syntax().first_token() {
                    lines.push(parsed.line_col(tok.text_range().start()).0);
                }
            }
        }
    }
    lines.sort_unstable();
    lines.dedup();
    lines
}

/// Checks whether an unsafe block is immediately preceded by a SAFETY comment.
/// Realises: [REQ-0608], [FLS: fls_cst_items]
fn has_safety_comment(block: &ast::BlockExpr) -> bool {
    let Some(first_tok) = block.syntax().first_token() else { return false; };
    let mut cur = first_tok.prev_token();
    while let Some(tok) = cur {
        if tok.kind() == SyntaxKind::WHITESPACE {
            cur = tok.prev_token();
        } else if tok.kind() == SyntaxKind::COMMENT {
            if tok.text().contains("SAFETY:") { return true; }
            cur = tok.prev_token();
        } else {
            break;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_fn_with_metrics_and_receiver() -> Result<(), String> {
        let src = r#"
/// Docs for calculate
#[inline]
pub fn calculate(&self, a: i32, b: i32) -> i32 {
    let sum = a + b;
    sum
}
"#;
        let parsed = ParsedRustFile::parse_rust_source(src);
        let items = list_rust_items(&parsed);
        assert_eq!(items.len(), 1);
        let f = items.first().ok_or_else(|| "missing item".to_string())?;
        assert_eq!((f.kind, f.name.as_str(), f.is_pub), (RustItemKind::Fn, "calculate", true));
        assert_eq!((f.fn_param_count, f.fn_exec_lines), (Some(2), Some(2)));
        assert_eq!(f.doc_comment_lines, vec!["/// Docs for calculate"]);
        assert_eq!(f.attrs, vec!["#[inline]"]);
        assert!(f.undocumented_unsafe_lines.is_empty());
        Ok(())
    }

    #[test]
    fn test_undocumented_unsafe_detection() -> Result<(), String> {
        let src = "fn safe_and_unsafe() {\n    // SAFETY: verified\n    unsafe { let _ = 1; }\n    unsafe { let _ = 2; }\n}";
        let parsed = ParsedRustFile::parse_rust_source(src);
        let items = list_rust_items(&parsed);
        assert_eq!(items.len(), 1);
        let first = items.first().ok_or_else(|| "missing item".to_string())?;
        assert_eq!(first.undocumented_unsafe_lines, vec![4]);
        Ok(())
    }

    #[test]
    fn test_item_kinds_coverage() {
        let src = "struct S; enum E {} trait T {} const C: i32 = 1; static ST: i32 = 2; type A = u8;";
        let parsed = ParsedRustFile::parse_rust_source(src);
        let kinds: Vec<_> = list_rust_items(&parsed).into_iter().map(|i| i.kind).collect();
        assert_eq!(kinds, vec![
            RustItemKind::Struct, RustItemKind::Enum, RustItemKind::Trait,
            RustItemKind::Const, RustItemKind::Static, RustItemKind::TypeAlias,
        ]);
    }
}
