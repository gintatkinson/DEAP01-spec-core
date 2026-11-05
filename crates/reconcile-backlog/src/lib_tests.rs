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

    fn edit_issue_body(&self, number: u64, new_body: &str, _repo: Option<&str>) -> Result<(), ReconcileError> {
        self.edited_bodies.borrow_mut().push((number, new_body.to_string()));
        if let Some(issue) = self.issues.borrow_mut().iter_mut().find(|i| i.number == number) {
            issue.body = new_body.to_string();
        }
        Ok(())
    }

    fn edit_issue_title(&self, number: u64, new_title: &str, _repo: Option<&str>) -> Result<(), ReconcileError> {
        if let Some(issue) = self.issues.borrow_mut().iter_mut().find(|i| i.number == number) {
            issue.title = new_title.to_string();
        }
        Ok(())
    }

    fn create_issue(&self, title: &str, body: &str, labels: &[String], _repo: Option<&str>) -> Result<u64, ReconcileError> {
        let mut next_id = self.next_id.borrow_mut();
        let id = *next_id;
        *next_id += 1;
        self.issues.borrow_mut().push(Issue {
            number: id, title: title.to_string(), state: "OPEN".to_string(), labels: labels.to_vec(), body: body.to_string(),
        });
        Ok(id)
    }

    fn reopen_issue(&self, number: u64, comment: &str, _repo: Option<&str>) -> Result<(), ReconcileError> {
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
        Issue { number: 10, title: "EPIC-001: Platform".to_string(), state: "OPEN".to_string(), labels: vec!["type:epic".to_string()], body: "Old epic body".to_string() },
        Issue { number: 20, title: "FEAT-001: Login".to_string(), state: "CLOSED".to_string(), labels: vec!["type:feature".to_string()], body: "Old feat body".to_string() },
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
        number: 10, title: "EPIC-001: Platform".to_string(), state: "OPEN".to_string(),
        labels: vec!["type:epic".to_string()], body: String::new(),
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

#[test]
fn test_issue_759_reconcile_invalidates_colliding_and_missing_frontmatter_issues(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let epics_dir = dir.path().join("docs/epics");
    let feats_dir = dir.path().join("docs/features");
    fs::create_dir_all(&epics_dir)?;
    fs::create_dir_all(&feats_dir)?;

    let epic_path = epics_dir.join("EPIC-07-Blueprint.md");
    fs::write(
        &epic_path,
        "| Attribute | Specification Detail |\n| :--- | :--- |\n| **Issue ID** | #732 |\n| **Title** | Epic 07: BP 07 Self Hosting 4 Level Modular Multi File Code Generator Blueprint |\n| **Type** | epic |\n\n# Epic 07: BP 07 Self Hosting 4 Level Modular Multi File Code Generator Blueprint\n\n## 2. Requirements & Checklist\n- [ ] #737 - [Feature 31: [BP 07 Self Hosting 4 Level Modular Multi File Code Generator Blueprint] FourLevelAstPartitioner](../features/FEAT-31-FourLevelAstPartitioner.md) (partitioner)\n- [ ] #742 - [Feature 36: [BP 07 Self Hosting 4 Level Modular Multi File Code Generator Blueprint] DagVerifier](../features/FEAT-36-DagVerifier.md) (verifier)\n",
    )?;
    let feat31_path = feats_dir.join("FEAT-31-FourLevelAstPartitioner.md");
    fs::write(
        &feat31_path,
        "| Attribute | Specification Detail |\n| :--- | :--- |\n| **Issue ID** | #737 |\n| **Title** | Feature 31: [BP 07 Self Hosting 4 Level Modular Multi File Code Generator Blueprint] FourLevelAstPartitioner |\n| **Type** | feature |\n| **Parent Epic** | [EPIC-07-Blueprint](../epics/EPIC-07-Blueprint.md) |\n\n# Feature 31: [BP 07 Self Hosting 4 Level Modular Multi File Code Generator Blueprint] FourLevelAstPartitioner\n\n## Overview\nPartitioner.\n",
    )?;
    let feat36_path = feats_dir.join("FEAT-36-DagVerifier.md");
    fs::write(
        &feat36_path,
        "| Attribute | Specification Detail |\n| :--- | :--- |\n| **Issue ID** | #742 |\n| **Title** | Feature 36: [BP 07 Self Hosting 4 Level Modular Multi File Code Generator Blueprint] DagVerifier |\n| **Type** | feature |\n| **Parent Epic** | [EPIC-07-Blueprint](../epics/EPIC-07-Blueprint.md) |\n\n# Feature 36: [BP 07 Self Hosting 4 Level Modular Multi File Code Generator Blueprint] DagVerifier\n\n## Overview\nVerifier.\n",
    )?;

    let tracker = MockTracker::new(vec![
        Issue { number: 732, title: "[AUDIT] [parser.rs]: Unchecked slice index".to_string(), state: "OPEN".to_string(), labels: vec!["bug".to_string()], body: "Original audit 732 body".to_string() },
        Issue { number: 737, title: "[AUDIT] [driver.rs]: Resolved audit defect".to_string(), state: "CLOSED".to_string(), labels: vec!["bug".to_string(), "status:fixed-resolved".to_string()], body: "Original audit 737 body".to_string() },
        Issue { number: 742, title: "BP-07: Self-Hosting 4-Level Modular Multi-File Code Generator Blueprint".to_string(), state: "OPEN".to_string(), labels: vec!["enhancement".to_string()], body: "Original BP-07 tracker body".to_string() },
    ]);
    *tracker.next_id.borrow_mut() = 761;

    let options = ReconcileOptions {
        workspace_root: dir.path().to_path_buf(), offline: false, dry_run: false,
        provider: "github".to_string(), sync_bodies: true,
        repo: Some("org/repo".to_string()), default_branch: Some("main".to_string()),
    };

    let mut summary = ReconcileSummary::default();
    let index = IssueIndex::from_issues(tracker.fetch_all_issues(Some("org/repo"))?);
    let discovered = discover_and_reconcile_frontmatters(
        &[epic_path.clone(), feat31_path.clone(), feat36_path.clone()],
        &options, Some("org/repo"), &index, &mut summary,
    )?;
    assert!(discovered.iter().all(|d| d.issue_number.is_none()));
    assert!(fs::read_to_string(&epic_path)?.contains("| **Issue ID** | TBD |"));

    let summary = run_reconciliation(&tracker, &options)?;
    assert_eq!(summary.issues_registered, 3);

    let epic_after = fs::read_to_string(&epic_path)?;
    assert!(epic_after.contains("| **Issue ID** | [#763](https://github.com/org/repo/issues/763) |"));
    assert!(!epic_after.contains("#732"));
    assert!(epic_after.contains("- [ ] #761 - [Feature 31:"));
    assert!(epic_after.contains("- [ ] #762 - [Feature 36:"));
    assert!(!epic_after.contains("#737 -"));
    assert!(!epic_after.contains("#742 -"));

    let issues = tracker.issues.borrow();
    let i732 = issues.iter().find(|i| i.number == 732).ok_or("missing 732")?;
    assert_eq!(i732.body, "Original audit 732 body");
    Ok(())
}
