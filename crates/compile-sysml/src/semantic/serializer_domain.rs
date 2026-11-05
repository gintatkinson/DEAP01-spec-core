#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! SysML v2 canonical serialization for domain safety, use-case, calculation, operation, interaction, and test-case AST nodes.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for domain AST serialization.
//! ## 2. Invariants & Mathematical Properties
//! Renders `ItemDef`, `HazardDef`, `RiskDef`, `UseCaseDef`, `CalcDef`, `CapabilityDef`, `OperationDef`, `InteractionDef`, and `TestCaseDef` deterministically.
//! ## 3. Preconditions
//! AST nodes conform to `deap_core::sysml_ast` structural invariants.
//! ## 4. Postconditions
//! Emits canonical 4-space indented SysML v2 notation with normalized string quoting and identifier formatting.
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(N) over AST element fields. Space Complexity: O(N) formatted string output.
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Total pure serialization with saturating indentation arithmetic and zero panics.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603, REQ-SEMANTIC-SERIALIZER-HAZARDDEF-TO-SYSML, REQ-SEMANTIC-SERIALIZER-USECASEDEF-TO-SYSML]

use super::serializer::{format_doc_comment, strip_enclosing_quotes, SysmlSerializable};
use deap_core::sysml_ast::{
    CalcDef, CapabilityDef, HazardDef, InteractionDef, ItemDef, OperationDef, RiskDef, TestCaseDef,
    UseCaseDef,
};

impl SysmlSerializable for ItemDef {
    /// Serializes `ItemDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-ITEMDEF-TO-SYSML] | SysML v2: ActionDef(ItemDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        lines.push(format!("{}item def {} {{", pad, self.name));
        let child_indent = indent.saturating_add(4);
        for a in &self.attributes {
            lines.push(a.to_sysml(child_indent));
        }
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for HazardDef {
    /// Serializes `HazardDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-HAZARDDEF-TO-SYSML] | SysML v2: ActionDef(HazardDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        lines.push(format!("{}hazard def {} {{", pad, self.name));
        lines.push(format!("{}    attribute severity : Integer = {};", pad, self.severity));
        if let Some(sp) = &self.source_port {
            lines.push(format!("{}    attribute source_port : String = \"{}\";", pad, strip_enclosing_quotes(sp)));
        }
        if let Some(tp) = &self.target_port {
            lines.push(format!("{}    attribute target_port : String = \"{}\";", pad, strip_enclosing_quotes(tp)));
        }
        if let Some(p) = &self.part_ref {
            lines.push(format!("{}    attribute part_ref : String = \"{}\";", pad, strip_enclosing_quotes(p)));
        }
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for RiskDef {
    /// Serializes `RiskDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-RISKDEF-TO-SYSML] | SysML v2: ActionDef(RiskDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        lines.push(format!("{}risk def {} {{", pad, self.name));
        lines.push(format!("{}    attribute severity : Integer = {};", pad, self.severity));
        if let Some(h) = &self.hazard_ref {
            lines.push(format!("{}    attribute hazard_ref : String = \"{}\";", pad, strip_enclosing_quotes(h)));
        }
        if let Some(sp) = &self.source_port {
            lines.push(format!("{}    attribute source_port : String = \"{}\";", pad, strip_enclosing_quotes(sp)));
        }
        if let Some(tp) = &self.target_port {
            lines.push(format!("{}    attribute target_port : String = \"{}\";", pad, strip_enclosing_quotes(tp)));
        }
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

/// Checks whether `value` is a valid unquoted SysML v2 simple or qualified identifier.
/// Realises: [REQ-SEMANTIC-SERIALIZER-USECASEDEF-TO-SYSML] | SysML v2: ActionDef(is_simple_sysml_identifier)
fn is_simple_sysml_identifier(value: &str) -> bool {
    let trimmed = value.trim();
    let mut chars = trimmed.chars();
    let valid_start = chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
    valid_start && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '.' | '-'))
}

/// Normalizes a `UseCaseDef` clause value into canonical quoted or identifier SysML v2 notation.
/// Realises: [REQ-SEMANTIC-SERIALIZER-USECASEDEF-TO-SYSML] | SysML v2: ActionDef(format_use_case_clause_value)
fn format_use_case_clause_value(raw: &str, always_quote: bool) -> String {
    let trimmed = raw.trim();
    let was_quoted = trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2;
    let inner = trimmed
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(trimmed)
        .trim();
    if always_quote || was_quoted || !is_simple_sysml_identifier(inner) {
        format!("\"{}\"", inner.replace('"', "\\\""))
    } else {
        inner.to_string()
    }
}

/// Appends `precondition`, `include`, `extend`, `step`, and `postcondition` clauses of `UseCaseDef` to `lines`.
/// Realises: [REQ-SEMANTIC-SERIALIZER-USECASEDEF-TO-SYSML]
fn append_use_case_relations_and_steps(uc: &UseCaseDef, pad: &str, lines: &mut Vec<String>) {
    for pre in &uc.preconditions {
        lines.push(format!("{}    precondition {};", pad, format_use_case_clause_value(pre, false)));
    }
    for inc in &uc.includes {
        let target = inc.trim().strip_prefix("use case").map(str::trim).unwrap_or(inc.trim());
        lines.push(format!("{}    include {};", pad, format_use_case_clause_value(target, false)));
    }
    for ext in &uc.extends {
        let target = ext.trim().strip_prefix("use case").map(str::trim).unwrap_or(ext.trim());
        lines.push(format!("{}    extend {};", pad, format_use_case_clause_value(target, false)));
    }
    for step in &uc.steps {
        lines.push(format!("{}    step {};", pad, format_use_case_clause_value(step, true)));
    }
    for post in &uc.postconditions {
        lines.push(format!("{}    postcondition {};", pad, format_use_case_clause_value(post, false)));
    }
}

impl SysmlSerializable for UseCaseDef {
    /// Serializes `UseCaseDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-USECASEDEF-TO-SYSML] | SysML v2: ActionDef(UseCaseDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        lines.push(format!("{}use case def {} {{", pad, self.name));
        if let Some(s) = &self.subject {
            lines.push(format!("{}    subject {};", pad, s));
        }
        if self.actors.is_empty() {
            if let Some(a) = &self.actor {
                lines.push(format!("{}    actor {};", pad, a));
            }
        } else {
            for a in &self.actors {
                lines.push(format!("{}    actor {};", pad, a));
            }
        }
        if let Some(obj) = &self.objective {
            lines.push(format!("{}    objective {};", pad, format_use_case_clause_value(obj, true)));
        }
        append_use_case_relations_and_steps(self, &pad, &mut lines);
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for CalcDef {
    /// Serializes `CalcDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-CALCDEF-TO-SYSML] | SysML v2: ActionDef(CalcDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        let ret_annot = self.return_type.as_ref().map(|r| format!(" : {}", r)).unwrap_or_default();
        lines.push(format!("{}calc def {}{} {{", pad, self.name, ret_annot));
        for p in &self.in_params {
            let def_val = p.default_value.as_ref().map(|v| format!(" = {}", v)).unwrap_or_default();
            lines.push(format!("{}    in {} : {}{};", pad, p.name, p.type_name, def_val));
        }
        if let Some(expr) = &self.expression {
            for expr_line in expr.lines() {
                let trimmed = expr_line.trim();
                if !trimmed.is_empty() {
                    lines.push(format!("{}    return {};", pad, trimmed.trim_end_matches(';')));
                }
            }
        }
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for CapabilityDef {
    /// Serializes `CapabilityDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-CAPABILITYDEF-TO-SYSML] | SysML v2: ActionDef(CapabilityDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        if let Some(sub) = &self.subsystem {
            lines.push(format!("{}capability def {} {{", pad, self.name));
            lines.push(format!("{}    subsystem {};", pad, sub));
            lines.push(format!("{}}}", pad));
        } else {
            lines.push(format!("{}capability def {};", pad, self.name));
        }
        lines.join("\n")
    }
}

impl SysmlSerializable for OperationDef {
    /// Serializes `OperationDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-OPERATIONDEF-TO-SYSML] | SysML v2: ActionDef(OperationDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let params: Vec<String> = self
            .parameters
            .iter()
            .map(|p| {
                let dir = p.default_value.as_deref().unwrap_or("in");
                format!("{} {} : {}", dir, p.name, p.type_name)
            })
            .collect();
        let param_str = if params.is_empty() { String::new() } else { format!("({})", params.join(", ")) };
        let ret_str = self.return_type.as_ref().map(|r| format!(" : {}", r)).unwrap_or_default();
        format!("{}{}operation def {}{}{};", doc, pad, self.name, param_str, ret_str)
    }
}

impl SysmlSerializable for InteractionDef {
    /// Serializes `InteractionDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-INTERACTIONDEF-TO-SYSML] | SysML v2: ActionDef(InteractionDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        lines.push(format!("{}interaction def {} {{", pad, self.name));
        for l in &self.lifelines {
            lines.push(format!("{}    lifeline {};", pad, l));
        }
        for m in &self.messages {
            lines.push(format!("{}    message {};", pad, m));
        }
        for t in &self.triggers {
            lines.push(format!("{}    trigger {};", pad, t));
        }
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for TestCaseDef {
    /// Serializes `TestCaseDef` into canonical SysML v2 notation.
    /// Realises: [REQ-SEMANTIC-SERIALIZER-TESTCASEDEF-TO-SYSML] | SysML v2: ActionDef(TestCaseDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        lines.push(format!("{}test case def {} {{", pad, self.name));
        if let Some(subj) = &self.subject_part {
            lines.push(format!("{}    subject {};", pad, subj));
        }
        if !self.verified_requirements.is_empty() {
            lines.push(format!("{}    verify requirement {};", pad, self.verified_requirements.join(", ")));
        }
        if let Some(obj) = &self.objective {
            lines.push(format!("{}    objective {};", pad, obj));
        }
        for step in &self.test_steps {
            lines.push(format!("{}    step {};", pad, step));
        }
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}
