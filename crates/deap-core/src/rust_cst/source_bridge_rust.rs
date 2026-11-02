#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Lossless Rust CST to unified source AST bridge helpers. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D qualification standards.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Item classification, code line masks, and brace depths derive deterministically from `ra_ap_syntax`.
//!
//! ## 3. Preconditions
//! - Input source buffers are valid UTF-8 strings; file paths use standard extensions.
//!
//! ## 4. Postconditions
//! - Produces normalized `ParsedSourceFileAst` and `(SourceItemKind, String)` tuples without panicking.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - $O(N)$ time and space complexity with respect to CST token count $N$.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Unrecognized syntax snippets or unsupported file extensions return `None`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0201], [REQ-0608]

use std::path::Path;

use ra_ap_syntax::ast::{self, AstNode, HasName, HasVisibility};
use ra_ap_syntax::{Edition, SourceFile, SyntaxKind};

use crate::rust_cst::items::{list_rust_items, RustCstItem, RustItemKind};
use crate::rust_cst::parse::ParsedRustFile;
use crate::rust_cst::source_bridge::{
    parse_dart_source_ast, ParsedSourceFileAst, SourceAstItem, SourceItemKind, SourceLanguage,
};

/// Parses Rust source text into a normalized `ParsedSourceFileAst` via `ra_ap_syntax`. Realises: [REQ-0201]
#[must_use]
pub fn parse_rust_source_ast(source: &str) -> ParsedSourceFileAst {
    let parsed = ParsedRustFile::parse_rust_source(source);
    let cst_items = list_rust_items(&parsed);
    let test_spans: Vec<(usize, usize)> = cst_items
        .iter()
        .filter(|i| i.attrs.iter().any(|a| a.contains("cfg(test)")) || (i.kind == RustItemKind::Mod && i.name == "tests"))
        .map(|i| (i.start_line, i.end_line))
        .collect();
    let trait_spans: Vec<(usize, usize)> = parsed
        .syntax()
        .descendants()
        .filter_map(ast::Impl::cast)
        .filter(|imp| imp.trait_().is_some())
        .map(|imp| (parsed.line_col(imp.syntax().text_range().start()).0, parsed.line_col(imp.syntax().text_range().end()).0))
        .collect();
    let items = cst_items.into_iter().map(|i| map_cst_item(i, &test_spans, &trait_spans)).collect();
    let (code_line_mask, brace_depths) = compute_line_masks(&parsed, source.lines().count());
    ParsedSourceFileAst { language: SourceLanguage::Rust, items, code_line_mask, brace_depths }
}

/// Maps a single `RustCstItem` into a `SourceAstItem` with test and trait-impl context. Realises: [REQ-0201]
fn map_cst_item(item: RustCstItem, test_spans: &[(usize, usize)], trait_spans: &[(usize, usize)]) -> SourceAstItem {
    let kind = match item.kind {
        RustItemKind::Fn => SourceItemKind::Function,
        RustItemKind::Struct | RustItemKind::Enum | RustItemKind::TypeAlias => SourceItemKind::Type,
        RustItemKind::Trait => SourceItemKind::TraitOrMixin,
        RustItemKind::Impl => SourceItemKind::Impl,
        RustItemKind::Mod => SourceItemKind::Module,
        RustItemKind::MacroRules => SourceItemKind::Macro,
        RustItemKind::Const | RustItemKind::Static => SourceItemKind::Const,
    };
    let doc_lines: Vec<String> = item
        .doc_comment_lines
        .iter()
        .map(|l| {
            let t = l.trim();
            t.strip_prefix("///").unwrap_or(t).trim().to_string()
        })
        .collect();
    let has_doc = !doc_lines.is_empty();
    let is_test_annotated = item.attrs.iter().any(|a| a.contains("test"));
    let is_inside_test_block = test_spans.iter().any(|&(s, e)| item.start_line >= s && item.end_line <= e);
    let is_trait_impl_member = kind != SourceItemKind::Impl && trait_spans.iter().any(|&(s, e)| item.start_line >= s && item.end_line <= e);
    SourceAstItem {
        kind, name: item.name, line: item.start_line, end_line: item.end_line,
        doc_lines, has_doc, is_test_annotated, is_inside_test_block, is_trait_impl_member,
    }
}

/// Computes executable code line mask and pre-line brace depth from CST tokens. Realises: [REQ-0201]
fn compute_line_masks(parsed: &ParsedRustFile, line_count: usize) -> (Vec<bool>, Vec<usize>) {
    let (mut code_mask, mut deltas) = (vec![false; line_count], vec![0isize; line_count]);
    for elem in parsed.syntax().descendants_with_tokens() {
        let Some(tok) = elem.into_token() else { continue };
        if tok.kind() == SyntaxKind::WHITESPACE || tok.text().starts_with("//") || tok.text().starts_with("/*") {
            continue;
        }
        let start_idx = parsed.line_col(tok.text_range().start()).0.saturating_sub(1);
        let end_idx = parsed.line_col(tok.text_range().end()).0.saturating_sub(1);
        for slot in code_mask.iter_mut().take(end_idx + 1).skip(start_idx) {
            *slot = true;
        }
        if let Some(delta) = deltas.get_mut(start_idx) {
            if tok.kind() == SyntaxKind::L_CURLY {
                *delta += 1;
            } else if tok.kind() == SyntaxKind::R_CURLY {
                *delta -= 1;
            }
        }
    }
    let mut depth = 0isize;
    let brace_depths = deltas.into_iter().map(|d| {
        let prior = usize::try_from(depth.max(0)).unwrap_or(0);
        depth += d;
        prior
    }).collect();
    (code_mask, brace_depths)
}

/// Classifies a single Rust declaration line with optional private-item inclusion. Realises: [REQ-0201]
#[must_use]
pub fn classify_rust_item_with_tier_ast(trimmed: &str, allow_private: bool) -> Option<(SourceItemKind, String)> {
    let clean = trimmed.trim();
    if clean.is_empty() {
        return None;
    }
    let candidates = if clean.ends_with(';') || clean.ends_with('}') {
        [clean.to_string(), format!("{clean} {{}}")]
    } else if clean.ends_with('{') {
        [format!("{clean}}}"), format!("{clean};")]
    } else {
        [format!("{clean} {{}}"), format!("{clean};")]
    };
    for snippet in candidates {
        let tree = SourceFile::parse(&snippet, Edition::CURRENT).tree();
        let Some(item) = tree.syntax().children().find_map(ast::Item::cast) else { continue };
        let (kind, name, is_pub) = match item {
            ast::Item::Fn(n) => (SourceItemKind::Function, n.name()?.to_string(), n.visibility().is_some()),
            ast::Item::Struct(n) => (SourceItemKind::Type, n.name()?.to_string(), n.visibility().is_some()),
            ast::Item::Enum(n) => (SourceItemKind::Type, n.name()?.to_string(), n.visibility().is_some()),
            ast::Item::Trait(n) => (SourceItemKind::TraitOrMixin, n.name()?.to_string(), n.visibility().is_some()),
            ast::Item::TypeAlias(n) => (SourceItemKind::Type, n.name()?.to_string(), n.visibility().is_some()),
            ast::Item::Const(n) => (SourceItemKind::Const, n.name()?.to_string(), n.visibility().is_some()),
            ast::Item::Static(n) => (SourceItemKind::Const, n.name()?.to_string(), n.visibility().is_some()),
            ast::Item::Module(n) => (SourceItemKind::Module, n.name()?.to_string(), n.visibility().is_some()),
            _ => return None,
        };
        if kind == SourceItemKind::Function && name.starts_with("test_") {
            return None;
        }
        return (is_pub || allow_private).then_some((kind, name));
    }
    None
}

/// Classifies a public Rust declaration snippet into `(SourceItemKind, String)`. Realises: [REQ-0201]
#[must_use]
pub fn classify_rust_item_ast(trimmed: &str) -> Option<(SourceItemKind, String)> {
    classify_rust_item_with_tier_ast(trimmed, false)
}

/// Dispatches file parsing by path extension to the appropriate CST bridge. Realises: [REQ-0201]
#[must_use]
pub fn parse_source_file_ast(path: &Path, source: &str) -> Option<ParsedSourceFileAst> {
    match SourceLanguage::from_path(path)? {
        SourceLanguage::Rust => Some(parse_rust_source_ast(source)),
        SourceLanguage::Dart => Some(parse_dart_source_ast(source)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rust_source_ast_and_dispatch() {
        let src = "/// Doc line\npub fn compute(x: i32) -> i32 {\n    x + 1\n}\n";
        let ast = parse_source_file_ast(Path::new("lib.rs"), src);
        assert!(ast.is_some());
        let parsed = parse_rust_source_ast(src);
        assert_eq!(parsed.items.first().map(|i| (i.kind, i.has_doc)), Some((SourceItemKind::Function, true)));
        assert_eq!(parsed.code_line_mask, vec![false, true, true, true]);
    }

    #[test]
    fn test_classify_rust_item_visibility_and_kinds() {
        assert_eq!(classify_rust_item_ast("pub fn alpha()"), Some((SourceItemKind::Function, "alpha".to_string())));
        assert_eq!(classify_rust_item_ast("fn hidden()"), None);
        assert_eq!(classify_rust_item_with_tier_ast("fn hidden()", true), Some((SourceItemKind::Function, "hidden".to_string())));
        assert_eq!(classify_rust_item_with_tier_ast("pub fn test_case()", true), None);
    }
}
