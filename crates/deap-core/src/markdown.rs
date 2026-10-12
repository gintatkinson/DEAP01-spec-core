//! Markdown token scanner, KaTeX, and math block parser.


/// Allowed LaTeX environments where bare `&` alignment operators are permitted.
pub const ALLOWED_ALIGNMENT_ENVS: &[&str] = &[
    "aligned",
    "alignedat",
    "matrix",
    "pmatrix",
    "bmatrix",
    "Bmatrix",
    "vmatrix",
    "Vmatrix",
    "cases",
    "dcases",
    "rcases",
    "array",
    "split",
    "gathered",
    "gather",
    "subarray",
    "smallmatrix",
];

/// Known valid Mermaid diagram type headers.
pub const VALID_MERMAID_HEADERS: &[&str] = &[
    "graph",
    "flowchart",
    "sequencediagram",
    "statediagram",
    "statediagram-v2",
    "classdiagram",
    "erdiagram",
    "gantt",
    "pie",
    "gitgraph",
    "c4context",
    "journey",
    "quadrantchart",
    "xychart-beta",
    "mindmap",
    "timeline",
    "sankey-beta",
    "requirementdiagram",
];

/// Remove fenced code blocks (```...``` and ~~~...~~~), replacing with spaces to preserve line count.
pub fn strip_fenced_code_blocks(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut in_fence = false;
    let mut fence_char = ' ';
    let mut fence_len = 0;

    for line in content.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if !in_fence {
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                let ch = trimmed.chars().next().unwrap();
                let len = trimmed.chars().take_while(|&c| c == ch).count();
                in_fence = true;
                fence_char = ch;
                fence_len = len;
                for c in line.chars() {
                    if c == '\n' || c == '\r' {
                        out.push(c);
                    } else {
                        out.push(' ');
                    }
                }
                continue;
            }
            out.push_str(line);
        } else {
            let is_closing = if trimmed.starts_with(fence_char) {
                let len = trimmed.chars().take_while(|&c| c == fence_char).count();
                len >= fence_len && trimmed[len..].trim().is_empty()
            } else {
                false
            };

            for c in line.chars() {
                if c == '\n' || c == '\r' {
                    out.push(c);
                } else {
                    out.push(' ');
                }
            }

            if is_closing {
                in_fence = false;
            }
        }
    }
    out
}

/// Remove GitLab Flavored Markdown (GLFM) inline math expressions `$(`+)(.+?)\1\$`.
/// Replaces them with spaces to prevent synthesising false `$$` display delimiters.
pub fn strip_glfm_inline_math(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut result = chars.clone();
    let mut i = 0;

    while i < n {
        if chars[i] == '$' && i + 1 < n && chars[i + 1] == '`' {
            let mut k = 0;
            while i + 1 + k < n && chars[i + 1 + k] == '`' {
                k += 1;
            }

            let mut close_idx = None;
            let mut j = i + 1 + k;
            while j + k < n {
                if chars[j] == '`' {
                    let mut count = 0;
                    while j + count < n && chars[j + count] == '`' {
                        count += 1;
                    }
                    if count == k && j + count < n && chars[j + count] == '$' {
                        close_idx = Some(j + count);
                        break;
                    }
                    j += count;
                } else {
                    j += 1;
                }
            }

            if let Some(end) = close_idx {
                for item in &mut result[i..=end] {
                    if *item != '\n' && *item != '\r' {
                        *item = ' ';
                    }
                }
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }

    result.into_iter().collect()
}

/// Remove inline code spans `+.*?`+ by replacing them with spaces.
pub fn strip_inline_code(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let n = chars.len();
    let mut result = chars.clone();
    let mut i = 0;

    while i < n {
        if chars[i] == '`' {
            let mut k = 0;
            while i + k < n && chars[i + k] == '`' {
                k += 1;
            }

            let mut close_idx = None;
            let mut j = i + k;
            while j < n {
                if chars[j] == '`' {
                    let mut count = 0;
                    while j + count < n && chars[j + count] == '`' {
                        count += 1;
                    }
                    if count == k {
                        close_idx = Some(j + count - 1);
                        break;
                    }
                    j += count;
                } else {
                    j += 1;
                }
            }

            if let Some(end) = close_idx {
                for item in &mut result[i..=end] {
                    if *item != '\n' && *item != '\r' {
                        *item = ' ';
                    }
                }
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }

    result.into_iter().collect()
}

/// Strips fenced code blocks, GLFM inline math, and raw inline code spans in robust order.
pub fn strip_code_and_glfm(content: &str) -> String {
    let no_fences = strip_fenced_code_blocks(content);
    let no_glfm = strip_glfm_inline_math(&no_fences);
    strip_inline_code(&no_glfm)
}

/// Checks LaTeX / KaTeX rendering syntax across markdown content.
pub fn check_latex_katex_syntax(content: &str, file_rel_path: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let cleaned = strip_code_and_glfm(content);

    // 1. Validate balanced $$ math blocks
    let parts: Vec<&str> = cleaned.split("$$").collect();
    if !(parts.len() - 1).is_multiple_of(2) {
        errors.push(format!(
            "Unbalanced $$ display math delimiters in {} (found {} delimiters).",
            file_rel_path,
            parts.len() - 1
        ));
        return errors;
    }

    // 2. Check balanced \begin{aligned} and \end{aligned} globally in the file
    let num_begin_aligned_all = cleaned.matches(r"\begin{aligned}").count();
    let num_end_aligned_all = cleaned.matches(r"\end{aligned}").count();
    if num_begin_aligned_all != num_end_aligned_all {
        errors.push(format!(
            "Unbalanced \\begin{{aligned}} ({}) and \\end{{aligned}} ({}) pairs in {}.",
            num_begin_aligned_all, num_end_aligned_all, file_rel_path
        ));
    }

    // 3. Validate each display math block
    for i in (1..parts.len()).step_by(2) {
        let block = parts[i];

        // Detect top-level \begin{align} or \begin{align*}
        if block.contains(r"\begin{align}") || block.contains(r"\begin{align*}") {
            errors.push(format!(
                "Forbidden \\begin{{align}} or \\begin{{align*}} found in display math block in {}. In markdown KaTeX, \\begin{{aligned}} must be used instead.",
                file_rel_path
            ));
        }

        // Validate balanced \begin{aligned} and \end{aligned} within block
        let num_begin = block.matches(r"\begin{aligned}").count();
        let num_end = block.matches(r"\end{aligned}").count();
        if num_begin != num_end {
            errors.push(format!(
                "Unbalanced \\begin{{aligned}} ({}) and \\end{{aligned}} ({}) in math block in {}.",
                num_begin, num_end, file_rel_path
            ));
        }

        // Detect bare alignment operator & outside allowed alignment environments
        let mut env_stack: Vec<String> = Vec::new();
        let mut idx = 0;
        let bytes = block.as_bytes();
        while idx < bytes.len() {
            if bytes[idx] == b'\\' {
                if block[idx..].starts_with(r"\begin{") {
                    let after = &block[idx + 7..];
                    if let Some(close_pos) = after.find('}') {
                        let env_name = &after[..close_pos];
                        if !env_name.is_empty() && env_name.chars().all(|c| c.is_ascii_alphabetic() || c == '*') {
                            env_stack.push(env_name.to_string());
                            idx += 7 + close_pos + 1;
                            continue;
                        }
                    }
                } else if block[idx..].starts_with(r"\end{") {
                    let after = &block[idx + 5..];
                    if let Some(close_pos) = after.find('}') {
                        let env_name = &after[..close_pos];
                        if !env_name.is_empty() && env_name.chars().all(|c| c.is_ascii_alphabetic() || c == '*') {
                            if let Some(pos) = env_stack.iter().rposition(|x| x == env_name) {
                                env_stack.truncate(pos);
                            }
                            idx += 5 + close_pos + 1;
                            continue;
                        }
                    }
                } else if block[idx..].starts_with(r"\&") {
                    idx += 2;
                    continue;
                }
                idx += 1;
            } else if bytes[idx] == b'&' {
                let inside_allowed = env_stack
                    .iter()
                    .any(|env| ALLOWED_ALIGNMENT_ENVS.contains(&env.as_str()));
                if !inside_allowed {
                    let start = idx.saturating_sub(20);
                    let end = (idx + 1 + 20).min(block.len());
                    let snippet = block[start..end].trim().replace('\n', " ");
                    errors.push(format!(
                        "Bare alignment operator '&' outside alignment environment in {}: \"...{}...\"",
                        file_rel_path, snippet
                    ));
                }
                idx += 1;
            } else {
                idx += 1;
            }
        }
    }

    errors
}

/// Helper to validate angle bracket escaping on a single Mermaid line.
fn validate_mermaid_angle_bracket_escaping(line: &str) -> Option<char> {
    let mut clean_line = line.trim().to_string();
    // Strip stereotypes: <<[a-zA-Z0-9_\-]+>>
    let mut stripped = String::with_capacity(clean_line.len());
    let mut idx = 0;
    while idx < clean_line.len() {
        if clean_line[idx..].starts_with("<<") {
            if let Some(close_pos) = clean_line[idx + 2..].find(">>") {
                let inner = &clean_line[idx + 2..idx + 2 + close_pos];
                if !inner.is_empty() && inner.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                    idx += 2 + close_pos + 2;
                    continue;
                }
            }
        }
        let ch = clean_line[idx..].chars().next().unwrap();
        stripped.push(ch);
        idx += ch.len_utf8();
    }
    clean_line = stripped;

    // Strip longest arrows first to prevent partial replacements (e.g. "-->" matching inside "<-->")
    let arrows = [
        "==>>", "-->>", "<<==", "<<--", "<-->", "<==>", "<-.-", "-.->",
        "==>", "-->", "->>", "<==", "<--", "<<-", "<->",
        "->", "<-",
    ];
    for arrow in arrows {
        clean_line = clean_line.replace(arrow, "");
    }

    let mut in_quotes = false;
    let mut escape = false;
    for ch in clean_line.chars() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_quotes = !in_quotes;
        } else if !in_quotes && (ch == '<' || ch == '>') {
            return Some(ch);
        }
    }
    None
}

/// Validates Mermaid syntax and checks for unclosed fences, invalid headers, and unquoted brackets.
pub fn check_mermaid_syntax(content: &str, file_rel_path: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let lines: Vec<&str> = content.split_inclusive('\n').collect();
    let n = lines.len();
    let mut i = 0;

    while i < n {
        let line = lines[i];
        let trimmed = line.trim();
        let is_mermaid_open = if trimmed.starts_with("```") {
            let backticks = trimmed.chars().take_while(|&c| c == '`').count();
            backticks >= 3 && trimmed[backticks..].trim() == "mermaid"
        } else {
            false
        };

        if is_mermaid_open {
            let start_line = i + 1;
            let mut body: Vec<(usize, &str)> = Vec::new();
            i += 1;
            let mut closed = false;

            while i < n {
                let inner = lines[i];
                let inner_trimmed = inner.trim();
                if inner_trimmed.starts_with("```") {
                    closed = true;
                    break;
                }
                body.push((i + 1, inner));
                i += 1;
            }

            if !closed {
                errors.push(format!(
                    "{}:{}: unclosed ```mermaid fence. Every diagram must be closed with a matching ``` fence on its own line.",
                    file_rel_path, start_line
                ));
            } else {
                // Find first non-empty, non-comment line to check header
                let mut header_opt = None;
                for &(lineno, body_line) in &body {
                    let trimmed = body_line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with("%%") {
                        header_opt = Some((lineno, trimmed));
                        break;
                    }
                }

                let mut kind = String::new();
                if let Some((header_line, header_text)) = header_opt {
                    let lower = header_text.to_lowercase();
                    if let Some(matched_kind) = VALID_MERMAID_HEADERS
                        .iter()
                        .find(|&&h| lower.starts_with(h))
                    {
                        kind = (*matched_kind).to_string();
                    } else {
                        errors.push(format!(
                            "{}:{}: missing or invalid Mermaid diagram header (found '{}'). The first non-comment line must declare a valid diagram type header.",
                            file_rel_path, header_line, header_text
                        ));
                    }
                } else {
                    errors.push(format!(
                        "{}:{}: missing or invalid Mermaid diagram header. The first non-comment line must declare a valid diagram type header.",
                        file_rel_path, start_line
                    ));
                }

                // Check forbidden semicolons in Mermaid Note statements and message text
                for &(lineno, body_line) in &body {
                    let trimmed = body_line.trim();
                    if trimmed.is_empty() || trimmed.starts_with("%%") {
                        continue;
                    }
                    let is_note = trimmed.starts_with("Note ") || trimmed.starts_with("note ");
                    let is_msg = trimmed.contains(">>")
                        || trimmed.contains("->>")
                        || trimmed.contains("-->>")
                        || (kind == "sequencediagram"
                            && trimmed.contains(':')
                            && !trimmed.starts_with("participant ")
                            && !trimmed.starts_with("actor "));
                    if (is_note || is_msg) && trimmed.contains(';') {
                        errors.push(format!(
                            "{}:{}: semicolon in Mermaid Note or message text: '{}'. Replace with commas, dashes, or spaces.",
                            file_rel_path, lineno, trimmed
                        ));
                    }
                }

                // Check angle bracket escaping in diagram kinds
                if ["graph", "flowchart", "sequencediagram", "statediagram", "statediagram-v2"]
                    .contains(&kind.as_str())
                {
                    for &(lineno, body_line) in &body {
                        let trimmed = body_line.trim();
                        if trimmed.is_empty() || trimmed.starts_with("%%") {
                            continue;
                        }
                        if let Some(unquoted_char) =
                            validate_mermaid_angle_bracket_escaping(trimmed)
                        {
                            errors.push(format!(
                                "{}:{}: unquoted '{}' character in Mermaid diagram line: '{}'. Transitions, labels, or guards containing comparison operators or brackets MUST be enclosed in double quotes.",
                                file_rel_path, lineno, unquoted_char, trimmed
                            ));
                        }
                    }
                }
            }
        }
        i += 1;
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glfm_inline_math_does_not_synthesize_display_math() {
        let sample = "The constraint `$``v_{lim} = 10``$` applies along with `$``a_{lim} = 2``$`.";
        let cleaned = strip_code_and_glfm(sample);
        assert!(
            !cleaned.contains("$$"),
            "GLFM math stripping must not synthesize false '$$' display delimiters! Cleaned: {:?}",
            cleaned
        );
        let errors = check_latex_katex_syntax(sample, "test.md");
        assert!(
            errors.is_empty(),
            "Expected 0 errors on valid GLFM math, got: {:?}",
            errors
        );
    }

    #[test]
    fn test_bare_alignment_inside_aligned_passes() {
        let sample = r#"
$$
\begin{aligned}
x &= 1 \\
y &= 2
\end{aligned}
$$
"#;
        let errors = check_latex_katex_syntax(sample, "test.md");
        assert!(
            errors.is_empty(),
            "Expected 0 errors for alignment operator '&' inside \\begin{{aligned}}, got: {:?}",
            errors
        );
    }

    #[test]
    fn test_bare_alignment_outside_aligned_fails() {
        let sample = r#"
$$
x & 1 \\
y & 2
$$
"#;
        let errors = check_latex_katex_syntax(sample, "test.md");
        assert!(
            !errors.is_empty(),
            "Expected error for bare '&' outside alignment environment"
        );
        assert!(
            errors[0].contains("Bare alignment operator '&' outside alignment environment"),
            "Error was: {}",
            errors[0]
        );
    }

    #[test]
    fn test_schema_req_0099_zero_errors() {
        let req_content = include_str!("../../../schema/REQ-0099.md");
        let katex_errors = check_latex_katex_syntax(req_content, "schema/REQ-0099.md");
        assert!(
            katex_errors.is_empty(),
            "schema/REQ-0099.md must have 0 KaTeX errors, got: {:?}",
            katex_errors
        );

        let mermaid_errors = check_mermaid_syntax(req_content, "schema/REQ-0099.md");
        assert!(
            mermaid_errors.is_empty(),
            "schema/REQ-0099.md must have 0 Mermaid errors, got: {:?}",
            mermaid_errors
        );
    }

    #[test]
    fn test_mermaid_unclosed_fence_detection() {
        let sample = "```mermaid\ngraph TD\nA --> B\n";
        let errors = check_mermaid_syntax(sample, "test.md");
        assert!(!errors.is_empty(), "Should detect unclosed mermaid fence");
        assert!(errors[0].contains("unclosed ```mermaid fence"));
    }

    #[test]
    fn test_mermaid_invalid_header_detection() {
        let sample = "```mermaid\ninvalidHeader TD\nA --> B\n```";
        let errors = check_mermaid_syntax(sample, "test.md");
        assert!(!errors.is_empty(), "Should detect invalid diagram header");
        assert!(errors[0].contains("missing or invalid Mermaid diagram header"));
    }

    #[test]
    fn test_mermaid_unquoted_angle_bracket_detection() {
        let sample = "```mermaid\ngraph TD\nA -->|x < 5| B\n```";
        let errors = check_mermaid_syntax(sample, "test.md");
        assert!(!errors.is_empty(), "Should detect unquoted '<' character");
        assert!(errors[0].contains("unquoted '<' character"));

        let quoted_sample = "```mermaid\ngraph TD\nA -->|\"x < 5\"| B\n```";
        let clean_errors = check_mermaid_syntax(quoted_sample, "test.md");
        assert!(
            clean_errors.is_empty(),
            "Quoted angle bracket should pass, got: {:?}",
            clean_errors
        );
    }
}
