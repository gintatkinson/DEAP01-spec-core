//! Markdown frontmatter table parser and updater routed through `deap_core::markdown_ast`.
//!
//! Specification documents in DEAP (`docs/epics/`, `docs/features/`, `docs/user-stories/`,
//! `docs/use-cases/`) store their metadata in a Markdown table at the top of the file
//! before the first `## ` (H2) section.
//!
//! Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER, Issue Tracker as Canonical SSOT, Zero Fallback Placeholders]

pub use crate::frontmatter_update::{
    collect_code_block_ranges, first_h2_byte_offset, is_offset_in_ranges, update_frontmatter_field,
    update_frontmatter_fields_batch,
};

use crate::ReconcileError;
use deap_core::markdown_ast::parse_markdown_ast;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// [DOC-TIER: A] Parsed specification frontmatter metadata.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-FRONTMATTER]
/// Contract: Captures canonical specification metadata fields and raw key-value rows.
/// Bounds & Faults: Time O(F), Space O(F) over frontmatter fields.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frontmatter {
    pub document_id: Option<String>,
    pub spec_type: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub github_issue: Option<String>,
    pub parent_epic: Option<String>,
    pub parent_feature: Option<String>,
    pub raw_fields: BTreeMap<String, String>,
}

/// Applies a parsed `(key, val)` frontmatter cell pair to `fm`.
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-PARSE-FRONTMATTER]
pub(crate) fn apply_frontmatter_cell_pair(fm: &mut Frontmatter, key_raw: &str, val_raw: &str) {
    let key = clean_markdown_key(key_raw);
    let val = clean_markdown_value(val_raw);

    if key.is_empty() || key == "field" || key == "attribute" || key == "property" {
        return;
    }

    fm.raw_fields.insert(key.clone(), val.clone());

    match key.as_str() {
        "document id" | "id" | "spec id" | "epic id" | "feature id" | "story id"
        | "use case id" => {
            fm.document_id = Some(val);
        }
        "type" | "document type" | "artifact type" | "spec type" => {
            fm.spec_type = Some(val);
        }
        "title" | "name" => {
            fm.title = Some(val);
        }
        "status" | "state" => {
            fm.status = Some(val);
        }
        "github issue" | "issue" | "tracker issue" | "issue id" | "github" => {
            fm.github_issue = Some(val_raw.trim().to_string());
        }
        "parent epic" | "epic" => {
            fm.parent_epic = Some(val_raw.trim().to_string());
        }
        "parent feature" | "feature" => {
            fm.parent_feature = Some(val_raw.trim().to_string());
        }
        _ => {}
    }
}

/// [DOC-TIER: A] Parses the Markdown metadata table from the top of a specification file via `deap_core::markdown_ast`.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-PARSE-FRONTMATTER]
/// Contract: Inspects AST sections and table rows before the first H2 heading outside fenced code blocks.
/// Bounds & Faults: Time O(N), Space O(F) where N is `content.len()` and F is frontmatter field count.
pub fn parse_frontmatter(content: &str) -> Result<Frontmatter, ReconcileError> {
    crate::cmark_reconciler::parse_frontmatter_via_cmark(content)
}

/// Extracts the first top-level H1 (`# Title`) heading from `content` using `deap_core::markdown_ast`.
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-EXTRACT-H1-TITLE]
pub(crate) fn extract_h1_title(content: &str) -> Option<String> {
    let ast = parse_markdown_ast(content);
    let h2_offset = ast
        .sections
        .iter()
        .find(|s| s.level == 2)
        .map(|s| s.start_offset)
        .unwrap_or(content.len());
    let h1_section = ast
        .sections
        .iter()
        .find(|s| s.level == 1 && s.start_offset < h2_offset)?;
    let raw_slice = content.get(h1_section.start_offset..h1_section.end_offset)?;
    let first_line = raw_slice.lines().next().unwrap_or("").trim();
    let title = first_line
        .strip_prefix("# ")
        .unwrap_or(&h1_section.heading)
        .trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}

/// Strips Markdown formatting (`**`, `*`, `` ` ``) from a table key cell and lowercases it.
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-CLEAN-MARKDOWN-KEY]
pub(crate) fn clean_markdown_key(raw: &str) -> String {
    let mut s = raw.trim().to_string();
    for pat in ["**", "*", "`", "_"] {
        s = s.replace(pat, "");
    }
    s.trim().to_lowercase()
}

/// Strips outer backticks or bold/italic wrappers from a Markdown table value cell.
/// Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-CLEAN-MARKDOWN-VALUE]
fn clean_markdown_value(raw: &str) -> String {
    let mut s = raw.trim();
    while (s.starts_with("**") && s.ends_with("**") && s.len() >= 4)
        || (s.starts_with('`') && s.ends_with('`') && s.len() >= 2)
        || (s.starts_with('*') && s.ends_with('*') && s.len() >= 2)
    {
        if s.starts_with("**") && s.ends_with("**") {
            s = s.get(2..s.len().saturating_sub(2)).unwrap_or("");
        } else {
            s = s.get(1..s.len().saturating_sub(1)).unwrap_or("");
        }
        s = s.trim();
    }
    s.to_string()
}

/// [DOC-TIER: A] Extracts a numeric issue ID from a frontmatter `GitHub Issue` field value.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-EXTRACT-ISSUE-NUMBER]
/// Contract: Parses `#123`, `[#123](url)`, `/issues/123`, or bare positive integers; rejects `0` and placeholders.
/// Bounds & Faults: Time O(N), Space O(N) where N is `val.len()`.
pub fn extract_issue_number(val: &str) -> Option<u64> {
    let cleaned = val.trim().replace('`', "");
    if is_placeholder_issue_id(&cleaned) {
        return None;
    }

    if let Some(hash_idx) = cleaned.find('#') {
        let after_hash = cleaned.get(hash_idx + 1..).unwrap_or("");
        let digits: String = after_hash
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if !digits.is_empty() {
            if let Ok(num) = digits.parse::<u64>() {
                if num > 0 {
                    return Some(num);
                }
            }
        }
    }

    if let Some(issues_idx) = cleaned.find("/issues/") {
        let after_issues = cleaned.get(issues_idx + "/issues/".len()..).unwrap_or("");
        let digits: String = after_issues
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if !digits.is_empty() {
            if let Ok(num) = digits.parse::<u64>() {
                if num > 0 {
                    return Some(num);
                }
            }
        }
    }

    if let Ok(num) = cleaned.parse::<u64>() {
        if num > 0 {
            return Some(num);
        }
    }

    None
}

/// [DOC-TIER: A] Checks if a frontmatter issue ID field contains an unallocated placeholder value.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-IS-PLACEHOLDER-ISSUE-ID]
/// Contract: Returns `true` for empty strings, `pending`, `tbd`, `todo`, `none`, `n/a`, `(to be created)`, `#000`, `#xxx`.
/// Bounds & Faults: Time O(N), Space O(N) where N is `val.len()`.
pub fn is_placeholder_issue_id(val: &str) -> bool {
    let mut s = val.trim().to_lowercase();
    for ch in ['`', '*', '_', '(', ')', '[', ']'] {
        s = s.replace(ch, "");
    }
    let s = s.trim();

    if s.is_empty() {
        return true;
    }

    matches!(
        s,
        "pending"
            | "tbd"
            | "todo"
            | "none"
            | "n/a"
            | "na"
            | "to be created"
            | "unassigned"
            | "unlinked"
            | "-"
            | "--"
            | "#000"
            | "#0"
            | "#xxx"
            | "#tbd"
    ) || s.contains("to be created")
        || s.contains("pending")
}

/// [DOC-TIER: A] Formats an issue number for frontmatter storage (`[#N](url)` or `#N`).
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER-FORMAT-ISSUE-LINK]
/// Contract: Emits `[#N](https://github.com/<repo>/issues/N)` when `repo` is non-empty, else `#N`.
/// Bounds & Faults: Time O(R), Space O(R) where R is repository name length.
pub fn format_issue_link(issue_number: u64, repo: Option<&str>) -> String {
    match repo {
        Some(r) if !r.trim().is_empty() => {
            format!(
                "[#{}](https://github.com/{}/issues/{})",
                issue_number,
                r.trim(),
                issue_number
            )
        }
        _ => format!("#{}", issue_number),
    }
}
