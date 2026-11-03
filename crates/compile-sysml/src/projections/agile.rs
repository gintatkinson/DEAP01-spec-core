#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! [DOC-TIER: A] Agile Specification Projection Facade (`EPIC`, `FEAT`, `UC`, `US`) (`BP-04` / `#751`).
//! ## 1. Safety Intent & Regulatory Scope
//! Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
//! ## 2. Invariants & Mathematical Properties
//! Contract: Re-exports deterministic Agile specification projection entry points (`forward_sync_sysml_to_specs`,
//! `ForwardSyncOptions`, `StoryMeta`, `project_agile_spec_suite`) and `pulldown_cmark` task-list merge helpers.
//! ## 3. Preconditions
//! ## 4. Postconditions
//! ## 5. Algorithmic Complexity Bounds
//! ## 6. Failure Modes & Fault Tolerance
//! Bounds & Faults: Operates in O(N log N) time and O(N) space; fails closed via `CompilerError`.
//! ## 7. Realization & Traceability

pub use super::forward::{
    collect_package_story_numbers, count_total_epics_inclusive, forward_sync_sysml_to_specs,
    resolve_parent_epic_link, ForwardSyncOptions, StoryMeta,
};
pub use super::{project_agile_spec_suite, SpecSuiteBundle};
