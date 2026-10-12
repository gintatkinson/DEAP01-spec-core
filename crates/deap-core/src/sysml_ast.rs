//! AST structures for SysML v2 entities, requirements, and system architecture.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
            test_case_defs: Vec::new(),
            requirement_defs: m.requirements,
            state_defs: Vec::new(),
            use_case_defs: Vec::new(),
            item_defs: Vec::new(),
            hazard_defs: Vec::new(),
            risk_defs: Vec::new(),
            connection_defs: Vec::new(),
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
    let mut actions = Vec::new();
    let mut idx = 0;
    while let Some(pos) = content[idx..].find("action") {
        let act_pos = idx + pos;
        if !is_word_boundary_before(content, act_pos) || !is_word_boundary_after(content, act_pos + 6) {
            idx = act_pos + 6;
            continue;
        }
        let doc = extract_doc_comment_before(content, act_pos);
        let mut curr = act_pos + 6;
        curr += skip_ws(&content[curr..]);
        if content[curr..].starts_with("def") && is_word_boundary_after(&content[curr..], 3) {
            curr += 3;
            curr += skip_ws(&content[curr..]);
        }
        let (name, name_len) = extract_ident(&content[curr..]);
        if name.is_empty() {
            idx = act_pos + 6;
            continue;
        }
        curr += name_len;
        curr += skip_ws(&content[curr..]);

        let mut params_str = None;
        if content[curr..].starts_with('(') {
            if let Some(close_paren) = content[curr + 1..].find(')') {
                params_str = Some(&content[curr + 1..curr + 1 + close_paren]);
                curr = curr + 1 + close_paren + 1;
                curr += skip_ws(&content[curr..]);
            } else {
                idx = act_pos + 6;
                continue;
            }
        }

        let mut body_str = None;
        if content[curr..].starts_with('{') {
            if let Some(close_pos) = find_matching_brace(content, curr) {
                body_str = Some(&content[curr + 1..close_pos]);
                idx = close_pos + 1;
            } else {
                idx = act_pos + 6;
                continue;
            }
        } else if content[curr..].starts_with(';') {
            idx = curr + 1;
        } else {
            idx = act_pos + 6;
            continue;
        }

        let mut in_params = Vec::new();
        let mut out_params = Vec::new();

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
                if let Some((is_in, attr)) = parse_action_param(line) {
                    if is_in {
                        in_params.push(attr);
                    } else {
                        out_params.push(attr);
                    }
                }
            }
        }

        actions.push(ActionDef {
            name: name.to_string(),
            doc,
            in_params,
            out_params,
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
        if !masked[curr..].starts_with("def") || !is_word_boundary_after(&masked[curr..], 3) {
            idx = part_pos + 4;
            continue;
        }
        curr += 3;
        curr += skip_ws(&masked[curr..]);
        let (name, name_len) = extract_ident(&masked[curr..]);
        if name.is_empty() {
            idx = part_pos + 4;
            continue;
        }
        curr += name_len;
        curr += skip_ws(&masked[curr..]);

        let (body, next_idx) = if masked[curr..].starts_with('{') {
            let open_pos = curr;
            let close_pos = find_matching_brace(content, open_pos);
            let b = if let Some(cp) = close_pos {
                &content[open_pos + 1..cp]
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

        parts.push(PartDef {
            name: name.to_string(),
            doc,
            is_def: true,
            type_name: None,
            attributes,
            ports,
            actions,
            constraints,
            requirements,
            parts: Vec::new(),
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

        SysmlModel {
            package_name: Some(package_name),
            doc,
            attributes,
            parts,
            ports,
            actions,
            constraints,
            requirements,
        }
    } else {
        let parts = parse_part_defs(content);
        let ports = parse_port_defs(content);
        let actions = parse_action_defs(content);
        let constraints = parse_constraint_defs(content);
        let requirements = parse_requirement_defs(content);
        let attributes = parse_attribute_defs(content);

        SysmlModel {
            package_name: None,
            doc: None,
            attributes,
            parts,
            ports,
            actions,
            constraints,
            requirements,
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

    attribute ruddervatorCount : Integer = 4;
    attribute tailConfiguration : String = "X-tail";
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
}


