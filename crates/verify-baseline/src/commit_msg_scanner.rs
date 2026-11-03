//! Lexical token scanner for commit message non-closure invariant enforcement.
//! Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-SCAN-COMMIT-MESSAGE]

use crate::commit_msg::CommitViolation;

const FORBIDDEN_VERBS: &[&str] = &[
    "resolving",
    "resolves",
    "resolved",
    "resolve",
    "closing",
    "closes",
    "closed",
    "close",
    "fixing",
    "fixes",
    "fixed",
    "fix",
];

/// Checks whether `chars[p..]` begins with a standalone `"issue"` or `"issues"` keyword.
/// Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-PARSE-ISSUE-REFERENCE]
fn skip_issue_keyword(chars: &[char], p: usize) -> Option<usize> {
    let len = chars.len();
    let remaining = len.saturating_sub(p);
    if remaining >= 6 {
        let s: String = chars
            .get(p..p + 6)
            .map(|sl| sl.iter().map(|c| c.to_ascii_lowercase()).collect())
            .unwrap_or_default();
        if s == "issues"
            && (p + 6 == len
                || chars
                    .get(p + 6)
                    .is_some_and(|c| !c.is_alphanumeric() && *c != '_'))
        {
            return Some(p + 6);
        }
    }
    if remaining >= 5 {
        let s: String = chars
            .get(p..p + 5)
            .map(|sl| sl.iter().map(|c| c.to_ascii_lowercase()).collect())
            .unwrap_or_default();
        if s == "issue"
            && (p + 5 == len
                || chars
                    .get(p + 5)
                    .is_some_and(|c| !c.is_alphanumeric() && *c != '_'))
        {
            return Some(p + 5);
        }
    }
    None
}

/// Parses a `#<digits>`, `gh-<digits>`, or `<owner>/<repo>#<digits>` token starting at `chars[pos..]`.
/// Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-PARSE-ISSUE-REFERENCE]
fn parse_issue_digits_token(chars: &[char], mut pos: usize) -> Option<(String, usize)> {
    let len = chars.len();
    if chars.get(pos) == Some(&'#') {
        pos += 1;
        while pos < len && chars.get(pos).is_some_and(|c| c.is_whitespace()) {
            pos += 1;
        }
        let digit_start = pos;
        while pos < len && chars.get(pos).is_some_and(|c| c.is_ascii_digit()) {
            pos += 1;
        }
        if pos > digit_start {
            return chars
                .get(digit_start..pos)
                .map(|sl| (sl.iter().collect::<String>(), pos));
        }
        return None;
    }
    if pos + 3 <= len
        && chars.get(pos).is_some_and(|c| c.eq_ignore_ascii_case(&'g'))
        && chars
            .get(pos + 1)
            .is_some_and(|c| c.eq_ignore_ascii_case(&'h'))
        && chars.get(pos + 2) == Some(&'-')
    {
        pos += 3;
        while pos < len && chars.get(pos).is_some_and(|c| c.is_whitespace()) {
            pos += 1;
        }
        let digit_start = pos;
        while pos < len && chars.get(pos).is_some_and(|c| c.is_ascii_digit()) {
            pos += 1;
        }
        if pos > digit_start {
            return chars
                .get(digit_start..pos)
                .map(|sl| (sl.iter().collect::<String>(), pos));
        }
        return None;
    }

    let repo_start = pos;
    while pos < len
        && chars
            .get(pos)
            .is_some_and(|&c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/')
    {
        pos += 1;
    }
    if pos > repo_start && pos < len && chars.get(pos) == Some(&'#') {
        pos += 1;
        while pos < len && chars.get(pos).is_some_and(|c| c.is_whitespace()) {
            pos += 1;
        }
        let digit_start = pos;
        while pos < len && chars.get(pos).is_some_and(|c| c.is_ascii_digit()) {
            pos += 1;
        }
        if pos > digit_start {
            return chars
                .get(digit_start..pos)
                .map(|sl| (sl.iter().collect::<String>(), pos));
        }
    }
    None
}

/// Parses an issue reference starting at `chars[pos..]`, returning `(issue_id, next_pos)`.
/// Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-PARSE-ISSUE-REFERENCE]
pub(crate) fn parse_issue_reference(chars: &[char], mut pos: usize) -> Option<(String, usize)> {
    let len = chars.len();
    while pos < len && chars.get(pos).is_some_and(|c| c.is_whitespace()) {
        pos += 1;
    }
    if pos >= len {
        return None;
    }
    if let Some(after_issue) = skip_issue_keyword(chars, pos) {
        pos = after_issue;
        if pos < len && chars.get(pos) == Some(&':') {
            pos += 1;
        }
        while pos < len && chars.get(pos).is_some_and(|c| c.is_whitespace()) {
            pos += 1;
        }
        if pos >= len {
            return None;
        }
    }

    let mut open_delim = None;
    if chars.get(pos).is_some_and(|&c| c == '(' || c == '[') {
        open_delim = chars.get(pos).copied();
        pos += 1;
        while pos < len && chars.get(pos).is_some_and(|c| c.is_whitespace()) {
            pos += 1;
        }
    }
    if pos >= len {
        return None;
    }

    if let Some(after_issue) = skip_issue_keyword(chars, pos) {
        pos = after_issue;
        if pos < len && chars.get(pos) == Some(&':') {
            pos += 1;
        }
        while pos < len && chars.get(pos).is_some_and(|c| c.is_whitespace()) {
            pos += 1;
        }
        if pos >= len {
            return None;
        }
    }

    let (id, next_pos) = parse_issue_digits_token(chars, pos)?;
    let mut final_pos = next_pos;
    while final_pos < len && chars.get(final_pos).is_some_and(|c| c.is_whitespace()) {
        final_pos += 1;
    }
    if let Some(open) = open_delim {
        let expected_close = if open == '(' { ')' } else { ']' };
        if final_pos < len && chars.get(final_pos) == Some(&expected_close) {
            final_pos += 1;
        }
    }
    Some((id, final_pos))
}

/// Matches a forbidden auto-closing verb at `chars[i..]`, returning `(verb, verb_len)`.
/// Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-SCAN-COMMIT-MESSAGE]
fn match_forbidden_verb_at(chars: &[char], i: usize) -> Option<(&'static str, usize)> {
    let len = chars.len();
    for &verb in FORBIDDEN_VERBS {
        let v_chars: Vec<char> = verb.chars().collect();
        let v_len = v_chars.len();
        if i + v_len <= len {
            let matches = chars.get(i..i + v_len).is_some_and(|sl| {
                sl.iter()
                    .zip(v_chars.iter())
                    .all(|(c1, c2)| c1.to_ascii_lowercase() == *c2)
            });
            if matches {
                if i + v_len < len
                    && chars
                        .get(i + v_len)
                        .is_some_and(|&next| next.is_alphanumeric() || next == '_')
                {
                    continue;
                }
                return Some((verb, v_len));
            }
        }
    }
    None
}

/// Scans chained issue references (`", #297"`, `" and #299"`) following an initial forbidden verb match.
/// Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-SCAN-COMMIT-MESSAGE]
fn collect_chained_issue_violations(
    chars: &[char],
    verb: &str,
    mut chain_pos: usize,
    violations: &mut Vec<CommitViolation>,
) -> usize {
    let len = chars.len();
    while chain_pos < len {
        let mut p = chain_pos;
        while p < len && chars.get(p).is_some_and(|c| c.is_whitespace()) {
            p += 1;
        }
        if p >= len {
            break;
        }

        let mut has_separator = false;
        if chars.get(p) == Some(&',') {
            has_separator = true;
            p += 1;
            while p < len && chars.get(p).is_some_and(|c| c.is_whitespace()) {
                p += 1;
            }
        }

        if p + 3 <= len
            && chars.get(p).is_some_and(|c| c.eq_ignore_ascii_case(&'a'))
            && chars
                .get(p + 1)
                .is_some_and(|c| c.eq_ignore_ascii_case(&'n'))
            && chars
                .get(p + 2)
                .is_some_and(|c| c.eq_ignore_ascii_case(&'d'))
            && (p + 3 == len
                || chars
                    .get(p + 3)
                    .is_some_and(|c| !c.is_alphanumeric() && *c != '_'))
        {
            has_separator = true;
            p += 3;
            while p < len && chars.get(p).is_some_and(|c| c.is_whitespace()) {
                p += 1;
            }
        }

        if has_separator {
            if let Some((chain_id, after_chain)) = parse_issue_reference(chars, p) {
                let chain_snippet: String = chars
                    .get(chain_pos..after_chain)
                    .map(|sl| sl.iter().collect())
                    .unwrap_or_default();
                let chain_line = chars
                    .get(..chain_pos)
                    .map(|sl| sl.iter().filter(|&&c| c == '\n').count())
                    .unwrap_or(0)
                    + 1;
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
    chain_pos
}

/// Deterministic lexical token scanner in pure Rust (ZERO regex).
/// Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-SCAN-COMMIT-MESSAGE]
pub fn scan_commit_message(text: &str) -> Vec<CommitViolation> {
    let mut violations = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if i > 0
            && chars
                .get(i - 1)
                .is_some_and(|&prev| prev.is_alphanumeric() || prev == '_')
        {
            i += 1;
            continue;
        }

        if let Some((verb, v_len)) = match_forbidden_verb_at(&chars, i) {
            let mut j = i + v_len;
            let mut has_colon = false;
            if j < len && chars.get(j) == Some(&':') {
                has_colon = true;
                j += 1;
            }
            let mut whitespace_count = 0;
            while j < len && chars.get(j).is_some_and(|c| c.is_whitespace()) {
                whitespace_count += 1;
                j += 1;
            }
            let has_delimiter = j < len && chars.get(j).is_some_and(|&c| c == '(' || c == '[');
            if has_colon || whitespace_count > 0 || has_delimiter {
                if let Some((issue_id, next_pos)) = parse_issue_reference(&chars, j) {
                    let snippet: String = chars
                        .get(i..next_pos)
                        .map(|sl| sl.iter().collect())
                        .unwrap_or_default();
                    let line_number = chars
                        .get(..i)
                        .map(|sl| sl.iter().filter(|&&c| c == '\n').count())
                        .unwrap_or(0)
                        + 1;
                    violations.push(CommitViolation {
                        verb: verb.to_string(),
                        issue_id,
                        offending_snippet: snippet,
                        line_number,
                    });
                    i = collect_chained_issue_violations(&chars, verb, next_pos, &mut violations);
                    continue;
                }
            }
        }
        i += 1;
    }
    violations
}
