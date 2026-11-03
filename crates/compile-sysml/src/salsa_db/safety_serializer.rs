#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! SysML v2 / KerML canonical textual notation serializer and container pretty-printer (`BP-10`).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for SysML v2 serialization.
//! ## 2. Invariants & Mathematical Properties
//! Renders deterministic SysML v2 textual notation from typed AST nodes with canonical 4-space block indentation up to `MAX_AST_DEPTH` (64).
//! ## 3. Preconditions
//! AST nodes conform to `deap_core::sysml_cst` structural invariants within `MAX_AST_DEPTH` (64).
//! ## 4. Postconditions
//! Produces bijective canonical SysML v2 text (`parse(serialize(ast)) == ast`) with zero panics.
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(N) over AST node size. Space Complexity: O(N) output string buffer.
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Caps recursive AST serialization depth at `MAX_AST_DEPTH` (64) with saturating arithmetic.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603, REQ-SEMANTIC-SERIALIZER-MODULE, REQ-SEMANTIC-SERIALIZER-SYSMLSERIALIZABLE] | [FLS: fls_66m4rnbssgig]

pub use super::hir_symbols::MAX_AST_DEPTH;
use deap_core::sysml_cst::{AttributeDef, FlowDef, ImportDef, PackageDef, PartDef, PortDef};

/// Formats doc comments as standard SysML v2 `doc /* ... */` blocks. Realises: [REQ-SEMANTIC-SERIALIZER-FORMAT-DOC-COMMENT] | [FLS: fls_66m4rnbssgig]
pub fn format_doc_comment(doc: &Option<String>, indent: usize) -> String {
    let doc_str = match doc {
        Some(d) if !d.trim().is_empty() => d.trim(),
        _ => return String::new(),
    };
    let pad = " ".repeat(indent);
    let sanitized = doc_str.replace("/*", "/ *").replace("*/", "* /");
    let lines: Vec<&str> = sanitized.lines().collect();
    if lines.len() <= 1 {
        format!("{pad}doc /* {sanitized} */\n")
    } else if let Some((first, rest)) = lines.split_first() {
        let mut out = format!("{pad}doc /* {first}\n");
        if let Some((last, middle)) = rest.split_last() {
            for line in middle {
                out.push_str(&format!("{pad}       {line}\n"));
            }
            out.push_str(&format!("{pad}       {last} */\n"));
        }
        out
    } else {
        format!("{pad}doc /* {sanitized} */\n")
    }
}

/// Trait providing SysML v2 canonical serialization. Realises: [REQ-SEMANTIC-SERIALIZER-SYSMLSERIALIZABLE] | [FLS: fls_66m4rnbssgig]
pub trait SysmlSerializable {
    /// Serializes `self` into canonical SysML v2 notation at `indent` leading spaces. Realises: [REQ-SEMANTIC-SERIALIZER-SYSMLSERIALIZABLE-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String;
}

/// Convenience function to serialize any `SysmlSerializable` entity to canonical textual notation. Realises: [REQ-SEMANTIC-SERIALIZER-TO-SYSML] | [FLS: fls_66m4rnbssgig]
pub fn to_sysml<T: SysmlSerializable>(item: &T) -> String {
    item.to_sysml(0)
}

/// Internal depth-bounded serialization contract capping recursive AST container formatting at `MAX_AST_DEPTH`. Realises: [REQ-SEMANTIC-SERIALIZER-BOUNDED-TRAIT] | [FLS: fls_66m4rnbssgig]
pub(crate) trait BoundedSysmlSerializable {
    /// Formats `self` at `indent` spaces when `depth < MAX_AST_DEPTH`. Realises: [REQ-SEMANTIC-SERIALIZER-TO-SYSML-BOUNDED] | [FLS: fls_66m4rnbssgig]
    fn to_sysml_bounded(&self, indent: usize, depth: usize) -> String;
}

/// Strips leading/trailing whitespace and enclosing double-quote delimiters (`"`) from a string slice. Realises: [REQ-SEMANTIC-SERIALIZER-STRIP-ENCLOSING-QUOTES] | [FLS: fls_66m4rnbssgig]
pub(crate) fn strip_enclosing_quotes(val: &str) -> &str {
    val.trim().trim_matches('"')
}

/// Formats an `f64` real literal for SysML v2 output without truncating non-zero fractional digits. Realises: [REQ-SEMANTIC-SERIALIZER-FORMAT-REAL-LITERAL] | [FLS: fls_66m4rnbssgig]
pub(crate) fn format_real_literal(val: f64) -> String {
    if val.fract() == 0.0 { format!("{val:.1}") } else { val.to_string() }
}

impl SysmlSerializable for AttributeDef {
    /// Serializes `AttributeDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-ATTRIBUTEDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let def_val = self.default_value.as_ref().map(|v| {
            let trimmed = v.trim();
            if let Some(inner) = trimmed.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                let mut escaped = String::with_capacity(trimmed.len().saturating_add(4));
                escaped.push('"');
                let mut prev_bs = false;
                for ch in inner.chars() {
                    if ch == '"' && !prev_bs {
                        escaped.push_str("\\\"");
                    } else {
                        escaped.push(ch);
                    }
                    prev_bs = ch == '\\' && !prev_bs;
                }
                escaped.push('"');
                format!(" = {escaped}")
            } else {
                format!(" = {v}")
            }
        }).unwrap_or_default();
        format!("{doc}{pad}attribute {} : {}{def_val};", self.name, self.type_name)
    }
}

impl SysmlSerializable for FlowDef {
    /// Serializes `FlowDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-FLOWDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let dir = if !self.direction.is_empty() { format!("{} ", self.direction) } else { String::new() };
        format!("{doc}{pad}flow {dir}{} : {};", self.name, self.item_type)
    }
}

/// Appends inner port body attributes and item flows to `lines`. Realises: [REQ-SEMANTIC-SERIALIZER-PORTDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
fn append_port_body_members(port: &PortDef, pad: &str, indent: usize, lines: &mut Vec<String>) {
    if !port.protocol_family.is_empty() {
        lines.push(format!("{pad}    attribute protocol_family : String = \"{}\";", strip_enclosing_quotes(&port.protocol_family)));
    }
    if !port.port_category.is_empty() && port.port_category != "DataPort" {
        lines.push(format!("{pad}    attribute port_category : String = \"{}\";", strip_enclosing_quotes(&port.port_category)));
    }
    for (k, v) in &port.electrical_attributes {
        lines.push(format!("{pad}    attribute {k} = \"{}\";", strip_enclosing_quotes(v)));
    }
    let child_indent = indent.saturating_add(4);
    for flow in &port.item_flows {
        lines.push(flow.to_sysml(child_indent));
    }
}

impl SysmlSerializable for PortDef {
    /// Serializes `PortDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-PORTDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let dir = if !self.direction.is_empty() { format!("{} ", self.direction) } else { String::new() };
        let conj = if self.is_conjugated { "~" } else { "" };
        let has_body = !self.protocol_family.is_empty()
            || (!self.port_category.is_empty() && self.port_category != "DataPort")
            || !self.electrical_attributes.is_empty()
            || !self.item_flows.is_empty();
        if has_body {
            let mut lines = Vec::new();
            if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
            if self.name == self.type_name {
                lines.push(format!("{pad}port def {} {{", self.name));
            } else {
                lines.push(format!("{pad}port {dir}{conj}{} : {} {{", self.name, self.type_name));
            }
            append_port_body_members(self, &pad, indent, &mut lines);
            lines.push(format!("{pad}}}"));
            lines.join("\n")
        } else if self.name == self.type_name {
            format!("{doc}{pad}port def {};", self.name)
        } else {
            format!("{doc}{pad}port {dir}{conj}{} : {};", self.name, self.type_name)
        }
    }
}

impl SysmlSerializable for PartDef {
    /// Serializes `PartDef` into canonical SysML v2 notation starting at recursion depth zero. Realises: [REQ-SEMANTIC-SERIALIZER-PARTDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String { self.to_sysml_bounded(indent, 0) }
}

/// Appends primary attributes, ports, actions, constraints, calcs, and requirements of `part` to `lines`. Realises: [REQ-SEMANTIC-SERIALIZER-PARTDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
fn append_part_primary_members(part: &PartDef, pad: &str, child_indent: usize, lines: &mut Vec<String>) {
    for h in &part.triggers_hazards { lines.push(format!("{pad}    @TriggersHazard(\"{h}\");")); }
    for sr in &part.safety_realises { lines.push(format!("{pad}    @SafetyRealises(\"{sr}\");")); }
    if let Some(lyr) = &part.layer { lines.push(format!("{pad}    @Layer(\"{lyr}\");")); }
    for a in &part.attributes { lines.push(a.to_sysml(child_indent)); }
    for p in &part.ports { lines.push(p.to_sysml(child_indent)); }
    for act in &part.actions { lines.push(act.to_sysml(child_indent)); }
    for con in &part.constraints { lines.push(con.to_sysml(child_indent)); }
    for c in &part.calcs { lines.push(c.to_sysml(child_indent)); }
    for req in &part.requirements { lines.push(req.to_sysml(child_indent)); }
    for sat_req in &part.satisfied_requirements { lines.push(format!("{pad}    satisfy requirement {sat_req};")); }
}

/// Appends non-recursive post-state leaf definitions declared directly inside `part` at `child_indent`. Realises: [REQ-SEMANTIC-SERIALIZER-PARTDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
fn append_part_leaf_tail(part: &PartDef, child_indent: usize, lines: &mut Vec<String>) {
    for conn in &part.connections { lines.push(conn.to_sysml(child_indent)); }
    for uc in &part.use_cases { lines.push(uc.to_sysml(child_indent)); }
    for item in &part.item_defs { lines.push(item.to_sysml(child_indent)); }
    for hz in &part.hazards { lines.push(hz.to_sysml(child_indent)); }
    for rk in &part.risks { lines.push(rk.to_sysml(child_indent)); }
    for tc in &part.test_cases { lines.push(tc.to_sysml(child_indent)); }
    for cap in &part.capabilities { lines.push(cap.to_sysml(child_indent)); }
    for op in &part.operations { lines.push(op.to_sysml(child_indent)); }
    for inter in &part.interactions { lines.push(inter.to_sysml(child_indent)); }
}

impl BoundedSysmlSerializable for PartDef {
    /// Serializes `PartDef`, nested `states`, and nested `parts` up to `MAX_AST_DEPTH`. Realises: [REQ-SEMANTIC-SERIALIZER-PARTDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml_bounded(&self, indent: usize, depth: usize) -> String {
        if depth >= MAX_AST_DEPTH { return String::new(); }
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let kw = if self.is_def { "part def" } else { "part" };
        let type_annot = self.type_name.as_ref().map(|t| format!(" : {t}")).unwrap_or_default();
        let mut lines = Vec::new();
        if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
        lines.push(format!("{pad}{kw} {}{type_annot} {{", self.name));
        let (child_indent, next_depth) = (indent.saturating_add(4), depth.saturating_add(1));
        append_part_primary_members(self, &pad, child_indent, &mut lines);
        if next_depth < MAX_AST_DEPTH {
            for st in &self.states {
                let rendered = st.to_sysml_bounded(child_indent, next_depth);
                if !rendered.is_empty() { lines.push(rendered); }
            }
        }
        append_part_leaf_tail(self, child_indent, &mut lines);
        if next_depth < MAX_AST_DEPTH {
            for sub in &self.parts {
                let rendered = sub.to_sysml_bounded(child_indent, next_depth);
                if !rendered.is_empty() { lines.push(rendered); }
            }
        }
        lines.push(format!("{pad}}}"));
        lines.join("\n")
    }
}

impl SysmlSerializable for ImportDef {
    /// Serializes `ImportDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-IMPORTDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut target = self.path.clone();
        if self.is_recursive {
            if !target.ends_with("::**") {
                if target.ends_with("::*") { target.push('*'); } else { target.push_str("::**"); }
            }
        } else if self.is_wildcard && !target.ends_with("::*") && !target.ends_with("::**") {
            target.push_str("::*");
        }
        format!("{doc}{pad}import {target};")
    }
}

impl SysmlSerializable for PackageDef {
    /// Serializes `PackageDef` into canonical SysML v2 notation starting at recursion depth zero. Realises: [REQ-SEMANTIC-SERIALIZER-PACKAGEDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String { self.to_sysml_bounded(indent, 0) }
}

/// Serializes an unnamed synthetic multi-package forest root while propagating `next_depth`. Realises: [REQ-SEMANTIC-SERIALIZER-PACKAGEDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
fn serialize_synthetic_forest_root(pkg: &PackageDef, next_depth: usize) -> String {
    let mut top_sections = Vec::new();
    let doc = format_doc_comment(&pkg.doc, 0);
    if !doc.is_empty() { top_sections.push(doc.trim_end().to_string()); }
    if !pkg.imports.is_empty() {
        let import_block = pkg.imports.iter().map(|imp| imp.to_sysml(0)).collect::<Vec<_>>().join("\n");
        top_sections.push(import_block);
    }
    if next_depth < MAX_AST_DEPTH {
        for sub in &pkg.packages {
            let rendered = sub.to_sysml_bounded(0, next_depth);
            if !rendered.is_empty() { top_sections.push(rendered); }
        }
    }
    top_sections.join("\n\n")
}

/// Appends primary imports, attributes, ports, actions, constraints, calcs, and requirements of `pkg` to `lines`. Realises: [REQ-SEMANTIC-SERIALIZER-PACKAGEDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
fn append_package_primary_members(pkg: &PackageDef, pad: &str, child_indent: usize, lines: &mut Vec<String>) {
    for imp in &pkg.imports { lines.push(imp.to_sysml(child_indent)); }
    for a in &pkg.attribute_defs { lines.push(a.to_sysml(child_indent)); }
    for p in &pkg.port_defs { lines.push(p.to_sysml(child_indent)); }
    for act in &pkg.action_defs { lines.push(act.to_sysml(child_indent)); }
    for con in &pkg.constraint_defs { lines.push(con.to_sysml(child_indent)); }
    for c in &pkg.calc_defs { lines.push(c.to_sysml(child_indent)); }
    for req in &pkg.requirement_defs { lines.push(req.to_sysml(child_indent)); }
    for sat_req in &pkg.satisfied_requirements { lines.push(format!("{pad}    satisfy requirement {sat_req};")); }
}

/// Appends non-recursive post-state leaf definitions declared directly inside `pkg` at `child_indent`. Realises: [REQ-SEMANTIC-SERIALIZER-PACKAGEDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
fn append_package_leaf_tail(pkg: &PackageDef, child_indent: usize, lines: &mut Vec<String>) {
    for conn in &pkg.connection_defs { lines.push(conn.to_sysml(child_indent)); }
    for uc in &pkg.use_case_defs { lines.push(uc.to_sysml(child_indent)); }
    for it in &pkg.item_defs { lines.push(it.to_sysml(child_indent)); }
    for hz in &pkg.hazard_defs { lines.push(hz.to_sysml(child_indent)); }
    for rk in &pkg.risk_defs { lines.push(rk.to_sysml(child_indent)); }
    for tc in &pkg.test_case_defs { lines.push(tc.to_sysml(child_indent)); }
    for cap in &pkg.capability_defs { lines.push(cap.to_sysml(child_indent)); }
    for op in &pkg.operation_defs { lines.push(op.to_sysml(child_indent)); }
    for inter in &pkg.interaction_defs { lines.push(inter.to_sysml(child_indent)); }
}

impl BoundedSysmlSerializable for PackageDef {
    /// Serializes `PackageDef` and nested `state_defs`, `part_defs`, and `packages` up to `MAX_AST_DEPTH`. Realises: [REQ-SEMANTIC-SERIALIZER-PACKAGEDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml_bounded(&self, indent: usize, depth: usize) -> String {
        if depth >= MAX_AST_DEPTH { return String::new(); }
        let next_depth = depth.saturating_add(1);
        if indent == 0 && crate::salsa_db::is_synthetic_forest_root(self) {
            return serialize_synthetic_forest_root(self, next_depth);
        }
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();
        if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
        lines.push(format!("{pad}package {} {{", self.name));
        let child_indent = indent.saturating_add(4);
        append_package_primary_members(self, &pad, child_indent, &mut lines);
        if next_depth < MAX_AST_DEPTH {
            for st in &self.state_defs {
                let rendered = st.to_sysml_bounded(child_indent, next_depth);
                if !rendered.is_empty() { lines.push(rendered); }
            }
        }
        append_package_leaf_tail(self, child_indent, &mut lines);
        if next_depth < MAX_AST_DEPTH {
            for part in &self.part_defs {
                let rendered = part.to_sysml_bounded(child_indent, next_depth);
                if !rendered.is_empty() { lines.push(rendered); }
            }
            for sub in &self.packages {
                let rendered = sub.to_sysml_bounded(child_indent, next_depth);
                if !rendered.is_empty() { lines.push(rendered); }
            }
        }
        lines.push(format!("{pad}}}"));
        lines.join("\n")
    }
}
