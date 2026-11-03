//! Symbol normalization, word-boundary matching, and rule evaluation for behavioral triggers.
//!
//! Provides zero-regex helpers used by `behavioral.rs` and `logical_ui.rs`.

use crate::spec_audit::behavioral::{AuditedSpec, BehavioralRule};
use crate::spec_audit::uml_validator::SysmlSymbolIndex;
use std::collections::BTreeSet;

/// Normalizes a symbol or schema node name into canonical lowercased comparison form.
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-NORMALIZE-SYMBOL-NAME]
pub fn normalize_symbol_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch != '-' && ch != '_' && !ch.is_whitespace() {
            out.extend(ch.to_lowercase());
        }
    }
    out
}

/// Collects all declared symbol identifiers across a `SysmlSymbolIndex` into a normalized lookup set.
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-COLLECT-NORMALIZED-SYMBOLS]
pub fn collect_normalized_symbols(index: &SysmlSymbolIndex) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for name in &index.part_names {
        set.insert(normalize_symbol_name(name));
    }
    for name in &index.port_names {
        set.insert(normalize_symbol_name(name));
    }
    for name in &index.action_names {
        set.insert(normalize_symbol_name(name));
    }
    for name in &index.state_names {
        set.insert(normalize_symbol_name(name));
    }
    for name in &index.attribute_names {
        set.insert(normalize_symbol_name(name));
    }
    for name in index.classifiers.keys() {
        set.insert(normalize_symbol_name(name));
    }
    set
}

/// Extracts all lines of a Markdown document that reside outside fenced code blocks.
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-EXTRACT-BODY-OUTSIDE-CODE-BLOCKS]
pub fn extract_body_outside_code_blocks(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }
        if !in_code_block {
            out.push_str(line);
            out.push('\n');
        }
    }

    out
}

/// Checks whether `content` contains `target` as a distinct token with word boundaries.
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-CONTAINS-TOKEN-WITH-WORD-BOUNDARY]
pub fn contains_token_with_word_boundary(content: &str, target: &str) -> bool {
    if target.is_empty() || content.is_empty() {
        return false;
    }
    let lower_content = content.to_ascii_lowercase();
    let lower_target = target.to_ascii_lowercase();

    let mut start = 0;
    while let Some(slice) = lower_content.get(start..) {
        let Some(pos) = slice.find(&lower_target) else {
            break;
        };
        let abs_pos = start + pos;
        let end_pos = abs_pos + lower_target.len();

        let before_ok = if abs_pos == 0 {
            true
        } else {
            !lower_content
                .as_bytes()
                .get(abs_pos - 1)
                .is_some_and(|&prev_byte| {
                    prev_byte.is_ascii_alphanumeric() || prev_byte == b'_' || prev_byte == b'-'
                })
        };

        let after_ok = if end_pos >= lower_content.len() {
            true
        } else {
            !lower_content
                .as_bytes()
                .get(end_pos)
                .is_some_and(|&next_byte| {
                    next_byte.is_ascii_alphanumeric() || next_byte == b'_' || next_byte == b'-'
                })
        };

        if before_ok && after_ok {
            return true;
        }

        let step = lower_content
            .get(abs_pos..)
            .and_then(|s| s.chars().next())
            .map(|c| c.len_utf8())
            .unwrap_or(1);
        start = abs_pos + step;
        if start >= lower_content.len() {
            break;
        }
    }

    false
}

/// Checks whether a specification references a trigger node by name.
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-SPEC-REFERENCES-NODE]
pub fn spec_references_node(content: &str, node: &str) -> bool {
    if node.is_empty() || content.is_empty() {
        return false;
    }
    if contains_token_with_word_boundary(content, node) {
        return true;
    }
    if node.contains('-') {
        let with_underscores = node.replace('-', "_");
        if contains_token_with_word_boundary(content, &with_underscores) {
            return true;
        }
    }
    if node.contains('-') || node.contains('_') {
        let with_spaces = node.replace(['-', '_'], " ");
        if contains_token_with_word_boundary(content, &with_spaces) {
            return true;
        }
    }
    let collapsed: String = node.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    if !collapsed.is_empty()
        && collapsed != node
        && contains_token_with_word_boundary(content, &collapsed)
    {
        return true;
    }
    false
}

/// Checks whether a specification document matches the target specification type.
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-MATCHES-TARGET-TYPE]
pub fn matches_target_type(spec_type: &str, rel_path: &str, target_type: &str) -> bool {
    let norm_target = target_type.to_lowercase().replace('_', "-");
    let norm_spec = spec_type.to_lowercase().replace('_', "-");
    let norm_path = rel_path.to_lowercase().replace('_', "-");

    if norm_target == "user-story" {
        norm_spec == "user-story"
            || norm_spec == "user-stories"
            || norm_spec == "story"
            || norm_path.contains("/user-stories/")
            || norm_path.contains("/user-story-")
            || norm_path.contains("/us-")
    } else if norm_target == "use-case" {
        norm_spec == "use-case"
            || norm_spec == "use-cases"
            || norm_path.contains("/use-cases/")
            || norm_path.contains("/use-case-")
            || norm_path.contains("/uc-")
    } else {
        norm_spec == norm_target || norm_path.contains(&norm_target)
    }
}

/// Checks if Markdown body text contains at least one of the specified match terms.
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-BODY-CONTAINS-ANY-TERM]
pub(crate) fn body_contains_any_term(body: &str, terms: &[String]) -> bool {
    if terms.is_empty() {
        return true;
    }
    let lower_body = body.to_lowercase();
    for term in terms {
        let lower_term = term.to_lowercase();
        if lower_body.contains(&lower_term) {
            return true;
        }
        if term.contains('-')
            && (lower_body.contains(&lower_term.replace('-', " "))
                || lower_body.contains(&lower_term.replace('-', "_")))
        {
            return true;
        }
        if term.contains('_')
            && (lower_body.contains(&lower_term.replace('_', " "))
                || lower_body.contains(&lower_term.replace('_', "-")))
        {
            return true;
        }
    }
    false
}

/// Evaluates whether a single `AuditedSpec` satisfies a `BehavioralRule`.
/// Realises: [REQ-SPEC-AUDIT-BEHAVIORAL-SPEC-SATISFIES-RULE]
pub(crate) fn spec_satisfies_behavioral_rule(spec: &AuditedSpec, rule: &BehavioralRule) -> bool {
    let mut file_valid = true;

    if let Some(ref req_mermaid) = rule.requires_mermaid_block {
        let matching_blocks: Vec<_> = spec
            .doc
            .mermaid_blocks
            .iter()
            .filter(|b| {
                b.header.eq_ignore_ascii_case(req_mermaid)
                    || b.raw
                        .lines()
                        .any(|l| l.trim().starts_with(req_mermaid.as_str()))
            })
            .collect();

        if matching_blocks.is_empty() {
            file_valid = false;
        } else if !rule.match_terms_in_mermaid.is_empty() {
            let term_found = matching_blocks.iter().any(|block| {
                rule.match_terms_in_mermaid.iter().any(|term| {
                    block.raw.contains(term.as_str())
                        || block.raw.to_lowercase().contains(&term.to_lowercase())
                })
            });
            if !term_found {
                file_valid = false;
            }
        }
    }

    if file_valid && !rule.match_terms_in_body.is_empty() {
        let body_text = extract_body_outside_code_blocks(&spec.content);
        if !body_contains_any_term(&body_text, &rule.match_terms_in_body) {
            file_valid = false;
        }
    }

    if file_valid && !rule.match_terms_in_body_secondary.is_empty() {
        let body_text = extract_body_outside_code_blocks(&spec.content);
        if !body_contains_any_term(&body_text, &rule.match_terms_in_body_secondary) {
            file_valid = false;
        }
    }

    file_valid
}
