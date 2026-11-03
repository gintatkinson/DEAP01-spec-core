#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] Tiered docstring compliance auditing and 4-Block Contract parsing.
//! Safety & Traceability: Realises: [REQ-FEAT-42-IS-COMPLIANT-DOCBLOCK-MODE], [REQ-CHECK-36-DOCSTRING-GATE], [REQ-0605]
//! Contract: Evaluates existing Rust/Dart docblocks for 7-section or 4-Block Contract compliance, placeholder cleanliness, and per-file body uniqueness.
//! Bounds & Faults: O(D) time and space in docblock line count `D`; pure non-panicking string slices.

use std::collections::BTreeSet;

/// Policy flags governing tiered docblock validation and per-file callable uniqueness checks.
/// Realises: [REQ-FEAT-42-DOC-COMPLIANCE-POLICY]
#[derive(Debug, Clone, Copy)]
pub(crate) struct DocCompliancePolicy {
    pub is_tier_a: bool,
    pub check_uniqueness: bool,
    pub allow_legacy_tier_bc_sections: bool,
}

/// Canonical block header discriminant for the Tier A Concise 4-Block Contract.
/// Realises: [REQ-FEAT-42-FOUR-BLOCK-HEADER]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FourBlockHeader {
    SafetyAndTraceability,
    Contract,
    BoundsAndFaults,
}

/// Detects legacy 7-section numbered H2 headers (`## 1.` through `## 7.`) on a docblock.
/// Realises: [REQ-FEAT-42-HAS-LEGACY-SEVEN-SECTION-HEADERS]
#[must_use]
pub(crate) fn has_legacy_seven_section_headers(doc_lines: &[String]) -> bool {
    doc_lines.iter().any(|line| {
        let text = strip_outer_doc_marker(line);
        matches!(
            text.get(..5),
            Some("## 1." | "## 2." | "## 3." | "## 4." | "## 5." | "## 6." | "## 7.")
        )
    })
}

/// Checks whether `doc_lines` satisfies tiered Rust/Dart documentation compliance (`is_tier_a`).
/// Realises: [REQ-FEAT-42-IS-COMPLIANT-DOCBLOCK-MODE]
#[must_use]
pub(crate) fn is_compliant_docblock_with_mode(
    doc_lines: &[String],
    is_tier_a: bool,
    allow_legacy_tier_bc_sections: bool,
) -> bool {
    if doc_lines.is_empty()
        || any_placeholder_in_doc(doc_lines)
        || has_malformed_prefix_or_hybrid_headers(doc_lines)
    {
        return false;
    }
    let joined = doc_lines.join("\n");
    if !docblock_has_non_empty_realises(&joined) {
        return false;
    }
    let is_module_doc = doc_lines
        .iter()
        .any(|line| line.trim_start().starts_with("//!"));
    if is_tier_a || is_module_doc {
        (has_all_seven_sections(&joined) && has_non_empty_tier_bc_summary(doc_lines))
            || has_valid_four_block_contract(doc_lines)
    } else if allow_legacy_tier_bc_sections {
        has_non_empty_tier_bc_summary(doc_lines)
    } else {
        !has_legacy_seven_section_headers(doc_lines) && has_non_empty_tier_bc_summary(doc_lines)
    }
}

/// [DOC-TIER: A] Checks whether `doc_lines` satisfies tiered Rust/Dart documentation compliance.
/// Safety & Traceability: Realises: [REQ-FEAT-42-IS-COMPLIANT-DOCBLOCK]
/// Contract: Validates 7-section or 4-Block Contract (when `is_tier_a`) or concise Tier-B/C summary + `Realises:`.
/// Bounds & Faults: O(D) time in docblock line count `D`; infallible.
#[must_use]
pub fn is_compliant_docblock(doc_lines: &[String], is_tier_a: bool) -> bool {
    is_compliant_docblock_with_mode(doc_lines, is_tier_a, false)
}

/// Evaluates `is_compliant_docblock_with_mode` and enforces per-file normalized body uniqueness.
/// Realises: [REQ-FEAT-42-IS-COMPLIANT-DOCBLOCK-IN-FILE]
pub(crate) fn is_compliant_docblock_in_file_with_mode(
    current_doc: &[String],
    policy: DocCompliancePolicy,
    seen_docblocks: &mut BTreeSet<String>,
) -> bool {
    if !is_compliant_docblock_with_mode(
        current_doc,
        policy.is_tier_a,
        policy.allow_legacy_tier_bc_sections,
    ) {
        return false;
    }
    if policy.check_uniqueness {
        let joined = current_doc.join("\n");
        if has_all_seven_sections(&joined) {
            let normalized = normalize_docblock_body(current_doc);
            if normalized.is_empty() || !seen_docblocks.insert(normalized) {
                return false;
            }
        }
    }
    true
}

/// Checks whether a joined docblock contains all 7 canonical DO-178C numbered H2 section headers.
/// Realises: [REQ-FEAT-42-HAS-ALL-SEVEN-SECTIONS]
#[must_use]
pub(crate) fn has_all_seven_sections(joined: &str) -> bool {
    joined.contains("## 1. Safety Intent & Regulatory Scope")
        && joined.contains("## 2. Invariants & Mathematical Properties")
        && joined.contains("## 3. Preconditions")
        && joined.contains("## 4. Postconditions")
        && joined.contains("## 5. Algorithmic Complexity Bounds")
        && joined.contains("## 6. Failure Modes & Fault Tolerance")
        && joined.contains("## 7. Realization & Traceability")
}

/// Strips the leading `///` or `//!` doc-comment marker and surrounding whitespace from a single line.
/// Realises: [REQ-FEAT-42-STRIP-OUTER-DOC-MARKER]
#[must_use]
pub(crate) fn strip_outer_doc_marker(line: &str) -> &str {
    let trimmed = line.trim();
    trimmed
        .strip_prefix("///")
        .or_else(|| trimmed.strip_prefix("//!"))
        .unwrap_or(trimmed)
        .trim()
}

/// Classifies a stripped doc-comment line against the three Tier A 4-Block Contract headers.
/// Realises: [REQ-FEAT-42-CLASSIFY-FOUR-BLOCK-HEADER]
#[must_use]
pub(crate) fn classify_four_block_header(line_text: &str) -> Option<(FourBlockHeader, &str)> {
    let clean = line_text
        .strip_prefix("- ")
        .or_else(|| line_text.strip_prefix("* "))
        .or_else(|| line_text.strip_prefix("### "))
        .or_else(|| line_text.strip_prefix("## "))
        .unwrap_or(line_text)
        .trim_start();
    let unbolded = clean.strip_prefix("**").unwrap_or(clean).trim_start();
    let (header, after_label) = if let Some(rest) = unbolded.strip_prefix("Safety & Traceability") {
        (FourBlockHeader::SafetyAndTraceability, rest)
    } else if let Some(rest) = unbolded.strip_prefix("Bounds & Faults") {
        (FourBlockHeader::BoundsAndFaults, rest)
    } else {
        let rest = unbolded.strip_prefix("Contract")?;
        (FourBlockHeader::Contract, rest)
    };
    let after_bold = after_label.trim_start();
    let after_bold = after_bold.strip_prefix("**").unwrap_or(after_bold).trim_start();
    if after_bold.is_empty() {
        return Some((header, ""));
    }
    let after_colon = after_bold.strip_prefix(':')?;
    let tail = after_colon.trim_start();
    Some((header, tail.strip_prefix("**").unwrap_or(tail).trim()))
}

/// Verifies that `doc_lines` satisfies the Tier A Concise 4-Block Contract.
/// Realises: [REQ-FEAT-42-HAS-VALID-FOUR-BLOCK-CONTRACT]
#[must_use]
pub(crate) fn has_valid_four_block_contract(doc_lines: &[String]) -> bool {
    let (mut mask, mut seen_headers, mut active_bit) = (0u8, 0u8, 0u8);
    for line in doc_lines {
        let text = strip_outer_doc_marker(line);
        if text.is_empty() {
            continue;
        }
        if let Some((header, inline_tail)) = classify_four_block_header(text) {
            let bit: u8 = match header {
                FourBlockHeader::SafetyAndTraceability => 0b0010,
                FourBlockHeader::Contract => 0b0100,
                FourBlockHeader::BoundsAndFaults => 0b1000,
            };
            seen_headers |= bit;
            if inline_tail.chars().any(char::is_alphanumeric) {
                mask |= bit;
                active_bit = 0;
            } else {
                active_bit = bit;
            }
        } else {
            if seen_headers == 0
                && !text.starts_with("Realises:")
                && !text.starts_with('#')
                && text.chars().any(char::is_alphanumeric)
            {
                mask |= 0b0001;
            }
            if active_bit != 0 && text.chars().any(char::is_alphanumeric) {
                mask |= active_bit;
            }
        }
    }
    mask == 0b1111
}

/// Checks whether `doc_lines` contains at least one non-empty descriptive summary line.
/// Realises: [REQ-FEAT-42-HAS-NON-EMPTY-TIER-BC-SUMMARY]
#[must_use]
pub(crate) fn has_non_empty_tier_bc_summary(doc_lines: &[String]) -> bool {
    for line in doc_lines {
        let text = strip_outer_doc_marker(line);
        if text.is_empty() || text.starts_with("## ") || text.starts_with("# ") {
            continue;
        }
        if let Some((before_realises, _)) = text.split_once("Realises:") {
            let prefix = before_realises.trim();
            if classify_four_block_header(prefix).is_none()
                && prefix.chars().any(char::is_alphanumeric)
            {
                return true;
            }
        } else if text.chars().any(char::is_alphanumeric) {
            return true;
        }
    }
    false
}

/// Detects doubled doc-comment prefixes, legacy H1 headers, or duplicate `Realises:` lines.
/// Realises: [REQ-FEAT-42-HAS-MALFORMED-PREFIX-OR-HYBRID-HEADERS]
#[must_use]
pub(crate) fn has_malformed_prefix_or_hybrid_headers(current_doc: &[String]) -> bool {
    let mut realises_count: usize = 0;
    for line in current_doc {
        let trimmed = line.trim();
        let after_outer = trimmed
            .strip_prefix("///")
            .or_else(|| trimmed.strip_prefix("//!"))
            .unwrap_or(trimmed)
            .trim_start();
        if after_outer.starts_with("///")
            || after_outer.starts_with("//!")
            || (after_outer.starts_with("# ") && !after_outer.starts_with("## "))
        {
            return true;
        }
        if after_outer.starts_with("Realises:") {
            realises_count = realises_count.saturating_add(1);
        }
    }
    realises_count > 1
}

/// Formats a single canonical `///` docstring line at `indent`.
/// Realises: [REQ-FEAT-42-FORMAT-NORMALIZED-DOC-LINE]
#[must_use]
pub(crate) fn format_normalized_doc_line(indent: &str, text: &str) -> String {
    let clean = clean_doc_line_payload(text);
    if clean.is_empty() {
        format!("{indent}///")
    } else {
        format!("{indent}/// {clean}")
    }
}

/// Normalizes a `///` docblock identically to Check 34 and Check 36.
/// Realises: [REQ-FEAT-42-NORMALIZE-DOCBLOCK-BODY]
#[must_use]
pub(crate) fn normalize_docblock_body(doc_lines: &[String]) -> String {
    doc_lines
        .iter()
        .skip(1)
        .filter(|doc_line| !doc_line.contains("Realises:"))
        .map(|doc_line| {
            doc_line
                .trim_start_matches("///")
                .trim_start_matches("//!")
                .trim()
        })
        .filter(|doc_text| !doc_text.is_empty())
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Strips repeated leading `///` or `//!` markers and surrounding whitespace from a docstring line.
/// Realises: [REQ-FEAT-42-CLEAN-DOC-LINE-PAYLOAD]
#[must_use]
pub(crate) fn clean_doc_line_payload(line: &str) -> &str {
    let mut text = line.trim();
    while let Some(rest) = text.strip_prefix("///").or_else(|| text.strip_prefix("//!")) {
        text = rest.trim_start();
    }
    text.trim()
}

/// Checks whether a joined docblock contains `Realises: [<non-empty>]`.
/// Realises: [REQ-FEAT-42-DOCBLOCK-NON-EMPTY-REALISES]
#[must_use]
pub(crate) fn docblock_has_non_empty_realises(doc_block: &str) -> bool {
    if let Some((_, after_prefix)) = doc_block.split_once("Realises: [") {
        if let Some((inside, _)) = after_prefix.split_once(']') {
            return !inside.contains('\n') && !inside.trim().is_empty();
        }
    }
    false
}

/// Checks whether any line in `doc_lines` is a forbidden placeholder docstring.
/// Realises: [REQ-FEAT-42-ANY-PLACEHOLDER-IN-DOC]
#[must_use]
pub(crate) fn any_placeholder_in_doc(doc_lines: &[String]) -> bool {
    doc_lines.iter().any(|l| has_placeholder_doc_line(l))
}

/// Replaces balanced inline backtick code spans and double-quoted literals on a single line with spaces.
/// Realises: [REQ-FEAT-42-STRIP-BALANCED-CODE-AND-QUOTE-SPANS]
fn strip_balanced_code_and_quote_spans(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let (mut offset, mut skip_until) = (0usize, 0usize);
    for ch in raw.chars() {
        let idx = offset;
        offset = offset.saturating_add(ch.len_utf8());
        if idx < skip_until {
            continue;
        }
        if ch == '`' || ch == '"' {
            let after_open = raw.get(offset..).unwrap_or("");
            if let Some(rel_close) = after_open.find(ch) {
                skip_until = offset.saturating_add(rel_close).saturating_add(1);
                out.push(' ');
                continue;
            }
        }
        out.push(ch);
    }
    out
}

/// Checks whether a single docstring line contains forbidden placeholder phrases or marker tokens.
/// Realises: [REQ-FEAT-42-HAS-PLACEHOLDER-DOC-LINE]
#[must_use]
pub(crate) fn has_placeholder_doc_line(doc_line: &str) -> bool {
    let raw = strip_outer_doc_marker(doc_line);
    let body = raw.to_ascii_lowercase();
    let mut tokens = body.split_whitespace();
    let matches_stub_phrase = match (tokens.next(), tokens.next(), tokens.next()) {
        (Some("member"), Some("documentation"), None) => true,
        (Some("member"), Some(second), _) => second.starts_with("documentation."),
        (Some("todo:"), Some(second), _) => second.starts_with("document"),
        (Some("placeholder"), Some(second), _) => second.starts_with("documentation"),
        _ => false,
    };
    if matches_stub_phrase {
        return true;
    }
    let unquoted = strip_balanced_code_and_quote_spans(raw);
    unquoted
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '!')
        .any(|tok| {
            tok.eq_ignore_ascii_case("todo")
                || tok.eq_ignore_ascii_case("tbd")
                || tok.eq_ignore_ascii_case("fixme")
                || tok.eq_ignore_ascii_case("xxx")
        })
}
