//! Specification metadata, title uniqueness, and filename convention validation.
//!
//! Conforms to DO-178C Level A and ISO 26262 ASIL D safety standards.
//! Validates required metadata fields, ISO 8601 dates, semantic versioning,
//! title normalization and uniqueness across spec types, and filename conventions.

pub use crate::spec_audit::metadata_validators::{
    validate_filenames, validate_metadata, validate_title_uniqueness,
};

/// Validates whether a string is a valid ISO 8601 calendar date (YYYY-MM-DD).
/// Realises: [REQ-SPEC-AUDIT-METADATA-IS-ISO-DATE]
pub fn is_iso_date(s: &str) -> bool {
    let trimmed = s.trim();
    let bytes = trimmed.as_bytes();
    if bytes.len() != 10 || bytes.get(4) != Some(&b'-') || bytes.get(7) != Some(&b'-') {
        return false;
    }

    let (Some(y_bytes), Some(m_bytes), Some(d_bytes)) =
        (bytes.get(0..4), bytes.get(5..7), bytes.get(8..10))
    else {
        return false;
    };

    if !y_bytes.iter().all(|b| b.is_ascii_digit())
        || !m_bytes.iter().all(|b| b.is_ascii_digit())
        || !d_bytes.iter().all(|b| b.is_ascii_digit())
    {
        return false;
    }

    let Some(year) = std::str::from_utf8(y_bytes).ok().and_then(|p| p.parse::<u32>().ok()) else {
        return false;
    };
    let Some(month) = std::str::from_utf8(m_bytes).ok().and_then(|p| p.parse::<u32>().ok()) else {
        return false;
    };
    let Some(day) = std::str::from_utf8(d_bytes).ok().and_then(|p| p.parse::<u32>().ok()) else {
        return false;
    };

    (1970..=2100).contains(&year) && (1..=12).contains(&month) && (1..=31).contains(&day)
}

/// Validates whether a string conforms to semantic versioning (v?X.Y[.Z][-prerelease][+build]).
/// Realises: [REQ-SPEC-AUDIT-METADATA-IS-SEMVER]
pub fn is_semver(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
    let rest = trimmed
        .strip_prefix(|c| c == 'v' || c == 'V')
        .unwrap_or(trimmed);
    if rest.is_empty() {
        return false;
    }

    let (version_core, prerelease) = match rest.find(['-', '+']) {
        Some(idx) => (rest.get(..idx).unwrap_or(rest), rest.get(idx + 1..)),
        None => (rest, None),
    };

    if let Some(pre) = prerelease {
        if pre.is_empty() || !pre.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-') {
            return false;
        }
    }

    let parts: Vec<&str> = version_core.split('.').collect();
    (2..=3).contains(&parts.len())
        && parts.iter().all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

/// Checks whether whitespace-separated words in `trimmed` contain a semantic version token.
/// Realises: [REQ-SPEC-AUDIT-METADATA-HAS-VERSION-WORD]
fn has_version_token_in_words(trimmed: &str) -> bool {
    for word in trimmed.split_whitespace() {
        let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.');
        if clean_word.starts_with('v') || clean_word.starts_with('V') {
            let rest = &clean_word[1..];
            let parts: Vec<&str> = rest.split('.').collect();
            if (parts.len() == 2 || parts.len() == 3)
                && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
            {
                return true;
            }
        } else {
            let parts: Vec<&str> = clean_word.split('.').collect();
            if parts.len() == 3 && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())) {
                return true;
            }
        }
    }
    false
}

/// Checks whether a document title contains concatenated metadata attributes.
/// Realises: [REQ-SPEC-AUDIT-METADATA-HAS-CONCATENATED-TITLE-METADATA]
pub fn has_concatenated_title_metadata(val: &str) -> bool {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_lowercase();

    let mut paren_rest = lower.as_str();
    while let Some(open) = paren_rest.find('(') {
        let after_open = &paren_rest[open + 1..];
        if let Some(close) = after_open.find(')') {
            let inside = &after_open[..close];
            if inside.contains("doc-") || inside.contains("doc_") {
                return true;
            }
            paren_rest = &after_open[close + 1..];
        } else {
            break;
        }
    }

    if let Some(idx) = lower.find("version") {
        if let Some(after_colon) = lower
            .get(idx + 7..)
            .and_then(|s| s.trim_start().strip_prefix(':'))
        {
            if after_colon
                .trim_start()
                .starts_with(|c: char| c.is_ascii_digit())
            {
                return true;
            }
        }
    }

    let bytes = trimmed.as_bytes();
    if bytes.len() >= 10 {
        for i in 0..=bytes.len() - 10 {
            if bytes.get(i + 4) == Some(&b'-') && bytes.get(i + 7) == Some(&b'-') {
                if let Some(sub) = bytes.get(i..i + 10) {
                    if let Ok(slice) = std::str::from_utf8(sub) {
                        if is_iso_date(slice) {
                            let before_ok = i == 0
                                || bytes.get(i - 1).is_none_or(|b| !b.is_ascii_alphanumeric());
                            let after_ok = i + 10 == bytes.len()
                                || bytes.get(i + 10).is_none_or(|b| !b.is_ascii_alphanumeric());
                            if before_ok && after_ok {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    has_version_token_in_words(trimmed)
}

/// Checks whether a specification title contains stuttering or redundant prefixes.
/// Realises: [REQ-SPEC-AUDIT-METADATA-CHECK-TITLE-ANTI-STUTTER]
pub fn check_title_anti_stutter(title: &str) -> Option<String> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();

    if lower.contains("uc_xx_")
        || lower.contains("uc-xx-")
        || lower.contains("ucxx_")
        || lower.contains("epic_xx_")
        || lower.contains("feat_xx_")
        || lower.contains("us_xx_")
    {
        return Some(format!(
            "Specification title contains stuttering placeholder prefix: '{}'",
            trimmed
        ));
    }

    if let Some(rest) = lower.strip_prefix("uc_") {
        let digits_len = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
        if digits_len > 0 && rest.get(digits_len..).is_some_and(|s| s.starts_with('_')) {
            return Some(format!(
                "Specification title contains raw stuttering prefix 'UC_XX_': '{}'",
                trimmed
            ));
        }
    }

    let colon_pos = lower.find(':')?;
    let prefix_part = lower.get(..colon_pos).unwrap_or("").trim();
    let after_colon = lower.get(colon_pos + 1..).unwrap_or("").trim();

    let is_spec_prefix = [
        "use case", "use_case", "use-case", "uc", "epic", "feature", "feat", "user story", "user-story", "us",
    ]
    .iter()
    .any(|p| prefix_part.starts_with(p));

    if is_spec_prefix {
        for sub in &["uc", "epic", "feat", "us"] {
            if let Some(rem) = after_colon.strip_prefix(sub) {
                let rem_clean = rem.trim_start_matches([' ', '-', '_']);
                if rem_clean.bytes().take_while(|b| b.is_ascii_digit()).count() > 0 {
                    return Some(format!(
                        "Specification title contains stuttering prefix after colon (e.g. 'Use Case XX: UC XX'): '{}'",
                        trimmed
                    ));
                }
            }
        }
        if ["use case", "epic", "feature", "user story"].iter().any(|p| after_colon.starts_with(p)) {
            let rem_after = after_colon.trim_start_matches(|c: char| !c.is_ascii_digit());
            if rem_after.bytes().take_while(|b| b.is_ascii_digit()).count() > 0 {
                return Some(format!("Specification title contains repeating title prefix: '{}'", trimmed));
            }
        }
    }

    None
}

/// Checks whether a specification filename contains stuttering or redundant prefixes.
/// Realises: [REQ-SPEC-AUDIT-METADATA-CHECK-FILENAME-ANTI-STUTTER]
pub fn check_filename_anti_stutter(filename: &str) -> Option<String> {
    let lower = filename.to_ascii_lowercase();
    let basename = match lower.rfind('/') {
        Some(idx) => lower.get(idx + 1..).unwrap_or(&lower),
        None => &lower,
    };
    let stem = basename.strip_suffix(".md").unwrap_or(basename);

    if stem.contains("uc_xx_")
        || stem.contains("uc-xx-")
        || stem.contains("ucxx_")
        || stem.contains("epic_xx_")
        || stem.contains("feat_xx_")
        || stem.contains("us_xx_")
    {
        return Some(format!(
            "Specification filename contains stuttering placeholder prefix: '{}'",
            filename
        ));
    }

    if let Some(rest) = stem.strip_prefix("uc_") {
        let digits_len = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
        if digits_len > 0 && rest.get(digits_len..).is_some_and(|s| s.starts_with('_')) {
            return Some(format!(
                "Specification filename contains raw stuttering prefix 'UC_XX_': '{}'",
                filename
            ));
        }
    }

    for p in &["uc", "epic", "feat", "us"] {
        let pattern = format!("{}-", p);
        if let Some(after_p) = stem.strip_prefix(&pattern) {
            let digit_len = after_p.bytes().take_while(|b| b.is_ascii_digit()).count();
            if digit_len > 0 {
                let after_ord = after_p.get(digit_len..).unwrap_or("");
                if after_ord.starts_with('-') || after_ord.starts_with('_') {
                    let slug = after_ord.get(1..).unwrap_or("");
                    if slug.starts_with(&format!("{}_", p)) || slug.starts_with(&format!("{}-", p))
                    {
                        return Some(format!(
                            "Specification filename contains stuttering prefix (e.g. 'UC-01-UC_01_'): '{}'",
                            filename
                        ));
                    }
                    if let Some(rem_slug) = slug.strip_prefix(p) {
                        let rem_clean = rem_slug.trim_start_matches(['_', '-']);
                        if rem_clean.bytes().take_while(|b| b.is_ascii_digit()).count() > 0 {
                            return Some(format!(
                                "Specification filename contains stuttering prefix (e.g. 'UC-01-UC01'): '{}'",
                                filename
                            ));
                        }
                    }
                }
            }
        }
    }

    None
}

/// Computes the byte length of a leading spec-type prefix (e.g. `EPIC-001: `, `Feature: `) in `lower`.
/// Realises: [REQ-SPEC-AUDIT-METADATA-MATCHED-PREFIX-LEN]
fn matched_title_prefix_len(lower: &str) -> usize {
    const PREFIX_KEYWORDS: &[&str] = &[
        "user-story", "user story", "user_story", "use-case", "use case", "use_case",
        "feature", "feat", "epic", "us", "uc",
    ];
    for kw in PREFIX_KEYWORDS {
        if let Some(mut rem) = lower.strip_prefix(kw) {
            let mut consumed = kw.len();
            if let Some(r) = rem.strip_prefix('s') {
                rem = r;
                consumed += 1;
            }
            if let Some(mut r) = rem.strip_prefix(':') {
                consumed += 1;
                while r.starts_with(' ') || r.starts_with('\t') {
                    r = r.get(1..).unwrap_or("");
                    consumed += 1;
                }
                return consumed;
            }

            let mut temp_rem = rem;
            let mut temp_consumed = consumed;
            while temp_rem.starts_with(['-', ' ', '_']) {
                temp_rem = temp_rem.get(1..).unwrap_or("");
                temp_consumed += 1;
            }
            let digit_len = temp_rem.bytes().take_while(|b| b.is_ascii_digit()).count();
            if digit_len > 0 {
                temp_rem = temp_rem.get(digit_len..).unwrap_or("");
                temp_consumed += digit_len;
                while temp_rem.starts_with([' ', ':', '-', '_']) {
                    temp_rem = temp_rem.get(1..).unwrap_or("");
                    temp_consumed += 1;
                }
                return temp_consumed;
            }
        }
    }
    0
}

/// Normalizes a specification title by stripping prefixes, removing punctuation, and collapsing whitespace.
/// Realises: [REQ-SPEC-AUDIT-METADATA-NORMALIZE-TITLE]
pub fn normalize_title(title: &str) -> String {
    let trimmed = title.trim().trim_matches(|c| c == '"' || c == '\'');
    if trimmed.is_empty() {
        return String::new();
    }

    let lower = trimmed.to_ascii_lowercase();
    let matched_len = matched_title_prefix_len(&lower);
    let stripped = if matched_len > 0 {
        trimmed.get(matched_len..).unwrap_or(trimmed)
    } else {
        trimmed
    };
    let working = if stripped.trim().is_empty() {
        trimmed
    } else {
        stripped
    };

    let filtered: String = working
        .replace(['-', '_'], " ")
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();

    filtered
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
#[path = "metadata_tests.rs"]
mod tests;
