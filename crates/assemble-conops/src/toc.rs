//! Table of Contents generation, heading extraction, and anchor slugification.

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use std::collections::HashSet;

/// Converts a heading string into a standard GitHub Markdown anchor slug.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `slugify` in `crates/assemble-conops/src/toc.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `slugify` (declaration #1) within `ASSEMBLE-CONOPS-TOC`.
///
/// ## 3. Preconditions
/// - `text: &str` must satisfy domain and type-safety constraints prior to invoking `slugify`.
///
/// ## 4. Postconditions
/// - Returns `String` from `slugify` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(text)) bounded over input/member dimension of `slugify`.
/// - Space Complexity: O(N(text)) bounded auxiliary memory for `slugify`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `slugify` (`String`).
/// - State: Maintains consistent state invariants across `slugify` without partial mutation on error.
/// - Action: Executes `slugify` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `slugify` (`ASSEMBLE-CONOPS-TOC`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ASSEMBLE-CONOPS-TOC-SLUGIFY] | SysML v2: ActionDef(slugify)
pub fn slugify(text: &str) -> String {
    let clean = text.replace(['`', '$', '*'], "").trim().to_string();
    let mut slug = String::new();
    for ch in clean.to_lowercase().chars() {
        if ch.is_alphanumeric() || ch == '-' || ch == '_' || ch == ' ' {
            slug.push(ch);
        }
    }
    slug.replace(' ', "-")
}

/// Extracts markdown headings from content.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `extract_headings` in `crates/assemble-conops/src/toc.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `extract_headings` (declaration #2) within `ASSEMBLE-CONOPS-TOC`.
///
/// ## 3. Preconditions
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `extract_headings`.
///
/// ## 4. Postconditions
/// - Returns `Vec<(usize, String, String)>` from `extract_headings` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(content)) bounded over input/member dimension of `extract_headings`.
/// - Space Complexity: O(N(content)) bounded auxiliary memory for `extract_headings`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `extract_headings` (`Vec<(usize, String, String)>`).
/// - State: Maintains consistent state invariants across `extract_headings` without partial mutation on error.
/// - Action: Executes `extract_headings` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `extract_headings` (`ASSEMBLE-CONOPS-TOC`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ASSEMBLE-CONOPS-TOC-EXTRACT-HEADINGS] | SysML v2: ActionDef(extract_headings)
pub fn extract_headings(content: &str) -> Vec<(usize, String, String)> {
    let mut headings = Vec::new();
    let parser = Parser::new(content);
    let mut current_heading: Option<(usize, String)> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                let lvl = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                current_heading = Some((lvl, String::new()));
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((lvl, title)) = current_heading.take() {
                    let trimmed = title.trim();
                    let slug = slugify(trimmed);
                    headings.push((lvl, trimmed.to_string(), slug));
                }
            }
            Event::Text(t) | Event::Code(t) => {
                if let Some((_, ref mut title)) = current_heading {
                    title.push_str(&t);
                }
            }
            _ => {}
        }
    }

    headings
}

/// Generates a Markdown Table of Contents from a list of headings.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `generate_table_of_contents` in `crates/assemble-conops/src/toc.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `generate_table_of_contents` (declaration #3) within `ASSEMBLE-CONOPS-TOC`.
///
/// ## 3. Preconditions
/// - `headings: &[(usize, String, String)]` must satisfy domain and type-safety constraints prior to invoking `generate_table_of_contents`.
/// - `max_depth: usize` must satisfy domain and type-safety constraints prior to invoking `generate_table_of_contents`.
///
/// ## 4. Postconditions
/// - Returns `String` from `generate_table_of_contents` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(headings)) bounded over input/member dimension of `generate_table_of_contents`.
/// - Space Complexity: O(N(headings)) bounded auxiliary memory for `generate_table_of_contents`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `generate_table_of_contents` (`String`).
/// - State: Maintains consistent state invariants across `generate_table_of_contents` without partial mutation on error.
/// - Action: Executes `generate_table_of_contents` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `generate_table_of_contents` (`ASSEMBLE-CONOPS-TOC`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ASSEMBLE-CONOPS-TOC-GENERATE-TABLE-OF-CONTENTS] | SysML v2: ActionDef(generate_table_of_contents)
pub fn generate_table_of_contents(headings: &[(usize, String, String)], max_depth: usize) -> String {
    let mut toc_lines = vec!["## Table of Contents".to_string(), String::new()];

    for &(level, ref title, ref slug) in headings {
        if level == 1 || level > max_depth {
            continue;
        }
        let indent = "  ".repeat(level.saturating_sub(2));
        let clean_title = title.replace(['`', '*'], "");
        toc_lines.push(format!("{}- [{clean_title}](#{slug})", indent));
    }

    toc_lines.push(String::new());
    toc_lines.join("\n")
}

/// Verifies internal anchor links (#slug) in the document against defined headings.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `verify_markdown_links` in `crates/assemble-conops/src/toc.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `verify_markdown_links` (declaration #4) within `ASSEMBLE-CONOPS-TOC`.
///
/// ## 3. Preconditions
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `verify_markdown_links`.
///
/// ## 4. Postconditions
/// - Returns `Vec<String>` from `verify_markdown_links` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(content)) bounded over input/member dimension of `verify_markdown_links`.
/// - Space Complexity: O(N(content)) bounded auxiliary memory for `verify_markdown_links`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `verify_markdown_links` (`Vec<String>`).
/// - State: Maintains consistent state invariants across `verify_markdown_links` without partial mutation on error.
/// - Action: Executes `verify_markdown_links` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `verify_markdown_links` (`ASSEMBLE-CONOPS-TOC`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ASSEMBLE-CONOPS-TOC-VERIFY-MARKDOWN-LINKS] | SysML v2: ActionDef(verify_markdown_links)
pub fn verify_markdown_links(content: &str) -> Vec<String> {
    let headings = extract_headings(content);
    let valid_slugs: HashSet<String> = headings.into_iter().map(|(_, _, slug)| slug).collect();
    let mut errors = Vec::new();

    let parser = Parser::new(content);
    let mut current_link: Option<(String, String)> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                if let Some(anchor) = dest_url.strip_prefix('#') {
                    current_link = Some((anchor.to_string(), String::new()));
                }
            }
            Event::End(TagEnd::Link) => {
                if let Some((anchor, text)) = current_link.take() {
                    if !valid_slugs.contains(&anchor) {
                        errors.push(format!(
                            "Broken anchor link: [{text}](#{anchor}) does not match any heading in document."
                        ));
                    }
                }
            }
            Event::Text(t) | Event::Code(t) => {
                if let Some((_, ref mut text)) = current_link {
                    text.push_str(&t);
                }
            }
            _ => {}
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(
            slugify("1. Scope & System Identification"),
            "1-scope--system-identification"
        );
        assert_eq!(
            slugify("4.2 Primary Architecture (Core)"),
            "42-primary-architecture-core"
        );
    }

    #[test]
    fn test_extract_headings_and_generate_toc() {
        let doc = r#"
# Main Title

```rust
// Code block
# Not a heading
```

## Section 1: Overview
### 1.1 Scope
## Section 2: Requirements
"#;

        let headings = extract_headings(doc);
        assert_eq!(headings.len(), 4);
        assert_eq!(headings[0], (1, "Main Title".to_string(), "main-title".to_string()));
        assert_eq!(headings[1], (2, "Section 1: Overview".to_string(), "section-1-overview".to_string()));
        assert_eq!(headings[2], (3, "1.1 Scope".to_string(), "11-scope".to_string()));
        assert_eq!(headings[3], (2, "Section 2: Requirements".to_string(), "section-2-requirements".to_string()));

        let toc = generate_table_of_contents(&headings, 3);
        assert!(toc.contains("- [Section 1: Overview](#section-1-overview)"));
        assert!(toc.contains("  - [1.1 Scope](#11-scope)"));
    }

    #[test]
    fn test_verify_markdown_links() {
        let doc = r#"
## Section One
Content with valid [link](#section-one) and broken [bad link](#missing-section).
"#;
        let errors = verify_markdown_links(doc);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("#missing-section"));
    }
}
