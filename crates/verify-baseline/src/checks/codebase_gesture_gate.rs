//! Baseline verification Check 40: Interactive Gesture, 1080p-6K Viewport & Pairwise 2D HUD Non-Overlap Gate.
//! Realises: [REQ-VERIFY-BASELINE-CODEBASE-CHECKS], [REQ-CHECK-40-INTERACTIVE-GESTURE-HUD-NON-OVERLAP]

use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use super::codebase_mock_gate::{find_ungestured_test_widgets_mutations, strip_dart_comments};

/// Check 40: Interactive Gesture, 1080p-6K Viewport & Pairwise 2D HUD Non-Overlap Gate.
/// Realises: [REQ-CHECK-40-INTERACTIVE-GESTURE-HUD-NON-OVERLAP]
pub fn check_40_interactive_gesture_and_hud_non_overlap_gate(
    repo_root: &Path,
) -> Result<(), Vec<String>> {
    let test_dir = repo_root.join("app_flutter").join("test");
    if !test_dir.is_dir() {
        return Ok(());
    }

    let mut violations = Vec::new();

    let contract_path = test_dir
        .join("generated")
        .join("schema_ui_contract_test.g.dart");
    if contract_path.is_file() {
        let rel = contract_path
            .strip_prefix(repo_root)
            .unwrap_or(&contract_path)
            .display()
            .to_string();
        verify_ui_contract_test_viewports(&contract_path, &rel, &mut violations);
    } else {
        violations.push(
            "Check 40 violation: Missing mandatory generated UI contract test app_flutter/test/generated/schema_ui_contract_test.g.dart."
                .to_string(),
        );
    }

    let test_dirs = [
        test_dir,
        repo_root.join("app_flutter").join("integration_test"),
    ];

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
                        scan_test_file_for_interactive_gestures(path, &rel, &mut violations);
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

/// Verifies that the generated schema UI contract test sweeps all 6 canonical viewports and 4 physical resolutions.
/// Realises: [REQ-CHECK-40-VERIFY-VIEWPORTS]
fn verify_ui_contract_test_viewports(
    contract_path: &Path,
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    let raw_content = match fs::read_to_string(contract_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let content = strip_dart_comments(&raw_content);

    for vp in [
        "FHD_1080P_1X",
        "QHD_1440P_1X",
        "UHD_4K_1X",
        "UHD_4K_2X",
        "XDR_6K_1X",
        "XDR_6K_2X",
    ] {
        if !content.contains(vp) {
            violations.push(format!(
                "Check 40 violation: Generated UI contract test {} is missing canonical viewport '{}'.",
                file_rel, vp
            ));
        }
    }

    for (w, h) in [
        ("1920", "1080"),
        ("2560", "1440"),
        ("3840", "2160"),
        ("6016", "3384"),
    ] {
        if !content.contains(w) || !content.contains(h) {
            violations.push(format!(
                "Check 40 violation: Generated UI contract test {} is missing physical resolution {}x{}.",
                file_rel, w, h
            ));
        }
    }

    for required_token in [
        "tester.getRect(",
        "tester.tap(",
        "assertPairwiseHudNonOverlap",
        ".overlaps(",
    ] {
        if !content.contains(required_token) {
            violations.push(format!(
                "Check 40 violation: Generated UI contract test {} is missing active '{}' invocation.",
                file_rel, required_token
            ));
        }
    }

    if content.contains("Rect.fromLTWH(") {
        violations.push(format!(
            "Check 40 violation: Generated UI contract test {} uses synthetic Rect.fromLTWH( instead of live tester.getRect(.",
            file_rel
        ));
    }
}

/// Scans a Dart test file for shallow probing helpers, direct bridge mutations, and missing HUD assertions.
/// Realises: [REQ-CHECK-40-SCAN-TEST-GESTURES]
fn scan_test_file_for_interactive_gestures(
    file_path: &Path,
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    let raw_content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let content = strip_dart_comments(&raw_content);

    for (idx, line) in content.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = line.trim();

        if trimmed.contains("probeWidgetKey(") || trimmed.contains("probeWidgetKey ") {
            violations.push(format!(
                "Check 40 violation: Shallow widget probing helper detected in {}:{}: probeWidgetKey is banned.",
                file_rel, line_no
            ));
        }

        if trimmed.contains("evaluate().isNotEmpty") {
            violations.push(format!(
                "Check 40 violation: Shallow evaluate().isNotEmpty check detected in {}:{}. Tests must pump gestures and assert state transitions.",
                file_rel, line_no
            ));
        }
    }

    for (line_no, mut_call) in find_ungestured_test_widgets_mutations(&content, &["bridge."]) {
        violations.push(format!(
            "Check 40 violation: Direct solver bridge mutation detected in {}:{}: '{}'. UI tests must actuate controls via tester.tap(find.byKey(...)).",
            file_rel, line_no, mut_call
        ));
    }

    let mounts_hud = content.contains("HudOverlay")
        || content.contains("TacticalHud")
        || content.contains("HudPanel")
        || content.contains("hud_panel")
        || content.contains("hud_overlay");

    if mounts_hud {
        let has_non_overlap =
            content.contains("assertPairwiseHudNonOverlap") || content.contains(".overlaps(");
        if !has_non_overlap {
            violations.push(format!(
                "Check 40 violation: {}: Test file mounts Tactical HUD overlays but lacks assertPairwiseHudNonOverlap or Rect.overlaps assertion.",
                file_rel
            ));
        }
    }
}

/// Extracts all un-interpolated single-quoted or double-quoted string literals from a Dart source line.
/// Realises: [REQ-CHECK-38-CHECK-PRESENTATION-LINE]
pub(crate) fn extract_uninterpolated_string_literals(line: &str) -> Vec<(usize, String)> {
    let mut results = Vec::new();
    let mut chars = line.chars();
    let mut byte_offset: usize = 0;

    while let Some(ch) = chars.next() {
        let idx = byte_offset;
        byte_offset = byte_offset.saturating_add(ch.len_utf8());
        if ch == '\'' || ch == '"' {
            let quote = ch;
            let mut buf = String::new();
            let mut closed = false;
            while let Some(c) = chars.next() {
                byte_offset = byte_offset.saturating_add(c.len_utf8());
                if c == '\\' {
                    buf.push(c);
                    if let Some(esc) = chars.next() {
                        byte_offset = byte_offset.saturating_add(esc.len_utf8());
                        buf.push(esc);
                    }
                } else if c == quote {
                    closed = true;
                    break;
                } else {
                    buf.push(c);
                }
            }
            if closed && !buf.contains('$') {
                results.push((idx, buf));
            }
        }
    }

    results
}

/// Evaluates whether an individual whitespace-delimited token matches a telemetry or designator pattern.
/// Realises: [REQ-CHECK-38-CHECK-PRESENTATION-LINE]
fn is_hardcoded_telemetry_token(raw_tok: &str) -> bool {
    if raw_tok.starts_with("(P")
        && raw_tok.ends_with(')')
        && raw_tok.len() >= 4
        && raw_tok
            .get(2..raw_tok.len().saturating_sub(1))
            .map(|s| s.chars().all(|c| c.is_ascii_digit()))
            .unwrap_or(false)
    {
        return true;
    }

    let clean_tok = raw_tok.trim_matches(|c: char| {
        c == '(' || c == ')' || c == '[' || c == ']' || c == ',' || c == ';' || c == ':'
    });
    if clean_tok.is_empty() {
        return false;
    }

    let digit_prefix_len = clean_tok.chars().take_while(|c| c.is_ascii_digit()).count();
    if digit_prefix_len >= 1 && digit_prefix_len < clean_tok.len() {
        let unit_suffix = clean_tok.get(digit_prefix_len..).unwrap_or("");
        if matches!(unit_suffix, "x" | "km" | "m" | "s" | "ms" | "Hz" | "dB") {
            return true;
        }
    }

    if clean_tok.contains('-') && clean_tok.len() >= 4 {
        let starts_upper = clean_tok
            .chars()
            .next()
            .map(|c| c.is_ascii_uppercase())
            .unwrap_or(false);
        let segments: Vec<&str> = clean_tok.split('-').collect();
        let valid_segments = segments.len() >= 2
            && segments.iter().all(|seg| {
                !seg.is_empty()
                    && seg
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == ':' || c == '_')
            })
            && segments.get(1..).unwrap_or(&[]).iter().any(|seg| {
                seg.chars()
                    .next()
                    .map(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                    .unwrap_or(false)
            });
        if starts_upper && valid_segments {
            return true;
        }
    }

    false
}

/// Evaluates whether an un-interpolated presentation string literal matches a structural telemetry,
/// status readout, badge, or hyphenated entity designator pattern without regular expressions.
/// Realises: [REQ-CHECK-38-CHECK-PRESENTATION-LINE]
pub(crate) fn is_hardcoded_telemetry_or_designator_literal(inner: &str) -> bool {
    let trimmed = inner.trim();
    if trimmed == "PASS" || trimmed == "FAIL" {
        return true;
    }

    if trimmed
        .as_bytes()
        .windows(2)
        .any(|w| w.first().is_some_and(u8::is_ascii_digit) && w.get(1) == Some(&b'%'))
    {
        return true;
    }

    if trimmed.len() >= 4 && trimmed.starts_with('[') && trimmed.ends_with(']') {
        let badge_body = trimmed.get(1..trimmed.len().saturating_sub(1)).unwrap_or("");
        let upper_count = badge_body.chars().filter(|c| c.is_ascii_uppercase()).count();
        let all_badge_chars = badge_body
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-' || c == '_');
        if upper_count >= 2 && all_badge_chars {
            return true;
        }
    }

    if let Some((left, right)) = trimmed.split_once(':') {
        let left_clean = left.trim_start_matches('(').trim();
        let right_clean = right.trim_end_matches(')').trim();
        if !left_clean.is_empty() && !right_clean.is_empty() {
            let upper_count = left_clean.chars().filter(|c| c.is_ascii_uppercase()).count();
            let has_lower = left_clean.chars().any(|c| c.is_ascii_lowercase());
            if upper_count >= 2 && !has_lower {
                return true;
            }
        }
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();

    for window in tokens.windows(3) {
        if let (Some(&w0), Some(&w1), Some(&w2)) = (window.first(), window.get(1), window.get(2)) {
            let left_is_num = w0.chars().all(|c| c.is_ascii_digit()) && !w0.is_empty();
            let right_is_num = w2.chars().all(|c| c.is_ascii_digit()) && !w2.is_empty();
            if left_is_num && w1 == "/" && right_is_num {
                return true;
            }
        }
    }

    for window in tokens.windows(2) {
        if let (Some(&w0), Some(&w1)) = (window.first(), window.get(1)) {
            let left_is_num = w0.chars().all(|c| c.is_ascii_digit()) && !w0.is_empty();
            let right_clean = w1.trim_matches(|c: char| !c.is_ascii_alphabetic());
            let right_is_upper_word =
                right_clean.len() >= 3 && right_clean.chars().all(|c| c.is_ascii_uppercase());
            if left_is_num && right_is_upper_word {
                return true;
            }
        }
    }

    tokens.iter().any(|tok| is_hardcoded_telemetry_token(tok))
}
