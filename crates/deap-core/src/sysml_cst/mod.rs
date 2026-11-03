//! # OMG SysML v2.0 & KerML 1.0 Concrete Syntax Tree (`logos` + `rowan` + `salsa`)
//! [DOC-TIER: A]
//!
//! ## 1. Module Description
//! Provides a 100% lossless Concrete Syntax Tree (CST) and incremental Salsa database
//! for OMG SysML v2.0 and KerML 1.0 textual notation (`BP-03`, `BP-10`).
//!
//! ## 2. Contract and Invariants
//! - **Lossless Trivia Preservation**: Whitespace, line comments, and block comments
//!   are retained in the Rowan green/red tree so `syntax.text().to_string() == source` holds.
//! - **6-Clause Grammar Coverage**: Supports Clauses 1--3 (`parser_clauses_1_3`) and
//!   Clauses 4--6 (`parser_clauses_4_6`), HIR lowering (`hir_lower`), lossless printing and
//!   disjoint span splicing (`printer`), bijective digests and `ariadne` diagnostics (`digest`),
//!   and incremental `salsa` memoization (`salsa_db`).
//!
//! ## 3. Normative References
//! - OMG Systems Modeling Language (SysML) v2.0 Specification
//! - OMG Kernel Modeling Language (KerML) v1.0 Specification
//!
//! ## 4. Safety and Security Considerations
//! - Total, panic-free lexical scanning and parsing bounded by `MAX_CST_PARSE_DEPTH` (`64`).
//!
//! ## 5. Verification and Testing
//! - Round-trip byte-for-byte identity tests, 6-clause construct recognition tests,
//!   disjoint span splicer tests, and Salsa early-cutoff invariance tests.
//!
//! ## 6. Performance Profile
//! - $O(N)$ linear-time DFA lexing (`logos`) and single-pass Rowan `GreenNodeBuilder` assembly.
//!
//! ## 7. Traceability
//! - Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]

#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]

mod actions;
mod connections_and_flows;
pub mod digest;
mod expressions;
pub mod hir_lower;
mod identifiers;
mod interactions;
mod interfaces_and_items;
pub mod lexer;
mod model_parser;
mod package_model;
pub mod parser;
pub mod parser_clauses_1_3;
pub mod parser_clauses_4_6;
mod parts_and_ports;
mod persistence;
pub mod printer;
mod requirements;
mod safety_and_fmeca;
pub mod salsa_db;
mod schema_extractor;
mod slice;
mod states;
pub mod syntax_kind;

pub use actions::{parse_action_defs, ActionDef};
pub use connections_and_flows::{parse_connection_defs, parse_port_defs, ConnectionDef};
pub use digest::{
    compute_bijective_digest, normalize_line_endings, render_ariadne_diagnostic,
    sha256_hex_digest, AriadneSpanReport, BijectiveDigestReceipt,
};
pub use expressions::{parse_calc_defs, parse_constraint_defs, CalcDef, ConstraintDef};
pub use hir_lower::{
    extract_cst_import_path, extract_cst_type_ref, lower_cst_to_hir, DefId, HirMemberEntry,
    HirPackageSummary, HirSourceMap,
};
pub use identifiers::{
    find_matching_brace, is_valid_subsystem_identifier, mask_string_literals, AttributeDef, FlowDef,
};
pub use interactions::{parse_test_case_defs, InteractionDef, TestCaseDef, UseCaseDef};
pub use interfaces_and_items::{parse_attribute_defs, parse_part_defs, InterfaceDef};
pub use lexer::{tokenize_sysml, LexedSysmlToken, SysmlToken};
pub use model_parser::{lower_to_hir_db, parse_sysml, parse_sysml_model, strip_sysml_comments};
pub use package_model::{AstSlice, ImportDef, PackageDef, SysmlModel};
pub use parser::{
    classify_keyword_kind, parse_sysml_cst, parse_sysml_green, ParsedSysmlCst, SysmlClause8Kind,
    SysmlConstructSummary, SysmlCstDiagnostic,
};
pub use parser_clauses_1_3::{SysmlCstBuilder, MAX_CST_PARSE_DEPTH};
pub use parts_and_ports::{
    CapabilityDef, FieldDescriptor, ItemDef, OperationDef, PartDef, PortDef,
    SupportedExtensionsCatalog, SupportedSchemaExtension, TypeDescriptor,
};
pub use persistence::{AstPersistenceEngine, CotAuditLog, CotAuditRecord, IncrementalAstCache};
pub use printer::{
    apply_formatting_preserving_splices, format_canonical_hir_summary, print_lossless_cst,
    validate_disjoint_splice_ranges, CstSplicePatch,
};
pub(in crate::sysml_cst) use requirements::parse_safety_and_layer_annotations;
pub use requirements::{parse_requirement_defs, parse_satisfied_requirements, RequirementDef};
pub(in crate::sysml_cst) use safety_and_fmeca::{
    component_matches_negated_stems, extract_negated_key_stem,
};
pub use safety_and_fmeca::{
    component_matches, extract_negated_components, is_component_negated, parse_hazard_defs,
    HazardDef, RiskDef,
};
pub use salsa_db::{
    compute_canonical_digest, evaluate_hir_early_cutoff, is_standard_library_package,
    lower_sysml_hir, resolve_standard_library_symbol, SalsaSysmlSource, SourceProgram,
    SysmlCompilerDb, SysmlHirPackage, SysmlSalsaStorage, tracked_lower_sysml_hir,
    STANDARD_LIBRARY_PACKAGES,
};
pub use schema_extractor::UnifiedSchemaModelExtractor;
pub use states::{parse_state_defs, StateDef, TransitionDef};
pub use syntax_kind::{
    token_to_syntax_kind, SysmlLanguage, SysmlSyntaxElement, SysmlSyntaxKind, SysmlSyntaxNode,
    SysmlSyntaxToken,
};

#[cfg(test)]
mod sysml_cst_model_tests;
#[cfg(test)]
mod sysml_cst_schema_tests;

#[cfg(test)]
mod tests {
    use super::*;

    const NORMATIVE_SYSML_SNIPPET: &str = r#"// Header line comment
library package VehicleSafety {
    doc /* ISO 26262 ASIL-D Vehicle Safety Package */
    public import ScalarValues::*;
    alias Mass for Real;

    metadata def SafetyCritical;
    attribute def Voltage :> Real;
    enum def GearState { enum Park; enum Drive; }
    item def BrakeFluid;

    abstract part def BrakingSubsystem :> BaseSubsystem {
        attribute nominalVoltage : Voltage [1..1] = 12.0;
        port brakeIn : ~BrakeCommandPort;
        state def BrakeMode {
            entry action initBrakes;
            transition first Off accept EngageCmd if nominalVoltage > 9.0 do applyPressure then Active;
        }
    }

    part primaryBrakes : BrakingSubsystem :>> subsystemRef [1..2] {
        satisfy EmergencyStopReq by primaryBrakes;
    }

    port def BrakeCommandPort;
    connection def BrakeLink;
    interface def BrakeBusInterface;
    flow def PressureFlow;
    allocation def HardwareAllocation;
    occurrence def BrakingEvent;
    action def ApplyBrakes;
    calc def ComputeStoppingDistance;
    constraint def MaxStoppingDistanceConstraint;
    requirement def EmergencyStopReq;
    concern def PassengerSafetyConcern;
    use case def EmergencyBrakingUseCase;
    analysis def StoppingMarginAnalysis;
    verification def VerifyStoppingDistance;
    view def SafetyArchitectureView;
    viewpoint def SafetyAssuranceViewpoint;
    rendering def BlockDiagramRendering;
    variation part def BrakeActuatorVariation;
}
"#;

    #[test]
    fn test_lossless_roundtrip_and_hierarchical_cst() {
        let parsed = parse_sysml_cst(NORMATIVE_SYSML_SNIPPET);
        assert_eq!(parsed.syntax().text().to_string(), NORMATIVE_SYSML_SNIPPET);
        assert!(parsed.diagnostics.is_empty());
        assert!(parsed.syntax().children().count() > 0);
        assert!(!parsed.nodes_of_kind(SysmlSyntaxKind::LibraryPackageDecl).is_empty());
        assert!(!parsed.nodes_of_kind(SysmlSyntaxKind::PartDef).is_empty());
        assert!(!parsed.nodes_of_kind(SysmlSyntaxKind::StateDef).is_empty());
        assert!(!parsed.nodes_of_kind(SysmlSyntaxKind::TransitionDecl).is_empty());
    }

    #[test]
    fn test_salsa_db_early_cutoff_and_stdlib_resolution() {
        let mut db = SysmlCompilerDb::new();
        let prog1 = SourceProgram::new("vehicle.sysml", NORMATIVE_SYSML_SNIPPET);
        let hir1 = db.lower_to_hir(&prog1);
        assert_eq!(hir1.package_name.as_deref(), Some("VehicleSafety"));
        let rev1 = db.revision();
        let hir2 = db.lower_to_hir(&prog1);
        assert_eq!(rev1, db.revision());
        assert_eq!(hir1.hir_structural_hash, hir2.hir_structural_hash);

        let prog_comment = SourceProgram::new(
            "vehicle.sysml",
            format!("// Extra trivia comment\n{NORMATIVE_SYSML_SNIPPET}\n"),
        );
        let hir3 = db.lower_to_hir(&prog_comment);
        assert_eq!(hir1.hir_structural_hash, hir3.hir_structural_hash);
        assert!(is_standard_library_package("ISQ::*"));
        assert_eq!(resolve_standard_library_symbol("Real"), Some("ScalarValues::Real"));
    }
}
