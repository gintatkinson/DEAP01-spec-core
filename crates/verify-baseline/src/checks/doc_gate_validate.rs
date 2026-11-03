#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Tier A 4-Block Contract, summary line, placeholder detection, and delimiter depth helpers for Check 36.
//! Realises: [REQ-CHECK-36-DOCSTRING-GATE], [DO-330-TQL-4-DOCSTRING-GATE]

use crate::checks::docstring_gate::{
    has_all_seven_sections, strip_doc_comment_marker, DocTier, DocstringSection, ItemDocTarget,
};
use deap_core::rust_cst::source_bridge::update_ast_delimiter_depths;
use std::collections::BTreeSet;

/// Strongly typed enumeration of the three mandatory contract headers in a Tier A Concise 4-Block Contract.
/// Realises: [REQ-CHECK-36-FOUR-BLOCK-HEADER-ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FourBlockHeader {
    SafetyAndTraceability,
    Contract,
    BoundsAndFaults,
}

impl FourBlockHeader {
    /// Complete catalog of all three mandatory Tier A 4-Block Contract headers.
    /// Realises: [REQ-CHECK-36-FOUR-BLOCK-HEADER-ENUM]
    pub(crate) const ALL: &'static [Self] = &[
        Self::SafetyAndTraceability,
        Self::BoundsAndFaults,
        Self::Contract,
    ];

    /// Projects a `FourBlockHeader` variant onto its canonical Option A2 heading prefix literal.
    /// Realises: [REQ-CHECK-36-FOUR-BLOCK-HEADER-LABEL]
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::SafetyAndTraceability => "Safety & Traceability",
            Self::Contract => "Contract",
            Self::BoundsAndFaults => "Bounds & Faults",
        }
    }

    /// Projects a `FourBlockHeader` variant onto its unique completion bit in `has_valid_four_block_contract`.
    /// Realises: [REQ-CHECK-36-FOUR-BLOCK-HEADER-BIT]
    pub(crate) const fn bit(self) -> u8 {
        match self {
            Self::SafetyAndTraceability => 0b0010,
            Self::Contract => 0b0100,
            Self::BoundsAndFaults => 0b1000,
        }
    }
}

/// Matches a stripped doc-comment line against the three Tier A Option A2 4-Block Contract headers.
/// Realises: [REQ-CHECK-36-CLASSIFY-FOUR-BLOCK-HEADER]
pub(crate) fn classify_four_block_header(line_text: &str) -> Option<(FourBlockHeader, &str)> {
    let clean = line_text
        .strip_prefix("- ")
        .or_else(|| line_text.strip_prefix("* "))
        .or_else(|| line_text.strip_prefix("### "))
        .or_else(|| line_text.strip_prefix("## "))
        .unwrap_or(line_text)
        .trim_start();
    let unbolded = clean.strip_prefix("**").unwrap_or(clean).trim_start();
    for &header in FourBlockHeader::ALL {
        if let Some(after_label) = unbolded.strip_prefix(header.label()) {
            let after_bold = after_label.trim_start();
            let after_bold = after_bold
                .strip_prefix("**")
                .unwrap_or(after_bold)
                .trim_start();
            if after_bold.is_empty() {
                return Some((header, ""));
            }
            if let Some(after_colon) = after_bold.strip_prefix(':') {
                let tail = after_colon.trim_start();
                let tail = tail.strip_prefix("**").unwrap_or(tail).trim();
                return Some((header, tail));
            }
        }
    }
    None
}

/// Verifies that `doc_lines` satisfies the Tier A Option A2 Concise 4-Block Contract.
/// Realises: [REQ-CHECK-36-HAS-VALID-FOUR-BLOCK-CONTRACT]
pub(crate) fn has_valid_four_block_contract(doc_lines: &[String]) -> bool {
    let mut mask: u8 = 0;
    let mut seen_headers: u8 = 0;
    let mut active_header_bit: u8 = 0;

    for line in doc_lines {
        let text = strip_doc_comment_marker(line);
        if text.is_empty() {
            continue;
        }
        if let Some((header, inline_tail)) = classify_four_block_header(text) {
            let bit = header.bit();
            seen_headers |= bit;
            if inline_tail.chars().any(char::is_alphanumeric) {
                mask |= bit;
                active_header_bit = 0;
            } else {
                active_header_bit = bit;
            }
        } else {
            if seen_headers == 0 && !text.starts_with("Realises:") && !text.starts_with('#') {
                let prose = super::docstring_tier_rules::strip_tier_and_trace_tags(text);
                if prose.chars().any(char::is_alphanumeric) {
                    mask |= 0b0001;
                }
            }
            if active_header_bit != 0 && text.chars().any(char::is_alphanumeric) {
                mask |= active_header_bit;
            }
        }
    }
    mask == 0b1111
}

/// Checks whether `doc_lines` contains at least one non-empty descriptive prose/summary line distinct from `Realises:`.
/// Realises: [REQ-CHECK-36-HAS-NON-EMPTY-SUMMARY-LINE]
pub(crate) fn has_non_empty_summary_line(doc_lines: &[String]) -> bool {
    for line in doc_lines {
        let text = strip_doc_comment_marker(line);
        if text.is_empty() || text.starts_with("# ") {
            continue;
        }
        if text.starts_with("## ") {
            let has_inline_section_prose = DocstringSection::ALL.iter().any(|sec| {
                text.strip_prefix(sec.header())
                    .and_then(|rest| rest.trim_start().strip_prefix(':'))
                    .is_some_and(|after_colon| {
                        let prose = after_colon
                            .split_once("Realises:")
                            .map(|(before, _)| before)
                            .unwrap_or(after_colon);
                        prose.chars().any(char::is_alphanumeric)
                    })
            });
            if has_inline_section_prose {
                return true;
            }
            continue;
        }
        if let Some((before_realises, _)) = text.split_once("Realises:") {
            let prefix =
                super::docstring_tier_rules::strip_tier_and_trace_tags(before_realises.trim());
            if classify_four_block_header(prefix.trim()).is_none()
                && prefix.chars().any(char::is_alphanumeric)
            {
                return true;
            }
        } else {
            let prose = super::docstring_tier_rules::strip_tier_and_trace_tags(text);
            if prose.chars().any(char::is_alphanumeric) {
                return true;
            }
        }
    }
    false
}

/// Replaces balanced single-line backtick code spans and double-quoted literals in `raw` with spaces.
/// Realises: [REQ-CHECK-36-STRIP-BALANCED-CODE-AND-QUOTE-SPANS]
pub(crate) fn strip_balanced_code_and_quote_spans(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut skip_until: usize = 0;
    let mut idx: usize = 0;

    for ch in raw.chars() {
        let next_idx = idx.saturating_add(ch.len_utf8());
        if idx < skip_until {
            idx = next_idx;
            continue;
        }
        if ch == '`' || ch == '"' {
            let after_open = raw.get(next_idx..).unwrap_or("");
            if let Some(rel_close) = after_open.find(ch) {
                skip_until = next_idx.saturating_add(rel_close).saturating_add(1);
                out.push(' ');
                idx = next_idx;
                continue;
            }
        }
        out.push(ch);
        idx = next_idx;
    }
    out
}

/// Detects forbidden placeholder docstrings and unquoted unfinished markers on a `///` or `//!` comment line.
/// Realises: [REQ-CHECK-36-PLACEHOLDER-DETECTION]
pub(crate) fn contains_placeholder_docstring(doc_line: &str) -> bool {
    let raw = strip_doc_comment_marker(doc_line);
    let lower = raw.to_ascii_lowercase();
    let mut words = lower.split_whitespace();
    let matches_stub_phrase = match (words.next(), words.next(), words.next()) {
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

/// Updates `brace_depth` and `paren_depth` across a single comment-stripped code line.
/// Realises: [REQ-CHECK-36-DELIMITER-DEPTH-TRACKER]
pub(crate) fn update_line_delimiter_depths(
    code: &str,
    brace_depth: &mut usize,
    paren_depth: &mut usize,
) {
    update_ast_delimiter_depths(code, brace_depth, paren_depth);
}

/// Validates `current_doc` against `target.tier`, `Realises: [...]` traceability, and per-file uniqueness.
/// Realises: [REQ-CHECK-36-VALIDATE-ITEM-DOCBLOCK]
pub(crate) fn validate_item_docblock(
    current_doc: &[String],
    target: &ItemDocTarget<'_>,
    seen_docblocks: &mut BTreeSet<String>,
    violations: &mut Vec<String>,
) {
    if current_doc.is_empty() {
        let msg = match target.tier {
            DocTier::RustTierA => format!(
                "{}:{}: Tier A {} '{}' missing mandatory docstring (requires 7-section Rustdoc or 4-Block Contract)",
                target.file_rel, target.line_no, target.kind, target.item_name
            ),
            DocTier::RustTierBC => format!(
                "{}:{}: Tier B/C {} '{}' missing mandatory docstring (requires concise summary and 'Realises: [...]')",
                target.file_rel, target.line_no, target.kind, target.item_name
            ),
            DocTier::DartSevenSection => format!(
                "{}:{}: {} '{}' missing mandatory 7-section docstring",
                target.file_rel, target.line_no, target.kind, target.item_name
            ),
        };
        violations.push(msg);
        return;
    }
    let doc_block = current_doc.join("\n");
    let is_seven_section = validate_item_tier_body(current_doc, &doc_block, target, violations);
    let has_traceability = match target.tier {
        DocTier::DartSevenSection => {
            super::docstring_tier_rules::has_non_empty_realises_tag(&doc_block)
        }
        DocTier::RustTierA | DocTier::RustTierBC => {
            super::docstring_tier_rules::has_traceability_anchor(&doc_block)
        }
    };
    if !has_traceability {
        violations.push(format!(
            "{}:{}: {} '{}' missing non-empty '[REQ-...]', '[FLS: fls_...]', or 'Realises: [...]' traceability tag",
            target.file_rel, target.line_no, target.kind, target.item_name
        ));
    }
    if target.check_uniqueness && is_seven_section {
        record_unique_seven_section_body(current_doc, target, seen_docblocks, violations);
    }
}

/// Validates tier-specific section headers, 4-block contracts, and line caps for `target`.
/// Realises: [REQ-CHECK-36-VALIDATE-ITEM-DOCBLOCK]
fn validate_item_tier_body(
    current_doc: &[String],
    doc_block: &str,
    target: &ItemDocTarget<'_>,
    violations: &mut Vec<String>,
) -> bool {
    let mut is_seven_section = false;
    match target.tier {
        DocTier::DartSevenSection => {
            let missing = collect_missing_seven_sections(doc_block);
            is_seven_section = missing.is_empty();
            if !missing.is_empty() {
                violations.push(format!(
                    "{}:{}: {} '{}' missing mandatory 7-section docstring (missing: {})",
                    target.file_rel,
                    target.line_no,
                    target.kind,
                    target.item_name,
                    missing.join(", ")
                ));
            }
        }
        DocTier::RustTierA => {
            let missing = collect_missing_seven_sections(doc_block);
            is_seven_section = has_all_seven_sections(doc_block) && has_non_empty_summary_line(current_doc);
            let is_four_block = has_valid_four_block_contract(current_doc);
            if !is_seven_section && !is_four_block {
                violations.push(format!(
                    "{}:{}: Tier A {} '{}' missing valid 7-section Rustdoc or 4-Block Contract (missing 7-section headers: {})",
                    target.file_rel,
                    target.line_no,
                    target.kind,
                    target.item_name,
                    missing.join(", ")
                ));
            }
        }
        DocTier::RustTierBC => {
            if !has_non_empty_summary_line(current_doc) {
                violations.push(format!(
                    "{}:{}: Tier B/C {} '{}' missing non-empty descriptive summary line",
                    target.file_rel, target.line_no, target.kind, target.item_name
                ));
            }
            if super::docstring_tier_rules::exceeds_unratcheted_tier_bc_line_cap(
                current_doc.len(),
                target.file_rel,
            ) {
                violations.push(format!(
                    "{}:{}: Tier B/C {} '{}' docstring has {} lines (exceeds <= {} line cap for non-ratcheted module)",
                    target.file_rel,
                    target.line_no,
                    target.kind,
                    target.item_name,
                    current_doc.len(),
                    super::docstring_tier_rules::MAX_TIER_BC_DOC_LINES
                ));
            }
        }
    }
    is_seven_section
}

/// Collects the ordered short diagnostic labels of any canonical 7-section headers absent from `doc_block`.
/// Realises: [REQ-CHECK-36-COLLECT-MISSING-SEVEN-SECTIONS]
fn collect_missing_seven_sections(doc_block: &str) -> Vec<&'static str> {
    let mut missing = Vec::new();
    for section in DocstringSection::ALL {
        if !doc_block.contains(section.header()) {
            missing.push(section.short_label());
        }
    }
    missing
}

/// Normalizes a 7-section callable docstring body and verifies per-file uniqueness in `seen_docblocks`.
/// Realises: [REQ-CHECK-36-RECORD-UNIQUE-SEVEN-SECTION-BODY]
fn record_unique_seven_section_body(
    current_doc: &[String],
    target: &ItemDocTarget<'_>,
    seen_docblocks: &mut BTreeSet<String>,
    violations: &mut Vec<String>,
) {
    let normalized_doc = current_doc
        .iter()
        .skip(1)
        .filter(|doc_line| !doc_line.contains("Realises:"))
        .map(|doc_line| strip_doc_comment_marker(doc_line))
        .filter(|doc_text| !doc_text.is_empty())
        .collect::<Vec<&str>>()
        .join("\n");
    if !normalized_doc.is_empty() && !seen_docblocks.insert(normalized_doc) {
        violations.push(format!(
            "{}:{}: {} '{}' has duplicate/boilerplate 7-section docstring body",
            target.file_rel, target.line_no, target.kind, target.item_name
        ));
    }
}
