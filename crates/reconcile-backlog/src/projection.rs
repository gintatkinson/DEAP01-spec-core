//! Tracker body projection and `pulldown-cmark` AST link rewriting for GitHub Issue synchronization.
//!
//! Strips frontmatter metadata tables using `deap_core::markdown_ast` section boundaries and
//! rewrites relative repository links into canonical GitHub `blob` URLs via `pulldown-cmark`
//! inline `Tag::Link` byte spans.
//!
//! Realises: [REQ-RECONCILE-BACKLOG-CORE, Issue Tracker as Canonical SSOT]

use crate::frontmatter::{collect_code_block_ranges, is_offset_in_ranges};
use deap_core::markdown_ast::parse_markdown_ast;
use pulldown_cmark::{Event, LinkType, Options, Parser, Tag};
use std::path::Path;

/// [DOC-TIER: A] Projects a local specification Markdown document into a clean tracker issue body.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-PROJECT-BODY-FOR-TRACKER]
/// Contract: Delegates to `project_body_for_tracker_with_prefix` with `upstream_prefix = None`.
/// Bounds & Faults: Time O(N), Space O(N) where N is `content.len()`.
pub fn project_body_for_tracker(
    content: &str,
    spec_rel_path: &str,
    repo: Option<&str>,
    branch: &str,
) -> String {
    project_body_for_tracker_with_prefix(content, spec_rel_path, repo, branch, None)
}

/// [DOC-TIER: A] Projects a local specification Markdown document with an optional upstream subdirectory prefix.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-PROJECT-BODY-FOR-TRACKER-WITH-PREFIX]
/// Contract: Strips the frontmatter table before the first AST H2 section and rewrites relative links via `pulldown-cmark`.
/// Bounds & Faults: Time O(N), Space O(N) where N is `content.len()`.
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
        let in_code = is_offset_in_ranges(byte_offset, &code_ranges);

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
            let b = if branch.trim().is_empty() {
                "main"
            } else {
                branch.trim()
            };
            let clean_rel = spec_rel_path
                .strip_prefix("./")
                .unwrap_or(spec_rel_path)
                .replace('\\', "/");
            let spec_dir = Path::new(&clean_rel)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("");
            rewrite_projected_links(&projected, spec_dir, r.trim(), b, upstream_prefix)
        }
        _ => projected,
    }
}

/// Rewrites inline Markdown links `[label](target)` in `body` using `pulldown-cmark` `Tag::Link` byte spans.
/// Realises: [REQ-RECONCILE-BACKLOG-LIB-REWRITE-PROJECTED-LINKS]
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
        if let Event::Start(Tag::Link {
            link_type: LinkType::Inline,
            ..
        }) = event
        {
            if range.start < last_idx {
                continue;
            }
            out.push_str(body.get(last_idx..range.start).unwrap_or(""));
            let link_slice = body.get(range.clone()).unwrap_or("");
            if let Some(sep_idx) = link_slice.rfind("](") {
                let label = link_slice.get(1..sep_idx).unwrap_or("");
                let target = link_slice
                    .get(sep_idx + 2..link_slice.len().saturating_sub(1))
                    .unwrap_or("");
                if let Some(rewritten) =
                    rewrite_single_link_target(target, spec_dir, repo, branch, upstream_prefix)
                {
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

/// Rewrites a single relative Markdown link target into a canonical GitHub `blob` URL if applicable.
/// Realises: [REQ-RECONCILE-BACKLOG-LIB-REWRITE-SINGLE-LINK-TARGET]
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

    let is_workspace_root_rel = path_part.starts_with("docs/")
        || path_part.starts_with("schema/")
        || path_part.starts_with(".pipeline/")
        || path_part.starts_with(".agents/")
        || path_part.starts_with("crates/")
        || path_part.starts_with("rules/")
        || path_part.starts_with("skills/");

    let is_dot_workspace_root_rel = path_part.starts_with("./docs/")
        || path_part.starts_with("./schema/")
        || path_part.starts_with("./.pipeline/")
        || path_part.starts_with("./.agents/")
        || path_part.starts_with("./crates/")
        || path_part.starts_with("./rules/")
        || path_part.starts_with("./skills/");

    let resolved_rel = if is_workspace_root_rel {
        normalize_lexical_path(path_part)
    } else if is_dot_workspace_root_rel {
        normalize_lexical_path(path_part.get(2..).unwrap_or(""))
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

    Some(format!(
        "https://github.com/{}/blob/{}/{}{}",
        repo, branch, final_path, anchor_suffix
    ))
}

/// Lexically resolves `.` and `..` path segments without filesystem access.
/// Realises: [REQ-RECONCILE-BACKLOG-LIB-NORMALIZE-LEXICAL-PATH]
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
