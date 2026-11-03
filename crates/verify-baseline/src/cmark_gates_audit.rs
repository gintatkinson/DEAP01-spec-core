//! Grammar-backed `CommonMark` AST specification audit extractors and validators (`spec_audit`). [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides `pulldown-cmark` AST-backed YAML frontmatter extraction, 2-column metadata table
//! extraction, ConOps table extraction with 1-based source line tracking, per-file Check 26
//! SI metrology auditing (`#744`), and Epic checklist/subsection validation (`#737`) under
//! DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D (`BP-04` / `WBS 01.04.05`).
//!
//! ## 2. Invariants & Mathematical Properties
//! - HTML comments (`<!-- ... -->`) and fenced code blocks are masked/excluded by `pulldown-cmark`
//!   while preserving exact UTF-8 byte offsets and 1-based line numbers.
//! - Zero ad-hoc `.lines()` + `.split('|')` state machines are used for table extraction.
//!
//! ## 3. Preconditions
//! - Input `content` is a valid UTF-8 Markdown slice.
//!
//! ## 4. Postconditions
//! - Produces deterministic metadata maps, `MarkdownTable` vectors, and `SpecAuditFinding` records.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear over Markdown byte length $N$.
//! - Space Complexity: $O(N)$ proportional to extracted AST structures.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Malformed Markdown degrades gracefully per `CommonMark` specification; invalid SI units
//!   and missing Epic subsections emit explicit `SpecAuditFinding` records.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0604], [REQ-0401], [BP-04] [FLS: fls_t4w1da759n2e, fls_qcb1n9c0e5hz]

#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]

use crate::checks::parity_check_icd_ast::{is_metrology_placeholder, is_valid_si_unit};
use crate::spec_audit::conops_audit::{MarkdownTable, TableRow};
use crate::spec_audit::markdown_placeholders::normalize_table_metadata_key;
use crate::spec_audit::provenance_refs::is_valid_epic_feature_checklist_item;
use crate::spec_audit::SpecAuditFinding;
use deap_core::markdown_ast::{
    cmark_options, is_br_html_tag, mask_html_comments_preserving_offsets,
    parse_frontmatter_kv_rows, parse_markdown_ast,
};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::collections::BTreeMap;

/// Computes byte offsets of line starts for $O(\log N)$ 1-based line lookups (`[REQ-0604]`, `[FLS: fls_t4w1da759n2e]`).
fn build_line_starts(content: &str) -> Vec<usize> {
    let mut starts = Vec::with_capacity(content.len() / 40 + 1);
    starts.push(0);
    for (idx, b) in content.bytes().enumerate() {
        if b == b'\n' {
            starts.push(idx.saturating_add(1));
        }
    }
    starts
}

/// Converts a byte offset into a 1-based line number via binary search (`[REQ-0604]`, `[FLS: fls_t4w1da759n2e]`).
fn offset_to_line(line_starts: &[usize], offset: usize) -> usize {
    match line_starts.binary_search(&offset) {
        Ok(idx) => idx.saturating_add(1),
        Err(idx) => idx.max(1),
    }
}

/// [DOC-TIER: A] Extracts YAML frontmatter key-value pairs via `deap_core::markdown_ast` metadata blocks.
/// Safety & Traceability: Realises: [REQ-0604], [FLS: fls_t4w1da759n2e]
/// Contract: Parses `MetadataBlockKind::YamlStyle` blocks via `parse_markdown_ast` into a `BTreeMap`.
/// Bounds & Faults: O(N) time and space; returns `None` when no valid YAML frontmatter block exists.
#[must_use]
pub fn extract_yaml_frontmatter_cmark(content: &str) -> Option<BTreeMap<String, String>> {
    let ast = parse_markdown_ast(content);
    let mut map = BTreeMap::new();
    for block in &ast.metadata_blocks {
        for entry in block.split('\n') {
            let trimmed = entry.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = trimmed.split_once(':') {
                let key = k
                    .trim()
                    .trim_matches(|c| c == '\'' || c == '"')
                    .to_ascii_lowercase();
                let val = v.trim().trim_matches(|c| c == '\'' || c == '"');
                if !key.is_empty() {
                    map.insert(key, val.to_string());
                }
            }
        }
    }
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

/// [DOC-TIER: A] Extracts 2-column metadata table key-value pairs via `deap_core::markdown_ast`.
/// Safety & Traceability: Realises: [REQ-0604], [FLS: fls_t4w1da759n2e]
/// Contract: Uses `parse_frontmatter_kv_rows` and `parse_markdown_ast` without `.lines()` splitting.
/// Bounds & Faults: O(N) time and space; returns a deterministic `BTreeMap<String, String>`.
#[must_use]
pub fn extract_table_metadata_cmark(content: &str) -> BTreeMap<String, String> {
    let mut table_metadata = BTreeMap::new();
    for row in parse_frontmatter_kv_rows(content) {
        insert_normalized_kv(&row.raw_key, &row.raw_value, &mut table_metadata);
    }
    if table_metadata.is_empty() {
        let ast = parse_markdown_ast(content);
        if let Some(first_tbl) = ast.tables.first() {
            for row in &first_tbl.rows {
                if let (Some(k), Some(v)) = (row.first(), row.get(1)) {
                    insert_normalized_kv(k, v, &mut table_metadata);
                }
            }
        }
    }
    table_metadata
}

/// Normalizes and inserts a single metadata key-value pair (`[REQ-0604]`, `[FLS: fls_t4w1da759n2e]`).
fn insert_normalized_kv(
    raw_key: &str,
    raw_val: &str,
    table_metadata: &mut BTreeMap<String, String>,
) {
    let normalized_key = normalize_table_metadata_key(raw_key);
    if normalized_key.is_empty()
        || matches!(normalized_key.as_str(), "attribute" | "key" | "field")
    {
        return;
    }
    let val = raw_val
        .trim()
        .trim_matches(|c| c == '`' || c == '"' || c == '\'');
    table_metadata.insert(normalized_key, val.to_string());
}

/// [DOC-TIER: A] Extracts all GFM tables with 1-based line numbers using `pulldown-cmark` offset events.
/// Safety & Traceability: Realises: [REQ-0604], [FLS: fls_t4w1da759n2e]
/// Contract: Replaces line-by-line pipe scanning in `conops_audit` with grammar-backed `Tag::Table` traversal.
/// Bounds & Faults: O(N) time and space; ignores tables inside fenced code blocks and HTML comments.
#[must_use]
pub fn extract_conops_tables_cmark(content: &str) -> Vec<MarkdownTable> {
    let masked = mask_html_comments_preserving_offsets(content);
    let line_starts = build_line_starts(content);
    let mut tables = Vec::new();
    let mut cur_headers = Vec::new();
    let mut cur_rows = Vec::new();
    let mut cur_cells = Vec::new();
    let mut cur_cell = String::new();
    let mut in_head = false;
    let mut in_cell = false;
    let mut table_start_line = 1usize;
    let mut row_line = 1usize;

    for (event, range) in Parser::new_ext(&masked, cmark_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::Table(_)) => {
                table_start_line = offset_to_line(&line_starts, range.start);
                cur_headers.clear();
                cur_rows.clear();
            }
            Event::Start(Tag::TableHead) => {
                in_head = true;
                cur_cells.clear();
            }
            Event::End(TagEnd::TableHead) => {
                in_head = false;
                cur_headers = std::mem::take(&mut cur_cells);
            }
            Event::Start(Tag::TableRow) => {
                row_line = offset_to_line(&line_starts, range.start);
                cur_cells.clear();
            }
            Event::End(TagEnd::TableRow) => {
                if !in_head {
                    cur_rows.push(TableRow {
                        cells: std::mem::take(&mut cur_cells),
                        line_no: row_line,
                    });
                }
            }
            Event::Start(Tag::TableCell) => {
                in_cell = true;
                cur_cell.clear();
            }
            Event::End(TagEnd::TableCell) => {
                in_cell = false;
                cur_cells.push(std::mem::take(&mut cur_cell).trim().to_string());
            }
            Event::Text(t) | Event::Code(t) if in_cell => cur_cell.push_str(&t),
            Event::Html(h) | Event::InlineHtml(h) if in_cell && is_br_html_tag(&h) => {
                cur_cell.push(' ');
            }
            Event::End(TagEnd::Table) => {
                tables.push(MarkdownTable {
                    headers: std::mem::take(&mut cur_headers),
                    rows: std::mem::take(&mut cur_rows),
                    start_line: table_start_line,
                });
            }
            _ => {}
        }
    }
    tables
}

/// [DOC-TIER: A] Enforces Check 26 SI 7D Parameter Metrology on an audited specification file (`#744`).
/// Safety & Traceability: Realises: [REQ-PARITY-GATE-26], [REQ-0604], [FLS: fls_t4w1da759n2e]
/// Contract: Validates `unit` columns in `signal`, `parameter`, and `metric` tables during `--spec-only` runs.
/// Bounds & Faults: O(N) time; returns `SpecAuditFinding` entries for missing, placeholder, or non-SI units.
#[must_use]
pub fn validate_file_si_metrology_cmark(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    for table_kind in ["signal", "parameter", "metric"] {
        let (headers, rows) =
            crate::cmark_gates::parity::parse_section_table_rows_cmark(content, table_kind);
        if let Some(unit_col) = headers.iter().position(|h| h.contains("unit")) {
            for row in &rows {
                if let Some(unit_val) = row.get(unit_col) {
                    let clean = unit_val.trim().trim_matches(|c| c == '`' || c == '*');
                    if clean.is_empty()
                        || is_metrology_placeholder(clean)
                        || !is_valid_si_unit(clean)
                    {
                        findings.push(SpecAuditFinding::new(
                            "standards-parameter-metrology",
                            format!(
                                "Check 26 failed: Invalid or missing SI dimensional unit '{clean}' in '{rel_path}'"
                            ),
                            rel_path,
                            None,
                        ));
                    }
                }
            }
        }
    }
    findings
}

/// [DOC-TIER: A] Validates Epic associated subsections and relative link checklists via `CommonMark` AST (`#737`).
/// Safety & Traceability: Realises: [REQ-0604], [FLS: fls_t4w1da759n2e]
/// Contract: Verifies `### Associated Use Cases & User Stories` and `#### Associated Use Cases` / `User Stories`.
/// Bounds & Faults: O(N) time; returns `SpecAuditFinding` records on missing subsections or unlinked items.
#[must_use]
pub fn validate_epic_checklist_and_subsections_cmark(
    content: &str,
    rel_path: &str,
) -> Vec<SpecAuditFinding> {
    let ast = parse_markdown_ast(content);
    let mut findings = Vec::new();
    for required in [
        "Associated Use Cases & User Stories",
        "Associated Use Cases",
        "Associated User Stories",
    ] {
        let req_lower = required.to_ascii_lowercase();
        if !ast
            .sections
            .iter()
            .any(|s| s.heading.to_ascii_lowercase().contains(&req_lower))
        {
            findings.push(SpecAuditFinding::new(
                "mandatory-sections-missing",
                format!("Epic '{rel_path}' missing required subsection '{required}' (#737)"),
                rel_path,
                None,
            ));
        }
    }
    for task in &ast.task_items {
        let slice = content
            .get(task.item_start_offset..task.item_end_offset)
            .unwrap_or("")
            .trim();
        if slice.contains("FEAT-") && !is_valid_epic_feature_checklist_item(slice) {
            findings.push(SpecAuditFinding::new(
                "epic-checklist-link-grammar",
                format!("Invalid Epic Feature checklist item grammar in '{rel_path}': '{slice}'"),
                rel_path,
                None,
            ));
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies frontmatter, metadata table, ConOps table, and SI metrology AST gates (`[REQ-0604]`).
    #[test]
    fn test_cmark_audit_extractors_and_metrology() {
        let md = "---\ntitle: \"Demo\"\ntype: feature\n---\n# H1\n| Attribute | Specification Detail |\n| :--- | :--- |\n| **Issue ID** | `42` |\n\n## Parameter Table\n| Parameter | Unit |\n| :--- | :--- |\n| `velocity` | `not_a_unit` |\n";
        let fm = extract_yaml_frontmatter_cmark(md);
        assert_eq!(
            fm.as_ref().and_then(|m| m.get("title")).map(String::as_str),
            Some("Demo")
        );
        let meta = extract_table_metadata_cmark(md);
        assert_eq!(meta.get("issue_id").map(String::as_str), Some("42"));
        let tables = extract_conops_tables_cmark(md);
        assert_eq!(tables.len(), 2);
        let findings = validate_file_si_metrology_cmark(md, "docs/features/FEAT-01.md");
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings.first().map(|f| f.rule_id.as_str()),
            Some("standards-parameter-metrology")
        );
    }

    /// Verifies AST section table extraction, separator checks, and lossless roundtrip (`[REQ-0604]`).
    #[test]
    fn test_cmark_gates_parity_and_blueprint_helpers() -> Result<(), String> {
        use crate::cmark_gates::{blueprint, parity};
        let md = "## Signal Flow Dictionary\n\n| Signal ID | Unit |\n| :--- | :--- |\n| `SIG-01` | `m/s` |\n";
        let (headers, rows) = parity::parse_section_table_rows_cmark(md, "signal flow");
        assert_eq!(headers, vec!["signal id".to_string(), "unit".to_string()]);
        assert_eq!(rows.len(), 1);
        assert!(parity::is_table_separator_line_cmark("| :--- | :---: |"));
        assert!(!parity::is_table_separator_line_cmark("| val | val |"));
        let cols = blueprint::split_table_row_cmark("| `a \\| b` | c |")
            .ok_or_else(|| "expected split".to_string())?;
        assert_eq!(cols.len(), 2);
        assert!(blueprint::is_table_separator_row_cmark(
            &[":---".to_string(), "---:".to_string()],
            2
        ));
        let rt = blueprint::verify_cmark_lossless_roundtrip("# Title\n\nParagraph.\n")
            .map_err(|e| e.to_string())?;
        assert!(rt.contains("# Title"));
        Ok(())
    }

    /// Verifies `#734`, `#745`, `#746`, and `#747` parity gates fail closed on violations (`[REQ-0604]`).
    #[test]
    fn test_issue_734_745_746_747_gates() {
        use crate::cmark_gates::parity;
        assert!(parity::verify_zero_hardcoded_airworthiness_clauses(
            "Enforces 14 CFR Part 107 limit of 25.0 kg."
        )
        .is_err());
        assert!(parity::validate_m2_performer_and_viewpoints_cmark(
            "| Performer | Type |\n| :--- | :--- |\n| TrackTarget | action def |\n",
            "rules/sysml-ssot-completeness.md"
        )
        .is_err());
        assert!(parity::validate_part_attribute_domain_conformance_cmark(
            "TelemetryRouter",
            "cyber",
            &["mass_kg".to_string(), "latency_ms".to_string()]
        )
        .is_err());
        assert!(parity::validate_pipeline0_phase05_registry_order(
            "Step 1: SAFETY_01_STPA_HAZARD_ANALYSIS.md",
            "README.md"
        )
        .is_err());
        assert!(parity::validate_pipeline0_phase05_registry_order(
            "Phase 0.5: docs/research/FAILURE_MODE_REGISTRY.md -> SAFETY_01_STPA_HAZARD_ANALYSIS.md",
            "README.md"
        )
        .is_ok());
    }
}
