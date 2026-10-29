//! Modular code generation subsystem entry point.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides a deterministic, pure schema-driven modular code generation
//! architecture separating intermediate domain representation, AST lowering,
//! and target emitters without hardcoded domain concepts.
//!
//! ## 2. Architectural & Structural Contract
//! Exposes Stage-0 sub-modules (`domain`, `lowering`, `emitters`) enforcing
//! strict unidirectional dependency flow from lowered SysML v2 IR to emitters.
//!
//! ## 3. Preconditions, Postconditions & Invariants
//! - Precondition: Upstream SysML v2 AST nodes pass semantic validation.
//! - Postcondition: Emitted target artifacts are byte-reproducible.
//! - Invariant: Zero side-effecting I/O inside domain and lowering layers.
//!
//! ## 4. Failure Modes & Mitigation
//! Unmapped or malformed AST constructs surface structured diagnostic errors
//! rather than panicking or silently emitting partial artifacts.
//!
//! ## 5. Determinism, Concurrency & Resource Bounds
//! All stage transformations enforce deterministic ordering (`BTreeMap`)
//! with bounded traversal over lowered IR structures.
//!
//! ## 6. Verification & Falsifiability
//! Verified via native Rust `#[test]` unit suites and negative mutation probes
//! across `domain`, `lowering`, and `emitters`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]

pub mod domain;
pub mod emitters;
pub mod generated;
pub mod lowering;

/// Projects self-hosted Stage-1 `ModularityBoundsSpec` thresholds into runtime `ModularityBounds`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Closes the metacircular self-hosting loop by deriving runtime modularity ceilings directly
/// from the compiled `.pipeline/upstream/schema/modular_codegen.sysml` Stage-1 domain contract
/// under DO-178C Level A and DO-330 TQL-4 tool qualification governance.
///
/// ## 2. Invariants & Mathematical Properties
/// Relies on the generated `ModularityBoundsSpec::default()` projection emitted from
/// `ModularCodegen::domain::ModularityBoundsSpec` with positive integer ceiling defaults.
///
/// ## 3. Preconditions
/// Returns a validated `domain::ModularityBounds` value matching the self-hosted schema ceilings,
/// falling back field-by-field to `domain::ModularityBounds::canonical()` on signed conversion failure.
///
/// ## 4. Postconditions
/// Total function with zero panics, traps, or unchecked casts (`usize::try_from` + `unwrap_or`).
///
/// ## 5. Algorithmic Complexity Bounds
/// Executes in constant `O(1)` time (`v(G) = 1`) and `O(1)` stack space with zero heap allocations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by `self_hosting_bootstrap` asserting Stage-1 `ModularityBoundsSpec` parity and
/// `SHA-256(Stage 1) == SHA-256(Stage 2)` fixed-point equivalence.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
#[must_use]
pub fn self_hosted_modularity_bounds() -> domain::ModularityBounds {
    let spec = generated::domain::modularity_bounds_spec::model::ModularityBoundsSpec::default();
    let canonical = domain::ModularityBounds::canonical();
    domain::ModularityBounds {
        max_file_lines: usize::try_from(spec.max_file_lines).unwrap_or(canonical.max_file_lines),
        max_items_per_file: usize::try_from(spec.max_items_per_file)
            .unwrap_or(canonical.max_items_per_file),
        max_cyclomatic_complexity: usize::try_from(spec.max_cyclomatic_complexity)
            .unwrap_or(canonical.max_cyclomatic_complexity),
        max_action_steps_per_stage: usize::try_from(spec.max_action_steps_per_stage)
            .unwrap_or(canonical.max_action_steps_per_stage),
        max_transitions_per_handler: usize::try_from(spec.max_transitions_per_handler)
            .unwrap_or(canonical.max_transitions_per_handler),
    }
}
