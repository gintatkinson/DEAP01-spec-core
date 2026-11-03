#![cfg(test)]

use super::*;

#[test]
fn test_parse_markdown_headings_and_metadata() {
    let content = r#"---
title: "Test Epic"
type: epic
---

# Test Epic H1

| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | 100 |
| **Title** | Test Epic |
| **Package** | SystemPackage |
| **Subsystem** | FlightControl |

## 1. Context
This is introductory prose text.

```mermaid
classDiagram
    class FlightControl {
        +execute() void
    }
```
"#;
    let doc = parse_markdown(content).unwrap_or_else(|_| MarkdownDoc {
        headings: Vec::new(),
        frontmatter: None,
        table_metadata: BTreeMap::new(),
        prose: Vec::new(),
        mermaid_blocks: Vec::new(),
        unresolved_placeholders: Vec::new(),
    });
    assert_eq!(doc.headings.len(), 2);
    assert_eq!(doc.headings.first().map(|h| h.level), Some(1));
    assert_eq!(
        doc.headings.first().map(|h| h.text.as_str()),
        Some("Test Epic H1")
    );
    assert_eq!(doc.headings.get(1).map(|h| h.level), Some(2));
    assert_eq!(
        doc.headings.get(1).map(|h| h.text.as_str()),
        Some("1. Context")
    );

    assert_eq!(
        doc.table_metadata.get("issue_id").map(|s| s.as_str()),
        Some("100")
    );
    assert_eq!(
        doc.table_metadata.get("package").map(|s| s.as_str()),
        Some("SystemPackage")
    );

    assert_eq!(doc.mermaid_blocks.len(), 1);
    assert_eq!(
        doc.mermaid_blocks.first().map(|b| b.header.as_str()),
        Some("classDiagram")
    );
    assert!(doc.unresolved_placeholders.is_empty());
}

#[test]
fn test_detect_unresolved_placeholders() {
    let content = "Check issue #[IssueID] here.\nAnother line with #TBD.\nValid text.\n";
    let findings = find_unresolved_placeholders(content, false);
    assert_eq!(findings.len(), 2);
    assert_eq!(findings.first().map(|f| f.line), Some(1));
    assert_eq!(findings.get(1).map(|f| f.line), Some(2));
}

#[test]
fn test_metadata_row_allowed_placeholders() {
    let content = "| **Issue ID** | #[IssueID] |\n";
    let findings = find_unresolved_placeholders(content, false);
    assert!(
        findings.is_empty(),
        "Allowed metadata placeholder should not trigger error"
    );
}
