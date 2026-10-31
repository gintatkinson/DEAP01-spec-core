//! Use Case doc-comment clause boundary recognition, inline splitting, Markdown decorator normalization, and parenthetical diagnostic extraction.
//!
//! Safety & Traceability: Realises: [REQ-SYNC-FORWARD-SPLIT-INLINE-UC-CLAUSES]
//! Contract: Recognizes canonical UML Use Case and BDD clause headers case-insensitively across plain and bolded Markdown forms, splits inline bullet, semicolon, and sentence clauses without breaking SysML statements, unwraps leading Markdown decorators, and extracts deduplicated parenthetical or bracketed diagnostic codes.
//! Bounds & Faults: Enforces bounds-checked UTF-8 slicing (`.get`, `.strip_prefix`, `.split_once`, `.char_indices`) and saturating arithmetic (`saturating_add`, `saturating_sub`) across bounded passes with zero regular expressions, direct indexing, or panics.

use super::*;

/// Canonical case-insensitive clause header prefixes recognized inside `UseCaseDef.doc` blocks.
/// Realises: [REQ-SYNC-FORWARD-UC-DOC-CLAUSE-LABELS]
pub(crate) const USE_CASE_DOC_CLAUSE_LABELS: &[&str] = &[
    "Actors:",
    "Actor:",
    "Subject:",
    "Preconditions:",
    "Precondition:",
    "Trigger:",
    "Main Success Scenario:",
    "Alternate and Exception Flows:",
    "Alternate Flows:",
    "Exception Flows:",
    "Postconditions:",
    "Postcondition:",
    "Given:",
    "When:",
    "Then:",
    "Diagnostic:",
];

/// Checks whether `s` begins with a recognized unbolded (`Label:`) or bolded (`**Label**:` / `**Label:**`) `UseCaseDef.doc` clause header.
/// Realises: [REQ-SYNC-FORWARD-RECOGNIZE-UC-CLAUSE-LABEL]
pub(crate) fn starts_with_recognized_uc_clause_label(s: &str) -> bool {
    let trimmed = s.trim_start();
    for label in USE_CASE_DOC_CLAUSE_LABELS {
        if strip_prefix_ignore_ascii_case(trimmed, label).is_some() {
            return true;
        }
        let stem = label.trim_end_matches(':');
        if let Some(after_stars) = trimmed.strip_prefix("**") {
            if let Some(after_stem) = strip_prefix_ignore_ascii_case(after_stars, stem) {
                let rem = after_stem.trim_start();
                if rem.starts_with("**:") || rem.starts_with(":**") {
                    return true;
                }
            }
        }
    }
    false
}

/// Appends a non-empty trimmed slice from `seg_opt` into `segments`.
/// Realises: [REQ-SYNC-FORWARD-SPLIT-INLINE-UC-CLAUSES]
fn push_trimmed_clause_segment<'a>(segments: &mut Vec<&'a str>, seg_opt: Option<&'a str>) {
    if let Some(seg) = seg_opt {
        let trimmed = seg.trim();
        if !trimmed.is_empty() {
            segments.push(trimmed);
        }
    }
}

/// Splits a single line of `UseCaseDef.doc` text into inline behavioral clause segments across bullet, semicolon, and sentence boundaries without splitting SysML statements or parenthetical diagnostics.
/// Realises: [REQ-SYNC-FORWARD-SPLIT-INLINE-UC-CLAUSES]
pub(crate) fn split_inline_use_case_clauses(line: &str) -> Vec<&str> {
    let mut segments: Vec<&str> = Vec::new();
    let mut seg_start: usize = 0;

    for (idx, ch) in line.char_indices() {
        if idx <= seg_start {
            continue;
        }
        let Some(tail) = line.get(idx..) else {
            continue;
        };

        if ch == ';' {
            if let Some(after_semi) = tail.get(1..) {
                let trimmed_after = after_semi.trim_start();
                let candidate = trimmed_after
                    .strip_prefix("- ")
                    .or_else(|| trimmed_after.strip_prefix("* "))
                    .or_else(|| trimmed_after.strip_prefix("+ "))
                    .map_or(trimmed_after, str::trim_start);
                if starts_with_recognized_uc_clause_label(candidate) {
                    push_trimmed_clause_segment(&mut segments, line.get(seg_start..idx));
                    seg_start = idx.saturating_add(1);
                    continue;
                }
            }
        }

        let prev_is_whitespace = line
            .get(..idx)
            .and_then(|prefix| prefix.chars().next_back())
            .is_some_and(char::is_whitespace);
        if !prev_is_whitespace {
            continue;
        }

        if let Some(after_bullet) = tail
            .strip_prefix("- ")
            .or_else(|| tail.strip_prefix("* "))
            .or_else(|| tail.strip_prefix("+ "))
        {
            let inner_candidate = after_bullet
                .trim_start()
                .strip_prefix("- ")
                .or_else(|| after_bullet.trim_start().strip_prefix("* "))
                .map_or(after_bullet, str::trim_start);
            if starts_with_recognized_uc_clause_label(inner_candidate) {
                push_trimmed_clause_segment(&mut segments, line.get(seg_start..idx));
                seg_start = idx;
                continue;
            }
        }

        let prev_sentence_ended = line
            .get(..idx)
            .map(str::trim_end)
            .is_some_and(|p| p.ends_with('.'));
        if prev_sentence_ended && starts_with_recognized_uc_clause_label(tail) {
            push_trimmed_clause_segment(&mut segments, line.get(seg_start..idx));
            seg_start = idx;
        }
    }

    push_trimmed_clause_segment(&mut segments, line.get(seg_start..));
    segments
}

/// Formats an unwrapped bold clause label stem and trailing remainder when `label_stem` contains only identifier/space characters.
/// Realises: [REQ-SYNC-FORWARD-NORMALIZE-DOC-CLAUSE-HEAD]
fn format_unwrapped_bold_clause(label_stem: &str, rest: &str) -> Option<String> {
    let valid_label = !label_stem.is_empty()
        && label_stem
            .chars()
            .all(|c| c.is_alphanumeric() || c == ' ' || c == '_' || c == '-');
    if !valid_label {
        return None;
    }
    let trimmed_rest = rest.trim_start();
    if trimmed_rest.is_empty() {
        Some(format!("{label_stem}:"))
    } else {
        Some(format!("{label_stem}: {trimmed_rest}"))
    }
}

/// Normalizes leading Markdown/comment decorators (`# `, `- `, `* `, `+ `, `> `, `<digits>. `, `<digits>) `) and unwraps leading bold label wrappers (`**Label**:` or `**Label:**`).
/// Realises: [REQ-SYNC-FORWARD-NORMALIZE-DOC-CLAUSE-HEAD]
pub(crate) fn normalize_doc_clause_head(raw: &str) -> String {
    let mut cur = raw.trim();

    let hash_count = cur.chars().take_while(|c| *c == '#').count();
    if hash_count > 0 {
        if let Some(after_hashes) = cur.get(hash_count..) {
            if after_hashes.starts_with(' ') || after_hashes.starts_with('\t') {
                cur = after_hashes.trim_start();
            }
        }
    }

    for _ in 0..4 {
        if let Some(rest) = cur
            .strip_prefix("- ")
            .or_else(|| cur.strip_prefix("* "))
            .or_else(|| cur.strip_prefix("+ "))
            .or_else(|| cur.strip_prefix("> "))
        {
            cur = rest.trim_start();
            continue;
        }
        let digit_len = cur.chars().take_while(|c| c.is_ascii_digit()).count();
        if digit_len > 0 {
            if let Some(after_digits) = cur.get(digit_len..) {
                if let Some(rest) = after_digits
                    .strip_prefix(". ")
                    .or_else(|| after_digits.strip_prefix(") "))
                {
                    cur = rest.trim_start();
                    continue;
                }
            }
        }
        break;
    }

    if let Some(after_open) = cur.strip_prefix("**") {
        if let Some((inner, after_close)) = after_open.split_once("**") {
            if inner.ends_with(':') {
                let label_stem = inner.trim_end_matches(':').trim();
                if let Some(unwrapped) = format_unwrapped_bold_clause(label_stem, after_close) {
                    return unwrapped;
                }
            } else if let Some(rest_raw) = after_close.trim_start().strip_prefix(':') {
                let label_stem = inner.trim();
                if let Some(unwrapped) = format_unwrapped_bold_clause(label_stem, rest_raw) {
                    return unwrapped;
                }
            }
        }
    }

    cur.to_string()
}

/// Matches a leading clause `label` case-insensitively and returns the trimmed value slice after `label`.
/// Realises: [REQ-SYNC-FORWARD-MATCH-UC-LABEL-CI]
pub(crate) fn match_case_insensitive_uc_label<'a>(s: &'a str, label: &str) -> Option<&'a str> {
    strip_prefix_ignore_ascii_case(s, label).map(str::trim)
}

/// Appends a non-empty trimmed clause to `target` while preserving first-seen insertion order and deduplicating exact matches.
/// Realises: [REQ-SYNC-FORWARD-PUSH-UNIQUE-UC-CLAUSE]
pub(crate) fn push_unique_uc_clause(target: &mut Vec<String>, value: &str) {
    let clean = value.trim();
    if !clean.is_empty() && !target.iter().any(|existing| existing == clean) {
        target.push(clean.to_string());
    }
}

/// Normalizes a `Diagnostic:` value by stripping outer whitespace, backticks, quotes, trailing `.`/`;`/`,` punctuation, and unmatched trailing `)` or `]` delimiters.
/// Realises: [REQ-SYNC-FORWARD-CLEAN-DIAGNOSTIC-VALUE]
pub(crate) fn clean_diagnostic_value(val: &str) -> String {
    let mut cur = val.trim();
    for _ in 0..8 {
        let prev = cur;
        cur = cur
            .trim()
            .trim_matches('`')
            .trim_matches('"')
            .trim_end_matches(['.', ';', ','])
            .trim();
        let open_parens = cur.chars().filter(|c| *c == '(').count();
        let close_parens = cur.chars().filter(|c| *c == ')').count();
        if close_parens > open_parens {
            if let Some(stripped) = cur.strip_suffix(')') {
                cur = stripped.trim();
            }
        }
        let open_brackets = cur.chars().filter(|c| *c == '[').count();
        let close_brackets = cur.chars().filter(|c| *c == ']').count();
        if close_brackets > open_brackets {
            if let Some(stripped) = cur.strip_suffix(']') {
                cur = stripped.trim();
            }
        }
        if cur == prev {
            break;
        }
    }
    cur.to_string()
}

/// Finds the UTF-8 byte offset of the matching closing delimiter (`)` or `]`) for a span starting immediately after `open_ch`.
/// Realises: [REQ-SYNC-FORWARD-FIND-MATCHING-DELIM]
pub(crate) fn find_matching_delimiter(
    after_open: &str,
    open_ch: char,
    close_ch: char,
) -> Option<usize> {
    let mut depth: usize = 1;
    for (idx, ch) in after_open.char_indices() {
        if ch == open_ch {
            depth = depth.saturating_add(1);
        } else if ch == close_ch {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}

/// Extracts inline parenthetical or bracketed diagnostic annotations such as ``(`Diagnostic: E0453`)`` or `(Diagnostic: E0453)` into `alternate_flows`.
/// Realises: [REQ-SYNC-FORWARD-EXTRACT-INLINE-DIAGNOSTICS]
pub(crate) fn extract_inline_diagnostics(text: &str, alternate_flows: &mut Vec<String>) {
    for (idx, ch) in text.char_indices() {
        let close_ch = match ch {
            '(' => ')',
            '[' => ']',
            _ => continue,
        };
        let Some(after_open) = text.get(idx.saturating_add(ch.len_utf8())..) else {
            continue;
        };
        let Some(close_offset) = find_matching_delimiter(after_open, ch, close_ch) else {
            continue;
        };
        let Some(inside) = after_open.get(..close_offset) else {
            continue;
        };
        let unbackticked: String = inside.chars().filter(|c| *c != '`').collect();
        let norm_inside = normalize_doc_clause_head(&unbackticked);
        if let Some(raw_diag) = match_case_insensitive_uc_label(&norm_inside, "Diagnostic:") {
            let cleaned = clean_diagnostic_value(raw_diag);
            if !cleaned.is_empty() {
                let formatted = format!("Diagnostic: {cleaned}");
                push_unique_uc_clause(alternate_flows, &formatted);
            }
        }
    }
}
