//! Placeholder token detection and metadata header row scanning for Markdown AST auditing.
//!
//! Identifies unresolved template tokens, bracketed placeholders, and stub strings
//! while respecting CommonMark metadata table header tolerances.

use crate::spec_audit::markdown_ast::PlaceholderFinding;

/// Normalizes a raw first-column table cell into a canonical snake_case metadata key.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-NORMALIZE-META-KEY]
pub(crate) fn normalize_table_metadata_key(first_cell: &str) -> String {
    let mut key = String::with_capacity(first_cell.len());
    for c in first_cell.chars() {
        if !matches!(c, '*' | '`' | ':' | '#') {
            key.push(c);
        }
    }
    let key = key.trim().to_lowercase();
    let mut normalized = String::with_capacity(key.len());
    let mut last_was_sep = false;
    for c in key.chars() {
        if c.is_whitespace() || c == '-' || c == '/' || c == '_' {
            if !last_was_sep {
                normalized.push('_');
                last_was_sep = true;
            }
        } else {
            normalized.push(c);
            last_was_sep = false;
        }
    }
    normalized.trim_matches('_').to_string()
}

/// Checks whether a single line is a CommonMark two-column metadata header table row.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-IS-METADATA-HEADER-TABLE-ROW]
pub fn is_metadata_header_table_row(line: &str) -> bool {
    let stripped = line.trim();
    if !stripped.starts_with('|') || !stripped.ends_with('|') {
        return false;
    }
    let inner = match stripped.strip_prefix('|').and_then(|s| s.strip_suffix('|')) {
        Some(s) => s,
        None => return false,
    };
    let cells: Vec<&str> = inner.split('|').map(|c| c.trim()).collect();
    if cells.len() < 2 {
        return false;
    }
    let first_cell = match cells.first() {
        Some(&c) => c,
        None => return false,
    };
    let normalized = normalize_table_metadata_key(first_cell);

    matches!(
        normalized.as_str(),
        "issue_id"
            | "issue"
            | "parent_epic"
            | "feature_id"
            | "epic_id"
            | "epic_issue_id"
            | "status"
            | "doc_status"
            | "document_status"
            | "state"
            | "user_story_id"
            | "use_case_id"
            | "story_id"
            | "id"
            | "doc_id"
            | "document_id"
            | "attribute"
            | "key"
            | "field"
            | "property"
            | "parent"
            | "epic"
            | "feature"
            | "type"
            | "title"
            | "package"
            | "subsystem"
            | "generation_mode"
            | "subagent_drafted"
            | "subagent"
            | "subagent_mode"
            | "specification_source"
            | "interface_type"
            | "schema_containers"
            | "version"
            | "date"
            | "release_date"
            | "target_baseline"
            | "sysml_interaction"
            | "sysml_test_case"
            | "primary_actor"
            | "scope"
            | "level"
    )
}

/// Checks whether a placeholder token is permitted within a metadata header row.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-IS-ALLOWED-METADATA-PLACEHOLDER]
fn is_allowed_metadata_placeholder(token: &str) -> bool {
    let trimmed = token.trim();
    if trimmed.eq_ignore_ascii_case("#tbd") {
        return true;
    }
    let without_hash = trimmed.strip_prefix('#').unwrap_or(trimmed);
    let lower = without_hash.to_lowercase();
    matches!(
        lower.as_str(),
        "[issueid]" | "[epicid]" | "[featureid]" | "[epicissueid]"
    )
}

/// Checks for `#[...]` and `#TBD` reference tokens on a single line.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-CHECK-HASH-TOKENS]
fn check_hash_and_tbd_tokens(line: &str, line_lower: &str, is_meta_row: bool) -> Option<&'static str> {
    let mut rest = line;
    while let Some(start) = rest.find("#[") {
        let after = &rest[start + 2..];
        if let Some(end) = after.find(']') {
            let token = &rest[start..=start + 2 + end];
            if !(is_meta_row && is_allowed_metadata_placeholder(token)) {
                return Some("unresolved issue reference token");
            }
            rest = &after[end + 1..];
        } else {
            break;
        }
    }

    let mut idx = 0;
    while let Some(pos) = line_lower.get(idx..).and_then(|s| s.find("#tbd")) {
        let abs_pos = idx + pos;
        let next_idx = abs_pos + 4;
        let boundary = next_idx == line_lower.len()
            || line_lower
                .as_bytes()
                .get(next_idx)
                .is_none_or(|b| !b.is_ascii_alphanumeric());
        if boundary {
            if let Some(token) = line.get(abs_pos..next_idx) {
                if !(is_meta_row && is_allowed_metadata_placeholder(token)) {
                    return Some("unresolved reference token");
                }
            }
        }
        idx = next_idx;
    }
    None
}

/// Checks bracketed `[...]` placeholder tokens on a single line.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-CHECK-BRACKET-TOKENS]
fn check_bracketed_placeholder_tokens(line: &str, is_meta_row: bool) -> Option<&'static str> {
    const PREFIXES: &[&str] = &[
        "feature",
        "feat",
        "epic",
        "user story",
        "user-story",
        "user_story",
        "use case",
        "use-case",
        "use_case",
        "story",
        "issue",
        "us",
        "uc",
    ];
    let mut rest = line;
    while let Some(open) = rest.find('[') {
        let after = rest.get(open + 1..).unwrap_or("");
        if let Some(close) = after.find(']') {
            let content = after.get(..close).unwrap_or("");
            let lower_content = content.trim().to_lowercase();
            let full_token = rest.get(open..=open + 1 + close).unwrap_or("");

            let matched_id = PREFIXES.iter().any(|p| {
                lower_content.strip_prefix(p).is_some_and(|after_p| {
                    let rem =
                        after_p.trim_matches(|c: char| c == '-' || c == '_' || c.is_whitespace());
                    rem == "id" || rem == "issueid"
                })
            });

            if matched_id {
                if !(is_meta_row && is_allowed_metadata_placeholder(full_token)) {
                    return Some("unresolved issue reference token");
                }
            } else if matches!(
                lower_content.as_str(),
                "title"
                    | "epic title"
                    | "feature title"
                    | "user story title"
                    | "use case title"
                    | "user-story title"
                    | "use-case title"
                    | "user_story title"
                    | "use_case title"
            ) {
                return Some("unpopulated template title");
            } else if lower_content.starts_with("populate:") {
                return Some("unreplaced [POPULATE:] placeholder token");
            } else {
                let single_space = lower_content.split_whitespace().collect::<Vec<_>>().join(" ");
                if matches!(
                    single_space.as_str(),
                    "repository base url"
                        | "branch name"
                        | "spec reference"
                        | "sysmlinteractionname"
                        | "sysmltestcasename"
                ) {
                    return Some("unpopulated template token");
                }
            }
            rest = &after[close + 1..];
        } else {
            break;
        }
    }
    None
}

/// Checks template escape tokens, placeholder file paths, and conditional stubs on a line.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-CHECK-TEMPLATE-TOKENS]
fn check_template_and_path_tokens(
    line: &str,
    line_lower: &str,
    include_conditional: bool,
) -> Option<&'static str> {
    let line_upper = line.to_uppercase();
    if line_upper.contains("{{REQUIRED_JUSTIFICATION}}")
        || line_upper.contains("{{REQUIRED_SOURCE_REF}}")
    {
        return Some("unreplaced template escape token");
    }
    if line_lower.contains("semantic linkage justification") {
        return Some("template text left in place of a written linkage justification");
    }
    for kw in &["epic-xx-name", "feat-xx-name", "us-xx-name", "uc-xx-name"] {
        if let Some(pos) = line_lower.find(kw) {
            let before_ok = pos == 0
                || line_lower
                    .as_bytes()
                    .get(pos - 1)
                    .is_none_or(|b| !b.is_ascii_alphanumeric());
            let after_pos = pos + kw.len();
            let after_ok = after_pos == line_lower.len()
                || line_lower
                    .as_bytes()
                    .get(after_pos)
                    .is_none_or(|b| !b.is_ascii_alphanumeric());
            if before_ok && after_ok {
                return Some("placeholder file path");
            }
        }
    }
    if line_lower.contains("<blob_path>") {
        return Some("unpopulated template path");
    }
    if include_conditional
        && (line_lower.contains("*(none)*")
            || line_lower.contains("*(none registered)*")
            || line_lower.contains("*(to be populated)*")
            || line_lower.contains("*(tbd)*")
            || line_lower.contains("*(n/a)*")
            || line_lower.contains("*to be populated*")
            || line_lower.contains("*tbd*")
            || line_lower.contains("*n/a*"))
    {
        return Some("placeholder stub");
    }
    None
}

/// Detects unresolved placeholders on a single line of Markdown text.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-DETECT-PLACEHOLDER-ON-LINE]
fn detect_placeholder_on_line(
    line: &str,
    is_meta_row: bool,
    include_conditional: bool,
) -> Option<&'static str> {
    let line_lower = line.to_lowercase();
    check_hash_and_tbd_tokens(line, &line_lower, is_meta_row)
        .or_else(|| check_bracketed_placeholder_tokens(line, is_meta_row))
        .or_else(|| check_template_and_path_tokens(line, &line_lower, include_conditional))
}

/// Scans Markdown text and identifies unresolved placeholder findings.
/// Realises: [REQ-SPEC-AUDIT-MARKDOWN-AST-FIND-UNRESOLVED-PLACEHOLDERS]
pub fn find_unresolved_placeholders(
    content: &str,
    include_conditional: bool,
) -> Vec<PlaceholderFinding> {
    let mut findings = Vec::new();
    for (lineno_0idx, line) in content.lines().enumerate() {
        let lineno = lineno_0idx + 1;
        let is_meta_row = is_metadata_header_table_row(line);
        if let Some(label) = detect_placeholder_on_line(line, is_meta_row, include_conditional) {
            findings.push(PlaceholderFinding {
                line: lineno,
                label: label.to_string(),
                text: line.trim().to_string(),
            });
        }
    }
    findings
}
