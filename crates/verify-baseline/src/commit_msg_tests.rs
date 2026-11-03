#![cfg(test)]
//! Unit tests for `commit_msg` non-closure invariant scanning and CLI helpers.
//! Realises: [REQ-VERIFY-BASELINE-COMMIT-MSG-SCAN-COMMIT-MESSAGE]

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
        (
            "feat(conops): operational authority (fixes #313)",
            "fixes",
            "313",
        ),
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
            violations
                .first()
                .map(|v| v.verb.to_ascii_lowercase())
                .as_deref(),
            Some(*expected_verb),
            "Verb mismatch on '{}'",
            text
        );
        assert_eq!(
            violations.first().map(|v| v.issue_id.as_str()),
            Some(*expected_id),
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
        assert_eq!(
            violations
                .first()
                .map(|v| v.verb.to_ascii_lowercase())
                .as_deref(),
            Some(*expected_verb)
        );
        assert_eq!(
            violations.first().map(|v| v.issue_id.as_str()),
            Some(*expected_id)
        );
    }
}

#[test]
fn test_whitespace_after_hash() {
    let text = "feat: something closes # 123";
    let violations = scan_commit_message(text);
    assert!(!violations.is_empty());
    assert_eq!(
        violations
            .first()
            .map(|v| v.verb.to_ascii_lowercase())
            .as_deref(),
        Some("closes")
    );
    assert_eq!(violations.first().map(|v| v.issue_id.as_str()), Some("123"));
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
        (
            "feat: something resolves org.name/my-repo#999",
            "resolves",
            "999",
        ),
    ];

    for (text, expected_verb, expected_id) in &cases {
        let violations = scan_commit_message(text);
        assert!(
            !violations.is_empty(),
            "Failed to detect shorthand/cross-repo in '{}'",
            text
        );
        assert_eq!(
            violations
                .first()
                .map(|v| v.verb.to_ascii_lowercase())
                .as_deref(),
            Some(*expected_verb)
        );
        assert_eq!(
            violations.first().map(|v| v.issue_id.as_str()),
            Some(*expected_id)
        );
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
        assert_eq!(
            violations
                .first()
                .map(|v| v.verb.to_ascii_lowercase())
                .as_deref(),
            Some(*expected_verb)
        );
        assert_eq!(
            violations.first().map(|v| v.issue_id.as_str()),
            Some(*expected_id)
        );
    }
}

#[test]
fn test_chained_issue_references() {
    let msg = "fix(conops): remediate compiler engine (fixes #296, #297, #299, #302)";
    let violations = scan_commit_message(msg);
    assert_eq!(violations.len(), 4, "Expected 4 violations in: {}", msg);
    assert_eq!(violations.first().map(|v| v.issue_id.as_str()), Some("296"));
    assert_eq!(violations.get(1).map(|v| v.issue_id.as_str()), Some("297"));
    assert_eq!(violations.get(2).map(|v| v.issue_id.as_str()), Some("299"));
    assert_eq!(violations.get(3).map(|v| v.issue_id.as_str()), Some("302"));

    let msg_and = "feat: resolves #10 and #11";
    let v_and = scan_commit_message(msg_and);
    assert_eq!(v_and.len(), 2, "Expected 2 violations in: {}", msg_and);
    assert_eq!(v_and.first().map(|v| v.issue_id.as_str()), Some("10"));
    assert_eq!(v_and.get(1).map(|v| v.issue_id.as_str()), Some("11"));

    let msg_oxford = "feat: closes #1, #2, and #3";
    let v_oxford = scan_commit_message(msg_oxford);
    assert_eq!(
        v_oxford.len(),
        3,
        "Expected 3 violations in: {}",
        msg_oxford
    );
    assert_eq!(v_oxford.first().map(|v| v.issue_id.as_str()), Some("1"));
    assert_eq!(v_oxford.get(1).map(|v| v.issue_id.as_str()), Some("2"));
    assert_eq!(v_oxford.get(2).map(|v| v.issue_id.as_str()), Some("3"));
}

#[test]
fn test_check_text_and_check_file() -> Result<(), Box<dyn std::error::Error>> {
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

    let mut f1 = std::fs::File::create(&valid_path)?;
    writeln!(f1, "feat(core): valid neutral citation (refs #42)")?;

    let mut f2 = std::fs::File::create(&invalid_path)?;
    writeln!(f2, "fix(core): forbidden auto-closing keyword (fixes #42)")?;

    assert!(check_file(&valid_path).is_ok());
    assert!(check_file(&invalid_path).is_err());

    let _ = std::fs::remove_file(valid_path);
    let _ = std::fs::remove_file(invalid_path);
    Ok(())
}
