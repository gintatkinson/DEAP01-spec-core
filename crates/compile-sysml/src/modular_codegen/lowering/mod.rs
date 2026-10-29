//! Modular code generation Layer 2 lowering facade -- AST partitioning, outlining, ANF, and DAGs.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Exposes the deterministic Layer 2 lowering pipeline transforming parsed SysML v2 AST packages
//! into verified, acyclic, bounded-complexity module graphs (DO-178C / IEC 61508 / ISO 26262).
//!
//! ## 2. Architectural Layer & Boundaries
//! Layer 2 (Logic & Lowering Facade). Depends solely on Layer 1 IR models and SysML v2 AST types;
//! strictly isolated from Layer 3 source rendering and filesystem I/O.
//!
//! ## 3. Inputs & Preconditions
//! Accepts immutable SysML v2 AST package and definition nodes with validated lowering bounds.
//!
//! ## 4. Outputs & Postconditions
//! Re-exports deterministic module graph partitioners, identifier/type outliners, ANF expression
//! hoisters, and Kahn topological DAG / architectural layer edge verifiers.
//!
//! ## 5. Failure Modes & Error Handling
//! Propagates typed diagnostic errors on cyclic module dependencies, illegal upward layer edges,
//! or structural/cyclomatic complexity bound violations without panicking.
//!
//! ## 6. Determinism & Complexity Bounds
//! Pure, side-effect-free barrel facade preserving deterministic lexicographic/topological order
//! and O(V + E) graph verification time complexity.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]

pub mod dag_verifier;
pub mod expr_anf;
pub mod outliner;
pub mod partitioner;

pub use dag_verifier::{run_kahn_topological_sort, verify_dag_and_layers, verify_layer_edges};
pub use expr_anf::{estimate_rust_fn_cyclomatic_complexity, hoist_constraint_anf};
pub use outliner::{
    map_sysml_type_to_rust, outline_action_def, outline_state_def, to_pascal_ident,
    to_snake_slug,
};
pub use partitioner::lower_package_to_module_graph;
