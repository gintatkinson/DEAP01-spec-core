//! Mermaid diagram syntax, fence closure, and angle bracket escaping verification.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives.
//! Verifies Mermaid diagram block integrity, diagram type headers, and unquoted angle bracket escaping.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Every opened Mermaid diagram block must be closed by a matching triple-backtick fence.
//! - First non-comment line must declare a valid diagram type.
//!
//! ## 3. Preconditions
//! - Caller provides valid UTF-8 markdown content.
//!
//! ## 4. Postconditions
//! - Returns list of diagnostic strings citing file and line coordinates for syntax violations.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) linear pass over markdown lines.
//! - Space Complexity: O(N) auxiliary diagnostic memory.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Reports unclosed fences and header mismatches fail-closed via diagnostics.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-MD-MODULE], [DO-330-TQL-4-MARKDOWN].

use super::mermaid_catalog::VALID_MERMAID_HEADERS;

/// Inspects a single Mermaid diagram statement line for unquoted `<` or `>` characters.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Prevents Mermaid parser failures.
///
/// ## 2. Invariants & Mathematical Properties
/// - Stereotype Exemption: Substrings matching `<<[A-Za-z0-9_-]+>>` are stripped prior to inspection.
/// - Longest-Match Arrow Exemption: All 17 Mermaid connector tokens are removed in decreasing length order.
///
/// ## 3. Preconditions
/// `line` is a single non-comment UTF-8 line from a Mermaid diagram body.
///
/// ## 4. Postconditions
/// Returns `Some('<')` or `Some('>')` if unquoted bracket remains; `None` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(L). Space Complexity: O(L).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Non-panicking total function.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MD-ANGLES]
pub fn validate_mermaid_angle_bracket_escaping(line: &str) -> Option<char> {
    let clean_line = strip_mermaid_stereotypes(line.trim());
    let stripped = strip_mermaid_arrows(&clean_line);

    let mut in_quotes = false;
    let mut escape = false;
    for ch in stripped.chars() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_quotes = !in_quotes;
        } else if !in_quotes && (ch == '<' || ch == '>') {
            return Some(ch);
        }
    }
    None
}

fn strip_mermaid_stereotypes(clean_line: &str) -> String {
    let mut stripped = String::with_capacity(clean_line.len());
    let mut idx = 0;
    while idx < clean_line.len() {
        if clean_line.get(idx..).is_some_and(|s| s.starts_with("<<")) {
            if let Some(close_pos) = clean_line.get(idx + 2..).and_then(|s| s.find(">>")) {
                let inner = clean_line.get(idx + 2..idx + 2 + close_pos).unwrap_or("");
                if !inner.is_empty()
                    && inner
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                {
                    idx += 2 + close_pos + 2;
                    continue;
                }
            }
        }
        let Some(ch) = clean_line.get(idx..).and_then(|s| s.chars().next()) else {
            break;
        };
        stripped.push(ch);
        idx += ch.len_utf8();
    }
    stripped
}

fn strip_mermaid_arrows(line: &str) -> String {
    let mut clean_line = line.to_string();
    let arrows = [
        "==>>", "-->>", "<<==", "<<--", "<-->", "<==>", "<-.-", "-.->", "==>", "-->", "->>", "<==",
        "<--", "<<-", "<->", "->", "<-",
    ];
    for arrow in arrows {
        clean_line = clean_line.replace(arrow, "");
    }
    clean_line
}

/// Validates all fenced ` ```mermaid ` diagrams in a markdown document.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Enforces Mermaid Syntax & Diagram Integrity Rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Fence Closure: Every opening fence has a matching closing fence.
/// - Header Validity: First non-comment line must match `VALID_MERMAID_HEADERS`.
///
/// ## 3. Preconditions
/// `content` is a valid UTF-8 markdown document slice; `file_rel_path` is a relative path.
///
/// ## 4. Postconditions
/// Returns empty vector on valid syntax; otherwise returns detected violations.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N). Space Complexity: O(N).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Non-panicking total function.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MD-MERMAID]
pub fn check_mermaid_syntax(content: &str, file_rel_path: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let lines: Vec<&str> = content.split_inclusive('\n').collect();
    let n = lines.len();
    let mut i = 0;

    while i < n {
        let Some(&line) = lines.get(i) else {
            break;
        };
        let trimmed = line.trim();
        let is_mermaid_open = if trimmed.starts_with("```") {
            let backticks = trimmed.chars().take_while(|&c| c == '`').count();
            backticks >= 3 && trimmed.get(backticks..).unwrap_or("").trim() == "mermaid"
        } else {
            false
        };

        if is_mermaid_open {
            let start_line = i + 1;
            let mut body: Vec<(usize, &str)> = Vec::new();
            i += 1;
            let mut closed = false;

            while i < n {
                let Some(&inner) = lines.get(i) else {
                    break;
                };
                let inner_trimmed = inner.trim();
                if inner_trimmed.starts_with("```") {
                    closed = true;
                    break;
                }
                body.push((i + 1, inner));
                i += 1;
            }

            if !closed {
                errors.push(format!(
                    "{}:{}: unclosed ```mermaid fence. Every diagram must be closed with a matching ``` fence on its own line.",
                    file_rel_path, start_line
                ));
            } else {
                validate_mermaid_block(&body, file_rel_path, start_line, &mut errors);
            }
        }
        i += 1;
    }

    errors
}

fn validate_mermaid_block(
    body: &[(usize, &str)],
    file_rel_path: &str,
    start_line: usize,
    errors: &mut Vec<String>,
) {
    let mut header_opt = None;
    for &(lineno, body_line) in body {
        let trimmed = body_line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("%%") {
            header_opt = Some((lineno, trimmed));
            break;
        }
    }

    let mut kind = String::new();
    if let Some((header_line, header_text)) = header_opt {
        let lower = header_text.to_lowercase();
        if let Some(matched_kind) = VALID_MERMAID_HEADERS.iter().find(|&h| lower.starts_with(h)) {
            kind = matched_kind.to_string();
        } else {
            errors.push(format!(
                "{}:{}: missing or invalid Mermaid diagram header (found '{}'). The first non-comment line must declare a valid diagram type header.",
                file_rel_path, header_line, header_text
            ));
        }
    } else {
        errors.push(format!(
            "{}:{}: missing or invalid Mermaid diagram header. The first non-comment line must declare a valid diagram type header.",
            file_rel_path, start_line
        ));
    }

    check_mermaid_notes_and_arrows(body, &kind, file_rel_path, errors);
}

fn check_mermaid_notes_and_arrows(
    body: &[(usize, &str)],
    kind: &str,
    file_rel_path: &str,
    errors: &mut Vec<String>,
) {
    let check_angles = matches!(
        kind,
        "graph" | "flowchart" | "sequencediagram" | "statediagram" | "statediagram-v2"
    );

    for &(lineno, body_line) in body {
        let trimmed = body_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }
        let is_note = trimmed.starts_with("Note ") || trimmed.starts_with("note ");
        let is_msg = trimmed.contains(">>")
            || trimmed.contains("->>")
            || trimmed.contains("-->>")
            || (kind == "sequencediagram"
                && trimmed.contains(':')
                && !trimmed.starts_with("participant ")
                && !trimmed.starts_with("actor "));
        if (is_note || is_msg) && trimmed.contains(';') {
            errors.push(format!(
                "{}:{}: semicolon in Mermaid Note or message text: '{}'. Replace with commas, dashes, or spaces.",
                file_rel_path, lineno, trimmed
            ));
        }

        if check_angles {
            if let Some(unquoted_char) = validate_mermaid_angle_bracket_escaping(trimmed) {
                errors.push(format!(
                    "{}:{}: unquoted '{}' character in Mermaid diagram line: '{}'. Transitions, labels, or guards containing comparison operators or brackets MUST be enclosed in double quotes.",
                    file_rel_path, lineno, unquoted_char, trimmed
                ));
            }
        }
    }
}
