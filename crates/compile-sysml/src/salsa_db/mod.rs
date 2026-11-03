#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Salsa 0.26 demand-driven incremental compiler database for `compile-sysml` (`BP-10`).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for demand-driven incremental SysML v2 compilation (`BP-03` & `BP-10`).
//! ## 2. Invariants & Mathematical Properties
//! Unifies `source_db` (`SalsaFileInput`, `DurabilityTier`, `discover_sysml_files`), `cst_db` (`parse_lossless_sysml_cst`, `Scanner`, `ExpressionParser`), `hir_db` (`lower_hir_tracked`, `SysmlParser`, `SymbolTable`), and `safety_db` (`validate_safety_tracked`, `SemanticValidator`, `SchemaDigest`, `SysmlSerializable`).
//! ## 3. Preconditions
//! Source strings are valid UTF-8 and AST/CST depth bounds (`<= 64`) hold across all inputs.
//! ## 4. Postconditions
//! Exposes a single cohesive `crate::salsa_db` surface replacing the 43 legacy `lexer`, `parser`, and `semantic` shards.
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(N) on cache miss; O(1) on Salsa memoized early cutoff. Space Complexity: O(N).
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Zero panics; all syntax and semantic errors are returned as typed diagnostics.
//! ## 7. Realization & Traceability
//! Realises: [REQ-COMPILER-SALSA-DB-001] | [FLS: fls_66m4rnbssgig]

pub mod cst_db;
pub mod cst_expr;
pub mod cst_scanner;
pub mod hir_db;
mod hir_parser_behavioral;
mod hir_parser_connections;
pub mod hir_parser_core;
mod hir_parser_cursor;
mod hir_parser_hazards;
mod hir_parser_parts;
mod hir_parser_safety;
mod hir_parser_structural;
mod hir_parser_transitions;
mod hir_parser_use_cases;
pub mod hir_symbols;
pub mod safety_db;
pub mod safety_digest;
pub mod safety_serializer;
mod safety_serializer_members;
pub mod safety_types;
pub mod source_db;

pub use cst_db::{
    parse_lossless_sysml_cst, resolve_polyglot_backend, CstParseReceipt, Span, Token, TokenKind,
};
pub use cst_expr::{parse_expression, BinaryOp, Expr, ExpressionParser, UnaryOp};
pub use cst_scanner::Scanner;
pub use hir_db::{
    evaluate_hir_early_cutoff, is_standard_library_package, is_synthetic_forest_root,
    lower_hir_tracked, merge_package_defs, parse_sysml_directory, parse_sysml_tree,
    resolve_standard_library_symbol, STANDARD_LIBRARY_PACKAGES,
};
pub use hir_parser_core::{ParseError, SysmlParser};
pub use hir_symbols::{Symbol, SymbolKind, SymbolTable, MAX_AST_DEPTH};
pub use safety_db::{
    apply_formatting_preserving_splices, compute_bijective_digest, render_ariadne_diagnostic,
    validate_disjoint_splice_ranges, validate_safety_tracked, AriadneSpanReport,
    BijectiveDigestReceipt, CstSplicePatch, DiagnosticSeverity, SemanticDiagnostic,
    SemanticValidator,
};
pub use safety_digest::{
    compute_sha256, count_structural_elements, generate_digest, FileDigestEntry,
    OperationalActivity, OperationalExchange, OperationalNode, OperationalScenario, SchemaDigest,
};
pub use safety_serializer::{format_doc_comment, to_sysml, SysmlSerializable};
pub use safety_types::{
    is_primitive_or_builtin_type, normalize_source_type, split_top_level_type_args,
};
pub use source_db::{
    compute_file_manifest, discover_sysml_files, normalize_line_endings, write_atomic,
    write_digest_atomic, DurabilityTier, FileId, SalsaFileInput,
};

/// Salsa 0.26 incremental compiler database for `compile-sysml`.
/// Realises: [REQ-COMPILER-SALSA-DB-001] | [FLS: fls_66m4rnbssgig]
#[salsa::db]
#[derive(Default, Clone)]
pub struct SysmlIncrementalDb {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for SysmlIncrementalDb {}

#[cfg(test)]
mod tests {
    use super::*;
    use deap_core::sysml_cst::{hir_lower::lower_cst_to_hir, parser::parse_sysml_cst};

    /// Verifies tracked CST parsing, HIR lowering, early-cutoff comment invariance, and safety validation on `SysmlIncrementalDb`.
    /// Realises: [REQ-COMPILER-SALSA-DB-001] | [FLS: fls_66m4rnbssgig]
    #[test]
    fn test_salsa_incremental_pipeline_and_early_cutoff() {
        let db = SysmlIncrementalDb::default();
        let src_v1 = "package FlightCtrl {\n    part def Actuator {\n        attribute gain : Real = 1.0;\n    }\n}\n";
        let src_v2 = "// Added header comment\npackage FlightCtrl {\n    part def Actuator {\n        attribute gain : Real = 1.0;\n    }\n}\n";

        let input_v1 = SalsaFileInput::new(
            &db,
            "schema/flight.sysml".to_string(),
            src_v1.to_string(),
            DurabilityTier::LowUserWorkspace,
        );
        let input_v2 = SalsaFileInput::new(
            &db,
            "schema/flight.sysml".to_string(),
            src_v2.to_string(),
            DurabilityTier::LowUserWorkspace,
        );

        let cst_v1 = parse_lossless_sysml_cst(&db, input_v1);
        assert!(cst_v1.lossless_roundtrip);
        assert!(cst_v1.diagnostics.is_empty());

        let hir_v1 = lower_hir_tracked(&db, input_v1);
        let hir_v2 = lower_hir_tracked(&db, input_v2);
        assert_eq!(hir_v1, hir_v2, "Trivia-only edit must trigger HIR early cutoff");

        let (summary_v1, _) = lower_cst_to_hir(&parse_sysml_cst(src_v1));
        let (summary_v2, _) = lower_cst_to_hir(&parse_sysml_cst(src_v2));
        assert!(evaluate_hir_early_cutoff(&summary_v1, &summary_v2));

        let diags = validate_safety_tracked(&db, input_v1);
        assert!(diags.is_empty(), "Expected 0 safety diagnostics, got: {diags:?}");
    }

    /// Verifies stdlib symbol resolution, CST formatting-preserving splice patching, and `ariadne` diagnostic rendering.
    /// Realises: [REQ-COMPILER-SALSA-DB-002] | [FLS: fls_66m4rnbssgig]
    #[test]
    fn test_stdlib_splice_and_ariadne_diagnostics() {
        assert!(is_standard_library_package("ISQ"));
        assert!(resolve_standard_library_symbol("ScalarValues::Real").is_some());

        let original = "package P {\n    part def A;\n}\n";
        let patches = vec![CstSplicePatch {
            byte_range: 25..26,
            replacement: "B".to_string(),
        }];
        let spliced = apply_formatting_preserving_splices(original, &patches);
        assert_eq!(spliced.as_deref(), Ok("package P {\n    part def B;\n}\n"));

        let report = render_ariadne_diagnostic(
            "schema/p.sysml",
            original,
            "E001",
            "Unresolved type 'Bogus'",
            16..26,
        );
        assert_eq!(report.code, "E001");
        assert!(report.rendered.contains("Unresolved type 'Bogus'"));
    }
}
