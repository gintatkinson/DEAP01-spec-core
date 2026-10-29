//! Synthesized SysML v2 modular artifact `lowering::FourLevelLoweringEngineSpec` derived from `lowering/four_level_lowering_engine_spec/mod.rs [Application]`.
//! ## 1. Safety Intent & Regulatory Scope: Isolates `lowering::FourLevelLoweringEngineSpec` within `lowering/four_level_lowering_engine_spec/mod.rs [Application]` under DO-178C and ISO 26262 modularity rules.
//! ## 2. Invariants & Mathematical Properties: Callers supply typed inputs satisfying the lowered contract of `lowering::FourLevelLoweringEngineSpec` (`lowering/four_level_lowering_engine_spec/mod.rs [Application]`).
//! ## 3. Preconditions: Preserves deterministic value semantics for `lowering::FourLevelLoweringEngineSpec` lowered from `lowering/four_level_lowering_engine_spec/mod.rs [Application]`.
//! ## 4. Postconditions: Executes total panic-free operations inside `lowering::FourLevelLoweringEngineSpec` (`lowering/four_level_lowering_engine_spec/mod.rs [Application]`) without traps.
//! ## 5. Algorithmic Complexity Bounds: Bounded complexity evaluation for `lowering::FourLevelLoweringEngineSpec` in `lowering/four_level_lowering_engine_spec/mod.rs [Application]` with zero global state.
//! ## 6. Failure Modes & Fault Tolerance: Verified by modular compilation, AST bounds auditing, and hashing of `lowering::FourLevelLoweringEngineSpec` (`lowering/four_level_lowering_engine_spec/mod.rs [Application]`).
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]
//! Traceable to lowered SysML v2 compilation context `lowering/four_level_lowering_engine_spec/mod.rs [Application]` for `lowering::FourLevelLoweringEngineSpec`.

pub mod model;
pub use model::*;
pub mod invariants;
pub use invariants::*;
pub mod ports;
pub use ports::*;
