//! Source reference clause integrity and cardinality bounds validators for `provenance.rs`.

use crate::spec_audit::markdown_ast::MarkdownDoc;
use crate::spec_audit::provenance::PROHIBITED_PLACEHOLDERS;
use crate::spec_audit::{SpecAuditError, SpecAuditFinding};

/// Extracts the text slice and starting line number of the `## Source References` section.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-EXTRACT-SOURCE-REFERENCES-SECTION]
pub fn extract_source_references_section(content: &str) -> Option<(usize, &str)> {
    let mut target_line = None;
    let mut target_level = 2;
    let mut start_byte = 0;

    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            let after_hash = trimmed[level..].trim().to_lowercase();

            if after_hash.contains("source reference")
                || (after_hash.contains("source") && after_hash.contains("reference"))
            {
                target_line = Some(idx + 1);
                target_level = level;
                let line_start = line.as_ptr() as usize - content.as_ptr() as usize;
                start_byte = line_start + line.len();
                if content.as_bytes().get(start_byte) == Some(&b'\n') {
                    start_byte += 1;
                }
                break;
            }
        }
    }

    let line_no = target_line?;
    if start_byte >= content.len() {
        return Some((line_no, ""));
    }

    let rem = &content[start_byte..];
    let mut end_byte = rem.len();

    for line in rem.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            if level <= target_level {
                let pos = line.as_ptr() as usize - rem.as_ptr() as usize;
                end_byte = pos;
                break;
            }
        }
    }

    Some((line_no, &rem[..end_byte]))
}

/// Verifies that `## Source References` sections in specifications cite verbatim clause numbers.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-VALIDATE-SOURCE-REFERENCE-CLAUSE-INTEGRITY]
pub fn validate_source_reference_clause_integrity(
    _doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let mut findings = Vec::new();

    let (sec_line, sec_text) = match extract_source_references_section(content) {
        Some((l, t)) => (l, t),
        None => {
            findings.push(SpecAuditFinding::new(
                "source-reference-section-missing",
                format!("Specification '{}' is missing a mandatory 'Source References' section.", rel_path),
                rel_path,
                None,
            ));
            return Ok(findings);
        }
    };

    if sec_text.trim().is_empty() {
        findings.push(SpecAuditFinding::new(
            "source-reference-section-empty",
            format!("Specification '{}' has an empty 'Source References' section on line {}. It must cite authoritative schema paths and normative standards.", rel_path, sec_line),
            rel_path,
            Some(sec_line),
        ));
        return Ok(findings);
    }

    let filename = rel_path.rsplit('/').next().unwrap_or(rel_path);

    for (offset, line) in sec_text.lines().enumerate() {
        let current_line = sec_line + offset + 1;
        let lower_line = line.trim().to_lowercase();

        for &placeholder in PROHIBITED_PLACEHOLDERS {
            if lower_line.contains(placeholder) {
                findings.push(SpecAuditFinding::new(
                    "source-reference-placeholder-detected",
                    format!("Specification '{}' contains unresolved placeholder '{}' in 'Source References' on line {}.", rel_path, placeholder, current_line),
                    rel_path,
                    Some(current_line),
                ));
                break;
            }
        }

        if lower_line.contains(filename) || lower_line.contains("(self)") {
            findings.push(SpecAuditFinding::new(
                "source-reference-self-referential",
                format!("Specification '{}' contains self-referential link '{}' in 'Source References' on line {}.", rel_path, filename, current_line),
                rel_path,
                Some(current_line),
            ));
        }
    }

    let lower_sec = sec_text.to_lowercase();
    let has_schema_path = lower_sec.contains("schema/") || lower_sec.contains(".sysml");
    let has_verbatim_clause =
        lower_sec.contains('§') || lower_sec.contains("clause ") || lower_sec.contains("section ");

    if !has_schema_path && !has_verbatim_clause {
        findings.push(SpecAuditFinding::new(
            "source-reference-clause-missing",
            format!("Specification '{}' 'Source References' section on line {} does not cite verbatim clause numbers (e.g. 'ISO/IEC/IEEE 29148:2018 §6.4.2') or schema paths (e.g. 'schema/*.sysml').", rel_path, sec_line),
            rel_path,
            Some(sec_line),
        ));
    }

    Ok(findings)
}

/// Validates whether a line matches the canonical Epic feature checklist grammar.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-VALID-EPIC-FEATURE-CHECKLIST-ITEM]
pub fn is_valid_epic_feature_checklist_item(line: &str) -> bool {
    let trimmed = line.trim();
    let Some(after_box) = trimmed
        .strip_prefix("- [ ] ")
        .or_else(|| trimmed.strip_prefix("- [x] "))
        .or_else(|| trimmed.strip_prefix("- [X] "))
        .or_else(|| trimmed.strip_prefix("* [ ] "))
        .or_else(|| trimmed.strip_prefix("* [x] "))
        .or_else(|| trimmed.strip_prefix("* [X] "))
    else {
        return false;
    };

    let after_ws = after_box.trim_start();
    let after_issue = if let Some(rest) = after_ws
        .strip_prefix("#[issueid]")
        .or_else(|| after_ws.strip_prefix("#[IssueID]"))
        .or_else(|| after_ws.strip_prefix("#[issue_id]"))
    {
        rest
    } else if let Some(rest) = after_ws.strip_prefix('#') {
        let digits_len = rest.chars().take_while(|c| c.is_ascii_digit()).count();
        if digits_len == 0 {
            return false;
        }
        &rest[digits_len..]
    } else {
        return false;
    };

    let after_sep = after_issue
        .trim_start()
        .trim_start_matches('-')
        .trim_start_matches(':')
        .trim_start();
    if !after_sep.starts_with('[') {
        return false;
    }
    let Some(sep_pos) = after_sep.find("](") else {
        return false;
    };
    let after_target_open = &after_sep[sep_pos + 2..];
    let Some(close_paren_rel) = after_target_open.find(')') else {
        return false;
    };
    let target = &after_target_open[..close_paren_rel];
    let title = &after_sep[1..sep_pos];

    let lower_title = title.to_lowercase();
    let lower_target = target.to_lowercase();
    if !lower_title.contains("feat-")
        && !lower_title.contains("feature")
        && !lower_target.contains("feat-")
        && !lower_target.contains("features/")
    {
        return false;
    }

    let after_link = after_target_open
        .get(close_paren_rel + 1..)
        .unwrap_or("")
        .trim();
    if !after_link.is_empty()
        && (!after_link.starts_with('(') || !after_link.ends_with(')') || after_link.len() < 3)
    {
        return false;
    }

    true
}

/// Counts referenced features in an Epic specification.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-COUNT-EPIC-FEATURES]
pub fn count_epic_features(content: &str) -> usize {
    let mut count = 0;
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if !in_code_block
            && (trimmed.starts_with("- [ ]")
                || trimmed.starts_with("- [x]")
                || trimmed.starts_with("- [X]")
                || trimmed.starts_with("* [ ]")
                || trimmed.starts_with("* [x]")
                || trimmed.starts_with("* [X]"))
            && is_valid_epic_feature_checklist_item(trimmed)
        {
            count += 1;
        }
    }
    count
}

/// Counts acceptance criteria in a Feature specification.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-COUNT-FEATURE-ACCEPTANCE-CRITERIA]
pub fn count_feature_acceptance_criteria(content: &str) -> usize {
    let mut count = 0;
    let mut in_code_block = false;
    let mut in_ac_section = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        if trimmed.starts_with('#') {
            let lower_heading = trimmed.to_lowercase();
            if lower_heading.contains("associated use cases")
                || lower_heading.contains("associated user stories")
                || lower_heading.contains("linked use cases")
                || lower_heading.contains("linked user stories")
                || lower_heading.contains("traceability")
            {
                in_ac_section = false;
            } else if lower_heading.contains("acceptance criteria")
                || lower_heading.contains("verification")
                || lower_heading.contains("conformance")
                || lower_heading.contains("requirements")
            {
                in_ac_section = true;
            } else if trimmed.starts_with("## ") {
                in_ac_section = false;
            }
            continue;
        }

        let lower = trimmed.to_lowercase();
        if trimmed.starts_with("- [ ]")
            || trimmed.starts_with("- [x]")
            || trimmed.starts_with("* [ ]")
            || trimmed.starts_with("* [x]")
        {
            if !trimmed.get(5..).unwrap_or("").trim_start().starts_with('#') {
                count += 1;
            }
        } else if in_ac_section
            && (trimmed.starts_with('-')
                || trimmed.starts_with('*')
                || (trimmed.as_bytes().first().is_some_and(|b| b.is_ascii_digit())
                    && trimmed.as_bytes().get(1) == Some(&b'.')))
            && (lower.contains("ac-")
                || lower.contains("req-")
                || lower.contains("shall")
                || lower.contains("given")
                || lower.contains("when")
                || lower.contains("then"))
        {
            count += 1;
        }
    }
    count
}

/// Audits specification cardinality bounds across Epics and Features.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-VALIDATE-CARDINALITY-BOUNDS]
pub fn validate_cardinality_bounds(
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let mut findings = Vec::new();
    let mut is_epic = false;
    let mut is_feature = false;

    if let Some(t) = doc
        .table_metadata
        .get("type")
        .map(|s| s.as_str())
        .or_else(|| doc.frontmatter.as_ref().and_then(|fm| fm.get("type").map(|s| s.as_str())))
    {
        if t.eq_ignore_ascii_case("epic") {
            is_epic = true;
        } else if t.eq_ignore_ascii_case("feature") {
            is_feature = true;
        }
    }

    let lower_path = rel_path.to_lowercase();
    if !is_epic && !is_feature {
        if lower_path.contains("/epics/") || lower_path.contains("epic-") {
            is_epic = true;
        } else if lower_path.contains("/features/") || lower_path.contains("feat-") {
            is_feature = true;
        }
    }

    if is_epic {
        let feature_count = count_epic_features(content);
        if feature_count < 3 {
            findings.push(SpecAuditFinding::new(
                "cardinality-bounds-epic-underflow",
                format!("Epic specification '{}' references only {} feature(s); recommended cardinality bounds are 3--15 features.", rel_path, feature_count),
                rel_path,
                None,
            ));
        } else if feature_count > 15 {
            findings.push(SpecAuditFinding::new(
                "cardinality-bounds-epic-overflow",
                format!("Epic specification '{}' references {} features, exceeding recommended upper bound of 15 features (3--15 recommended).", rel_path, feature_count),
                rel_path,
                None,
            ));
        }
    } else if is_feature {
        let ac_count = count_feature_acceptance_criteria(content);
        if ac_count < 3 {
            findings.push(SpecAuditFinding::new(
                "cardinality-bounds-feature-underflow",
                format!("Feature specification '{}' defines only {} acceptance criteria; recommended cardinality bounds are 3--10 criteria.", rel_path, ac_count),
                rel_path,
                None,
            ));
        } else if ac_count > 10 {
            findings.push(SpecAuditFinding::new(
                "cardinality-bounds-feature-overflow",
                format!("Feature specification '{}' defines {} acceptance criteria, exceeding recommended upper bound of 10 criteria (3--10 recommended).", rel_path, ac_count),
                rel_path,
                None,
            ));
        }
    }

    Ok(findings)
}
