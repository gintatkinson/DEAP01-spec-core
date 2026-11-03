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
use crate::frontmatter::extract_issue_number;
use crate::tracker::IssueIndex;
use crate::ReconcileError;
use pulldown_cmark::{Event, Options, Parser, Tag};

/// Strips a leading `#<num> - ` or `#<num> -- ` prefix from `after_checkbox` if present.
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-EXTRACT-CHECKLIST-TITLE-AND-ISSUE]
fn strip_leading_issue_prefix(after_checkbox: &str) -> (Option<u64>, &str) {
    if let Some(after_hash) = after_checkbox.strip_prefix('#') {
        let digit_len = after_hash.chars().take_while(|c| c.is_ascii_digit()).count();
        if digit_len > 0 {
            let digits = after_hash.get(..digit_len).unwrap_or("");
            if let Ok(num) = digits.parse::<u64>() {
                if num > 0 {
                    let rest = after_hash.get(digit_len..).unwrap_or("").trim_start();
                    let stripped = rest
                        .strip_prefix("-- ")
                        .or_else(|| rest.strip_prefix("- "))
                        .unwrap_or(rest)
                        .trim_start();
                    if !stripped.is_empty() {
                        return (Some(num), stripped);
                    }
                }
            }
        }
    }
    (None, after_checkbox)
}

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

    let (leading_issue, candidate) = strip_leading_issue_prefix(after_checkbox);

    if candidate.starts_with('[') {
        for (event, range) in Parser::new_ext(candidate, Options::all()).into_offset_iter() {
            if let Event::Start(Tag::Link { dest_url, .. }) = event {
                if range.start == 0 {
                    let link_slice = candidate.get(range.clone()).unwrap_or("");
                    if let Some(sep_idx) = link_slice.rfind("](") {
                        let title = link_slice.get(1..sep_idx).unwrap_or("").trim().to_string();
                        if !title.is_empty() {
                            let url_issue = extract_issue_number(&dest_url);
                            let rest_after = candidate.get(range.end..).unwrap_or("").trim();
                            let trailing_issue = extract_trailing_issue_ref(rest_after);
                            return Some((title, url_issue.or(leading_issue).or(trailing_issue)));
                        }
                    }
                }
                break;
            }
        }
        if let Some(sep_idx) = candidate.find("](") {
            let after_sep = candidate.get(sep_idx + 2..).unwrap_or("");
            if let Some(close_paren) = after_sep.find(')') {
                let title = candidate.get(1..sep_idx).unwrap_or("").trim().to_string();
                let dest_url = after_sep.get(..close_paren).unwrap_or("");
                let rest_after = after_sep.get(close_paren + 1..).unwrap_or("").trim();
                if !title.is_empty() {
                    let url_issue = extract_issue_number(dest_url);
                    let trailing_issue = extract_trailing_issue_ref(rest_after);
                    return Some((title, url_issue.or(leading_issue).or(trailing_issue)));
                }
            }
        }
    }

    let issue_num = leading_issue.or_else(|| extract_trailing_issue_ref(candidate));
    let mut clean_title = candidate.to_string();

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
pub(crate) fn update_checkbox_state_on_line(line: &str, should_be_checked: bool) -> Option<String> {
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

/// Replaces exact token occurrences of `<prefix><old_num>` (not followed by an ASCII digit) with `<prefix><new_num>`.
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-RECONCILE-CHECKLISTS-IN-CONTENT]
fn replace_issue_token_with_prefix(s: &str, prefix: &str, old_num: u64, new_num: u64) -> String {
    let needle = format!("{}{}", prefix, old_num);
    let replacement = format!("{}{}", prefix, new_num);
    let mut out = String::with_capacity(s.len());
    let mut rem = s;
    while let Some(pos) = rem.find(&needle) {
        out.push_str(rem.get(..pos).unwrap_or(""));
        let after = rem.get(pos + needle.len()..).unwrap_or("");
        if after.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            out.push_str(&needle);
        } else {
            out.push_str(&replacement);
        }
        rem = after;
    }
    out.push_str(rem);
    out
}

/// Replaces a stale issue number (`#old_num` or `/issues/old_num`) in `line` with `new_num`.
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-RECONCILE-CHECKLISTS-IN-CONTENT]
pub(crate) fn replace_stale_issue_in_checklist_line(
    line: &str,
    old_num: u64,
    new_num: u64,
) -> Option<String> {
    if old_num == new_num {
        return None;
    }
    let step1 = replace_issue_token_with_prefix(line, "#", old_num, new_num);
    let step2 = replace_issue_token_with_prefix(&step1, "/issues/", old_num, new_num);
    if step2 != line {
        Some(step2)
    } else {
        None
    }
}

/// Updates an existing relative-link checklist line (`](../`, `](./`, or `#<num> - [`) in-place.
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-RECONCILE-CHECKLISTS-IN-CONTENT]
pub(crate) fn reconcile_relative_checklist_line(
    line: &str,
    item_issue: Option<u64>,
    target_issue: Option<u64>,
    is_resolved: bool,
) -> Option<String> {
    if !(line.contains("](../") || line.contains("](./") || line.contains(" - [")) {
        return None;
    }
    let mut current =
        update_checkbox_state_on_line(line, is_resolved).unwrap_or_else(|| line.to_string());
    if let Some(target_num) = target_issue.filter(|&n| n > 0) {
        if let Some(old_num) = item_issue {
            if let Some(repatched) =
                replace_stale_issue_in_checklist_line(&current, old_num, target_num)
            {
                current = repatched;
            }
        } else {
            let trimmed_start = current.trim_start();
            let indent_len = current.len().saturating_sub(trimmed_start.len());
            let indent = current.get(..indent_len).unwrap_or("");
            let box_prefix = trimmed_start.get(..6).unwrap_or("- [ ] ");
            let rest = trimmed_start.get(6..).unwrap_or("").trim_start();
            current = format!("{}{}#{} - {}", indent, box_prefix, target_num, rest);
        }
    }
    Some(current)
}

/// Formats a checklist item for a matched section (relative `../` link when `repo.is_none()` in compile-sysml sections).
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-FORMAT-CANONICAL-CHECKLIST-ITEM]
pub(crate) fn format_section_checklist_item(
    spec: &DiscoveredSpec,
    repo: Option<&str>,
    branch: &str,
    section_heading_line: &str,
) -> String {
    let h_low = section_heading_line.to_lowercase();
    if repo.is_none()
        && (h_low.contains("requirements & checklist") || h_low.contains("associated "))
    {
        let checkbox = if spec.is_resolved { "[x]" } else { "[ ]" };
        let clean_rel = spec
            .rel_path
            .strip_prefix("./")
            .unwrap_or(&spec.rel_path)
            .replace('\\', "/");
        let rel_target = clean_rel
            .strip_prefix("docs/")
            .map(|s| format!("../{}", s))
            .unwrap_or(clean_rel);
        return match spec.issue_number.filter(|&n| n > 0) {
            Some(num) => format!("- {} #{} - [{}]({})", checkbox, num, spec.title, rel_target),
            None => format!("- {} [{}]({})", checkbox, spec.title, rel_target),
        };
    }
    format_canonical_checklist_item(spec, repo, branch)
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
    crate::cmark_reconciler::reconcile_checklists_via_cmark(content, index)
}
