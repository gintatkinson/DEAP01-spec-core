//! Feature operation coverage and User Story test case binding validation gates.
//! Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR]

use std::collections::BTreeSet;

use crate::spec_audit::markdown_ast::MarkdownDoc;
use crate::spec_audit::uml_validator::{ClassifierSymbolInfo, SysmlSymbolIndex};

/// Extracts leading alphanumeric and underscore identifier characters.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-EXTRACT-SIMPLE-IDENT]
fn extract_simple_ident(s: &str) -> &str {
    let trimmed = s.trim();
    let end = trimmed.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(trimmed.len());
    &trimmed[..end]
}

/// Checks if `needle` appears in `haystack` as a distinct whole word.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-CONTAINS-WHOLE-WORD]
fn contains_whole_word(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(needle) {
        let abs_pos = start + pos;
        let end_pos = abs_pos + needle.len();
        let left_ok = abs_pos == 0
            || !haystack[..abs_pos]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_');
        let right_ok = end_pos >= haystack.len()
            || !haystack[end_pos..].chars().next().is_some_and(|c| c.is_alphanumeric() || c == '_');
        if left_ok && right_ok {
            return true;
        }
        start = abs_pos + 1;
    }
    false
}

/// Validates that all SysML actions and operations declared on bound parts are covered in the feature specification.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-VALIDATE-FEATURE-OPERATION-COVERAGE]
pub fn validate_feature_operation_coverage(
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
    index: &SysmlSymbolIndex,
) -> Vec<crate::spec_audit::SpecAuditFinding> {
    let mut findings = Vec::new();
    let filename = std::path::Path::new(rel_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| rel_path.to_string());

    let mut matched_classifiers: Vec<&ClassifierSymbolInfo> = Vec::new();
    let mut container_vals = Vec::new();
    if let Some(ref fm) = doc.frontmatter {
        if let Some(val) = fm.get("schema_containers") {
            container_vals.push(val.as_str());
        }
    }
    for (k, v) in &doc.table_metadata {
        if k.to_ascii_lowercase().replace(' ', "_") == "schema_containers" {
            container_vals.push(v.as_str());
        }
    }

    for c_val in container_vals {
        let cleaned = c_val.trim_matches(|c| matches!(c, '[' | ']' | '`'));
        for part_entry in cleaned.split(',') {
            let entry = part_entry.trim().trim_matches('"').trim_matches('\'').trim();
            if entry.is_empty() {
                continue;
            }
            let leaf = entry
                .rsplit('/').next().unwrap_or(entry)
                .rsplit("::").next().unwrap_or(entry)
                .rsplit(':').next().unwrap_or(entry)
                .trim()
                .trim_matches('`');
            let leaf_lower = leaf.to_ascii_lowercase();
            for (c_name, c_info) in &index.classifiers {
                if c_name.to_ascii_lowercase() == leaf_lower
                    && !matched_classifiers.iter().any(|existing| existing.name == c_info.name)
                {
                    matched_classifiers.push(c_info);
                }
            }
        }
    }

    if matched_classifiers.is_empty() {
        let explicit_declared_parts = collect_fallback_declared_parts(doc, content);
        for (c_name, c_info) in &index.classifiers {
            if explicit_declared_parts.contains(&c_name.to_ascii_lowercase())
                && !matched_classifiers.iter().any(|existing| existing.name == c_info.name)
            {
                matched_classifiers.push(c_info);
            }
        }
    }

    for classifier in matched_classifiers {
        let mut ops_to_check = BTreeSet::new();
        ops_to_check.extend(&classifier.operations);
        ops_to_check.extend(&classifier.actions);
        for op_name in ops_to_check {
            if !contains_whole_word(content, op_name) {
                findings.push(crate::spec_audit::SpecAuditFinding::new(
                    "feature-operation-missing",
                    format!("Feature '{}': SysML action/operation '{}' on part '{}' is not extracted or bound in feature specification.", filename, op_name, classifier.name),
                    rel_path,
                    None,
                ));
            }
        }
    }

    findings
}

/// Collects explicitly declared part names from Mermaid `classDiagram` blocks and `sysml` code blocks.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-VALIDATE-FEATURE-OPERATION-COVERAGE]
fn collect_fallback_declared_parts(doc: &MarkdownDoc, content: &str) -> BTreeSet<String> {
    let mut explicit_declared_parts = BTreeSet::new();
    for block in &doc.mermaid_blocks {
        if block.header == "classDiagram" {
            for line in block.raw.lines() {
                let mut line_str = line.trim();
                if let Some(idx) = line_str.find("%%") {
                    line_str = line_str[..idx].trim();
                }
                if line_str.is_empty() {
                    continue;
                }
                let candidate = if let Some(after_class) = line_str.strip_prefix("class ") {
                    Some(after_class.trim().trim_matches('{').trim())
                } else if line_str.contains(':') && !line_str.to_ascii_lowercase().starts_with("note") {
                    line_str.split_once(':').map(|(lhs, _)| lhs.trim())
                } else if line_str.contains('{') {
                    line_str.split_once('{').map(|(lhs, _)| lhs.trim())
                } else {
                    None
                };
                if let Some(ident) = candidate {
                    let ident_clean = extract_simple_ident(ident);
                    if !ident_clean.is_empty() {
                        explicit_declared_parts.insert(ident_clean.to_ascii_lowercase());
                    }
                }
            }
        }
    }

    let mut in_sysml_block = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            let tag = trimmed.trim_start_matches('`').trim();
            in_sysml_block = !in_sysml_block && (tag == "sysml" || tag == "sysmlv2");
            continue;
        }
        if in_sysml_block {
            let rest_opt = trimmed
                .find("part def ")
                .map(|pos| &trimmed[pos + 9..])
                .or_else(|| trimmed.find("part ").map(|pos| &trimmed[pos + 5..]));
            if let Some(rest) = rest_opt {
                let ident = extract_simple_ident(rest);
                if !ident.is_empty() {
                    explicit_declared_parts.insert(ident.to_ascii_lowercase());
                }
            }
        }
    }
    explicit_declared_parts
}

/// Validates that test cases bound in User Story specifications exist in the SysML AST.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-VALIDATE-USER-STORY-TEST-CASES]
pub fn validate_user_story_test_cases(
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
    index: &SysmlSymbolIndex,
) -> Vec<crate::spec_audit::SpecAuditFinding> {
    let mut findings = Vec::new();
    if index.test_cases.is_empty() {
        return findings;
    }

    let filename = std::path::Path::new(rel_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| rel_path.to_string());
    let tc_refs = collect_story_test_case_refs(doc, content);

    for tc_name in &tc_refs {
        match index.test_cases.get(tc_name) {
            None => {
                findings.push(crate::spec_audit::SpecAuditFinding::new(
                    "user-story-test-case-not-in-ast",
                    format!(
                        "User Story '{}': Bound test case '{}' is not defined in SysML AST.",
                        filename, tc_name
                    ),
                    rel_path,
                    None,
                ));
            }
            Some(verified_reqs) => {
                if verified_reqs.is_empty() {
                    findings.push(crate::spec_audit::SpecAuditFinding::new(
                        "test-case-missing-verify-requirement",
                        format!(
                            "User Story '{}': Test case def '{}' does not declare any 'verify requirement' bindings for safety requirements.",
                            filename, tc_name
                        ),
                        rel_path,
                        None,
                    ));
                } else {
                    for v_target in verified_reqs {
                        let is_valid = index.requirement_ids.contains(v_target)
                            || index.constraint_ids.contains(v_target);
                        if !is_valid {
                            findings.push(crate::spec_audit::SpecAuditFinding::new(
                                "test-case-verify-requirement-invalid",
                                format!(
                                    "User Story '{}': Test case def '{}' specifies verify requirement '{}' which is not defined in SysML AST.",
                                    filename, tc_name, v_target
                                ),
                                rel_path,
                                None,
                            ));
                        }
                    }
                }
            }
        }
    }

    findings
}

/// Extracts deduplicated test case references from frontmatter, table metadata, and `TC_` tokens.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-VALIDATE-USER-STORY-TEST-CASES]
fn collect_story_test_case_refs(doc: &MarkdownDoc, content: &str) -> Vec<String> {
    let mut tc_refs = Vec::new();
    if let Some(ref fm) = doc.frontmatter {
        for key in &["sysml_test_case", "test_case", "test_case_def", "test_cases"] {
            if let Some(val) = fm.get(*key) {
                let cleaned = val.trim_matches(|c| matches!(c, '[' | ']' | '`'));
                for t in cleaned.split(',') {
                    let trimmed = t.trim().trim_matches('"').trim_matches('\'').trim();
                    if !trimmed.is_empty() {
                        tc_refs.push(trimmed.to_string());
                    }
                }
            }
        }
    }
    for (k, v) in &doc.table_metadata {
        let norm_k = k.to_ascii_lowercase().replace(' ', "_");
        if matches!(norm_k.as_str(), "sysml_test_case" | "test_case" | "test_case_def") {
            let cleaned = v.trim_matches(|c| matches!(c, '[' | ']' | '`'));
            for t in cleaned.split(',') {
                let trimmed = t.trim().trim_matches('"').trim_matches('\'').trim();
                if !trimmed.is_empty() {
                    tc_refs.push(trimmed.to_string());
                }
            }
        }
    }
    for word in content.split(|c: char| !c.is_alphanumeric() && c != '_') {
        if word.starts_with("TC_") && word.len() > 3 {
            tc_refs.push(word.to_string());
        }
    }
    tc_refs.retain(|t| {
        !matches!(t.as_str(), "TestCase" | "test_case" | "test_case_def" | "Name" | "TC_")
    });
    tc_refs.sort();
    tc_refs.dedup();
    tc_refs
}
