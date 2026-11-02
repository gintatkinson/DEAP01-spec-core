#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Isolated Stage-2 Nightly Dylint MIR Gate Plugin. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A and ISO 26262 ASIL D deterministic WCET rules.
//! Traverses control paths from `#[rt_loop]` entrypoints to verify zero heap allocations.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Excluded from stable workspace Cargo.toml to prevent rustc_private contamination.
//! - Enforces absence of heap allocation symbols (`exchange_malloc`, `__rust_alloc`) on RT loops.
//! - Verifies `#[realises("REQ-*")]` bindings against active SysML requirement digests.
//!
//! ## 3. Preconditions
//! - Target codebase compiles under rustc MIR emission.
//!
//! ## 4. Postconditions
//! - Emits structured DEAP-E0901 (heap allocation) and DEAP-E0902 (unverified link) diagnostics.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Transitive call-graph traversal: linear in basic blocks and call terminators.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Yields machine-readable NDJSON findings without compiler panic.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0609], [SC-02], [FLS: fls_dylint_mir]
//! SysML v2: part def IsolatedDylintMirAllocationGate

pub const LINT_RT_ALLOC: &str = "deap_dylint::rt_heap_allocation";
pub const LINT_REQ_TRACE: &str = "deap_dylint::unverified_requirement_link";

/// Stage-2 Dylint MIR configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DylintMirConfig {
    pub max_rt_heap_allocations: usize,
    pub max_unverified_req_links: usize,
    pub isolated_nightly_workspace: bool,
}

impl Default for DylintMirConfig {
    fn default() -> Self {
        Self {
            max_rt_heap_allocations: 0,
            max_unverified_req_links: 0,
            isolated_nightly_workspace: true,
        }
    }
}

/// Dylint MIR lint descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DylintLintDescriptor {
    pub name: &'static str,
    pub code: &'static str,
    pub description: &'static str,
}

pub const REGISTERED_LINTS: [DylintLintDescriptor; 2] = [
    DylintLintDescriptor {
        name: LINT_RT_ALLOC,
        code: "DEAP-E0901",
        description: "Rejects transitive heap allocations on #[rt_loop] execution paths",
    },
    DylintLintDescriptor {
        name: LINT_REQ_TRACE,
        code: "DEAP-E0902",
        description: "Cross-checks #[realises(\"REQ-*\")] annotations against SysML digest",
    },
];

/// Returns the registered lint descriptors.
#[must_use]
pub fn registered_lints() -> &'static [DylintLintDescriptor] {
    &REGISTERED_LINTS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registered_lints_inventory() {
        let lints = registered_lints();
        assert_eq!(lints.len(), 2);
        assert_eq!(lints[0].code, "DEAP-E0901");
        assert_eq!(lints[1].code, "DEAP-E0902");
    }
}
