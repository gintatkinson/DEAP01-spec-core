//! Markdown checklist reconciliation, placeholder cleanup, and bottom-up hierarchy synthesis.
//!
//! Routes checklist item link extraction and code-fence scoping through `pulldown_cmark`
//! and `deap_core::markdown_ast` while preserving all public API signatures.
//!
//! Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST, Issue Tracker as Canonical SSOT, Zero Fallback Placeholders, Automated Spec Reconciliation]

pub use crate::checklist_hierarchy::{
    format_canonical_checklist_item, synthesize_and_reconcile_hierarchy, DiscoveredSpec,
};
pub use crate::checklist_placeholders::{
    clean_unpopulated_placeholders, clean_unpopulated_placeholders_counted,
    contains_unpopulated_placeholders,
};

use crate::checklist_placeholders::is_placeholder_bullet;
use crate::frontmatter::{collect_code_block_ranges, extract_issue_number, is_offset_in_ranges};
use crate::tracker::IssueIndex;
use crate::ReconcileError;
use pulldown_cmark::{Event, Options, Parser, Tag};

/// [DOC-TIER: A] Extracts `(title, Option<issue_number>)` from a Markdown checklist line using `pulldown-cmark`.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-EXTRACT-CHECKLIST-TITLE-AND-ISSUE]
/// Contract: Parses `- [ ]` / `- [x]` task list lines and extracts link title and issue ID via `Tag::Link`.
/// Bounds & Faults: Time O(N), Space O(N) where N is `line.len()`.
pub fn extract_checklist_title_and_issue(line: &str) -> Option<(String, Option<u64>)> {
    let trimmed = line.trim();
    let after_checkbox = trimmed
        .strip_prefix("- [ ] ")
        .or_else(|| trimmed.strip_prefix("- [x] "))
        .or_else(|| trimmed.strip_prefix("- [X] "))
        .or_else(|| trimmed.strip_prefix("* [ ] "))
        .or_else(|| trimmed.strip_prefix("* [x] "))
        .or_else(|| trimmed.strip_prefix("* [X] "))?
        .trim();

    if after_checkbox.is_empty() || is_placeholder_bullet(trimmed) {
        return None;
    }

    if after_checkbox.starts_with('[') {
        for (event, range) in Parser::new_ext(after_checkbox, Options::all()).into_offset_iter() {
            if let Event::Start(Tag::Link { dest_url, .. }) = event {
                if range.start == 0 {
                    let link_slice = after_checkbox.get(range.clone()).unwrap_or("");
                    if let Some(sep_idx) = link_slice.rfind("](") {
                        let title = link_slice.get(1..sep_idx).unwrap_or("").trim().to_string();
                        if !title.is_empty() {
                            let url_issue = extract_issue_number(&dest_url);
                            let rest_after = after_checkbox.get(range.end..).unwrap_or("").trim();
                            let trailing_issue = extract_trailing_issue_ref(rest_after);
                            return Some((title, url_issue.or(trailing_issue)));
                        }
                    }
                }
                break;
            }
        }
    }

    let issue_num = extract_trailing_issue_ref(after_checkbox);
    let mut clean_title = after_checkbox.to_string();

    if let Some(paren_idx) = clean_title.rfind("(#") {
        clean_title = clean_title.get(..paren_idx).unwrap_or("").trim().to_string();
    } else if let Some(dash_idx) = clean_title.find(" -- ") {
        clean_title = clean_title.get(..dash_idx).unwrap_or("").trim().to_string();
    }

    if clean_title.is_empty() {
        None
    } else {
        Some((clean_title, issue_num))
    }
}

/// Extracts an issue number from trailing text such as `(#123)` or `-- #123`.
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-EXTRACT-TRAILING-ISSUE-REF]
fn extract_trailing_issue_ref(s: &str) -> Option<u64> {
    if let Some(hash_idx) = s.find('#') {
        let after_hash = s.get(hash_idx + 1..).unwrap_or("");
        let digits: String = after_hash
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(num) = digits.parse::<u64>() {
            if num > 0 {
                return Some(num);
            }
        }
    }
    extract_issue_number(s)
}

/// [DOC-TIER: A] Reconciles checklist items (`- [ ]` / `- [x]`) against `IssueIndex`.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-RECONCILE-PARENT-CHECKLISTS]
/// Contract: Delegates to `reconcile_checklists_in_content` and returns the reconciled Markdown string.
/// Bounds & Faults: Time O(N), Space O(N) where N is `content.len()`.
pub fn reconcile_parent_checklists(
    content: &str,
    index: &IssueIndex,
) -> Result<String, ReconcileError> {
    Ok(reconcile_checklists_in_content(content, index, None)?.0)
}

/// Rewrites a single checklist line's checkbox marker to match `should_be_checked`.
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-RECONCILE-CHECKLISTS-IN-CONTENT]
fn update_checkbox_state_on_line(line: &str, should_be_checked: bool) -> Option<String> {
    let trimmed_start = line.trim_start();
    let indent_len = line.len().saturating_sub(trimmed_start.len());
    let indent = line.get(..indent_len).unwrap_or("");

    let is_currently_checked = trimmed_start.starts_with("- [x] ")
        || trimmed_start.starts_with("- [X] ")
        || trimmed_start.starts_with("* [x] ")
        || trimmed_start.starts_with("* [X] ");

    if is_currently_checked == should_be_checked {
        return None;
    }

    let bullet_char = if trimmed_start.starts_with('*') { '*' } else { '-' };
    let after_box = trimmed_start.get(6..).unwrap_or("");
    let new_box = if should_be_checked { "[x]" } else { "[ ]" };
    Some(format!("{}{} {} {}", indent, bullet_char, new_box, after_box))
}

/// [DOC-TIER: A] Reconciles checklist items in `content` and returns `(reconciled_content, items_updated_count)`.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-RECONCILE-CHECKLISTS-IN-CONTENT]
/// Contract: Purges unpopulated placeholders and updates `- [ ]` / `- [x]` checkboxes outside code blocks.
/// Bounds & Faults: Time O(N), Space O(N) where N is `content.len()`.
pub fn reconcile_checklists_in_content(
    content: &str,
    index: &IssueIndex,
    _parent_spec_type: Option<&str>,
) -> Result<(String, usize), ReconcileError> {
    let (cleaned, _purged_count) = clean_unpopulated_placeholders_counted(content);
    let line_ending = if cleaned.contains("\r\n") { "\r\n" } else { "\n" };
    let code_ranges = collect_code_block_ranges(&cleaned);
    let mut updated_count = 0usize;
    let mut out_lines: Vec<String> = Vec::new();
    let mut byte_offset = 0usize;

    for raw_line in cleaned.split('\n') {
        let line_len = raw_line.len();
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let in_code = is_offset_in_ranges(byte_offset, &code_ranges);

        if !in_code {
            if let Some((title, issue_num)) = extract_checklist_title_and_issue(line) {
                let resolved_opt = if let Some(num) = issue_num {
                    index.is_issue_resolved(num)
                } else {
                    index
                        .find_issue_for_spec(&title, None)
                        .and_then(|num| index.is_issue_resolved(num))
                };

                if let Some(should_be_checked) = resolved_opt {
                    if let Some(rewritten) = update_checkbox_state_on_line(line, should_be_checked) {
                        out_lines.push(rewritten);
                        updated_count += 1;
                        byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
                        continue;
                    }
                }
            }
        }

        out_lines.push(line.to_string());
        byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
    }

    Ok((out_lines.join(line_ending), updated_count))
}
