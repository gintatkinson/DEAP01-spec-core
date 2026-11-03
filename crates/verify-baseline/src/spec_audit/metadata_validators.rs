//! Metadata table, title uniqueness, and filename convention validators for specifications.
//!
//! Enforces CommonMark metadata table headers, spec-type-specific attributes,
//! normalized title uniqueness, and zero-padded ordinal filename conventions.

use crate::spec_audit::markdown_ast::MarkdownDoc;
use crate::spec_audit::metadata::{
    check_filename_anti_stutter, check_title_anti_stutter, has_concatenated_title_metadata,
    is_iso_date, is_semver, normalize_title,
};
use std::collections::BTreeMap;

/// Validates the CommonMark metadata table header and separator within lines 1-10.
/// Realises: [REQ-SPEC-AUDIT-METADATA-VALIDATE-TABLE-HEADER]
fn validate_commonmark_metadata_table(
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
    errors: &mut Vec<String>,
) {
    if content.trim_start().starts_with("---") || doc.frontmatter.is_some() {
        errors.push(format!(
            "{}: Raw YAML frontmatter ('--- ... ---') is strictly forbidden. Specifications must use a native CommonMark metadata table at lines 1-10 per rules/specification-metadata-integrity.md.",
            rel_path
        ));
    }

    let lines: Vec<&str> = content.lines().take(10).collect();
    let header_pos = lines
        .iter()
        .position(|l| l.trim() == "| Attribute | Specification Detail |");
    match header_pos {
        Some(pos) => {
            for (idx, prev_line) in lines.get(..pos).unwrap_or_default().iter().enumerate() {
                let t = prev_line.trim();
                if !t.is_empty() && !t.starts_with("<!--") {
                    errors.push(format!(
                        "{}: Specification metadata table must begin within lines 1-10 (line {} contains unexpected text before metadata table).",
                        rel_path,
                        idx + 1
                    ));
                    break;
                }
            }
            if lines
                .get(pos + 1)
                .is_none_or(|l| l.trim() != "| :--- | :--- |")
            {
                errors.push(format!(
                    "{}: Invalid metadata table separator row at line {}. Expected '| :--- | :--- |'.",
                    rel_path,
                    pos + 2
                ));
            }
        }
        None => {
            errors.push(format!(
                "{}: Missing native CommonMark metadata table at lines 1-10 with exact header '| Attribute | Specification Detail |'.",
                rel_path
            ));
        }
    }
}

/// Looks up a metadata field from `table_metadata` or `frontmatter`.
/// Realises: [REQ-SPEC-AUDIT-METADATA-GET-FIELD]
fn get_doc_field<'a>(doc: &'a MarkdownDoc, key: &str) -> Option<&'a str> {
    doc.table_metadata
        .get(key)
        .map(|s| s.as_str())
        .or_else(|| doc.frontmatter.as_ref().and_then(|fm| fm.get(key).map(|s| s.as_str())))
}

/// Validates spec-type-specific mandatory metadata attributes (Epic, Feature, User Story, Use Case).
/// Realises: [REQ-SPEC-AUDIT-METADATA-VALIDATE-SPEC-TYPE-FIELDS]
fn validate_spec_type_metadata_fields(norm_type: &str, doc: &MarkdownDoc, rel_path: &str, errors: &mut Vec<String>) {
    let require_non_empty = |field: &str, label: &str, errs: &mut Vec<String>| {
        if !get_doc_field(doc, field).is_some_and(|v| !v.trim().is_empty()) {
            errs.push(format!("{}: {} is missing mandatory '{}' metadata.", rel_path, label, field));
        }
    };

    if norm_type.contains("epic") {
        require_non_empty("package", "Epic", errors);
        require_non_empty("subsystem", "Epic", errors);
    } else if norm_type.contains("feature") {
        require_non_empty("parent_epic", "Feature", errors);
        require_non_empty("interface_type", "Feature", errors);
        require_non_empty("schema_containers", "Feature", errors);
    } else if norm_type.contains("user_story") || norm_type.contains("user-story") {
        require_non_empty("parent_epic", "User story", errors);
        require_non_empty("sysml_interaction", "User story", errors);
        require_non_empty("sysml_test_case", "User story", errors);
    } else if norm_type.contains("use_case") || norm_type.contains("use-case") {
        require_non_empty("primary_actor", "Use case", errors);
        require_non_empty("scope", "Use case", errors);
        require_non_empty("level", "Use case", errors);
    }
}

/// Validates frontmatter and CommonMark metadata table attributes for a specification.
/// Realises: [REQ-SPEC-AUDIT-METADATA-VALIDATE-METADATA]
pub fn validate_metadata(spec_type: &str, doc: &MarkdownDoc, content: &str, rel_path: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let norm_type = spec_type.to_lowercase();
    validate_commonmark_metadata_table(doc, content, rel_path, &mut errors);
    if !get_doc_field(doc, "issue_id").is_some_and(|v| !v.trim().is_empty()) {
        errors.push(format!("{}: Missing mandatory metadata field 'issue_id'.", rel_path));
    }
    if !get_doc_field(doc, "type").is_some_and(|v| !v.trim().is_empty()) {
        errors.push(format!("{}: Missing mandatory metadata field 'type'.", rel_path));
    }

    let title_opt = get_doc_field(doc, "title")
        .or_else(|| doc.headings.iter().find(|h| h.level == 1).map(|h| h.text.as_str()));
    match title_opt {
        Some(val) if !val.trim().is_empty() => {
            if has_concatenated_title_metadata(val) {
                errors.push(format!(
                    "{}: Document title contains concatenated metadata attributes ('{}'). Titles must be clean canonical names without embedded versions, dates, or IDs.",
                    rel_path, val
                ));
            }
            if let Some(stutter_err) = check_title_anti_stutter(val) {
                errors.push(format!("{}: Anti-Stutter Gate violation: {}", rel_path, stutter_err));
            }
        }
        _ => errors.push(format!("{}: Missing mandatory document title.", rel_path)),
    }

    if let Some(stutter_err) = check_filename_anti_stutter(rel_path) {
        errors.push(format!("{}: Anti-Stutter Gate violation: {}", rel_path, stutter_err));
    }

    let gen_mode = [
        "generation_mode", "generation-mode", "subagent_drafted", "subagent-drafted",
        "subagentdrafted", "subagent", "subagent_mode",
    ]
    .iter()
    .find_map(|k| get_doc_field(doc, k));
    if !gen_mode.is_some_and(|v| !v.trim().is_empty()) {
        errors.push(format!("{}: Missing mandatory metadata field 'generation_mode'.", rel_path));
    }

    if !get_doc_field(doc, "specification_source").is_some_and(|v| !v.trim().is_empty()) {
        errors.push(format!("{}: Missing mandatory metadata field 'specification_source'.", rel_path));
    }

    validate_spec_type_metadata_fields(&norm_type, doc, rel_path, &mut errors);

    if let Some(date_val) = get_doc_field(doc, "date").or_else(|| get_doc_field(doc, "release_date")) {
        if !date_val.trim().is_empty() && !is_iso_date(date_val) {
            errors.push(format!("{}: Invalid date format: '{}'. Expected ISO 8601 format (YYYY-MM-DD).", rel_path, date_val));
        }
    }

    if let Some(ver_val) = get_doc_field(doc, "version") {
        if !ver_val.trim().is_empty() && !is_semver(ver_val) {
            errors.push(format!("{}: Invalid version format: '{}'. Expected semantic versioning format (v?X.Y[.Z]).", rel_path, ver_val));
        }
    }

    errors
}

/// Validates title uniqueness across specifications within the same spec type.
/// Realises: [REQ-SPEC-AUDIT-METADATA-VALIDATE-TITLE-UNIQUENESS]
pub fn validate_title_uniqueness(docs: &[(String, String, MarkdownDoc)]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut claims: BTreeMap<(String, String), Vec<(&str, &str)>> = BTreeMap::new();

    for (spec_type, rel_path, doc) in docs {
        let norm_spec_type = match spec_type.to_lowercase().as_str() {
            "epic" | "epics" => "epic".to_string(),
            "feature" | "features" => "feature".to_string(),
            "user_story" | "user-story" | "user_stories" | "user-stories" | "us" => {
                "user_story".to_string()
            }
            "use_case" | "use-case" | "use_cases" | "use-cases" | "uc" => "use_case".to_string(),
            other => other.to_string(),
        };

        let raw_title = get_doc_field(doc, "title").or_else(|| {
            doc.headings
                .iter()
                .find(|h| h.level == 1)
                .map(|h| h.text.as_str())
        });

        if let Some(title) = raw_title {
            let norm_title = normalize_title(title);
            if !norm_title.is_empty() {
                claims
                    .entry((norm_spec_type, norm_title))
                    .or_default()
                    .push((rel_path.as_str(), title));
            }
        }
    }

    for ((spec_type, norm_title), colliding) in &claims {
        if let Some(first_claim) = colliding.first().filter(|_| colliding.len() > 1) {
            let listed = colliding
                .iter()
                .map(|(p, t)| format!("{} ('{}')", p, t))
                .collect::<Vec<_>>()
                .join(", ");

            errors.push(format!(
                "{}: duplicate {} title -- {} specifications normalise to '{}' ({}). reconcile_backlog.py addresses tracker issues by normalised title, so duplicate titles collide.",
                first_claim.0,
                spec_type,
                colliding.len(),
                norm_title,
                listed
            ));
        }
    }

    errors
}

/// Validates a single filename against `<expected_prefix>-<ordinal>-<kebab-name>.md`.
/// Realises: [REQ-SPEC-AUDIT-METADATA-VALIDATE-SINGLE-FILENAME]
fn validate_single_filename<'a>(
    name: &'a str,
    expected_prefix: &str,
    dir_rel: &str,
    by_ordinal: &mut BTreeMap<u64, Vec<&'a str>>,
    widths: &mut BTreeMap<usize, Vec<&'a str>>,
) -> Option<String> {
    let stem = name.strip_suffix(".md")?;
    let (prefix, ordinal, kebab_name) = match stem.find('-') {
        Some(i1) => {
            let p = &stem[..i1];
            let rest = &stem[i1 + 1..];
            match rest.find('-') {
                Some(i2) => (p, &rest[..i2], &rest[i2 + 1..]),
                None => ("", "", ""),
            }
        }
        None => ("", "", ""),
    };

    let valid_prefix = !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_alphabetic());
    let valid_ordinal = !ordinal.is_empty() && ordinal.chars().all(|c| c.is_ascii_digit());
    let valid_kebab = !kebab_name.is_empty()
        && kebab_name
            .split('-')
            .all(|seg| !seg.is_empty() && seg.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));

    if !valid_prefix || !valid_ordinal || !valid_kebab {
        return Some(format!(
            "{}/{}: filename does not match the documented convention '{}-<zero-padded-ordinal>-<kebab-name>.md'. Names must be lowercase and dash-separated.",
            dir_rel, name, expected_prefix
        ));
    }

    if !prefix.eq_ignore_ascii_case(expected_prefix) {
        return Some(format!(
            "{}/{}: directory prefix mismatch -- found '{}-', but files in {} must use the '{}-' prefix.",
            dir_rel, name, prefix, dir_rel, expected_prefix
        ));
    }

    if let Some(stutter_err) = check_filename_anti_stutter(name) {
        return Some(format!(
            "{}/{}: Anti-Stutter Gate violation: {}",
            dir_rel, name, stutter_err
        ));
    }

    if let Ok(ord_val) = ordinal.parse::<u64>() {
        by_ordinal.entry(ord_val).or_default().push(name);
        widths.entry(ordinal.len()).or_default().push(name);
    }

    None
}

/// Validates filename conventions (<prefix>-<ordinal>-<kebab-name>.md), ordinal uniqueness,
/// and consistent zero-padding width across a specification directory.
/// Realises: [REQ-SPEC-AUDIT-METADATA-VALIDATE-FILENAMES]
pub fn validate_filenames(spec_type: &str, filenames: &[&str], dir_rel: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let spec_type_lower = spec_type.to_lowercase();
    let expected_prefix = match spec_type_lower.as_str() {
        "epic" | "epics" => "epic",
        "feature" | "features" => "feat",
        "user_story" | "user-story" | "user_stories" | "user-stories" | "us" => "us",
        "use_case" | "use-case" | "use_cases" | "use-cases" | "uc" => "uc",
        _ => spec_type,
    };

    let mut by_ordinal: BTreeMap<u64, Vec<&str>> = BTreeMap::new();
    let mut widths: BTreeMap<usize, Vec<&str>> = BTreeMap::new();

    for &name in filenames {
        if name.starts_with('.') || !name.ends_with(".md") || name == "README.md" {
            continue;
        }
        if let Some(err) = validate_single_filename(
            name,
            expected_prefix,
            dir_rel,
            &mut by_ordinal,
            &mut widths,
        ) {
            errors.push(err);
        }
    }

    for (ord, colliding) in &by_ordinal {
        if colliding.len() > 1 {
            errors.push(format!(
                "{}: duplicate ordinal -- {} is claimed by {} files ({}). Ordinals must be unique, because backlog reconciliation and Epic checklists reference specifications by number.",
                dir_rel,
                ord,
                colliding.len(),
                colliding.join(", ")
            ));
        }
    }

    if widths.len() > 1 {
        let summary = widths
            .iter()
            .map(|(w, files)| format!("{} digit(s): {}", w, files.join(", ")))
            .collect::<Vec<_>>()
            .join("; ");

        errors.push(format!(
            "{}: inconsistent ordinal padding width across the directory ({}). Pick one width and apply it uniformly, otherwise lexical ordering does not match numeric ordering.",
            dir_rel, summary
        ));
    }

    errors
}
