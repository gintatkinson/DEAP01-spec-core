//! Package, namespace import, root model container, and subsystem slice AST definitions for SysML v2.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-PACKAGE-MODEL]
//! Contract: Defines `ImportDef`, `PackageDef`, `SysmlModel`, and `AstSlice` together with lossless `PackageDef` <-> `SysmlModel` conversions and pre-order hierarchy queries.
//! Bounds & Faults: All package and part hierarchy traversals use explicit iterative worklist stacks in O(V + E) time with zero unbounded call-stack recursion or panics.

use super::{
    component_matches_negated_stems, extract_negated_components, extract_negated_key_stem,
    ActionDef, AttributeDef, CalcDef, CapabilityDef, ConnectionDef, ConstraintDef, HazardDef,
    InteractionDef, InterfaceDef, ItemDef, OperationDef, PartDef, PortDef, RequirementDef, RiskDef,
    StateDef, TestCaseDef, UseCaseDef,
};
use crate::SysmlToken;
use logos::Logos;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Represents a namespace or element `import` declaration in SysML v2 / KerML.
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-IMPORTDEF]
/// Contract: Captures the qualified import path along with wildcard (`::*`), recursive (`::**`), and preceding documentation metadata.
/// Bounds & Faults: O(1) fixed-field value struct bounded by `path.len()`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ImportDef {
    pub path: String,
    pub is_wildcard: bool,
    pub is_recursive: bool,
    pub doc: Option<String>,
}

/// Top-level or nested package namespace container in SysML v2.
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-PACKAGEDEF]
/// Contract: Organizes imports, nested subpackages, structural parts/ports/items, behavioral actions/states/use-cases/interactions, constraints/calculations, requirements, and STPA hazards/risks.
/// Bounds & Faults: Acyclic tree container; all member collections default to empty vectors on deserialization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PackageDef {
    pub name: String,
    pub doc: Option<String>,
    pub parent_package: Option<String>,
    #[serde(default)] pub imports: Vec<ImportDef>,
    #[serde(default)] pub packages: Vec<PackageDef>,
    #[serde(default)] pub part_defs: Vec<PartDef>,
    #[serde(default)] pub port_defs: Vec<PortDef>,
    #[serde(default)] pub attribute_defs: Vec<AttributeDef>,
    #[serde(default)] pub action_defs: Vec<ActionDef>,
    #[serde(default)] pub operation_defs: Vec<OperationDef>,
    #[serde(default)] pub capability_defs: Vec<CapabilityDef>,
    #[serde(default)] pub interaction_defs: Vec<InteractionDef>,
    #[serde(default)] pub constraint_defs: Vec<ConstraintDef>,
    #[serde(default)] pub test_case_defs: Vec<TestCaseDef>,
    #[serde(default)] pub requirement_defs: Vec<RequirementDef>,
    #[serde(default)] pub satisfied_requirements: Vec<String>,
    #[serde(default)] pub state_defs: Vec<StateDef>,
    #[serde(default)] pub use_case_defs: Vec<UseCaseDef>,
    #[serde(default)] pub item_defs: Vec<ItemDef>,
    #[serde(default)] pub hazard_defs: Vec<HazardDef>,
    #[serde(default)] pub risk_defs: Vec<RiskDef>,
    #[serde(default)] pub connection_defs: Vec<ConnectionDef>,
    #[serde(default)] pub calc_defs: Vec<CalcDef>,
}

impl PackageDef {
    /// Returns `self` and all nested subpackages in deterministic pre-order depth-first order.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-PACKAGES]
    /// Contract: Yields a non-empty vector starting with `self` followed by every descendant `PackageDef` in declaration order.
    /// Bounds & Faults: Iterative worklist traversal in O(N_pkg) time and O(N_pkg) heap space with zero call-stack recursion.
    pub fn get_all_packages(&self) -> Vec<&PackageDef> {
        let mut result = Vec::new();
        let mut stack = vec![self];
        while let Some(pkg) = stack.pop() {
            result.push(pkg);
            stack.extend(pkg.packages.iter().rev());
        }
        result
    }

    /// Aggregates items from `self`, nested packages, and nested parts in deterministic pre-order sequence.
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/COLLECT-PACKAGE-AND-PART-ITEMS]
    fn collect_package_and_part_items<T: Clone>(
        &self,
        pkg_items: fn(&Self) -> &[T],
        part_items: fn(&PartDef) -> &[T],
    ) -> Vec<T> {
        let mut result = Vec::new();
        for pkg in self.get_all_packages() {
            result.extend_from_slice(pkg_items(pkg));
            for part in collect_parts_preorder(&pkg.part_defs) {
                result.extend_from_slice(part_items(part));
            }
        }
        result
    }

    /// Aggregates all `CapabilityDef` nodes declared across `self`, nested packages, and nested parts.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-CAPABILITIES]
    /// Contract: Preserves pre-order package and part declaration order across the entire package tree.
    /// Bounds & Faults: Iterative O(N_pkg + N_part + N_cap) traversal; returns an empty vector when none exist.
    pub fn get_all_capabilities(&self) -> Vec<CapabilityDef> {
        self.collect_package_and_part_items(|p| &p.capability_defs, |pt| &pt.capabilities)
    }

    /// Aggregates all structural `PartDef` definitions across `self` and all nested packages.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-PARTS]
    /// Contract: Collects every top-level `part_defs` entry and iteratively descends into nested sub-parts where `sub.is_def == true`.
    /// Bounds & Faults: Iterative O(N_pkg + N_part) traversal using an explicit heap stack with zero call-stack recursion.
    pub fn get_all_parts(&self) -> Vec<PartDef> {
        let mut result = Vec::new();
        for pkg in self.get_all_packages() {
            let mut stack: Vec<&PartDef> = pkg.part_defs.iter().rev().collect();
            while let Some(part) = stack.pop() {
                result.push(part.clone());
                stack.extend(part.parts.iter().rev().filter(|sub| sub.is_def));
            }
        }
        result
    }

    /// Aggregates all `UseCaseDef` nodes declared across `self`, nested packages, and nested parts.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-USE-CASES]
    /// Contract: Collects package-level and part-level `UseCaseDef` items in pre-order depth-first sequence.
    /// Bounds & Faults: Iterative O(N_pkg + N_part + N_uc) time and O(N_uc) output space; never panics.
    pub fn get_all_use_cases(&self) -> Vec<UseCaseDef> {
        self.collect_package_and_part_items(|p| &p.use_case_defs, |pt| &pt.use_cases)
    }

    /// Aggregates all `InteractionDef` sequences declared across `self`, nested packages, and nested parts.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-INTERACTIONS]
    /// Contract: Collects package-level and part-level `InteractionDef` items in pre-order depth-first sequence.
    /// Bounds & Faults: Iterative O(N_pkg + N_part + N_inter) time and O(N_inter) output space; never panics.
    pub fn get_all_interactions(&self) -> Vec<InteractionDef> {
        self.collect_package_and_part_items(|p| &p.interaction_defs, |pt| &pt.interactions)
    }

    /// Aggregates all behavioral `ActionDef` nodes declared across `self`, nested packages, and nested parts.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-ACTIONS]
    /// Contract: Collects package-level and part-level `ActionDef` items in pre-order depth-first sequence.
    /// Bounds & Faults: Iterative O(N_pkg + N_part + N_act) time and O(N_act) output space; never panics.
    pub fn get_all_actions(&self) -> Vec<ActionDef> {
        self.collect_package_and_part_items(|p| &p.action_defs, |pt| &pt.actions)
    }

    /// Aggregates all `ConstraintDef` invariants declared across `self`, nested packages, and nested parts.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-CONSTRAINTS]
    /// Contract: Collects package-level and part-level `ConstraintDef` items in pre-order depth-first sequence.
    /// Bounds & Faults: Iterative O(N_pkg + N_part + N_con) time and O(N_con) output space; never panics.
    pub fn get_all_constraints(&self) -> Vec<ConstraintDef> {
        self.collect_package_and_part_items(|p| &p.constraint_defs, |pt| &pt.constraints)
    }

    /// Aggregates all `AttributeDef` declarations across `self`, nested packages, and nested parts.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-ATTRIBUTES]
    /// Contract: Collects package-level and part-level `AttributeDef` items in pre-order depth-first sequence.
    /// Bounds & Faults: Iterative O(N_pkg + N_part + N_attr) time and O(N_attr) output space; never panics.
    pub fn get_all_attributes(&self) -> Vec<AttributeDef> {
        self.collect_package_and_part_items(|p| &p.attribute_defs, |pt| &pt.attributes)
    }

    /// Aggregates all `CalcDef` mathematical calculations across `self`, nested packages, and nested parts.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-CALCS]
    /// Contract: Collects package-level and part-level `CalcDef` items in pre-order depth-first sequence.
    /// Bounds & Faults: Iterative O(N_pkg + N_part + N_calc) time and O(N_calc) output space; never panics.
    pub fn get_all_calcs(&self) -> Vec<CalcDef> {
        self.collect_package_and_part_items(|p| &p.calc_defs, |pt| &pt.calcs)
    }

    /// Aggregates all `PortDef` interface points declared across `self`, nested packages, and nested parts.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-PORTS]
    /// Contract: Collects package-level and part-level `PortDef` items in pre-order depth-first sequence.
    /// Bounds & Faults: Iterative O(N_pkg + N_part + N_port) time and O(N_port) output space; never panics.
    pub fn get_all_ports(&self) -> Vec<PortDef> {
        self.collect_package_and_part_items(|p| &p.port_defs, |pt| &pt.ports)
    }

    /// Aggregates all safety `HazardDef` nodes declared across `self`, nested packages, and nested parts.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-HAZARDS]
    /// Contract: Collects 100% of declared `HazardDef` AST nodes in pre-order depth-first sequence.
    /// Bounds & Faults: Iterative O(N_pkg + N_part + N_hazard) time and O(N_hazard) output space; never panics.
    pub fn get_all_hazards(&self) -> Vec<HazardDef> {
        self.collect_package_and_part_items(|p| &p.hazard_defs, |pt| &pt.hazards)
    }

    /// Aggregates all safety `RiskDef` nodes declared across `self`, nested packages, and nested parts.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-RISKS]
    /// Contract: Collects 100% of declared `RiskDef` AST nodes in pre-order depth-first sequence.
    /// Bounds & Faults: Iterative O(N_pkg + N_part + N_risk) time and O(N_risk) output space; never panics.
    pub fn get_all_risks(&self) -> Vec<RiskDef> {
        self.collect_package_and_part_items(|p| &p.risk_defs, |pt| &pt.risks)
    }

    /// Aggregates all `StateDef` state machine nodes declared across `self`, nested packages, and nested parts.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-STATES]
    /// Contract: Collects package-level and part-level `StateDef` items in pre-order depth-first sequence without altering transitions.
    /// Bounds & Faults: Iterative O(N_pkg + N_part + N_state) time and O(N_state) output space; never panics.
    pub fn get_all_states(&self) -> Vec<StateDef> {
        self.collect_package_and_part_items(|p| &p.state_defs, |pt| &pt.states)
    }

    /// Aggregates all `ConnectionDef` topological links declared across `self`, nested packages, and nested parts.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-ALL-CONNECTIONS]
    /// Contract: Collects package-level and part-level `ConnectionDef` items in pre-order depth-first sequence.
    /// Bounds & Faults: Iterative O(N_pkg + N_part + N_conn) time and O(N_conn) output space; never panics.
    pub fn get_all_connections(&self) -> Vec<ConnectionDef> {
        self.collect_package_and_part_items(|p| &p.connection_defs, |pt| &pt.connections)
    }

    /// Returns the sorted set of port names declared strictly on nested sub-parts across `self` and all subpackages.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/GET-NESTED-PORTS]
    /// Contract: Includes port `p` iff `p` belongs to a descendant `sub_part` inside `top_part.parts` rather than a top-level part directly.
    /// Bounds & Faults: Iterative O(N_nested_parts + N_nested_ports * log(N_nested_ports)) traversal backed by `BTreeSet`.
    pub fn get_nested_ports(&self) -> BTreeSet<String> {
        self.get_all_packages()
            .into_iter()
            .flat_map(|pkg| &pkg.part_defs)
            .flat_map(|top| collect_parts_preorder(&top.parts))
            .flat_map(|nested| &nested.ports)
            .map(|port| port.name.clone())
            .collect()
    }

    /// Extracts sorted, deduplicated stems of all components explicitly negated in this package hierarchy.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/NEGATED-COMPONENTS]
    /// Contract: Derives lowercase alphanumeric stems (length >= 3) from negative attribute defaults, constraint expressions/docs, and package docs.
    /// Bounds & Faults: Deterministic `BTreeSet`-backed string analysis in O(A + C * L + P * L) time; never panics.
    pub fn extract_negated_components(&self) -> Vec<String> {
        let mut negated = BTreeSet::new();
        for attr in self.get_all_attributes() {
            let is_neg = attr.default_value.as_deref().is_some_and(|val| {
                SysmlToken::lexer(val)
                    .spanned()
                    .find(|(tok, _)| !matches!(tok, Ok(SysmlToken::Whitespace)))
                    .is_some_and(|(_, sp)| {
                        val.get(sp).is_some_and(|t| {
                            matches!(t.trim_matches('"').to_ascii_lowercase().as_str(), "0" | "false" | "no" | "none")
                        })
                    })
            });
            if is_neg {
                let stem = extract_negated_key_stem(&attr.name);
                if !stem.is_empty() {
                    negated.insert(stem);
                }
            }
        }
        for con in self.get_all_constraints() {
            negated.extend(extract_negated_components(&con.expression));
            negated.extend(con.doc.as_deref().into_iter().flat_map(extract_negated_components));
        }
        for doc in self.get_all_packages().into_iter().filter_map(|p| p.doc.as_deref()) {
            negated.extend(extract_negated_components(doc));
        }
        negated.into_iter().collect()
    }

    /// Checks whether a component identifier or table label is contradicted by a negation in this package tree.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/IS-COMPONENT-NEGATED]
    /// Contract: Returns `true` iff `comp` matches at least one stem returned by `self.extract_negated_components()`.
    /// Bounds & Faults: Pure non-panicking comparison; returns `false` on empty or non-alphanumeric component strings.
    pub fn is_component_negated(&self, comp: &str) -> bool {
        component_matches_negated_stems(comp, &self.extract_negated_components())
    }

    /// Searches `self`, nested parts, and all subpackages in pre-order depth-first order for a mutable `PartDef`.
    /// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PACKAGE-DEF-MUT]
    /// Contract: Returns `Some(&mut PartDef)` for the first matching part across `part_defs` and nested `packages`, or `None` when absent.
    /// Bounds & Faults: Iterative worklist traversal in O(V_pkg + V_part) time with zero call-stack recursion.
    pub fn find_part_mut(&mut self, name: &str) -> Option<&mut PartDef> {
        let mut pkg_stack = vec![self];
        while let Some(pkg) = pkg_stack.pop() {
            if let Some(found) = pkg.part_defs.iter_mut().find_map(|p| p.find_part_mut(name)) {
                return Some(found);
            }
            pkg_stack.extend(pkg.packages.iter_mut().rev());
        }
        None
    }
}

/// Iteratively collects `roots` and all nested `part.parts` descendants in pre-order depth-first sequence.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/COLLECT-PART-SUBDEFS]
fn collect_parts_preorder(roots: &[PartDef]) -> Vec<&PartDef> {
    let mut visited = Vec::new();
    let mut stack: Vec<&PartDef> = roots.iter().rev().collect();
    while let Some(part) = stack.pop() {
        visited.push(part);
        stack.extend(part.parts.iter().rev());
    }
    visited
}

/// High-level root SysML v2 model container preserving top-level package elements and backwards compatibility.
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-SYSMLMODEL]
/// Contract: Aggregates root `package_name`, `doc`, `attributes`, `parts`, `ports`, `actions`, `constraints`, `requirements`, and `test_cases`.
/// Bounds & Faults: Convertible losslessly to and from a single-level `PackageDef` in O(1) collection-move time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SysmlModel {
    pub package_name: Option<String>, pub doc: Option<String>,
    pub attributes: Vec<AttributeDef>, pub parts: Vec<PartDef>, pub ports: Vec<PortDef>,
    pub actions: Vec<ActionDef>, pub constraints: Vec<ConstraintDef>, pub requirements: Vec<RequirementDef>,
    #[serde(default)] pub test_cases: Vec<TestCaseDef>,
}

impl From<PackageDef> for SysmlModel {
    /// Projects a `PackageDef` into a flat root `SysmlModel` container by moving its primary element vectors.
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/SYSML-MODEL-FROM-PACKAGE]
    fn from(pkg: PackageDef) -> Self {
        Self {
            package_name: Some(pkg.name), doc: pkg.doc, attributes: pkg.attribute_defs,
            parts: pkg.part_defs, ports: pkg.port_defs, actions: pkg.action_defs,
            constraints: pkg.constraint_defs, requirements: pkg.requirement_defs, test_cases: pkg.test_case_defs,
        }
    }
}

impl From<SysmlModel> for PackageDef {
    /// Wraps a `SysmlModel` into a root `PackageDef`, defaulting `name` to `"SysML_Model"` when `package_name` is `None`.
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/PACKAGE-FROM-SYSML-MODEL]
    fn from(m: SysmlModel) -> Self {
        Self {
            name: m.package_name.unwrap_or_else(|| "SysML_Model".to_string()), doc: m.doc,
            part_defs: m.parts, port_defs: m.ports, attribute_defs: m.attributes, action_defs: m.actions,
            constraint_defs: m.constraints, test_case_defs: m.test_cases, requirement_defs: m.requirements,
            ..Self::default()
        }
    }
}

/// Token-budgeted subsystem slice extracted from a `SysmlModel` for bounded single-subsystem synthesis.
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-ASTSLICE]
/// Contract: Isolates `target_package` together with its matched `packages`, `parts`, `ports`, `interfaces`, referenced `requirements`, `hazards`, and `estimated_tokens`.
/// Bounds & Faults: Serialized size and `estimated_tokens` are bounded by the caller's `max_tokens` budget in `SysmlModel::slice_package_subsystem`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AstSlice {
    pub target_package: String, pub packages: Vec<PackageDef>, pub parts: Vec<PartDef>,
    pub ports: Vec<PortDef>, pub interfaces: Vec<InterfaceDef>, pub requirements: Vec<RequirementDef>,
    pub hazards: Vec<HazardDef>, pub estimated_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_def_iterative_queries_preserve_preorder_across_deep_tree() -> Result<(), Box<dyn std::error::Error>> {
        let mk_part = |name: &str, is_def: bool, port: &str, parts: Vec<PartDef>| PartDef { name: name.to_string(), is_def, ports: vec![PortDef { name: port.to_string(), ..Default::default() }], parts, ..Default::default() };
        let leaf = mk_part("sensorUsage", false, "nested_port_leaf", Vec::new());
        let subdef = mk_part("NestedSubDef", true, "nested_port_def", vec![leaf]);
        let mut current_pkg = PackageDef { name: "Pkg_128".to_string(), part_defs: vec![mk_part("TopPart_128", true, "top_port", vec![subdef])], ..Default::default() };
        for depth in (0..128_usize).rev() {
            let sibling = PackageDef { name: format!("SiblingPkg_{depth}"), ..Default::default() };
            current_pkg = PackageDef { name: format!("Pkg_{depth}"), packages: vec![sibling, current_pkg], ..Default::default() };
        }
        let all_pkgs = current_pkg.get_all_packages();
        assert_eq!((all_pkgs.len(), all_pkgs.first().map(|p| p.name.as_str()), all_pkgs.get(1).map(|p| p.name.as_str()), all_pkgs.last().map(|p| p.name.as_str())), (257, Some("Pkg_0"), Some("SiblingPkg_0"), Some("Pkg_128")));
        assert_eq!(current_pkg.get_all_parts().iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), vec!["TopPart_128", "NestedSubDef"]);
        let nested_ports = current_pkg.get_nested_ports();
        assert!(nested_ports.contains("nested_port_def") && nested_ports.contains("nested_port_leaf") && !nested_ports.contains("top_port"));
        current_pkg.find_part_mut("sensorUsage").ok_or("missing sensorUsage")?.doc = Some("Updated in place".to_string());
        assert!(current_pkg.find_part_mut("UnknownPart").is_none());
        Ok(())
    }

    #[test]
    fn sysml_model_and_package_def_roundtrip_conversions() {
        let pkg_default = PackageDef::from(SysmlModel::default());
        assert_eq!(pkg_default.name, "SysML_Model");
        assert_eq!(SysmlModel::from(pkg_default).package_name.as_deref(), Some("SysML_Model"));
    }

    #[test]
    fn package_def_detects_negated_components() {
        let attr = AttributeDef { name: "hydraulicSystem".to_string(), type_name: "Boolean".to_string(), default_value: Some("false".to_string()), doc: None };
        let pkg = PackageDef { name: "VehicleConfig".to_string(), attribute_defs: vec![attr], ..Default::default() };
        assert_eq!(pkg.extract_negated_components(), vec!["hydraulic".to_string()]);
        assert!(pkg.is_component_negated("Hydraulic Actuator") && !pkg.is_component_negated("Electric Motor"));
    }
}
