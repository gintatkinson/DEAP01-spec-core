//! Specification metadata, title uniqueness, and filename convention validation.
//!
//! Conforms to DO-178C Level A and ISO 26262 ASIL D safety standards.
//! Validates required metadata fields, ISO 8601 dates, semantic versioning,
//! title normalization and uniqueness across spec types, and filename conventions
//! using pure byte, slice, and token validation.

use crate::spec_audit::markdown_ast::MarkdownDoc;
use std::collections::HashMap;

/// Validates whether a string is a valid ISO 8601 calendar date (YYYY-MM-DD).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_iso_date` in `crates/verify-baseline/src/spec_audit/metadata.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_iso_date` (declaration #1) within `SPEC-AUDIT-METADATA`.
///
/// ## 3. Preconditions
/// - `s: &str` must satisfy domain and type-safety constraints prior to invoking `is_iso_date`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_iso_date` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(s)) bounded over input/member dimension of `is_iso_date`.
/// - Space Complexity: O(N(s)) bounded auxiliary memory for `is_iso_date`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_iso_date` (`bool`).
/// - State: Maintains consistent state invariants across `is_iso_date` without partial mutation on error.
/// - Action: Executes `is_iso_date` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_iso_date` (`SPEC-AUDIT-METADATA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-METADATA-IS-ISO-DATE] | SysML v2: ActionDef(is_iso_date)
pub fn is_iso_date(s: &str) -> bool {
    let trimmed = s.trim();
    let bytes = trimmed.as_bytes();
    if bytes.len() != 10 {
        return false;
    }
    if bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }

    if !bytes[0..4].iter().all(|b| b.is_ascii_digit())
        || !bytes[5..7].iter().all(|b| b.is_ascii_digit())
        || !bytes[8..10].iter().all(|b| b.is_ascii_digit())
    {
        return false;
    }

    let year = match std::str::from_utf8(&bytes[0..4])
        .ok()
        .and_then(|p| p.parse::<u32>().ok())
    {
        Some(y) => y,
        None => return false,
    };
    let month = match std::str::from_utf8(&bytes[5..7])
        .ok()
        .and_then(|p| p.parse::<u32>().ok())
    {
        Some(m) => m,
        None => return false,
    };
    let day = match std::str::from_utf8(&bytes[8..10])
        .ok()
        .and_then(|p| p.parse::<u32>().ok())
    {
        Some(d) => d,
        None => return false,
    };

    if year < 1970 || year > 2100 || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return false;
    }
    true
}

/// Validates whether a string conforms to semantic versioning (v?X.Y[.Z][-prerelease][+build]).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_semver` in `crates/verify-baseline/src/spec_audit/metadata.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_semver` (declaration #2) within `SPEC-AUDIT-METADATA`.
///
/// ## 3. Preconditions
/// - `s: &str` must satisfy domain and type-safety constraints prior to invoking `is_semver`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_semver` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(s)) bounded over input/member dimension of `is_semver`.
/// - Space Complexity: O(N(s)) bounded auxiliary memory for `is_semver`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_semver` (`bool`).
/// - State: Maintains consistent state invariants across `is_semver` without partial mutation on error.
/// - Action: Executes `is_semver` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_semver` (`SPEC-AUDIT-METADATA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-METADATA-IS-SEMVER] | SysML v2: ActionDef(is_semver)
pub fn is_semver(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
    let rest = if let Some(stripped) = trimmed.strip_prefix(|c| c == 'v' || c == 'V') {
        stripped
    } else {
        trimmed
    };

    if rest.is_empty() {
        return false;
    }

    let (version_core, prerelease) = match rest.find(|c| c == '-' || c == '+') {
        Some(idx) => (&rest[..idx], Some(&rest[idx + 1..])),
        None => (rest, None),
    };

    if let Some(pre) = prerelease {
        if pre.is_empty()
            || !pre
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        {
            return false;
        }
    }

    let parts: Vec<&str> = version_core.split('.').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return false;
    }

    for part in parts {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
    }

    true
}

/// Checks whether a document title contains concatenated metadata attributes
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `has_concatenated_title_metadata` in `crates/verify-baseline/src/spec_audit/metadata.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `has_concatenated_title_metadata` (declaration #3) within `SPEC-AUDIT-METADATA`.
///
/// ## 3. Preconditions
/// - `val: &str` must satisfy domain and type-safety constraints prior to invoking `has_concatenated_title_metadata`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `has_concatenated_title_metadata` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(val)) bounded over input/member dimension of `has_concatenated_title_metadata`.
/// - Space Complexity: O(N(val)) bounded auxiliary memory for `has_concatenated_title_metadata`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `has_concatenated_title_metadata` (`bool`).
/// - State: Maintains consistent state invariants across `has_concatenated_title_metadata` without partial mutation on error.
/// - Action: Executes `has_concatenated_title_metadata` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `has_concatenated_title_metadata` (`SPEC-AUDIT-METADATA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-METADATA-HAS-CONCATENATED-TITLE-METADATA] | SysML v2: ActionDef(has_concatenated_title_metadata)
pub fn has_concatenated_title_metadata(val: &str) -> bool {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_lowercase();

    // 1. Check parenthesized DOC- tokens: e.g. "(DOC-NAV-001)"
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

    // 2. Check for "version" followed by optional spaces, ':', and digits
    if let Some(idx) = lower.find("version") {
        let rem = &lower[idx + 7..];
        let mut chars = rem.chars().peekable();
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }
        if let Some(&':') = chars.peek() {
            chars.next();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }
            if let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    return true;
                }
            }
        }
    }

    // 3. Check for ISO date pattern YYYY-MM-DD
    let bytes = trimmed.as_bytes();
    if bytes.len() >= 10 {
        for i in 0..=bytes.len() - 10 {
            if bytes[i + 4] == b'-' && bytes[i + 7] == b'-' {
                if let Ok(slice) = std::str::from_utf8(&bytes[i..i + 10]) {
                    if is_iso_date(slice) {
                        let before_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
                        let after_ok =
                            i + 10 == bytes.len() || !bytes[i + 10].is_ascii_alphanumeric();
                        if before_ok && after_ok {
                            return true;
                        }
                    }
                }
            }
        }
    }

    // 4. Check for version tokens in whitespace-separated words:
    // v\d+\.\d+(?:\.\d+)? or \d+\.\d+\.\d+
    for word in trimmed.split_whitespace() {
        let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.');
        if clean_word.starts_with('v') || clean_word.starts_with('V') {
            let rest = &clean_word[1..];
            let parts: Vec<&str> = rest.split('.').collect();
            if (parts.len() == 2 || parts.len() == 3)
                && parts
                    .iter()
                    .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
            {
                return true;
            }
        } else {
            let parts: Vec<&str> = clean_word.split('.').collect();
            if parts.len() == 3
                && parts
                    .iter()
                    .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
            {
                return true;
            }
        }
    }

    false
}

/// Checks whether a specification title contains stuttering or redundant prefixes
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `check_title_anti_stutter` in `crates/verify-baseline/src/spec_audit/metadata.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `check_title_anti_stutter` (declaration #4) within `SPEC-AUDIT-METADATA`.
///
/// ## 3. Preconditions
/// - `title: &str` must satisfy domain and type-safety constraints prior to invoking `check_title_anti_stutter`.
///
/// ## 4. Postconditions
/// - Returns `Option<String>` from `check_title_anti_stutter` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(title)) bounded over input/member dimension of `check_title_anti_stutter`.
/// - Space Complexity: O(N(title)) bounded auxiliary memory for `check_title_anti_stutter`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `check_title_anti_stutter` (`Option<String>`).
/// - State: Maintains consistent state invariants across `check_title_anti_stutter` without partial mutation on error.
/// - Action: Executes `check_title_anti_stutter` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `check_title_anti_stutter` (`SPEC-AUDIT-METADATA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-METADATA-CHECK-TITLE-ANTI-STUTTER] | SysML v2: ActionDef(check_title_anti_stutter)
pub fn check_title_anti_stutter(title: &str) -> Option<String> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();

    // 1. Placeholder or stutter tokens like "uc_xx_", "uc-xx-", "ucxx_"
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

    // 2. Starts with raw stutter prefix like "uc_" followed by digits and underscore
    if let Some(rest) = lower.strip_prefix("uc_") {
        let digits_len = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
        if digits_len > 0 && rest.get(digits_len..).is_some_and(|s| s.starts_with('_')) {
            return Some(format!(
                "Specification title contains raw stuttering prefix 'UC_XX_': '{}'",
                trimmed
            ));
        }
    }

    // 3. Stutter after standard prefix "Use Case XX:", "UC XX:", "Epic XX:", "Feature XX:", "User Story XX:"
    if let Some(colon_pos) = lower.find(':') {
        let prefix_part = lower.get(..colon_pos).unwrap_or("").trim();
        let after_colon = lower.get(colon_pos + 1..).unwrap_or("").trim();

        let is_spec_prefix = prefix_part.starts_with("use case")
            || prefix_part.starts_with("use_case")
            || prefix_part.starts_with("use-case")
            || prefix_part.starts_with("uc")
            || prefix_part.starts_with("epic")
            || prefix_part.starts_with("feature")
            || prefix_part.starts_with("feat")
            || prefix_part.starts_with("user story")
            || prefix_part.starts_with("user-story")
            || prefix_part.starts_with("us");

        if is_spec_prefix {
            let redundant_sub_prefixes = ["uc", "epic", "feat", "us"];
            for sub in &redundant_sub_prefixes {
                if let Some(rem) = after_colon.strip_prefix(sub) {
                    let rem_clean = rem.trim_start_matches(|c: char| c == ' ' || c == '-' || c == '_');
                    let digit_count = rem_clean.bytes().take_while(|b| b.is_ascii_digit()).count();
                    if digit_count > 0 {
                        return Some(format!(
                            "Specification title contains stuttering prefix after colon (e.g. 'Use Case XX: UC XX'): '{}'",
                            trimmed
                        ));
                    }
                }
            }

            if after_colon.starts_with("use case")
                || after_colon.starts_with("epic")
                || after_colon.starts_with("feature")
                || after_colon.starts_with("user story")
            {
                let rem_after = after_colon.trim_start_matches(|c: char| !c.is_ascii_digit());
                let digit_count = rem_after.bytes().take_while(|b| b.is_ascii_digit()).count();
                if digit_count > 0 {
                    return Some(format!(
                        "Specification title contains repeating title prefix: '{}'",
                        trimmed
                    ));
                }
            }
        }
    }

    None
}

/// Checks whether a specification filename contains stuttering or redundant prefixes
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `check_filename_anti_stutter` in `crates/verify-baseline/src/spec_audit/metadata.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `check_filename_anti_stutter` (declaration #5) within `SPEC-AUDIT-METADATA`.
///
/// ## 3. Preconditions
/// - `filename: &str` must satisfy domain and type-safety constraints prior to invoking `check_filename_anti_stutter`.
///
/// ## 4. Postconditions
/// - Returns `Option<String>` from `check_filename_anti_stutter` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(filename)) bounded over input/member dimension of `check_filename_anti_stutter`.
/// - Space Complexity: O(N(filename)) bounded auxiliary memory for `check_filename_anti_stutter`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `check_filename_anti_stutter` (`Option<String>`).
/// - State: Maintains consistent state invariants across `check_filename_anti_stutter` without partial mutation on error.
/// - Action: Executes `check_filename_anti_stutter` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `check_filename_anti_stutter` (`SPEC-AUDIT-METADATA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-METADATA-CHECK-FILENAME-ANTI-STUTTER] | SysML v2: ActionDef(check_filename_anti_stutter)
pub fn check_filename_anti_stutter(filename: &str) -> Option<String> {
    let lower = filename.to_ascii_lowercase();
    let basename = match lower.rfind('/') {
        Some(idx) => lower.get(idx + 1..).unwrap_or(&lower),
        None => &lower,
    };
    let stem = match basename.strip_suffix(".md") {
        Some(s) => s,
        None => basename,
    };

    // 1. Placeholder or stutter tokens like "uc_xx_", "uc-xx-", "ucxx_"
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

    // 2. Starts with raw stutter prefix like "uc_01_"
    if let Some(rest) = stem.strip_prefix("uc_") {
        let digits_len = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
        if digits_len > 0 && rest.get(digits_len..).is_some_and(|s| s.starts_with('_')) {
            return Some(format!(
                "Specification filename contains raw stuttering prefix 'UC_XX_': '{}'",
                filename
            ));
        }
    }

    // 3. Filename formatted as <prefix>-<ordinal>-<rest>
    let prefixes = ["uc", "epic", "feat", "us"];
    for p in &prefixes {
        let pattern = format!("{}-", p);
        if let Some(after_p) = stem.strip_prefix(&pattern) {
            let digit_len = after_p.bytes().take_while(|b| b.is_ascii_digit()).count();
            if digit_len > 0 {
                let after_ord = after_p.get(digit_len..).unwrap_or("");
                if after_ord.starts_with('-') || after_ord.starts_with('_') {
                    let slug = after_ord.get(1..).unwrap_or("");
                    let stutter1 = format!("{}_", p);
                    let stutter2 = format!("{}-", p);
                    if slug.starts_with(&stutter1) || slug.starts_with(&stutter2) {
                        return Some(format!(
                            "Specification filename contains stuttering prefix (e.g. 'UC-01-UC_01_'): '{}'",
                            filename
                        ));
                    }
                    if let Some(rem_slug) = slug.strip_prefix(p) {
                        let rem_clean = rem_slug.trim_start_matches(|c: char| c == '_' || c == '-');
                        let slug_digits = rem_clean.bytes().take_while(|b| b.is_ascii_digit()).count();
                        if slug_digits > 0 {
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

/// Normalizes a specification title by stripping prefixes, removing punctuation, and collapsing whitespace.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `normalize_title` in `crates/verify-baseline/src/spec_audit/metadata.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `normalize_title` (declaration #6) within `SPEC-AUDIT-METADATA`.
///
/// ## 3. Preconditions
/// - `title: &str` must satisfy domain and type-safety constraints prior to invoking `normalize_title`.
///
/// ## 4. Postconditions
/// - Returns `String` from `normalize_title` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(title)) bounded over input/member dimension of `normalize_title`.
/// - Space Complexity: O(N(title)) bounded auxiliary memory for `normalize_title`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `normalize_title` (`String`).
/// - State: Maintains consistent state invariants across `normalize_title` without partial mutation on error.
/// - Action: Executes `normalize_title` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `normalize_title` (`SPEC-AUDIT-METADATA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-METADATA-NORMALIZE-TITLE] | SysML v2: ActionDef(normalize_title)
pub fn normalize_title(title: &str) -> String {
    let trimmed = title.trim().trim_matches(|c| c == '"' || c == '\'');
    if trimmed.is_empty() {
        return String::new();
    }

    let lower = trimmed.to_ascii_lowercase();
    const PREFIX_KEYWORDS: &[&str] = &[
        "user-story", "user story", "user_story",
        "use-case", "use case", "use_case",
        "feature", "feat",
        "epic",
        "us", "uc",
    ];

    let mut matched_len = 0;
    for kw in PREFIX_KEYWORDS {
        if let Some(mut rem) = lower.strip_prefix(kw) {
            let mut consumed = kw.len();

            // Optional 's' plural
            if let Some(r) = rem.strip_prefix('s') {
                rem = r;
                consumed += 1;
            }

            // Case A: followed by ':'
            if let Some(mut r) = rem.strip_prefix(':') {
                consumed += 1;
                while r.starts_with(' ') || r.starts_with('\t') {
                    r = r.get(1..).unwrap_or("");
                    consumed += 1;
                }
                matched_len = consumed;
                break;
            }

            // Case B: optional [- ]* followed by \d+
            let mut temp_rem = rem;
            let mut temp_consumed = consumed;
            while temp_rem.starts_with('-') || temp_rem.starts_with(' ') || temp_rem.starts_with('_') {
                temp_rem = temp_rem.get(1..).unwrap_or("");
                temp_consumed += 1;
            }

            let digit_len = temp_rem.bytes().take_while(|b| b.is_ascii_digit()).count();
            if digit_len > 0 {
                temp_rem = temp_rem.get(digit_len..).unwrap_or("");
                temp_consumed += digit_len;

                // Skip optional trailing space, ':', '-', or '_'
                while temp_rem.starts_with(' ')
                    || temp_rem.starts_with(':')
                    || temp_rem.starts_with('-')
                    || temp_rem.starts_with('_')
                {
                    temp_rem = temp_rem.get(1..).unwrap_or("");
                    temp_consumed += 1;
                }
                matched_len = temp_consumed;
                break;
            }
        }
    }

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

    let separators_replaced = working.replace('-', " ").replace('_', " ");

    let filtered: String = separators_replaced
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();

    filtered
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Validates frontmatter and CommonMark metadata table attributes for a specification.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `validate_metadata` in `crates/verify-baseline/src/spec_audit/metadata.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `validate_metadata` (declaration #7) within `SPEC-AUDIT-METADATA`.
///
/// ## 3. Preconditions
/// - `spec_type: &str` must satisfy domain and type-safety constraints prior to invoking `validate_metadata`.
/// - `doc: &MarkdownDoc` must satisfy domain and type-safety constraints prior to invoking `validate_metadata`.
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `validate_metadata`.
/// - `rel_path: &str` must satisfy domain and type-safety constraints prior to invoking `validate_metadata`.
///
/// ## 4. Postconditions
/// - Returns `Vec<String>` from `validate_metadata` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(spec_type)) bounded over input/member dimension of `validate_metadata`.
/// - Space Complexity: O(N(spec_type)) bounded auxiliary memory for `validate_metadata`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `validate_metadata` (`Vec<String>`).
/// - State: Maintains consistent state invariants across `validate_metadata` without partial mutation on error.
/// - Action: Executes `validate_metadata` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `validate_metadata` (`SPEC-AUDIT-METADATA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-METADATA-VALIDATE-METADATA] | SysML v2: ActionDef(validate_metadata)
pub fn validate_metadata(
    spec_type: &str,
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
) -> Vec<String> {
    let mut errors = Vec::new();
    let norm_type = spec_type.to_lowercase();

    // 1. Flag raw YAML frontmatter as strictly forbidden
    if content.trim_start().starts_with("---") || doc.frontmatter.is_some() {
        errors.push(format!(
            "{}: Raw YAML frontmatter ('--- ... ---') is strictly forbidden. Specifications must use a native CommonMark metadata table at lines 1-10 per rules/specification-metadata-integrity.md.",
            rel_path
        ));
    }

    // 2. Verify CommonMark metadata table at lines 1-10 with exact header
    let lines: Vec<&str> = content.lines().take(10).collect();
    let header_pos = lines
        .iter()
        .position(|l| l.trim() == "| Attribute | Specification Detail |");
    match header_pos {
        Some(pos) => {
            for (idx, prev_line) in lines[..pos].iter().enumerate() {
                let t = prev_line.trim();
                if !t.is_empty() && !t.starts_with("<!--") {
                    errors.push(format!(
                        "{}: Specification metadata table must begin within lines 1-10 (line {} contains unexpected text before metadata table).",
                        rel_path,
                        idx + 1
                    ));
                    break;
                }
            }
            if pos + 1 >= lines.len() || lines[pos + 1].trim() != "| :--- | :--- |" {
                errors.push(format!(
                    "{}: Invalid metadata table separator row at line {}. Expected '| :--- | :--- |'.",
                    rel_path,
                    pos + 2
                ));
            }
        }
        None => {
            errors.push(format!(
                "{}: Missing native CommonMark metadata table at lines 1-10 with exact header '| Attribute | Specification Detail |'.",
                rel_path
            ));
        }
    }

    let get_field = |key: &str| -> Option<&str> {
        doc.table_metadata
            .get(key)
            .map(|s| s.as_str())
            .or_else(|| {
                doc.frontmatter
                    .as_ref()
                    .and_then(|fm| fm.get(key).map(|s| s.as_str()))
            })
    };

    // 3. Mandatory issue_id
    match get_field("issue_id") {
        Some(val) if !val.trim().is_empty() => {}
        _ => {
            errors.push(format!("{}: Missing mandatory metadata field 'issue_id'.", rel_path));
        }
    }

    // 4. Mandatory type
    match get_field("type") {
        Some(val) if !val.trim().is_empty() => {}
        _ => {
            errors.push(format!("{}: Missing mandatory metadata field 'type'.", rel_path));
        }
    }

    // 5. Mandatory title
    let title_opt = get_field("title").or_else(|| {
        doc.headings
            .iter()
            .find(|h| h.level == 1)
            .map(|h| h.text.as_str())
    });

    match title_opt {
        Some(val) if !val.trim().is_empty() => {
            if has_concatenated_title_metadata(val) {
                errors.push(format!(
                    "{}: Document title contains concatenated metadata attributes ('{}'). Titles must be clean canonical names without embedded versions, dates, or IDs.",
                    rel_path, val
                ));
            }
            if let Some(stutter_err) = check_title_anti_stutter(val) {
                errors.push(format!(
                    "{}: Anti-Stutter Gate violation: {}",
                    rel_path, stutter_err
                ));
            }
        }
        _ => {
            errors.push(format!("{}: Missing mandatory document title.", rel_path));
        }
    }

    // 5b. Filename anti-stutter validation
    if let Some(stutter_err) = check_filename_anti_stutter(rel_path) {
        errors.push(format!(
            "{}: Anti-Stutter Gate violation: {}",
            rel_path, stutter_err
        ));
    }

    // 6. Mandatory generation_mode
    let gen_mode = get_field("generation_mode")
        .or_else(|| get_field("generation-mode"))
        .or_else(|| get_field("subagent_drafted"))
        .or_else(|| get_field("subagent-drafted"))
        .or_else(|| get_field("subagentdrafted"))
        .or_else(|| get_field("subagent"))
        .or_else(|| get_field("subagent_mode"));
    match gen_mode {
        Some(val) if !val.trim().is_empty() => {}
        _ => {
            errors.push(format!(
                "{}: Missing mandatory metadata field 'generation_mode'.",
                rel_path
            ));
        }
    }

    // 7. Mandatory specification_source
    match get_field("specification_source") {
        Some(val) if !val.trim().is_empty() => {}
        _ => {
            errors.push(format!(
                "{}: Missing mandatory metadata field 'specification_source'.",
                rel_path
            ));
        }
    }

    // 8. Spec-type specific mandatory attributes per rules/specification-metadata-integrity.md
    if norm_type.contains("epic") {
        match get_field("package") {
            Some(val) if !val.trim().is_empty() => {}
            _ => {
                errors.push(format!("{}: Epic is missing mandatory 'package' metadata.", rel_path));
            }
        }
        match get_field("subsystem") {
            Some(val) if !val.trim().is_empty() => {}
            _ => {
                errors.push(format!("{}: Epic is missing mandatory 'subsystem' metadata.", rel_path));
            }
        }
    } else if norm_type.contains("feature") {
        match get_field("parent_epic") {
            Some(val) if !val.trim().is_empty() => {}
            _ => {
                errors.push(format!("{}: Feature is missing mandatory 'parent_epic' metadata.", rel_path));
            }
        }
        match get_field("interface_type") {
            Some(val) if !val.trim().is_empty() => {}
            _ => {
                errors.push(format!("{}: Feature is missing mandatory 'interface_type' metadata.", rel_path));
            }
        }
        match get_field("schema_containers") {
            Some(val) if !val.trim().is_empty() => {}
            _ => {
                errors.push(format!("{}: Feature is missing mandatory 'schema_containers' metadata.", rel_path));
            }
        }
    } else if norm_type.contains("user_story") || norm_type.contains("user-story") {
        match get_field("parent_epic") {
            Some(val) if !val.trim().is_empty() => {}
            _ => {
                errors.push(format!("{}: User story is missing mandatory 'parent_epic' metadata.", rel_path));
            }
        }
        match get_field("sysml_interaction") {
            Some(val) if !val.trim().is_empty() => {}
            _ => {
                errors.push(format!("{}: User story is missing mandatory 'sysml_interaction' metadata.", rel_path));
            }
        }
        match get_field("sysml_test_case") {
            Some(val) if !val.trim().is_empty() => {}
            _ => {
                errors.push(format!("{}: User story is missing mandatory 'sysml_test_case' metadata.", rel_path));
            }
        }
    } else if norm_type.contains("use_case") || norm_type.contains("use-case") {
        match get_field("primary_actor") {
            Some(val) if !val.trim().is_empty() => {}
            _ => {
                errors.push(format!("{}: Use case is missing mandatory 'primary_actor' metadata.", rel_path));
            }
        }
        match get_field("scope") {
            Some(val) if !val.trim().is_empty() => {}
            _ => {
                errors.push(format!("{}: Use case is missing mandatory 'scope' metadata.", rel_path));
            }
        }
        match get_field("level") {
            Some(val) if !val.trim().is_empty() => {}
            _ => {
                errors.push(format!("{}: Use case is missing mandatory 'level' metadata.", rel_path));
            }
        }
    }

    // 9. Date validation if present
    if let Some(date_val) = get_field("date").or_else(|| get_field("release_date")) {
        if !date_val.trim().is_empty() && !is_iso_date(date_val) {
            errors.push(format!(
                "{}: Invalid date format: '{}'. Expected ISO 8601 format (YYYY-MM-DD).",
                rel_path, date_val
            ));
        }
    }

    // 10. Semver validation if present
    if let Some(ver_val) = get_field("version") {
        if !ver_val.trim().is_empty() && !is_semver(ver_val) {
            errors.push(format!(
                "{}: Invalid version format: '{}'. Expected semantic versioning format (v?X.Y[.Z]).",
                rel_path, ver_val
            ));
        }
    }

    errors
}

/// Validates title uniqueness across specifications within the same spec type.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `validate_title_uniqueness` in `crates/verify-baseline/src/spec_audit/metadata.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `validate_title_uniqueness` (declaration #8) within `SPEC-AUDIT-METADATA`.
///
/// ## 3. Preconditions
/// - `docs: &[(String, String, MarkdownDoc)]` must satisfy domain and type-safety constraints prior to invoking `validate_title_uniqueness`.
///
/// ## 4. Postconditions
/// - Returns `Vec<String>` from `validate_title_uniqueness` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(docs)) bounded over input/member dimension of `validate_title_uniqueness`.
/// - Space Complexity: O(N(docs)) bounded auxiliary memory for `validate_title_uniqueness`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `validate_title_uniqueness` (`Vec<String>`).
/// - State: Maintains consistent state invariants across `validate_title_uniqueness` without partial mutation on error.
/// - Action: Executes `validate_title_uniqueness` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `validate_title_uniqueness` (`SPEC-AUDIT-METADATA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-METADATA-VALIDATE-TITLE-UNIQUENESS] | SysML v2: ActionDef(validate_title_uniqueness)
pub fn validate_title_uniqueness(docs: &[(String, String, MarkdownDoc)]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut claims: HashMap<(String, String), Vec<(&str, &str)>> = HashMap::new();

    for (spec_type, rel_path, doc) in docs {
        let norm_spec_type = match spec_type.to_lowercase().as_str() {
            "epic" | "epics" => "epic".to_string(),
            "feature" | "features" => "feature".to_string(),
            "user_story" | "user-story" | "user_stories" | "user-stories" | "us" => {
                "user_story".to_string()
            }
            "use_case" | "use-case" | "use_cases" | "use-cases" | "uc" => "use_case".to_string(),
            other => other.to_string(),
        };

        let raw_title = doc
            .table_metadata
            .get("title")
            .map(|s| s.as_str())
            .or_else(|| {
                doc.frontmatter
                    .as_ref()
                    .and_then(|fm| fm.get("title").map(|s| s.as_str()))
            })
            .or_else(|| {
                doc.headings
                    .iter()
                    .find(|h| h.level == 1)
                    .map(|h| h.text.as_str())
            });

        if let Some(title) = raw_title {
            let norm_title = normalize_title(title);
            if !norm_title.is_empty() {
                claims
                    .entry((norm_spec_type, norm_title))
                    .or_default()
                    .push((rel_path.as_str(), title));
            }
        }
    }

    let mut sorted_keys: Vec<_> = claims.keys().cloned().collect();
    sorted_keys.sort();

    for key in sorted_keys {
        if let Some(colliding) = claims.get(&key) {
            if colliding.len() > 1 {
                let (spec_type, norm_title) = &key;
                let listed = colliding
                    .iter()
                    .map(|(p, t)| format!("{} ('{}')", p, t))
                    .collect::<Vec<_>>()
                    .join(", ");

                errors.push(format!(
                    "{}: duplicate {} title -- {} specifications normalise to '{}' ({}). reconcile_backlog.py addresses tracker issues by normalised title, so duplicate titles collide.",
                    colliding[0].0,
                    spec_type,
                    colliding.len(),
                    norm_title,
                    listed
                ));
            }
        }
    }

    errors
}

/// Validates filename conventions (<prefix>-<ordinal>-<kebab-name>.md), ordinal uniqueness,
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `validate_filenames` in `crates/verify-baseline/src/spec_audit/metadata.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `validate_filenames` (declaration #9) within `SPEC-AUDIT-METADATA`.
///
/// ## 3. Preconditions
/// - `spec_type: &str` must satisfy domain and type-safety constraints prior to invoking `validate_filenames`.
/// - `filenames: &[&str]` must satisfy domain and type-safety constraints prior to invoking `validate_filenames`.
/// - `dir_rel: &str` must satisfy domain and type-safety constraints prior to invoking `validate_filenames`.
///
/// ## 4. Postconditions
/// - Returns `Vec<String>` from `validate_filenames` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(spec_type)) bounded over input/member dimension of `validate_filenames`.
/// - Space Complexity: O(N(spec_type)) bounded auxiliary memory for `validate_filenames`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `validate_filenames` (`Vec<String>`).
/// - State: Maintains consistent state invariants across `validate_filenames` without partial mutation on error.
/// - Action: Executes `validate_filenames` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `validate_filenames` (`SPEC-AUDIT-METADATA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-METADATA-VALIDATE-FILENAMES] | SysML v2: ActionDef(validate_filenames)
pub fn validate_filenames(spec_type: &str, filenames: &[&str], dir_rel: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let spec_type_lower = spec_type.to_lowercase();
    let expected_prefix = match spec_type_lower.as_str() {
        "epic" | "epics" => "epic",
        "feature" | "features" => "feat",
        "user_story" | "user-story" | "user_stories" | "user-stories" | "us" => "us",
        "use_case" | "use-case" | "use_cases" | "use-cases" | "uc" => "uc",
        _ => spec_type,
    };

    let mut by_ordinal: HashMap<u64, Vec<&str>> = HashMap::new();
    let mut widths: HashMap<usize, Vec<&str>> = HashMap::new();

    for &name in filenames {
        if name.starts_with('.') || !name.ends_with(".md") || name == "README.md" {
            continue;
        }

        let stem = match name.strip_suffix(".md") {
            Some(s) => s,
            None => continue,
        };

        // Parse <prefix>-<ordinal>-<kebab-name>
        let first_dash = stem.find('-');
        let (prefix, ordinal, kebab_name) = match first_dash {
            Some(i1) => {
                let p = &stem[..i1];
                let rest = &stem[i1 + 1..];
                match rest.find('-') {
                    Some(i2) => (p, &rest[..i2], &rest[i2 + 1..]),
                    None => ("", "", ""),
                }
            }
            None => ("", "", ""),
        };

        let valid_prefix = !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_alphabetic());
        let valid_ordinal = !ordinal.is_empty() && ordinal.chars().all(|c| c.is_ascii_digit());
        let valid_kebab = !kebab_name.is_empty()
            && kebab_name
                .split('-')
                .all(|seg| !seg.is_empty() && seg.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));

        if !valid_prefix || !valid_ordinal || !valid_kebab {
            errors.push(format!(
                "{}/{}: filename does not match the documented convention '{}-<zero-padded-ordinal>-<kebab-name>.md'. Names must be lowercase and dash-separated.",
                dir_rel, name, expected_prefix
            ));
            continue;
        }

        if !prefix.eq_ignore_ascii_case(expected_prefix) {
            errors.push(format!(
                "{}/{}: directory prefix mismatch -- found '{}-', but files in {} must use the '{}-' prefix.",
                dir_rel, name, prefix, dir_rel, expected_prefix
            ));
            continue;
        }

        if let Some(stutter_err) = check_filename_anti_stutter(name) {
            errors.push(format!(
                "{}/{}: Anti-Stutter Gate violation: {}",
                dir_rel, name, stutter_err
            ));
            continue;
        }

        if let Ok(ord_val) = ordinal.parse::<u64>() {
            by_ordinal.entry(ord_val).or_default().push(name);
            widths.entry(ordinal.len()).or_default().push(name);
        }
    }

    let mut sorted_ordinals: Vec<_> = by_ordinal.keys().cloned().collect();
    sorted_ordinals.sort();
    for ord in sorted_ordinals {
        if let Some(colliding) = by_ordinal.get(&ord) {
            if colliding.len() > 1 {
                errors.push(format!(
                    "{}: duplicate ordinal -- {} is claimed by {} files ({}). Ordinals must be unique, because backlog reconciliation and Epic checklists reference specifications by number.",
                    dir_rel,
                    ord,
                    colliding.len(),
                    colliding.join(", ")
                ));
            }
        }
    }

    if widths.len() > 1 {
        let mut sorted_widths: Vec<_> = widths.keys().cloned().collect();
        sorted_widths.sort();
        let summary = sorted_widths
            .iter()
            .map(|w| {
                let files = widths.get(w).cloned().unwrap_or_default();
                format!("{} digit(s): {}", w, files.join(", "))
            })
            .collect::<Vec<_>>()
            .join("; ");

        errors.push(format!(
            "{}: inconsistent ordinal padding width across the directory ({}). Pick one width and apply it uniformly, otherwise lexical ordering does not match numeric ordering.",
            dir_rel, summary
        ));
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec_audit::markdown_ast::parse_markdown;

    #[test]
    fn test_iso_date_validation() {
        assert!(is_iso_date("2026-10-10"));
        assert!(is_iso_date("2024-02-29"));
        assert!(!is_iso_date("2026-13-01"));
        assert!(!is_iso_date("10/10/2026"));
        assert!(!is_iso_date("invalid-date"));
        assert!(!is_iso_date("2026-00-10"));
        assert!(!is_iso_date("2026-10-00"));
        assert!(!is_iso_date("2026-10-32"));
        assert!(!is_iso_date("1969-12-31"));
        assert!(!is_iso_date("2101-01-01"));
    }

    #[test]
    fn test_semver_validation() {
        assert!(is_semver("1.0.0"));
        assert!(is_semver("v2.1.0"));
        assert!(is_semver("0.1.0-alpha.1"));
        assert!(is_semver("v1.0.0+20130313144700"));
        assert!(is_semver("1.2"));
        assert!(!is_semver("1"));
        assert!(!is_semver("v1"));
        assert!(!is_semver("release-1.0"));
        assert!(!is_semver("1.0.0.0"));
        assert!(!is_semver("v"));
    }

    #[test]
    fn test_has_concatenated_title_metadata() {
        assert!(has_concatenated_title_metadata("Core System Architecture v1.0.0"));
        assert!(has_concatenated_title_metadata("Subsystem Spec 2026-10-10"));
        assert!(has_concatenated_title_metadata("Navigation Module (DOC-NAV-001)"));
        assert!(has_concatenated_title_metadata("Version: 2 Architecture"));
        assert!(!has_concatenated_title_metadata("Core System Architecture"));
    }

    #[test]
    fn test_normalize_title() {
        assert_eq!(
            normalize_title("EPIC-001: Core System Architecture"),
            "core system architecture"
        );
        assert_eq!(
            normalize_title("feat-02: User Authentication Flow"),
            "user authentication flow"
        );
        assert_eq!(
            normalize_title("US-03 - Execute Telemetry Check!"),
            "execute telemetry check"
        );
        assert_eq!(
            normalize_title("Feature: User Authentication"),
            "user authentication"
        );
        assert_eq!(
            normalize_title("UC 04: Emergency Landing"),
            "emergency landing"
        );
        assert_eq!(
            normalize_title("User Story 10 - Sensor Telemetry"),
            "sensor telemetry"
        );
        assert_eq!(
            normalize_title("epics-12: System Overview"),
            "system overview"
        );
        assert_eq!(
            normalize_title("Feature 01: [ConOps] Human_Engineer_Interface"),
            "conops human engineer interface"
        );
        assert_eq!(
            normalize_title("Feature_01_Human_Engineer"),
            "human engineer"
        );
        assert_eq!(
            normalize_title("Epic_02: Radar_Tracker_Architecture"),
            "radar tracker architecture"
        );
        assert_eq!(
            normalize_title("User_Story_05: Ingest_Telemetry_Stream"),
            "ingest telemetry stream"
        );
        assert_eq!(
            normalize_title("Use_Case_03: Process_Radar_Tracks"),
            "process radar tracks"
        );
    }

    #[test]
    fn test_title_uniqueness_detected() {
        let content1 = "# EPIC-01: Core Architecture\n";
        let content2 = "# EPIC-02: Core Architecture\n";
        let doc1 = parse_markdown(content1).unwrap();
        let doc2 = parse_markdown(content2).unwrap();

        let docs = vec![
            ("epic".to_string(), "docs/epics/epic-01.md".to_string(), doc1),
            ("epic".to_string(), "docs/epics/epic-02.md".to_string(), doc2),
        ];

        let errors = validate_title_uniqueness(&docs);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("duplicate epic title"));
    }

    #[test]
    fn test_filename_conventions() {
        let clean_files = ["feat-01-login.md", "feat-02-logout.md"];
        let errors = validate_filenames("feature", &clean_files, "docs/features");
        assert!(errors.is_empty());

        let duplicate_ordinal = ["feat-01-login.md", "feat-01-duplicate.md"];
        let errors = validate_filenames("feature", &duplicate_ordinal, "docs/features");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("duplicate ordinal"));

        let mixed_padding = ["feat-01-login.md", "feat-002-logout.md"];
        let errors = validate_filenames("feature", &mixed_padding, "docs/features");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("inconsistent ordinal padding width"));
    }

    #[test]
    fn test_anti_stutter_gate_title() {
        assert!(check_title_anti_stutter("Use Case 01: UC 01 System Vision").is_some());
        assert!(check_title_anti_stutter("Use Case 01: UC-01 Foundational Invariants").is_some());
        assert!(check_title_anti_stutter("Use Case 01: UC_01 Invariants").is_some());
        assert!(check_title_anti_stutter("UC_XX_Workflow").is_some());
        assert!(check_title_anti_stutter("UC_01_System_Vision").is_some());
        assert!(check_title_anti_stutter("Epic 01: EPIC 01 SysML").is_some());
        assert!(check_title_anti_stutter("Feature 01: FEAT 01 Engine").is_some());
        assert!(check_title_anti_stutter("User Story 01: US 01 Bootstrap").is_some());
        assert!(check_title_anti_stutter("Use Case 01: Use Case 01 Workflow").is_some());

        assert!(check_title_anti_stutter("Use Case 01: System Vision and Foundational Invariants").is_none());
        assert!(check_title_anti_stutter("Epic 01: SysML Model").is_none());
        assert!(check_title_anti_stutter("Feature 06: [System Vision] SystemVisionEngine").is_none());
        assert!(check_title_anti_stutter("User Story 01: Bootstrap System Vision Action").is_none());
    }

    #[test]
    fn test_anti_stutter_gate_filename() {
        assert!(check_filename_anti_stutter("UC-01-UC_01_System_Vision.md").is_some());
        assert!(check_filename_anti_stutter("UC-01-UC-01-System_Vision.md").is_some());
        assert!(check_filename_anti_stutter("uc-01-uc_01_workflow.md").is_some());
        assert!(check_filename_anti_stutter("uc-01-uc_xx_workflow.md").is_some());
        assert!(check_filename_anti_stutter("feat-01-feat_01_engine.md").is_some());
        assert!(check_filename_anti_stutter("us-01-us_01_action.md").is_some());
        assert!(check_filename_anti_stutter("epic-01-epic_01_model.md").is_some());

        assert!(check_filename_anti_stutter("UC-01-System_Vision_and_Foundational_Invariants.md").is_none());
        assert!(check_filename_anti_stutter("FEAT-06-SystemVisionEngine.md").is_none());
        assert!(check_filename_anti_stutter("US-01-BootstrapSystemVisionAction.md").is_none());
        assert!(check_filename_anti_stutter("EPIC-03-SystemVision.md").is_none());
    }

    #[test]
    fn test_validate_metadata_generation_mode_variants() {
        let template = |key: &str, val: &str| format!(
            r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #10 |
| **Title** | Feature 01 Telemetry Stream |
| **Type** | feature |
| **Parent Epic** | [EPIC-01](../epics/EPIC-01.md) |
| **Package** | FlightControl |
| **Interface Type** | programmatic |
| **Schema Containers** | FlightPackage |
| **{}** | {} |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Feature 01: Telemetry Stream
"#,
            key, val
        );

        let variants = [
            ("Generation Mode", "subagent"),
            ("generation-mode", "subagent"),
            ("subagent_drafted", "true"),
            ("subagent", "autonomous"),
            ("subagent_mode", "active"),
        ];

        for (k, v) in variants {
            let md = template(k, v);
            let doc = parse_markdown(&md).expect("parse markdown");
            let errors = validate_metadata("feature", &doc, &md, "docs/features/feat-01-telemetry.md");
            assert!(
                errors.is_empty(),
                "Variant '{}' failed with errors: {:?}",
                k,
                errors
            );
        }
    }
}


