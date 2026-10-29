//! Synthesized SysML v2 modular artifact `domain::LayerHierarchySpec::invariants` derived from `domain/layer_hierarchy_spec/invariants.rs [Domain]`.
//! ## 1. Safety Intent & Regulatory Scope: Isolates `domain::LayerHierarchySpec::invariants` within `domain/layer_hierarchy_spec/invariants.rs [Domain]` under DO-178C and ISO 26262 modularity rules.
//! ## 2. Invariants & Mathematical Properties: Callers supply typed inputs satisfying the lowered contract of `domain::LayerHierarchySpec::invariants` (`domain/layer_hierarchy_spec/invariants.rs [Domain]`).
//! ## 3. Preconditions: Preserves deterministic value semantics for `domain::LayerHierarchySpec::invariants` lowered from `domain/layer_hierarchy_spec/invariants.rs [Domain]`.
//! ## 4. Postconditions: Executes total panic-free operations inside `domain::LayerHierarchySpec::invariants` (`domain/layer_hierarchy_spec/invariants.rs [Domain]`) without traps.
//! ## 5. Algorithmic Complexity Bounds: Bounded complexity evaluation for `domain::LayerHierarchySpec::invariants` in `domain/layer_hierarchy_spec/invariants.rs [Domain]` with zero global state.
//! ## 6. Failure Modes & Fault Tolerance: Verified by modular compilation, AST bounds auditing, and hashing of `domain::LayerHierarchySpec::invariants` (`domain/layer_hierarchy_spec/invariants.rs [Domain]`).
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]
//! Traceable to lowered SysML v2 compilation context `domain/layer_hierarchy_spec/invariants.rs [Domain]` for `domain::LayerHierarchySpec::invariants`.

use super::model::LayerHierarchySpec;
/// Synthesized SysML v2 modular artifact `pub fn validate_non_upward_layer_import(_ctx: &LayerHierarchySpec, importer_tier: i64, imported_tier: i64) -> bool` derived from `domain::LayerHierarchySpec::invariants`.
/// ## 1. Safety Intent & Regulatory Scope: Isolates `pub fn validate_non_upward_layer_import(_ctx: &LayerHierarchySpec, importer_tier: i64, imported_tier: i64) -> bool` within `domain::LayerHierarchySpec::invariants` under DO-178C and ISO 26262 modularity rules.
/// ## 2. Invariants & Mathematical Properties: Callers supply typed inputs satisfying the lowered contract of `pub fn validate_non_upward_layer_import(_ctx: &LayerHierarchySpec, importer_tier: i64, imported_tier: i64) -> bool` (`domain::LayerHierarchySpec::invariants`).
/// ## 3. Preconditions: Preserves deterministic value semantics for `pub fn validate_non_upward_layer_import(_ctx: &LayerHierarchySpec, importer_tier: i64, imported_tier: i64) -> bool` lowered from `domain::LayerHierarchySpec::invariants`.
/// ## 4. Postconditions: Executes total panic-free operations inside `pub fn validate_non_upward_layer_import(_ctx: &LayerHierarchySpec, importer_tier: i64, imported_tier: i64) -> bool` (`domain::LayerHierarchySpec::invariants`) without traps.
/// ## 5. Algorithmic Complexity Bounds: Bounded complexity evaluation for `pub fn validate_non_upward_layer_import(_ctx: &LayerHierarchySpec, importer_tier: i64, imported_tier: i64) -> bool` in `domain::LayerHierarchySpec::invariants` with zero global state.
/// ## 6. Failure Modes & Fault Tolerance: Verified by modular compilation, AST bounds auditing, and hashing of `pub fn validate_non_upward_layer_import(_ctx: &LayerHierarchySpec, importer_tier: i64, imported_tier: i64) -> bool` (`domain::LayerHierarchySpec::invariants`).
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Traceable to lowered SysML v2 compilation context `domain::LayerHierarchySpec::invariants` for `pub fn validate_non_upward_layer_import(_ctx: &LayerHierarchySpec, importer_tier: i64, imported_tier: i64) -> bool`.
pub fn validate_non_upward_layer_import(_ctx: &LayerHierarchySpec, importer_tier: i64, imported_tier: i64) -> bool {
    (1..=3).contains(&importer_tier) && (1..=3).contains(&imported_tier) && importer_tier >= imported_tier
}
