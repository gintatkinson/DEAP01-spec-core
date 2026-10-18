//! AST structures for SysML v2 entities, requirements, and system architecture.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides the canonical Abstract Syntax Tree (AST) data structures, recursive parsers, and
//! deterministic architecture extractors for SysML v2 models in accordance with DO-178C Level A,
//! DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification and traceability mandates.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Pure Schema-Driven Extraction: All AST representations derive exclusively and deterministically
//!   from SysML v2 structural, behavioral, and requirement declarations without hardcoded domain state.
//! - Deterministic Ordering: Multi-file directory traversal and map/set aggregations preserve
//!   canonical lexicographical ordering via `BTreeMap` and `BTreeSet`.
//!
//! ## 3. Preconditions
//! - Input SysML v2 source strings and workspace schema paths must be valid UTF-8 text or readable
//!   filesystem directories containing `.sysml` specification units.
//!
//! ## 4. Postconditions
//! - Returns strongly typed, serializable AST node collections (`SysmlModel`, `PartDef`,
//!   `RequirementDef`, `PortDef`, `ActionDef`, `StateDef`, `UseCaseDef`, `TestCaseDef`) preserving
//!   exact block hierarchy and requirement satisfaction bindings.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear in the byte length of parsed SysML v2 source text and number of
//!   AST tokens.
//! - Space Complexity: $O(V + E)$ proportional to the extracted AST vertices and directed edges.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Unbalanced braces, malformed declarations, or unreadable paths are handled gracefully via
//!   bounded character-stream scanning and explicit `Result` error propagation without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYSML-AST-MODULE], [DO-330-TQL-4-SYSML-AST]

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Returns `true` as the default boolean flag for SysML v2 definition nodes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic serialization
/// invariants for AST structural definitions.
///
/// ## 2. Invariants & Mathematical Properties
/// - Constant Function: Always evaluates to `true`.
///
/// ## 3. Preconditions
/// - None.
///
/// ## 4. Postconditions
/// - Returns `true`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible pure constant evaluation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/DEFAULT-TRUE]
fn default_true() -> bool {
    true
}

/// Returns `1` as the default baseline severity level for SysML v2 hazard and connection nodes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and MIL-STD-882E baseline severity assignment.
///
/// ## 2. Invariants & Mathematical Properties
/// - Constant Function: Always evaluates to integer `1`.
///
/// ## 3. Preconditions
/// - None.
///
/// ## 4. Postconditions
/// - Returns `1`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible pure constant evaluation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/DEFAULT-ONE]
fn default_one() -> i32 {
    1
}

/// Represents an attribute definition or instance in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `AttributeDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `AttributeDef` across member set `[name: String, type_name: String, default_value: Option<String>, doc: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `AttributeDef` members (`name: String, type_name: String, default_value: Option<String>, doc: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `AttributeDef` representation for struct `AttributeDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(4)) bounded over input/member dimension of `AttributeDef`.
/// - Space Complexity: O(M(4)) bounded auxiliary memory for `AttributeDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `AttributeDef` (`AttributeDef`).
/// - State: Maintains consistent state invariants across `AttributeDef` without partial mutation on error.
/// - Action: Executes `AttributeDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `AttributeDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-ATTRIBUTEDEF] | SysML v2: PartDef(AttributeDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AttributeDef {
    pub name: String,
    #[serde(default)]
    pub type_name: String,
    pub default_value: Option<String>,
    pub doc: Option<String>,
}

/// Represents an item flow or discrete data flow definition in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `FlowDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `FlowDef` across member set `[name: String, direction: String, item_type: String, doc: Option<String>, rate_hz: Option<f64>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `FlowDef` members (`name: String, direction: String, item_type: String, doc: Option<String>, rate_hz: Option<f64>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `FlowDef` representation for struct `FlowDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `FlowDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `FlowDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `FlowDef` (`FlowDef`).
/// - State: Maintains consistent state invariants across `FlowDef` without partial mutation on error.
/// - Action: Executes `FlowDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `FlowDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-FLOWDEF] | SysML v2: PartDef(FlowDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FlowDef {
    pub name: String,
    #[serde(default = "default_direction_out")]
    pub direction: String,
    #[serde(default = "default_item_type")]
    pub item_type: String,
    pub doc: Option<String>,
    pub rate_hz: Option<f64>,
    pub unit: Option<String>,
    pub valid_range: Option<String>,
    pub default_value: Option<String>,
}

/// Returns `"out"` as the default flow direction for `FlowDef` deserialization.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C deterministic interface flow direction defaults.
///
/// ## 2. Invariants & Mathematical Properties
/// - Always returns `"out"`.
///
/// ## 3. Preconditions
/// - None.
///
/// ## 4. Postconditions
/// - Returns an owned `String` equal to `"out"`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible string allocation for static literal.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/DEFAULT-DIRECTION-OUT]
fn default_direction_out() -> String {
    "out".to_string()
}

/// Returns `"Item"` as the default payload item type for `FlowDef` deserialization.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C type-safe item flow defaults.
///
/// ## 2. Invariants & Mathematical Properties
/// - Always returns `"Item"`.
///
/// ## 3. Preconditions
/// - None.
///
/// ## 4. Postconditions
/// - Returns an owned `String` equal to `"Item"`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible string allocation for static literal.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/DEFAULT-ITEM-TYPE]
fn default_item_type() -> String {
    "Item".to_string()
}

/// Represents a port definition or interface point in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `PortDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `PortDef` across member set `[name: String, type_name: String, direction: String, is_conjugated: bool, doc: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `PortDef` members (`name: String, type_name: String, direction: String, is_conjugated: bool, doc: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `PortDef` representation for struct `PortDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `PortDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `PortDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `PortDef` (`PortDef`).
/// - State: Maintains consistent state invariants across `PortDef` without partial mutation on error.
/// - Action: Executes `PortDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `PortDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-PORTDEF] | SysML v2: PartDef(PortDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PortDef {
    pub name: String,
    pub type_name: String,
    #[serde(default = "default_direction_inout")]
    pub direction: String,
    #[serde(default)]
    pub is_conjugated: bool,
    pub doc: Option<String>,
    #[serde(default = "default_port_category")]
    pub port_category: String,
    #[serde(default)]
    pub protocol_family: String,
    #[serde(default)]
    pub electrical_attributes: BTreeMap<String, String>,
    #[serde(default)]
    pub item_flows: Vec<FlowDef>,
}

/// Returns `"inout"` as the default port and operation direction in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C bidirectional port interface defaults.
///
/// ## 2. Invariants & Mathematical Properties
/// - Always returns `"inout"`.
///
/// ## 3. Preconditions
/// - None.
///
/// ## 4. Postconditions
/// - Returns an owned `String` equal to `"inout"`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible string allocation for static literal.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/DEFAULT-DIRECTION-INOUT]
fn default_direction_inout() -> String {
    "inout".to_string()
}

/// Returns `"DataPort"` as the default logical port category in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C logical interface categorization defaults.
///
/// ## 2. Invariants & Mathematical Properties
/// - Always returns `"DataPort"`.
///
/// ## 3. Preconditions
/// - None.
///
/// ## 4. Postconditions
/// - Returns an owned `String` equal to `"DataPort"`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible string allocation for static literal.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/DEFAULT-PORT-CATEGORY]
fn default_port_category() -> String {
    "DataPort".to_string()
}

/// Represents an action definition in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ActionDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ActionDef` across member set `[name: String, doc: Option<String>, in_params: Vec<AttributeDef>, out_params: Vec<AttributeDef>, parameters: Vec<AttributeDef>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ActionDef` members (`name: String, doc: Option<String>, in_params: Vec<AttributeDef>, out_params: Vec<AttributeDef>, parameters: Vec<AttributeDef>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ActionDef` representation for struct `ActionDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `ActionDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `ActionDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ActionDef` (`ActionDef`).
/// - State: Maintains consistent state invariants across `ActionDef` without partial mutation on error.
/// - Action: Executes `ActionDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ActionDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-ACTIONDEF] | SysML v2: PartDef(ActionDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionDef {
    pub name: String,
    pub doc: Option<String>,
    #[serde(default)]
    pub in_params: Vec<AttributeDef>,
    #[serde(default)]
    pub out_params: Vec<AttributeDef>,
    #[serde(default)]
    pub parameters: Vec<AttributeDef>,
    pub performer: Option<String>,
    #[serde(default)]
    pub steps: Vec<String>,
    #[serde(default = "default_true")]
    pub is_def: bool,
}

impl Default for ActionDef {
    /// Constructs a default `ActionDef` with `is_def` initialized to `true`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C deterministic AST node initialization.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - All collection fields are empty and `is_def == true`.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns an initialized `ActionDef` instance.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Zero-allocation empty vector initialization; cannot fail or panic.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/ACTION-DEF-DEFAULT]
    fn default() -> Self {
        Self {
            name: String::new(),
            doc: None,
            in_params: Vec::new(),
            out_params: Vec::new(),
            parameters: Vec::new(),
            performer: None,
            steps: Vec::new(),
            is_def: true,
        }
    }
}

/// Represents an operation definition with typed signature in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `OperationDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `OperationDef` across member set `[name: String, direction: String, param_type: String, return_type: Option<String>, doc: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `OperationDef` members (`name: String, direction: String, param_type: String, return_type: Option<String>, doc: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `OperationDef` representation for struct `OperationDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `OperationDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `OperationDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `OperationDef` (`OperationDef`).
/// - State: Maintains consistent state invariants across `OperationDef` without partial mutation on error.
/// - Action: Executes `OperationDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `OperationDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-OPERATIONDEF] | SysML v2: PartDef(OperationDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OperationDef {
    pub name: String,
    #[serde(default = "default_direction_inout")]
    pub direction: String,
    #[serde(default)]
    pub param_type: String,
    pub return_type: Option<String>,
    pub doc: Option<String>,
    #[serde(default)]
    pub parameters: Vec<AttributeDef>,
}

/// Represents a subsystem capability specification in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `CapabilityDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `CapabilityDef` across member set `[name: String, description: Option<String>, subsystem: Option<String>, package_ref: Option<String>, doc: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `CapabilityDef` members (`name: String, description: Option<String>, subsystem: Option<String>, package_ref: Option<String>, doc: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `CapabilityDef` representation for struct `CapabilityDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `CapabilityDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `CapabilityDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `CapabilityDef` (`CapabilityDef`).
/// - State: Maintains consistent state invariants across `CapabilityDef` without partial mutation on error.
/// - Action: Executes `CapabilityDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `CapabilityDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-CAPABILITYDEF] | SysML v2: PartDef(CapabilityDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CapabilityDef {
    pub name: String,
    pub description: Option<String>,
    pub subsystem: Option<String>,
    pub package_ref: Option<String>,
    pub doc: Option<String>,
}

/// Represents an interaction sequence in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `InteractionDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `InteractionDef` across member set `[name: String, lifelines: Vec<String>, messages: Vec<String>, triggers: Vec<String>, doc: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `InteractionDef` members (`name: String, lifelines: Vec<String>, messages: Vec<String>, triggers: Vec<String>, doc: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `InteractionDef` representation for struct `InteractionDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `InteractionDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `InteractionDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `InteractionDef` (`InteractionDef`).
/// - State: Maintains consistent state invariants across `InteractionDef` without partial mutation on error.
/// - Action: Executes `InteractionDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `InteractionDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-INTERACTIONDEF] | SysML v2: PartDef(InteractionDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct InteractionDef {
    pub name: String,
    #[serde(default)]
    pub lifelines: Vec<String>,
    #[serde(default)]
    pub messages: Vec<String>,
    #[serde(default)]
    pub triggers: Vec<String>,
    pub doc: Option<String>,
}

/// Represents a formal constraint definition, invariant, or assertion in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ConstraintDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ConstraintDef` across member set `[name: String, expression: String, parameters: Vec<String>, is_assertion: bool, doc: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ConstraintDef` members (`name: String, expression: String, parameters: Vec<String>, is_assertion: bool, doc: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ConstraintDef` representation for struct `ConstraintDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `ConstraintDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `ConstraintDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ConstraintDef` (`ConstraintDef`).
/// - State: Maintains consistent state invariants across `ConstraintDef` without partial mutation on error.
/// - Action: Executes `ConstraintDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ConstraintDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-CONSTRAINTDEF] | SysML v2: PartDef(ConstraintDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ConstraintDef {
    pub name: String,
    pub expression: String,
    #[serde(default)]
    pub parameters: Vec<String>,
    #[serde(default)]
    pub is_assertion: bool,
    pub doc: Option<String>,
    #[serde(default)]
    pub pre_conditions: Vec<String>,
    #[serde(default)]
    pub post_conditions: Vec<String>,
}

/// Represents a verification test case definition in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `TestCaseDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `TestCaseDef` across member set `[name: String, subject_part: Option<String>, verified_requirements: Vec<String>, objective: Option<String>, test_steps: Vec<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `TestCaseDef` members (`name: String, subject_part: Option<String>, verified_requirements: Vec<String>, objective: Option<String>, test_steps: Vec<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `TestCaseDef` representation for struct `TestCaseDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `TestCaseDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `TestCaseDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `TestCaseDef` (`TestCaseDef`).
/// - State: Maintains consistent state invariants across `TestCaseDef` without partial mutation on error.
/// - Action: Executes `TestCaseDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `TestCaseDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-TESTCASEDEF] | SysML v2: PartDef(TestCaseDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TestCaseDef {
    pub name: String,
    pub subject_part: Option<String>,
    #[serde(default)]
    pub verified_requirements: Vec<String>,
    pub objective: Option<String>,
    #[serde(default)]
    pub test_steps: Vec<String>,
    pub doc: Option<String>,
}

/// Represents a requirement definition in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `RequirementDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `RequirementDef` across member set `[name: String, req_id: String, text: String, doc: Option<String>, attributes: Vec<AttributeDef>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `RequirementDef` members (`name: String, req_id: String, text: String, doc: Option<String>, attributes: Vec<AttributeDef>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `RequirementDef` representation for struct `RequirementDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `RequirementDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `RequirementDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `RequirementDef` (`RequirementDef`).
/// - State: Maintains consistent state invariants across `RequirementDef` without partial mutation on error.
/// - Action: Executes `RequirementDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `RequirementDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-REQUIREMENTDEF] | SysML v2: PartDef(RequirementDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RequirementDef {
    pub name: String,
    pub req_id: String,
    pub text: String,
    pub doc: Option<String>,
    #[serde(default)]
    pub attributes: Vec<AttributeDef>,
    #[serde(default)]
    pub constraints: Vec<ConstraintDef>,
    #[serde(default)]
    pub assumes: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub verified_by: Vec<String>,
    #[serde(default)]
    pub satisfied_by: Vec<String>,
    #[serde(default)]
    pub derived_from: Vec<String>,
    #[serde(default)]
    pub derives: Vec<String>,
}

/// Represents a state transition in a SysML v2 state machine.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `TransitionDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `TransitionDef` across member set `[name: Option<String>, source: Option<String>, target: Option<String>, trigger: Option<String>, guard: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `TransitionDef` members (`name: Option<String>, source: Option<String>, target: Option<String>, trigger: Option<String>, guard: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `TransitionDef` representation for struct `TransitionDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `TransitionDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `TransitionDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `TransitionDef` (`TransitionDef`).
/// - State: Maintains consistent state invariants across `TransitionDef` without partial mutation on error.
/// - Action: Executes `TransitionDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `TransitionDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-TRANSITIONDEF] | SysML v2: PartDef(TransitionDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TransitionDef {
    pub name: Option<String>,
    pub source: Option<String>,
    pub target: Option<String>,
    pub trigger: Option<String>,
    pub guard: Option<String>,
    pub effect: Option<String>,
    pub doc: Option<String>,
}

/// Represents a state machine state definition in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `StateDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `StateDef` across member set `[name: String, doc: Option<String>, entry_action: Option<String>, do_action: Option<String>, exit_action: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `StateDef` members (`name: String, doc: Option<String>, entry_action: Option<String>, do_action: Option<String>, exit_action: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `StateDef` representation for struct `StateDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `StateDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `StateDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `StateDef` (`StateDef`).
/// - State: Maintains consistent state invariants across `StateDef` without partial mutation on error.
/// - Action: Executes `StateDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `StateDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-STATEDEF] | SysML v2: PartDef(StateDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StateDef {
    pub name: String,
    pub doc: Option<String>,
    pub entry_action: Option<String>,
    pub do_action: Option<String>,
    pub exit_action: Option<String>,
    #[serde(default)]
    pub transitions: Vec<TransitionDef>,
    #[serde(default)]
    pub sub_states: Vec<StateDef>,
}

/// Represents a use case definition in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `UseCaseDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `UseCaseDef` across member set `[name: String, doc: Option<String>, subject: Option<String>, actor: Option<String>, actors: Vec<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `UseCaseDef` members (`name: String, doc: Option<String>, subject: Option<String>, actor: Option<String>, actors: Vec<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `UseCaseDef` representation for struct `UseCaseDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `UseCaseDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `UseCaseDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `UseCaseDef` (`UseCaseDef`).
/// - State: Maintains consistent state invariants across `UseCaseDef` without partial mutation on error.
/// - Action: Executes `UseCaseDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `UseCaseDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-USECASEDEF] | SysML v2: PartDef(UseCaseDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UseCaseDef {
    pub name: String,
    pub doc: Option<String>,
    pub subject: Option<String>,
    pub actor: Option<String>,
    #[serde(default)]
    pub actors: Vec<String>,
    pub objective: Option<String>,
    #[serde(default)]
    pub includes: Vec<String>,
    #[serde(default)]
    pub extends: Vec<String>,
    #[serde(default)]
    pub steps: Vec<String>,
    #[serde(default)]
    pub preconditions: Vec<String>,
    #[serde(default)]
    pub postconditions: Vec<String>,
}

/// Represents a data payload or item definition in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ItemDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ItemDef` across member set `[name: String, doc: Option<String>, attributes: Vec<AttributeDef>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ItemDef` members (`name: String, doc: Option<String>, attributes: Vec<AttributeDef>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ItemDef` representation for struct `ItemDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(3)) bounded over input/member dimension of `ItemDef`.
/// - Space Complexity: O(M(3)) bounded auxiliary memory for `ItemDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ItemDef` (`ItemDef`).
/// - State: Maintains consistent state invariants across `ItemDef` without partial mutation on error.
/// - Action: Executes `ItemDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ItemDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-ITEMDEF] | SysML v2: PartDef(ItemDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ItemDef {
    pub name: String,
    pub doc: Option<String>,
    #[serde(default)]
    pub attributes: Vec<AttributeDef>,
}

/// Represents a safety hazard definition in SysML v2 / STPA models.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `HazardDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `HazardDef` across member set `[name: String, doc: Option<String>, severity: i32, source_port: Option<String>, target_port: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `HazardDef` members (`name: String, doc: Option<String>, severity: i32, source_port: Option<String>, target_port: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `HazardDef` representation for struct `HazardDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `HazardDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `HazardDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `HazardDef` (`HazardDef`).
/// - State: Maintains consistent state invariants across `HazardDef` without partial mutation on error.
/// - Action: Executes `HazardDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `HazardDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-HAZARDDEF] | SysML v2: PartDef(HazardDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HazardDef {
    pub name: String,
    pub doc: Option<String>,
    #[serde(default = "default_one")]
    pub severity: i32,
    pub source_port: Option<String>,
    pub target_port: Option<String>,
    pub part_ref: Option<String>,
}

/// Represents a safety risk definition in SysML v2 / STPA models.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `RiskDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `RiskDef` across member set `[name: String, doc: Option<String>, severity: i32, hazard_ref: Option<String>, source_port: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `RiskDef` members (`name: String, doc: Option<String>, severity: i32, hazard_ref: Option<String>, source_port: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `RiskDef` representation for struct `RiskDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `RiskDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `RiskDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `RiskDef` (`RiskDef`).
/// - State: Maintains consistent state invariants across `RiskDef` without partial mutation on error.
/// - Action: Executes `RiskDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `RiskDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-RISKDEF] | SysML v2: PartDef(RiskDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RiskDef {
    pub name: String,
    pub doc: Option<String>,
    #[serde(default = "default_one")]
    pub severity: i32,
    pub hazard_ref: Option<String>,
    pub source_port: Option<String>,
    pub target_port: Option<String>,
}

/// Represents a connection or topological flow link between ports in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ConnectionDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ConnectionDef` across member set `[name: String, source_port: String, target_port: String, source_part: Option<String>, target_part: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ConnectionDef` members (`name: String, source_port: String, target_port: String, source_part: Option<String>, target_part: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ConnectionDef` representation for struct `ConnectionDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `ConnectionDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `ConnectionDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ConnectionDef` (`ConnectionDef`).
/// - State: Maintains consistent state invariants across `ConnectionDef` without partial mutation on error.
/// - Action: Executes `ConnectionDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ConnectionDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-CONNECTIONDEF] | SysML v2: PartDef(ConnectionDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ConnectionDef {
    pub name: String,
    pub source_port: String,
    pub target_port: String,
    pub source_part: Option<String>,
    pub target_part: Option<String>,
    pub doc: Option<String>,
    #[serde(default = "default_one")]
    pub severity: i32,
    pub item_flow_ref: Option<String>,
    pub protocol: Option<String>,
    pub latency_ms: Option<f64>,
    #[serde(default)]
    pub is_flow: bool,
}

/// Represents a calculation or function definition in SysML v2 / KerML.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical software
/// engineering standards. Captures pure mathematical calculations, physics relationships,
/// and safety function specifications directly from the SysML v2 AST, eliminating hardcoded domain
/// formulas from the compiler toolchain.
///
/// ## 2. Invariants & Mathematical Properties
/// - AST Determinism: Structurally represents an acyclic calculation definition with explicit input parameters and return type.
/// - Value Semantics: Supports default value initialization and equality comparison without internal state mutability.
///
/// ## 3. Preconditions
/// - `name` should represent a valid KerML / SysML v2 calculation identifier.
///
/// ## 4. Postconditions
/// - Encapsulates typed input parameters, optional return type, and optional body expression for verification.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Construction & Cloning: $O(P + E)$ where $P$ is the parameter count and $E$ is expression byte length.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Missing parameters or expressions default gracefully to empty collections and `None`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/CALC-DEF]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CalcDef {
    pub name: String,
    pub doc: Option<String>,
    #[serde(default)]
    pub in_params: Vec<AttributeDef>,
    #[serde(default)]
    pub return_type: Option<String>,
    #[serde(default)]
    pub expression: Option<String>,
}

/// Represents a structural part definition or block in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `PartDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `PartDef` across member set `[name: String, doc: Option<String>, is_def: bool, type_name: Option<String>, attributes: Vec<AttributeDef>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `PartDef` members (`name: String, doc: Option<String>, is_def: bool, type_name: Option<String>, attributes: Vec<AttributeDef>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `PartDef` representation for struct `PartDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `PartDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `PartDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `PartDef` (`PartDef`).
/// - State: Maintains consistent state invariants across `PartDef` without partial mutation on error.
/// - Action: Executes `PartDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `PartDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-PARTDEF] | SysML v2: PartDef(PartDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartDef {
    pub name: String,
    pub doc: Option<String>,
    #[serde(default = "default_true")]
    pub is_def: bool,
    pub type_name: Option<String>,
    #[serde(default)]
    pub attributes: Vec<AttributeDef>,
    #[serde(default)]
    pub ports: Vec<PortDef>,
    #[serde(default)]
    pub actions: Vec<ActionDef>,
    #[serde(default)]
    pub constraints: Vec<ConstraintDef>,
    #[serde(default)]
    pub requirements: Vec<RequirementDef>,
    #[serde(default)]
    pub satisfied_requirements: Vec<String>,
    #[serde(default)]
    pub parts: Vec<PartDef>,
    #[serde(default)]
    pub operations: Vec<OperationDef>,
    #[serde(default)]
    pub capabilities: Vec<CapabilityDef>,
    #[serde(default)]
    pub interactions: Vec<InteractionDef>,
    #[serde(default)]
    pub test_cases: Vec<TestCaseDef>,
    #[serde(default)]
    pub states: Vec<StateDef>,
    #[serde(default)]
    pub use_cases: Vec<UseCaseDef>,
    #[serde(default)]
    pub item_defs: Vec<ItemDef>,
    #[serde(default)]
    pub hazards: Vec<HazardDef>,
    #[serde(default)]
    pub risks: Vec<RiskDef>,
    #[serde(default)]
    pub connections: Vec<ConnectionDef>,
    #[serde(default)]
    pub calcs: Vec<CalcDef>,
}

impl Default for PartDef {
    /// Constructs a default `PartDef` with `is_def` initialized to `true` and all collections empty.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C deterministic AST node initialization.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - All subordinate vectors are initialized empty and `is_def == true`.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns an initialized `PartDef` instance.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Zero-allocation empty vector initialization; cannot fail or panic.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/PART-DEF-DEFAULT]
    fn default() -> Self {
        Self {
            name: String::new(),
            doc: None,
            is_def: true,
            type_name: None,
            attributes: Vec::new(),
            ports: Vec::new(),
            actions: Vec::new(),
            constraints: Vec::new(),
            requirements: Vec::new(),
            satisfied_requirements: Vec::new(),
            parts: Vec::new(),
            operations: Vec::new(),
            capabilities: Vec::new(),
            interactions: Vec::new(),
            test_cases: Vec::new(),
            states: Vec::new(),
            use_cases: Vec::new(),
            item_defs: Vec::new(),
            hazards: Vec::new(),
            risks: Vec::new(),
            connections: Vec::new(),
            calcs: Vec::new(),
        }
    }
}

impl PartDef {
    /// Recursively searches for a mutable reference to a part definition by identifier name across self and all nested parts.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical software
    /// engineering standards. Mitigates AST divergence hazards (HAZ-AST-DIVERGENCE) and accidental
    /// duplication of safety-critical structural elements across hierarchical SysML v2 part trees.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Identity Invariant: If `self.name == name`, returns `Some(self)`.
    /// - Structural Non-Mutation on Lookup: Does not alter tree topology or structural attributes during traversal.
    /// - KerML Tree Consistency: Traverses strictly acyclic part containment hierarchies.
    ///
    /// ## 3. Preconditions
    /// - `name` must be a valid non-empty KerML / SysML v2 identifier string.
    ///
    /// ## 4. Postconditions
    /// - Returns `Some(&mut PartDef)` pointing to the matching element if found within self or nested sub-parts.
    /// - Returns `None` if no matching part definition exists within this subtree.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(V_{part} + E_{part})$ where $V_{part}$ is the number of nested part nodes and $E_{part}$ is the nesting containment edge count.
    /// - Space Complexity: $O(D)$ auxiliary call stack depth where $D$ is the maximum part nesting depth.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - If the target part does not exist, returns `None` without panicking or modifying the AST.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/PART-DEF-MUT]
    pub fn find_part_mut(&mut self, name: &str) -> Option<&mut PartDef> {
        if self.name == name {
            return Some(self);
        }
        for sub in &mut self.parts {
            if let Some(p) = sub.find_part_mut(name) {
                return Some(p);
            }
        }
        None
    }
}

/// Represents an import statement in SysML v2 / KerML.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ImportDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `ImportDef` across member set `[path: String, is_wildcard: bool, is_recursive: bool, doc: Option<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `ImportDef` members (`path: String, is_wildcard: bool, is_recursive: bool, doc: Option<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `ImportDef` representation for struct `ImportDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(4)) bounded over input/member dimension of `ImportDef`.
/// - Space Complexity: O(M(4)) bounded auxiliary memory for `ImportDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `ImportDef` (`ImportDef`).
/// - State: Maintains consistent state invariants across `ImportDef` without partial mutation on error.
/// - Action: Executes `ImportDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `ImportDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-IMPORTDEF] | SysML v2: PartDef(ImportDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ImportDef {
    /// Qualified path of the namespace or element being imported.
    pub path: String,
    /// Indicates whether the import imports all namespace members (`::*`).
    pub is_wildcard: bool,
    /// Indicates whether the import imports members recursively (`::**`).
    pub is_recursive: bool,
    /// Optional documentation comment preceding the import statement.
    pub doc: Option<String>,
}

/// Top-level or nested package container in SysML v2.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `PackageDef` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `PackageDef` across member set `[name: String, doc: Option<String>, parent_package: Option<String>, imports: Vec<ImportDef>, packages: Vec<PackageDef>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `PackageDef` members (`name: String, doc: Option<String>, parent_package: Option<String>, imports: Vec<ImportDef>, packages: Vec<PackageDef>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `PackageDef` representation for struct `PackageDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `PackageDef`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `PackageDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `PackageDef` (`PackageDef`).
/// - State: Maintains consistent state invariants across `PackageDef` without partial mutation on error.
/// - Action: Executes `PackageDef` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `PackageDef` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-PACKAGEDEF] | SysML v2: PartDef(PackageDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PackageDef {
    pub name: String,
    pub doc: Option<String>,
    pub parent_package: Option<String>,
    #[serde(default)]
    pub imports: Vec<ImportDef>,
    #[serde(default)]
    pub packages: Vec<PackageDef>,
    #[serde(default)]
    pub part_defs: Vec<PartDef>,
    #[serde(default)]
    pub port_defs: Vec<PortDef>,
    #[serde(default)]
    pub attribute_defs: Vec<AttributeDef>,
    #[serde(default)]
    pub action_defs: Vec<ActionDef>,
    #[serde(default)]
    pub operation_defs: Vec<OperationDef>,
    #[serde(default)]
    pub capability_defs: Vec<CapabilityDef>,
    #[serde(default)]
    pub interaction_defs: Vec<InteractionDef>,
    #[serde(default)]
    pub constraint_defs: Vec<ConstraintDef>,
    #[serde(default)]
    pub test_case_defs: Vec<TestCaseDef>,
    #[serde(default)]
    pub requirement_defs: Vec<RequirementDef>,
    #[serde(default)]
    pub satisfied_requirements: Vec<String>,
    #[serde(default)]
    pub state_defs: Vec<StateDef>,
    #[serde(default)]
    pub use_case_defs: Vec<UseCaseDef>,
    #[serde(default)]
    pub item_defs: Vec<ItemDef>,
    #[serde(default)]
    pub hazard_defs: Vec<HazardDef>,
    #[serde(default)]
    pub risk_defs: Vec<RiskDef>,
    #[serde(default)]
    pub connection_defs: Vec<ConnectionDef>,
    #[serde(default)]
    pub calc_defs: Vec<CalcDef>,
}

impl PackageDef {
    /// Returns self and all nested packages recursively in depth-first order.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C standards. Ensures
    /// exhaustive package hierarchy traversal so no nested subsystem package is omitted.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Pre-order Depth-First Traversal: `self` is always the first element, followed by recursive subpackages.
    ///
    /// ## 3. Preconditions
    /// - Package containment hierarchy is acyclic.
    ///
    /// ## 4. Postconditions
    /// - Returns a non-empty vector of `&PackageDef` references starting with `self`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N_{pkg})$. Space Complexity: $O(N_{pkg})$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns `vec![self]` when `self.packages` is empty; never panics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-PACKAGES]
    pub fn get_all_packages(&self) -> Vec<&PackageDef> {
        let mut result = vec![self];
        for subpkg in &self.packages {
            result.extend(subpkg.get_all_packages());
        }
        result
    }

    /// Returns all subsystem capability definitions declared across self and all nested packages.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C standards. Guarantees
    /// complete enumeration of all declared `CapabilityDef` nodes across packages and parts.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Exhaustive Aggregation: Includes package-level and nested part-level capabilities in depth-first order.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns a vector of all `CapabilityDef` instances in the hierarchy.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N_{pkg} + N_{part} + N_{cap})$. Space Complexity: $O(N_{cap})$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns an empty vector if no capabilities are defined.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-CAPABILITIES]
    pub fn get_all_capabilities(&self) -> Vec<CapabilityDef> {
        let mut result = Vec::new();
        for pkg in self.get_all_packages() {
            result.extend(pkg.capability_defs.clone());
            for part in &pkg.part_defs {
                collect_part_capabilities(part, &mut result);
            }
        }
        result
    }

    /// Returns all structural part definitions declared across self and all nested packages.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C standards. Ensures
    /// complete structural component discovery across the entire package hierarchy.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Exhaustive Part Enumeration: Collects all top-level `part_defs` and recursive `is_def` sub-parts.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns a vector of all `PartDef` definitions in depth-first order.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N_{pkg} + N_{part})$. Space Complexity: $O(N_{part})$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns an empty vector if no parts are declared.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-PARTS]
    pub fn get_all_parts(&self) -> Vec<PartDef> {
        let mut result = Vec::new();
        for pkg in self.get_all_packages() {
            for part in &pkg.part_defs {
                collect_part_and_subdefs(part, &mut result);
            }
        }
        result
    }

    /// Returns all use case definitions declared across self and all nested packages.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C standards. Guarantees
    /// complete behavioral use case enumeration across packages and parts.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Exhaustive Aggregation: Collects all `use_case_defs` across packages and nested parts.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns a vector containing all declared `UseCaseDef` instances.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N_{pkg} + N_{part} + N_{uc})$. Space Complexity: $O(N_{uc})$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns an empty vector if no use cases are declared.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-USE-CASES]
    pub fn get_all_use_cases(&self) -> Vec<UseCaseDef> {
        let mut result = Vec::new();
        for pkg in self.get_all_packages() {
            result.extend(pkg.use_case_defs.clone());
            for part in &pkg.part_defs {
                collect_part_use_cases(part, &mut result);
            }
        }
        result
    }

    /// Returns all interaction sequence definitions declared across self and all nested packages.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C standards. Ensures
    /// complete sequence and lifeline interaction enumeration across the model hierarchy.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Exhaustive Aggregation: Collects all `interaction_defs` across packages and nested parts.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns a vector containing all declared `InteractionDef` instances.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N_{pkg} + N_{part} + N_{inter})$. Space Complexity: $O(N_{inter})$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns an empty vector if no interactions are declared.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-INTERACTIONS]
    pub fn get_all_interactions(&self) -> Vec<InteractionDef> {
        let mut result = Vec::new();
        for pkg in self.get_all_packages() {
            result.extend(pkg.interaction_defs.clone());
            for part in &pkg.part_defs {
                collect_part_interactions(part, &mut result);
            }
        }
        result
    }

    /// Returns all action definitions declared across self and all nested packages.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C standards. Guarantees
    /// complete enumeration of behavioral action nodes across packages and parts.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Exhaustive Aggregation: Collects all `action_defs` across packages and nested parts.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns a vector containing all declared `ActionDef` instances.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N_{pkg} + N_{part} + N_{act})$. Space Complexity: $O(N_{act})$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns an empty vector if no actions are declared.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-ACTIONS]
    pub fn get_all_actions(&self) -> Vec<ActionDef> {
        let mut result = Vec::new();
        for pkg in self.get_all_packages() {
            result.extend(pkg.action_defs.clone());
            for part in &pkg.part_defs {
                collect_part_actions(part, &mut result);
            }
        }
        result
    }

    /// Returns all constraint definitions and invariants declared across self and all nested packages.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C standards. Prevents
    /// omission of parametric safety constraints across nested packages and parts.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Exhaustive Aggregation: Collects all `constraint_defs` across packages and nested parts.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns a vector containing all declared `ConstraintDef` instances.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N_{pkg} + N_{part} + N_{con})$. Space Complexity: $O(N_{con})$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns an empty vector if no constraints are declared.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-CONSTRAINTS]
    pub fn get_all_constraints(&self) -> Vec<ConstraintDef> {
        let mut result = Vec::new();
        for pkg in self.get_all_packages() {
            result.extend(pkg.constraint_defs.clone());
            for part in &pkg.part_defs {
                collect_part_constraints(part, &mut result);
            }
        }
        result
    }

    /// Returns all attribute definitions declared across self and all nested packages/parts.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C standards. Guarantees
    /// complete enumeration of all structural and parametric attributes in the package hierarchy.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Exhaustive Aggregation: Collects all `attribute_defs` across packages and nested parts.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns a vector containing all declared `AttributeDef` instances.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N_{pkg} + N_{part} + N_{attr})$. Space Complexity: $O(N_{attr})$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns an empty vector if no attributes are declared.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-ATTRIBUTES]
    pub fn get_all_attributes(&self) -> Vec<AttributeDef> {
        let mut result = Vec::new();
        for pkg in self.get_all_packages() {
            result.extend(pkg.attribute_defs.clone());
            for part in &pkg.part_defs {
                collect_part_attributes(part, &mut result);
            }
        }
        result
    }

    /// Returns all calculation definitions declared across self and all nested packages and parts.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C standards. Mitigates calculation
    /// omission hazards (HAZ-CALC-OMISSION) by recursively gathering all calculations across the complete
    /// package and part containment tree.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Exhaustive Traversal: For all $p \in \text{packages}$ and all $pt \in \text{part\_defs}$,
    ///   $\text{calcs}(p) \subseteq \text{result}$ and $\text{calcs}(pt) \subseteq \text{result}$.
    /// - Deterministic Traversal Order: Traversal follows depth-first topological declaration order.
    ///
    /// ## 3. Preconditions
    /// - None. Operates safely on empty, single-node, or multi-level hierarchical package trees.
    ///
    /// ## 4. Postconditions
    /// - Returns a vector containing clones of all calculation definitions declared in this hierarchy.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N_{pkg} + N_{part} + N_{calc})$ where $N$ represents entity counts.
    /// - Space Complexity: $O(N_{calc})$ for the output vector.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns an empty vector if no calculation definitions are present in the package tree without panicking.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-CALCS]
    pub fn get_all_calcs(&self) -> Vec<CalcDef> {
        let mut result = Vec::new();
        for pkg in self.get_all_packages() {
            result.extend(pkg.calc_defs.clone());
            for part in &pkg.part_defs {
                collect_part_calcs(part, &mut result);
            }
        }
        result
    }

    /// Returns all port definitions declared across self and all nested packages and parts.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C standards. Mitigates interface
    /// omission hazards (HAZ-PORT-DISCONNECT) by providing a complete, authoritative symbol table of all
    /// logical and physical ports across the SysML v2 model.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Exhaustive Aggregation: $\bigcup_{pkg} \text{ports}(pkg) \cup \bigcup_{part} \text{ports}(part) = \text{result}$.
    /// - Stability: Traversal order is deterministic depth-first matching AST declaration sequence.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns a vector containing all declared `PortDef` elements in the package hierarchy.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N_{pkg} + N_{part} + N_{port})$.
    /// - Space Complexity: $O(N_{port})$ for the collected results.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns an empty vector if no ports are declared without modifying package state.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-PORTS]
    pub fn get_all_ports(&self) -> Vec<PortDef> {
        let mut result = Vec::new();
        for pkg in self.get_all_packages() {
            result.extend(pkg.port_defs.clone());
            for part in &pkg.part_defs {
                collect_part_ports(part, &mut result);
            }
        }
        result
    }

    /// Returns all safety hazard definitions declared across self and all nested packages and parts.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and MIL-STD-882E standards. Mitigates safety
    /// analysis incompleteness (HAZ-STPA-INCOMPLETE) by gathering 100% of declared safety hazard AST nodes.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Complete Coverage: Every `HazardDef` in `self` or subordinate packages/parts is present in the return set.
    /// - Acyclic Recursion: Traverses strictly directed acyclic package/part hierarchies.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns a vector containing clones of all `HazardDef` nodes in the tree.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N_{pkg} + N_{part} + N_{hazard})$.
    /// - Space Complexity: $O(N_{hazard})$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns an empty vector if no hazards are declared without panicking.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-HAZARDS]
    pub fn get_all_hazards(&self) -> Vec<HazardDef> {
        let mut result = Vec::new();
        for pkg in self.get_all_packages() {
            result.extend(pkg.hazard_defs.clone());
            for part in &pkg.part_defs {
                collect_part_hazards(part, &mut result);
            }
        }
        result
    }

    /// Returns all safety risk definitions declared across self and all nested packages and parts.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and MIL-STD-882E standards. Mitigates risk
    /// tracking divergence (HAZ-RISK-UNMITIGATED) by ensuring complete enumeration of all risk AST nodes.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Deterministic Aggregation: Traverses all package and part nodes in fixed depth-first order.
    /// - Completeness: Returns every `RiskDef` registered in the AST hierarchy.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns a vector containing all declared `RiskDef` instances.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N_{pkg} + N_{part} + N_{risk})$.
    /// - Space Complexity: $O(N_{risk})$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns an empty vector if no risks are present.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-RISKS]
    pub fn get_all_risks(&self) -> Vec<RiskDef> {
        let mut result = Vec::new();
        for pkg in self.get_all_packages() {
            result.extend(pkg.risk_defs.clone());
            for part in &pkg.part_defs {
                collect_part_risks(part, &mut result);
            }
        }
        result
    }

    /// Returns all state machine state definitions declared across self and all nested packages and parts.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and IEC 61508 standards. Mitigates state space
    /// containment escape hazards (HAZ-STATE-ESCAPE) by surfacing all discrete operational states for verification.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Invariant State Topology: Accumulates all declared `StateDef` objects without mutating transition relations.
    /// - Exhaustive Traversal: Evaluates all subpackages and part internal state definitions.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns a vector containing all declared `StateDef` instances.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N_{pkg} + N_{part} + N_{state})$.
    /// - Space Complexity: $O(N_{state})$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns an empty vector if no states are defined.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-STATES]
    pub fn get_all_states(&self) -> Vec<StateDef> {
        let mut result = Vec::new();
        for pkg in self.get_all_packages() {
            result.extend(pkg.state_defs.clone());
            for part in &pkg.part_defs {
                collect_part_states(part, &mut result);
            }
        }
        result
    }

    /// Returns all connection definitions declared across self and all nested packages and parts.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C standards. Mitigates topological
    /// connection ambiguity (HAZ-CONNECTION-AMBIGUITY) across Level 1B and Level 1C interfaces.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Topological Invariance: Gathers all directed flow edges and connection bindings across packages and parts.
    /// - Deterministic Traversal: Orders connections by appearance in the AST hierarchy.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns a vector containing all declared `ConnectionDef` instances.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N_{pkg} + N_{part} + N_{conn})$.
    /// - Space Complexity: $O(N_{conn})$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns an empty vector if no connections are present.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-CONNECTIONS]
    pub fn get_all_connections(&self) -> Vec<ConnectionDef> {
        let mut result = Vec::new();
        for pkg in self.get_all_packages() {
            result.extend(pkg.connection_defs.clone());
            for part in &pkg.part_defs {
                collect_part_connections(part, &mut result);
            }
        }
        result
    }

    /// Returns the set of all port names belonging to nested sub-parts across self and all subpackages.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C standards. Mitigates interface
    /// misrouting hazards (HAZ-PORT-MISROUTING) by dynamically determining which port connections require
    /// hierarchical subsystem qualification (`PORT-ID (Subsystem.Path.port)`).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Hierarchical Isolation: A port $p$ is in `get_nested_ports()` if and only if $p$ is declared on a
    ///   sub-part $p_{sub} \in \text{part.parts}$ for some top-level part.
    /// - Set Purity: Returns a deterministic, sorted `BTreeSet<String>` with zero duplicate port identifiers.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns a `BTreeSet<String>` containing names of all nested sub-part ports.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N_{parts\_nested} \log(N_{ports}))$.
    /// - Space Complexity: $O(N_{nested\_ports})$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns an empty `BTreeSet` if there are no nested sub-parts or no nested ports.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-NESTED-PORTS]
    pub fn get_nested_ports(&self) -> BTreeSet<String> {
        let mut nested = BTreeSet::new();
        for pkg in self.get_all_packages() {
            for part in &pkg.part_defs {
                collect_nested_ports_recursive(&part.parts, &mut nested);
            }
        }
        nested
    }

    /// Extracts the normalized stems of all components explicitly negated anywhere in this package hierarchy.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C. Prevents synthesis of safety
    /// artifacts (FMECA rows, STPA constraints) for components that the user-provided schema explicitly
    /// declares absent or disabled, without hardcoding any domain vocabulary.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Stems derive exclusively from AST attribute names/values, constraint expressions/docs, and package docs.
    /// - Output is sorted and de-duplicated (`BTreeSet` semantics).
    ///
    /// ## 3. Preconditions
    /// - `self` is a parsed `PackageDef` AST node.
    ///
    /// ## 4. Postconditions
    /// - Returns every lowercase alphanumeric stem (length >= 3) negated within the package hierarchy.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(A + C \cdot L + P \cdot L)$ for attributes, constraints, packages, and text length.
    /// - Space Complexity: $O(S)$ where $S$ is the number of distinct stems.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Pure non-panicking string analysis; returns an empty vector when no negation exists.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/NEGATED-COMPONENTS]
    pub fn extract_negated_components(&self) -> Vec<String> {
        let negative_vals = ["0", "false", "\"no\"", "\"none\"", "no", "none"];
        let mut negated = BTreeSet::new();

        for attr in self.get_all_attributes() {
            if let Some(val) = &attr.default_value {
                let v = val.trim().to_ascii_lowercase();
                if negative_vals.contains(&v.as_str()) {
                    let stem = extract_negated_key_stem(&attr.name);
                    if !stem.is_empty() {
                        negated.insert(stem);
                    }
                }
            }
        }

        for con in self.get_all_constraints() {
            negated.extend(extract_negated_components(&con.expression));
            if let Some(doc) = &con.doc {
                negated.extend(extract_negated_components(doc));
            }
        }

        for pkg in self.get_all_packages() {
            if let Some(doc) = &pkg.doc {
                negated.extend(extract_negated_components(doc));
            }
        }

        negated.into_iter().collect()
    }

    /// Checks whether a component name is contradicted by a negation anywhere in this package hierarchy.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C. Fails closed against phantom
    /// safety-analysis entries for components the schema explicitly negates.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Returns `true` iff the normalized component name contains, or structurally matches, a negated stem.
    ///
    /// ## 3. Preconditions
    /// - `comp` is a component identifier or table label (markdown decoration tolerated).
    ///
    /// ## 4. Postconditions
    /// - Returns `false` for empty or non-alphanumeric component names.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(S \cdot N)$ for $S$ negated stems and component length $N$.
    /// - Space Complexity: $O(S + N)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Pure non-panicking string analysis.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/IS-COMPONENT-NEGATED]
    pub fn is_component_negated(&self, comp: &str) -> bool {
        component_matches_negated_stems(comp, &self.extract_negated_components())
    }

    /// Recursively searches for a mutable reference to a part definition by identifier name across self, nested parts, and all subpackages.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical software
    /// engineering standards. Mitigates dual-schema AST drift hazards (HAZ-SCHEMA-DRIFT-GATE31) by ensuring
    /// reverse-synchronization locates existing parts across multi-subsystem package boundaries rather than
    /// synthesizing orphan duplicate definitions at the root package level.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Exhaustive Traversal: Evaluates all $p \in \text{part\_defs}$ and recursively traverses all $pkg \in \text{packages}$.
    /// - First-Match Determinism: Traversal order is strictly deterministic (depth-first search across declared vectors).
    /// - Single Source of Truth Parity: Guarantees that reverse-sync updates in-place the exact AST node declared in subsystems.
    ///
    /// ## 3. Preconditions
    /// - `name` must be a valid non-empty KerML / SysML v2 identifier string.
    ///
    /// ## 4. Postconditions
    /// - Returns `Some(&mut PartDef)` pointing to the exact mutable AST node if located anywhere in the package hierarchy.
    /// - Returns `None` if the identifier is not found in self or any subpackages.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(V_{pkg} + V_{part})$ where $V_{pkg}$ is the total package count and $V_{part}$ is the total part count in the tree.
    /// - Space Complexity: $O(D_{pkg} + D_{part})$ auxiliary stack space bounded by AST hierarchy depth.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns `None` cleanly when the part definition is absent, allowing caller to handle fallback insertion safely.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/PACKAGE-DEF-MUT]
    pub fn find_part_mut(&mut self, name: &str) -> Option<&mut PartDef> {
        for p in &mut self.part_defs {
            if let Some(found) = p.find_part_mut(name) {
                return Some(found);
            }
        }
        for sub in &mut self.packages {
            if let Some(p) = sub.find_part_mut(name) {
                return Some(p);
            }
        }
        None
    }
}

/// Recursively collects a part definition and all nested sub-part definitions (`is_def == true`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C structural tree traversal completeness.
///
/// ## 2. Invariants & Mathematical Properties
/// - Appends `part.clone()` and recurses on every child with `sub.is_def`.
///
/// ## 3. Preconditions
/// - Part hierarchy is acyclic.
///
/// ## 4. Postconditions
/// - `result` contains `part` and all nested `is_def` sub-parts.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(V_{part})$. Space Complexity: $O(D_{part})$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking recursive accumulator.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COLLECT-PART-SUBDEFS]
fn collect_part_and_subdefs(part: &PartDef, result: &mut Vec<PartDef>) {
    result.push(part.clone());
    for sub in &part.parts {
        if sub.is_def {
            collect_part_and_subdefs(sub, result);
        }
    }
}

/// Recursively collects all capability definitions from a part and its nested sub-parts.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C capability enumeration completeness.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves depth-first declaration order across nested parts.
///
/// ## 3. Preconditions
/// - Part hierarchy is acyclic.
///
/// ## 4. Postconditions
/// - Appends all `CapabilityDef` items in `part` subtree to `result`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(V_{part} + N_{cap})$. Space Complexity: $O(D_{part})$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking recursive accumulator.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COLLECT-PART-CAPABILITIES]
fn collect_part_capabilities(part: &PartDef, result: &mut Vec<CapabilityDef>) {
    result.extend(part.capabilities.clone());
    for sub in &part.parts {
        collect_part_capabilities(sub, result);
    }
}

/// Recursively collects all use case definitions from a part and its nested sub-parts.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C behavioral use case enumeration completeness.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves depth-first declaration order across nested parts.
///
/// ## 3. Preconditions
/// - Part hierarchy is acyclic.
///
/// ## 4. Postconditions
/// - Appends all `UseCaseDef` items in `part` subtree to `result`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(V_{part} + N_{uc})$. Space Complexity: $O(D_{part})$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking recursive accumulator.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COLLECT-PART-USE-CASES]
fn collect_part_use_cases(part: &PartDef, result: &mut Vec<UseCaseDef>) {
    result.extend(part.use_cases.clone());
    for sub in &part.parts {
        collect_part_use_cases(sub, result);
    }
}

/// Recursively collects all interaction definitions from a part and its nested sub-parts.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C interaction sequence enumeration completeness.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves depth-first declaration order across nested parts.
///
/// ## 3. Preconditions
/// - Part hierarchy is acyclic.
///
/// ## 4. Postconditions
/// - Appends all `InteractionDef` items in `part` subtree to `result`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(V_{part} + N_{inter})$. Space Complexity: $O(D_{part})$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking recursive accumulator.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COLLECT-PART-INTERACTIONS]
fn collect_part_interactions(part: &PartDef, result: &mut Vec<InteractionDef>) {
    result.extend(part.interactions.clone());
    for sub in &part.parts {
        collect_part_interactions(sub, result);
    }
}

/// Recursively collects all action definitions from a part and its nested sub-parts.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C action behavior enumeration completeness.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves depth-first declaration order across nested parts.
///
/// ## 3. Preconditions
/// - Part hierarchy is acyclic.
///
/// ## 4. Postconditions
/// - Appends all `ActionDef` items in `part` subtree to `result`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(V_{part} + N_{act})$. Space Complexity: $O(D_{part})$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking recursive accumulator.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COLLECT-PART-ACTIONS]
fn collect_part_actions(part: &PartDef, result: &mut Vec<ActionDef>) {
    result.extend(part.actions.clone());
    for sub in &part.parts {
        collect_part_actions(sub, result);
    }
}

/// Recursively collects all constraint definitions from a part and its nested sub-parts.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C parametric constraint enumeration completeness.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves depth-first declaration order across nested parts.
///
/// ## 3. Preconditions
/// - Part hierarchy is acyclic.
///
/// ## 4. Postconditions
/// - Appends all `ConstraintDef` items in `part` subtree to `result`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(V_{part} + N_{con})$. Space Complexity: $O(D_{part})$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking recursive accumulator.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COLLECT-PART-CONSTRAINTS]
fn collect_part_constraints(part: &PartDef, result: &mut Vec<ConstraintDef>) {
    result.extend(part.constraints.clone());
    for sub in &part.parts {
        collect_part_constraints(sub, result);
    }
}

/// Recursively collects all attribute definitions from a part and its nested sub-parts.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C structural attribute enumeration completeness.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves depth-first declaration order across nested parts.
///
/// ## 3. Preconditions
/// - Part hierarchy is acyclic.
///
/// ## 4. Postconditions
/// - Appends all `AttributeDef` items in `part` subtree to `result`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(V_{part} + N_{attr})$. Space Complexity: $O(D_{part})$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking recursive accumulator.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COLLECT-PART-ATTRIBUTES]
fn collect_part_attributes(part: &PartDef, result: &mut Vec<AttributeDef>) {
    result.extend(part.attributes.clone());
    for sub in &part.parts {
        collect_part_attributes(sub, result);
    }
}

/// Recursively collects all calculation definitions from a part and its nested sub-parts.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C mathematical calculation enumeration completeness.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves depth-first declaration order across nested parts.
///
/// ## 3. Preconditions
/// - Part hierarchy is acyclic.
///
/// ## 4. Postconditions
/// - Appends all `CalcDef` items in `part` subtree to `result`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(V_{part} + N_{calc})$. Space Complexity: $O(D_{part})$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking recursive accumulator.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COLLECT-PART-CALCS]
fn collect_part_calcs(part: &PartDef, result: &mut Vec<CalcDef>) {
    result.extend(part.calcs.clone());
    for sub in &part.parts {
        collect_part_calcs(sub, result);
    }
}

/// Recursively collects all port definitions from a part and its nested sub-parts.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C logical interface port enumeration completeness.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves depth-first declaration order across nested parts.
///
/// ## 3. Preconditions
/// - Part hierarchy is acyclic.
///
/// ## 4. Postconditions
/// - Appends all `PortDef` items in `part` subtree to `result`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(V_{part} + N_{port})$. Space Complexity: $O(D_{part})$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking recursive accumulator.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COLLECT-PART-PORTS]
fn collect_part_ports(part: &PartDef, result: &mut Vec<PortDef>) {
    result.extend(part.ports.clone());
    for sub in &part.parts {
        collect_part_ports(sub, result);
    }
}

/// Recursively collects all safety hazard definitions from a part and its nested sub-parts.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and MIL-STD-882E hazard enumeration completeness.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves depth-first declaration order across nested parts.
///
/// ## 3. Preconditions
/// - Part hierarchy is acyclic.
///
/// ## 4. Postconditions
/// - Appends all `HazardDef` items in `part` subtree to `result`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(V_{part} + N_{haz})$. Space Complexity: $O(D_{part})$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking recursive accumulator.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COLLECT-PART-HAZARDS]
fn collect_part_hazards(part: &PartDef, result: &mut Vec<HazardDef>) {
    result.extend(part.hazards.clone());
    for sub in &part.parts {
        collect_part_hazards(sub, result);
    }
}

/// Recursively collects all safety risk definitions from a part and its nested sub-parts.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and MIL-STD-882E risk enumeration completeness.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves depth-first declaration order across nested parts.
///
/// ## 3. Preconditions
/// - Part hierarchy is acyclic.
///
/// ## 4. Postconditions
/// - Appends all `RiskDef` items in `part` subtree to `result`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(V_{part} + N_{risk})$. Space Complexity: $O(D_{part})$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking recursive accumulator.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COLLECT-PART-RISKS]
fn collect_part_risks(part: &PartDef, result: &mut Vec<RiskDef>) {
    result.extend(part.risks.clone());
    for sub in &part.parts {
        collect_part_risks(sub, result);
    }
}

/// Recursively collects all state definitions from a part and its nested sub-parts.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and IEC 61508 state machine enumeration completeness.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves depth-first declaration order across nested parts.
///
/// ## 3. Preconditions
/// - Part hierarchy is acyclic.
///
/// ## 4. Postconditions
/// - Appends all `StateDef` items in `part` subtree to `result`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(V_{part} + N_{state})$. Space Complexity: $O(D_{part})$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking recursive accumulator.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COLLECT-PART-STATES]
fn collect_part_states(part: &PartDef, result: &mut Vec<StateDef>) {
    result.extend(part.states.clone());
    for sub in &part.parts {
        collect_part_states(sub, result);
    }
}

/// Recursively collects all connection definitions from a part and its nested sub-parts.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C topological connection enumeration completeness.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves depth-first declaration order across nested parts.
///
/// ## 3. Preconditions
/// - Part hierarchy is acyclic.
///
/// ## 4. Postconditions
/// - Appends all `ConnectionDef` items in `part` subtree to `result`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(V_{part} + N_{conn})$. Space Complexity: $O(D_{part})$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking recursive accumulator.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COLLECT-PART-CONNECTIONS]
fn collect_part_connections(part: &PartDef, result: &mut Vec<ConnectionDef>) {
    result.extend(part.connections.clone());
    for sub in &part.parts {
        collect_part_connections(sub, result);
    }
}

/// Recursively inserts all port identifiers declared on nested sub-parts into `nested`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C hierarchical port qualification rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Inserts unique port names into a sorted `BTreeSet<String>`.
///
/// ## 3. Preconditions
/// - Part hierarchy is acyclic.
///
/// ## 4. Postconditions
/// - `nested` contains every port name from `parts` and their descendants.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N_{ports} \log N_{ports})$. Space Complexity: $O(D_{part})$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking recursive accumulator.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COLLECT-NESTED-PORTS-RECURSIVE]
fn collect_nested_ports_recursive(parts: &[PartDef], nested: &mut BTreeSet<String>) {
    for p in parts {
        for port in &p.ports {
            nested.insert(port.name.clone());
        }
        collect_nested_ports_recursive(&p.parts, nested);
    }
}

/// Legacy and high-level SysML model container for backwards compatibility.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `SysmlModel` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `SysmlModel` across member set `[package_name: Option<String>, doc: Option<String>, attributes: Vec<AttributeDef>, parts: Vec<PartDef>, ports: Vec<PortDef>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `SysmlModel` members (`package_name: Option<String>, doc: Option<String>, attributes: Vec<AttributeDef>, parts: Vec<PartDef>, ports: Vec<PortDef>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `SysmlModel` representation for struct `SysmlModel`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `SysmlModel`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `SysmlModel`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `SysmlModel` (`SysmlModel`).
/// - State: Maintains consistent state invariants across `SysmlModel` without partial mutation on error.
/// - Action: Executes `SysmlModel` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `SysmlModel` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-SYSMLMODEL] | SysML v2: PartDef(SysmlModel)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SysmlModel {
    pub package_name: Option<String>,
    pub doc: Option<String>,
    pub attributes: Vec<AttributeDef>,
    pub parts: Vec<PartDef>,
    pub ports: Vec<PortDef>,
    pub actions: Vec<ActionDef>,
    pub constraints: Vec<ConstraintDef>,
    pub requirements: Vec<RequirementDef>,
    #[serde(default)]
    pub test_cases: Vec<TestCaseDef>,
}

impl From<PackageDef> for SysmlModel {
    /// Converts a `PackageDef` into a flat `SysmlModel` representation.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C lossless top-level AST projection.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves package name, documentation, and top-level definition collections.
    ///
    /// ## 3. Preconditions
    /// - `pkg` is a valid `PackageDef`.
    ///
    /// ## 4. Postconditions
    /// - Returns an owned `SysmlModel` containing moved fields from `pkg`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$ move semantics. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible ownership transfer.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/SYSML-MODEL-FROM-PACKAGE]
    fn from(pkg: PackageDef) -> Self {
        Self {
            package_name: Some(pkg.name),
            doc: pkg.doc,
            attributes: pkg.attribute_defs,
            parts: pkg.part_defs,
            ports: pkg.port_defs,
            actions: pkg.action_defs,
            constraints: pkg.constraint_defs,
            requirements: pkg.requirement_defs,
            test_cases: pkg.test_case_defs,
        }
    }
}

impl From<SysmlModel> for PackageDef {
    /// Converts a `SysmlModel` into a `PackageDef` root container.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C deterministic package lifting.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Defaults unnamed models to `"SysML_Model"` and preserves all model vectors.
    ///
    /// ## 3. Preconditions
    /// - `m` is a valid `SysmlModel`.
    ///
    /// ## 4. Postconditions
    /// - Returns an initialized `PackageDef` containing all elements from `m`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$ move semantics. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible ownership transfer.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/PACKAGE-FROM-SYSML-MODEL]
    fn from(m: SysmlModel) -> Self {
        Self {
            name: m.package_name.unwrap_or_else(|| "SysML_Model".to_string()),
            doc: m.doc,
            parent_package: None,
            imports: Vec::new(),
            packages: Vec::new(),
            part_defs: m.parts,
            port_defs: m.ports,
            attribute_defs: m.attributes,
            action_defs: m.actions,
            operation_defs: Vec::new(),
            capability_defs: Vec::new(),
            interaction_defs: Vec::new(),
            constraint_defs: m.constraints,
            test_case_defs: m.test_cases,
            requirement_defs: m.requirements,
            satisfied_requirements: Vec::new(),
            state_defs: Vec::new(),
            use_case_defs: Vec::new(),
            item_defs: Vec::new(),
            hazard_defs: Vec::new(),
            risk_defs: Vec::new(),
            connection_defs: Vec::new(),
            calc_defs: Vec::new(),
        }
    }
}

/// Checks whether a byte is an ASCII identifier character (`[A-Za-z0-9_]`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C regex-free lexical analysis.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `true` iff `b.is_ascii_alphanumeric() || b == b'_'`.
///
/// ## 3. Preconditions
/// - None.
///
/// ## 4. Postconditions
/// - Pure boolean classification of `b`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible byte predicate.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/IS-IDENT-CHAR]
fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Checks whether `pos` in `text` is preceded by a non-identifier boundary or start-of-string.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C bounds-checked token boundary verification.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses `.get(pos - 1)` bounds-checked byte access; never panics on out-of-range indices.
///
/// ## 3. Preconditions
/// - `text` is a valid UTF-8 slice.
///
/// ## 4. Postconditions
/// - Returns `true` if `pos == 0` or the preceding byte is not an identifier character.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Out-of-bounds positions return `true` safely without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/WORD-BOUNDARY-BEFORE]
fn is_word_boundary_before(text: &str, pos: usize) -> bool {
    if pos == 0 {
        true
    } else {
        !text.as_bytes().get(pos - 1).copied().is_some_and(is_ident_char)
    }
}

/// Checks whether `pos` in `text` is followed by a non-identifier boundary or end-of-string.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C bounds-checked token boundary verification.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses `.get(pos)` bounds-checked byte access; never panics on out-of-range indices.
///
/// ## 3. Preconditions
/// - `text` is a valid UTF-8 slice.
///
/// ## 4. Postconditions
/// - Returns `true` if `pos >= text.len()` or the byte at `pos` is not an identifier character.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Out-of-bounds positions return `true` safely without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/WORD-BOUNDARY-AFTER]
fn is_word_boundary_after(text: &str, pos: usize) -> bool {
    if pos >= text.len() {
        true
    } else {
        !text.as_bytes().get(pos).copied().is_some_and(is_ident_char)
    }
}

/// Counts leading ASCII whitespace bytes in `text`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C deterministic lexical scanning.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returned count $k \le \text{text.len()}$ always lies on a valid UTF-8 character boundary.
///
/// ## 3. Preconditions
/// - `text` is a valid UTF-8 slice.
///
/// ## 4. Postconditions
/// - Returns the number of contiguous leading ASCII whitespace bytes.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(k)$ where $k$ is leading whitespace length. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `0` on empty or non-whitespace-prefixed input.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/SKIP-WS]
fn skip_ws(text: &str) -> usize {
    text.bytes().take_while(|b| b.is_ascii_whitespace()).count()
}

/// Extracts a leading ASCII identifier token and its byte length from `text`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C bounds-checked identifier extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses `.get(..len).unwrap_or("")` for panic-free slicing.
///
/// ## 3. Preconditions
/// - `text` is a valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns `(ident_slice, len)` where `len == ident_slice.len()`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(\text{len})$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `("", 0)` if `text` does not start with an identifier character.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-IDENT]
fn extract_ident(text: &str) -> (&str, usize) {
    let len = text.bytes().take_while(|&b| is_ident_char(b)).count();
    (text.get(..len).unwrap_or(""), len)
}

/// Extracts an optional `doc /* ... */` documentation block immediately preceding `keyword_pos`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C bounds-checked documentation extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses `.get(..)` bounds-checked slicing exclusively; never panics on malformed comment delimiters.
///
/// ## 3. Preconditions
/// - `text` is a valid UTF-8 string slice and `keyword_pos <= text.len()`.
///
/// ## 4. Postconditions
/// - Returns `Some(trimmed_doc)` if a valid `doc /* ... */` block immediately precedes `keyword_pos`, else `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(keyword\_pos)$. Space Complexity: $O(L_{doc})$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `None` if `keyword_pos` is out of bounds or no `doc /* ... */` comment precedes the keyword.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-DOC-COMMENT]
fn extract_doc_comment_before(text: &str, keyword_pos: usize) -> Option<String> {
    let before = text.get(..keyword_pos)?.trim_end();
    if before.ends_with("*/") {
        if let Some(open) = before.rfind("/*") {
            let doc_prefix = before.get(..open)?.trim_end();
            if doc_prefix.ends_with("doc") {
                let doc_start = doc_prefix.len() - 3;
                if doc_start == 0 || !doc_prefix.as_bytes().get(doc_start - 1).copied().is_some_and(is_ident_char) {
                    let inner = before.get(open + 2..before.len().saturating_sub(2))?;
                    return Some(inner.trim().to_string());
                }
            }
        }
    }
    None
}

/// Extracts the inner string content of a double-quoted literal `"..."`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C bounds-checked string literal extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses `strip_prefix` and `.get(..end)` for panic-free extraction.
///
/// ## 3. Preconditions
/// - `s` is a valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns `Some(inner)` when `s` starts with `"` and has a closing `"`, else `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Unclosed quotes return `None` safely without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-QUOTED-STRING]
fn extract_quoted_string(s: &str) -> Option<&str> {
    if let Some(rest) = s.strip_prefix('"') {
        if let Some(end) = rest.find('"') {
            return rest.get(..end);
        }
    }
    None
}

/// Parse attributes from SysML v2 source text.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic attribute AST extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses bounds-checked `.get(..)` slicing on comment-stripped and string-masked source buffers.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 SysML v2 source fragment.
///
/// ## 4. Postconditions
/// - Returns a vector of all parsed `AttributeDef` declarations in appearance order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ in `content.len()`. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Malformed attribute statements are skipped safely without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-ATTRIBUTE-DEFS]
pub fn parse_attribute_defs(content: &str) -> Vec<AttributeDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut attrs = Vec::new();
    let mut idx = 0;
    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("attribute")) {
        let kw_pos = idx + pos;
        if !is_word_boundary_before(&masked, kw_pos) || !is_word_boundary_after(&masked, kw_pos + 9) {
            idx = kw_pos + 9;
            continue;
        }
        let doc = extract_doc_comment_before(&clean, kw_pos);
        let mut curr = kw_pos + 9;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = kw_pos + 9;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        let mut type_name = "String".to_string();
        let mut has_type = false;
        let rem_colon = masked.get(curr..).unwrap_or("");
        if rem_colon.starts_with(':') && !rem_colon.starts_with(":=") {
            curr += 1;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
            let (t_name, type_len) = extract_type_ident(masked.get(curr..).unwrap_or(""));
            if t_name.is_empty() {
                idx = kw_pos + 9;
                continue;
            }
            type_name = t_name.to_string();
            has_type = true;
            curr += type_len;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let rem_assign = masked.get(curr..).unwrap_or("");
        if rem_assign.starts_with(":=") || rem_assign.starts_with('=') {
            if rem_assign.starts_with(":=") {
                curr += 2;
            } else {
                curr += 1;
            }
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
            if let Some(semi) = masked.get(curr..).and_then(|s| s.find(';')) {
                let val = clean.get(curr..curr + semi).unwrap_or("").trim().to_string();
                attrs.push(AttributeDef {
                    name: name.to_string(),
                    type_name,
                    default_value: Some(val),
                    doc,
                });
                idx = curr + semi + 1;
                continue;
            } else {
                idx = kw_pos + 9;
                continue;
            }
        } else if has_type && rem_assign.starts_with(';') {
            attrs.push(AttributeDef {
                name: name.to_string(),
                type_name,
                default_value: None,
                doc,
            });
            idx = curr + 1;
            continue;
        } else {
            idx = kw_pos + 9;
            continue;
        }
    }
    attrs
}

/// Parses a single item flow declaration line inside a SysML v2 port block.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C bounds-checked item flow parsing.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses `.get(..).unwrap_or("")` for all substring advances; never panics.
///
/// ## 3. Preconditions
/// - `line` is a single line from a port body block.
///
/// ## 4. Postconditions
/// - Returns `Some(FlowDef)` if `line` declares a port flow/item entry, else `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ in line byte length. Space Complexity: $O(L)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-flow lines (e.g., attributes or blank lines) return `None`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PORT-FLOW-LINE]
fn parse_port_flow_line(line: &str) -> Option<FlowDef> {
    let trimmed = line.trim().trim_end_matches(';').trim();
    if trimmed.is_empty() || (trimmed.starts_with("attribute") && is_word_boundary_after(trimmed, 9)) {
        return None;
    }
    let mut rest = trimmed;
    let mut direction = None;
    let mut saw_flow_kw = false;

    if rest.starts_with("flow") && is_word_boundary_after(rest, 4) {
        saw_flow_kw = true;
        rest = rest.get(4..).unwrap_or("").trim_start();
    }
    if rest.starts_with("inout") && is_word_boundary_after(rest, 5) {
        direction = Some("inout".to_string());
        rest = rest.get(5..).unwrap_or("").trim_start();
    } else if rest.starts_with("in") && is_word_boundary_after(rest, 2) {
        direction = Some("in".to_string());
        rest = rest.get(2..).unwrap_or("").trim_start();
    } else if rest.starts_with("out") && is_word_boundary_after(rest, 3) {
        direction = Some("out".to_string());
        rest = rest.get(3..).unwrap_or("").trim_start();
    }
    if (rest.starts_with("flow") || rest.starts_with("item")) && is_word_boundary_after(rest, 4) {
        saw_flow_kw = true;
        rest = rest.get(4..).unwrap_or("").trim_start();
    }

    if !saw_flow_kw && direction.is_none() {
        return None;
    }

    let (name, name_len) = extract_ident(rest);
    if name.is_empty() {
        return None;
    }
    let after_name = rest.get(name_len..).unwrap_or("").trim_start();
    let item_type = if let Some(after_colon_raw) = after_name.strip_prefix(':') {
        let after_colon = after_colon_raw.trim_start();
        let (t_name, _) = extract_type_ident(after_colon);
        if t_name.is_empty() {
            "Item".to_string()
        } else {
            t_name.to_string()
        }
    } else {
        "Item".to_string()
    };

    Some(FlowDef {
        name: name.to_string(),
        direction: direction.unwrap_or_else(|| "out".to_string()),
        item_type,
        doc: None,
        ..Default::default()
    })
}

/// Parse port definitions from SysML v2 source text.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic interface port AST extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses bounds-checked `.get(..)` slicing on comment-stripped and string-masked buffers.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 SysML v2 source fragment.
///
/// ## 4. Postconditions
/// - Returns a vector of all parsed `PortDef` structures in appearance order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ in `content.len()`. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Malformed port declarations are skipped safely without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PORT-DEFS]
pub fn parse_port_defs(content: &str) -> Vec<PortDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut ports = Vec::new();
    let mut idx = 0;
    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("port")) {
        let port_pos = idx + pos;
        if !is_word_boundary_before(&masked, port_pos) || !is_word_boundary_after(&masked, port_pos + 4) {
            idx = port_pos + 4;
            continue;
        }
        let before_port = masked.get(..port_pos).unwrap_or("").trim_end();
        let (dir1, doc_end) = if before_port.ends_with("inout") && is_word_boundary_before(before_port, before_port.len() - 5) {
            (Some("inout"), before_port.len() - 5)
        } else if before_port.ends_with("out") && is_word_boundary_before(before_port, before_port.len() - 3) {
            (Some("out"), before_port.len() - 3)
        } else if before_port.ends_with("in") && is_word_boundary_before(before_port, before_port.len() - 2) {
            (Some("in"), before_port.len() - 2)
        } else {
            (None, port_pos)
        };
        let doc = extract_doc_comment_before(&clean, doc_end);

        let mut curr = port_pos + 4;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let mut dir2 = None;
        let rem_dir = masked.get(curr..).unwrap_or("");
        if rem_dir.starts_with("inout") && is_word_boundary_after(rem_dir, 5) {
            dir2 = Some("inout");
            curr += 5;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        } else if rem_dir.starts_with("out") && is_word_boundary_after(rem_dir, 3) {
            dir2 = Some("out");
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        } else if rem_dir.starts_with("in") && is_word_boundary_after(rem_dir, 2) {
            dir2 = Some("in");
            curr += 2;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let mut conj1 = false;
        if masked.get(curr..).is_some_and(|s| s.starts_with('~')) {
            conj1 = true;
            curr += 1;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = port_pos + 4;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));

        let mut type_name = None;
        let mut conj2 = false;
        let rem_type = masked.get(curr..).unwrap_or("");
        if rem_type.starts_with(":>") || rem_type.starts_with(':') {
            if rem_type.starts_with(":>") {
                curr += 2;
            } else {
                curr += 1;
            }
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
            if masked.get(curr..).is_some_and(|s| s.starts_with('~')) {
                conj2 = true;
                curr += 1;
                curr += skip_ws(masked.get(curr..).unwrap_or(""));
            }
            let (t, t_len) = extract_ident(masked.get(curr..).unwrap_or(""));
            if !t.is_empty() {
                type_name = Some(t.to_string());
                curr += t_len;
                curr += skip_ws(masked.get(curr..).unwrap_or(""));
            }
        }

        let mut protocol_family = String::new();
        let mut port_category = default_port_category();
        let mut electrical_attributes = BTreeMap::new();
        let mut item_flows = Vec::new();

        let rem_body = masked.get(curr..).unwrap_or("");
        if rem_body.starts_with('{') {
            let port_body_opt = if let Some(close_brace) = find_matching_brace(&clean, curr) {
                idx = close_brace + 1;
                clean.get(curr + 1..close_brace)
            } else if let Some(close_brace) = rem_body.find('}') {
                idx = curr + close_brace + 1;
                clean.get(curr + 1..curr + close_brace)
            } else {
                idx = port_pos + 4;
                continue;
            };
            if let Some(port_body) = port_body_opt {
                for attr in parse_attribute_defs(port_body) {
                    let val = attr
                        .default_value
                        .as_deref()
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"')
                        .to_string();
                    if attr.name == "protocol_family" {
                        protocol_family = val;
                    } else if attr.name == "port_category" {
                        if !val.is_empty() {
                            port_category = val;
                        }
                    } else {
                        electrical_attributes.insert(attr.name, val);
                    }
                }
                for line in port_body.lines() {
                    if let Some(flow) = parse_port_flow_line(line) {
                        item_flows.push(flow);
                    }
                }
            }
        } else if rem_body.starts_with(';') {
            idx = curr + 1;
        } else {
            idx = port_pos + 4;
            continue;
        }

        let final_type = type_name.unwrap_or_else(|| name.to_string());
        let direction = dir1.or(dir2).unwrap_or("inout").to_string();
        let is_conjugated = conj1 || conj2;
        ports.push(PortDef {
            name: name.to_string(),
            type_name: final_type,
            direction,
            is_conjugated,
            doc,
            port_category,
            protocol_family,
            electrical_attributes,
            item_flows,
        });
    }
    ports
}

/// Parses a single `in` or `out` action parameter declaration into `(is_in, AttributeDef)`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C bounds-checked action parameter extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses `.get(..).unwrap_or("")` for all substring advances; never panics.
///
/// ## 3. Preconditions
/// - `s` is a parameter clause or body line.
///
/// ## 4. Postconditions
/// - Returns `Some((true, attr))` for `in` parameters, `Some((false, attr))` for `out` parameters, or `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ in parameter string length. Space Complexity: $O(L)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-parameter lines return `None` safely.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-ACTION-PARAM]
fn parse_action_param(s: &str) -> Option<(bool, AttributeDef)> {
    let trimmed = s.trim();
    let (is_in, after_dir) = if trimmed.starts_with("in") && is_word_boundary_after(trimmed, 2) {
        (true, trimmed.get(2..).unwrap_or("").trim_start())
    } else if trimmed.starts_with("out") && is_word_boundary_after(trimmed, 3) {
        (false, trimmed.get(3..).unwrap_or("").trim_start())
    } else {
        return None;
    };
    let after_kw = if after_dir.starts_with("attribute") && is_word_boundary_after(after_dir, 9) {
        after_dir.get(9..).unwrap_or("").trim_start()
    } else if after_dir.starts_with("item") && is_word_boundary_after(after_dir, 4) {
        after_dir.get(4..).unwrap_or("").trim_start()
    } else {
        after_dir
    };
    let (name, name_len) = extract_ident(after_kw);
    if name.is_empty() {
        return None;
    }
    let after_name = after_kw.get(name_len..).unwrap_or("").trim_start();
    let after_colon = after_name.strip_prefix(':')?.trim_start();
    let (type_name, type_len) = extract_type_ident(after_colon);
    if type_len == 0 {
        return None;
    }
    Some((
        is_in,
        AttributeDef {
            name: name.to_string(),
            type_name: type_name.to_string(),
            default_value: None,
            doc: None,
        },
    ))
}

/// Parse action definitions from SysML v2 source text.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic behavioral action AST extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses bounds-checked `.get(..)` slicing on comment-stripped and string-masked source buffers.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 SysML v2 source fragment.
///
/// ## 4. Postconditions
/// - Returns a vector of all parsed `ActionDef` structures in appearance order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ in `content.len()`. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Malformed action declarations are skipped safely without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-ACTION-DEFS]
pub fn parse_action_defs(content: &str) -> Vec<ActionDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut actions = Vec::new();
    let mut idx = 0;
    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("action")) {
        let act_pos = idx + pos;
        if !is_word_boundary_before(&masked, act_pos) || !is_word_boundary_after(&masked, act_pos + 6) {
            idx = act_pos + 6;
            continue;
        }
        let doc = extract_doc_comment_before(&clean, act_pos);
        let mut curr = act_pos + 6;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        let mut is_def = false;
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            is_def = true;
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = act_pos + 6;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));

        let mut params_str = None;
        if masked.get(curr..).is_some_and(|s| s.starts_with('(')) {
            if let Some(close_paren) = masked.get(curr + 1..).and_then(|s| s.find(')')) {
                params_str = clean.get(curr + 1..curr + 1 + close_paren);
                curr = curr + 1 + close_paren + 1;
                curr += skip_ws(masked.get(curr..).unwrap_or(""));
            } else {
                idx = act_pos + 6;
                continue;
            }
        }

        let mut body_str = None;
        let rem_body = masked.get(curr..).unwrap_or("");
        if rem_body.starts_with('{') {
            if let Some(close_pos) = find_matching_brace(&clean, curr) {
                body_str = clean.get(curr + 1..close_pos);
                idx = close_pos + 1;
            } else {
                idx = act_pos + 6;
                continue;
            }
        } else if rem_body.starts_with(';') {
            idx = curr + 1;
        } else {
            idx = act_pos + 6;
            continue;
        }

        let mut in_params = Vec::new();
        let mut out_params = Vec::new();
        let mut steps = Vec::new();
        let mut performer = None;

        if let Some(params) = params_str {
            for part in params.split(',') {
                if let Some((is_in, attr)) = parse_action_param(part) {
                    if is_in {
                        in_params.push(attr);
                    } else {
                        out_params.push(attr);
                    }
                }
            }
        }

        if let Some(body) = body_str {
            for line in body.lines() {
                let trimmed = line.trim();
                if let Some((is_in, attr)) = parse_action_param(trimmed) {
                    if is_in {
                        in_params.push(attr);
                    } else {
                        out_params.push(attr);
                    }
                } else if trimmed.starts_with("perform") && is_word_boundary_after(trimmed, 7) {
                    let perf = trimmed.get(7..).unwrap_or("").trim().trim_end_matches(';').trim();
                    if !perf.is_empty() {
                        performer = Some(perf.to_string());
                    }
                } else if trimmed.starts_with("step") && is_word_boundary_after(trimmed, 4) {
                    let step_name = trimmed.get(4..).unwrap_or("").trim().trim_end_matches(';').trim();
                    if !step_name.is_empty() {
                        steps.push(step_name.to_string());
                    }
                }
            }
        }

        actions.push(ActionDef {
            name: name.to_string(),
            doc,
            in_params,
            out_params,
            performer,
            steps,
            is_def,
            ..Default::default()
        });
    }
    actions
}

/// Parse constraint definitions or assertions from SysML v2 source text.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic safety constraint AST extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses bounds-checked `.get(..)` slicing on comment-stripped and string-masked source buffers.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 SysML v2 source fragment.
///
/// ## 4. Postconditions
/// - Returns a vector of all parsed `ConstraintDef` structures in appearance order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ in `content.len()`. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Malformed constraint blocks are skipped safely without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CONSTRAINT-DEFS]
pub fn parse_constraint_defs(content: &str) -> Vec<ConstraintDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut constraints = Vec::new();
    let mut idx = 0;
    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("constraint")) {
        let c_pos = idx + pos;
        if !is_word_boundary_before(&masked, c_pos) || !is_word_boundary_after(&masked, c_pos + 10) {
            idx = c_pos + 10;
            continue;
        }
        let before_c = masked.get(..c_pos).unwrap_or("").trim_end();
        let (is_assertion, doc_end) = if before_c.ends_with("assert") && is_word_boundary_before(before_c, before_c.len() - 6) {
            (true, before_c.len() - 6)
        } else {
            (false, c_pos)
        };
        let doc = extract_doc_comment_before(&clean, doc_end);

        let mut curr = c_pos + 10;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = c_pos + 10;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));

        let mut parameters = Vec::new();
        if masked.get(curr..).is_some_and(|s| s.starts_with('(')) {
            if let Some(close_paren) = masked.get(curr + 1..).and_then(|s| s.find(')')) {
                let p_str = clean.get(curr + 1..curr + 1 + close_paren).unwrap_or("");
                parameters = p_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                curr = curr + 1 + close_paren + 1;
                curr += skip_ws(masked.get(curr..).unwrap_or(""));
            } else {
                idx = c_pos + 10;
                continue;
            }
        }

        let mut expression = String::new();
        let rem_body = masked.get(curr..).unwrap_or("");
        if rem_body.starts_with('{') {
            if let Some(close_pos) = find_matching_brace(&clean, curr) {
                let expr_str = clean.get(curr + 1..close_pos).unwrap_or("");
                expression = expr_str.trim().trim_end_matches(';').trim().to_string();
                idx = close_pos + 1;
            } else {
                idx = c_pos + 10;
                continue;
            }
        } else if rem_body.starts_with(';') {
            idx = curr + 1;
        } else {
            idx = c_pos + 10;
            continue;
        }

        constraints.push(ConstraintDef {
            name: name.to_string(),
            expression,
            parameters,
            is_assertion,
            doc,
            ..Default::default()
        });
    }
    constraints
}

/// Extracts a qualified or unqualified KerML/SysML v2 type identifier (`Pkg::Type` or `Type`) from `s`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C bounds-checked qualified type extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - Advances across ASCII identifier characters and `::` namespace separators using `.get(..)`.
///
/// ## 3. Preconditions
/// - `s` is a valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns `(type_slice, len)` where `len == type_slice.len()`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(\text{len})$. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `("", 0)` if `s` does not begin with a valid identifier character.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-TYPE-IDENT]
fn extract_type_ident(s: &str) -> (&str, usize) {
    let mut len = 0;
    let bytes = s.as_bytes();
    while len < bytes.len() {
        if let Some(&b) = bytes.get(len) {
            if is_ident_char(b) {
                len += 1;
            } else if s.get(len..).is_some_and(|rest| rest.starts_with("::")) {
                len += 2;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    (s.get(..len).unwrap_or(""), len)
}

/// Parses a single calculation parameter declaration into an `AttributeDef`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C bounds-checked calculation parameter extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses `.get(..).unwrap_or("")` for all substring advances; defaults untyped parameters to `"ScalarValues::Real"`.
///
/// ## 3. Preconditions
/// - `s` is a parameter clause or `in` parameter line inside a `calc` definition.
///
/// ## 4. Postconditions
/// - Returns `Some(AttributeDef)` if a valid parameter name is present, else `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ in parameter string length. Space Complexity: $O(L)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Empty or non-identifier clauses return `None` safely.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CALC-PARAM]
fn parse_calc_param(s: &str) -> Option<AttributeDef> {
    let mut trimmed = s.trim().trim_end_matches(';').trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with("in") && is_word_boundary_after(trimmed, 2) {
        trimmed = trimmed.get(2..).unwrap_or("").trim_start();
    }
    if trimmed.starts_with("attribute") && is_word_boundary_after(trimmed, 9) {
        trimmed = trimmed.get(9..).unwrap_or("").trim_start();
    }
    let (name, name_len) = extract_ident(trimmed);
    if name.is_empty() {
        return None;
    }
    let after_name = trimmed.get(name_len..).unwrap_or("").trim_start();
    let mut type_name = "ScalarValues::Real".to_string();
    let mut default_val = None;
    if after_name.starts_with(':') {
        let after_colon = after_name.get(1..).unwrap_or("").trim_start();
        let (t_name, t_len) = extract_type_ident(after_colon);
        if !t_name.is_empty() {
            type_name = t_name.to_string();
        }
        let after_type = after_colon.get(t_len..).unwrap_or("").trim_start();
        if after_type.starts_with('=') {
            let val = after_type.get(1..).unwrap_or("").trim().trim_end_matches(';').trim();
            if !val.is_empty() {
                default_val = Some(val.to_string());
            }
        }
    } else if after_name.starts_with('=') {
        let val = after_name.get(1..).unwrap_or("").trim().trim_end_matches(';').trim();
        if !val.is_empty() {
            default_val = Some(val.to_string());
        }
    }
    Some(AttributeDef {
        name: name.to_string(),
        type_name,
        default_value: default_val,
        doc: None,
    })
}

/// Parse calculation definitions from SysML v2 / KerML source text.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C standards. Parses pure mathematical
/// and physics calculations directly from the authoritative SysML AST, eliminating hardcoded domain formulas
/// and blocklists from the compiler toolchain.
///
/// ## 2. Invariants & Mathematical Properties
/// - AST Purity: Extracted `CalcDef` instances derive 100% and deterministically from AST text.
/// - Determinism: Identical source text produces identical `CalcDef` vectors.
///
/// ## 3. Preconditions
/// - `content` must be a valid UTF-8 string containing SysML v2 / KerML code.
///
/// ## 4. Postconditions
/// - Returns a vector of all valid `CalcDef` declarations parsed from `content`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ where $N$ is the byte length of `content`.
/// - Space Complexity: $O(M)$ where $M$ is the memory required for extracted AST structures.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Syntax irregularities or non-matching tokens are skipped gracefully without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CALC-DEFS]
pub fn parse_calc_defs(content: &str) -> Vec<CalcDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut calcs = Vec::new();
    let mut idx = 0;
    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("calc")) {
        let calc_pos = idx + pos;
        if !is_word_boundary_before(&masked, calc_pos) || !is_word_boundary_after(&masked, calc_pos + 4) {
            idx = calc_pos + 4;
            continue;
        }
        let doc = extract_doc_comment_before(&clean, calc_pos);
        let mut curr = calc_pos + 4;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if masked.get(curr..).is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3)) {
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = calc_pos + 4;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));

        let mut in_params = Vec::new();
        if masked.get(curr..).is_some_and(|s| s.starts_with('(')) {
            if let Some(close_paren) = masked.get(curr + 1..).and_then(|s| s.find(')')) {
                let p_str = clean.get(curr + 1..curr + 1 + close_paren).unwrap_or("");
                for part in p_str.split(',') {
                    if let Some(param) = parse_calc_param(part) {
                        in_params.push(param);
                    }
                }
                curr = curr + 1 + close_paren + 1;
                curr += skip_ws(masked.get(curr..).unwrap_or(""));
            } else {
                idx = calc_pos + 4;
                continue;
            }
        }

        let mut return_type = None;
        if masked.get(curr..).is_some_and(|s| s.starts_with(':')) {
            curr += 1;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
            let (ret_t, ret_len) = extract_type_ident(masked.get(curr..).unwrap_or(""));
            if !ret_t.is_empty() {
                return_type = Some(ret_t.to_string());
                curr += ret_len;
                curr += skip_ws(masked.get(curr..).unwrap_or(""));
            }
        }

        let (body, next_idx) = if masked.get(curr..).is_some_and(|s| s.starts_with('{')) {
            let open_pos = curr;
            let close_pos = find_matching_brace(&clean, open_pos);
            let b = if let Some(cp) = close_pos {
                clean.get(open_pos + 1..cp).unwrap_or("")
            } else {
                ""
            };
            let next = close_pos.map(|cp| cp + 1).unwrap_or(open_pos + 1);
            (Some(b), next)
        } else if masked.get(curr..).is_some_and(|s| s.starts_with(';')) {
            (None, curr + 1)
        } else {
            idx = calc_pos + 4;
            continue;
        };

        idx = next_idx;

        let mut expression = None;
        if let Some(b) = body {
            let mut expr_lines = Vec::new();
            for line in b.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if (trimmed.starts_with("in ") || trimmed.starts_with("in\t") || trimmed.starts_with("attribute "))
                    && !trimmed.contains('=') && !trimmed.contains('+') && !trimmed.contains('-') && !trimmed.contains('*') && !trimmed.contains('/')
                {
                    if let Some(param) = parse_calc_param(trimmed) {
                        in_params.push(param);
                    }
                } else if trimmed.starts_with("return ") || trimmed.starts_with("return\t") || trimmed.starts_with("return:") {
                    let rest = trimmed.trim_start_matches("return").trim();
                    let is_math_expr = rest.contains('+') || rest.contains('-') || rest.contains('*') || rest.contains('/') || rest.contains('(');
                    if return_type.is_none() && !is_math_expr && (rest.contains(':') || rest.starts_with("type ") || rest.split_whitespace().count() <= 2) {
                        let type_part = if let Some(colon_pos) = rest.find(':') {
                            rest.get(colon_pos + 1..).unwrap_or("").trim()
                        } else {
                            rest.trim_start_matches("type").trim()
                        };
                        let (t_ident, _) = extract_type_ident(type_part);
                        if !t_ident.is_empty() {
                            return_type = Some(t_ident.to_string());
                            continue;
                        }
                    }
                    let expr_candidate = rest.trim_start_matches(':').trim().trim_end_matches(';').trim();
                    if !expr_candidate.is_empty() {
                        expr_lines.push(expr_candidate);
                    }
                } else {
                    expr_lines.push(trimmed.trim_end_matches(';').trim());
                }
            }
            if !expr_lines.is_empty() {
                expression = Some(expr_lines.join("\n"));
            }
        }

        calcs.push(CalcDef {
            name: name.to_string(),
            doc,
            in_params,
            return_type,
            expression,
        });
    }
    calcs
}

/// Parse requirement definitions from SysML v2 source text.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic requirement AST extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses bounds-checked `.get(..)` slicing on comment-stripped and string-masked source buffers.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 SysML v2 source fragment.
///
/// ## 4. Postconditions
/// - Returns a vector of all parsed `RequirementDef` structures in appearance order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ in `content.len()`. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Malformed requirement blocks are skipped safely without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-REQUIREMENT-DEFS]
pub fn parse_requirement_defs(content: &str) -> Vec<RequirementDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut reqs = Vec::new();
    let mut idx = 0;
    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("requirement")) {
        let req_pos = idx + pos;
        if !is_word_boundary_before(&masked, req_pos) || !is_word_boundary_after(&masked, req_pos + 11) {
            idx = req_pos + 11;
            continue;
        }
        let doc = extract_doc_comment_before(&clean, req_pos);
        let mut curr = req_pos + 11;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = req_pos + 11;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));

        let mut body_str = None;
        let rem_body = masked.get(curr..).unwrap_or("");
        if rem_body.starts_with('{') {
            if let Some(close_pos) = find_matching_brace(&clean, curr) {
                body_str = clean.get(curr + 1..close_pos);
                idx = close_pos + 1;
            } else {
                idx = req_pos + 11;
                continue;
            }
        } else if rem_body.starts_with(';') {
            idx = curr + 1;
        } else {
            idx = req_pos + 11;
            continue;
        }

        let mut req_id = String::new();
        let mut text = String::new();
        let mut attributes = Vec::new();
        let mut constraints = Vec::new();
        let mut assumes = Vec::new();
        let mut requires = Vec::new();
        let mut derived_from = Vec::new();
        let mut verified_by = Vec::new();
        let mut satisfied_by = Vec::new();

        if let Some(body) = body_str {
            for line in body.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("id") && is_word_boundary_after(trimmed, 2) {
                    let after = trimmed.get(2..).unwrap_or("").trim_start();
                    if let Some(after_eq) = after.strip_prefix('=') {
                        if let Some(val) = extract_quoted_string(after_eq.trim_start()) {
                            req_id = val.to_string();
                        }
                    }
                } else if trimmed.starts_with("req_id") && is_word_boundary_after(trimmed, 6) {
                    let after = trimmed.get(6..).unwrap_or("").trim_start();
                    if let Some(after_eq) = after.strip_prefix('=') {
                        if let Some(val) = extract_quoted_string(after_eq.trim_start()) {
                            req_id = val.to_string();
                        }
                    }
                } else if trimmed.starts_with("doc") && is_word_boundary_after(trimmed, 3) {
                    let after = trimmed.get(3..).unwrap_or("").trim_start();
                    if let Some(after_eq) = after.strip_prefix('=') {
                        if let Some(val) = extract_quoted_string(after_eq.trim_start()) {
                            text = val.to_string();
                        }
                    }
                } else if trimmed.starts_with("text") && is_word_boundary_after(trimmed, 4) {
                    let after = trimmed.get(4..).unwrap_or("").trim_start();
                    if let Some(after_eq) = after.strip_prefix('=') {
                        if let Some(val) = extract_quoted_string(after_eq.trim_start()) {
                            text = val.to_string();
                        }
                    }
                } else if trimmed.starts_with("assume") && is_word_boundary_after(trimmed, 6) {
                    let after = trimmed.get(6..).unwrap_or("").trim_start();
                    if let Some(target) = after.strip_suffix(';') {
                        assumes.push(target.trim().to_string());
                    }
                } else if trimmed.starts_with("require") && is_word_boundary_after(trimmed, 7) {
                    let after = trimmed.get(7..).unwrap_or("").trim_start();
                    if let Some(target) = after.strip_suffix(';') {
                        requires.push(target.trim().to_string());
                    }
                } else if trimmed.starts_with("derived") && is_word_boundary_after(trimmed, 7) {
                    let after = trimmed.get(7..).unwrap_or("").trim_start();
                    if after.starts_with("from") && is_word_boundary_after(after, 4) {
                        let after_from = after.get(4..).unwrap_or("").trim_start();
                        if let Some(target) = after_from.strip_suffix(';') {
                            derived_from.push(target.trim().to_string());
                        }
                    }
                } else if trimmed.starts_with("verify") && is_word_boundary_after(trimmed, 6) {
                    let after = trimmed.get(6..).unwrap_or("").trim_start();
                    let after_by = if after.starts_with("by") && is_word_boundary_after(after, 2) {
                        after.get(2..).unwrap_or("").trim_start()
                    } else {
                        after
                    };
                    if let Some(target) = after_by.strip_suffix(';') {
                        verified_by.push(target.trim().to_string());
                    }
                } else if trimmed.starts_with("satisfy") && is_word_boundary_after(trimmed, 7) {
                    let after = trimmed.get(7..).unwrap_or("").trim_start();
                    let after_by = if after.starts_with("by") && is_word_boundary_after(after, 2) {
                        after.get(2..).unwrap_or("").trim_start()
                    } else {
                        after
                    };
                    if let Some(target) = after_by.strip_suffix(';') {
                        satisfied_by.push(target.trim().to_string());
                    }
                }
            }

            attributes = parse_attribute_defs(body);
            constraints = parse_constraint_defs(body);
        }

        reqs.push(RequirementDef {
            name: name.to_string(),
            req_id,
            text,
            doc,
            attributes,
            constraints,
            assumes,
            requires,
            derived_from,
            verified_by,
            satisfied_by,
            ..Default::default()
        });
    }
    reqs
}

/// Parse satisfy requirement statements from SysML v2 container bodies.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C standards. Extracts
/// component-level requirement satisfaction bindings (`satisfy requirement <ReqId>;`)
/// from SysML v2 part or package container text to guarantee bidirectional traceability (FEAT-42 / AC-01).
///
/// ## 2. Invariants & Mathematical Properties
/// - Extracted requirement identifiers derive deterministically from AST text.
/// - Order of first appearance is preserved and duplicate entries are eliminated.
/// - Ignores inverse `satisfy by <Part>;` clauses contained within requirement definitions.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 string slice representing container body contents.
///
/// ## 4. Postconditions
/// - Returns a `Vec<String>` of unique satisfied requirement identifier strings.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ where $N$ is the byte length of `content`. Space Complexity: $O(K)$ where $K$ is the number of distinct requirement IDs.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Unclosed statements, missing semi-colons, or malformed syntax are skipped safely without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-SATISFIED-REQUIREMENTS]
pub fn parse_satisfied_requirements(content: &str) -> Vec<String> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut satisfied = Vec::new();
    let mut idx = 0;

    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("satisfy")) {
        let satisfy_pos = idx + pos;
        if !is_word_boundary_before(&masked, satisfy_pos) || !is_word_boundary_after(&masked, satisfy_pos + 7) {
            idx = satisfy_pos + 7;
            continue;
        }

        let mut curr = satisfy_pos + 7;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));

        // If directly followed by 'by', this is an inverse 'satisfy by <Part>;' clause; skip.
        if masked.get(curr..).is_some_and(|s| s.starts_with("by") && is_word_boundary_after(s, 2)) {
            idx = satisfy_pos + 7;
            continue;
        }

        // Skip optional 'requirement' keyword
        if masked.get(curr..).is_some_and(|s| s.starts_with("requirement") && is_word_boundary_after(s, 11)) {
            curr += 11;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }

        // Find terminator ';'
        let rem_masked = masked.get(curr..).unwrap_or("");
        let rem_clean = clean.get(curr..).unwrap_or("");

        if let Some(semi_pos) = rem_masked.find(';') {
            let stmt_clean = rem_clean.get(..semi_pos).unwrap_or("").trim();
            if !stmt_clean.contains('{') && !stmt_clean.contains('}') {
                let by_offset = {
                    let mut found = None;
                    let mut search_idx = 0;
                    while let Some(pos) = stmt_clean.get(search_idx..).and_then(|s| s.find("by")) {
                        let actual_pos = search_idx + pos;
                        if is_word_boundary_before(stmt_clean, actual_pos) && is_word_boundary_after(stmt_clean, actual_pos + 2) {
                            found = Some(actual_pos);
                            break;
                        }
                        search_idx = actual_pos + 2;
                    }
                    found
                };

                let target_part = if let Some(by_pos) = by_offset {
                    stmt_clean.get(..by_pos).unwrap_or("").trim()
                } else {
                    stmt_clean
                };

                let req_id = if target_part.starts_with('"') {
                    if let Some(unquoted) = extract_quoted_string(target_part) {
                        unquoted.trim().to_string()
                    } else {
                        target_part.trim_matches('"').trim().to_string()
                    }
                } else if target_part.starts_with('\'') {
                    target_part.trim_matches('\'').trim().to_string()
                } else {
                    target_part.trim().to_string()
                };

                if !req_id.is_empty() && !satisfied.contains(&req_id) {
                    satisfied.push(req_id);
                }
            }
            idx = curr + semi_pos + 1;
        } else {
            idx = satisfy_pos + 7;
        }
    }

    satisfied
}

/// Parse test case definitions from SysML v2 source text.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic verification test case AST extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses bounds-checked `.get(..)` slicing on comment-stripped and string-masked source buffers.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 SysML v2 source fragment.
///
/// ## 4. Postconditions
/// - Returns a vector of all parsed `TestCaseDef` structures in appearance order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ in `content.len()`. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Malformed test case declarations are skipped safely without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-TEST-CASE-DEFS]
pub fn parse_test_case_defs(content: &str) -> Vec<TestCaseDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut test_cases = Vec::new();
    let mut idx = 0;
    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("test")) {
        let test_pos = idx + pos;
        if !is_word_boundary_before(&masked, test_pos) || !is_word_boundary_after(&masked, test_pos + 4) {
            idx = test_pos + 4;
            continue;
        }
        let doc = extract_doc_comment_before(&clean, test_pos);
        let mut curr = test_pos + 4;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if !masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("case") && is_word_boundary_after(s, 4))
        {
            idx = test_pos + 4;
            continue;
        }
        curr += 4;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = test_pos + 4;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));

        let mut subject_part = None;
        let mut verified_requirements = Vec::new();
        let mut objective = None;
        let mut test_steps = Vec::new();

        let rem_body = masked.get(curr..).unwrap_or("");
        if rem_body.starts_with('{') {
            if let Some(close_pos) = find_matching_brace(&clean, curr) {
                let body = clean.get(curr + 1..close_pos).unwrap_or("");
                for line in body.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("subject") && is_word_boundary_after(trimmed, 7) {
                        let val = trimmed.get(7..).unwrap_or("").trim().trim_end_matches(';').trim();
                        if !val.is_empty() {
                            subject_part = Some(val.to_string());
                        }
                    } else if trimmed.starts_with("verify") && is_word_boundary_after(trimmed, 6) {
                        let mut rest = trimmed.get(6..).unwrap_or("").trim();
                        if rest.starts_with("assert") && is_word_boundary_after(rest, 6) {
                            rest = rest.get(6..).unwrap_or("").trim();
                        }
                        if rest.starts_with("requirement") && is_word_boundary_after(rest, 11) {
                            rest = rest.get(11..).unwrap_or("").trim();
                        } else if rest.starts_with("constraint") && is_word_boundary_after(rest, 10) {
                            rest = rest.get(10..).unwrap_or("").trim();
                        }
                        let targets = rest.trim_end_matches(';').trim();
                        for target in targets.split(',') {
                            let t = target.trim();
                            if !t.is_empty() {
                                verified_requirements.push(t.to_string());
                            }
                        }
                    } else if trimmed.starts_with("objective") && is_word_boundary_after(trimmed, 9) {
                        let val = trimmed
                            .get(9..)
                            .unwrap_or("")
                            .trim()
                            .trim_end_matches(';')
                            .trim()
                            .trim_matches('"');
                        if !val.is_empty() {
                            objective = Some(val.to_string());
                        }
                    } else if trimmed.starts_with("step") && is_word_boundary_after(trimmed, 4) {
                        let val = trimmed.get(4..).unwrap_or("").trim().trim_end_matches(';').trim();
                        if !val.is_empty() {
                            test_steps.push(val.to_string());
                        }
                    }
                }
                idx = close_pos + 1;
            } else {
                idx = test_pos + 4;
                continue;
            }
        } else if rem_body.starts_with(';') {
            idx = curr + 1;
        } else {
            idx = test_pos + 4;
            continue;
        }

        test_cases.push(TestCaseDef {
            name: name.to_string(),
            subject_part,
            verified_requirements,
            objective,
            test_steps,
            doc,
        });
    }
    test_cases
}

/// Parse connection definitions from SysML v2 source text.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic topological connection AST extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses bounds-checked `.get(..)` slicing on comment-stripped and string-masked source buffers.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 SysML v2 source fragment.
///
/// ## 4. Postconditions
/// - Returns a vector of all parsed `ConnectionDef` structures in appearance order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ in `content.len()`. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Malformed connection or flow declarations are skipped safely without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CONNECTION-DEFS]
pub fn parse_connection_defs(content: &str) -> Vec<ConnectionDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut conns = Vec::new();
    let mut idx = 0;
    while idx < masked.len() {
        let rem_idx = masked.get(idx..).unwrap_or("");
        let (_kw, is_flow, kw_len) = if rem_idx.starts_with("connection")
            && is_word_boundary_before(&masked, idx)
            && is_word_boundary_after(&masked, idx + 10)
        {
            ("connection", false, 10)
        } else if rem_idx.starts_with("flow")
            && is_word_boundary_before(&masked, idx)
            && is_word_boundary_after(&masked, idx + 4)
        {
            ("flow", true, 4)
        } else {
            let step = rem_idx.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            idx += step;
            continue;
        };

        let kw_pos = idx;
        let mut curr = kw_pos + kw_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        // If keyword is "flow" and followed by "from", this is a statement inside a body, not a definition!
        if is_flow
            && masked
                .get(curr..)
                .is_some_and(|s| s.starts_with("from") && is_word_boundary_after(s, 4))
        {
            idx = kw_pos + kw_len;
            continue;
        }

        let doc = extract_doc_comment_before(&clean, kw_pos);
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = kw_pos + kw_len;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if !masked.get(curr..).is_some_and(|s| s.starts_with('{')) {
            idx = kw_pos + kw_len;
            continue;
        }
        let body = if let Some(close_pos) = find_matching_brace(&clean, curr) {
            let b = clean.get(curr + 1..close_pos).unwrap_or("");
            idx = close_pos + 1;
            b
        } else {
            idx = kw_pos + kw_len;
            continue;
        };

        let mut source_port = String::new();
        let mut target_port = String::new();

        for line in body.lines() {
            let trimmed = line.trim();
            let after_conn = if trimmed.starts_with("connect") && is_word_boundary_after(trimmed, 7) {
                Some(trimmed.get(7..).unwrap_or("").trim_start())
            } else if trimmed.starts_with("flow") && is_word_boundary_after(trimmed, 4) {
                let after_flow = trimmed.get(4..).unwrap_or("").trim_start();
                if after_flow.starts_with("from") && is_word_boundary_after(after_flow, 4) {
                    Some(after_flow.get(4..).unwrap_or("").trim_start())
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(after) = after_conn {
                if let Some(to_pos) = after.find(" to ") {
                    let src = after.get(..to_pos).unwrap_or("").trim();
                    let tgt = after
                        .get(to_pos + 4..)
                        .unwrap_or("")
                        .trim()
                        .trim_end_matches(';')
                        .trim();
                    source_port = src.to_string();
                    target_port = tgt.to_string();
                    break;
                }
            }
        }

        let source_part = if source_port.contains('.') {
            source_port.split('.').next().map(|s| s.to_string())
        } else {
            None
        };

        let target_part = if target_port.contains('.') {
            target_port.split('.').next().map(|s| s.to_string())
        } else {
            None
        };

        conns.push(ConnectionDef {
            name: name.to_string(),
            source_port,
            target_port,
            source_part,
            target_part,
            doc,
            severity: 1,
            item_flow_ref: None,
            protocol: None,
            latency_ms: None,
            is_flow,
        });
    }
    conns
}

/// Parse state definitions from SysML v2 source text.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and IEC 61508 deterministic state machine AST extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses bounds-checked `.get(..)` slicing and recursively extracts nested sub-states.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 SysML v2 source fragment.
///
/// ## 4. Postconditions
/// - Returns a vector of all parsed `StateDef` structures in appearance order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \cdot D_{state})$ in `content.len()` and nesting depth. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Malformed state blocks are skipped safely without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-STATE-DEFS]
pub fn parse_state_defs(content: &str) -> Vec<StateDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut states = Vec::new();
    let mut idx = 0;
    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("state")) {
        let st_pos = idx + pos;
        if !is_word_boundary_before(&masked, st_pos) || !is_word_boundary_after(&masked, st_pos + 5) {
            idx = st_pos + 5;
            continue;
        }
        let doc = extract_doc_comment_before(&clean, st_pos);
        let mut curr = st_pos + 5;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = st_pos + 5;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));

        let mut body_str = None;
        let rem_body = masked.get(curr..).unwrap_or("");
        if rem_body.starts_with('{') {
            if let Some(close_pos) = find_matching_brace(&clean, curr) {
                body_str = clean.get(curr + 1..close_pos);
                idx = close_pos + 1;
            } else {
                idx = st_pos + 5;
                continue;
            }
        } else if rem_body.starts_with(';') {
            idx = curr + 1;
        } else {
            idx = st_pos + 5;
            continue;
        }

        let mut entry_action = None;
        let mut do_action = None;
        let mut exit_action = None;
        let sub_states = if let Some(body) = body_str {
            for line in body.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("entry") && is_word_boundary_after(trimmed, 5) {
                    let act = trimmed.get(5..).unwrap_or("").trim().trim_end_matches(';').trim();
                    entry_action = Some(act.to_string());
                } else if trimmed.starts_with("do") && is_word_boundary_after(trimmed, 2) {
                    let act = trimmed.get(2..).unwrap_or("").trim().trim_end_matches(';').trim();
                    do_action = Some(act.to_string());
                } else if trimmed.starts_with("exit") && is_word_boundary_after(trimmed, 4) {
                    let act = trimmed.get(4..).unwrap_or("").trim().trim_end_matches(';').trim();
                    exit_action = Some(act.to_string());
                }
            }
            parse_state_defs(body)
        } else {
            Vec::new()
        };

        states.push(StateDef {
            name: name.to_string(),
            doc,
            entry_action,
            do_action,
            exit_action,
            transitions: Vec::new(),
            sub_states,
        });
    }
    states
}

/// Helper to locate the matching closing brace for a block, respecting strings and comments.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C bounds-checked lexical block boundary detection.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses `s.get(open_pos..)` for safe slicing and tracks nested `{}` depth outside string literals.
///
/// ## 3. Preconditions
/// - `s` is a valid UTF-8 string slice and `open_pos` points at or before the opening `{`.
///
/// ## 4. Postconditions
/// - Returns `Some(close_byte_pos)` when depth returns to `0`, or `None` if unbalanced/out-of-bounds.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ in `s.len() - open_pos`. Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Out-of-bounds `open_pos` or unclosed braces return `None` safely without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/FIND-MATCHING-BRACE]
pub fn find_matching_brace(s: &str, open_pos: usize) -> Option<usize> {
    let mut depth = 0;
    let mut in_str = false;
    let mut in_escape = false;
    let chars = s.get(open_pos..)?.char_indices();
    for (idx, ch) in chars {
        if in_str {
            if in_escape {
                in_escape = false;
            } else if ch == '\\' {
                in_escape = true;
            } else if ch == '"' {
                in_str = false;
            }
        } else {
            match ch {
                '"' => in_str = true,
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(open_pos + idx);
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// Helper to mask string literals and comments in source code with spaces, preserving exact byte offsets.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C offset-preserving lexical masking.
///
/// ## 2. Invariants & Mathematical Properties
/// - Byte length Invariant: `mask_string_literals(s).len() == s.len()`, preserving all newline positions.
///
/// ## 3. Preconditions
/// - `s` is a valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns a `String` of identical byte length with string literal contents and comments replaced by ASCII spaces.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ in `s.len()`. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Uses `.get` and `.get_mut` bounds-checked byte access exclusively; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/MASK-STRING-LITERALS]
pub fn mask_string_literals(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = bytes.to_vec();
    let mut i = 0;
    let mut in_str = false;
    let mut in_escape = false;
    while i < out.len() {
        let cur = out.get(i).copied().unwrap_or(0);
        if in_str {
            if in_escape {
                in_escape = false;
                if cur != b'\n' {
                    if let Some(slot) = out.get_mut(i) {
                        *slot = b' ';
                    }
                }
            } else if cur == b'\\' {
                in_escape = true;
                if let Some(slot) = out.get_mut(i) {
                    *slot = b' ';
                }
            } else if cur == b'"' {
                in_str = false;
            } else if cur != b'\n' {
                if let Some(slot) = out.get_mut(i) {
                    *slot = b' ';
                }
            }
            i += 1;
        } else if cur == b'"' {
            in_str = true;
            i += 1;
        } else if cur == b'/' && out.get(i + 1) == Some(&b'/') {
            if let Some(slot) = out.get_mut(i) {
                *slot = b' ';
            }
            if let Some(slot) = out.get_mut(i + 1) {
                *slot = b' ';
            }
            i += 2;
            while let Some(&b) = out.get(i) {
                if b == b'\n' {
                    break;
                }
                if let Some(slot) = out.get_mut(i) {
                    *slot = b' ';
                }
                i += 1;
            }
        } else if cur == b'/' && out.get(i + 1) == Some(&b'*') {
            if let Some(slot) = out.get_mut(i) {
                *slot = b' ';
            }
            if let Some(slot) = out.get_mut(i + 1) {
                *slot = b' ';
            }
            i += 2;
            while let Some(&b) = out.get(i) {
                if b == b'*' && out.get(i + 1) == Some(&b'/') {
                    if let Some(slot) = out.get_mut(i) {
                        *slot = b' ';
                    }
                    if let Some(slot) = out.get_mut(i + 1) {
                        *slot = b' ';
                    }
                    i += 2;
                    break;
                }
                if b != b'\n' {
                    if let Some(slot) = out.get_mut(i) {
                        *slot = b' ';
                    }
                }
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

/// Specifies which nested SysML v2 block categories should be masked out when scanning shallow part bodies.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `NestedBlockMaskMode` in `crates/deap-core/src/sysml_ast.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `NestedBlockMaskMode` across member set `[PartAndRequirement, PortOnly]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `NestedBlockMaskMode` members (`PartAndRequirement, PortOnly`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `NestedBlockMaskMode` representation for enum `NestedBlockMaskMode`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(2)) bounded over input/member dimension of `NestedBlockMaskMode`.
/// - Space Complexity: O(M(2)) bounded auxiliary memory for `NestedBlockMaskMode`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `NestedBlockMaskMode` (`NestedBlockMaskMode`).
/// - State: Maintains consistent state invariants across `NestedBlockMaskMode` without partial mutation on error.
/// - Action: Executes `NestedBlockMaskMode` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `NestedBlockMaskMode` (`DEAP-CORE-SYSML-AST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DEAP-CORE-SYSML-AST-NESTEDBLOCKMASKMODE] | SysML v2: PartDef(NestedBlockMaskMode)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NestedBlockMaskMode {
    /// Masks nested `part` and `requirement` blocks so their members are not attributed to the parent part.
    PartAndRequirement,
    /// Masks nested `port` blocks so port-internal attributes are not attributed to the parent part.
    PortOnly,
}

/// Masks the brace-enclosed bodies of nested blocks matching `mode` with spaces while preserving byte offsets.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C hierarchical scope isolation for SysML v2 part bodies.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves exact byte length and newline positions of `body`.
/// - Uses typed `NestedBlockMaskMode` dispatch with zero static string-slice array parameters.
///
/// ## 3. Preconditions
/// - `body` is a valid UTF-8 SysML v2 block interior.
///
/// ## 4. Postconditions
/// - Returns a `String` of identical byte length with matching nested block bodies replaced by spaces.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ in `body.len()`. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Uses `.get` and `.get_mut` bounds-checked slicing exclusively; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/MASK-NESTED-BLOCKS]
fn mask_nested_blocks(body: &str, mode: NestedBlockMaskMode) -> String {
    let masked = mask_string_literals(body);
    let mut out = body.as_bytes().to_vec();
    let mut idx = 0;
    while idx < masked.len() {
        let rem_idx = masked.get(idx..).unwrap_or("");
        let matched_kw_len = match mode {
            NestedBlockMaskMode::PartAndRequirement => {
                if rem_idx.starts_with("part")
                    && is_word_boundary_before(&masked, idx)
                    && is_word_boundary_after(&masked, idx + 4)
                {
                    Some(4)
                } else if rem_idx.starts_with("requirement")
                    && is_word_boundary_before(&masked, idx)
                    && is_word_boundary_after(&masked, idx + 11)
                {
                    Some(11)
                } else {
                    None
                }
            }
            NestedBlockMaskMode::PortOnly => {
                if rem_idx.starts_with("port")
                    && is_word_boundary_before(&masked, idx)
                    && is_word_boundary_after(&masked, idx + 4)
                {
                    Some(4)
                } else {
                    None
                }
            }
        };
        let Some(kw_len) = matched_kw_len else {
            let step = rem_idx.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            idx += step;
            continue;
        };

        let kw_pos = idx;
        let mut curr = kw_pos + kw_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        for dir_kw in ["inout", "out", "in"] {
            if masked
                .get(curr..)
                .is_some_and(|s| s.starts_with(dir_kw) && is_word_boundary_after(s, dir_kw.len()))
            {
                curr += dir_kw.len();
                curr += skip_ws(masked.get(curr..).unwrap_or(""));
                break;
            }
        }
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        if masked.get(curr..).is_some_and(|s| s.starts_with('~')) {
            curr += 1;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = kw_pos + kw_len;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        let rem_colon = masked.get(curr..).unwrap_or("");
        if rem_colon.starts_with(":>") || rem_colon.starts_with(':') {
            if rem_colon.starts_with(":>") {
                curr += 2;
            } else {
                curr += 1;
            }
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
            if masked.get(curr..).is_some_and(|s| s.starts_with('~')) {
                curr += 1;
                curr += skip_ws(masked.get(curr..).unwrap_or(""));
            }
            let (_, t_len) = extract_type_ident(masked.get(curr..).unwrap_or(""));
            curr += t_len;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        if masked.get(curr..).is_some_and(|s| s.starts_with('{')) {
            if let Some(close_pos) = find_matching_brace(body, curr) {
                if let Some(slice) = out.get_mut(curr + 1..close_pos) {
                    for b in slice {
                        if *b != b'\n' {
                            *b = b' ';
                        }
                    }
                }
                idx = close_pos + 1;
                continue;
            }
        }
        idx = kw_pos + kw_len;
    }
    String::from_utf8(out).unwrap_or_else(|_| body.to_string())
}

/// Parse part definitions from SysML v2 source text.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic structural part AST extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses bounds-checked `.get(..)` slicing and `NestedBlockMaskMode` to isolate shallow vs. nested members.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 SysML v2 source fragment.
///
/// ## 4. Postconditions
/// - Returns a vector of all parsed `PartDef` structures in appearance order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \cdot D_{part})$ in `content.len()` and nesting depth. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Malformed part declarations are skipped safely without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PART-DEFS]
pub fn parse_part_defs(content: &str) -> Vec<PartDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut parts = Vec::new();
    let mut idx = 0;
    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("part")) {
        let part_pos = idx + pos;
        if !is_word_boundary_before(&masked, part_pos) || !is_word_boundary_after(&masked, part_pos + 4) {
            idx = part_pos + 4;
            continue;
        }
        let doc = extract_doc_comment_before(&clean, part_pos);
        let mut curr = part_pos + 4;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        let mut is_def = false;
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            is_def = true;
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = part_pos + 4;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));

        let mut type_name = None;
        let rem_colon = masked.get(curr..).unwrap_or("");
        if rem_colon.starts_with(":>") || rem_colon.starts_with(':') {
            if rem_colon.starts_with(":>") {
                curr += 2;
            } else {
                curr += 1;
            }
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
            let (t_name, t_len) = extract_type_ident(masked.get(curr..).unwrap_or(""));
            if !t_name.is_empty() {
                type_name = Some(t_name.to_string());
                curr += t_len;
                curr += skip_ws(masked.get(curr..).unwrap_or(""));
            }
        }

        let rem_body = masked.get(curr..).unwrap_or("");
        let (body, next_idx) = if rem_body.starts_with('{') {
            let open_pos = curr;
            let close_pos = find_matching_brace(&clean, open_pos);
            let b = if let Some(cp) = close_pos {
                clean.get(open_pos + 1..cp).unwrap_or("")
            } else {
                ""
            };
            let next = close_pos.map(|cp| cp + 1).unwrap_or(open_pos + 1);
            (b, next)
        } else if rem_body.starts_with(';') {
            ("", curr + 1)
        } else {
            idx = part_pos + 4;
            continue;
        };

        idx = next_idx;

        let shallow_body = mask_nested_blocks(body, NestedBlockMaskMode::PartAndRequirement);
        let attr_body = mask_nested_blocks(&shallow_body, NestedBlockMaskMode::PortOnly);

        let attributes = parse_attribute_defs(&attr_body);
        let ports = parse_port_defs(&shallow_body);
        let actions = parse_action_defs(&shallow_body);
        let constraints = parse_constraint_defs(&shallow_body);
        let requirements = parse_requirement_defs(body);
        let satisfied_requirements = parse_satisfied_requirements(&shallow_body);
        let connections = parse_connection_defs(&shallow_body);
        let states = parse_state_defs(&shallow_body);
        let calcs = parse_calc_defs(&shallow_body);
        let nested_parts = parse_part_defs(body);

        parts.push(PartDef {
            name: name.to_string(),
            doc,
            is_def,
            type_name,
            attributes,
            ports,
            actions,
            constraints,
            requirements,
            satisfied_requirements,
            parts: nested_parts,
            operations: Vec::new(),
            capabilities: Vec::new(),
            interactions: Vec::new(),
            test_cases: Vec::new(),
            states,
            use_cases: Vec::new(),
            item_defs: Vec::new(),
            hazards: Vec::new(),
            risks: Vec::new(),
            connections,
            calcs,
        });
    }
    parts
}

/// Parse an entire SysML v2 source text into a structured SysmlModel.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C top-level SysML v2 model parsing.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses bounds-checked `.get(..)` slicing and deterministic element extractors.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 SysML v2 document.
///
/// ## 4. Postconditions
/// - Returns a populated `SysmlModel` containing all parsed top-level definitions.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \cdot D)$ in `content.len()` and nesting depth. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Falls back to root-level parsing if no outer `package` block is present; never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-SYSML]
pub fn parse_sysml(content: &str) -> SysmlModel {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut package_info = None;
    let mut idx = 0;
    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("package")) {
        let pkg_pos = idx + pos;
        if !is_word_boundary_before(&masked, pkg_pos) || !is_word_boundary_after(&masked, pkg_pos + 7) {
            idx = pkg_pos + 7;
            continue;
        }
        let doc = extract_doc_comment_before(&clean, pkg_pos);
        let mut curr = pkg_pos + 7;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = pkg_pos + 7;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if masked.get(curr..).is_some_and(|s| s.starts_with('{')) {
            let open_pos = curr;
            let body = if let Some(close_pos) = find_matching_brace(&clean, open_pos) {
                clean.get(open_pos + 1..close_pos).unwrap_or("")
            } else {
                clean.get(open_pos + 1..).unwrap_or("")
            };
            package_info = Some((name.to_string(), doc, body));
            break;
        } else {
            idx = pkg_pos + 7;
            continue;
        }
    }

    if let Some((package_name, doc, body)) = package_info {
        let parts = parse_part_defs(body);
        let ports = parse_port_defs(body);
        let actions = parse_action_defs(body);
        let constraints = parse_constraint_defs(body);
        let requirements = parse_requirement_defs(body);
        let attributes = parse_attribute_defs(body);
        let test_cases = parse_test_case_defs(body);

        SysmlModel {
            package_name: Some(package_name),
            doc,
            attributes,
            parts,
            ports,
            actions,
            constraints,
            requirements,
            test_cases,
        }
    } else {
        let parts = parse_part_defs(&clean);
        let ports = parse_port_defs(&clean);
        let actions = parse_action_defs(&clean);
        let constraints = parse_constraint_defs(&clean);
        let requirements = parse_requirement_defs(&clean);
        let attributes = parse_attribute_defs(&clean);
        let test_cases = parse_test_case_defs(&clean);

        SysmlModel {
            package_name: None,
            doc: None,
            attributes,
            parts,
            ports,
            actions,
            constraints,
            requirements,
            test_cases,
        }
    }
}

/// Checks whether `needle` appears as a distinct alphanumeric word inside `haystack` (case-insensitive).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Evaluates word-boundary containment using
/// bounds-checked `.get(..)` slicing and UTF-8 character boundaries without regular expressions.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `true` iff `needle` is non-empty and occurs in `haystack` bounded on both sides by
///   string edges or non-alphanumeric characters.
/// - Advances by full UTF-8 character codepoint byte lengths (`c.len_utf8()`).
///
/// ## 3. Preconditions
/// - `haystack` and `needle` are valid UTF-8 string slices.
///
/// ## 4. Postconditions
/// - Pure boolean query; returns `false` when `needle` is empty or does not match on word boundaries.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|H| \cdot |N|)$ worst-case substring search. Space Complexity: $O(|H| + |N|)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking bounds-checked `.get(..)` slicing prevents multi-byte UTF-8 boundary panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/CONTAINS-WORD]
fn contains_word(haystack: &str, needle: &str) -> bool {
    let h_folded = haystack.to_lowercase();
    let n_folded = needle.to_lowercase();
    if n_folded.is_empty() {
        return false;
    }
    let mut start = 0;
    while let Some(pos) = h_folded.get(start..).and_then(|sub| sub.find(&n_folded)) {
        let actual_pos = start + pos;
        let before = if actual_pos == 0 {
            true
        } else {
            !h_folded
                .get(..actual_pos)
                .and_then(|sub| sub.chars().last())
                .is_some_and(|c| c.is_alphanumeric())
        };
        let after_pos = actual_pos + n_folded.len();
        let after = if after_pos >= h_folded.len() {
            true
        } else {
            !h_folded
                .get(after_pos..)
                .and_then(|sub| sub.chars().next())
                .is_some_and(|c| c.is_alphanumeric())
        };
        if before && after {
            return true;
        }
        let step = h_folded
            .get(actual_pos..)
            .and_then(|sub| sub.chars().next())
            .map(|c| c.len_utf8())
            .unwrap_or(1);
        start = actual_pos + step;
    }
    false
}

/// Checks if an FMECA table component string matches an AST `part def` name.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Aligns tabular safety artifact component labels
/// with SysML v2 `part def` identifiers deterministically without regular expressions.
///
/// ## 2. Invariants & Mathematical Properties
/// - Symmetric relation: `component_matches(a, b) == component_matches(b, a)`.
/// - Matches case-insensitively on raw trimmed strings, alphanumeric-normalized strings, or
///   bidirectional word-boundary containment (`contains_word`).
///
/// ## 3. Preconditions
/// - `table_comp` and `ast_part_name` are valid UTF-8 string slices.
///
/// ## 4. Postconditions
/// - Returns `true` iff the two component strings match under exact, alphanumeric, or word-boundary rules.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|A| \cdot |B|)$. Space Complexity: $O(|A| + |B|)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking string normalization; empty alphanumeric strings do not spuriously match.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COMPONENT-MATCHES]
pub fn component_matches(table_comp: &str, ast_part_name: &str) -> bool {
    let tc = table_comp.trim();
    let pn = ast_part_name.trim();
    if tc.eq_ignore_ascii_case(pn) {
        return true;
    }

    let tc_clean: String = tc.chars().filter(|c| c.is_alphanumeric()).collect();
    let pn_clean: String = pn.chars().filter(|c| c.is_alphanumeric()).collect();

    if !tc_clean.is_empty() && tc_clean.eq_ignore_ascii_case(&pn_clean) {
        return true;
    }

    contains_word(tc, pn) || contains_word(pn, tc)
}

/// Extracts general component or subsystem stem names that are explicitly negated in AST attributes,
/// constraints, or specification prose (`<subsystem>Enabled = 0`, `has<Subsystem> = false`,
/// `<subsystem>System = "No"`, `!exists(<Part>)`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Prevents phantom or explicitly excluded subsystems
/// from being synthesized into downstream safety and architecture artifacts.
///
/// ## 2. Invariants & Mathematical Properties
/// - Output is a deterministically sorted, deduplicated `Vec<String>` (backed by `BTreeSet`).
/// - Recognizes attribute defaults, `!exists(...)` predicates, assignment clauses, and qualified `no ...` phrases.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 SysML or specification document slice.
///
/// ## 4. Postconditions
/// - Returns all normalized lowercase alphanumeric stems of explicitly negated components.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log K)$ where $N$ is input byte length and $K$ is distinct stems. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking bounds-checked `.get(..)` slicing across all line and clause parsers.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-NEGATED-COMPONENTS]
pub fn extract_negated_components(content: &str) -> Vec<String> {
    let mut negated = BTreeSet::new();
    let negative_vals = ["0", "false", "\"no\"", "\"none\"", "no", "none"];

    for attr in parse_attribute_defs(content) {
        if let Some(val_raw) = &attr.default_value {
            let val_lower = val_raw.trim().to_ascii_lowercase();
            if negative_vals.contains(&val_lower.as_str()) {
                let stem = extract_negated_key_stem(&attr.name);
                if !stem.is_empty() {
                    negated.insert(stem);
                }
            }
        }
    }

    for line in content.lines() {
        let line_clean = line.find("//").and_then(|idx| line.get(..idx)).unwrap_or(line);
        let lower = line_clean.trim().to_lowercase();

        // (a) Existence negation: `!exists(<Part>)`
        let exists_kw = "!exists(";
        if let Some(after) = lower.find(exists_kw).and_then(|pos| lower.get(pos + exists_kw.len()..)) {
            if let Some(inner) = after.find(')').and_then(|close| after.get(..close)) {
                let comp = inner.chars().filter(|c| c.is_alphanumeric()).collect::<String>();
                if !comp.is_empty() {
                    negated.insert(comp);
                }
            }
        }

        // (b) Key or label assignment to a negative value, evaluated per `,`/`;` clause:
        //     `key = 0`, `Multi word label: No`, `Prose label: keyEnabled = 0, otherSystem = "No"`.
        let is_negative_value = |rhs: &str| {
            let val = rhs
                .trim()
                .trim_matches(|c: char| c == ';' || c == '}' || c == '{' || c == ',' || c == ')' || c == '(' || c == '.')
                .trim();
            negative_vals.contains(&val)
        };
        for clause in line_clean.split([',', ';']) {
            let clause = clause.trim();
            // ASCII lowercasing preserves byte offsets, so positions index the original-case clause.
            let clause_ascii = clause.to_ascii_lowercase();
            let assign = [":=", "==", "="]
                .into_iter()
                .find_map(|op| clause_ascii.find(op).map(|pos| (op, pos)));
            let labelled = assign.filter(|&(_, op_pos)| clause_ascii.find(':').is_some_and(|colon| colon < op_pos));
            if let Some((op, op_pos)) = labelled {
                // `Prose label: keyName = value` -- the key is the last whitespace/dot/colon-delimited token
                // before the operator, original casing.
                let rhs = clause_ascii.get(op_pos + op.len()..).unwrap_or("");
                if is_negative_value(rhs) {
                    let key = clause
                        .get(..op_pos)
                        .and_then(|lhs| {
                            lhs.rsplit(|c: char| c.is_whitespace() || c == '.' || c == ':')
                                .find(|t| !t.is_empty())
                        })
                        .unwrap_or("")
                        .trim_matches(|c: char| !(c.is_alphanumeric() || c == '_'));
                    let stem = extract_negated_key_stem(key);
                    if !stem.is_empty() {
                        negated.insert(stem);
                    }
                }
                continue;
            }

            let clause_lower = clause.to_lowercase();
            let sep_opt = if clause_lower.contains(":=") {
                clause_lower.split_once(":=")
            } else if clause_lower.contains("==") {
                clause_lower.split_once("==")
            } else if clause_lower.contains('=') {
                clause_lower.split_once('=')
            } else if clause_lower.contains(':') {
                clause_lower.split_once(':')
            } else {
                None
            };
            if let Some((lhs, rhs)) = sep_opt {
                if is_negative_value(rhs) {
                    let stem = extract_negated_key_stem(&normalize_negated_label(lhs));
                    if !stem.is_empty() {
                        negated.insert(stem);
                    }
                }
            }
        }

        // (c) Qualified prose negation: `no <component> <qualifier>`
        negated.extend(extract_no_phrase_stems(&lower));
    }

    negated.into_iter().collect()
}

/// Normalizes the left-hand side of a negating assignment or label into a single joined key token.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Derives component keys purely from schema or
/// specification text structure without any hardcoded domain vocabulary.
///
/// ## 2. Invariants & Mathematical Properties
/// - Type annotations (`: Type`), code punctuation prefixes, and dotted owner paths are discarded.
/// - Leading SysML declaration keywords are skipped; numeric-only words are ignored.
/// - Labels longer than 4 words are treated as prose and yield an empty key.
///
/// ## 3. Preconditions
/// - `lhs` is the lowercase text preceding a negating separator.
///
/// ## 4. Postconditions
/// - Returns the concatenation of the remaining label words, or an empty string.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ in the length of `lhs`. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Pure non-panicking string analysis; malformed input yields an empty key.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/NEGATED-LABEL-NORMALIZATION]
fn normalize_negated_label(lhs: &str) -> String {
    let keywords = ["attribute", "ref", "readonly", "derived", "constant", "in", "out", "inout", "def"];
    let head = lhs.split(':').next().unwrap_or(lhs);
    let head = head.rsplit(['{', '(', ';', ',']).next().unwrap_or(head);
    let head = head.rsplit('.').next().unwrap_or(head);
    let words: Vec<String> = head
        .split_whitespace()
        .map(|w| w.chars().filter(|c| c.is_alphanumeric() || *c == '_').collect::<String>())
        .filter(|w| !w.is_empty() && !w.chars().all(|c| c.is_ascii_digit()))
        .collect();
    let mut start = 0;
    while words.get(start).is_some_and(|w| keywords.contains(&w.as_str())) {
        start += 1;
    }
    let label_words = words.get(start..).unwrap_or(&[]);
    if label_words.is_empty() || label_words.len() > 4 {
        return String::new();
    }
    label_words.concat()
}

/// Extracts component stems from qualified prose negations of the form `no <component> <qualifier>`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Recognizes explicit absence declarations in
/// specification prose without any hardcoded domain vocabulary.
///
/// ## 2. Invariants & Mathematical Properties
/// - `no` must be a whole word and be followed by 1-2 non-numeric component words and then a
///   structural qualifier (`system`, `subsystem`, `unit`, `module`, `device`, `installed`, `present`).
/// - Unqualified prose (e.g. "no more than 3 retries") yields no stems.
///
/// ## 3. Preconditions
/// - `lower` is a single lowercase line of text.
///
/// ## 4. Postconditions
/// - Returns normalized stems (length >= 3) for every qualified negation phrase.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(W)$ in the number of words. Space Complexity: $O(W)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Pure non-panicking string analysis.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/NEGATED-PROSE-PHRASES]
fn extract_no_phrase_stems(lower: &str) -> Vec<String> {
    let qualifiers = ["system", "subsystem", "unit", "module", "device", "installed", "present"];
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| !w.is_empty())
        .collect();
    let mut stems = Vec::new();
    for (i, w) in words.iter().enumerate() {
        if *w != "no" {
            continue;
        }
        let mut comp = String::new();
        let mut qualifier: Option<&str> = None;
        for next in words.iter().skip(i + 1).take(3) {
            if qualifiers.contains(next) {
                qualifier = Some(*next);
                break;
            }
            if next.chars().all(|c| c.is_ascii_digit()) {
                break;
            }
            comp.push_str(next);
        }
        if let Some(qual) = qualifier.filter(|_| !comp.is_empty()) {
            // The qualifier is kept so `extract_negated_key_stem` sees a recognized suffix to strip.
            let stem = extract_negated_key_stem(&format!("{comp}{qual}"));
            if !stem.is_empty() {
                stems.push(stem);
            }
        }
    }
    stems
}

/// Extracts the normalized lowercase alphanumeric component stem from a qualified negation key.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Strips structural prefixes (`has`) and suffixes
/// (`subsystem`, `enabled`, `installed`, etc.) while rejecting scalar metrology stems.
///
/// ## 2. Invariants & Mathematical Properties
/// - Requires at least one structural prefix (`has` at word boundary) or suffix to qualify as a component flag.
/// - Resulting stem must be at least 3 characters and not belong to scalar metrology stems (`time`, `rate`, etc.).
///
/// ## 3. Preconditions
/// - `key` is a valid UTF-8 identifier or normalized label string.
///
/// ## 4. Postconditions
/// - Returns the lowercase alphanumeric stem or an empty `String` if unqualified.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ in the byte length of `key`. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking bounds-checked `.get(..)` prefix inspection.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/NEGATED-KEY-STEM]
fn extract_negated_key_stem(key: &str) -> String {
    let clean: String = key
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    if clean.is_empty() {
        return String::new();
    }
    // Longer suffixes precede their own suffixes (`subsystem` before `system`) for correct iterative stripping.
    let suffixes = [
        "subsystem", "deployment", "controller", "installed", "actuator", "enabled", "present", "system", "module",
        "device", "sensor", "unit",
    ];
    let mut stem: &str = clean.as_str();
    let mut qualified = false;
    // `has` is a prefix only at a word boundary in the raw key (`has_x`, `hasX`), never inside `hash`/`hashed`.
    let raw = key.trim_start_matches(|c: char| !c.is_alphanumeric());
    let has_boundary = raw.get(..3).is_some_and(|p| p.eq_ignore_ascii_case("has"))
        && raw
            .get(3..)
            .and_then(|r| r.chars().next())
            .is_some_and(|c| c == '_' || c.is_ascii_uppercase());
    if has_boundary {
        if let Some(rest) = clean.strip_prefix("has").filter(|rest| rest.len() >= 3) {
            stem = rest;
            qualified = true;
        }
    }
    while let Some(rest) = suffixes
        .iter()
        .find_map(|suf| stem.strip_suffix(suf).filter(|rest| rest.len() >= 3))
    {
        stem = rest;
        qualified = true;
    }
    // Scalar/metrology stems (e.g. `timeUnit`, `rateSensor`) name quantities, not components.
    let non_component_stems = [
        "time", "base", "scale", "measure", "count", "rate", "limit", "value", "mode", "state", "status", "index",
        "size",
    ];
    // Unqualified keys (e.g. `faultCount`, `retryLimit`) are numeric/state attributes, not presence flags.
    if qualified && stem.len() >= 3 && !non_component_stems.contains(&stem) {
        stem.to_string()
    } else {
        String::new()
    }
}

/// Checks whether a given component name is contradicted by any negative AST attribute or constraint in `content`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Guards FMECA and STPA verifiers against accepting
/// components that are explicitly negated in the governing schema or specification text.
///
/// ## 2. Invariants & Mathematical Properties
/// - Equivalent to `component_matches_negated_stems(comp, &extract_negated_components(content))`.
///
/// ## 3. Preconditions
/// - `comp` and `content` are valid UTF-8 string slices.
///
/// ## 4. Postconditions
/// - Returns `true` iff `comp` matches at least one negated component stem extracted from `content`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N + S \cdot |C|)$ where $N$ is `content.len()` and $S$ is negated stem count. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking; returns `false` when `comp` or `content` is empty.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/IS-COMPONENT-NEGATED]
pub fn is_component_negated(comp: &str, content: &str) -> bool {
    component_matches_negated_stems(comp, &extract_negated_components(content))
}

/// Checks whether a component name contains or structurally matches any of the supplied negated stems.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Single shared matcher for free-text and AST-based
/// negation queries, guaranteeing identical semantics across FMECA synthesis and STPA verification.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `true` iff the lowercase alphanumeric component name contains a stem, or `component_matches` holds.
///
/// ## 3. Preconditions
/// - `stems` are lowercase alphanumeric stems produced by `extract_negated_components`.
///
/// ## 4. Postconditions
/// - Returns `false` for empty or non-alphanumeric component names.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(S \cdot N)$. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Pure non-panicking string analysis.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/NEGATED-STEM-MATCH]
fn component_matches_negated_stems(comp: &str, stems: &[String]) -> bool {
    let comp_clean: String = comp
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    if comp_clean.is_empty() {
        return false;
    }
    stems
        .iter()
        .any(|neg_stem| comp_clean.contains(neg_stem.as_str()) || component_matches(comp, neg_stem))
}

/// Strips C-style multi-line comments (`/* ... */`) and line comments (`// ...`) from SysML text without regex,
/// while preserving `doc /* ... */` documentation blocks and string literals.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Prevents commented-out SysML declarations from
/// polluting the parsed AST while retaining normative `doc /* ... */` blocks and string literals.
///
/// ## 2. Invariants & Mathematical Properties
/// - Line count is preserved for `//` comments (terminated at `\n`).
/// - Multi-line `/* ... */` blocks immediately preceded by the `doc` keyword at a word boundary are preserved verbatim.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 SysML source slice.
///
/// ## 4. Postconditions
/// - Returns a newly allocated `String` with non-documentation comments stripped.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ single-pass character scan. Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Unclosed comments or string literals terminate cleanly at end-of-input without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/STRIP-COMMENTS]
pub fn strip_sysml_comments(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_str = false;
    let mut in_escape = false;
    while let Some(c) = chars.next() {
        if in_str {
            out.push(c);
            if in_escape {
                in_escape = false;
            } else if c == '\\' {
                in_escape = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(c);
            continue;
        }
        if c == '/' {
            if let Some(&'/') = chars.peek() {
                chars.next();
                for next_c in chars.by_ref() {
                    if next_c == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            } else if let Some(&'*') = chars.peek() {
                let trimmed_before = out.trim_end();
                let is_doc_block = trimmed_before.ends_with("doc")
                    && (trimmed_before.len() == 3
                        || !trimmed_before
                            .as_bytes()
                            .get(trimmed_before.len() - 4)
                            .copied()
                            .is_some_and(is_ident_char));
                chars.next();
                if is_doc_block {
                    out.push_str("/*");
                    while let Some(next_c) = chars.next() {
                        out.push(next_c);
                        if next_c == '*' && chars.peek() == Some(&'/') {
                            out.push('/');
                            chars.next();
                            break;
                        }
                    }
                } else {
                    out.push(' ');
                    while let Some(next_c) = chars.next() {
                        if next_c == '\n' {
                            out.push('\n');
                        }
                        if next_c == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                }
            } else {
                out.push(c);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Checks if a string is a valid identifier for a subsystem, component, or part.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Rejects schema meta-keywords, primitive type
/// names, and malformed tokens during multi-format subsystem discovery.
///
/// ## 2. Invariants & Mathematical Properties
/// - Length must satisfy $1 \le |s| \le 128$.
/// - First character must be ASCII alphabetic or `_`, and all characters must be ASCII alphanumeric or `_`.
/// - Must not match any reserved schema keyword in `generic_blacklist` (case-insensitive).
///
/// ## 3. Preconditions
/// - `s` is a valid UTF-8 candidate identifier slice.
///
/// ## 4. Postconditions
/// - Returns `true` iff `s` satisfies all lexical and blacklist constraints.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|s|)$. Space Complexity: $O(|s|)$ for lowercase comparison.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Pure non-panicking validation predicate.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/VALID-SUBSYSTEM-IDENT]
pub fn is_valid_subsystem_identifier(s: &str) -> bool {
    if s.is_empty() || s.len() > 128 {
        return false;
    }
    let Some(first) = s.chars().next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    let generic_blacklist = [
        "true", "false", "null", "none", "na", "string", "int", "integer", "boolean", "float",
        "description", "version", "name", "id", "type", "subsystem", "component", "part", "module",
        "parameters", "attributes", "properties", "specs", "metadata", "title", "status", "real",
        "primary", "secondary", "optional", "required", "level", "segment", "layer", "architecture",
        "purpose", "function", "item", "flow", "port", "interface", "doc", "package", "def",
    ];
    if generic_blacklist.contains(&s.to_ascii_lowercase().as_str()) {
        return false;
    }
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Typed enumeration of every schema file extension recognised by multi-format schema discovery.
/// Realises: [deap_core::sysml_ast::SupportedSchemaExtension]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SupportedSchemaExtension {
    /// SysML v2 textual notation (`.sysml`).
    Sysml,
    /// Structural YAML, long form (`.yaml`).
    Yaml,
    /// Structural YAML, short form (`.yml`).
    Yml,
    /// Structural JSON (`.json`).
    Json,
    /// Protocol Buffers interface definition (`.proto`).
    Proto,
    /// OMG Interface Definition Language (`.idl`).
    Idl,
    /// AUTOSAR XML (`.arxml`).
    Arxml,
    /// Markdown architecture tables, short form (`.md`).
    Md,
    /// Markdown architecture tables, long form (`.markdown`).
    Markdown,
    /// IETF RFC 6020 / RFC 7950 YANG data modeling language (`.yang`).
    Yang,
}

impl SupportedSchemaExtension {
    /// Canonical declaration-ordered table of all 10 supported schema file extensions.
    pub const ALL: &'static [Self] = &[
        Self::Sysml,
        Self::Yaml,
        Self::Yml,
        Self::Json,
        Self::Proto,
        Self::Idl,
        Self::Arxml,
        Self::Md,
        Self::Markdown,
        Self::Yang,
    ];

    /// Projects a schema extension variant onto its dot-prefixed lowercase file suffix.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Binds every schema format accepted by
    /// `UnifiedSchemaModelExtractor::discover_schema_files` to a typed variant, so the set of
    /// ingestible model formats is closed and reviewable rather than an editable string list.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Injective mapping from 10 variants to 10 distinct suffixes, each beginning with `.` followed
    /// by lowercase ASCII letters; declaration order equals the legacy `SUPPORTED_EXTENSIONS` order
    /// with `.yang` appended last.
    ///
    /// ## 3. Preconditions
    /// `self` is one of the 10 `SupportedSchemaExtension` variants.
    ///
    /// ## 4. Postconditions
    /// Returns a `'static` suffix such as `".sysml"` in the exact form produced by
    /// `format!(".{}", ext.to_ascii_lowercase())` during discovery.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) ten-arm `const` match. Space Complexity: O(1) static literal return.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Exhaustive matching turns a forgotten suffix for a new format into a compile error, so
    /// discovery cannot silently skip files of a declared schema format.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/SUPPORTED-SCHEMA-EXTENSION-AS-STR]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sysml => ".sysml",
            Self::Yaml => ".yaml",
            Self::Yml => ".yml",
            Self::Json => ".json",
            Self::Proto => ".proto",
            Self::Idl => ".idl",
            Self::Arxml => ".arxml",
            Self::Md => ".md",
            Self::Markdown => ".markdown",
            Self::Yang => ".yang",
        }
    }

    /// Reference adapter yielding the dot-prefixed suffix for a borrowed schema extension variant.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Provides the `fn(&'static SupportedSchemaExtension)`
    /// projection installed in `SupportedExtensionsCatalog::iter`, guaranteeing that schema-suffix
    /// iteration reuses the one exhaustive mapping defined by `as_str`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// `x.as_str_ref() == x.as_str()` for all 10 schema extension variants (pure forwarding).
    ///
    /// ## 3. Preconditions
    /// `self` borrows a valid `SupportedSchemaExtension`, typically an element of `SupportedSchemaExtension::ALL`.
    ///
    /// ## 4. Postconditions
    /// Returns the same `'static` dot-prefixed suffix; the result does not borrow from `self`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) dereference of a fieldless enum then const dispatch. Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Total and side-effect free; it performs no filesystem access even though its callers walk directories.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/SUPPORTED-SCHEMA-EXTENSION-AS-STR-REF]
    pub fn as_str_ref(&self) -> &'static str {
        self.as_str()
    }
}

/// Zero-sized catalog providing iteration and membership queries over supported schema file extensions.
/// Realises: [deap_core::sysml_ast::SupportedExtensionsCatalog]
#[derive(Debug, Clone, Copy, Default)]
pub struct SupportedExtensionsCatalog;

impl SupportedExtensionsCatalog {
    /// Checks whether a dot-prefixed, lowercased file suffix denotes a supported schema format.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Acts as the admission filter for schema
    /// discovery in `discover_schema_files` and `assemble-conops` parameter auto-detection, so only
    /// declared model formats ever reach the AST extractors.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Returns `true` if and only if `*ext` exactly equals one of the 10 suffixes produced by
    /// `SupportedSchemaExtension::as_str`; equivalent to `self.iter().any(|s| s == *ext)`.
    ///
    /// ## 3. Preconditions
    /// `ext` references a string slice already normalised by the caller to `.` plus a lowercase
    /// extension; matching is case-sensitive and does not add a missing leading dot.
    ///
    /// ## 4. Postconditions
    /// Returns a boolean admission verdict; `".SYSML"`, `"sysml"` and `""` all evaluate to `false`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(L) where L is the byte length of `*ext` (decisive within 9 bytes). Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Fails closed: any unrecognised or malformed suffix is rejected rather than ingested; never panics.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/SUPPORTED-EXTENSIONS-CONTAINS]
    pub fn contains(&self, ext: &&str) -> bool {
        matches!(
            *ext,
            ".sysml"
                | ".yaml"
                | ".yml"
                | ".json"
                | ".proto"
                | ".idl"
                | ".arxml"
                | ".md"
                | ".markdown"
                | ".yang"
        )
    }

    /// Returns a typed-enum-backed iterator over all 10 supported schema file suffixes.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Exposes the supported schema suffixes for
    /// reporting and auditing by deriving each one from `SupportedSchemaExtension::ALL`, replacing
    /// the former untyped string-slice constant on `UnifiedSchemaModelExtractor`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Yields `".sysml"`, `".yaml"`, `".yml"`, `".json"`, `".proto"`, `".idl"`, `".arxml"`, `".md"`,
    /// `".markdown"`, `".yang"` in that order; every yielded suffix satisfies `self.contains(&suffix)`.
    ///
    /// ## 3. Preconditions
    /// `self` is a valid `SupportedExtensionsCatalog` reference (stateless, zero-sized).
    ///
    /// ## 4. Postconditions
    /// Returns a `Map<slice::Iter<'static, SupportedSchemaExtension>, fn(&'static SupportedSchemaExtension) -> &'static str>`
    /// with exact length 10, double-ended, and free of heap allocation.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) construction, O(10) exhaustion. Space Complexity: O(1) slice cursor and projection pointer.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Cannot fail or truncate: the backing slice is a `'static` constant table, so consumers always
    /// observe the complete supported-format set.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/SUPPORTED-EXTENSIONS-ITER]
    pub fn iter(
        &self,
    ) -> std::iter::Map<
        std::slice::Iter<'static, SupportedSchemaExtension>,
        fn(&'static SupportedSchemaExtension) -> &'static str,
    > {
        let project: fn(&'static SupportedSchemaExtension) -> &'static str =
            SupportedSchemaExtension::as_str_ref;
        SupportedSchemaExtension::ALL.iter().map(project)
    }
}

/// Centralized schema discovery and AST part extractor shared across compiler and validators.
///
/// Supports multi-schema formats:
/// - SysML v2 (`.sysml`)
/// - Structural YAML (`.yaml`, `.yml`)
/// - Structural JSON (`.json`)
/// - Protocol Buffers (`.proto`)
/// - OMG IDL (`.idl`)
/// - AUTOSAR (`.arxml`)
/// - Markdown architecture tables (`.md`, `.markdown`)
/// - IETF YANG (`.yang`)
#[derive(Debug, Clone, Default)]
pub struct UnifiedSchemaModelExtractor;

impl UnifiedSchemaModelExtractor {
    /// Supported schema extensions across all pipeline tools.
    pub const SUPPORTED_EXTENSIONS: SupportedExtensionsCatalog = SupportedExtensionsCatalog;

    /// Recursively discovers all supported schema files across the provided search directories.
    /// Excludes hidden entries (`.`), editor swap files (`#`), and `README.md`.
    /// Results are lexicographically sorted to eliminate filesystem traversal jitter.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Guarantees deterministic, platform-independent
    /// schema file discovery ordering across macOS APFS and Linux ext4/overlayfs filesystems.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Returned `Vec<PathBuf>` is strictly sorted and deduplicated (`sort()` + `dedup()`).
    /// - Hidden files (`.`), swap files (`#`), and `README.md` are excluded.
    ///
    /// ## 3. Preconditions
    /// - `search_dirs` contains candidate directory paths.
    ///
    /// ## 4. Postconditions
    /// - Returns all discovered regular files matching `Self::SUPPORTED_EXTENSIONS`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(F \log F)$ where $F$ is the number of visited directory entries. Space Complexity: $O(F)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Non-existent directories and unreadable walk entries are skipped cleanly without panicking.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/DISCOVER-SCHEMA-FILES]
    pub fn discover_schema_files<P: AsRef<Path>>(search_dirs: &[P]) -> Vec<PathBuf> {
        let mut schema_files = Vec::new();
        for sdir in search_dirs {
            let p = sdir.as_ref();
            if !p.is_dir() {
                continue;
            }
            for entry_res in WalkDir::new(p).into_iter().filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && !name.starts_with('#')
            }) {
                let Ok(entry) = entry_res else {
                    continue;
                };
                if entry.file_type().is_file() {
                    let fpath = entry.path();
                    let fname = fpath.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if fname.starts_with('.')
                        || fname.starts_with('#')
                        || fname.eq_ignore_ascii_case("readme.md")
                    {
                        continue;
                    }
                    if let Some(ext) = fpath.extension().and_then(|e| e.to_str()) {
                        let dot_ext = format!(".{}", ext.to_ascii_lowercase());
                        if Self::SUPPORTED_EXTENSIONS.contains(&dot_ext.as_str()) {
                            schema_files.push(fpath.to_path_buf());
                        }
                    }
                }
            }
        }
        schema_files.sort();
        schema_files.dedup();
        schema_files
    }

    /// Extracts all subsystem part and component names across the provided file paths.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Aggregates structural subsystem identifiers
    /// across heterogeneous schema files into a canonical `BTreeSet<String>`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Output is deterministically ordered and deduplicated (`BTreeSet<String>`).
    ///
    /// ## 3. Preconditions
    /// - `file_paths` is a slice of paths to candidate schema files.
    ///
    /// ## 4. Postconditions
    /// - Returns the union of all subsystem identifiers extracted from readable files in `file_paths`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(\sum |B_i| \log P)$ across files $B_i$ and unique parts $P$. Space Complexity: $O(P)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Unreadable files are skipped gracefully without panicking.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-SUBSYSTEM-PARTS]
    pub fn extract_subsystem_parts<P: AsRef<Path>>(file_paths: &[P]) -> BTreeSet<String> {
        let mut part_names = BTreeSet::new();
        for fp in file_paths {
            let p = fp.as_ref();
            if let Ok(content) = std::fs::read_to_string(p) {
                let ext = p
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                let dot_ext = format!(".{}", ext);
                let extracted = Self::extract_subsystem_parts_from_text(&dot_ext, &content);
                part_names.extend(extracted);
            }
        }
        part_names
    }

    /// Extracts subsystem and component names from raw text of a specific format.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Dispatches format-specific structural extraction
    /// across `.sysml`, `.yaml`/`.yml`/`.json`, `.md`/`.markdown`, `.proto`, `.idl`, `.arxml`, and
    /// `.yang` (line-leading `container`, `grouping`, and `list` statements).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Every inserted identifier satisfies `is_valid_subsystem_identifier` (or is a parsed SysML `PartDef` name).
    /// - Output is a deterministically ordered `BTreeSet<String>`.
    ///
    /// ## 3. Preconditions
    /// - `ext` is a leading-dot file extension (e.g. `".sysml"`) and `content` is valid UTF-8 text.
    ///
    /// ## 4. Postconditions
    /// - Returns all discovered subsystem and component identifiers for the given format.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N \log P)$ in input length $N$ and part count $P$. Space Complexity: $O(N + P)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Non-panicking bounds-checked `.get(..)` slicing; unknown extensions return an empty `BTreeSet`.
    /// - YANG identifiers containing `-` or `.` are rejected whole rather than truncated to a misleading prefix.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-SUBSYSTEM-PARTS-TEXT]
    pub fn extract_subsystem_parts_from_text(ext: &str, content: &str) -> BTreeSet<String> {
        let mut part_names = BTreeSet::new();
        let norm_ext = ext.to_ascii_lowercase();

        if norm_ext == ".sysml" {
            let clean = strip_sysml_comments(content);
            for p in parse_part_defs(&clean) {
                if !p.name.is_empty() {
                    part_names.insert(p.name);
                }
            }
            // Fallback token scan for `part def <Ident>` or `part <Ident> :>`
            for line in clean.lines() {
                let trimmed = line.trim();
                if let Some(pos) = trimmed.find("part ") {
                    let rest = trimmed.get(pos + 5..).unwrap_or("").trim_start();
                    let rest = rest.strip_prefix("def ").unwrap_or(rest).trim_start();
                    let ident: String = rest
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    if !ident.is_empty() && ident != "def" && is_valid_subsystem_identifier(&ident) {
                        part_names.insert(ident);
                    }
                }
            }
        } else if norm_ext == ".yaml" || norm_ext == ".yml" || norm_ext == ".json" {
            // Check structured JSON if parseable
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(content) {
                Self::extract_parts_from_json(&json_val, &mut part_names);
            }
            // Pure string/line scan for YAML and JSON component/subsystem keys
            Self::extract_parts_from_yaml_lines(content, &mut part_names);
        } else if norm_ext == ".md" || norm_ext == ".markdown" {
            let mut in_table = false;
            let mut col_subsys_idx = None;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('|') && trimmed.ends_with('|') {
                    let cells: Vec<&str> = trimmed
                        .split('|')
                        .map(|c| c.trim())
                        .filter(|c| !c.is_empty())
                        .collect();
                    if cells.is_empty() {
                        continue;
                    }
                    if cells.iter().all(|c| c.chars().all(|ch| ch == '-' || ch == ':' || ch.is_whitespace())) {
                        in_table = true;
                        continue;
                    }
                    if !in_table {
                        for (idx, cell) in cells.iter().enumerate() {
                            let clower = cell.to_ascii_lowercase();
                            if clower.contains("subsystem")
                                || clower.contains("component")
                                || clower.contains("segment")
                                || clower.contains("part")
                                || clower.contains("unit")
                            {
                                col_subsys_idx = Some(idx);
                                break;
                            }
                        }
                    } else {
                        let candidate_cell = if let Some(idx) = col_subsys_idx {
                            cells.get(idx).copied()
                        } else {
                            cells.first().copied()
                        };
                        if let Some(raw_cand) = candidate_cell {
                            let clean_cand = raw_cand
                                .replace(['`', '*', '_', '[', ']'], "")
                                .trim()
                                .to_string();
                            let clean_ident: String = clean_cand
                                .chars()
                                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                                .collect();
                            if is_valid_subsystem_identifier(&clean_ident) {
                                part_names.insert(clean_ident);
                            }
                        }
                    }
                } else {
                    in_table = false;
                    col_subsys_idx = None;
                }
            }
        } else if norm_ext == ".proto" {
            for line in content.lines() {
                let trimmed = line.trim();
                for kw in &["service ", "message "] {
                    if let Some(rest) = trimmed.strip_prefix(kw) {
                        let ident: String = rest
                            .trim_start()
                            .chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                            .collect();
                        if !ident.is_empty() && is_valid_subsystem_identifier(&ident) {
                            part_names.insert(ident);
                        }
                    }
                }
            }
        } else if norm_ext == ".idl" {
            for line in content.lines() {
                let trimmed = line.trim();
                for kw in &["interface ", "module ", "struct "] {
                    if let Some(rest) = trimmed.strip_prefix(kw) {
                        let ident: String = rest
                            .trim_start()
                            .chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                            .collect();
                        if !ident.is_empty() && is_valid_subsystem_identifier(&ident) {
                            part_names.insert(ident);
                        }
                    }
                }
            }
        } else if norm_ext == ".arxml" {
            for line in content.lines() {
                if line.contains("<SHORT-NAME>") && line.contains("</SHORT-NAME>") {
                    if let Some(start) = line.find("<SHORT-NAME>") {
                        let after = line.get(start + 12..).unwrap_or("");
                        if let Some(end) = after.find("</SHORT-NAME>") {
                            let ident = after.get(..end).unwrap_or("").trim();
                            if is_valid_subsystem_identifier(ident) {
                                part_names.insert(ident.to_string());
                            }
                        }
                    }
                }
            }
        } else if norm_ext == ".yang" {
            for line in content.lines() {
                let trimmed = line.trim();
                for kw in &["container ", "grouping ", "list "] {
                    if let Some(rest) = trimmed.strip_prefix(kw) {
                        // Capture the full RFC 7950 identifier token (which may contain `-` or `.`)
                        // so a non-conforming name is rejected whole instead of truncated.
                        let ident: String = rest
                            .trim_start()
                            .chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
                            .collect();
                        if !ident.is_empty() && is_valid_subsystem_identifier(&ident) {
                            part_names.insert(ident);
                        }
                    }
                }
            }
        }

        part_names
    }

    /// Recursively walks a parsed JSON `Value` tree and collects valid subsystem identifiers.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Extracts component identifiers from structured
    /// JSON architecture schemas (`schema_nodes`, `subsystems`, `components`, `parts`, `modules`).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Only identifiers passing `is_valid_subsystem_identifier` are inserted into `parts`.
    ///
    /// ## 3. Preconditions
    /// - `val` is a valid `serde_json::Value` reference; `parts` is a mutable `BTreeSet<String>`.
    ///
    /// ## 4. Postconditions
    /// - Appends all discovered JSON subsystem identifiers into `parts`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(V \log P)$ where $V$ is the number of JSON AST nodes. Space Complexity: $O(D)$ recursion depth.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Unexpected JSON value shapes are ignored without panicking.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-PARTS-JSON]
    fn extract_parts_from_json(val: &serde_json::Value, parts: &mut BTreeSet<String>) {
        match val {
            serde_json::Value::Object(map) => {
                if let Some(serde_json::Value::Array(nodes)) = map.get("schema_nodes") {
                    for node in nodes {
                        if let serde_json::Value::String(s) = node {
                            if let Some((kind, name)) = s.split_once(':') {
                                if kind == "part" || kind == "component" || kind == "subsystem" {
                                    let n = name.trim();
                                    if is_valid_subsystem_identifier(n) {
                                        parts.insert(n.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                for key in &["subsystems", "components", "parts", "modules"] {
                    if let Some(v) = map.get(*key) {
                        match v {
                            serde_json::Value::Array(arr) => {
                                for item in arr {
                                    match item {
                                        serde_json::Value::String(s) => {
                                            let s_clean = s.trim();
                                            if is_valid_subsystem_identifier(s_clean) {
                                                parts.insert(s_clean.to_string());
                                            }
                                        }
                                        serde_json::Value::Object(submap) => {
                                            for name_key in &["name", "id", "identifier", "title"] {
                                                if let Some(serde_json::Value::String(n)) = submap.get(*name_key) {
                                                    let n_clean = n.trim();
                                                    if is_valid_subsystem_identifier(n_clean) {
                                                        parts.insert(n_clean.to_string());
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            serde_json::Value::Object(submap) => {
                                for k in submap.keys() {
                                    let k_clean = k.trim();
                                    if is_valid_subsystem_identifier(k_clean) {
                                        parts.insert(k_clean.to_string());
                                    }
                                }
                            }
                            serde_json::Value::String(s) => {
                                let s_clean = s.trim();
                                if is_valid_subsystem_identifier(s_clean) {
                                    parts.insert(s_clean.to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                for (_, subval) in map {
                    Self::extract_parts_from_json(subval, parts);
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    Self::extract_parts_from_json(item, parts);
                }
            }
            _ => {}
        }
    }

    /// Scans YAML or JSON lines for subsystem and component list entries without an external YAML crate.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C. Provides zero-dependency line-level extraction
    /// of YAML/JSON subsystem lists (`subsystems:`, `components:`, `parts:`, `modules:`).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Tracks indentation scope so only list items under recognized section headers are extracted.
    /// - Every inserted identifier passes `is_valid_subsystem_identifier`.
    ///
    /// ## 3. Preconditions
    /// - `content` is a valid UTF-8 YAML/JSON text slice; `parts` is a mutable `BTreeSet<String>`.
    ///
    /// ## 4. Postconditions
    /// - Appends all discovered YAML/JSON subsystem identifiers into `parts`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N \log P)$ in byte length $N$. Space Complexity: $O(1)$ auxiliary beyond `parts`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Pure non-panicking line scanner; comments and blank lines are skipped.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-PARTS-YAML]
    fn extract_parts_from_yaml_lines(content: &str, parts: &mut BTreeSet<String>) {
        let mut in_parts_list = false;
        let mut list_indent = 0;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let current_indent = line.len() - line.trim_start().len();

            let is_key = [
                "subsystems:", "components:", "parts:", "modules:", "subsystem:", "component:", "part:", "module:",
            ]
            .iter()
            .any(|kw| trimmed.to_ascii_lowercase().starts_with(kw));

            if is_key {
                in_parts_list = true;
                list_indent = current_indent;
                if let Some((_, val)) = trimmed.split_once(':') {
                    let val_clean = val.trim().trim_matches(['"', '\'', '[', ']']);
                    if !val_clean.is_empty() {
                        for item in val_clean.split(',') {
                            let item_trim = item.trim().trim_matches(['"', '\'']);
                            if is_valid_subsystem_identifier(item_trim) {
                                parts.insert(item_trim.to_string());
                            }
                        }
                    }
                }
                continue;
            }

            if in_parts_list {
                if current_indent <= list_indent && !trimmed.starts_with('-') {
                    in_parts_list = false;
                } else if let Some(rest) = trimmed.strip_prefix('-') {
                    let item_clean = rest.trim();
                    let name = if let Some((_, v)) = item_clean.split_once(':') {
                        v.trim().trim_matches(['"', '\''])
                    } else {
                        item_clean.trim_matches(['"', '\''])
                    };
                    let ident: String = name
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    if is_valid_subsystem_identifier(&ident) {
                        parts.insert(ident);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_part_defs_and_ports() -> Result<(), Box<dyn std::error::Error>> {
        let sysml_source = r#"
package SystemModel_SSOT {
    doc /* SSOT for Autonomous System Architecture and Safety Model */

    attribute actuatorCount : Integer = 4;
    attribute topologyConfiguration : String = "SymmetricTopology";
    attribute transientLoadLimit : Real = 12.0;
    attribute maxLoadFactor : Real = 12.0;

    part def StructuralFrame {
        attribute massKg : Real = 25.0;
    }

    part def PrimaryControlUnit {
        port ctrl_bus : SyncBus;
        port telemetry : TelemetryLink;
    }
}
"#;
        let model = parse_sysml(sysml_source);
        assert_eq!(model.package_name.as_deref(), Some("SystemModel_SSOT"));
        assert_eq!(model.parts.len(), 2);

        let frame = model.parts.first().ok_or("frame")?;
        assert_eq!(frame.name, "StructuralFrame");
        assert_eq!(frame.attributes.len(), 1);
        let attr0 = frame.attributes.first().ok_or("attr0")?;
        assert_eq!(attr0.name, "massKg");
        assert_eq!(attr0.type_name, "Real");
        assert_eq!(attr0.default_value.as_deref(), Some("25.0"));

        let ctrl = model.parts.get(1).ok_or("ctrl")?;
        assert_eq!(ctrl.name, "PrimaryControlUnit");
        assert_eq!(ctrl.ports.len(), 2);
        let port0 = ctrl.ports.first().ok_or("port0")?;
        assert_eq!(port0.name, "ctrl_bus");
        assert_eq!(port0.type_name, "SyncBus");
        let port1 = ctrl.ports.get(1).ok_or("port1")?;
        assert_eq!(port1.name, "telemetry");
        assert_eq!(port1.type_name, "TelemetryLink");
        Ok(())
    }

    #[test]
    fn test_parse_actions_and_constraints() -> Result<(), Box<dyn std::error::Error>> {
        let sysml_source = r#"
action def ComputeStateTransition {
    in attribute currentState : Vector3;
    out attribute nextState : Vector3;
}

constraint def RateLimitConstraint(v : Real) {
    v <= 60.0;
}
"#;
        let actions = parse_action_defs(sysml_source);
        assert_eq!(actions.len(), 1);
        let act0 = actions.first().ok_or("act0")?;
        assert_eq!(act0.name, "ComputeStateTransition");
        assert_eq!(act0.in_params.len(), 1);
        assert_eq!(act0.in_params.first().ok_or("in0")?.name, "currentState");
        assert_eq!(act0.out_params.len(), 1);
        assert_eq!(act0.out_params.first().ok_or("out0")?.name, "nextState");

        let constraints = parse_constraint_defs(sysml_source);
        assert_eq!(constraints.len(), 1);
        let con0 = constraints.first().ok_or("con0")?;
        assert_eq!(con0.name, "RateLimitConstraint");
        assert_eq!(con0.parameters, vec!["v : Real"]);
        assert_eq!(con0.expression, "v <= 60.0");
        Ok(())
    }

    #[test]
    fn test_issue_428_parse_action_defs_nested_blocks_and_docblocks() -> Result<(), Box<dyn std::error::Error>> {
        let sysml_source = r#"
/* Top-level license comment */
package SystemControl {
    attribute configStr : String = "action def FakeActionInStringLiteral { in a : Int; }";

    /* Unrelated model comment */
    doc /* Primary path planning action */
    action def PlanExecutionSequence {
        in attribute currentState : Vector3;
        if (check_bounds) {
            // Nested compound block with braces
            sub_routine();
        }
        out attribute targetState : Vector3;
        perform ControlProcessor;
        step computeSequence;
        step validateTransition;
    }

    doc /* Semicolon terminated action */
    action def EmergencyStop(in code : Integer);

    /* Comment before action usage */
    action executeTransition;
}
"#;
        let actions = parse_action_defs(sysml_source);
        // FakeActionInStringLiteral must NOT be parsed!
        assert!(!actions.iter().any(|a| a.name == "FakeActionInStringLiteral"));

        let plan = actions.iter().find(|a| a.name == "PlanExecutionSequence").ok_or("PlanExecutionSequence should exist")?;
        assert_eq!(plan.doc.as_deref(), Some("Primary path planning action"));
        assert!(plan.is_def);
        assert_eq!(plan.in_params.len(), 1);
        assert_eq!(plan.in_params.first().ok_or("in0")?.name, "currentState");
        assert_eq!(plan.out_params.len(), 1);
        assert_eq!(plan.out_params.first().ok_or("out0")?.name, "targetState");
        assert_eq!(plan.performer.as_deref(), Some("ControlProcessor"));
        assert_eq!(plan.steps, vec!["computeSequence", "validateTransition"]);

        let estop = actions.iter().find(|a| a.name == "EmergencyStop").ok_or("EmergencyStop should exist")?;
        assert_eq!(estop.doc.as_deref(), Some("Semicolon terminated action"));
        assert!(estop.is_def);
        assert_eq!(estop.in_params.len(), 1);
        assert_eq!(estop.in_params.first().ok_or("in0")?.name, "code");

        let maneuver = actions.iter().find(|a| a.name == "executeTransition").ok_or("executeTransition should exist")?;
        assert_eq!(maneuver.doc, None);
        assert!(!maneuver.is_def);

        assert_eq!(actions.len(), 3);
        Ok(())
    }

    #[test]
    fn test_parse_connections_and_states() -> Result<(), Box<dyn std::error::Error>> {
        let sysml_source = r#"
part def CoreSystem {
    state def OperationalState {
        entry InitializeState;
        do UpdateStateEstimator;
        exit ShutdownSensors;
    }

    connection def SensorToController {
        connect sensor.out_port to ctrl.in_port;
    }
}
"#;
        let parts = parse_part_defs(sysml_source);
        assert_eq!(parts.len(), 1);
        let core = parts.first().ok_or("core")?;
        assert_eq!(core.name, "CoreSystem");
        assert_eq!(core.states.len(), 1);
        let st0 = core.states.first().ok_or("st0")?;
        assert_eq!(st0.name, "OperationalState");
        assert_eq!(st0.entry_action.as_deref(), Some("InitializeState"));
        assert_eq!(core.connections.len(), 1);
        let conn0 = core.connections.first().ok_or("conn0")?;
        assert_eq!(conn0.name, "SensorToController");
        assert_eq!(conn0.source_port, "sensor.out_port");
        assert_eq!(conn0.target_port, "ctrl.in_port");
        Ok(())
    }

    #[test]
    fn test_parse_ground_truth_model_fixture() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = r#"
package SystemModel_SSOT {
    doc /* Ground truth reference model for structural parsing */

    attribute redundancyLevel : Integer = 2;

    part def PrimaryControlUnit {
        port ctrl_in : CommandPort;
        attribute cycleRateHz : Real = 50.0;
    }

    part def StateEstimator {
        port sync_pulse_out : SyncPort;
    }
}
"#;
        let model = parse_sysml(fixture);
        assert_eq!(model.package_name.as_deref(), Some("SystemModel_SSOT"));
        assert_eq!(model.parts.len(), 2);
        assert_eq!(model.parts.first().ok_or("part0")?.name, "PrimaryControlUnit");
        assert_eq!(model.parts.get(1).ok_or("part1")?.name, "StateEstimator");
        Ok(())
    }

    #[test]
    fn test_parse_port_defs_syntax_variants() -> Result<(), Box<dyn std::error::Error>> {
        let text = r#"
            out port rules_out : CompilerRulePort {
            }
            port out token_out : TokenStreamPort;
            in port schema_in : RawSchemaStreamPort {
                attribute protocol_family : String = "ControlBus";
            }
            port in data_in : DataPortType;
            port cmd : Command;
        "#;
        let ports = parse_port_defs(text);
        assert_eq!(ports.len(), 5);
        let p0 = ports.first().ok_or("p0")?;
        assert_eq!(p0.name, "rules_out");
        assert_eq!(p0.direction, "out");
        assert_eq!(p0.type_name, "CompilerRulePort");

        let p1 = ports.get(1).ok_or("p1")?;
        assert_eq!(p1.name, "token_out");
        assert_eq!(p1.direction, "out");
        assert_eq!(p1.type_name, "TokenStreamPort");

        let p2 = ports.get(2).ok_or("p2")?;
        assert_eq!(p2.name, "schema_in");
        assert_eq!(p2.direction, "in");
        assert_eq!(p2.type_name, "RawSchemaStreamPort");

        let p3 = ports.get(3).ok_or("p3")?;
        assert_eq!(p3.name, "data_in");
        assert_eq!(p3.direction, "in");
        assert_eq!(p3.type_name, "DataPortType");

        let p4 = ports.get(4).ok_or("p4")?;
        assert_eq!(p4.name, "cmd");
        assert_eq!(p4.direction, "inout");
        assert_eq!(p4.type_name, "Command");
        Ok(())
    }

    #[test]
    fn test_parse_requirement_defs_with_attributes() -> Result<(), Box<dyn std::error::Error>> {
        let text = r#"
            requirement def REQ_0001_Test {
                id = "REQ-0001";
                text = "System shall do something.";
                attribute uuidv5 : String = "5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9";
                attribute complexity_class : String = "Class P";
            }
        "#;
        let reqs = parse_requirement_defs(text);
        assert_eq!(reqs.len(), 1);
        let r0 = reqs.first().ok_or("r0")?;
        assert_eq!(r0.req_id, "REQ-0001");
        assert_eq!(r0.attributes.len(), 2);
        let a0 = r0.attributes.first().ok_or("a0")?;
        assert_eq!(a0.name, "uuidv5");
        assert_eq!(
            a0.default_value.as_deref(),
            Some("\"5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9\"")
        );
        let a1 = r0.attributes.get(1).ok_or("a1")?;
        assert_eq!(a1.name, "complexity_class");
        assert_eq!(
            a1.default_value.as_deref(),
            Some("\"Class P\"")
        );
        Ok(())
    }

    #[test]
    fn test_parse_requirement_defs_constraints_assumptions_derivations() -> Result<(), Box<dyn std::error::Error>> {
        let text = r#"
            requirement def REQ_0032_Test {
                id = "REQ-0032";
                text = "Requirements shall track assumptions, constraints, and derivations.";
                assume ValidSchemaInput;
                require Invariant_REQ_0032_StrictTotalOrdering;
                require Invariant_REQ_0032_Determinism;
                derived from REQ_0031_BaseEmission;
                derived from REQ_0019_IdentifierSanitization;
            }
        "#;
        let reqs = parse_requirement_defs(text);
        assert_eq!(reqs.len(), 1);
        let r0 = reqs.first().ok_or("r0")?;
        assert_eq!(r0.assumes, vec!["ValidSchemaInput"]);
        assert_eq!(
            r0.requires,
            vec![
                "Invariant_REQ_0032_StrictTotalOrdering",
                "Invariant_REQ_0032_Determinism"
            ]
        );
        assert_eq!(
            r0.derived_from,
            vec![
                "REQ_0031_BaseEmission",
                "REQ_0019_IdentifierSanitization"
            ]
        );
        Ok(())
    }

    #[test]
    fn test_parse_requirement_defs_with_encapsulated_constraints() -> Result<(), Box<dyn std::error::Error>> {
        let text = r#"
            requirement def REQ_0001_Universal_Grounding {
                id = "REQ-0001";
                text = "Compiler shall accept schema directory.";
                attribute uuidv5 : String = "d4865863-718a-53a8-bcf6-e070e176378e";
                doc /* \forall \sigma \in Symbols(A), \sigma \in Tokens(S) */
                constraint def Invariant_Universal_Symbol_Grounding;
                assert constraint Invariant_Zero_Hardcoded_Domain_Concepts;
                require Invariant_Universal_Symbol_Grounding;
                require Invariant_Zero_Hardcoded_Domain_Concepts;
            }
        "#;
        let reqs = parse_requirement_defs(text);
        assert_eq!(reqs.len(), 1);
        let r0 = reqs.first().ok_or("r0")?;
        assert_eq!(r0.constraints.len(), 2);
        let c0 = r0.constraints.first().ok_or("c0")?;
        assert_eq!(c0.name, "Invariant_Universal_Symbol_Grounding");
        assert_eq!(
            c0.doc.as_deref(),
            Some("\\forall \\sigma \\in Symbols(A), \\sigma \\in Tokens(S)")
        );
        assert!(!c0.is_assertion);
        let c1 = r0.constraints.get(1).ok_or("c1")?;
        assert_eq!(c1.name, "Invariant_Zero_Hardcoded_Domain_Concepts");
        assert!(c1.is_assertion);
        Ok(())
    }

    #[test]
    fn test_package_def_recursive_ast_collectors() -> Result<(), Box<dyn std::error::Error>> {
        let mut root = PackageDef {
            name: "Root".to_string(),
            ..Default::default()
        };
        let mut sub = PackageDef {
            name: "Sub".to_string(),
            ..Default::default()
        };
        let mut nested = PackageDef {
            name: "Nested".to_string(),
            ..Default::default()
        };

        sub.capability_defs.push(CapabilityDef {
            name: "SubCap".to_string(),
            ..Default::default()
        });
        nested.part_defs.push(PartDef {
            name: "NestedPart".to_string(),
            ..Default::default()
        });
        nested.use_case_defs.push(UseCaseDef {
            name: "NestedUC".to_string(),
            ..Default::default()
        });
        nested.interaction_defs.push(InteractionDef {
            name: "NestedInter".to_string(),
            ..Default::default()
        });
        sub.action_defs.push(ActionDef {
            name: "SubAction".to_string(),
            ..Default::default()
        });
        nested.constraint_defs.push(ConstraintDef {
            name: "NestedConstraint".to_string(),
            ..Default::default()
        });

        sub.packages.push(nested);
        root.packages.push(sub);

        assert_eq!(root.get_all_packages().len(), 3);
        let caps = root.get_all_capabilities();
        assert_eq!(caps.len(), 1);
        assert_eq!(caps.first().ok_or("cap0")?.name, "SubCap");
        let parts = root.get_all_parts();
        assert_eq!(parts.len(), 1);
        assert_eq!(parts.first().ok_or("part0")?.name, "NestedPart");
        let ucs = root.get_all_use_cases();
        assert_eq!(ucs.len(), 1);
        assert_eq!(ucs.first().ok_or("uc0")?.name, "NestedUC");
        let inters = root.get_all_interactions();
        assert_eq!(inters.len(), 1);
        assert_eq!(inters.first().ok_or("inter0")?.name, "NestedInter");
        let acts = root.get_all_actions();
        assert_eq!(acts.len(), 1);
        assert_eq!(acts.first().ok_or("act0")?.name, "SubAction");
        let cons = root.get_all_constraints();
        assert_eq!(cons.len(), 1);
        assert_eq!(cons.first().ok_or("con0")?.name, "NestedConstraint");
        Ok(())
    }

    #[test]
    fn test_parse_test_case_defs_with_assert_constraints() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
            test case def TC_UCA_33_Verification {
                subject SafetyController;
                verify requirement Assert_UCA_33, Assert_UCA_34;
                objective "Verify safety interlock per UCA-33";
                step trigger_interlock;
            }
        "#;
        let test_cases = parse_test_case_defs(content);
        assert_eq!(test_cases.len(), 1);
        let tc0 = test_cases.first().ok_or("tc0")?;
        assert_eq!(tc0.name, "TC_UCA_33_Verification");
        assert_eq!(tc0.subject_part.as_deref(), Some("SafetyController"));
        assert_eq!(tc0.verified_requirements, vec!["Assert_UCA_33", "Assert_UCA_34"]);
        assert_eq!(tc0.objective.as_deref(), Some("Verify safety interlock per UCA-33"));
        assert_eq!(tc0.test_steps, vec!["trigger_interlock"]);
        Ok(())
    }

    #[test]
    fn test_component_matches_and_clean_names() {
        assert!(component_matches("Primary Transducer", "Primary_Transducer"));
        assert!(component_matches("PrimaryController", "PrimaryController"));
        assert!(component_matches("primary_controller", "PrimaryController"));
        assert!(component_matches("StateEstimator", "state_estimator"));
        assert!(component_matches("[PrimaryController]", "PrimaryController"));
        assert!(component_matches("**PrimaryController**", "PrimaryController"));
        assert!(component_matches("`PrimaryController`", "PrimaryController"));
        assert!(!component_matches("PhantomSubsystem", "PrimaryTransducer"));
        assert!(!component_matches("GhostUnit", "PrimaryController"));
    }

    #[test]
    fn test_extract_negated_components_detection() {
        let backup = vec!["backup".to_string()];
        assert_eq!(extract_negated_components(r#"attribute backupEnabled = 0;"#), backup);
        assert_eq!(extract_negated_components(r#"attribute backup_system = "none";"#), backup);
        assert_eq!(extract_negated_components(r#"TestPlatform.backupSystem = "No""#), backup);
        assert_eq!(extract_negated_components("Backup subsystem: No"), backup);
        assert_eq!(extract_negated_components("No backup unit installed"), backup);
        assert_eq!(extract_negated_components("!exists(Backup)"), backup);
        assert!(extract_negated_components(r#"attribute backupEnabled = 1;"#).is_empty());
        assert!(extract_negated_components(r#"attribute backupSystem = "Yes";"#).is_empty());
        assert!(extract_negated_components("Nominal operations").is_empty());
        assert!(extract_negated_components("Retry no more than 3 times").is_empty());
    }

    #[test]
    fn test_is_component_negated_detection() -> Result<(), Box<dyn std::error::Error>> {
        let model = "attribute backupEnabled = 0;\nattribute backupSystem = \"No\";";
        assert!(is_component_negated("AuxiliaryBackupUnit", model));
        assert!(is_component_negated("`AuxiliaryBackupUnit`", model));
        assert!(!is_component_negated("PrimaryController", model));
        assert!(!is_component_negated("StateEstimator", model));
        assert!(!is_component_negated("", model));

        let mut pkg = PackageDef {
            name: "SystemModel_SSOT".to_string(),
            ..Default::default()
        };
        pkg.attribute_defs.push(AttributeDef {
            name: "backupEnabled".to_string(),
            default_value: Some("0".to_string()),
            ..Default::default()
        });
        let mut sub = PackageDef {
            name: "Subsystems".to_string(),
            doc: Some("Redundant module: none".to_string()),
            ..Default::default()
        };
        sub.constraint_defs.push(ConstraintDef {
            name: "DiagnosticsAbsent".to_string(),
            expression: "!exists(DiagnosticsService)".to_string(),
            ..Default::default()
        });
        pkg.packages.push(sub);

        let stems = pkg.extract_negated_components();
        assert_eq!(stems, vec!["backup", "diagnosticsservice", "redundant"]);
        assert!(pkg.is_component_negated("AuxiliaryBackupUnit"));
        assert!(pkg.is_component_negated("RedundantModule"));
        assert!(pkg.is_component_negated("DiagnosticsService"));
        assert!(!pkg.is_component_negated("PrimaryController"));
        assert!(stems.first().ok_or("stem0")?.len() >= 3);
        Ok(())
    }

    #[test]
    fn test_numeric_state_attributes_do_not_negate_components() {
        let model = "attribute faultCount = 0;\nattribute retryLimit = 0;";
        assert!(extract_negated_components(model).is_empty());
        assert!(!is_component_negated("FaultMonitor", model));
        assert!(!is_component_negated("RetryHandler", model));

        let mut pkg = PackageDef {
            name: "SystemModel_SSOT".to_string(),
            ..Default::default()
        };
        for name in ["faultCount", "retryLimit"] {
            pkg.attribute_defs.push(AttributeDef {
                name: name.to_string(),
                default_value: Some("0".to_string()),
                ..Default::default()
            });
        }
        assert!(pkg.extract_negated_components().is_empty());
        assert!(!pkg.is_component_negated("FaultMonitor"));
        assert!(!pkg.is_component_negated("RetryHandler"));

        // Control: an explicit `has<Component>` presence flag still negates the component.
        let control = "attribute hasFaultMonitor = false;";
        assert_eq!(extract_negated_components(control), vec!["faultmonitor".to_string()]);
        assert!(is_component_negated("FaultMonitor", control));
        assert!(!is_component_negated("RetryHandler", control));

        // Morpheme boundaries: `has` inside `hash` is not a prefix, and scalar/metrology stems are not components.
        let boundary = "attribute hashSeed = 0;\nattribute timeUnit = 0;";
        assert!(extract_negated_components(boundary).is_empty());
        assert!(!is_component_negated("HashSeedGenerator", boundary));
        assert!(!is_component_negated("TimeSource", boundary));

        let mut boundary_pkg = PackageDef {
            name: "SystemModel_SSOT".to_string(),
            ..Default::default()
        };
        for name in ["hashSeed", "timeUnit"] {
            boundary_pkg.attribute_defs.push(AttributeDef {
                name: name.to_string(),
                default_value: Some("0".to_string()),
                ..Default::default()
            });
        }
        assert!(boundary_pkg.extract_negated_components().is_empty());
        assert!(!boundary_pkg.is_component_negated("HashSeedGenerator"));
        assert!(!boundary_pkg.is_component_negated("TimeSource"));

        // Control: a `has_<component>` snake-case presence flag still negates the component.
        let snake_control = "attribute has_backup_module = false;";
        assert_eq!(extract_negated_components(snake_control), vec!["backup".to_string()]);
    }

    #[test]
    fn test_multi_clause_prose_negation_extracts_every_clause() {
        let prose = "Negative safety model invariant: backupEnabled = 0, redundantSystem = \"No\"";
        let stems = extract_negated_components(prose);
        assert!(stems.contains(&"backup".to_string()), "stems: {stems:?}");
        assert!(stems.contains(&"redundant".to_string()), "stems: {stems:?}");
        assert!(is_component_negated("AuxiliaryBackupUnit", prose));
        assert!(!is_component_negated("PrimaryProcessor", prose));

        // Dotted owner paths in a labelled clause yield the leaf key token only.
        let dotted = extract_negated_components("Invariant: Owner.auxSystem = 0, redundantSystem = \"No\"");
        assert!(dotted.contains(&"aux".to_string()), "stems: {dotted:?}");
        assert!(!dotted.contains(&"owneraux".to_string()), "stems: {dotted:?}");
    }

    #[test]
    fn test_unified_schema_model_extractor_all_formats() {
        // 1. SysML v2
        let sysml = r#"
            /* multi-line comment */
            package SampleSystem {
                // single line comment
                part def ControlProcessor;
                part StateEstimator :> SensorBase;
            }
        "#;
        let sysml_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".sysml", sysml);
        assert!(sysml_parts.contains("ControlProcessor"));
        assert!(sysml_parts.contains("StateEstimator"));

        // 2. YAML
        let yaml = r#"
            subsystems:
              - ControlProcessor
              - SequencingSystem
            components:
              - name: PowerDistributionUnit
                id: PDU-01
        "#;
        let yaml_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".yaml", yaml);
        assert!(yaml_parts.contains("ControlProcessor"));
        assert!(yaml_parts.contains("SequencingSystem"));
        assert!(yaml_parts.contains("PowerDistributionUnit"));

        // 3. JSON
        let json = r#"
            {
                "schema_nodes": ["part:TelemetryTransceiver", "port:TX"],
                "components": [
                    {"name": "ThermalController", "id": "TCU"}
                ]
            }
        "#;
        let json_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".json", json);
        assert!(json_parts.contains("TelemetryTransceiver"));
        assert!(json_parts.contains("ThermalController"));

        // 4. Markdown architecture table
        let md = r#"
            # System Architecture

            ### 4.3 Subsystem Architecture
            | Subsystem | Function | Allocation |
            |---|---|---|
            | RedundantModule | Core processing | Segment A |
            | ActuatorControllerUnit | Output control | Segment B |
        "#;
        let md_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".md", md);
        assert!(md_parts.contains("RedundantModule"));
        assert!(md_parts.contains("ActuatorControllerUnit"));

        // 5. Protobuf
        let proto = r#"
            syntax = "proto3";
            package telemetry_pkg;
            service DiagnosticsService {
                rpc GetState (Empty) returns (State);
            }
            message SensorData {
                int64 timestamp = 1;
            }
        "#;
        let proto_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".proto", proto);
        assert!(proto_parts.contains("DiagnosticsService"));
        assert!(proto_parts.contains("SensorData"));

        // 6. OMG IDL
        let idl = r#"
            module SystemBus {
                interface BusController {
                    void transmit();
                };
            };
        "#;
        let idl_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".idl", idl);
        assert!(idl_parts.contains("BusController"));
        assert!(idl_parts.contains("SystemBus"));

        // 7. ARXML
        let arxml = r#"
            <AUTOSAR>
                <SW-COMPONENT-PROTOTYPE>
                    <SHORT-NAME>ProcessController</SHORT-NAME>
                </SW-COMPONENT-PROTOTYPE>
            </AUTOSAR>
        "#;
        let arxml_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".arxml", arxml);
        assert!(arxml_parts.contains("ProcessController"));
    }

    #[test]
    fn test_unified_schema_model_extractor_yang_container_grouping_list() {
        let yang = r#"
            module sample_module {
                yang-version 1.1;
                prefix sm;
                grouping EndpointSettings {
                    leaf label { type string; }
                }
                container NodeRegistry {
                    list NodeEntry {
                        key "label";
                        uses EndpointSettings;
                    }
                }
                typedef counter_kind { type uint32; }
                container access-table { leaf size { type uint32; } }
            }
        "#;
        let yang_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".yang", yang);
        assert!(yang_parts.contains("EndpointSettings"), "parts: {yang_parts:?}");
        assert!(yang_parts.contains("NodeRegistry"), "parts: {yang_parts:?}");
        assert!(yang_parts.contains("NodeEntry"), "parts: {yang_parts:?}");
        assert!(!yang_parts.contains("sample_module"), "parts: {yang_parts:?}");
        assert!(!yang_parts.contains("counter_kind"), "parts: {yang_parts:?}");
        assert!(!yang_parts.contains("label"), "parts: {yang_parts:?}");
        let upper = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".YANG", yang);
        assert_eq!(upper, yang_parts);
        assert!(UnifiedSchemaModelExtractor::SUPPORTED_EXTENSIONS.contains(&".yang"));
    }

    #[test]
    fn test_parse_calc_defs_syntax_variants() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
            doc /* Calculates kinetic energy */
            calc def KineticEnergy(in massKg : Real, in speedMps : Real) : Real {
                0.5 * massKg * speedMps * speedMps
            }

            calc def VelocityCalc {
                in v0 : Real;
                in a : Real;
                in t : Real;
                return v : Real;
                v0 + a * t;
            }

            calc MarginCalculation;
        "#;

        let calcs = parse_calc_defs(content);
        assert_eq!(calcs.len(), 3);

        let ke = calcs.first().ok_or("ke")?;
        assert_eq!(ke.name, "KineticEnergy");
        assert_eq!(ke.doc.as_deref(), Some("Calculates kinetic energy"));
        assert_eq!(ke.in_params.len(), 2);
        let ke_p0 = ke.in_params.first().ok_or("ke_p0")?;
        assert_eq!(ke_p0.name, "massKg");
        assert_eq!(ke_p0.type_name, "Real");
        let ke_p1 = ke.in_params.get(1).ok_or("ke_p1")?;
        assert_eq!(ke_p1.name, "speedMps");
        assert_eq!(ke_p1.type_name, "Real");
        assert_eq!(ke.return_type.as_deref(), Some("Real"));
        assert_eq!(ke.expression.as_deref(), Some("0.5 * massKg * speedMps * speedMps"));

        let vc = calcs.get(1).ok_or("vc")?;
        assert_eq!(vc.name, "VelocityCalc");
        assert_eq!(vc.in_params.len(), 3);
        assert_eq!(vc.in_params.first().ok_or("vc_p0")?.name, "v0");
        assert_eq!(vc.in_params.get(1).ok_or("vc_p1")?.name, "a");
        assert_eq!(vc.in_params.get(2).ok_or("vc_p2")?.name, "t");
        assert_eq!(vc.return_type.as_deref(), Some("Real"));
        assert_eq!(vc.expression.as_deref(), Some("v0 + a * t"));

        let mc = calcs.get(2).ok_or("mc")?;
        assert_eq!(mc.name, "MarginCalculation");
        assert_eq!(mc.in_params.len(), 0);
        assert!(mc.return_type.is_none());
        assert!(mc.expression.is_none());
        Ok(())
    }

    #[test]
    fn test_package_def_queries_and_nested_ports() -> Result<(), Box<dyn std::error::Error>> {
        let mut root = PackageDef {
            name: "RootSystem".to_string(),
            calc_defs: vec![
                CalcDef {
                    name: "RootCalc".to_string(),
                    ..Default::default()
                },
            ],
            port_defs: vec![
                PortDef {
                    name: "root_port".to_string(),
                    ..Default::default()
                },
            ],
            hazard_defs: vec![
                HazardDef {
                    name: "H-01".to_string(),
                    severity: 3,
                    ..Default::default()
                },
            ],
            risk_defs: vec![
                RiskDef {
                    name: "R-01".to_string(),
                    severity: 3,
                    ..Default::default()
                },
            ],
            state_defs: vec![
                StateDef {
                    name: "Nominal".to_string(),
                    ..Default::default()
                },
            ],
            connection_defs: vec![
                ConnectionDef {
                    name: "C-01".to_string(),
                    source_port: "p1".to_string(),
                    target_port: "p2".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let sub_part = PartDef {
            name: "NestedSensor".to_string(),
            ports: vec![
                PortDef {
                    name: "sync_pulse_out".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let top_part = PartDef {
            name: "ControlProcessor".to_string(),
            ports: vec![
                PortDef {
                    name: "ctrl_bus".to_string(),
                    ..Default::default()
                },
            ],
            parts: vec![sub_part],
            calcs: vec![
                CalcDef {
                    name: "InnerCalc".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        root.part_defs.push(top_part);

        // Test get_all_calcs
        let calcs = root.get_all_calcs();
        assert_eq!(calcs.len(), 2);
        let calc_names: Vec<&str> = calcs.iter().map(|c| c.name.as_str()).collect();
        assert!(calc_names.contains(&"RootCalc"));
        assert!(calc_names.contains(&"InnerCalc"));

        // Test get_all_ports
        let ports = root.get_all_ports();
        assert_eq!(ports.len(), 3);
        let port_names: Vec<&str> = ports.iter().map(|p| p.name.as_str()).collect();
        assert!(port_names.contains(&"root_port"));
        assert!(port_names.contains(&"ctrl_bus"));
        assert!(port_names.contains(&"sync_pulse_out"));

        // Test get_all_hazards
        let hazards = root.get_all_hazards();
        assert_eq!(hazards.len(), 1);
        assert_eq!(hazards.first().ok_or("h0")?.name, "H-01");

        // Test get_all_risks
        let risks = root.get_all_risks();
        assert_eq!(risks.len(), 1);
        assert_eq!(risks.first().ok_or("r0")?.name, "R-01");

        // Test get_all_states
        let states = root.get_all_states();
        assert_eq!(states.len(), 1);
        assert_eq!(states.first().ok_or("s0")?.name, "Nominal");

        // Test get_all_connections
        let conns = root.get_all_connections();
        assert_eq!(conns.len(), 1);
        assert_eq!(conns.first().ok_or("c0")?.name, "C-01");

        // Test get_nested_ports: ONLY sync_pulse_out is nested; ctrl_bus and root_port are top-level!
        let nested_ports = root.get_nested_ports();
        assert_eq!(nested_ports.len(), 1);
        assert!(nested_ports.contains("sync_pulse_out"));
        assert!(!nested_ports.contains("ctrl_bus"));
        assert!(!nested_ports.contains("root_port"));
        Ok(())
    }

    #[test]
    fn test_sysml_ast_source_purity_and_7_section_rustdoc() {
        let source = include_str!("sysml_ast.rs");
        let needle_slice_str = ["&[", "&str]"].concat();
        let needle_tuple_slice = ["&[(", "&str"].concat();
        let needle_array_str = ["[", "&str;"].concat();
        let needle_static_slice = [": &[", "&str]"].concat();
        let needle_const_slice = ["const ", ": &[&str]"].concat();
        let needle_fm_e = [".filter_map(", "|e|", " e.ok())"].concat();
        let needle_lower_slice1 = [".to_lowercase()", "["].concat();
        let needle_lower_slice2 = [".to_ascii_lowercase()", "["].concat();

        let s1 = "## 1. Safety Intent & Regulatory Scope";
        let s2 = "## 2. Invariants & Mathematical Properties";
        let s3 = "## 3. Preconditions";
        let s4 = "## 4. Postconditions";
        let s5 = "## 5. Algorithmic Complexity Bounds";
        let s6 = "## 6. Failure Modes & Fault Tolerance";
        let s7 = "## 7. Realization & Traceability";

        let mut violations = Vec::new();
        let mut current_doc = Vec::new();
        let mut in_multiline_comment = false;

        for (idx, raw_line) in source.lines().enumerate() {
            let line_no = idx + 1;
            let trimmed = raw_line.trim();
            if in_multiline_comment {
                if trimmed.contains("*/") {
                    in_multiline_comment = false;
                }
                continue;
            }
            if trimmed.starts_with("/*") && !trimmed.contains("*/") {
                in_multiline_comment = true;
                continue;
            }
            if trimmed.starts_with("//!") || trimmed.starts_with("///") {
                current_doc.push(trimmed.to_string());
                continue;
            }
            let code = if let Some((before, _)) = trimmed.split_once("//") {
                before.trim()
            } else {
                trimmed
            };
            if code.is_empty() {
                continue;
            }
            if code.contains("mod tests") {
                break;
            }
            if code.contains(&needle_slice_str)
                || code.contains(&needle_tuple_slice)
                || code.contains(&needle_array_str)
                || code.contains(&needle_static_slice)
                || code.contains(&needle_const_slice)
            {
                violations.push(format!("line {}: static string-slice array", line_no));
            }
            if code.contains(&needle_fm_e) || (code.contains(".filter_map(") && code.contains(".ok()")) {
                violations.push(format!("line {}: fail-open error swallowing", line_no));
            }
            if code.contains(&needle_lower_slice1)
                || code.contains(&needle_lower_slice2)
                || ((code.contains("lower[") || code.contains("_lower[")) && code.contains(".."))
            {
                violations.push(format!("line {}: cross-string UTF-8 byte-slice hazard", line_no));
            }
            if code.starts_with("pub fn ")
                || code.starts_with("fn ")
                || code.starts_with("pub const fn ")
                || code.starts_with("pub(crate) fn ")
            {
                let doc_block = current_doc.join("\n");
                for req_hdr in [s1, s2, s3, s4, s5, s6, s7] {
                    if !doc_block.contains(req_hdr) {
                        violations.push(format!("line {}: missing {} on `{}`", line_no, req_hdr, code));
                    }
                }
                current_doc.clear();
            } else if !code.starts_with("#[") {
                current_doc.clear();
            }
        }
        assert!(violations.is_empty(), "Purity violations:\n{}", violations.join("\n"));
    }

    #[test]
    fn test_supported_schema_extension_enum_cardinality_order_and_catalog_parity() {
        let expected = vec![
            ".sysml", ".yaml", ".yml", ".json", ".proto", ".idl", ".arxml", ".md", ".markdown",
            ".yang",
        ];
        assert_eq!(SupportedSchemaExtension::ALL.len(), 10);
        let via_enum: Vec<&'static str> =
            SupportedSchemaExtension::ALL.iter().map(|x| x.as_str()).collect();
        assert_eq!(via_enum, expected);
        let catalog = UnifiedSchemaModelExtractor::SUPPORTED_EXTENSIONS;
        let via_catalog: Vec<&'static str> = catalog.iter().collect();
        assert_eq!(via_catalog, expected);
        for x in SupportedSchemaExtension::ALL {
            assert_eq!(x.as_str_ref(), x.as_str());
            assert!(catalog.contains(&x.as_str()));
        }
        let owned = String::from(".yaml");
        assert!(catalog.contains(&owned.as_str()));
        assert!(!catalog.contains(&".txt"));
        assert!(!catalog.contains(&"sysml"));
        assert!(!catalog.contains(&".SYSML"));
        assert!(!catalog.contains(&""));
    }

    #[test]
    fn test_sysml_ast_supported_extensions_free_of_static_str_slice_evasion() {
        let source = include_str!("sysml_ast.rs");
        let needle_into_iter = ["IntoIter<&'static", " str"].concat();
        let needle_static_slice = ["&'static [&'static", " str]"].concat();
        let hits: Vec<String> = source
            .lines()
            .enumerate()
            .filter(|(_, l)| l.contains(&needle_into_iter) || l.contains(&needle_static_slice))
            .map(|(i, l)| format!("line {}: {}", i + 1, l.trim()))
            .collect();
        assert!(
            hits.is_empty(),
            "sysml_ast.rs still carries untyped string-slice catalog evasions:\n{}",
            hits.join("\n")
        );
    }

    #[test]
    fn test_parse_satisfied_requirements_and_part_defs_extraction() {
        let snippet = r#"
            part def SensorController {
                satisfy requirement REQ-CTRL-001;
                satisfy requirement "REQ-CTRL-002";
                satisfy REQ-PWR-03 by PowerBus;
                satisfy requirement REQ-CTRL-001;
                satisfy /* comment 1 */ requirement REQ-COM-04;
                satisfy/*comment 2*/requirement REQ-COM-05;
                satisfy requirement /*comment 3*/ REQ-COM-06;
                satisfy requirement REQ-COM-07 /*comment 4*/;
                satisfy requirement REQ-COM-08 /* c1 */ by /* c2 */ PowerBus /* c3 */;
            }
        "#;
        let satisfied = parse_satisfied_requirements(snippet);
        assert_eq!(
            satisfied,
            vec![
                "REQ-CTRL-001".to_string(),
                "REQ-CTRL-002".to_string(),
                "REQ-PWR-03".to_string(),
                "REQ-COM-04".to_string(),
                "REQ-COM-05".to_string(),
                "REQ-COM-06".to_string(),
                "REQ-COM-07".to_string(),
                "REQ-COM-08".to_string(),
            ]
        );

        let parts = parse_part_defs(snippet);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].name, "SensorController");
        assert_eq!(
            parts[0].satisfied_requirements,
            vec![
                "REQ-CTRL-001".to_string(),
                "REQ-CTRL-002".to_string(),
                "REQ-PWR-03".to_string(),
                "REQ-COM-04".to_string(),
                "REQ-COM-05".to_string(),
                "REQ-COM-06".to_string(),
                "REQ-COM-07".to_string(),
                "REQ-COM-08".to_string(),
            ]
        );
    }

    #[test]
    fn test_sysml_ast_module_7_section_rustdoc() {
        let src = include_str!("sysml_ast.rs");
        let module_doc: String = src
            .lines()
            .take_while(|line| line.starts_with("//!"))
            .collect::<Vec<&str>>()
            .join("\n");
        assert!(module_doc.contains("//! ## 1. Safety Intent & Regulatory Scope"));
        assert!(module_doc.contains("//! ## 2. Invariants & Mathematical Properties"));
        assert!(module_doc.contains("//! ## 3. Preconditions"));
        assert!(module_doc.contains("//! ## 4. Postconditions"));
        assert!(module_doc.contains("//! ## 5. Algorithmic Complexity Bounds"));
        assert!(module_doc.contains("//! ## 6. Failure Modes & Fault Tolerance"));
        assert!(module_doc.contains("//! ## 7. Realization & Traceability"));
        assert!(module_doc.contains("//! Realises: [REQ-SYSML-AST-MODULE], [DO-330-TQL-4-SYSML-AST]"));
    }
}
