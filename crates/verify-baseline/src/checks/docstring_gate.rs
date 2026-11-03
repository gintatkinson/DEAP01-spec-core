#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Check 36: Tiered per-item safety docstring completeness gate for Rust and Dart sources.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, ECSS-E-ST-40C, and DEAP Rust Coding
//! Standard v2.0 Section 4. Mechanically enforces tiered per-item safety docstring coverage and
//! `Realises: [...]` requirement bindings across all non-test Rust source files in `crates/` and
//! 7-section docstrings across all non-generated Dart source files in `app_flutter/lib/`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Zero Module-Level Exemptions: Inner module comments (`//!`) never exempt outer item docstrings (`///`).
//! - Zero File Allowlists: Walks 100% of non-test `.rs` files in `crates/` and `.dart` files in `app_flutter/lib/`.
//! - Zero Placeholder Docstrings: Unfinished placeholder markers outside balanced code/quote spans fail closed.
//!
//! ## 3. Preconditions
//! - `repo_root` references an accessible local workspace directory on disk.
//!
//! ## 4. Postconditions
//! - Returns `Ok(())` iff every production item satisfies its normative tier contract, else `Err(Vec<String>)`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(F \times L)$ over scanned files $F$ and lines $L$. Space Complexity: $O(D + V)$.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Unreadable directories, non-UTF-8 source files, or malformed tags fail closed without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-CHECK-36-DOCSTRING-GATE], [DO-330-TQL-4-DOCSTRING-GATE]

pub use crate::checks::doc_gate_dart::audit_dart_source_docstrings;
pub use crate::checks::doc_gate_rust::audit_rust_source_docstrings;
pub(crate) use crate::checks::doc_gate_validate::{
    contains_placeholder_docstring, validate_item_docblock,
};
use crate::checks::doc_gate_validate::{has_non_empty_summary_line, has_valid_four_block_contract};
pub use crate::checks::docstring_cst_gate;
use std::{fs, path::Path};
use walkdir::WalkDir;

/// Strongly typed enumeration of the 7 mandatory DO-178C safety docstring sections enforced by Check 36.
/// Realises: [REQ-CHECK-36-DOCSTRING-SECTION-ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DocstringSection {
    SafetyIntent,
    Invariants,
    Preconditions,
    Postconditions,
    ComplexityBounds,
    FailureModes,
    RealizationTraceability,
}

impl DocstringSection {
    /// Declaration-ordered catalog of all 7 mandatory safety docstring sections.
    /// Realises: [REQ-CHECK-36-DOCSTRING-SECTION-ENUM]
    pub const ALL: &'static [Self] = &[
        Self::SafetyIntent,
        Self::Invariants,
        Self::Preconditions,
        Self::Postconditions,
        Self::ComplexityBounds,
        Self::FailureModes,
        Self::RealizationTraceability,
    ];

    /// Projects a `DocstringSection` variant onto its exact required Markdown H2 heading string.
    /// Realises: [REQ-CHECK-36-DOCSTRING-SECTION-HEADER]
    pub const fn header(self) -> &'static str {
        match self {
            Self::SafetyIntent => "## 1. Safety Intent & Regulatory Scope",
            Self::Invariants => "## 2. Invariants & Mathematical Properties",
            Self::Preconditions => "## 3. Preconditions",
            Self::Postconditions => "## 4. Postconditions",
            Self::ComplexityBounds => "## 5. Algorithmic Complexity Bounds",
            Self::FailureModes => "## 6. Failure Modes & Fault Tolerance",
            Self::RealizationTraceability => "## 7. Realization & Traceability",
        }
    }

    /// Projects a `DocstringSection` variant onto its concise diagnostic summary label.
    /// Realises: [REQ-CHECK-36-DOCSTRING-SECTION-LABEL]
    pub const fn short_label(self) -> &'static str {
        match self {
            Self::SafetyIntent => "1. Safety Intent",
            Self::Invariants => "2. Invariants",
            Self::Preconditions => "3. Preconditions",
            Self::Postconditions => "4. Postconditions",
            Self::ComplexityBounds => "5. Algorithmic Complexity",
            Self::FailureModes => "6. Failure Modes",
            Self::RealizationTraceability => "7. Realization & Traceability",
        }
    }
}

/// Normative documentation tier assigned to an audited Rust or Dart declaration under Check 36.
/// Realises: [REQ-CHECK-36-DOC-TIER-ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DocTier {
    RustTierA,
    RustTierBC,
    DartSevenSection,
}

/// Descriptor identifying an audited Rust or Dart declaration site for `validate_item_docblock`.
/// Realises: [REQ-CHECK-36-ITEM-DOC-TARGET]
pub(crate) struct ItemDocTarget<'a> {
    pub(crate) file_rel: &'a str,
    pub(crate) line_no: usize,
    pub(crate) kind: &'a str,
    pub(crate) item_name: &'a str,
    pub(crate) tier: DocTier,
    pub(crate) check_uniqueness: bool,
}

/// Check 36: Enforces 100% per-item safety docstring coverage across Rust and Dart sources.
/// Realises: [REQ-CHECK-36-DOCSTRING-COMPLETENESS]
pub fn check_docstring_completeness(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    let crates_dir = repo_root.join("crates");
    if crates_dir.is_dir() {
        collect_rust_docstring_violations(&crates_dir, repo_root, &mut violations);
    }
    let app_flutter_lib = repo_root.join("app_flutter").join("lib");
    if app_flutter_lib.is_dir() {
        collect_dart_docstring_violations(&app_flutter_lib, repo_root, &mut violations);
    }
    let root_lib = repo_root.join("lib");
    if repo_root.join("pubspec.yaml").is_file() && root_lib.is_dir() {
        collect_dart_docstring_violations(&root_lib, repo_root, &mut violations);
    }
    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        violations.dedup();
        Err(violations)
    }
}

/// Alias entrypoint for Check 36 7-section safety docstring verification gate.
/// Realises: [REQ-CHECK-36-DOCSTRING-GATE-ALIAS]
pub fn check_7_section_docstring_gate(repo_root: &Path) -> Result<(), Vec<String>> {
    check_docstring_completeness(repo_root)
}

/// Recursively walks `crates_dir` and audits every non-test `.rs` file for per-item docstrings.
/// Realises: [REQ-CHECK-36-COLLECT-RUST-DOCSTRINGS]
fn collect_rust_docstring_violations(
    crates_dir: &Path,
    repo_root: &Path,
    violations: &mut Vec<String>,
) {
    for entry_res in WalkDir::new(crates_dir).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let dir_name = e.file_name().to_string_lossy();
            !is_excluded_rust_dir(&dir_name)
        } else {
            true
        }
    }) {
        let entry = match entry_res {
            Ok(val) => val,
            Err(err) => {
                violations.push(format!("Check 36 Rust directory walk I/O error: {}", err));
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if !path.extension().is_some_and(|ext| ext == "rs") {
            continue;
        }
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(path)
            .display()
            .to_string();
        if crate::checks::doc_gate_rust::is_unit_test_submodule_file(&rel) {
            continue;
        }
        match fs::read_to_string(path) {
            Ok(content) => {
                if let Err(errs) = audit_rust_source_docstrings(&content, &rel) {
                    violations.extend(errs);
                }
            }
            Err(err) => {
                violations.push(format!(
                    "Check 36: Unable to read Rust file '{}': {}",
                    rel, err
                ));
            }
        }
    }
}

/// Recursively walks `dart_lib_dir` and audits every non-generated `.dart` file for 7-section docstrings.
/// Realises: [REQ-CHECK-36-COLLECT-DART-DOCSTRINGS]
fn collect_dart_docstring_violations(
    dart_lib_dir: &Path,
    repo_root: &Path,
    violations: &mut Vec<String>,
) {
    for entry_res in WalkDir::new(dart_lib_dir).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let dir_name = e.file_name().to_string_lossy();
            !is_excluded_dart_dir(&dir_name)
        } else {
            true
        }
    }) {
        let entry = match entry_res {
            Ok(val) => val,
            Err(err) => {
                violations.push(format!("Check 36 Dart directory walk I/O error: {}", err));
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy();
        let is_dart = path.extension().is_some_and(|ext| ext == "dart");
        if !is_dart || is_generated_dart_file(&file_name) {
            continue;
        }
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(path)
            .display()
            .to_string();
        match fs::read_to_string(path) {
            Ok(content) => {
                if let Err(errs) = audit_dart_source_docstrings(&content, &rel) {
                    violations.extend(errs);
                }
            }
            Err(err) => {
                violations.push(format!(
                    "Check 36: Unable to read Dart file '{}': {}",
                    rel, err
                ));
            }
        }
    }
}

/// Determines whether a directory basename inside `crates/` should be excluded from production Rust scanning.
/// Realises: [REQ-CHECK-36-EXCLUDED-RUST-DIR]
fn is_excluded_rust_dir(dir_name: &str) -> bool {
    matches!(
        dir_name,
        "target"
            | ".git"
            | "build"
            | "Pods"
            | "ephemeral"
            | ".dart_tool"
            | "tests"
            | "benches"
            | "examples"
    )
}

/// Determines whether a directory basename inside `app_flutter/lib` should be excluded from Dart scanning.
/// Realises: [REQ-CHECK-36-EXCLUDED-DART-DIR]
fn is_excluded_dart_dir(dir_name: &str) -> bool {
    matches!(
        dir_name,
        ".git" | ".dart_tool" | "build" | "Pods" | "ephemeral" | "generated"
    )
}

/// Determines whether a `.dart` filename corresponds to a tool-generated artifact.
/// Realises: [REQ-CHECK-36-GENERATED-DART-FILE]
fn is_generated_dart_file(file_name: &str) -> bool {
    file_name == "generated_plugin_registrant.dart"
        || file_name.ends_with(".g.dart")
        || file_name.ends_with(".freezed.dart")
        || file_name.ends_with(".gr.dart")
        || file_name.ends_with(".mocks.dart")
}

/// Evaluates whether a standalone Rust outer (`///`) or inner (`//!`) docblock satisfies its normative tier contract.
/// Realises: [REQ-CHECK-36-IS-COMPLIANT-RUST-DOCBLOCK]
pub fn is_compliant_rust_docblock(doc_lines: &[String], is_tier_a: bool) -> bool {
    if doc_lines.is_empty()
        || doc_lines
            .iter()
            .any(|line| contains_placeholder_docstring(line))
    {
        return false;
    }
    let doc_block = doc_lines.join("\n");
    if !super::docstring_tier_rules::has_traceability_anchor(&doc_block) {
        return false;
    }
    let is_module_doc = doc_lines
        .iter()
        .any(|line| line.trim_start().starts_with("//!"));
    let effective_tier_a = is_tier_a
        || is_module_doc
        || super::docstring_tier_rules::has_explicit_tier_a_marker(doc_lines);
    if effective_tier_a {
        (has_all_seven_sections(&doc_block) && has_non_empty_summary_line(doc_lines))
            || has_valid_four_block_contract(doc_lines)
    } else {
        doc_lines.len() <= super::docstring_tier_rules::MAX_TIER_BC_DOC_LINES
            && has_non_empty_summary_line(doc_lines)
    }
}

/// Checks whether `doc_block` contains all seven canonical `DocstringSection::ALL` headings.
/// Realises: [REQ-CHECK-36-HAS-ALL-SEVEN-SECTIONS]
pub(crate) fn has_all_seven_sections(doc_block: &str) -> bool {
    DocstringSection::ALL
        .iter()
        .all(|section| doc_block.contains(section.header()))
}

/// Strips a single leading `///` or `//!` doc-comment prefix and surrounding whitespace from `line`.
/// Realises: [REQ-CHECK-36-STRIP-DOC-COMMENT-MARKER]
pub(crate) fn strip_doc_comment_marker(line: &str) -> &str {
    let trimmed = line.trim();
    trimmed
        .strip_prefix("///")
        .or_else(|| trimmed.strip_prefix("//!"))
        .unwrap_or(trimmed)
        .trim()
}
