//! Native commit message linter and Commit Message Non-Closure Invariant gate.
//! Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-SCAN-COMMIT-MESSAGE]

pub use crate::commit_msg_scanner::scan_commit_message;
use std::path::{Path, PathBuf};
use std::process::Command;

/// CLI arguments for `verify-baseline check-commit-msg`.
/// Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-COMMITMSGARGS]
#[derive(Debug, Clone)]
pub struct CommitMsgArgs {
    pub msg_file: Option<PathBuf>,
    pub range: Option<String>,
    pub head: bool,
    pub check_text: Option<String>,
    pub target: Option<String>,
}

/// Detected commit-message non-closure violation record.
/// Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-COMMITVIOLATION]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitViolation {
    pub verb: String,
    pub issue_id: String,
    pub offending_snippet: String,
    pub line_number: usize,
}

/// Unit error returned when commit-message policy validation fails.
/// - **Safety & Traceability**: Enforces Commit Message Non-Closure Invariant. Realises: [REQ-COMMIT-MSG-ERROR]
/// - **Contract**: Pre: none. Post: signals policy violation. Invariant: zero-sized unit error.
/// - **Bounds & Faults**: O(1) time, O(1) space; infallible value type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommitMsgError;

/// Prints a formatted diagnostic error message for commit message non-closure violations.
/// Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-FORMAT-VIOLATION-ERROR]
pub fn format_violation_error(context: &str, text: &str, violations: &[CommitViolation]) {
    eprintln!("ERROR: Commit message violates the Commit Message Non-Closure Invariant (.pipeline/constitution.md:266).");
    if !context.is_empty() {
        eprintln!("Context: {context}");
    }
    eprintln!("Forbidden auto-closing trigger detected.");
    for v in violations {
        eprintln!(
            "  - Forbidden verb '{}' targeting #{} detected on line {}",
            v.verb, v.issue_id, v.line_number
        );
    }
    eprintln!();
    eprintln!("Agents and automated scripts are strictly prohibited from using issue auto-closing keywords");
    eprintln!(
        "(fix, fixes, fixed, close, closes, closed, resolve, resolves, resolved preceding #<id>)."
    );
    eprintln!("All commit messages referencing issues MUST use neutral citations:");
    eprintln!("  - '(refs #<id>)'");
    eprintln!("  - '(#<id>)'");
    eprintln!();
    eprintln!("Offending message:");
    for line in text.lines() {
        eprintln!("  > {line}");
    }
}

/// Checks raw commit message text for forbidden auto-closing keywords.
/// Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-CHECK-TEXT]
pub fn check_text(text: &str, context: &str) -> Result<(), CommitMsgError> {
    let violations = scan_commit_message(text);
    if !violations.is_empty() {
        format_violation_error(context, text, &violations);
        Err(CommitMsgError)
    } else {
        Ok(())
    }
}

/// Reads and checks a commit message file on disk.
/// Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-CHECK-FILE]
pub fn check_file(path: &Path) -> Result<(), CommitMsgError> {
    if !path.is_file() {
        eprintln!(
            "Error: Commit message file does not exist: {}",
            path.display()
        );
        return Err(CommitMsgError);
    }
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading commit message file {}: {e}", path.display());
            return Err(CommitMsgError);
        }
    };
    check_text(&content, &format!("file {}", path.display()))
}

/// Checks the latest `HEAD` commit message via `git log -1 --pretty=%B`.
/// Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-CHECK-HEAD]
pub fn check_head() -> Result<(), CommitMsgError> {
    let output = match Command::new("git").args(["log", "-1", "--pretty=%B"]).output() {
        Ok(out) => out,
        Err(e) => {
            eprintln!("Error executing git log: {e}");
            return Err(CommitMsgError);
        }
    };

    if !output.status.success() {
        eprintln!("git log -1 command failed");
        return Err(CommitMsgError);
    }

    let msg = String::from_utf8_lossy(&output.stdout);
    check_text(&msg, "HEAD commit")
}

/// Checks all commit messages in a revision range via `git log`.
/// Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-CHECK-RANGE]
pub fn check_range(rev_range: &str) -> Result<(), CommitMsgError> {
    let output = match Command::new("git")
        .args(["log", "--format=---COMMIT_SEPARATOR---%n%H%n%B", rev_range])
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            eprintln!("Error executing git log for range {rev_range}: {e}");
            return Err(CommitMsgError);
        }
    };

    if !output.status.success() {
        eprintln!("git log failed for revision range: {rev_range}");
        return Err(CommitMsgError);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut violations_count = 0;

    for block in stdout.split("---COMMIT_SEPARATOR---") {
        let trimmed = block.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut lines = trimmed.lines();
        let commit_hash = lines.next().unwrap_or("").trim();
        let commit_msg: String = lines.collect::<Vec<_>>().join("\n");
        if commit_msg.trim().is_empty() {
            continue;
        }
        let context = format!("commit {commit_hash} in range {rev_range}");
        if check_text(&commit_msg, &context).is_err() {
            violations_count += 1;
        }
    }

    if violations_count > 0 {
        Err(CommitMsgError)
    } else {
        Ok(())
    }
}

/// Dispatches `verify-baseline check-commit-msg` arguments to the appropriate check entrypoint.
/// Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-RUN-CHECK-COMMIT-MSG]
pub fn run_check_commit_msg(args: &CommitMsgArgs) -> Result<(), CommitMsgError> {
    if let Some(ref path) = args.msg_file {
        check_file(path)
    } else if let Some(ref range) = args.range {
        check_range(range)
    } else if args.head {
        check_head()
    } else if let Some(ref text) = args.check_text {
        check_text(text, "raw text")
    } else if let Some(ref target) = args.target {
        let p = Path::new(target);
        if p.is_file() {
            check_file(p)
        } else {
            check_text(target, "positional input")
        }
    } else {
        eprintln!("Usage: verify-baseline check-commit-msg [--msg-file <file> | --range <rev-range> | --head | --check-text <text>]");
        Err(CommitMsgError)
    }
}

#[cfg(test)]
#[path = "commit_msg_tests.rs"]
mod tests;
