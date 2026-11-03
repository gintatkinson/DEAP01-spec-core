//! Label, title, specification type, and ordinal normalization utilities for tracker reconciliation.
//! Realises: [REQ-RECONCILE-BACKLOG-TRACKER]

/// [DOC-TIER: A] Checks whether a tracker label indicates a resolved/completed item.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-IS-RESOLVED-LABEL]
/// Contract: Matches `status:fixed-resolved`, `status::fixed-resolved`, `status:resolved`, `status:done`, `fixed-resolved`, `resolved`, `done`.
/// Bounds & Faults: Time O(N), Space O(N) where N is `label.len()`.
pub fn is_resolved_label(label: &str) -> bool {
    let norm = normalize_label(label);
    matches!(
        norm.as_str(),
        "status:fixed-resolved"
            | "status:resolved"
            | "status:done"
            | "status:completed"
            | "fixed-resolved"
            | "resolved"
            | "done"
    )
}

/// [DOC-TIER: A] Normalizes a tracker label string (lowercased, `::` collapsed to `:`, spaces/underscores to `-`).
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-NORMALIZE-LABEL]
/// Contract: Produces a canonical lowercase label representation.
/// Bounds & Faults: Time O(N), Space O(N) where N is `label.len()`.
pub fn normalize_label(label: &str) -> String {
    let mut s = label.trim().to_lowercase();
    while s.contains("::") {
        s = s.replace("::", ":");
    }
    s = s.replace(['_', ' '], "-");
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    s
}

/// [DOC-TIER: A] Canonicalizes a specification type string or label into `"epic"`, `"feature"`, `"user_story"`, or `"use_case"`.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-CANONICALIZE-SPEC-TYPE]
/// Contract: Maps variations (`type:epic`, `User Story`, `use-case`) to canonical snake_case identifiers.
/// Bounds & Faults: Time O(N), Space O(N) where N is `raw.len()`.
pub fn canonicalize_spec_type(raw: &str) -> String {
    let norm = normalize_label(raw);
    let stripped = norm
        .strip_prefix("type:")
        .or_else(|| norm.strip_prefix("kind:"))
        .unwrap_or(&norm)
        .trim_matches('-');

    match stripped {
        "epic" | "epics" => "epic".to_string(),
        "feature" | "features" | "feat" => "feature".to_string(),
        "user-story" | "userstory" | "user-stories" | "story" | "stories" | "us" => {
            "user_story".to_string()
        }
        "use-case" | "usecase" | "use-cases" | "uc" => "use_case".to_string(),
        other => other.replace('-', "_"),
    }
}

/// [DOC-TIER: A] Infers the canonical specification type from a title prefix (`EPIC-`, `FEAT-`, `US-`, `UC-`).
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-INFER-SPEC-TYPE-FROM-TITLE]
/// Contract: Returns `Some("epic" | "feature" | "user_story" | "use_case")` or `None`.
/// Bounds & Faults: Time O(N), Space O(N) where N is `title.len()`.
pub fn infer_spec_type_from_title(title: &str) -> Option<String> {
    let mut s = title.trim().to_uppercase();
    while s.starts_with('[') || s.starts_with('`') || s.starts_with('*') || s.starts_with('#') {
        s = s.get(1..).unwrap_or("").trim_start().to_string();
    }

    if s.starts_with("EPIC-") || s.starts_with("EPIC ") || s.starts_with("EPIC:") {
        Some("epic".to_string())
    } else if s.starts_with("FEAT-")
        || s.starts_with("FEATURE-")
        || s.starts_with("FEAT ")
        || s.starts_with("FEATURE:")
        || s.starts_with("FEATURE ")
    {
        Some("feature".to_string())
    } else if s.starts_with("US-")
        || s.starts_with("STORY-")
        || s.starts_with("USER STORY")
        || s.starts_with("US ")
    {
        Some("user_story".to_string())
    } else if s.starts_with("UC-")
        || s.starts_with("USE CASE")
        || s.starts_with("USE-CASE")
        || s.starts_with("UC ")
    {
        Some("use_case".to_string())
    } else {
        None
    }
}

/// [DOC-TIER: A] Checks whether an issue title carries an `[AUDIT]` defect prefix.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER]
/// Contract: Returns `true` if `title` starts with `[AUDIT]` or `[audit]` after leading whitespace.
/// Bounds & Faults: Time O(N), Space O(1) where N is `title.len()`.
pub fn is_audit_issue_title(title: &str) -> bool {
    let trimmed = title.trim_start();
    trimmed.starts_with("[AUDIT]") || trimmed.starts_with("[audit]")
}

/// [DOC-TIER: A] Normalizes a specification or issue title for deterministic comparison.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-NORMALIZE-TITLE]
/// Contract: Strips Markdown links, formatting tokens, and punctuation; lowercases and collapses whitespace.
/// Bounds & Faults: Time O(N), Space O(N) where N is `title.len()`.
pub fn normalize_title(title: &str) -> String {
    let mut s = title.trim().to_string();

    if s.starts_with('[') {
        if let Some(close_bracket) = s.find("](") {
            s = s.get(1..close_bracket).unwrap_or(&s).to_string();
        }
    }

    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();

    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    tokens.join(" ")
}

/// [DOC-TIER: A] Extracts the canonical `(prefix, numeric_ordinal)` from a specification title (e.g. `("feat", 1)`).
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-EXTRACT-SPEC-ORDINAL]
/// Contract: Recognizes `EPIC-001`, `FEAT-02`, `US-10`, `UC-003`, `Use Case 32`, `User Story 45` and normalizes prefix aliases.
/// Bounds & Faults: Time O(N), Space O(N) where N is `title.len()`.
pub fn extract_spec_ordinal(title: &str) -> Option<(String, u64)> {
    let norm = normalize_title(title);
    let parts: Vec<&str> = norm.split_whitespace().collect();
    if let (Some(&first), Some(&second)) = (parts.first(), parts.get(1)) {
        let single_prefix = match first {
            "epic" => Some("epic"),
            "feat" | "feature" => Some("feat"),
            "us" | "story" => Some("us"),
            "uc" => Some("uc"),
            _ => None,
        };
        if let Some(p) = single_prefix {
            if let Ok(num) = second.parse::<u64>() {
                return Some((p.to_string(), num));
            }
        }
        let two_word_prefix = match (first, second) {
            ("use", "case") => Some("uc"),
            ("user", "story") => Some("us"),
            _ => None,
        };
        if let (Some(p), Some(&third)) = (two_word_prefix, parts.get(2)) {
            if let Ok(num) = third.parse::<u64>() {
                return Some((p.to_string(), num));
            }
        }
    }
    None
}

/// [DOC-TIER: A] Checks whether two titles have compatible specification ordinals (`EPIC-001` vs `EPIC-002`).
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-ORDINAL-COMPATIBLE]
/// Contract: Returns `true` only when both carry identical ordinals or neither carries an ordinal.
/// Bounds & Faults: Time O(N), Space O(N) over title lengths.
pub fn ordinal_compatible(title_a: &str, title_b: &str) -> bool {
    match (extract_spec_ordinal(title_a), extract_spec_ordinal(title_b)) {
        (Some(ord_a), Some(ord_b)) => ord_a == ord_b,
        (None, None) => true,
        _ => false,
    }
}

/// Checks whether `spec_title` (normalized as `norm_spec`) and normalized `cand_title` match fuzzily with ordinal safety.
/// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-ORDINAL-COMPATIBLE]
pub(crate) fn titles_match_fuzzy(spec_title: &str, norm_spec: &str, cand_title: &str) -> bool {
    if !ordinal_compatible(spec_title, cand_title) {
        return false;
    }
    if cand_title.contains(norm_spec) || norm_spec.contains(cand_title) {
        return true;
    }
    if extract_spec_ordinal(spec_title).is_some() && extract_spec_ordinal(cand_title).is_some() {
        let body_a = norm_spec.split_whitespace().skip(2).collect::<Vec<_>>().join(" ");
        let body_b = cand_title.split_whitespace().skip(2).collect::<Vec<_>>().join(" ");
        return !body_a.is_empty()
            && !body_b.is_empty()
            && (body_a.contains(&body_b) || body_b.contains(&body_a));
    }
    false
}
