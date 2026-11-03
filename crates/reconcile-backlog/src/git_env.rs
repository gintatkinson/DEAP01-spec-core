//! Git repository, branch, and upstream subdirectory prefix detection helpers.
//! Realises: [REQ-RECONCILE-BACKLOG-CORE, Issue Tracker as Canonical SSOT]

use std::path::Path;
use std::process::Command;

/// [DOC-TIER: A] Normalizes an `owner/repo` string by stripping trailing slashes and `.git` suffixes.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-CLEAN-REPO-NAME]
/// Contract: Returns `Some("owner/repo")` when valid, or `None` when empty/malformed.
/// Bounds & Faults: Time O(N), Space O(N) where N is `raw.len()`.
pub fn clean_repo_name(raw: &str) -> Option<String> {
    let mut s = raw.trim().trim_end_matches('/');
    if let Some(stripped) = s.strip_suffix(".git") {
        s = stripped.trim_end_matches('/');
    }
    let s = s.trim_start_matches('/');
    let parts: Vec<&str> = s.split('/').filter(|p| !p.is_empty()).collect();
    if let (Some(&owner), Some(&repo)) = (
        parts.len().checked_sub(2).and_then(|i| parts.get(i)),
        parts.last(),
    ) {
        if !owner.is_empty() && !repo.is_empty() {
            return Some(format!("{}/{}", owner, repo));
        }
    }
    None
}

/// [DOC-TIER: A] Extracts `owner/repo` from an HTTPS or SSH Git remote URL.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-PARSE-REPO-FROM-REMOTE-URL]
/// Contract: Supports `https://github.com/owner/repo(.git)` and `git@github.com:owner/repo(.git)`.
/// Bounds & Faults: Time O(N), Space O(N) where N is `url.len()`.
pub fn parse_repo_from_remote_url(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(colon_idx) = trimmed.find(':') {
        let before_colon = trimmed.get(..colon_idx).unwrap_or("");
        if !before_colon.contains("//") && before_colon.contains('@') {
            let after_colon = trimmed.get(colon_idx + 1..).unwrap_or("");
            return clean_repo_name(after_colon);
        }
    }

    if let Some(scheme_idx) = trimmed.find("://") {
        let after_scheme = trimmed.get(scheme_idx + 3..).unwrap_or("");
        if let Some(slash_idx) = after_scheme.find('/') {
            let path_part = after_scheme.get(slash_idx + 1..).unwrap_or("");
            return clean_repo_name(path_part);
        }
    }

    clean_repo_name(trimmed)
}

/// [DOC-TIER: A] Auto-detects the GitHub `owner/repo` for `workspace_root` via `git remote get-url origin`.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-DETECT-GIT-REPO]
/// Contract: Returns `Some("owner/repo")` when `origin` remote is configured, else `None`.
/// Bounds & Faults: Time O(1) subprocess call, Space O(R).
pub fn detect_git_repo(workspace_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace_root)
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let url = String::from_utf8_lossy(&output.stdout);
    parse_repo_from_remote_url(&url)
}

/// [DOC-TIER: A] Auto-detects the current or default Git branch for `workspace_root` (fallback `"main"`).
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-DETECT-GIT-BRANCH]
/// Contract: Queries `git rev-parse --abbrev-ref HEAD` and falls back to `"main"` on detached HEAD or error.
/// Bounds & Faults: Time O(1) subprocess call, Space O(B).
pub fn detect_git_branch(workspace_root: &Path) -> String {
    if let Ok(output) = Command::new("git")
        .arg("-C")
        .arg(workspace_root)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
    {
        if output.status.success() {
            let b = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !b.is_empty() && b != "HEAD" {
                return b;
            }
        }
    }
    "main".to_string()
}

/// [DOC-TIER: A] Detects the relative subdirectory prefix of `workspace_root` inside its enclosing Git repository.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-DETECT-UPSTREAM-PREFIX]
/// Contract: Uses `git rev-parse --show-prefix` so nested workspaces generate accurate GitHub blob URLs.
/// Bounds & Faults: Time O(1) subprocess call, Space O(P).
pub fn detect_upstream_prefix(workspace_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace_root)
        .args(["rev-parse", "--show-prefix"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let prefix = String::from_utf8_lossy(&output.stdout)
        .trim()
        .trim_matches('/')
        .to_string();

    if prefix.is_empty() {
        None
    } else {
        Some(prefix)
    }
}
