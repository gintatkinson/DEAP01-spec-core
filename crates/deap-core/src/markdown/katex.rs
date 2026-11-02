//! LaTeX and KaTeX syntax validation in markdown documents.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives.
//! Enforces mathematical syntax rules, delimiter balancing, and alignment environment discipline.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Delimiter parity: `$$` count must be an even integer.
//! - Alignment discipline: Bare `&` operators are forbidden outside whitelisted alignment environments.
//!
//! ## 3. Preconditions
//! - Input strings must be valid UTF-8 sequences.
//!
//! ## 4. Postconditions
//! - Returns list of syntax diagnostic strings citing workspace-relative file paths.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) bounded linearly by document byte length.
//! - Space Complexity: O(N) auxiliary diagnostic memory.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Fails closed by returning formatted diagnostic error strings.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-MD-MODULE], [DO-330-TQL-4-MARKDOWN].

use super::alignment::ALLOWED_ALIGNMENT_ENVS;
use super::sanitization::strip_code_and_glfm;
use pulldown_cmark::{Event, Options, Parser};

/// Validates LaTeX / KaTeX rendering syntax across markdown content.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Enforces Universal LaTeX & KaTeX Mathematical Rendering Rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Display Delimiter Parity: Number of `$$` tokens in sanitized content must be even.
/// - Environment Stack Discipline: Every unescaped `&` must be within an allowed environment.
///
/// ## 3. Preconditions
/// `content` is a valid UTF-8 markdown string slice; `file_rel_path` is a relative path.
///
/// ## 4. Postconditions
/// Returns empty vector on valid syntax; otherwise returns descriptive diagnostic strings.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N). Space Complexity: O(N).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Non-panicking total function. Uses character boundary clamping when extracting snippets.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-MD-KATEX]
pub fn check_latex_katex_syntax(content: &str, file_rel_path: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let cleaned = strip_code_and_glfm(content);

    let delimiter_count = cleaned.matches("$$").count();
    if !delimiter_count.is_multiple_of(2) {
        errors.push(format!(
            "Unbalanced $$ display math delimiters in {} (found {} delimiters).",
            file_rel_path, delimiter_count
        ));
        return errors;
    }

    validate_aligned_pairs(&cleaned, file_rel_path, &mut errors);

    let parser = Parser::new_ext(&cleaned, Options::all() | Options::ENABLE_MATH);
    let mut math_spans = Vec::new();

    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::DisplayMath(math) | Event::InlineMath(math) => {
                math_spans.push((math.to_string(), range));
            }
            _ => {}
        }
    }

    if math_spans.is_empty() && delimiter_count > 0 {
        let indices: Vec<usize> = cleaned.match_indices("$$").map(|(i, _)| i).collect();
        let (chunks, _) = indices.as_chunks::<2>();
        for &[open_idx, close_idx] in chunks {
            let start = open_idx + 2;
            let end = close_idx;
            if let Some(block) = cleaned.get(start..end) {
                math_spans.push((block.to_string(), start..end));
            }
        }
    }

    for (block, _) in &math_spans {
        validate_single_math_block(block, file_rel_path, &mut errors);
    }

    errors
}

fn validate_aligned_pairs(cleaned: &str, file_rel_path: &str, errors: &mut Vec<String>) {
    let num_begin = cleaned.matches(r"\begin{aligned}").count();
    let num_end = cleaned.matches(r"\end{aligned}").count();
    if num_begin != num_end {
        errors.push(format!(
            "Unbalanced \\begin{{aligned}} ({}) and \\end{{aligned}} ({}) pairs in {}.",
            num_begin, num_end, file_rel_path
        ));
    }
}

fn validate_single_math_block(block: &str, file_rel_path: &str, errors: &mut Vec<String>) {
    if block.contains(r"\begin{align}") || block.contains(r"\begin{align*}") {
        errors.push(format!(
            "Forbidden \\begin{{align}} or \\begin{{align*}} found in display math block in {}. In markdown KaTeX, \\begin{{aligned}} must be used instead.",
            file_rel_path
        ));
    }

    let num_begin = block.matches(r"\begin{aligned}").count();
    let num_end = block.matches(r"\end{aligned}").count();
    if num_begin != num_end {
        errors.push(format!(
            "Unbalanced \\begin{{aligned}} ({}) and \\end{{aligned}} ({}) in math block in {}.",
            num_begin, num_end, file_rel_path
        ));
    }

    scan_math_block_env(block, file_rel_path, errors);
}

fn scan_math_block_env(block: &str, file_rel_path: &str, errors: &mut Vec<String>) {
    let mut env_stack: Vec<String> = Vec::new();
    let mut idx = 0;
    let bytes = block.as_bytes();

    while idx < bytes.len() {
        if bytes.get(idx) == Some(&b'\\') {
            if let Some((env, advance)) = parse_begin_env(block, idx) {
                env_stack.push(env);
                idx += advance;
                continue;
            } else if let Some((env, advance)) = parse_end_env(block, idx) {
                if let Some(pos) = env_stack.iter().rposition(|x| x == &env) {
                    env_stack.truncate(pos);
                }
                idx += advance;
                continue;
            } else if block.get(idx..).is_some_and(|s| s.starts_with(r"\&")) {
                idx += 2;
                continue;
            }
            idx += 1;
        } else if bytes.get(idx) == Some(&b'&') {
            check_bare_alignment(block, idx, &env_stack, file_rel_path, errors);
            idx += 1;
        } else {
            idx += 1;
        }
    }
}

fn parse_begin_env(block: &str, idx: usize) -> Option<(String, usize)> {
    if block.get(idx..).is_some_and(|s| s.starts_with(r"\begin{")) {
        let after = block.get(idx + 7..).unwrap_or("");
        if let Some(close_pos) = after.find('}') {
            let env_name = after.get(..close_pos).unwrap_or("");
            if !env_name.is_empty() && env_name.chars().all(|c| c.is_ascii_alphabetic() || c == '*') {
                return Some((env_name.to_string(), 7 + close_pos + 1));
            }
        }
    }
    None
}

fn parse_end_env(block: &str, idx: usize) -> Option<(String, usize)> {
    if block.get(idx..).is_some_and(|s| s.starts_with(r"\end{")) {
        let after = block.get(idx + 5..).unwrap_or("");
        if let Some(close_pos) = after.find('}') {
            let env_name = after.get(..close_pos).unwrap_or("");
            if !env_name.is_empty() && env_name.chars().all(|c| c.is_ascii_alphabetic() || c == '*') {
                return Some((env_name.to_string(), 5 + close_pos + 1));
            }
        }
    }
    None
}

fn check_bare_alignment(
    block: &str,
    idx: usize,
    env_stack: &[String],
    file_rel_path: &str,
    errors: &mut Vec<String>,
) {
    let inside_allowed = env_stack
        .iter()
        .any(|env| ALLOWED_ALIGNMENT_ENVS.contains(&env.as_str()));
    if !inside_allowed {
        let mut start = idx.saturating_sub(20);
        while start > 0 && !block.is_char_boundary(start) {
            start -= 1;
        }
        let mut end = (idx + 1 + 20).min(block.len());
        while end < block.len() && !block.is_char_boundary(end) {
            end += 1;
        }
        let snippet = block
            .get(start..end)
            .unwrap_or("")
            .trim()
            .replace('\n', " ");
        errors.push(format!(
            "Bare alignment operator '&' outside alignment environment in {}: \"...{}...\"",
            file_rel_path, snippet
        ));
    }
}
