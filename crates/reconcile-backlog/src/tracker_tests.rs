//! Unit tests for tracker JSON parsing, label normalization, title normalization, and `IssueIndex` matching.
//! Realises: [REQ-RECONCILE-BACKLOG-TRACKER]

use crate::tracker::*;

#[test]
fn test_issue_is_resolved() {
    let closed_issue = Issue {
        number: 1,
        title: "Test".to_string(),
        state: "CLOSED".to_string(),
        labels: vec![],
        body: String::new(),
    };
    assert!(closed_issue.is_resolved());

    let open_with_label = Issue {
        number: 2,
        title: "Test".to_string(),
        state: "OPEN".to_string(),
        labels: vec!["status:fixed-resolved".to_string()],
        body: String::new(),
    };
    assert!(open_with_label.is_resolved());

    let open_with_gitlab_label = Issue {
        number: 3,
        title: "Test".to_string(),
        state: "OPEN".to_string(),
        labels: vec!["status::fixed-resolved".to_string()],
        body: String::new(),
    };
    assert!(open_with_gitlab_label.is_resolved());

    let open_unresolved = Issue {
        number: 4,
        title: "Test".to_string(),
        state: "OPEN".to_string(),
        labels: vec!["type:feature".to_string()],
        body: String::new(),
    };
    assert!(!open_unresolved.is_resolved());
}

#[test]
fn test_normalize_label_and_canonicalize_spec_type() {
    assert_eq!(
        normalize_label("status::fixed-resolved"),
        "status:fixed-resolved"
    );
    assert_eq!(normalize_label("type::user_story"), "type:user-story");
    assert_eq!(canonicalize_spec_type("type:epic"), "epic");
    assert_eq!(canonicalize_spec_type("Feature"), "feature");
    assert_eq!(canonicalize_spec_type("User Story"), "user_story");
    assert_eq!(canonicalize_spec_type("type::use-case"), "use_case");
}

#[test]
fn test_normalize_title_and_ordinals() {
    assert_eq!(
        normalize_title("[FEAT-001: Auth Module](docs/features/feat-001.md)"),
        "feat 001 auth module"
    );
    assert_eq!(
        extract_spec_ordinal("FEAT-001: Auth Module"),
        Some(("feat".to_string(), 1))
    );
    assert_eq!(
        extract_spec_ordinal("FEATURE-1: Auth"),
        Some(("feat".to_string(), 1))
    );
    assert!(ordinal_compatible("FEAT-001: Auth", "FEAT-1: Auth Module"));
    assert!(!ordinal_compatible(
        "FEAT-001: Auth Module",
        "FEAT-002: Auth Module"
    ));
    assert!(!ordinal_compatible("EPIC-001: Core", "FEAT-001: Core"));
}

#[test]
fn test_issue_index_exact_and_fuzzy_matching() {
    let issues = vec![
        Issue {
            number: 10,
            title: "EPIC-001: Platform Core".to_string(),
            state: "OPEN".to_string(),
            labels: vec!["type:epic".to_string()],
            body: String::new(),
        },
        Issue {
            number: 20,
            title: "FEAT-001: Platform Core".to_string(),
            state: "OPEN".to_string(),
            labels: vec!["type:feature".to_string()],
            body: String::new(),
        },
        Issue {
            number: 30,
            title: "FEAT-002: Authentication Service".to_string(),
            state: "CLOSED".to_string(),
            labels: vec!["type:feature".to_string()],
            body: String::new(),
        },
    ];

    let index = IssueIndex::from_issues(issues);

    assert_eq!(
        index.find_issue_for_spec("EPIC-001: Platform Core", Some("Epic")),
        Some(10)
    );
    assert_eq!(
        index.find_issue_for_spec("FEAT-001: Platform Core", Some("Feature")),
        Some(20)
    );
    assert_eq!(
        index.find_issue_for_spec("FEAT-002: Authentication", Some("Feature")),
        Some(30)
    );
    assert_eq!(
        index.find_issue_for_spec("FEAT-003: Authentication Service", Some("Feature")),
        None
    );
}

#[test]
fn test_parse_gh_json() -> Result<(), Box<dyn std::error::Error>> {
    let raw = r#"[
        {
            "number": 42,
            "title": "EPIC-001: Core",
            "state": "OPEN",
            "labels": [{"name": "type:epic"}, {"name": "status:fixed-resolved"}],
            "body": "Body text"
        },
        {
            "number": 43,
            "title": "FEAT-001: Sub",
            "state": "CLOSED",
            "labels": [],
            "body": null
        }
    ]"#;

    let parsed = GitHubTracker::parse_gh_json(raw)?;
    assert_eq!(parsed.len(), 2);
    let first = parsed.first().ok_or("missing first")?;
    let second = parsed.get(1).ok_or("missing second")?;
    assert_eq!(first.number, 42);
    assert!(first.is_resolved());
    assert_eq!(second.number, 43);
    assert!(second.is_resolved());
    assert_eq!(second.body, "");
    Ok(())
}

#[test]
fn test_offline_tracker() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = OfflineTracker::with_issues(vec![Issue {
        number: 1,
        title: "EPIC-001: Offline".to_string(),
        state: "OPEN".to_string(),
        labels: vec![],
        body: "Hello".to_string(),
    }]);

    let fetched = tracker.fetch_all_issues(None)?;
    assert_eq!(fetched.len(), 1);
    assert!(tracker.edit_issue_body(1, "New", None).is_ok());
    assert_eq!(tracker.create_issue("New", "Body", &[], None)?, 0);
    Ok(())
}
