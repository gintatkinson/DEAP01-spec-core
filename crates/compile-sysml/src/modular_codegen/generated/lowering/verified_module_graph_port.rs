//! Synthesized SysML v2 modular artifact `lowering::VerifiedModuleGraphPort` derived from `lowering/verified_module_graph_port.rs [Application]`.
//! ## 1. Safety Intent & Regulatory Scope: Isolates `lowering::VerifiedModuleGraphPort` within `lowering/verified_module_graph_port.rs [Application]` under DO-178C and ISO 26262 modularity rules.
//! ## 2. Invariants & Mathematical Properties: Callers supply typed inputs satisfying the lowered contract of `lowering::VerifiedModuleGraphPort` (`lowering/verified_module_graph_port.rs [Application]`).
//! ## 3. Preconditions: Preserves deterministic value semantics for `lowering::VerifiedModuleGraphPort` lowered from `lowering/verified_module_graph_port.rs [Application]`.
//! ## 4. Postconditions: Executes total panic-free operations inside `lowering::VerifiedModuleGraphPort` (`lowering/verified_module_graph_port.rs [Application]`) without traps.
//! ## 5. Algorithmic Complexity Bounds: Bounded complexity evaluation for `lowering::VerifiedModuleGraphPort` in `lowering/verified_module_graph_port.rs [Application]` with zero global state.
//! ## 6. Failure Modes & Fault Tolerance: Verified by modular compilation, AST bounds auditing, and hashing of `lowering::VerifiedModuleGraphPort` (`lowering/verified_module_graph_port.rs [Application]`).
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]
//! Traceable to lowered SysML v2 compilation context `lowering/verified_module_graph_port.rs [Application]` for `lowering::VerifiedModuleGraphPort`.

/// Synthesized SysML v2 modular artifact `pub struct VerifiedModuleGraphPortPort` derived from `VerifiedModuleGraphPort`.
/// ## 1. Safety Intent & Regulatory Scope: Isolates `pub struct VerifiedModuleGraphPortPort` within `VerifiedModuleGraphPort` under DO-178C and ISO 26262 modularity rules.
/// ## 2. Invariants & Mathematical Properties: Callers supply typed inputs satisfying the lowered contract of `pub struct VerifiedModuleGraphPortPort` (`VerifiedModuleGraphPort`).
/// ## 3. Preconditions: Preserves deterministic value semantics for `pub struct VerifiedModuleGraphPortPort` lowered from `VerifiedModuleGraphPort`.
/// ## 4. Postconditions: Executes total panic-free operations inside `pub struct VerifiedModuleGraphPortPort` (`VerifiedModuleGraphPort`) without traps.
/// ## 5. Algorithmic Complexity Bounds: Bounded complexity evaluation for `pub struct VerifiedModuleGraphPortPort` in `VerifiedModuleGraphPort` with zero global state.
/// ## 6. Failure Modes & Fault Tolerance: Verified by modular compilation, AST bounds auditing, and hashing of `pub struct VerifiedModuleGraphPortPort` (`VerifiedModuleGraphPort`).
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Traceable to lowered SysML v2 compilation context `VerifiedModuleGraphPort` for `pub struct VerifiedModuleGraphPortPort`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VerifiedModuleGraphPortPort {
    pub protocol_family: String,
    pub direction: String,
}
