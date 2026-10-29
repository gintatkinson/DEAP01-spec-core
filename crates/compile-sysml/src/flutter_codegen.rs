//! Deterministic Flutter and C-ABI Contract Generator Barrel Facade.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and MIL-STD-1472H standards.
//! Re-exports the canonical modular Flutter contract emitter API surface for backward compatibility.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Pure Schema-Driven Synthesis: 100% domain-agnostic; all entity definitions, attributes,
//!   and ports are derived dynamically from SysML v2 AST models and topology datasets.
//! - Bijective API Parity: Preserves zero-drift symbol parity with `modular_codegen::emitters::flutter`.
//!
//! ## 3. Preconditions
//! - Callers invoke re-exported contract generators with semantically validated `SysmlModel` inputs.
//!
//! ## 4. Postconditions
//! - Delegates deterministically to `crate::modular_codegen::emitters::flutter` implementations.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(1)$ compile-time module re-export resolution; runtime complexity
//!   matches the delegated linear $O(V + E + P)$ emitters.
//! - Space Complexity: $O(1)$ facade overhead; $O(V + E + P)$ bounded emitter buffers at runtime.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Propagates strongly-typed `FlutterCodegenError` diagnostics fail-closed without partial writes.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-COMPILER-FLUTTER-CODEGEN], [REQ-EMIT-FLUTTER-CONTRACTS], [REQ-0607]

pub use crate::modular_codegen::emitters::flutter::{
    compute_schema_hash, emit_dart_solver_bridge, emit_flutter_contracts,
    emit_modular_flutter_contracts, escape_dart_single_quoted, extract_bounds_from_doc,
    format_display_name, generate_domain_seed_dart, generate_schema_models_dart,
    generate_solver_bridge_dart, generate_ui_contract_test_dart, parse_topology_json,
    split_ident_into_words, to_camel_case, to_pascal_case, to_snake_case_dart,
    FlutterCodegenError, FlutterContractOptions, FlutterContractReport, TopologyNode,
};
