//! AST-guided detection and removal of unpopulated specification template placeholders.
//!
//! Uses `pulldown-cmark` `Tag::CodeBlock` byte ranges via `frontmatter::collect_code_block_ranges`
//! so placeholder tokens inside fenced or indented code blocks are never falsely flagged or stripped.
//!
//! Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST, Zero Fallback Placeholders]

use crate::frontmatter::{collect_code_block_ranges, is_offset_in_ranges};

/// Checks if a single trimmed line is an unpopulated template placeholder bullet.
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-IS-PLACEHOLDER-BULLET]
pub(crate) fn is_placeholder_bullet(trimmed: &str) -> bool {
    let bullet_body = if let Some(rest) = trimmed
        .strip_prefix("- [ ] ")
        .or_else(|| trimmed.strip_prefix("* [ ] "))
        .or_else(|| trimmed.strip_prefix("- [x] "))
        .or_else(|| trimmed.strip_prefix("* [x] "))
        .or_else(|| trimmed.strip_prefix("- [X] "))
        .or_else(|| trimmed.strip_prefix("* [X] "))
        .or_else(|| trimmed.strip_prefix("- "))
        .or_else(|| trimmed.strip_prefix("* "))
    {
        rest.trim()
    } else {
        return false;
    };

    let lower = bullet_body.to_lowercase();
    let cleaned: String = lower
        .chars()
        .filter(|c| !matches!(c, '*' | '_' | '`' | '(' | ')' | '[' | ']'))
        .collect();
    let cleaned = cleaned.trim();

    if matches!(
        cleaned,
        "to be created"
            | "tbd"
            | "todo"
            | "pending"
            | "none"
            | "n/a"
            | "placeholder"
            | "to be defined"
            | "to be determined"
    ) {
        return true;
    }

    if lower.contains("(to be created)")
        || lower.contains("*(to be created)*")
        || lower.contains("[tbd]")
        || lower.contains("(tbd)")
        || lower.contains("(todo)")
        || lower.contains("[todo]")
        || lower.starts_with("tbd:")
        || lower.starts_with("todo:")
        || lower.contains("#000")
        || lower.contains("#xxx")
    {
        return true;
    }

    false
}

/// [DOC-TIER: A] Checks if Markdown content contains unpopulated template placeholders outside code blocks.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-CONTAINS-UNPOPULATED-PLACEHOLDERS]
/// Contract: Uses `pulldown-cmark` code block spans to ignore fenced examples; returns `true` on any placeholder.
/// Bounds & Faults: Time O(N), Space O(C) where N is `content.len()` and C is code block count.
pub fn contains_unpopulated_placeholders(content: &str) -> bool {
    let code_ranges = collect_code_block_ranges(content);
    let mut byte_offset = 0usize;

    for raw_line in content.split('\n') {
        let line_len = raw_line.len();
        let trimmed = raw_line.trim();
        if !is_offset_in_ranges(byte_offset, &code_ranges) {
            if is_placeholder_bullet(trimmed) {
                return true;
            }
            let lower = trimmed.to_lowercase();
            if lower.contains("(to be created)") || lower.contains("*(to be created)*") {
                return true;
            }
        }
        byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
    }

    false
}

/// [DOC-TIER: A] Removes unpopulated placeholder bullet lines from Markdown content.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-CLEAN-UNPOPULATED-PLACEHOLDERS]
/// Contract: Delegates to `clean_unpopulated_placeholders_counted` and returns the cleaned string.
/// Bounds & Faults: Time O(N), Space O(N) where N is `content.len()`.
pub fn clean_unpopulated_placeholders(content: &str) -> String {
    clean_unpopulated_placeholders_counted(content).0
}

/// [DOC-TIER: A] Removes unpopulated placeholder bullet lines and returns `(cleaned_content, removed_count)`.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-CLEAN-UNPOPULATED-PLACEHOLDERS-COUNTED]
/// Contract: Preserves fenced code blocks via `pulldown-cmark` spans and collapses excess blank lines.
/// Bounds & Faults: Time O(N), Space O(N) where N is `content.len()`.
pub fn clean_unpopulated_placeholders_counted(content: &str) -> (String, usize) {
    let line_ending = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let code_ranges = collect_code_block_ranges(content);
    let mut removed = 0usize;
    let mut out_lines: Vec<&str> = Vec::new();
    let mut prev_was_removed = false;
    let mut byte_offset = 0usize;

    for raw_line in content.split('\n') {
        let line_len = raw_line.len();
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let trimmed = line.trim();
        let in_code = is_offset_in_ranges(byte_offset, &code_ranges);

        if !in_code && is_placeholder_bullet(trimmed) {
            removed += 1;
            prev_was_removed = true;
            byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
            continue;
        }

        if prev_was_removed
            && trimmed.is_empty()
            && out_lines.last().is_some_and(|last| last.trim().is_empty())
        {
            byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
            continue;
        }

        prev_was_removed = false;
        out_lines.push(line);
        byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
    }

    (out_lines.join(line_ending), removed)
}
