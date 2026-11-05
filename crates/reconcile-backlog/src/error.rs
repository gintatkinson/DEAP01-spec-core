//! Core error enum, reconciliation options/summary models, atomic file persistence, and spec discovery.
//! Realises: [REQ-RECONCILE-BACKLOG-CORE, Issue Tracker as Canonical SSOT, Zero Fallback Placeholders]

use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// [DOC-TIER: A] Canonical error type for all `reconcile-backlog` operations.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-RECONCILEERROR]
/// Contract: Distinguishes I/O, tracker transport, parse, precondition, and publish-gate failures.
/// Bounds & Faults: Time O(1), Space O(M) over diagnostic message length M.
#[derive(Debug)]
pub enum ReconcileError {
    /// Filesystem I/O failure
    Io(std::io::Error),
    /// Issue tracker CLI or transport error
    TrackerError(String),
    /// Markdown or JSON parse error
    ParseError(String),
    /// Precondition check failed (e.g. duplicate issue, unpopulated placeholders)
    PreconditionFailed(String),
    /// Publish gate integrity violation (e.g. unresolved child issue reference in checklist)
    PublishGateViolation(String),
}

impl fmt::Display for ReconcileError {
    /// Formats `ReconcileError` into a human-readable diagnostic message.
    /// Realises: [REQ-RECONCILE-BACKLOG-LIB-FMT]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReconcileError::Io(err) => write!(f, "I/O error: {}", err),
            ReconcileError::TrackerError(msg) => write!(f, "Issue tracker error: {}", msg),
            ReconcileError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ReconcileError::PreconditionFailed(msg) => {
                write!(f, "Precondition failed: {}", msg)
            }
            ReconcileError::PublishGateViolation(msg) => {
                write!(f, "Publish gate violation: {}", msg)
            }
        }
    }
}

impl std::error::Error for ReconcileError {}

impl From<std::io::Error> for ReconcileError {
    /// Converts `std::io::Error` into `ReconcileError::Io`.
    /// Realises: [REQ-RECONCILE-BACKLOG-LIB-FROM]
    fn from(err: std::io::Error) -> Self {
        ReconcileError::Io(err)
    }
}

impl From<serde_json::Error> for ReconcileError {
    /// Converts `serde_json::Error` into `ReconcileError::ParseError`.
    /// Realises: [REQ-RECONCILE-BACKLOG-LIB-FROM-1]
    fn from(err: serde_json::Error) -> Self {
        ReconcileError::ParseError(format!("JSON parse error: {}", err))
    }
}

/// [DOC-TIER: A] Execution options for workspace backlog reconciliation.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-RECONCILEOPTIONS]
/// Contract: Configures workspace root, offline/dry-run flags, body sync, repo, and branch.
/// Bounds & Faults: Time O(1), Space O(P) over path and string fields.
#[derive(Debug, Clone)]
pub struct ReconcileOptions {
    pub workspace_root: PathBuf,
    pub offline: bool,
    pub dry_run: bool,
    pub provider: String,
    pub sync_bodies: bool,
    pub repo: Option<String>,
    pub default_branch: Option<String>,
}

impl Default for ReconcileOptions {
    /// Returns default `ReconcileOptions` rooted at `"."` with online GitHub synchronization enabled.
    /// Realises: [REQ-RECONCILE-BACKLOG-LIB-DEFAULT]
    fn default() -> Self {
        Self {
            workspace_root: PathBuf::from("."),
            offline: false,
            dry_run: false,
            provider: "github".to_string(),
            sync_bodies: true,
            repo: None,
            default_branch: None,
        }
    }
}

/// [DOC-TIER: A] Quantitative summary of actions performed during a reconciliation pass.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-RECONCILESUMMARY]
/// Contract: Tracks scanned specs, frontmatter updates, checklist updates, purged placeholders, and synced/registered issues.
/// Bounds & Faults: Time O(1), Space O(1).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReconcileSummary {
    pub total_specs_scanned: usize,
    pub frontmatters_updated: usize,
    pub checklists_updated: usize,
    pub placeholders_purged: usize,
    pub remote_issues_synced: usize,
    pub issues_registered: usize,
}

/// [DOC-TIER: A] Atomically writes `content` to `path` via a sibling temporary file and `fs::rename`.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-WRITE-ATOMIC]
/// Contract: Guarantees crash-safe persistence without partial file truncation.
/// Bounds & Faults: Time O(N), Space O(1) disk buffer where N is `content.len()`.
pub fn write_atomic(path: &Path, content: &str) -> Result<(), ReconcileError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("spec.md");
    let tmp_path = parent.join(format!(".{}.reconcile.tmp.{}", file_name, std::process::id()));
    {
        let mut f = fs::File::create(&tmp_path)?;
        f.write_all(content.as_bytes())?;
        f.sync_all()?;
    }
    if let Err(err) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(ReconcileError::Io(err));
    }
    Ok(())
}

/// [DOC-TIER: A] Collects all Markdown specification files across canonical landing zones in deterministic order.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-COLLECT-SPEC-FILES]
/// Contract: Scans `docs/{epics,features,user-stories,use-cases}` (or `workspace_root` directly) and excludes `.gitkeep`/`README.md`.
/// Bounds & Faults: Time O(F log F), Space O(F) over discovered files F.
pub fn collect_spec_files(workspace_root: &Path) -> Result<Vec<PathBuf>, ReconcileError> {
    let subdirs = [
        "docs/epics",
        "docs/features",
        "docs/user-stories",
        "docs/use-cases",
    ];

    let mut files = Vec::new();
    let mut found_standard_dirs = false;

    for sub in &subdirs {
        let dir = workspace_root.join(sub);
        if dir.exists() && dir.is_dir() {
            found_standard_dirs = true;
            walk_markdown_files(&dir, &mut files)?;
        }
    }

    if !found_standard_dirs {
        let alt_subdirs = ["epics", "features", "user-stories", "use-cases"];
        for sub in &alt_subdirs {
            let dir = workspace_root.join(sub);
            if dir.exists() && dir.is_dir() {
                found_standard_dirs = true;
                walk_markdown_files(&dir, &mut files)?;
            }
        }
    }

    if !found_standard_dirs && workspace_root.exists() && workspace_root.is_dir() {
        walk_markdown_files(workspace_root, &mut files)?;
    }

    files.sort();
    files.dedup();
    Ok(files)
}

/// Recursively collects `.md` specification files under `dir`, skipping hidden dirs and `README.md`.
/// Realises: [REQ-RECONCILE-BACKLOG-LIB-WALK-MARKDOWN-FILES]
fn walk_markdown_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), ReconcileError> {
    let mut entries: Vec<_> = fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        if file_name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            if matches!(file_name, "target" | "node_modules" | "vendor" | "build") {
                continue;
            }
            walk_markdown_files(&path, out)?;
        } else if path.is_file() {
            let is_md = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("md"));
            if is_md && !file_name.eq_ignore_ascii_case("README.md") {
                out.push(path);
            }
        }
    }
    Ok(())
}
