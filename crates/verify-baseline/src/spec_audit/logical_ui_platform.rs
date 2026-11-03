//! Platform independence and DOM/pixel/framework leak detection for Tier 1 specifications.
//!
//! Prohibits DOM attributes (`aria-*`, `role="..."`, `tabindex=`), HTML tags,
//! hardcoded pixel dimensions (`120px`), and concrete framework library references.

use crate::spec_audit::behavioral::contains_token_with_word_boundary;
use crate::spec_audit::markdown_ast::MarkdownDoc;
use crate::spec_audit::{SpecAuditError, SpecAuditFinding};

/// Forbidden concrete platform framework libraries that must not appear in Tier 1 specifications.
const FORBIDDEN_FRAMEWORKS: &[&str] = &[
    "flutter", "react-dom", "@angular/core", "vue", "angular", "electron", "gtk",
    "qt5", "qt6", "swiftui", "uikit", "wpf", "winforms",
];

/// Prohibited DOM-specific HTML tags outside code blocks.
const FORBIDDEN_DOM_TAGS: &[&str] = &[
    "<div", "<span", "<button", "<input", "<select", "<textarea", "<form", "<table",
];

/// Prohibited DOM event handlers outside code blocks.
const FORBIDDEN_DOM_EVENTS: &[&str] = &[
    "onclick=", "onchange=", "onsubmit=", "onkeydown=", "onkeyup=", "onkeypress=", "onload=", "onerror=",
];

/// Checks a single lowercased line for forbidden DOM attributes (`aria-*`, `role=`, `tabindex=`) and events.
/// Realises: [REQ-SPEC-AUDIT-LOGICAL-UI-SCAN-DOM-ATTRS]
fn scan_line_for_dom_attrs_and_events(
    lower: &str,
    rel_path: &str,
    line_no: usize,
    findings: &mut Vec<SpecAuditFinding>,
) {
    if let Some(pos) = lower.find("aria-") {
        let rem = lower.get(pos + 5..).unwrap_or("");
        if rem.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
            findings.push(SpecAuditFinding::new(
                "platform-independence-dom-attribute-leak",
                format!(
                    "Tier 1 specification '{}' contains forbidden DOM attribute 'aria-*' on line {}.",
                    rel_path, line_no
                ),
                rel_path,
                Some(line_no),
            ));
        }
    }

    if let Some(pos) = lower.find("role") {
        let rem = lower.get(pos + 4..).unwrap_or("").trim_start();
        if let Some(after_eq) = rem.strip_prefix('=') {
            let after_eq_trimmed = after_eq.trim_start();
            if after_eq_trimmed.starts_with('"') || after_eq_trimmed.starts_with('\'') {
                findings.push(SpecAuditFinding::new(
                    "platform-independence-dom-attribute-leak",
                    format!(
                        "Tier 1 specification '{}' contains forbidden DOM attribute 'role=\"...\"' on line {}.",
                        rel_path, line_no
                    ),
                    rel_path,
                    Some(line_no),
                ));
            }
        }
    }

    if let Some(pos) = lower.find("tabindex") {
        let rem = lower.get(pos + 8..).unwrap_or("").trim_start();
        if rem.starts_with('=') {
            findings.push(SpecAuditFinding::new(
                "platform-independence-dom-attribute-leak",
                format!(
                    "Tier 1 specification '{}' contains forbidden DOM attribute 'tabindex=' on line {}.",
                    rel_path, line_no
                ),
                rel_path,
                Some(line_no),
            ));
        }
    }

    for event in FORBIDDEN_DOM_EVENTS {
        if lower.contains(event) {
            findings.push(SpecAuditFinding::new(
                "platform-independence-dom-attribute-leak",
                format!(
                    "Tier 1 specification '{}' contains forbidden DOM event handler '{}' on line {}.",
                    rel_path, event, line_no
                ),
                rel_path,
                Some(line_no),
            ));
            break;
        }
    }
}

/// Checks a single lowercased line for forbidden HTML element tags (`<div`, `<span`, etc.).
/// Realises: [REQ-SPEC-AUDIT-LOGICAL-UI-SCAN-HTML-TAGS]
fn scan_line_for_forbidden_html_tags(
    lower: &str,
    rel_path: &str,
    line_no: usize,
    findings: &mut Vec<SpecAuditFinding>,
) {
    for tag in FORBIDDEN_DOM_TAGS {
        let mut search_from = 0;
        while let Some(slice) = lower.get(search_from..) {
            let Some(pos) = slice.find(tag) else {
                break;
            };
            let abs_pos = search_from + pos;
            let end_pos = abs_pos + tag.len();
            let is_tag_boundary = if end_pos >= lower.len() {
                true
            } else {
                lower.as_bytes().get(end_pos).is_some_and(|&next_b| {
                    next_b == b' ' || next_b == b'>' || next_b == b'/' || next_b == b'\t'
                })
            };

            if is_tag_boundary {
                findings.push(SpecAuditFinding::new(
                    "platform-independence-dom-attribute-leak",
                    format!(
                        "Tier 1 specification '{}' contains forbidden HTML element '{}' on line {}.",
                        rel_path, tag, line_no
                    ),
                    rel_path,
                    Some(line_no),
                ));
                break;
            }
            search_from = end_pos;
        }
    }
}

/// Scans text lines outside code blocks for prohibited DOM-specific attributes, event handlers, and web tags.
/// Realises: [REQ-SPEC-AUDIT-LOGICAL-UI-DETECT-DOM-LEAKS]
pub fn detect_dom_leaks(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let mut in_code_block = false;

    for (idx, line) in content.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        let lower = line.to_ascii_lowercase();
        scan_line_for_dom_attrs_and_events(&lower, rel_path, line_no, &mut findings);
        scan_line_for_forbidden_html_tags(&lower, rel_path, line_no, &mut findings);
    }

    findings
}

/// Identifies all hardcoded pixel dimensions (e.g. `120px`, `450px`, `1000px`) on a single text line.
/// Realises: [REQ-SPEC-AUDIT-LOGICAL-UI-FIND-PIXEL-DIMENSIONS-IN-LINE]
pub fn find_pixel_dimensions_in_line(line: &str) -> Vec<String> {
    let mut found = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut idx = 0;

    while idx + 1 < len {
        let is_p = bytes.get(idx).is_some_and(|&b| b == b'p' || b == b'P');
        let is_x = bytes.get(idx + 1).is_some_and(|&b| b == b'x' || b == b'X');
        if is_p && is_x {
            let px_start = idx;
            let px_end = idx + 2;
            let after_ok = if px_end < len {
                !bytes
                    .get(px_end)
                    .is_some_and(|&next_b| next_b.is_ascii_alphanumeric() || next_b == b'_')
            } else {
                true
            };

            if after_ok && px_start > 0 {
                let mut digit_start = px_start;
                let mut dot_seen = false;
                while digit_start > 0 {
                    let Some(&prev_b) = bytes.get(digit_start - 1) else {
                        break;
                    };
                    if prev_b.is_ascii_digit() {
                        digit_start -= 1;
                    } else if prev_b == b'.'
                        && !dot_seen
                        && digit_start - 1 > 0
                        && bytes.get(digit_start - 2).is_some_and(u8::is_ascii_digit)
                    {
                        dot_seen = true;
                        digit_start -= 1;
                    } else {
                        break;
                    }
                }

                if digit_start < px_start {
                    let before_ok = if digit_start > 0 {
                        !bytes
                            .get(digit_start - 1)
                            .is_some_and(|&b| b.is_ascii_alphabetic() || b == b'_')
                    } else {
                        true
                    };
                    if before_ok {
                        if let Some(token) = line.get(digit_start..px_end) {
                            found.push(token.to_string());
                        }
                    }
                }
            }
            idx = px_end;
        } else {
            idx += 1;
        }
    }

    found
}

/// Scans text lines outside code blocks for hardcoded pixel dimensions.
/// Realises: [REQ-SPEC-AUDIT-LOGICAL-UI-DETECT-PIXEL-DIMENSIONS]
pub fn detect_pixel_dimensions(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let mut in_code_block = false;

    for (idx, line) in content.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        for dim in find_pixel_dimensions_in_line(line) {
            findings.push(SpecAuditFinding::new(
                "platform-independence-hardcoded-pixel-dimension",
                format!(
                    "Tier 1 specification '{}' contains hardcoded pixel dimension '{}' on line {}.",
                    rel_path, dim, line_no
                ),
                rel_path,
                Some(line_no),
            ));
        }
    }

    findings
}

/// Scans a single non-code line for concrete framework names (`flutter`, `react`, etc.).
/// Realises: [REQ-SPEC-AUDIT-LOGICAL-UI-SCAN-FRAMEWORK-LINE]
fn scan_line_for_framework_leaks(
    trimmed: &str,
    rel_path: &str,
    line_no: usize,
    findings: &mut Vec<SpecAuditFinding>,
) {
    for framework in FORBIDDEN_FRAMEWORKS {
        if contains_token_with_word_boundary(trimmed, framework) {
            findings.push(SpecAuditFinding::new(
                "platform-independence-framework-leak",
                format!(
                    "Tier 1 specification '{}' references forbidden concrete framework library '{}' on line {}.",
                    rel_path, framework, line_no
                ),
                rel_path,
                Some(line_no),
            ));
        }
    }

    if contains_token_with_word_boundary(trimmed, "react") {
        let lower = trimmed.to_lowercase();
        let verb_phrases = [
            "react to",
            "react with",
            "react on",
            "reacts to",
            "reaction to",
        ];
        if !verb_phrases.iter().any(|phrase| lower.contains(phrase)) {
            findings.push(SpecAuditFinding::new(
                "platform-independence-framework-leak",
                format!(
                    "Tier 1 specification '{}' references forbidden concrete framework library 'react' on line {}.",
                    rel_path, line_no
                ),
                rel_path,
                Some(line_no),
            ));
        }
    }
}

/// Scans normative functional requirements outside code blocks for concrete platform framework libraries.
/// Realises: [REQ-SPEC-AUDIT-LOGICAL-UI-DETECT-FRAMEWORK-LEAKS]
pub fn detect_framework_leaks(
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();

    let platform_field = doc
        .table_metadata
        .get("platform")
        .map(|s| s.as_str())
        .or_else(|| {
            doc.frontmatter
                .as_ref()
                .and_then(|fm| fm.get("platform").map(|s| s.as_str()))
        });
    if let Some(platform_val) = platform_field {
        findings.push(SpecAuditFinding::new(
            "platform-independence-frontmatter-platform-field",
            format!(
                "Specification '{}' contains forbidden 'platform: {}' field in metadata. Tier 1 specs must be platform-agnostic.",
                rel_path, platform_val
            ),
            rel_path,
            None,
        ));
    }

    let mut in_code_block = false;
    for (idx, line) in content.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        scan_line_for_framework_leaks(trimmed, rel_path, line_no, &mut findings);
    }

    findings
}

/// Validates Tier 1 platform independence rules across DOM attributes, pixel dimensions, and framework leaks.
/// Realises: [REQ-SPEC-AUDIT-LOGICAL-UI-VALIDATE-PLATFORM-INDEPENDENCE]
pub fn validate_platform_independence(
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let mut findings = Vec::new();
    findings.extend(detect_dom_leaks(content, rel_path));
    findings.extend(detect_pixel_dimensions(content, rel_path));
    findings.extend(detect_framework_leaks(doc, content, rel_path));
    Ok(findings)
}
