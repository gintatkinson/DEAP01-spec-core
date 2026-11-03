//! Regression and reproducer integration tests (Issues #750, #751, #791, #792).
//! Realises: [REQ-RECONCILE-BACKLOG-TESTS]

use reconcile_backlog::checklist::{
    clean_unpopulated_placeholders_counted, synthesize_and_reconcile_hierarchy, DiscoveredSpec,
};
use reconcile_backlog::frontmatter::{parse_frontmatter, update_frontmatter_fields_batch};
use reconcile_backlog::tracker::{Issue, IssueTracker};
use reconcile_backlog::{run_reconciliation, ReconcileError, ReconcileOptions};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static REPRO_HIER_COUNTER: AtomicU64 = AtomicU64::new(0);

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
    let seq = REPRO_HIER_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "reconcile_repro_hier_dir_{}_{}",
        std::process::id(),
        seq
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path)?;
    Ok(TempDirGuard { path })
}

struct HierarchyMockTracker {
    issues: RefCell<Vec<Issue>>,
    next_id: RefCell<u64>,
}

impl HierarchyMockTracker {
    fn new(initial: Vec<Issue>) -> Self {
        let max_id = initial.iter().map(|i| i.number).max().unwrap_or(0);
        Self {
            issues: RefCell::new(initial),
            next_id: RefCell::new(max_id + 1),
        }
    }
}

impl IssueTracker for HierarchyMockTracker {
    fn fetch_all_issues(&self, _repo: Option<&str>) -> Result<Vec<Issue>, ReconcileError> {
        Ok(self.issues.borrow().clone())
    }

    fn edit_issue_body(
        &self,
        number: u64,
        new_body: &str,
        _repo: Option<&str>,
    ) -> Result<(), ReconcileError> {
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
fn test_issue_750_bottom_up_hierarchy_synthesis() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let epics_dir = dir.path().join("docs/epics");
    let feats_dir = dir.path().join("docs/features");
    fs::create_dir_all(&epics_dir)?;
    fs::create_dir_all(&feats_dir)?;

    let epic_path = epics_dir.join("epic-001.md");
    fs::write(&epic_path, "# EPIC-001: Core\n\n## Overview\nCore.\n")?;

    let feat_path = feats_dir.join("feat-001.md");
    fs::write(&feat_path, "# FEAT-001: Child\n\n## Overview\nChild.\n")?;

    let specs = vec![
        DiscoveredSpec {
            path: epic_path.clone(),
            rel_path: "docs/epics/epic-001.md".to_string(),
            spec_type: "Epic".to_string(),
            document_id: Some("EPIC-001".to_string()),
            title: "EPIC-001: Core".to_string(),
            issue_number: Some(1),
            is_resolved: false,
            parent_epic: None,
            parent_feature: None,
        },
        DiscoveredSpec {
            path: feat_path,
            rel_path: "docs/features/feat-001.md".to_string(),
            spec_type: "Feature".to_string(),
            document_id: Some("FEAT-001".to_string()),
            title: "FEAT-001: Child".to_string(),
            issue_number: Some(2),
            is_resolved: true,
            parent_epic: Some("EPIC-001".to_string()),
            parent_feature: None,
        },
    ];

    let changes =
        synthesize_and_reconcile_hierarchy(&specs, Some("org/repo"), "main", false)?;
    assert_eq!(changes, 1);

    let epic_md = fs::read_to_string(&epic_path)?;
    assert!(epic_md.contains("## Features"));
    assert!(epic_md.contains("- [x] [FEAT-001: Child](https://github.com/org/repo/issues/2) -- [docs/features/feat-001.md](https://github.com/org/repo/blob/main/docs/features/feat-001.md)"));
    Ok(())
}

#[test]
fn test_issue_751_bottom_up_registration_and_publish_gate() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let epics_dir = dir.path().join("docs/epics");
    let feats_dir = dir.path().join("docs/features");
    fs::create_dir_all(&epics_dir)?;
    fs::create_dir_all(&feats_dir)?;

    fs::write(
        epics_dir.join("epic-001.md"),
        "# EPIC-001: Auto Register\n\n| **Type** | `Epic` |\n| **GitHub Issue** | Pending |\n\n## Overview\nEpic.\n",
    )?;
    fs::write(
        feats_dir.join("feat-001.md"),
        "# FEAT-001: Auto Child\n\n| **Type** | `Feature` |\n| **Parent Epic** | EPIC-001 |\n| **GitHub Issue** | Pending |\n\n## Overview\nFeat.\n",
    )?;

    let tracker = HierarchyMockTracker::new(vec![]);
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
    assert_eq!(summary.issues_registered, 2);

    let created = tracker.issues.borrow();
    assert_eq!(created.len(), 2);
    let first = created.first().ok_or("missing first created issue")?;
    let second = created.get(1).ok_or("missing second created issue")?;
    assert_eq!(first.title, "FEAT-001: Auto Child");
    assert_eq!(second.title, "EPIC-001: Auto Register");
    assert!(second
        .body
        .contains("- [ ] [FEAT-001: Auto Child](https://github.com/org/repo/issues/1)"));
    Ok(())
}

#[test]
fn test_issue_791_tilde_fences_ignored_across_all_scanners() -> Result<(), Box<dyn std::error::Error>> {
    let md = "\
~~~markdown
# Fake H1
| **Title** | Fake |
- [ ] (To be created)
~~~

# Real H1

| **Type** | `Epic` |

## Features
- [ ] (To be created)
";
    let fm = parse_frontmatter(md)?;
    assert_eq!(fm.title.as_deref(), Some("Real H1"));

    let (cleaned, count) = clean_unpopulated_placeholders_counted(md);
    assert_eq!(count, 1);
    assert!(cleaned.contains("~~~markdown\n# Fake H1\n| **Title** | Fake |\n- [ ] (To be created)\n~~~"));
    Ok(())
}

#[test]
fn test_issue_792_batch_frontmatter_updates_and_crlf_preservation() -> Result<(), Box<dyn std::error::Error>> {
    let md = "# FEAT-001: Batch\r\n\r\n| Field | Value |\r\n|---|---|\r\n| **Type** | `Feature` |\r\n| **Status** | `Draft` |\r\n";
    let updated = update_frontmatter_fields_batch(
        md,
        &[("Status", "`Approved`"), ("GitHub Issue", "`#88`")],
    )?;

    assert!(updated.contains("\r\n"));
    assert!(updated.contains("| **Status** | `Approved` |\r\n"));
    assert!(updated.contains("| **GitHub Issue** | `#88` |\r\n"));
    Ok(())
}
