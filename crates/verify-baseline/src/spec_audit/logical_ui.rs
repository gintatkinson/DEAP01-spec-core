//! Native Rust LUMI 3-Layer Quality and Platform Independence Validator.
//!
//! Conforms to DO-178C Level A and ISO 26262 ASIL D safety standards.
//! Validates LUMI 3-Layer Semantic Chain, CSS container isolation, and Tier 1 platform independence.

use crate::spec_audit::behavioral::contains_token_with_word_boundary;
pub use crate::spec_audit::logical_ui_platform::{
    detect_dom_leaks, detect_framework_leaks, detect_pixel_dimensions,
    find_pixel_dimensions_in_line, validate_platform_independence,
};
use crate::spec_audit::markdown_ast::{parse_markdown, MarkdownDoc};
use crate::spec_audit::{SpecAuditError, SpecAuditFinding};

/// Checks whether a specification document represents a LUMI (Logical User & Machine Interface) specification.
/// Realises: [REQ-SPEC-AUDIT-LOGICAL-UI-IS-LUMI-SPEC]
pub fn is_lumi_spec(doc: &MarkdownDoc, content: &str) -> bool {
    if let Some(iface) = doc
        .table_metadata
        .get("interface_type")
        .or_else(|| doc.table_metadata.get("interface_types"))
    {
        let lower = iface.to_lowercase();
        if lower.contains("ui")
            || lower.contains("gui")
            || lower.contains("lumi")
            || lower.contains("user")
        {
            return true;
        }
    }

    if let Some(ref fm) = doc.frontmatter {
        if let Some(iface) = fm
            .get("interface_type")
            .or_else(|| fm.get("interface_types"))
        {
            let lower = iface.to_lowercase();
            if lower.contains("ui")
                || lower.contains("gui")
                || lower.contains("lumi")
                || lower.contains("user")
            {
                return true;
            }
        }
    }

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

    let lower_content = content.to_lowercase();
    lower_content.contains("## logical ui & layout bindings")
        || lower_content.contains("## 3. visual layout & arrangement")
        || lower_content.contains("logical ui & interface bindings")
}

/// Concatenates headings, table metadata, and prose into a lowercased searchable string.
/// Realises: [REQ-SPEC-AUDIT-LOGICAL-UI-BUILD-SEARCHABLE-TEXT]
fn build_lumi_searchable_text(doc: &MarkdownDoc, capacity: usize) -> String {
    let mut full_text = String::with_capacity(capacity);
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
    full_text.to_lowercase()
}

/// Validates that a LUMI specification defines the mandatory 3-Layer Semantic Chain.
/// Realises: [REQ-SPEC-AUDIT-LOGICAL-UI-VALIDATE-3LAYER-CHAIN]
pub fn validate_3layer_chain(
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    if !is_lumi_spec(doc, content) {
        return findings;
    }

    let lower_text = build_lumi_searchable_text(doc, content.len());

    let has_layer_1 = lower_text.contains("layer 1")
        || lower_text.contains("domain state & signal model")
        || lower_text.contains("domain state and signal model")
        || (lower_text.contains("domain state") && lower_text.contains("signal model"))
        || lower_text.contains("domain model");

    let has_layer_2 = lower_text.contains("layer 2")
        || lower_text.contains("logic & safety state management")
        || lower_text.contains("logic and safety state management")
        || lower_text.contains("safety state")
        || lower_text.contains("state management")
        || lower_text.contains("viewmodel")
        || lower_text.contains("logic & state");

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
/// Realises: [REQ-SPEC-AUDIT-LOGICAL-UI-VALIDATE-CONTAINER-QUERIES]
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
/// Realises: [REQ-SPEC-AUDIT-LOGICAL-UI-VALIDATE-LAYOUT-GUIDELINES]
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

        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            for cell in trimmed.split('|') {
                let cell_clean = cell
                    .trim()
                    .trim_matches(|c| c == '*' || c == '`' || c == '_');
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

/// Validates both LUMI 3-Layer Semantic Chain compliance and Tier 1 Platform Independence.
/// Realises: [REQ-SPEC-AUDIT-LOGICAL-UI-VALIDATE-LOGICAL-UI]
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
/// Realises: [REQ-SPEC-AUDIT-LOGICAL-UI-VALIDATE-LOGICAL-UI-CONTENT]
pub fn validate_logical_ui_content(
    content: &str,
    rel_path: &str,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let doc = parse_markdown(content)?;
    validate_logical_ui(&doc, content, rel_path)
}

#[cfg(test)]
#[path = "logical_ui_tests.rs"]
mod tests;
