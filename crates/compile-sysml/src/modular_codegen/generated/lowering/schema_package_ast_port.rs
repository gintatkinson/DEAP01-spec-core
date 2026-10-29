//! Synthesized SysML v2 modular artifact `lowering::SchemaPackageAstPort` derived from `lowering/schema_package_ast_port.rs [Application]`.
//! ## 1. Safety Intent & Regulatory Scope: Isolates `lowering::SchemaPackageAstPort` within `lowering/schema_package_ast_port.rs [Application]` under DO-178C and ISO 26262 modularity rules.
//! ## 2. Invariants & Mathematical Properties: Callers supply typed inputs satisfying the lowered contract of `lowering::SchemaPackageAstPort` (`lowering/schema_package_ast_port.rs [Application]`).
//! ## 3. Preconditions: Preserves deterministic value semantics for `lowering::SchemaPackageAstPort` lowered from `lowering/schema_package_ast_port.rs [Application]`.
//! ## 4. Postconditions: Executes total panic-free operations inside `lowering::SchemaPackageAstPort` (`lowering/schema_package_ast_port.rs [Application]`) without traps.
//! ## 5. Algorithmic Complexity Bounds: Bounded complexity evaluation for `lowering::SchemaPackageAstPort` in `lowering/schema_package_ast_port.rs [Application]` with zero global state.
//! ## 6. Failure Modes & Fault Tolerance: Verified by modular compilation, AST bounds auditing, and hashing of `lowering::SchemaPackageAstPort` (`lowering/schema_package_ast_port.rs [Application]`).
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]
//! Traceable to lowered SysML v2 compilation context `lowering/schema_package_ast_port.rs [Application]` for `lowering::SchemaPackageAstPort`.

/// Synthesized SysML v2 modular artifact `pub struct SchemaPackageAstPortPort` derived from `SchemaPackageAstPort`.
/// ## 1. Safety Intent & Regulatory Scope: Isolates `pub struct SchemaPackageAstPortPort` within `SchemaPackageAstPort` under DO-178C and ISO 26262 modularity rules.
/// ## 2. Invariants & Mathematical Properties: Callers supply typed inputs satisfying the lowered contract of `pub struct SchemaPackageAstPortPort` (`SchemaPackageAstPort`).
/// ## 3. Preconditions: Preserves deterministic value semantics for `pub struct SchemaPackageAstPortPort` lowered from `SchemaPackageAstPort`.
/// ## 4. Postconditions: Executes total panic-free operations inside `pub struct SchemaPackageAstPortPort` (`SchemaPackageAstPort`) without traps.
/// ## 5. Algorithmic Complexity Bounds: Bounded complexity evaluation for `pub struct SchemaPackageAstPortPort` in `SchemaPackageAstPort` with zero global state.
/// ## 6. Failure Modes & Fault Tolerance: Verified by modular compilation, AST bounds auditing, and hashing of `pub struct SchemaPackageAstPortPort` (`SchemaPackageAstPort`).
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Traceable to lowered SysML v2 compilation context `SchemaPackageAstPort` for `pub struct SchemaPackageAstPortPort`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SchemaPackageAstPortPort {
    pub protocol_family: String,
    pub direction: String,
}
