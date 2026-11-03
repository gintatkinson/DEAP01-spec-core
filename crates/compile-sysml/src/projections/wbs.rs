#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! [DOC-TIER: A] MIL-STD-881E Work Breakdown Structure (`WBS_01..03`) Projection (`BP-04` / `#751`).
//! ## 1. Safety Intent & Regulatory Scope
//! Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
//! ## 2. Invariants & Mathematical Properties
//! Contract: Re-exports `compile_wbs_artifacts`, `try_compile_wbs_artifacts`, `emit_wbs_artifacts`,
//! `synthesize_mil_std_881e_wbs`, `WbsExportBundle`, and `WbsNode`.
//! ## 3. Preconditions
//! ## 4. Postconditions
//! ## 5. Algorithmic Complexity Bounds
//! ## 6. Failure Modes & Fault Tolerance
//! Bounds & Faults: Operates in O(W log W) time and O(W) space; rejects cyclic dependencies and orphan requirements.
//! ## 7. Realization & Traceability

pub use super::forward::wbs_emitter::{
    compile_wbs_artifacts, emit_wbs_artifacts, try_compile_wbs_artifacts, WbsExportBundle, WbsNode,
};
pub use super::synthesize_mil_std_881e_wbs;
