//! Unit tests for `create_issue` precondition validation, argument normalization, and idempotent issue creation.
//! Realises: [REQ-RECONCILE-BACKLOG-CREATE-ISSUE]

use crate::create_issue::*;
use crate::lib_tests::NamedTempFile;
use crate::tracker::{Issue, IssueTracker};
use crate::ReconcileError;
use std::cell::RefCell;
use std::io::Write;
use std::path::PathBuf;

struct MockTracker {
    issues: RefCell<Vec<Issue>>,
    next_id: RefCell<u64>,
}

impl MockTracker {
    fn new(initial: Vec<Issue>) -> Self {
        let max_id = initial.iter().map(|i| i.number).max().unwrap_or(0);
        Self {
            issues: RefCell::new(initial),
            next_id: RefCell::new(max_id + 1),
        }
    }
}

impl IssueTracker for MockTracker {
    fn fetch_all_issues(&self, _repo: Option<&str>) -> Result<Vec<Issue>, ReconcileError> {
        Ok(self.issues.borrow().clone())
    }

    fn edit_issue_body(
        &self,
        number: u64,
        new_body: &str,
        _repo: Option<&str>,
    ) -> Result<(), ReconcileError> {
        let mut issues = self.issues.borrow_mut();
        if let Some(issue) = issues.iter_mut().find(|i| i.number == number) {
            issue.body = new_body.to_string();
        }
        Ok(())
    }

    fn create_issue(
        &self,
        title: &str,
        body: &str,
        labels: &[String],
        _repo: Option<&str>,
    ) -> Result<u64, ReconcileError> {
        let mut next_id = self.next_id.borrow_mut();
        let id = *next_id;
        *next_id += 1;

        self.issues.borrow_mut().push(Issue {
            number: id,
            title: title.to_string(),
            state: "OPEN".to_string(),
            labels: labels.to_vec(),
            body: body.to_string(),
        });

        Ok(id)
    }

    fn reopen_issue(
        &self,
        number: u64,
        comment: &str,
        _repo: Option<&str>,
    ) -> Result<(), ReconcileError> {
        let mut issues = self.issues.borrow_mut();
        if let Some(issue) = issues.iter_mut().find(|i| i.number == number) {
            issue.state = "OPEN".to_string();
            issue.body.push_str(&format!("\n\nReopened: {}", comment));
        }
        Ok(())
    }
}

#[test]
fn test_normalize_args_flags() -> Result<(), Box<dyn std::error::Error>> {
    let args = CreateIssueArgs {
        body_file: Some("body.md".to_string()),
        label: Some("bug,high".to_string()),
        title: Some("Fix crash".to_string()),
        repo: Some("org/repo".to_string()),
        reopen_with_analysis: None,
        provider: "github".to_string(),
        positional: vec![],
    };

    let norm = normalize_args(&args)?;
    assert_eq!(norm.body_file, PathBuf::from("body.md"));
    assert_eq!(norm.labels, vec!["bug", "high"]);
    assert_eq!(norm.title, "Fix crash");
    assert_eq!(norm.repo, Some("org/repo".to_string()));
    Ok(())
}

#[test]
fn test_normalize_args_positional() -> Result<(), Box<dyn std::error::Error>> {
    let args = CreateIssueArgs {
        body_file: None,
        label: None,
        title: None,
        repo: None,
        reopen_with_analysis: None,
        provider: "github".to_string(),
        positional: vec![
            "spec.md".to_string(),
            "type:epic".to_string(),
            "EPIC-001:".to_string(),
            "Core".to_string(),
            "Engine".to_string(),
        ],
    };

    let norm = normalize_args(&args)?;
    assert_eq!(norm.body_file, PathBuf::from("spec.md"));
    assert_eq!(norm.labels, vec!["type:epic"]);
    assert_eq!(norm.title, "EPIC-001: Core Engine");
    Ok(())
}

#[test]
fn test_validate_body_file_valid() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp = NamedTempFile::new()?;
    writeln!(
        tmp,
        "# EPIC-001: Title\n\n## Overview\nValid content here.\n```rust\nfn main() {{}}\n```"
    )?;

    let content = validate_body_file(tmp.path())?;
    assert!(content.contains("Valid content here."));
    Ok(())
}

#[test]
fn test_validate_body_file_empty() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp = NamedTempFile::new()?;
    writeln!(tmp, "   \n  ")?;

    assert!(matches!(
        validate_body_file(tmp.path()),
        Err(ReconcileError::PreconditionFailed(_))
    ));
    Ok(())
}

#[test]
fn test_validate_body_file_placeholders() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp = NamedTempFile::new()?;
    writeln!(tmp, "# Title\n\n- [ ] (To be created)")?;

    assert!(matches!(
        validate_body_file(tmp.path()),
        Err(ReconcileError::PreconditionFailed(_))
    ));
    Ok(())
}

#[test]
fn test_validate_body_file_unbalanced_fences() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp = NamedTempFile::new()?;
    writeln!(tmp, "# Title\n\n```rust\nfn unclosed() {{")?;

    assert!(matches!(
        validate_body_file(tmp.path()),
        Err(ReconcileError::PreconditionFailed(_))
    ));
    Ok(())
}

#[test]
fn test_validate_body_file_tilde_fences() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp_no_heading = NamedTempFile::new()?;
    writeln!(
        tmp_no_heading,
        "~~~python\n# comment inside tilde fence\n~~~"
    )?;
    assert!(matches!(
        validate_body_file(tmp_no_heading.path()),
        Err(ReconcileError::PreconditionFailed(_))
    ));

    let mut tmp_unclosed = NamedTempFile::new()?;
    writeln!(tmp_unclosed, "# Title\n\n~~~\nunclosed tilde fence")?;
    assert!(matches!(
        validate_body_file(tmp_unclosed.path()),
        Err(ReconcileError::PreconditionFailed(_))
    ));

    let mut tmp_valid = NamedTempFile::new()?;
    writeln!(
        tmp_valid,
        "# Real Heading\n\n~~~rust\n# inside tilde fence\nfn main() {{}}\n~~~"
    )?;
    assert!(validate_body_file(tmp_valid.path()).is_ok());
    Ok(())
}

#[test]
fn test_execute_create_issue_creates_new() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp = NamedTempFile::new()?;
    writeln!(tmp, "# US-001: Login\n\n## Story\nUser logs in.")?;

    let tracker = MockTracker::new(vec![]);
    let args = CreateIssueArgs {
        body_file: Some(tmp.path().to_string_lossy().to_string()),
        label: Some("type:user-story".to_string()),
        title: Some("US-001: Login".to_string()),
        repo: None,
        reopen_with_analysis: None,
        provider: "github".to_string(),
        positional: vec![],
    };

    let num = execute_create_issue_with_tracker(&tracker, &args)?;
    assert_eq!(num, 1);
    assert_eq!(tracker.issues.borrow().len(), 1);
    Ok(())
}

#[test]
fn test_execute_create_issue_blocks_open_duplicate() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp = NamedTempFile::new()?;
    writeln!(tmp, "# US-001: Login\n\n## Story\nUser logs in.")?;

    let tracker = MockTracker::new(vec![Issue {
        number: 42,
        title: "US-001: Login".to_string(),
        state: "OPEN".to_string(),
        labels: vec!["type:user-story".to_string()],
        body: "Existing".to_string(),
    }]);

    let args = CreateIssueArgs {
        body_file: Some(tmp.path().to_string_lossy().to_string()),
        label: Some("type:user-story".to_string()),
        title: Some("US-001: Login".to_string()),
        repo: None,
        reopen_with_analysis: None,
        provider: "github".to_string(),
        positional: vec![],
    };

    let res = execute_create_issue_with_tracker(&tracker, &args);
    assert!(matches!(res, Err(ReconcileError::PreconditionFailed(msg)) if msg.contains("#42")));
    Ok(())
}

#[test]
fn test_execute_create_issue_blocks_closed_without_reopen_flag() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp = NamedTempFile::new()?;
    writeln!(tmp, "# US-001: Login\n\n## Story\nUser logs in.")?;

    let tracker = MockTracker::new(vec![Issue {
        number: 42,
        title: "US-001: Login".to_string(),
        state: "CLOSED".to_string(),
        labels: vec!["type:user-story".to_string()],
        body: "Existing".to_string(),
    }]);

    let args = CreateIssueArgs {
        body_file: Some(tmp.path().to_string_lossy().to_string()),
        label: Some("type:user-story".to_string()),
        title: Some("US-001: Login".to_string()),
        repo: None,
        reopen_with_analysis: None,
        provider: "github".to_string(),
        positional: vec![],
    };

    let res = execute_create_issue_with_tracker(&tracker, &args);
    assert!(matches!(res, Err(ReconcileError::PreconditionFailed(msg)) if msg.contains("CLOSED")));
    Ok(())
}

#[test]
fn test_execute_create_issue_reopens_closed_with_valid_analysis() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp_body = NamedTempFile::new()?;
    writeln!(tmp_body, "# US-001: Login\n\n## Story\nUser logs in.")?;

    let mut tmp_analysis = NamedTempFile::new()?;
    writeln!(tmp_analysis, "# Root Cause Analysis\n\nRegression in auth.")?;

    let tracker = MockTracker::new(vec![Issue {
        number: 42,
        title: "US-001: Login".to_string(),
        state: "CLOSED".to_string(),
        labels: vec!["type:user-story".to_string()],
        body: "Existing".to_string(),
    }]);

    let args = CreateIssueArgs {
        body_file: Some(tmp_body.path().to_string_lossy().to_string()),
        label: Some("type:user-story".to_string()),
        title: Some("US-001: Login".to_string()),
        repo: None,
        reopen_with_analysis: Some(tmp_analysis.path().to_string_lossy().to_string()),
        provider: "github".to_string(),
        positional: vec![],
    };

    let num = execute_create_issue_with_tracker(&tracker, &args)?;
    assert_eq!(num, 42);
    let issues = tracker.issues.borrow();
    let reopened = issues.first().ok_or("missing issue")?;
    assert_eq!(reopened.state, "OPEN");
    assert!(reopened.body.contains("Root Cause Analysis"));
    Ok(())
}
