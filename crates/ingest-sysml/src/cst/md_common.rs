//! Shared Markdown AST sanitization, frontmatter, and lexical helpers for `ingest-sysml`.
//!
//! **[DOC-TIER: A]**
//! Realises: [REQ-SYSML-INGEST-MD], [REQ-TRANSLATORS-MARKDOWN-FRONTMATTER]

use crate::table::{compose_grounded_doc, MarkdownTable};

/// Extracted YAML frontmatter metadata from a Markdown requirement or blueprint document.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-FRONTMATTER]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frontmatter {
    /// Authoritative requirement identifier (e.g. `REQ-0001`).
    pub id: Option<String>,
    /// Verbatim requirement title.
    pub title: Option<String>,
    /// Subsystem classification tag.
    pub subsystem: Option<String>,
    /// Deterministic RFC 4122 UUIDv5 anchor hash.
    pub uuidv5: Option<String>,
}

/// Intermediate parsed Markdown section between structural headings.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-PARSEDSECTION]
#[derive(Debug, Clone, Default)]
pub struct ParsedSection {
    /// Component identifier if this section introduces or describes a specific component.
    pub component_target: Option<String>,
    /// Aggregated prose documentation lines within this section.
    pub section_doc: String,
    /// Structured Markdown tables parsed within this section.
    pub tables: Vec<MarkdownTable>,
}

/// Parses YAML frontmatter or top-level metadata table from a Markdown document via `cmark_ingestor`.
/// Realises: [REQ-SYSML-INGEST-MD/parse_frontmatter], [REQ-0604]
#[must_use]
pub fn parse_frontmatter(content: &str) -> (Frontmatter, &str) {
    crate::cmark_ingestor::parse_cmark_frontmatter(content)
}

/// Strips `[label](url)` Markdown links while retaining visible `label` text.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-STRIP-MARKDOWN-LINKS]
pub fn strip_markdown_links(val: &str) -> String {
    let mut out = String::with_capacity(val.len());
    let mut rest = val;
    while let Some(start_bracket) = rest.find('[') {
        out.push_str(rest.get(..start_bracket).unwrap_or(""));
        let after_start = rest.get(start_bracket + 1..).unwrap_or("");
        if let Some(end_bracket) = after_start.find(']') {
            let visible_text = after_start.get(..end_bracket).unwrap_or("");
            let after_end_bracket = after_start.get(end_bracket + 1..).unwrap_or("");
            if let Some(end_paren) = after_end_bracket
                .find(')')
                .filter(|_| after_end_bracket.starts_with('('))
            {
                out.push_str(visible_text);
                rest = after_end_bracket.get(end_paren + 1..).unwrap_or("");
                continue;
            }
        }
        out.push('[');
        rest = after_start;
    }
    out.push_str(rest);
    out
}

/// Collapses runs of whitespace characters into single ASCII spaces.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-COLLAPSE-WHITESPACE]
pub fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Strips LaTeX math formatting commands while preserving inner alphanumeric identifiers.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-STRIP-LATEX-MATH-COMMANDS]
pub fn strip_latex_math_commands(text: &str, cmds: Option<&[&str]>) -> String {
    let (mut out, mut rest) = (String::with_capacity(text.len()), text);
    while let Some(slash_idx) = rest.find('\\') {
        out.push_str(rest.get(..slash_idx).unwrap_or(""));
        let after_slash = rest.get(slash_idx + 1..).unwrap_or("");
        let cmd_len = after_slash
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .count();
        let cmd_slice = after_slash.get(..cmd_len).unwrap_or("");
        if cmd_len > 0 && cmds.is_none_or(|allowed| allowed.contains(&cmd_slice)) {
            let after_cmd = after_slash.get(cmd_len..).unwrap_or("");
            if let Some(close_brace) = after_cmd.find('}').filter(|_| after_cmd.starts_with('{')) {
                let inner = after_cmd.get(1..close_brace).unwrap_or("");
                if inner
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c.is_whitespace())
                {
                    out.push_str(inner);
                    rest = after_cmd.get(close_brace + 1..).unwrap_or("");
                    continue;
                }
            }
            rest = after_cmd;
            continue;
        }
        out.push('\\');
        rest = after_slash;
    }
    out.push_str(rest);
    out
}

/// Cleans normative requirement prose and bounds output length to 300 characters.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-CLEAN-NORMATIVE-TEXT]
pub fn clean_normative_text(raw: &str) -> String {
    let text = strip_markdown_links(raw).replace(['`', '$'], "");
    let text = strip_latex_math_commands(&text, Some(&["mathbf", "mathcal", "text", "oint"]))
        .replace('\\', "");
    let mut cleaned = collapse_whitespace(&text)
        .replace("*/", "* /")
        .replace("/*", "/ *")
        .replace('"', "\\\"");
    if cleaned.len() > 300 {
        let mut bound_300 = 300;
        while bound_300 > 0 && !cleaned.is_char_boundary(bound_300) {
            bound_300 -= 1;
        }
        let prefix = cleaned.get(..bound_300).unwrap_or(&cleaned);
        let truncate_pos = match prefix.rfind(' ') {
            Some(pos) if pos > 200 => pos,
            _ => bound_300,
        };
        let mut s = cleaned.get(..truncate_pos).unwrap_or(&cleaned).trim_end().to_string();
        if s.ends_with('\\') {
            s.pop();
        }
        cleaned = s;
    }
    cleaned
}

/// Sanitizes metadata table cell values.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-CLEAN-META-VAL]
pub fn clean_meta_val(val: &str) -> String {
    let s = val.replace(['`', '*', '$'], "");
    let s = strip_latex_math_commands(&s, Some(&["mathbf", "mathcal", "text"]));
    collapse_whitespace(&s)
        .replace("*/", "* /")
        .replace("/*", "/ *")
}

/// Sanitizes BDD scenario prose and escapes interior double quotes and comment delimiters.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-CLEAN-BDD-TEXT]
pub fn clean_bdd_text(raw: &str) -> String {
    let text = strip_markdown_links(raw).replace(['`', '$'], "");
    let text = strip_latex_math_commands(&text, None).replace('\\', "");
    collapse_whitespace(&text)
        .replace("*/", "* /")
        .replace("/*", "/ *")
        .replace('"', "\\\"")
}

/// Strips generic package noise words from an H1 heading.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-STRIP-PKG-WORDS]
pub fn strip_pkg_words(s: &str) -> String {
    const PKG_WORDS: &[&str] = &["package", "specification", "model", "spec", "system"];
    let mut out = String::with_capacity(s.len());
    let (mut i, bytes) = (0, s.as_bytes());
    while let Some(&b) = bytes.get(i) {
        if (b as char).is_ascii_alphanumeric() {
            let start = i;
            while bytes
                .get(i)
                .is_some_and(|&cb| (cb as char).is_ascii_alphanumeric() || cb == b'_')
            {
                i += 1;
            }
            let word = s.get(start..i).unwrap_or("");
            if PKG_WORDS.contains(&word.to_ascii_lowercase().as_str()) {
                out.push(' ');
            } else {
                out.push_str(word);
            }
        } else {
            out.push(b as char);
            i += 1;
        }
    }
    collapse_whitespace(&out)
}

/// Strips leading numeric section numbers and `Component:` / `Module:` prefixes, returning `(has_explicit_prefix, stripped)`.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-STRIP-COMP-PREFIX]
pub fn parse_comp_heading(header: &str) -> (bool, String) {
    const PREFIXES: &[&str] = &[
        "component",
        "part",
        "subsystem",
        "assembly",
        "block",
        "module",
    ];
    let mut candidate = header.trim();
    if let Some((_, rest)) = candidate
        .split_once(char::is_whitespace)
        .filter(|(t, _)| !t.is_empty() && t.chars().any(|c| c.is_ascii_digit()) && t.chars().all(|c| c.is_ascii_digit() || c == '.'))
    {
        candidate = rest.trim_start();
    }
    let lower = candidate.to_ascii_lowercase();
    for &prefix in PREFIXES {
        if let Some(rest) = lower.starts_with(prefix).then(|| candidate.get(prefix.len()..)).flatten() {
            if let Some(after_sep) = rest.trim_start().strip_prefix(':').or_else(|| rest.trim_start().strip_prefix('-')) {
                return (true, after_sep.trim().to_string());
            }
        }
    }
    (false, candidate.trim().to_string())
}

/// Strips leading numeric section or clause prefixes (e.g. `5.3`, `Section 4.2:`) from a heading.
/// Realises: [REQ-SYSML-INGEST-MD/strip_leading_section_number]
pub fn strip_leading_section_number(s: &str) -> &str {
    let trimmed = s.trim();
    let without_sec = trimmed
        .strip_prefix("section ")
        .or_else(|| trimmed.strip_prefix("clause "))
        .unwrap_or(trimmed)
        .trim_start();
    let num_len = without_sec
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .count();
    if let Some(rest) = without_sec
        .get(..num_len)
        .filter(|p| !p.is_empty() && p.chars().any(|c| c.is_ascii_digit()))
        .and_then(|_| without_sec.get(num_len..))
    {
        let stripped = rest.trim_start_matches([':', '-', ')', ' ']).trim_start();
        if !stripped.is_empty() {
            return stripped;
        }
    }
    trimmed
}

/// Sanitizes and wraps a raw Markdown cell string as a valid SysML v2 double-quoted string literal.
/// Realises: [REQ-TRANSLATORS-MARKDOWN-ESCAPE-SYSML-STRING-LITERAL]
pub fn escape_sysml_string_literal(raw: &str) -> String {
    let mut s = raw.trim().trim_end_matches(';').trim();
    if let Some(inner) = s.strip_prefix('`').and_then(|v| v.strip_suffix('`')) {
        s = inner.trim();
    }
    if let Some(inner) = s
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .or_else(|| s.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
    {
        s = inner;
    }
    let mut out = String::with_capacity(s.len() + 4);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' | '\r' => out.push(' '),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// Composes a grounded SysML docstring and sanitizes lexical comment delimiters (`/*` and `*/`).
/// Realises: [REQ-TRANSLATORS-MARKDOWN-COMPOSE-SANITIZED-DOC]
pub fn compose_sanitized_doc(desc: &str, citation: &str, unit: &str) -> String {
    compose_grounded_doc(desc, citation, unit)
        .replace("*/", "* /")
        .replace("/*", "/ *")
}

/// Extracts `REQ-XXXX` tokens from a text slice.
/// Realises: [REQ-SYSML-INGEST-MD/extract_requirement_cross_references]
pub fn find_req_ids(text: &str) -> Vec<&str> {
    let (mut ids, mut rest) = (Vec::new(), text);
    while let Some(idx) = rest.find("REQ-") {
        let cand = rest.get(idx..).unwrap_or("");
        if cand.len() >= 8
            && cand.get(4..8).is_some_and(|d| d.chars().all(|c| c.is_ascii_digit()))
            && (cand.len() == 8 || !cand.as_bytes().get(8).is_some_and(u8::is_ascii_digit))
        {
            if let Some(id_slice) = cand.get(..8) {
                ids.push(id_slice);
            }
        }
        rest = rest.get(idx + 4..).unwrap_or("");
    }
    ids
}
