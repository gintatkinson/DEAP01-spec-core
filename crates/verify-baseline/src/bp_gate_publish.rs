#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Checks 11 and 12 (Source Reference Existence and Closed-Loop GitHub SSOT Publishing) for `blueprint_gate`.
//! Realises: [REQ-BLUEPRINT-CHECK-11-SOURCE-REFERENCES], [REQ-BLUEPRINT-CHECK-12-PUBLISH-SSOT]

use crate::blueprint_gate::{BlueprintCliOptions, BlueprintMetadata};
use std::path::Path;

/// Extracts candidate intra-repository reference tokens from backtick code spans and markdown links on a Section 7 line.
/// Realises: [REQ-BLUEPRINT-CHECK-11-CANDIDATE-EXTRACTOR]
fn extract_reference_candidates_from_line(line: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0usize;

    while i < len {
        let ch = chars.get(i).copied().unwrap_or(' ');
        if ch == '`' {
            let mut j = i.saturating_add(1);
            let mut span = String::new();
            let mut closed = false;
            while j < len {
                let c = chars.get(j).copied().unwrap_or(' ');
                if c == '`' {
                    closed = true;
                    break;
                }
                span.push(c);
                j = j.saturating_add(1);
            }
            if closed {
                let trimmed = span.trim();
                if !trimmed.is_empty() {
                    candidates.push(trimmed.to_string());
                }
                i = j.saturating_add(1);
                continue;
            }
        } else if ch == ']' && chars.get(i.saturating_add(1)).copied() == Some('(') {
            let mut j = i.saturating_add(2);
            let mut target = String::new();
            let mut closed = false;
            while j < len {
                let c = chars.get(j).copied().unwrap_or(' ');
                if c == ')' {
                    closed = true;
                    break;
                }
                target.push(c);
                j = j.saturating_add(1);
            }
            if closed {
                let trimmed = target.trim();
                if !trimmed.is_empty() {
                    candidates.push(trimmed.to_string());
                }
                i = j.saturating_add(1);
                continue;
            }
        }
        i = i.saturating_add(1);
    }
    candidates
}

/// Normalizes a candidate reference string from Section 7 and classifies whether it is an intra-repository path.
/// Realises: [REQ-BLUEPRINT-CHECK-11-NORMALIZER]
fn normalize_repo_reference(raw_candidate: &str) -> Option<String> {
    let trimmed = raw_candidate.trim();
    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("mailto:")
        || trimmed.contains(char::is_whitespace)
    {
        return None;
    }

    let without_fragment = trimmed.split_once('#').map_or(trimmed, |(before, _)| before);
    let without_line = without_fragment.split_once(':').map_or(without_fragment, |(before, _)| before);
    let clean = without_line.strip_prefix("./").unwrap_or(without_line).trim();

    let repo_prefixes = ["crates/", ".pipeline/", ".agents/", "skills/", "docs/", "rules/"];
    let repo_root_files = ["Cargo.toml", "AGENTS.md", "README.md"];

    let is_repo_path =
        repo_prefixes.iter().any(|p| clean.starts_with(p)) || repo_root_files.contains(&clean);
    is_repo_path.then(|| clean.to_string())
}

/// Check 11 (Source Reference Existence Gate -- Check 28/30): Verifies cited intra-repository paths exist on disk.
/// Realises: [REQ-BLUEPRINT-CHECK-11-SOURCE-REFERENCES]
pub fn check_11_source_references(
    workspace_root: &Path,
    section_7_lines: &[&str],
) -> Result<Vec<String>, Vec<String>> {
    let mut errors = Vec::new();
    let mut verified_refs: Vec<String> = Vec::new();

    for (idx, raw_line) in section_7_lines.iter().enumerate() {
        let line_no = idx.saturating_add(1);
        for candidate in extract_reference_candidates_from_line(raw_line) {
            let Some(repo_rel) = normalize_repo_reference(&candidate) else {
                continue;
            };
            if repo_rel.contains("..") {
                errors.push(format!(
                    "Check 11 violation (Section 7 line {line_no}): Cited repository path '{repo_rel}' must not contain '..' parent traversal"
                ));
                continue;
            }
            let full_path = workspace_root.join(&repo_rel);
            if !full_path.exists() {
                errors.push(format!(
                    "Check 11 violation (Section 7 line {line_no}): Cited repository file '{repo_rel}' does not exist under workspace root '{}'",
                    workspace_root.display()
                ));
            }
            if !verified_refs.contains(&repo_rel) {
                verified_refs.push(repo_rel);
            }
        }
    }

    if verified_refs.is_empty() {
        errors.push(
            "Check 11 violation (Section 7): Section 7 ('## 7. Source References') must cite >= 1 intra-repository file path"
                .to_string(),
        );
    }

    if errors.is_empty() {
        Ok(verified_refs)
    } else {
        Err(errors)
    }
}

/// Validates the JSON stdout returned by `gh issue view` for closed-loop GitHub SSOT verification.
/// Realises: [REQ-BLUEPRINT-CHECK-12-CLOSED-LOOP-HELPER]
pub fn verify_closed_loop_github_payload(
    view_json_stdout: &str,
    expected_title: &str,
    expected_req_id: &str,
) -> Result<String, String> {
    let val: serde_json::Value = serde_json::from_str(view_json_stdout)
        .map_err(|e| format!("Check 12 failed: invalid JSON from 'gh issue view': {e}"))?;

    let number = val
        .get("number")
        .and_then(serde_json::Value::as_u64)
        .filter(|&n| n > 0)
        .ok_or_else(|| "Check 12 failed: closed-loop JSON payload is missing a positive 'number' field".to_string())?;
    let title = val
        .get("title")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Check 12 failed: closed-loop JSON payload has missing or empty 'title'".to_string())?;
    let body = val
        .get("body")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Check 12 failed: closed-loop JSON payload has missing or empty 'body'".to_string())?;
    let url = val
        .get("url")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Check 12 failed: closed-loop JSON payload has missing or empty 'url'".to_string())?;

    let exp_title = expected_title.trim();
    if title != exp_title {
        return Err(format!("Check 12 failed: closed-loop issue title mismatch: expected '{exp_title}', found '{title}'"));
    }
    let exp_req = expected_req_id.trim();
    if exp_req.is_empty() || !body.contains(exp_req) {
        return Err(format!("Check 12 failed: closed-loop issue body does not contain expected Requirement ID '{exp_req}'"));
    }

    Ok(format!("[CLOSED-LOOP VERIFIED] Issue #{number} ({url}) title='{title}' req_id='{exp_req}'"))
}

/// Executes `gh issue edit` or `gh issue create` and returns the issue ID or URL.
/// Realises: [REQ-BLUEPRINT-CHECK-12-PUBLISH-SSOT]
fn publish_or_edit_github_issue(
    options: &BlueprintCliOptions,
    repo: &str,
    title: &str,
) -> Result<String, Vec<String>> {
    if let Some(issue_id) = options.issue {
        let issue_str = issue_id.to_string();
        let edit_out = std::process::Command::new("gh")
            .args(["issue", "edit", &issue_str, "--repo", repo, "--title", title, "--body-file"])
            .arg(&options.path)
            .output()
            .map_err(|e| vec![format!("Check 12 failed: unable to invoke 'gh issue edit': {e}")])?;
        if !edit_out.status.success() {
            let stderr = String::from_utf8_lossy(&edit_out.stderr);
            return Err(vec![format!(
                "Check 12 failed: 'gh issue edit #{issue_id}' exited with {}: {}",
                edit_out.status,
                stderr.trim()
            )]);
        }
        Ok(issue_str)
    } else {
        let create_out = std::process::Command::new("gh")
            .args(["issue", "create", "--repo", repo, "--title", title, "--body-file"])
            .arg(&options.path)
            .output()
            .map_err(|e| vec![format!("Check 12 failed: unable to invoke 'gh issue create': {e}")])?;
        if !create_out.status.success() {
            let stderr = String::from_utf8_lossy(&create_out.stderr);
            return Err(vec![format!(
                "Check 12 failed: 'gh issue create' exited with {}: {}",
                create_out.status,
                stderr.trim()
            )]);
        }
        let stdout = String::from_utf8_lossy(&create_out.stdout);
        let created_ref = stdout.lines().rev().map(str::trim).find(|l| !l.is_empty()).unwrap_or("").to_string();
        if created_ref.is_empty() {
            return Err(vec!["Check 12 failed: 'gh issue create' returned empty stdout without issue URL or number".to_string()]);
        }
        Ok(created_ref)
    }
}

/// Check 12 (Closed-Loop GitHub SSOT Publishing Gate -- `--publish`).
/// Realises: [REQ-BLUEPRINT-CHECK-12-PUBLISH-SSOT]
pub fn check_12_publish_github_ssot(
    options: &BlueprintCliOptions,
    metadata: &BlueprintMetadata,
    body: &str,
) -> Result<String, Vec<String>> {
    if body.trim().is_empty() {
        return Err(vec!["Check 12 failed: blueprint body cannot be empty prior to publishing".to_string()]);
    }
    if !options.publish || options.dry_run {
        return Ok("[DRY-RUN: READY TO PUBLISH]".to_string());
    }

    let repo = options.repo.as_deref().unwrap_or("gintatkinson/DEAP01-spec-core");
    let title = options.title.as_deref().unwrap_or(&metadata.title);
    let id_or_url = publish_or_edit_github_issue(options, repo, title)?;

    let view_out = std::process::Command::new("gh")
        .args(["issue", "view", &id_or_url, "--repo", repo, "--json", "number,title,body,url"])
        .output()
        .map_err(|e| vec![format!("Check 12 failed: unable to invoke 'gh issue view': {e}")])?;
    if !view_out.status.success() {
        let stderr = String::from_utf8_lossy(&view_out.stderr);
        return Err(vec![format!(
            "Check 12 failed: 'gh issue view {id_or_url}' exited with {}: {}",
            view_out.status,
            stderr.trim()
        )]);
    }

    let view_stdout = String::from_utf8_lossy(&view_out.stdout);
    let summary = verify_closed_loop_github_payload(&view_stdout, title, &metadata.requirement_id)
        .map_err(|e| vec![e])?;
    if !metadata.blueprint_id.trim().is_empty() && !view_stdout.contains(metadata.blueprint_id.trim()) {
        return Err(vec![format!(
            "Check 12 failed: closed-loop issue body does not contain expected Blueprint ID '{}'",
            metadata.blueprint_id
        )]);
    }
    Ok(summary)
}
