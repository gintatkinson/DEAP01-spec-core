//! Synthesized SysML v2 modular artifact `domain::CompilationUnitSpec::invariants` derived from `domain/compilation_unit_spec/invariants.rs [Domain]`.
//! ## 1. Safety Intent & Regulatory Scope: Isolates `domain::CompilationUnitSpec::invariants` within `domain/compilation_unit_spec/invariants.rs [Domain]` under DO-178C and ISO 26262 modularity rules.
//! ## 2. Invariants & Mathematical Properties: Callers supply typed inputs satisfying the lowered contract of `domain::CompilationUnitSpec::invariants` (`domain/compilation_unit_spec/invariants.rs [Domain]`).
//! ## 3. Preconditions: Preserves deterministic value semantics for `domain::CompilationUnitSpec::invariants` lowered from `domain/compilation_unit_spec/invariants.rs [Domain]`.
//! ## 4. Postconditions: Executes total panic-free operations inside `domain::CompilationUnitSpec::invariants` (`domain/compilation_unit_spec/invariants.rs [Domain]`) without traps.
//! ## 5. Algorithmic Complexity Bounds: Bounded complexity evaluation for `domain::CompilationUnitSpec::invariants` in `domain/compilation_unit_spec/invariants.rs [Domain]` with zero global state.
//! ## 6. Failure Modes & Fault Tolerance: Verified by modular compilation, AST bounds auditing, and hashing of `domain::CompilationUnitSpec::invariants` (`domain/compilation_unit_spec/invariants.rs [Domain]`).
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]
//! Traceable to lowered SysML v2 compilation context `domain/compilation_unit_spec/invariants.rs [Domain]` for `domain::CompilationUnitSpec::invariants`.

use super::model::CompilationUnitSpec;
/// Synthesized SysML v2 modular artifact `pub fn validate_unit_obeys_bounds(ctx: &CompilationUnitSpec, max_file_lines: i64, max_items_per_file: i64, max_cyclomatic_complexity: i64) -> bool` derived from `domain::CompilationUnitSpec::invariants`.
/// ## 1. Safety Intent & Regulatory Scope: Isolates `pub fn validate_unit_obeys_bounds(ctx: &CompilationUnitSpec, max_file_lines: i64, max_items_per_file: i64, max_cyclomatic_complexity: i64) -> bool` within `domain::CompilationUnitSpec::invariants` under DO-178C and ISO 26262 modularity rules.
/// ## 2. Invariants & Mathematical Properties: Callers supply typed inputs satisfying the lowered contract of `pub fn validate_unit_obeys_bounds(ctx: &CompilationUnitSpec, max_file_lines: i64, max_items_per_file: i64, max_cyclomatic_complexity: i64) -> bool` (`domain::CompilationUnitSpec::invariants`).
/// ## 3. Preconditions: Preserves deterministic value semantics for `pub fn validate_unit_obeys_bounds(ctx: &CompilationUnitSpec, max_file_lines: i64, max_items_per_file: i64, max_cyclomatic_complexity: i64) -> bool` lowered from `domain::CompilationUnitSpec::invariants`.
/// ## 4. Postconditions: Executes total panic-free operations inside `pub fn validate_unit_obeys_bounds(ctx: &CompilationUnitSpec, max_file_lines: i64, max_items_per_file: i64, max_cyclomatic_complexity: i64) -> bool` (`domain::CompilationUnitSpec::invariants`) without traps.
/// ## 5. Algorithmic Complexity Bounds: Bounded complexity evaluation for `pub fn validate_unit_obeys_bounds(ctx: &CompilationUnitSpec, max_file_lines: i64, max_items_per_file: i64, max_cyclomatic_complexity: i64) -> bool` in `domain::CompilationUnitSpec::invariants` with zero global state.
/// ## 6. Failure Modes & Fault Tolerance: Verified by modular compilation, AST bounds auditing, and hashing of `pub fn validate_unit_obeys_bounds(ctx: &CompilationUnitSpec, max_file_lines: i64, max_items_per_file: i64, max_cyclomatic_complexity: i64) -> bool` (`domain::CompilationUnitSpec::invariants`).
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Traceable to lowered SysML v2 compilation context `domain::CompilationUnitSpec::invariants` for `pub fn validate_unit_obeys_bounds(ctx: &CompilationUnitSpec, max_file_lines: i64, max_items_per_file: i64, max_cyclomatic_complexity: i64) -> bool`.
pub fn validate_unit_obeys_bounds(ctx: &CompilationUnitSpec, max_file_lines: i64, max_items_per_file: i64, max_cyclomatic_complexity: i64) -> bool {
    (1..=max_file_lines).contains(&ctx.line_count) && (1..=max_items_per_file).contains(&ctx.item_count) && (1..=max_cyclomatic_complexity).contains(&ctx.max_function_vg)
}
