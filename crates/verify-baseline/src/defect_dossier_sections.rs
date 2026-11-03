//! Section-level validators (Sections 1, 2, 4, 6) for defect dossiers.
//! Realises: [REQ-VERIFY-BASELINE-DEFECT-DOSSIER], [DO-330-TQL-4-DEFECT-GATE]

use deap_core::markdown::check_mermaid_syntax;

use crate::defect_dossier::{is_unfilled_placeholder, DossierSeverity};

/// Validates Section 1 (`## 1. Context and References`) for Check 5 and Check 12 compliance.
/// Realises: [REQ-DEFECT-DOSSIER-SECTION1-VALIDATOR]
pub(crate) fn validate_section_1_context(
    sec1_lines: &[String],
    errors: &mut Vec<String>,
) -> (String, String) {
    let mut comment_target: Option<(usize, String)> = None;
    let mut first_bullet_idx: Option<usize> = None;
    let mut file_bullet: Option<String> = None;
    let mut pillar_bullet: Option<String> = None;
    let mut symptom_bullet: Option<String> = None;
    let mut test_target_bullet: Option<String> = None;

    for (idx, raw_line) in sec1_lines.iter().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
            let inner = trimmed.strip_prefix("<!--").and_then(|s| s.strip_suffix("-->")).unwrap_or("").trim();
            if let Some(after_tag) = inner.strip_prefix("test-target:") {
                let target_val = after_tag.trim().trim_matches('`').trim();
                if !target_val.is_empty()
                    && !is_unfilled_placeholder(target_val)
                    && !target_val.chars().any(char::is_whitespace)
                {
                    comment_target = Some((idx, target_val.to_string()));
                }
            }
            continue;
        }

        let bullet_rest = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")).map(str::trim_start);
        if let Some(rest) = bullet_rest {
            if first_bullet_idx.is_none() {
                first_bullet_idx = Some(idx);
            }
            if let Some(val) = rest.strip_prefix("**File**:") {
                let clean = val.trim().trim_matches('`').trim();
                if !clean.is_empty() && !is_unfilled_placeholder(clean) {
                    file_bullet = Some(clean.to_string());
                }
            } else if let Some(val) = rest.strip_prefix("**Pillar**:") {
                let clean = val.trim();
                if !clean.is_empty() && !is_unfilled_placeholder(clean) {
                    pillar_bullet = Some(clean.to_string());
                }
            } else if let Some(val) = rest.strip_prefix("**Symptom**:") {
                let clean = val.trim();
                if !clean.is_empty() && !is_unfilled_placeholder(clean) {
                    symptom_bullet = Some(clean.to_string());
                }
            } else if let Some(val) = rest.strip_prefix("**Test-Target**:") {
                let clean = val.trim().trim_matches('`').trim();
                if !clean.is_empty() && !is_unfilled_placeholder(clean) && !clean.chars().any(char::is_whitespace) {
                    test_target_bullet = Some(clean.to_string());
                }
            }
        }
    }

    check_section_1_results(
        (file_bullet.is_some(), pillar_bullet.is_some(), symptom_bullet.is_some()),
        first_bullet_idx,
        &comment_target,
        &test_target_bullet,
        errors,
    );

    (symptom_bullet.unwrap_or_default(), test_target_bullet.unwrap_or_default())
}

/// Checks Section 1 extracted fields and appends diagnostics to `errors`.
/// Realises: [REQ-DEFECT-DOSSIER-SECTION1-VALIDATOR]
fn check_section_1_results(
    present_flags: (bool, bool, bool),
    first_bullet_idx: Option<usize>,
    comment_target: &Option<(usize, String)>,
    test_target_bullet: &Option<String>,
    errors: &mut Vec<String>,
) {
    if !present_flags.0 {
        errors.push("Section 1 missing mandatory '- **File**:' bullet (Check 5).".to_string());
    }
    if !present_flags.1 {
        errors.push("Section 1 missing mandatory '- **Pillar**:' bullet (Check 5).".to_string());
    }
    if !present_flags.2 {
        errors.push("Section 1 missing mandatory '- **Symptom**:' bullet (Check 5).".to_string());
    }
    if test_target_bullet.is_none() {
        errors.push(
            "Section 1 missing mandatory '- **Test-Target**:' bullet with valid path (Check 5 / Check 12)."
                .to_string(),
        );
    }

    match (comment_target, test_target_bullet) {
        (None, _) => {
            errors.push(
                "Section 1 missing mandatory '<!-- test-target: <path> -->' annotation (Check 12)."
                    .to_string(),
            );
        }
        (Some((c_idx, c_path)), Some(b_path)) => {
            if let Some(b_idx) = first_bullet_idx {
                if *c_idx > b_idx {
                    errors.push(
                        "Section 1 '<!-- test-target: <path> -->' must precede the four bullet lines (Check 12)."
                            .to_string(),
                    );
                }
            }
            if c_path != b_path {
                errors.push(format!(
                    "Section 1 test-target mismatch (Check 12): HTML comment has '{}' but '- **Test-Target**:' bullet has '{}'.",
                    c_path, b_path
                ));
            }
        }
        (Some(_), None) => {}
    }
}

/// Validates Section 2 (`## 2. Root Cause Analysis (5 Whys)`) for Step D Check 6 compliance.
/// Realises: [REQ-DEFECT-DOSSIER-SECTION2-WHYS-VALIDATOR]
pub(crate) fn validate_section_2_whys(sec2_lines: &[String], errors: &mut Vec<String>) {
    for num in 1..=5 {
        let prefix = format!("{}. **Why ", num);
        let mut matched_valid = false;
        for raw_line in sec2_lines {
            let trimmed = raw_line.trim();
            if let Some(after_why) = trimmed.strip_prefix(&prefix) {
                if let Some((question, because_clause)) = after_why.split_once("?** Because ") {
                    let q_clean = question.trim();
                    let b_clean = because_clause.trim();
                    if !q_clean.is_empty()
                        && !is_unfilled_placeholder(q_clean)
                        && !b_clean.is_empty()
                        && !is_unfilled_placeholder(b_clean)
                    {
                        matched_valid = true;
                        break;
                    }
                }
            }
        }
        if !matched_valid {
            errors.push(format!(
                "Section 2 missing or malformed Why item {}: must match '{}. **Why <question>?** Because <reason>.' (Check 6).",
                num, num
            ));
        }
    }
}

/// Validates Section 4 (`## 4. UML Diagrams`) for Step D Check 7 and Check 8 compliance.
/// Realises: [REQ-DEFECT-DOSSIER-SECTION4-MERMAID-VALIDATOR]
pub(crate) fn validate_section_4_uml(
    sec4_lines: &[String],
    full_content: &str,
    severity: Option<DossierSeverity>,
    errors: &mut Vec<String>,
) {
    let has_mermaid_fence = sec4_lines.iter().any(|l| {
        let t = l.trim();
        t == "```mermaid" || t.starts_with("```mermaid ")
    });
    let has_na_marker = sec4_lines.iter().any(|l| l.contains("N/A -- "));

    match severity {
        Some(sev) if sev.requires_mermaid() => {
            if !has_mermaid_fence {
                errors.push(format!(
                    "Section 4 missing mandatory ```mermaid diagram block for {} severity finding (Check 7).",
                    sev.as_str()
                ));
            }
        }
        Some(sev) if !has_na_marker && !has_mermaid_fence => {
            errors.push(format!(
                "Section 4 for {} severity must contain 'N/A -- ' or a valid ```mermaid diagram (Check 8).",
                sev.as_str()
            ));
        }
        Some(_) | None => {}
    }

    if has_mermaid_fence || full_content.contains("```mermaid") {
        let mermaid_errs = check_mermaid_syntax(full_content, "defect_dossier.md");
        for err in mermaid_errs {
            errors.push(format!("Mermaid syntax violation (Check 7): {}", err));
        }

        let mut in_mermaid = false;
        let mut is_class_diagram = false;
        for raw_line in sec4_lines {
            let trimmed = raw_line.trim();
            if !in_mermaid {
                if trimmed == "```mermaid" || trimmed.starts_with("```mermaid ") {
                    in_mermaid = true;
                    is_class_diagram = false;
                }
            } else if trimmed.starts_with("```") {
                in_mermaid = false;
                is_class_diagram = false;
            } else if !trimmed.is_empty() && !trimmed.starts_with("%%") {
                if trimmed.eq_ignore_ascii_case("classDiagram")
                    || trimmed.to_ascii_lowercase().starts_with("classdiagram ")
                {
                    is_class_diagram = true;
                } else if is_class_diagram
                    && (trimmed.contains('{') || trimmed.contains('}'))
                    && !trimmed.starts_with("class ")
                    && trimmed != "{"
                    && trimmed != "}"
                {
                    errors.push(format!(
                        "Mermaid syntax violation (Check 7): curly braces '{{}}' forbidden in class member line: '{}'.",
                        trimmed
                    ));
                }
            }
        }
    }
}

/// Validates Section 6 (`## 6. Proposed Correction`), global fence parity, ASCII art prohibition,
/// and Unicode em dash prohibition (Checks 9 and 10).
/// Realises: [REQ-DEFECT-DOSSIER-FENCES-AND-ASCII-ART-VALIDATOR]
pub(crate) fn validate_fences_ascii_art_and_em_dash(
    sec6_lines: &[String],
    full_content: &str,
    errors: &mut Vec<String>,
) {
    let mut in_sec6_fence = false;
    let mut found_tagged_sec6_fence = false;
    for raw_line in sec6_lines {
        let trimmed = raw_line.trim();
        if let Some(after_ticks) = trimmed.strip_prefix("```") {
            if !in_sec6_fence {
                in_sec6_fence = true;
                let lang_tag = after_ticks.trim();
                if !lang_tag.is_empty() && !is_unfilled_placeholder(lang_tag) {
                    found_tagged_sec6_fence = true;
                }
            } else {
                in_sec6_fence = false;
            }
        }
    }
    if !found_tagged_sec6_fence {
        errors.push(
            "Section 6 ('## 6. Proposed Correction') must contain a triple-backtick code block with a non-empty language tag (e.g. ```rust)."
                .to_string(),
        );
    }

    let mut fence_count = 0usize;
    let mut in_any_fence = false;
    let mut in_mermaid_fence = false;

    for (idx, raw_line) in full_content.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = raw_line.trim_start();
        if let Some(after_ticks) = trimmed.strip_prefix("```") {
            fence_count = fence_count.saturating_add(1);
            if !in_any_fence {
                in_any_fence = true;
                let tag = after_ticks.trim();
                in_mermaid_fence = tag.eq_ignore_ascii_case("mermaid");
            } else {
                in_any_fence = false;
                in_mermaid_fence = false;
            }
            continue;
        }

        if !in_mermaid_fence && (raw_line.contains("->>") || raw_line.contains('\u{2192}')) {
            errors.push(format!(
                "Line {}: forbidden ASCII art arrow ('->>' or '\\u2192') outside ```mermaid block (Check 10).",
                line_no
            ));
        }

        if !in_any_fence && trimmed.starts_with("# ") {
            errors.push(format!(
                "Line {}: forbidden leading '# ' H1 header in dossier body; start directly with '## 1. Context and References'.",
                line_no
            ));
        }
    }

    if !fence_count.is_multiple_of(2) {
        errors.push(format!(
            "Unbalanced code blocks (Check 9): found odd number ({}) of ``` fences.",
            fence_count
        ));
    }

    if full_content.contains('\u{2014}') {
        errors.push(
            "Dossier contains forbidden Unicode em dash (\\u2014); use ASCII '--' instead."
                .to_string(),
        );
    }
}
