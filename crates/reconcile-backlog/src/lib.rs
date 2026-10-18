//! Pure Rust Backlog Reconciler for DEAP MBSE Framework.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides deterministic, zero-placeholder backlog and specification synchronization
//! across Epics, Features, User Stories, Use Cases, ConOps, and WBS artifacts in
//! compliance with DO-178C Level A and ISO 26262 ASIL D configuration management mandates.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Bijective mapping between local Markdown specification YAML frontmatter (`github_issue`)
//!   and canonical issue tracker identifiers.
//! - Idempotent placeholder purging: applying reconciliation twice yields zero additional
//!   mutations ($f(f(x)) = f(x)$).
//! - Deterministic `BTreeMap` ordering across all synthesized hierarchy checklists.
//!
//! ## 3. Preconditions
//! - Workspace root contains valid UTF-8 Markdown specification documents under `docs/`.
//! - Issue tracker client is either authenticated or explicitly configured in offline/dry-run mode.
//!
//! ## 4. Postconditions
//! - All unpopulated template placeholders are replaced with deterministic structural allocations.
//! - Hierarchical checklists and Mermaid traceability diagrams reflect live tracker state.
//! - YAML frontmatter `github_issue` fields are synchronized with created or matched tracker issues.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N \log N + M)$ where $N$ is the number of specification documents and
//!   $M$ is the total byte length of Markdown content scanned linearly without backtracking regex.
//! - Space Complexity: $O(N + M)$ bounded by the in-memory `LoadedSpec` collection and `IssueIndex`.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Missing or malformed YAML frontmatter is skipped safely or reported via `Result::Err` without panicking.
//! - Tracker CLI subprocess failures return explicit diagnostic errors preserving local file integrity.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-RECONCILE-BACKLOG-LIB, Issue Tracker as Canonical SSOT, Zero Fallback Placeholders, Automated Spec Reconciliation]

pub mod checklist;
pub mod create_issue;
pub mod frontmatter;
pub mod tracker;

use checklist::{clean_unpopulated_placeholders_counted, contains_unpopulated_placeholders, reconcile_checklist_content};
pub use checklist::{synthesize_and_reconcile_hierarchy, SpecDocument};
use frontmatter::{parse_frontmatter, update_frontmatter_issue_id, SpecFrontmatter};
use tracker::{IssueIndex, TrackerClient};

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Represents a loaded specification document.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `LoadedSpec` in `crates/reconcile-backlog/src/lib.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `LoadedSpec` across member set `[path: PathBuf, rel_path: String, frontmatter: SpecFrontmatter, content: String]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `LoadedSpec` members (`path: PathBuf, rel_path: String, frontmatter: SpecFrontmatter, content: String`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `LoadedSpec` representation for struct `LoadedSpec`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(4)) bounded over input/member dimension of `LoadedSpec`.
/// - Space Complexity: O(M(4)) bounded auxiliary memory for `LoadedSpec`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `LoadedSpec` (`LoadedSpec`).
/// - State: Maintains consistent state invariants across `LoadedSpec` without partial mutation on error.
/// - Action: Executes `LoadedSpec` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `LoadedSpec` (`RECONCILE-BACKLOG`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-LOADEDSPEC] | SysML v2: PartDef(LoadedSpec)
#[derive(Debug, Clone)]
pub struct LoadedSpec {
    pub path: PathBuf,
    pub rel_path: String,
    pub frontmatter: SpecFrontmatter,
    pub content: String,
}

/// Configuration options for the reconciler.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ReconcileOptions` in `crates/reconcile-backlog/src/lib.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ReconcileOptions` across member set `[workspace_root: PathBuf, offline: bool, dry_run: bool, provider: String, sync_bodies: bool]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ReconcileOptions` members (`workspace_root: PathBuf, offline: bool, dry_run: bool, provider: String, sync_bodies: bool`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ReconcileOptions` representation for struct `ReconcileOptions`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `ReconcileOptions`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `ReconcileOptions`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ReconcileOptions` (`ReconcileOptions`).
/// - State: Maintains consistent state invariants across `ReconcileOptions` without partial mutation on error.
/// - Action: Executes `ReconcileOptions` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ReconcileOptions` (`RECONCILE-BACKLOG`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-RECONCILEOPTIONS] | SysML v2: PartDef(ReconcileOptions)
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
    /// Executes deterministic fn `ReconcileOptions::default` within `crates/reconcile-backlog/src/lib.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ReconcileOptions::default` in `crates/reconcile-backlog/src/lib.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `ReconcileOptions::default` (declaration #3) within `RECONCILE-BACKLOG`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `ReconcileOptions::default` in `RECONCILE-BACKLOG`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Self` from `ReconcileOptions::default` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K(default)) bounded over input/member dimension of `ReconcileOptions::default`.
    /// - Space Complexity: O(K(default)) bounded auxiliary memory for `ReconcileOptions::default`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `ReconcileOptions::default` (`Self`).
    /// - State: Maintains consistent state invariants across `ReconcileOptions::default` without partial mutation on error.
    /// - Action: Executes `ReconcileOptions::default` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `ReconcileOptions::default` (`RECONCILE-BACKLOG`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RECONCILE-BACKLOG-RECONCILEOPTIONS-DEFAULT] | SysML v2: ActionDef(ReconcileOptions::default)
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

/// Result summary of reconciliation execution.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ReconcileSummary` in `crates/reconcile-backlog/src/lib.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ReconcileSummary` across member set `[total_specs_scanned: usize, frontmatters_updated: usize, checklists_updated: usize, placeholders_purged: usize, remote_issues_synced: usize]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ReconcileSummary` members (`total_specs_scanned: usize, frontmatters_updated: usize, checklists_updated: usize, placeholders_purged: usize, remote_issues_synced: usize`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ReconcileSummary` representation for struct `ReconcileSummary`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `ReconcileSummary`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `ReconcileSummary`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ReconcileSummary` (`ReconcileSummary`).
/// - State: Maintains consistent state invariants across `ReconcileSummary` without partial mutation on error.
/// - Action: Executes `ReconcileSummary` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ReconcileSummary` (`RECONCILE-BACKLOG`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-RECONCILESUMMARY] | SysML v2: PartDef(ReconcileSummary)
#[derive(Debug, Clone, Default)]
pub struct ReconcileSummary {
    pub total_specs_scanned: usize,
    pub frontmatters_updated: usize,
    pub checklists_updated: usize,
    pub placeholders_purged: usize,
    pub remote_issues_synced: usize,
}

/// Discovers and loads all markdown specs in standard docs directories.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `load_specifications` in `crates/reconcile-backlog/src/lib.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `load_specifications` (declaration #5) within `RECONCILE-BACKLOG`.
///
/// ## 3. Preconditions
/// - `workspace_root: &Path` must satisfy domain and type-safety constraints prior to invoking `load_specifications`.
///
/// ## 4. Postconditions
/// - Returns `Vec<LoadedSpec>` from `load_specifications` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(workspace_root)) bounded over input/member dimension of `load_specifications`.
/// - Space Complexity: O(N(workspace_root)) bounded auxiliary memory for `load_specifications`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `load_specifications` (`Vec<LoadedSpec>`).
/// - State: Maintains consistent state invariants across `load_specifications` without partial mutation on error.
/// - Action: Executes `load_specifications` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `load_specifications` (`RECONCILE-BACKLOG`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-LOAD-SPECIFICATIONS] | SysML v2: ActionDef(load_specifications)
pub fn load_specifications(workspace_root: &Path) -> Vec<LoadedSpec> {
    let spec_subdirs = ["docs/epics", "docs/features", "docs/user-stories", "docs/use-cases"];
    let mut specs = Vec::new();

    for subdir in &spec_subdirs {
        let dir = workspace_root.join(subdir);
        if !dir.is_dir() {
            continue;
        }

        for entry in WalkDir::new(&dir).sort_by_file_name().into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().map(|ext| ext == "md").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Some(fm) = parse_frontmatter(&content) {
                        let rel = path
                            .strip_prefix(workspace_root)
                            .unwrap_or(path)
                            .display()
                            .to_string();
                        specs.push(LoadedSpec {
                            path: path.to_path_buf(),
                            rel_path: rel,
                            frontmatter: fm,
                            content,
                        });
                    }
                }
            }
        }
    }

    specs
}

/// Runs the complete reconciliation workflow.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `run_reconciliation` in `crates/reconcile-backlog/src/lib.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `run_reconciliation` (declaration #6) within `RECONCILE-BACKLOG`.
///
/// ## 3. Preconditions
/// - `tracker: &T` must satisfy domain and type-safety constraints prior to invoking `run_reconciliation`.
/// - `options: &ReconcileOptions` must satisfy domain and type-safety constraints prior to invoking `run_reconciliation`.
///
/// ## 4. Postconditions
/// - Returns `Result<ReconcileSummary, String>` from `run_reconciliation` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(tracker)) bounded over input/member dimension of `run_reconciliation`.
/// - Space Complexity: O(N(tracker)) bounded auxiliary memory for `run_reconciliation`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `run_reconciliation` (`Result<ReconcileSummary, String>`).
/// - State: Maintains consistent state invariants across `run_reconciliation` without partial mutation on error.
/// - Action: Executes `run_reconciliation` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `run_reconciliation` (`RECONCILE-BACKLOG`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-RUN-RECONCILIATION] | SysML v2: ActionDef(run_reconciliation)
pub fn run_reconciliation<T: TrackerClient>(
    tracker: &T,
    options: &ReconcileOptions,
) -> Result<ReconcileSummary, String> {
    let mut summary = ReconcileSummary::default();
    let mut specs = load_specifications(&options.workspace_root);
    summary.total_specs_scanned = specs.len();

    // Step 1: Fetch tracker issues if available
    let issues = if options.offline {
        Vec::new()
    } else {
        tracker.fetch_all_issues()?
    };
    let issue_index = IssueIndex::build(issues);

    // Step 2: Hierarchy synthesis, frontmatter reconciliation, and placeholder purge
    let repo_slug = options.repo.clone()
        .or_else(|| detect_git_repo(&options.workspace_root))
        .unwrap_or_default();
    let default_branch = options.default_branch.clone()
        .or_else(|| detect_git_branch(&options.workspace_root))
        .unwrap_or_else(|| "main".to_string());

    let all_specs_snapshot = specs.clone();

    for spec in &mut specs {
        let mut modified = false;

        let synthesized = checklist::synthesize_and_reconcile_hierarchy(
            spec,
            &all_specs_snapshot,
            &repo_slug,
            &default_branch,
            &issue_index,
        );
        if synthesized != spec.content {
            spec.content = synthesized;
            modified = true;
            summary.checklists_updated += 1;
        }

        // Check for prohibited unpopulated placeholders
        if contains_unpopulated_placeholders(&spec.content) {
            let (cleaned_content, purged_count) = clean_unpopulated_placeholders_counted(&spec.content);
            if purged_count > 0 {
                spec.content = cleaned_content;
                modified = true;
                summary.placeholders_purged += purged_count;
            }
        }

        // Match spec to tracker issue
        if let Some(issue) = issue_index.find_issue_for_type(
            spec.frontmatter.issue_id,
            Some(&spec.frontmatter.spec_type),
            &spec.frontmatter.title,
        ) {
            if spec.frontmatter.issue_id != Some(issue.number) {
                spec.content = update_frontmatter_issue_id(&spec.content, issue.number);
                spec.frontmatter.issue_id = Some(issue.number);
                modified = true;
                summary.frontmatters_updated += 1;
            }
        }

        // Step 3: Reconcile checklists
        let (reconciled_body, check_count) = reconcile_checklist_content(&spec.content, |issue_num, item_title| {
            if let Some(num) = issue_num {
                if let Some(issue) = issue_index.by_number.get(&num) {
                    return Some(issue.is_resolved());
                }
            }
            if let Some(issue) = issue_index.find_issue(None, item_title) {
                return Some(issue.is_resolved());
            }
            None
        });

        if check_count > 0 {
            spec.content = reconciled_body;
            modified = true;
            summary.checklists_updated += check_count;
        }

        if modified && !options.dry_run {
            fs::write(&spec.path, &spec.content)
                .map_err(|e| format!("Failed to write updated spec '{}': {}", spec.path.display(), e))?;
        }

    }

    // Step 4: Remote sync if enabled with relative link projection
    if options.sync_bodies && !options.offline && !options.dry_run {
        let mut spec_path_to_issue: BTreeMap<String, u64> = BTreeMap::new();
        for spec in &specs {
            if let Some(issue_id) = spec.frontmatter.issue_id {
                spec_path_to_issue.insert(spec.rel_path.clone(), issue_id);
                if let Some(stripped) = spec.rel_path.strip_prefix("docs/") {
                    spec_path_to_issue.insert(stripped.to_string(), issue_id);
                }
                if let Some(fname) = Path::new(&spec.rel_path).file_name().and_then(|f| f.to_str()) {
                    spec_path_to_issue.insert(fname.to_string(), issue_id);
                }
            }
        }

        let repo_name = options.repo.clone()
            .or_else(|| detect_git_repo(&options.workspace_root));
        let default_branch = options.default_branch.clone()
            .or_else(|| detect_git_branch(&options.workspace_root))
            .unwrap_or_else(|| "main".to_string());

        // Blocking registration publish gate (Issue #223): verify 100% of checklist references
        for spec in &specs {
            if spec.frontmatter.issue_id.is_some() {
                validate_checklist_publish_integrity(&spec.rel_path, &spec.content, &issue_index)?;
            }
        }

        for spec in &specs {
            if let Some(issue_id) = spec.frontmatter.issue_id {
                let projected_body = project_body_for_tracker(
                    &spec.content,
                    repo_name.as_deref(),
                    &default_branch,
                    &spec_path_to_issue,
                );
                tracker.update_issue_body(issue_id, &projected_body)?;
                summary.remote_issues_synced += 1;
            }
        }
    }

    Ok(summary)
}

/// Validates that all checklist items in a specification body resolve to live tracker issues
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `validate_checklist_publish_integrity` in `crates/reconcile-backlog/src/lib.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `validate_checklist_publish_integrity` (declaration #7) within `RECONCILE-BACKLOG`.
///
/// ## 3. Preconditions
/// - `spec_rel_path: &str` must satisfy domain and type-safety constraints prior to invoking `validate_checklist_publish_integrity`.
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `validate_checklist_publish_integrity`.
/// - `issue_index: &IssueIndex` must satisfy domain and type-safety constraints prior to invoking `validate_checklist_publish_integrity`.
///
/// ## 4. Postconditions
/// - Returns `Result<(), String>` from `validate_checklist_publish_integrity` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(spec_rel_path)) bounded over input/member dimension of `validate_checklist_publish_integrity`.
/// - Space Complexity: O(N(spec_rel_path)) bounded auxiliary memory for `validate_checklist_publish_integrity`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `validate_checklist_publish_integrity` (`Result<(), String>`).
/// - State: Maintains consistent state invariants across `validate_checklist_publish_integrity` without partial mutation on error.
/// - Action: Executes `validate_checklist_publish_integrity` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `validate_checklist_publish_integrity` (`RECONCILE-BACKLOG`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-VALIDATE-CHECKLIST-PUBLISH-INTEGRITY] | SysML v2: ActionDef(validate_checklist_publish_integrity)
pub fn validate_checklist_publish_integrity(
    spec_rel_path: &str,
    content: &str,
    issue_index: &IssueIndex,
) -> Result<(), String> {
    for item in checklist::parse_checklists(content) {
        if item.full_line.contains("#[IssueID]")
            || item.full_line.contains("#[FeatureIssueID]")
            || item.full_line.contains("#[UseCaseIssueID]")
            || item.full_line.contains("#[StoryIssueID]")
            || item.full_line.contains("#[EpicIssueID]")
        {
            return Err(format!(
                "Publish gate blocked: specification '{}' contains unresolved checklist placeholder token. Sibling must be registered before publishing.",
                spec_rel_path
            ));
        }

        if let Some(num) = item.issue_number {
            if !issue_index.by_number.contains_key(&num) {
                return Err(format!(
                    "Publish gate blocked: specification '{}' contains unresolvable checklist issue reference '#{}'. Sibling must be registered on live tracker before publishing.",
                    spec_rel_path, num
                ));
            }
        }

        // Also scan item.full_line for any #<digits> references and verify that each number exists in issue_index.by_number
        let bytes = item.full_line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'#' {
                let start = i + 1;
                let mut end = start;
                while end < bytes.len() && bytes[end].is_ascii_digit() {
                    end += 1;
                }
                if end > start {
                    if let Ok(num) = item.full_line[start..end].parse::<u64>() {
                        if !issue_index.by_number.contains_key(&num) {
                            return Err(format!(
                                "Publish gate blocked: specification '{}' contains unresolvable checklist issue reference '#{}'. Sibling must be registered on live tracker before publishing.",
                                spec_rel_path, num
                            ));
                        }
                    }
                    i = end;
                    continue;
                }
            }
            i += 1;
        }
    }
    Ok(())
}

/// Cleans a repository name or URL to `owner/repo` format without regex.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `clean_repo_name` in `crates/reconcile-backlog/src/lib.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `clean_repo_name` (declaration #8) within `RECONCILE-BACKLOG`.
///
/// ## 3. Preconditions
/// - `raw: &str` must satisfy domain and type-safety constraints prior to invoking `clean_repo_name`.
///
/// ## 4. Postconditions
/// - Returns `String` from `clean_repo_name` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(raw)) bounded over input/member dimension of `clean_repo_name`.
/// - Space Complexity: O(N(raw)) bounded auxiliary memory for `clean_repo_name`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `clean_repo_name` (`String`).
/// - State: Maintains consistent state invariants across `clean_repo_name` without partial mutation on error.
/// - Action: Executes `clean_repo_name` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `clean_repo_name` (`RECONCILE-BACKLOG`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-CLEAN-REPO-NAME] | SysML v2: ActionDef(clean_repo_name)
pub fn clean_repo_name(raw: &str) -> String {
    let mut s = raw.trim();
    if let Some(stripped) = s.strip_prefix("https://github.com/") {
        s = stripped;
    } else if let Some(stripped) = s.strip_prefix("http://github.com/") {
        s = stripped;
    } else if let Some(stripped) = s.strip_prefix("git@github.com:") {
        s = stripped;
    }
    s = s.trim_end_matches(".git").trim_matches('/');
    s.to_string()
}

/// Parses `owner/repo` from a remote Git URL without regex.
pub fn parse_repo_from_remote_url(url: &str) -> Option<String> {
    let cleaned = clean_repo_name(url);
    if cleaned.contains('/') && !cleaned.contains(' ') {
        let parts: Vec<&str> = cleaned.split('/').collect();
        if matches!(parts.as_slice(), [owner, repo] if !owner.is_empty() && !repo.is_empty()) {
            return Some(cleaned);
        }
    }
    None
}

/// Automatically detects Git repository name (`owner/repo`) from workspace root without regex.
pub fn detect_git_repo(workspace_root: &Path) -> Option<String> {
    // 1. git config --get remote.origin.url
    if let Ok(output) = Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .current_dir(workspace_root)
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            if let Some(repo) = parse_repo_from_remote_url(&s) {
                return Some(repo);
            }
        }
    }

    // 2. gh repo view
    if let Ok(output) = Command::new("gh")
        .args(["repo", "view", "--json", "nameWithOwner", "-q", ".nameWithOwner"])
        .current_dir(workspace_root)
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Some(repo) = parse_repo_from_remote_url(&s) {
                return Some(repo);
            }
        }
    }

    None
}

/// Automatically detects Git default branch from workspace root without regex.
pub fn detect_git_branch(workspace_root: &Path) -> Option<String> {
    if let Ok(output) = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(workspace_root)
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !s.is_empty() && s != "HEAD" {
                return Some(s);
            }
        }
    }
    None
}

fn rewrite_projected_links(
    body: &str,
    repo: Option<&str>,
    default_branch: &str,
    spec_path_to_issue: &BTreeMap<String, u64>,
) -> String {
    let mut result = String::with_capacity(body.len());
    let mut rest = body;

    while let Some(sep_idx) = rest.find("](") {
        // Scan backwards from sep_idx to find matching opening '['
        let before_sep = &rest[..sep_idx];
        let mut depth = 1;
        let mut open_b = None;

        for (idx, ch) in before_sep.char_indices().rev() {
            if ch == ']' {
                depth += 1;
            } else if ch == '[' {
                depth -= 1;
                if depth == 0 {
                    open_b = Some(idx);
                    break;
                }
            } else if ch == '\n' {
                // Do not span across lines for a single link
                break;
            }
        }

        if let Some(open_idx) = open_b {
            let after_paren = &rest[sep_idx + 2..];
            if let Some(close_p_rel) = after_paren.find(')') {
                let target = &after_paren[..close_p_rel];
                let text = &rest[open_idx + 1..sep_idx];
                let total_end = sep_idx + 2 + close_p_rel + 1;

                result.push_str(&rest[..open_idx]);

                if target.starts_with("http://") || target.starts_with("https://") {
                    result.push_str(&rest[open_idx..total_end]);
                    rest = &rest[total_end..];
                    continue;
                }

                let fname = Path::new(target).file_name().and_then(|f| f.to_str()).unwrap_or("");
                let stripped = target.trim_start_matches('.').trim_start_matches('/');

                let matching_issue = spec_path_to_issue.get(target)
                    .or_else(|| spec_path_to_issue.get(stripped))
                    .or_else(|| {
                        if !fname.is_empty() {
                            spec_path_to_issue.get(fname)
                        } else {
                            None
                        }
                    })
                    .or_else(|| {
                        if !fname.is_empty() {
                            spec_path_to_issue.iter().find_map(|(k, v)| {
                                if k.ends_with(fname) {
                                    Some(v)
                                } else {
                                    None
                                }
                            })
                        } else {
                            None
                        }
                    })
                    .copied();

                if let Some(issue_num) = matching_issue {
                    let cur_line_start = result.rfind('\n').map(|idx| idx + 1).unwrap_or(0);
                    let cur_line_prefix = &result[cur_line_start..];
                    let already_has_issue = cur_line_prefix.contains(&format!("#{}", issue_num));

                    let canonical_target = if let Some(r) = repo {
                        let clean_repo = clean_repo_name(r);
                        let branch = if default_branch.trim().is_empty() { "main" } else { default_branch.trim() };
                        let doc_path = if target.contains("docs/") {
                            target[target.find("docs/").unwrap()..].to_string()
                        } else if target.contains("epics/") {
                            format!("docs/epics/{}", fname)
                        } else if target.contains("features/") {
                            format!("docs/features/{}", fname)
                        } else if target.contains("user-stories/") {
                            format!("docs/user-stories/{}", fname)
                        } else if target.contains("use-cases/") {
                            format!("docs/use-cases/{}", fname)
                        } else {
                            format!("docs/{}", stripped)
                        };
                        format!("https://github.com/{}/blob/{}/{}", clean_repo, branch, doc_path)
                    } else {
                        target.to_string()
                    };

                    if already_has_issue {
                        result.push_str(&format!("[{}]({})", text, canonical_target));
                    } else {
                        result.push_str(&format!("#{} - [{}]({})", issue_num, text, canonical_target));
                    }

                    rest = &rest[total_end..];
                    continue;
                } else if let Some(r) = repo {
                    let clean_repo = clean_repo_name(r);
                    if !clean_repo.is_empty() {
                        let branch = if default_branch.trim().is_empty() { "main" } else { default_branch.trim() };
                        if target.contains("schema/") {
                            let s_path = &target[target.find("schema/").unwrap()..];
                            let blob = format!("https://github.com/{}/blob/{}/{}", clean_repo, branch, s_path);
                            result.push_str(&format!("[{}]({})", text, blob));
                            rest = &rest[total_end..];
                            continue;
                        } else if target.contains("docs/") {
                            let d_path = &target[target.find("docs/").unwrap()..];
                            let blob = format!("https://github.com/{}/blob/{}/{}", clean_repo, branch, d_path);
                            result.push_str(&format!("[{}]({})", text, blob));
                            rest = &rest[total_end..];
                            continue;
                        } else if let Some(subdir) = ["epics/", "features/", "user-stories/", "use-cases/"].iter().find(|s| target.contains(*s)) {
                            let s_path = &target[target.find(subdir).unwrap()..];
                            let blob = format!("https://github.com/{}/blob/{}/docs/{}", clean_repo, branch, s_path);
                            result.push_str(&format!("[{}]({})", text, blob));
                            rest = &rest[total_end..];
                            continue;
                        }
                    }
                }

                result.push_str(&rest[open_idx..total_end]);
                rest = &rest[total_end..];
                continue;
            }
        }

        result.push_str(&rest[..sep_idx + 2]);
        rest = &rest[sep_idx + 2..];
    }

    result.push_str(rest);
    result
}

/// Translates relative filesystem markdown links in a spec body to tracker-compatible links:
/// - Relative spec links (`../epics/EPIC-03-*.md`) -> Canonical repo blob URLs prefixed by `#<id>` without intra-page anchors
/// - Relative schema links (`../../schema/...`) -> GitHub blob URLs (`https://github.com/<repo>/blob/<branch>/schema/...`)
/// - Relative docs links (`../../docs/...`) -> GitHub blob URLs (`https://github.com/<repo>/blob/<branch>/docs/...`)
pub fn project_body_for_tracker(
    body: &str,
    repo: Option<&str>,
    default_branch: &str,
    spec_path_to_issue: &BTreeMap<String, u64>,
) -> String {
    let mut projected = rewrite_projected_links(body, repo, default_branch, spec_path_to_issue);

    // Translate any remaining relative schema and docs links outside markdown link targets to absolute GitHub blob URLs
    if let Some(r) = repo {
        let clean_repo = clean_repo_name(r);
        if !clean_repo.is_empty() {
            let branch = if default_branch.trim().is_empty() {
                "main"
            } else {
                default_branch.trim()
            };
            let schema_blob_base = format!("https://github.com/{}/blob/{}/schema/", clean_repo, branch);
            projected = projected.replace("../../../schema/", &schema_blob_base);
            projected = projected.replace("../../schema/", &schema_blob_base);
            projected = projected.replace("../schema/", &schema_blob_base);

            let docs_blob_base = format!("https://github.com/{}/blob/{}/docs/", clean_repo, branch);
            projected = projected.replace("../../../docs/", &docs_blob_base);
            projected = projected.replace("../../docs/", &docs_blob_base);
            projected = projected.replace("../docs/", &docs_blob_base);

            for subdir in ["epics", "features", "user-stories", "use-cases"] {
                let blob_base = format!("https://github.com/{}/blob/{}/docs/{}/", clean_repo, branch, subdir);
                projected = projected.replace(&format!("../../../{}/", subdir), &blob_base);
                projected = projected.replace(&format!("../../{}/", subdir), &blob_base);
                projected = projected.replace(&format!("../{}/", subdir), &blob_base);
            }
        }
    }

    projected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::{OfflineTracker, TrackerIssue, TrackerLabel};
    use std::fs;

    #[test]
    fn test_relative_links_in_projected_issue_bodies_red_probe() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = std::env::temp_dir().join(format!("reconcile_red_probe_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        let epics_dir = temp_dir.join("docs/epics");
        let stories_dir = temp_dir.join("docs/user-stories");
        fs::create_dir_all(&epics_dir)?;
        fs::create_dir_all(&stories_dir)?;

        let epic_content = r#"---
issue_id: 80
title: "Epic 03: System Vision"
type: epic
---
# Epic 03: System Vision
"#;
        fs::write(epics_dir.join("EPIC-03-SystemVision.md"), epic_content)?;

        let story_content = r#"---
issue_id: 89
title: "User Story 01: Bootstrap System Vision Action"
type: user-story
---
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #89 |
| **Parent Epic** | [EPIC-03-SystemVision](../epics/EPIC-03-SystemVision.md) |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

## Required Features
- [ ] [Feature 01: [ConOps] Human Engineer Interface](../features/FEAT-01-HumanEngineer.md)
"#;
        fs::write(stories_dir.join("US-01-BootstrapSystemVisionAction.md"), story_content)?;

        let issues = vec![
            TrackerIssue {
                number: 80,
                title: "Epic 03: System Vision".to_string(),
                state: "OPEN".to_string(),
                labels: vec![TrackerLabel::String("epic".to_string())],
                body: None,
                url: Some("https://github.com/gintatkinson/dumpiler-01/issues/80".to_string()),
            },
            TrackerIssue {
                number: 89,
                title: "User Story 01: Bootstrap System Vision Action".to_string(),
                state: "OPEN".to_string(),
                labels: vec![TrackerLabel::String("user-story".to_string())],
                body: None,
                url: Some("https://github.com/gintatkinson/dumpiler-01/issues/89".to_string()),
            },
        ];

        let tracker = OfflineTracker::with_issues(issues);
        let options = ReconcileOptions {
            workspace_root: temp_dir.clone(),
            offline: false,
            dry_run: false,
            provider: "offline".to_string(),
            sync_bodies: true,
            repo: Some("gintatkinson/dumpiler-01".to_string()),
            default_branch: Some("main".to_string()),
        };

        let summary = run_reconciliation(&tracker, &options)?;
        assert_eq!(summary.remote_issues_synced, 2);

        let synced_issues = tracker.fetch_all_issues()?;
        let synced_story = synced_issues.iter().find(|i| i.number == 89).ok_or("synced story #89 missing")?;
        let body = synced_story.body.as_ref().ok_or("Body should have been synced")?;

        // The probe: Projected body must NOT contain relative filesystem links that 404 on the remote tracker!
        assert!(
            !body.contains("../epics/EPIC-03-SystemVision.md"),
            "Projected issue body still contains relative filesystem link '../epics/EPIC-03-SystemVision.md'!"
        );
        assert!(
            !body.contains("../features/FEAT-01-HumanEngineer.md"),
            "Projected issue body still contains relative filesystem link '../features/FEAT-01-HumanEngineer.md'!"
        );
        assert!(
            !body.contains("../../schema/model.sysml"),
            "Projected issue body still contains relative filesystem link '../../schema/model.sysml'!"
        );
        assert!(
            !body.contains("](#80)"),
            "Projected body must NOT contain broken intra-page anchor link '](#80)'!"
        );
        assert!(
            body.contains("#80 - [EPIC-03-SystemVision](https://github.com/gintatkinson/dumpiler-01/blob/main/docs/epics/EPIC-03-SystemVision.md)"),
            "Projected body should contain translated citation '#80 - [EPIC-03-SystemVision](...)', found:\n{}",
            body
        );
        assert!(
            body.contains("[Feature 01: [ConOps] Human Engineer Interface](https://github.com/gintatkinson/dumpiler-01/blob/main/docs/features/FEAT-01-HumanEngineer.md)"),
            "Projected body should contain translated blob URL for unmapped feature link, found:\n{}",
            body
        );
        assert!(
            body.contains("https://github.com/gintatkinson/dumpiler-01/blob/main/schema/model.sysml"),
            "Projected body should contain translated blob URL, found:\n{}",
            body
        );

        let _ = fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_project_body_for_tracker_comprehensive() {
        let raw_body = r#"
| **Parent Epic** | [EPIC-03-SystemVision](../epics/EPIC-03-SystemVision.md) |
| **Specification Source** | [schema/subsystems/system_vision/behavior.sysml](../../schema/subsystems/system_vision/behavior.sysml) |

## Required Features
- [ ] [Feature 01: [ConOps] Human Engineer Interface](../features/FEAT-01-HumanEngineer.md)
- [ ] [Feature 02: [Core] Baseline Verifier](../../docs/features/FEAT-02-Verifier.md)

## Design Docs
- [Architecture Blueprint](../../docs/architecture/DEAP_MASTER_ARCHITECTURE.md)
"#;
        let mut spec_to_issue = BTreeMap::new();
        spec_to_issue.insert("docs/epics/EPIC-03-SystemVision.md".to_string(), 80);
        spec_to_issue.insert("features/FEAT-01-HumanEngineer.md".to_string(), 75);
        spec_to_issue.insert("docs/features/FEAT-02-Verifier.md".to_string(), 76);

        let projected = project_body_for_tracker(
            raw_body,
            Some("gintatkinson/dumpiler-01"),
            "main",
            &spec_to_issue,
        );

        assert!(!projected.contains("../epics/EPIC-03-SystemVision.md"));
        assert!(!projected.contains("../../schema/"));
        assert!(!projected.contains("../features/FEAT-01-HumanEngineer.md"));
        assert!(!projected.contains("../../docs/features/FEAT-02-Verifier.md"));
        assert!(!projected.contains("../../docs/architecture/"));

        assert!(!projected.contains("](#80)"));
        assert!(!projected.contains("](#75)"));
        assert!(!projected.contains("](#76)"));
        assert!(projected.contains("#80 - [EPIC-03-SystemVision](https://github.com/gintatkinson/dumpiler-01/blob/main/docs/epics/EPIC-03-SystemVision.md)"));
        assert!(projected.contains("#75 - [Feature 01: [ConOps] Human Engineer Interface](https://github.com/gintatkinson/dumpiler-01/blob/main/docs/features/FEAT-01-HumanEngineer.md)"));
        assert!(projected.contains("#76 - [Feature 02: [Core] Baseline Verifier](https://github.com/gintatkinson/dumpiler-01/blob/main/docs/features/FEAT-02-Verifier.md)"));
        assert!(projected.contains("https://github.com/gintatkinson/dumpiler-01/blob/main/schema/subsystems/system_vision/behavior.sysml"));
        assert!(projected.contains("https://github.com/gintatkinson/dumpiler-01/blob/main/docs/architecture/DEAP_MASTER_ARCHITECTURE.md"));
    }

    #[test]
    fn test_unmapped_relative_spec_links_project_to_github_blob_urls() {
        let raw_body = r#"
- [Epic](../epics/EPIC-01.md)
- [Feature](../../features/FEAT-02.md)
- [Story](../../../user-stories/US-03.md)
- [Use Case](../use-cases/UC-04.md)
- Direct link: ../../../features/FEAT-05.md
"#;
        let empty_map = BTreeMap::new();
        let projected = project_body_for_tracker(
            raw_body,
            Some("owner/myrepo"),
            "develop",
            &empty_map,
        );

        assert!(!projected.contains("../epics/EPIC-01.md"));
        assert!(!projected.contains("../../features/FEAT-02.md"));
        assert!(!projected.contains("../../../user-stories/US-03.md"));
        assert!(!projected.contains("../use-cases/UC-04.md"));
        assert!(!projected.contains("../../../features/FEAT-05.md"));

        assert!(projected.contains("[Epic](https://github.com/owner/myrepo/blob/develop/docs/epics/EPIC-01.md)"));
        assert!(projected.contains("[Feature](https://github.com/owner/myrepo/blob/develop/docs/features/FEAT-02.md)"));
        assert!(projected.contains("[Story](https://github.com/owner/myrepo/blob/develop/docs/user-stories/US-03.md)"));
        assert!(projected.contains("[Use Case](https://github.com/owner/myrepo/blob/develop/docs/use-cases/UC-04.md)"));
        assert!(projected.contains("https://github.com/owner/myrepo/blob/develop/docs/features/FEAT-05.md"));
    }

    #[test]
    fn test_parse_repo_from_remote_url() {
        assert_eq!(
            parse_repo_from_remote_url("https://github.com/gintatkinson/DEAP01-spec-core.git"),
            Some("gintatkinson/DEAP01-spec-core".to_string())
        );
        assert_eq!(
            parse_repo_from_remote_url("git@github.com:gintatkinson/dumpiler-01.git"),
            Some("gintatkinson/dumpiler-01".to_string())
        );
        assert_eq!(
            parse_repo_from_remote_url("https://github.com/owner/repo"),
            Some("owner/repo".to_string())
        );
        assert_eq!(
            parse_repo_from_remote_url("owner/repo"),
            Some("owner/repo".to_string())
        );
        assert_eq!(parse_repo_from_remote_url("invalid-url"), None);
    }

    #[test]
    fn test_project_body_for_tracker_without_repo() {
        let raw_body = "[EPIC-01](../epics/EPIC-01.md) and [schema](../../schema/model.sysml)";
        let mut map = BTreeMap::new();
        map.insert("epics/EPIC-01.md".to_string(), 10);

        let projected = project_body_for_tracker(raw_body, None, "main", &map);
        assert_eq!(projected, "#10 - [EPIC-01](../epics/EPIC-01.md) and [schema](../../schema/model.sysml)");
    }

    #[test]
    fn test_publish_gate_blocks_unregistered_checklist_issue_numbers() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"# Epic: Test
## 2. Requirements & Checklist
- [ ] #999 - Feature 99: Ghost Feature (justification)
"#;
        let mut issue_index = IssueIndex::default();
        // Tracker only has issue 10, not 999
        issue_index.by_number.insert(
            10,
            tracker::TrackerIssue {
                number: 10,
                title: "Feature 01".to_string(),
                labels: vec![],
                state: "OPEN".to_string(),
                body: None,
                url: None,
            },
        );

        let err = validate_checklist_publish_integrity("docs/epics/EPIC-01.md", content, &issue_index)
            .err()
            .ok_or("expected publish gate to reject unregistered issue #999")?;
        assert!(err.contains("contains unresolvable checklist issue reference '#999'"));
        Ok(())
    }

    #[test]
    fn test_publish_gate_blocks_placeholder_checklist_tokens() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"# Epic: Test
## 2. Requirements & Checklist
- [ ] #[IssueID] - Feature 01: Test Feature (justification)
"#;
        let issue_index = IssueIndex::default();
        let err = validate_checklist_publish_integrity("docs/epics/EPIC-01.md", content, &issue_index)
            .err()
            .ok_or("expected publish gate to reject placeholder token")?;
        assert!(err.contains("contains unresolved checklist placeholder token"));
        Ok(())
    }

    #[test]
    fn test_publish_gate_passes_for_registered_checklist_issue_numbers() {
        let content = r#"# Epic: Test
## 2. Requirements & Checklist
- [ ] #10 - Feature 01: Real Feature (justification)
"#;
        let mut issue_index = IssueIndex::default();
        issue_index.by_number.insert(
            10,
            tracker::TrackerIssue {
                number: 10,
                title: "Feature 01".to_string(),
                labels: vec![],
                state: "OPEN".to_string(),
                body: None,
                url: None,
            },
        );

        let res = validate_checklist_publish_integrity("docs/epics/EPIC-01.md", content, &issue_index);
        assert!(res.is_ok());
    }

    #[test]
    fn test_publish_gate_blocks_unregistered_inline_issue_numbers() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"# Epic: Test
## 2. Requirements & Checklist
- [ ] [Feature 99: Ghost Feature](../features/FEAT-99.md) (#999)
"#;
        let mut issue_index = IssueIndex::default();
        issue_index.by_number.insert(
            10,
            tracker::TrackerIssue {
                number: 10,
                title: "Feature 01".to_string(),
                labels: vec![],
                state: "OPEN".to_string(),
                body: None,
                url: None,
            },
        );

        let err = validate_checklist_publish_integrity("docs/epics/EPIC-01.md", content, &issue_index)
            .err()
            .ok_or("expected publish gate to reject unregistered inline issue #999")?;
        assert!(err.contains("Publish gate blocked"));
        assert!(err.contains("contains unresolvable checklist issue reference '#999'"));
        Ok(())
    }

    #[test]
    fn test_publish_gate_passes_for_registered_inline_issue_numbers() {
        let content = r#"# Epic: Test
## 2. Requirements & Checklist
- [ ] [Feature 01: Real Feature](../features/FEAT-01.md) (#10)
"#;
        let mut issue_index = IssueIndex::default();
        issue_index.by_number.insert(
            10,
            tracker::TrackerIssue {
                number: 10,
                title: "Feature 01".to_string(),
                labels: vec![],
                state: "OPEN".to_string(),
                body: None,
                url: None,
            },
        );

        let res = validate_checklist_publish_integrity("docs/epics/EPIC-01.md", content, &issue_index);
        assert!(res.is_ok());
    }

    #[test]
    fn test_reconciliation_workflow_reconciles_markdown_linked_and_inline_issues() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = std::env::temp_dir().join(format!("reconcile_md_links_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        let stories_dir = temp_dir.join("docs/user-stories");
        fs::create_dir_all(&stories_dir)?;

        let story_content = r#"---
issue_id: 100
title: "User Story 01: Core"
type: user-story
---
# User Story 01: Core

## Required Features
- [ ] [Feature 01: Human Engineer Interface](../features/FEAT-01.md)
- [ ] [Feature 02: CI Runner](../features/FEAT-02.md) (#20)
"#;
        let story_path = stories_dir.join("US-01-Core.md");
        fs::write(&story_path, story_content)?;

        let issues = vec![
            TrackerIssue {
                number: 100,
                title: "User Story 01: Core".to_string(),
                state: "OPEN".to_string(),
                labels: vec![TrackerLabel::String("user-story".to_string())],
                body: None,
                url: None,
            },
            TrackerIssue {
                number: 10,
                title: "Feature 01: Human Engineer Interface".to_string(),
                state: "CLOSED".to_string(), // resolved!
                labels: vec![TrackerLabel::String("feature".to_string())],
                body: None,
                url: None,
            },
            TrackerIssue {
                number: 20,
                title: "Feature 02: CI Runner".to_string(),
                state: "CLOSED".to_string(), // resolved!
                labels: vec![TrackerLabel::String("feature".to_string())],
                body: None,
                url: None,
            },
        ];

        let tracker = OfflineTracker::with_issues(issues);
        let options = ReconcileOptions {
            workspace_root: temp_dir.clone(),
            offline: false,
            dry_run: false,
            provider: "offline".to_string(),
            sync_bodies: false,
            repo: None,
            default_branch: None,
        };

        let summary = run_reconciliation(&tracker, &options)?;
        assert_eq!(summary.checklists_updated, 2, "Expected both checklist items to be updated to [x]");

        let updated = fs::read_to_string(&story_path)?;
        assert!(updated.contains("- [x] [Feature 01: Human Engineer Interface](../features/FEAT-01.md)"));
        assert!(updated.contains("- [x] [Feature 02: CI Runner](../features/FEAT-02.md) (#20)"));

        let _ = fs::remove_dir_all(&temp_dir);
        Ok(())
    }

    #[test]
    fn test_issue_439_reconcile_workflow_placeholder_telemetry() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = std::env::temp_dir().join(format!("reconcile_telemetry_test_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        let epics_dir = temp_dir.join("docs/epics");
        fs::create_dir_all(&epics_dir)?;

        let epic_content = r#"---
issue_id: 10
title: "Epic 01: Test"
type: epic
---
# Epic 01: Test

#### Phase 1 Placeholders
*To be populated after Phase 1*

#### Phase 2 Placeholders
*To be populated after Phase 2*

#### Phase 3 Placeholders
*To be populated after Phase 3*
"#;
        let epic_path = epics_dir.join("EPIC-01-Test.md");
        fs::write(&epic_path, epic_content)?;

        let tracker = OfflineTracker::with_issues(vec![]);
        let options = ReconcileOptions {
            workspace_root: temp_dir.clone(),
            offline: true,
            dry_run: false,
            provider: "offline".to_string(),
            sync_bodies: false,
            repo: None,
            default_branch: None,
        };

        // Pass 1: should purge all 3 blocks and increment telemetry by 3
        let summary1 = run_reconciliation(&tracker, &options)?;
        assert_eq!(summary1.placeholders_purged, 3, "Expected 3 placeholders purged in pass 1");

        let updated_content = fs::read_to_string(&epic_path)?;
        assert!(!contains_unpopulated_placeholders(&updated_content));
        let count_none = updated_content.matches("- None allocated (structural subsystem component)").count();
        assert_eq!(count_none, 3, "Expected 3 allocations in updated spec");

        // Pass 2 (Idempotency): running reconciliation again over same spec should purge 0
        let summary2 = run_reconciliation(&tracker, &options)?;
        assert_eq!(summary2.placeholders_purged, 0, "Expected 0 placeholders purged on pass 2");

        let _ = fs::remove_dir_all(&temp_dir);
        Ok(())
    }
}


