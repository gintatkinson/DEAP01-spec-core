#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Deterministic workspace scanner and lexical masker for `audit-code` (`BP-06`).
//! Realises: [REQ-0606-AUDIT-CODE-MODULE]

use deap_core::rust_cst::source_bridge::SourceAstItem;
use std::fs;
use std::path::{Path, PathBuf};

/// Iteratively walks `target_path` in sorted pre-order, collecting `.rs` and `.dart` files.
/// Realises: [REQ-0606-COLLECT-TARGET-FILES]
pub(crate) fn collect_target_source_files(
    target_path: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), i32> {
    if target_path.is_file() {
        let name = target_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if name.ends_with(".rs") || name.ends_with(".dart") {
            out.push(target_path.to_path_buf());
        }
        return Ok(());
    }
    if !target_path.is_dir() {
        return Ok(());
    }
    let mut stack: Vec<PathBuf> = vec![target_path.to_path_buf()];
    while let Some(current_path) = stack.pop() {
        if !current_path.is_dir() {
            out.push(current_path);
            continue;
        }
        let entries = match fs::read_dir(&current_path) {
            Ok(rd) => rd,
            Err(err) => {
                eprintln!(
                    "[AUDIT-CODE IO ERROR] Unable to read directory {}: {err}",
                    current_path.display()
                );
                return Err(1);
            }
        };
        let mut children = Vec::new();
        for entry_res in entries {
            match entry_res {
                Ok(entry) => children.push(entry.path()),
                Err(err) => {
                    eprintln!(
                        "[AUDIT-CODE IO ERROR] Directory entry error in {}: {err}",
                        current_path.display()
                    );
                    return Err(1);
                }
            }
        }
        children.sort();
        for child in children.into_iter().rev() {
            let name = child
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let is_excluded = name.starts_with('.')
                || name == "target"
                || name == "build"
                || name == "node_modules";
            if !is_excluded && (child.is_dir() || name.ends_with(".rs") || name.ends_with(".dart"))
            {
                stack.push(child);
            }
        }
    }
    Ok(())
}

/// Normalizes `file_path` relative to `workspace_root` using forward-slash separators.
/// Realises: [REQ-0606-NORMALIZE-REL-PATH]
pub(crate) fn normalize_rel_path(workspace_root: &Path, file_path: &Path) -> String {
    let rel = file_path.strip_prefix(workspace_root).unwrap_or(file_path);
    let normalized = rel.to_string_lossy().replace('\\', "/");
    let trimmed = normalized.trim();
    trimmed.strip_prefix("./").unwrap_or(trimmed).to_string()
}

/// Resolves the canonical `Test-Target` path for Section 1 of a synthesized defect dossier.
/// Realises: [REQ-0606-RESOLVE-TEST-TARGET]
pub(crate) fn resolve_canonical_test_target(workspace_root: &Path, rel_path: &str) -> String {
    let is_upstream_ws = workspace_root.join(".pipeline").join("upstream").is_dir();
    let trimmed = rel_path.trim();
    if is_upstream_ws {
        let in_upstream_crate = trimmed.starts_with("crates/deap-core/")
            || trimmed.starts_with("crates/verify-baseline/")
            || trimmed.starts_with("crates/compile-sysml/")
            || trimmed.starts_with("crates/ingest-sysml/")
            || trimmed.starts_with("crates/assemble-conops/")
            || trimmed.starts_with("crates/reconcile-backlog/");
        if !in_upstream_crate {
            return "crates/verify-baseline/tests/blueprint_gate_tests.rs".to_string();
        }
    }
    trimmed.to_string()
}

/// Converts an arbitrary file/symbol string into a deterministic kebab-case ASCII slug.
/// Realises: [REQ-0606-SANITIZE-SLUG]
pub(crate) fn sanitize_slug(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut prev_dash = true;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "finding".to_string()
    } else {
        out
    }
}

/// Mutable state for `mask_source_lines`.
/// Realises: [REQ-0606-MASK-SOURCE-LINES]
#[derive(Default)]
struct MaskState {
    in_line_comment: bool,
    block_depth: usize,
    in_double_quote: bool,
    quote_escaped: bool,
}

/// Advances `MaskState` when inside an active line comment, block comment, or double-quoted string.
/// Realises: [REQ-0606-MASK-SOURCE-LINES]
fn step_active_mask_state(
    state: &mut MaskState,
    ch: char,
    next: Option<char>,
    mask_strings: bool,
    out: &mut String,
) -> Option<usize> {
    if state.in_line_comment {
        if ch == '\n' {
            state.in_line_comment = false;
            out.push('\n');
        } else {
            out.push(if ch == '\r' { '\r' } else { ' ' });
        }
        return Some(1);
    }
    if state.block_depth > 0 {
        if ch == '/' && next == Some('*') {
            state.block_depth = state.block_depth.saturating_add(1);
            out.push_str("  ");
            return Some(2);
        } else if ch == '*' && next == Some('/') {
            state.block_depth = state.block_depth.saturating_sub(1);
            out.push_str("  ");
            return Some(2);
        }
        out.push(if ch == '\n' || ch == '\r' { ch } else { ' ' });
        return Some(1);
    }
    if state.in_double_quote {
        if state.quote_escaped {
            state.quote_escaped = false;
            out.push(if ch == '\n' || ch == '\r' || !mask_strings {
                ch
            } else {
                ' '
            });
        } else if ch == '\\' {
            state.quote_escaped = true;
            out.push(if mask_strings { ' ' } else { ch });
        } else if ch == '"' {
            state.in_double_quote = false;
            out.push(if mask_strings { ' ' } else { '"' });
        } else {
            out.push(if ch == '\n' || ch == '\r' || !mask_strings {
                ch
            } else {
                ' '
            });
        }
        return Some(1);
    }
    None
}

/// Masks comments (and optionally string/char literals) with spaces while preserving line boundaries.
/// Realises: [REQ-0606-MASK-SOURCE-LINES]
pub(crate) fn mask_source_lines(source: &str, mask_strings: bool) -> Vec<String> {
    let mut out = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let total = chars.len();
    let mut idx: usize = 0;
    let mut state = MaskState::default();

    while idx < total {
        let ch = chars.get(idx).copied().unwrap_or(' ');
        let next = chars.get(idx.saturating_add(1)).copied();
        if let Some(step) = step_active_mask_state(&mut state, ch, next, mask_strings, &mut out) {
            idx = idx.saturating_add(step);
            continue;
        }
        if ch == '/' && next == Some('/') {
            state.in_line_comment = true;
            out.push_str("  ");
            idx = idx.saturating_add(2);
        } else if ch == '/' && next == Some('*') {
            state.block_depth = 1;
            out.push_str("  ");
            idx = idx.saturating_add(2);
        } else if ch == '"' {
            state.in_double_quote = true;
            state.quote_escaped = false;
            out.push(if mask_strings { ' ' } else { '"' });
            idx = idx.saturating_add(1);
        } else if mask_strings && ch == '\'' {
            let c1 = chars.get(idx.saturating_add(1)).copied();
            let c2 = chars.get(idx.saturating_add(2)).copied();
            let c3 = chars.get(idx.saturating_add(3)).copied();
            if c1 == Some('\\') && c2 != Some('\n') && c3 == Some('\'') {
                out.push_str("    ");
                idx = idx.saturating_add(4);
            } else if c1 != Some('\\') && c1 != Some('\n') && c2 == Some('\'') {
                out.push_str("   ");
                idx = idx.saturating_add(3);
            } else {
                out.push('\'');
                idx = idx.saturating_add(1);
            }
        } else {
            out.push(ch);
            idx = idx.saturating_add(1);
        }
    }
    out.lines().map(ToOwned::to_owned).collect()
}

/// Computes a line-aligned boolean mask marking non-test production lines (`true`) in Rust source.
/// Realises: [REQ-0606-RUST-NON-TEST-MASK]
pub(crate) fn compute_rust_non_test_mask(masked_lines: &[String]) -> Vec<bool> {
    let mut mask = Vec::with_capacity(masked_lines.len());
    let mut brace_depth: usize = 0;
    let mut pending_test_attr = false;
    let mut test_entry_depth: Option<usize> = None;

    for line in masked_lines {
        let trimmed = line.trim();
        if test_entry_depth.is_none()
            && (trimmed.starts_with("#[cfg(test)]")
                || trimmed.starts_with("#[test]")
                || trimmed.contains("::test]"))
        {
            pending_test_attr = true;
        }
        let in_test_now = test_entry_depth.is_some() || pending_test_attr;
        for ch in line.chars() {
            if ch == '{' {
                if pending_test_attr && test_entry_depth.is_none() {
                    test_entry_depth = Some(brace_depth);
                    pending_test_attr = false;
                }
                brace_depth = brace_depth.saturating_add(1);
            } else if ch == '}' {
                brace_depth = brace_depth.saturating_sub(1);
                if let Some(entry) = test_entry_depth {
                    if brace_depth <= entry {
                        test_entry_depth = None;
                    }
                }
            }
        }
        if pending_test_attr && trimmed.ends_with(';') {
            pending_test_attr = false;
        }
        mask.push(!in_test_now);
    }
    mask
}

/// Locates the 0-based closing line index of the brace-delimited block opening at or after `start_idx`.
/// Realises: [REQ-0606-FIND-BRACE-BLOCK-END]
pub(crate) fn find_brace_block_end(masked_lines: &[String], start_idx: usize) -> usize {
    let mut depth: usize = 0;
    let mut opened = false;
    let mut idx = start_idx;
    while let Some(line) = masked_lines.get(idx) {
        for ch in line.chars() {
            if ch == '{' {
                depth = depth.saturating_add(1);
                opened = true;
            } else if ch == '}' && opened {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return idx;
                }
            }
        }
        idx = idx.saturating_add(1);
    }
    start_idx
}

/// Resolves the nearest enclosing `SourceAstItem` name at or preceding `line_1based`.
/// Realises: [REQ-0606-FIND-ENCLOSING-SYMBOL]
pub(crate) fn find_enclosing_symbol(ast_items: &[SourceAstItem], line_1based: usize) -> String {
    let mut best: Option<&SourceAstItem> = None;
    for item in ast_items {
        if item.line_no <= line_1based {
            match best {
                Some(prev) if prev.line_no > item.line_no => {}
                _ => best = Some(item),
            }
        }
    }
    match best {
        Some(item) if !item.name.trim().is_empty() => item.name.trim().to_string(),
        _ => "ModuleScope".to_string(),
    }
}
