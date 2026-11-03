#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! SysML v2 canonical serialization for behavioral, safety, use-case, calculation, operation, and test-case AST nodes (`BP-10`).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for behavioral and domain SysML v2 serialization.
//! ## 2. Invariants & Mathematical Properties
//! Renders `ActionDef`, `ConstraintDef`, `RequirementDef`, `StateDef`, `ConnectionDef`, `ItemDef`, `HazardDef`, `RiskDef`, `UseCaseDef`, `CalcDef`, `CapabilityDef`, `OperationDef`, `InteractionDef`, and `TestCaseDef` deterministically with 4-space indentation.
//! ## 3. Preconditions
//! AST nodes conform to `deap_core::sysml_cst` structural invariants within `MAX_AST_DEPTH` (64).
//! ## 4. Postconditions
//! Emits canonical SysML v2 textual notation preserving round-trip parse equivalence (`parse(serialize(ast)) == ast`).
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(N) over AST subtree size. Space Complexity: O(N) output string buffer.
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Caps nested `StateDef` recursion at `MAX_AST_DEPTH` (64) with saturating arithmetic.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603, REQ-SEMANTIC-SERIALIZER-ACTIONDEF-TO-SYSML, REQ-SEMANTIC-SERIALIZER-STATEDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]

use super::safety_serializer::{
    format_doc_comment, format_real_literal, strip_enclosing_quotes, BoundedSysmlSerializable,
    SysmlSerializable, MAX_AST_DEPTH,
};
use deap_core::sysml_cst::{
    ActionDef, CalcDef, CapabilityDef, ConnectionDef, ConstraintDef, HazardDef, InteractionDef,
    ItemDef, OperationDef, RequirementDef, RiskDef, StateDef, TestCaseDef, UseCaseDef,
};

impl SysmlSerializable for ActionDef {
    /// Serializes `ActionDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-ACTIONDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut param_strs = Vec::new();
        for p in &self.in_params {
            let def_val = p.default_value.as_ref().map(|v| format!(" = {v}")).unwrap_or_default();
            param_strs.push(format!("in {} : {}{def_val}", p.name, p.type_name));
        }
        for p in &self.out_params {
            let def_val = p.default_value.as_ref().map(|v| format!(" = {v}")).unwrap_or_default();
            param_strs.push(format!("out {} : {}{def_val}", p.name, p.type_name));
        }
        let params_header = if param_strs.is_empty() { String::new() } else { format!("({})", param_strs.join(", ")) };
        let kw = if self.is_def { "action def" } else { "action" };
        let has_body = self.performer.is_some() || !self.steps.is_empty() || !self.parameters.is_empty();
        if has_body {
            let mut lines = Vec::new();
            if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
            lines.push(format!("{pad}{kw} {}{params_header} {{", self.name));
            let child_indent = indent.saturating_add(4);
            for attr in &self.parameters { lines.push(attr.to_sysml(child_indent)); }
            if let Some(perf) = &self.performer { lines.push(format!("{pad}    perform {perf};")); }
            for s in &self.steps { lines.push(format!("{pad}    step {s};")); }
            lines.push(format!("{pad}}}"));
            lines.join("\n")
        } else {
            format!("{doc}{pad}{kw} {}{params_header};", self.name)
        }
    }
}

impl SysmlSerializable for ConstraintDef {
    /// Serializes `ConstraintDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-CONSTRAINTDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let kw = if self.is_assertion { "assert constraint" } else { "constraint def" };
        if self.expression.is_empty() && self.parameters.is_empty() {
            let mut lines = Vec::new();
            if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
            lines.push(format!("{pad}{kw} {};", self.name));
            return lines.join("\n");
        }
        let params = if self.parameters.is_empty() { String::new() } else { format!("({})", self.parameters.join(", ")) };
        let mut lines = Vec::new();
        if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
        lines.push(format!("{pad}{kw} {}{params} {{", self.name));
        if !self.expression.is_empty() { lines.push(format!("{pad}    {};", self.expression)); }
        lines.push(format!("{pad}}}"));
        lines.join("\n")
    }
}

impl SysmlSerializable for RequirementDef {
    /// Serializes `RequirementDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-REQUIREMENTDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();
        if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
        lines.push(format!("{pad}requirement def {} {{", self.name));
        if !self.req_id.is_empty() { lines.push(format!("{pad}    id = \"{}\";", self.req_id)); }
        if !self.text.is_empty() { lines.push(format!("{pad}    text = \"{}\";", self.text)); }
        let child_indent = indent.saturating_add(4);
        for a in &self.attributes { lines.push(a.to_sysml(child_indent)); }
        for c in &self.constraints { lines.push(c.to_sysml(child_indent)); }
        for a in &self.assumes { lines.push(format!("{pad}    assume {a};")); }
        for r in &self.requires { lines.push(format!("{pad}    require {r};")); }
        for d in &self.derived_from { lines.push(format!("{pad}    derived from {d};")); }
        for d in &self.derives { lines.push(format!("{pad}    derives {d};")); }
        for v in &self.verified_by { lines.push(format!("{pad}    verify by {v};")); }
        for s in &self.satisfied_by { lines.push(format!("{pad}    satisfy by {s};")); }
        for h in &self.triggers_hazards { lines.push(format!("{pad}    @TriggersHazard(\"{h}\");")); }
        for sr in &self.safety_realises { lines.push(format!("{pad}    @SafetyRealises(\"{sr}\");")); }
        if let Some(lyr) = &self.layer { lines.push(format!("{pad}    @Layer(\"{lyr}\");")); }
        lines.push(format!("{pad}}}"));
        lines.join("\n")
    }
}

impl SysmlSerializable for StateDef {
    /// Serializes `StateDef` into canonical SysML v2 notation starting at recursion depth zero. Realises: [REQ-SEMANTIC-SERIALIZER-STATEDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String { self.to_sysml_bounded(indent, 0) }
}

impl BoundedSysmlSerializable for StateDef {
    /// Serializes `StateDef` and nested `sub_states` up to `MAX_AST_DEPTH`. Realises: [REQ-SEMANTIC-SERIALIZER-STATEDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml_bounded(&self, indent: usize, depth: usize) -> String {
        if depth >= MAX_AST_DEPTH { return String::new(); }
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();
        if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
        lines.push(format!("{pad}state def {} {{", self.name));
        if let Some(entry) = &self.entry_action { lines.push(format!("{pad}    entry {entry};")); }
        if let Some(do_act) = &self.do_action { lines.push(format!("{pad}    do {do_act};")); }
        if let Some(exit) = &self.exit_action { lines.push(format!("{pad}    exit {exit};")); }
        let (child_indent, next_depth) = (indent.saturating_add(4), depth.saturating_add(1));
        if next_depth < MAX_AST_DEPTH {
            for sub in &self.sub_states {
                if sub.entry_action.is_none() && sub.do_action.is_none() && sub.exit_action.is_none() && sub.transitions.is_empty() && sub.sub_states.is_empty() {
                    let sub_doc = format_doc_comment(&sub.doc, child_indent);
                    if !sub_doc.is_empty() { lines.push(sub_doc.trim_end().to_string()); }
                    lines.push(format!("{pad}    state {};", sub.name));
                } else {
                    let rendered = sub.to_sysml_bounded(child_indent, next_depth);
                    if !rendered.is_empty() { lines.push(rendered); }
                }
            }
        }
        for t in &self.transitions {
            let t_doc = format_doc_comment(&t.doc, child_indent);
            if !t_doc.is_empty() { lines.push(t_doc.trim_end().to_string()); }
            let clause = crate::salsa_db::SysmlParser::format_transition_clause(t);
            let stmt = if clause.is_empty() { format!("{pad}    transition;") } else { format!("{pad}    transition {clause};") };
            lines.push(stmt);
        }
        lines.push(format!("{pad}}}"));
        lines.join("\n")
    }
}

impl SysmlSerializable for ConnectionDef {
    /// Serializes `ConnectionDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-CONNECTIONDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let kw = if self.is_flow { "flow def" } else { "connection def" };
        let mut lines = Vec::new();
        if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
        lines.push(format!("{pad}{kw} {} {{", self.name));
        if !self.source_port.is_empty() && !self.target_port.is_empty() {
            if self.is_flow {
                lines.push(format!("{pad}    flow from {} to {};", self.source_port, self.target_port));
            } else {
                lines.push(format!("{pad}    connect {} to {};", self.source_port, self.target_port));
            }
        }
        append_connection_metadata_lines(self, &pad, &mut lines);
        lines.push(format!("{pad}}}"));
        lines.join("\n")
    }
}

/// Appends canonical `ConnectionDef` metadata attributes to `lines`. Realises: [REQ-SEMANTIC-SERIALIZER-CONNECTIONDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
fn append_connection_metadata_lines(conn: &ConnectionDef, pad: &str, lines: &mut Vec<String>) {
    let inferred_source = conn.source_port.split_once('.').map(|(owner, _)| owner);
    if conn.source_part.as_deref() != inferred_source {
        let sp = strip_enclosing_quotes(conn.source_part.as_deref().unwrap_or(""));
        lines.push(format!("{pad}    attribute source_part : String = \"{sp}\";"));
    }
    let inferred_target = conn.target_port.split_once('.').map(|(owner, _)| owner);
    if conn.target_part.as_deref() != inferred_target {
        let tp = strip_enclosing_quotes(conn.target_part.as_deref().unwrap_or(""));
        lines.push(format!("{pad}    attribute target_part : String = \"{tp}\";"));
    }
    if let Some(flow_ref) = &conn.item_flow_ref {
        lines.push(format!("{pad}    attribute item_flow_ref : String = \"{}\";", strip_enclosing_quotes(flow_ref)));
    }
    if conn.severity != 1 { lines.push(format!("{pad}    attribute severity : Integer = {};", conn.severity)); }
    if let Some(proto) = &conn.protocol {
        lines.push(format!("{pad}    attribute protocol : String = \"{}\";", strip_enclosing_quotes(proto)));
    }
    if let Some(lat) = conn.latency_ms {
        lines.push(format!("{pad}    attribute latency_ms : Real = {};", format_real_literal(lat)));
    }
}

impl SysmlSerializable for ItemDef {
    /// Serializes `ItemDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-ITEMDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let (pad, doc, mut lines) = (" ".repeat(indent), format_doc_comment(&self.doc, indent), Vec::new());
        if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
        lines.push(format!("{pad}item def {} {{", self.name));
        let child_indent = indent.saturating_add(4);
        for a in &self.attributes { lines.push(a.to_sysml(child_indent)); }
        lines.push(format!("{pad}}}"));
        lines.join("\n")
    }
}

impl SysmlSerializable for HazardDef {
    /// Serializes `HazardDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-HAZARDDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let (pad, doc, mut lines) = (" ".repeat(indent), format_doc_comment(&self.doc, indent), Vec::new());
        if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
        lines.push(format!("{pad}hazard def {} {{", self.name));
        lines.push(format!("{pad}    attribute severity : Integer = {};", self.severity));
        if let Some(sp) = &self.source_port { lines.push(format!("{pad}    attribute source_port : String = \"{}\";", strip_enclosing_quotes(sp))); }
        if let Some(tp) = &self.target_port { lines.push(format!("{pad}    attribute target_port : String = \"{}\";", strip_enclosing_quotes(tp))); }
        if let Some(p) = &self.part_ref { lines.push(format!("{pad}    attribute part_ref : String = \"{}\";", strip_enclosing_quotes(p))); }
        lines.push(format!("{pad}}}"));
        lines.join("\n")
    }
}

impl SysmlSerializable for RiskDef {
    /// Serializes `RiskDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-RISKDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let (pad, doc, mut lines) = (" ".repeat(indent), format_doc_comment(&self.doc, indent), Vec::new());
        if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
        lines.push(format!("{pad}risk def {} {{", self.name));
        lines.push(format!("{pad}    attribute severity : Integer = {};", self.severity));
        if let Some(h) = &self.hazard_ref { lines.push(format!("{pad}    attribute hazard_ref : String = \"{}\";", strip_enclosing_quotes(h))); }
        if let Some(sp) = &self.source_port { lines.push(format!("{pad}    attribute source_port : String = \"{}\";", strip_enclosing_quotes(sp))); }
        if let Some(tp) = &self.target_port { lines.push(format!("{pad}    attribute target_port : String = \"{}\";", strip_enclosing_quotes(tp))); }
        lines.push(format!("{pad}}}"));
        lines.join("\n")
    }
}

/// Checks whether `value` is a valid unquoted SysML v2 simple or qualified identifier. Realises: [REQ-SEMANTIC-SERIALIZER-USECASEDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
fn is_simple_sysml_identifier(value: &str) -> bool {
    let mut chars = value.trim().chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '.' | '-'))
}

/// Normalizes a `UseCaseDef` clause value into canonical quoted or identifier SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-USECASEDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
fn format_use_case_clause_value(raw: &str, always_quote: bool) -> String {
    let trimmed = raw.trim();
    let was_quoted = trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2;
    let inner = trimmed.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(trimmed).trim();
    if always_quote || was_quoted || !is_simple_sysml_identifier(inner) {
        format!("\"{}\"", inner.replace('"', "\\\""))
    } else {
        inner.to_string()
    }
}

/// Appends `precondition`, `include`, `extend`, `step`, and `postcondition` clauses of `UseCaseDef` to `lines`. Realises: [REQ-SEMANTIC-SERIALIZER-USECASEDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
fn append_use_case_relations_and_steps(uc: &UseCaseDef, pad: &str, lines: &mut Vec<String>) {
    for pre in &uc.preconditions { lines.push(format!("{pad}    precondition {};", format_use_case_clause_value(pre, false))); }
    for inc in &uc.includes {
        let target = inc.trim().strip_prefix("use case").map(str::trim).unwrap_or(inc.trim());
        lines.push(format!("{pad}    include {};", format_use_case_clause_value(target, false)));
    }
    for ext in &uc.extends {
        let target = ext.trim().strip_prefix("use case").map(str::trim).unwrap_or(ext.trim());
        lines.push(format!("{pad}    extend {};", format_use_case_clause_value(target, false)));
    }
    for step in &uc.steps { lines.push(format!("{pad}    step {};", format_use_case_clause_value(step, true))); }
    for post in &uc.postconditions { lines.push(format!("{pad}    postcondition {};", format_use_case_clause_value(post, false))); }
}

impl SysmlSerializable for UseCaseDef {
    /// Serializes `UseCaseDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-USECASEDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let (pad, doc, mut lines) = (" ".repeat(indent), format_doc_comment(&self.doc, indent), Vec::new());
        if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
        lines.push(format!("{pad}use case def {} {{", self.name));
        if let Some(s) = &self.subject { lines.push(format!("{pad}    subject {s};")); }
        if self.actors.is_empty() {
            if let Some(a) = &self.actor { lines.push(format!("{pad}    actor {a};")); }
        } else {
            for a in &self.actors { lines.push(format!("{pad}    actor {a};")); }
        }
        if let Some(obj) = &self.objective { lines.push(format!("{pad}    objective {};", format_use_case_clause_value(obj, true))); }
        append_use_case_relations_and_steps(self, &pad, &mut lines);
        lines.push(format!("{pad}}}"));
        lines.join("\n")
    }
}

impl SysmlSerializable for CalcDef {
    /// Serializes `CalcDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-CALCDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let (pad, doc, mut lines) = (" ".repeat(indent), format_doc_comment(&self.doc, indent), Vec::new());
        if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
        let ret_annot = self.return_type.as_ref().map(|r| format!(" : {r}")).unwrap_or_default();
        lines.push(format!("{pad}calc def {}{ret_annot} {{", self.name));
        for p in &self.in_params {
            let def_val = p.default_value.as_ref().map(|v| format!(" = {v}")).unwrap_or_default();
            lines.push(format!("{pad}    in {} : {}{def_val};", p.name, p.type_name));
        }
        if let Some(expr) = &self.expression {
            for expr_line in expr.lines() {
                let trimmed = expr_line.trim();
                if !trimmed.is_empty() { lines.push(format!("{pad}    return {};", trimmed.trim_end_matches(';'))); }
            }
        }
        lines.push(format!("{pad}}}"));
        lines.join("\n")
    }
}

impl SysmlSerializable for CapabilityDef {
    /// Serializes `CapabilityDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-CAPABILITYDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let (pad, doc, mut lines) = (" ".repeat(indent), format_doc_comment(&self.doc, indent), Vec::new());
        if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
        if let Some(sub) = &self.subsystem {
            lines.push(format!("{pad}capability def {} {{", self.name));
            lines.push(format!("{pad}    subsystem {sub};"));
            lines.push(format!("{pad}}}"));
        } else {
            lines.push(format!("{pad}capability def {};", self.name));
        }
        lines.join("\n")
    }
}

impl SysmlSerializable for OperationDef {
    /// Serializes `OperationDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-OPERATIONDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let (pad, doc) = (" ".repeat(indent), format_doc_comment(&self.doc, indent));
        let params: Vec<String> = self.parameters.iter().map(|p| {
            let dir = p.default_value.as_deref().unwrap_or("in");
            format!("{dir} {} : {}", p.name, p.type_name)
        }).collect();
        let param_str = if params.is_empty() { String::new() } else { format!("({})", params.join(", ")) };
        let ret_str = self.return_type.as_ref().map(|r| format!(" : {r}")).unwrap_or_default();
        format!("{doc}{pad}operation def {}{param_str}{ret_str};", self.name)
    }
}

impl SysmlSerializable for InteractionDef {
    /// Serializes `InteractionDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-INTERACTIONDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let (pad, doc, mut lines) = (" ".repeat(indent), format_doc_comment(&self.doc, indent), Vec::new());
        if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
        lines.push(format!("{pad}interaction def {} {{", self.name));
        for l in &self.lifelines { lines.push(format!("{pad}    lifeline {l};")); }
        for m in &self.messages { lines.push(format!("{pad}    message {m};")); }
        for t in &self.triggers { lines.push(format!("{pad}    trigger {t};")); }
        lines.push(format!("{pad}}}"));
        lines.join("\n")
    }
}

impl SysmlSerializable for TestCaseDef {
    /// Serializes `TestCaseDef` into canonical SysML v2 notation. Realises: [REQ-SEMANTIC-SERIALIZER-TESTCASEDEF-TO-SYSML] | [FLS: fls_66m4rnbssgig]
    fn to_sysml(&self, indent: usize) -> String {
        let (pad, doc, mut lines) = (" ".repeat(indent), format_doc_comment(&self.doc, indent), Vec::new());
        if !doc.is_empty() { lines.push(doc.trim_end().to_string()); }
        lines.push(format!("{pad}test case def {} {{", self.name));
        if let Some(subj) = &self.subject_part { lines.push(format!("{pad}    subject {subj};")); }
        if !self.verified_requirements.is_empty() { lines.push(format!("{pad}    verify requirement {};", self.verified_requirements.join(", "))); }
        if let Some(obj) = &self.objective { lines.push(format!("{pad}    objective {obj};")); }
        for step in &self.test_steps { lines.push(format!("{pad}    step {step};")); }
        lines.push(format!("{pad}}}"));
        lines.join("\n")
    }
}
