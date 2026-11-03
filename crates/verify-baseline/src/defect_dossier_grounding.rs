//! Upstream compiler crate grounding, zero-regex section 6 scanning, and section index helpers.
//! Realises: [REQ-VERIFY-BASELINE-DEFECT-DOSSIER], [DO-330-TQL-4-DEFECT-GATE]

use deap_core::workspace::is_upstream_compiler;
use std::path::Path;

use crate::defect_dossier::{is_unfilled_placeholder, DossierSection, DossierSeverity};

/// Checks whether a crate identifier is one of the six canonical Tier 0 upstream compiler crates.
/// Realises: [REQ-IS-VALID-UPSTREAM-CRATE-NAME]
pub(crate) fn is_valid_upstream_crate_name(name: &str) -> bool {
    name == "deap-core"
        || name == "compile-sysml"
        || name == "ingest-sysml"
        || name == "verify-baseline"
        || name == "assemble-conops"
        || name == "reconcile-backlog"
}

/// Checks whether `haystack` contains `prefix` immediately followed by `suffix`.
/// Realises: [REQ-VERIFY-034]
pub(crate) fn has_split_token(haystack: &str, prefix: &str, suffix: &str) -> bool {
    haystack.match_indices(prefix).any(|(idx, _)| {
        haystack
            .get(idx.saturating_add(prefix.len())..)
            .is_some_and(|rest| rest.starts_with(suffix))
    })
}

/// Inspects a proposed code line for forbidden regular-expression crate tokens using split-token matching.
/// Realises: [REQ-CHECK-FORBIDDEN-REGEX-TOKENS]
pub(crate) fn check_forbidden_regex_tokens(line: &str) -> Option<String> {
    const FORBIDDEN_REGEX_TOKENS: [(&str, &str); 5] = [
        ("re", "gex::"),
        ("Re", "gex::new"),
        ("Re", "gexSet"),
        ("use re", "gex"),
        ("extern crate r", "egex"),
    ];

    let stripped_diff = line
        .trim_start()
        .trim_start_matches(['+', '-'])
        .trim_start();
    let collapsed = stripped_diff
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let normalized = collapsed
        .replace(" :: ", "::")
        .replace(" ::", "::")
        .replace(":: ", "::");

    for candidate in [line, collapsed.as_str(), normalized.as_str()] {
        for (prefix, suffix) in FORBIDDEN_REGEX_TOKENS {
            if has_split_token(candidate, prefix, suffix) {
                return Some(format!("{prefix}{suffix}"));
            }
        }
    }
    None
}

/// Validates Section 6 (`## 6. Proposed Correction`) code blocks for Check 34 Zero Regex Invariant compliance.
/// Realises: [REQ-DEFECT-DOSSIER-SECTION6-ZERO-REGEX-GATE]
pub(crate) fn validate_section_6_zero_regex(sec6_lines: &[String], errors: &mut Vec<String>) {
    let mut in_checked_fence = false;
    for raw_line in sec6_lines {
        let trimmed = raw_line.trim();
        if let Some(after_ticks) = trimmed.strip_prefix("```") {
            if !in_checked_fence {
                let tag = after_ticks.trim();
                let lower = tag.to_ascii_lowercase();
                if lower == "rust"
                    || lower == "rs"
                    || lower == "diff"
                    || lower == "patch"
                    || lower.starts_with("rust ")
                    || lower.starts_with("rust,")
                    || lower.starts_with("rs ")
                    || lower.starts_with("rs,")
                    || lower.starts_with("diff ")
                    || lower.starts_with("diff,")
                    || lower.starts_with("patch ")
                    || lower.starts_with("patch,")
                {
                    in_checked_fence = true;
                }
            } else {
                in_checked_fence = false;
            }
            continue;
        }

        if in_checked_fence {
            if let Some(token) = check_forbidden_regex_tokens(trimmed) {
                errors.push(format!(
                    "Check 34 Zero Regex Invariant violation in Section 6: forbidden regex token '{}' in proposed correction.",
                    token
                ));
            }
        }
    }
}

/// Validates that Section 1 `test-target` is grounded in a concrete file path within one of the 6
/// upstream compiler crates when in an `UPSTREAM_SPEC_CORE_COMPILER` workspace.
/// Realises: [REQ-DEFECT-DOSSIER-UPSTREAM-TEST-TARGET-GROUNDING]
pub(crate) fn validate_upstream_test_target_grounding(
    test_target: &str,
    is_upstream: bool,
    errors: &mut Vec<String>,
) {
    if !is_upstream {
        return;
    }

    let clean = test_target.trim().trim_matches('`').trim();
    let is_valid = if let Some(after_crates) = clean.strip_prefix("crates/") {
        if let Some((crate_name, subpath)) = after_crates.split_once('/') {
            let clean_subpath = subpath.trim().trim_matches('/');
            is_valid_upstream_crate_name(crate_name) && !clean_subpath.is_empty()
        } else {
            false
        }
    } else {
        false
    };

    if !is_valid {
        errors.push(format!(
            "Upstream test-target crate grounding violation: '{}' does not resolve to a file inside one of the 6 upstream compiler crates (crates/{{deap-core, compile-sysml, ingest-sysml, verify-baseline, assemble-conops, reconcile-backlog}}/<file>).",
            clean
        ));
    }
}

/// Detects whether the current workspace or provided root directory is an UPSTREAM_SPEC_CORE_COMPILER workspace.
/// Realises: [REQ-DETECT-IS-UPSTREAM-COMPILER]
pub(crate) fn detect_is_upstream_compiler(workspace_root: Option<&Path>) -> bool {
    if let Some(root) = workspace_root {
        if is_upstream_compiler(root) {
            return true;
        }
        let mut cur = root;
        while let Some(parent) = cur.parent() {
            if is_upstream_compiler(parent) {
                return true;
            }
            cur = parent;
        }
    }
    if is_upstream_compiler(Path::new(".")) {
        return true;
    }
    if let Ok(abs_cur) = std::env::current_dir() {
        let mut cur = abs_cur.as_path();
        loop {
            if is_upstream_compiler(cur) {
                return true;
            }
            match cur.parent() {
                Some(p) => cur = p,
                None => break,
            }
        }
    }
    false
}

/// Locates the 7 canonical section headers and `## Audit Source` and validates ordering.
/// Realises: [REQ-VALIDATE-DEFECT-DOSSIER-SCOPED]
pub(crate) fn locate_and_verify_dossier_sections(
    lines: &[String],
    errors: &mut Vec<String>,
) -> (Vec<(DossierSection, usize)>, Option<usize>) {
    let mut section_indices: Vec<(DossierSection, usize)> = Vec::new();
    let mut audit_source_idx: Option<usize> = None;
    let mut in_fence = false;

    for (idx, raw_line) in lines.iter().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        for &sec in DossierSection::ALL {
            if trimmed == sec.header() && !section_indices.iter().any(|(s, _)| *s == sec) {
                section_indices.push((sec, idx));
            }
        }
        if trimmed == "## Audit Source" && audit_source_idx.is_none() {
            audit_source_idx = Some(idx);
        }
    }

    let mut prev_line_idx = 0usize;
    for &sec in DossierSection::ALL {
        match section_indices.iter().find(|(s, _)| *s == sec) {
            Some((_, idx)) => {
                if *idx < prev_line_idx {
                    errors.push(format!(
                        "Section header '{}' is out of order (Check 1).",
                        sec.header()
                    ));
                }
                prev_line_idx = *idx;
            }
            None => {
                errors.push(format!(
                    "Missing mandatory exact section header '{}' (Check 1).",
                    sec.header()
                ));
            }
        }
    }

    match audit_source_idx {
        Some(idx) => {
            if idx < prev_line_idx {
                errors.push(
                    "Section header '## Audit Source' must appear after Section 7 (Check 2)."
                        .to_string(),
                );
            }
        }
        None => {
            errors
                .push("Missing mandatory '## Audit Source' section header (Check 2).".to_string());
        }
    }

    (section_indices, audit_source_idx)
}

/// Extracts and validates `SEVERITY:` (Check 3) and `FILE_LOCATION:` (Check 4) lines.
/// Realises: [REQ-VALIDATE-DEFECT-DOSSIER-SCOPED]
pub(crate) fn extract_severity_and_file_location(
    lines: &[String],
    errors: &mut Vec<String>,
) -> (Option<DossierSeverity>, String, String) {
    let mut parsed_severity: Option<DossierSeverity> = None;
    for raw_line in lines {
        let trimmed = raw_line.trim();
        if let Some(rest) = trimmed.strip_prefix("SEVERITY:") {
            if let Some(sev) = DossierSeverity::parse(rest) {
                parsed_severity = Some(sev);
                break;
            }
        }
    }
    let severity_str = match parsed_severity {
        Some(sev) => sev.as_str().to_string(),
        None => {
            errors.push(
                "Missing or invalid 'SEVERITY:' line (must be Critical, Important, Suggestion, or Nitpick) (Check 3)."
                    .to_string(),
            );
            String::new()
        }
    };

    let mut file_location_opt: Option<String> = None;
    for raw_line in lines {
        let trimmed = raw_line.trim();
        if let Some(rest) = trimmed.strip_prefix("FILE_LOCATION:") {
            let val = rest.trim().trim_matches('`').trim();
            if !val.is_empty() && !is_unfilled_placeholder(val) {
                if let Some((path_part, line_part)) = val.rsplit_once(':') {
                    let has_valid_line = line_part
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false);
                    if !path_part.trim().is_empty()
                        && !is_unfilled_placeholder(path_part)
                        && has_valid_line
                    {
                        file_location_opt = Some(val.to_string());
                        break;
                    }
                }
            }
        }
    }
    let file_location = match file_location_opt {
        Some(loc) => loc,
        None => {
            errors.push(
                "Missing or invalid mandatory 'FILE_LOCATION: <path>:<line>' line (Check 4)."
                    .to_string(),
            );
            String::new()
        }
    };

    (parsed_severity, severity_str, file_location)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_split_token_and_forbidden_regex_scanner() {
        assert!(has_split_token("alpha_beta_gamma", "alpha_", "beta"));
        assert!(has_split_token("\u{1F50D}token_tail", "token_", "tail"));
        assert!(!has_split_token("token_gap_tail", "token_", "tail"));

        let sample_use = format!("+ {} {};", "use", "regex");
        assert_eq!(
            check_forbidden_regex_tokens(&sample_use),
            Some(format!("{}{}", "use re", "gex"))
        );

        let sample_spaced = format!("let x = {} :: new(\"a\");", "Regex");
        assert_eq!(
            check_forbidden_regex_tokens(&sample_spaced),
            Some(format!("{}{}", "Re", "gex::new"))
        );

        assert_eq!(
            check_forbidden_regex_tokens("let ok = text.strip_prefix(\"abc\");"),
            None
        );
    }
}
