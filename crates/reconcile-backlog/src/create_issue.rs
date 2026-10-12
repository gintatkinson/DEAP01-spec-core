//! Idempotent gated issue creation and reopening for DEAP MBSE Framework.
//!
//! Realises: [Issue Tracker as Canonical SSOT, Closed-Loop Payload Verification]

use std::fs;
use std::path::Path;

use crate::tracker::{normalize_title, GitHubTracker, OfflineTracker, TrackerClient};

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

        let repo = if let Some(ref r) = self.repo {
            Some(r.clone())
        } else if let Some(p) = self.positional.get(pos_idx) {
            Some(p.clone())
        } else {
            None
        };

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
    let norm_target = normalize_title(&params.title);

    let existing = issues.iter().find(|i| {
        let norm_existing = normalize_title(&i.title);
        (!norm_target.is_empty() && norm_existing == norm_target)
            || i.title.trim().eq_ignore_ascii_case(params.title.trim())
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
        let (number, url) = tracker.create_issue(
            &params.title,
            &body_content,
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
        fn new(name: &str, content: &str) -> Self {
            let rand_id = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
            let path = std::env::temp_dir().join(format!("{}_{}.md", name, rand_id));
            let mut file = fs::File::create(&path).unwrap();
            file.write_all(content.as_bytes()).unwrap();
            Self { path }
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    #[test]
    fn test_args_resolution_flags_and_positional() {
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
        let resolved = args_flags.resolve().unwrap();
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
        let resolved_pos = args_pos.resolve().unwrap();
        assert_eq!(resolved_pos.body_file, "pos_body.md");
        assert_eq!(resolved_pos.label, "feature");
        assert_eq!(resolved_pos.title, "Pos Title");
        assert_eq!(resolved_pos.repo, Some("pos/repo".to_string()));
    }

    #[test]
    fn test_validate_body_file_errors() {
        // Empty file
        let empty_tf = TempFile::new("empty", "");
        let err = validate_body_file(empty_tf.path.to_str().unwrap()).unwrap_err();
        assert!(err.contains("empty"));

        // Unbalanced fences
        let fence_tf = TempFile::new("fence", "```rust\nlet a = 1;");
        let err = validate_body_file(fence_tf.path.to_str().unwrap()).unwrap_err();
        assert!(err.contains("Unbalanced code fences"));

        // Unreplaced token
        let token_tf = TempFile::new("token", "Sample {{REQUIRED_FOO}} here\n```\nblock\n```\n");
        let err = validate_body_file(token_tf.path.to_str().unwrap()).unwrap_err();
        assert!(err.contains("Unreplaced {{REQUIRED_*}} escape token"));

        // Valid file
        let valid_tf = TempFile::new("valid", "# Valid\n```rust\nlet a = 1;\n```\nAll good.");
        let content = validate_body_file(valid_tf.path.to_str().unwrap()).unwrap();
        assert!(content.contains("# Valid"));
    }

    #[test]
    fn test_run_create_issue_workflow_offline() {
        let body_tf = TempFile::new("body", "# Issue Content\n```rust\nfn test() {}\n```\n");
        let analysis_tf = TempFile::new("analysis", "Reopen rationale with details");

        let existing_closed = TrackerIssue {
            number: 10,
            title: "Feature 01: Core Ingest Pipeline".to_string(),
            state: "CLOSED".to_string(),
            labels: vec![TrackerLabel::String("feature".to_string())],
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

        let tracker = OfflineTracker::with_issues(vec![existing_closed, existing_open]);

        // 1. Reopen closed issue with analysis
        let reopen_params = ResolvedCreateIssueParams {
            body_file: body_tf.path.to_str().unwrap().to_string(),
            label: "feature".to_string(),
            title: "Feature: Core Ingest Pipeline".to_string(), // normalizes to "core ingest pipeline"
            repo: Some("org/repo".to_string()),
            reopen_with_analysis: Some(analysis_tf.path.to_str().unwrap().to_string()),
            provider: "offline".to_string(),
        };
        let outcome = run_create_issue(&tracker, &reopen_params).unwrap();
        assert_eq!(
            outcome,
            IssueActionOutcome::Reopened {
                number: 10,
                url: "https://github.com/org/repo/issues/10".to_string(),
            }
        );

        // 2. Open issue idempotency
        let open_params = ResolvedCreateIssueParams {
            body_file: body_tf.path.to_str().unwrap().to_string(),
            label: "feature".to_string(),
            title: "Feature 02: Baseline Verifier".to_string(),
            repo: Some("org/repo".to_string()),
            reopen_with_analysis: None,
            provider: "offline".to_string(),
        };
        let outcome = run_create_issue(&tracker, &open_params).unwrap();
        assert_eq!(
            outcome,
            IssueActionOutcome::AlreadyOpen {
                number: 20,
                url: "https://github.com/org/repo/issues/20".to_string(),
            }
        );

        // 3. Create brand new issue
        let new_params = ResolvedCreateIssueParams {
            body_file: body_tf.path.to_str().unwrap().to_string(),
            label: "bug".to_string(),
            title: "Bug 03: Brittle Shell Sprawl".to_string(),
            repo: Some("org/repo".to_string()),
            reopen_with_analysis: None,
            provider: "offline".to_string(),
        };
        let outcome = run_create_issue(&tracker, &new_params).unwrap();
        assert_eq!(
            outcome,
            IssueActionOutcome::Created {
                number: 21,
                url: "https://github.com/org/repo/issues/21".to_string(),
            }
        );
    }
}

