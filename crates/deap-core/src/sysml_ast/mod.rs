//! AST structures for SysML v2 entities, requirements, and system architecture.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides the canonical Abstract Syntax Tree (AST) data structures, recursive parsers, and
//! deterministic architecture extractors for SysML v2 models in accordance with DO-178C Level A,
//! DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification and traceability mandates.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Pure Schema-Driven Extraction: All AST representations derive exclusively and deterministically
//!   from SysML v2 structural, behavioral, and requirement declarations without hardcoded domain state.
//! - Deterministic Ordering: Multi-file directory traversal and map/set aggregations preserve
//!   canonical lexicographical ordering via `BTreeMap` and `BTreeSet`.
//!
//! ## 3. Preconditions
//! - Input SysML v2 source strings and workspace schema paths must be valid UTF-8 text or readable
//!   filesystem directories containing `.sysml` specification units.
//!
//! ## 4. Postconditions
//! - Returns strongly typed, serializable AST node collections (`SysmlModel`, `PartDef`,
//!   `RequirementDef`, `PortDef`, `ActionDef`, `StateDef`, `UseCaseDef`, `TestCaseDef`) preserving
//!   exact block hierarchy and requirement satisfaction bindings.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear in the byte length of parsed SysML v2 source text and number of
//!   AST tokens.
//! - Space Complexity: $O(V + E)$ proportional to the extracted AST vertices and directed edges.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Unbalanced braces, malformed declarations, or unreadable paths are handled gracefully via
//!   bounded character-stream scanning and explicit `Result` error propagation without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYSML-AST-MODULE], [DO-330-TQL-4-SYSML-AST]

mod actions;
mod cache;
mod connections_and_flows;
mod cot_audit;
mod expressions;
mod identifiers;
mod interactions;
mod interfaces_and_items;
mod package_model;
mod parser;
mod parts_and_ports;
mod persistence;
mod requirements;
mod safety_and_fmeca;
mod schema_extractor;
mod schema_extractor_data;
mod slice;
mod states;
mod test_cases;

pub use actions::{parse_action_defs, ActionDef};
pub use cache::IncrementalAstCache;
pub use connections_and_flows::{parse_connection_defs, parse_port_defs, ConnectionDef};
pub use cot_audit::{CotAuditLog, CotAuditRecord};
pub use expressions::{parse_calc_defs, parse_constraint_defs, CalcDef, ConstraintDef};
pub use identifiers::{
    find_matching_brace, is_valid_subsystem_identifier, mask_string_literals, AttributeDef, FlowDef,
};
pub use interactions::{InteractionDef, UseCaseDef};
pub use interfaces_and_items::{parse_attribute_defs, parse_part_defs, InterfaceDef};
pub use package_model::{AstSlice, ImportDef, PackageDef, SysmlModel};
pub use parser::{lower_to_hir_db, parse_sysml, parse_sysml_model, strip_sysml_comments};
pub use parts_and_ports::{
    CapabilityDef, FieldDescriptor, ItemDef, OperationDef, PartDef, PortDef,
    SupportedExtensionsCatalog, SupportedSchemaExtension, TypeDescriptor,
};
pub use persistence::AstPersistenceEngine;
pub(in crate::sysml_ast) use requirements::parse_safety_and_layer_annotations;
pub use requirements::{parse_requirement_defs, parse_satisfied_requirements, RequirementDef};
pub(in crate::sysml_ast) use safety_and_fmeca::{
    component_matches_negated_stems, extract_negated_key_stem,
};
pub use safety_and_fmeca::{
    component_matches, extract_negated_components, is_component_negated, parse_hazard_defs,
    HazardDef, RiskDef,
};
pub use schema_extractor::UnifiedSchemaModelExtractor;
pub use states::{parse_state_defs, StateDef, TransitionDef};
pub use test_cases::{parse_test_case_defs, TestCaseDef};

#[cfg(test)]
mod sysml_ast_schema_tests;
#[cfg(test)]
mod sysml_ast_tests;
#[cfg(test)]
mod sysml_ast_unit_tests;
