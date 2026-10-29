//! Synthesized SysML v2 modular artifact `domain::CompilationUnitSpec` derived from `domain/compilation_unit_spec/mod.rs [Domain]`.
//! ## 1. Safety Intent & Regulatory Scope: Isolates `domain::CompilationUnitSpec` within `domain/compilation_unit_spec/mod.rs [Domain]` under DO-178C and ISO 26262 modularity rules.
//! ## 2. Invariants & Mathematical Properties: Callers supply typed inputs satisfying the lowered contract of `domain::CompilationUnitSpec` (`domain/compilation_unit_spec/mod.rs [Domain]`).
//! ## 3. Preconditions: Preserves deterministic value semantics for `domain::CompilationUnitSpec` lowered from `domain/compilation_unit_spec/mod.rs [Domain]`.
//! ## 4. Postconditions: Executes total panic-free operations inside `domain::CompilationUnitSpec` (`domain/compilation_unit_spec/mod.rs [Domain]`) without traps.
//! ## 5. Algorithmic Complexity Bounds: Bounded complexity evaluation for `domain::CompilationUnitSpec` in `domain/compilation_unit_spec/mod.rs [Domain]` with zero global state.
//! ## 6. Failure Modes & Fault Tolerance: Verified by modular compilation, AST bounds auditing, and hashing of `domain::CompilationUnitSpec` (`domain/compilation_unit_spec/mod.rs [Domain]`).
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]
//! Traceable to lowered SysML v2 compilation context `domain/compilation_unit_spec/mod.rs [Domain]` for `domain::CompilationUnitSpec`.

pub mod model;
pub use model::*;
pub mod invariants;
pub use invariants::*;
