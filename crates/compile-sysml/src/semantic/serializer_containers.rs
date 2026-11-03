#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! SysML v2 canonical serialization for composite container and import AST nodes (`PartDef`, `ImportDef`, `PackageDef`).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for composite AST serialization.
//! ## 2. Invariants & Mathematical Properties
//! Renders nested `PartDef`, `ImportDef`, and `PackageDef` hierarchies deterministically up to `MAX_AST_DEPTH` (64).
//! ## 3. Preconditions
//! AST containers conform to `deap_core::sysml_ast` structural invariants.
//! ## 4. Postconditions
//! Emits canonical SysML v2 container blocks while truncating recursion safely at `MAX_AST_DEPTH` (64).
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(V) over AST subtree within depth 64. Space Complexity: O(V) output buffer.
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Bounded depth recursion with saturating arithmetic and zero panics.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603, REQ-SEMANTIC-SERIALIZER-PARTDEF-TO-SYSML, REQ-SEMANTIC-SERIALIZER-PACKAGEDEF-TO-SYSML]

use super::serializer::{
    format_doc_comment, BoundedSysmlSerializable, SysmlSerializable, MAX_AST_DEPTH,
};
use deap_core::sysml_ast::{ImportDef, PackageDef, PartDef};

impl SysmlSerializable for PartDef {
    /// Serializes `PartDef` into canonical SysML v2 notation starting at recursion depth zero.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-PARTDEF-TO-SYSML]
    fn to_sysml(&self, indent: usize) -> String {
        self.to_sysml_bounded(indent, 0)
    }
}

/// Appends primary attributes, ports, actions, constraints, calcs, and requirements of `part` to `lines`.
/// Realises: [REQ-SEMANTIC-SERIALIZER-PARTDEF-TO-SYSML]
fn append_part_primary_members(
    part: &PartDef,
    pad: &str,
    child_indent: usize,
    lines: &mut Vec<String>,
) {
    for h in &part.triggers_hazards {
        lines.push(format!("{}    @TriggersHazard(\"{}\");", pad, h));
    }
    for sr in &part.safety_realises {
        lines.push(format!("{}    @SafetyRealises(\"{}\");", pad, sr));
    }
    if let Some(lyr) = &part.layer {
        lines.push(format!("{}    @Layer(\"{}\");", pad, lyr));
    }
    for a in &part.attributes {
        lines.push(a.to_sysml(child_indent));
    }
    for p in &part.ports {
        lines.push(p.to_sysml(child_indent));
    }
    for act in &part.actions {
        lines.push(act.to_sysml(child_indent));
    }
    for con in &part.constraints {
        lines.push(con.to_sysml(child_indent));
    }
    for c in &part.calcs {
        lines.push(c.to_sysml(child_indent));
    }
    for req in &part.requirements {
        lines.push(req.to_sysml(child_indent));
    }
    for sat_req in &part.satisfied_requirements {
        lines.push(format!("{}    satisfy requirement {};", pad, sat_req));
    }
}

/// Appends non-recursive post-state leaf definitions declared directly inside `part` at `child_indent`.
/// Realises: [REQ-SEMANTIC-SERIALIZER-PARTDEF-TO-SYSML]
fn append_part_leaf_tail(part: &PartDef, child_indent: usize, lines: &mut Vec<String>) {
    for conn in &part.connections {
        lines.push(conn.to_sysml(child_indent));
    }
    for uc in &part.use_cases {
        lines.push(uc.to_sysml(child_indent));
    }
    for item in &part.item_defs {
        lines.push(item.to_sysml(child_indent));
    }
    for hz in &part.hazards {
        lines.push(hz.to_sysml(child_indent));
    }
    for rk in &part.risks {
        lines.push(rk.to_sysml(child_indent));
    }
    for tc in &part.test_cases {
        lines.push(tc.to_sysml(child_indent));
    }
    for cap in &part.capabilities {
        lines.push(cap.to_sysml(child_indent));
    }
    for op in &part.operations {
        lines.push(op.to_sysml(child_indent));
    }
    for inter in &part.interactions {
        lines.push(inter.to_sysml(child_indent));
    }
}

impl BoundedSysmlSerializable for PartDef {
    /// Serializes `PartDef`, nested `states`, and nested `parts` up to `MAX_AST_DEPTH` with saturating arithmetic.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-PARTDEF-TO-SYSML]
    fn to_sysml_bounded(&self, indent: usize, depth: usize) -> String {
        if depth >= MAX_AST_DEPTH {
            return String::new();
        }
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let kw = if self.is_def { "part def" } else { "part" };
        let type_annot = self
            .type_name
            .as_ref()
            .map(|t| format!(" : {}", t))
            .unwrap_or_default();

        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        lines.push(format!("{}{} {}{} {{", pad, kw, self.name, type_annot));

        let child_indent = indent.saturating_add(4);
        let next_depth = depth.saturating_add(1);
        append_part_primary_members(self, &pad, child_indent, &mut lines);
        if next_depth < MAX_AST_DEPTH {
            for st in &self.states {
                let rendered = st.to_sysml_bounded(child_indent, next_depth);
                if !rendered.is_empty() {
                    lines.push(rendered);
                }
            }
        }
        append_part_leaf_tail(self, child_indent, &mut lines);
        if next_depth < MAX_AST_DEPTH {
            for sub in &self.parts {
                let rendered = sub.to_sysml_bounded(child_indent, next_depth);
                if !rendered.is_empty() {
                    lines.push(rendered);
                }
            }
        }

        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for ImportDef {
    /// Serializes `ImportDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-IMPORTDEF-TO-SYSML] | SysML v2: ActionDef(ImportDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut target = self.path.clone();
        if self.is_recursive {
            if !target.ends_with("::**") {
                if target.ends_with("::*") {
                    target.push('*');
                } else {
                    target.push_str("::**");
                }
            }
        } else if self.is_wildcard && !target.ends_with("::*") && !target.ends_with("::**") {
            target.push_str("::*");
        }
        format!("{}{}import {};", doc, pad, target)
    }
}

impl SysmlSerializable for PackageDef {
    /// Serializes `PackageDef` into canonical SysML v2 notation starting at recursion depth zero.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-PACKAGEDEF-TO-SYSML]
    fn to_sysml(&self, indent: usize) -> String {
        self.to_sysml_bounded(indent, 0)
    }
}

/// Serializes an unnamed synthetic multi-package forest root while propagating `next_depth` to subpackages.
/// Realises: [REQ-SEMANTIC-SERIALIZER-PACKAGEDEF-TO-SYSML]
fn serialize_synthetic_forest_root(pkg: &PackageDef, next_depth: usize) -> String {
    let mut top_sections = Vec::new();
    let doc = format_doc_comment(&pkg.doc, 0);
    if !doc.is_empty() {
        top_sections.push(doc.trim_end().to_string());
    }
    if !pkg.imports.is_empty() {
        let import_block = pkg
            .imports
            .iter()
            .map(|imp| imp.to_sysml(0))
            .collect::<Vec<_>>()
            .join("\n");
        top_sections.push(import_block);
    }
    if next_depth < MAX_AST_DEPTH {
        for sub in &pkg.packages {
            let rendered = sub.to_sysml_bounded(0, next_depth);
            if !rendered.is_empty() {
                top_sections.push(rendered);
            }
        }
    }
    top_sections.join("\n\n")
}

/// Appends primary imports, attributes, ports, actions, constraints, calcs, and requirements of `pkg` to `lines`.
/// Realises: [REQ-SEMANTIC-SERIALIZER-PACKAGEDEF-TO-SYSML]
fn append_package_primary_members(
    pkg: &PackageDef,
    pad: &str,
    child_indent: usize,
    lines: &mut Vec<String>,
) {
    for imp in &pkg.imports {
        lines.push(imp.to_sysml(child_indent));
    }
    for a in &pkg.attribute_defs {
        lines.push(a.to_sysml(child_indent));
    }
    for p in &pkg.port_defs {
        lines.push(p.to_sysml(child_indent));
    }
    for act in &pkg.action_defs {
        lines.push(act.to_sysml(child_indent));
    }
    for con in &pkg.constraint_defs {
        lines.push(con.to_sysml(child_indent));
    }
    for c in &pkg.calc_defs {
        lines.push(c.to_sysml(child_indent));
    }
    for req in &pkg.requirement_defs {
        lines.push(req.to_sysml(child_indent));
    }
    for sat_req in &pkg.satisfied_requirements {
        lines.push(format!("{}    satisfy requirement {};", pad, sat_req));
    }
}

/// Appends non-recursive post-state leaf definitions declared directly inside `pkg` at `child_indent`.
/// Realises: [REQ-SEMANTIC-SERIALIZER-PACKAGEDEF-TO-SYSML]
fn append_package_leaf_tail(pkg: &PackageDef, child_indent: usize, lines: &mut Vec<String>) {
    for conn in &pkg.connection_defs {
        lines.push(conn.to_sysml(child_indent));
    }
    for uc in &pkg.use_case_defs {
        lines.push(uc.to_sysml(child_indent));
    }
    for it in &pkg.item_defs {
        lines.push(it.to_sysml(child_indent));
    }
    for hz in &pkg.hazard_defs {
        lines.push(hz.to_sysml(child_indent));
    }
    for rk in &pkg.risk_defs {
        lines.push(rk.to_sysml(child_indent));
    }
    for tc in &pkg.test_case_defs {
        lines.push(tc.to_sysml(child_indent));
    }
    for cap in &pkg.capability_defs {
        lines.push(cap.to_sysml(child_indent));
    }
    for op in &pkg.operation_defs {
        lines.push(op.to_sysml(child_indent));
    }
    for inter in &pkg.interaction_defs {
        lines.push(inter.to_sysml(child_indent));
    }
}

impl BoundedSysmlSerializable for PackageDef {
    /// Serializes `PackageDef` and nested `state_defs`, `part_defs`, and `packages` up to `MAX_AST_DEPTH`.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-PACKAGEDEF-TO-SYSML]
    fn to_sysml_bounded(&self, indent: usize, depth: usize) -> String {
        if depth >= MAX_AST_DEPTH {
            return String::new();
        }
        let next_depth = depth.saturating_add(1);
        if indent == 0 && crate::parser::grammar::is_synthetic_forest_root(self) {
            return serialize_synthetic_forest_root(self, next_depth);
        }

        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        lines.push(format!("{}package {} {{", pad, self.name));

        let child_indent = indent.saturating_add(4);
        append_package_primary_members(self, &pad, child_indent, &mut lines);
        if next_depth < MAX_AST_DEPTH {
            for st in &self.state_defs {
                let rendered = st.to_sysml_bounded(child_indent, next_depth);
                if !rendered.is_empty() {
                    lines.push(rendered);
                }
            }
        }
        append_package_leaf_tail(self, child_indent, &mut lines);
        if next_depth < MAX_AST_DEPTH {
            for part in &self.part_defs {
                let rendered = part.to_sysml_bounded(child_indent, next_depth);
                if !rendered.is_empty() {
                    lines.push(rendered);
                }
            }
            for sub in &self.packages {
                let rendered = sub.to_sysml_bounded(child_indent, next_depth);
                if !rendered.is_empty() {
                    lines.push(rendered);
                }
            }
        }

        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}
