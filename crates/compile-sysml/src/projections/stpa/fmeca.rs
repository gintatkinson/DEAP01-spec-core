#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! FMECA Criticality Matrix Core Types, Universal Failure Dimensions, and Templates.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! safety objectives for module `crates/compile-sysml/src/stpa/fmeca.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Defines the 4 universal failure dimensions (`Interface`, `State`, `Action`, `Resource`),
//!   `FmecaRow`, `FmecaScoringConfig`, `RegistryFailureEntry`, and domain-neutral MBSE templates.
//! - Re-exports all registry parsers and FMECA criticality matrix synthesis entrypoints.
//!
//! ## 3. Preconditions
//! - Caller supplies well-formed AST `PackageDef` models and optional scoring/registry records.
//!
//! ## 4. Postconditions
//! - Preserves 100% of public FMECA types and synthesis entrypoints across `crate::stpa::fmeca`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(1) for variant projection and template lookup; O(L) for label parsing.
//! - Space Complexity: O(1) static `.rodata` storage for dimension templates.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Rejects unrecognized dimension labels cleanly via `Option::None`.
//! - State: Pure immutable type and constant definitions.
//! - Action: Executes deterministically with zero panics or `.unwrap()`.
//! - Resource: Zero unbounded heap allocations.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-FMECA-FAILUREDIMENSION] | SysML v2: PackageDef(stpa_fmeca)

pub mod members;
pub mod registry;
pub mod synthesis;
#[cfg(test)]
mod tests;

pub use registry::*;
pub use synthesis::*;

use serde::{Deserialize, Serialize};

/// The 4 universal failure dimensions across structural and behavioral system components.
/// Realises: [REQ-STPA-FMECA-FAILUREDIMENSION]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureDimension {
    Interface,
    State,
    Action,
    Resource,
}

impl FailureDimension {
    /// Projects `FailureDimension` onto its canonical static string label.
    /// Realises: [REQ-STPA-FMECA-FAILUREDIMENSION-AS-STR]
    pub fn as_str(&self) -> &'static str {
        match self {
            FailureDimension::Interface => "Interface",
            FailureDimension::State => "State",
            FailureDimension::Action => "Action",
            FailureDimension::Resource => "Resource",
        }
    }

    /// Parses a Markdown registry dimension token into one of the four universal failure dimensions.
    /// Realises: [REQ-STPA-FMECA-FAILUREDIMENSION-FROM-LABEL]
    pub fn from_label(raw: &str) -> Option<Self> {
        let lower = raw.trim().to_ascii_lowercase();
        if lower.contains("interface") {
            Some(FailureDimension::Interface)
        } else if lower.contains("state") {
            Some(FailureDimension::State)
        } else if lower.contains("action") {
            Some(FailureDimension::Action)
        } else if lower.contains("resource") {
            Some(FailureDimension::Resource)
        } else {
            None
        }
    }
}

/// A single row in the FMECA matrix.
/// Realises: [REQ-STPA-FMECA-FMECAROW]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FmecaRow {
    pub id: String,
    pub component: String,
    pub dimension: String,
    pub failure_mode: String,
    pub effect: String,
    pub severity: u32,
    pub occurrence: u32,
    pub detection: u32,
    pub rpn: u32,
    pub severity_cell: String,
    pub occurrence_cell: String,
    pub detection_cell: String,
    pub rpn_cell: String,
    pub mitigation: String,
    pub basis: String,
}

/// Generic scoring scale entry from optional configuration.
/// Realises: [REQ-STPA-FMECA-SCORINGSCALEENTRY]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringScaleEntry {
    pub label: String,
    pub score: u32,
}

/// Generic FMECA scoring configuration.
/// Realises: [REQ-STPA-FMECA-FMECASCORINGCONFIG]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FmecaScoringConfig {
    pub severity_scale: Vec<ScoringScaleEntry>,
    pub occurrence_scale: Vec<ScoringScaleEntry>,
    pub detection_scale: Vec<ScoringScaleEntry>,
}

/// Researched component failure mode record parsed from `FAILURE_MODE_REGISTRY.md`.
/// Realises: [REQ-STPA-FMECA-REGISTRY-ENTRY]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryFailureEntry {
    pub component: String,
    pub dimension: FailureDimension,
    pub failure_mode: String,
    pub effect: String,
    pub severity: u32,
    pub occurrence: u32,
    pub detection: u32,
    pub mitigation: String,
    pub is_ssot_evidenced: bool,
}

/// Domain-agnostic abstract MBSE contract failure template per universal dimension.
/// Realises: [REQ-STPA-FMECA-DIMENSIONTEMPLATE]
#[derive(Clone, Copy)]
pub(crate) struct DimensionTemplate {
    pub(crate) dimension: FailureDimension,
    pub(crate) mode_suffix: &'static str,
    pub(crate) effect_template: &'static str,
    pub(crate) mitigation_template: &'static str,
    pub(crate) default_s: u32,
    pub(crate) default_o: u32,
    pub(crate) default_d: u32,
}

pub(crate) const INTERFACE_DIMENSION_TEMPLATE: DimensionTemplate = DimensionTemplate {
    dimension: FailureDimension::Interface,
    mode_suffix: "Interface Contract Violation / Payload Malformation",
    effect_template: "Loss of inter-component contract exchange or rejected boundary message",
    mitigation_template: "Enforce boundary schema contract validation and deterministic rejection",
    default_s: 8,
    default_o: 3,
    default_d: 2,
};

pub(crate) const STATE_DIMENSION_TEMPLATE: DimensionTemplate = DimensionTemplate {
    dimension: FailureDimension::State,
    mode_suffix: "State Invariant Divergence / Transition Deadlock",
    effect_template: "Component fails to satisfy target lifecycle state invariant",
    mitigation_template:
        "Enforce guarded state transition preconditions and deterministic recovery transition",
    default_s: 7,
    default_o: 3,
    default_d: 2,
};

pub(crate) const ACTION_DIMENSION_TEMPLATE: DimensionTemplate = DimensionTemplate {
    dimension: FailureDimension::Action,
    mode_suffix: "Behavioral Execution Fault / Postcondition Violation",
    effect_template: "Incomplete or non-terminating behavioral transformation",
    mitigation_template:
        "Enforce precondition guards, bounded execution steps, and postcondition assertions",
    default_s: 8,
    default_o: 2,
    default_d: 3,
};

pub(crate) const RESOURCE_DIMENSION_TEMPLATE: DimensionTemplate = DimensionTemplate {
    dimension: FailureDimension::Resource,
    mode_suffix: "Resource Capacity Exhaustion / Attribute Bound Breach",
    effect_template: "Degraded structural capacity or rejected allocation under peak load",
    mitigation_template:
        "Enforce explicit capacity bounds, quota checks, and admission backpressure",
    default_s: 6,
    default_o: 3,
    default_d: 2,
};

pub(crate) const DIMENSION_TEMPLATES: [DimensionTemplate; 4] = [
    INTERFACE_DIMENSION_TEMPLATE,
    STATE_DIMENSION_TEMPLATE,
    ACTION_DIMENSION_TEMPLATE,
    RESOURCE_DIMENSION_TEMPLATE,
];

pub(crate) const ABSTRACT_STRUCTURAL_SUB_MODES: [DimensionTemplate; 4] = [
    DimensionTemplate {
        dimension: FailureDimension::Interface,
        mode_suffix: "Boundary Protocol Desynchronization / Contract Mismatch",
        effect_template: "Boundary interaction sequence diverges from declared port contract",
        mitigation_template:
            "Enforce strict interface contract validation and ordering preconditions",
        default_s: 8,
        default_o: 3,
        default_d: 2,
    },
    DimensionTemplate {
        dimension: FailureDimension::Action,
        mode_suffix: "Behavioral Pipeline Stall / Termination Bound Breach",
        effect_template: "Behavioral step execution exceeds bounded step budget without completion",
        mitigation_template: "Enforce deterministic step bounds and fail-safe abort postconditions",
        default_s: 8,
        default_o: 2,
        default_d: 3,
    },
    DimensionTemplate {
        dimension: FailureDimension::State,
        mode_suffix: "Lifecycle State Invariant Drift",
        effect_template: "Internal state representation violates declared structural constraint",
        mitigation_template:
            "Enforce invariant assertions prior to state commit and atomic rollback",
        default_s: 7,
        default_o: 3,
        default_d: 2,
    },
    DimensionTemplate {
        dimension: FailureDimension::Resource,
        mode_suffix: "Capacity Quota Saturation / Allocation Contention",
        effect_template: "Concurrent resource demand exceeds declared structural attribute limit",
        mitigation_template:
            "Enforce bounded allocation quotas and deterministic admission backpressure",
        default_s: 6,
        default_o: 3,
        default_d: 2,
    },
];

/// Looks up the static `DimensionTemplate` corresponding to a `FailureDimension`.
/// Realises: [REQ-STPA-FMECA-TEMPLATE-FOR-DIMENSION]
pub(crate) fn template_for_dimension(dimension: FailureDimension) -> &'static DimensionTemplate {
    match dimension {
        FailureDimension::Interface => &INTERFACE_DIMENSION_TEMPLATE,
        FailureDimension::State => &STATE_DIMENSION_TEMPLATE,
        FailureDimension::Action => &ACTION_DIMENSION_TEMPLATE,
        FailureDimension::Resource => &RESOURCE_DIMENSION_TEMPLATE,
    }
}
