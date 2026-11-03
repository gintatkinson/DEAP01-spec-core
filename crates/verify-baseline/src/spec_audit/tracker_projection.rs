//! Remote GitHub Tracker Projection Parity Gate for `verify-baseline`.
//!
//! Safety & Traceability: Realises: [REQ-REMOTE-TRACKER-PROJECTION-PARITY-GATE]
//! Contract: Fails closed when any local spec's Issue ID is missing remotely, collides with `[AUDIT]`, diverges in title/body, or has premature `status:fixed-resolved`.

use crate::spec_audit::{collect_spec_files, parse_markdown, MarkdownDoc};
use deap_core::markdown_ast::parse_markdown_ast;
use pulldown_cmark::{Event, LinkType, Options, Parser, Tag};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Snapshot of a remote GitHub/GitLab tracker issue used for projection parity verification.
/// Realises: [REQ-REMOTE-TRACKER-PROJECTION-PARITY-GATE]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteTrackerIssue {
    /// Numeric issue identifier on the remote tracker.
    pub number: u64,
    /// Issue title on the remote tracker.
    pub title: String,
    /// Raw Markdown body on the remote tracker.
    pub body: String,
    /// Issue state (e.g., `OPEN` or `CLOSED`).
    pub state: String,
    /// Labels attached to the remote issue.
    pub labels: Vec<String>,
}

/// Projects a local specification Markdown document into the canonical remote tracker body format.
/// Realises: [REQ-REMOTE-TRACKER-PROJECTION-PARITY-GATE]
pub fn project_body_for_tracker_with_prefix(
    content: &str,
    spec_rel_path: &str,
    repo: Option<&str>,
    branch: &str,
    upstream_prefix: Option<&str>,
) -> String {
    let line_ending = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let ast = parse_markdown_ast(content);
    let h2_offset = ast
        .sections
        .iter()
        .find(|s| s.level == 2)
        .map(|s| s.start_offset)
        .unwrap_or(content.len());
    let code_ranges = collect_code_block_ranges(content);

    let mut out_lines: Vec<&str> = Vec::new();
    let mut in_frontmatter_region = true;
    let mut skipped_table_row = false;
    let mut byte_offset = 0usize;

    for raw_line in content.split('\n') {
        let line_len = raw_line.len();
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let trimmed = line.trim();
        let in_code = code_ranges.iter().any(|&(s, e)| byte_offset >= s && byte_offset < e);

        if !in_code && byte_offset >= h2_offset && trimmed.starts_with("## ") {
            in_frontmatter_region = false;
        }
        if !in_code && in_frontmatter_region && trimmed.starts_with('|') && trimmed.ends_with('|') {
            skipped_table_row = true;
            byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
            continue;
        }
        if skipped_table_row && trimmed.is_empty() {
            if out_lines.last().is_some_and(|l| l.trim().is_empty()) {
                byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
                continue;
            }
            skipped_table_row = false;
        }
        out_lines.push(line);
        byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
    }

    let projected = out_lines.join(line_ending);
    match repo {
        Some(r) if !r.trim().is_empty() => {
            let b = if branch.trim().is_empty() { "main" } else { branch.trim() };
            let clean_rel = spec_rel_path.strip_prefix("./").unwrap_or(spec_rel_path).replace('\\', "/");
            let spec_dir = Path::new(&clean_rel).parent().and_then(|p| p.to_str()).unwrap_or("");
            rewrite_projected_links(&projected, spec_dir, r.trim(), b, upstream_prefix)
        }
        _ => projected,
    }
}

/// Collects byte offset ranges `(start, end)` of fenced code blocks in `content`.
/// Realises: [REQ-REMOTE-TRACKER-PROJECTION-PARITY-GATE]
fn collect_code_block_ranges(content: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut open_start: Option<usize> = None;
    for (event, range) in Parser::new_ext(content, Options::all()).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(_)) => open_start = Some(range.start),
            Event::End(pulldown_cmark::TagEnd::CodeBlock) => {
                if let Some(s) = open_start.take() {
                    ranges.push((s, range.end));
                }
            }
            _ => {}
        }
    }
    ranges
}

/// Rewrites inline Markdown links `[label](target)` using `pulldown-cmark` `Tag::Link` spans.
/// Realises: [REQ-REMOTE-TRACKER-PROJECTION-PARITY-GATE]
fn rewrite_projected_links(
    body: &str,
    spec_dir: &str,
    repo: &str,
    branch: &str,
    upstream_prefix: Option<&str>,
) -> String {
    let mut out = String::with_capacity(body.len());
    let mut last_idx = 0usize;

    for (event, range) in Parser::new_ext(body, Options::all()).into_offset_iter() {
        if let Event::Start(Tag::Link { link_type: LinkType::Inline, .. }) = event {
            if range.start < last_idx {
                continue;
            }
            out.push_str(body.get(last_idx..range.start).unwrap_or(""));
            let link_slice = body.get(range.clone()).unwrap_or("");
            if let Some(sep_idx) = link_slice.rfind("](") {
                let label = link_slice.get(1..sep_idx).unwrap_or("");
                let target = link_slice.get(sep_idx + 2..link_slice.len().saturating_sub(1)).unwrap_or("");
                if let Some(rewritten) = rewrite_single_link_target(target, spec_dir, repo, branch, upstream_prefix) {
                    out.push('[');
                    out.push_str(label);
                    out.push_str("](");
                    out.push_str(&rewritten);
                    out.push(')');
                } else {
                    out.push_str(link_slice);
                }
            } else {
                out.push_str(link_slice);
            }
            last_idx = range.end;
        }
    }
    out.push_str(body.get(last_idx..).unwrap_or(""));
    out
}

/// Rewrites a single relative link target into a canonical GitHub `blob` URL.
/// Realises: [REQ-REMOTE-TRACKER-PROJECTION-PARITY-GATE]
fn rewrite_single_link_target(
    target: &str,
    spec_dir: &str,
    repo: &str,
    branch: &str,
    upstream_prefix: Option<&str>,
) -> Option<String> {
    let t = target.trim();
    if t.is_empty()
        || t.starts_with("http://")
        || t.starts_with("https://")
        || t.starts_with("mailto:")
        || t.starts_with('#')
        || t.starts_with('/')
    {
        return None;
    }
    let (path_part, anchor_suffix) = match t.find('#') {
        Some(idx) => (t.get(..idx).unwrap_or(""), t.get(idx..).unwrap_or("")),
        None => (t, ""),
    };
    if path_part.is_empty() {
        return None;
    }
    let root_prefixes = ["docs/", "schema/", ".pipeline/", ".agents/", "crates/", "rules/", "skills/"];
    let stripped = path_part.strip_prefix("./").unwrap_or(path_part);
    let resolved_rel = if root_prefixes.iter().any(|p| stripped.starts_with(p)) {
        normalize_lexical_path(stripped)
    } else if !spec_dir.is_empty() {
        normalize_lexical_path(&format!("{}/{}", spec_dir, path_part))
    } else {
        normalize_lexical_path(path_part)
    };
    if resolved_rel.is_empty() || resolved_rel.starts_with("..") {
        return None;
    }
    let final_path = match upstream_prefix {
        Some(pref) if !pref.trim().trim_matches('/').is_empty() => {
            let clean_pref = pref.trim().trim_matches('/');
            if resolved_rel.starts_with(&format!("{}/", clean_pref)) {
                resolved_rel
            } else {
                format!("{}/{}", clean_pref, resolved_rel)
            }
        }
        _ => resolved_rel,
    };
    Some(format!("https://github.com/{}/blob/{}/{}{}", repo, branch, final_path, anchor_suffix))
}

/// Lexically normalizes `.` and `..` path segments without filesystem access.
/// Realises: [REQ-REMOTE-TRACKER-PROJECTION-PARITY-GATE]
fn normalize_lexical_path(path_str: &str) -> String {
    let unified = path_str.replace('\\', "/");
    let mut stack: Vec<&str> = Vec::new();
    for seg in unified.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                if stack.last().is_some_and(|&last| last != "..") {
                    stack.pop();
                } else {
                    stack.push("..");
                }
            }
            other => stack.push(other),
        }
    }
    stack.join("/")
}

/// Returns true if the Markdown content outside fenced code blocks contains unchecked `- [ ]` task items.
/// Realises: [REQ-REMOTE-TRACKER-PROJECTION-PARITY-GATE]
pub fn has_unchecked_markdown_task_items(raw_markdown: &str) -> bool {
    let code_ranges = collect_code_block_ranges(raw_markdown);
    let mut byte_offset = 0usize;
    for raw_line in raw_markdown.split('\n') {
        let line_len = raw_line.len();
        if !code_ranges.iter().any(|&(s, e)| byte_offset >= s && byte_offset < e) {
            let trimmed = raw_line.trim_start();
            if trimmed.starts_with("- [ ]") || trimmed.starts_with("* [ ]") {
                return true;
            }
        }
        byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
    }
    false
}

/// Parses the numeric issue ID from a specification's metadata table (`issue_id` or `github_issue`).
/// Realises: [REQ-REMOTE-TRACKER-PROJECTION-PARITY-GATE]
pub fn parse_spec_issue_number(doc: &MarkdownDoc) -> Option<u64> {
    let raw = doc
        .table_metadata
        .get("issue_id")
        .or_else(|| doc.table_metadata.get("github_issue"))
        .or_else(|| doc.table_metadata.get("issue"))?;
    let trimmed = raw.trim().trim_matches('`');
    if let Some(hash_idx) = trimmed.rfind('#') {
        let digits: String = trimmed.get(hash_idx + 1..)?.chars().take_while(|c| c.is_ascii_digit()).collect();
        return digits.parse::<u64>().ok();
    }
    let digits: String = trimmed.chars().skip_while(|c| !c.is_ascii_digit()).take_while(|c| c.is_ascii_digit()).collect();
    digits.parse::<u64>().ok()
}

/// Validates a single local specification against the remote tracker snapshot.
/// Realises: [REQ-REMOTE-TRACKER-PROJECTION-PARITY-GATE]
pub fn validate_spec_remote_tracker_projection(
    rel_path: &str,
    doc: &MarkdownDoc,
    content: &str,
    remote_issues: &BTreeMap<u64, RemoteTrackerIssue>,
    projection_ctx: (Option<&str>, &str, Option<&str>),
) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(issue_num) = parse_spec_issue_number(doc) else {
        errors.push(format!("{}: Missing or non-numeric Issue ID in specification metadata table", rel_path));
        return errors;
    };
    let Some(remote) = remote_issues.get(&issue_num) else {
        errors.push(format!("{}: Local specification Issue ID #{} does not exist on the remote tracker", rel_path, issue_num));
        return errors;
    };
    if remote.title.contains("[AUDIT]") {
        errors.push(format!("{}: Issue ID #{} collides with remote [AUDIT] issue '{}'", rel_path, issue_num, remote.title));
    }
    let local_title = doc
        .headings
        .iter()
        .find(|h| h.level == 1)
        .map(|h| h.text.as_str())
        .or_else(|| doc.table_metadata.get("title").map(|s| s.as_str()))
        .unwrap_or("")
        .trim();
    if !local_title.is_empty() && remote.title.trim() != local_title {
        errors.push(format!("{}: Issue #{} remote title '{}' diverges from local specification title '{}'", rel_path, issue_num, remote.title.trim(), local_title));
    }
    let (repo, branch, upstream_prefix) = projection_ctx;
    let projected = project_body_for_tracker_with_prefix(content, rel_path, repo, branch, upstream_prefix);
    if remote.body.trim() != projected.trim() {
        errors.push(format!("{}: Issue #{} remote body diverges from projected local specification body", rel_path, issue_num));
    }
    let is_marked_resolved = remote.labels.iter().any(|l| {
        let lower = l.to_ascii_lowercase();
        lower == "status:fixed-resolved" || lower == "status::fixed-resolved"
    }) || remote.state.eq_ignore_ascii_case("closed");
    if is_marked_resolved && has_unchecked_markdown_task_items(content) {
        errors.push(format!("{}: Issue #{} carries status:fixed-resolved or CLOSED state while local checklist items remain unchecked (- [ ])", rel_path, issue_num));
    }
    errors
}

/// Validates all specifications in `docs/{epics,features,use-cases,user-stories}/` against remote tracker issues.
/// Realises: [REQ-REMOTE-TRACKER-PROJECTION-PARITY-GATE]
pub fn validate_workspace_remote_tracker_projection(
    repo_root: &Path,
    remote_issues: &BTreeMap<u64, RemoteTrackerIssue>,
    projection_ctx: (Option<&str>, &str, Option<&str>),
) -> Vec<String> {
    let mut errors = Vec::new();
    let Ok(spec_files) = collect_spec_files(repo_root, None) else {
        return errors;
    };
    for file_path in spec_files {
        let rel_path = file_path.strip_prefix(repo_root).unwrap_or(&file_path).to_string_lossy().replace('\\', "/");
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Ok(doc) = parse_markdown(&content) {
                errors.extend(validate_spec_remote_tracker_projection(&rel_path, &doc, &content, remote_issues, projection_ctx));
            }
        }
    }
    errors
}
