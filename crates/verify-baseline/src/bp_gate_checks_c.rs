#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Check 8 (BDD Acceptance Criteria & Test-Target Gate) for `blueprint_gate`.
//! Realises: [REQ-BLUEPRINT-CHECK-8-BDD-TEST-TARGET]

pub use crate::bp_gate_contract::check_9_rust_code_blocks;
#[cfg(test)]
pub(crate) use crate::bp_gate_contract::{has_split_token, scan_rust_contract_line};
use crate::bp_gate_tables::strip_markdown_decorations;
use std::path::Path;

/// Checks whether `target` starts with one of the 6 canonical upstream workspace crate prefixes.
/// Realises: [REQ-BLUEPRINT-CHECK-8-CRATE-WHITELIST]
fn is_allowed_upstream_crate_test_target(target: &str) -> bool {
    target.starts_with("crates/deap-core/")
        || target.starts_with("crates/verify-baseline/")
        || target.starts_with("crates/compile-sysml/")
        || target.starts_with("crates/ingest-sysml/")
        || target.starts_with("crates/reconcile-backlog/")
        || target.starts_with("crates/assemble-conops/")
}

/// Extracts an `AC-XX` identifier if `trimmed` begins an acceptance criterion heading or bullet.
/// Realises: [REQ-BLUEPRINT-CHECK-8-AC-HEADER]
fn extract_ac_header_id(trimmed: &str) -> Option<String> {
    let is_heading_or_bullet = trimmed.starts_with('#')
        || trimmed.starts_with("- **AC-")
        || trimmed.starts_with("* **AC-")
        || trimmed.starts_with("- AC-")
        || trimmed.starts_with("* AC-")
        || trimmed.starts_with("**AC-");
    if !is_heading_or_bullet {
        return None;
    }
    let stripped = trimmed
        .trim_start_matches('#')
        .trim_start_matches('-')
        .trim_start_matches('*')
        .trim()
        .trim_start_matches('*')
        .trim_start_matches('`')
        .trim();
    let rest = stripped.strip_prefix("AC-")?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    (!digits.is_empty()).then(|| format!("AC-{digits}"))
}

/// Checks whether an acceptance criteria text block contains Given/When/Then and negative keywords.
/// Realises: [REQ-BLUEPRINT-CHECK-8-AC-CLAUSES]
fn validate_single_ac_block(ac_id: &str, ac_lines: &[&str], errors: &mut Vec<String>) -> bool {
    let joined = ac_lines.join("\n");
    let has_given = joined.contains("**Given**") || joined.contains("Given:") || joined.contains("Given ");
    let has_when = joined.contains("**When**") || joined.contains("When:") || joined.contains("When ");
    let has_then = joined.contains("**Then**") || joined.contains("Then:") || joined.contains("Then ");

    if !has_given || !has_when || !has_then {
        errors.push(format!(
            "Check 8 violation (Section 5 {ac_id}): Acceptance criterion '{ac_id}' must contain Given, When, and Then clauses (Given={has_given}, When={has_when}, Then={has_then})"
        ));
    }

    let lower = joined.to_ascii_lowercase();
    let neg_tokens = [
        "negative", "fail-closed", "fail closed", "reject", "invalid", "violation", "error", "non-zero",
    ];
    neg_tokens.iter().any(|tok| lower.contains(tok))
}

/// Validates that a single Section 5 test-target path is a relative `.rs` file in an upstream crate.
/// Realises: [REQ-BLUEPRINT-CHECK-8-BDD-TEST-TARGET]
fn validate_test_target_path(target: &str, errors: &mut Vec<String>) {
    let valid_crate_prefix = is_allowed_upstream_crate_test_target(target);
    let valid_rs_ext = Path::new(target)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"));
    if !valid_crate_prefix || !valid_rs_ext || target.contains("..") {
        errors.push(format!(
            "Check 8 violation (Section 5): test-target '{target}' must be a relative '.rs' path inside one of the 6 upstream crates (crates/deap-core/, crates/verify-baseline/, crates/compile-sysml/, crates/ingest-sysml/, crates/reconcile-backlog/, crates/assemble-conops/)"
        ));
    }
}

/// Collects every `<!-- test-target: ... -->` comment and `- **Test-Target**: ...` bullet in Section 5.
/// Realises: [REQ-BLUEPRINT-CHECK-8-BDD-TEST-TARGET]
fn collect_section_5_test_targets(section_5_lines: &[&str]) -> (Vec<String>, Vec<String>) {
    let mut comment_targets: Vec<String> = Vec::new();
    let mut bullet_targets: Vec<String> = Vec::new();
    for raw_line in section_5_lines {
        let trimmed = raw_line.trim();
        if let Some(inner) = trimmed.strip_prefix("<!--").and_then(|s| s.strip_suffix("-->")) {
            if let Some(rest) = inner.trim().strip_prefix("test-target:") {
                comment_targets.push(rest.trim().to_string());
            }
        }
        let bullet_stripped = trimmed
            .strip_prefix('-')
            .or_else(|| trimmed.strip_prefix('*'))
            .map_or(trimmed, str::trim);
        if let Some(rest) = bullet_stripped.strip_prefix("**Test-Target**:") {
            bullet_targets.push(strip_markdown_decorations(rest));
        }
    }
    (comment_targets, bullet_targets)
}

/// Reconciles multi-target `<!-- test-target: ... -->` comments against `- **Test-Target**: ...` bullets.
/// Realises: [REQ-BLUEPRINT-CHECK-8-BDD-TEST-TARGET]
fn reconcile_test_target_bindings(
    comment_targets: &[String],
    bullet_targets: &[String],
    errors: &mut Vec<String>,
) -> Vec<String> {
    if comment_targets.is_empty() && bullet_targets.is_empty() {
        errors.push("Check 8 violation (Section 5): Missing both '<!-- test-target: <path> -->' and '- **Test-Target**: `<path>`' declarations".to_string());
        return Vec::new();
    }
    if comment_targets.is_empty() {
        errors.push("Check 8 violation (Section 5): Missing mandatory '<!-- test-target: <path> -->' HTML comment binding".to_string());
    } else if bullet_targets.is_empty() {
        errors.push("Check 8 violation (Section 5): Missing mandatory '- **Test-Target**: `<path>`' field in Section 5".to_string());
    }
    if comment_targets.len() != bullet_targets.len() {
        errors.push(format!(
            "Check 8 violation (Section 5): test-target declaration count mismatch: {} HTML comment(s) vs {} '- **Test-Target**' bullet(s); both counts must be equal and non-zero",
            comment_targets.len(),
            bullet_targets.len()
        ));
    }
    for (idx, (c_path, b_path)) in comment_targets.iter().zip(bullet_targets.iter()).enumerate() {
        if c_path != b_path {
            errors.push(format!(
                "Check 8 violation (Section 5): test-target declaration #{} HTML comment '<!-- test-target: {c_path} -->' does not match '- **Test-Target**: `{b_path}`'",
                idx.saturating_add(1)
            ));
        }
    }

    let mut validated_paths: Vec<&String> = Vec::new();
    for target in comment_targets.iter().chain(bullet_targets.iter()) {
        if !validated_paths.contains(&target) {
            validated_paths.push(target);
        }
    }
    for target in validated_paths {
        validate_test_target_path(target, errors);
    }
    comment_targets.to_vec()
}

/// Check 8 (BDD Acceptance Criteria & Test-Target Gate): Validates Section 5 Given/When/Then ACs and multi-target bindings.
/// Realises: [REQ-BLUEPRINT-CHECK-8-BDD-TEST-TARGET]
pub fn check_8_bdd_and_test_target(
    section_5_lines: &[&str],
) -> Result<(usize, Vec<String>), Vec<String>> {
    let mut errors = Vec::new();
    let mut ac_blocks: Vec<(String, Vec<&str>)> = Vec::new();

    for raw_line in section_5_lines {
        let trimmed = raw_line.trim();
        if let Some(ac_id) = extract_ac_header_id(trimmed) {
            ac_blocks.push((ac_id, vec![*raw_line]));
        } else if let Some((_, ref mut lines_vec)) = ac_blocks.last_mut() {
            lines_vec.push(*raw_line);
        }
    }

    let mut distinct_ids: Vec<String> = Vec::new();
    let mut has_negative_scenario = false;
    for (ac_id, block_lines) in &ac_blocks {
        if !distinct_ids.contains(ac_id) {
            distinct_ids.push(ac_id.clone());
        }
        if validate_single_ac_block(ac_id, block_lines, &mut errors) {
            has_negative_scenario = true;
        }
    }

    if distinct_ids.len() < 5 {
        errors.push(format!(
            "Check 8 violation (Section 5): Section 5 must contain >= 5 distinct AC-XX acceptance criteria scenarios; found {}",
            distinct_ids.len()
        ));
    }
    if !has_negative_scenario {
        errors.push("Check 8 violation (Section 5): Section 5 must include >= 1 negative / fail-closed acceptance scenario".to_string());
    }

    let (comment_targets, bullet_targets) = collect_section_5_test_targets(section_5_lines);
    let verified_targets =
        reconcile_test_target_bindings(&comment_targets, &bullet_targets, &mut errors);
    if errors.is_empty() {
        Ok((distinct_ids.len(), verified_targets))
    } else {
        Err(errors)
    }
}
