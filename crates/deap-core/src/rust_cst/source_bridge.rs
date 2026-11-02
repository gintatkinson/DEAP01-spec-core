//! Unified source language AST bridge structures and Dart CST delegation helpers. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D qualification standards.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Item classification and doc extraction are pure, deterministic, and side-effect free.
//!
//! ## 3. Preconditions
//! - Input source buffers are valid UTF-8 strings; file paths use standard OS extensions.
//!
//! ## 4. Postconditions
//! - Returns normalized `ParsedSourceFileAst` and `SourceAstItem` records without panicking.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - $O(N)$ time and space complexity with respect to source line count $N$.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Unrecognized extensions or declarations return `None`; out-of-range lookups return defaults.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0201], [REQ-0602]

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use std::path::Path;

use logos::Logos;

use crate::dart_cst::{DartItemKind, DartToken};
pub use super::source_bridge_helpers::*;
pub use super::source_bridge_rust::{
    classify_rust_item_ast, classify_rust_item_line_ast, classify_rust_item_with_tier_ast,
    contains_direct_range_slice, extract_component_method_table, parse_dart_source_ast,
    parse_rust_source_ast, parse_rust_source_file_ast, parse_source_file_ast,
    validate_source_syntax, validate_source_syntax_for_language, ComponentMethodTable,
    ExtractedMethodSignature,
};

/// Supported source languages for CST/AST governance analysis. Realises: [REQ-0201]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SourceLanguage {
    /// Rust source file (`.rs`). Realises: [REQ-0201]
    #[default]
    Rust,
    /// Dart source file (`.dart`). Realises: [REQ-0201]
    Dart,
    /// Automatic language detection from file extension or content. Realises: [REQ-0201]
    Auto,
}

impl SourceLanguage {
    /// Detects the source language from a file path extension. Realises: [REQ-0201]
    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "rs" => Some(Self::Rust),
            "dart" => Some(Self::Dart),
            _ => None,
        }
    }

    /// Detects the source language from a file path extension (alias). Realises: [REQ-0201]
    #[must_use]
    pub fn detect_from_extension(path: &Path) -> Option<Self> {
        Self::from_path(path)
    }

    /// Returns human-readable lowercase language identifier. Realises: [REQ-0201]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Dart => "dart",
            Self::Auto => "auto",
        }
    }

    /// Parses source language from CLI flag or returns a typed `CoreError`. Realises: [REQ-0201]
    ///
    /// # Errors
    /// Returns `CoreError::Validation` if `flag` does not match a supported language name.
    pub fn from_flag(flag: &str) -> Result<Self, crate::error::CoreError> {
        match flag.to_ascii_lowercase().as_str() {
            "rust" | "rs" => Ok(Self::Rust),
            "dart" => Ok(Self::Dart),
            "auto" => Ok(Self::Auto),
            other => Err(crate::error::CoreError::validation(format!("unsupported language: '{other}'"))),
        }
    }
}

/// Normalized structural item kind across supported languages. Realises: [REQ-0201]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourceItemKind {
    /// Function or method declaration. Realises: [REQ-0201]
    Function,
    /// Struct, enum, union, type alias, class, or extension type. Realises: [REQ-0201]
    Type,
    /// Trait or Dart mixin declaration. Realises: [REQ-0201]
    TraitOrMixin,
    /// Implementation or extension block. Realises: [REQ-0201]
    Impl,
    /// Module declaration. Realises: [REQ-0201]
    Module,
    /// Macro definition. Realises: [REQ-0201]
    Macro,
    /// Constant or static item. Realises: [REQ-0201]
    Const,
    /// Struct declaration. Realises: [REQ-0201]
    Struct,
    /// Enum declaration. Realises: [REQ-0201]
    Enum,
    /// Trait declaration. Realises: [REQ-0201]
    Trait,
    /// Class declaration. Realises: [REQ-0201]
    Class,
    /// Mixin declaration. Realises: [REQ-0201]
    Mixin,
    /// Extension declaration. Realises: [REQ-0201]
    Extension,
    /// Typedef declaration. Realises: [REQ-0201]
    Typedef,
    /// Method declaration. Realises: [REQ-0201]
    Method,
    /// Constructor declaration. Realises: [REQ-0201]
    Constructor,
}

impl SourceItemKind {
    /// Returns the human-readable governance label for this item kind. Realises: [REQ-0201]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Type | Self::Typedef => "type",
            Self::TraitOrMixin | Self::Trait => "trait",
            Self::Impl => "impl",
            Self::Module => "module",
            Self::Macro => "macro",
            Self::Const => "const",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Class => "class",
            Self::Mixin => "mixin",
            Self::Extension => "extension",
            Self::Method => "method",
            Self::Constructor => "constructor",
        }
    }

    /// Returns the canonical source keyword label for `self`. Realises: [REQ-0201]
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Trait | Self::TraitOrMixin => "trait",
            Self::Class => "class",
            Self::Mixin => "mixin",
            Self::Extension => "extension",
            Self::Typedef | Self::Type => "type",
            Self::Function => "fn",
            Self::Method => "method",
            Self::Constructor => "constructor",
            Self::Impl => "impl",
            Self::Module => "mod",
            Self::Macro => "macro_rules",
            Self::Const => "const",
        }
    }

    /// Returns `true` if `self` represents a callable function, method, or constructor. Realises: [REQ-0201]
    #[must_use]
    pub const fn is_callable(self) -> bool {
        matches!(self, Self::Function | Self::Method | Self::Constructor)
    }
}

/// Formal input parameter extracted from a callable signature. Realises: [REQ-0201]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceParam {
    pub name: String,
    pub type_name: String,
}

/// Structural field or callback property extracted from a type declaration. Realises: [REQ-0201]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceField {
    pub name: String,
    pub type_name: String,
    pub is_callback_or_port: bool,
    pub doc: Option<String>,
}

/// Normalized AST item descriptor with line span and doc metadata. Realises: [REQ-0201]
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct SourceAstItem {
    pub kind: SourceItemKind,
    pub name: String,
    pub line: usize,
    pub line_no: usize,
    pub end_line: usize,
    pub doc_lines: Vec<String>,
    pub has_doc: bool,
    pub is_test_annotated: bool,
    pub is_inside_test_block: bool,
    pub is_trait_impl_member: bool,
    pub owner_name: Option<String>,
    pub parent_type: Option<String>,
    pub fields: Vec<SourceField>,
    pub params: Vec<SourceParam>,
    pub return_type: Option<String>,
    pub satisfied_requirements: Vec<String>,
    pub invariants: Vec<String>,
    pub states: Vec<String>,
}

/// Parsed source file representation with items and line masks. Realises: [REQ-0201]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSourceFileAst {
    /// Source language of the parsed file. Realises: [REQ-0201]
    pub language: SourceLanguage,
    /// Extracted structural items in source order. Realises: [REQ-0201]
    pub items: Vec<SourceAstItem>,
    /// Per-line boolean mask indicating executable code lines. Realises: [REQ-0201]
    pub code_line_mask: Vec<bool>,
    /// Per-line lexical brace depth before the line begins. Realises: [REQ-0201]
    pub brace_depths: Vec<usize>,
}

impl ParsedSourceFileAst {
    /// Returns true if the 0-based `line_index` is an active code line. Realises: [REQ-0201]
    #[must_use]
    pub fn is_code_line(&self, line_index: usize) -> bool {
        self.code_line_mask.get(line_index).copied().unwrap_or(false)
    }

    /// Returns the brace nesting depth prior to 0-based `line_index`. Realises: [REQ-0201]
    #[must_use]
    pub fn brace_depth_before_line(&self, line_index: usize) -> usize {
        self.brace_depths.get(line_index).copied().unwrap_or(0)
    }
}

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

/// Extracts contiguous `///` doc comments preceding `item_line_idx`. Realises: [REQ-0201]
#[must_use]
pub fn extract_item_doc_block_ast(lines: &[&str], item_line_idx: usize) -> (Vec<String>, bool) {
    let (mut docs, mut idx) = (Vec::new(), item_line_idx);
    while idx > 0 {
        idx -= 1;
        let trimmed = lines.get(idx).copied().unwrap_or("").trim();
        if let Some(rest) = trimmed.strip_prefix("///") {
            docs.push(rest.trim().to_string());
        } else if !trimmed.starts_with("#[") && !trimmed.starts_with('@') {
            break;
        }
    }
    docs.reverse();
    let has_doc = !docs.is_empty();
    (docs, has_doc)
}

/// Extracts requirement identifiers from `Realises: [...]` docstring tags. Realises: [REQ-0201]
#[must_use]
pub fn extract_realises_ids_from_doc_lines(doc_lines: &[String]) -> Vec<String> {
    let mut req_ids = Vec::new();
    for line in doc_lines {
        let trimmed = line.trim_start_matches('/').trim_start_matches('*').trim();
        if let Some(pos) = trimmed.find("Realises:") {
            let before_pipe = trimmed.get(pos.saturating_add(9)..).unwrap_or("").split('|').next().unwrap_or("").trim();
            for group in before_pipe.split('[').skip(1) {
                if let Some((inner, _)) = group.split_once(']') {
                    for raw_id in inner.split(',') {
                        let clean = raw_id.trim();
                        if !clean.is_empty() && !req_ids.iter().any(|r: &String| r == clean) {
                            req_ids.push(clean.to_string());
                        }
                    }
                }
            }
        }
    }
    req_ids
}
