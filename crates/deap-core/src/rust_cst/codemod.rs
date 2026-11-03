#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Lossless Rust CST item inventory and reverse-order disjoint span splicing codemod engine. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C qualification.
//! Implements deterministic CST item inventory (TD-14), extractable item cataloguing (TD-16), and reverse-order span splicing (TD-17).
//!
//! ## 2. Invariants & Mathematical Properties
//! - All item inventories use `BTreeMap<(u32, String), _>` and `BTreeSet<String>` for deterministic ordering.
//! - Span splicing validates UTF-8 character boundaries and pairwise disjointness before reverse-offset mutation.
//! - Post-splice output is re-parsed via `ra_ap_syntax::SourceFile::parse` to guarantee zero syntax corruption.
//!
//! ## 3. Preconditions
//! - Callers provide non-empty UTF-8 Rust source text and non-overlapping `[byte_start, byte_end)` spans.
//!
//! ## 4. Postconditions
//! - Returns verified `RustItemInventory`, `BTreeMap<String, RustItemDescriptor>`, or lossless spliced source.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Inventory: $O(N \log K)$ over $N$ CST tokens and $K$ top-level items.
//! - Splicing: $O(E \log E + N)$ over $E$ edits and $N$ source bytes.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Fails closed via `RustCstInventoryError` or `CodemodError` (`DEAP-E0801`..`DEAP-E0804`) without panics.
//!
//! ## 7. Realization & Traceability
//! Realises: [ZeroTokenDeterministicCodemodEngine/list_rust_cst_items], [REQ-0608]
//! SysML v2: part def ZeroTokenDeterministicCodemodEngine

use super::doc_tier::{node_enters_test_scope, DocTier};
use super::envelope::MAX_AST_DEPTH;
use ra_ap_syntax::ast::{self, AstNode, HasModuleItem, HasName, HasVisibility};
use ra_ap_syntax::{Edition, SourceFile};
use std::collections::{BTreeMap, BTreeSet};

/// Verified UTF-8 half-open byte span `[byte_start, byte_end)`.
/// Realises: [REQ-0608]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Utf8ByteSpan {
    pub byte_start: usize,
    pub byte_end: usize,
}

/// Top-level Rust CST item descriptor ordered by `(byte_start, name)`.
/// Realises: [REQ-0608]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RustCstItemDescriptor {
    pub name: String,
    pub kind: String,
    pub span: Utf8ByteSpan,
    pub is_pub: bool,
    pub doc_tier: DocTier,
}

/// Deterministic `BTreeMap`/`BTreeSet`-backed inventory of top-level Rust CST items.
/// Realises: [REQ-0608]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RustItemInventory {
    pub file_path: String,
    pub items: BTreeMap<(u32, String), RustCstItemDescriptor>,
    pub symbol_index: BTreeSet<String>,
}

/// Fail-closed error taxonomy for Rust CST item inventory extraction (`TD-14`).
/// Realises: [REQ-0608]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RustCstInventoryError {
    EmptySourceOrPath,
    SyntaxError(String),
    InvalidUtf8Span,
    IoError(String),
}

impl RustCstInventoryError {
    /// Returns canonical `DEAP-E0801`..`DEAP-E0804` diagnostic code.
    /// Realises: [REQ-0608]
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::EmptySourceOrPath => "DEAP-E0801",
            Self::SyntaxError(_) => "DEAP-E0802",
            Self::InvalidUtf8Span => "DEAP-E0803",
            Self::IoError(_) => "DEAP-E0804",
        }
    }
}

impl std::fmt::Display for RustCstInventoryError {
    /// Formats inventory error with diagnostic code. Realises: [REQ-0608]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] CST inventory error: {self:?}", self.diagnostic_code())
    }
}

impl std::error::Error for RustCstInventoryError {}

/// Configuration parameters for zero-token deterministic CST codemod operations.
/// Realises: [REQ-0608]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ZeroTokenCodemodConfig {
    pub max_ast_depth: usize,
    pub preserve_cfg_test: bool,
}

impl Default for ZeroTokenCodemodConfig {
    /// Returns canonical default codemod configuration (`max_ast_depth = 64`).
    /// Realises: [REQ-0608]
    fn default() -> Self {
        Self {
            max_ast_depth: MAX_AST_DEPTH,
            preserve_cfg_test: true,
        }
    }
}

/// Extractable Rust CST item descriptor with outer trivia span and `Realises:` bindings (`TD-16`).
/// Realises: [REQ-0608]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RustItemDescriptor {
    pub name: String,
    pub kind: String,
    pub span: Utf8ByteSpan,
    pub realises_reqs: BTreeSet<String>,
    pub doc_tier: DocTier,
}

/// Disjoint span replacement edit specification (`TD-17`).
/// Realises: [REQ-0608]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SpanSpliceEdit {
    pub span: Utf8ByteSpan,
    pub replacement: String,
}

/// Fail-closed error taxonomy for zero-token deterministic CST codemod operations (`TD-16`/`TD-17`).
/// Realises: [REQ-0608]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CodemodError {
    EmptyInput,
    OverlappingOrInvalidSpan,
    PostSpliceSyntaxError(String),
    ItemNotFound(String),
    IoError(String),
}

impl CodemodError {
    /// Returns canonical `DEAP-E0802`..`DEAP-E0804` diagnostic code.
    /// Realises: [REQ-0608]
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::EmptyInput | Self::PostSpliceSyntaxError(_) => "DEAP-E0802",
            Self::OverlappingOrInvalidSpan | Self::ItemNotFound(_) => "DEAP-E0803",
            Self::IoError(_) => "DEAP-E0804",
        }
    }
}

impl std::fmt::Display for CodemodError {
    /// Formats codemod error with diagnostic code. Realises: [REQ-0608]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] codemod error: {self:?}", self.diagnostic_code())
    }
}

impl std::error::Error for CodemodError {}

/// Extracts `(name, kind, is_pub)` from a top-level `ra_ap_syntax::ast::Item`.
/// Realises: [REQ-0608]
fn classify_top_level_item(item: &ast::Item) -> Option<(String, &'static str, bool)> {
    match item {
        ast::Item::Fn(it) => Some((it.name()?.text().to_string(), "fn", it.visibility().is_some())),
        ast::Item::Struct(it) => Some((it.name()?.text().to_string(), "struct", it.visibility().is_some())),
        ast::Item::Enum(it) => Some((it.name()?.text().to_string(), "enum", it.visibility().is_some())),
        ast::Item::Trait(it) => Some((it.name()?.text().to_string(), "trait", it.visibility().is_some())),
        ast::Item::Impl(it) => {
            let ty = it.self_ty().map_or_else(|| "impl".to_string(), |t| t.syntax().text().to_string());
            Some((ty, "impl", false))
        }
        ast::Item::Module(it) => Some((it.name()?.text().to_string(), "mod", it.visibility().is_some())),
        ast::Item::Const(it) => Some((it.name()?.text().to_string(), "const", it.visibility().is_some())),
        ast::Item::Static(it) => Some((it.name()?.text().to_string(), "static", it.visibility().is_some())),
        ast::Item::TypeAlias(it) => Some((it.name()?.text().to_string(), "type", it.visibility().is_some())),
        ast::Item::Union(it) => Some((it.name()?.text().to_string(), "union", it.visibility().is_some())),
        ast::Item::MacroRules(it) => Some((it.name()?.text().to_string(), "macro_rules", false)),
        ast::Item::MacroDef(it) => Some((it.name()?.text().to_string(), "macro", it.visibility().is_some())),
        _ => None,
    }
}

/// Validates that `[byte_start, byte_end)` falls on UTF-8 character boundaries within `source`.
/// Realises: [REQ-0608]
fn is_valid_utf8_span(source: &str, span: Utf8ByteSpan) -> bool {
    if span.byte_start > span.byte_end || span.byte_end > source.len() {
        return false;
    }
    source.is_char_boundary(span.byte_start) && source.is_char_boundary(span.byte_end)
}

/// [DOC-TIER: A] Lists top-level Rust CST items ordered by `(byte_start, name)` (`TD-14`).
///
/// Safety & Traceability: Realises: [ZeroTokenDeterministicCodemodEngine/list_rust_cst_items], [REQ-0608]
/// Contract: Parses `source` with `ra_ap_syntax::SourceFile::parse`, rejecting syntax errors, and collects top-level items with outer doc/attribute trivia spans.
/// Bounds & Faults: $O(N \log K)$ over $N$ bytes and $K$ items; fails closed via `RustCstInventoryError`.
///
/// # Errors
/// Returns `RustCstInventoryError` on empty inputs, syntax errors, or invalid UTF-8 spans.
pub fn list_rust_cst_items(
    file_path: &str,
    source: &str,
) -> Result<RustItemInventory, RustCstInventoryError> {
    if file_path.trim().is_empty() || source.trim().is_empty() {
        return Err(RustCstInventoryError::EmptySourceOrPath);
    }
    let parsed = SourceFile::parse(source, Edition::CURRENT);
    if let Some(first_err) = parsed.errors().first() {
        return Err(RustCstInventoryError::SyntaxError(first_err.to_string()));
    }
    let mut items = BTreeMap::new();
    let mut symbol_index = BTreeSet::new();
    for item in parsed.tree().items() {
        let Some((name, kind, is_pub)) = classify_top_level_item(&item) else {
            continue;
        };
        let range = item.syntax().text_range();
        let start_u32 = u32::from(range.start());
        let span = Utf8ByteSpan {
            byte_start: start_u32 as usize,
            byte_end: u32::from(range.end()) as usize,
        };
        if !is_valid_utf8_span(source, span) {
            return Err(RustCstInventoryError::InvalidUtf8Span);
        }
        let doc_tier = if node_enters_test_scope(item.syntax()) {
            DocTier::TestExcluded
        } else if is_pub {
            DocTier::RustTierA
        } else {
            DocTier::RustTierBC
        };
        symbol_index.insert(name.clone());
        let desc = RustCstItemDescriptor { name: name.clone(), kind: kind.to_string(), span, is_pub, doc_tier };
        items.insert((start_u32, name), desc);
    }
    Ok(RustItemInventory { file_path: file_path.to_string(), items, symbol_index })
}

/// Extracts `Realises: [REQ-...]` identifiers from an item's outer doc comment slice.
/// Realises: [REQ-0608]
fn extract_realises_reqs(item_slice: &str) -> BTreeSet<String> {
    let mut reqs = BTreeSet::new();
    for line in item_slice.lines() {
        let Some((_, after)) = line.split_once("Realises:") else { continue };
        let mut rest = after;
        while let Some((_, post_open)) = rest.split_once('[') {
            let Some((id, post_close)) = post_open.split_once(']') else { break };
            let clean = id.trim();
            if !clean.is_empty() {
                reqs.insert(clean.to_string());
            }
            rest = post_close;
        }
    }
    reqs
}

/// [DOC-TIER: A] Inventories extractable top-level items with outer doc/attribute trivia spans (`TD-16`).
///
/// Safety & Traceability: Realises: [ZeroTokenDeterministicCodemodEngine/inventory_extractable_items], [REQ-0608]
/// Contract: Catalogues top-level CST items into a `BTreeMap<String, RustItemDescriptor>` preserving outer doc comments, attributes, and `Realises:` bindings.
/// Bounds & Faults: $O(N \log K)$ over $N$ source bytes and $K$ items; fails closed via `CodemodError`.
///
/// # Errors
/// Returns `CodemodError` on empty inputs, syntax errors, or invalid UTF-8 spans.
pub fn inventory_extractable_items(
    file_path: &str,
    source: &str,
) -> Result<BTreeMap<String, RustItemDescriptor>, CodemodError> {
    if file_path.trim().is_empty() || source.trim().is_empty() {
        return Err(CodemodError::EmptyInput);
    }
    let inv = list_rust_cst_items(file_path, source).map_err(|err| match err {
        RustCstInventoryError::EmptySourceOrPath => CodemodError::EmptyInput,
        RustCstInventoryError::SyntaxError(msg) => CodemodError::PostSpliceSyntaxError(msg),
        RustCstInventoryError::InvalidUtf8Span => CodemodError::OverlappingOrInvalidSpan,
        RustCstInventoryError::IoError(msg) => CodemodError::IoError(msg),
    })?;
    let mut catalog = BTreeMap::new();
    for desc in inv.items.into_values() {
        let slice = source
            .get(desc.span.byte_start..desc.span.byte_end)
            .ok_or(CodemodError::OverlappingOrInvalidSpan)?;
        let realises_reqs = extract_realises_reqs(slice);
        let entry = RustItemDescriptor {
            name: desc.name.clone(),
            kind: desc.kind,
            span: desc.span,
            realises_reqs,
            doc_tier: desc.doc_tier,
        };
        catalog.insert(desc.name, entry);
    }
    Ok(catalog)
}

/// [DOC-TIER: A] Splices disjoint UTF-8 byte spans in descending reverse offset order (`TD-17`).
///
/// Safety & Traceability: Realises: [ZeroTokenDeterministicCodemodEngine/splice_disjoint_reverse_spans], [REQ-0608]
/// Contract: Validates UTF-8 boundaries and pairwise non-overlap across `edits`, applies replacements in descending `byte_start` order, and verifies syntax via `ra_ap_syntax`.
/// Bounds & Faults: $O(E \log E + N)$ over $E$ edits and $N$ source bytes; fails closed via `CodemodError`.
///
/// # Errors
/// Returns `CodemodError` on empty source, invalid or overlapping spans, or post-splice syntax errors.
pub fn splice_disjoint_reverse_spans(
    source: &str,
    edits: &[SpanSpliceEdit],
) -> Result<String, CodemodError> {
    if source.is_empty() {
        return Err(CodemodError::EmptyInput);
    }
    for edit in edits {
        if !is_valid_utf8_span(source, edit.span) {
            return Err(CodemodError::OverlappingOrInvalidSpan);
        }
    }
    let mut sorted_asc: Vec<&SpanSpliceEdit> = edits.iter().collect();
    sorted_asc.sort_by_key(|e| (e.span.byte_start, e.span.byte_end));
    for pair in sorted_asc.windows(2) {
        let Some(prev) = pair.first() else {
            continue;
        };
        let Some(curr) = pair.get(1) else {
            continue;
        };
        if prev.span.byte_end > curr.span.byte_start || prev.span.byte_start == curr.span.byte_start
        {
            return Err(CodemodError::OverlappingOrInvalidSpan);
        }
    }
    let mut sorted_desc = sorted_asc;
    sorted_desc.sort_by_key(|a| std::cmp::Reverse(a.span.byte_start));
    let mut out = source.to_string();
    for edit in sorted_desc {
        let prefix = out
            .get(..edit.span.byte_start)
            .ok_or(CodemodError::OverlappingOrInvalidSpan)?;
        let suffix = out
            .get(edit.span.byte_end..)
            .ok_or(CodemodError::OverlappingOrInvalidSpan)?;
        let mut next = String::with_capacity(prefix.len() + edit.replacement.len() + suffix.len());
        next.push_str(prefix);
        next.push_str(&edit.replacement);
        next.push_str(suffix);
        out = next;
    }
    let reparsed = SourceFile::parse(&out, Edition::CURRENT);
    if let Some(first_err) = reparsed.errors().first() {
        return Err(CodemodError::PostSpliceSyntaxError(first_err.to_string()));
    }
    Ok(out)
}
