#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Bidirectional Markdown Synchronization Engine.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical
//! software engineering standards. Coordinates forward (`SysML v2 -> CommonMark`) and reverse
//! (`CommonMark -> SysML v2`) synchronization passes under deterministic AST parity gates.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Re-exports the complete public synchronization interface across `forward` and `reverse`.
//! - Preserves roundtrip AST conservation and SHA-256 digest determinism across sync cycles.
//!
//! ## 3. Preconditions
//! - Invoked with parsed `PackageDef` models and validated workspace synchronization options.
//!
//! ## 4. Postconditions
//! - Exposes `ForwardSyncOptions`, `ReverseSyncOptions`, forward/reverse sync entry points, and ICD/WBS emitters.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(1)$ module routing overhead.
//! - Space Complexity: $O(1)$ static namespace re-exports.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Propagates typed `Result<_, String>` diagnostics from underlying forward and reverse engines.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0604]

pub mod forward;
pub mod reverse;

pub use forward::*;
pub use reverse::*;
