#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! CommonMark table helpers and Checks 1-3 for the engineering design blueprint gate.
//! Realises: [REQ-VERIFY-BASELINE-BLUEPRINT-GATE], [DO-330-TQL-4-BLUEPRINT-GATE]

use crate::blueprint_gate::BlueprintMetadata;
use std::path::{Component, Path, PathBuf};

/// Splits a CommonMark pipe-delimited table row into trimmed cell strings while respecting `\|`.
/// Realises: [REQ-BLUEPRINT-TABLE-ROW-SPLIT]
pub fn split_markdown_table_row(line: &str) -> Option<Vec<String>> {
    crate::cmark_gates::blueprint::split_table_row_cmark(line)
}

/// Checks whether a parsed markdown table row is a valid CommonMark alignment separator row.
/// Realises: [REQ-BLUEPRINT-TABLE-SEPARATOR]
pub fn is_table_separator_row(cells: &[String], expected_cols: usize) -> bool {
    crate::cmark_gates::blueprint::is_table_separator_row_cmark(cells, expected_cols)
}

/// Strips outer whitespace, bold/italic asterisks (`*`), and backticks (`` ` ``) from a cell token.
/// Realises: [REQ-BLUEPRINT-STRIP-DECORATIONS]
pub fn strip_markdown_decorations(raw: &str) -> String {
    raw.trim()
        .trim_matches(|c: char| c.is_whitespace() || c == '*' || c == '`')
        .to_string()
}

/// Strips outer markdown emphasis (`*`) and unwraps single code spans.
/// Realises: [REQ-BLUEPRINT-CLEAN-METADATA-VALUE]
pub(crate) fn clean_metadata_value(raw: &str) -> String {
    let trimmed = raw
        .trim()
        .trim_matches(|c: char| c.is_whitespace() || c == '*');
    if let Some(inner) = trimmed.strip_prefix('`').and_then(|s| s.strip_suffix('`')) {
        if !inner.contains('`') {
            return inner.trim().to_string();
        }
    }
    trimmed.to_string()
}

/// Determines whether a table cell is empty or contains a forbidden placeholder token.
/// Realises: [REQ-BLUEPRINT-PLACEHOLDER-CHECK]
pub fn is_empty_or_placeholder_cell(raw: &str) -> bool {
    let cleaned = strip_markdown_decorations(raw);
    if cleaned.is_empty() {
        return true;
    }
    matches!(
        cleaned.to_ascii_lowercase().as_str(),
        "-" | "--" | "---" | "n/a" | "na" | "tbd" | "todo"
    )
}

/// Validates that `ident` is a non-empty `PascalCase` ASCII alphanumeric identifier.
/// Realises: [REQ-BLUEPRINT-PASCAL-CASE-IDENT]
pub fn is_pascal_case_identifier(ident: &str) -> bool {
    !ident.is_empty()
        && ident.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        && ident.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Validates that `prop_type` is a legal SysML v2 property type.
/// Realises: [REQ-BLUEPRINT-SYSML-PROP-TYPE]
pub fn is_valid_sysml_property_type(prop_type: &str) -> bool {
    matches!(prop_type, "Integer" | "Boolean" | "Real" | "String")
        || is_pascal_case_identifier(prop_type)
}

/// Checks whether a path string touches a forbidden ephemeral directory prefix.
/// Realises: [REQ-BLUEPRINT-EPHEMERAL-PREFIX]
fn has_ephemeral_prefix(path_str: &str) -> bool {
    const PREFIXES: [&str; 4] = [
        "/tmp",
        "/private/tmp",
        "/var/folders",
        "/private/var/folders",
    ];
    PREFIXES.iter().any(|prefix| {
        path_str == *prefix
            || path_str
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

/// Rejects ephemeral `/tmp` or `/var/folders` paths for Check 1.
/// Realises: [REQ-BLUEPRINT-CHECK-1-PATH]
fn verify_non_ephemeral_roots(
    workspace_root: &Path,
    blueprint_path: &Path,
) -> Result<(), String> {
    let bp_str = blueprint_path.to_string_lossy().replace('\\', "/");
    let ws_str = workspace_root.to_string_lossy().replace('\\', "/");
    let temp_dir = std::env::temp_dir();

    if bp_str == "/tmp"
        || bp_str.starts_with("/tmp/")
        || bp_str == "/private/tmp"
        || bp_str.starts_with("/private/tmp/")
        || ws_str == "/tmp"
        || ws_str.starts_with("/tmp/")
        || ws_str == "/private/tmp"
        || ws_str.starts_with("/private/tmp/")
    {
        return Err(format!(
            "Check 1 failed: ephemeral path '{}' under /tmp is forbidden; blueprint must reside under docs/designs/blueprint-*.md",
            blueprint_path.display()
        ));
    }

    let ws_is_isolated_test_subdir = workspace_root != temp_dir
        && workspace_root.starts_with(&temp_dir)
        && workspace_root.is_dir();

    let bp_in_ephemeral = has_ephemeral_prefix(&bp_str) || blueprint_path.starts_with(&temp_dir);
    if bp_in_ephemeral
        && !(ws_is_isolated_test_subdir && blueprint_path.starts_with(workspace_root))
    {
        return Err(format!(
            "Check 1 failed: ephemeral path '{}' (/tmp/, /var/folders/, or std::env::temp_dir()) is forbidden; blueprint must reside under docs/designs/blueprint-*.md",
            blueprint_path.display()
        ));
    }

    if (has_ephemeral_prefix(&ws_str) || workspace_root.starts_with(&temp_dir))
        && !ws_is_isolated_test_subdir
    {
        return Err(format!(
            "Check 1 failed: workspace root '{}' cannot be an ephemeral directory",
            workspace_root.display()
        ));
    }
    Ok(())
}

/// Check 1 (Zero-Ephemeral Path Gate): Validates that `blueprint_path` resides under `docs/designs/blueprint-*.md`.
/// Realises: [REQ-BLUEPRINT-CHECK-1-PATH]
pub fn check_1_zero_ephemeral_path(
    workspace_root: &Path,
    blueprint_path: &Path,
) -> Result<PathBuf, String> {
    if blueprint_path
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return Err(format!(
            "Check 1 failed: blueprint path '{}' must not contain parent directory traversal ('..')",
            blueprint_path.display()
        ));
    }
    verify_non_ephemeral_roots(workspace_root, blueprint_path)?;

    let rel_path: &Path = if blueprint_path.is_absolute() {
        blueprint_path.strip_prefix(workspace_root).map_err(|_| {
            format!(
                "Check 1 failed: blueprint path '{}' does not reside inside workspace root '{}'",
                blueprint_path.display(),
                workspace_root.display()
            )
        })?
    } else if !workspace_root.as_os_str().is_empty()
        && workspace_root != Path::new(".")
        && blueprint_path.starts_with(workspace_root)
    {
        blueprint_path
            .strip_prefix(workspace_root)
            .unwrap_or(blueprint_path)
    } else {
        blueprint_path
    };

    let normal_components: Vec<&str> = rel_path
        .components()
        .filter_map(|c| match c {
            Component::Normal(os_str) => os_str.to_str(),
            _ => None,
        })
        .collect();

    if normal_components.len() != 3
        || normal_components.first().copied() != Some("docs")
        || normal_components.get(1).copied() != Some("designs")
    {
        return Err(format!(
            "Check 1 failed: blueprint path '{}' must reside under 'docs/designs/blueprint-*.md' relative to workspace root '{}'",
            blueprint_path.display(),
            workspace_root.display()
        ));
    }

    let file_name = normal_components.get(2).copied().unwrap_or("");
    if !file_name.starts_with("blueprint-")
        || !file_name.ends_with(".md")
        || file_name.len() <= "blueprint-.md".len()
    {
        return Err(format!(
            "Check 1 failed: blueprint file name '{file_name}' must match 'blueprint-<slug>.md' with a non-empty slug"
        ));
    }

    Ok(workspace_root.join("docs").join("designs").join(file_name))
}

/// Validates required fields, `REQ-` prefix, and `subagent` mode on a parsed `BlueprintMetadata`.
/// Realises: [REQ-BLUEPRINT-CHECK-2-METADATA]
fn validate_parsed_metadata_fields(meta: &mut BlueprintMetadata, errors: &mut Vec<String>) {
    let required_fields: [(&str, &str); 8] = [
        ("Blueprint ID", &meta.blueprint_id),
        ("Requirement ID", &meta.requirement_id),
        ("Title", &meta.title),
        ("Status", &meta.status),
        ("Target Crates", &meta.target_crates),
        ("Tracked Issues", &meta.tracked_issues),
        ("Generation Mode", &meta.generation_mode),
        ("Specification Source", &meta.specification_source),
    ];

    for (field_name, field_val) in required_fields {
        if field_val.trim().is_empty() {
            errors.push(format!(
                "Check 2 failed: missing or empty required metadata key '{field_name}' in lines 1-12"
            ));
        }
    }

    if !meta.requirement_id.is_empty() && !meta.requirement_id.contains("REQ-") {
        errors.push(format!(
            "Check 2 failed: 'Requirement ID' value '{}' must contain/start with 'REQ-'",
            meta.requirement_id
        ));
    }

    if !meta.generation_mode.is_empty() {
        let normalized_mode = strip_markdown_decorations(&meta.generation_mode);
        if !normalized_mode.eq_ignore_ascii_case("subagent") {
            errors.push(format!(
                "Check 2 failed: 'Generation Mode' value '{}' must equal 'subagent' case-insensitively",
                meta.generation_mode
            ));
        } else {
            meta.generation_mode = normalized_mode.to_ascii_lowercase();
        }
    }
}

/// Check 2 (CommonMark Metadata Table Gate): Inspects lines 1-12 for a valid two-column CommonMark metadata table.
/// Realises: [REQ-BLUEPRINT-CHECK-2-METADATA]
pub fn check_2_metadata_table(content: &str) -> Result<BlueprintMetadata, Vec<String>> {
    let top_lines: Vec<&str> = content.lines().take(12).collect();
    let mut errors = Vec::new();
    let mut meta = BlueprintMetadata::default();

    let header_idx = top_lines.iter().position(|line| {
        split_markdown_table_row(line).is_some_and(|cols| {
            cols.len() == 2
                && strip_markdown_decorations(cols.first().map_or("", String::as_str))
                    .eq_ignore_ascii_case("Attribute")
                && strip_markdown_decorations(cols.get(1).map_or("", String::as_str))
                    .eq_ignore_ascii_case("Specification Detail")
        })
    });

    let Some(h_idx) = header_idx else {
        return Err(vec![
            "Check 2 failed: lines 1-12 must contain a 2-column CommonMark metadata table header '| Attribute | Specification Detail |'"
                .to_string(),
        ]);
    };

    let sep_line = top_lines.get(h_idx.saturating_add(1)).copied().unwrap_or("");
    if !split_markdown_table_row(sep_line).is_some_and(|cols| is_table_separator_row(&cols, 2)) {
        errors.push(
            "Check 2 failed: metadata table header must be immediately followed by a 2-column CommonMark separator row ('| :--- | :--- |')"
                .to_string(),
        );
    }

    for (offset, line) in top_lines.iter().skip(h_idx.saturating_add(2)).enumerate() {
        let line_no = h_idx.saturating_add(3).saturating_add(offset);
        let trimmed = line.trim();
        let Some(cols) = (!trimmed.is_empty()).then(|| split_markdown_table_row(trimmed)).flatten() else {
            break;
        };
        if cols.len() != 2 {
            errors.push(format!(
                "Check 2 failed: metadata row on line {line_no} has {} columns; expected 2",
                cols.len()
            ));
            continue;
        }
        let raw_key = strip_markdown_decorations(cols.first().map_or("", String::as_str));
        let raw_val = cols.get(1).map_or("", String::as_str);
        if is_empty_or_placeholder_cell(raw_val) {
            errors.push(format!(
                "Check 2 failed: metadata attribute '{raw_key}' on line {line_no} has an empty or placeholder value"
            ));
            continue;
        }
        let cleaned_val = clean_metadata_value(raw_val);
        match raw_key.to_ascii_lowercase().as_str() {
            "blueprint id" => meta.blueprint_id = cleaned_val,
            "requirement id" => meta.requirement_id = cleaned_val,
            "title" => meta.title = cleaned_val,
            "status" => meta.status = cleaned_val,
            "target crates" => meta.target_crates = cleaned_val,
            "tracked issues" => meta.tracked_issues = cleaned_val,
            "generation mode" => meta.generation_mode = strip_markdown_decorations(raw_val),
            "specification source" => meta.specification_source = cleaned_val,
            _ => {}
        }
    }

    validate_parsed_metadata_fields(&mut meta, &mut errors);
    if errors.is_empty() {
        Ok(meta)
    } else {
        Err(errors)
    }
}
