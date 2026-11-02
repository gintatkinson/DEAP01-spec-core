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
    parse_dart_source_file_ast, ParsedSourceFileAst, SourceAstItem, SourceField, SourceItemKind,
    SourceLanguage, SourceParam,
};

pub use crate::rust_cst::source_bridge::{
    ComponentMethodTable, ExtractedMethodSignature, extract_component_method_table,
};

/// Parses Rust source text into a normalized `ParsedSourceFileAst` via `ra_ap_syntax`. Realises: [REQ-0201]
#[must_use]
pub fn parse_rust_source_file_ast(source: &str) -> ParsedSourceFileAst {
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
    let mut items: Vec<SourceAstItem> = cst_items.into_iter().map(|i| map_cst_item(i, &test_spans, &trait_spans)).collect();
    enrich_rust_items_from_cst(&parsed, &mut items);
    let (code_line_mask, brace_depths) = compute_line_masks(&parsed, source.lines().count());
    ParsedSourceFileAst { language: SourceLanguage::Rust, items, code_line_mask, brace_depths }
}

/// Enriches mapped `SourceAstItem` records with struct fields, impl owners, callable signatures, and `assert!` invariants. Realises: [REQ-0201]
fn enrich_rust_items_from_cst(parsed: &ParsedRustFile, items: &mut [SourceAstItem]) {
    for node in parsed.syntax().descendants() {
        if let Some(st) = ast::Struct::cast(node.clone()) {
            let Some(st_name) = st.name().map(|n| n.to_string()) else { continue };
            if let Some(target) = items.iter_mut().find(|i| i.kind == SourceItemKind::Struct && i.name == st_name) {
                if let Some(ast::FieldList::RecordFieldList(rfl)) = st.field_list() {
                    for f in rfl.fields() {
                        let Some(name) = f.name().map(|n| n.to_string()) else { continue };
                        let type_name = f.ty().map(|t| t.syntax().text().to_string()).unwrap_or_default();
                        target.fields.push(SourceField { name, type_name, is_callback_or_port: false, doc: None });
                    }
                }
            }
        } else if let Some(fn_node) = ast::Fn::cast(node) {
            let Some(fn_name) = fn_node.name().map(|n| n.to_string()) else { continue };
            let end_line = parsed.line_col(fn_node.syntax().text_range().end()).0;
            let Some(target) = items.iter_mut().find(|i| i.kind.is_callable() && i.name == fn_name && i.end_line == end_line) else { continue };
            if let Some(imp) = fn_node.syntax().ancestors().find_map(ast::Impl::cast) {
                target.kind = SourceItemKind::Method;
                target.owner_name = imp.self_ty().map(|t| t.syntax().text().to_string().trim().to_string());
            }
            if let Some(plist) = fn_node.param_list() {
                for p in plist.params() {
                    let Some(name) = p.pat().map(|pat| pat.syntax().text().to_string()) else { continue };
                    let type_name = p.ty().map(|t| t.syntax().text().to_string()).unwrap_or_default();
                    target.params.push(SourceParam { name, type_name });
                }
            }
            target.return_type = fn_node.ret_type().and_then(|r| r.ty()).map(|t| t.syntax().text().to_string());
            if let Some(body) = fn_node.body() {
                for mc in body.syntax().descendants().filter_map(ast::MacroCall::cast) {
                    if mc.path().is_some_and(|p| p.syntax().text().to_string().ends_with("assert")) {
                        if let Some(tt) = mc.token_tree() {
                            let raw = tt.syntax().text().to_string();
                            let expr = raw.trim().trim_start_matches('(').trim_end_matches(')').trim().to_string();
                            if !expr.is_empty() { target.invariants.push(expr); }
                        }
                    }
                }
            }
        }
    }
}

/// Maps a single `RustCstItem` into a `SourceAstItem` with test and trait-impl context. Realises: [REQ-0201]
fn map_cst_item(item: RustCstItem, test_spans: &[(usize, usize)], trait_spans: &[(usize, usize)]) -> SourceAstItem {
    let kind = match item.kind {
        RustItemKind::Fn => SourceItemKind::Function,
        RustItemKind::Struct => SourceItemKind::Struct,
        RustItemKind::Enum => SourceItemKind::Enum,
        RustItemKind::TypeAlias => SourceItemKind::Typedef,
        RustItemKind::Trait => SourceItemKind::Trait,
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
    let satisfied_requirements = crate::rust_cst::source_bridge::extract_realises_ids_from_doc_lines(&doc_lines);
    let is_test_annotated = item.attrs.iter().any(|a| a.contains("test"));
    let is_inside_test_block = test_spans.iter().any(|&(s, e)| item.start_line >= s && item.end_line <= e);
    let is_trait_impl_member = kind != SourceItemKind::Impl && trait_spans.iter().any(|&(s, e)| item.start_line >= s && item.end_line <= e);
    SourceAstItem {
        kind,
        name: item.name,
        line: item.start_line,
        line_no: item.start_line,
        end_line: item.end_line,
        doc_lines,
        has_doc,
        is_test_annotated,
        is_inside_test_block,
        is_trait_impl_member,
        owner_name: None,
        parent_type: None,
        fields: Vec::new(),
        params: Vec::new(),
        return_type: None,
        satisfied_requirements,
        invariants: Vec::new(),
        states: Vec::new(),
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
            ast::Item::Struct(n) => (SourceItemKind::Struct, n.name()?.to_string(), n.visibility().is_some()),
            ast::Item::Enum(n) => (SourceItemKind::Enum, n.name()?.to_string(), n.visibility().is_some()),
            ast::Item::Trait(n) => (SourceItemKind::Trait, n.name()?.to_string(), n.visibility().is_some()),
            ast::Item::TypeAlias(n) => (SourceItemKind::Typedef, n.name()?.to_string(), n.visibility().is_some()),
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

/// Classifies a single Rust declaration line (`struct`, `enum`, `trait`, or `fn`) returning kind and identifier name. Realises: [REQ-0201]
#[must_use]
pub fn classify_rust_item_line_ast(trimmed: &str) -> Option<(SourceItemKind, String)> {
    let clean = trimmed.trim();
    if clean.is_empty() {
        return None;
    }
    let candidates = if clean.ends_with(';') || clean.ends_with('}') {
        [clean.to_string(), format!("{clean} {{}}")]
    } else if clean.ends_with('{') {
        [format!("{clean}}}"), format!("{clean};")]
    } else if clean.contains('(') && !clean.contains(')') {
        [format!("{clean}) {{}}"), format!("{clean});")]
    } else {
        [format!("{clean} {{}}"), format!("{clean};")]
    };
    for snippet in candidates {
        let tree = SourceFile::parse(&snippet, Edition::CURRENT).tree();
        let Some(item) = tree.syntax().children().find_map(ast::Item::cast) else { continue };
        let (kind, name) = match item {
            ast::Item::Fn(n) => (SourceItemKind::Function, n.name()?.to_string()),
            ast::Item::Struct(n) => (SourceItemKind::Struct, n.name()?.to_string()),
            ast::Item::Enum(n) => (SourceItemKind::Enum, n.name()?.to_string()),
            ast::Item::Trait(n) => (SourceItemKind::Trait, n.name()?.to_string()),
            _ => continue,
        };
        return Some((kind, name));
    }
    None
}

/// Classifies a public Rust declaration snippet into `(SourceItemKind, String)`. Realises: [REQ-0201]
#[must_use]
pub fn classify_rust_item_ast(trimmed: &str) -> Option<(SourceItemKind, String)> {
    classify_rust_item_with_tier_ast(trimmed, false)
}

/// Checks whether a code snippet contains an unchecked direct range slice index (`[a..b]`). Realises: [REQ-0602]
#[must_use]
pub fn contains_direct_range_slice(code: &str) -> bool {
    let parsed = SourceFile::parse(&format!("fn _f() {{ {code} }}"), Edition::CURRENT);
    for node in parsed.tree().syntax().descendants() {
        if let Some(index_expr) = ast::IndexExpr::cast(node) {
            if let Some(index) = index_expr.index() {
                if matches!(index, ast::Expr::RangeExpr(_)) {
                    return true;
                }
            }
        }
    }
    false
}

/// Dispatches file parsing by path extension to the appropriate CST bridge. Realises: [REQ-0201]
#[must_use]
pub fn parse_source_file_ast(path: &Path, source: &str) -> Option<ParsedSourceFileAst> {
    match SourceLanguage::from_path(path)? {
        SourceLanguage::Rust | SourceLanguage::Auto => Some(parse_rust_source_file_ast(source)),
        SourceLanguage::Dart => Some(parse_dart_source_file_ast(source)),
    }
}

/// Validates syntax for explicit language. Realises: [REQ-0201]
///
/// # Errors
/// Returns syntax error diagnostics if language-specific source is malformed.
pub fn validate_source_syntax_for_language(source: &str, file_label: &str, lang: SourceLanguage) -> Result<(), crate::error::CoreError> {
    if lang == SourceLanguage::Rust {
        let parsed = ParsedRustFile::parse_rust_source(source);
        if !parsed.syntax_errors().is_empty() {
            let (line, col) = parsed
                .syntax()
                .descendants_with_tokens()
                .filter_map(rowan::NodeOrToken::into_token)
                .find(|t| t.kind() == SyntaxKind::L_CURLY)
                .map_or((1, 1), |t| parsed.line_col(t.text_range().start()));
            return Err(crate::error::CoreError::syntax(format!(
                "{file_label}: syntax error at line {line}, column {col}: {:?}",
                parsed.syntax_errors()
            )));
        }
    } else if lang == SourceLanguage::Dart {
        use logos::Logos;
        let (mut braces, mut parens, mut brackets) = (0isize, 0isize, 0isize);
        for (res, span) in crate::dart_cst::DartToken::lexer(source).spanned() {
            let Ok(tok) = res else {
                return Err(crate::error::CoreError::syntax(format!("{file_label}: lexical error at {span:?}")));
            };
            if tok == crate::dart_cst::DartToken::Punct {
                match source.get(span).unwrap_or("") {
                    "{" => braces += 1,
                    "}" => braces -= 1,
                    "(" => parens += 1,
                    ")" => parens -= 1,
                    "[" => brackets += 1,
                    "]" => brackets -= 1,
                    _ => {}
                }
                if braces < 0 || parens < 0 || brackets < 0 {
                    return Err(crate::error::CoreError::syntax(format!("{file_label}: unbalanced closing delimiter")));
                }
            }
        }
        if braces != 0 || parens != 0 || brackets != 0 {
            return Err(crate::error::CoreError::syntax(format!("{file_label}: unclosed Dart delimiter")));
        }
        let items = crate::dart_cst::parse_dart_items(source);
        if items.iter().any(|item| item.name.is_empty()) {
            return Err(crate::error::CoreError::syntax(format!("{file_label}: empty Dart item identifier")));
        }
    }
    Ok(())
}

/// Validates syntax of source text. Realises: [REQ-0201]
///
/// # Errors
/// Returns syntax error diagnostics if source is malformed.
pub fn validate_source_syntax(source: &str, file_label: &str) -> Result<(), crate::error::CoreError> {
    let lang = match Path::new(file_label).extension().and_then(|e| e.to_str()) {
        Some("rs") => SourceLanguage::Rust,
        Some("dart") => SourceLanguage::Dart,
        _ => SourceLanguage::Auto,
    };
    validate_source_syntax_for_language(source, file_label, lang)
}

/// Parses Rust source file into `SourceAstItem` vector. Realises: [REQ-0201]
///
/// # Errors
/// Returns validation errors if the Rust source syntax is invalid.
pub fn parse_rust_source_ast(source: &str, file_label: &str) -> Result<Vec<SourceAstItem>, crate::error::CoreError> {
    validate_source_syntax_for_language(source, file_label, SourceLanguage::Rust)?;
    Ok(parse_rust_source_file_ast(source).items)
}

/// Parses Dart source file into `SourceAstItem` vector. Realises: [REQ-0201]
///
/// # Errors
/// Returns validation errors if the Dart source syntax is invalid.
pub fn parse_dart_source_ast(source: &str, file_label: &str) -> Result<Vec<SourceAstItem>, crate::error::CoreError> {
    validate_source_syntax_for_language(source, file_label, SourceLanguage::Dart)?;
    Ok(parse_dart_source_file_ast(source).items)
}
