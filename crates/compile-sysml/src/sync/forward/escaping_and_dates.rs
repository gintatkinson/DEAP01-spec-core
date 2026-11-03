#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Specification Date/Issue Resolution and JSON/CommonMark Escaping Helpers.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical
//! software engineering standards. Resolves historical specification dates and tracker issue IDs
//! without wall-clock calls and escapes RFC 8259 JSON and CommonMark table strings safely.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Zero Wall-Clock Calls (`ARCH-06`): `resolve_date` reads authored dates from disk or returns
//!   the deterministic `fallback_iso` literal.
//! - RFC 8259 Escaping: `escape_json_string` escapes quotes, backslashes, and control characters
//!   `U+0000..=U+001F`.
//!
//! ## 3. Preconditions
//! - `target_out_dir` is a valid workspace path and `raw` strings are valid UTF-8 slices.
//!
//! ## 4. Postconditions
//! - Returns deterministic dates, issue IDs, and escaped JSON/Markdown fragments without panicking.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(L)$ over inspected leading metadata lines or string length.
//! - Space Complexity: $O(L)$ for escaped output strings.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Unreadable or absent specification files return `fallback_iso` or `None` cleanly.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0604] [REQ-SYSML-FORWARD-SYNC-NAMING-AND-DATES]

use super::*;

/// Extracts an authored `date` from the leading metadata of an existing specification file, returning `fallback_iso` when absent.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-RESOLVE-DATE]
pub(crate) fn resolve_date(target_out_dir: &Path, rel_path: &str, fallback_iso: &str) -> String {
    let full = target_out_dir.join(rel_path);
    if let Ok(content) = fs::read_to_string(&full) {
        for line in content.lines().take(25) {
            let trimmed = line.trim();
            if trimmed.starts_with("date:") {
                let d = trimmed
                    .trim_start_matches("date:")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if !d.is_empty() {
                    return d.to_string();
                }
            } else if trimmed.starts_with('|') && trimmed.ends_with('|') {
                let inner = trimmed.trim_matches('|');
                let parts: Vec<&str> = inner.split('|').map(str::trim).collect();
                if let [key_cell, value_cell, ..] = parts.as_slice() {
                    let mut key = String::new();
                    for c in key_cell.chars() {
                        if !matches!(c, '*' | '`' | '_' | ':' | '#') {
                            key.push(c);
                        }
                    }
                    let key = key.trim().to_lowercase().replace(' ', "_");
                    if key == "date" {
                        let d = value_cell
                            .trim()
                            .trim_matches('*')
                            .trim_matches('`')
                            .trim_matches('"')
                            .trim_matches('\'');
                        if !d.is_empty() {
                            return d.to_string();
                        }
                    }
                }
            }
        }
    }
    fallback_iso.to_string()
}

/// Parses a positive tracker issue ID from the leading YAML frontmatter or Markdown metadata table of `content`.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-PARSE-ISSUE-ID-FROM-CONTENT]
pub(crate) fn parse_issue_id_from_content(content: &str) -> Option<usize> {
    for line in content.lines().take(25) {
        let trimmed = line.trim();
        if trimmed.starts_with("issue_id:") {
            let d = trimmed
                .trim_start_matches("issue_id:")
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim_start_matches('#');
            if let Ok(parsed) = d.parse::<usize>() {
                if parsed > 0 {
                    return Some(parsed);
                }
            }
        } else if trimmed.starts_with('|') && trimmed.ends_with('|') {
            let inner = trimmed.trim_matches('|');
            let parts: Vec<&str> = inner.split('|').map(str::trim).collect();
            if let [key_cell, value_cell, ..] = parts.as_slice() {
                let mut key = String::new();
                for c in key_cell.chars() {
                    if !matches!(c, '*' | '`' | '_' | ':' | '#') {
                        key.push(c);
                    }
                }
                let key = key.trim().to_lowercase().replace([' ', '-'], "_");
                if key == "issue_id" || key == "issue" || key == "id" {
                    let d = value_cell
                        .trim()
                        .trim_matches('*')
                        .trim_matches('`')
                        .trim_matches('"')
                        .trim_matches('\'')
                        .trim_start_matches('#')
                        .trim();
                    if let Ok(parsed) = d.parse::<usize>() {
                        if parsed > 0 {
                            return Some(parsed);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Resolves an existing positive tracker issue ID from `rel_path` or a same-prefix renamed `.md` sibling on disk without inventing fallback IDs.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-RESOLVE-ISSUE-ID]
pub(crate) fn resolve_existing_issue_id(target_out_dir: &Path, rel_path: &str) -> Option<usize> {
    let full = target_out_dir.join(rel_path);
    if let Ok(content) = fs::read_to_string(&full) {
        if let Some(id) = parse_issue_id_from_content(&content) {
            return Some(id);
        }
    }
    if let Some(parent) = Path::new(rel_path).parent() {
        if let Some(file_name) = Path::new(rel_path).file_name().and_then(|f| f.to_str()) {
            if let Some(prefix) = extract_id_prefix(file_name) {
                let dir_path = target_out_dir.join(parent);
                if let Ok(entries) = fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with(prefix) && name_str.ends_with(".md") {
                            if let Ok(content) = fs::read_to_string(entry.path()) {
                                if let Some(id) = parse_issue_id_from_content(&content) {
                                    return Some(id);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Formats an optional resolved tracker issue ID as `"#<id>"` when positive or `"TBD"` when unallocated.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-RESOLVE-ISSUE-ID]
pub(crate) fn format_issue_id_cell(issue_id: Option<usize>) -> String {
    match issue_id {
        Some(id) if id > 0 => format!("#{}", id),
        _ => "TBD".to_string(),
    }
}

/// Escapes quotes, backslashes, and `U+0000..=U+001F` control characters for RFC 8259 JSON string literal embedding.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-ESCAPE-JSON-STRING]
pub(crate) fn escape_json_string(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 => {
                let code = u32::from(c);
                out.push_str(&format!("\\u{code:04x}"));
            }
            c => out.push(c),
        }
    }
    out
}

/// Formats an AST `AttributeDef::default_value` literal into a typed RFC 8259 JSON value token, quoting non-finite or symbolic expressions safely.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-FORMAT-JSON-DEFAULT-LITERAL]
pub(crate) fn format_json_default_literal(raw_default: &str, json_type: &str) -> String {
    let trimmed = raw_default.trim();
    match json_type {
        "integer" if trimmed.parse::<i64>().is_ok() || trimmed.parse::<u64>().is_ok() => {
            return trimmed.to_string();
        }
        "number" if trimmed.parse::<f64>().is_ok_and(f64::is_finite) => {
            return trimmed.to_string();
        }
        "boolean" if trimmed == "true" || trimmed == "false" => {
            return trimmed.to_string();
        }
        _ => {}
    }
    let unquoted = trimmed
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(trimmed);
    format!("\"{}\"", escape_json_string(unquoted))
}

/// Neutralizes inline Markdown link targets and hardcoded GitHub URLs in AST prose or constraint fragments.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-SANITIZE-PROSE-LINKS]
pub(crate) fn sanitize_prose_links(raw: &str) -> String {
    raw.trim()
        .replace("https://github.com/", "github-ref://")
        .replace("](", "] (")
}

/// Escapes `|` delimiters as `\|` and collapses CR/LF line breaks into single spaces for CommonMark table cells.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-ESCAPE-MD-TABLE-CELL]
pub(crate) fn escape_markdown_table_cell(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.trim().chars() {
        match ch {
            '|' => out.push_str("\\|"),
            '\n' | '\r' => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

/// Resolves the canonical Parent Epic relative Markdown link for `pkg_name` against `ordered_packages`.
/// Realises: [REQ-0604]
pub fn resolve_parent_epic_link(ordered_packages: &[&PackageDef], pkg_name: &str) -> String {
    if pkg_name == "ConOps" {
        "[EPIC-02-ConOps](../epics/EPIC-02-ConOps.md)".to_string()
    } else if pkg_name == "SysML_Model"
        || ordered_packages.first().map(|p| p.name.as_str()) == Some(pkg_name)
    {
        let root = ordered_packages
            .first()
            .map(|p| p.name.as_str())
            .unwrap_or(pkg_name);
        format!("[EPIC-01-{root}](../epics/EPIC-01-{root}.md)")
    } else {
        let ep_idx = ordered_packages
            .iter()
            .position(|p| p.name == pkg_name)
            .map(|idx| idx.saturating_add(1))
            .unwrap_or(1);
        format!("[EPIC-{ep_idx:02}-{pkg_name}](../epics/EPIC-{ep_idx:02}-{pkg_name}.md)")
    }
}

/// Collects 1-based User Story numbers (`s_num`) belonging strictly to `pkg_name` in source order.
/// Realises: [REQ-0604]
pub fn collect_package_story_numbers(story_metas: &[StoryMeta], pkg_name: &str) -> Vec<usize> {
    story_metas
        .iter()
        .filter(|s| s.pkg_name == pkg_name)
        .map(|s| s.s_num)
        .collect()
}

/// Counts distinct `EPIC-*.md` specifications inclusively across both the parent Epic and `child_specs` (`Diagnostic: E0400`).
/// Realises: [REQ-0604]
pub fn count_total_epics_inclusive(
    parent_rel_path: &str,
    parent_is_epic: bool,
    child_specs: &[(String, String)],
) -> usize {
    let mut epic_paths: BTreeSet<&str> = BTreeSet::new();
    let trimmed_parent = parent_rel_path.trim();
    if parent_is_epic
        && !trimmed_parent.is_empty()
        && (trimmed_parent.contains("epics/EPIC-") || trimmed_parent.starts_with("EPIC-"))
    {
        epic_paths.insert(trimmed_parent);
    }
    for (rel, _) in child_specs {
        let r = rel.trim();
        if r.contains("epics/EPIC-") || r.starts_with("EPIC-") {
            epic_paths.insert(r);
        }
    }
    epic_paths.len()
}
