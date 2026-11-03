//! Unit tests for repository URL parsing, tracker body projection, atomic writes, and full workspace reconciliation.
//! Realises: [REQ-RECONCILE-BACKLOG-CORE]

use super::*;
use crate::tracker::{Issue, IssueTracker};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(crate) fn tempdir() -> Result<TempDirGuard, std::io::Error> {
    let seq = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "reconcile_backlog_test_dir_{}_{}",
        std::process::id(),
        seq
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path)?;
    Ok(TempDirGuard { path })
}

pub(crate) struct NamedTempFile {
    path: PathBuf,
    file: fs::File,
}

impl NamedTempFile {
    pub fn new() -> Result<Self, std::io::Error> {
        let seq = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "reconcile_backlog_test_file_{}_{}.md",
            std::process::id(),
            seq
        ));
        let file = fs::File::create(&path)?;
        Ok(Self { path, file })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl std::io::Write for NamedTempFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

impl Drop for NamedTempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub(crate) struct MockTracker {
    pub issues: RefCell<Vec<Issue>>,
    pub edited_bodies: RefCell<Vec<(u64, String)>>,
    pub next_id: RefCell<u64>,
}

impl MockTracker {
    pub fn new(initial: Vec<Issue>) -> Self {
        let max_id = initial.iter().map(|i| i.number).max().unwrap_or(0);
        Self {
            issues: RefCell::new(initial),
            edited_bodies: RefCell::new(Vec::new()),
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
        self.edited_bodies
            .borrow_mut()
            .push((number, new_body.to_string()));
        if let Some(issue) = self.issues.borrow_mut().iter_mut().find(|i| i.number == number) {
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
        if let Some(issue) = self.issues.borrow_mut().iter_mut().find(|i| i.number == number) {
            issue.state = "OPEN".to_string();
            issue.body.push_str(&format!("\n{}", comment));
        }
        Ok(())
    }
}

#[test]
fn test_clean_repo_name_and_parse_remote_url() {
    assert_eq!(
        clean_repo_name("owner/repo.git"),
        Some("owner/repo".to_string())
    );
    assert_eq!(
        clean_repo_name("  /owner/repo/ "),
        Some("owner/repo".to_string())
    );
    assert_eq!(
        parse_repo_from_remote_url("https://github.com/org/project.git"),
        Some("org/project".to_string())
    );
    assert_eq!(
        parse_repo_from_remote_url("git@github.com:org/project.git"),
        Some("org/project".to_string())
    );
}

#[test]
fn test_project_body_for_tracker_strips_frontmatter_and_rewrites_links() {
    let md = "\
# EPIC-001: Core

| Field | Value |
|---|---|
| **Type** | `Epic` |
| **GitHub Issue** | `#10` |

## Overview
See [FEAT-001](docs/features/feat-001.md) and [Sibling](./epic-002.md) and [Schema](schema/domain.sysml).
";
    let projected =
        project_body_for_tracker(md, "docs/epics/epic-001.md", Some("org/repo"), "main");
    assert!(!projected.contains("| **Type** |"));
    assert!(projected.contains("[FEAT-001](https://github.com/org/repo/blob/main/docs/features/feat-001.md)"));
    assert!(projected.contains("[Sibling](https://github.com/org/repo/blob/main/docs/epics/epic-002.md)"));
    assert!(projected.contains("[Schema](https://github.com/org/repo/blob/main/schema/domain.sysml)"));
}

#[test]
fn test_project_body_for_tracker_preserves_external_and_anchor_links() {
    let md = "\
# FEAT-001: Auth

## Links
- [External](https://example.com/spec)
- [Section](#overview)
- [Parent](../epics/epic-001.md#scope)
";
    let projected =
        project_body_for_tracker(md, "docs/features/feat-001.md", Some("org/repo"), "develop");
    assert!(projected.contains("[External](https://example.com/spec)"));
    assert!(projected.contains("[Section](#overview)"));
    assert!(projected.contains("[Parent](https://github.com/org/repo/blob/develop/docs/epics/epic-001.md#scope)"));
}

#[test]
fn test_project_body_for_tracker_upstream_prefix() {
    let md = "\
# FEAT-001: Nested

## Links
See [Story](docs/user-stories/us-001.md).
";
    let projected = project_body_for_tracker_with_prefix(
        md,
        "docs/features/feat-001.md",
        Some("org/monorepo"),
        "main",
        Some("packages/subproject"),
    );
    assert!(projected.contains("[Story](https://github.com/org/monorepo/blob/main/packages/subproject/docs/user-stories/us-001.md)"));
}

#[test]
fn test_write_atomic() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let target = dir.path().join("atomic.md");
    write_atomic(&target, "Hello atomic world")?;
    assert_eq!(fs::read_to_string(&target)?, "Hello atomic world");
    Ok(())
}

#[test]
fn test_run_reconciliation_updates_frontmatter_and_checklists_and_syncs_body(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let epics_dir = dir.path().join("docs/epics");
    let feats_dir = dir.path().join("docs/features");
    fs::create_dir_all(&epics_dir)?;
    fs::create_dir_all(&feats_dir)?;

    let epic_path = epics_dir.join("epic-001.md");
    fs::write(
        &epic_path,
        "# EPIC-001: Platform\n\n| Field | Value |\n|---|---|\n| **Type** | `Epic` |\n| **GitHub Issue** | Pending |\n\n## Features\n- [ ] (To be created)\n",
    )?;

    let feat_path = feats_dir.join("feat-001.md");
    fs::write(
        &feat_path,
        "# FEAT-001: Login\n\n| Field | Value |\n|---|---|\n| **Type** | `Feature` |\n| **Parent Epic** | EPIC-001 |\n| **GitHub Issue** | `#20` |\n\n## Overview\nLogin feature.\n",
    )?;

    let tracker = MockTracker::new(vec![
        Issue {
            number: 10,
            title: "EPIC-001: Platform".to_string(),
            state: "OPEN".to_string(),
            labels: vec!["type:epic".to_string()],
            body: "Old epic body".to_string(),
        },
        Issue {
            number: 20,
            title: "FEAT-001: Login".to_string(),
            state: "CLOSED".to_string(),
            labels: vec!["type:feature".to_string()],
            body: "Old feat body".to_string(),
        },
    ]);

    let options = ReconcileOptions {
        workspace_root: dir.path().to_path_buf(),
        offline: false,
        dry_run: false,
        provider: "github".to_string(),
        sync_bodies: true,
        repo: Some("org/repo".to_string()),
        default_branch: Some("main".to_string()),
    };

    let summary = run_reconciliation(&tracker, &options)?;
    assert_eq!(summary.total_specs_scanned, 2);
    assert!(summary.frontmatters_updated >= 1);
    assert!(summary.remote_issues_synced >= 1);

    let updated_epic = fs::read_to_string(&epic_path)?;
    assert!(updated_epic.contains("[#10](https://github.com/org/repo/issues/10)"));
    assert!(updated_epic.contains("- [x] [FEAT-001: Login](https://github.com/org/repo/issues/20)"));
    Ok(())
}

#[test]
fn test_run_reconciliation_dry_run_makes_no_writes() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let epics_dir = dir.path().join("docs/epics");
    fs::create_dir_all(&epics_dir)?;
    let epic_path = epics_dir.join("epic-001.md");
    let original = "# EPIC-001: Platform\n\n| Field | Value |\n|---|---|\n| **GitHub Issue** | Pending |\n";
    fs::write(&epic_path, original)?;

    let tracker = MockTracker::new(vec![Issue {
        number: 10,
        title: "EPIC-001: Platform".to_string(),
        state: "OPEN".to_string(),
        labels: vec!["type:epic".to_string()],
        body: String::new(),
    }]);

    let options = ReconcileOptions {
        workspace_root: dir.path().to_path_buf(),
        offline: false,
        dry_run: true,
        provider: "github".to_string(),
        sync_bodies: true,
        repo: Some("org/repo".to_string()),
        default_branch: Some("main".to_string()),
    };

    let summary = run_reconciliation(&tracker, &options)?;
    assert_eq!(summary.frontmatters_updated, 1);
    assert_eq!(fs::read_to_string(&epic_path)?, original);
    assert!(tracker.edited_bodies.borrow().is_empty());
    Ok(())
}
