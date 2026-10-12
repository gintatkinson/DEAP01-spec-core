//! Native commit message linter and Commit Message Non-Closure Invariant gate.
//!
//! Enforces .pipeline/constitution.md:266:
//! Strictly zero regex. Deterministic pure-Rust lexical scanner.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(clap::Args, Debug, Clone)]
pub struct CommitMsgArgs {
    /// Path to file containing commit message
    #[arg(long = "msg-file")]
    pub msg_file: Option<PathBuf>,

    /// Git revision range to inspect (e.g. origin/main..HEAD)
    #[arg(long = "range")]
    pub range: Option<String>,

    /// Check the latest HEAD commit
    #[arg(long = "head")]
    pub head: bool,

    /// Check raw text directly
    #[arg(long = "check-text")]
    pub check_text: Option<String>,

    /// Positional file or text to check
    #[arg(value_name = "TARGET")]
    pub target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitViolation {
    pub verb: String,
    pub issue_id: String,
    pub offending_snippet: String,
    pub line_number: usize,
}

const FORBIDDEN_VERBS: &[&str] = &[
    "fix", "fixes", "fixed",
    "close", "closes", "closed",
    "resolve", "resolves", "resolved",
];

/// Deterministic lexical token scanner in pure Rust (ZERO regex).
///
/// Detects forbidden auto-closing verbs immediately preceding #<digits>.
/// Validates neutral citations like `(#<id>)` and `(refs #<id>)`.
pub fn scan_commit_message(text: &str) -> Vec<CommitViolation> {
    let mut violations = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    let mut i = 0;
    while i < len {
        // Word boundary check: previous character must not be alphanumeric or '_'
        if i > 0 {
            let prev = chars[i - 1];
            if prev.is_alphanumeric() || prev == '_' {
                i += 1;
                continue;
            }
        }

        // Check if any forbidden verb matches starting at index i
        let mut matched_verb = None;
        for &verb in FORBIDDEN_VERBS {
            let v_chars: Vec<char> = verb.chars().collect();
            let v_len = v_chars.len();
            if i + v_len <= len {
                let matches = chars[i..i + v_len]
                    .iter()
                    .zip(v_chars.iter())
                    .all(|(c1, c2)| c1.to_ascii_lowercase() == *c2);
                if matches {
                    // Check word boundary after verb: next character must not be alphanumeric or '_'
                    if i + v_len < len {
                        let next = chars[i + v_len];
                        if next.is_alphanumeric() || next == '_' {
                            continue;
                        }
                    }
                    matched_verb = Some((verb, v_len));
                    break;
                }
            }
        }

        if let Some((verb, v_len)) = matched_verb {
            let mut j = i + v_len;
            let mut has_colon = false;
            if j < len && chars[j] == ':' {
                has_colon = true;
                j += 1;
            }

            let mut whitespace_count = 0;
            while j < len && chars[j].is_whitespace() {
                whitespace_count += 1;
                j += 1;
            }

            // Invariant trigger requirement:
            // Either a colon was present (with optional whitespace),
            // or at least one whitespace character was present.
            if (has_colon || whitespace_count > 0) && j < len && chars[j] == '#' {
                j += 1; // skip '#'
                let digit_start = j;
                while j < len && chars[j].is_ascii_digit() {
                    j += 1;
                }

                if j > digit_start {
                    // Valid issue number detected following forbidden verb!
                    let issue_id: String = chars[digit_start..j].iter().collect();
                    let snippet: String = chars[i..j].iter().collect();

                    // Calculate line number (1-based)
                    let line_number = chars[..i].iter().filter(|&&c| c == '\n').count() + 1;

                    violations.push(CommitViolation {
                        verb: verb.to_string(),
                        issue_id,
                        offending_snippet: snippet,
                        line_number,
                    });

                    i = j;
                    continue;
                }
            }
        }

        i += 1;
    }

    violations
}

pub fn format_violation_error(context: &str, text: &str, violations: &[CommitViolation]) {
    eprintln!("ERROR: Commit message violates the Commit Message Non-Closure Invariant (.pipeline/constitution.md:266).");
    if !context.is_empty() {
        eprintln!("Context: {}", context);
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
    eprintln!("(fix, fixes, fixed, close, closes, closed, resolve, resolves, resolved preceding #<id>).");
    eprintln!("All commit messages referencing issues MUST use neutral citations:");
    eprintln!("  - '(refs #<id>)'");
    eprintln!("  - '(#<id>)'");
    eprintln!();
    eprintln!("Offending message:");
    for line in text.lines() {
        eprintln!("  > {}", line);
    }
}

pub fn check_text(text: &str, context: &str) -> Result<(), ()> {
    let violations = scan_commit_message(text);
    if !violations.is_empty() {
        format_violation_error(context, text, &violations);
        Err(())
    } else {
        Ok(())
    }
}

pub fn check_file(path: &Path) -> Result<(), ()> {
    if !path.is_file() {
        eprintln!("Error: Commit message file does not exist: {}", path.display());
        return Err(());
    }
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading commit message file {}: {}", path.display(), e);
            return Err(());
        }
    };
    check_text(&content, &format!("file {}", path.display()))
}

pub fn check_head() -> Result<(), ()> {
    let output = match Command::new("git")
        .args(["log", "-1", "--pretty=%B"])
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            eprintln!("Error executing git log: {}", e);
            return Err(());
        }
    };

    if !output.status.success() {
        eprintln!("git log -1 command failed");
        return Err(());
    }

    let msg = String::from_utf8_lossy(&output.stdout);
    check_text(&msg, "HEAD commit")
}

pub fn check_range(rev_range: &str) -> Result<(), ()> {
    let output = match Command::new("git")
        .args(["log", "--format=---COMMIT_SEPARATOR---%n%H%n%B", rev_range])
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            eprintln!("Error executing git log for range {}: {}", rev_range, e);
            return Err(());
        }
    };

    if !output.status.success() {
        eprintln!("git log failed for revision range: {}", rev_range);
        return Err(());
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

        let context = format!("commit {} in range {}", commit_hash, rev_range);
        if check_text(&commit_msg, &context).is_err() {
            violations_count += 1;
        }
    }

    if violations_count > 0 {
        Err(())
    } else {
        Ok(())
    }
}

pub fn run_check_commit_msg(args: &CommitMsgArgs) -> Result<(), ()> {
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
        Err(())
    }
}
