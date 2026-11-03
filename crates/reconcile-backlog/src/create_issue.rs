//! Safe `create-issue` subcommand implementation with strict precondition checks.
//!
//! Replaces legacy shell-script wrappers by natively enforcing:
//! 1. Markdown body file existence, non-empty content, and structural AST validity (`deap_core::markdown_ast`).
//! 2. Zero unpopulated template placeholders (`(To be created)`, `TBD`, `TODO`).
//! 3. Duplicate issue detection across OPEN and CLOSED issues in the canonical tracker.
//! 4. Safe reopen workflow requiring a validated `--reopen-with-analysis <file>` document.
//!
//! Realises: [REQ-RECONCILE-BACKLOG-CREATE-ISSUE, Issue Tracker as Canonical SSOT, Zero Fallback Placeholders]

use crate::checklist::contains_unpopulated_placeholders;
use crate::tracker::{normalize_title, GitHubTracker, Issue, IssueTracker};
use crate::ReconcileError;
use deap_core::markdown_ast::parse_markdown_ast;
use std::fs;
use std::path::{Path, PathBuf};

/// [DOC-TIER: A] Command-line arguments for the `create-issue` subcommand.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-CREATE-ISSUE-CREATEISSUEARGS]
/// Contract: Accepts named flags (`--body-file`, `--label`, `--title`) or positional arguments.
/// Bounds & Faults: Time O(A), Space O(A) over CLI arguments; validated by `normalize_args`.
#[derive(Debug, Clone)]
pub struct CreateIssueArgs {
    /// Path to the Markdown file containing the issue body (flag form: `--body-file`)
    pub body_file: Option<String>,

    /// Comma-separated or single label (flag form: `--label`)
    pub label: Option<String>,

    /// Issue title (flag form: `--title`)
    pub title: Option<String>,

    /// Target repository (e.g. `owner/repo`)
    pub repo: Option<String>,

    /// Optional path to root-cause analysis Markdown file to reopen an existing CLOSED issue
    pub reopen_with_analysis: Option<String>,

    /// Tracker provider (`github` or `gitlab`, defaults to `github`)
    pub provider: String,

    /// Positional arguments fallback: `[<body_file> <label> <title>...]`
    pub positional: Vec<String>,
}

impl Default for CreateIssueArgs {
    /// Returns default empty `CreateIssueArgs` (`[REQ-RECONCILE-BACKLOG-CREATE-ISSUE-DEFAULT]`).
    /// Realises: [REQ-RECONCILE-BACKLOG-CREATE-ISSUE-DEFAULT]
    fn default() -> Self {
        Self {
            body_file: None,
            label: None,
            title: None,
            repo: None,
            reopen_with_analysis: None,
            provider: "github".to_string(),
            positional: Vec::new(),
        }
    }
}

/// [DOC-TIER: A] Normalized, validated arguments for `create-issue`.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-CREATE-ISSUE-NORMALIZEDCREATEISSUEARGS]
/// Contract: Guarantees `body_file`, `labels`, and `title` are non-empty.
/// Bounds & Faults: Time O(1), Space O(L) over label and path fields.
#[derive(Debug, Clone)]
pub struct NormalizedCreateIssueArgs {
    pub body_file: PathBuf,
    pub labels: Vec<String>,
    pub title: String,
    pub repo: Option<String>,
    pub reopen_with_analysis: Option<PathBuf>,
}

/// [DOC-TIER: A] Normalizes `CreateIssueArgs` from either named flags or positional arguments.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-CREATE-ISSUE-NORMALIZE-ARGS]
/// Contract: Returns `Ok(NormalizedCreateIssueArgs)` or `Err(ReconcileError::PreconditionFailed)`.
/// Bounds & Faults: Time O(P + L), Space O(P + L) over positional and label inputs.
pub fn normalize_args(
    args: &CreateIssueArgs,
) -> Result<NormalizedCreateIssueArgs, ReconcileError> {
    let (raw_body, raw_label, raw_title) =
        if args.body_file.is_some() || args.label.is_some() || args.title.is_some() {
            let b = args.body_file.clone().ok_or_else(|| {
                ReconcileError::PreconditionFailed(
                    "Missing required --body-file argument for create-issue".to_string(),
                )
            })?;
            let l = args.label.clone().ok_or_else(|| {
                ReconcileError::PreconditionFailed(
                    "Missing required --label argument for create-issue".to_string(),
                )
            })?;
            let t = args.title.clone().ok_or_else(|| {
                ReconcileError::PreconditionFailed(
                    "Missing required --title argument for create-issue".to_string(),
                )
            })?;
            (b, l, t)
        } else if let (Some(b), Some(l), Some(t_slice)) = (
            args.positional.first(),
            args.positional.get(1),
            args.positional.get(2..).filter(|s| !s.is_empty()),
        ) {
            (b.clone(), l.clone(), t_slice.join(" "))
        } else {
            return Err(ReconcileError::PreconditionFailed(
                "Usage: reconcile-backlog create-issue --body-file <path> --label <label> --title <title> (or positional: <body_file> <label> <title>)".to_string(),
            ));
        };

    let labels: Vec<String> = raw_label
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if labels.is_empty() {
        return Err(ReconcileError::PreconditionFailed(
            "At least one non-empty label must be provided".to_string(),
        ));
    }

    let title = raw_title.trim().to_string();
    if title.is_empty() {
        return Err(ReconcileError::PreconditionFailed(
            "Issue title cannot be empty".to_string(),
        ));
    }

    Ok(NormalizedCreateIssueArgs {
        body_file: PathBuf::from(raw_body),
        labels,
        title,
        repo: args.repo.clone(),
        reopen_with_analysis: args.reopen_with_analysis.as_ref().map(PathBuf::from),
    })
}

/// [DOC-TIER: A] Validates a Markdown body file using `deap_core::markdown_ast` before issue creation or reopen.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-CREATE-ISSUE-VALIDATE-BODY-FILE]
/// Contract: Rejects missing/empty files, unpopulated placeholders, unclosed fences, or missing AST headings.
/// Bounds & Faults: Time O(N), Space O(N) where N is the byte length of `path`.
pub fn validate_body_file(path: &Path) -> Result<String, ReconcileError> {
    if !path.exists() || !path.is_file() {
        return Err(ReconcileError::PreconditionFailed(format!(
            "Body file does not exist or is not a regular file: {}",
            path.display()
        )));
    }

    let content = fs::read_to_string(path)?;
    let trimmed = content.trim();

    if trimmed.is_empty() {
        return Err(ReconcileError::PreconditionFailed(format!(
            "Body file is empty: {}",
            path.display()
        )));
    }

    if contains_unpopulated_placeholders(&content) {
        return Err(ReconcileError::PreconditionFailed(format!(
            "Body file contains unpopulated template placeholders ((To be created), TBD, TODO): {}",
            path.display()
        )));
    }

    let mut active_fence: Option<char> = None;
    for line in content.lines() {
        let line_trim = line.trim();
        if line_trim.starts_with("```") {
            match active_fence {
                None => active_fence = Some('`'),
                Some('`') => active_fence = None,
                Some(_) => {}
            }
        } else if line_trim.starts_with("~~~") {
            match active_fence {
                None => active_fence = Some('~'),
                Some('~') => active_fence = None,
                Some(_) => {}
            }
        }
    }

    if active_fence.is_some() {
        return Err(ReconcileError::PreconditionFailed(format!(
            "Body file contains unclosed markdown code fence: {}",
            path.display()
        )));
    }

    let ast = parse_markdown_ast(&content);
    if ast.sections.is_empty() {
        return Err(ReconcileError::PreconditionFailed(format!(
            "Body file does not appear to contain valid Markdown structure (missing headings): {}",
            path.display()
        )));
    }

    Ok(content)
}

/// Checks if an issue with the same normalized title already exists in `existing_issues`.
/// Realises: [REQ-RECONCILE-BACKLOG-CREATE-ISSUE-CHECK-DUPLICATE-ISSUE]
fn check_duplicate_issue<'a>(title: &str, existing_issues: &'a [Issue]) -> Option<&'a Issue> {
    let target_norm = normalize_title(title);
    if target_norm.is_empty() {
        return None;
    }

    existing_issues
        .iter()
        .find(|issue| normalize_title(&issue.title) == target_norm)
}

/// [DOC-TIER: A] Executes the `create-issue` workflow using the live `GitHubTracker`.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-CREATE-ISSUE-EXECUTE-CREATE-ISSUE]
/// Contract: Delegates to `execute_create_issue_with_tracker` and returns the created or reopened issue ID.
/// Bounds & Faults: Time O(N + B), Space O(N + B) over tracker issues and body file size.
pub fn execute_create_issue(args: &CreateIssueArgs) -> Result<u64, ReconcileError> {
    if args.provider != "github" {
        return Err(ReconcileError::PreconditionFailed(format!(
            "Unsupported issue tracker provider '{}'. Currently only 'github' is supported natively.",
            args.provider
        )));
    }
    let tracker = GitHubTracker::new();
    execute_create_issue_with_tracker(&tracker, args)
}

/// [DOC-TIER: A] Executes the `create-issue` workflow against an injected `IssueTracker`.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-CREATE-ISSUE-EXECUTE-CREATE-ISSUE-WITH-TRACKER]
/// Contract: Enforces body validation, duplicate blocking, and closed-issue reopen analysis requirements.
/// Bounds & Faults: Time O(N + B), Space O(N + B); returns `PreconditionFailed` on duplicate or missing analysis.
pub fn execute_create_issue_with_tracker(
    tracker: &dyn IssueTracker,
    args: &CreateIssueArgs,
) -> Result<u64, ReconcileError> {
    let norm = normalize_args(args)?;
    let body_content = validate_body_file(&norm.body_file)?;
    let existing_issues = tracker.fetch_all_issues(norm.repo.as_deref())?;

    if let Some(dup) = check_duplicate_issue(&norm.title, &existing_issues) {
        if dup.state.eq_ignore_ascii_case("OPEN") {
            return Err(ReconcileError::PreconditionFailed(format!(
                "Duplicate OPEN issue #{} ('{}') already exists. Refusing to create duplicate issue.",
                dup.number, dup.title
            )));
        }

        if let Some(ref analysis_path) = norm.reopen_with_analysis {
            let analysis_content = validate_body_file(analysis_path)?;
            println!(
                "Reopening existing CLOSED issue #{} ('{}') with root-cause analysis from {}...",
                dup.number,
                dup.title,
                analysis_path.display()
            );
            tracker.reopen_issue(dup.number, &analysis_content, norm.repo.as_deref())?;
            println!("Successfully reopened issue #{}", dup.number);
            return Ok(dup.number);
        }

        return Err(ReconcileError::PreconditionFailed(format!(
            "Matching CLOSED issue #{} ('{}') already exists. To reopen it with a regression/root-cause analysis, pass --reopen-with-analysis <markdown_file>.",
            dup.number, dup.title
        )));
    }

    if norm.reopen_with_analysis.is_some() {
        return Err(ReconcileError::PreconditionFailed(format!(
            "--reopen-with-analysis was provided, but no existing CLOSED issue matching '{}' was found.",
            norm.title
        )));
    }

    let number = tracker.create_issue(
        &norm.title,
        &body_content,
        &norm.labels,
        norm.repo.as_deref(),
    )?;
    println!("Successfully created issue #{}: {}", number, norm.title);
    Ok(number)
}
