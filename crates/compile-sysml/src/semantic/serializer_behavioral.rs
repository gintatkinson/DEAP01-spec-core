#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! SysML v2 canonical serialization for behavioral, constraint, requirement, state, and connection AST nodes.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for behavioral SysML v2 serialization.
//! ## 2. Invariants & Mathematical Properties
//! Renders `ActionDef`, `ConstraintDef`, `RequirementDef`, `StateDef`, and `ConnectionDef` deterministically with 4-space indentation.
//! ## 3. Preconditions
//! AST nodes conform to `deap_core::sysml_ast` invariants within `MAX_AST_DEPTH` (64).
//! ## 4. Postconditions
//! Emits canonical SysML v2 textual notation preserving round-trip parse equivalence (`parse(serialize(ast)) == ast`).
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(N) over AST subtree size. Space Complexity: O(N) output string buffer.
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Caps nested `StateDef` recursion at `MAX_AST_DEPTH` (64) with saturating arithmetic.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603, REQ-SEMANTIC-SERIALIZER-ACTIONDEF-TO-SYSML, REQ-SEMANTIC-SERIALIZER-STATEDEF-TO-SYSML]

use super::serializer::{
    escape_sysml_string_content, format_doc_comment, format_real_literal, strip_enclosing_quotes,
    BoundedSysmlSerializable, SysmlSerializable, MAX_AST_DEPTH,
};
use deap_core::sysml_ast::{ActionDef, ConnectionDef, ConstraintDef, RequirementDef, StateDef};

impl SysmlSerializable for ActionDef {
    /// Serializes `ActionDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-ACTIONDEF-TO-SYSML] | SysML v2: ActionDef(ActionDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut param_strs = Vec::new();
        for p in &self.in_params {
            let def_val = p
                .default_value
                .as_ref()
                .map(|v| format!(" = {}", v))
                .unwrap_or_default();
            param_strs.push(format!("in {} : {}{}", p.name, p.type_name, def_val));
        }
        for p in &self.out_params {
            let def_val = p
                .default_value
                .as_ref()
                .map(|v| format!(" = {}", v))
                .unwrap_or_default();
            param_strs.push(format!("out {} : {}{}", p.name, p.type_name, def_val));
        }
        let params_header = if param_strs.is_empty() {
            String::new()
        } else {
            format!("({})", param_strs.join(", "))
        };

        let kw = if self.is_def { "action def" } else { "action" };
        let has_body =
            self.performer.is_some() || !self.steps.is_empty() || !self.parameters.is_empty();

        if has_body {
            let mut lines = Vec::new();
            if !doc.is_empty() {
                lines.push(doc.trim_end().to_string());
            }
            lines.push(format!("{}{} {}{} {{", pad, kw, self.name, params_header));
            let child_indent = indent.saturating_add(4);
            for attr in &self.parameters {
                lines.push(attr.to_sysml(child_indent));
            }
            if let Some(perf) = &self.performer {
                lines.push(format!("{}    perform {};", pad, perf));
            }
            for s in &self.steps {
                lines.push(format!("{}    step {};", pad, s));
            }
            lines.push(format!("{}}}", pad));
            lines.join("\n")
        } else {
            format!("{}{}{} {}{};", doc, pad, kw, self.name, params_header)
        }
    }
}

impl SysmlSerializable for ConstraintDef {
    /// Serializes `ConstraintDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-CONSTRAINTDEF-TO-SYSML] | SysML v2: ActionDef(ConstraintDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let kw = if self.is_assertion {
            "assert constraint"
        } else {
            "constraint def"
        };
        if self.expression.is_empty() && self.parameters.is_empty() {
            let mut lines = Vec::new();
            if !doc.is_empty() {
                lines.push(doc.trim_end().to_string());
            }
            lines.push(format!("{}{} {};", pad, kw, self.name));
            return lines.join("\n");
        }
        let params = if self.parameters.is_empty() {
            String::new()
        } else {
            format!("({})", self.parameters.join(", "))
        };

        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        lines.push(format!("{}{} {}{} {{", pad, kw, self.name, params));
        if !self.expression.is_empty() {
            lines.push(format!("{}    {};", pad, self.expression));
        }
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for RequirementDef {
    /// Serializes `RequirementDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-REQUIREMENTDEF-TO-SYSML] | SysML v2: ActionDef(RequirementDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        lines.push(format!("{}requirement def {} {{", pad, self.name));
        if !self.req_id.is_empty() {
            lines.push(format!(
                "{}    id = \"{}\";",
                pad,
                escape_sysml_string_content(&self.req_id)
            ));
        }
        if !self.text.is_empty() {
            lines.push(format!(
                "{}    text = \"{}\";",
                pad,
                escape_sysml_string_content(&self.text)
            ));
        }
        let child_indent = indent.saturating_add(4);
        for a in &self.attributes {
            lines.push(a.to_sysml(child_indent));
        }
        for c in &self.constraints {
            lines.push(c.to_sysml(child_indent));
        }
        for a in &self.assumes {
            lines.push(format!("{}    assume {};", pad, a));
        }
        for r in &self.requires {
            lines.push(format!("{}    require {};", pad, r));
        }
        for d in &self.derived_from {
            lines.push(format!("{}    derived from {};", pad, d));
        }
        for d in &self.derives {
            lines.push(format!("{}    derives {};", pad, d));
        }
        for v in &self.verified_by {
            lines.push(format!("{}    verify by {};", pad, v));
        }
        for s in &self.satisfied_by {
            lines.push(format!("{}    satisfy by {};", pad, s));
        }
        for h in &self.triggers_hazards {
            lines.push(format!("{}    @TriggersHazard(\"{}\");", pad, h));
        }
        for sr in &self.safety_realises {
            lines.push(format!("{}    @SafetyRealises(\"{}\");", pad, sr));
        }
        if let Some(lyr) = &self.layer {
            lines.push(format!("{}    @Layer(\"{}\");", pad, lyr));
        }
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for StateDef {
    /// Serializes `StateDef` into canonical SysML v2 notation starting at recursion depth zero.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-STATEDEF-TO-SYSML]
    fn to_sysml(&self, indent: usize) -> String {
        self.to_sysml_bounded(indent, 0)
    }
}

impl BoundedSysmlSerializable for StateDef {
    /// Serializes `StateDef` and nested `sub_states` up to `MAX_AST_DEPTH` with saturating depth/indent arithmetic.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-STATEDEF-TO-SYSML]
    fn to_sysml_bounded(&self, indent: usize, depth: usize) -> String {
        if depth >= MAX_AST_DEPTH {
            return String::new();
        }
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        lines.push(format!("{}state def {} {{", pad, self.name));
        if let Some(entry) = &self.entry_action {
            lines.push(format!("{}    entry {};", pad, entry));
        }
        if let Some(do_act) = &self.do_action {
            lines.push(format!("{}    do {};", pad, do_act));
        }
        if let Some(exit) = &self.exit_action {
            lines.push(format!("{}    exit {};", pad, exit));
        }
        let child_indent = indent.saturating_add(4);
        let next_depth = depth.saturating_add(1);
        if next_depth < MAX_AST_DEPTH {
            for sub in &self.sub_states {
                if sub.entry_action.is_none()
                    && sub.do_action.is_none()
                    && sub.exit_action.is_none()
                    && sub.transitions.is_empty()
                    && sub.sub_states.is_empty()
                {
                    let sub_doc = format_doc_comment(&sub.doc, child_indent);
                    if !sub_doc.is_empty() {
                        lines.push(sub_doc.trim_end().to_string());
                    }
                    lines.push(format!("{}    state {};", pad, sub.name));
                } else {
                    let rendered = sub.to_sysml_bounded(child_indent, next_depth);
                    if !rendered.is_empty() {
                        lines.push(rendered);
                    }
                }
            }
        }
        for t in &self.transitions {
            let t_doc = format_doc_comment(&t.doc, child_indent);
            if !t_doc.is_empty() {
                lines.push(t_doc.trim_end().to_string());
            }
            let clause = crate::parser::grammar::SysmlParser::format_transition_clause(t);
            let stmt = if clause.is_empty() {
                format!("{}    transition;", pad)
            } else {
                format!("{}    transition {};", pad, clause)
            };
            lines.push(stmt);
        }
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for ConnectionDef {
    /// Serializes `ConnectionDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-CONNECTIONDEF-TO-SYSML] | SysML v2: ActionDef(ConnectionDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let kw = if self.is_flow {
            "flow def"
        } else {
            "connection def"
        };
        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        lines.push(format!("{}{} {} {{", pad, kw, self.name));
        if !self.source_port.is_empty() && !self.target_port.is_empty() {
            if self.is_flow {
                lines.push(format!(
                    "{}    flow from {} to {};",
                    pad, self.source_port, self.target_port
                ));
            } else {
                lines.push(format!(
                    "{}    connect {} to {};",
                    pad, self.source_port, self.target_port
                ));
            }
        }
        append_connection_metadata_lines(self, &pad, &mut lines);
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

/// Appends canonical `ConnectionDef` metadata attributes (`source_part`, `target_part`, `item_flow_ref`, `severity`, `protocol`, `latency_ms`) to `lines`.
/// Realises: [REQ-SEMANTIC-SERIALIZER-CONNECTIONDEF-TO-SYSML] | SysML v2: ActionDef(append_connection_metadata_lines)
fn append_connection_metadata_lines(conn: &ConnectionDef, pad: &str, lines: &mut Vec<String>) {
    let inferred_source = conn.source_port.split_once('.').map(|(owner, _)| owner);
    if conn.source_part.as_deref() != inferred_source {
        let sp = strip_enclosing_quotes(conn.source_part.as_deref().unwrap_or(""));
        lines.push(format!(
            "{}    attribute source_part : String = \"{}\";",
            pad, sp
        ));
    }
    let inferred_target = conn.target_port.split_once('.').map(|(owner, _)| owner);
    if conn.target_part.as_deref() != inferred_target {
        let tp = strip_enclosing_quotes(conn.target_part.as_deref().unwrap_or(""));
        lines.push(format!(
            "{}    attribute target_part : String = \"{}\";",
            pad, tp
        ));
    }
    if let Some(flow_ref) = &conn.item_flow_ref {
        lines.push(format!(
            "{}    attribute item_flow_ref : String = \"{}\";",
            pad,
            strip_enclosing_quotes(flow_ref)
        ));
    }
    if conn.severity != 1 {
        lines.push(format!(
            "{}    attribute severity : Integer = {};",
            pad, conn.severity
        ));
    }
    if let Some(proto) = &conn.protocol {
        lines.push(format!(
            "{}    attribute protocol : String = \"{}\";",
            pad,
            strip_enclosing_quotes(proto)
        ));
    }
    if let Some(lat) = conn.latency_ms {
        lines.push(format!(
            "{}    attribute latency_ms : Real = {};",
            pad,
            format_real_literal(lat)
        ));
    }
}
