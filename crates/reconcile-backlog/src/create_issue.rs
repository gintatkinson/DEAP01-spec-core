//! Idempotent gated issue creation and reopening for DEAP MBSE Framework.
//!
//! Realises: [Issue Tracker as Canonical SSOT, Closed-Loop Payload Verification]

use std::fs;
use std::path::Path;

use crate::tracker::{normalize_title, GitHubTracker, IssueIndex, OfflineTracker, TrackerClient};

/// CLI arguments for `create-issue` subcommand.
#[derive(clap::Args, Debug, Clone)]
pub struct CreateIssueArgs {
    /// Issue body markdown file
    #[arg(long = "body-file")]
    pub body_file: Option<String>,

    /// Label to apply (e.g. epic, feature, user-story, use-case, bug)
    #[arg(long = "label")]
    pub label: Option<String>,

    /// Issue title
    #[arg(long = "title")]
    pub title: Option<String>,

    /// Target repository (e.g. owner/repo)
    #[arg(long = "repo")]
    pub repo: Option<String>,

    /// Optional file containing analysis to post if reopening an existing closed issue
    #[arg(long = "reopen-with-analysis")]
    pub reopen_with_analysis: Option<String>,

    /// Issue tracker provider (e.g. github, offline)
    #[arg(long = "provider", default_value = "github")]
    pub provider: String,

    /// Positional arguments for positional CLI invocation:
    /// <body-file> <label> <title> [repo]
    #[arg(value_name = "ARGS")]
    pub positional: Vec<String>,
}

/// Resolved parameters after coalescing flags and positional arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedCreateIssueParams {
    pub body_file: String,
    pub label: String,
    pub title: String,
    pub repo: Option<String>,
    pub reopen_with_analysis: Option<String>,
    pub provider: String,
}

impl CreateIssueArgs {
    /// Resolves flags and positional arguments, honoring explicit flags first
    /// and then falling back to positional arguments in order.
    pub fn resolve(&self) -> Result<ResolvedCreateIssueParams, String> {
        let mut pos_idx = 0;

        let body_file = if let Some(ref bf) = self.body_file {
            bf.clone()
        } else if let Some(p) = self.positional.get(pos_idx) {
            pos_idx += 1;
            p.clone()
        } else {
            return Err("Missing required body file. Specify via --body-file <FILE> or positional argument <body-file>".to_string());
        };

        let label = if let Some(ref l) = self.label {
            l.clone()
        } else if let Some(p) = self.positional.get(pos_idx) {
            pos_idx += 1;
            p.clone()
        } else {
            return Err("Missing required label. Specify via --label <LABEL> or positional argument <label>".to_string());
        };

        let title = if let Some(ref t) = self.title {
            t.clone()
        } else if let Some(p) = self.positional.get(pos_idx) {
            pos_idx += 1;
            p.clone()
        } else {
            return Err("Missing required title. Specify via --title <TITLE> or positional argument <title>".to_string());
        };

        let repo = self
            .repo
            .clone()
            .or_else(|| self.positional.get(pos_idx).cloned());

        Ok(ResolvedCreateIssueParams {
            body_file,
            label,
            title,
            repo,
            reopen_with_analysis: self.reopen_with_analysis.clone(),
            provider: self.provider.clone(),
        })
    }
}

/// Validates issue body file preconditions:
/// 1. File exists on filesystem
/// 2. File size is non-empty
/// 3. Zero unreplaced {{REQUIRED_*}} escape tokens
/// 4. Balanced code fences
pub fn validate_body_file(body_file: &str) -> Result<String, String> {
    let path = Path::new(body_file);
    if !path.is_file() {
        return Err(format!("FATAL: Body file not found: {}", body_file));
    }

    let metadata = fs::metadata(path)
        .map_err(|e| format!("FATAL: Cannot read metadata for {}: {}", body_file, e))?;

    if metadata.len() == 0 {
        return Err(format!("FATAL: Body file is empty: {}", body_file));
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("FATAL: Cannot read body file {}: {}", body_file, e))?;

    if content.trim().is_empty() {
        return Err(format!("FATAL: Body file is empty: {}", body_file));
    }

    if content.contains("{{REQUIRED_") {
        return Err(format!(
            "FATAL: Unreplaced {{{{REQUIRED_*}}}} escape token detected in: {}",
            body_file
        ));
    }

    // Check for unreplaced checklist placeholder tokens (Issue #223 blocking registration gate)
    for line in content.lines() {
        let trimmed = line.trim_start();
        let is_checkbox = if let Some(after_dash) = trimmed.strip_prefix('-') {
            let after_dash_trimmed = after_dash.trim_start();
            after_dash_trimmed.starts_with("[ ]")
                || after_dash_trimmed.starts_with("[x]")
                || after_dash_trimmed.starts_with("[X]")
        } else {
            false
        };

        if is_checkbox
            && (trimmed.contains("#[IssueID]")
                || trimmed.contains("#[FeatureIssueID]")
                || trimmed.contains("#[UseCaseIssueID]")
                || trimmed.contains("#[StoryIssueID]")
                || trimmed.contains("#[EpicIssueID]"))
        {
            return Err(format!(
                "FATAL: Unresolved checklist placeholder token detected in: {}. Sibling specifications must be registered on the tracker before publishing.",
                body_file
            ));
        }
    }

    let fence_count = content.lines().filter(|l| l.contains("```")).count();
    if fence_count % 2 != 0 {
        return Err(format!(
            "FATAL: Unbalanced code fences in: {} (found {} fences)",
            body_file, fence_count
        ));
    }

    Ok(content)
}

/// Validates that the reopen analysis file exists and reads its content.
pub fn validate_reopen_analysis_file(reopen_file: &str) -> Result<String, String> {
    let path = Path::new(reopen_file);
    if !path.is_file() {
        return Err(format!("FATAL: Reopen analysis file not found: {}", reopen_file));
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("FATAL: Cannot read reopen analysis file {}: {}", reopen_file, e))?;

    Ok(content)
}

/// Action outcome from executing issue creation or idempotency check.
#[derive(Debug, Clone, PartialEq)]
pub enum IssueActionOutcome {
    Created { number: u64, url: String },
    AlreadyOpen { number: u64, url: String },
    Reopened { number: u64, url: String },
}

/// Executes duplicate search, reopening, or issue creation against a TrackerClient.
pub fn run_create_issue<T: TrackerClient>(
    tracker: &T,
    params: &ResolvedCreateIssueParams,
) -> Result<IssueActionOutcome, String> {
    // 1. Precondition validation
    let body_content = validate_body_file(&params.body_file)?;

    let reopen_analysis_content = if let Some(ref reopen_file) = params.reopen_with_analysis {
        Some(validate_reopen_analysis_file(reopen_file)?)
    } else {
        None
    };

    // 2. Idempotency & duplicate search
    let issues = tracker.fetch_all_issues()?;
    let issue_index = IssueIndex::build(issues.clone());

    // Enforce Issue #223 blocking registration gate against live tracker index
    crate::validate_checklist_publish_integrity(&params.body_file, &body_content, &issue_index)?;

    let norm_target = normalize_title(&params.title);
    let target_type = crate::tracker::canonicalize_spec_type(&params.label);

    let existing = issue_index
        .find_issue_for_type(None, Some(&params.label), &params.title)
        .cloned()
        .or_else(|| {
            issues.iter().find(|i| {
                let matches_type = i
                    .labels
                    .iter()
                    .any(|l| crate::tracker::canonicalize_spec_type(l.name()) == target_type)
                    || crate::tracker::infer_spec_type_from_title(&i.title).as_deref()
                        == Some(&target_type);
                if !matches_type {
                    return false;
                }
                let norm_existing = normalize_title(&i.title);
                (!norm_target.is_empty() && norm_existing == norm_target)
                    || i.title.trim().eq_ignore_ascii_case(params.title.trim())
            }).cloned()
        });

    if let Some(existing_issue) = existing {
        let is_closed = existing_issue.state.eq_ignore_ascii_case("closed");
        let issue_url = existing_issue.url.clone().unwrap_or_else(|| {
            format!("https://github.com/issues/{}", existing_issue.number)
        });

        if is_closed {
            // Reopen and post analysis comment if provided
            tracker.reopen_issue(
                existing_issue.number,
                reopen_analysis_content.as_deref(),
                params.repo.as_deref(),
            )?;
            Ok(IssueActionOutcome::Reopened {
                number: existing_issue.number,
                url: issue_url,
            })
        } else {
            // Existing issue is already open: report existing issue number and URL, exit 0
            Ok(IssueActionOutcome::AlreadyOpen {
                number: existing_issue.number,
                url: issue_url,
            })
        }
    } else {
        // New issue: ensure label exists, create issue, report issue number and URL, exit 0
        tracker.ensure_label_exists(&params.label, params.repo.as_deref())?;

        let body_path = Path::new(&params.body_file);
        let candidate_root = body_path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .unwrap_or_else(|| Path::new("."));
        let workspace_root = if candidate_root.join("docs").is_dir() {
            candidate_root
        } else if Path::new("docs").is_dir() {
            Path::new(".")
        } else {
            candidate_root
        };

        let mut spec_path_to_issue = std::collections::BTreeMap::new();
        for spec in crate::load_specifications(workspace_root) {
            if let Some(id) = spec.frontmatter.issue_id.or_else(|| {
                issue_index
                    .find_issue_for_type(None, Some(&spec.frontmatter.spec_type), &spec.frontmatter.title)
                    .map(|i| i.number)
            }) {
                spec_path_to_issue.insert(spec.rel_path.clone(), id);
                if let Some(stripped) = spec.rel_path.strip_prefix("docs/") {
                    spec_path_to_issue.insert(stripped.to_string(), id);
                }
                if let Some(fname) = Path::new(&spec.rel_path).file_name().and_then(|f| f.to_str()) {
                    spec_path_to_issue.insert(fname.to_string(), id);
                }
            }
        }

        let repo_slug = params
            .repo
            .clone()
            .or_else(|| crate::detect_git_repo(workspace_root));
        let branch = crate::detect_git_branch(workspace_root).unwrap_or_else(|| "main".to_string());

        let projected_body = crate::project_body_for_tracker(
            &body_content,
            repo_slug.as_deref(),
            &branch,
            &spec_path_to_issue,
        );

        let (number, url) = tracker.create_issue(
            &params.title,
            &projected_body,
            &params.label,
            params.repo.as_deref(),
        )?;
        Ok(IssueActionOutcome::Created { number, url })
    }
}

/// Top-level coordinator for `create-issue` command execution.
pub fn execute_create_issue(args: &CreateIssueArgs) -> Result<(), String> {
    let params = args.resolve()?;

    let provider_norm = params.provider.trim().to_lowercase();
    let outcome = match provider_norm.as_str() {
        "github" | "auto" => {
            let tracker = if let Some(ref r) = params.repo {
                GitHubTracker::with_repo(r.clone())
            } else {
                GitHubTracker::new()
            };
            run_create_issue(&tracker, &params)?
        }
        "offline" => {
            let tracker = OfflineTracker::new();
            run_create_issue(&tracker, &params)?
        }
        other => {
            return Err(format!(
                "Unsupported issue tracker provider '{}'. Supported providers: github, offline",
                other
            ));
        }
    };

    match outcome {
        IssueActionOutcome::Created { number, url } => {
            println!("[CREATED] Issue #{} created: {}", number, url);
            println!("{}", url);
        }
        IssueActionOutcome::AlreadyOpen { number, url } => {
            println!("[IDEMPOTENT] Issue #{} already open: {}", number, url);
            println!("{}", url);
        }
        IssueActionOutcome::Reopened { number, url } => {
            println!("[REOPEN] Issue #{} reopened: {}", number, url);
            println!("{}", url);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::{TrackerIssue, TrackerLabel};
    use std::io::Write;

    struct TempFile {
        path: std::path::PathBuf,
    }

    impl TempFile {
        fn new(name: &str, content: &str) -> Result<Self, Box<dyn std::error::Error>> {
            let rand_id = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
            let path = std::env::temp_dir().join(format!("{}_{}.md", name, rand_id));
            let mut file = fs::File::create(&path)?;
            file.write_all(content.as_bytes())?;
            Ok(Self { path })
        }

        fn path_str(&self) -> Result<&str, Box<dyn std::error::Error>> {
            Ok(self.path.to_str().ok_or("temp file path is not valid UTF-8")?)
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    #[test]
    fn test_args_resolution_flags_and_positional() -> Result<(), Box<dyn std::error::Error>> {
        // Flag-based
        let args_flags = CreateIssueArgs {
            body_file: Some("body.md".to_string()),
            label: Some("bug".to_string()),
            title: Some("My Title".to_string()),
            repo: Some("org/repo".to_string()),
            reopen_with_analysis: None,
            provider: "github".to_string(),
            positional: Vec::new(),
        };
        let resolved = args_flags.resolve()?;
        assert_eq!(resolved.body_file, "body.md");
        assert_eq!(resolved.label, "bug");
        assert_eq!(resolved.title, "My Title");
        assert_eq!(resolved.repo, Some("org/repo".to_string()));

        // Positional-based
        let args_pos = CreateIssueArgs {
            body_file: None,
            label: None,
            title: None,
            repo: None,
            reopen_with_analysis: None,
            provider: "github".to_string(),
            positional: vec![
                "pos_body.md".to_string(),
                "feature".to_string(),
                "Pos Title".to_string(),
                "pos/repo".to_string(),
            ],
        };
        let resolved_pos = args_pos.resolve()?;
        assert_eq!(resolved_pos.body_file, "pos_body.md");
        assert_eq!(resolved_pos.label, "feature");
        assert_eq!(resolved_pos.title, "Pos Title");
        assert_eq!(resolved_pos.repo, Some("pos/repo".to_string()));
        Ok(())
    }

    #[test]
    fn test_validate_body_file_errors() -> Result<(), Box<dyn std::error::Error>> {
        // Empty file
        let empty_tf = TempFile::new("empty", "")?;
        let err = validate_body_file(empty_tf.path_str()?).err().ok_or("expected empty-file rejection")?;
        assert!(err.contains("empty"));

        // Unbalanced fences
        let fence_tf = TempFile::new("fence", "```rust\nlet a = 1;")?;
        let err = validate_body_file(fence_tf.path_str()?).err().ok_or("expected unbalanced-fence rejection")?;
        assert!(err.contains("Unbalanced code fences"));

        // Unreplaced token
        let token_tf = TempFile::new("token", "Sample {{REQUIRED_FOO}} here\n```\nblock\n```\n")?;
        let err = validate_body_file(token_tf.path_str()?).err().ok_or("expected unreplaced-token rejection")?;
        assert!(err.contains("Unreplaced {{REQUIRED_*}} escape token"));

        // Unresolved checklist placeholder token probe (Issue #223 RED phase)
        let ph_tf = TempFile::new(
            "placeholder_checklist",
            "# Epic\n## 2. Requirements & Checklist\n- [ ] #[IssueID] - Feature 01\n```\nblock\n```\n",
        )?;
        let err = validate_body_file(ph_tf.path_str()?).err().ok_or("expected placeholder-token rejection")?;
        assert!(err.contains("Unresolved checklist placeholder token"));

        // Multi-space checkbox prefix with unreplaced placeholder
        let ph_multi = TempFile::new("multi_space", "# Epic\n-   [ ] #[IssueID] - Sibling\n```\nblock\n```\n")?;
        let err = validate_body_file(ph_multi.path_str()?).err().ok_or("expected multi-space rejection")?;
        assert!(err.contains("Unresolved checklist placeholder token"));

        // Tab checkbox prefix with unreplaced placeholder
        let ph_tab = TempFile::new("tab_space", "# Epic\n-\t[x] #[StoryIssueID] - Sibling\n```\nblock\n```\n")?;
        let err = validate_body_file(ph_tab.path_str()?).err().ok_or("expected tab rejection")?;
        assert!(err.contains("Unresolved checklist placeholder token"));

        // Indented multi-space checkbox prefix
        let ph_indent = TempFile::new("indent_space", "# Epic\n   -  [X] #[UseCaseIssueID] - Sibling\n```\nblock\n```\n")?;
        let err = validate_body_file(ph_indent.path_str()?).err().ok_or("expected indented rejection")?;
        assert!(err.contains("Unresolved checklist placeholder token"));

        // Valid file
        let valid_tf = TempFile::new("valid", "# Valid\n```rust\nlet a = 1;\n```\nAll good.")?;
        let content = validate_body_file(valid_tf.path_str()?)?;
        assert!(content.contains("# Valid"));
        Ok(())
    }

    struct SpyTracker {
        inner: OfflineTracker,
        reopen_calls: std::sync::Mutex<Vec<(u64, Option<String>, Option<String>)>>,
    }

    impl SpyTracker {
        fn with_issues(issues: Vec<TrackerIssue>) -> Self {
            Self {
                inner: OfflineTracker::with_issues(issues),
                reopen_calls: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    impl TrackerClient for SpyTracker {
        fn fetch_all_issues(&self) -> Result<Vec<TrackerIssue>, String> {
            self.inner.fetch_all_issues()
        }

        fn update_issue_body(&self, issue_number: u64, body: &str) -> Result<(), String> {
            self.inner.update_issue_body(issue_number, body)
        }

        fn add_label(&self, issue_number: u64, label: &str) -> Result<(), String> {
            self.inner.add_label(issue_number, label)
        }

        fn create_issue(&self, title: &str, body: &str, label: &str, repo: Option<&str>) -> Result<(u64, String), String> {
            self.inner.create_issue(title, body, label, repo)
        }

        fn ensure_label_exists(&self, label: &str, repo: Option<&str>) -> Result<(), String> {
            self.inner.ensure_label_exists(label, repo)
        }

        fn reopen_issue(&self, issue_number: u64, comment: Option<&str>, repo: Option<&str>) -> Result<(), String> {
            self.reopen_calls.lock().unwrap().push((
                issue_number,
                comment.map(|s| s.to_string()),
                repo.map(|s| s.to_string()),
            ));
            self.inner.reopen_issue(issue_number, comment, repo)
        }
    }

    #[test]
    fn test_run_create_issue_blocks_unregistered_checklist_issues() -> Result<(), Box<dyn std::error::Error>> {
        let body_tf = TempFile::new(
            "unregistered_body",
            "# Epic 01\n## Checklist\n- [ ] #999 - Nonexistent Feature\n```rust\nfn f() {}\n```\n",
        )?;
        let tracker = OfflineTracker::with_issues(vec![]);
        let params = ResolvedCreateIssueParams {
            body_file: body_tf.path_str()?.to_string(),
            label: "epic".to_string(),
            title: "Epic 01: Test".to_string(),
            repo: Some("org/repo".to_string()),
            reopen_with_analysis: None,
            provider: "offline".to_string(),
        };
        let res = run_create_issue(&tracker, &params);
        assert!(res.is_err(), "Expected error for unregistered checklist issue #999");
        let err = res.unwrap_err();
        assert!(err.contains("unresolvable checklist issue reference '#999'"));
        assert!(err.contains("Publish gate blocked"));
        Ok(())
    }

    #[test]
    fn test_run_create_issue_projects_relative_links_and_persists_payload() -> Result<(), Box<dyn std::error::Error>> {
        // Create temp workspace structure: <temp_dir>/docs/features/FEAT-02.md
        let rand_id = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let ws_dir = std::env::temp_dir().join(format!("test_ws_{}", rand_id));
        let feats_dir = ws_dir.join("docs").join("features");
        fs::create_dir_all(&feats_dir)?;

        let feat_content = "---\ntitle: \"Feature 02: Baseline Verifier\"\nissue_id: 20\n---\n# Feature 02\n";
        fs::write(feats_dir.join("FEAT-02.md"), feat_content)?;

        let epic_content = "# Epic 01\n## Checklist\n- [ ] #20 - [Feature 02](../features/FEAT-02.md)\nSchema: [Model](../../schema/core.sysml)\n```\ncode\n```\n";
        let epics_dir = ws_dir.join("docs").join("epics");
        fs::create_dir_all(&epics_dir)?;
        let epic_path = epics_dir.join("EPIC-01.md");
        fs::write(&epic_path, epic_content)?;

        let existing_feat = TrackerIssue {
            number: 20,
            title: "Feature 02: Baseline Verifier".to_string(),
            state: "OPEN".to_string(),
            labels: vec![TrackerLabel::String("feature".to_string())],
            body: Some("Old body".to_string()),
            url: Some("https://github.com/org/repo/issues/20".to_string()),
        };
        let tracker = OfflineTracker::with_issues(vec![existing_feat]);

        let params = ResolvedCreateIssueParams {
            body_file: epic_path.to_str().unwrap().to_string(),
            label: "epic".to_string(),
            title: "Epic 01: Core Platform".to_string(),
            repo: Some("org/repo".to_string()),
            reopen_with_analysis: None,
            provider: "offline".to_string(),
        };

        let outcome = run_create_issue(&tracker, &params)?;
        assert_eq!(
            outcome,
            IssueActionOutcome::Created {
                number: 21,
                url: "https://github.com/org/repo/issues/21".to_string(),
            }
        );

        let all_issues = tracker.fetch_all_issues()?;
        let created = all_issues.iter().find(|i| i.number == 21).ok_or("Created issue not found")?;
        let body = created.body.as_ref().ok_or("Created issue body is None")?;

        // Must project link to canonical GitHub URL with issue #20
        assert!(body.contains("#20 - [Feature 02](https://github.com/org/repo/blob/main/docs/features/FEAT-02.md)"));
        assert!(body.contains("[Model](https://github.com/org/repo/blob/main/schema/core.sysml)"));
        assert!(!body.contains("../features/FEAT-02.md"), "Raw relative spec link must not remain");

        // Cleanup
        let _ = fs::remove_dir_all(ws_dir);
        Ok(())
    }

    #[test]
    fn test_run_create_issue_workflow_offline() -> Result<(), Box<dyn std::error::Error>> {
        let body_tf = TempFile::new("body", "# Issue Content\n```rust\nfn test() {}\n```\n")?;
        let analysis_tf = TempFile::new("analysis", "Reopen rationale with details")?;

        let existing_closed = TrackerIssue {
            number: 10,
            title: "Feature 01: Core Ingest Pipeline".to_string(),
            state: "CLOSED".to_string(),
            labels: vec![
                TrackerLabel::String("feature".to_string()),
                TrackerLabel::String("status:fixed-resolved".to_string()),
            ],
            body: Some("Old body".to_string()),
            url: Some("https://github.com/org/repo/issues/10".to_string()),
        };

        let existing_open = TrackerIssue {
            number: 20,
            title: "Feature 02: Baseline Verifier".to_string(),
            state: "OPEN".to_string(),
            labels: vec![TrackerLabel::String("feature".to_string())],
            body: Some("Old body".to_string()),
            url: Some("https://github.com/org/repo/issues/20".to_string()),
        };

        let tracker = SpyTracker::with_issues(vec![existing_closed, existing_open]);

        // 1. Reopen closed issue with analysis
        let reopen_params = ResolvedCreateIssueParams {
            body_file: body_tf.path_str()?.to_string(),
            label: "feature".to_string(),
            title: "Feature: Core Ingest Pipeline".to_string(), // normalizes to "core ingest pipeline"
            repo: Some("org/repo".to_string()),
            reopen_with_analysis: Some(analysis_tf.path_str()?.to_string()),
            provider: "offline".to_string(),
        };
        let outcome = run_create_issue(&tracker, &reopen_params)?;
        assert_eq!(
            outcome,
            IssueActionOutcome::Reopened {
                number: 10,
                url: "https://github.com/org/repo/issues/10".to_string(),
            }
        );
        let calls = tracker.reopen_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, 10);
        assert_eq!(calls[0].1.as_deref(), Some("Reopen rationale with details"));
        assert_eq!(calls[0].2.as_deref(), Some("org/repo"));
        drop(calls);

        // 2. Open issue idempotency
        let open_params = ResolvedCreateIssueParams {
            body_file: body_tf.path_str()?.to_string(),
            label: "feature".to_string(),
            title: "Feature 02: Baseline Verifier".to_string(),
            repo: Some("org/repo".to_string()),
            reopen_with_analysis: None,
            provider: "offline".to_string(),
        };
        let outcome = run_create_issue(&tracker, &open_params)?;
        assert_eq!(
            outcome,
            IssueActionOutcome::AlreadyOpen {
                number: 20,
                url: "https://github.com/org/repo/issues/20".to_string(),
            }
        );

        // 3. Create brand new issue
        let new_params = ResolvedCreateIssueParams {
            body_file: body_tf.path_str()?.to_string(),
            label: "bug".to_string(),
            title: "Bug 03: Brittle Shell Sprawl".to_string(),
            repo: Some("org/repo".to_string()),
            reopen_with_analysis: None,
            provider: "offline".to_string(),
        };
        let outcome = run_create_issue(&tracker, &new_params)?;
        assert_eq!(
            outcome,
            IssueActionOutcome::Created {
                number: 21,
                url: "https://github.com/org/repo/issues/21".to_string(),
            }
        );

        let all_issues = tracker.fetch_all_issues()?;
        let issue_10 = all_issues.iter().find(|i| i.number == 10).ok_or("issue 10 missing")?;
        assert_eq!(issue_10.state, "OPEN");
        assert!(!issue_10.is_resolved(), "Reopened issue 10 must not be resolved");
        assert!(
            issue_10.body.as_ref().map(|b| b.contains("Reopen rationale with details")).unwrap_or(false),
            "Reopened issue 10 body must contain reopen comment"
        );
        let issue_20 = all_issues.iter().find(|i| i.number == 20).ok_or("issue 20 missing")?;
        assert_eq!(issue_20.state, "OPEN");
        let issue_21 = all_issues.iter().find(|i| i.number == 21).ok_or("issue 21 missing")?;
        assert_eq!(issue_21.state, "OPEN");
        assert_eq!(issue_21.title, "Bug 03: Brittle Shell Sprawl");
        assert!(issue_21.labels.iter().any(|l| match l {
            TrackerLabel::String(s) => s == "bug",
            _ => false,
        }));
        assert!(issue_21.body.as_ref().map(|b| b.contains("# Issue Content")).unwrap_or(false));
        Ok(())
    }

    #[test]
    fn test_create_issue_does_not_collide_feature_with_existing_epic() -> Result<(), Box<dyn std::error::Error>> {
        let body_tf = TempFile::new(
            "feat_collision",
            "# Feature 01: [ConOps] Human Engineer Interface\n\nBody content\n",
        )?;

        let existing_epic = TrackerIssue {
            number: 10,
            title: "Epic 01: [ConOps] Human Engineer Interface".to_string(),
            state: "OPEN".to_string(),
            labels: vec![TrackerLabel::String("epic".to_string())],
            body: Some("Epic body".to_string()),
            url: Some("https://github.com/org/repo/issues/10".to_string()),
        };

        let tracker = OfflineTracker::with_issues(vec![existing_epic]);
        let params = ResolvedCreateIssueParams {
            body_file: body_tf.path_str()?.to_string(),
            label: "feature".to_string(),
            title: "Feature 01: [ConOps] Human Engineer Interface".to_string(),
            repo: Some("org/repo".to_string()),
            reopen_with_analysis: None,
            provider: "offline".to_string(),
        };

        let outcome = run_create_issue(&tracker, &params)?;
        match outcome {
            IssueActionOutcome::Created { number, .. } => {
                assert_ne!(number, 10, "Feature must be created as new issue, not collide with Epic #10");
            }
            IssueActionOutcome::AlreadyOpen { number, .. } => {
                panic!("Feature must NOT be treated as AlreadyOpen due to Epic #10 collision! (got issue #{})", number);
            }
            IssueActionOutcome::Reopened { number, .. } => {
                panic!("Feature must NOT be reopened! (got issue #{})", number);
            }
        }
        Ok(())
    }
}

