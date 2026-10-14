//! AST structures for SysML v2 entities, requirements, and system architecture.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn default_true() -> bool {
    true
}

fn default_one() -> i32 {
    1
}

/// Represents an attribute definition or instance in SysML v2.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AttributeDef {
    pub name: String,
    #[serde(default)]
    pub type_name: String,
    pub default_value: Option<String>,
    pub doc: Option<String>,
}

/// Represents an item flow or discrete data flow definition in SysML v2.
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

fn default_direction_out() -> String {
    "out".to_string()
}

fn default_item_type() -> String {
    "Item".to_string()
}

/// Represents a port definition or interface point in SysML v2.
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

fn default_direction_inout() -> String {
    "inout".to_string()
}

fn default_port_category() -> String {
    "DataPort".to_string()
}

/// Represents an action definition in SysML v2.
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CapabilityDef {
    pub name: String,
    pub description: Option<String>,
    pub subsystem: Option<String>,
    pub package_ref: Option<String>,
    pub doc: Option<String>,
}

/// Represents an interaction sequence in SysML v2.
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ItemDef {
    pub name: String,
    pub doc: Option<String>,
    #[serde(default)]
    pub attributes: Vec<AttributeDef>,
}

/// Represents a safety hazard definition in SysML v2 / STPA models.
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
    pub fn get_all_packages(&self) -> Vec<&PackageDef> {
        let mut result = vec![self];
        for subpkg in &self.packages {
            result.extend(subpkg.get_all_packages());
        }
        result
    }

    /// Returns all subsystem capability definitions declared across self and all nested packages.
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

    /// Checks whether this package or any nested subpackages/parts declare attributes or constraints negating recovery systems.
    pub fn has_negative_recovery_constraint(&self) -> bool {
        let recovery_keys = [
            "recoverysystem",
            "recovery_system",
            "chuteenabled",
            "chute_enabled",
            "recoveryenabled",
            "recovery_enabled",
            "hasrecovery",
            "has_recovery",
            "hasparachute",
            "has_parachute",
        ];
        let negative_vals = ["0", "false", "\"no\"", "\"none\"", "no", "none"];

        for attr in self.get_all_attributes() {
            let name_clean = attr.name.to_lowercase().replace('_', "");
            if recovery_keys.iter().any(|k| name_clean == k.replace('_', "")) {
                if let Some(val) = &attr.default_value {
                    let v = val.trim().to_lowercase();
                    if negative_vals.iter().any(|neg| v == *neg || v.starts_with(neg)) {
                        return true;
                    }
                }
            }
        }

        for con in self.get_all_constraints() {
            if has_negative_recovery_constraint(&con.expression) {
                return true;
            }
        }

        for pkg in self.get_all_packages() {
            if let Some(doc) = &pkg.doc {
                if has_negative_recovery_constraint(doc) {
                    return true;
                }
            }
        }

        false
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

fn collect_part_and_subdefs(part: &PartDef, result: &mut Vec<PartDef>) {
    result.push(part.clone());
    for sub in &part.parts {
        if sub.is_def {
            collect_part_and_subdefs(sub, result);
        }
    }
}

fn collect_part_capabilities(part: &PartDef, result: &mut Vec<CapabilityDef>) {
    result.extend(part.capabilities.clone());
    for sub in &part.parts {
        collect_part_capabilities(sub, result);
    }
}

fn collect_part_use_cases(part: &PartDef, result: &mut Vec<UseCaseDef>) {
    result.extend(part.use_cases.clone());
    for sub in &part.parts {
        collect_part_use_cases(sub, result);
    }
}

fn collect_part_interactions(part: &PartDef, result: &mut Vec<InteractionDef>) {
    result.extend(part.interactions.clone());
    for sub in &part.parts {
        collect_part_interactions(sub, result);
    }
}

fn collect_part_actions(part: &PartDef, result: &mut Vec<ActionDef>) {
    result.extend(part.actions.clone());
    for sub in &part.parts {
        collect_part_actions(sub, result);
    }
}

fn collect_part_constraints(part: &PartDef, result: &mut Vec<ConstraintDef>) {
    result.extend(part.constraints.clone());
    for sub in &part.parts {
        collect_part_constraints(sub, result);
    }
}

fn collect_part_attributes(part: &PartDef, result: &mut Vec<AttributeDef>) {
    result.extend(part.attributes.clone());
    for sub in &part.parts {
        collect_part_attributes(sub, result);
    }
}

fn collect_part_calcs(part: &PartDef, result: &mut Vec<CalcDef>) {
    result.extend(part.calcs.clone());
    for sub in &part.parts {
        collect_part_calcs(sub, result);
    }
}

fn collect_part_ports(part: &PartDef, result: &mut Vec<PortDef>) {
    result.extend(part.ports.clone());
    for sub in &part.parts {
        collect_part_ports(sub, result);
    }
}

fn collect_part_hazards(part: &PartDef, result: &mut Vec<HazardDef>) {
    result.extend(part.hazards.clone());
    for sub in &part.parts {
        collect_part_hazards(sub, result);
    }
}

fn collect_part_risks(part: &PartDef, result: &mut Vec<RiskDef>) {
    result.extend(part.risks.clone());
    for sub in &part.parts {
        collect_part_risks(sub, result);
    }
}

fn collect_part_states(part: &PartDef, result: &mut Vec<StateDef>) {
    result.extend(part.states.clone());
    for sub in &part.parts {
        collect_part_states(sub, result);
    }
}

fn collect_part_connections(part: &PartDef, result: &mut Vec<ConnectionDef>) {
    result.extend(part.connections.clone());
    for sub in &part.parts {
        collect_part_connections(sub, result);
    }
}

fn collect_nested_ports_recursive(parts: &[PartDef], nested: &mut BTreeSet<String>) {
    for p in parts {
        for port in &p.ports {
            nested.insert(port.name.clone());
        }
        collect_nested_ports_recursive(&p.parts, nested);
    }
}

/// Legacy and high-level SysML model container for backwards compatibility.
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

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn is_word_boundary_before(text: &str, pos: usize) -> bool {
    if pos == 0 {
        true
    } else {
        !is_ident_char(text.as_bytes()[pos - 1])
    }
}

fn is_word_boundary_after(text: &str, pos: usize) -> bool {
    if pos >= text.len() {
        true
    } else {
        !is_ident_char(text.as_bytes()[pos])
    }
}

fn skip_ws(text: &str) -> usize {
    text.bytes().take_while(|b| b.is_ascii_whitespace()).count()
}

fn extract_ident(text: &str) -> (&str, usize) {
    let len = text.bytes().take_while(|&b| is_ident_char(b)).count();
    (&text[..len], len)
}

fn extract_doc_comment_before(text: &str, keyword_pos: usize) -> Option<String> {
    let before = text[..keyword_pos].trim_end();
    if before.ends_with("*/") {
        if let Some(open) = before.rfind("/*") {
            let doc_prefix = before[..open].trim_end();
            if doc_prefix.ends_with("doc") {
                let doc_start = doc_prefix.len() - 3;
                if doc_start == 0 || !is_ident_char(doc_prefix.as_bytes()[doc_start - 1]) {
                    return Some(before[open + 2..before.len() - 2].trim().to_string());
                }
            }
        }
    }
    None
}

fn extract_quoted_string(s: &str) -> Option<&str> {
    if s.starts_with('"') {
        if let Some(end) = s[1..].find('"') {
            return Some(&s[1..1 + end]);
        }
    }
    None
}

/// Parse attributes from SysML v2 source text.
pub fn parse_attribute_defs(content: &str) -> Vec<AttributeDef> {
    let mut attrs = Vec::new();
    let mut idx = 0;
    while let Some(pos) = content[idx..].find("attribute") {
        let kw_pos = idx + pos;
        if !is_word_boundary_before(content, kw_pos) || !is_word_boundary_after(content, kw_pos + 9) {
            idx = kw_pos + 9;
            continue;
        }
        let doc = extract_doc_comment_before(content, kw_pos);
        let mut curr = kw_pos + 9;
        curr += skip_ws(&content[curr..]);
        if content[curr..].starts_with("def") && is_word_boundary_after(&content[curr..], 3) {
            curr += 3;
            curr += skip_ws(&content[curr..]);
        }
        let (name, name_len) = extract_ident(&content[curr..]);
        if name.is_empty() {
            idx = kw_pos + 9;
            continue;
        }
        curr += name_len;
        curr += skip_ws(&content[curr..]);
        if !content[curr..].starts_with(':') {
            idx = kw_pos + 9;
            continue;
        }
        curr += 1;
        curr += skip_ws(&content[curr..]);
        let (type_name, type_len) = extract_ident(&content[curr..]);
        if type_name.is_empty() {
            idx = kw_pos + 9;
            continue;
        }
        curr += type_len;
        curr += skip_ws(&content[curr..]);
        if content[curr..].starts_with('=') {
            curr += 1;
            curr += skip_ws(&content[curr..]);
            if let Some(semi) = content[curr..].find(';') {
                let val = content[curr..curr + semi].trim().to_string();
                attrs.push(AttributeDef {
                    name: name.to_string(),
                    type_name: type_name.to_string(),
                    default_value: Some(val),
                    doc,
                });
                idx = curr + semi + 1;
                continue;
            } else {
                idx = kw_pos + 9;
                continue;
            }
        } else if content[curr..].starts_with(';') {
            attrs.push(AttributeDef {
                name: name.to_string(),
                type_name: type_name.to_string(),
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

/// Parse port definitions from SysML v2 source text.
pub fn parse_port_defs(content: &str) -> Vec<PortDef> {
    let mut ports = Vec::new();
    let mut idx = 0;
    while let Some(pos) = content[idx..].find("port") {
        let port_pos = idx + pos;
        if !is_word_boundary_before(content, port_pos) || !is_word_boundary_after(content, port_pos + 4) {
            idx = port_pos + 4;
            continue;
        }
        let before_port = content[..port_pos].trim_end();
        let (dir1, doc_end) = if before_port.ends_with("inout") && is_word_boundary_before(before_port, before_port.len() - 5) {
            (Some("inout"), before_port.len() - 5)
        } else if before_port.ends_with("out") && is_word_boundary_before(before_port, before_port.len() - 3) {
            (Some("out"), before_port.len() - 3)
        } else if before_port.ends_with("in") && is_word_boundary_before(before_port, before_port.len() - 2) {
            (Some("in"), before_port.len() - 2)
        } else {
            (None, port_pos)
        };
        let doc = extract_doc_comment_before(content, doc_end);

        let mut curr = port_pos + 4;
        curr += skip_ws(&content[curr..]);
        if content[curr..].starts_with("def") && is_word_boundary_after(&content[curr..], 3) {
            curr += 3;
            curr += skip_ws(&content[curr..]);
        }
        let mut dir2 = None;
        if content[curr..].starts_with("inout") && is_word_boundary_after(&content[curr..], 5) {
            dir2 = Some("inout");
            curr += 5;
            curr += skip_ws(&content[curr..]);
        } else if content[curr..].starts_with("out") && is_word_boundary_after(&content[curr..], 3) {
            dir2 = Some("out");
            curr += 3;
            curr += skip_ws(&content[curr..]);
        } else if content[curr..].starts_with("in") && is_word_boundary_after(&content[curr..], 2) {
            dir2 = Some("in");
            curr += 2;
            curr += skip_ws(&content[curr..]);
        }
        if content[curr..].starts_with("def") && is_word_boundary_after(&content[curr..], 3) {
            curr += 3;
            curr += skip_ws(&content[curr..]);
        }
        let mut conj1 = false;
        if content[curr..].starts_with('~') {
            conj1 = true;
            curr += 1;
            curr += skip_ws(&content[curr..]);
        }
        let (name, name_len) = extract_ident(&content[curr..]);
        if name.is_empty() {
            idx = port_pos + 4;
            continue;
        }
        curr += name_len;
        curr += skip_ws(&content[curr..]);

        let mut type_name = None;
        let mut conj2 = false;
        if content[curr..].starts_with(':') {
            curr += 1;
            curr += skip_ws(&content[curr..]);
            if content[curr..].starts_with('~') {
                conj2 = true;
                curr += 1;
                curr += skip_ws(&content[curr..]);
            }
            let (t, t_len) = extract_ident(&content[curr..]);
            if !t.is_empty() {
                type_name = Some(t.to_string());
                curr += t_len;
                curr += skip_ws(&content[curr..]);
            }
        }

        if content[curr..].starts_with('{') {
            if let Some(close_brace) = find_matching_brace(content, curr) {
                idx = close_brace + 1;
            } else if let Some(close_brace) = content[curr..].find('}') {
                idx = curr + close_brace + 1;
            } else {
                idx = port_pos + 4;
                continue;
            }
        } else if content[curr..].starts_with(';') {
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
            ..Default::default()
        });
    }
    ports
}

fn parse_action_param(s: &str) -> Option<(bool, AttributeDef)> {
    let trimmed = s.trim();
    let (is_in, after_dir) = if trimmed.starts_with("in") && is_word_boundary_after(trimmed, 2) {
        (true, trimmed[2..].trim_start())
    } else if trimmed.starts_with("out") && is_word_boundary_after(trimmed, 3) {
        (false, trimmed[3..].trim_start())
    } else {
        return None;
    };
    let after_kw = if after_dir.starts_with("attribute") && is_word_boundary_after(after_dir, 9) {
        after_dir[9..].trim_start()
    } else if after_dir.starts_with("item") && is_word_boundary_after(after_dir, 4) {
        after_dir[4..].trim_start()
    } else {
        after_dir
    };
    let (name, name_len) = extract_ident(after_kw);
    if name.is_empty() {
        return None;
    }
    let after_name = after_kw[name_len..].trim_start();
    if !after_name.starts_with(':') {
        return None;
    }
    let after_colon = after_name[1..].trim_start();
    let mut type_len = 0;
    let bytes = after_colon.as_bytes();
    while type_len < bytes.len() {
        if is_ident_char(bytes[type_len]) {
            type_len += 1;
        } else if after_colon[type_len..].starts_with("::") {
            type_len += 2;
        } else {
            break;
        }
    }
    if type_len == 0 {
        return None;
    }
    let type_name = &after_colon[..type_len];
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
pub fn parse_action_defs(content: &str) -> Vec<ActionDef> {
    let masked = mask_string_literals(content);
    let mut actions = Vec::new();
    let mut idx = 0;
    while let Some(pos) = masked[idx..].find("action") {
        let act_pos = idx + pos;
        if !is_word_boundary_before(&masked, act_pos) || !is_word_boundary_after(&masked, act_pos + 6) {
            idx = act_pos + 6;
            continue;
        }
        let doc = extract_doc_comment_before(content, act_pos);
        let mut curr = act_pos + 6;
        curr += skip_ws(&masked[curr..]);
        let mut is_def = false;
        if masked[curr..].starts_with("def") && is_word_boundary_after(&masked[curr..], 3) {
            is_def = true;
            curr += 3;
            curr += skip_ws(&masked[curr..]);
        }
        let (name, name_len) = extract_ident(&masked[curr..]);
        if name.is_empty() {
            idx = act_pos + 6;
            continue;
        }
        curr += name_len;
        curr += skip_ws(&masked[curr..]);

        let mut params_str = None;
        if masked[curr..].starts_with('(') {
            if let Some(close_paren) = masked[curr + 1..].find(')') {
                params_str = Some(&content[curr + 1..curr + 1 + close_paren]);
                curr = curr + 1 + close_paren + 1;
                curr += skip_ws(&masked[curr..]);
            } else {
                idx = act_pos + 6;
                continue;
            }
        }

        let mut body_str = None;
        if masked[curr..].starts_with('{') {
            if let Some(close_pos) = find_matching_brace(content, curr) {
                body_str = Some(&content[curr + 1..close_pos]);
                idx = close_pos + 1;
            } else {
                idx = act_pos + 6;
                continue;
            }
        } else if masked[curr..].starts_with(';') {
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
                    let perf = trimmed[7..].trim().trim_end_matches(';').trim();
                    if !perf.is_empty() {
                        performer = Some(perf.to_string());
                    }
                } else if trimmed.starts_with("step") && is_word_boundary_after(trimmed, 4) {
                    let step_name = trimmed[4..].trim().trim_end_matches(';').trim();
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
pub fn parse_constraint_defs(content: &str) -> Vec<ConstraintDef> {
    let mut constraints = Vec::new();
    let mut idx = 0;
    while let Some(pos) = content[idx..].find("constraint") {
        let c_pos = idx + pos;
        if !is_word_boundary_before(content, c_pos) || !is_word_boundary_after(content, c_pos + 10) {
            idx = c_pos + 10;
            continue;
        }
        let before_c = content[..c_pos].trim_end();
        let (is_assertion, doc_end) = if before_c.ends_with("assert") && is_word_boundary_before(before_c, before_c.len() - 6) {
            (true, before_c.len() - 6)
        } else {
            (false, c_pos)
        };
        let doc = extract_doc_comment_before(content, doc_end);

        let mut curr = c_pos + 10;
        curr += skip_ws(&content[curr..]);
        if content[curr..].starts_with("def") && is_word_boundary_after(&content[curr..], 3) {
            curr += 3;
            curr += skip_ws(&content[curr..]);
        }
        let (name, name_len) = extract_ident(&content[curr..]);
        if name.is_empty() {
            idx = c_pos + 10;
            continue;
        }
        curr += name_len;
        curr += skip_ws(&content[curr..]);

        let mut parameters = Vec::new();
        if content[curr..].starts_with('(') {
            if let Some(close_paren) = content[curr + 1..].find(')') {
                let p_str = &content[curr + 1..curr + 1 + close_paren];
                parameters = p_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                curr = curr + 1 + close_paren + 1;
                curr += skip_ws(&content[curr..]);
            } else {
                idx = c_pos + 10;
                continue;
            }
        }

        let mut expression = String::new();
        if content[curr..].starts_with('{') {
            if let Some(close_pos) = find_matching_brace(content, curr) {
                let expr_str = &content[curr + 1..close_pos];
                expression = expr_str.trim().trim_end_matches(';').trim().to_string();
                idx = close_pos + 1;
            } else {
                idx = c_pos + 10;
                continue;
            }
        } else if content[curr..].starts_with(';') {
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
    let masked = mask_string_literals(content);
    let mut calcs = Vec::new();
    let mut idx = 0;
    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("calc")) {
        let calc_pos = idx + pos;
        if !is_word_boundary_before(&masked, calc_pos) || !is_word_boundary_after(&masked, calc_pos + 4) {
            idx = calc_pos + 4;
            continue;
        }
        let doc = extract_doc_comment_before(content, calc_pos);
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
                let p_str = content.get(curr + 1..curr + 1 + close_paren).unwrap_or("");
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
            let close_pos = find_matching_brace(content, open_pos);
            let b = if let Some(cp) = close_pos {
                content.get(open_pos + 1..cp).unwrap_or("")
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
pub fn parse_requirement_defs(content: &str) -> Vec<RequirementDef> {
    let mut reqs = Vec::new();
    let mut idx = 0;
    while let Some(pos) = content[idx..].find("requirement") {
        let req_pos = idx + pos;
        if !is_word_boundary_before(content, req_pos) || !is_word_boundary_after(content, req_pos + 11) {
            idx = req_pos + 11;
            continue;
        }
        let doc = extract_doc_comment_before(content, req_pos);
        let mut curr = req_pos + 11;
        curr += skip_ws(&content[curr..]);
        if content[curr..].starts_with("def") && is_word_boundary_after(&content[curr..], 3) {
            curr += 3;
            curr += skip_ws(&content[curr..]);
        }
        let (name, name_len) = extract_ident(&content[curr..]);
        if name.is_empty() {
            idx = req_pos + 11;
            continue;
        }
        curr += name_len;
        curr += skip_ws(&content[curr..]);

        let mut body_str = None;
        if content[curr..].starts_with('{') {
            if let Some(close_pos) = find_matching_brace(content, curr) {
                body_str = Some(&content[curr + 1..close_pos]);
                idx = close_pos + 1;
            } else {
                idx = req_pos + 11;
                continue;
            }
        } else if content[curr..].starts_with(';') {
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
                    let after = trimmed[2..].trim_start();
                    if after.starts_with('=') {
                        if let Some(val) = extract_quoted_string(after[1..].trim_start()) {
                            req_id = val.to_string();
                        }
                    }
                } else if trimmed.starts_with("req_id") && is_word_boundary_after(trimmed, 6) {
                    let after = trimmed[6..].trim_start();
                    if after.starts_with('=') {
                        if let Some(val) = extract_quoted_string(after[1..].trim_start()) {
                            req_id = val.to_string();
                        }
                    }
                } else if trimmed.starts_with("doc") && is_word_boundary_after(trimmed, 3) {
                    let after = trimmed[3..].trim_start();
                    if after.starts_with('=') {
                        if let Some(val) = extract_quoted_string(after[1..].trim_start()) {
                            text = val.to_string();
                        }
                    }
                } else if trimmed.starts_with("text") && is_word_boundary_after(trimmed, 4) {
                    let after = trimmed[4..].trim_start();
                    if after.starts_with('=') {
                        if let Some(val) = extract_quoted_string(after[1..].trim_start()) {
                            text = val.to_string();
                        }
                    }
                } else if trimmed.starts_with("assume") && is_word_boundary_after(trimmed, 6) {
                    let after = trimmed[6..].trim_start();
                    if let Some(target) = after.strip_suffix(';') {
                        assumes.push(target.trim().to_string());
                    }
                } else if trimmed.starts_with("require") && is_word_boundary_after(trimmed, 7) {
                    let after = trimmed[7..].trim_start();
                    if let Some(target) = after.strip_suffix(';') {
                        requires.push(target.trim().to_string());
                    }
                } else if trimmed.starts_with("derived") && is_word_boundary_after(trimmed, 7) {
                    let after = trimmed[7..].trim_start();
                    if after.starts_with("from") && is_word_boundary_after(after, 4) {
                        let after_from = after[4..].trim_start();
                        if let Some(target) = after_from.strip_suffix(';') {
                            derived_from.push(target.trim().to_string());
                        }
                    }
                } else if trimmed.starts_with("verify") && is_word_boundary_after(trimmed, 6) {
                    let after = trimmed[6..].trim_start();
                    let after_by = if after.starts_with("by") && is_word_boundary_after(after, 2) {
                        after[2..].trim_start()
                    } else {
                        after
                    };
                    if let Some(target) = after_by.strip_suffix(';') {
                        verified_by.push(target.trim().to_string());
                    }
                } else if trimmed.starts_with("satisfy") && is_word_boundary_after(trimmed, 7) {
                    let after = trimmed[7..].trim_start();
                    let after_by = if after.starts_with("by") && is_word_boundary_after(after, 2) {
                        after[2..].trim_start()
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

/// Parse test case definitions from SysML v2 source text.
pub fn parse_test_case_defs(content: &str) -> Vec<TestCaseDef> {
    let masked = mask_string_literals(content);
    let mut test_cases = Vec::new();
    let mut idx = 0;
    while let Some(pos) = masked[idx..].find("test") {
        let test_pos = idx + pos;
        if !is_word_boundary_before(&masked, test_pos) || !is_word_boundary_after(&masked, test_pos + 4) {
            idx = test_pos + 4;
            continue;
        }
        let doc = extract_doc_comment_before(content, test_pos);
        let mut curr = test_pos + 4;
        curr += skip_ws(&masked[curr..]);
        if !masked[curr..].starts_with("case") || !is_word_boundary_after(&masked[curr..], 4) {
            idx = test_pos + 4;
            continue;
        }
        curr += 4;
        curr += skip_ws(&masked[curr..]);
        if masked[curr..].starts_with("def") && is_word_boundary_after(&masked[curr..], 3) {
            curr += 3;
            curr += skip_ws(&masked[curr..]);
        }
        let (name, name_len) = extract_ident(&masked[curr..]);
        if name.is_empty() {
            idx = test_pos + 4;
            continue;
        }
        curr += name_len;
        curr += skip_ws(&masked[curr..]);

        let mut subject_part = None;
        let mut verified_requirements = Vec::new();
        let mut objective = None;
        let mut test_steps = Vec::new();

        if masked[curr..].starts_with('{') {
            if let Some(close_pos) = find_matching_brace(content, curr) {
                let body = &content[curr + 1..close_pos];
                for line in body.lines() {
                    let mut line_str = line;
                    if let Some(comment_pos) = line_str.find("//") {
                        line_str = &line_str[..comment_pos];
                    }
                    let trimmed = line_str.trim();
                    if trimmed.starts_with("subject") && is_word_boundary_after(trimmed, 7) {
                        let val = trimmed[7..].trim().trim_end_matches(';').trim();
                        if !val.is_empty() {
                            subject_part = Some(val.to_string());
                        }
                    } else if trimmed.starts_with("verify") && is_word_boundary_after(trimmed, 6) {
                        let mut rest = trimmed[6..].trim();
                        if rest.starts_with("assert") && is_word_boundary_after(rest, 6) {
                            rest = rest[6..].trim();
                        }
                        if rest.starts_with("requirement") && is_word_boundary_after(rest, 11) {
                            rest = rest[11..].trim();
                        } else if rest.starts_with("constraint") && is_word_boundary_after(rest, 10) {
                            rest = rest[10..].trim();
                        }
                        let targets = rest.trim_end_matches(';').trim();
                        for target in targets.split(',') {
                            let t = target.trim();
                            if !t.is_empty() {
                                verified_requirements.push(t.to_string());
                            }
                        }
                    } else if trimmed.starts_with("objective") && is_word_boundary_after(trimmed, 9) {
                        let val = trimmed[9..].trim().trim_end_matches(';').trim().trim_matches('"');
                        if !val.is_empty() {
                            objective = Some(val.to_string());
                        }
                    } else if trimmed.starts_with("step") && is_word_boundary_after(trimmed, 4) {
                        let val = trimmed[4..].trim().trim_end_matches(';').trim();
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
        } else if masked[curr..].starts_with(';') {
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
pub fn parse_connection_defs(content: &str) -> Vec<ConnectionDef> {
    let mut conns = Vec::new();
    let mut idx = 0;
    while idx < content.len() {
        let (_kw, is_flow, kw_len) = if content[idx..].starts_with("connection")
            && is_word_boundary_before(content, idx)
            && is_word_boundary_after(content, idx + 10)
        {
            ("connection", false, 10)
        } else if content[idx..].starts_with("flow")
            && is_word_boundary_before(content, idx)
            && is_word_boundary_after(content, idx + 4)
        {
            ("flow", true, 4)
        } else {
            idx += 1;
            continue;
        };

        let kw_pos = idx;
        let mut curr = kw_pos + kw_len;
        curr += skip_ws(&content[curr..]);
        // If keyword is "flow" and followed by "from", this is a statement inside a body, not a definition!
        if is_flow && content[curr..].starts_with("from") && is_word_boundary_after(&content[curr..], 4) {
            idx = kw_pos + kw_len;
            continue;
        }

        let doc = extract_doc_comment_before(content, kw_pos);
        if content[curr..].starts_with("def") && is_word_boundary_after(&content[curr..], 3) {
            curr += 3;
            curr += skip_ws(&content[curr..]);
        }
        let (name, name_len) = extract_ident(&content[curr..]);
        if name.is_empty() {
            idx = kw_pos + kw_len;
            continue;
        }
        curr += name_len;
        curr += skip_ws(&content[curr..]);
        if !content[curr..].starts_with('{') {
            idx = kw_pos + kw_len;
            continue;
        }
        let body = if let Some(close_pos) = find_matching_brace(content, curr) {
            let b = &content[curr + 1..close_pos];
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
                Some(trimmed[7..].trim_start())
            } else if trimmed.starts_with("flow") && is_word_boundary_after(trimmed, 4) {
                let after_flow = trimmed[4..].trim_start();
                if after_flow.starts_with("from") && is_word_boundary_after(after_flow, 4) {
                    Some(after_flow[4..].trim_start())
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(after) = after_conn {
                if let Some(to_pos) = after.find(" to ") {
                    let src = after[..to_pos].trim();
                    let tgt = after[to_pos + 4..].trim().trim_end_matches(';').trim();
                    source_port = src.to_string();
                    target_port = tgt.to_string();
                    break;
                }
            }
        }

        let source_part = if source_port.contains('.') {
            Some(source_port.split('.').next().unwrap().to_string())
        } else {
            None
        };

        let target_part = if target_port.contains('.') {
            Some(target_port.split('.').next().unwrap().to_string())
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
pub fn parse_state_defs(content: &str) -> Vec<StateDef> {
    let mut states = Vec::new();
    let mut idx = 0;
    while let Some(pos) = content[idx..].find("state") {
        let st_pos = idx + pos;
        if !is_word_boundary_before(content, st_pos) || !is_word_boundary_after(content, st_pos + 5) {
            idx = st_pos + 5;
            continue;
        }
        let doc = extract_doc_comment_before(content, st_pos);
        let mut curr = st_pos + 5;
        curr += skip_ws(&content[curr..]);
        if content[curr..].starts_with("def") && is_word_boundary_after(&content[curr..], 3) {
            curr += 3;
            curr += skip_ws(&content[curr..]);
        }
        let (name, name_len) = extract_ident(&content[curr..]);
        if name.is_empty() {
            idx = st_pos + 5;
            continue;
        }
        curr += name_len;
        curr += skip_ws(&content[curr..]);

        let mut body_str = None;
        if content[curr..].starts_with('{') {
            if let Some(close_pos) = find_matching_brace(content, curr) {
                body_str = Some(&content[curr + 1..close_pos]);
                idx = close_pos + 1;
            } else {
                idx = st_pos + 5;
                continue;
            }
        } else if content[curr..].starts_with(';') {
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
                    let act = trimmed[5..].trim().trim_end_matches(';').trim();
                    entry_action = Some(act.to_string());
                } else if trimmed.starts_with("do") && is_word_boundary_after(trimmed, 2) {
                    let act = trimmed[2..].trim().trim_end_matches(';').trim();
                    do_action = Some(act.to_string());
                } else if trimmed.starts_with("exit") && is_word_boundary_after(trimmed, 4) {
                    let act = trimmed[4..].trim().trim_end_matches(';').trim();
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
pub fn find_matching_brace(s: &str, open_pos: usize) -> Option<usize> {
    let mut depth = 0;
    let mut in_str = false;
    let mut in_escape = false;
    let chars = s[open_pos..].char_indices();
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

/// Helper to mask string literals in source code with spaces, preserving exact byte offsets.
pub fn mask_string_literals(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = bytes.to_vec();
    let mut in_str = false;
    let mut in_escape = false;
    for b in out.iter_mut() {
        if in_str {
            if in_escape {
                in_escape = false;
                *b = b' ';
            } else if *b == b'\\' {
                in_escape = true;
                *b = b' ';
            } else if *b == b'"' {
                in_str = false;
            } else if *b == b'\n' {
                // keep newline
            } else {
                *b = b' ';
            }
        } else if *b == b'"' {
            in_str = true;
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

/// Parse part definitions from SysML v2 source text.
pub fn parse_part_defs(content: &str) -> Vec<PartDef> {
    let masked = mask_string_literals(content);
    let mut parts = Vec::new();
    let mut idx = 0;
    while let Some(pos) = masked[idx..].find("part") {
        let part_pos = idx + pos;
        if !is_word_boundary_before(&masked, part_pos) || !is_word_boundary_after(&masked, part_pos + 4) {
            idx = part_pos + 4;
            continue;
        }
        let doc = extract_doc_comment_before(content, part_pos);
        let mut curr = part_pos + 4;
        curr += skip_ws(&masked[curr..]);
        let mut is_def = false;
        if masked[curr..].starts_with("def") && is_word_boundary_after(&masked[curr..], 3) {
            is_def = true;
            curr += 3;
            curr += skip_ws(&masked[curr..]);
        }
        let (name, name_len) = extract_ident(&masked[curr..]);
        if name.is_empty() {
            idx = part_pos + 4;
            continue;
        }
        curr += name_len;
        curr += skip_ws(&masked[curr..]);

        let mut type_name = None;
        if masked[curr..].starts_with(':') {
            curr += 1;
            curr += skip_ws(&masked[curr..]);
            let (t_name, t_len) = extract_type_ident(&masked[curr..]);
            if !t_name.is_empty() {
                type_name = Some(t_name.to_string());
                curr += t_len;
                curr += skip_ws(&masked[curr..]);
            }
        }

        let (body, next_idx) = if masked[curr..].starts_with('{') {
            let open_pos = curr;
            let close_pos = find_matching_brace(content, open_pos);
            let b = if let Some(cp) = close_pos {
                content.get(open_pos + 1..cp).unwrap_or("")
            } else {
                ""
            };
            let next = close_pos.map(|cp| cp + 1).unwrap_or(open_pos + 1);
            (b, next)
        } else if masked[curr..].starts_with(';') {
            ("", curr + 1)
        } else {
            idx = part_pos + 4;
            continue;
        };

        idx = next_idx;

        let attributes = parse_attribute_defs(body);
        let ports = parse_port_defs(body);
        let actions = parse_action_defs(body);
        let constraints = parse_constraint_defs(body);
        let requirements = parse_requirement_defs(body);
        let connections = parse_connection_defs(body);
        let states = parse_state_defs(body);
        let calcs = parse_calc_defs(body);
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
pub fn parse_sysml(content: &str) -> SysmlModel {
    let mut package_info = None;
    let mut idx = 0;
    while let Some(pos) = content[idx..].find("package") {
        let pkg_pos = idx + pos;
        if !is_word_boundary_before(content, pkg_pos) || !is_word_boundary_after(content, pkg_pos + 7) {
            idx = pkg_pos + 7;
            continue;
        }
        let doc = extract_doc_comment_before(content, pkg_pos);
        let mut curr = pkg_pos + 7;
        curr += skip_ws(&content[curr..]);
        let (name, name_len) = extract_ident(&content[curr..]);
        if name.is_empty() {
            idx = pkg_pos + 7;
            continue;
        }
        curr += name_len;
        curr += skip_ws(&content[curr..]);
        if content[curr..].starts_with('{') {
            let open_pos = curr;
            let body = if let Some(close_pos) = find_matching_brace(content, open_pos) {
                &content[open_pos + 1..close_pos]
            } else {
                &content[open_pos + 1..]
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
        let parts = parse_part_defs(content);
        let ports = parse_port_defs(content);
        let actions = parse_action_defs(content);
        let constraints = parse_constraint_defs(content);
        let requirements = parse_requirement_defs(content);
        let attributes = parse_attribute_defs(content);
        let test_cases = parse_test_case_defs(content);

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

/// Checks if an FMECA table component string matches an AST part def name.
/// Matches case-insensitively, ignoring whitespace, underscores, hyphens, and markdown formatting.
/// Also checks word boundary inclusion without regular expressions.
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

    // Word boundary containment: check if needle appears as a distinct word in haystack
    fn contains_word(haystack: &str, needle: &str) -> bool {
        let h_lower = haystack.to_lowercase();
        let n_lower = needle.to_lowercase();
        if n_lower.is_empty() {
            return false;
        }
        let mut start = 0;
        while let Some(pos) = h_lower[start..].find(&n_lower) {
            let actual_pos = start + pos;
            let before = if actual_pos == 0 {
                true
            } else {
                !h_lower[..actual_pos].chars().last().unwrap().is_alphanumeric()
            };
            let after_pos = actual_pos + n_lower.len();
            let after = if after_pos >= h_lower.len() {
                true
            } else {
                !h_lower[after_pos..].chars().next().unwrap().is_alphanumeric()
            };
            if before && after {
                return true;
            }
            start = actual_pos + 1;
        }
        false
    }

    contains_word(tc, pn) || contains_word(pn, tc)
}

/// Checks whether a component represents a recovery subsystem (e.g. Parachute, Ballistic Recovery, Chute).
pub fn is_recovery_component(comp: &str) -> bool {
    let lower = comp.to_lowercase();
    let clean: String = lower.chars().filter(|c| c.is_alphanumeric()).collect();
    clean.contains("parachute")
        || clean.contains("ballisticrecovery")
        || clean.contains("chuterecovery")
        || (clean.contains("recovery")
            && (clean.contains("system")
                || clean.contains("unit")
                || clean.contains("device")
                || clean.contains("chute")
                || clean.contains("subsystem")
                || clean.contains("module")
                || clean == "recovery"))
        || clean.starts_with("chute")
}

/// Checks whether model text or attributes/constraints explicitly negate recovery systems.
pub fn has_negative_recovery_constraint(content: &str) -> bool {
    let lower = content.to_lowercase();

    // Prose patterns in text / markdown
    let prose_patterns = [
        "recovery system: no",
        "recovery system : no",
        "recovery system: none",
        "recovery system : none",
        "no recovery system",
        "no vehicle recovery",
        "no parachute",
    ];
    for p in &prose_patterns {
        if lower.contains(p) {
            return true;
        }
    }

    let recovery_keys = [
        "recoverysystem",
        "recovery_system",
        "chuteenabled",
        "chute_enabled",
        "recoveryenabled",
        "recovery_enabled",
        "hasrecovery",
        "has_recovery",
        "hasparachute",
        "has_parachute",
    ];

    let negative_vals = ["0", "false", "\"no\"", "\"none\"", "no", "none"];

    for line in lower.lines() {
        let line_clean = if let Some(idx) = line.find("//") {
            &line[..idx]
        } else {
            line
        };

        for key in &recovery_keys {
            if line_clean.contains(key) {
                let parts: Vec<&str> = if line_clean.contains('=') {
                    line_clean.split('=').collect()
                } else if line_clean.contains(':') {
                    line_clean.split(':').collect()
                } else {
                    Vec::new()
                };

                if let Some(val_part) = parts.last() {
                    let val = val_part
                        .trim()
                        .trim_matches(|c: char| c == ';' || c == '}' || c == '{' || c == ',' || c == ')' || c == '(' || c == '.')
                        .trim();
                    for neg in &negative_vals {
                        if val == *neg || val.starts_with(neg) {
                            return true;
                        }
                    }
                }
            }
        }
    }

    // Direct substring checks on clean alphanumeric string
    let clean: String = lower.chars().filter(|c| c.is_alphanumeric() || *c == '=' || *c == '"' || *c == ':').collect();
    if clean.contains("recoverysystem=\"no\"")
        || clean.contains("recoverysystem==\"no\"")
        || clean.contains("recoverysystem:=\"no\"")
        || clean.contains("recoverysystem:\"no\"")
        || clean.contains("recoverysystem:no")
        || clean.contains("recoverysystem=no")
        || clean.contains("recoverysystem:=no")
        || clean.contains("recoverysystem=\"none\"")
        || clean.contains("recoverysystem==\"none\"")
        || clean.contains("recoverysystem:=\"none\"")
        || clean.contains("recoverysystem:\"none\"")
        || clean.contains("recoverysystem:none")
        || clean.contains("recoverysystem=none")
        || clean.contains("recoverysystem:=none")
        || clean.contains("chuteenabled=0")
        || clean.contains("chuteenabled==0")
        || clean.contains("chuteenabled:=0")
        || clean.contains("chuteenabled=false")
        || clean.contains("chuteenabled==false")
        || clean.contains("chuteenabled:=false")
        || clean.contains("chuteenabled:0")
        || clean.contains("chuteenabled:false")
        || clean.contains("recoveryenabled=0")
        || clean.contains("recoveryenabled==0")
        || clean.contains("recoveryenabled:=0")
        || clean.contains("recoveryenabled=false")
        || clean.contains("recoveryenabled==false")
        || clean.contains("recoveryenabled:=false")
        || clean.contains("hasrecovery=false")
        || clean.contains("hasrecovery:=false")
        || clean.contains("hasrecovery=0")
        || clean.contains("hasrecovery:=0")
        || clean.contains("hasparachute=false")
        || clean.contains("hasparachute:=false")
        || clean.contains("hasparachute=0")
        || clean.contains("hasparachute:=0")
    {
        return true;
    }

    false
}

/// Strips C-style multi-line comments (`/* ... */`) and line comments (`// ...`) from SysML text without regex.
pub fn strip_sysml_comments(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    while let Some(c) = chars.next() {
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
                chars.next();
                while let Some(next_c) = chars.next() {
                    if next_c == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        break;
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
pub fn is_valid_subsystem_identifier(s: &str) -> bool {
    if s.is_empty() || s.len() > 128 {
        return false;
    }
    let first = s.chars().next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    let generic_blacklist = &[
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
#[derive(Debug, Clone, Default)]
pub struct UnifiedSchemaModelExtractor;

impl UnifiedSchemaModelExtractor {
    /// Supported schema extensions across all pipeline tools.
    pub const SUPPORTED_EXTENSIONS: &'static [&'static str] = &[
        ".sysml", ".yaml", ".yml", ".json", ".proto", ".idl", ".arxml", ".md", ".markdown",
    ];

    /// Recursively discovers all supported schema files across the provided search directories.
    /// Excludes hidden entries (`.`), editor swap files (`#`), and `README.md`.
    /// Results are lexicographically sorted to eliminate filesystem traversal jitter.
    pub fn discover_schema_files<P: AsRef<Path>>(search_dirs: &[P]) -> Vec<PathBuf> {
        let mut schema_files = Vec::new();
        for sdir in search_dirs {
            let p = sdir.as_ref();
            if !p.is_dir() {
                continue;
            }
            for entry in WalkDir::new(p)
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    !name.starts_with('.') && !name.starts_with('#')
                })
                .filter_map(|e| e.ok())
            {
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
                    let rest = &trimmed[pos + 5..].trim_start();
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
                        let after = &line[start + 12..];
                        if let Some(end) = after.find("</SHORT-NAME>") {
                            let ident = after[..end].trim();
                            if is_valid_subsystem_identifier(ident) {
                                part_names.insert(ident.to_string());
                            }
                        }
                    }
                }
            }
        }

        part_names
    }

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
    fn test_parse_part_defs_and_ports() {
        let sysml_source = r#"
package AutonomousVehicle_SSOT {
    doc /* SSOT for Autonomous System Architecture and Safety Model */

    attribute actuatorCount : Integer = 4;
    attribute topologyConfiguration : String = "Cruciform";
    attribute catapultLaunchLimitG : Real = 12.0;
    attribute maxGLoad : Real = 12.0;

    part def Airframe {
        attribute massKg : Real = 25.0;
    }

    part def FlightControlComputer {
        port c2_bus : RS485;
        port telemetry : MAVLink;
    }
}
"#;
        let model = parse_sysml(sysml_source);
        assert_eq!(model.package_name.as_deref(), Some("AutonomousVehicle_SSOT"));
        assert_eq!(model.parts.len(), 2);

        let airframe = &model.parts[0];
        assert_eq!(airframe.name, "Airframe");
        assert_eq!(airframe.attributes.len(), 1);
        assert_eq!(airframe.attributes[0].name, "massKg");
        assert_eq!(airframe.attributes[0].type_name, "Real");
        assert_eq!(airframe.attributes[0].default_value.as_deref(), Some("25.0"));

        let fcc = &model.parts[1];
        assert_eq!(fcc.name, "FlightControlComputer");
        assert_eq!(fcc.ports.len(), 2);
        assert_eq!(fcc.ports[0].name, "c2_bus");
        assert_eq!(fcc.ports[0].type_name, "RS485");
        assert_eq!(fcc.ports[1].name, "telemetry");
        assert_eq!(fcc.ports[1].type_name, "MAVLink");
    }

    #[test]
    fn test_parse_actions_and_constraints() {
        let sysml_source = r#"
action def CalculateTrajectory {
    in attribute currentPos : Vector3;
    out attribute nextWaypoint : Vector3;
}

constraint def VelocityLimitConstraint(v : Real) {
    v <= 60.0;
}
"#;
        let actions = parse_action_defs(sysml_source);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].name, "CalculateTrajectory");
        assert_eq!(actions[0].in_params.len(), 1);
        assert_eq!(actions[0].in_params[0].name, "currentPos");
        assert_eq!(actions[0].out_params.len(), 1);
        assert_eq!(actions[0].out_params[0].name, "nextWaypoint");

        let constraints = parse_constraint_defs(sysml_source);
        assert_eq!(constraints.len(), 1);
        assert_eq!(constraints[0].name, "VelocityLimitConstraint");
        assert_eq!(constraints[0].parameters, vec!["v : Real"]);
        assert_eq!(constraints[0].expression, "v <= 60.0");
    }

    #[test]
    fn test_issue_428_parse_action_defs_nested_blocks_and_docblocks() {
        let sysml_source = r#"
/* Top-level license comment */
package VehicleControl {
    attribute configStr : String = "action def FakeActionInStringLiteral { in a : Int; }";

    /* Unrelated model comment */
    doc /* Primary path planning action */
    action def PlanTrajectory {
        in attribute currentPos : Vector3;
        if (check_bounds) {
            // Nested compound block with braces
            sub_routine();
        }
        out attribute targetWaypoint : Vector3;
        perform FlightComputer;
        step computeWaypoints;
        step smoothTrajectory;
    }

    doc /* Semicolon terminated action */
    action def EmergencyStop(in code : Integer);

    /* Comment before action usage */
    action executeManeuver;
}
"#;
        let actions = parse_action_defs(sysml_source);
        // FakeActionInStringLiteral must NOT be parsed!
        assert!(!actions.iter().any(|a| a.name == "FakeActionInStringLiteral"));

        let plan = actions.iter().find(|a| a.name == "PlanTrajectory").expect("PlanTrajectory should exist");
        assert_eq!(plan.doc.as_deref(), Some("Primary path planning action"));
        assert!(plan.is_def);
        assert_eq!(plan.in_params.len(), 1);
        assert_eq!(plan.in_params[0].name, "currentPos");
        assert_eq!(plan.out_params.len(), 1);
        assert_eq!(plan.out_params[0].name, "targetWaypoint");
        assert_eq!(plan.performer.as_deref(), Some("FlightComputer"));
        assert_eq!(plan.steps, vec!["computeWaypoints", "smoothTrajectory"]);

        let estop = actions.iter().find(|a| a.name == "EmergencyStop").expect("EmergencyStop should exist");
        assert_eq!(estop.doc.as_deref(), Some("Semicolon terminated action"));
        assert!(estop.is_def);
        assert_eq!(estop.in_params.len(), 1);
        assert_eq!(estop.in_params[0].name, "code");

        let maneuver = actions.iter().find(|a| a.name == "executeManeuver").expect("executeManeuver should exist");
        assert_eq!(maneuver.doc, None);
        assert!(!maneuver.is_def);

        assert_eq!(actions.len(), 3);
    }

    #[test]
    fn test_parse_connections_and_states() {
        let sysml_source = r#"
part def VehicleSystem {
    state def NavigationState {
        entry InitializeINS;
        do UpdateExtendedKalmanFilter;
        exit ShutdownSensors;
    }

    connection def GpsToFcc {
        connect gps.fix to fcc.nav_input;
    }
}
"#;
        let parts = parse_part_defs(sysml_source);
        assert_eq!(parts.len(), 1);
        let vehicle = &parts[0];
        assert_eq!(vehicle.name, "VehicleSystem");
        assert_eq!(vehicle.states.len(), 1);
        assert_eq!(vehicle.states[0].name, "NavigationState");
        assert_eq!(vehicle.states[0].entry_action.as_deref(), Some("InitializeINS"));
        assert_eq!(vehicle.connections.len(), 1);
        assert_eq!(vehicle.connections[0].name, "GpsToFcc");
        assert_eq!(vehicle.connections[0].source_port, "gps.fix");
        assert_eq!(vehicle.connections[0].target_port, "fcc.nav_input");
    }

    #[test]
    fn test_parse_ground_truth_model_fixture() {
        let fixture = include_str!("../../../tests/fixtures/safety/ground_truth_model.sysml");
        let model = parse_sysml(fixture);
        assert_eq!(model.package_name.as_deref(), Some("AutonomousVehicle_SSOT"));
        assert_eq!(model.parts.len(), 2);
    }

    #[test]
    fn test_parse_port_defs_syntax_variants() {
        let text = r#"
            out port rules_out : CompilerRulePort {
            }
            port out token_out : TokenStreamPort;
            in port schema_in : RawSchemaStreamPort {
                attribute protocol_family : String = "CAN";
            }
            port in data_in : DataPortType;
            port cmd : Command;
        "#;
        let ports = parse_port_defs(text);
        assert_eq!(ports.len(), 5);
        assert_eq!(ports[0].name, "rules_out");
        assert_eq!(ports[0].direction, "out");
        assert_eq!(ports[0].type_name, "CompilerRulePort");

        assert_eq!(ports[1].name, "token_out");
        assert_eq!(ports[1].direction, "out");
        assert_eq!(ports[1].type_name, "TokenStreamPort");

        assert_eq!(ports[2].name, "schema_in");
        assert_eq!(ports[2].direction, "in");
        assert_eq!(ports[2].type_name, "RawSchemaStreamPort");

        assert_eq!(ports[3].name, "data_in");
        assert_eq!(ports[3].direction, "in");
        assert_eq!(ports[3].type_name, "DataPortType");

        assert_eq!(ports[4].name, "cmd");
        assert_eq!(ports[4].direction, "inout");
        assert_eq!(ports[4].type_name, "Command");
    }

    #[test]
    fn test_parse_requirement_defs_with_attributes() {
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
        assert_eq!(reqs[0].req_id, "REQ-0001");
        assert_eq!(reqs[0].attributes.len(), 2);
        assert_eq!(reqs[0].attributes[0].name, "uuidv5");
        assert_eq!(
            reqs[0].attributes[0].default_value.as_deref(),
            Some("\"5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9\"")
        );
        assert_eq!(reqs[0].attributes[1].name, "complexity_class");
        assert_eq!(
            reqs[0].attributes[1].default_value.as_deref(),
            Some("\"Class P\"")
        );
    }

    #[test]
    fn test_parse_requirement_defs_constraints_assumptions_derivations() {
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
        assert_eq!(reqs[0].assumes, vec!["ValidSchemaInput"]);
        assert_eq!(
            reqs[0].requires,
            vec![
                "Invariant_REQ_0032_StrictTotalOrdering",
                "Invariant_REQ_0032_Determinism"
            ]
        );
        assert_eq!(
            reqs[0].derived_from,
            vec![
                "REQ_0031_BaseEmission",
                "REQ_0019_IdentifierSanitization"
            ]
        );
    }

    #[test]
    fn test_parse_requirement_defs_with_encapsulated_constraints() {
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
        assert_eq!(reqs[0].constraints.len(), 2);
        assert_eq!(reqs[0].constraints[0].name, "Invariant_Universal_Symbol_Grounding");
        assert_eq!(
            reqs[0].constraints[0].doc.as_deref(),
            Some("\\forall \\sigma \\in Symbols(A), \\sigma \\in Tokens(S)")
        );
        assert!(!reqs[0].constraints[0].is_assertion);
        assert_eq!(reqs[0].constraints[1].name, "Invariant_Zero_Hardcoded_Domain_Concepts");
        assert!(reqs[0].constraints[1].is_assertion);
    }

    #[test]
    fn test_package_def_recursive_ast_collectors() {
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
        assert_eq!(root.get_all_capabilities().len(), 1);
        assert_eq!(root.get_all_capabilities()[0].name, "SubCap");
        assert_eq!(root.get_all_parts().len(), 1);
        assert_eq!(root.get_all_parts()[0].name, "NestedPart");
        assert_eq!(root.get_all_use_cases().len(), 1);
        assert_eq!(root.get_all_use_cases()[0].name, "NestedUC");
        assert_eq!(root.get_all_interactions().len(), 1);
        assert_eq!(root.get_all_interactions()[0].name, "NestedInter");
        assert_eq!(root.get_all_actions().len(), 1);
        assert_eq!(root.get_all_actions()[0].name, "SubAction");
        assert_eq!(root.get_all_constraints().len(), 1);
        assert_eq!(root.get_all_constraints()[0].name, "NestedConstraint");
    }

    #[test]
    fn test_parse_test_case_defs_with_assert_constraints() {
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
        assert_eq!(test_cases[0].name, "TC_UCA_33_Verification");
        assert_eq!(test_cases[0].subject_part.as_deref(), Some("SafetyController"));
        assert_eq!(test_cases[0].verified_requirements, vec!["Assert_UCA_33", "Assert_UCA_34"]);
        assert_eq!(test_cases[0].objective.as_deref(), Some("Verify safety interlock per UCA-33"));
        assert_eq!(test_cases[0].test_steps, vec!["trigger_interlock"]);
    }

    #[test]
    fn test_component_matches_and_clean_names() {
        assert!(component_matches("Electric Motor", "Electric_Motor"));
        assert!(component_matches("FlightController", "FlightController"));
        assert!(component_matches("flight_controller", "FlightController"));
        assert!(component_matches("NavigationSensor", "navigation_sensor"));
        assert!(component_matches("[FlightController]", "FlightController"));
        assert!(component_matches("**FlightController**", "FlightController"));
        assert!(component_matches("`FlightController`", "FlightController"));
        assert!(!component_matches("PhantomSubsystem", "ElectricMotor"));
        assert!(!component_matches("GhostUnit", "FlightController"));
    }

    #[test]
    fn test_is_recovery_component_detection() {
        assert!(is_recovery_component("ParachuteRecovery"));
        assert!(is_recovery_component("Parachute"));
        assert!(is_recovery_component("BallisticRecoveryUnit"));
        assert!(is_recovery_component("ChuteDeploymentSystem"));
        assert!(is_recovery_component("EmergencyRecoverySubsystem"));
        assert!(!is_recovery_component("FlightController"));
        assert!(!is_recovery_component("NavigationSensor"));
        assert!(!is_recovery_component("ElectricMotor"));
    }

    #[test]
    fn test_has_negative_recovery_constraint_detection() {
        assert!(has_negative_recovery_constraint(r#"attribute recoverySystem = "No";"#));
        assert!(has_negative_recovery_constraint(r#"attribute chuteEnabled = 0;"#));
        assert!(has_negative_recovery_constraint(r#"attribute recovery_system = "none";"#));
        assert!(has_negative_recovery_constraint("Recovery system: No"));
        assert!(has_negative_recovery_constraint("No recovery system installed"));
        assert!(has_negative_recovery_constraint(r#"MockPlatform.recoverySystem = "No""#));
        assert!(has_negative_recovery_constraint(r#"MockPlatform.chuteEnabled = 0"#));
        assert!(!has_negative_recovery_constraint(r#"attribute recoverySystem = "Yes";"#));
        assert!(!has_negative_recovery_constraint(r#"attribute chuteEnabled = 1;"#));
        assert!(!has_negative_recovery_constraint("Nominal flight operations"));
    }

    #[test]
    fn test_unified_schema_model_extractor_all_formats() {
        // 1. SysML v2
        let sysml = r#"
            /* multi-line comment */
            package SampleSystem {
                // single line comment
                part def FlightComputer;
                part NavigationSensor :> SensorBase;
            }
        "#;
        let sysml_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".sysml", sysml);
        assert!(sysml_parts.contains("FlightComputer"));
        assert!(sysml_parts.contains("NavigationSensor"));

        // 2. YAML
        let yaml = r#"
            subsystems:
              - FlightComputer
              - GuidanceSystem
            components:
              - name: PowerDistributionUnit
                id: PDU-01
        "#;
        let yaml_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".yaml", yaml);
        assert!(yaml_parts.contains("FlightComputer"));
        assert!(yaml_parts.contains("GuidanceSystem"));
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
            | FlightManagementComputer | Core guidance | Segment A |
            | ActuatorControllerUnit | Surface control | Segment B |
        "#;
        let md_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".md", md);
        assert!(md_parts.contains("FlightManagementComputer"));
        assert!(md_parts.contains("ActuatorControllerUnit"));

        // 5. Protobuf
        let proto = r#"
            syntax = "proto3";
            package avionics;
            service AutopilotService {
                rpc GetState (Empty) returns (State);
            }
            message SensorData {
                int64 timestamp = 1;
            }
        "#;
        let proto_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".proto", proto);
        assert!(proto_parts.contains("AutopilotService"));
        assert!(proto_parts.contains("SensorData"));

        // 6. OMG IDL
        let idl = r#"
            module AvionicsBus {
                interface BusController {
                    void transmit();
                };
            };
        "#;
        let idl_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".idl", idl);
        assert!(idl_parts.contains("BusController"));
        assert!(idl_parts.contains("AvionicsBus"));

        // 7. ARXML
        let arxml = r#"
            <AUTOSAR>
                <SW-COMPONENT-PROTOTYPE>
                    <SHORT-NAME>EngineController</SHORT-NAME>
                </SW-COMPONENT-PROTOTYPE>
            </AUTOSAR>
        "#;
        let arxml_parts = UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".arxml", arxml);
        assert!(arxml_parts.contains("EngineController"));
    }

    #[test]
    fn test_parse_calc_defs_syntax_variants() {
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

        let ke = &calcs[0];
        assert_eq!(ke.name, "KineticEnergy");
        assert_eq!(ke.doc.as_deref(), Some("Calculates kinetic energy"));
        assert_eq!(ke.in_params.len(), 2);
        assert_eq!(ke.in_params[0].name, "massKg");
        assert_eq!(ke.in_params[0].type_name, "Real");
        assert_eq!(ke.in_params[1].name, "speedMps");
        assert_eq!(ke.in_params[1].type_name, "Real");
        assert_eq!(ke.return_type.as_deref(), Some("Real"));
        assert_eq!(ke.expression.as_deref(), Some("0.5 * massKg * speedMps * speedMps"));

        let vc = &calcs[1];
        assert_eq!(vc.name, "VelocityCalc");
        assert_eq!(vc.in_params.len(), 3);
        assert_eq!(vc.in_params[0].name, "v0");
        assert_eq!(vc.in_params[1].name, "a");
        assert_eq!(vc.in_params[2].name, "t");
        assert_eq!(vc.return_type.as_deref(), Some("Real"));
        assert_eq!(vc.expression.as_deref(), Some("v0 + a * t"));

        let mc = &calcs[2];
        assert_eq!(mc.name, "MarginCalculation");
        assert_eq!(mc.in_params.len(), 0);
        assert!(mc.return_type.is_none());
        assert!(mc.expression.is_none());
    }

    #[test]
    fn test_package_def_queries_and_nested_ports() {
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
            name: "NestedGps".to_string(),
            ports: vec![
                PortDef {
                    name: "gps_pps_out".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let top_part = PartDef {
            name: "FlightComputer".to_string(),
            ports: vec![
                PortDef {
                    name: "c2_bus".to_string(),
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
        assert!(port_names.contains(&"c2_bus"));
        assert!(port_names.contains(&"gps_pps_out"));

        // Test get_all_hazards
        let hazards = root.get_all_hazards();
        assert_eq!(hazards.len(), 1);
        assert_eq!(hazards[0].name, "H-01");

        // Test get_all_risks
        let risks = root.get_all_risks();
        assert_eq!(risks.len(), 1);
        assert_eq!(risks[0].name, "R-01");

        // Test get_all_states
        let states = root.get_all_states();
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].name, "Nominal");

        // Test get_all_connections
        let conns = root.get_all_connections();
        assert_eq!(conns.len(), 1);
        assert_eq!(conns[0].name, "C-01");

        // Test get_nested_ports: ONLY gps_pps_out is nested; c2_bus and root_port are top-level!
        let nested_ports = root.get_nested_ports();
        assert_eq!(nested_ports.len(), 1);
        assert!(nested_ports.contains("gps_pps_out"));
        assert!(!nested_ports.contains("c2_bus"));
        assert!(!nested_ports.contains("root_port"));
    }
}


