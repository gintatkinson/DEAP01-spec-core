//! Mermaid syntax invariant validation for `spec_audit::diagram_ast`.
//! Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]

use crate::spec_audit::markdown_ast::MermaidBlock;
use crate::spec_audit::SpecAuditError;

/// Valid Mermaid relation and edge arrow tokens.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-ARROWS]
pub(crate) const ARROWS: &[&str] = &[
    "==>>", "-->>", "<<==", "<<--", "<-->", "<==>", "<|--", "--|>", "<|..", "..|>", "<|==", "==|>",
    "*--", "--*", "o--", "--o", "<-.-", "-.->", "==>", "-->", "->>", "<==", "<--", "<<-", "<->",
    "->", "<-", "..>", "<..", "..", "--",
];

/// Validates Mermaid block rendering invariants across all diagram types.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-VALIDATE-MERMAID-INVARIANTS]
pub fn validate_mermaid_invariants(block: &MermaidBlock) -> Result<(), Vec<SpecAuditError>> {
    let mut errors = Vec::new();
    if block.end_line <= block.start_line {
        errors.push(SpecAuditError::ValidationError(format!(
            "Unclosed or invalid Mermaid code fence starting at line {}.",
            block.start_line
        )));
    }

    let mut open_braces = 0usize;
    let mut close_braces = 0usize;
    let mut subgraphs = 0usize;
    let mut ends = 0usize;
    let is_class_diagram = block.header.to_lowercase().starts_with("classdiagram");
    let mut in_class_block = false;

    for (idx, line) in block.raw.lines().enumerate() {
        let line_num = block.start_line + idx;
        let clean = strip_mermaid_comment(line).trim();
        if clean.is_empty() {
            continue;
        }
        check_line_backticks_and_subgraphs(clean, line_num, &mut subgraphs, &mut ends, &mut errors);
        count_unquoted_braces(clean, &mut open_braces, &mut close_braces);

        if let Err(err) = check_unquoted_angle_brackets(clean, line_num) {
            errors.push(err);
        }
        if let Err(err) = check_node_label_quoting(clean, line_num) {
            errors.push(err);
        }
        if is_class_diagram {
            validate_class_diagram_line(clean, line_num, &mut in_class_block, &mut errors);
        }
    }

    if open_braces != close_braces {
        errors.push(SpecAuditError::ValidationError(format!(
            "Unbalanced curly braces in Mermaid diagram ({} opening '{{' vs {} closing '}}') between lines {}-{}.",
            open_braces, close_braces, block.start_line, block.end_line
        )));
    }
    if subgraphs != ends {
        errors.push(SpecAuditError::ValidationError(format!(
            "Unbalanced subgraph blocks in Mermaid diagram ({} 'subgraph' vs {} 'end') between lines {}-{}.",
            subgraphs, ends, block.start_line, block.end_line
        )));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Checks inline backticks and subgraph/end balancing on a single Mermaid line.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-VALIDATE-MERMAID-INVARIANTS]
fn check_line_backticks_and_subgraphs(
    clean: &str,
    line_num: usize,
    subgraphs: &mut usize,
    ends: &mut usize,
    errors: &mut Vec<SpecAuditError>,
) {
    if clean.starts_with("```") || clean.starts_with("~~~") {
        errors.push(SpecAuditError::ValidationError(format!(
            "Stray or nested markdown code fence within Mermaid diagram at line {}.",
            line_num
        )));
    } else if clean.contains('`') {
        errors.push(SpecAuditError::ValidationError(format!(
            "Inline backtick character ('`') is forbidden inside Mermaid diagram at line {} (triggers GitHub Mermaid <md_string> lexical error).",
            line_num
        )));
    }

    if let Some(after_subgraph_raw) = clean.strip_prefix("subgraph") {
        *subgraphs += 1;
        let title = after_subgraph_raw.trim();
        if !title.is_empty() {
            let is_quoted = title.starts_with('"') && title.ends_with('"');
            if !is_quoted && (title.contains(' ') || title.contains('-')) {
                errors.push(SpecAuditError::ValidationError(format!(
                    "Subgraph title '{}' at line {} contains spaces or hyphens and must be enclosed in double quotes.",
                    title, line_num
                )));
            }
        }
    } else if clean == "end" || clean.starts_with("end ") {
        *ends += 1;
    }
}

/// Counts opening and closing braces outside double quotes.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-VALIDATE-MERMAID-INVARIANTS]
fn count_unquoted_braces(clean: &str, open_braces: &mut usize, close_braces: &mut usize) {
    let mut in_quotes = false;
    let mut prev_char = ' ';
    for c in clean.chars() {
        if c == '"' && prev_char != '\\' {
            in_quotes = !in_quotes;
        } else if !in_quotes {
            if c == '{' {
                *open_braces += 1;
            } else if c == '}' {
                *close_braces += 1;
            }
        }
        prev_char = c;
    }
}

/// Validates class-body member syntax invariants for a single `classDiagram` line.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-VALIDATE-MERMAID-INVARIANTS]
fn validate_class_diagram_line(
    clean: &str,
    line_num: usize,
    in_class_block: &mut bool,
    errors: &mut Vec<SpecAuditError>,
) {
    if clean.contains('{') && clean.starts_with("class ") {
        *in_class_block = true;
        return;
    }
    if clean == "}" {
        *in_class_block = false;
        return;
    }
    if *in_class_block {
        if clean.starts_with("<<") && clean.ends_with(">>") {
            return;
        }
        if clean.contains('{') || clean.contains('}') {
            errors.push(SpecAuditError::ValidationError(format!(
                "Class member string '{}' at line {} contains prohibited curly braces '{{}}'.",
                clean, line_num
            )));
        }
        if clean.contains(':') {
            errors.push(SpecAuditError::ValidationError(format!(
                "Class member string '{}' at line {} contains prohibited colon ':'. Use standard spacing instead (e.g. '+ReturnType methodName(Type arg)').",
                clean, line_num
            )));
        }
    } else if let Some((left, right)) = clean.split_once(':') {
        let left_trimmed = left.trim();
        let right_trimmed = right.trim();
        if !contains_relation_arrow(left_trimmed) {
            if right_trimmed.contains('{') || right_trimmed.contains('}') {
                errors.push(SpecAuditError::ValidationError(format!(
                    "Class member string '{}' at line {} contains prohibited curly braces '{{}}'.",
                    right_trimmed, line_num
                )));
            }
            if right_trimmed.contains(':') {
                errors.push(SpecAuditError::ValidationError(format!(
                    "Class member string '{}' at line {} contains prohibited secondary colon ':'. Use standard spacing instead (e.g. '+ReturnType methodName(Type arg)').",
                    right_trimmed, line_num
                )));
            }
        }
    }
}

/// Strips trailing `%%` comments while preserving `%%` inside quoted strings.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-STRIP-MERMAID-COMMENT]
pub(crate) fn strip_mermaid_comment(line: &str) -> &str {
    let mut in_quotes = false;
    let mut prev = b' ';
    let bytes = line.as_bytes();
    let mut i = 0;
    while let Some(b) = bytes.get(i).copied() {
        if b == b'"' && prev != b'\\' {
            in_quotes = !in_quotes;
        } else if !in_quotes && b == b'%' && bytes.get(i + 1) == Some(&b'%') {
            return line.get(..i).unwrap_or(line);
        }
        prev = b;
        i += 1;
    }
    line
}

/// Checks whether a line contains any Mermaid relation or edge arrow token.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CONTAINS-RELATION-ARROW]
pub(crate) fn contains_relation_arrow(s: &str) -> bool {
    ARROWS.iter().any(|&arrow| s.contains(arrow))
}

/// Checks that all angle brackets `<` and `>` are inside quotes or valid Mermaid operators.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CHECK-UNQUOTED-ANGLE-BRACKETS]
fn check_unquoted_angle_brackets(line: &str, line_num: usize) -> Result<(), SpecAuditError> {
    let mut in_quotes = false;
    let mut prev_char = ' ';
    let bytes = line.as_bytes();
    let mut i = 0;

    while let Some(&b) = bytes.get(i) {
        if b == b'"' && prev_char != '\\' {
            in_quotes = !in_quotes;
            prev_char = b as char;
            i += 1;
            continue;
        }
        if in_quotes {
            prev_char = b as char;
            i += 1;
            continue;
        }

        let rest = line.get(i..).unwrap_or("");
        if let Some(&arrow) = ARROWS.iter().find(|&&a| rest.starts_with(a)) {
            i += arrow.len();
            prev_char = ' ';
            continue;
        }

        if rest.starts_with("<<") {
            if let Some(close_idx) = rest.find(">>") {
                let inner = rest.get(2..close_idx).unwrap_or("");
                if !inner.contains('"') {
                    i += close_idx + 2;
                    prev_char = ' ';
                    continue;
                }
            }
        }

        if b == b'<' || b == b'>' {
            return Err(SpecAuditError::ValidationError(format!(
                "Unquoted '{}' character at line {}: transitions, labels, or guards containing comparison operators or brackets MUST be enclosed in double quotes.",
                b as char, line_num
            )));
        }

        prev_char = b as char;
        i += 1;
    }

    Ok(())
}

/// Enforces double-quote wrapping on node labels containing special characters.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CHECK-NODE-LABEL-QUOTING]
fn check_node_label_quoting(line: &str, line_num: usize) -> Result<(), SpecAuditError> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("%%") {
        return Ok(());
    }

    let mut in_quotes = false;
    let mut prev = ' ';
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while let Some(&c) = chars.get(i) {
        if c == '"' && prev != '\\' {
            in_quotes = !in_quotes;
            prev = c;
            i += 1;
            continue;
        }
        if in_quotes {
            prev = c;
            i += 1;
            continue;
        }

        if c == '[' || c == '(' || c == '{' {
            if let Some(end_idx) = find_matching_close(&chars, i, c) {
                let inner_slice: String =
                    chars.get(i + 1..end_idx).unwrap_or_default().iter().collect();
                let cleaned = inner_slice
                    .trim()
                    .trim_matches(|ch: char| matches!(ch, '[' | ']' | '(' | ')' | '{' | '}' | '/'));
                let is_quoted = cleaned.starts_with('"') && cleaned.ends_with('"');
                if !is_quoted
                    && !cleaned.is_empty()
                    && cleaned.chars().any(|ch| matches!(ch, '/' | ':' | '(' | ')' | '[' | ']'))
                {
                    return Err(SpecAuditError::ValidationError(format!(
                        "Node label '{}' at line {} contains special characters ('/', ':', '()', '[]') and must be enclosed in double quotes.",
                        cleaned, line_num
                    )));
                }
                i = end_idx + 1;
                prev = ' ';
                continue;
            }
        }
        prev = c;
        i += 1;
    }

    Ok(())
}

/// Finds the matching closing bracket outside double quotes for `open_ch`.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-CHECK-NODE-LABEL-QUOTING]
fn find_matching_close(chars: &[char], start_idx: usize, open_ch: char) -> Option<usize> {
    let close_ch = match open_ch {
        '[' => ']',
        '(' => ')',
        '{' => '}',
        _ => ' ',
    };
    let mut depth = 1usize;
    let mut j = start_idx + 1;
    let mut inner_in_quotes = false;
    let mut inner_prev = ' ';

    while let Some(&jc) = chars.get(j) {
        if jc == '"' && inner_prev != '\\' {
            inner_in_quotes = !inner_in_quotes;
        } else if !inner_in_quotes {
            if jc == open_ch {
                depth += 1;
            } else if jc == close_ch {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
        }
        inner_prev = jc;
        j += 1;
    }
    None
}
