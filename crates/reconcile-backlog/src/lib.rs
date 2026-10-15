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

/// Translates relative filesystem markdown links in a spec body to tracker-compatible links:
/// - Relative spec links (`../epics/EPIC-03-*.md`) -> Issue references (`#<id>` or canonical issue links)
/// - Relative schema links (`../../schema/...`) -> GitHub blob URLs (`https://github.com/<repo>/blob/<branch>/schema/...`)
/// - Relative docs links (`../../docs/...`) -> GitHub blob URLs (`https://github.com/<repo>/blob/<branch>/docs/...`)
pub fn project_body_for_tracker(
    body: &str,
    repo: Option<&str>,
    default_branch: &str,
    spec_path_to_issue: &BTreeMap<String, u64>,
) -> String {
    let mut projected = body.to_string();

    // 1. Translate relative spec links to issue citations
    let mut replacements: Vec<(String, String)> = Vec::new();

    for (spec_path, issue_num) in spec_path_to_issue {
        let issue_ref = format!("#{}", issue_num);

        let file_name = Path::new(spec_path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("");

        replacements.push((format!("../../{}", spec_path), issue_ref.clone()));
        replacements.push((format!("../{}", spec_path), issue_ref.clone()));

        if let Some(stripped) = spec_path.strip_prefix("docs/") {
            replacements.push((format!("../../{}", stripped), issue_ref.clone()));
            replacements.push((format!("../{}", stripped), issue_ref.clone()));
        } else {
            replacements.push((format!("../../docs/{}", spec_path), issue_ref.clone()));
            replacements.push((format!("../docs/{}", spec_path), issue_ref.clone()));
        }

        if !file_name.is_empty() {
            replacements.push((format!("../epics/{}", file_name), issue_ref.clone()));
            replacements.push((format!("../features/{}", file_name), issue_ref.clone()));
            replacements.push((format!("../user-stories/{}", file_name), issue_ref.clone()));
            replacements.push((format!("../use-cases/{}", file_name), issue_ref.clone()));
            replacements.push((format!("../../epics/{}", file_name), issue_ref.clone()));
            replacements.push((format!("../../features/{}", file_name), issue_ref.clone()));
            replacements.push((format!("../../user-stories/{}", file_name), issue_ref.clone()));
            replacements.push((format!("../../use-cases/{}", file_name), issue_ref.clone()));
            replacements.push((format!("./{}", file_name), issue_ref.clone()));
        }
    }

    // Sort by pattern length descending to avoid partial matches
    replacements.sort_by_key(|r| std::cmp::Reverse(r.0.len()));
    replacements.dedup_by(|a, b| a.0 == b.0);

    for (pattern, replacement) in replacements {
        projected = projected.replace(&pattern, &replacement);
    }

    // 2. Translate relative schema and docs links to absolute GitHub blob URLs
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
            !body.contains("../../schema/model.sysml"),
            "Projected issue body still contains relative filesystem link '../../schema/model.sysml'!"
        );
        assert!(
            body.contains("[EPIC-03-SystemVision](#80)"),
            "Projected body should contain translated citation '[EPIC-03-SystemVision](#80)', found:\n{}",
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

        assert!(projected.contains("[EPIC-03-SystemVision](#80)"));
        assert!(projected.contains("[Feature 01: [ConOps] Human Engineer Interface](#75)"));
        assert!(projected.contains("[Feature 02: [Core] Baseline Verifier](#76)"));
        assert!(projected.contains("https://github.com/gintatkinson/dumpiler-01/blob/main/schema/subsystems/system_vision/behavior.sysml"));
        assert!(projected.contains("https://github.com/gintatkinson/dumpiler-01/blob/main/docs/architecture/DEAP_MASTER_ARCHITECTURE.md"));
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
        assert_eq!(projected, "[EPIC-01](#10) and [schema](../../schema/model.sysml)");
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
}


