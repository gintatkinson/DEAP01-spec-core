//! Table of Contents generation, heading extraction, and anchor slugification.

use regex::Regex;
use std::collections::HashSet;

/// Converts a heading string into a standard GitHub Markdown anchor slug.
/// Example: "1. Scope & System Identification" -> "1-scope--system-identification"
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
/// Returns list of tuples: (level, title, slug).
/// Ignores code blocks and math blocks.
pub fn extract_headings(content: &str) -> Vec<(usize, String, String)> {
    let mut headings = Vec::new();
    let mut in_code_block = false;
    let mut in_math_block = false;

    let heading_re = Regex::new(r"^(#{1,6})\s+(.+)$").unwrap();

    for line in content.lines() {
        let stripped = line.trim();
        if stripped.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if stripped.starts_with("$$") {
            if stripped.len() > 2 && stripped.ends_with("$$") {
                continue;
            }
            in_math_block = !in_math_block;
            continue;
        }
        if in_code_block || in_math_block {
            continue;
        }

        if let Some(caps) = heading_re.captures(stripped) {
            let hashes = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let title = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            let level = hashes.len();
            let slug = slugify(title);
            headings.push((level, title.to_string(), slug));
        }
    }

    headings
}

/// Generates a Markdown Table of Contents from a list of headings.
/// Skips the top-level H1 title.
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
/// Returns list of error messages for any broken anchor links.
pub fn verify_markdown_links(content: &str) -> Vec<String> {
    let headings = extract_headings(content);
    let valid_slugs: HashSet<String> = headings.into_iter().map(|(_, _, slug)| slug).collect();
    let mut errors = Vec::new();

    let link_re = Regex::new(r"\[([^\]]+)\]\(#([^\)]+)\)").unwrap();
    for caps in link_re.captures_iter(content) {
        let text = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let anchor = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        if !valid_slugs.contains(anchor) {
            errors.push(format!(
                "Broken anchor link: [{text}](#{anchor}) does not match any heading in document."
            ));
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
            slugify("4.2 Primary Architecture (UAV)"),
            "42-primary-architecture-uav"
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
