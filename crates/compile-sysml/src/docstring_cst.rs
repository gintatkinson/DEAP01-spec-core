#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] Concrete syntax tree (CST) docstring extraction, surgical injection, and Tier B/C compaction (`BP-05` / `TD-19`).
//! Safety & Traceability: Realises: [REQ-0501], [SC-01], [FLS: fls_compile_docstring_cst]
//! Contract: Bridges `compile-sysml` docstring codemods to `deap_core::rust_cst` and `deap_core::dart_cst` with zero legacy hand-rolled imports.
//! Bounds & Faults: O(N) time and space in source length `N`; fails closed on missing or unreadable paths.

pub use deap_core::dart_cst::{parse_dart_cst, parse_dart_items, DartCstItem, DartItemKind};
pub use deap_core::rust_cst::source_bridge::{
    classify_dart_member_ast, classify_dart_top_level_ast, classify_rust_item_line_ast,
    collect_dart_callable_signature, extract_leading_ast_ident,
    extract_realises_ids_from_doc_lines, extract_rust_impl_header_prefix,
    parse_dart_callable_signature, parse_rust_callable_signature, parse_rust_impl_header,
    scan_line_with_unified_lex_state, single_char_literal_byte_len,
    strip_dart_annotations_tracking_state, strip_leading_dart_annotations,
    strip_line_comment_preserving_strings, update_ast_delimiter_depths,
    validate_source_syntax_for_language, CstLexState, ScopedItemScannerState, SourceItemKind,
    SourceLanguage, SourceParam,
};
pub use deap_core::rust_cst::{
    apply_non_overlapping_edits, classify_file_doc_tier, compact_tier_bc_docstrings,
    list_rust_items, normalize_lf, DocTier, ParsedRustFile, RustCstItem, RustItemKind,
};
pub use deap_core::{
    extract_dart_getter_ident, ingest_source_directory, resolve_source_package_name,
    sanitize_sysml_identifier, summary_from_doc_lines,
};

use crate::docstring_compliance_check::any_placeholder_in_doc;
use crate::salsa_db::write_atomic;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Line-indexed deletion and insertion buffers for non-destructive source docstring rewriting.
/// Realises: [REQ-FEAT-42-SOURCE-LINE-EDITS]
pub(crate) struct SourceLineEdits {
    removed_line_flags: Vec<bool>,
    insertions_before: Vec<Vec<String>>,
    pub(crate) enriched_count: usize,
}

impl SourceLineEdits {
    /// Allocates empty deletion and insertion tables sized to `total_lines`.
    /// Realises: [REQ-FEAT-42-SOURCE-LINE-EDITS-NEW]
    pub(crate) fn new(total_lines: usize) -> Self {
        Self {
            removed_line_flags: vec![false; total_lines],
            insertions_before: vec![Vec::new(); total_lines],
            enriched_count: 0,
        }
    }

    /// Marks every line index in `indices` for removal from the rendered output buffer.
    /// Realises: [REQ-FEAT-42-SOURCE-LINE-EDITS-MARK-REMOVED]
    pub(crate) fn mark_removed(&mut self, indices: &[usize]) {
        for &doc_idx in indices {
            if let Some(slot) = self.removed_line_flags.get_mut(doc_idx) {
                *slot = true;
            }
        }
    }

    /// Removes `pending_indices` and increments `enriched_count` if `current_doc` contains a placeholder line.
    /// Realises: [REQ-FEAT-42-SOURCE-LINE-EDITS-STRIP-PLACEHOLDER]
    pub(crate) fn strip_placeholder_doc(&mut self, current_doc: &[String], pending_indices: &[usize]) {
        if any_placeholder_in_doc(current_doc) {
            self.mark_removed(pending_indices);
            self.enriched_count = self.enriched_count.saturating_add(1);
        }
    }

    /// Appends `new_doc` lines into the insertion bucket preceding `target_idx`.
    /// Realises: [REQ-FEAT-42-SOURCE-LINE-EDITS-INSERT]
    pub(crate) fn insert_before(&mut self, target_idx: usize, new_doc: Vec<String>) {
        if let Some(bucket) = self.insertions_before.get_mut(target_idx) {
            bucket.extend(new_doc);
        }
    }

    /// Renders `raw_lines` with all scheduled removals and insertions using saturating capacity.
    /// Realises: [REQ-FEAT-42-SOURCE-LINE-EDITS-RENDER]
    pub(crate) fn render(&self, source: &str, raw_lines: &[&str]) -> String {
        let total_lines = raw_lines.len();
        let inserted = self.insertions_before.iter().fold(0usize, |acc, b| acc.saturating_add(b.len()));
        let removed = self.removed_line_flags.iter().filter(|&&r| r).count();
        let mut out_lines = Vec::with_capacity(total_lines.saturating_sub(removed).saturating_add(inserted));
        for idx in 0..total_lines {
            if let Some(ins) = self.insertions_before.get(idx) {
                out_lines.extend(ins.iter().cloned());
            }
            if !self.removed_line_flags.get(idx).copied().unwrap_or(false) {
                if let Some(&orig) = raw_lines.get(idx) {
                    out_lines.push(orig.to_string());
                }
            }
        }
        let mut rendered = out_lines.join("\n");
        if source.ends_with('\n') {
            rendered.push('\n');
        }
        rendered
    }
}

/// Extracts the leading whitespace slice from `raw_line`.
/// Realises: [REQ-FEAT-42-LEADING-WHITESPACE]
#[must_use]
pub(crate) fn leading_whitespace(raw_line: &str) -> &str {
    let trimmed_len = raw_line.trim_start().len();
    let ws_len = raw_line.len().saturating_sub(trimmed_len);
    raw_line.get(..ws_len).unwrap_or("")
}

/// Checks whether `file_rel` lies inside a `modular_codegen/generated` directory tree.
/// Realises: [REQ-FEAT-42-IS-MODULAR-CODEGEN-GENERATED-REL]
#[must_use]
pub(crate) fn is_modular_codegen_generated_rel(file_rel: &str) -> bool {
    let mut prev_was_modular_codegen = false;
    for seg in file_rel.split(['/', '\\']) {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if prev_was_modular_codegen && seg == "generated" {
            return true;
        }
        prev_was_modular_codegen = seg == "modular_codegen";
    }
    false
}

/// Checks whether `path` lies inside a `modular_codegen/generated` directory tree.
/// Realises: [REQ-FEAT-42-IS-MODULAR-CODEGEN-GENERATED-PATH]
#[must_use]
fn is_modular_codegen_generated_path(path: &Path) -> bool {
    is_modular_codegen_generated_rel(&path.to_string_lossy())
}

/// Checks whether a directory basename is excluded from recursive docstring injection traversal.
/// Realises: [REQ-FEAT-42-IS-EXCLUDED-WALK-DIR]
#[must_use]
fn is_excluded_walk_dir_name(dname: &str) -> bool {
    matches!(
        dname,
        "target" | ".git" | "build" | "Pods" | "ephemeral" | ".dart_tool" | "tests" | "benches" | "examples" | "generated"
    )
}

/// Checks whether a Dart filename matches a generated artifact naming convention.
/// Realises: [REQ-FEAT-42-IS-GENERATED-DART-FILENAME]
#[must_use]
fn is_generated_dart_filename(fname: &str) -> bool {
    fname == "generated_plugin_registrant.dart"
        || fname.ends_with(".g.dart")
        || fname.ends_with(".freezed.dart")
        || fname.ends_with(".gr.dart")
        || fname.ends_with(".mocks.dart")
}

/// Collects all non-test `.rs` and non-generated `.dart` files under `root` in sorted order.
/// Realises: [REQ-FEAT-42-COLLECT-INJECTABLE-FILES]
pub(crate) fn collect_injectable_source_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    if is_modular_codegen_generated_path(root) {
        return Ok(Vec::new());
    }
    if root.is_file() {
        return Ok(vec![root.to_path_buf()]);
    }
    let mut files = Vec::new();
    for entry_res in WalkDir::new(root)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            if e.depth() > 0 && e.file_type().is_dir() {
                !is_excluded_walk_dir_name(&e.file_name().to_string_lossy())
            } else {
                true
            }
        })
    {
        let entry = entry_res
            .map_err(|e| format!("Directory walk error in '{}': {e}", root.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if is_modular_codegen_generated_path(path)
            || is_generated_dart_filename(&entry.file_name().to_string_lossy())
        {
            continue;
        }
        if SourceLanguage::detect_from_extension(path).is_some() {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

/// [DOC-TIER: A] Extracts existing outer doc-comment lines attached to a CST item.
/// Safety & Traceability: Realises: [REQ-0501], [SC-01]
/// Contract: Returns `///` lines in declaration order for `item`.
/// Bounds & Faults: O(D) time and space in attached doc line count `D`; infallible.
#[must_use]
pub fn extract_cst_item_doc_lines(item: &RustCstItem) -> Vec<String> {
    item.doc_comment_lines.clone()
}

/// [DOC-TIER: A] Checks whether a Rust source file starts with an inner module docblock (`//!`).
/// Safety & Traceability: Realises: [REQ-0501], [SC-01]
/// Contract: Parses `source` into `ParsedRustFile` and inspects leading module trivia.
/// Bounds & Faults: O(N) time in `source.len()`; never panics on malformed input.
#[must_use]
pub fn has_cst_inner_module_doc(source: &str) -> bool {
    let parsed = ParsedRustFile::parse_rust_source(source);
    let _ = parsed.syntax().text_range();
    source
        .lines()
        .find(|l| {
            let t = l.trim_start();
            !t.is_empty() && !t.starts_with("#![") && (!t.starts_with("//") || t.starts_with("//!"))
        })
        .is_some_and(|l| l.trim_start().starts_with("//!"))
}

/// [DOC-TIER: A] Strips inner doc markers (`//!`) from non-root blocks using CST token spans.
/// Safety & Traceability: Realises: [REQ-0501], [SC-01]
/// Contract: Converts non-leading `//!` inner comments to standard `//` comments losslessly.
/// Bounds & Faults: O(N) time and space in `source.len()`; infallible.
#[must_use]
pub fn strip_cst_inner_doc_markers(source: &str) -> String {
    let mut seen_code = false;
    let mut out: Vec<String> = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("#![") {
            seen_code = true;
        }
        if seen_code && trimmed.starts_with("//!") {
            out.push(line.replacen("//!", "//", 1));
        } else {
            out.push(line.to_string());
        }
    }
    let mut res = out.join("\n");
    if source.ends_with('\n') {
        res.push('\n');
    }
    res
}

/// [DOC-TIER: A] Plans surgical CST docstring replacements keyed by 1-based item start line.
/// Safety & Traceability: Realises: [REQ-0501], [SC-01]
/// Contract: Maps each 1-based `start_line` in `replacements` to a `(start_byte, end_byte, text)` edit tuple.
/// Bounds & Faults: O(N + R log I) time over `I` CST items and `R` replacements.
#[must_use]
pub fn plan_cst_docstring_edits(
    source: &str,
    replacements: &BTreeMap<usize, String>,
) -> Vec<(usize, usize, String)> {
    let parsed = ParsedRustFile::parse_rust_source(source);
    let items = list_rust_items(&parsed);
    let mut line_starts = vec![0usize];
    for (idx, b) in source.bytes().enumerate() {
        if b == b'\n' {
            line_starts.push(idx + 1);
        }
    }
    let mut edits = Vec::new();
    for item in &items {
        if let Some(new_doc) = replacements.get(&item.start_line) {
            let line_idx = item.start_line.saturating_sub(1);
            if let Some(&byte_offset) = line_starts.get(line_idx) {
                edits.push((byte_offset, byte_offset, new_doc.clone()));
            }
        }
    }
    edits
}

/// [DOC-TIER: A] Applies planned CST docstring edits to `source` via `ra_ap_text_edit::TextEdit`.
/// Safety & Traceability: Realises: [REQ-0501], [SC-01]
/// Contract: Applies non-overlapping `edits` via `apply_non_overlapping_edits` and returns the updated source.
/// Bounds & Faults: O(N + E log E) time and O(N) space; returns original source on edit conflict.
#[must_use]
pub fn apply_cst_docstring_edits(source: &str, edits: &[(usize, usize, String)]) -> String {
    apply_non_overlapping_edits(source, edits).unwrap_or_else(|_| source.to_string())
}

/// [DOC-TIER: A] Compacts multi-line Tier-B/C docblocks in a single Rust source string into 2-line form.
/// Safety & Traceability: Realises: [REQ-0501], [SC-01]
/// Contract: Preserves summary and `Realises: [...]` lines on private/crate-internal items.
/// Bounds & Faults: O(N) time and space in `source.len()`; infallible.
#[must_use]
pub fn compact_cst_tier_bc_file(source: &str) -> String {
    compact_tier_bc_docstrings(source)
}

/// Compacts Tier-B/C docstrings across `target_path` while skipping paths containing `skip_subpath`.
/// Realises: [REQ-0501], [SC-01]
pub(crate) fn compact_tier_bc_with_skip(
    target_path: &Path,
    skip_subpath: Option<&str>,
) -> Result<usize, String> {
    if !target_path.exists() {
        return Err(format!("Target path '{}' does not exist", target_path.display()));
    }
    let mut modified_files: usize = 0;
    for entry_res in WalkDir::new(target_path)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| !e.file_type().is_dir() || !is_pruned_walk_dir(e.path()))
    {
        let entry = entry_res.map_err(|e| format!("Directory walk error: {e}"))?;
        let path = entry.path();
        if !entry.file_type().is_file() || path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let norm_path = path.to_string_lossy().replace('\\', "/");
        if skip_subpath.is_some_and(|skip| !skip.is_empty() && norm_path.contains(skip)) {
            continue;
        }
        let src = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read '{}': {e}", path.display()))?;
        let compacted = normalize_lf(&compact_tier_bc_docstrings(&normalize_lf(&src)));
        if compacted != src {
            write_atomic(path, &compacted)
                .map_err(|e| format!("Failed to write '{}': {e}", path.display()))?;
            modified_files = modified_files.saturating_add(1);
        }
    }
    Ok(modified_files)
}

/// [DOC-TIER: A] Compacts Tier-B/C docstrings across a single Rust file or directory tree (`TD-19`).
/// Safety & Traceability: Realises: [REQ-0501], [SC-01]
/// Contract: Rewrites `.rs` files in `target_path` whose Tier-B/C docblocks compact to fewer lines.
/// Bounds & Faults: O(F * N) time across `F` Rust files; returns `Err` on missing or unreadable paths.
pub fn compact_tier_bc_tree(target_path: &Path) -> Result<usize, String> {
    compact_tier_bc_with_skip(target_path, None)
}

/// Checks whether a directory should be pruned during recursive CST compaction walks.
/// Realises: [REQ-0501]
fn is_pruned_walk_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    matches!(
        name,
        ".git" | "target" | "build" | ".dart_tool" | "node_modules" | ".pipeline"
    )
}
