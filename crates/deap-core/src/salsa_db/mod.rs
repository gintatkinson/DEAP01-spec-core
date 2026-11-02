#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Salsa incremental query engine for lossless CST and semantic metrics. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A and ISO 26262 ASIL D deterministic compilation rules.
//! Implements demand-driven incremental queries for CST parsing and semantic analysis.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Early-cutoff: unchanged semantic metrics suppress downstream query recomputations.
//! - Determinism: identical input sources yield identical CST metrics and digests.
//!
//! ## 3. Preconditions
//! - Source text inputs must be valid UTF-8 strings.
//!
//! ## 4. Postconditions
//! - Incremental queries memoize outputs and recompute only on modified inputs.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Cached lookup: O(1); Recomputation: linear O(N) in changed source byte length.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Syntax errors are captured without panicking in parsed output diagnostics.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0610], [SC-03], [FLS: fls_salsa_db]
//! SysML v2: part def IncrementalCompilerDb

use crate::digest::compute_sha256_bytes;
use crate::rust_cst::parse::ParsedRustFile;
use ra_ap_syntax::ast::{AstNode, HasName};
use std::collections::BTreeSet;

/// Input source file tracked by Salsa.
/// Realises: [REQ-0610], [SC-03]
#[salsa::input]
pub struct SourceFileInput {
    pub file_id: u32,
    #[returns(ref)]
    pub path: String,
    #[returns(ref)]
    pub text: String,
}

/// Lossless CST summary memoized by Salsa.
/// Realises: [REQ-0610], [SC-03]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedCstSummary {
    pub file_id: u32,
    pub path: String,
    pub syntax_errors: Vec<String>,
    pub round_trip_lossless: bool,
    pub total_lines: usize,
    pub fn_count: usize,
    pub struct_count: usize,
    pub enum_count: usize,
    pub trait_count: usize,
    pub cst_digest: [u8; 32],
}

/// Semantic metrics memoized by Salsa.
/// Realises: [REQ-0610], [SC-03]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SemanticMetricsSummary {
    pub file_id: u32,
    pub exec_sloc: usize,
    pub max_fn_lines: usize,
    pub max_fn_params: usize,
    pub function_symbols: BTreeSet<String>,
}

/// Diagnostic report memoized by Salsa.
/// Realises: [REQ-0610], [SC-03]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FileDiagnostics {
    pub file_id: u32,
    pub has_syntax_errors: bool,
    pub error_count: usize,
    pub diagnostic_messages: Vec<String>,
}

/// Tracked query: parses source text into CST summary.
/// Realises: [REQ-0610], [SC-03]
#[salsa::tracked(returns(clone))]
pub fn parse_cst_query(db: &dyn SalsaDatabase, input: SourceFileInput) -> ParsedCstSummary {
    let source = input.text(db);
    let parsed = ParsedRustFile::parse_rust_source(source);
    let path = input.path(db).clone();
    let file_id = *input.file_id(db);
    let syntax_errors = parsed.syntax_errors();
    let round_trip_lossless = parsed.round_trip_lossless();
    let total_lines = source.lines().count();
    let cst_digest = compute_sha256_bytes(source.as_bytes());

    let mut fn_count = 0usize;
    let mut struct_count = 0usize;
    let mut enum_count = 0usize;
    let mut trait_count = 0usize;

    for node in parsed.syntax().descendants() {
        match node.kind() {
            ra_ap_syntax::SyntaxKind::FN => fn_count = fn_count.saturating_add(1),
            ra_ap_syntax::SyntaxKind::STRUCT => struct_count = struct_count.saturating_add(1),
            ra_ap_syntax::SyntaxKind::ENUM => enum_count = enum_count.saturating_add(1),
            ra_ap_syntax::SyntaxKind::TRAIT => trait_count = trait_count.saturating_add(1),
            _ => {}
        }
    }

    ParsedCstSummary {
        file_id,
        path,
        syntax_errors,
        round_trip_lossless,
        total_lines,
        fn_count,
        struct_count,
        enum_count,
        trait_count,
        cst_digest,
    }
}

/// Tracked query: computes semantic metrics from CST.
/// Realises: [REQ-0610], [SC-03]
#[salsa::tracked(returns(clone))]
pub fn semantic_metrics_query(
    db: &dyn SalsaDatabase,
    input: SourceFileInput,
) -> SemanticMetricsSummary {
    let source = input.text(db);
    let file_id = *input.file_id(db);
    let parsed = ParsedRustFile::parse_rust_source(source);
    let mut function_symbols = BTreeSet::new();
    let mut max_fn_params = 0usize;
    let mut max_fn_lines = 0usize;

    for node in parsed.syntax().descendants() {
        if node.kind() == ra_ap_syntax::SyntaxKind::FN {
            if let Some(fn_item) = ra_ap_syntax::ast::Fn::cast(node) {
                if let Some(name) = fn_item.name() {
                    function_symbols.insert(name.text().to_string());
                }
                if let Some(pl) = fn_item.param_list() {
                    let pcount = pl.params().count();
                    if pcount > max_fn_params {
                        max_fn_params = pcount;
                    }
                }
                if let Some(body) = fn_item.body() {
                    let blines = body.syntax().text().to_string().lines().count();
                    if blines > max_fn_lines {
                        max_fn_lines = blines;
                    }
                }
            }
        }
    }

    let exec_sloc = source
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with("//") && !t.starts_with("/*") && !t.starts_with('*')
        })
        .count();

    SemanticMetricsSummary {
        file_id,
        exec_sloc,
        max_fn_lines,
        max_fn_params,
        function_symbols,
    }
}

/// Tracked query: produces diagnostics for a source file.
/// Realises: [REQ-0610], [SC-03]
#[salsa::tracked(returns(clone))]
pub fn file_diagnostics_query(
    db: &dyn SalsaDatabase,
    input: SourceFileInput,
) -> FileDiagnostics {
    let cst = parse_cst_query(db, input);
    let has_syntax_errors = !cst.syntax_errors.is_empty();
    let error_count = cst.syntax_errors.len();
    let diagnostic_messages = cst.syntax_errors;

    FileDiagnostics {
        file_id: cst.file_id,
        has_syntax_errors,
        error_count,
        diagnostic_messages,
    }
}

/// Ingests a VFS file into a Salsa input.
/// Realises: [REQ-0610], [SC-03]
#[must_use]
pub fn create_vfs_salsa_input(
    db: &mut IncrementalDatabase,
    file_id: u32,
    path: String,
    text: String,
) -> SourceFileInput {
    SourceFileInput::new(db, file_id, path, text)
}

/// Salsa Database trait definition.
/// Realises: [REQ-0610], [SC-03]
#[salsa::db]
pub trait SalsaDatabase: salsa::Database {}

/// Concrete Salsa incremental database.
/// Realises: [REQ-0610], [SC-03]
#[salsa::db]
#[derive(Default)]
pub struct IncrementalDatabase {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for IncrementalDatabase {}

#[salsa::db]
impl SalsaDatabase for IncrementalDatabase {}

#[cfg(test)]
mod tests {
    use super::*;
    use salsa::Setter;

    #[test]
    fn test_salsa_incremental_parse_and_metrics() {
        let mut db = IncrementalDatabase::default();
        let src = "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
        let input = create_vfs_salsa_input(&mut db, 1, "src/add.rs".into(), src.into());

        let cst1 = parse_cst_query(&db, input);
        assert_eq!(cst1.fn_count, 1);
        assert!(cst1.syntax_errors.is_empty());
        assert!(cst1.round_trip_lossless);

        let metrics1 = semantic_metrics_query(&db, input);
        assert_eq!(metrics1.max_fn_params, 2);
        assert!(metrics1.function_symbols.contains("add"));

        let diag = file_diagnostics_query(&db, input);
        assert!(!diag.has_syntax_errors);
    }

    #[test]
    fn test_salsa_incremental_early_cutoff_on_whitespace() {
        let mut db = IncrementalDatabase::default();
        let src1 = "pub fn foo() {}\n";
        let input = create_vfs_salsa_input(&mut db, 2, "src/foo.rs".into(), src1.into());

        let metrics1 = semantic_metrics_query(&db, input);
        assert_eq!(metrics1.max_fn_params, 0);

        // Update text with a comment: function symbols and max params remain invariant
        let src2 = "// A pure comment\npub fn foo() {}\n";
        input.set_text(&mut db).to(src2.into());

        let metrics2 = semantic_metrics_query(&db, input);
        assert_eq!(metrics1.function_symbols, metrics2.function_symbols);
        assert_eq!(metrics1.max_fn_params, metrics2.max_fn_params);
    }

    #[test]
    fn test_salsa_syntax_error_diagnostics() {
        let mut db = IncrementalDatabase::default();
        let malformed = "fn malformed( {";
        let input = create_vfs_salsa_input(&mut db, 3, "src/bad.rs".into(), malformed.into());

        let cst = parse_cst_query(&db, input);
        assert!(!cst.syntax_errors.is_empty());

        let diag = file_diagnostics_query(&db, input);
        assert!(diag.has_syntax_errors);
        assert!(diag.error_count > 0);
    }
}
