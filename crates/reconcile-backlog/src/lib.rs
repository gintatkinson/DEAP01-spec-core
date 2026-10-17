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
    pub repo: Option<String>,
    pub default_branch: Option<String>,
}

impl Default for ReconcileOptions {
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
/// and contain zero unresolved placeholder tokens before publishing to the tracker.
///
/// Preconditions: `issue_index` contains live issues from the tracker.
/// Postconditions: Returns Ok(()) if all checklist items are valid, Err otherwise.
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


