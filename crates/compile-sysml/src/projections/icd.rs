#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! [DOC-TIER: A] Level 1C Logical Interface Control Document (`ICD_01`, `ICD_02`) Projection (`BP-04` / `#751`).
//! ## 1. Safety Intent & Regulatory Scope
//! Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
//! ## 2. Invariants & Mathematical Properties
//! Contract: Re-exports `compile_logical_icd_artifacts`, `try_compile_logical_icd_artifacts`,
//! `emit_logical_icd_artifacts`, `compile_logical_icd_matrix`, `IcdDocumentPair`, and `SignalDictionaryRow`.
//! ## 3. Preconditions
//! ## 4. Postconditions
//! ## 5. Algorithmic Complexity Bounds
//! ## 6. Failure Modes & Fault Tolerance
//! Bounds & Faults: Operates in O(P^2 + C) time and space; fails closed with `Diagnostic: E0200` on dangling endpoints.
//! ## 7. Realization & Traceability

pub use super::forward::icd_wbs_emitter::{
    compile_logical_icd_artifacts, emit_logical_icd_artifacts, execute_emit_cli_args,
    run_cli_with_emit_flags, try_compile_logical_icd_artifacts, IcdDocumentPair,
    SignalDictionaryRow,
};
pub use super::compile_logical_icd_matrix;
