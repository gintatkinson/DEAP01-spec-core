//! External issue tracker abstraction and synchronization.
//!
//! Realises: [Issue Tracker as Canonical SSOT, Closed-Loop Payload Verification]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    /// According to constitution.md:161, an issue is resolved when it carries `status:fixed-resolved`
    /// or `status::fixed-resolved` (or state is CLOSED).
    pub fn is_resolved(&self) -> bool {
        if self.state.eq_ignore_ascii_case("closed") {
            return true;
        }
        for label in &self.labels {
            let l_norm = normalize_label(label.name());
            if l_norm == "status:fixed-resolved" || l_norm == "status::fixed-resolved" || l_norm == "fixed-resolved" {
                return true;
            }
        }
        false
    }
}

/// Normalizes issue labels for robust matching.
pub fn normalize_label(label: &str) -> String {
    let lower = label.trim().to_lowercase();
    lower.replace('_', "-").replace(' ', "-")
}

/// Canonical title normalization matching Python `reconcile_backlog.py` and `metadata.rs`.
pub fn normalize_title(title: &str) -> String {
    let trimmed = title.trim().trim_matches(|c| c == '"' || c == '\'');
    if trimmed.is_empty() {
        return String::new();
    }

    let lower = trimmed.to_lowercase();
    const PREFIX_KEYWORDS: &[&str] = &[
        "user-story", "user story", "user_story",
        "use-case", "use case", "use_case",
        "feature", "feat",
        "epic",
        "us", "uc",
    ];

    let mut matched_len = 0;
    for kw in PREFIX_KEYWORDS {
        if lower.starts_with(kw) {
            let mut rem = &lower[kw.len()..];
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

    let hyphens_replaced = working.replace('-', " ");
    let filtered: String = hyphens_replaced
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

impl TrackerClient for OfflineTracker {
    fn fetch_all_issues(&self) -> Result<Vec<TrackerIssue>, String> {
        let guard = self.state.lock().map_err(|e| e.to_string())?;
        Ok(guard.issues.clone())
    }

    fn update_issue_body(&self, issue_number: u64, body: &str) -> Result<(), String> {
        let mut guard = self.state.lock().map_err(|e| e.to_string())?;
        if let Some(issue) = guard.issues.iter_mut().find(|i| i.number == issue_number) {
            issue.body = Some(body.to_string());
        }
        Ok(())
    }

    fn add_label(&self, issue_number: u64, label: &str) -> Result<(), String> {
        let mut guard = self.state.lock().map_err(|e| e.to_string())?;
        if let Some(issue) = guard.issues.iter_mut().find(|i| i.number == issue_number) {
            issue.labels.push(TrackerLabel::String(label.to_string()));
        }
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

    fn reopen_issue(&self, issue_number: u64, _comment: Option<&str>, _repo: Option<&str>) -> Result<(), String> {
        let mut guard = self.state.lock().map_err(|e| e.to_string())?;
        if let Some(issue) = guard.issues.iter_mut().find(|i| i.number == issue_number) {
            issue.state = "OPEN".to_string();
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

impl TrackerClient for GitHubTracker {
    fn fetch_all_issues(&self) -> Result<Vec<TrackerIssue>, String> {
        let mut cmd = Command::new("gh");
        cmd.args(["issue", "list", "--limit", "1000", "--state", "all", "--json", "number,title,state,labels,url"]);
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
        let temp_path = temp_file.path.clone();

        let mut cmd = Command::new("gh");
        cmd.args(["issue", "edit", &issue_number.to_string(), "--body-file", &temp_path]);
        if let Some(ref r) = self.repo {
            cmd.args(["--repo", r]);
        }
        let output = cmd.output()
            .map_err(|e| format!("Failed to execute 'gh issue edit': {}", e))?;

        let _ = std::fs::remove_file(&temp_path);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("'gh issue edit' failed for issue #{}: {}", issue_number, stderr));
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
        let temp_path = temp_file.path.clone();

        let mut cmd = Command::new("gh");
        cmd.args(["issue", "create", "--title", title, "--label", label, "--body-file", &temp_path]);
        if let Some(r) = target_repo {
            cmd.args(["--repo", r]);
        }

        let output = cmd.output()
            .map_err(|e| format!("Failed to execute 'gh issue create': {}", e))?;

        let _ = std::fs::remove_file(&temp_path);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("'gh issue create' failed: {}", stderr));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let url_line = output_str
            .lines()
            .map(|l| l.trim())
            .filter(|l| l.starts_with("http"))
            .last()
            .unwrap_or_else(|| output_str.trim());

        let url = url_line.to_string();
        let number = url
            .rsplit('/')
            .next()
            .and_then(|seg| seg.trim().parse::<u64>().ok())
            .ok_or_else(|| format!("Could not parse issue number from created URL: '{}'", url))?;

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

        if let Some(comment_text) = comment {
            use std::io::Write;
            let mut temp_file = tempfile_named()?;
            temp_file.write_all(comment_text.as_bytes())
                .map_err(|e| format!("Failed to write temporary comment file: {}", e))?;
            let temp_path = temp_file.path.clone();

            let mut comment_cmd = Command::new("gh");
            comment_cmd.args(["issue", "comment", &issue_number.to_string(), "--body-file", &temp_path]);
            if let Some(r) = target_repo {
                comment_cmd.args(["--repo", r]);
            }
            let c_output = comment_cmd.output()
                .map_err(|e| format!("Failed to execute 'gh issue comment': {}", e))?;
            let _ = std::fs::remove_file(&temp_path);
            if !c_output.status.success() {
                let stderr = String::from_utf8_lossy(&c_output.stderr);
                return Err(format!("'gh issue comment' failed for issue #{}: {}", issue_number, stderr));
            }
        }

        Ok(())
    }
}

/// Helper struct for temporary file without extra crate dependencies.
struct TempFileNamed {
    path: String,
    file: std::fs::File,
}

impl std::io::Write for TempFileNamed {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

fn tempfile_named() -> Result<TempFileNamed, String> {
    let rand_id = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let path = std::env::temp_dir().join(format!("reconcile_body_{}.md", rand_id));
    let path_str = path.display().to_string();
    let file = std::fs::File::create(&path)
        .map_err(|e| format!("Unable to create temp file {}: {}", path_str, e))?;
    Ok(TempFileNamed { path: path_str, file })
}

/// Indexes issues by number and normalized title.
#[derive(Debug, Default)]
pub struct IssueIndex {
    pub by_number: HashMap<u64, TrackerIssue>,
    pub by_normalized_title: HashMap<String, TrackerIssue>,
}

impl IssueIndex {
    pub fn build(issues: Vec<TrackerIssue>) -> Self {
        let mut by_number = HashMap::new();
        let mut by_normalized_title = HashMap::new();

        for issue in issues {
            let norm = normalize_title(&issue.title);
            if !norm.is_empty() {
                by_normalized_title.insert(norm, issue.clone());
            }
            by_number.insert(issue.number, issue);
        }

        Self {
            by_number,
            by_normalized_title,
        }
    }

    pub fn find_issue(&self, issue_id: Option<u64>, title: &str) -> Option<&TrackerIssue> {
        if let Some(id) = issue_id {
            if let Some(issue) = self.by_number.get(&id) {
                return Some(issue);
            }
        }
        let norm = normalize_title(title);
        self.by_normalized_title.get(&norm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
