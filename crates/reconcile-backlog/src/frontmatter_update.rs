//! AST-guided frontmatter table field mutation and insertion helpers.
//!
//! Uses `deap_core::markdown_ast::parse_markdown_ast` and `pulldown_cmark` offset spans
//! to locate H2 section boundaries and fenced code blocks without ad-hoc fence state machines.
//!
//! Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER]

use crate::frontmatter::clean_markdown_key;
use crate::ReconcileError;
use deap_core::markdown_ast::parse_markdown_ast;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::ops::Range;

/// Returns the byte offset of the first H2 (`## `) section in `content` using `deap_core::markdown_ast`.
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-PARSE-FRONTMATTER]
pub fn first_h2_byte_offset(content: &str) -> usize {
    let ast = parse_markdown_ast(content);
    ast.sections
        .iter()
        .find(|s| s.level == 2)
        .map(|s| s.start_offset)
        .unwrap_or(content.len())
}

/// Collects byte offset ranges of all fenced/indented `Tag::CodeBlock` nodes in `content` via `pulldown-cmark`.
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-PARSE-FRONTMATTER]
pub fn collect_code_block_ranges(content: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut active_start: Option<usize> = None;
    for (event, range) in Parser::new_ext(content, Options::all()).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(_)) => {
                active_start = Some(range.start);
            }
            Event::End(TagEnd::CodeBlock) => {
                let start = active_start.take().unwrap_or(range.start);
                ranges.push(start..range.end);
            }
            _ => {}
        }
    }
    if let Some(start) = active_start {
        ranges.push(start..content.len());
    }
    ranges
}

/// Checks whether `byte_offset` falls inside any range in `ranges`.
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-PARSE-FRONTMATTER]
pub fn is_offset_in_ranges(byte_offset: usize, ranges: &[Range<usize>]) -> bool {
    ranges
        .iter()
        .any(|r| byte_offset >= r.start && byte_offset < r.end)
}

/// [DOC-TIER: A] Updates or inserts a single frontmatter field in the Markdown table.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-UPDATE-FRONTMATTER-FIELD]
/// Contract: Delegates to `update_frontmatter_fields_batch` with a 1-element slice.
/// Bounds & Faults: Time O(N), Space O(N) where N is `content.len()`.
pub fn update_frontmatter_field(
    content: &str,
    field_name: &str,
    new_value: &str,
) -> Result<String, ReconcileError> {
    update_frontmatter_fields_batch(content, &[(field_name, new_value)])
}

/// Tracks structural line positions discovered while scanning the frontmatter region.
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-UPDATE-FRONTMATTER-FIELDS-BATCH]
struct FrontmatterScanPositions {
    last_table_line_idx: Option<usize>,
    h1_line_idx: Option<usize>,
}

/// Checks whether two normalized frontmatter table keys refer to the same logical field (including issue ID aliases).
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-UPDATE-FRONTMATTER-FIELDS-BATCH]
fn frontmatter_keys_match(raw_key: &str, target_key: &str) -> bool {
    let target = target_key.trim().to_lowercase();
    if raw_key == target {
        return true;
    }
    let is_issue_alias = |k: &str| {
        matches!(
            k,
            "github issue" | "issue" | "tracker issue" | "issue id" | "github"
        )
    };
    is_issue_alias(raw_key) && is_issue_alias(&target)
}

/// Scans `content` lines before the first AST H2 section, updating matching frontmatter rows in place.
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-UPDATE-FRONTMATTER-FIELDS-BATCH]
fn scan_and_update_frontmatter_lines(
    content: &str,
    updates: &[(&str, &str)],
    updated_flags: &mut [bool],
    out_lines: &mut Vec<String>,
) -> FrontmatterScanPositions {
    let h2_offset = first_h2_byte_offset(content);
    let code_ranges = collect_code_block_ranges(content);
    let mut in_frontmatter_region = true;
    let mut positions = FrontmatterScanPositions {
        last_table_line_idx: None,
        h1_line_idx: None,
    };
    let mut byte_offset = 0usize;

    for raw_line in content.split('\n') {
        let line_len = raw_line.len();
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let trimmed = line.trim();
        let in_code = is_offset_in_ranges(byte_offset, &code_ranges);

        if !in_code && byte_offset >= h2_offset && trimmed.starts_with("## ") {
            in_frontmatter_region = false;
        }
        if !in_code && in_frontmatter_region && positions.h1_line_idx.is_none() && trimmed.starts_with("# ") {
            positions.h1_line_idx = Some(out_lines.len());
        }

        let mut line_replaced = false;
        if !in_code && in_frontmatter_region && trimmed.starts_with('|') && trimmed.ends_with('|') {
            let inner = trimmed.get(1..trimmed.len().saturating_sub(1)).unwrap_or("");
            let cells: Vec<&str> = inner.split('|').collect();
            if let (Some(&first_cell), Some(_)) = (cells.first(), cells.get(1)) {
                let raw_key = clean_markdown_key(first_cell);
                let is_sep = cells.iter().all(|c| {
                    let t = c.trim();
                    !t.is_empty() && t.chars().all(|ch| ch == '-' || ch == ':' || ch == ' ')
                });
                if is_sep || !raw_key.is_empty() {
                    positions.last_table_line_idx = Some(out_lines.len());
                }
                for ((field_name, new_value), flag) in updates.iter().zip(updated_flags.iter_mut()) {
                    if !*flag && frontmatter_keys_match(&raw_key, field_name) {
                        let key_cell = first_cell.trim();
                        let display_key = if key_cell.starts_with("**") && key_cell.ends_with("**") {
                            key_cell.to_string()
                        } else {
                            format!("**{}**", field_name.trim())
                        };
                        out_lines.push(format!("| {} | {} |", display_key, new_value));
                        *flag = true;
                        line_replaced = true;
                        break;
                    }
                }
            }
        }

        if !line_replaced {
            out_lines.push(line.to_string());
        }
        byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
    }

    positions
}

/// Inserts any un-updated `(field_name, new_value)` rows into `out_lines` at the discovered table or H1 position.
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-UPDATE-FRONTMATTER-FIELDS-BATCH]
fn insert_missing_frontmatter_rows(
    updates: &[(&str, &str)],
    updated_flags: &[bool],
    positions: &FrontmatterScanPositions,
    out_lines: &mut Vec<String>,
) {
    let remaining: Vec<(&str, &str)> = updates
        .iter()
        .zip(updated_flags)
        .filter(|(_, updated)| !**updated)
        .map(|(pair, _)| *pair)
        .collect();

    if remaining.is_empty() {
        return;
    }

    if let Some(last_idx) = positions.last_table_line_idx {
        for (offset, (field_name, new_value)) in remaining.into_iter().enumerate() {
            out_lines.insert(
                last_idx + 1 + offset,
                format!("| **{}** | {} |", field_name.trim(), new_value),
            );
        }
    } else if let Some(h1_idx) = positions.h1_line_idx {
        let base_pos = h1_idx + 1;
        out_lines.insert(base_pos, String::new());
        out_lines.insert(base_pos + 1, "| Field | Value |".to_string());
        out_lines.insert(base_pos + 2, "|---|---|".to_string());
        for (offset, (field_name, new_value)) in remaining.into_iter().enumerate() {
            out_lines.insert(
                base_pos + 3 + offset,
                format!("| **{}** | {} |", field_name.trim(), new_value),
            );
        }
    } else {
        let mut prefix_lines = vec![
            "| Field | Value |".to_string(),
            "|---|---|".to_string(),
        ];
        for (field_name, new_value) in remaining {
            prefix_lines.push(format!("| **{}** | {} |", field_name.trim(), new_value));
        }
        prefix_lines.push(String::new());
        prefix_lines.append(out_lines);
        *out_lines = prefix_lines;
    }
}

/// [DOC-TIER: A] Updates or inserts multiple frontmatter fields in a single pass while preserving line endings.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-UPDATE-FRONTMATTER-FIELDS-BATCH]
/// Contract: Mutates only rows before the first AST H2 section outside `pulldown-cmark` code blocks.
/// Bounds & Faults: Time O(N * U), Space O(N) where N is `content.len()` and U is `updates.len()`.
pub fn update_frontmatter_fields_batch(
    content: &str,
    updates: &[(&str, &str)],
) -> Result<String, ReconcileError> {
    if updates.is_empty() {
        return Ok(content.to_string());
    }

    let line_ending = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let mut updated_flags = vec![false; updates.len()];
    let mut out_lines: Vec<String> = Vec::new();

    let positions = scan_and_update_frontmatter_lines(
        content,
        updates,
        &mut updated_flags,
        &mut out_lines,
    );
    insert_missing_frontmatter_rows(updates, &updated_flags, &positions, &mut out_lines);

    Ok(out_lines.join(line_ending))
}
