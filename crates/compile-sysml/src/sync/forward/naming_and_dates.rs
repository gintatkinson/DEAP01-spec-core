//! Identifier sanitization, acronym-aware display humanization, specification date/issue resolution, and JSON/Markdown escaping helpers.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-FORWARD-SYNC-NAMING-AND-DATES]
//! Contract: Normalizes SysML v2 identifiers into filesystem slugs, snake_case names, and acronym-preserving display titles, resolves historical dates and tracker issue IDs from existing specification artifacts without wall-clock calls or synthetic fallback IDs, and escapes RFC 8259 JSON literals and CommonMark table/prose fragments.
//! Bounds & Faults: Enforces bounds-checked UTF-8 slicing (`.get`, `.strip_prefix`, `.split_once`, `.char_indices`) and saturating arithmetic across all lexical passes with zero panics, regular expressions, or direct slice indexing.

use super::*;

/// Sanitizes an identifier into an alphanumeric/underscore/hyphen slug, returning `"Default"` when empty.
/// Realises: [REQ-SYNC-FORWARD-SANITIZE-ID]
pub(crate) fn sanitize_id(id: &str) -> String {
    let clean: String = id
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    if clean.is_empty() {
        "Default".to_string()
    } else {
        clean
    }
}

/// Converts a `PascalCase` or `camelCase` identifier into deterministic `snake_case`.
/// Realises: [REQ-SYNC-FORWARD-TO-SNAKE-CASE]
pub(crate) fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 && !result.ends_with('_') {
                result.push('_');
            }
            result.extend(ch.to_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Canonical mixed-case technical tokens ordered longest-prefix-first and preserved intact during title humanization.
/// Realises: [REQ-SYNC-FORWARD-CANONICAL-MIXED-TOKENS]
pub(crate) const CANONICAL_MIXED_TOKENS: &[&str] = &[
    "CommonMark",
    "Stateflow",
    "OpenAPI",
    "GraphQL",
    "SQLite",
    "GitHub",
    "GitLab",
    "UUIDv5",
    "UUIDv4",
    "SHA256",
    "SHA512",
    "SysMLv2",
    "SysML",
    "KerML",
    "UTF16",
    "UTF8",
    "IPv6",
    "IPv4",
];

/// Matches a leading token from `CANONICAL_MIXED_TOKENS` at the start of `rem` when followed by a valid word boundary.
/// Realises: [REQ-SYNC-FORWARD-MATCH-CANONICAL-MIXED-TOKEN]
pub(crate) fn match_canonical_mixed_token(rem: &str) -> Option<usize> {
    for &tok in CANONICAL_MIXED_TOKENS {
        if let Some(after) = rem.strip_prefix(tok) {
            if after.is_empty() {
                return Some(tok.len());
            }
            if after == "s" {
                return Some(tok.len().saturating_add(1));
            }
            if after
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_uppercase() || !c.is_ascii_alphabetic())
            {
                return Some(tok.len());
            }
        }
    }
    None
}

/// Computes the UTF-8 byte length of a leading uppercase word or acronym sub-token in `rem`, stopping before embedded canonical mixed tokens.
/// Realises: [REQ-SYNC-FORWARD-NEXT-DISPLAY-SUBTOKEN-LEN]
fn next_uppercase_subtoken_len(rem: &str) -> usize {
    let mut upper_offsets: Vec<usize> = Vec::new();
    let mut end_of_uppers = rem.len();
    for (rel_idx, ch) in rem.char_indices() {
        if rel_idx > 0
            && rem
                .get(rel_idx..)
                .and_then(match_canonical_mixed_token)
                .is_some()
        {
            end_of_uppers = rel_idx;
            break;
        }
        if ch.is_uppercase() {
            upper_offsets.push(rel_idx);
        } else {
            end_of_uppers = rel_idx;
            break;
        }
    }
    let after_uppers = rem.get(end_of_uppers..).unwrap_or("");
    if upper_offsets.len() >= 2 {
        if after_uppers == "s" {
            return end_of_uppers.saturating_add(1);
        }
        if after_uppers.chars().next().is_some_and(char::is_lowercase) {
            if let Some(&last_upper_off) = upper_offsets.last() {
                if last_upper_off > 0 {
                    return last_upper_off;
                }
            }
        }
        let digit_bytes = after_uppers.bytes().take_while(u8::is_ascii_digit).count();
        return end_of_uppers.saturating_add(digit_bytes);
    }
    let mut consumed = end_of_uppers;
    for (rel_idx, ch) in after_uppers.char_indices() {
        if ch.is_lowercase() {
            consumed = end_of_uppers
                .saturating_add(rel_idx)
                .saturating_add(ch.len_utf8());
        } else {
            break;
        }
    }
    let after_lowers = rem.get(consumed..).unwrap_or("");
    let digit_bytes = after_lowers.bytes().take_while(u8::is_ascii_digit).count();
    consumed.saturating_add(digit_bytes)
}

/// Computes the UTF-8 byte length of the next logical word, acronym, or canonical mixed-case token at the start of `rem`.
/// Realises: [REQ-SYNC-FORWARD-NEXT-DISPLAY-SUBTOKEN-LEN]
pub(crate) fn next_display_subtoken_len(rem: &str) -> usize {
    if let Some(canon_len) = match_canonical_mixed_token(rem) {
        return canon_len;
    }
    let Some(first_ch) = rem.chars().next() else {
        return 0;
    };
    if first_ch == '-' {
        return first_ch.len_utf8();
    }
    if first_ch.is_uppercase() {
        return next_uppercase_subtoken_len(rem);
    }
    if first_ch.is_lowercase() {
        let mut consumed = 0usize;
        for (rel_idx, ch) in rem.char_indices() {
            if ch.is_lowercase() {
                consumed = rel_idx.saturating_add(ch.len_utf8());
            } else {
                break;
            }
        }
        let after_lowers = rem.get(consumed..).unwrap_or("");
        let digit_bytes = after_lowers.bytes().take_while(u8::is_ascii_digit).count();
        return consumed.saturating_add(digit_bytes);
    }
    if first_ch.is_ascii_digit() {
        let digit_bytes = rem.bytes().take_while(u8::is_ascii_digit).count();
        let after_digits = rem.get(digit_bytes..).unwrap_or("");
        let mut lower_bytes = 0usize;
        for (rel_idx, ch) in after_digits.char_indices() {
            if ch.is_lowercase() {
                lower_bytes = rel_idx.saturating_add(ch.len_utf8());
            } else {
                break;
            }
        }
        return digit_bytes.saturating_add(lower_bytes);
    }
    first_ch.len_utf8()
}

/// Converts a `PascalCase`, `camelCase`, or `snake_case` identifier into a space-separated title preserving technical acronyms and canonical mixed-case tokens.
/// Realises: [REQ-SYNC-FORWARD-TO-DISPLAY-NAME]
pub(crate) fn to_display_name(s: &str) -> String {
    let mut rendered_segments: Vec<String> = Vec::new();
    for seg in s
        .split(|c: char| c == '_' || c.is_ascii_whitespace())
        .filter(|part| !part.is_empty())
    {
        let mut seg_out = String::with_capacity(seg.len().saturating_add(4));
        let mut pos = 0usize;
        while pos < seg.len() {
            let Some(rem) = seg.get(pos..) else {
                break;
            };
            if rem.is_empty() {
                break;
            }
            let step = next_display_subtoken_len(rem);
            if step == 0 {
                break;
            }
            if let Some(tok) = rem.get(..step) {
                if !seg_out.is_empty() && !seg_out.ends_with('-') && !tok.starts_with('-') {
                    seg_out.push(' ');
                }
                seg_out.push_str(tok);
            }
            pos = pos.saturating_add(step);
        }
        if !seg_out.is_empty() {
            rendered_segments.push(seg_out);
        }
    }
    rendered_segments.join(" ")
}

/// Strips redundant leading prefixes (`UC_01_`, `UC-01-`, `UC01_`, `UC_AC_01_`, `US_01_`, `AC_01_`) from a Use Case or Story identifier.
/// Realises: [REQ-SYNC-FORWARD-STRIP-USE-CASE-PREFIX]
pub(crate) fn strip_use_case_prefix(name: &str) -> &str {
    let mut cur = name.trim();
    for _ in 0..4 {
        let has_prefix = cur.get(..2).is_some_and(|p| {
            p.eq_ignore_ascii_case("uc")
                || p.eq_ignore_ascii_case("us")
                || p.eq_ignore_ascii_case("ac")
        });
        if !has_prefix {
            break;
        }
        let after_prefix = cur.get(2..).unwrap_or("");
        let mut rest = after_prefix
            .strip_prefix('_')
            .or_else(|| after_prefix.strip_prefix('-'))
            .unwrap_or(after_prefix);
        if rest.get(..2).is_some_and(|p| p.eq_ignore_ascii_case("ac")) {
            let after_ac = rest.get(2..).unwrap_or("");
            if after_ac.starts_with('_')
                || after_ac.starts_with('-')
                || after_ac.chars().next().is_some_and(|c| c.is_ascii_digit())
            {
                rest = after_ac
                    .strip_prefix('_')
                    .or_else(|| after_ac.strip_prefix('-'))
                    .unwrap_or(after_ac);
            }
        }
        let digit_count = rest.chars().take_while(|c| c.is_ascii_digit()).count();
        if digit_count > 0 {
            let after_digits = rest.get(digit_count..).unwrap_or("");
            let clean = after_digits
                .strip_prefix('_')
                .or_else(|| after_digits.strip_prefix('-'))
                .unwrap_or(after_digits);
            if !clean.is_empty() {
                cur = clean;
                continue;
            }
        }
        break;
    }
    cur
}

/// Extracts an authored `date` from the leading metadata of an existing specification file, returning `fallback_iso` when absent.
/// Realises: [REQ-SYNC-FORWARD-RESOLVE-DATE]
pub(crate) fn resolve_date(target_out_dir: &Path, rel_path: &str, fallback_iso: &str) -> String {
    let full = target_out_dir.join(rel_path);
    if let Ok(content) = fs::read_to_string(&full) {
        for line in content.lines().take(25) {
            let trimmed = line.trim();
            if trimmed.starts_with("date:") {
                let d = trimmed
                    .trim_start_matches("date:")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if !d.is_empty() {
                    return d.to_string();
                }
            } else if trimmed.starts_with('|') && trimmed.ends_with('|') {
                let inner = trimmed.trim_matches('|');
                let parts: Vec<&str> = inner.split('|').map(|s| s.trim()).collect();
                if let [key_cell, value_cell, ..] = parts.as_slice() {
                    let mut key = String::new();
                    for c in key_cell.chars() {
                        if !matches!(c, '*' | '`' | '_' | ':' | '#') {
                            key.push(c);
                        }
                    }
                    let key = key.trim().to_lowercase().replace(' ', "_");
                    if key == "date" {
                        let d = value_cell
                            .trim()
                            .trim_matches('*')
                            .trim_matches('`')
                            .trim_matches('"')
                            .trim_matches('\'');
                        if !d.is_empty() {
                            return d.to_string();
                        }
                    }
                }
            }
        }
    }
    fallback_iso.to_string()
}

/// Parses a positive tracker issue ID from the leading YAML frontmatter or Markdown metadata table of `content`.
/// Realises: [REQ-SYNC-FORWARD-PARSE-ISSUE-ID-FROM-CONTENT]
pub(crate) fn parse_issue_id_from_content(content: &str) -> Option<usize> {
    for line in content.lines().take(25) {
        let trimmed = line.trim();
        if trimmed.starts_with("issue_id:") {
            let d = trimmed
                .trim_start_matches("issue_id:")
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim_start_matches('#');
            if let Ok(parsed) = d.parse::<usize>() {
                if parsed > 0 {
                    return Some(parsed);
                }
            }
        } else if trimmed.starts_with('|') && trimmed.ends_with('|') {
            let inner = trimmed.trim_matches('|');
            let parts: Vec<&str> = inner.split('|').map(|s| s.trim()).collect();
            if let [key_cell, value_cell, ..] = parts.as_slice() {
                let mut key = String::new();
                for c in key_cell.chars() {
                    if !matches!(c, '*' | '`' | '_' | ':' | '#') {
                        key.push(c);
                    }
                }
                let key = key.trim().to_lowercase().replace([' ', '-'], "_");
                if key == "issue_id" || key == "issue" || key == "id" {
                    let d = value_cell
                        .trim()
                        .trim_matches('*')
                        .trim_matches('`')
                        .trim_matches('"')
                        .trim_matches('\'')
                        .trim_start_matches('#')
                        .trim();
                    if let Ok(parsed) = d.parse::<usize>() {
                        if parsed > 0 {
                            return Some(parsed);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Resolves an existing positive tracker issue ID from `rel_path` or a same-prefix renamed `.md` sibling on disk without inventing fallback IDs.
/// Realises: [REQ-SYNC-FORWARD-RESOLVE-ISSUE-ID]
pub(crate) fn resolve_existing_issue_id(target_out_dir: &Path, rel_path: &str) -> Option<usize> {
    let full = target_out_dir.join(rel_path);
    if let Ok(content) = fs::read_to_string(&full) {
        if let Some(id) = parse_issue_id_from_content(&content) {
            return Some(id);
        }
    }
    if let Some(parent) = Path::new(rel_path).parent() {
        if let Some(file_name) = Path::new(rel_path).file_name().and_then(|f| f.to_str()) {
            if let Some(prefix) = extract_id_prefix(file_name) {
                let dir_path = target_out_dir.join(parent);
                if let Ok(entries) = fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with(prefix) && name_str.ends_with(".md") {
                            if let Ok(content) = fs::read_to_string(entry.path()) {
                                if let Some(id) = parse_issue_id_from_content(&content) {
                                    return Some(id);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Formats an optional resolved tracker issue ID as `"#<id>"` when positive or `"TBD"` when unallocated.
/// Realises: [REQ-SYNC-FORWARD-RESOLVE-ISSUE-ID]
pub(crate) fn format_issue_id_cell(issue_id: Option<usize>) -> String {
    match issue_id {
        Some(id) if id > 0 => format!("#{}", id),
        _ => "TBD".to_string(),
    }
}

/// Escapes quotes, backslashes, and `U+0000..=U+001F` control characters for RFC 8259 JSON string literal embedding.
/// Realises: [REQ-SYNC-FORWARD-ESCAPE-JSON-STRING]
pub(crate) fn escape_json_string(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 => {
                let code = u32::from(c);
                out.push_str(&format!("\\u{code:04x}"));
            }
            c => out.push(c),
        }
    }
    out
}

/// Formats an AST `AttributeDef::default_value` literal into a typed RFC 8259 JSON value token, quoting non-finite or symbolic expressions safely.
/// Realises: [REQ-SYNC-FORWARD-FORMAT-JSON-DEFAULT-LITERAL]
pub(crate) fn format_json_default_literal(raw_default: &str, json_type: &str) -> String {
    let trimmed = raw_default.trim();
    match json_type {
        "integer" if trimmed.parse::<i64>().is_ok() || trimmed.parse::<u64>().is_ok() => {
            return trimmed.to_string();
        }
        "number" if trimmed.parse::<f64>().is_ok_and(f64::is_finite) => {
            return trimmed.to_string();
        }
        "boolean" if trimmed == "true" || trimmed == "false" => {
            return trimmed.to_string();
        }
        _ => {}
    }
    let unquoted = trimmed
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(trimmed);
    format!("\"{}\"", escape_json_string(unquoted))
}

/// Neutralizes inline Markdown link targets and hardcoded GitHub URLs in AST prose or constraint fragments.
/// Realises: [REQ-SYNC-FORWARD-SANITIZE-PROSE-LINKS]
pub(crate) fn sanitize_prose_links(raw: &str) -> String {
    raw.trim()
        .replace("https://github.com/", "github-ref://")
        .replace("](", "] (")
}

/// Escapes `|` delimiters as `\|` and collapses CR/LF line breaks into single spaces for CommonMark table cells.
/// Realises: [REQ-SYNC-FORWARD-ESCAPE-MD-TABLE-CELL]
pub(crate) fn escape_markdown_table_cell(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.trim().chars() {
        match ch {
            '|' => out.push_str("\\|"),
            '\n' | '\r' => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

/// Strips an ASCII prefix case-insensitively using bounds-checked `.get(..)` UTF-8 boundary slicing.
/// Realises: [REQ-SYNC-FORWARD-STRIP-PREFIX-CI]
pub(crate) fn strip_prefix_ignore_ascii_case<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let head = s.get(..prefix.len())?;
    if head.eq_ignore_ascii_case(prefix) {
        s.get(prefix.len()..)
    } else {
        None
    }
}
