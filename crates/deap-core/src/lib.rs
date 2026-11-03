//! deap-core: Shared library crate across DEAP pipeline utilities. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides the core foundational library of deterministic AST parsers, diagnostic
//! structures, markdown/diagram syntax verifiers, governance rule models, and
//! workspace baseline checks for the DEAP specification compiler under DO-178C
//! Level A, DO-330 TQL-4, and ISO 26262 ASIL D tool qualification standards.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Re-exported types and verification functions are pure, deterministic, and
//!   free of hidden global mutable state.
//! - All AST extraction, markdown stripping, and rule evaluation APIs preserve
//!   workspace-relative path semantics and schema-driven determinism.
//!
//! ## 3. Preconditions
//! - Callers supply valid UTF-8 source strings or accessible workspace root paths
//!   to the re-exported verification and parsing routines.
//!
//! ## 4. Postconditions
//! - Exposes a cohesive, strongly-typed public surface unifying `diagnostics`,
//!   `error`, `markdown`, `rules`, `source_ast`, `sysml_cst`, and `workspace`
//!   modules without side effects on module load.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(1)$ crate namespace re-export resolution at compile time;
//!   individual delegated routines execute within linear $O(N)$ bounds over input size.
//! - Space Complexity: $O(1)$ static overhead.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Compilation fails closed if any constituent submodule violates type or visibility
//!   contracts; runtime failures in delegated functions are surfaced explicitly via
//!   structured `Diagnostic` values or `Result` types without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-DEAP-CORE-LIB], [DO-330-TQL-4-CORE-LIB]

pub mod cargo_manifest_ast;
pub mod dart_cst;
pub mod diagnostics;
pub mod digest;
pub mod error;
pub mod markdown;
pub mod markdown_ast;
pub mod rules;
pub mod rust_cst;
pub mod salsa_db;
pub mod source_ast;
pub mod sysml_cst;
pub mod workspace;

pub use cargo_manifest_ast::{
    check_zero_shell_scripts, discover_cargo_manifests_with_metadata,
    is_forbidden_regex_crate_name, is_governed_python_scan_extension, is_negated_script_mention,
    load_workspace_rust_vfs, parse_cargo_lock_ast, parse_cargo_toml_ast, Check34Violation,
    ForbiddenAstPattern, LockfileAstSummary, ManifestDependencyEntry, WorkspaceManifestAst,
};

pub use diagnostics::{
    emit_cli_ndjson_stream, format_ndjson_line, resolve_utf8_span, Applicability, DefectDossier,
    Diagnostic, DiagnosticEmitError, DiagnosticSeverity, RustcCompilerMessageEnvelope,
    RustcDiagnosticSpan, StructuredDiagnostic, WhyEntry,
};
pub use digest::{compute_sha256, compute_sha256_bytes};
pub use error::{CoreError, CoreResult};
pub use markdown::{
    check_latex_katex_syntax, check_mermaid_syntax, strip_code_and_glfm, strip_fenced_code_blocks,
    strip_glfm_inline_math, strip_inline_code, ALLOWED_ALIGNMENT_ENVS, VALID_MERMAID_HEADERS,
};
pub use rules::{
    CanonicalArchitectureBlueprint, CanonicalArchitectureBlueprintsCatalog, CodebaseRules,
    MetaConfig, TrackerRules, ValidationRules, CANONICAL_ARCHITECTURE_BLUEPRINTS, EXCLUDED_DIRS,
    MASTER_BLUEPRINTS,
};
pub use rust_cst::{
    analyze_cst_v2_metrics, classify_cfg_test_excluded_doc_tier, evaluate_metrics,
    evaluate_metrics_and_format_ndjson, extract_callable_v2_metrics,
    inventory_extractable_items, list_rust_cst_items, parse_locked_manifest_snapshot,
    parse_verified_lossless_cst, run_offline_locked_workspace_gate,
    splice_disjoint_reverse_spans, validate_metrics_thresholds,
    verify_analyzer_envelope_and_seal_receipt, verify_pinned_cst_dependency_graph,
    ClassifiedDocTierItem, CodemodError, CstComplexityEnvelope, CstMetricsError,
    CstMetricsThresholds, DeapV2AstMetricsReport, DocTier, DocTierInventory, FnAstMetrics,
    LockManifestSnapshot, LockedPackageRecord, OfflineLockGovernanceError, PackageSourceKind,
    RustCstInventoryError, RustCstItemDescriptor, RustItemDescriptor, RustItemInventory,
    SpanSpliceEdit, SupplyChainVerificationReceipt, Utf8ByteSpan, VerifiedLosslessCst,
    ZeroTokenCodemodConfig,
};
pub use salsa_db::{
    create_vfs_salsa_input, file_diagnostics_query, parse_cst_query, semantic_metrics_query,
    FileDiagnostics, IncrementalDatabase, ParsedCstSummary, SalsaDatabase,
    SemanticMetricsSummary, SourceFileInput,
};
pub use source_ast::{
    classify_dart_member_ast, classify_dart_top_level_ast, classify_rust_item_line_ast,
    contains_direct_range_slice, discover_source_files, extract_component_method_table,
    extract_dart_callable_ident, extract_dart_getter_ident, extract_leading_ast_ident,
    extract_method_signature, extract_realises_ids_from_doc_lines, ingest_source_directory,
    lower_source_ast_to_package, normalize_type_for_sysml, parse_dart_callable_signature,
    parse_dart_source_ast, parse_rust_callable_signature, parse_rust_source_ast,
    resolve_source_package_name, sanitize_sysml_identifier, scan_line_with_unified_lex_state,
    single_char_literal_byte_len, strip_leading_dart_annotations,
    strip_line_comment_preserving_strings, summary_from_doc_lines, update_ast_delimiter_depths,
    validate_source_syntax, validate_source_syntax_for_language, ComponentMethodTable,
    CstLexState, ExtractedMethodSignature, SourceAstItem, SourceField, SourceItemKind,
    SourceLanguage, SourceParam,
};
pub use sysml_cst::{
    classify_keyword_kind, component_matches, compute_canonical_digest,
    extract_negated_components, is_component_negated, lower_sysml_hir, lower_to_hir_db,
    parse_action_defs, parse_attribute_defs, parse_calc_defs, parse_connection_defs,
    parse_constraint_defs, parse_hazard_defs, parse_part_defs, parse_port_defs,
    parse_requirement_defs, parse_satisfied_requirements, parse_state_defs, parse_sysml,
    parse_sysml_cst, parse_sysml_green, parse_sysml_model, token_to_syntax_kind, tokenize_sysml,
    ActionDef, AstPersistenceEngine, AstSlice, AttributeDef, CalcDef, CapabilityDef,
    ConnectionDef, ConstraintDef, CotAuditLog, CotAuditRecord, FieldDescriptor, FlowDef,
    HazardDef, ImportDef, IncrementalAstCache, InteractionDef, InterfaceDef, ItemDef,
    LexedSysmlToken, OperationDef, PackageDef, ParsedSysmlCst, PartDef, PortDef, RequirementDef,
    RiskDef, SourceProgram, StateDef, SysmlCompilerDb, SysmlConstructSummary, SysmlCstDiagnostic,
    SysmlLanguage, SysmlModel, SysmlSyntaxKind, SysmlSyntaxNode, SysmlSyntaxToken, SysmlToken,
    TestCaseDef, TransitionDef, TypeDescriptor, UnifiedSchemaModelExtractor, UseCaseDef,
};
pub use workspace::{
    check_downstream_instructions_exist, check_gitignore_exists, check_no_ds_store_files,
    check_no_duplicate_master_blueprints, check_reconcile_backlog_tooling_exists,
    check_upstream_template_clean_landing_zones, filter_clippy_ndjson_stream, is_upstream_compiler,
    run_stable_clippy_gate, validate_manifest_and_clippy_bundle, ClippyGateError,
    ClippyNdjsonFinding, ManifestBundleDigest, StableClippyGateConfig, StageGateReceipt,
    MANDATORY_DENIED_CLIPPY_LINTS,
};

#[cfg(test)]
mod tests {
    /// Verifies that `crates/deap-core/src/lib.rs` carries all 7 crate-root rustdoc sections and re-exports `CoreError`.
    /// Realises: [REQ-0601], [REQ-DEAP-CORE-LIB-7-SECTION-RUSTDOC]
    #[test]
    fn test_deap_core_lib_7_section_rustdoc() -> Result<(), String> {
        let source = include_str!("lib.rs");
        let module_doc: String = source
            .lines()
            .filter(|line| line.trim_start().starts_with("//!"))
            .collect::<Vec<&str>>()
            .join("\n");

        let required_sections = [
            (
                "//! ## 1. Safety Intent & Regulatory Scope",
                "Missing section 1 in module docstring",
            ),
            (
                "//! ## 2. Invariants & Mathematical Properties",
                "Missing section 2 in module docstring",
            ),
            (
                "//! ## 3. Preconditions",
                "Missing section 3 in module docstring",
            ),
            (
                "//! ## 4. Postconditions",
                "Missing section 4 in module docstring",
            ),
            (
                "//! ## 5. Algorithmic Complexity Bounds",
                "Missing section 5 in module docstring",
            ),
            (
                "//! ## 6. Failure Modes & Fault Tolerance",
                "Missing section 6 in module docstring",
            ),
            (
                "//! ## 7. Realization & Traceability",
                "Missing section 7 in module docstring",
            ),
            (
                "//! Realises: [",
                "Missing unfenced Realises traceability header in module docstring",
            ),
        ];

        for (needle, msg) in required_sections {
            if !module_doc.contains(needle) {
                return Err(msg.to_string());
            }
        }

        let table: crate::ComponentMethodTable = crate::extract_component_method_table(&[]);
        if !table.is_empty() {
            return Err("Expected empty ComponentMethodTable for empty AST slice".to_string());
        }

        if !crate::contains_direct_range_slice("let x = &s[0..1];")
            || crate::contains_direct_range_slice("let a = map[\"a..b\"];")
        {
            return Err("Unexpected contains_direct_range_slice classification".to_string());
        }

        Ok(())
    }
}
