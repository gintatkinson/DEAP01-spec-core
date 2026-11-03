//! Use Case anti-boilerplate validation helpers for specification auditing.
//!
//! Enforces minimum scenario step counts, alternate/exception flow counts,
//! and rejects canned template fallback strings.

use crate::spec_audit::markdown_ast::{HeadingNode, MarkdownDoc};
use crate::spec_audit::sections::normalize_tokens;

/// Finds the Main Success Scenario heading node in a parsed Markdown document.
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-FIND-MAIN-SCENARIO]
fn find_main_scenario_heading(doc: &MarkdownDoc) -> Option<&HeadingNode> {
    doc.headings.iter().find(|h| {
        let h_tokens = normalize_tokens(&h.text);
        (h_tokens.contains(&"main".to_string()) && h_tokens.contains(&"scenario".to_string()))
            || h.text.to_lowercase().contains("main success scenario")
    })
}

/// Finds the Alternate and Exception Flows heading node in a parsed Markdown document.
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-FIND-ALT-FLOWS]
fn find_alt_flows_heading(doc: &MarkdownDoc) -> Option<&HeadingNode> {
    doc.headings.iter().find(|h| {
        let h_tokens = normalize_tokens(&h.text);
        (h_tokens.contains(&"alternate".to_string()) && h_tokens.contains(&"flows".to_string()))
            || (h_tokens.contains(&"exception".to_string())
                && h_tokens.contains(&"flows".to_string()))
            || h.text
                .to_lowercase()
                .contains("alternate and exception flows")
    })
}

/// Computes the 1-indexed line number of the next peer or higher-level heading.
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-NEXT-HEADING-LINE]
fn next_peer_heading_line(doc: &MarkdownDoc, h: &HeadingNode) -> usize {
    doc.headings
        .iter()
        .filter(|next_h| next_h.line > h.line && next_h.level <= h.level)
        .map(|next_h| next_h.line)
        .min()
        .unwrap_or(usize::MAX)
}

/// Counts numbered scenario steps between `start_line` and `end_line`.
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-COUNT-MAIN-STEPS]
fn count_numbered_steps(content: &str, start_line: usize, end_line: usize) -> usize {
    let mut step_count = 0;
    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        if line_num > start_line && line_num < end_line {
            let trimmed = line.trim();
            let digit_count = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
            if digit_count > 0 {
                let after_digits = &trimmed[digit_count..];
                if (after_digits.starts_with('.') || after_digits.starts_with(')'))
                    && after_digits[1..].starts_with(' ')
                {
                    step_count += 1;
                }
            }
        }
    }
    step_count
}

/// Counts alphanumeric exception flow items (e.g. `5a.`, `5b:`) between `start_line` and `end_line`.
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-COUNT-EXCEPTION-CASES]
fn count_exception_cases(content: &str, start_line: usize, end_line: usize) -> usize {
    let mut exception_count = 0;
    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        if line_num > start_line && line_num < end_line {
            let mut trimmed = line.trim();
            if trimmed.starts_with('-') || trimmed.starts_with('*') {
                trimmed = trimmed[1..].trim();
            }
            let digit_count = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
            if digit_count > 0 {
                let after_digits = &trimmed[digit_count..];
                let letter_count = after_digits
                    .chars()
                    .take_while(|c| c.is_ascii_alphabetic())
                    .count();
                if letter_count > 0 {
                    let after_letters = &after_digits[letter_count..];
                    if after_letters.starts_with('.')
                        || after_letters.starts_with(':')
                        || after_letters.starts_with(' ')
                    {
                        exception_count += 1;
                    }
                }
            }
        }
    }
    exception_count
}

/// Validates Use Case content against canned fallback boilerplate, minimum scenario step count,
/// and minimum alternate/exception flow count.
/// Realises: [REQ-SPEC-AUDIT-SECTIONS-VALIDATE-USE-CASE-ANTI-BOILERPLATE]
pub fn validate_use_case_anti_boilerplate(
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
) -> Vec<String> {
    let mut errors = Vec::new();
    let lower_content = content.to_lowercase();

    const CANNED_STRINGS: &[&str] = &[
        "initiates workflow execution",
        "processes structural elements and validates semantic constraints",
    ];

    for canned in CANNED_STRINGS {
        if lower_content.contains(canned) {
            errors.push(format!(
                "{}: Anti-Boilerplate Gate violation: Use Case contains canned fallback string '{}'. Use Cases must specify genuine domain-specific operational workflows.",
                rel_path, canned
            ));
        }
    }

    if let Some(h) = find_main_scenario_heading(doc) {
        let next_line = next_peer_heading_line(doc, h);
        let step_count = count_numbered_steps(content, h.line, next_line);
        if step_count < 5 {
            errors.push(format!(
                "{}: Anti-Boilerplate Gate violation: Main Success Scenario has fewer than 5 steps (found {} step(s), minimum 5 required).",
                rel_path, step_count
            ));
        }
    }

    if let Some(h) = find_alt_flows_heading(doc) {
        let next_line = next_peer_heading_line(doc, h);
        let exception_count = count_exception_cases(content, h.line, next_line);
        if exception_count < 3 {
            errors.push(format!(
                "{}: Anti-Boilerplate Gate violation: Alternate and Exception Flows has fewer than 3 exception cases (found {} case(s), minimum 3 required, e.g. 5a, 5b, 5c).",
                rel_path, exception_count
            ));
        }
    }

    errors
}
