//! Native Git hook configuration and pipeline infrastructure provisioning.
//!
//! Replaces setup_git_hooks.sh with pure cross-platform Rust logic.

use clap::Args;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Arguments for setup-hooks subcommand.
#[derive(Args, Debug, Clone)]
pub struct SetupHooksArgs {
    /// Target repository root or destination directory (defaults to current directory)
    #[arg(default_value = ".")]
    pub destination: String,
}

const COMMIT_MSG_HOOK_CONTENT: &str = r#"#!/bin/sh
# Git commit-msg hook installed by verify-baseline setup-hooks
set -e

MSG_FILE="$1"
if [ -x "./target/release/verify-baseline" ]; then
    exec ./target/release/verify-baseline check-commit-msg --msg-file "$MSG_FILE"
elif [ -x "./target/release/verify-baseline.exe" ]; then
    exec ./target/release/verify-baseline.exe check-commit-msg --msg-file "$MSG_FILE"
elif [ -x "./target/debug/verify-baseline" ]; then
    exec ./target/debug/verify-baseline check-commit-msg --msg-file "$MSG_FILE"
elif [ -x "./target/debug/verify-baseline.exe" ]; then
    exec ./target/debug/verify-baseline.exe check-commit-msg --msg-file "$MSG_FILE"
elif command -v cargo >/dev/null 2>&1; then
    exec cargo run --quiet --bin verify-baseline -- check-commit-msg --msg-file "$MSG_FILE"
else
    echo "Warning: verify-baseline binary or cargo not found. Skipping commit-msg validation." >&2
    exit 0
fi
"#;

const PRE_COMMIT_HOOK_CONTENT: &str = r#"#!/bin/sh
# Git pre-commit hook installed by verify-baseline setup-hooks
set -e

if [ -x "./target/release/verify-baseline" ]; then
    ./target/release/verify-baseline check-output --dir docs 2>/dev/null || true
    exec ./target/release/verify-baseline . --no-domain
elif [ -x "./target/release/verify-baseline.exe" ]; then
    ./target/release/verify-baseline.exe check-output --dir docs 2>/dev/null || true
    exec ./target/release/verify-baseline.exe . --no-domain
elif [ -x "./target/debug/verify-baseline" ]; then
    ./target/debug/verify-baseline check-output --dir docs 2>/dev/null || true
    exec ./target/debug/verify-baseline . --no-domain
elif [ -x "./target/debug/verify-baseline.exe" ]; then
    ./target/debug/verify-baseline.exe check-output --dir docs 2>/dev/null || true
    exec ./target/debug/verify-baseline.exe . --no-domain
elif command -v cargo >/dev/null 2>&1; then
    cargo run --quiet --bin verify-baseline -- check-output --dir docs 2>/dev/null || true
    exec cargo run --quiet --bin verify-baseline -- . --no-domain
fi
"#;

const PIPELINE_WHITELIST_PATTERNS: &[&str] = &[
    "!/skills/",
    "!/skills/**",
    "!/rules/",
    "!/rules/**",
    "!/.pipeline/",
    "!/.pipeline/**",
    "!/.agents/",
    "!/.agents/**",
];

/// Remove any .DS_Store files across the tree silently.
fn clean_ds_store(repo_root: &Path) {
    for entry in WalkDir::new(repo_root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.file_name() == ".DS_Store")
    {
        let _ = fs::remove_file(entry.path());
    }
}

/// Ensure pipeline patterns are present in .gitignore.
fn update_gitignore(repo_root: &Path) -> Result<(), std::io::Error> {
    let gitignore_path = repo_root.join(".gitignore");
    let current_content = if gitignore_path.exists() {
        fs::read_to_string(&gitignore_path).unwrap_or_default()
    } else {
        String::new()
    };

    let mut to_append = Vec::new();
    for pat in PIPELINE_WHITELIST_PATTERNS {
        if !current_content.lines().any(|l| l.trim() == *pat) {
            to_append.push(*pat);
        }
    }

    if !to_append.is_empty() {
        let mut new_content = current_content;
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        for pat in to_append {
            new_content.push_str(pat);
            new_content.push('\n');
        }
        fs::write(&gitignore_path, new_content)?;
    }

    Ok(())
}

/// Execute native git hooks setup.
pub fn run_setup_hooks(args: &SetupHooksArgs) -> Result<(), i32> {
    let repo_root = PathBuf::from(&args.destination);
    let git_dir = repo_root.join(".git");

    if !git_dir.is_dir() {
        eprintln!(
            "Error: .git directory not found at {}",
            git_dir.display()
        );
        return Err(1);
    }

    clean_ds_store(&repo_root);

    let hooks_dir = git_dir.join("hooks");
    if let Err(e) = fs::create_dir_all(&hooks_dir) {
        eprintln!("Error creating hooks directory: {}", e);
        return Err(1);
    }

    let pre_commit_path = hooks_dir.join("pre-commit");
    if let Err(e) = fs::write(&pre_commit_path, PRE_COMMIT_HOOK_CONTENT) {
        eprintln!("Error writing pre-commit hook: {}", e);
        return Err(1);
    }

    let commit_msg_path = hooks_dir.join("commit-msg");
    if let Err(e) = fs::write(&commit_msg_path, COMMIT_MSG_HOOK_CONTENT) {
        eprintln!("Error writing commit-msg hook: {}", e);
        return Err(1);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        let _ = fs::set_permissions(&pre_commit_path, perms.clone());
        let _ = fs::set_permissions(&commit_msg_path, perms);
    }

    let pre_push_path = hooks_dir.join("pre-push");
    if pre_push_path.exists() {
        let _ = fs::remove_file(pre_push_path);
    }

    if let Err(e) = update_gitignore(&repo_root) {
        eprintln!("Warning: Failed to update .gitignore: {}", e);
    }

    let _ = Command::new("git")
        .current_dir(&repo_root)
        .args(["add", ".pipeline/", "skills/", "rules/", ".agents/"])
        .output();

    println!("Successfully configured git hooks and staged pipeline infrastructure.");
    Ok(())
}
