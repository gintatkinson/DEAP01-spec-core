//! External issue tracker abstraction and synchronization.
//!
//! Realises: [Issue Tracker as Canonical SSOT, Closed-Loop Payload Verification]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::process::Command;

/// Representation of an issue from GitHub or GitLab.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackerIssue {
    pub number: u64,
    pub title: String,
    pub state: String,
    #[serde(default)]
    pub labels: Vec<TrackerLabel>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TrackerLabel {
    Object { name: String },
    String(String),
}

impl TrackerLabel {
    pub fn name(&self) -> &str {
        match self {
            TrackerLabel::Object { name } => name.as_str(),
            TrackerLabel::String(s) => s.as_str(),
        }
    }
}

impl TrackerIssue {
    /// Determines whether an issue is considered resolved or fixed.
    ///
    /// According to constitution.md:256-265, an issue is resolved when it carries `status:fixed-resolved`
    /// or `status::fixed-resolved` (or state is CLOSED).
    pub fn is_resolved(&self) -> bool {
        if self.state.eq_ignore_ascii_case("closed") {
            return true;
        }
        self.labels.iter().any(|label| is_resolved_label(label.name()))
    }
}

/// Returns true if the label represents a resolved status.
///
/// According to constitution.md:256-265, an issue is resolved when it carries `status:fixed-resolved`
/// or `status::fixed-resolved` (or `fixed-resolved`).
pub fn is_resolved_label(label: &str) -> bool {
    let l_norm = normalize_label(label);
    l_norm == "status:fixed-resolved" || l_norm == "status::fixed-resolved" || l_norm == "fixed-resolved"
}

/// Normalizes issue labels for robust matching.
pub fn normalize_label(label: &str) -> String {
    let lower = label.trim().to_lowercase();
    lower.replace(['_', ' '], "-")
}

/// Canonicalizes a specification type or label for indexing.
pub fn canonicalize_spec_type(raw: &str) -> String {
    let norm = normalize_label(raw);
    match norm.as_str() {
        "epic" | "epics" => "epic".to_string(),
        "feature" | "features" | "feat" => "feature".to_string(),
        "user-story" | "user-stories" | "user_story" | "user_stories" | "us" => "user-story".to_string(),
        "use-case" | "use-cases" | "use_case" | "use_cases" | "uc" => "use-case".to_string(),
        other => other.to_string(),
    }
}

/// Infers the canonical specification type from a title string prefix if present.
pub fn infer_spec_type_from_title(title: &str) -> Option<String> {
    let trimmed = title.trim().trim_matches(|c| c == '"' || c == '\'');
    let lower = trimmed.to_ascii_lowercase();
    const PREFIXES: &[(&str, &str)] = &[
        ("user-story", "user-story"),
        ("user story", "user-story"),
        ("user_story", "user-story"),
        ("us", "user-story"),
        ("use-case", "use-case"),
        ("use case", "use-case"),
        ("use_case", "use-case"),
        ("uc", "use-case"),
        ("feature", "feature"),
        ("feat", "feature"),
        ("epic", "epic"),
    ];
    for (prefix, canonical) in PREFIXES {
        if let Some(rem) = lower.strip_prefix(prefix) {
            if rem.is_empty()
                || rem.starts_with(' ')
                || rem.starts_with('-')
                || rem.starts_with('_')
                || rem.starts_with(':')
                || rem.starts_with(|c: char| c.is_ascii_digit())
                || (rem.starts_with('s') && (rem.len() == 1 || rem[1..].starts_with([' ', '-', '_', ':'])))
            {
                return Some((*canonical).to_string());
            }
        }
    }
    None
}

/// Canonical title normalization matching Python `reconcile_backlog.py` and `metadata.rs`.
pub fn normalize_title(title: &str) -> String {
    let trimmed = title.trim().trim_matches(|c| c == '"' || c == '\'');
    if trimmed.is_empty() {
        return String::new();
    }

    let lower = trimmed.to_ascii_lowercase();
    const PREFIX_KEYWORDS: &[&str] = &[
        "user-story", "user story", "user_story",
        "use-case", "use case", "use_case",
        "feature", "feat",
        "epic",
        "us", "uc",
    ];

    let mut matched_len = 0;
    for kw in PREFIX_KEYWORDS {
        if let Some(kw_rest) = lower.strip_prefix(kw) {
            let mut rem = kw_rest;
            let mut consumed = kw.len();

            if rem.starts_with('s') {
                rem = &rem[1..];
                consumed += 1;
            }

            if rem.starts_with(':') {
                rem = &rem[1..];
                consumed += 1;
                while rem.starts_with(' ') || rem.starts_with('\t') {
                    rem = &rem[1..];
                    consumed += 1;
                }
                matched_len = consumed;
                break;
            }

            let mut temp_rem = rem;
            let mut temp_consumed = consumed;
            while temp_rem.starts_with('-') || temp_rem.starts_with(' ') || temp_rem.starts_with('_') {
                temp_rem = &temp_rem[1..];
                temp_consumed += 1;
            }

            let digit_count = temp_rem.chars().take_while(|c| c.is_ascii_digit()).count();
            if digit_count > 0 {
                temp_rem = &temp_rem[digit_count..];
                temp_consumed += digit_count;

                while temp_rem.starts_with(' ')
                    || temp_rem.starts_with(':')
                    || temp_rem.starts_with('-')
                    || temp_rem.starts_with('_')
                {
                    temp_rem = &temp_rem[1..];
                    temp_consumed += 1;
                }
                matched_len = temp_consumed;
                break;
            }
        }
    }

    let stripped = if matched_len > 0 {
        &trimmed[matched_len..]
    } else {
        trimmed
    };

    let working = if stripped.trim().is_empty() {
        trimmed
    } else {
        stripped
    };

    let separators_replaced = working.replace('-', " ").replace('_', " ");
    let filtered: String = separators_replaced
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();

    filtered
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Interface for querying and syncing with external issue trackers.
pub trait TrackerClient {
    fn fetch_all_issues(&self) -> Result<Vec<TrackerIssue>, String>;
    fn update_issue_body(&self, issue_number: u64, body: &str) -> Result<(), String>;
    fn add_label(&self, issue_number: u64, label: &str) -> Result<(), String>;
    fn create_issue(&self, title: &str, body: &str, label: &str, repo: Option<&str>) -> Result<(u64, String), String>;
    fn ensure_label_exists(&self, label: &str, repo: Option<&str>) -> Result<(), String>;
    fn reopen_issue(&self, issue_number: u64, comment: Option<&str>, repo: Option<&str>) -> Result<(), String>;
}

#[derive(Default)]
struct OfflineState {
    issues: Vec<TrackerIssue>,
    labels: Vec<String>,
    next_id: u64,
}

/// Offline / In-memory tracker implementation.
pub struct OfflineTracker {
    state: std::sync::Mutex<OfflineState>,
}

impl OfflineTracker {
    pub fn new() -> Self {
        Self {
            state: std::sync::Mutex::new(OfflineState {
                issues: Vec::new(),
                labels: Vec::new(),
                next_id: 1,
            }),
        }
    }

    pub fn with_issues(issues: Vec<TrackerIssue>) -> Self {
        let max_id = issues.iter().map(|i| i.number).max().unwrap_or(0);
        Self {
            state: std::sync::Mutex::new(OfflineState {
                issues,
                labels: Vec::new(),
                next_id: max_id + 1,
            }),
        }
    }
}

impl Default for OfflineTracker {
    /// Constructs an empty offline tracker; equivalent to [`OfflineTracker::new`].
    fn default() -> Self {
        Self::new()
    }
}

impl TrackerClient for OfflineTracker {
    fn fetch_all_issues(&self) -> Result<Vec<TrackerIssue>, String> {
        let guard = self.state.lock().map_err(|e| e.to_string())?;
        Ok(guard.issues.clone())
    }

    fn update_issue_body(&self, issue_number: u64, body: &str) -> Result<(), String> {
        let mut guard = self.state.lock().map_err(|e| e.to_string())?;
        let issue = guard
            .issues
            .iter_mut()
            .find(|i| i.number == issue_number)
            .ok_or_else(|| format!("Offline tracker: issue #{} not found", issue_number))?;
        issue.body = Some(body.to_string());
        Ok(())
    }

    fn add_label(&self, issue_number: u64, label: &str) -> Result<(), String> {
        let mut guard = self.state.lock().map_err(|e| e.to_string())?;
        let issue = guard
            .issues
            .iter_mut()
            .find(|i| i.number == issue_number)
            .ok_or_else(|| format!("Offline tracker: issue #{} not found", issue_number))?;
        issue.labels.push(TrackerLabel::String(label.to_string()));
        Ok(())
    }

    fn create_issue(&self, title: &str, body: &str, label: &str, repo: Option<&str>) -> Result<(u64, String), String> {
        let mut guard = self.state.lock().map_err(|e| e.to_string())?;
        let id = guard.next_id;
        guard.next_id += 1;
        let url = format!(
            "https://github.com/{}/issues/{}",
            repo.unwrap_or("local/offline"),
            id
        );
        guard.issues.push(TrackerIssue {
            number: id,
            title: title.to_string(),
            state: "OPEN".to_string(),
            labels: vec![TrackerLabel::String(label.to_string())],
            body: Some(body.to_string()),
            url: Some(url.clone()),
        });
        Ok((id, url))
    }

    fn ensure_label_exists(&self, label: &str, _repo: Option<&str>) -> Result<(), String> {
        let mut guard = self.state.lock().map_err(|e| e.to_string())?;
        let norm = normalize_label(label);
        if !guard.labels.iter().any(|l| normalize_label(l) == norm) {
            guard.labels.push(label.to_string());
        }
        Ok(())
    }

    fn reopen_issue(&self, issue_number: u64, comment: Option<&str>, _repo: Option<&str>) -> Result<(), String> {
        let mut guard = self.state.lock().map_err(|e| e.to_string())?;
        let issue = guard
            .issues
            .iter_mut()
            .find(|i| i.number == issue_number)
            .ok_or_else(|| format!("Offline tracker: issue #{} not found", issue_number))?;

        issue.state = "OPEN".to_string();
        issue.labels.retain(|l| !is_resolved_label(l.name()));

        if let Some(c) = comment {
            let trimmed = c.trim();
            if !trimmed.is_empty() {
                if let Some(ref mut body) = issue.body {
                    if !body.is_empty() {
                        body.push_str("\n\n");
                        body.push_str(trimmed);
                    } else {
                        *body = trimmed.to_string();
                    }
                } else {
                    issue.body = Some(trimmed.to_string());
                }
            }
        }
        Ok(())
    }
}

/// GitHub CLI (`gh`) based tracker client.
pub struct GitHubTracker {
    pub repo: Option<String>,
}

impl GitHubTracker {
    pub fn new() -> Self {
        Self { repo: None }
    }

    pub fn with_repo(repo: impl Into<String>) -> Self {
        Self { repo: Some(repo.into()) }
    }
}

impl Default for GitHubTracker {
    /// Constructs a tracker bound to the current repository; equivalent to [`GitHubTracker::new`].
    fn default() -> Self {
        Self::new()
    }
}

impl TrackerClient for GitHubTracker {
    fn fetch_all_issues(&self) -> Result<Vec<TrackerIssue>, String> {
        let mut cmd = Command::new("gh");
        cmd.args(["issue", "list", "--limit", "1000", "--state", "all", "--json", "number,title,state,labels,body,url"]);
        if let Some(ref r) = self.repo {
            cmd.args(["--repo", r]);
        }
        let output = cmd.output()
            .map_err(|e| format!("Failed to execute 'gh issue list': {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("'gh issue list' exited with error: {}", stderr));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let issues: Vec<TrackerIssue> = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse 'gh issue list' JSON output: {}", e))?;

        Ok(issues)
    }

    fn update_issue_body(&self, issue_number: u64, body: &str) -> Result<(), String> {
        use std::io::Write;
        let mut temp_file = tempfile_named()?;
        temp_file.write_all(body.as_bytes())
            .map_err(|e| format!("Failed to write temporary issue body file: {}", e))?;
        temp_file.flush()
            .map_err(|e| format!("Failed to flush temporary issue body file: {}", e))?;
        let temp_path = temp_file.path.clone();

        let mut cmd = Command::new("gh");
        cmd.args(["issue", "edit", &issue_number.to_string(), "--body-file", &temp_path]);
        if let Some(ref r) = self.repo {
            cmd.args(["--repo", r]);
        }
        let output = cmd.output()
            .map_err(|e| format!("Failed to execute 'gh issue edit': {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("'gh issue edit' failed for issue #{}: {}", issue_number, stderr));
        }

        // Closed-loop readback verification
        let mut view_cmd = Command::new("gh");
        view_cmd.args(["issue", "view", &issue_number.to_string(), "--json", "number,body"]);
        if let Some(ref r) = self.repo {
            view_cmd.args(["--repo", r]);
        }
        let view_output = view_cmd.output()
            .map_err(|e| format!("Failed to execute 'gh issue view' for closed-loop verification: {}", e))?;

        if !view_output.status.success() {
            let stderr = String::from_utf8_lossy(&view_output.stderr);
            return Err(format!(
                "'gh issue view' closed-loop verification failed for issue #{}: {}",
                issue_number, stderr
            ));
        }

        #[derive(Deserialize)]
        struct UpdateViewPayload {
            number: u64,
            #[serde(default)]
            body: Option<String>,
        }

        let view_str = String::from_utf8_lossy(&view_output.stdout);
        let parsed: UpdateViewPayload = serde_json::from_str(&view_str)
            .map_err(|e| format!("Failed to parse 'gh issue view' closed-loop verification output: {}", e))?;

        if parsed.number != issue_number {
            return Err(format!(
                "Closed-loop verification mismatch: expected issue #{}, got #{}",
                issue_number, parsed.number
            ));
        }

        if !body.trim().is_empty() && parsed.body.as_deref().unwrap_or("").trim().is_empty() {
            return Err(format!(
                "Closed-loop verification error: remote body for issue #{} is empty",
                issue_number
            ));
        }

        Ok(())
    }

    fn add_label(&self, issue_number: u64, label: &str) -> Result<(), String> {
        let mut cmd = Command::new("gh");
        cmd.args(["issue", "edit", &issue_number.to_string(), "--add-label", label]);
        if let Some(ref r) = self.repo {
            cmd.args(["--repo", r]);
        }
        let output = cmd.output()
            .map_err(|e| format!("Failed to execute 'gh issue edit --add-label': {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("'gh issue edit --add-label' failed for issue #{}: {}", issue_number, stderr));
        }

        Ok(())
    }

    fn create_issue(&self, title: &str, body: &str, label: &str, repo: Option<&str>) -> Result<(u64, String), String> {
        use std::io::Write;
        let target_repo = repo.or(self.repo.as_deref());
        let mut temp_file = tempfile_named()?;
        temp_file.write_all(body.as_bytes())
            .map_err(|e| format!("Failed to write temporary issue body file: {}", e))?;
        temp_file.flush()
            .map_err(|e| format!("Failed to flush temporary issue body file: {}", e))?;
        let temp_path = temp_file.path.clone();

        let mut cmd = Command::new("gh");
        cmd.args(["issue", "create", "--title", title, "--label", label, "--body-file", &temp_path]);
        if let Some(r) = target_repo {
            cmd.args(["--repo", r]);
        }

        let output = cmd.output()
            .map_err(|e| format!("Failed to execute 'gh issue create': {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("'gh issue create' failed: {}", stderr));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let url_line = output_str
            .lines()
            .map(|l| l.trim())
            .rfind(|l| l.starts_with("http"))
            .unwrap_or_else(|| output_str.trim());

        let url = url_line.to_string();
        let number = url
            .rsplit('/')
            .next()
            .and_then(|seg| seg.trim().parse::<u64>().ok())
            .ok_or_else(|| format!("Could not parse issue number from created URL: '{}'", url))?;

        // Closed-loop readback verification
        let mut view_cmd = Command::new("gh");
        view_cmd.args([
            "issue",
            "view",
            &number.to_string(),
            "--json",
            "number,title,state,body,url",
        ]);
        if let Some(r) = target_repo {
            view_cmd.args(["--repo", r]);
        }
        let view_output = view_cmd.output()
            .map_err(|e| format!("Failed to execute 'gh issue view' for closed-loop verification: {}", e))?;

        if !view_output.status.success() {
            let stderr = String::from_utf8_lossy(&view_output.stderr);
            return Err(format!(
                "'gh issue view' closed-loop verification failed for created issue #{}: {}",
                number, stderr
            ));
        }

        let view_str = String::from_utf8_lossy(&view_output.stdout);
        let parsed: TrackerIssue = serde_json::from_str(&view_str)
            .map_err(|e| format!("Failed to parse 'gh issue view' verification output for issue #{}: {}", number, e))?;

        if parsed.number != number {
            return Err(format!(
                "Closed-loop verification mismatch: expected issue #{}, got #{}",
                number, parsed.number
            ));
        }

        Ok((number, url))
    }

    fn ensure_label_exists(&self, label: &str, repo: Option<&str>) -> Result<(), String> {
        let target_repo = repo.or(self.repo.as_deref());
        let mut list_cmd = Command::new("gh");
        list_cmd.args(["label", "list", "--limit", "1000", "--json", "name"]);
        if let Some(r) = target_repo {
            list_cmd.args(["--repo", r]);
        }
        let output = list_cmd.output()
            .map_err(|e| format!("Failed to execute 'gh label list': {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("'gh label list' exited with error: {}", stderr));
        }

        #[derive(Deserialize)]
        struct GhLabelItem {
            name: String,
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let labels: Vec<GhLabelItem> = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse 'gh label list' output: {}", e))?;

        let norm_target = normalize_label(label);
        let exists = labels.iter().any(|l| normalize_label(&l.name) == norm_target || l.name.eq_ignore_ascii_case(label));

        if !exists {
            let mut create_cmd = Command::new("gh");
            create_cmd.args([
                "label", "create", label,
                "--color", "0366d6",
                "--description", &format!("{} specification", label),
            ]);
            if let Some(r) = target_repo {
                create_cmd.args(["--repo", r]);
            }
            let create_out = create_cmd.output()
                .map_err(|e| format!("Failed to execute 'gh label create': {}", e))?;
            if !create_out.status.success() {
                let stderr = String::from_utf8_lossy(&create_out.stderr);
                if !stderr.to_lowercase().contains("already exists") {
                    return Err(format!("'gh label create' failed for label '{}': {}", label, stderr));
                }
            }
        }

        Ok(())
    }

    fn reopen_issue(&self, issue_number: u64, comment: Option<&str>, repo: Option<&str>) -> Result<(), String> {
        let target_repo = repo.or(self.repo.as_deref());
        let mut reopen_cmd = Command::new("gh");
        reopen_cmd.args(["issue", "reopen", &issue_number.to_string()]);
        if let Some(r) = target_repo {
            reopen_cmd.args(["--repo", r]);
        }
        let output = reopen_cmd.output()
            .map_err(|e| format!("Failed to execute 'gh issue reopen': {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.to_lowercase().contains("already open") {
                return Err(format!("'gh issue reopen' failed for issue #{}: {}", issue_number, stderr));
            }
        }

        // Remove stale resolved labels
        for label in ["status:fixed-resolved", "status::fixed-resolved", "fixed-resolved"] {
            let mut remove_cmd = Command::new("gh");
            remove_cmd.args(["issue", "edit", &issue_number.to_string(), "--remove-label", label]);
            if let Some(r) = target_repo {
                remove_cmd.args(["--repo", r]);
            }
            let _ = remove_cmd.output();
        }

        if let Some(comment_text) = comment {
            let trimmed = comment_text.trim();
            if !trimmed.is_empty() {
                use std::io::Write;
                let mut temp_file = tempfile_named()?;
                temp_file.write_all(trimmed.as_bytes())
                    .map_err(|e| format!("Failed to write temporary comment file: {}", e))?;
                temp_file.flush()
                    .map_err(|e| format!("Failed to flush temporary comment file: {}", e))?;
                let temp_path = temp_file.path.clone();

                let mut comment_cmd = Command::new("gh");
                comment_cmd.args(["issue", "comment", &issue_number.to_string(), "--body-file", &temp_path]);
                if let Some(r) = target_repo {
                    comment_cmd.args(["--repo", r]);
                }
                let c_output = comment_cmd.output()
                    .map_err(|e| format!("Failed to execute 'gh issue comment': {}", e))?;
                if !c_output.status.success() {
                    let stderr = String::from_utf8_lossy(&c_output.stderr);
                    return Err(format!("'gh issue comment' failed for issue #{}: {}", issue_number, stderr));
                }
            }
        }

        Ok(())
    }
}

/// Helper struct for temporary file without extra crate dependencies.
struct TempFileNamed {
    path: String,
    file: Option<std::fs::File>,
}

impl std::io::Write for TempFileNamed {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.as_mut().expect("file handle open").write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.as_mut().expect("file handle open").flush()
    }
}

impl Drop for TempFileNamed {
    fn drop(&mut self) {
        let _ = self.file.take();
        let _ = std::fs::remove_file(&self.path);
    }
}

static TEMP_FILE_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn tempfile_named() -> Result<TempFileNamed, String> {
    let nanos = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let pid = std::process::id();
    let seq = TEMP_FILE_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("reconcile_body_{}_{}_{}.md", pid, nanos, seq));
    let path_str = path.display().to_string();
    let file = std::fs::File::create(&path)
        .map_err(|e| format!("Unable to create temp file {}: {}", path_str, e))?;
    Ok(TempFileNamed {
        path: path_str,
        file: Some(file),
    })
}

/// Indexes issues by number and normalized title.
#[derive(Debug, Default)]
pub struct IssueIndex {
    pub by_number: BTreeMap<u64, TrackerIssue>,
    pub by_type_and_title: BTreeMap<(String, String), TrackerIssue>,
    pub by_normalized_title: BTreeMap<String, TrackerIssue>,
}

impl IssueIndex {
    pub fn build(issues: Vec<TrackerIssue>) -> Self {
        let mut by_number = BTreeMap::new();
        let mut by_type_and_title = BTreeMap::new();
        let mut by_normalized_title = BTreeMap::new();

        for issue in issues {
            let norm = normalize_title(&issue.title);
            if !norm.is_empty() {
                for label in &issue.labels {
                    let canonical = canonicalize_spec_type(label.name());
                    by_type_and_title.insert((canonical, norm.clone()), issue.clone());
                }
                if let Some(inferred) = infer_spec_type_from_title(&issue.title) {
                    by_type_and_title.insert((inferred, norm.clone()), issue.clone());
                }
                by_normalized_title.insert(norm, issue.clone());
            }
            by_number.insert(issue.number, issue);
        }

        Self {
            by_number,
            by_type_and_title,
            by_normalized_title,
        }
    }

    pub fn find_issue_for_type(
        &self,
        issue_id: Option<u64>,
        spec_type: Option<&str>,
        title: &str,
    ) -> Option<&TrackerIssue> {
        if let Some(id) = issue_id {
            if let Some(issue) = self.by_number.get(&id) {
                return Some(issue);
            }
        }
        let norm = normalize_title(title);
        if let Some(st) = spec_type {
            let canonical = canonicalize_spec_type(st);
            if let Some(issue) = self.by_type_and_title.get(&(canonical, norm.clone())) {
                return Some(issue);
            }
        }
        if let Some(inferred) = infer_spec_type_from_title(title) {
            if let Some(issue) = self.by_type_and_title.get(&(inferred, norm.clone())) {
                return Some(issue);
            }
        }
        if let Some(issue) = self.by_normalized_title.get(&norm) {
            let target_type = spec_type
                .map(canonicalize_spec_type)
                .or_else(|| infer_spec_type_from_title(title));
            if let Some(tt) = target_type {
                let matches_type = issue
                    .labels
                    .iter()
                    .any(|l| canonicalize_spec_type(l.name()) == tt)
                    || infer_spec_type_from_title(&issue.title).as_deref() == Some(&tt);
                let has_any_type = !issue.labels.is_empty()
                    || infer_spec_type_from_title(&issue.title).is_some();
                if matches_type || !has_any_type {
                    return Some(issue);
                }
            } else {
                return Some(issue);
            }
        }
        None
    }

    pub fn find_issue(&self, issue_id: Option<u64>, title: &str) -> Option<&TrackerIssue> {
        let inferred = infer_spec_type_from_title(title);
        self.find_issue_for_type(issue_id, inferred.as_deref(), title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_title_handles_underscores_and_snake_case() {
        assert_eq!(
            normalize_title("Feature 01: [ConOps] Human_Engineer_Interface"),
            "conops human engineer interface"
        );
        assert_eq!(
            normalize_title("Feature_01_Human_Engineer"),
            "human engineer"
        );
        assert_eq!(
            normalize_title("Epic_02: Radar_Tracker_Architecture"),
            "radar tracker architecture"
        );
        assert_eq!(
            normalize_title("User_Story_05: Ingest_Telemetry_Stream"),
            "ingest telemetry stream"
        );
        assert_eq!(
            normalize_title("Use_Case_03: Process_Radar_Tracks"),
            "process radar tracks"
        );
    }

    #[test]
    fn test_issue_index_cross_tier_isolation_and_find_issue_for_type() {
        let epic_issue = TrackerIssue {
            number: 10,
            title: "Epic 01: [ConOps] Human Engineer Interface".to_string(),
            state: "OPEN".to_string(),
            labels: vec![TrackerLabel::String("epic".to_string())],
            body: Some("Epic body".to_string()),
            url: Some("https://github.com/org/repo/issues/10".to_string()),
        };
        let feature_issue = TrackerIssue {
            number: 25,
            title: "Feature 01: [ConOps] Human Engineer Interface".to_string(),
            state: "OPEN".to_string(),
            labels: vec![TrackerLabel::String("feature".to_string())],
            body: Some("Feature body".to_string()),
            url: Some("https://github.com/org/repo/issues/25".to_string()),
        };
        let story_issue = TrackerIssue {
            number: 30,
            title: "User Story 01: [ConOps] Human Engineer Interface".to_string(),
            state: "OPEN".to_string(),
            labels: vec![TrackerLabel::String("user-story".to_string())],
            body: Some("Story body".to_string()),
            url: Some("https://github.com/org/repo/issues/30".to_string()),
        };
        let case_issue = TrackerIssue {
            number: 40,
            title: "Use Case 01: [ConOps] Human Engineer Interface".to_string(),
            state: "OPEN".to_string(),
            labels: vec![TrackerLabel::String("use-case".to_string())],
            body: Some("Case body".to_string()),
            url: Some("https://github.com/org/repo/issues/40".to_string()),
        };

        let issues = vec![
            epic_issue.clone(),
            feature_issue.clone(),
            story_issue.clone(),
            case_issue.clone(),
        ];
        let index = IssueIndex::build(issues);

        // Explicit type lookup must return isolated issues without collision
        assert_eq!(
            index
                .find_issue_for_type(None, Some("epic"), "Epic 01: [ConOps] Human Engineer Interface")
                .map(|i| i.number),
            Some(10)
        );
        assert_eq!(
            index
                .find_issue_for_type(None, Some("feature"), "Feature 01: [ConOps] Human Engineer Interface")
                .map(|i| i.number),
            Some(25)
        );
        assert_eq!(
            index
                .find_issue_for_type(None, Some("user-story"), "User Story 01: [ConOps] Human Engineer Interface")
                .map(|i| i.number),
            Some(30)
        );
        assert_eq!(
            index
                .find_issue_for_type(None, Some("use-case"), "Use Case 01: [ConOps] Human Engineer Interface")
                .map(|i| i.number),
            Some(40)
        );

        // Inferred type lookup via find_issue
        assert_eq!(
            index
                .find_issue(None, "Epic 01: [ConOps] Human Engineer Interface")
                .map(|i| i.number),
            Some(10)
        );
        assert_eq!(
            index
                .find_issue(None, "Feature 01: [ConOps] Human Engineer Interface")
                .map(|i| i.number),
            Some(25)
        );
        assert_eq!(
            index
                .find_issue(None, "User Story 01: [ConOps] Human Engineer Interface")
                .map(|i| i.number),
            Some(30)
        );
        assert_eq!(
            index
                .find_issue(None, "Use Case 01: [ConOps] Human Engineer Interface")
                .map(|i| i.number),
            Some(40)
        );
    }

    #[test]
    fn test_normalize_title_matches_spec() {
        assert_eq!(
            normalize_title("Feature 01: [ConOps] Human Engineer Interface"),
            normalize_title("[ConOps] Human Engineer Interface")
        );
        assert_eq!(
            normalize_title("Epic 01: System Architecture and Compiler Primacy"),
            "system architecture and compiler primacy"
        );
        assert_eq!(
            normalize_title("User Story 01: Ingest OEM Documentation and Schema Artifacts"),
            "ingest oem documentation and schema artifacts"
        );
    }

    #[test]
    fn test_issue_is_resolved() {
        let resolved_issue = TrackerIssue {
            number: 10,
            title: "Test".to_string(),
            state: "OPEN".to_string(),
            labels: vec![TrackerLabel::String("status:fixed-resolved".to_string())],
            body: None,
            url: None,
        };
        assert!(resolved_issue.is_resolved());

        let closed_issue = TrackerIssue {
            number: 11,
            title: "Test".to_string(),
            state: "CLOSED".to_string(),
            labels: vec![],
            body: None,
            url: None,
        };
        assert!(closed_issue.is_resolved());

        let open_issue = TrackerIssue {
            number: 12,
            title: "Test".to_string(),
            state: "OPEN".to_string(),
            labels: vec![TrackerLabel::String("bug".to_string())],
            body: None,
            url: None,
        };
        assert!(!open_issue.is_resolved());
    }

    #[test]
    fn test_reopen_issue_strips_resolved_labels_and_persists_comment() {
        let tracker = OfflineTracker::with_issues(vec![
            TrackerIssue {
                number: 101,
                title: "Defect 101".to_string(),
                state: "CLOSED".to_string(),
                labels: vec![
                    TrackerLabel::String("bug".to_string()),
                    TrackerLabel::String("status:fixed-resolved".to_string()),
                ],
                body: Some("Original body".to_string()),
                url: Some("https://github.com/org/repo/issues/101".to_string()),
            },
            TrackerIssue {
                number: 102,
                title: "Defect 102".to_string(),
                state: "CLOSED".to_string(),
                labels: vec![
                    TrackerLabel::String("status::fixed-resolved".to_string()),
                ],
                body: None,
                url: None,
            },
            TrackerIssue {
                number: 103,
                title: "Defect 103".to_string(),
                state: "CLOSED".to_string(),
                labels: vec![
                    TrackerLabel::String("fixed-resolved".to_string()),
                ],
                body: Some("Another body".to_string()),
                url: None,
            },
        ]);

        tracker.reopen_issue(101, Some("Audit reopen reason"), None).unwrap();
        tracker.reopen_issue(102, None, None).unwrap();
        tracker.reopen_issue(103, Some("Reopen comment 103"), None).unwrap();

        let issues = tracker.fetch_all_issues().unwrap();
        let i101 = issues.iter().find(|i| i.number == 101).unwrap();
        assert_eq!(i101.state, "OPEN");
        assert!(!i101.is_resolved(), "Reopened issue 101 must not be resolved");
        assert_eq!(i101.labels.len(), 1);
        assert_eq!(i101.labels[0].name(), "bug");
        let body101 = i101.body.as_deref().unwrap();
        assert!(body101.contains("Original body"));
        assert!(body101.contains("Audit reopen reason"));

        let i102 = issues.iter().find(|i| i.number == 102).unwrap();
        assert_eq!(i102.state, "OPEN");
        assert!(!i102.is_resolved(), "Reopened issue 102 must not be resolved");
        assert!(i102.labels.is_empty());

        let i103 = issues.iter().find(|i| i.number == 103).unwrap();
        assert_eq!(i103.state, "OPEN");
        assert!(!i103.is_resolved(), "Reopened issue 103 must not be resolved");
        assert!(i103.labels.is_empty());
        let body103 = i103.body.as_deref().unwrap();
        assert!(body103.contains("Another body"));
        assert!(body103.contains("Reopen comment 103"));

        let err = tracker.reopen_issue(999, None, None);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("issue #999 not found"));
    }

    #[test]
    fn test_offline_tracker_error_parity_on_nonexistent_issue() {
        let tracker = OfflineTracker::new();
        let res_update = tracker.update_issue_body(999, "new body");
        assert!(res_update.is_err(), "update_issue_body on nonexistent issue must fail");
        assert!(
            res_update.unwrap_err().contains("issue #999 not found"),
            "Error message must specify issue #999 not found"
        );

        let res_label = tracker.add_label(999, "bug");
        assert!(res_label.is_err(), "add_label on nonexistent issue must fail");
        assert!(
            res_label.unwrap_err().contains("issue #999 not found"),
            "Error message must specify issue #999 not found"
        );
    }

    #[test]
    fn test_temp_file_named_raii_drop_cleanup() {
        use std::io::Write;
        let path = {
            let mut temp = tempfile_named().expect("create temp file");
            let path = temp.path.clone();
            temp.write_all(b"temporary test data").expect("write bytes");
            temp.flush().expect("flush bytes");
            assert!(std::path::Path::new(&path).exists(), "Temp file must exist while in scope");
            path
        };
        assert!(
            !std::path::Path::new(&path).exists(),
            "Temp file must be removed when TempFileNamed drops out of scope"
        );
    }

    #[test]
    fn test_tracker_issue_deserialization_with_body() {
        let json_with_body = r#"{
            "number": 42,
            "title": "Issue with body",
            "state": "OPEN",
            "labels": ["bug"],
            "body": "some text",
            "url": "https://github.com/org/repo/issues/42"
        }"#;
        let issue: TrackerIssue = serde_json::from_str(json_with_body).unwrap();
        assert_eq!(issue.body, Some("some text".to_string()));

        let json_without_body = r#"{
            "number": 43,
            "title": "Issue without body",
            "state": "OPEN",
            "labels": []
        }"#;
        let issue_no_body: TrackerIssue = serde_json::from_str(json_without_body).unwrap();
        assert_eq!(issue_no_body.body, None);

        let json_null_body = r#"{
            "number": 44,
            "title": "Issue with null body",
            "state": "OPEN",
            "labels": [],
            "body": null
        }"#;
        let issue_null_body: TrackerIssue = serde_json::from_str(json_null_body).unwrap();
        assert_eq!(issue_null_body.body, None);
    }
}


