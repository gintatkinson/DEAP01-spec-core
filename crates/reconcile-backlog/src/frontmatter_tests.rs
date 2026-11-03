//! Unit tests for `frontmatter` parsing, issue number extraction, placeholder detection, and batch updates.
//! Realises: [REQ-RECONCILE-BACKLOG-FRONTMATTER]

use crate::frontmatter::*;

#[test]
fn test_parse_frontmatter_basic() -> Result<(), Box<dyn std::error::Error>> {
    let md = "\
# EPIC-001: Some Title

| Field | Value |
|---|---|
| **Document ID** | `EPIC-001` |
| **Type** | `Epic` |
| **Title** | EPIC-001: Some Title |
| **Status** | `Approved` |
| **GitHub Issue** | `[#10](https://github.com/org/repo/issues/10)` |

## 1. Overview
| **GitHub Issue** | `#999` |
";
    let fm = parse_frontmatter(md)?;
    assert_eq!(fm.document_id.as_deref(), Some("EPIC-001"));
    assert_eq!(fm.spec_type.as_deref(), Some("Epic"));
    assert_eq!(fm.title.as_deref(), Some("EPIC-001: Some Title"));
    assert_eq!(fm.status.as_deref(), Some("Approved"));
    assert_eq!(
        fm.github_issue.as_deref(),
        Some("`[#10](https://github.com/org/repo/issues/10)`")
    );
    Ok(())
}

#[test]
fn test_parse_frontmatter_case_insensitive() -> Result<(), Box<dyn std::error::Error>> {
    let md = "\
| document id | epic-002 |
| type | epic |
| title | Lowercase Epic |
| status | draft |
| github issue | #99 |
";
    let fm = parse_frontmatter(md)?;
    assert_eq!(fm.document_id.as_deref(), Some("epic-002"));
    assert_eq!(fm.spec_type.as_deref(), Some("epic"));
    assert_eq!(fm.title.as_deref(), Some("Lowercase Epic"));
    assert_eq!(fm.status.as_deref(), Some("draft"));
    assert_eq!(fm.github_issue.as_deref(), Some("#99"));
    Ok(())
}

#[test]
fn test_parse_frontmatter_parent_hierarchy() -> Result<(), Box<dyn std::error::Error>> {
    let md = "\
# US-001: Login Story

| **Type** | `User Story` |
| **Parent Epic** | [EPIC-001: Auth](docs/epics/epic-001.md) |
| **Parent Feature** | [FEAT-001: Login](docs/features/feat-001.md) |
";
    let fm = parse_frontmatter(md)?;
    assert_eq!(fm.spec_type.as_deref(), Some("User Story"));
    assert_eq!(
        fm.parent_epic.as_deref(),
        Some("[EPIC-001: Auth](docs/epics/epic-001.md)")
    );
    assert_eq!(
        fm.parent_feature.as_deref(),
        Some("[FEAT-001: Login](docs/features/feat-001.md)")
    );
    Ok(())
}

#[test]
fn test_parse_frontmatter_fenced_code_blocks() -> Result<(), Box<dyn std::error::Error>> {
    let md = "\
```markdown
# Fake H1 Inside Code Block
| **Title** | Fake Title |
## Fake H2 Inside Code Block
```

# Real H1 Title

| **Type** | `Feature` |
| **GitHub Issue** | `#42` |

## Real H2 Section
";
    let fm = parse_frontmatter(md)?;
    assert_eq!(fm.title.as_deref(), Some("Real H1 Title"));
    assert_eq!(fm.spec_type.as_deref(), Some("Feature"));
    assert_eq!(fm.github_issue.as_deref(), Some("`#42`"));
    Ok(())
}

#[test]
fn test_parse_frontmatter_tilde_fenced_code_blocks() -> Result<(), Box<dyn std::error::Error>> {
    let md = "\
~~~markdown
# Fake Tilde H1
| **Title** | Fake Tilde Title |
## Fake Tilde H2
~~~

# Real H1 After Tilde

| **Type** | `Use Case` |
| **GitHub Issue** | `#77` |

## Real Section
";
    let fm = parse_frontmatter(md)?;
    assert_eq!(fm.title.as_deref(), Some("Real H1 After Tilde"));
    assert_eq!(fm.spec_type.as_deref(), Some("Use Case"));
    assert_eq!(fm.github_issue.as_deref(), Some("`#77`"));
    Ok(())
}

#[test]
fn test_extract_issue_number() {
    assert_eq!(extract_issue_number("#42"), Some(42));
    assert_eq!(extract_issue_number("`#105`"), Some(105));
    assert_eq!(
        extract_issue_number("[#12](https://github.com/org/repo/issues/12)"),
        Some(12)
    );
    assert_eq!(
        extract_issue_number("https://github.com/org/repo/issues/999"),
        Some(999)
    );
    assert_eq!(extract_issue_number("1234"), Some(1234));
    assert_eq!(extract_issue_number("Pending"), None);
    assert_eq!(extract_issue_number("(To be created)"), None);
}

#[test]
fn test_is_placeholder_issue_id() {
    assert!(is_placeholder_issue_id(""));
    assert!(is_placeholder_issue_id("Pending"));
    assert!(is_placeholder_issue_id("`Pending`"));
    assert!(is_placeholder_issue_id("(To be created)"));
    assert!(is_placeholder_issue_id("TBD"));
    assert!(is_placeholder_issue_id("TODO"));
    assert!(is_placeholder_issue_id("None"));
    assert!(is_placeholder_issue_id("N/A"));
    assert!(is_placeholder_issue_id("#000"));
    assert!(is_placeholder_issue_id("#XXX"));
    assert!(!is_placeholder_issue_id("#42"));
    assert!(!is_placeholder_issue_id(
        "[#42](https://github.com/org/repo/issues/42)"
    ));
}

#[test]
fn test_update_frontmatter_field_replaces_existing() -> Result<(), Box<dyn std::error::Error>> {
    let md = "\
# FEAT-001: Something

| Field | Value |
|---|---|
| **Type** | `Feature` |
| **GitHub Issue** | *(Pending)* |

## Details
";
    let updated = update_frontmatter_field(
        md,
        "GitHub Issue",
        "[#55](https://github.com/org/repo/issues/55)",
    )?;
    assert!(updated.contains("| **GitHub Issue** | [#55](https://github.com/org/repo/issues/55) |"));
    assert!(!updated.contains("Pending"));
    Ok(())
}

#[test]
fn test_update_frontmatter_field_inserts_into_existing_table() -> Result<(), Box<dyn std::error::Error>> {
    let md = "\
# FEAT-001: Something

| Field | Value |
|---|---|
| **Type** | `Feature` |

## Details
";
    let updated = update_frontmatter_field(md, "GitHub Issue", "#55")?;
    assert!(updated.contains("| **Type** | `Feature` |\n| **GitHub Issue** | #55 |"));
    Ok(())
}

#[test]
fn test_update_frontmatter_field_creates_table_after_h1() -> Result<(), Box<dyn std::error::Error>> {
    let md = "\
# FEAT-001: No Table Yet

## Details
Some text.
";
    let updated = update_frontmatter_field(md, "GitHub Issue", "#55")?;
    assert!(updated.contains(
        "# FEAT-001: No Table Yet\n\n| Field | Value |\n|---|---|\n| **GitHub Issue** | #55 |"
    ));
    Ok(())
}

#[test]
fn test_update_frontmatter_field_preserves_crlf() -> Result<(), Box<dyn std::error::Error>> {
    let md = "# FEAT-001: Title\r\n\r\n| Field | Value |\r\n|---|---|\r\n| **GitHub Issue** | Pending |\r\n";
    let updated = update_frontmatter_field(md, "GitHub Issue", "#10")?;
    assert!(updated.contains("\r\n"));
    assert!(updated.contains("| **GitHub Issue** | #10 |\r\n"));
    Ok(())
}
