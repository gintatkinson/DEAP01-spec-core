//! Native Rust LUMI 3-Layer Quality and Platform Independence Validator.
//!
//! Conforms to DO-178C Level A and ISO 26262 ASIL D safety standards.
//!
//! Validates:
//! 1. LUMI (Logical User & Machine Interface) 3-Layer Semantic Chain:
//!    - Layer 1: Domain State & Signal Model
//!    - Layer 2: Logic & Safety State Management
//!    - Layer 3: Presentation & Actuator Interface Binding
//! 2. CSS Container Queries & Layout check:
//!    - UI specifications specifying responsive split panes or resizable containers must specify `@container` isolation rules.
//!    - Prohibition of raw "N/A" fallback strings and literal placeholder tokens in interface bindings.
//! 3. Platform Independence & DOM Leak Detection:
//!    - Prohibits DOM-specific attributes (`aria-*`, `role="..."`, `tabindex=`).
//!    - Prohibits hardcoded pixel dimensions (e.g. `120px`, `450px`, `1000px`).
//!    - Prohibits concrete platform framework libraries (`flutter`, `react`, `react-dom`, `vue`, `angular`, `@angular/core`, `electron`, `gtk`, `qt5`) in normative functional requirements.
//!
//! Strictly zero regex, zero unwrap/expect/panic in non-test code, zero Unicode em dashes (uses ASCII -- exclusively).

use crate::spec_audit::behavioral::contains_token_with_word_boundary;
use crate::spec_audit::markdown_ast::{parse_markdown, MarkdownDoc};
use crate::spec_audit::{SpecAuditError, SpecAuditFinding};

/// Forbidden concrete platform framework libraries that must not appear in Tier 1 specifications.
const FORBIDDEN_FRAMEWORKS: &[&str] = &[
    "flutter",
    "react-dom",
    "@angular/core",
    "vue",
    "angular",
    "electron",
    "gtk",
    "qt5",
    "qt6",
    "swiftui",
    "uikit",
    "wpf",
    "winforms",
];

/// Prohibited DOM-specific HTML tags outside code blocks.
const FORBIDDEN_DOM_TAGS: &[&str] = &[
    "<div",
    "<span",
    "<button",
    "<input",
    "<select",
    "<textarea",
    "<form",
    "<table",
];

/// Prohibited DOM event handlers outside code blocks.
const FORBIDDEN_DOM_EVENTS: &[&str] = &[
    "onclick=",
    "onchange=",
    "onsubmit=",
    "onkeydown=",
    "onkeyup=",
    "onkeypress=",
    "onload=",
    "onerror=",
];

/// Checks whether a specification document represents a LUMI (Logical User & Machine Interface) specification.
///
/// Inspects metadata tables, YAML frontmatter, headings, and section declarations.
///
/// Preconditions: None.
/// Postconditions: Returns true if the specification declares a UI/LUMI interface channel.
/// Algorithmic Complexity: O(H + M) where H is headings count and M is metadata entries count.
pub fn is_lumi_spec(doc: &MarkdownDoc, content: &str) -> bool {
    // 1. Check table metadata interface_type or channel
    if let Some(iface) = doc.table_metadata.get("interface_type") {
        let lower = iface.to_lowercase();
        if lower.contains("ui") || lower.contains("gui") || lower.contains("lumi") || lower.contains("user") {
            return true;
        }
    }

    // 2. Check frontmatter interface_type or interface_types
    if let Some(ref fm) = doc.frontmatter {
        if let Some(iface) = fm.get("interface_type") {
            let lower = iface.to_lowercase();
            if lower.contains("ui") || lower.contains("gui") || lower.contains("lumi") || lower.contains("user") {
                return true;
            }
        }
        if let Some(ifaces) = fm.get("interface_types") {
            let lower = ifaces.to_lowercase();
            if lower.contains("ui") || lower.contains("gui") || lower.contains("lumi") || lower.contains("user") {
                return true;
            }
        }
    }

    // 3. Check headings for LUMI indicators
    for heading in &doc.headings {
        let lower = heading.text.to_lowercase();
        if lower.contains("logical ui")
            || lower.contains("ui & layout")
            || lower.contains("ui and layout")
            || lower.contains("visual layout")
            || lower.contains("presentation & actuator")
            || lower.contains("presentation and actuator")
        {
            return true;
        }
    }

    // 4. Fallback search for canonical LUMI sections in content
    let lower_content = content.to_lowercase();
    lower_content.contains("## logical ui & layout bindings")
        || lower_content.contains("## 3. visual layout & arrangement")
        || lower_content.contains("logical ui & interface bindings")
}

/// Validates that a LUMI specification defines the mandatory 3-Layer Semantic Chain.
///
/// Verified layers:
/// - Layer 1: Domain State & Signal Model
/// - Layer 2: Logic & Safety State Management
/// - Layer 3: Presentation & Actuator Interface Binding
///
/// Preconditions: None.
/// Postconditions: Returns findings if any mandatory layer is missing from a LUMI specification.
/// Algorithmic Complexity: O(H + P) where H is headings and P is prose segments.
pub fn validate_3layer_chain(doc: &MarkdownDoc, content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();

    if !is_lumi_spec(doc, content) {
        // Non-LUMI specifications (e.g. backend services, pure computational pipelines) are exempt
        return findings;
    }

    // Build aggregate searchable text from headings, table metadata, and prose
    let mut full_text = String::with_capacity(content.len());
    for h in &doc.headings {
        full_text.push_str(&h.text);
        full_text.push(' ');
    }
    for (k, v) in &doc.table_metadata {
        full_text.push_str(k);
        full_text.push(' ');
        full_text.push_str(v);
        full_text.push(' ');
    }
    for p in &doc.prose {
        full_text.push_str(&p.text);
        full_text.push(' ');
    }
    let lower_text = full_text.to_lowercase();

    // Check Layer 1: Domain State & Signal Model
    let has_layer_1 = lower_text.contains("layer 1")
        || lower_text.contains("domain state & signal model")
        || lower_text.contains("domain state and signal model")
        || (lower_text.contains("domain state") && lower_text.contains("signal model"))
        || lower_text.contains("domain model");

    // Check Layer 2: Logic & Safety State Management
    let has_layer_2 = lower_text.contains("layer 2")
        || lower_text.contains("logic & safety state management")
        || lower_text.contains("logic and safety state management")
        || lower_text.contains("safety state")
        || lower_text.contains("state management")
        || lower_text.contains("viewmodel")
        || lower_text.contains("logic & state");

    // Check Layer 3: Presentation & Actuator Interface Binding
    let has_layer_3 = lower_text.contains("layer 3")
        || lower_text.contains("presentation & actuator interface binding")
        || lower_text.contains("presentation and actuator interface binding")
        || lower_text.contains("logical ui & layout bindings")
        || lower_text.contains("presentation & actuator")
        || lower_text.contains("actuator interface")
        || lower_text.contains("visual layout & arrangement")
        || lower_text.contains("lui widget binding")
        || lower_text.contains("presentation binding");

    if !has_layer_1 {
        findings.push(SpecAuditFinding::new(
            "lumi-3-layer-chain-incomplete",
            format!(
                "LUMI specification '{}' does not define Layer 1: Domain State & Signal Model.",
                rel_path
            ),
            rel_path,
            None,
        ));
    }

    if !has_layer_2 {
        findings.push(SpecAuditFinding::new(
            "lumi-3-layer-chain-incomplete",
            format!(
                "LUMI specification '{}' does not define Layer 2: Logic & Safety State Management.",
                rel_path
            ),
            rel_path,
            None,
        ));
    }

    if !has_layer_3 {
        findings.push(SpecAuditFinding::new(
            "lumi-3-layer-chain-incomplete",
            format!(
                "LUMI specification '{}' does not define Layer 3: Presentation & Actuator Interface Binding.",
                rel_path
            ),
            rel_path,
            None,
        ));
    }

    findings
}

/// Validates that UI specs describing responsive split panes or resizable containers declare `@container` isolation rules.
///
/// Preconditions: None.
/// Postconditions: Returns findings if split panes or resizable layouts are specified without `@container` rules.
/// Algorithmic Complexity: O(C) where C is content byte length.
pub fn validate_container_queries(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let lower = content.to_lowercase();

    let has_split_or_resizable = lower.contains("split pane")
        || lower.contains("split-pane")
        || lower.contains("splitter")
        || lower.contains("resizable split")
        || lower.contains("responsive split")
        || lower.contains("split container")
        || lower.contains("resizable container")
        || lower.contains("split workspace");

    if has_split_or_resizable && !content.contains("@container") {
        findings.push(SpecAuditFinding::new(
            "lumi-missing-container-query",
            format!(
                "UI specification '{}' defines responsive split panes or resizable containers but fails to specify '@container' isolation rules.",
                rel_path
            ),
            rel_path,
            None,
        ));
    }

    findings
}

/// Validates that layout interface bindings do not contain raw "N/A" fallback strings or literal template placeholders.
///
/// Preconditions: None.
/// Postconditions: Returns findings for raw N/A fallbacks or unresolved template placeholders.
/// Algorithmic Complexity: O(L) where L is lines count.
pub fn validate_layout_guidelines(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
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

        // Check for raw N/A fallback in table rows
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            let cells = trimmed.split('|');
            for cell in cells {
                let cell_clean = cell.trim().trim_matches(|c| c == '*' || c == '`' || c == '_');
                if cell_clean.eq_ignore_ascii_case("n/a") {
                    findings.push(SpecAuditFinding::new(
                        "lumi-prohibit-raw-na-fallback",
                        format!(
                            "Specification '{}' contains prohibited raw 'N/A' fallback string in interface bindings on line {}.",
                            rel_path, line_no
                        ),
                        rel_path,
                        Some(line_no),
                    ));
                    break;
                }
            }
        }

        // Check for literal placeholder tokens in interface bindings
        if contains_token_with_word_boundary(trimmed, "#X")
            || contains_token_with_word_boundary(trimmed, "Task Y")
        {
            findings.push(SpecAuditFinding::new(
                "lumi-prohibit-placeholder-string",
                format!(
                    "Specification '{}' contains literal placeholder token ('#X' or 'Task Y') on line {}.",
                    rel_path, line_no
                ),
                rel_path,
                Some(line_no),
            ));
        }
    }

    findings
}

/// Scans text lines outside code blocks for prohibited DOM-specific attributes, event handlers, and web tags.
///
/// Preconditions: None.
/// Postconditions: Returns findings for any detected DOM leaks.
/// Algorithmic Complexity: O(L * K) where L is lines and K is line length.
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

        let lower = line.to_lowercase();

        // 1. Check aria-* attributes
        if let Some(pos) = lower.find("aria-") {
            let rem = &lower[pos + 5..];
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

        // 2. Check role="..." attributes
        if let Some(pos) = lower.find("role") {
            let rem = lower[pos + 4..].trim_start();
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

        // 3. Check tabindex= attributes
        if let Some(pos) = lower.find("tabindex") {
            let rem = lower[pos + 8..].trim_start();
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

        // 4. Check forbidden DOM events
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

        // 5. Check forbidden HTML tags
        for tag in FORBIDDEN_DOM_TAGS {
            let mut search_from = 0;
            while let Some(pos) = lower[search_from..].find(tag) {
                let abs_pos = search_from + pos;
                let end_pos = abs_pos + tag.len();
                let is_tag_boundary = if end_pos >= lower.len() {
                    true
                } else {
                    let next_b = lower.as_bytes()[end_pos];
                    next_b == b' ' || next_b == b'>' || next_b == b'/' || next_b == b'\t'
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

    findings
}

/// Identifies all hardcoded pixel dimensions (e.g. `120px`, `450px`, `1000px`) on a single text line.
///
/// Strictly zero regex: iterates over byte positions looking for `px` unit indicators preceded by digits.
///
/// Preconditions: None.
/// Postconditions: Returns list of pixel dimension strings found on the line.
/// Algorithmic Complexity: O(K) where K is line length.
pub fn find_pixel_dimensions_in_line(line: &str) -> Vec<String> {
    let mut found = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut idx = 0;

    while idx + 1 < len {
        // Look for 'p' or 'P' followed by 'x' or 'X'
        if (bytes[idx] == b'p' || bytes[idx] == b'P') && (bytes[idx + 1] == b'x' || bytes[idx + 1] == b'X') {
            let px_start = idx;
            let px_end = idx + 2;

            // Boundary check after 'px': must not be an alphanumeric char or '_'
            let after_ok = if px_end < len {
                let next_b = bytes[px_end];
                !next_b.is_ascii_alphanumeric() && next_b != b'_'
            } else {
                true
            };

            if after_ok && px_start > 0 {
                // Scan backward to collect preceding digits and optional single decimal point
                let mut digit_start = px_start;
                let mut dot_seen = false;

                while digit_start > 0 {
                    let prev_b = bytes[digit_start - 1];
                    if prev_b.is_ascii_digit() {
                        digit_start -= 1;
                    } else if prev_b == b'.' && !dot_seen && digit_start - 1 > 0 && bytes[digit_start - 2].is_ascii_digit() {
                        dot_seen = true;
                        digit_start -= 1;
                    } else {
                        break;
                    }
                }

                if digit_start < px_start {
                    // Preceding boundary check: character before digits must not be an ASCII letter or '_'
                    let before_ok = if digit_start > 0 {
                        let b = bytes[digit_start - 1];
                        !b.is_ascii_alphabetic() && b != b'_'
                    } else {
                        true
                    };

                    if before_ok {
                        let token = &line[digit_start..px_end];
                        found.push(token.to_string());
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
///
/// Preconditions: None.
/// Postconditions: Returns findings for any hardcoded pixel dimensions detected.
/// Algorithmic Complexity: O(L * K) where L is lines and K is line length.
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

        let pixel_dims = find_pixel_dimensions_in_line(line);
        for dim in pixel_dims {
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

/// Scans normative functional requirements outside code blocks for concrete platform framework libraries.
///
/// Inspects against forbidden libraries: Flutter, React, Vue, Angular, Electron, GTK, QT5, etc.
/// Distinguishes between framework references ("React component", "using React") and the English verb "react to / react with".
///
/// Preconditions: None.
/// Postconditions: Returns findings for any concrete platform framework references.
/// Algorithmic Complexity: O(L * F) where L is lines and F is forbidden frameworks count.
pub fn detect_framework_leaks(doc: &MarkdownDoc, content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();

    // 1. Check YAML frontmatter for forbidden platform field
    if let Some(ref fm) = doc.frontmatter {
        if let Some(platform_val) = fm.get("platform") {
            findings.push(SpecAuditFinding::new(
                "platform-independence-frontmatter-platform-field",
                format!(
                    "Specification '{}' contains forbidden 'platform: {}' field in YAML frontmatter. Tier 1 specs must be platform-agnostic.",
                    rel_path, platform_val
                ),
                rel_path,
                None,
            ));
        }
    }

    // 2. Scan lines outside code blocks for forbidden frameworks
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

        // Check each forbidden framework
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

        // Special handling for "react" to distinguish from the English verb "react to" or "react with"
        if contains_token_with_word_boundary(trimmed, "react") {
            let lower = trimmed.to_lowercase();
            let mut is_english_verb = false;

            let verb_phrases = ["react to", "react with", "react on", "reacts to", "reaction to"];
            for phrase in verb_phrases {
                if lower.contains(phrase) {
                    is_english_verb = true;
                    break;
                }
            }

            if !is_english_verb {
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

    findings
}

/// Validates Tier 1 platform independence rules across DOM attributes, pixel dimensions, and framework leaks.
///
/// Preconditions: `doc` is parsed MarkdownDoc, `content` is raw text.
/// Postconditions: Returns Ok(Vec<SpecAuditFinding>) detailing any platform independence violations.
/// Algorithmic Complexity: O(N) where N is content byte length.
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

/// Validates both LUMI 3-Layer Semantic Chain compliance and Tier 1 Platform Independence.
///
/// Preconditions: `doc` is parsed MarkdownDoc, `content` is raw text.
/// Postconditions: Returns Ok(Vec<SpecAuditFinding>) detailing all LUMI and platform independence findings.
/// Algorithmic Complexity: O(N) where N is content byte length.
pub fn validate_logical_ui(
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let mut findings = Vec::new();

    findings.extend(validate_3layer_chain(doc, content, rel_path));
    findings.extend(validate_container_queries(content, rel_path));
    findings.extend(validate_layout_guidelines(content, rel_path));
    findings.extend(validate_platform_independence(doc, content, rel_path)?);

    Ok(findings)
}

/// Convenience entry point that parses Markdown content and validates LUMI and platform independence.
///
/// Preconditions: `content` is valid UTF-8 Markdown text.
/// Postconditions: Returns Ok(Vec<SpecAuditFinding>) or Err(SpecAuditError) if Markdown parsing fails.
/// Algorithmic Complexity: O(N) where N is content byte length.
pub fn validate_logical_ui_content(
    content: &str,
    rel_path: &str,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let doc = parse_markdown(content)?;
    validate_logical_ui(&doc, content, rel_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_lumi_spec_passes() {
        let content = r#"---
title: "Telemetry Viewport Feature"
interface_type: "ui"
---

# Feat-01 Telemetry Viewport

## 1. Context
Visual telemetry monitoring interface.

## 3. Visual Layout & Arrangement
The UI uses responsive split panes with @container isolation rules.

### Layer 1: Domain State & Signal Model
- SensorTelemetryStream
- TelemetryBuffer

### Layer 2: Logic & Safety State Management
- TelemetryViewModel
- SafetyStateController

### Layer 3: Presentation & Actuator Interface Binding
- TelemetryViewportWidget
- ActuatorControlPanel
"#;
        let findings = validate_logical_ui_content(content, "docs/features/feat-01.md").unwrap();
        assert!(
            findings.is_empty(),
            "Expected valid LUMI spec to produce 0 findings, got: {:?}",
            findings
        );
    }

    #[test]
    fn test_lumi_missing_layer_fails() {
        let content = r#"---
title: "Telemetry Viewport Feature"
interface_type: "ui"
---

# Feat-01 Telemetry Viewport

## 3. Visual Layout & Arrangement
The UI uses responsive split panes with @container isolation rules.

### Layer 1: Domain State & Signal Model
- SensorTelemetryStream

### Layer 3: Presentation & Actuator Interface Binding
- TelemetryViewportWidget
"#;
        let findings = validate_logical_ui_content(content, "docs/features/feat-01.md").unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "lumi-3-layer-chain-incomplete");
        assert!(findings[0].message.contains("Layer 2: Logic & Safety State Management"));
    }

    #[test]
    fn test_lumi_split_panes_missing_container_query_fails() {
        let content = r#"---
title: "Telemetry Viewport Feature"
interface_type: "ui"
---

# Feat-01 Telemetry Viewport

## 3. Visual Layout & Arrangement
The UI features a resizable split pane between telemetry and controls.

### Layer 1: Domain State & Signal Model
- TelemetryModel

### Layer 2: Logic & Safety State Management
- TelemetryViewModel

### Layer 3: Presentation & Actuator Interface Binding
- ViewportWidget
"#;
        let findings = validate_logical_ui_content(content, "docs/features/feat-01.md").unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "lumi-missing-container-query");
    }

    #[test]
    fn test_platform_independence_dom_attribute_leak_fails() {
        let content = r#"# US-01 User Authentication

The user inputs credentials in the form.

The input field carries aria-label="Username" and role="textbox" with tabindex="0".
"#;
        let findings = validate_logical_ui_content(content, "docs/user-stories/us-01.md").unwrap();
        assert_eq!(findings.len(), 3);
        for f in &findings {
            assert_eq!(f.rule_id, "platform-independence-dom-attribute-leak");
        }
    }

    #[test]
    fn test_platform_independence_hardcoded_pixels_fails() {
        let content = r#"# Feat-02 Status Card

The card width is fixed at 450px and height at 120px.
Maximum bound is 1000px.
"#;
        let findings = validate_logical_ui_content(content, "docs/features/feat-02.md").unwrap();
        assert_eq!(findings.len(), 3);
        assert_eq!(findings[0].rule_id, "platform-independence-hardcoded-pixel-dimension");
        assert!(findings[0].message.contains("450px"));
        assert!(findings[1].message.contains("120px"));
        assert!(findings[2].message.contains("1000px"));
    }

    #[test]
    fn test_platform_independence_framework_leak_fails() {
        let content = r#"# Feat-03 Flight Display

Acceptance Criteria:
- Built with Flutter BLoC pattern.
- State rendered using React component.
- Supports Electron desktop packaging.
"#;
        let findings = validate_logical_ui_content(content, "docs/features/feat-03.md").unwrap();
        assert_eq!(findings.len(), 3);
        for f in &findings {
            assert_eq!(f.rule_id, "platform-independence-framework-leak");
        }
    }

    #[test]
    fn test_english_verb_react_to_is_not_flagged() {
        let content = r#"# US-02 Emergency Response

The vehicle flight computer must react to emergency abort signals within 10 milliseconds.
The system will react with an automatic parachute deployment.
"#;
        let findings = validate_logical_ui_content(content, "docs/user-stories/us-02.md").unwrap();
        assert!(
            findings.is_empty(),
            "English verb 'react to' / 'react with' should not be flagged as framework leak: {:?}",
            findings
        );
    }

    #[test]
    fn test_pixel_detection_boundary_cases() {
        assert_eq!(find_pixel_dimensions_in_line("width: 120px;"), vec!["120px"]);
        assert_eq!(find_pixel_dimensions_in_line("margin: 2.5px;"), vec!["2.5px"]);
        assert!(find_pixel_dimensions_in_line("complex rendering").is_empty());
        assert!(find_pixel_dimensions_in_line("px-to-rem").is_empty());
        assert!(find_pixel_dimensions_in_line("approx 10").is_empty());
    }

    #[test]
    fn test_layout_guidelines_raw_na_fails() {
        let content = r#"# Feat-04 Navigation Pane

| Channel | Component | Container |
| :--- | :--- | :--- |
| GUI | NavigationTree | N/A |
"#;
        let findings = validate_logical_ui_content(content, "docs/features/feat-04.md").unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "lumi-prohibit-raw-na-fallback");
    }
}
