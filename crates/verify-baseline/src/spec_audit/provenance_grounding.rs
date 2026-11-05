//! Factual grounding and naked numeric quantity scanner for `provenance.rs`.

use crate::spec_audit::markdown_ast::{parse_markdown, MarkdownDoc};
use crate::spec_audit::provenance::{
    find_engineering_parameter, is_recognized_unit, validate_sequence_diagram_temporal_safety,
    NORMATIVE_STANDARDS,
};
use crate::spec_audit::uml_validator::SysmlSymbolIndex;
use crate::spec_audit::{SpecAuditError, SpecAuditFinding};
use deap_core::sysml_ast::{AttributeDef, ConstraintDef};

/// Checks if a numeric span `[start..end]` in `line` is part of an identifier, label, or section number.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-FIND-NAKED-NUMERIC-QUANTITIES]
fn is_identifier_or_section_number(line: &str, start: usize, end: usize) -> bool {
    let bytes = line.as_bytes();
    let mut word_start = start;
    while word_start > 0 {
        let Some(&b) = bytes.get(word_start - 1) else {
            break;
        };
        if b.is_ascii_whitespace() || matches!(b, b'[' | b'(' | b'{' | b'<' | b'|') {
            break;
        }
        word_start -= 1;
    }
    let prefix = bytes.get(word_start..start).unwrap_or_default();
    let is_id_prefix = prefix
        .iter()
        .any(|b| b.is_ascii_alphabetic() || *b == b'_' || *b == b'#');

    let is_id_suffix = if let Some(&b) = bytes.get(end) {
        if b == b'_' || b == b'-' {
            true
        } else if b.is_ascii_alphabetic() {
            let mut unit_end = end;
            while bytes.get(unit_end).is_some_and(|&ub| {
                ub.is_ascii_alphanumeric() || ub == b'/' || ub == b'^' || ub == b'%'
            }) {
                unit_end += 1;
            }
            !is_recognized_unit(line.get(end..unit_end).unwrap_or(""))
        } else {
            false
        }
    } else {
        false
    };

    let is_label_word = if word_start > 0 {
        let before_text = line.get(..word_start).unwrap_or("").trim_end();
        let last_word = before_text
            .rsplit(|c: char| c.is_whitespace() || matches!(c, '|' | '*' | '`' | '#'))
            .next()
            .unwrap_or("")
            .trim()
            .to_lowercase();
        matches!(
            last_word.as_str(),
            "feature" | "feat" | "epic" | "req" | "ac" | "step" | "item" | "section" | "table"
                | "figure" | "level" | "mode" | "id" | "issue" | "case" | "story" | "check"
                | "bp" | "blueprint"
        )
    } else {
        false
    };

    if is_id_prefix || is_id_suffix || is_label_word {
        return true;
    }

    let is_outline_label = if bytes.get(end).is_some_and(|b| b.is_ascii_alphabetic()) {
        let mut letter_end = end;
        while bytes.get(letter_end).is_some_and(|b| b.is_ascii_alphabetic()) {
            letter_end += 1;
        }
        (letter_end - end <= 2) && bytes.get(letter_end).is_some_and(|&lb| lb == b'.' || lb == b')' || lb == b':')
    } else {
        false
    };

    bytes.get(end).is_some_and(|&eb| eb == b'.' || eb == b')' || eb == b':')
        || is_outline_label
        || (start > 0 && bytes.get(start - 1).is_some_and(|&pb| pb == b'.' || pb == b'v' || pb == b'V'))
}

/// Checks if a numeric span ending at `end` in `line` is followed by a recognized physical unit.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-FIND-NAKED-NUMERIC-QUANTITIES]
fn has_attached_or_trailing_unit(line: &str, end: usize) -> bool {
    let bytes = line.as_bytes();
    if bytes.get(end).is_some_and(|b| b.is_ascii_alphabetic()) {
        let mut unit_end = end;
        while bytes.get(unit_end).is_some_and(|&ub| {
            ub.is_ascii_alphanumeric() || ub == b'/' || ub == b'^' || ub == b'%'
        }) {
            unit_end += 1;
        }
        return is_recognized_unit(line.get(end..unit_end).unwrap_or(""));
    }
    if bytes.get(end) == Some(&b'%') {
        return true;
    }
    let rem = line.get(end..).unwrap_or("").trim_start();
    if !rem.is_empty() {
        let mut unit_end = 0;
        let rem_bytes = rem.as_bytes();
        while rem_bytes.get(unit_end).is_some_and(|&ub| {
            ub.is_ascii_alphanumeric() || ub == b'/' || ub == b'^' || ub == b'%'
        }) {
            unit_end += 1;
        }
        if unit_end > 0 && is_recognized_unit(rem.get(..unit_end).unwrap_or("")) {
            return true;
        }
    }
    false
}

/// Scans a line for naked numeric literals (digits without an immediate or adjacent unit).
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-FIND-NAKED-NUMERIC-QUANTITIES]
pub fn find_naked_numeric_quantities(line: &str) -> Vec<(String, usize)> {
    let mut naked = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut idx = 0;

    while idx < len {
        if bytes.get(idx).is_some_and(|b| b.is_ascii_digit()) {
            let start = idx;
            let mut dot_seen = false;
            while let Some(&b) = bytes.get(idx) {
                if b.is_ascii_digit() {
                    idx += 1;
                } else if b == b'.'
                    && !dot_seen
                    && bytes.get(idx + 1).is_some_and(|nb| nb.is_ascii_digit())
                {
                    dot_seen = true;
                    idx += 1;
                } else {
                    break;
                }
            }
            let end = idx;
            if start < end
                && !is_identifier_or_section_number(line, start, end)
                && !has_attached_or_trailing_unit(line, end)
            {
                let num_str = line.get(start..end).unwrap_or("");
                naked.push((num_str.to_string(), start));
            }
        } else {
            idx += 1;
        }
    }
    naked
}

/// Checks if an engineering parameter or assertion is grounded by SysML symbols or citations.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-ASSERTION-GROUNDED]
pub fn is_assertion_grounded(
    line: &str,
    content: &str,
    symbol_index: Option<&SysmlSymbolIndex>,
    constraints: &[ConstraintDef],
    attributes: &[AttributeDef],
) -> bool {
    let lower_line = line.to_ascii_lowercase();
    if NORMATIVE_STANDARDS.iter().any(|&std| lower_line.contains(std)) {
        return true;
    }
    if lower_line.contains("schema/")
        || lower_line.contains(".sysml")
        || lower_line.contains('§')
        || lower_line.contains("ssot")
        || lower_line.contains("grounding:")
        || lower_line.contains("source:")
        || lower_line.contains("ref:")
    {
        return true;
    }

    if let Some(idx) = symbol_index {
        let in_index = idx
            .attribute_names
            .iter()
            .chain(idx.part_names.iter())
            .chain(idx.action_names.iter())
            .chain(idx.port_names.iter())
            .any(|sym| lower_line.contains(&sym.to_ascii_lowercase()));
        if in_index {
            return true;
        }
    }

    if constraints
        .iter()
        .any(|c| !c.name.is_empty() && lower_line.contains(&c.name.to_ascii_lowercase()))
        || attributes
            .iter()
            .any(|a| !a.name.is_empty() && lower_line.contains(&a.name.to_ascii_lowercase()))
    {
        return true;
    }

    let lower_doc = content.to_ascii_lowercase();
    (lower_doc.contains("## source references") || lower_doc.contains("## 6. source references"))
        && (lower_doc.contains("schema/")
            || lower_doc.contains(".sysml")
            || NORMATIVE_STANDARDS.iter().any(|&std| lower_doc.contains(std)))
}

/// Checks whether a Markdown table row is a non-normative metadata or divider row.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-VALIDATE-FACTUAL-GROUNDING]
fn is_non_normative_table_row(trimmed: &str) -> bool {
    if !trimmed.starts_with('|') {
        return false;
    }
    if crate::spec_audit::markdown_ast::is_metadata_header_table_row(trimmed) {
        return true;
    }
    if trimmed
        .chars()
        .all(|c| c == '|' || c == '-' || c == ':' || c.is_ascii_whitespace())
    {
        return true;
    }
    let parts: Vec<&str> = trimmed
        .split('|')
        .map(|c| c.trim())
        .filter(|c| !c.is_empty())
        .collect();
    if let Some(&first_part) = parts.first().filter(|_| parts.len() == 2) {
        let k_norm = first_part.replace(['*', '_', '`'], "").trim().to_lowercase();
        return matches!(
            k_norm.as_str(),
            "title"
                | "package"
                | "subsystem"
                | "version"
                | "date"
                | "author"
                | "type"
                | "status"
                | "generation mode"
                | "generation_mode"
                | "generation-mode"
                | "subagent drafted"
                | "subagent_drafted"
                | "specification source"
                | "parent epic"
                | "interface type"
                | "schema containers"
                | "primary actor"
                | "scope"
                | "level"
                | "issue id"
                | "issue"
        );
    }
    false
}

/// Audits numeric parameters, units, and engineering assertions against SysML schema AST symbols.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-VALIDATE-FACTUAL-GROUNDING]
pub fn validate_factual_grounding(
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
    symbol_index: Option<&SysmlSymbolIndex>,
    schema_defs: (&[ConstraintDef], &[AttributeDef]),
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let (constraints, attributes) = schema_defs;
    let mut findings = Vec::new();
    let mut in_code_block = false;

    for (idx, line) in content.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block
            || trimmed.is_empty()
            || trimmed.starts_with('#')
            || is_non_normative_table_row(trimmed)
        {
            continue;
        }

        if let Some(param) = find_engineering_parameter(trimmed) {
            for (num_str, _) in find_naked_numeric_quantities(trimmed) {
                findings.push(SpecAuditFinding::new(
                    "factual-grounding-missing-unit",
                    format!(
                        "Engineering parameter '{}' on line {} in '{}' specifies numeric quantity '{}' without an explicit physical/engineering unit.",
                        param, line_no, rel_path, num_str
                    ),
                    rel_path,
                    Some(line_no),
                ));
            }
            if !is_assertion_grounded(trimmed, content, symbol_index, constraints, attributes) {
                findings.push(SpecAuditFinding::new(
                    "factual-grounding-ungrounded-assertion",
                    format!(
                        "Engineering assertion on line {} in '{}' mentioning '{}' lacks empirical grounding against SysML schema AST symbols or normative standards citations.",
                        line_no, rel_path, param
                    ),
                    rel_path,
                    Some(line_no),
                ));
            }
        }
    }

    findings.extend(validate_sequence_diagram_temporal_safety(
        doc, content, rel_path,
    ));
    Ok(findings)
}

/// Convenience entry point for validating factual grounding directly from raw Markdown content.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-VALIDATE-FACTUAL-GROUNDING-CONTENT]
pub fn validate_factual_grounding_content(
    content: &str,
    rel_path: &str,
    symbol_index: Option<&SysmlSymbolIndex>,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let doc = parse_markdown(content)?;
    validate_factual_grounding(&doc, content, rel_path, symbol_index, (&[], &[]))
}
