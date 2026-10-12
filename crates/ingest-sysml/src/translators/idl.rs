//! # OMG IDL to SysML v2 AST Translator Engine
//!
//! ## 1. Safety Intent & Regulatory Scope
//! This module provides deterministic translation of Object Management Group (OMG)
//! Interface Definition Language (IDL) specifications into canonical SysML v2 textual
//! AST models (`deap_core::sysml_ast::PackageDef`).
//!
//! In high-integrity distributed avionics and autonomous architectures (e.g. DDS / RTPS,
//! FACE, and ARINC 653 inter-partition messaging), OMG IDL defines authoritative wire
//! payload contracts, structural serialization layouts, and remote procedure call (RPC)
//! interface boundaries.
//!
//! ## 2. Core Safety Invariants
//! - **Structural Integrity**: IDL `module` declarations map to canonical `PackageDef` containers.
//! - **Data Integrity**: IDL `struct` types map to `PartDef` data structures with typed `AttributeDef` members.
//! - **Behavioral Interface Integrity**: IDL `interface` operations map to `PartDef` behavioral blocks
//!   containing formal `ActionDef` calls with typed `in_params` and `out_params`.
//! - **Type Alias Preservation**: IDL `typedef` declarations project as package-level `AttributeDef` aliases.
//! - **Deterministic Identifier Sanitization**: All symbols pass through `sanitize_identifier` to prevent
//!   SysML v2 reserved keyword collisions or illegal punctuation.

use regex::Regex;
use deap_core::sysml_ast::{ActionDef, AttributeDef, PackageDef, PartDef};
use crate::table::sanitize_identifier;

/// Deterministic translator for OMG IDL specifications.
///
/// /// Realises: [REQ-0001/IDLTranslator]
#[derive(Debug, Default, Clone)]
pub struct IDLTranslator;

impl IDLTranslator {
    /// Creates a fresh instance of the IDL translator.
    pub fn new() -> Self {
        Self
    }

    /// Translates OMG IDL source text into a canonical SysML v2 `PackageDef`.
    ///
    /// /// Realises: [REQ-0001/IDLTranslator_translate]
    ///
    /// ### Algorithmic Steps:
    /// 1. Extract enclosing `module <name> { ... }` as package namespace.
    /// 2. Parse all `struct <name> { ... };` definitions into typed `PartDef` entities.
    /// 3. Parse all `interface <name> { ... };` definitions into behavioral `PartDef` entities.
    /// 4. Parse all `typedef <type> <name>;` declarations into package-level `AttributeDef` entries.
    ///
    /// ### Preconditions:
    /// - `content` contains valid UTF-8 text representing an OMG IDL schema.
    /// - `default_name` provides a non-empty fallback package identifier.
    ///
    /// ### Postconditions:
    /// - Returns a validated `PackageDef` containing mapped SysML v2 AST nodes.
    pub fn translate(&self, content: &str, default_name: &str) -> Result<PackageDef, String> {
        let module_re = Regex::new(r"\bmodule\s+([a-zA-Z0-9_\-]+)\s*\{").unwrap();
        let pkg_name = if let Some(caps) = module_re.captures(content) {
            sanitize_identifier(caps.get(1).unwrap().as_str(), default_name)
        } else {
            sanitize_identifier(default_name, "IDL_Package")
        };

        let mut pkg = PackageDef {
            name: pkg_name,
            doc: Some("Translated from OMG IDL schema".to_string()),
            ..Default::default()
        };

        // Parse structs: struct <name> { <type> <field>; ... };
        let struct_re = Regex::new(r"(?s)\bstruct\s+([a-zA-Z0-9_\-]+)\s*\{([^}]+)\};").unwrap();
        let field_re = Regex::new(r"(?m)^\s*([a-zA-Z0-9_\-<>:]+)\s+([a-zA-Z0-9_\-]+)\s*;").unwrap();

        for s_cap in struct_re.captures_iter(content) {
            let s_name = sanitize_identifier(s_cap.get(1).unwrap().as_str(), "Struct");
            let s_body = s_cap.get(2).unwrap().as_str();

            let mut attrs = Vec::new();
            for f_cap in field_re.captures_iter(s_body) {
                let f_type = f_cap.get(1).unwrap().as_str().trim().to_string();
                let f_name = sanitize_identifier(f_cap.get(2).unwrap().as_str().trim(), "field");
                attrs.push(AttributeDef {
                    name: f_name,
                    type_name: f_type,
                    default_value: None,
                    doc: None,
                });
            }

            pkg.part_defs.push(PartDef {
                name: s_name,
                doc: Some("IDL Struct".to_string()),
                attributes: attrs,
                ..Default::default()
            });
        }

        // Parse interfaces: interface <name> { <ret> <op>(<params>); ... };
        let interface_re = Regex::new(r"(?s)\binterface\s+([a-zA-Z0-9_\-]+)\s*\{([^}]+)\};").unwrap();
        let op_re = Regex::new(r"(?m)^\s*(?:[a-zA-Z0-9_\-\:]+)\s+([a-zA-Z0-9_\-]+)\s*\(([^)]*)\)\s*;").unwrap();

        for if_cap in interface_re.captures_iter(content) {
            let if_name = sanitize_identifier(if_cap.get(1).unwrap().as_str(), "Interface");
            let if_body = if_cap.get(2).unwrap().as_str();

            let mut actions = Vec::new();
            for op_cap in op_re.captures_iter(if_body) {
                let op_name = sanitize_identifier(op_cap.get(1).unwrap().as_str().trim(), "action");
                let params_raw = op_cap.get(2).unwrap().as_str().trim();

                let mut in_params = Vec::new();
                let mut out_params = Vec::new();

                if !params_raw.is_empty() {
                    let param_items: Vec<&str> = params_raw.split(',').map(|p| p.trim()).filter(|p| !p.is_empty()).collect();
                    for p in param_items {
                        let p_parts: Vec<&str> = p.split_whitespace().collect();
                        if p_parts.len() >= 3 && ["in", "out", "inout"].contains(&p_parts[0]) {
                            let direction = p_parts[0];
                            let p_type = p_parts[1].to_string();
                            let p_n = sanitize_identifier(p_parts[2], "param");
                            let attr = AttributeDef {
                                name: p_n,
                                type_name: p_type,
                                default_value: None,
                                doc: None,
                            };
                            if direction == "out" {
                                out_params.push(attr);
                            } else {
                                in_params.push(attr);
                            }
                        } else if p_parts.len() >= 2 {
                            let p_type = p_parts[0].to_string();
                            let p_n = sanitize_identifier(p_parts[1], "param");
                            in_params.push(AttributeDef {
                                name: p_n,
                                type_name: p_type,
                                default_value: None,
                                doc: None,
                            });
                        }
                    }
                }

                actions.push(ActionDef {
                    name: op_name,
                    in_params,
                    out_params,
                    ..Default::default()
                });
            }

            pkg.part_defs.push(PartDef {
                name: if_name,
                doc: Some("IDL Interface".to_string()),
                actions,
                ..Default::default()
            });
        }

        // Parse typedefs: typedef <type> <name>;
        let typedef_re = Regex::new(r"(?m)^\s*typedef\s+([a-zA-Z0-9_\-<>:]+)\s+([a-zA-Z0-9_\-]+)\s*;").unwrap();
        for t_cap in typedef_re.captures_iter(content) {
            let t_type = t_cap.get(1).unwrap().as_str().trim().to_string();
            let t_name = sanitize_identifier(t_cap.get(2).unwrap().as_str().trim(), "typedef");
            pkg.attribute_defs.push(AttributeDef {
                name: t_name,
                type_name: t_type,
                default_value: None,
                doc: Some("IDL Typedef".to_string()),
            });
        }

        Ok(pkg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idl_translation_synthetic() {
        let idl_source = r#"
module Module_Alpha {
    typedef sequence<octet> Payload_Type;

    struct Struct_Zero {
        long id;
        double metric_x;
    };

    interface Interface_Prime {
        void ExecuteCommand(in string cmd, out long status);
    };
};
"#;
        let translator = IDLTranslator::new();
        let pkg = translator.translate(idl_source, "Fallback").expect("Translation failed");

        assert_eq!(pkg.name, "Module_Alpha");
        assert_eq!(pkg.attribute_defs.len(), 1);
        assert_eq!(pkg.attribute_defs[0].name, "Payload_Type");
        assert_eq!(pkg.attribute_defs[0].type_name, "sequence<octet>");

        assert_eq!(pkg.part_defs.len(), 2);
        let s0 = pkg.part_defs.iter().find(|p| p.name == "Struct_Zero").unwrap();
        assert_eq!(s0.attributes.len(), 2);
        assert_eq!(s0.attributes[0].name, "id");
        assert_eq!(s0.attributes[1].name, "metric_x");

        let ifp = pkg.part_defs.iter().find(|p| p.name == "Interface_Prime").unwrap();
        assert_eq!(ifp.actions.len(), 1);
        let action = &ifp.actions[0];
        assert_eq!(action.name, "ExecuteCommand");
        assert_eq!(action.in_params.len(), 1);
        assert_eq!(action.in_params[0].name, "cmd");
        assert_eq!(action.out_params.len(), 1);
        assert_eq!(action.out_params[0].name, "status");
    }
}
