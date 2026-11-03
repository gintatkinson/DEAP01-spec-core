//! Unit tests for placeholder cleaning, checklist item extraction, and parent checklist reconciliation.
//! Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST]

use crate::checklist::*;
use crate::tracker::{Issue, IssueIndex};

#[test]
fn test_contains_unpopulated_placeholders() {
    assert!(contains_unpopulated_placeholders("- [ ] (To be created)"));
    assert!(contains_unpopulated_placeholders("- [ ] *(To be created)*"));
    assert!(contains_unpopulated_placeholders("- [ ] TBD"));
    assert!(contains_unpopulated_placeholders("- [ ] TODO"));
    assert!(contains_unpopulated_placeholders("- [ ] Pending"));
    assert!(contains_unpopulated_placeholders("- (To be created)"));
    assert!(contains_unpopulated_placeholders("* [ ] #000"));
    assert!(!contains_unpopulated_placeholders(
        "- [ ] [FEAT-001: Valid Feature](#42)"
    ));
    assert!(!contains_unpopulated_placeholders(
        "```markdown\n- [ ] (To be created)\n```"
    ));
}

#[test]
fn test_contains_unpopulated_placeholders_tilde_fences() {
    let md_in_tilde = "~~~markdown\n- [ ] (To be created)\n- [ ] TBD\n~~~";
    assert!(!contains_unpopulated_placeholders(md_in_tilde));

    let md_after_tilde = "~~~markdown\ncode\n~~~\n- [ ] (To be created)";
    assert!(contains_unpopulated_placeholders(md_after_tilde));
}

#[test]
fn test_clean_unpopulated_placeholders() {
    let md = "\
## Features

- [ ] (To be created)
- [ ] [FEAT-001: Real Feature](#10)
- [ ] TBD
- [ ] *(To be created)*

```markdown
- [ ] (To be created)
```
";
    let (cleaned, count) = clean_unpopulated_placeholders_counted(md);
    assert_eq!(count, 3);
    assert!(cleaned.contains("- [ ] [FEAT-001: Real Feature](#10)"));
    assert!(cleaned.contains("```markdown\n- [ ] (To be created)\n```"));
    assert!(!cleaned.contains("TBD"));
}

#[test]
fn test_clean_unpopulated_placeholders_tilde_fences() {
    let md = "\
## Features

~~~markdown
- [ ] (To be created)
- [ ] TBD
~~~

- [ ] (To be created)
- [ ] [FEAT-001: Keep Me](#10)
";
    let (cleaned, count) = clean_unpopulated_placeholders_counted(md);
    assert_eq!(count, 1);
    assert!(cleaned.contains("~~~markdown\n- [ ] (To be created)\n- [ ] TBD\n~~~"));
    assert!(cleaned.contains("- [ ] [FEAT-001: Keep Me](#10)"));
}

#[test]
fn test_extract_checklist_title_and_issue() {
    assert_eq!(
        extract_checklist_title_and_issue(
            "- [ ] [FEAT-001: Title](https://github.com/org/repo/issues/123)"
        ),
        Some(("FEAT-001: Title".to_string(), Some(123)))
    );
    assert_eq!(
        extract_checklist_title_and_issue("- [x] [FEAT-002: Title](#456)"),
        Some(("FEAT-002: Title".to_string(), Some(456)))
    );
    assert_eq!(
        extract_checklist_title_and_issue(
            "- [ ] [FEAT-003: Title](docs/features/feat-003.md) (#789)"
        ),
        Some(("FEAT-003: Title".to_string(), Some(789)))
    );
    assert_eq!(
        extract_checklist_title_and_issue("- [ ] Plain text item (#101)"),
        Some(("Plain text item".to_string(), Some(101)))
    );
    assert_eq!(
        extract_checklist_title_and_issue("- [ ] Plain text item without issue"),
        Some(("Plain text item without issue".to_string(), None))
    );
    assert_eq!(extract_checklist_title_and_issue("Not a checklist"), None);
}

#[test]
fn test_reconcile_parent_checklists_ticks_and_unticks() -> Result<(), Box<dyn std::error::Error>> {
    let issues = vec![
        Issue {
            number: 10,
            title: "FEAT-001: Auth".to_string(),
            state: "CLOSED".to_string(),
            labels: vec!["type:feature".to_string()],
            body: String::new(),
        },
        Issue {
            number: 11,
            title: "FEAT-002: Billing".to_string(),
            state: "OPEN".to_string(),
            labels: vec!["type:feature".to_string()],
            body: String::new(),
        },
        Issue {
            number: 12,
            title: "FEAT-003: Resolved via Label".to_string(),
            state: "OPEN".to_string(),
            labels: vec![
                "type:feature".to_string(),
                "status:fixed-resolved".to_string(),
            ],
            body: String::new(),
        },
    ];
    let index = IssueIndex::from_issues(issues);

    let md = "\
# EPIC-001: Platform

## Features
- [ ] [FEAT-001: Auth](https://github.com/org/repo/issues/10)
- [x] [FEAT-002: Billing](https://github.com/org/repo/issues/11)
- [ ] FEAT-003: Resolved via Label
- [ ] (To be created)
";

    let (reconciled, changes) = reconcile_checklists_in_content(md, &index, Some("Epic"))?;
    assert_eq!(changes, 3);
    assert!(reconciled.contains("- [x] [FEAT-001: Auth](https://github.com/org/repo/issues/10)"));
    assert!(reconciled.contains("- [ ] [FEAT-002: Billing](https://github.com/org/repo/issues/11)"));
    assert!(reconciled.contains("- [x] FEAT-003: Resolved via Label"));
    assert!(!reconciled.contains("(To be created)"));
    Ok(())
}

#[test]
fn test_reconcile_checklists_ignores_tilde_fenced_blocks() -> Result<(), Box<dyn std::error::Error>> {
    let issues = vec![Issue {
        number: 10,
        title: "FEAT-001: Auth".to_string(),
        state: "CLOSED".to_string(),
        labels: vec!["type:feature".to_string()],
        body: String::new(),
    }];
    let index = IssueIndex::from_issues(issues);

    let md = "\
~~~markdown
- [ ] [FEAT-001: Auth](#10)
~~~
- [ ] [FEAT-001: Auth](#10)
";
    let (reconciled, changes) = reconcile_checklists_in_content(md, &index, Some("Epic"))?;
    assert_eq!(changes, 1);
    assert!(reconciled.contains("~~~markdown\n- [ ] [FEAT-001: Auth](#10)\n~~~"));
    assert!(reconciled.ends_with("- [x] [FEAT-001: Auth](#10)\n"));
    Ok(())
}
