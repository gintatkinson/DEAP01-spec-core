//! Live GitHub CLI (`gh`) tracker adapter and `serde_json` issue deserialization models.
//! Realises: [REQ-RECONCILE-BACKLOG-TRACKER, Issue Tracker as Canonical SSOT]

use crate::tracker::{Issue, IssueTracker};
use crate::ReconcileError;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// [DOC-TIER: A] Live GitHub tracker backed by the `gh` CLI and `serde_json` payloads.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-GITHUBTRACKER]
/// Contract: Queries and mutates GitHub issues deterministically via `gh issue` subcommands.
/// Bounds & Faults: Time O(I), Space O(I) over fetched issues; returns `ReconcileError::TrackerError` on CLI failure.
#[derive(Debug, Default)]
pub struct GitHubTracker;

/// Typed `serde_json` model for a raw `gh issue list --json` label object.
/// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-PARSE-GH-JSON]
#[derive(Deserialize, Serialize)]
struct GhRawLabel {
    name: String,
}

/// Typed `serde_json` model for a raw `gh issue list --json` issue entry.
/// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-PARSE-GH-JSON]
#[derive(Deserialize, Serialize)]
struct GhRawIssue {
    number: u64,
    title: String,
    state: String,
    #[serde(default)]
    labels: Vec<GhRawLabel>,
    #[serde(default)]
    body: Option<String>,
}

impl GitHubTracker {
    /// [DOC-TIER: A] Constructs a new `GitHubTracker`.
    /// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-NEW-1]
    /// Contract: Stateless adapter construction.
    /// Bounds & Faults: Time O(1), Space O(1).
    pub fn new() -> Self {
        Self
    }

    /// Verifies that the `gh` CLI binary is installed and reachable in `PATH`.
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-CHECK-GH-AVAILABLE]
    fn check_gh_available() -> Result<(), ReconcileError> {
        let output = Command::new("gh").arg("--version").output().map_err(|e| {
            ReconcileError::TrackerError(format!(
                "GitHub CLI ('gh') is not installed or not found in PATH: {}",
                e
            ))
        })?;

        if !output.status.success() {
            return Err(ReconcileError::TrackerError(
                "GitHub CLI ('gh --version') returned non-zero exit status".to_string(),
            ));
        }
        Ok(())
    }

    /// [DOC-TIER: A] Parses raw JSON output from `gh issue list --json number,title,state,labels,body`.
    /// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-PARSE-GH-JSON]
    /// Contract: Deserializes `GhRawIssue` array via `serde_json` into canonical `Vec<Issue>`.
    /// Bounds & Faults: Time O(B), Space O(I) over JSON byte size B and issue count I.
    pub fn parse_gh_json(json_str: &str) -> Result<Vec<Issue>, ReconcileError> {
        let trimmed = json_str.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        let raw_issues: Vec<GhRawIssue> = serde_json::from_str(trimmed)?;
        let issues = raw_issues
            .into_iter()
            .map(|r| Issue {
                number: r.number,
                title: r.title,
                state: r.state,
                labels: r.labels.into_iter().map(|l| l.name).collect(),
                body: r.body.unwrap_or_default(),
            })
            .collect();

        Ok(issues)
    }
}

impl IssueTracker for GitHubTracker {
    /// Fetches all issues (`--state all --limit 1000`) from GitHub via `gh`.
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-FETCH-ALL-ISSUES-1]
    fn fetch_all_issues(&self, repo: Option<&str>) -> Result<Vec<Issue>, ReconcileError> {
        Self::check_gh_available()?;

        let mut cmd = Command::new("gh");
        cmd.args([
            "issue",
            "list",
            "--state",
            "all",
            "--limit",
            "1000",
            "--json",
            "number,title,state,labels,body",
        ]);
        if let Some(r) = repo {
            if !r.trim().is_empty() {
                cmd.args(["--repo", r.trim()]);
            }
        }

        let output = cmd.output().map_err(|e| {
            ReconcileError::TrackerError(format!("Failed to execute 'gh issue list': {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ReconcileError::TrackerError(format!(
                "'gh issue list' failed with status {}: {}",
                output.status,
                stderr.trim()
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_gh_json(&stdout)
    }

    /// Updates a remote GitHub issue body via `gh issue edit --body-file`.
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-EDIT-ISSUE-BODY-1]
    fn edit_issue_body(
        &self,
        number: u64,
        new_body: &str,
        repo: Option<&str>,
    ) -> Result<(), ReconcileError> {
        Self::check_gh_available()?;
        let tmp = tempfile_named(new_body)?;

        let mut cmd = Command::new("gh");
        cmd.args(["issue", "edit", &number.to_string(), "--body-file"]);
        cmd.arg(tmp.path());
        if let Some(r) = repo {
            if !r.trim().is_empty() {
                cmd.args(["--repo", r.trim()]);
            }
        }

        let output = cmd.output().map_err(|e| {
            ReconcileError::TrackerError(format!(
                "Failed to execute 'gh issue edit {}': {}",
                number, e
            ))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ReconcileError::TrackerError(format!(
                "'gh issue edit {}' failed: {}",
                number,
                stderr.trim()
            )));
        }
        Ok(())
    }

    /// Updates a remote GitHub issue title via `gh issue edit --title`.
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-EDIT-ISSUE-BODY-1]
    fn edit_issue_title(
        &self,
        number: u64,
        new_title: &str,
        repo: Option<&str>,
    ) -> Result<(), ReconcileError> {
        Self::check_gh_available()?;

        let mut cmd = Command::new("gh");
        cmd.args(["issue", "edit", &number.to_string(), "--title", new_title]);
        if let Some(r) = repo {
            if !r.trim().is_empty() {
                cmd.args(["--repo", r.trim()]);
            }
        }

        let output = cmd.output().map_err(|e| {
            ReconcileError::TrackerError(format!(
                "Failed to execute 'gh issue edit {} --title': {}",
                number, e
            ))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ReconcileError::TrackerError(format!(
                "'gh issue edit {} --title' failed: {}",
                number,
                stderr.trim()
            )));
        }
        Ok(())
    }

    /// Creates a new GitHub issue via `gh issue create` and parses the returned issue number.
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-CREATE-ISSUE-1]
    fn create_issue(
        &self,
        title: &str,
        body: &str,
        labels: &[String],
        repo: Option<&str>,
    ) -> Result<u64, ReconcileError> {
        Self::check_gh_available()?;
        let tmp = tempfile_named(body)?;

        let mut cmd = Command::new("gh");
        cmd.args(["issue", "create", "--title", title, "--body-file"]);
        cmd.arg(tmp.path());
        for label in labels {
            if !label.trim().is_empty() {
                cmd.args(["--label", label.trim()]);
            }
        }
        if let Some(r) = repo {
            if !r.trim().is_empty() {
                cmd.args(["--repo", r.trim()]);
            }
        }

        let output = cmd.output().map_err(|e| {
            ReconcileError::TrackerError(format!("Failed to execute 'gh issue create': {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ReconcileError::TrackerError(format!(
                "'gh issue create' failed: {}",
                stderr.trim()
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().rev() {
            if let Some(num) = crate::frontmatter::extract_issue_number(line.trim()) {
                return Ok(num);
            }
        }

        Err(ReconcileError::TrackerError(format!(
            "Could not parse created issue number from 'gh issue create' output: {}",
            stdout.trim()
        )))
    }

    /// Reopens a closed GitHub issue with a root-cause comment via `gh issue reopen`.
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-REOPEN-ISSUE-1]
    fn reopen_issue(
        &self,
        number: u64,
        comment: &str,
        repo: Option<&str>,
    ) -> Result<(), ReconcileError> {
        Self::check_gh_available()?;

        let mut cmd = Command::new("gh");
        cmd.args([
            "issue",
            "reopen",
            &number.to_string(),
            "--comment",
            comment,
        ]);
        if let Some(r) = repo {
            if !r.trim().is_empty() {
                cmd.args(["--repo", r.trim()]);
            }
        }

        let output = cmd.output().map_err(|e| {
            ReconcileError::TrackerError(format!(
                "Failed to execute 'gh issue reopen {}': {}",
                number, e
            ))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ReconcileError::TrackerError(format!(
                "'gh issue reopen {}' failed: {}",
                number,
                stderr.trim()
            )));
        }
        Ok(())
    }
}

/// RAII temporary file guard for passing Markdown bodies to `gh --body-file`.
/// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-TEMPFILENAMED]
struct TempFileNamed {
    path_buf: std::path::PathBuf,
}

impl TempFileNamed {
    /// Returns the filesystem path of the temporary file.
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-PATH]
    fn path(&self) -> &std::path::Path {
        &self.path_buf
    }
}

impl Drop for TempFileNamed {
    /// Removes the temporary file on drop.
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-DROP]
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path_buf);
    }
}

/// Writes `content` to a temporary file and flushes it for CLI consumption.
/// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-TEMPFILE-NAMED]
fn tempfile_named(content: &str) -> Result<TempFileNamed, ReconcileError> {
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path_buf = std::env::temp_dir().join(format!(
        "reconcile_backlog_body_{}_{}.md",
        std::process::id(),
        seq
    ));
    let mut file = std::fs::File::create(&path_buf)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()?;
    Ok(TempFileNamed { path_buf })
}
