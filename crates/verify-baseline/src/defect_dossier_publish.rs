//! Closed-loop GitHub SSOT synchronization and CLI execution helpers for defect dossiers.
//! Realises: [REQ-VERIFY-BASELINE-DEFECT-DOSSIER], [DO-330-TQL-4-DEFECT-GATE]

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::defect_dossier::{
    derive_defect_title, ensure_persisted_in_pipeline_defects, is_ephemeral_tmp_path,
    resolve_target_repo, validate_defect_dossier_scoped, validate_defect_title, DefectDossierArgs,
    DefectSchemaValidation,
};

/// Normalizes CRLF (`\r\n`) line endings to LF (`\n`) and strips trailing newline characters for
/// closed-loop payload comparison.
/// Realises: [REQ-DEFECT-DOSSIER-CRLF-NORMALIZATION]
fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n")
        .trim_end_matches(['\r', '\n'])
        .to_string()
}

/// Verifies the JSON output of `gh issue view --json title,body,url` against the local title and body.
/// Realises: [REQ-DEFECT-DOSSIER-CLOSED-LOOP-GATE]
pub fn verify_closed_loop_payload(
    gh_json_output: &str,
    expected_title: &str,
    expected_body: &str,
) -> Result<String, String> {
    let parsed: serde_json::Value = serde_json::from_str(gh_json_output).map_err(|e| {
        format!(
            "Closed-Loop Payload Verification Gate failed to parse 'gh issue view' JSON: {}",
            e
        )
    })?;

    let remote_title = parsed
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            "Closed-Loop Payload Verification Gate failed: missing 'title' field in 'gh issue view' JSON."
                .to_string()
        })?;

    let remote_body = parsed
        .get("body")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            "Closed-Loop Payload Verification Gate failed: missing 'body' field in 'gh issue view' JSON."
                .to_string()
        })?;

    let remote_url = parsed
        .get("url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if remote_title != expected_title {
        return Err(format!(
            "Closed-Loop Payload Verification Gate FAILED: remote title '{}' does not match expected title '{}'.",
            remote_title, expected_title
        ));
    }

    let norm_remote = normalize_line_endings(remote_body);
    let norm_local = normalize_line_endings(expected_body);
    if norm_remote != norm_local {
        return Err(format!(
            "Closed-Loop Payload Verification Gate FAILED: remote issue body ({} bytes) does not match local .pipeline/defects/ file ({} bytes).",
            norm_remote.len(),
            norm_local.len()
        ));
    }

    Ok(remote_url)
}

/// Constructs a `gh` CLI subprocess `Command` configured with `/etc/ssl/cert.pem` TLS certificate
/// fallback for macOS `sandbox-exec` environments where `Security.framework` IPC is blocked.
/// Realises: [REQ-DEFECT-DOSSIER-BUILD-GH-COMMAND]
pub fn build_gh_command() -> Command {
    let mut cmd = Command::new("gh");
    if std::env::var_os("SSL_CERT_FILE").is_none() {
        let cert_bundle = Path::new("/etc/ssl/cert.pem");
        if cert_bundle.is_file() {
            cmd.env("SSL_CERT_FILE", cert_bundle);
        }
    }
    cmd
}

/// Publishes a new GitHub issue or edits an existing GitHub issue from the persisted `.pipeline/defects/`
/// file and immediately executes the Closed-Loop Payload Verification Gate.
/// Realises: [REQ-DEFECT-DOSSIER-GITHUB-SYNC]
pub fn publish_or_edit_github_issue(
    repo_and_title: (&str, &str),
    label: &str,
    persisted_path: &Path,
    local_body: &str,
    issue_id: Option<u64>,
) -> Result<String, String> {
    let (repo, title) = repo_and_title;
    let body_path_str = persisted_path.to_string_lossy().to_string();

    let issue_identifier = if let Some(id) = issue_id {
        let id_str = id.to_string();
        let output = build_gh_command()
            .args(["issue", "edit", &id_str, "--repo", repo, "--title", title, "--body-file", &body_path_str, "--add-label", label])
            .output()
            .map_err(|e| format!("Failed to execute 'gh issue edit': {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("'gh issue edit {}' failed: {}", id_str, stderr.trim()));
        }
        id_str
    } else {
        let output = build_gh_command()
            .args(["issue", "create", "--repo", repo, "--title", title, "--label", label, "--body-file", &body_path_str])
            .output()
            .map_err(|e| format!("Failed to execute 'gh issue create': {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("'gh issue create' failed: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let created_ref = stdout
            .lines()
            .rev()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .ok_or_else(|| "'gh issue create' succeeded but emitted empty stdout.".to_string())?;
        created_ref.to_string()
    };

    let view_output = build_gh_command()
        .args(["issue", "view", &issue_identifier, "--repo", repo, "--json", "title,body,url"])
        .output()
        .map_err(|e| format!("Failed to execute 'gh issue view': {}", e))?;

    if !view_output.status.success() {
        let stderr = String::from_utf8_lossy(&view_output.stderr);
        return Err(format!("'gh issue view {}' failed: {}", issue_identifier, stderr.trim()));
    }

    let view_stdout = String::from_utf8_lossy(&view_output.stdout);
    let verified_url = verify_closed_loop_payload(&view_stdout, title, local_body)?;
    if verified_url.is_empty() { Ok(issue_identifier) } else { Ok(verified_url) }
}

/// Handles persistence, telemetry printing, and remote publication for a validated defect dossier.
/// Realises: [REQ-RUN-CHECK-DEFECT-ENTRYPOINT]
fn handle_validated_dossier(
    args: &DefectDossierArgs,
    body_file: &Path,
    work_dir: &Path,
    content: &str,
    validation: &DefectSchemaValidation,
) -> Result<(), i32> {
    let resolved_repo = match resolve_target_repo(args.repo.as_deref(), work_dir) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error resolving target repository: {}", e);
            return Err(1);
        }
    };

    let label = args.label.clone().unwrap_or_else(|| {
        if args.provider == "gitlab" {
            if validation.suggested_label == "bug" { "type::bug".to_string() } else { "type::feature".to_string() }
        } else {
            validation.suggested_label.clone()
        }
    });

    let title = match &args.title {
        Some(explicit_title) => {
            if let Err(title_err) = validate_defect_title(explicit_title, &validation.file_location) {
                eprintln!("Defect dossier validation FAILED for {} (Check 11 title error):", body_file.display());
                eprintln!("  - {}", title_err);
                return Err(1);
            }
            explicit_title.trim().to_string()
        }
        None => derive_defect_title(&validation.file_location, &validation.symptom),
    };

    let persisted_path = if !args.dry_run {
        ensure_persisted_in_pipeline_defects(body_file, content).map_err(|e| {
            eprintln!("Error persisting dossier to .pipeline/defects/: {}", e);
            1
        })?
    } else {
        body_file.to_path_buf()
    };

    println!("Defect dossier validation PASSED for {}.", persisted_path.display());
    println!("[VALIDATION] Target payload:");
    println!("  Provider: {}", args.provider);
    println!("  Repo: {}", resolved_repo);
    println!("  Title: {}", title);
    println!("  Severity: {}", validation.severity);
    println!("  File Location: {}", validation.file_location);
    println!("  Test Target: {}", validation.test_target);
    println!("  Label: {}", label);
    println!("  Body file: {}", persisted_path.display());

    if !args.dry_run {
        match publish_or_edit_github_issue((&resolved_repo, &title), &label, &persisted_path, content, args.issue) {
            Ok(url) => println!("[CLOSED-LOOP VERIFIED] Remote GitHub SSOT issue synchronized and verified byte-for-byte: {}", url),
            Err(err) => {
                eprintln!("Error synchronizing defect dossier to GitHub SSOT: {}", err);
                return Err(1);
            }
        }
    }

    Ok(())
}

/// CLI execution handler for `verify-baseline check-defect`.
/// Realises: [REQ-RUN-CHECK-DEFECT-ENTRYPOINT]
pub(crate) fn execute_check_defect(args: &DefectDossierArgs) -> Result<(), i32> {
    let body_path = args.body_file.as_ref().or(args.target.as_ref());

    let body_file = match body_path {
        Some(p) => p,
        None => {
            eprintln!("Error: --body-file (or positional BODY_FILE) is required and must exist.");
            return Err(1);
        }
    };

    if is_ephemeral_tmp_path(body_file) {
        eprintln!(
            "Error: Ephemeral /tmp/ path '{}' is forbidden. Write defect dossiers directly to .pipeline/defects/<name>.md.",
            body_file.display()
        );
        return Err(1);
    }

    if !body_file.is_file() {
        eprintln!(
            "Error: Defect body file does not exist: {}",
            body_file.display()
        );
        return Err(1);
    }

    let content = match fs::read_to_string(body_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading body file {}: {}", body_file.display(), e);
            return Err(1);
        }
    };

    let work_dir = body_file
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    let target_repo = args
        .repo
        .clone()
        .or_else(|| resolve_target_repo(None, work_dir).ok());

    match validate_defect_dossier_scoped(&content, target_repo.as_deref(), Some(work_dir)) {
        Ok(validation) => {
            handle_validated_dossier(args, body_file, work_dir, &content, &validation)
        }
        Err(errors) => {
            eprintln!(
                "Defect dossier validation FAILED for {} with {} violation(s):",
                body_file.display(),
                errors.len()
            );
            for err in errors {
                eprintln!("  - {}", err);
            }
            Err(1)
        }
    }
}
