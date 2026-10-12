//! SysML v2 / KerML canonical textual notation serializer / pretty-printer.

use deap_core::sysml_ast::*;

/// Helper to format doc comments as standard SysML v2 doc blocks.
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
    } else {
        let mut out = format!("{}doc /* {}\n", pad, lines[0]);
        for line in &lines[1..lines.len() - 1] {
            out.push_str(&format!("{}       {}\n", pad, line));
        }
        out.push_str(&format!("{}       {} */\n", pad, lines[lines.len() - 1]));
        out
    }
}

/// Trait providing SysML v2 canonical serialization.
pub trait SysmlSerializable {
    fn to_sysml(&self, indent: usize) -> String;
}

/// Convenience function to serialize any SysmlSerializable entity to canonical textual notation.
pub fn to_sysml<T: SysmlSerializable>(item: &T) -> String {
    item.to_sysml(0)
}


impl SysmlSerializable for AttributeDef {
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

impl SysmlSerializable for PartDef {
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
        for req in &self.requirements {
            lines.push(req.to_sysml(indent + 4));
        }
        for st in &self.states {
            lines.push(st.to_sysml(indent + 4));
        }
        for conn in &self.connections {
            lines.push(conn.to_sysml(indent + 4));
        }
        for item in &self.item_defs {
            lines.push(item.to_sysml(indent + 4));
        }
        for sub in &self.parts {
            lines.push(sub.to_sysml(indent + 4));
        }

        lines.push(format!("{}}}", pad));
        lines.join("\n")
    }
}

impl SysmlSerializable for ImportDef {
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
        for req in &self.requirement_defs {
            lines.push(req.to_sysml(indent + 4));
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
            name: "SimpleVehicle".to_string(),
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
                    name: "Airframe".to_string(),
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
        assert!(sysml_text.contains("package SimpleVehicle {"));
        assert!(sysml_text.contains("doc /* Simple autonomous platform */"));
        assert!(sysml_text.contains("attribute massKg : Real = 15.5;"));
        assert!(sysml_text.contains("part def Airframe {"));
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
}



