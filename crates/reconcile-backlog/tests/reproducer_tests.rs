//! Regression and reproducer integration tests (Issues #527, #528, #529, #530, #531, #749).
//! Realises: [REQ-RECONCILE-BACKLOG-TESTS]

use reconcile_backlog::checklist::{
    contains_unpopulated_placeholders, reconcile_checklists_in_content,
};
use reconcile_backlog::create_issue::{execute_create_issue_with_tracker, CreateIssueArgs};
use reconcile_backlog::tracker::{Issue, IssueIndex, IssueTracker};
use reconcile_backlog::{
    project_body_for_tracker_with_prefix, run_reconciliation, ReconcileError, ReconcileOptions,
};
use std::cell::RefCell;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static REPRO_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn tempdir() -> Result<TempDirGuard, std::io::Error> {
    let seq = REPRO_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "reconcile_repro_dir_{}_{}",
        std::process::id(),
        seq
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path)?;
    Ok(TempDirGuard { path })
}

struct NamedTempFile {
    path: PathBuf,
    file: fs::File,
}

impl NamedTempFile {
    fn new() -> Result<Self, std::io::Error> {
        let seq = REPRO_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "reconcile_repro_file_{}_{}.md",
            std::process::id(),
            seq
        ));
        let file = fs::File::create(&path)?;
        Ok(Self { path, file })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Write for NamedTempFile {
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

struct ReproducerMockTracker {
    issues: RefCell<Vec<Issue>>,
    edits: RefCell<Vec<(u64, String)>>,
    next_id: RefCell<u64>,
}

impl ReproducerMockTracker {
    fn new(initial: Vec<Issue>) -> Self {
        let max_id = initial.iter().map(|i| i.number).max().unwrap_or(0);
        Self {
            issues: RefCell::new(initial),
            edits: RefCell::new(Vec::new()),
            next_id: RefCell::new(max_id + 1),
        }
    }
}

impl IssueTracker for ReproducerMockTracker {
    fn fetch_all_issues(&self, _repo: Option<&str>) -> Result<Vec<Issue>, ReconcileError> {
        Ok(self.issues.borrow().clone())
    }

    fn edit_issue_body(
        &self,
        number: u64,
        new_body: &str,
        _repo: Option<&str>,
    ) -> Result<(), ReconcileError> {
        self.edits.borrow_mut().push((number, new_body.to_string()));
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
fn test_issue_527_false_positive_epic_feature_collision() {
    let issues = vec![
        Issue {
            number: 100,
            title: "EPIC-001: System Security".to_string(),
            state: "OPEN".to_string(),
            labels: vec!["type:epic".to_string()],
            body: String::new(),
        },
        Issue {
            number: 200,
            title: "FEAT-001: System Security".to_string(),
            state: "OPEN".to_string(),
            labels: vec!["type:feature".to_string()],
            body: String::new(),
        },
    ];
    let index = IssueIndex::from_issues(issues);

    assert_eq!(
        index.find_issue_for_spec("EPIC-001: System Security", Some("Epic")),
        Some(100)
    );
    assert_eq!(
        index.find_issue_for_spec("FEAT-001: System Security", Some("Feature")),
        Some(200)
    );
}

#[test]
fn test_issue_528_ordinal_guard_prevents_cross_matching() {
    let issues = vec![Issue {
        number: 10,
        title: "FEAT-001: Core Engine".to_string(),
        state: "OPEN".to_string(),
        labels: vec!["type:feature".to_string()],
        body: String::new(),
    }];
    let index = IssueIndex::from_issues(issues);

    assert_eq!(
        index.find_issue_for_spec("FEAT-002: Core Engine", Some("Feature")),
        None
    );
    assert_eq!(
        index.find_issue_for_spec("FEAT-1: Core Engine", Some("Feature")),
        Some(10)
    );
}

#[test]
fn test_issue_529_gitlab_scoped_resolved_labels_tick_checklists() -> Result<(), Box<dyn std::error::Error>> {
    let issues = vec![Issue {
        number: 55,
        title: "US-001: Telemetry Export".to_string(),
        state: "OPEN".to_string(),
        labels: vec![
            "type::user_story".to_string(),
            "status::fixed-resolved".to_string(),
        ],
        body: String::new(),
    }];
    let index = IssueIndex::from_issues(issues);

    let md = "## User Stories\n- [ ] [US-001: Telemetry Export](#55)\n";
    let (reconciled, count) =
        reconcile_checklists_in_content(md, &index, Some("Feature"))?;
    assert_eq!(count, 1);
    assert!(reconciled.contains("- [x] [US-001: Telemetry Export](#55)"));
    Ok(())
}

#[test]
fn test_issue_530_placeholder_purging_and_create_issue_gate() -> Result<(), Box<dyn std::error::Error>> {
    assert!(contains_unpopulated_placeholders(
        "## Stories\n- [ ] (To be created)\n"
    ));

    let mut tmp = NamedTempFile::new()?;
    writeln!(tmp, "# FEAT-001: Test\n\n## Stories\n- [ ] (To be created)")?;

    let tracker = ReproducerMockTracker::new(vec![]);
    let args = CreateIssueArgs {
        body_file: Some(tmp.path().to_string_lossy().to_string()),
        label: Some("type:feature".to_string()),
        title: Some("FEAT-001: Test".to_string()),
        repo: None,
        reopen_with_analysis: None,
        provider: "github".to_string(),
        positional: vec![],
    };

    let res = execute_create_issue_with_tracker(&tracker, &args);
    assert!(matches!(res, Err(ReconcileError::PreconditionFailed(_))));
    Ok(())
}

#[test]
fn test_issue_531_remote_body_sync_idempotency() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let epics_dir = dir.path().join("docs/epics");
    fs::create_dir_all(&epics_dir)?;

    let epic_path = epics_dir.join("epic-001.md");
    fs::write(
        &epic_path,
        "# EPIC-001: Idempotent Sync\n\n| Field | Value |\n|---|---|\n| **Type** | `Epic` |\n| **GitHub Issue** | `[#10](https://github.com/org/repo/issues/10)` |\n\n## Overview\nStable body.\n",
    )?;

    let tracker = ReproducerMockTracker::new(vec![Issue {
        number: 10,
        title: "EPIC-001: Idempotent Sync".to_string(),
        state: "OPEN".to_string(),
        labels: vec!["type:epic".to_string()],
        body: "Outdated body".to_string(),
    }]);

    let options = ReconcileOptions {
        workspace_root: dir.path().to_path_buf(),
        offline: false,
        dry_run: false,
        provider: "github".to_string(),
        sync_bodies: true,
        repo: Some("org/repo".to_string()),
        default_branch: Some("main".to_string()),
    };

    let summary1 = run_reconciliation(&tracker, &options)?;
    assert_eq!(summary1.remote_issues_synced, 1);

    let summary2 = run_reconciliation(&tracker, &options)?;
    assert_eq!(summary2.remote_issues_synced, 0);
    Ok(())
}

#[test]
fn test_issue_749_link_projection_and_upstream_prefix() {
    let md = "\
# FEAT-001: Link Projection

| **Type** | `Feature` |

## Traceability
See [Story](docs/user-stories/us-001.md) and [Schema](schema/sys.sysml) and [External](https://ietf.org).
";
    let projected = project_body_for_tracker_with_prefix(
        md,
        "docs/features/feat-001.md",
        Some("org/repo"),
        "main",
        Some("sub/dir"),
    );

    assert!(!projected.contains("| **Type** |"));
    assert!(projected.contains(
        "[Story](https://github.com/org/repo/blob/main/sub/dir/docs/user-stories/us-001.md)"
    ));
    assert!(
        projected.contains("[Schema](https://github.com/org/repo/blob/main/sub/dir/schema/sys.sysml)")
    );
    assert!(projected.contains("[External](https://ietf.org)"));
}
