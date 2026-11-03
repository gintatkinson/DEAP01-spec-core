#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Checks 6 and 7 (Mermaid E1-E5 and LaTeX/KaTeX Integrity) for `blueprint_gate`.
//! Realises: [REQ-BLUEPRINT-CHECK-6-MERMAID], [REQ-BLUEPRINT-CHECK-7-LATEX-KATEX]

/// Strips double-quoted (`"..."`) segments from a single line of text, replacing interior chars with spaces.
/// Realises: [REQ-BLUEPRINT-CHECK-6-QUOTE-MASK]
fn mask_double_quoted_segments(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_quote = false;
    let mut escaped = false;
    for ch in line.chars() {
        if in_quote {
            if escaped {
                escaped = false;
                out.push(' ');
            } else if ch == '\\' {
                escaped = true;
                out.push(' ');
            } else if ch == '"' {
                in_quote = false;
                out.push('"');
            } else {
                out.push(' ');
            }
        } else {
            if ch == '"' {
                in_quote = true;
            }
            out.push(ch);
        }
    }
    out
}

/// Checks whether a Mermaid diagram body line contains any unquoted `<` or `>` outside valid arrows/stereotypes.
/// Realises: [REQ-BLUEPRINT-CHECK-6-ANGLE-BRACKETS]
fn mermaid_line_has_unquoted_angle(trimmed: &str) -> bool {
    let mut masked = mask_double_quoted_segments(trimmed);
    while let Some(start) = masked.find("<<") {
        let after_start = start.saturating_add(2);
        let Some(rel_end) = masked.get(after_start..).unwrap_or("").find(">>") else { break };
        let end_idx = after_start.saturating_add(rel_end).saturating_add(2);
        if masked.get(start..end_idx).is_some() {
            masked.replace_range(start..end_idx, " ");
        } else {
            break;
        }
    }
    for arrow in [
        "<|--", "--|>", "<|..", "..|>", "<<--", "-->>", "<<==", "==>>", "<-->", "<==>", "<-.-",
        "-.->", "==>", "-->", "->>", "<==", "<--", "<<-", "<->", "..>", "<..", "->", "<-",
    ] {
        masked = masked.replace(arrow, " ");
    }
    masked.contains('<') || masked.contains('>')
}

/// Validates a single body line inside a `classDiagram` Mermaid block against Rule E2 (`{}` and `:`).
/// Realises: [REQ-BLUEPRINT-CHECK-6-CLASS-MEMBER-SYNTAX]
fn check_class_diagram_member_line(trimmed: &str, line_no: usize, errors: &mut Vec<String>) {
    let masked = mask_double_quoted_segments(trimmed);
    let is_structural_open = (masked.starts_with("class ") || masked.starts_with("namespace "))
        && masked.trim_end().ends_with('{')
        && !masked.contains('(')
        && !masked.contains('}');
    let is_structural_close = masked.trim() == "}";
    if (masked.contains('{') || masked.contains('}')) && !is_structural_open && !is_structural_close {
        errors.push(format!("Check 6 violation (Section 3 line {line_no}): Curly braces '{{}}' are prohibited in classDiagram members: '{trimmed}'"));
    }
    let without_scope = masked.replace("::", "  ");
    let colon_count = without_scope.chars().filter(|c| *c == ':').count();
    let is_vis = trimmed.starts_with('+') || trimmed.starts_with('-') || trimmed.starts_with('#') || trimmed.starts_with('~');
    if colon_count >= 2 || (is_vis && colon_count >= 1) {
        errors.push(format!("Check 6 violation (Section 3 line {line_no}): Secondary or member colons (':') are prohibited in classDiagram members: '{trimmed}'"));
    }
}

/// Checks whether a Mermaid header line matches one of the 4 allowed Section 3 diagram types.
/// Realises: [REQ-BLUEPRINT-CHECK-6-HEADER-WHITELIST]
fn is_allowed_section_3_mermaid_header(trimmed: &str) -> bool {
    matches!(trimmed, "classDiagram" | "stateDiagram-v2" | "sequenceDiagram" | "flowchart TD" | "graph TD")
        || trimmed.starts_with("flowchart TD ")
        || trimmed.starts_with("graph TD ")
}

/// Check 6 (Mermaid E1-E5 Syntax Gate): Validates that Section 3 contains `>= 1` valid ` ```mermaid ` diagram.
/// Realises: [REQ-BLUEPRINT-CHECK-6-MERMAID]
pub fn check_6_mermaid_syntax(section_3_lines: &[&str]) -> Result<usize, Vec<String>> {
    let joined = section_3_lines.join("\n");
    let mut errors = deap_core::markdown::check_mermaid_syntax(&joined, "Section 3");

    let mut in_mermaid = false;
    let mut in_other_fence = false;
    let mut mermaid_count = 0usize;
    let mut current_header: Option<String> = None;

    for (idx, raw_line) in section_3_lines.iter().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = raw_line.trim();

        if !in_mermaid && !in_other_fence {
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                let lang = trimmed.trim_start_matches('`').trim_start_matches('~').trim();
                if lang.eq_ignore_ascii_case("mermaid") {
                    in_mermaid = true;
                    mermaid_count = mermaid_count.saturating_add(1);
                    current_header = None;
                } else {
                    in_other_fence = true;
                }
            }
            continue;
        }
        if in_other_fence {
            if trimmed == "```" || trimmed == "~~~" {
                in_other_fence = false;
            }
            continue;
        }
        if trimmed == "```" || trimmed == "~~~" {
            if current_header.is_none() {
                errors.push(format!("Check 6 violation (Section 3 line {line_no}): Empty Mermaid block without diagram type header"));
            }
            in_mermaid = false;
            current_header = None;
            continue;
        }
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            errors.push(format!("Check 6 violation (Section 3 line {line_no}): Leaking or nested fence '{trimmed}' inside Mermaid block"));
            in_mermaid = false;
            current_header = None;
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }
        if let Some(ref hdr) = current_header {
            if mermaid_line_has_unquoted_angle(trimmed) {
                errors.push(format!("Check 6 violation (Section 3 line {line_no}): Unquoted '<' or '>' in Mermaid diagram line: '{trimmed}'"));
            }
            if hdr == "classDiagram" {
                check_class_diagram_member_line(trimmed, line_no, &mut errors);
            }
        } else {
            if !is_allowed_section_3_mermaid_header(trimmed) {
                errors.push(format!("Check 6 violation (Section 3 line {line_no}): Mermaid diagram header must be one of classDiagram, stateDiagram-v2, sequenceDiagram, or flowchart TD / graph TD; found '{trimmed}'"));
            }
            current_header = Some(trimmed.to_string());
        }
    }

    if in_mermaid || in_other_fence {
        errors.push("Check 6 violation (Section 3): Unclosed / leaking code fence at end of Section 3".to_string());
    }
    if mermaid_count == 0 {
        errors.push("Check 6 violation (Section 3): Section 3 ('## 3. UML & Behavioral Topology') must contain >= 1 fenced ```mermaid diagram".to_string());
    }
    if errors.is_empty() { Ok(mermaid_count) } else { Err(errors) }
}

/// Checks whether a candidate string inside inline `$ ... $` delimiters is a non-mathematical uppercase identifier.
/// Realises: [REQ-BLUEPRINT-CHECK-7-INLINE-ID]
fn is_inline_math_identifier_token(candidate: &str) -> bool {
    let trimmed = candidate.trim();
    let Some((prefix, rest)) = trimmed.split_once('-') else {
        return false;
    };
    if prefix.is_empty() || rest.is_empty() {
        return false;
    }
    let valid_prefix = prefix.chars().all(|c| c.is_ascii_uppercase());
    let has_digit = rest.chars().any(|c| c.is_ascii_digit());
    let valid_rest = rest
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-' || c == '_');
    valid_prefix && has_digit && valid_rest
}

/// Scans a single non-code markdown line for inline `$...$` spans wrapping non-mathematical identifiers.
/// Realises: [REQ-BLUEPRINT-CHECK-7-LINE-SCAN]
fn scan_line_for_inline_math_ids(
    line: &str,
    scope_label: &str,
    line_no: usize,
    errors: &mut Vec<String>,
) {
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut idx = 0usize;
    let mut in_code_span = false;

    while idx < len {
        let ch = chars.get(idx).copied().unwrap_or(' ');
        if ch == '\\' {
            idx = idx.saturating_add(2);
            continue;
        }
        if ch == '`' {
            in_code_span = !in_code_span;
            idx = idx.saturating_add(1);
            continue;
        }
        if in_code_span {
            idx = idx.saturating_add(1);
            continue;
        }
        if ch == '$' {
            if chars.get(idx.saturating_add(1)).copied() == Some('$') {
                idx = idx.saturating_add(2);
                continue;
            }
            let mut close_idx = idx.saturating_add(1);
            let mut span = String::new();
            let mut found_close = false;
            while close_idx < len {
                let c = chars.get(close_idx).copied().unwrap_or(' ');
                if c == '\\' {
                    if let Some(esc) = chars.get(close_idx.saturating_add(1)).copied() {
                        span.push(c);
                        span.push(esc);
                    }
                    close_idx = close_idx.saturating_add(2);
                    continue;
                }
                if c == '$' {
                    found_close = true;
                    break;
                }
                span.push(c);
                close_idx = close_idx.saturating_add(1);
            }
            if found_close {
                if is_inline_math_identifier_token(&span) {
                    errors.push(format!(
                        "Check 7 violation ({scope_label} line {line_no}): Non-mathematical identifier '${span}$' must not use '$...$' math delimiters; use bold or code spans instead"
                    ));
                }
                idx = close_idx.saturating_add(1);
                continue;
            }
        }
        idx = idx.saturating_add(1);
    }
}

/// Scans the full blueprint document outside fenced code blocks for Check 7 math hygiene violations.
/// Realises: [REQ-BLUEPRINT-CHECK-7-LATEX-KATEX]
fn scan_full_blueprint_math_lines(full_content: &str, errors: &mut Vec<String>) {
    let mut doc_in_fence = false;
    let mut doc_in_math = false;
    for (idx, raw_line) in full_content.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = raw_line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            doc_in_fence = !doc_in_fence;
            continue;
        }
        if doc_in_fence {
            continue;
        }
        if trimmed.contains("\\begin{align}") || trimmed.contains("\\end{align}") {
            errors.push(format!(
                "Check 7 violation (Blueprint line {line_no}): Top-level '\\begin{{align}}' / '\\end{{align}}' is forbidden"
            ));
        }
        if trimmed.contains("$$") && trimmed != "$$" {
            errors.push(format!(
                "Check 7 violation (Blueprint line {line_no}): Display math '$$' delimiter must be isolated on its own line"
            ));
        }
        if trimmed == "$$" {
            doc_in_math = !doc_in_math;
            continue;
        }
        if !doc_in_math {
            scan_line_for_inline_math_ids(raw_line, "Blueprint", line_no, errors);
        }
    }
}

/// Check 7 (LaTeX / KaTeX Integrity Gate -- Check 27): Validates mathematical rendering integrity.
/// Realises: [REQ-BLUEPRINT-CHECK-7-LATEX-KATEX]
pub fn check_7_latex_katex(
    section_4_lines: &[&str],
    full_content: &str,
) -> Result<usize, Vec<String>> {
    let sec4_joined = section_4_lines.join("\n");
    let mut errors = deap_core::markdown::check_latex_katex_syntax(full_content, "Blueprint");
    for err in deap_core::markdown::check_latex_katex_syntax(&sec4_joined, "Section 4") {
        if !errors.contains(&err) {
            errors.push(err);
        }
    }

    let mut in_fence = false;
    let mut in_display_math = false;
    let mut current_has_begin_aligned = false;
    let mut current_has_end_aligned = false;
    let mut aligned_block_count = 0usize;

    for (idx, raw_line) in section_4_lines.iter().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = raw_line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if trimmed.contains("\\begin{align}") || trimmed.contains("\\end{align}") {
            errors.push(format!(
                "Check 7 violation (Section 4 line {line_no}): Top-level '\\begin{{align}}' / '\\end{{align}}' is forbidden; use '\\begin{{aligned}} ... \\end{{aligned}}' inside isolated '$$'"
            ));
        }
        if trimmed.contains("$$") && trimmed != "$$" {
            errors.push(format!(
                "Check 7 violation (Section 4 line {line_no}): Display math '$$' delimiter must be on its own isolated line: '{trimmed}'"
            ));
        }
        if trimmed == "$$" {
            if !in_display_math {
                in_display_math = true;
                current_has_begin_aligned = false;
                current_has_end_aligned = false;
            } else {
                if current_has_begin_aligned && current_has_end_aligned {
                    aligned_block_count = aligned_block_count.saturating_add(1);
                }
                in_display_math = false;
            }
            continue;
        }
        if in_display_math {
            if trimmed.contains("\\begin{aligned}") {
                current_has_begin_aligned = true;
            }
            if trimmed.contains("\\end{aligned}") && current_has_begin_aligned {
                current_has_end_aligned = true;
            }
        } else {
            scan_line_for_inline_math_ids(raw_line, "Section 4", line_no, &mut errors);
        }
    }

    if in_display_math {
        errors.push("Check 7 violation (Section 4): Unclosed '$$' display math block in Section 4".to_string());
    }
    if aligned_block_count == 0 {
        errors.push(
            "Check 7 violation (Section 4): Section 4 ('## 4. Formal Mathematical & Safety Constraints') must contain >= 1 isolated '$$' block wrapping '\\begin{aligned} ... \\end{aligned}'"
                .to_string(),
        );
    }

    scan_full_blueprint_math_lines(full_content, &mut errors);
    if errors.is_empty() {
        Ok(aligned_block_count)
    } else {
        Err(errors)
    }
}
