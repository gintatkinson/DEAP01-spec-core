//! Pure Rust Backlog Reconciler for DEAP MBSE Framework.
//!
//! Realises: [Issue Tracker as Canonical SSOT, Zero Fallback Placeholders, Automated Spec Reconciliation]

pub mod checklist;
pub mod create_issue;
pub mod frontmatter;
pub mod tracker;

use checklist::{clean_unpopulated_placeholders, contains_unpopulated_placeholders, reconcile_checklist_content};
use frontmatter::{parse_frontmatter, update_frontmatter_issue_id, SpecFrontmatter};
use tracker::{IssueIndex, TrackerClient};

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Represents a loaded specification document.
#[derive(Debug, Clone)]
pub struct LoadedSpec {
    pub path: PathBuf,
    pub rel_path: String,
    pub frontmatter: SpecFrontmatter,
    pub content: String,
}

/// Configuration options for the reconciler.
#[derive(Debug, Clone)]
pub struct ReconcileOptions {
    pub workspace_root: PathBuf,
    pub offline: bool,
    pub dry_run: bool,
    pub provider: String,
    pub sync_bodies: bool,
}

/// Result summary of reconciliation execution.
#[derive(Debug, Clone, Default)]
pub struct ReconcileSummary {
    pub total_specs_scanned: usize,
    pub frontmatters_updated: usize,
    pub checklists_updated: usize,
    pub placeholders_purged: usize,
    pub remote_issues_synced: usize,
}

/// Discovers and loads all markdown specs in standard docs directories.
pub fn load_specifications(workspace_root: &Path) -> Vec<LoadedSpec> {
    let spec_subdirs = ["docs/epics", "docs/features", "docs/user-stories", "docs/use-cases"];
    let mut specs = Vec::new();

    for subdir in &spec_subdirs {
        let dir = workspace_root.join(subdir);
        if !dir.is_dir() {
            continue;
        }

        for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
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

    // Step 2: Frontmatter reconciliation and placeholder purge
    for spec in &mut specs {
        let mut modified = false;

        // Check for prohibited unpopulated placeholders
        if contains_unpopulated_placeholders(&spec.content) {
            spec.content = clean_unpopulated_placeholders(&spec.content);
            modified = true;
            summary.placeholders_purged += 1;
        }

        // Match spec to tracker issue
        if let Some(issue) = issue_index.find_issue(spec.frontmatter.issue_id, &spec.frontmatter.title) {
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

        // Step 4: Remote sync if enabled
        if options.sync_bodies && !options.offline && !options.dry_run {
            if let Some(issue_id) = spec.frontmatter.issue_id {
                tracker.update_issue_body(issue_id, &spec.content)?;
                summary.remote_issues_synced += 1;
            }
        }
    }

    Ok(summary)
}
