#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Concrete syntax tree (CST) item docstring verification gate. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D qualification standards.
//! Enforces CST-backed safety docstrings, traceability anchors, and anti-padding rules
//! over Rust source files under REQ-0602.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Deterministic parsing: Syntax trees are parsed losslessly using `ParsedRustFile`.
//! - Bounded line limits: Tier-B/C docstrings must not exceed maximum permitted lines.
//! - Traceability completeness: Every item carries valid requirement and safety anchors.
//!
//! ## 3. Preconditions
//! - Callers supply valid UTF-8 source strings and relative file paths.
//!
//! ## 4. Postconditions
//! - Returns `Ok(count)` of verified items or `Err(Vec<String>)` containing violations.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear in source text length and item count.
//! - Space Complexity: $O(K)$ bounded by extracted item count and violation lists.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Syntax errors and placeholder docstrings fail closed via `Result::Err`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0602], [SC-01], [FLS: fls_verify_docstring_cst_gate]

use crate::checks::docstring_gate::{
    contains_placeholder_docstring, validate_item_docblock, DocTier as GateDocTier, ItemDocTarget,
};
use crate::checks::docstring_tier_rules::has_explicit_tier_a_marker;
use deap_core::rust_cst::{
    classify_file_doc_tier, list_rust_items, DocTier, ParsedRustFile, RustCstItem, RustItemKind,
};
use std::collections::BTreeSet;

/// Audits a Rust source file for docstring completeness using lossless CST parsing.
/// Realises: [REQ-0602]
pub fn audit_rust_file_cst_docstrings(source: &str, file_rel: &str) -> Result<usize, Vec<String>> {
    let mut violations = Vec::new();
    audit_module_docstrings(source, file_rel, &mut violations);

    let _file_tier: DocTier = classify_file_doc_tier(source);
    let parsed = ParsedRustFile::parse_rust_source(source);
    for err in parsed.syntax_errors() {
        violations.push(format!("{}: syntax error: {}", file_rel, err));
    }
    if !violations.is_empty() && !parsed.syntax_errors().is_empty() {
        return Err(violations);
    }

    let items = list_rust_items(&parsed);
    let test_scopes = collect_test_scopes(&items);
    let mut documented_count: usize = 0;
    let mut seen_docblocks = BTreeSet::new();

    for item in &items {
        let Some(kind_str) = classify_auditable_kind(item.kind) else {
            continue;
        };
        let in_test = item.attrs.iter().any(|a| a.contains("test") || a.contains("cfg(test)"))
            || test_scopes.iter().any(|(s, e)| item.start_line >= *s && item.end_line <= *e);
        if in_test {
            continue;
        }

        audit_item_entry(item, kind_str, file_rel, &mut seen_docblocks, &mut violations);
        documented_count = documented_count.saturating_add(1);
    }

    if violations.is_empty() {
        Ok(documented_count)
    } else {
        Err(violations)
    }
}

/// Audits module-level inner docstrings (`//!`) for placeholder markers.
/// Realises: [REQ-0602]
fn audit_module_docstrings(source: &str, file_rel: &str, violations: &mut Vec<String>) {
    for (idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//!") && contains_placeholder_docstring(trimmed) {
            violations.push(format!(
                "{}:{}: placeholder module docstring '{}' detected",
                file_rel,
                idx.saturating_add(1),
                trimmed
            ));
        }
    }
}

/// Identifies auditable declaration kinds: fn, struct, enum, trait.
/// Realises: [REQ-0602]
fn classify_auditable_kind(kind: RustItemKind) -> Option<&'static str> {
    match kind {
        RustItemKind::Fn => Some("fn"),
        RustItemKind::Struct => Some("struct"),
        RustItemKind::Enum => Some("enum"),
        RustItemKind::Trait => Some("trait"),
        _ => None,
    }
}

/// Collects start and end line ranges of test modules and cfg(test) blocks.
/// Realises: [REQ-0602]
fn collect_test_scopes(items: &[RustCstItem]) -> Vec<(usize, usize)> {
    let mut scopes = Vec::new();
    for item in items {
        if (item.kind == RustItemKind::Mod && item.name == "tests")
            || item.attrs.iter().any(|a| a.contains("cfg(test)"))
        {
            scopes.push((item.start_line, item.end_line));
        }
    }
    scopes
}

/// Audits a single CST item docstring against tier and traceability constraints.
/// Realises: [REQ-0602]
fn audit_item_entry(
    item: &RustCstItem,
    kind_str: &'static str,
    file_rel: &str,
    seen_docblocks: &mut BTreeSet<String>,
    violations: &mut Vec<String>,
) {
    for line in &item.doc_comment_lines {
        let trimmed = line.trim();
        if contains_placeholder_docstring(trimmed) {
            violations.push(format!(
                "{}:{}: placeholder docstring '{}' detected",
                file_rel, item.start_line, trimmed
            ));
        }
    }

    let is_tier_a = has_explicit_tier_a_marker(&item.doc_comment_lines);
    let target_tier = if is_tier_a {
        GateDocTier::RustTierA
    } else {
        GateDocTier::RustTierBC
    };

    let target = ItemDocTarget {
        file_rel,
        line_no: item.start_line,
        kind: kind_str,
        item_name: &item.name,
        tier: target_tier,
        check_uniqueness: target_tier == GateDocTier::RustTierA && kind_str == "fn",
    };

    validate_item_docblock(&item.doc_comment_lines, &target, seen_docblocks, violations);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_rust_file_cst_docstrings_valid() {
        let src = concat!(
            "//! Module header.\n",
            "//! Safety & Traceability: Realises: [REQ-0602]\n",
            "/// Summary for record.\n",
            "/// Realises: [REQ-0602]\n",
            "pub struct Record;\n",
            "/// Summary for calculate.\n",
            "/// Realises: [REQ-0602]\n",
            "pub fn calculate() {}\n",
        );
        let res = audit_rust_file_cst_docstrings(src, "crates/test/src/lib.rs");
        assert_eq!(res, Ok(2));
    }

    #[test]
    fn test_audit_rust_file_cst_docstrings_missing() {
        let src = "pub fn undocumented() {}\n";
        let res = audit_rust_file_cst_docstrings(src, "crates/test/src/lib.rs");
        assert!(res.is_err());
        let errs = res.err().unwrap_or_default();
        assert!(errs.iter().any(|e| e.contains("undocumented")));
    }

    #[test]
    fn test_audit_rust_file_cst_docstrings_skips_tests() {
        let src = concat!(
            "/// Summary.\n",
            "/// Realises: [REQ-0602]\n",
            "pub fn prod() {}\n",
            "#[cfg(test)]\n",
            "mod tests {\n",
            "    #[test]\n",
            "    fn test_something() {}\n",
            "}\n",
        );
        let res = audit_rust_file_cst_docstrings(src, "crates/test/src/lib.rs");
        assert_eq!(res, Ok(1));
    }
}
