//! Table of Contents generation, heading extraction, and anchor slugification.

use deap_core::markdown_ast::parse_markdown_ast;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::collections::BTreeSet;

/// [DOC-TIER: A] Converts a heading string into a standard GitHub Markdown anchor slug.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-TOC-SLUGIFY] | SysML v2: ActionDef(slugify)
/// Contract: Accepts raw heading `text: &str` and returns lowercase hyphenated anchor slug `String` stripped of formatting tokens.
/// Bounds & Faults: O(N) time and space over `text.len()`; infallible deterministic character filtering.
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

/// [DOC-TIER: A] Extracts markdown headings from content via the `deap_core::markdown_ast` (`pulldown-cmark`) section AST.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-TOC-EXTRACT-HEADINGS] | SysML v2: ActionDef(extract_headings)
/// Contract: Parses `content: &str` into `ParsedMarkdownAst` and returns `Vec<(usize, String, String)>` of `(level, title, slug)`.
/// Bounds & Faults: O(N) time and space over `content.len()`; ignores `#` lines inside fenced code blocks via AST parsing.
pub fn extract_headings(content: &str) -> Vec<(usize, String, String)> {
    let ast = parse_markdown_ast(content);
    ast.sections
        .into_iter()
        .map(|sec| {
            let slug = slugify(&sec.heading);
            (usize::from(sec.level), sec.heading, slug)
        })
        .collect()
}

/// [DOC-TIER: A] Generates a Markdown Table of Contents from a list of extracted headings.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-TOC-GENERATE-TABLE-OF-CONTENTS] | SysML v2: ActionDef(generate_table_of_contents)
/// Contract: Formats `headings` in `2..=max_depth` into an indented Markdown bullet list with `#slug` links.
/// Bounds & Faults: O(N) time and space over `headings.len()`; saturating indentation prevents underflow.
pub fn generate_table_of_contents(
    headings: &[(usize, String, String)],
    max_depth: usize,
) -> String {
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

/// [DOC-TIER: A] Verifies internal anchor links (`#slug`) in the document against AST-extracted headings.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-TOC-VERIFY-MARKDOWN-LINKS] | SysML v2: ActionDef(verify_markdown_links)
/// Contract: Returns `Vec<String>` of diagnostic messages for every internal `#anchor` link missing from `extract_headings(content)`.
/// Bounds & Faults: O(N log H) time and O(N) space over `content.len()`; deterministic link event stream traversal.
pub fn verify_markdown_links(content: &str) -> Vec<String> {
    let headings = extract_headings(content);
    let valid_slugs: BTreeSet<String> = headings.into_iter().map(|(_, _, slug)| slug).collect();
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
        assert_eq!(
            headings.first(),
            Some(&(1, "Main Title".to_string(), "main-title".to_string()))
        );
        assert_eq!(
            headings.get(1),
            Some(&(
                2,
                "Section 1: Overview".to_string(),
                "section-1-overview".to_string()
            ))
        );
        assert_eq!(
            headings.get(2),
            Some(&(3, "1.1 Scope".to_string(), "11-scope".to_string()))
        );
        assert_eq!(
            headings.get(3),
            Some(&(
                2,
                "Section 2: Requirements".to_string(),
                "section-2-requirements".to_string()
            ))
        );

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
        assert!(errors
            .first()
            .is_some_and(|e| e.contains("#missing-section")));
    }
}
