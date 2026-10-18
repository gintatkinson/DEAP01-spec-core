//! deap-core: Shared library crate across DEAP pipeline utilities.
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
//!   `markdown`, `rules`, `sysml_ast`, and `workspace` modules without side effects
//!   on module load.
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

pub mod diagnostics;
pub mod markdown;
pub mod rules;
pub mod source_ast;
pub mod sysml_ast;
pub mod workspace;

pub use diagnostics::{DefectDossier, Diagnostic, DiagnosticSeverity, WhyEntry};
pub use markdown::{
    check_latex_katex_syntax, check_mermaid_syntax, strip_code_and_glfm, strip_fenced_code_blocks,
    strip_glfm_inline_math, strip_inline_code, ALLOWED_ALIGNMENT_ENVS, VALID_MERMAID_HEADERS,
};
pub use rules::{
    CodebaseRules, MetaConfig, TrackerRules, ValidationRules, EXCLUDED_DIRS, MASTER_BLUEPRINTS,
};
pub use source_ast::{
    classify_dart_member_ast, classify_dart_top_level_ast, classify_rust_item_line_ast,
    discover_source_files, extract_dart_callable_ident, extract_leading_ast_ident,
    extract_realises_ids_from_doc_lines, ingest_source_directory, lower_source_ast_to_package,
    normalize_type_for_sysml, parse_dart_callable_signature, parse_dart_source_ast,
    parse_rust_callable_signature, parse_rust_source_ast, sanitize_sysml_identifier,
    summary_from_doc_lines, update_ast_delimiter_depths, validate_source_syntax,
    validate_source_syntax_for_language, SourceAstItem, SourceField, SourceItemKind,
    SourceLanguage, SourceParam,
};
pub use sysml_ast::{
    component_matches, extract_negated_components, is_component_negated,
    parse_action_defs, parse_attribute_defs, parse_calc_defs, parse_connection_defs, parse_constraint_defs,
    parse_part_defs, parse_port_defs, parse_requirement_defs, parse_satisfied_requirements, parse_state_defs, parse_sysml,
    ActionDef, AttributeDef, CalcDef, CapabilityDef, ConnectionDef, ConstraintDef, FlowDef, HazardDef,
    ImportDef, InteractionDef, ItemDef, OperationDef, PackageDef, PartDef, PortDef, RequirementDef, RiskDef,
    StateDef, SysmlModel, TestCaseDef, TransitionDef, UnifiedSchemaModelExtractor, UseCaseDef,
};
pub use workspace::{
    check_downstream_instructions_exist, check_gitignore_exists, check_no_ds_store_files,
    check_no_duplicate_master_blueprints, check_reconcile_backlog_tooling_exists,
    check_upstream_template_clean_landing_zones, is_upstream_compiler,
};

#[cfg(test)]
mod tests {
    #[test]
    fn test_deap_core_lib_7_section_rustdoc() {
        let source = include_str!("lib.rs");
        let module_doc: String = source
            .lines()
            .filter(|line| line.trim_start().starts_with("//!"))
            .collect::<Vec<&str>>()
            .join("\n");

        assert!(
            module_doc.contains("//! ## 1. Safety Intent & Regulatory Scope"),
            "Missing section 1 in module docstring"
        );
        assert!(
            module_doc.contains("//! ## 2. Invariants & Mathematical Properties"),
            "Missing section 2 in module docstring"
        );
        assert!(
            module_doc.contains("//! ## 3. Preconditions"),
            "Missing section 3 in module docstring"
        );
        assert!(
            module_doc.contains("//! ## 4. Postconditions"),
            "Missing section 4 in module docstring"
        );
        assert!(
            module_doc.contains("//! ## 5. Algorithmic Complexity Bounds"),
            "Missing section 5 in module docstring"
        );
        assert!(
            module_doc.contains("//! ## 6. Failure Modes & Fault Tolerance"),
            "Missing section 6 in module docstring"
        );
        assert!(
            module_doc.contains("//! ## 7. Realization & Traceability"),
            "Missing section 7 in module docstring"
        );
        assert!(
            module_doc.contains("//! Realises: ["),
            "Missing unfenced Realises traceability header in module docstring"
        );
    }
}

