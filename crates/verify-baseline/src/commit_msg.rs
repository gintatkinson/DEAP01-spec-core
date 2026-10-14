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
    "resolving", "resolves", "resolved", "resolve",
    "closing", "closes", "closed", "close",
    "fixing", "fixes", "fixed", "fix",
];

/// Helper to parse an issue reference starting at `chars[pos..]`.
/// Returns `Some((issue_id_string, next_pos))` if an issue reference is found.
/// Supports:
/// 1. Optional "issue" or "issues" keyword
/// 2. Optional opening delimiter '(' or '['
/// 3. Issue reference formats:
///    a) '#' followed by optional whitespace and digits (e.g. "#123", "# 123")
///    b) "gh-" or "GH-" followed by optional whitespace and digits (e.g. "gh-123")
///    c) owner/repo#digits (e.g. "owner/repo#123", "repo#123")
/// 4. Optional closing delimiter ')' or ']' (matching opening delimiter if present)
fn parse_issue_reference(chars: &[char], mut pos: usize) -> Option<(String, usize)> {
    let len = chars.len();
    while pos < len && chars[pos].is_whitespace() {
        pos += 1;
    }
    if pos >= len {
        return None;
    }

    // Check for optional "issue" or "issues" keyword
    let is_issue_keyword = |p: usize| -> Option<usize> {
        let remaining = len - p;
        if remaining >= 6 {
            let s: String = chars[p..p + 6].iter().map(|c| c.to_ascii_lowercase()).collect();
            if s == "issues" {
                if p + 6 == len || (!chars[p + 6].is_alphanumeric() && chars[p + 6] != '_') {
                    return Some(p + 6);
                }
            }
        }
        if remaining >= 5 {
            let s: String = chars[p..p + 5].iter().map(|c| c.to_ascii_lowercase()).collect();
            if s == "issue" {
                if p + 5 == len || (!chars[p + 5].is_alphanumeric() && chars[p + 5] != '_') {
                    return Some(p + 5);
                }
            }
        }
        None
    };

    if let Some(after_issue) = is_issue_keyword(pos) {
        pos = after_issue;
        if pos < len && chars[pos] == ':' {
            pos += 1;
        }
        while pos < len && chars[pos].is_whitespace() {
            pos += 1;
        }
        if pos >= len {
            return None;
        }
    }

    let mut open_delim = None;
    if chars[pos] == '(' || chars[pos] == '[' {
        open_delim = Some(chars[pos]);
        pos += 1;
        while pos < len && chars[pos].is_whitespace() {
            pos += 1;
        }
    }
    if pos >= len {
        return None;
    }

    // Second check for "issue" / "issues" inside delimiter, e.g. "(issue #123)"
    if let Some(after_issue) = is_issue_keyword(pos) {
        pos = after_issue;
        if pos < len && chars[pos] == ':' {
            pos += 1;
        }
        while pos < len && chars[pos].is_whitespace() {
            pos += 1;
        }
        if pos >= len {
            return None;
        }
    }

    let mut issue_id = None;
    let mut next_pos = pos;

    if chars[pos] == '#' {
        pos += 1;
        while pos < len && chars[pos].is_whitespace() {
            pos += 1;
        }
        let digit_start = pos;
        while pos < len && chars[pos].is_ascii_digit() {
            pos += 1;
        }
        if pos > digit_start {
            issue_id = Some(chars[digit_start..pos].iter().collect::<String>());
            next_pos = pos;
        }
    } else if pos + 3 <= len
        && (chars[pos].to_ascii_lowercase() == 'g'
            && chars[pos + 1].to_ascii_lowercase() == 'h'
            && chars[pos + 2] == '-')
    {
        pos += 3;
        while pos < len && chars[pos].is_whitespace() {
            pos += 1;
        }
        let digit_start = pos;
        while pos < len && chars[pos].is_ascii_digit() {
            pos += 1;
        }
        if pos > digit_start {
            issue_id = Some(chars[digit_start..pos].iter().collect::<String>());
            next_pos = pos;
        }
    } else {
        // Check for cross-repo syntax: e.g. "gintatkinson/DEAP01-spec-core#123" or "repo#123"
        let repo_start = pos;
        while pos < len
            && (chars[pos].is_alphanumeric()
                || chars[pos] == '-'
                || chars[pos] == '_'
                || chars[pos] == '.'
                || chars[pos] == '/')
        {
            pos += 1;
        }
        if pos > repo_start && pos < len && chars[pos] == '#' {
            pos += 1;
            while pos < len && chars[pos].is_whitespace() {
                pos += 1;
            }
            let digit_start = pos;
            while pos < len && chars[pos].is_ascii_digit() {
                pos += 1;
            }
            if pos > digit_start {
                issue_id = Some(chars[digit_start..pos].iter().collect::<String>());
                next_pos = pos;
            }
        }
    }

    if let Some(id) = issue_id {
        let mut final_pos = next_pos;
        while final_pos < len && chars[final_pos].is_whitespace() {
            final_pos += 1;
        }
        if let Some(open) = open_delim {
            let expected_close = if open == '(' { ')' } else { ']' };
            if final_pos < len && chars[final_pos] == expected_close {
                final_pos += 1;
            }
        }
        Some((id, final_pos))
    } else {
        None
    }
}

/// Deterministic lexical token scanner in pure Rust (ZERO regex).
///
/// Detects forbidden auto-closing verbs preceding #<digits>, gh-<digits>, or repo#<digits>.
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

            // Either a colon was present, whitespace was present, or an opening delimiter is following
            let has_delimiter = j < len && (chars[j] == '(' || chars[j] == '[');
            if has_colon || whitespace_count > 0 || has_delimiter {
                if let Some((issue_id, next_pos)) = parse_issue_reference(&chars, j) {
                    let snippet: String = chars[i..next_pos].iter().collect();
                    let line_number = chars[..i].iter().filter(|&&c| c == '\n').count() + 1;

                    violations.push(CommitViolation {
                        verb: verb.to_string(),
                        issue_id,
                        offending_snippet: snippet,
                        line_number,
                    });

                    // Check for chained issue references (e.g. ", #297", " and #299", ", and #302")
                    let mut chain_pos = next_pos;
                    while chain_pos < len {
                        let mut p = chain_pos;
                        while p < len && chars[p].is_whitespace() {
                            p += 1;
                        }
                        if p >= len {
                            break;
                        }

                        let mut has_separator = false;
                        if chars[p] == ',' {
                            has_separator = true;
                            p += 1;
                            while p < len && chars[p].is_whitespace() {
                                p += 1;
                            }
                        }

                        // Check for optional "and"
                        if p + 3 <= len
                            && chars[p].to_ascii_lowercase() == 'a'
                            && chars[p + 1].to_ascii_lowercase() == 'n'
                            && chars[p + 2].to_ascii_lowercase() == 'd'
                        {
                            if p + 3 == len || (!chars[p + 3].is_alphanumeric() && chars[p + 3] != '_') {
                                has_separator = true;
                                p += 3;
                                while p < len && chars[p].is_whitespace() {
                                    p += 1;
                                }
                            }
                        }

                        if has_separator {
                            if let Some((chain_id, after_chain)) = parse_issue_reference(&chars, p) {
                                let chain_snippet: String = chars[chain_pos..after_chain].iter().collect();
                                let chain_line = chars[..chain_pos].iter().filter(|&&c| c == '\n').count() + 1;
                                violations.push(CommitViolation {
                                    verb: verb.to_string(),
                                    issue_id: chain_id,
                                    offending_snippet: chain_snippet,
                                    line_number: chain_line,
                                });
                                chain_pos = after_chain;
                                continue;
                            }
                        }
                        break;
                    }

                    i = chain_pos;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_valid_neutral_commit_messages() {
        let valid_messages = [
            "feat(core): update model (refs #123)",
            "fix(parser): syntax (#456)",
            "docs: update handoff (#12)",
            "feat(compiler): multi-mode FMECA extraction (#50, #51)",
            "chore: routine housekeeping without issue reference",
            "fix: correct edge condition in statechart parser",
            "refactor: clean up signal matrix refs #99",
            "feat(gui): implement telemetry panel (ref #10)",
            "fix(validator): resolve tokenizer blind spot (refs #287)",
            "prefix #50",
            "unfixed #50",
            "disclose #50",
            "fixedly #50",
            "resolves-issue #50",
        ];

        for msg in &valid_messages {
            let violations = scan_commit_message(msg);
            assert!(
                violations.is_empty(),
                "False positive violation on valid message '{}': {:?}",
                msg,
                violations
            );
        }
    }

    #[test]
    fn test_forbidden_auto_closing_commit_messages() {
        let forbidden_cases = [
            ("fixes #123", "fixes", "123"),
            ("closes #45", "closes", "45"),
            ("resolved #789", "resolved", "789"),
            ("Fix #1", "fix", "1"),
            ("feat(conops): operational authority (fixes #313)", "fixes", "313"),
            ("docs: update handoff (resolved #12)", "resolved", "12"),
            ("feat: add feature closes #99", "closes", "99"),
            ("fix: #100", "fix", "100"),
            ("fixes: #101", "fixes", "101"),
            ("closes: #102", "closes", "102"),
            ("resolves: #103", "resolves", "103"),
            ("fixing #104", "fixing", "104"),
            ("closing #105", "closing", "105"),
            ("resolving #106", "resolving", "106"),
        ];

        for (text, expected_verb, expected_id) in &forbidden_cases {
            let violations = scan_commit_message(text);
            assert!(
                !violations.is_empty(),
                "Failed to detect forbidden auto-closing trigger in '{}'",
                text
            );
            assert_eq!(
                violations[0].verb.to_ascii_lowercase(),
                *expected_verb,
                "Verb mismatch on '{}'",
                text
            );
            assert_eq!(
                violations[0].issue_id,
                *expected_id,
                "Issue ID mismatch on '{}'",
                text
            );
        }
    }

    #[test]
    fn test_delimiters_and_parentheses_bypass() {
        let cases = [
            ("feat: something fixes (#123)", "fixes", "123"),
            ("feat: something closes [#123]", "closes", "123"),
            ("feat: something fix(#123)", "fix", "123"),
            ("feat: something (fixes #123)", "fixes", "123"),
            ("feat: something [closes #123]", "closes", "123"),
        ];

        for (text, expected_verb, expected_id) in &cases {
            let violations = scan_commit_message(text);
            assert!(
                !violations.is_empty(),
                "Failed to detect delimited trigger in '{}'",
                text
            );
            assert_eq!(violations[0].verb.to_ascii_lowercase(), *expected_verb);
            assert_eq!(violations[0].issue_id, *expected_id);
        }
    }

    #[test]
    fn test_whitespace_after_hash() {
        let text = "feat: something closes # 123";
        let violations = scan_commit_message(text);
        assert!(!violations.is_empty());
        assert_eq!(violations[0].verb.to_ascii_lowercase(), "closes");
        assert_eq!(violations[0].issue_id, "123");
    }

    #[test]
    fn test_github_shorthand_and_cross_repo() {
        let cases = [
            ("feat: something fixes gh-123", "fixes", "123"),
            ("feat: something closes GH-456", "closes", "456"),
            (
                "feat: something closes gintatkinson/DEAP01-spec-core#789",
                "closes",
                "789",
            ),
            ("feat: something resolves org.name/my-repo#999", "resolves", "999"),
        ];

        for (text, expected_verb, expected_id) in &cases {
            let violations = scan_commit_message(text);
            assert!(
                !violations.is_empty(),
                "Failed to detect shorthand/cross-repo in '{}'",
                text
            );
            assert_eq!(violations[0].verb.to_ascii_lowercase(), *expected_verb);
            assert_eq!(violations[0].issue_id, *expected_id);
        }
    }

    #[test]
    fn test_issue_keyword_token() {
        let cases = [
            ("feat: something fixes issue #123", "fixes", "123"),
            ("feat: something resolves issues #456", "resolves", "456"),
            ("feat: something closes (issue #789)", "closes", "789"),
            ("feat: something fixes issue: #999", "fixes", "999"),
        ];

        for (text, expected_verb, expected_id) in &cases {
            let violations = scan_commit_message(text);
            assert!(
                !violations.is_empty(),
                "Failed to detect issue-keyword in '{}'",
                text
            );
            assert_eq!(violations[0].verb.to_ascii_lowercase(), *expected_verb);
            assert_eq!(violations[0].issue_id, *expected_id);
        }
    }

    #[test]
    fn test_chained_issue_references() {
        let msg = "fix(conops): remediate compiler engine (fixes #296, #297, #299, #302)";
        let violations = scan_commit_message(msg);
        assert_eq!(violations.len(), 4, "Expected 4 violations in: {}", msg);
        assert_eq!(violations[0].issue_id, "296");
        assert_eq!(violations[1].issue_id, "297");
        assert_eq!(violations[2].issue_id, "299");
        assert_eq!(violations[3].issue_id, "302");

        let msg_and = "feat: resolves #10 and #11";
        let v_and = scan_commit_message(msg_and);
        assert_eq!(v_and.len(), 2, "Expected 2 violations in: {}", msg_and);
        assert_eq!(v_and[0].issue_id, "10");
        assert_eq!(v_and[1].issue_id, "11");

        let msg_oxford = "feat: closes #1, #2, and #3";
        let v_oxford = scan_commit_message(msg_oxford);
        assert_eq!(v_oxford.len(), 3, "Expected 3 violations in: {}", msg_oxford);
        assert_eq!(v_oxford[0].issue_id, "1");
        assert_eq!(v_oxford[1].issue_id, "2");
        assert_eq!(v_oxford[2].issue_id, "3");
    }

    #[test]
    fn test_check_text_and_check_file() {
        assert!(check_text("feat(core): update pipeline (refs #10)", "test").is_ok());
        assert!(check_text("feat(core): update pipeline (fixes #10)", "test").is_err());

        let temp_dir = std::env::temp_dir();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let pid = std::process::id();
        let valid_path = temp_dir.join(format!("test_commit_valid_{}_{}.txt", pid, now));
        let invalid_path = temp_dir.join(format!("test_commit_invalid_{}_{}.txt", pid, now));

        let mut f1 = std::fs::File::create(&valid_path).unwrap();
        writeln!(f1, "feat(core): valid neutral citation (refs #42)").unwrap();

        let mut f2 = std::fs::File::create(&invalid_path).unwrap();
        writeln!(f2, "fix(core): forbidden auto-closing keyword (fixes #42)").unwrap();

        assert!(check_file(&valid_path).is_ok());
        assert!(check_file(&invalid_path).is_err());

        let _ = std::fs::remove_file(valid_path);
        let _ = std::fs::remove_file(invalid_path);
    }
}
