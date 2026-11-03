//! Baseline verification Check 39: Zero-Mocking Test AST & SQLite Disposal Gate.
//! Realises: [REQ-VERIFY-BASELINE-CODEBASE-CHECKS], [REQ-CHECK-39-ZERO-MOCKING-TEST-AST-GATE]

use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Check 39: Zero-Mocking Test AST & SQLite Disposal Gate.
/// Realises: [REQ-CHECK-39-ZERO-MOCKING-TEST-AST-GATE]
pub fn check_39_zero_mocking_test_ast_gate(repo_root: &Path) -> Result<(), Vec<String>> {
    let test_dirs = [
        repo_root.join("app_flutter").join("test"),
        repo_root.join("app_flutter").join("integration_test"),
    ];

    let mut violations = Vec::new();

    for tdir in &test_dirs {
        if tdir.is_dir() {
            for entry_res in WalkDir::new(tdir) {
                let entry = match entry_res {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if path.extension().map(|ext| ext == "dart").unwrap_or(false) {
                        let rel = path
                            .strip_prefix(repo_root)
                            .unwrap_or(path)
                            .display()
                            .to_string();
                        scan_test_file_for_mocking(path, &rel, &mut violations);
                    }
                }
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        Err(violations)
    }
}

/// Strips `//` line comments and `/* ... */` block comments from Dart source while preserving newlines.
/// Realises: [REQ-VERIFY-BASELINE-CODEBASE-CHECKS]
pub fn strip_dart_comments(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let mut idx: usize = 0;
    let mut in_single = false;
    let mut in_double = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while let Some(&ch) = chars.get(idx) {
        idx = idx.saturating_add(1);
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
                out.push('\n');
            }
        } else if in_block_comment {
            if ch == '*' && chars.get(idx) == Some(&'/') {
                idx = idx.saturating_add(1);
                in_block_comment = false;
            } else if ch == '\n' {
                out.push('\n');
            }
        } else if in_single {
            out.push(ch);
            if ch == '\\' {
                if let Some(&esc) = chars.get(idx) {
                    idx = idx.saturating_add(1);
                    out.push(esc);
                }
            } else if ch == '\'' {
                in_single = false;
            }
        } else if in_double {
            out.push(ch);
            if ch == '\\' {
                if let Some(&esc) = chars.get(idx) {
                    idx = idx.saturating_add(1);
                    out.push(esc);
                }
            } else if ch == '"' {
                in_double = false;
            }
        } else if ch == '/' && chars.get(idx) == Some(&'/') {
            idx = idx.saturating_add(1);
            in_line_comment = true;
        } else if ch == '/' && chars.get(idx) == Some(&'*') {
            idx = idx.saturating_add(1);
            in_block_comment = true;
        } else {
            if ch == '\'' {
                in_single = true;
            } else if ch == '"' {
                in_double = true;
            }
            out.push(ch);
        }
    }
    out
}

/// Extracts a Dart class identifier from a line regardless of Dart 3 class modifiers.
/// Realises: [REQ-CHECK-39-SCAN-TEST-FILE]
pub(crate) fn extract_dart_class_name(trimmed_line: &str) -> Option<&str> {
    let header = trimmed_line.split('{').next().unwrap_or(trimmed_line);
    let mut tokens = header.split_whitespace();
    while let Some(tok) = tokens.next() {
        if tok == "class" {
            if let Some(raw_name) = tokens.next() {
                let clean = raw_name.split('<').next().unwrap_or(raw_name).trim();
                if !clean.is_empty() {
                    return Some(clean);
                }
            }
        }
    }
    None
}

/// Extracts a receiver method call `<receiver>.<method>(` from a comment-stripped Dart line.
/// Realises: [REQ-CHECK-39-SCAN-TEST-FILE], [REQ-CHECK-40-SCAN-TEST-GESTURES]
pub(crate) fn extract_receiver_method_call(line: &str, receiver_prefix: &str) -> Option<String> {
    let idx = line.find(receiver_prefix)?;
    if idx > 0 {
        let prev = line
            .as_bytes()
            .get(idx.saturating_sub(1))
            .copied()
            .unwrap_or(b' ');
        if prev.is_ascii_alphanumeric() || prev == b'_' {
            return None;
        }
    }
    let after = line.get(idx.saturating_add(receiver_prefix.len())..)?;
    let method_name: String = after
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if method_name.is_empty() {
        return None;
    }
    let rest = after.get(method_name.len()..)?.trim_start();
    if !rest.starts_with('(') {
        return None;
    }
    let benign = [
        "dispose",
        "close",
        "addListener",
        "removeListener",
        "toString",
    ];
    if benign.contains(&method_name.as_str()) {
        return None;
    }
    Some(
        format!("{}{}((", receiver_prefix, method_name)
            .trim_end_matches('(')
            .to_string()
            + "(",
    )
}

/// Scans comment-stripped `testWidgets` blocks for direct receiver mutations without `tester.` gestures.
/// Realises: [REQ-CHECK-39-SCAN-TEST-FILE], [REQ-CHECK-40-SCAN-TEST-GESTURES]
pub(crate) fn find_ungestured_test_widgets_mutations(
    stripped: &str,
    receivers: &[&str],
) -> Vec<(usize, String)> {
    let mut results = Vec::new();
    let mut in_block = false;
    let mut brace_depth: isize = 0;
    let mut has_gesture = false;
    let mut block_mutations: Vec<(usize, String)> = Vec::new();
    let gestures = [
        "tester.tap(",
        "tester.drag(",
        "tester.enterText(",
        "tester.longPress(",
        "tester.sendKeyEvent(",
    ];

    for (idx, line) in stripped.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        if !in_block && line.contains("testWidgets(") {
            in_block = true;
            brace_depth = 0;
            has_gesture = false;
            block_mutations.clear();
        }
        if in_block {
            if gestures.iter().any(|g| line.contains(g)) {
                has_gesture = true;
            }
            for rec in receivers {
                if let Some(call) = extract_receiver_method_call(line, rec) {
                    block_mutations.push((line_no, call));
                }
            }
            for ch in line.chars() {
                if ch == '{' {
                    brace_depth = brace_depth.saturating_add(1);
                } else if ch == '}' {
                    brace_depth = brace_depth.saturating_sub(1);
                }
            }
            if brace_depth <= 0 && line.contains("});") {
                if !has_gesture {
                    results.append(&mut block_mutations);
                }
                in_block = false;
            }
        }
    }
    if in_block && !has_gesture {
        results.append(&mut block_mutations);
    }
    results
}

/// Inspects a single comment-stripped line for mock/fake/stub declarations, forbidden `implements`, and tautologies.
/// Realises: [REQ-CHECK-39-SCAN-TEST-FILE]
fn check_line_for_mocking_violations(
    trimmed: &str,
    file_rel: &str,
    line_no: usize,
    violations: &mut Vec<String>,
) {
    if let Some(cls) = extract_dart_class_name(trimmed) {
        let is_mock_cls = cls.starts_with("Mock")
            || cls.starts_with("_Mock")
            || cls.starts_with("Fake")
            || cls.starts_with("_Fake")
            || cls.starts_with("Stub")
            || cls.starts_with("_Stub")
            || cls.contains("Mock")
            || cls.contains("Fake")
            || cls.contains("Stub");
        if is_mock_cls {
            violations.push(format!(
                "Check 39 violation: Forbidden mock or fake class declaration in {}:{}: '{}'. Live persistence required.",
                file_rel, line_no, trimmed
            ));
        }
    }
    for ext in ["extends Mock", "extends Fake", "with Mock", "with Fake"] {
        if trimmed.contains(ext) {
            violations.push(format!(
                "Check 39 violation: Forbidden inheritance from Mock/Fake in {}:{}: '{}'.",
                file_rel, line_no, trimmed
            ));
        }
    }
    if let Some((_, after_imp)) = trimmed.split_once("implements ") {
        let suffixes = [
            "Repository",
            "DataSource",
            "Transport",
            "Service",
            "Store",
            "Client",
        ];
        for raw_tok in after_imp.split(|c: char| c == ',' || c == '{' || c.is_whitespace()) {
            let tok = raw_tok.trim();
            if suffixes.iter().any(|s| tok.ends_with(s)) {
                violations.push(format!(
                    "Check 39 violation: Forbidden mock repository/datasource implementation in {}:{}: '{}'.",
                    file_rel, line_no, trimmed
                ));
            }
        }
    }
    if trimmed.contains("TestThemeService") || trimmed.contains("FakeThemeService") {
        violations.push(format!(
            "Check 39 violation: Forbidden mock theme service in {}:{}: '{}'.",
            file_rel, line_no, trimmed
        ));
    }
    for stub in [
        "Result.success([])",
        "TopologyData(coordinateMapping: {}, nodes: [], links: [])",
        "TopologyData(nodes: [], links: [])",
        "expect(true, isTrue)",
        "expect(false, isFalse)",
    ] {
        if trimmed.contains(stub) {
            violations.push(format!(
                "Check 39 violation: Hardcoded empty result stub or tautological assertion in {}:{}: '{}'.",
                file_rel, line_no, trimmed
            ));
        }
    }
}

/// Inspects a single test file for forbidden mock classes, empty stubs, and leaking SQLite connections.
/// Realises: [REQ-CHECK-39-SCAN-TEST-FILE]
fn scan_test_file_for_mocking(file_path: &Path, file_rel: &str, violations: &mut Vec<String>) {
    let raw_content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let content = strip_dart_comments(&raw_content);

    for (idx, line) in content.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            check_line_for_mocking_violations(trimmed, file_rel, line_no, violations);
        }
    }

    for (line_no, call) in
        find_ungestured_test_widgets_mutations(&content, &["bridge.", "controller.", "notifier."])
    {
        violations.push(format!(
            "Check 39 violation: Direct state mutation '{}' without tester gesture in {}:{}.",
            call, file_rel, line_no
        ));
    }

    let has_sqlite_setup = content.contains("openDatabase(")
        || content.contains("DatabaseInitializer")
        || content.contains("DomainSeedStrategy")
        || content.contains("seedGeneratedDomainDatabase");

    if has_sqlite_setup && content.contains("setUpAll(") {
        let has_valid_teardown = content.contains("tearDownAll(")
            && (content.contains(".close()") || content.contains(".dispose()"));
        if !has_valid_teardown {
            violations.push(format!(
                "Check 39 violation: {}: setUpAll initializes SQLite database without matching tearDownAll (.close()/.dispose()) disposal.",
                file_rel
            ));
        }
    }
}
