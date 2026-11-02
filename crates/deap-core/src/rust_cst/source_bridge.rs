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

use crate::dart_cst::DartItemKind;
pub use super::source_bridge_rust::{classify_rust_item_ast, classify_rust_item_with_tier_ast, parse_rust_source_ast, parse_source_file_ast};

/// Supported source languages for CST/AST governance analysis. Realises: [REQ-0201]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLanguage {
    /// Rust source file (`.rs`). Realises: [REQ-0201]
    Rust,
    /// Dart source file (`.dart`). Realises: [REQ-0201]
    Dart,
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
}

/// Normalized structural item kind across supported languages. Realises: [REQ-0201]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

impl SourceItemKind {
    /// Returns the human-readable governance label for this item kind. Realises: [REQ-0201]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Type => "type",
            Self::TraitOrMixin => "trait",
            Self::Impl => "impl",
            Self::Module => "module",
            Self::Macro => "macro",
            Self::Const => "const",
        }
    }
}

/// Normalized AST item descriptor with line span and doc metadata. Realises: [REQ-0201]
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct SourceAstItem {
    /// Structural category of the item. Realises: [REQ-0201]
    pub kind: SourceItemKind,
    /// Identifier name of the item. Realises: [REQ-0201]
    pub name: String,
    /// 1-based start line of the item declaration. Realises: [REQ-0201]
    pub line: usize,
    /// 1-based inclusive end line of the item declaration. Realises: [REQ-0201]
    pub end_line: usize,
    /// Extracted documentation comment lines preceding the item. Realises: [REQ-0201]
    pub doc_lines: Vec<String>,
    /// Whether at least one documentation comment line is attached. Realises: [REQ-0201]
    pub has_doc: bool,
    /// Whether the item carries a direct test annotation. Realises: [REQ-0201]
    pub is_test_annotated: bool,
    /// Whether the item resides inside a test module or block. Realises: [REQ-0201]
    pub is_inside_test_block: bool,
    /// Whether the item is a member of a trait implementation block. Realises: [REQ-0201]
    pub is_trait_impl_member: bool,
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

/// Parses Dart source into a normalized `ParsedSourceFileAst` via `dart_cst`. Realises: [REQ-0201]
#[must_use]
pub fn parse_dart_source_ast(source: &str) -> ParsedSourceFileAst {
    let lines: Vec<&str> = source.lines().collect();
    let items = crate::dart_cst::parse_dart_items(source)
        .into_iter()
        .map(|d| {
            let kind = match d.kind {
                DartItemKind::Class | DartItemKind::Enum | DartItemKind::TypeAlias => SourceItemKind::Type,
                DartItemKind::Mixin => SourceItemKind::TraitOrMixin,
                DartItemKind::Extension => SourceItemKind::Impl,
                DartItemKind::Function => SourceItemKind::Function,
                DartItemKind::Const => SourceItemKind::Const,
            };
            let (doc_lines, has_doc) = extract_item_doc_block_ast(&lines, d.start_line.saturating_sub(1));
            SourceAstItem {
                kind, name: d.name, line: d.start_line, end_line: d.end_line,
                doc_lines, has_doc, is_test_annotated: false, is_inside_test_block: false, is_trait_impl_member: false,
            }
        })
        .collect();
    let code_line_mask = lines.iter().map(|l| {
        let t = l.trim();
        !t.is_empty() && !t.starts_with("//") && !t.starts_with("/*") && !t.starts_with('*')
    }).collect();
    ParsedSourceFileAst { language: SourceLanguage::Dart, items, code_line_mask, brace_depths: vec![0; lines.len()] }
}

/// Classifies a single Dart top-level snippet using `dart_cst`. Realises: [REQ-0201]
#[must_use]
pub fn classify_dart_top_level_ast(trimmed: &str) -> Option<(SourceItemKind, String)> {
    let mut probe = trimmed.to_string();
    if !probe.ends_with('}') && !probe.ends_with(';') {
        probe.push_str(" {}");
    }
    parse_dart_source_ast(&probe).items.into_iter().next().map(|i| (i.kind, i.name))
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
