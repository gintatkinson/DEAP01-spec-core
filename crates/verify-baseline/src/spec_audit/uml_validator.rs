//! Strongly-typed cross-validation of Mermaid UML diagrams against SysML v2 models.
//! Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR]

use std::collections::{BTreeMap, BTreeSet};

use deap_core::sysml_ast::{PackageDef, PartDef, StateDef, SysmlModel};

pub use crate::spec_audit::uml_compliance_gates::{
    validate_epic_statechart_heterogeneity, validate_parent_epic_alignment,
    validate_use_case_diagram_compliance,
};
pub use crate::spec_audit::uml_coverage_gates::{
    validate_feature_operation_coverage, validate_user_story_test_cases,
};
pub use crate::spec_audit::uml_diagram_gates::{
    validate_class_diagram, validate_document_uml, validate_sequence_diagram,
    validate_source_references, validate_state_diagram,
};

/// Pre-indexed symbol set for a single classifier (Part / Block) in SysML AST.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-CLASSIFIERSYMBOLINFO]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClassifierSymbolInfo {
    /// Classifier identifier name (e.g. `FlightController`, `SensorHub`).
    pub name: String,
    /// Declared attribute names.
    pub attributes: BTreeSet<String>,
    /// Declared operation names.
    pub operations: BTreeSet<String>,
    /// Declared action names.
    pub actions: BTreeSet<String>,
    /// Declared port names.
    pub ports: BTreeSet<String>,
    /// Declared state names if classifier owns a state machine.
    pub states: BTreeSet<String>,
}

/// Pre-indexed symbol lookup tables providing O(1) resolution against SysML v2 models.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-SYSMLSYMBOLINDEX]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SysmlSymbolIndex {
    /// Valid Part / Block names in the model.
    pub part_names: BTreeSet<String>,
    /// Valid State names across all state machines in the model.
    pub state_names: BTreeSet<String>,
    /// Valid Action and Operation names across all parts and packages.
    pub action_names: BTreeSet<String>,
    /// Valid Port names across all parts and packages.
    pub port_names: BTreeSet<String>,
    /// Valid Attribute names across all parts and packages.
    pub attribute_names: BTreeSet<String>,
    /// Mapping of classifier name -> ClassifierSymbolInfo.
    pub classifiers: BTreeMap<String, ClassifierSymbolInfo>,
    /// Valid Requirement IDs and names.
    pub requirement_ids: BTreeSet<String>,
    /// Valid Constraint IDs and names (including assert constraints).
    pub constraint_ids: BTreeSet<String>,
    /// Test case name -> list of verified requirement/constraint IDs.
    pub test_cases: BTreeMap<String, Vec<String>>,
}

impl SysmlSymbolIndex {
    /// Constructs an empty symbol index.
    /// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-SYSMLSYMBOLINDEX-NEW]
    pub fn new() -> Self {
        Self::default()
    }

    /// Pre-indexes a high-level `SysmlModel` into O(1) lookup tables.
    /// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-SYSMLSYMBOLINDEX-FROM-MODEL]
    pub fn from_model(model: &SysmlModel) -> Self {
        let mut index = Self::new();
        for attr in &model.attributes {
            if !attr.name.is_empty() {
                index.attribute_names.insert(attr.name.clone());
            }
        }
        for port in &model.ports {
            if !port.name.is_empty() {
                index.port_names.insert(port.name.clone());
            }
        }
        for act in &model.actions {
            if !act.name.is_empty() {
                index.action_names.insert(act.name.clone());
            }
        }
        for c in &model.constraints {
            if !c.name.is_empty() {
                index.constraint_ids.insert(c.name.clone());
            }
        }
        for req in &model.requirements {
            if !req.name.is_empty() {
                index.requirement_ids.insert(req.name.clone());
            }
            if !req.req_id.is_empty() {
                index.requirement_ids.insert(req.req_id.clone());
            }
            for c in &req.constraints {
                if !c.name.is_empty() {
                    index.constraint_ids.insert(c.name.clone());
                }
            }
        }
        for tc in &model.test_cases {
            if !tc.name.is_empty() {
                index.test_cases.insert(tc.name.clone(), tc.verified_requirements.clone());
            }
        }
        for part in &model.parts {
            index.index_part(part);
        }
        index.link_type_names();
        index
    }

    /// Pre-indexes a hierarchical `PackageDef` into O(1) lookup tables.
    /// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-SYSMLSYMBOLINDEX-FROM-PACKAGE]
    pub fn from_package(pkg: &PackageDef) -> Self {
        let mut index = Self::new();
        index.index_package_recursive(pkg);
        index.link_type_names();
        index
    }

    /// Recursively indexes a [`PackageDef`] and its nested packages.
    /// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-SYSMLSYMBOLINDEX-INDEX-PACKAGE-RECURSIVE]
    fn index_package_recursive(&mut self, pkg: &PackageDef) {
        for attr in &pkg.attribute_defs {
            if !attr.name.is_empty() {
                self.attribute_names.insert(attr.name.clone());
            }
        }
        for port in &pkg.port_defs {
            if !port.name.is_empty() {
                self.port_names.insert(port.name.clone());
            }
        }
        for act in &pkg.action_defs {
            if !act.name.is_empty() {
                self.action_names.insert(act.name.clone());
            }
        }
        for op in &pkg.operation_defs {
            if !op.name.is_empty() {
                self.action_names.insert(op.name.clone());
            }
        }
        for c in &pkg.constraint_defs {
            if !c.name.is_empty() {
                self.constraint_ids.insert(c.name.clone());
            }
        }
        for req in &pkg.requirement_defs {
            if !req.name.is_empty() {
                self.requirement_ids.insert(req.name.clone());
            }
            if !req.req_id.is_empty() {
                self.requirement_ids.insert(req.req_id.clone());
            }
            for c in &req.constraints {
                if !c.name.is_empty() {
                    self.constraint_ids.insert(c.name.clone());
                }
            }
        }
        for tc in &pkg.test_case_defs {
            if !tc.name.is_empty() {
                self.test_cases.insert(tc.name.clone(), tc.verified_requirements.clone());
            }
        }
        for st in &pkg.state_defs {
            self.index_state(st);
        }
        for part in &pkg.part_defs {
            self.index_part(part);
        }
        for subpkg in &pkg.packages {
            self.index_package_recursive(subpkg);
        }
    }

    /// Recursively indexes a [`PartDef`] and its member declarations.
    /// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-SYSMLSYMBOLINDEX-INDEX-PART]
    fn index_part(&mut self, part: &PartDef) {
        if !part.name.is_empty() {
            self.index_part_classifier_members(part);
        }
        for c in &part.constraints {
            if !c.name.is_empty() {
                self.constraint_ids.insert(c.name.clone());
            }
        }
        for req in &part.requirements {
            if !req.name.is_empty() {
                self.requirement_ids.insert(req.name.clone());
            }
            if !req.req_id.is_empty() {
                self.requirement_ids.insert(req.req_id.clone());
            }
            for c in &req.constraints {
                if !c.name.is_empty() {
                    self.constraint_ids.insert(c.name.clone());
                }
            }
        }
        for tc in &part.test_cases {
            if !tc.name.is_empty() {
                self.test_cases.insert(tc.name.clone(), tc.verified_requirements.clone());
            }
        }
        for st in &part.states {
            self.index_state(st);
        }
        if let Some(ref type_name) = part.type_name {
            if !type_name.is_empty() {
                self.part_names.insert(type_name.clone());
            }
        }
        for subpart in &part.parts {
            self.index_part(subpart);
        }
    }

    /// Populates classifier symbol sets for a named [`PartDef`].
    /// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-SYSMLSYMBOLINDEX-INDEX-PART]
    fn index_part_classifier_members(&mut self, part: &PartDef) {
        self.part_names.insert(part.name.clone());
        let mut classifier_states = BTreeSet::new();
        for st in &part.states {
            collect_state_names(st, &mut classifier_states);
        }
        let classifier = self
            .classifiers
            .entry(part.name.clone())
            .or_insert_with(|| ClassifierSymbolInfo { name: part.name.clone(), ..Default::default() });

        for attr in &part.attributes {
            if !attr.name.is_empty() {
                classifier.attributes.insert(attr.name.clone());
                self.attribute_names.insert(attr.name.clone());
            }
        }
        for op in &part.operations {
            if !op.name.is_empty() {
                classifier.operations.insert(op.name.clone());
                self.action_names.insert(op.name.clone());
            }
        }
        for act in &part.actions {
            if !act.name.is_empty() {
                classifier.actions.insert(act.name.clone());
                self.action_names.insert(act.name.clone());
            }
        }
        for port in &part.ports {
            if !port.name.is_empty() {
                classifier.ports.insert(port.name.clone());
                self.port_names.insert(port.name.clone());
            }
        }
        classifier.states.extend(classifier_states);
    }

    /// Recursively indexes a [`StateDef`] and its transitions.
    /// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-SYSMLSYMBOLINDEX-INDEX-STATE]
    pub fn index_state(&mut self, state: &StateDef) {
        if !state.name.is_empty() {
            self.state_names.insert(state.name.clone());
        }
        for act in [&state.entry_action, &state.do_action, &state.exit_action]
            .into_iter()
            .flatten()
        {
            let ident = extract_ident(act);
            if !ident.is_empty() {
                self.action_names.insert(ident.to_string());
            }
        }
        for trans in &state.transitions {
            for a in [&trans.name, &trans.effect, &trans.trigger]
                .into_iter()
                .flatten()
            {
                let ident = extract_ident(a);
                if !ident.is_empty() {
                    self.action_names.insert(ident.to_string());
                }
            }
            for st in [&trans.source, &trans.target].into_iter().flatten() {
                if st != "Initial" && st != "Terminal" && st != "[*]" && !st.is_empty() {
                    self.state_names.insert(st.clone());
                }
            }
        }
        for sub in &state.sub_states {
            self.index_state(sub);
        }
    }

    /// Links classifier instances with their declared type definition symbols.
    /// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-SYSMLSYMBOLINDEX-LINK-TYPE-NAMES]
    fn link_type_names(&mut self) {
        let known = self.classifiers.clone();
        for (name, classifier) in &mut self.classifiers {
            if let Some(suffix_idx) = name.rfind(':') {
                let type_cand = name[suffix_idx + 1..].trim();
                if let Some(parent) = known.get(type_cand) {
                    classifier.attributes.extend(parent.attributes.clone());
                    classifier.operations.extend(parent.operations.clone());
                    classifier.actions.extend(parent.actions.clone());
                    classifier.ports.extend(parent.ports.clone());
                }
            }
        }
    }
}

/// Extracts a clean alphanumeric identifier from an operation or action invocation string.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-EXTRACT-IDENT]
pub(crate) fn extract_ident(s: &str) -> &str {
    let trimmed = s.trim().trim_start_matches(['+', '-', '#', '~']).trim();
    if let Some(open_p) = trimmed.find('(') {
        trimmed[..open_p].trim()
    } else {
        trimmed
    }
}

/// Recursively collects all state names declared in a state hierarchy.
/// Realises: [REQ-SPEC-AUDIT-UML-VALIDATOR-COLLECT-STATE-NAMES]
fn collect_state_names(state: &StateDef, names: &mut BTreeSet<String>) {
    if !state.name.is_empty() {
        names.insert(state.name.clone());
    }
    for sub in &state.sub_states {
        collect_state_names(sub, names);
    }
}

#[cfg(test)]
#[path = "uml_tests_a.rs"]
mod uml_tests_a;
#[cfg(test)]
#[path = "uml_tests_b.rs"]
mod uml_tests_b;
