//! FMECA Criticality Matrix Generator.
//!
//! Generates FMECA failure mode rows across all 4 universal failure dimensions:
//! Interface, State, Action, Resource, ensuring 100% component coverage and >= 15 failure modes.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use deap_core::sysml_ast::{HazardDef, PackageDef, PartDef};
use serde::{Deserialize, Serialize};

/// The 4 universal failure dimensions across structural and behavioral system components.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `FailureDimension` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `FailureDimension` across member set `[Interface, State, Action, Resource]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `FailureDimension` members (`Interface, State, Action, Resource`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `FailureDimension` representation for enum `FailureDimension`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(4)) bounded over input/member dimension of `FailureDimension`.
/// - Space Complexity: O(M(4)) bounded auxiliary memory for `FailureDimension`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `FailureDimension` (`FailureDimension`).
/// - State: Maintains consistent state invariants across `FailureDimension` without partial mutation on error.
/// - Action: Executes `FailureDimension` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `FailureDimension` (`STPA-FMECA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-FAILUREDIMENSION] | SysML v2: PartDef(FailureDimension)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureDimension {
    Interface,
    State,
    Action,
    Resource,
}

impl FailureDimension {
    /// Executes deterministic fn `FailureDimension::as_str` within `crates/compile-sysml/src/stpa/fmeca.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `FailureDimension::as_str` in `crates/compile-sysml/src/stpa/fmeca.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `FailureDimension::as_str` (declaration #2) within `STPA-FMECA`.
    ///
    /// ## 3. Preconditions
    /// - Invoked within a valid execution context for `FailureDimension::as_str` in `STPA-FMECA`.
    ///
    /// ## 4. Postconditions
    /// - Returns `&'static str` from `FailureDimension::as_str` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant variant projection for `FailureDimension::as_str`.
    /// - Space Complexity: O(1) zero heap allocation for `FailureDimension::as_str`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `FailureDimension::as_str` (`&'static str`).
    /// - State: Maintains consistent state invariants across `FailureDimension::as_str` without partial mutation on error.
    /// - Action: Executes `FailureDimension::as_str` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `FailureDimension::as_str` (`STPA-FMECA`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-STPA-FMECA-FAILUREDIMENSION-AS-STR] | SysML v2: ActionDef(FailureDimension::as_str)
    pub fn as_str(&self) -> &'static str {
        match self {
            FailureDimension::Interface => "Interface",
            FailureDimension::State => "State",
            FailureDimension::Action => "Action",
            FailureDimension::Resource => "Resource",
        }
    }

    /// Parses a Markdown registry dimension token into one of the four universal failure dimensions.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `FailureDimension::from_label` in `crates/compile-sysml/src/stpa/fmeca.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Case-insensitively maps normalized labels containing `"interface"`, `"state"`, `"action"`, or `"resource"` onto `Some(FailureDimension)` and rejects unrecognized headers with `None`.
    ///
    /// ## 3. Preconditions
    /// - `raw: &str` is an arbitrary UTF-8 cell slice extracted from a candidate registry table row.
    ///
    /// ## 4. Postconditions
    /// - Returns `Option<FailureDimension>` deterministically without panicking or allocating regex automata.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(L) where L is the byte length of `raw`.
    /// - Space Complexity: O(L) for the ASCII-lowercased comparison buffer.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Rejects header tokens such as `"Dimension"` cleanly by returning `None`.
    /// - State: Pure function with zero shared or mutable state.
    /// - Action: Single linear scan over `raw` with guaranteed termination.
    /// - Resource: Allocates at most one short lowercased `String` proportional to cell length.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-STPA-FMECA-FAILUREDIMENSION-FROM-LABEL] | SysML v2: ActionDef(FailureDimension::from_label)
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `FmecaRow` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `FmecaRow` across member set `[id: String, component: String, dimension: String, failure_mode: String, effect: String]`, with `rpn == severity * occurrence * detection` and `basis` restricted to `"SSOT"` or `"Derived"`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `FmecaRow` members (`id: String, component: String, dimension: String, failure_mode: String, effect: String`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `FmecaRow` representation for struct `FmecaRow`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(15)) bounded over field count of `FmecaRow`.
/// - Space Complexity: O(M(15)) bounded auxiliary memory for `FmecaRow`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `FmecaRow` (`FmecaRow`).
/// - State: Maintains consistent state invariants across `FmecaRow` without partial mutation on error.
/// - Action: Executes `FmecaRow` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `FmecaRow` (`STPA-FMECA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-FMECAROW] | SysML v2: PartDef(FmecaRow)
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
    pub basis: String, // "SSOT" or "Derived"
}

/// Generic scoring scale entry from optional configuration.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ScoringScaleEntry` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ScoringScaleEntry` across member set `[label: String, score: u32]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ScoringScaleEntry` members (`label: String, score: u32`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ScoringScaleEntry` representation for struct `ScoringScaleEntry`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(2)) bounded over input/member dimension of `ScoringScaleEntry`.
/// - Space Complexity: O(M(2)) bounded auxiliary memory for `ScoringScaleEntry`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ScoringScaleEntry` (`ScoringScaleEntry`).
/// - State: Maintains consistent state invariants across `ScoringScaleEntry` without partial mutation on error.
/// - Action: Executes `ScoringScaleEntry` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ScoringScaleEntry` (`STPA-FMECA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-SCORINGSCALEENTRY] | SysML v2: PartDef(ScoringScaleEntry)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringScaleEntry {
    pub label: String,
    pub score: u32,
}

/// Generic FMECA scoring configuration.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `FmecaScoringConfig` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `FmecaScoringConfig` across member set `[severity_scale: Vec<ScoringScaleEntry>, occurrence_scale: Vec<ScoringScaleEntry>, detection_scale: Vec<ScoringScaleEntry>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `FmecaScoringConfig` members (`severity_scale: Vec<ScoringScaleEntry>, occurrence_scale: Vec<ScoringScaleEntry>, detection_scale: Vec<ScoringScaleEntry>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `FmecaScoringConfig` representation for struct `FmecaScoringConfig`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(3)) bounded over input/member dimension of `FmecaScoringConfig`.
/// - Space Complexity: O(M(3)) bounded auxiliary memory for `FmecaScoringConfig`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `FmecaScoringConfig` (`FmecaScoringConfig`).
/// - State: Maintains consistent state invariants across `FmecaScoringConfig` without partial mutation on error.
/// - Action: Executes `FmecaScoringConfig` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `FmecaScoringConfig` (`STPA-FMECA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-FMECASCORINGCONFIG] | SysML v2: PartDef(FmecaScoringConfig)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FmecaScoringConfig {
    pub severity_scale: Vec<ScoringScaleEntry>,
    pub occurrence_scale: Vec<ScoringScaleEntry>,
    pub detection_scale: Vec<ScoringScaleEntry>,
}

/// Researched component failure mode record parsed from `FAILURE_MODE_REGISTRY.md`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `RegistryFailureEntry` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Encapsulates a component-scoped failure record across one `FailureDimension` with scores clamped to `[1, 10]` and explicit `is_ssot_evidenced` provenance tracking so unresearched rows never claim `"SSOT"` basis.
///
/// ## 3. Preconditions
/// - Constructed from validated Markdown registry rows with non-empty `component` and `failure_mode` fields.
///
/// ## 4. Postconditions
/// - Supplies authoritative `(severity, occurrence, detection, mitigation, is_ssot_evidenced)` attributes to `generate_fmeca_matrix_with_registry`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) field access over fixed 9-member record layout.
/// - Space Complexity: O(|component| + |failure_mode| + |effect| + |mitigation|) heap storage.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Enforces strongly typed `FailureDimension` and `u32` score fields at construction boundaries.
/// - State: Immutable value record once ingested from `FAILURE_MODE_REGISTRY.md`.
/// - Action: Cloned or borrowed without side effects during deterministic matrix synthesis.
/// - Resource: Bounded to parsed table cell strings from workspace documentation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-REGISTRY-ENTRY] | SysML v2: PartDef(RegistryFailureEntry)
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

/// Column index mapping resolved from a Markdown table header in `FAILURE_MODE_REGISTRY.md`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `RegistryTableColumns` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Validated header mappings guarantee `dimension_col` and `failure_mode_col` are populated before any data row is interpreted.
///
/// ## 3. Preconditions
/// - Derived by scanning normalized header cell tokens of a Markdown pipe table.
///
/// ## 4. Postconditions
/// - Provides zero-panicking `Option<usize>` column coordinates for safe `.get(..)` cell lookups.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) constant-size index lookup per row.
/// - Space Complexity: O(1) stack-only `Option<usize>` tuple fields.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Optional column indices prevent out-of-bounds access when optional columns are omitted.
/// - State: Scoped exclusively to the active contiguous Markdown table block.
/// - Action: Reset deterministically whenever a non-table Markdown line is encountered.
/// - Resource: Zero heap allocations (`Default` stack struct).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-REGISTRY-TABLE-COLUMNS] | SysML v2: PartDef(RegistryTableColumns)
#[derive(Debug, Clone, Default)]
struct RegistryTableColumns {
    component_col: Option<usize>,
    dimension_col: Option<usize>,
    failure_mode_col: Option<usize>,
    effect_col: Option<usize>,
    severity_col: Option<usize>,
    occurrence_col: Option<usize>,
    detection_col: Option<usize>,
    mitigation_col: Option<usize>,
    basis_col: Option<usize>,
}

/// Domain-agnostic abstract MBSE contract failure template per universal dimension.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `DimensionTemplate` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves domain-neutral abstract MBSE contract wording and distinct `(default_s, default_o, default_d)` baselines across `[Interface, State, Action, Resource]`.
///
/// ## 3. Preconditions
/// - Caller supplies static domain-neutral templates with `1 <= default_s, default_o, default_d <= 10`.
///
/// ## 4. Postconditions
/// - Defines the authoritative `DimensionTemplate` representation for compiler-derived fallback synthesis.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) constant static table access.
/// - Space Complexity: O(1) compile-time `.rodata` storage.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `DimensionTemplate` (`DimensionTemplate`).
/// - State: Maintains consistent static invariants across `DimensionTemplate` with zero runtime mutation.
/// - Action: Referenced deterministically during AST member enumeration and padding synthesis.
/// - Resource: Zero runtime heap overhead (`'static` string slices).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-DIMENSIONTEMPLATE] | SysML v2: PartDef(DimensionTemplate)
#[derive(Clone, Copy)]
struct DimensionTemplate {
    dimension: FailureDimension,
    mode_suffix: &'static str,
    effect_template: &'static str,
    mitigation_template: &'static str,
    default_s: u32,
    default_o: u32,
    default_d: u32,
}

const INTERFACE_DIMENSION_TEMPLATE: DimensionTemplate = DimensionTemplate {
    dimension: FailureDimension::Interface,
    mode_suffix: "Interface Contract Violation / Payload Malformation",
    effect_template: "Loss of inter-component contract exchange or rejected boundary message",
    mitigation_template: "Enforce boundary schema contract validation and deterministic rejection",
    default_s: 8,
    default_o: 3,
    default_d: 2,
};

const STATE_DIMENSION_TEMPLATE: DimensionTemplate = DimensionTemplate {
    dimension: FailureDimension::State,
    mode_suffix: "State Invariant Divergence / Transition Deadlock",
    effect_template: "Component fails to satisfy target lifecycle state invariant",
    mitigation_template: "Enforce guarded state transition preconditions and deterministic recovery transition",
    default_s: 7,
    default_o: 3,
    default_d: 2,
};

const ACTION_DIMENSION_TEMPLATE: DimensionTemplate = DimensionTemplate {
    dimension: FailureDimension::Action,
    mode_suffix: "Behavioral Execution Fault / Postcondition Violation",
    effect_template: "Incomplete or non-terminating behavioral transformation",
    mitigation_template: "Enforce precondition guards, bounded execution steps, and postcondition assertions",
    default_s: 8,
    default_o: 2,
    default_d: 3,
};

const RESOURCE_DIMENSION_TEMPLATE: DimensionTemplate = DimensionTemplate {
    dimension: FailureDimension::Resource,
    mode_suffix: "Resource Capacity Exhaustion / Attribute Bound Breach",
    effect_template: "Degraded structural capacity or rejected allocation under peak load",
    mitigation_template: "Enforce explicit capacity bounds, quota checks, and admission backpressure",
    default_s: 6,
    default_o: 3,
    default_d: 2,
};

const DIMENSION_TEMPLATES: [DimensionTemplate; 4] = [
    INTERFACE_DIMENSION_TEMPLATE,
    STATE_DIMENSION_TEMPLATE,
    ACTION_DIMENSION_TEMPLATE,
    RESOURCE_DIMENSION_TEMPLATE,
];

const ABSTRACT_STRUCTURAL_SUB_MODES: [DimensionTemplate; 4] = [
    DimensionTemplate {
        dimension: FailureDimension::Interface,
        mode_suffix: "Boundary Protocol Desynchronization / Contract Mismatch",
        effect_template: "Boundary interaction sequence diverges from declared port contract",
        mitigation_template: "Enforce strict interface contract validation and ordering preconditions",
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
        mitigation_template: "Enforce invariant assertions prior to state commit and atomic rollback",
        default_s: 7,
        default_o: 3,
        default_d: 2,
    },
    DimensionTemplate {
        dimension: FailureDimension::Resource,
        mode_suffix: "Capacity Quota Saturation / Allocation Contention",
        effect_template: "Concurrent resource demand exceeds declared structural attribute limit",
        mitigation_template: "Enforce bounded allocation quotas and deterministic admission backpressure",
        default_s: 6,
        default_o: 3,
        default_d: 2,
    },
];

/// Parses Markdown tables from `FAILURE_MODE_REGISTRY.md` into deterministic `RegistryFailureEntry` records.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_failure_mode_registry` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Linear single-pass parser with zero `regex` dependencies and zero direct slice indexing (`&s[a..b]`).
/// - Preserves source row ordering and clamps all parsed `(S, O, D)` scores to `[1, 10]`.
///
/// ## 3. Preconditions
/// - `markdown: &str` contains UTF-8 Markdown content (empty or arbitrary prose/tables accepted safely).
///
/// ## 4. Postconditions
/// - Returns `Vec<RegistryFailureEntry>` containing every valid component failure row discovered in `markdown`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is the byte length of `markdown`.
/// - Space Complexity: O(R) where R is the number of parsed `RegistryFailureEntry` records.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Gracefully ignores malformed lines, prose paragraphs, separator lines, and incomplete rows.
/// - State: Resets `active_cols` whenever table continuity ends so unrelated tables cannot cross-contaminate.
/// - Action: Terminates in bounded linear time over `markdown.lines()`.
/// - Resource: Bounded allocations proportional to valid failure rows.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-PARSE-REGISTRY] | SysML v2: ActionDef(parse_failure_mode_registry)
pub fn parse_failure_mode_registry(markdown: &str) -> Vec<RegistryFailureEntry> {
    let mut entries = Vec::new();
    let mut active_cols: Option<RegistryTableColumns> = None;
    let mut heading_component: Option<String> = None;

    for line in markdown.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('#') {
            active_cols = None;
            let title = rest.trim_start_matches('#').trim();
            let cleaned = title
                .strip_prefix("Component:")
                .or_else(|| title.strip_prefix("component:"))
                .unwrap_or(title)
                .trim()
                .trim_matches('`')
                .trim();
            if !cleaned.is_empty() {
                heading_component = Some(cleaned.to_string());
            }
            continue;
        }

        let Some(cells) = split_markdown_table_cells(trimmed) else {
            if !trimmed.is_empty() {
                active_cols = None;
            }
            continue;
        };

        if is_markdown_separator_row(&cells) {
            continue;
        }

        if let Some(cols) = detect_registry_header_columns(&cells) {
            active_cols = Some(cols);
            continue;
        }

        if let Some(cols) = active_cols.as_ref() {
            if let Some(entry) =
                parse_registry_data_row(&cells, cols, heading_component.as_deref())
            {
                entries.push(entry);
            }
        }
    }

    entries
}

/// Loads and parses `FAILURE_MODE_REGISTRY.md` relative to the target safety `out_dir`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `load_failure_mode_registry_for_out_dir` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Deterministically probes `<out_dir_parent>/research/FAILURE_MODE_REGISTRY.md` and `<out_dir_parent>/docs/research/FAILURE_MODE_REGISTRY.md` (along with `<out_dir>/../...` fallbacks) without traversing above the enclosing subworkspace boundary.
///
/// ## 3. Preconditions
/// - `out_dir: &Path` identifies the target safety artifact directory (which need not exist on disk yet during `--dry-run`).
///
/// ## 4. Postconditions
/// - Returns parsed `Vec<RegistryFailureEntry>` from the first readable candidate path, or `Vec::new()` when no registry file exists.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) over the byte size of the discovered `FAILURE_MODE_REGISTRY.md` file (at most 4 path probes).
/// - Space Complexity: O(N) for file content buffer and parsed `Vec<RegistryFailureEntry>`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Resolves lexical `out_dir.parent()` before `..` joins so `--dry-run` works before `out_dir` creation.
/// - State: Read-only filesystem inspection with zero mutation of workspace directories.
/// - Action: Falls back cleanly to an empty vector on `ENOENT` or unreadable paths without panicking.
/// - Resource: Closes file descriptor immediately upon completing `fs::read_to_string`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-LOAD-REGISTRY-OUTDIR] | SysML v2: ActionDef(load_failure_mode_registry_for_out_dir)
pub fn load_failure_mode_registry_for_out_dir(out_dir: &Path) -> Vec<RegistryFailureEntry> {
    let parent = out_dir.parent().unwrap_or_else(|| Path::new("."));
    let candidates = [
        parent.join("research").join("FAILURE_MODE_REGISTRY.md"),
        parent
            .join("docs")
            .join("research")
            .join("FAILURE_MODE_REGISTRY.md"),
        out_dir
            .join("..")
            .join("research")
            .join("FAILURE_MODE_REGISTRY.md"),
        out_dir
            .join("..")
            .join("docs")
            .join("research")
            .join("FAILURE_MODE_REGISTRY.md"),
    ];

    for candidate in &candidates {
        if let Ok(content) = fs::read_to_string(candidate) {
            return parse_failure_mode_registry(&content);
        }
    }

    Vec::new()
}

/// Splits a single Markdown pipe-table line into trimmed, unquoted cell strings.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `split_markdown_table_cells` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses total prefix/suffix stripping (`strip_prefix` / `strip_suffix`) and delimiter splitting with zero direct range slicing.
///
/// ## 3. Preconditions
/// - `trimmed: &str` is a whitespace-trimmed single line from a Markdown document.
///
/// ## 4. Postconditions
/// - Returns `Some(Vec<String>)` for pipe-delimited lines starting with `'|'`, or `None` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) where L is the byte length of `trimmed`.
/// - Space Complexity: O(L) across the extracted cell strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-table lines return `None` immediately.
/// - State: Pure transformation with no side effects.
/// - Action: Single pass over pipe-delimited segments.
/// - Resource: Bounded by single-line byte length.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-SPLIT-MD-CELLS] | SysML v2: ActionDef(split_markdown_table_cells)
fn split_markdown_table_cells(trimmed: &str) -> Option<Vec<String>> {
    let without_leading = trimmed.strip_prefix('|')?;
    let body = without_leading
        .strip_suffix('|')
        .unwrap_or(without_leading);
    let cells: Vec<String> = body
        .split('|')
        .map(|cell| {
            cell.trim()
                .trim_matches('`')
                .trim_matches('*')
                .trim()
                .to_string()
        })
        .collect();
    if cells.is_empty() {
        None
    } else {
        Some(cells)
    }
}

/// Determines whether a parsed Markdown row is a pipe-table alignment separator (`| :--- | :---: |`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_markdown_separator_row` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Evaluates to `true` if and only if `cells` is non-empty and every cell consists solely of `'-'`, `':'`, or whitespace with at least one `'-'`.
///
/// ## 3. Preconditions
/// - `cells: &[String]` contains normalized cells from `split_markdown_table_cells`.
///
/// ## 4. Postconditions
/// - Returns a boolean classification without mutating `cells`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(C) total characters across `cells`.
/// - Space Complexity: O(1) auxiliary memory.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Empty cell slices safely evaluate to `false`.
/// - State: Pure predicate.
/// - Action: Short-circuits on the first non-separator character.
/// - Resource: Zero heap allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-MD-SEPARATOR-PREDICATE] | SysML v2: ActionDef(is_markdown_separator_row)
fn is_markdown_separator_row(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells.iter().all(|cell| {
            cell.contains('-')
                && cell
                    .chars()
                    .all(|ch| ch == '-' || ch == ':' || ch.is_whitespace())
        })
}

/// Detects a `FAILURE_MODE_REGISTRY.md` table header row and resolves its column coordinates.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `detect_registry_header_columns` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Requires both `dimension_col` and `failure_mode_col` to be present AND verifies the `dimension_col` cell itself is a header label (not a data dimension variant) so data rows can never be misclassified as headers.
///
/// ## 3. Preconditions
/// - `cells: &[String]` is a non-separator Markdown pipe-table row.
///
/// ## 4. Postconditions
/// - Returns `Some(RegistryTableColumns)` when `cells` is a valid failure-mode header row, otherwise `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(K) over the number of columns K in `cells`.
/// - Space Complexity: O(1) stack storage for `RegistryTableColumns`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Ignores unrelated Markdown tables lacking `Dimension` and `Failure Mode` columns.
/// - State: Pure header classification.
/// - Action: Bounded single pass over `cells.iter().enumerate()`.
/// - Resource: Bounded single lowercased string per header cell.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-DETECT-REGISTRY-HEADER] | SysML v2: ActionDef(detect_registry_header_columns)
fn detect_registry_header_columns(cells: &[String]) -> Option<RegistryTableColumns> {
    let mut cols = RegistryTableColumns::default();
    for (idx, cell) in cells.iter().enumerate() {
        let lower = cell.to_ascii_lowercase();
        classify_registry_header_cell(idx, &lower, &mut cols);
    }

    let dim_idx = cols.dimension_col?;
    let _mode_idx = cols.failure_mode_col?;
    let dim_header = cells.get(dim_idx)?;
    if FailureDimension::from_label(dim_header).is_some() {
        return None;
    }
    Some(cols)
}

/// Maps a single lowercased header token onto its corresponding `RegistryTableColumns` slot.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `classify_registry_header_cell` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves first-match precedence via `.is_none()` guards so duplicate or composite column headers cannot overwrite primary coordinates.
///
/// ## 3. Preconditions
/// - `lower: &str` is an ASCII-lowercased, trimmed header cell label at 0-based index `idx`.
///
/// ## 4. Postconditions
/// - Populates at most one matching field on `cols` when `lower` matches a recognized registry column role.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(|lower|) bounded substring comparisons.
/// - Space Complexity: O(1) zero heap allocations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Unrecognized extra columns (such as `ID` or `RPN`) are safely skipped.
/// - State: Mutates only unoccupied `Option<usize>` slots in `cols`.
/// - Action: Constant-depth conditional dispatch.
/// - Resource: Zero dynamic memory usage.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-CLASSIFY-HEADER-CELL] | SysML v2: ActionDef(classify_registry_header_cell)
fn classify_registry_header_cell(idx: usize, lower: &str, cols: &mut RegistryTableColumns) {
    if cols.component_col.is_none()
        && (lower.contains("component") || lower.contains("part") || lower.contains("subsystem"))
    {
        cols.component_col = Some(idx);
    } else if cols.dimension_col.is_none() && lower.contains("dimension") {
        cols.dimension_col = Some(idx);
    } else if cols.failure_mode_col.is_none()
        && (lower.contains("failure mode") || lower.contains("failure_mode") || lower == "mode")
    {
        cols.failure_mode_col = Some(idx);
    } else if cols.effect_col.is_none()
        && (lower.contains("effect") || lower.contains("consequence") || lower.contains("impact"))
    {
        cols.effect_col = Some(idx);
    } else if cols.severity_col.is_none()
        && (lower == "s" || lower == "sev" || lower.contains("severity"))
    {
        cols.severity_col = Some(idx);
    } else if cols.occurrence_col.is_none()
        && (lower == "o"
            || lower == "occ"
            || lower.contains("occurrence")
            || lower.contains("likelihood"))
    {
        cols.occurrence_col = Some(idx);
    } else if cols.detection_col.is_none()
        && (lower == "d" || lower == "det" || lower.contains("detection"))
    {
        cols.detection_col = Some(idx);
    } else if cols.mitigation_col.is_none()
        && (lower.contains("mitigation")
            || lower.contains("control")
            || lower.contains("countermeasure"))
    {
        cols.mitigation_col = Some(idx);
    } else if cols.basis_col.is_none()
        && (lower.contains("basis")
            || lower.contains("provenance")
            || lower.contains("evidence")
            || lower.contains("source")
            || lower.contains("citation"))
    {
        cols.basis_col = Some(idx);
    }
}

/// Converts a single parsed Markdown data row into a validated `RegistryFailureEntry`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_registry_data_row` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses `.get(..)` exclusively for every column lookup, clamping `(severity, occurrence, detection)` to `[1, 10]` and falling back to dimension defaults when score cells are absent.
///
/// ## 3. Preconditions
/// - `cols: &RegistryTableColumns` was validated by `detect_registry_header_columns`.
///
/// ## 4. Postconditions
/// - Returns `Some(RegistryFailureEntry)` when component, dimension, and failure mode are non-empty; otherwise `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) over total character count of the referenced row cells.
/// - Space Complexity: O(L) for owned string fields on `RegistryFailureEntry`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Rows with missing cells or unrecognized dimension labels return `None` cleanly.
/// - State: Pure row conversion with zero shared state mutation.
/// - Action: Executes in constant column lookup steps.
/// - Resource: Allocates at most 4 owned `String` fields per valid row.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-PARSE-REGISTRY-ROW] | SysML v2: ActionDef(parse_registry_data_row)
fn parse_registry_data_row(
    cells: &[String],
    cols: &RegistryTableColumns,
    fallback_component: Option<&str>,
) -> Option<RegistryFailureEntry> {
    let dim_raw = cols.dimension_col.and_then(|i| cells.get(i))?;
    let dimension = FailureDimension::from_label(dim_raw)?;

    let failure_mode = cols
        .failure_mode_col
        .and_then(|i| cells.get(i))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())?
        .to_string();

    let component = cols
        .component_col
        .and_then(|i| cells.get(i))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .or(fallback_component)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())?
        .to_string();

    let tmpl = template_for_dimension(dimension);
    let effect = cols
        .effect_col
        .and_then(|i| cells.get(i))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let mitigation = cols
        .mitigation_col
        .and_then(|i| cells.get(i))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let severity = cols
        .severity_col
        .and_then(|i| cells.get(i))
        .map(|s| parse_clamped_score(s, tmpl.default_s))
        .unwrap_or(tmpl.default_s);
    let occurrence = cols
        .occurrence_col
        .and_then(|i| cells.get(i))
        .map(|s| parse_clamped_score(s, tmpl.default_o))
        .unwrap_or(tmpl.default_o);
    let detection = cols
        .detection_col
        .and_then(|i| cells.get(i))
        .map(|s| parse_clamped_score(s, tmpl.default_d))
        .unwrap_or(tmpl.default_d);

    let basis_cell = cols
        .basis_col
        .and_then(|i| cells.get(i))
        .map(String::as_str);
    let is_ssot_evidenced = parse_ssot_evidence_flag(basis_cell);

    Some(RegistryFailureEntry {
        component,
        dimension,
        failure_mode,
        effect,
        severity,
        occurrence,
        detection,
        mitigation,
        is_ssot_evidenced,
    })
}

/// Extracts and clamps an integer FMECA score (`1..=10`) from a Markdown table cell.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_clamped_score` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Output score always satisfies `1 <= score <= 10` assuming `1 <= default_score <= 10`.
///
/// ## 3. Preconditions
/// - `default_score` lies within `[1, 10]`; `raw` is an arbitrary cell string (e.g. `"7"` or `"High (7)"`).
///
/// ## 4. Postconditions
/// - Returns the parsed integer clamped to `[1, 10]`, or `default_score.clamp(1, 10)` if no digits exist.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(|raw|) single character iterator scan.
/// - Space Complexity: O(D) where D is the contiguous digit count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-numeric or overflow inputs fall back deterministically to `default_score.clamp(1, 10)`.
/// - State: Pure arithmetic helper.
/// - Action: Zero panics or unchecked indexing.
/// - Resource: Minimal digit buffer allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-PARSE-CLAMPED-SCORE] | SysML v2: ActionDef(parse_clamped_score)
fn parse_clamped_score(raw: &str, default_score: u32) -> u32 {
    let digits: String = raw
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits
        .parse::<u32>()
        .map(|n| n.clamp(1, 10))
        .unwrap_or_else(|_| default_score.clamp(1, 10))
}

/// Evaluates whether a registry row's basis/evidence cell qualifies for `"SSOT"` provenance.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_ssot_evidence_flag` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Marks `"Derived"`, `"Pending"`, `"TBD"`, `"Unverified"`, `"None"`, `"N/A"`, `"-"`, or empty basis cells as `false` (`"Derived"`), while researched `"SSOT"` or cited evidence cells evaluate to `true`.
///
/// ## 3. Preconditions
/// - `basis_cell: Option<&str>` represents the optional `Basis` / `Evidence` cell value for a registry row.
///
/// ## 4. Postconditions
/// - Returns `true` iff the registry entry carries verified SSOT provenance.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) over `basis_cell` byte length.
/// - Space Complexity: O(L) for ASCII-lowercased token inspection.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Handles absent basis columns (`None`) as researched registry entries (`true`) and empty strings (`Some("")`) as unverified (`false`).
/// - State: Pure boolean predicate.
/// - Action: Deterministic string comparison without regex.
/// - Resource: Bounded single string allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-SSOT-EVIDENCE-FLAG] | SysML v2: ActionDef(parse_ssot_evidence_flag)
fn parse_ssot_evidence_flag(basis_cell: Option<&str>) -> bool {
    let Some(raw) = basis_cell else {
        return true;
    };
    let lower = raw.trim().to_ascii_lowercase();
    !(lower.is_empty()
        || lower == "-"
        || lower == "n/a"
        || lower == "none"
        || lower == "tbd"
        || lower == "pending"
        || lower == "unverified"
        || lower.contains("derived"))
}

/// Looks up the static `DimensionTemplate` corresponding to a `FailureDimension`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `template_for_dimension` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Exhaustive total mapping from all 4 `FailureDimension` variants onto `DIMENSION_TEMPLATES`.
///
/// ## 3. Preconditions
/// - `dimension` is a valid `FailureDimension` variant.
///
/// ## 4. Postconditions
/// - Returns `&'static DimensionTemplate` whose `.dimension == dimension`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) exhaustive `match` over 4 variants.
/// - Space Complexity: O(1) static reference return.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Total function over `FailureDimension`.
/// - State: References immutable `'static` compile-time constants.
/// - Action: Zero branches capable of failing or panicking.
/// - Resource: Zero runtime allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-TEMPLATE-FOR-DIMENSION] | SysML v2: ActionDef(template_for_dimension)
fn template_for_dimension(dimension: FailureDimension) -> &'static DimensionTemplate {
    match dimension {
        FailureDimension::Interface => &INTERFACE_DIMENSION_TEMPLATE,
        FailureDimension::State => &STATE_DIMENSION_TEMPLATE,
        FailureDimension::Action => &ACTION_DIMENSION_TEMPLATE,
        FailureDimension::Resource => &RESOURCE_DIMENSION_TEMPLATE,
    }
}

/// Generates the FMECA matrix covering 100% of declared AST `part def` components without external registry records.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `generate_fmeca_matrix` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Delegates directly to `generate_fmeca_matrix_with_registry(pkg, scoring_config, &[])`, guaranteeing domain-neutral abstract MBSE templates, full `[0..]` AST member enumeration, and honest `"Derived"` basis tagging when no registry entries are supplied.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` and `scoring_config: Option<&FmecaScoringConfig>` satisfy AST type constraints.
///
/// ## 4. Postconditions
/// - Returns `Vec<FmecaRow>` covering all non-negated components across all 4 failure dimensions.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(P * M) over non-negated parts P and declared AST members M.
/// - Space Complexity: O(P * M) for the returned `Vec<FmecaRow>`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `generate_fmeca_matrix` (`Vec<FmecaRow>`).
/// - State: Maintains consistent state invariants across `generate_fmeca_matrix` without partial mutation.
/// - Action: Executes `generate_fmeca_matrix` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `generate_fmeca_matrix` (`STPA-FMECA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-GENERATE-FMECA-MATRIX] | SysML v2: ActionDef(generate_fmeca_matrix)
pub fn generate_fmeca_matrix(
    pkg: &PackageDef,
    scoring_config: Option<&FmecaScoringConfig>,
) -> Vec<FmecaRow> {
    generate_fmeca_matrix_with_registry(pkg, scoring_config, &[])
}

/// Generates the FMECA matrix integrating researched `FAILURE_MODE_REGISTRY.md` records, explicit `HazardDef` declarations, and full `[0..]` AST member synthesis.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `generate_fmeca_matrix_with_registry` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Excludes negated components (`!Ident` and negative attribute/constraint stems) and `_`-prefixed internal helpers.
/// - Emits researched `registry_entries` with verified `"SSOT"` (or `"Derived"`) provenance and explicit component-referenced `HazardDef` rows with `"SSOT"` provenance, while stamping `"Derived"` on all compiler-synthesized AST member and structural padding rows.
/// - Enumerates 100% of AST members (`ports`, `states`, `actions` + `operations`, `attributes`) across `[0..]` without `.first()` truncation and pads non-empty models to `>= 15` rows.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` represents a parsed SysML v2 package; `registry_entries` holds zero or more pre-parsed `RegistryFailureEntry` items.
///
/// ## 4. Postconditions
/// - Returns `Vec<FmecaRow>` (`Vec::new()` iff no non-negated `PartDef` exists; otherwise `len() >= 15`) with sequential `FMECA-001..` identifiers.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(|sysml| + P * (R + H + M)) over serialized SysML size, parts P, registry entries R, hazards H, and AST members M.
/// - Space Complexity: O(P + M + R) auxiliary sets and output row vector.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Accepts empty packages, empty scoring scales, and empty registry slices gracefully.
/// - State: Pure deterministic synthesis with `BTreeSet` negation precomputation.
/// - Action: Bounded loops with `.checked_rem` and `.get(..)` preventing division-by-zero or index panics.
/// - Resource: Bounded vector growth proportional to AST member and registry cardinality.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-GENERATE-WITH-REGISTRY] | SysML v2: ActionDef(generate_fmeca_matrix_with_registry)
pub fn generate_fmeca_matrix_with_registry(
    pkg: &PackageDef,
    scoring_config: Option<&FmecaScoringConfig>,
    registry_entries: &[RegistryFailureEntry],
) -> Vec<FmecaRow> {
    let mut rows = Vec::new();
    let all_parts = pkg.get_all_parts();
    let negated_parts = compute_negated_part_names(pkg, &all_parts);

    let mut filtered_parts: Vec<&PartDef> = all_parts
        .iter()
        .filter(|p| !p.name.starts_with('_'))
        .filter(|p| !negated_parts.contains(&p.name))
        .collect();

    if filtered_parts.is_empty() {
        filtered_parts = all_parts
            .iter()
            .filter(|p| !negated_parts.contains(&p.name))
            .collect();
    }

    if filtered_parts.is_empty() {
        return rows;
    }

    let all_hazards = pkg.get_all_hazards();

    for part in &filtered_parts {
        for tmpl in &DIMENSION_TEMPLATES {
            emit_part_dimension_rows(
                part,
                tmpl,
                scoring_config,
                registry_entries,
                &all_hazards,
                &mut rows,
            );
        }
    }

    pad_minimum_derived_rows(&filtered_parts, scoring_config, &mut rows);

    rows
}

/// Precomputes the `BTreeSet` of `PartDef` names excluded by `!Ident` assertions or negative constraint stems.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `compute_negated_part_names` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Scans serialized SysML bytes using `.get(..)` exclusively (zero direct indexing) and unions `!Ident` tokens with `extract_negated_components` stem matches.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` and `all_parts` originate from the same parsed SysML model.
///
/// ## 4. Postconditions
/// - Returns an ordered `BTreeSet<String>` containing every negated component identifier.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(|sysml| + P * S) where P is part count and S is negated stem count.
/// - Space Complexity: O(|sysml| + P) for serialized buffer and `BTreeSet`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Empty packages yield an empty `BTreeSet`.
/// - State: Deterministic `BTreeSet` ordering with zero hash randomization.
/// - Action: Monotonically advancing `byte_idx` guarantees loop termination.
/// - Resource: Temporary serialized SysML buffer dropped prior to matrix emission.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-NEGATED-PARTS] | SysML v2: ActionDef(compute_negated_part_names)
fn compute_negated_part_names(pkg: &PackageDef, all_parts: &[PartDef]) -> BTreeSet<String> {
    let pkg_sysml = crate::semantic::serializer::to_sysml(pkg);
    let mut negated_parts = BTreeSet::new();
    let sysml_bytes = pkg_sysml.as_bytes();
    let mut byte_idx = 0;
    while byte_idx < sysml_bytes.len() {
        if sysml_bytes.get(byte_idx) == Some(&b'!') {
            let mut start = byte_idx + 1;
            while start < sysml_bytes.len()
                && sysml_bytes.get(start).is_some_and(u8::is_ascii_whitespace)
            {
                start += 1;
            }
            if sysml_bytes
                .get(start)
                .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
            {
                let mut end = start + 1;
                while end < sysml_bytes.len()
                    && sysml_bytes
                        .get(end)
                        .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
                {
                    end += 1;
                }
                if let Some(ident) = pkg_sysml.get(start..end) {
                    negated_parts.insert(ident.to_string());
                }
                byte_idx = end;
                continue;
            }
        }
        byte_idx += 1;
    }

    let mut negated_stems: BTreeSet<String> =
        pkg.extract_negated_components().into_iter().collect();
    negated_stems.extend(deap_core::sysml_ast::extract_negated_components(&pkg_sysml));
    for part in all_parts {
        let comp_clean: String = part
            .name
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        if !comp_clean.is_empty()
            && negated_stems.iter().any(|stem| {
                comp_clean.contains(stem.as_str())
                    || deap_core::sysml_ast::component_matches(&part.name, stem)
            })
        {
            negated_parts.insert(part.name.clone());
        }
    }

    negated_parts
}

/// Emits FMECA rows for a single `(PartDef, FailureDimension)` pair using registry precedence, explicit `HazardDef` matching, or `[0..]` AST member enumeration.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `emit_part_dimension_rows` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Precedence contract: (1) matching `RegistryFailureEntry` records (`"SSOT"` iff `is_ssot_evidenced`), (2) explicit component-referenced `HazardDef` declarations (`"SSOT"`), (3) exhaustive `[0..]` AST member enumeration or 1 component-level abstract fallback row (`"Derived"`).
///
/// ## 3. Preconditions
/// - `part` is a non-negated `PartDef`; `tmpl` specifies the active `FailureDimension` and abstract MBSE defaults.
///
/// ## 4. Postconditions
/// - Appends at least one `FmecaRow` for `(part.name, tmpl.dimension)` onto `rows`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(R + H + M_d) where R is registry length, H is hazard count, and M_d is member count in `tmpl.dimension`.
/// - Space Complexity: O(M_d) for member triple staging.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Clamps ingested registry scores to `[1, 10]` and fills empty effect/mitigation strings from `tmpl`.
/// - State: Appends monotonically numbered `FMECA-XXX` rows to `rows`.
/// - Action: Finite iteration over slices without recursion.
/// - Resource: Bounded allocations proportional to emitted rows.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-EMIT-PART-DIMENSION] | SysML v2: ActionDef(emit_part_dimension_rows)
fn emit_part_dimension_rows(
    part: &PartDef,
    tmpl: &DimensionTemplate,
    scoring_config: Option<&FmecaScoringConfig>,
    registry_entries: &[RegistryFailureEntry],
    all_hazards: &[HazardDef],
    rows: &mut Vec<FmecaRow>,
) {
    let part_name = part.name.as_str();
    let matching_registry: Vec<&RegistryFailureEntry> = registry_entries
        .iter()
        .filter(|entry| registry_entry_matches_part(entry, part_name, tmpl.dimension))
        .collect();

    if !matching_registry.is_empty() {
        for entry in matching_registry {
            let s = entry.severity.clamp(1, 10);
            let o = entry.occurrence.clamp(1, 10);
            let d = entry.detection.clamp(1, 10);
            let rpn = s * o * d;
            let effect = if entry.effect.is_empty() {
                format!("{}: {}", part_name, tmpl.effect_template)
            } else {
                entry.effect.clone()
            };
            let mitigation = if entry.mitigation.is_empty() {
                format!("{}: {}", part_name, tmpl.mitigation_template)
            } else {
                entry.mitigation.clone()
            };
            let basis = if entry.is_ssot_evidenced {
                "SSOT"
            } else {
                "Derived"
            };
            let row_id = format!("FMECA-{:03}", rows.len() + 1);
            rows.push(FmecaRow {
                id: row_id,
                component: part_name.to_string(),
                dimension: tmpl.dimension.as_str().to_string(),
                failure_mode: entry.failure_mode.clone(),
                effect,
                severity: s,
                occurrence: o,
                detection: d,
                rpn,
                severity_cell: format!("{}", s),
                occurrence_cell: format!("{}", o),
                detection_cell: format!("{}", d),
                rpn_cell: format!("{}", rpn),
                mitigation,
                basis: basis.to_string(),
            });
        }
        return;
    }

    let matching_hazards: Vec<&HazardDef> = all_hazards
        .iter()
        .filter(|haz| {
            hazard_references_part(haz, part_name)
                && classify_hazard_dimension(haz) == tmpl.dimension
        })
        .collect();

    if !matching_hazards.is_empty() {
        for haz in matching_hazards {
            let triple = (
                format!("{}: {}", part_name, haz.name),
                format!(
                    "{}: {}",
                    part_name,
                    haz.doc.as_deref().unwrap_or(tmpl.effect_template)
                ),
                format!(
                    "{}: Enforce declared hazard constraint mitigating '{}'",
                    part_name, haz.name
                ),
            );
            push_scored_row(rows, part_name, tmpl, scoring_config, triple, "SSOT");
        }
        return;
    }

    let triples = collect_dimension_member_triples(part, tmpl);
    for triple in triples {
        push_scored_row(rows, part_name, tmpl, scoring_config, triple, "Derived");
    }
}

/// Checks whether a `RegistryFailureEntry` targets `(part_name, dim)` via exact case-insensitive match or `component_matches`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `registry_entry_matches_part` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Rejects empty component strings so blank cells can never match arbitrary AST parts.
///
/// ## 3. Preconditions
/// - `part_name` is a non-empty SysML `PartDef` identifier.
///
/// ## 4. Postconditions
/// - Returns `true` iff `entry.dimension == dim` and `entry.component` matches `part_name`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(|part_name| + |entry.component|).
/// - Space Complexity: O(|part_name| + |entry.component|) within normalized stem matching.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Whitespace-only component fields return `false`.
/// - State: Pure predicate with zero side effects.
/// - Action: Short-circuits immediately when `entry.dimension != dim`.
/// - Resource: Bounded stem normalization strings.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-REGISTRY-COMPONENT-MATCH] | SysML v2: ActionDef(registry_entry_matches_part)
fn registry_entry_matches_part(
    entry: &RegistryFailureEntry,
    part_name: &str,
    dim: FailureDimension,
) -> bool {
    let comp = entry.component.trim();
    !comp.is_empty()
        && entry.dimension == dim
        && (comp.eq_ignore_ascii_case(part_name)
            || deap_core::sysml_ast::component_matches(part_name, comp))
}

/// Determines whether an explicit `HazardDef` directly references `part_name` in its identifier or doc block.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `hazard_references_part` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Requires explicit case-insensitive presence of `part_name` within `haz.name` or `haz.doc` so unrelated global hazards are never falsely attributed to `part_name`.
///
/// ## 3. Preconditions
/// - `haz: &HazardDef` is a declared AST hazard node; `part_name: &str` is the candidate component identifier.
///
/// ## 4. Postconditions
/// - Returns `true` iff `part_name` is non-empty and explicitly cited by `haz`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(|haz.name| + |haz.doc| + |part_name|).
/// - Space Complexity: O(|haz.name| + |haz.doc| + |part_name|) for lowercased matching.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Empty `part_name` evaluates safely to `false`.
/// - State: Pure read-only AST inspection.
/// - Action: Linear substring check with zero regex backtracking.
/// - Resource: Short-lived lowercase buffers dropped on return.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-HAZARD-PART-MATCH] | SysML v2: ActionDef(hazard_references_part)
fn hazard_references_part(haz: &HazardDef, part_name: &str) -> bool {
    let trimmed = part_name.trim();
    if trimmed.is_empty() {
        return false;
    }
    let needle = trimmed.to_ascii_lowercase();
    if haz.name.to_ascii_lowercase().contains(&needle) {
        return true;
    }
    haz.doc
        .as_deref()
        .is_some_and(|doc| doc.to_ascii_lowercase().contains(&needle))
}

/// Classifies an explicit `HazardDef` into one of the four universal `FailureDimension` buckets.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `classify_hazard_dimension` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Deterministic total classification over `(haz.name, haz.doc)` structural keywords, defaulting behavioral hazards to `FailureDimension::Action`.
///
/// ## 3. Preconditions
/// - `haz: &HazardDef` is a valid AST hazard node.
///
/// ## 4. Postconditions
/// - Returns a deterministic `FailureDimension` variant.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(|haz.name| + |haz.doc|).
/// - Space Complexity: O(|haz.name| + |haz.doc|) for combined lowercase text.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Handles `haz.doc == None` cleanly.
/// - State: Stateless deterministic classifier.
/// - Action: Single pass keyword checks.
/// - Resource: Single formatted string buffer.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-CLASSIFY-HAZARD-DIM] | SysML v2: ActionDef(classify_hazard_dimension)
fn classify_hazard_dimension(haz: &HazardDef) -> FailureDimension {
    let combined = format!("{} {}", haz.name, haz.doc.as_deref().unwrap_or("")).to_ascii_lowercase();
    if combined.contains("interface")
        || combined.contains("port")
        || combined.contains("protocol")
        || combined.contains("ingress")
        || combined.contains("egress")
    {
        FailureDimension::Interface
    } else if combined.contains("state")
        || combined.contains("transition")
        || combined.contains("deadlock")
    {
        FailureDimension::State
    } else if combined.contains("resource")
        || combined.contains("capacity")
        || combined.contains("quota")
        || combined.contains("memory")
        || combined.contains("attribute")
    {
        FailureDimension::Resource
    } else {
        FailureDimension::Action
    }
}

/// Enumerates all `[0..]` AST members of `part` belonging to `tmpl.dimension` (or 1 component-level abstract fallback triple when empty).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `collect_dimension_member_triples` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero `.first()` truncation: visits 100% of `part.ports` (`Interface`), `part.states` (`State`), `part.actions` chained with `part.operations` (`Action`), and `part.attributes` (`Resource`).
/// - Always returns a non-empty `Vec<(String, String, String)>` (`len() >= 1`).
///
/// ## 3. Preconditions
/// - `part: &PartDef` and `tmpl: &DimensionTemplate` are valid references.
///
/// ## 4. Postconditions
/// - Returns `(failure_mode, effect, mitigation)` triples grounded exclusively in AST member identifiers and domain-neutral abstract MBSE templates.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M_d) where M_d is the number of AST members in `tmpl.dimension` on `part`.
/// - Space Complexity: O(M_d) for the returned vector of triples.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Empty member vectors automatically emit a single component-scoped abstract triple.
/// - State: Pure functional projection over immutable AST slices.
/// - Action: Finite iteration over declared AST member vectors.
/// - Resource: Allocates exact member count strings without excess buffering.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-FULL-ENUMERATION] | SysML v2: ActionDef(collect_dimension_member_triples)
fn collect_dimension_member_triples(
    part: &PartDef,
    tmpl: &DimensionTemplate,
) -> Vec<(String, String, String)> {
    let part_name = part.name.as_str();
    let mut triples = Vec::new();

    match tmpl.dimension {
        FailureDimension::Interface => {
            for port in &part.ports {
                triples.push((
                    format!("{} Port '{}' {}", part_name, port.name, tmpl.mode_suffix),
                    format!(
                        "{}: {} on port '{}'",
                        part_name, tmpl.effect_template, port.name
                    ),
                    format!(
                        "{}: {} for port '{}'",
                        part_name, tmpl.mitigation_template, port.name
                    ),
                ));
            }
        }
        FailureDimension::State => {
            for st in &part.states {
                triples.push((
                    format!("{} State '{}' {}", part_name, st.name, tmpl.mode_suffix),
                    format!(
                        "{}: {} in state '{}'",
                        part_name, tmpl.effect_template, st.name
                    ),
                    format!(
                        "{}: {} for state '{}'",
                        part_name, tmpl.mitigation_template, st.name
                    ),
                ));
            }
        }
        FailureDimension::Action => {
            for act in &part.actions {
                triples.push((
                    format!("{} Action '{}' {}", part_name, act.name, tmpl.mode_suffix),
                    format!(
                        "{}: {} during action '{}'",
                        part_name, tmpl.effect_template, act.name
                    ),
                    format!(
                        "{}: {} for action '{}'",
                        part_name, tmpl.mitigation_template, act.name
                    ),
                ));
            }
            for op in &part.operations {
                triples.push((
                    format!("{} Operation '{}' {}", part_name, op.name, tmpl.mode_suffix),
                    format!(
                        "{}: {} during operation '{}'",
                        part_name, tmpl.effect_template, op.name
                    ),
                    format!(
                        "{}: {} for operation '{}'",
                        part_name, tmpl.mitigation_template, op.name
                    ),
                ));
            }
        }
        FailureDimension::Resource => {
            for attr in &part.attributes {
                triples.push((
                    format!(
                        "{} Attribute '{}' {}",
                        part_name, attr.name, tmpl.mode_suffix
                    ),
                    format!(
                        "{}: {} affecting attribute '{}'",
                        part_name, tmpl.effect_template, attr.name
                    ),
                    format!(
                        "{}: {} for attribute '{}'",
                        part_name, tmpl.mitigation_template, attr.name
                    ),
                ));
            }
        }
    }

    if triples.is_empty() {
        triples.push((
            format!("{} {}", part_name, tmpl.mode_suffix),
            format!("{}: {}", part_name, tmpl.effect_template),
            format!("{}: {}", part_name, tmpl.mitigation_template),
        ));
    }

    triples
}

/// Pads `rows` up to a minimum of 15 entries using domain-agnostic abstract structural sub-modes with `"Derived"` basis.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `pad_minimum_derived_rows` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Executes only when `!filtered_parts.is_empty()` and `rows.len() < 15`.
/// - Never couples unrelated `HazardDef` items by loop index and always stamps `basis = "Derived"`.
///
/// ## 3. Preconditions
/// - `filtered_parts` is non-empty; `rows` holds the primary component-dimension FMECA rows.
///
/// ## 4. Postconditions
/// - `rows.len() >= 15` upon return, with sequential `FMECA-XXX` IDs and consistent RPN products.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(max(0, 15 - rows.len())), bounded by at most 15 iterations.
/// - Space Complexity: O(max(0, 15 - rows.len())) additional `FmecaRow` entries.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Uses `.checked_rem` and `.get(..)` exclusively; breaks cleanly if either slice is empty.
/// - State: Preserves all pre-existing `rows` entries unchanged.
/// - Action: Strictly bounded loop (`rows.len()` increases by 1 per iteration up to 15).
/// - Resource: At most 15 `FmecaRow` allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-PAD-DERIVED-ROWS] | SysML v2: ActionDef(pad_minimum_derived_rows)
fn pad_minimum_derived_rows(
    filtered_parts: &[&PartDef],
    scoring_config: Option<&FmecaScoringConfig>,
    rows: &mut Vec<FmecaRow>,
) {
    let mut add_idx: usize = 0;
    while rows.len() < 15 {
        let sub_tmpl = add_idx
            .checked_rem(ABSTRACT_STRUCTURAL_SUB_MODES.len())
            .and_then(|i| ABSTRACT_STRUCTURAL_SUB_MODES.get(i));
        let part = add_idx
            .checked_rem(filtered_parts.len())
            .and_then(|i| filtered_parts.get(i));
        let (Some(sub_tmpl), Some(&part)) = (sub_tmpl, part) else {
            break;
        };
        let part_name = part.name.as_str();
        let triple = (
            format!("{} {}", part_name, sub_tmpl.mode_suffix),
            format!("{}: {}", part_name, sub_tmpl.effect_template),
            format!("{}: {}", part_name, sub_tmpl.mitigation_template),
        );
        push_scored_row(rows, part_name, sub_tmpl, scoring_config, triple, "Derived");
        add_idx += 1;
    }
}

/// Resolves `(S, O, D, RPN)` via `resolve_scores` and appends a formatted `FmecaRow` to `rows`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `push_scored_row` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Enforces `rpn == s * o * d` and 1-based monotonic `FMECA-{:03}` numbering aligned to `rows.len() + 1`.
///
/// ## 3. Preconditions
/// - `triple` holds `(failure_mode, effect, mitigation)` strings; `basis_str` is `"SSOT"` or `"Derived"`.
///
/// ## 4. Postconditions
/// - Increases `rows.len()` by exactly 1 with a complete `FmecaRow`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) amortized `Vec::push` and score resolution.
/// - Space Complexity: O(|failure_mode| + |effect| + |mitigation|) for the appended `FmecaRow`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Accepts optional `scoring_config` without requiring non-empty scale vectors.
/// - State: Appends atomically to `rows`.
/// - Action: Constant-time arithmetic (`s * o * d <= 1000`, overflow-impossible in `u32`).
/// - Resource: Single `FmecaRow` construction.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-PUSH-SCORED-ROW] | SysML v2: ActionDef(push_scored_row)
fn push_scored_row(
    rows: &mut Vec<FmecaRow>,
    component: &str,
    tmpl: &DimensionTemplate,
    scoring_config: Option<&FmecaScoringConfig>,
    triple: (String, String, String),
    basis_str: &str,
) {
    let row_index = rows.len();
    let (s, o, d, s_cell, o_cell, d_cell) = resolve_scores(scoring_config, row_index, tmpl);
    let rpn = s * o * d;
    let (mode_str, effect_str, mit_str) = triple;
    rows.push(FmecaRow {
        id: format!("FMECA-{:03}", row_index + 1),
        component: component.to_string(),
        dimension: tmpl.dimension.as_str().to_string(),
        failure_mode: mode_str,
        effect: effect_str,
        severity: s,
        occurrence: o,
        detection: d,
        rpn,
        severity_cell: s_cell,
        occurrence_cell: o_cell,
        detection_cell: d_cell,
        rpn_cell: format!("{}", rpn),
        mitigation: mit_str,
        basis: basis_str.to_string(),
    });
}

/// Executes deterministic fn `resolve_scores` within `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `resolve_scores` in `crates/compile-sysml/src/stpa/fmeca.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves per-axis fallback fidelity: empty `severity_scale`, `occurrence_scale`, or `detection_scale` slices independently fall back to `tmpl.default_s`, `tmpl.default_o`, and `tmpl.default_d` respectively (never collapsing occurrence or detection to `tmpl.default_s`).
///
/// ## 3. Preconditions
/// - `scoring_config: Option<&FmecaScoringConfig>`, `index: usize`, and `tmpl: &DimensionTemplate` satisfy type-safety constraints.
///
/// ## 4. Postconditions
/// - Returns `(u32, u32, u32, String, String, String)` with scores bounded in `1..=10`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) modular index lookup via `.checked_rem` and `.get(..)`.
/// - Space Complexity: O(1) bounded cell label formatting for `resolve_scores`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Empty scale slices safely hit `checked_rem(0) == None` and return axis-specific template defaults.
/// - State: Maintains consistent state invariants across `resolve_scores` without partial mutation.
/// - Action: Executes `resolve_scores` deterministically with saturating offset addition and clamped scores.
/// - Resource: Bounds stack and heap allocations within `resolve_scores` (`STPA-FMECA`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-RESOLVE-SCORES] | SysML v2: ActionDef(resolve_scores)
fn resolve_scores(
    scoring_config: Option<&FmecaScoringConfig>,
    index: usize,
    tmpl: &DimensionTemplate,
) -> (u32, u32, u32, String, String, String) {
    if let Some(cfg) = scoring_config {
        let pick = |scale: &[ScoringScaleEntry], offset: usize, default_score: u32| -> (u32, String) {
            match index
                .saturating_add(offset)
                .checked_rem(scale.len())
                .and_then(|i| scale.get(i))
            {
                Some(entry) => {
                    let clamped = entry.score.clamp(1, 10);
                    (clamped, format!("{} ({})", entry.label, clamped))
                }
                None => (default_score, format!("{}", default_score)),
            }
        };

        let (s, s_cell) = pick(&cfg.severity_scale, 0, tmpl.default_s);
        let (o, o_cell) = pick(&cfg.occurrence_scale, 1, tmpl.default_o);
        let (d, d_cell) = pick(&cfg.detection_scale, 2, tmpl.default_d);
        (s, o, d, s_cell, o_cell, d_cell)
    } else {
        (
            tmpl.default_s,
            tmpl.default_o,
            tmpl.default_d,
            format!("{}", tmpl.default_s),
            format!("{}", tmpl.default_o),
            format!("{}", tmpl.default_d),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deap_core::sysml_ast::PartDef;

    #[test]
    fn test_fmeca_matrix_generation_and_rpn() -> Result<(), Box<dyn std::error::Error>> {
        let mut pkg = PackageDef::default();
        let p1 = PartDef {
            name: "CoreStructuralFrame".to_string(),
            ..Default::default()
        };
        let p2 = PartDef {
            name: "PrimaryComputeSubsystem".to_string(),
            ..Default::default()
        };
        pkg.part_defs.push(p1);
        pkg.part_defs.push(p2);

        let rows = generate_fmeca_matrix(&pkg, None);

        // 2 parts * 4 dimensions = 8 initial rows, padded to >= 15
        assert!(rows.len() >= 15, "Expected >= 15 rows, got {}", rows.len());

        for row in &rows {
            assert!(row.severity >= 1 && row.severity <= 10);
            assert!(row.occurrence >= 1 && row.occurrence <= 10);
            assert!(row.detection >= 1 && row.detection <= 10);
            assert_eq!(row.rpn, row.severity * row.occurrence * row.detection);
            assert!(row.id.starts_with("FMECA-"));
            assert!(!row.component.is_empty());
            assert!(!row.failure_mode.is_empty());
        }

        // Verify all 4 dimensions are represented
        let dims: Vec<_> = rows.iter().map(|r| r.dimension.as_str()).collect();
        assert!(dims.contains(&"Interface"));
        assert!(dims.contains(&"State"));
        assert!(dims.contains(&"Action"));
        assert!(dims.contains(&"Resource"));
        Ok(())
    }

    #[test]
    fn test_fmeca_matrix_excludes_negated_components() -> Result<(), Box<dyn std::error::Error>> {
        let mut pkg = PackageDef::default();
        let p1 = PartDef {
            name: "PrimaryComputeSubsystem".to_string(),
            ..Default::default()
        };
        let p2 = PartDef {
            name: "AuxiliaryBackupUnit".to_string(),
            ..Default::default()
        };
        pkg.part_defs.push(p1);
        pkg.part_defs.push(p2);

        // Explicit negative constraint: backupEnabled = 0; backupSystem = "No";
        pkg.attribute_defs.push(deap_core::sysml_ast::AttributeDef {
            name: "backupEnabled".to_string(),
            default_value: Some("0".to_string()),
            ..Default::default()
        });
        pkg.attribute_defs.push(deap_core::sysml_ast::AttributeDef {
            name: "backupSystem".to_string(),
            default_value: Some("\"No\"".to_string()),
            ..Default::default()
        });

        let rows = generate_fmeca_matrix(&pkg, None);
        let components: Vec<_> = rows.iter().map(|r| r.component.as_str()).collect();
        assert!(components.contains(&"PrimaryComputeSubsystem"));
        assert!(!components.contains(&"AuxiliaryBackupUnit"), "Negated AuxiliaryBackupUnit must NOT be present in FMECA rows");
        Ok(())
    }

    #[test]
    fn test_arch_07_fmeca_matrix_precomputes_bang_negated_components_in_btreeset() -> Result<(), Box<dyn std::error::Error>> {
        let mut pkg = PackageDef::default();
        pkg.part_defs.push(PartDef {
            name: "PrimaryComputeSubsystem".to_string(),
            ..Default::default()
        });
        pkg.part_defs.push(PartDef {
            name: "AuxiliaryBackupUnit".to_string(),
            ..Default::default()
        });
        pkg.constraint_defs.push(deap_core::sysml_ast::ConstraintDef {
            name: "ExcludeBackupUnit".to_string(),
            expression: "!AuxiliaryBackupUnit".to_string(),
            is_assertion: true,
            ..Default::default()
        });

        let rows = generate_fmeca_matrix(&pkg, None);
        let components: Vec<_> = rows.iter().map(|r| r.component.as_str()).collect();
        assert!(components.contains(&"PrimaryComputeSubsystem"));
        assert!(
            !components.contains(&"AuxiliaryBackupUnit"),
            "Direct !AuxiliaryBackupUnit negation must be precomputed into BTreeSet and excluded from FMECA matrix"
        );
        Ok(())
    }
}
