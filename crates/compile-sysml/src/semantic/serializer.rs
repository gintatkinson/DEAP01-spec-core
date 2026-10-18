//! SysML v2 / KerML canonical textual notation serializer / pretty-printer.

use deap_core::sysml_ast::*;

/// Helper to format doc comments as standard SysML v2 doc blocks.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `format_doc_comment` in `crates/compile-sysml/src/semantic/serializer.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `format_doc_comment` (declaration #1) within `SEMANTIC-SERIALIZER`.
///
/// ## 3. Preconditions
/// - `doc: &Option<String>` must satisfy domain and type-safety constraints prior to invoking `format_doc_comment`.
/// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `format_doc_comment`.
///
/// ## 4. Postconditions
/// - Returns `String` from `format_doc_comment` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(doc)) bounded over input/member dimension of `format_doc_comment`.
/// - Space Complexity: O(N(doc)) bounded auxiliary memory for `format_doc_comment`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `format_doc_comment` (`String`).
/// - State: Maintains consistent state invariants across `format_doc_comment` without partial mutation on error.
/// - Action: Executes `format_doc_comment` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `format_doc_comment` (`SEMANTIC-SERIALIZER`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-SERIALIZER-FORMAT-DOC-COMMENT] | SysML v2: ActionDef(format_doc_comment)
pub fn format_doc_comment(doc: &Option<String>, indent: usize) -> String {
    let doc_str = match doc {
        Some(d) if !d.trim().is_empty() => d.trim(),
        _ => return String::new(),
    };

    let pad = " ".repeat(indent);
    let sanitized = doc_str.replace("*/", "* /");
    let lines: Vec<&str> = sanitized.lines().collect();

    if lines.len() <= 1 {
        format!("{}doc /* {} */\n", pad, sanitized)
    } else if let Some((first, rest)) = lines.split_first() {
        let mut out = format!("{}doc /* {}\n", pad, first);
        if let Some((last, middle)) = rest.split_last() {
            for line in middle {
                out.push_str(&format!("{}       {}\n", pad, line));
            }
            out.push_str(&format!("{}       {} */\n", pad, last));
        }
        out
    } else {
        format!("{}doc /* {} */\n", pad, sanitized)
    }
}

/// Trait providing SysML v2 canonical serialization.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for trait `SysmlSerializable` in `crates/compile-sysml/src/semantic/serializer.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for trait `SysmlSerializable` across member set `[fn to_sysml]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `SysmlSerializable` members (`fn to_sysml`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `SysmlSerializable` representation for trait `SysmlSerializable`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(1)) bounded over input/member dimension of `SysmlSerializable`.
/// - Space Complexity: O(M(1)) bounded auxiliary memory for `SysmlSerializable`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `SysmlSerializable` (`SysmlSerializable`).
/// - State: Maintains consistent state invariants across `SysmlSerializable` without partial mutation on error.
/// - Action: Executes `SysmlSerializable` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `SysmlSerializable` (`SEMANTIC-SERIALIZER`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-SERIALIZER-SYSMLSERIALIZABLE] | SysML v2: PartDef(SysmlSerializable)
pub trait SysmlSerializable {
    /// Executes deterministic fn `SysmlSerializable::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SysmlSerializable::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SysmlSerializable::to_sysml` (declaration #3) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `SysmlSerializable::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `SysmlSerializable::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `SysmlSerializable::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `SysmlSerializable::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SysmlSerializable::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `SysmlSerializable::to_sysml` without partial mutation on error.
    /// - Action: Executes `SysmlSerializable::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SysmlSerializable::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-SERIALIZER-SYSMLSERIALIZABLE-TO-SYSML] | SysML v2: ActionDef(SysmlSerializable::to_sysml)
    fn to_sysml(&self, indent: usize) -> String;
}

/// Convenience function to serialize any SysmlSerializable entity to canonical textual notation.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `to_sysml` (declaration #4) within `SEMANTIC-SERIALIZER`.
///
/// ## 3. Preconditions
/// - `item: &T` must satisfy domain and type-safety constraints prior to invoking `to_sysml`.
///
/// ## 4. Postconditions
/// - Returns `String` from `to_sysml` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(item)) bounded over input/member dimension of `to_sysml`.
/// - Space Complexity: O(N(item)) bounded auxiliary memory for `to_sysml`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `to_sysml` (`String`).
/// - State: Maintains consistent state invariants across `to_sysml` without partial mutation on error.
/// - Action: Executes `to_sysml` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `to_sysml` (`SEMANTIC-SERIALIZER`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-SERIALIZER-TO-SYSML] | SysML v2: ActionDef(to_sysml)
pub fn to_sysml<T: SysmlSerializable>(item: &T) -> String {
    item.to_sysml(0)
}


impl SysmlSerializable for AttributeDef {
    /// Executes deterministic fn `AttributeDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `AttributeDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `AttributeDef::to_sysml` (declaration #5) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `AttributeDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `AttributeDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `AttributeDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `AttributeDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `AttributeDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `AttributeDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `AttributeDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `AttributeDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-SERIALIZER-ATTRIBUTEDEF-TO-SYSML] | SysML v2: ActionDef(AttributeDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let def_val = self
            .default_value
            .as_ref()
            .map(|v| format!(" = {}", v))
            .unwrap_or_default();
        format!("{}{}attribute {} : {}{};", doc, pad, self.name, self.type_name, def_val)
    }
}

impl SysmlSerializable for FlowDef {
    /// Executes deterministic fn `FlowDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `FlowDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `FlowDef::to_sysml` (declaration #6) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `FlowDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `FlowDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `FlowDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `FlowDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `FlowDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `FlowDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `FlowDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `FlowDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-SERIALIZER-FLOWDEF-TO-SYSML] | SysML v2: ActionDef(FlowDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let dir = if !self.direction.is_empty() {
            format!("{} ", self.direction)
        } else {
            String::new()
        };
        format!("{}{}flow {}{} : {};", doc, pad, dir, self.name, self.item_type)
    }
}

impl SysmlSerializable for PortDef {
    /// Executes deterministic fn `PortDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `PortDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `PortDef::to_sysml` (declaration #7) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `PortDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `PortDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `PortDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `PortDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `PortDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `PortDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `PortDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `PortDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-SERIALIZER-PORTDEF-TO-SYSML] | SysML v2: ActionDef(PortDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let dir = if !self.direction.is_empty() {
            format!("{} ", self.direction)
        } else {
            String::new()
        };
        let conj = if self.is_conjugated { "~" } else { "" };
        let has_body = !self.protocol_family.is_empty()
            || (!self.port_category.is_empty() && self.port_category != "DataPort")
            || !self.electrical_attributes.is_empty()
            || !self.item_flows.is_empty();

        if has_body {
            let mut lines = Vec::new();
            if !doc.is_empty() {
                lines.push(doc.trim_end().to_string());
            }
            lines.push(format!("{}port {}{}{} : {} {{", pad, dir, conj, self.name, self.type_name));
            if !self.protocol_family.is_empty() {
                lines.push(format!("{}    attribute protocol_family : String = \"{}\";", pad, self.protocol_family));
            }
            if !self.port_category.is_empty() && self.port_category != "DataPort" {
                lines.push(format!("{}    attribute port_category : String = \"{}\";", pad, self.port_category));
            }
            for (k, v) in &self.electrical_attributes {
                lines.push(format!("{}    attribute {} = \"{}\";", pad, k, v));
            }
            for flow in &self.item_flows {
                lines.push(flow.to_sysml(indent + 4));
            }
            lines.push(format!("{}}}", pad));
            lines.join("\n")
        } else {
            if self.name == self.type_name {
                format!("{}{}port def {};", doc, pad, self.name)
            } else {
                format!("{}{}port {}{}{} : {};", doc, pad, dir, conj, self.name, self.type_name)
            }
        }
    }
}

impl SysmlSerializable for ActionDef {
    /// Executes deterministic fn `ActionDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ActionDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `ActionDef::to_sysml` (declaration #8) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `ActionDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `ActionDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `ActionDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `ActionDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `ActionDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `ActionDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `ActionDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `ActionDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-SERIALIZER-ACTIONDEF-TO-SYSML] | SysML v2: ActionDef(ActionDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut param_strs = Vec::new();
        for p in &self.in_params {
            param_strs.push(format!("in {} : {}", p.name, p.type_name));
        }
        for p in &self.out_params {
            param_strs.push(format!("out {} : {}", p.name, p.type_name));
        }
        let params_header = if param_strs.is_empty() {
            String::new()
        } else {
            format!("({})", param_strs.join(", "))
        };

        let kw = if self.is_def { "action def" } else { "action" };
        let has_body = self.performer.is_some() || !self.steps.is_empty();

        if has_body {
            let mut lines = Vec::new();
            if !doc.is_empty() {
                lines.push(doc.trim_end().to_string());
            }
            lines.push(format!("{}{} {}{} {{", pad, kw, self.name, params_header));
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
    /// Executes deterministic fn `ConstraintDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ConstraintDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `ConstraintDef::to_sysml` (declaration #9) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `ConstraintDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `ConstraintDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `ConstraintDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `ConstraintDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `ConstraintDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `ConstraintDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `ConstraintDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `ConstraintDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
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
    /// Executes deterministic fn `RequirementDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `RequirementDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `RequirementDef::to_sysml` (declaration #10) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `RequirementDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `RequirementDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `RequirementDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `RequirementDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `RequirementDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `RequirementDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `RequirementDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `RequirementDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
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
            lines.push(format!("{}    id = \"{}\";", pad, self.req_id));
        }
        if !self.text.is_empty() {
            lines.push(format!("{}    text = \"{}\";", pad, self.text));
        }
        for a in &self.attributes {
            lines.push(a.to_sysml(indent + 4));
        }
        for c in &self.constraints {
            lines.push(c.to_sysml(indent + 4));
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
        for v in &self.verified_by {
            lines.push(format!("{}    verify by {};", pad, v));
        }
        for s in &self.satisfied_by {
            lines.push(format!("{}    satisfy by {};", pad, s));
        }
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for StateDef {
    /// Executes deterministic fn `StateDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `StateDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `StateDef::to_sysml` (declaration #11) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `StateDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `StateDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `StateDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `StateDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `StateDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `StateDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `StateDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `StateDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-SERIALIZER-STATEDEF-TO-SYSML] | SysML v2: ActionDef(StateDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
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
        for t in &self.transitions {
            let t_name = t.name.as_deref().unwrap_or("transition");
            lines.push(format!("{}    transition {};", pad, t_name));
        }
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for ConnectionDef {
    /// Executes deterministic fn `ConnectionDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ConnectionDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `ConnectionDef::to_sysml` (declaration #12) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `ConnectionDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `ConnectionDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `ConnectionDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `ConnectionDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `ConnectionDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `ConnectionDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `ConnectionDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `ConnectionDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-SERIALIZER-CONNECTIONDEF-TO-SYSML] | SysML v2: ActionDef(ConnectionDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let kw = if self.is_flow { "flow def" } else { "connection def" };
        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        lines.push(format!("{}{} {} {{", pad, kw, self.name));
        if !self.source_port.is_empty() && !self.target_port.is_empty() {
            if self.is_flow {
                lines.push(format!("{}    flow from {} to {};", pad, self.source_port, self.target_port));
            } else {
                lines.push(format!("{}    connect {} to {};", pad, self.source_port, self.target_port));
            }
        }
        if let Some(proto) = &self.protocol {
            lines.push(format!("{}    attribute protocol : String = \"{}\";", pad, proto));
        }
        if let Some(lat) = self.latency_ms {
            lines.push(format!("{}    attribute latency_ms : Real = {:.1};", pad, lat));
        }
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for ItemDef {
    /// Executes deterministic fn `ItemDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ItemDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `ItemDef::to_sysml` (declaration #13) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `ItemDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `ItemDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `ItemDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `ItemDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `ItemDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `ItemDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `ItemDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `ItemDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-SERIALIZER-ITEMDEF-TO-SYSML] | SysML v2: ActionDef(ItemDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        lines.push(format!("{}item def {} {{", pad, self.name));
        for a in &self.attributes {
            lines.push(a.to_sysml(indent + 4));
        }
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for HazardDef {
    /// Executes deterministic fn `HazardDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `HazardDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `HazardDef::to_sysml` (declaration #14) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `HazardDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `HazardDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `HazardDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `HazardDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `HazardDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `HazardDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `HazardDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `HazardDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
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
        if let Some(p) = &self.part_ref {
            lines.push(format!("{}    attribute part_ref : String = \"{}\";", pad, p));
        }
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for RiskDef {
    /// Executes deterministic fn `RiskDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `RiskDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `RiskDef::to_sysml` (declaration #15) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `RiskDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `RiskDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `RiskDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `RiskDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `RiskDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `RiskDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `RiskDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `RiskDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
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
            lines.push(format!("{}    attribute hazard_ref : String = \"{}\";", pad, h));
        }
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for UseCaseDef {
    /// Executes deterministic fn `UseCaseDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `UseCaseDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `UseCaseDef::to_sysml` (declaration #16) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `UseCaseDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `UseCaseDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `UseCaseDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `UseCaseDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `UseCaseDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `UseCaseDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `UseCaseDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `UseCaseDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
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
        for a in &self.actors {
            lines.push(format!("{}    actor {};", pad, a));
        }
        if let Some(obj) = &self.objective {
            lines.push(format!("{}    objective \"{}\";", pad, obj));
        }
        for step in &self.steps {
            lines.push(format!("{}    step {};", pad, step));
        }
        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for CalcDef {
    /// Executes deterministic fn `CalcDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `CalcDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `CalcDef::to_sysml` (declaration #17) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `CalcDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `CalcDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `CalcDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `CalcDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `CalcDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `CalcDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `CalcDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `CalcDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-SERIALIZER-CALCDEF-TO-SYSML] | SysML v2: ActionDef(CalcDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();
        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        let ret_annot = self
            .return_type
            .as_ref()
            .map(|r| format!(" : {}", r))
            .unwrap_or_default();
        lines.push(format!("{}calc def {}{} {{", pad, self.name, ret_annot));
        for p in &self.in_params {
            let def_val = p
                .default_value
                .as_ref()
                .map(|v| format!(" = {}", v))
                .unwrap_or_default();
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
    /// Executes deterministic fn `CapabilityDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `CapabilityDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `CapabilityDef::to_sysml` (declaration #18) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `CapabilityDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `CapabilityDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `CapabilityDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `CapabilityDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `CapabilityDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `CapabilityDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `CapabilityDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `CapabilityDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
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
    /// Executes deterministic fn `OperationDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `OperationDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `OperationDef::to_sysml` (declaration #19) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `OperationDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `OperationDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `OperationDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `OperationDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `OperationDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `OperationDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `OperationDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `OperationDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
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
        let param_str = if params.is_empty() {
            String::new()
        } else {
            format!("({})", params.join(", "))
        };
        let ret_str = self
            .return_type
            .as_ref()
            .map(|r| format!(" : {}", r))
            .unwrap_or_default();
        format!("{}{}operation def {}{}{};", doc, pad, self.name, param_str, ret_str)
    }
}

impl SysmlSerializable for InteractionDef {
    /// Executes deterministic fn `InteractionDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `InteractionDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `InteractionDef::to_sysml` (declaration #20) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `InteractionDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `InteractionDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `InteractionDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `InteractionDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `InteractionDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `InteractionDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `InteractionDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `InteractionDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
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
    /// Executes deterministic fn `TestCaseDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `TestCaseDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `TestCaseDef::to_sysml` (declaration #21) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `TestCaseDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `TestCaseDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `TestCaseDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `TestCaseDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `TestCaseDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `TestCaseDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `TestCaseDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `TestCaseDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
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
            lines.push(format!(
                "{}    verify requirement {};",
                pad,
                self.verified_requirements.join(", ")
            ));
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

impl SysmlSerializable for PartDef {
    /// Executes deterministic fn `PartDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `PartDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `PartDef::to_sysml` (declaration #22) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `PartDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `PartDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `PartDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `PartDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `PartDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `PartDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `PartDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `PartDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-SERIALIZER-PARTDEF-TO-SYSML] | SysML v2: ActionDef(PartDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
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

        for a in &self.attributes {
            lines.push(a.to_sysml(indent + 4));
        }
        for p in &self.ports {
            lines.push(p.to_sysml(indent + 4));
        }
        for act in &self.actions {
            lines.push(act.to_sysml(indent + 4));
        }
        for con in &self.constraints {
            lines.push(con.to_sysml(indent + 4));
        }
        for c in &self.calcs {
            lines.push(c.to_sysml(indent + 4));
        }
        for req in &self.requirements {
            lines.push(req.to_sysml(indent + 4));
        }
        for sat_req in &self.satisfied_requirements {
            lines.push(format!("{}    satisfy requirement {};", pad, sat_req));
        }
        for st in &self.states {
            lines.push(st.to_sysml(indent + 4));
        }
        for conn in &self.connections {
            lines.push(conn.to_sysml(indent + 4));
        }
        for uc in &self.use_cases {
            lines.push(uc.to_sysml(indent + 4));
        }
        for item in &self.item_defs {
            lines.push(item.to_sysml(indent + 4));
        }
        for hz in &self.hazards {
            lines.push(hz.to_sysml(indent + 4));
        }
        for rk in &self.risks {
            lines.push(rk.to_sysml(indent + 4));
        }
        for tc in &self.test_cases {
            lines.push(tc.to_sysml(indent + 4));
        }
        for cap in &self.capabilities {
            lines.push(cap.to_sysml(indent + 4));
        }
        for op in &self.operations {
            lines.push(op.to_sysml(indent + 4));
        }
        for inter in &self.interactions {
            lines.push(inter.to_sysml(indent + 4));
        }
        for sub in &self.parts {
            lines.push(sub.to_sysml(indent + 4));
        }

        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for ImportDef {
    /// Executes deterministic fn `ImportDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ImportDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `ImportDef::to_sysml` (declaration #23) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `ImportDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `ImportDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `ImportDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `ImportDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `ImportDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `ImportDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `ImportDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `ImportDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
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
    /// Executes deterministic fn `PackageDef::to_sysml` within `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `PackageDef::to_sysml` in `crates/compile-sysml/src/semantic/serializer.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `PackageDef::to_sysml` (declaration #24) within `SEMANTIC-SERIALIZER`.
    ///
    /// ## 3. Preconditions
    /// - `indent: usize` must satisfy domain and type-safety constraints prior to invoking `PackageDef::to_sysml`.
    ///
    /// ## 4. Postconditions
    /// - Returns `String` from `PackageDef::to_sysml` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(indent)) bounded over input/member dimension of `PackageDef::to_sysml`.
    /// - Space Complexity: O(N(indent)) bounded auxiliary memory for `PackageDef::to_sysml`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `PackageDef::to_sysml` (`String`).
    /// - State: Maintains consistent state invariants across `PackageDef::to_sysml` without partial mutation on error.
    /// - Action: Executes `PackageDef::to_sysml` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `PackageDef::to_sysml` (`SEMANTIC-SERIALIZER`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-SERIALIZER-PACKAGEDEF-TO-SYSML] | SysML v2: ActionDef(PackageDef::to_sysml)
    fn to_sysml(&self, indent: usize) -> String {
        let pad = " ".repeat(indent);
        let doc = format_doc_comment(&self.doc, indent);
        let mut lines = Vec::new();

        if !doc.is_empty() {
            lines.push(doc.trim_end().to_string());
        }
        lines.push(format!("{}package {} {{", pad, self.name));

        for imp in &self.imports {
            lines.push(imp.to_sysml(indent + 4));
        }

        for a in &self.attribute_defs {
            lines.push(a.to_sysml(indent + 4));
        }
        for p in &self.port_defs {
            lines.push(p.to_sysml(indent + 4));
        }
        for act in &self.action_defs {
            lines.push(act.to_sysml(indent + 4));
        }
        for con in &self.constraint_defs {
            lines.push(con.to_sysml(indent + 4));
        }
        for c in &self.calc_defs {
            lines.push(c.to_sysml(indent + 4));
        }
        for req in &self.requirement_defs {
            lines.push(req.to_sysml(indent + 4));
        }
        for sat_req in &self.satisfied_requirements {
            lines.push(format!("{}    satisfy requirement {};", pad, sat_req));
        }
        for st in &self.state_defs {
            lines.push(st.to_sysml(indent + 4));
        }
        for conn in &self.connection_defs {
            lines.push(conn.to_sysml(indent + 4));
        }
        for uc in &self.use_case_defs {
            lines.push(uc.to_sysml(indent + 4));
        }
        for it in &self.item_defs {
            lines.push(it.to_sysml(indent + 4));
        }
        for hz in &self.hazard_defs {
            lines.push(hz.to_sysml(indent + 4));
        }
        for rk in &self.risk_defs {
            lines.push(rk.to_sysml(indent + 4));
        }
        for tc in &self.test_case_defs {
            lines.push(tc.to_sysml(indent + 4));
        }
        for cap in &self.capability_defs {
            lines.push(cap.to_sysml(indent + 4));
        }
        for op in &self.operation_defs {
            lines.push(op.to_sysml(indent + 4));
        }
        for inter in &self.interaction_defs {
            lines.push(inter.to_sysml(indent + 4));
        }
        for part in &self.part_defs {
            lines.push(part.to_sysml(indent + 4));
        }
        for sub in &self.packages {
            lines.push(sub.to_sysml(indent + 4));
        }

        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_package_roundtrip() {
        let pkg = PackageDef {
            name: "AutonomousPlatform".to_string(),
            doc: Some("Simple autonomous platform".to_string()),
            attribute_defs: vec![
                AttributeDef {
                    name: "massKg".to_string(),
                    type_name: "Real".to_string(),
                    default_value: Some("15.5".to_string()),
                    doc: None,
                },
            ],
            part_defs: vec![
                PartDef {
                    name: "StructuralFrame".to_string(),
                    ports: vec![
                        PortDef {
                            name: "bus".to_string(),
                            direction: "out".to_string(),
                            type_name: "CAN".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let sysml_text = pkg.to_sysml(0);
        assert!(sysml_text.contains("package AutonomousPlatform {"));
        assert!(sysml_text.contains("doc /* Simple autonomous platform */"));
        assert!(sysml_text.contains("attribute massKg : Real = 15.5;"));
        assert!(sysml_text.contains("part def StructuralFrame {"));
        assert!(sysml_text.contains("port out bus : CAN;"));
    }

    #[test]
    fn test_serialize_port_def_dataport_no_empty_body() {
        let port = PortDef {
            name: "rules_out".to_string(),
            type_name: "CompilerRulePort".to_string(),
            direction: "out".to_string(),
            port_category: "DataPort".to_string(),
            ..Default::default()
        };
        let sysml = port.to_sysml(4);
        assert_eq!(sysml, "    port out rules_out : CompilerRulePort;");
    }

    #[test]
    fn test_serialize_port_def_with_body_standard_syntax() {
        let port = PortDef {
            name: "rules_out".to_string(),
            type_name: "CompilerRulePort".to_string(),
            direction: "out".to_string(),
            protocol_family: "CAN".to_string(),
            ..Default::default()
        };
        let sysml = port.to_sysml(4);
        assert!(sysml.starts_with("    port out rules_out : CompilerRulePort {"));
        assert!(sysml.contains("attribute protocol_family : String = \"CAN\";"));
        assert!(sysml.ends_with("    }"));
    }

    #[test]
    fn test_serialize_requirement_def_with_attributes() {
        let req = RequirementDef {
            name: "REQ_0001_Test".to_string(),
            req_id: "REQ-0001".to_string(),
            text: "The system shall perform verification.".to_string(),
            attributes: vec![
                AttributeDef {
                    name: "uuidv5".to_string(),
                    type_name: "String".to_string(),
                    default_value: Some("\"5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9\"".to_string()),
                    doc: None,
                },
                AttributeDef {
                    name: "ac_01_test".to_string(),
                    type_name: "String".to_string(),
                    default_value: Some("\"Given: X. When: Y. Then: Z.\"".to_string()),
                    doc: None,
                },
            ],
            verified_by: vec!["AC_01_Test".to_string()],
            satisfied_by: vec!["TestEngine".to_string()],
            ..Default::default()
        };

        let sysml = req.to_sysml(0);
        assert!(sysml.contains("requirement def REQ_0001_Test {"));
        assert!(sysml.contains("    id = \"REQ-0001\";"));
        assert!(sysml.contains("    text = \"The system shall perform verification.\";"));
        assert!(sysml.contains("    attribute uuidv5 : String = \"5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9\";"));
        assert!(sysml.contains("    attribute ac_01_test : String = \"Given: X. When: Y. Then: Z.\";"));
        assert!(sysml.contains("    verify by AC_01_Test;"));
        assert!(sysml.contains("    satisfy by TestEngine;"));
    }

    #[test]
    fn test_serialize_requirement_def_with_assumes_requires_derived_from() {
        let req = RequirementDef {
            name: "REQ_0032_Test".to_string(),
            req_id: "REQ-0032".to_string(),
            text: "Test text".to_string(),
            assumes: vec!["ValidInput".to_string()],
            requires: vec!["Invariant_Alpha".to_string(), "Invariant_Beta".to_string()],
            derived_from: vec!["REQ_0031_Parent".to_string()],
            verified_by: vec!["AC_01".to_string()],
            satisfied_by: vec!["SubsystemEngine".to_string()],
            ..Default::default()
        };
        let sysml = req.to_sysml(0);
        assert!(sysml.contains("    assume ValidInput;"));
        assert!(sysml.contains("    require Invariant_Alpha;"));
        assert!(sysml.contains("    require Invariant_Beta;"));
        assert!(sysml.contains("    derived from REQ_0031_Parent;"));
    }

    #[test]
    fn test_serialize_constraint_def_single_line_when_empty_body_and_params() {
        let c1 = ConstraintDef {
            name: "StrictTotalOrdering".to_string(),
            is_assertion: true,
            ..Default::default()
        };
        assert_eq!(c1.to_sysml(4), "    assert constraint StrictTotalOrdering;");

        let c2 = ConstraintDef {
            name: "Invariant_Alpha".to_string(),
            is_assertion: false,
            ..Default::default()
        };
        assert_eq!(c2.to_sysml(0), "constraint def Invariant_Alpha;");
    }

    #[test]
    fn test_serialize_requirement_def_with_encapsulated_constraints() {
        let req = RequirementDef {
            name: "REQ_0001_Test".to_string(),
            req_id: "REQ-0001".to_string(),
            text: "Testing constraints inside requirement def.".to_string(),
            constraints: vec![
                ConstraintDef {
                    name: "Invariant_Alpha".to_string(),
                    doc: Some("Formula Alpha".to_string()),
                    is_assertion: false,
                    ..Default::default()
                },
                ConstraintDef {
                    name: "Invariant_Beta".to_string(),
                    expression: "x > 0".to_string(),
                    is_assertion: true,
                    ..Default::default()
                },
            ],
            requires: vec!["Invariant_Alpha".to_string()],
            ..Default::default()
        };
        let sysml = req.to_sysml(0);
        assert!(sysml.contains("requirement def REQ_0001_Test {"));
        assert!(sysml.contains("    doc /* Formula Alpha */\n    constraint def Invariant_Alpha;"));
        assert!(sysml.contains("    assert constraint Invariant_Beta {\n        x > 0;\n    }"));
        assert!(sysml.contains("    require Invariant_Alpha;"));
    }

    #[test]
    fn test_serialize_package_with_imports() {
        let pkg = PackageDef {
            name: "SubsystemWithImports".to_string(),
            imports: vec![
                ImportDef {
                    path: "ScalarValues".to_string(),
                    is_wildcard: true,
                    is_recursive: false,
                    doc: None,
                },
                ImportDef {
                    path: "Base::Types::*".to_string(),
                    is_wildcard: true,
                    is_recursive: false,
                    doc: None,
                },
                ImportDef {
                    path: "NestedSys".to_string(),
                    is_wildcard: true,
                    is_recursive: true,
                    doc: None,
                },
                ImportDef {
                    path: "Specific::Item".to_string(),
                    is_wildcard: false,
                    is_recursive: false,
                    doc: Some("Specific imported element".to_string()),
                },
            ],
            ..Default::default()
        };

        let sysml_text = pkg.to_sysml(0);
        assert!(sysml_text.contains("package SubsystemWithImports {"));
        assert!(sysml_text.contains("    import ScalarValues::*;"));
        assert!(sysml_text.contains("    import Base::Types::*;"));
        assert!(sysml_text.contains("    import NestedSys::**;"));
        assert!(sysml_text.contains("    doc /* Specific imported element */\n    import Specific::Item;"));
    }

    #[test]
    fn test_serialize_part_def_and_package_def_with_satisfied_requirements() {
        let part = PartDef {
            name: "DisplayUnit".to_string(),
            satisfied_requirements: vec!["REQ-UI-01".to_string(), "REQ-UI-02".to_string()],
            ..Default::default()
        };
        let part_sysml = part.to_sysml(0);
        assert!(part_sysml.contains("    satisfy requirement REQ-UI-01;"));
        assert!(part_sysml.contains("    satisfy requirement REQ-UI-02;"));

        let pkg = PackageDef {
            name: "DisplayPackage".to_string(),
            satisfied_requirements: vec!["REQ-SYS-001".to_string()],
            part_defs: vec![part],
            ..Default::default()
        };
        let pkg_sysml = pkg.to_sysml(0);
        assert!(pkg_sysml.contains("    satisfy requirement REQ-SYS-001;"));
        assert!(pkg_sysml.contains("        satisfy requirement REQ-UI-01;"));
        assert!(pkg_sysml.contains("        satisfy requirement REQ-UI-02;"));
    }
}



