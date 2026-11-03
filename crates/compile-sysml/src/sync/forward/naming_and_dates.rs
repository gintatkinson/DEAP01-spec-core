#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Identifier Sanitization, Acronym-Aware Display Humanization, and Prefix Stripping Helpers.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical
//! software engineering standards. Normalizes SysML v2 identifiers into filesystem slugs,
//! `snake_case` names, and acronym-preserving display titles without regular expressions.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Bounds-Checked UTF-8 Slicing: Enforces `.get(..)`, `.strip_prefix(..)`, and `.chars()`
//!   iteration with saturating byte offsets (`saturating_add`).
//! - Canonical Mixed-Case Preservation: Preserves `CommonMark`, `SysML`, `UUIDv5`, `SHA256`, and
//!   all `CANONICAL_MIXED_TOKENS` intact during `to_display_name` segmentation.
//!
//! ## 3. Preconditions
//! - Input strings are valid UTF-8 identifier or prefix slices.
//!
//! ## 4. Postconditions
//! - Returns normalized slugs, `snake_case` tokens, humanized titles, and prefix-stripped slices.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(L)$ linear in identifier byte length.
//! - Space Complexity: $O(L)$ for formatted string allocations.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Empty identifiers default to `"Default"` in `sanitize_id` and return `""` in `to_display_name`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0604] [REQ-SYSML-FORWARD-SYNC-NAMING-AND-DATES]

/// Sanitizes an identifier into an alphanumeric/underscore/hyphen slug, returning `"Default"` when empty.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-SANITIZE-ID]
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
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-TO-SNAKE-CASE]
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
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-CANONICAL-MIXED-TOKENS]
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
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-MATCH-CANONICAL-MIXED-TOKEN]
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
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-NEXT-DISPLAY-SUBTOKEN-LEN]
fn next_uppercase_subtoken_len(rem: &str) -> usize {
    let mut upper_offsets: Vec<usize> = Vec::new();
    let mut end_of_uppers = rem.len();
    let mut rel_idx = 0usize;
    for ch in rem.chars() {
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
        rel_idx = rel_idx.saturating_add(ch.len_utf8());
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
    let lower_bytes: usize = after_uppers
        .chars()
        .take_while(|c| c.is_lowercase())
        .map(char::len_utf8)
        .sum();
    let consumed = end_of_uppers.saturating_add(lower_bytes);
    let after_lowers = rem.get(consumed..).unwrap_or("");
    let digit_bytes = after_lowers.bytes().take_while(u8::is_ascii_digit).count();
    consumed.saturating_add(digit_bytes)
}

/// Computes the UTF-8 byte length of the next logical word, acronym, or canonical mixed-case token at the start of `rem`.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-NEXT-DISPLAY-SUBTOKEN-LEN]
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
        let lower_bytes: usize = rem
            .chars()
            .take_while(|c| c.is_lowercase())
            .map(char::len_utf8)
            .sum();
        let after_lowers = rem.get(lower_bytes..).unwrap_or("");
        let digit_bytes = after_lowers.bytes().take_while(u8::is_ascii_digit).count();
        return lower_bytes.saturating_add(digit_bytes);
    }
    if first_ch.is_ascii_digit() {
        let digit_bytes = rem.bytes().take_while(u8::is_ascii_digit).count();
        let after_digits = rem.get(digit_bytes..).unwrap_or("");
        let lower_bytes: usize = after_digits
            .chars()
            .take_while(|c| c.is_lowercase())
            .map(char::len_utf8)
            .sum();
        return digit_bytes.saturating_add(lower_bytes);
    }
    first_ch.len_utf8()
}

/// Converts a `PascalCase`, `camelCase`, or `snake_case` identifier into a space-separated title preserving technical acronyms and canonical mixed-case tokens.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-TO-DISPLAY-NAME]
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
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-STRIP-USE-CASE-PREFIX]
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

/// Strips an ASCII prefix case-insensitively using bounds-checked `.get(..)` UTF-8 boundary slicing.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-STRIP-PREFIX-CI]
pub(crate) fn strip_prefix_ignore_ascii_case<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let head = s.get(..prefix.len())?;
    if head.eq_ignore_ascii_case(prefix) {
        s.get(prefix.len()..)
    } else {
        None
    }
}
