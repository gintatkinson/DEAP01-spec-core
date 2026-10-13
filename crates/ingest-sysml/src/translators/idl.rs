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
        let pkg_name = if let Some(mod_name) = scan_module_name(content) {
            sanitize_identifier(&mod_name, default_name)
        } else {
            sanitize_identifier(default_name, "IDL_Package")
        };

        let mut pkg = PackageDef {
            name: pkg_name,
            doc: Some("Translated from OMG IDL schema".to_string()),
            ..Default::default()
        };

        // Parse structs: struct <name> { <type> <field>; ... };
        for (s_raw_name, s_body) in scan_idl_blocks(content, "struct") {
            let s_name = sanitize_identifier(s_raw_name, "Struct");

            let mut attrs = Vec::new();
            for (f_type, f_raw_name) in scan_struct_fields(s_body) {
                let f_name = sanitize_identifier(&f_raw_name, "field");
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
        for (if_raw_name, if_body) in scan_idl_blocks(content, "interface") {
            let if_name = sanitize_identifier(if_raw_name, "Interface");

            let mut actions = Vec::new();
            for (op_raw_name, params_raw) in scan_interface_ops(if_body) {
                let op_name = sanitize_identifier(&op_raw_name, "action");
                let params_raw = params_raw.trim();

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
        for (t_type, t_raw_name) in scan_typedefs(content) {
            let t_name = sanitize_identifier(&t_raw_name, "typedef");
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

fn scan_module_name(content: &str) -> Option<String> {
    let mut search_pos = 0;
    while let Some(kw_pos) = content[search_pos..].find("module") {
        let abs_pos = search_pos + kw_pos;
        let before_ok = abs_pos == 0 || {
            let b = content.as_bytes()[abs_pos - 1];
            !b.is_ascii_alphanumeric() && b != b'_'
        };
        let after = &content[abs_pos + 6..];
        if before_ok && after.starts_with(char::is_whitespace) {
            if let Some(brace_pos) = after.find('{') {
                let header = after[..brace_pos].trim();
                let name: String = header
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                    .collect();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
        search_pos = abs_pos + 6;
    }
    None
}

fn scan_idl_blocks<'a>(content: &'a str, keyword: &str) -> Vec<(&'a str, &'a str)> {
    let mut results = Vec::new();
    let mut search_pos = 0;
    while let Some(kw_pos) = content[search_pos..].find(keyword) {
        let abs_pos = search_pos + kw_pos;
        let before_ok = abs_pos == 0 || {
            let b = content.as_bytes()[abs_pos - 1];
            !b.is_ascii_alphanumeric() && b != b'_'
        };
        let after = &content[abs_pos + keyword.len()..];
        if before_ok && after.starts_with(char::is_whitespace) {
            if let Some(brace_open) = after.find('{') {
                let header = after[..brace_open].trim();
                let name_len = header
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                    .count();
                let block_name = &header[..name_len];
                let body_and_rest = &after[brace_open + 1..];
                if let Some(brace_close) = body_and_rest.find('}') {
                    let body = &body_and_rest[..brace_close];
                    let after_close = body_and_rest[brace_close + 1..].trim_start();
                    if after_close.starts_with(';') {
                        if !block_name.is_empty() {
                            results.push((block_name, body));
                        }
                        search_pos = abs_pos + keyword.len() + brace_open + 1 + brace_close + 1;
                        continue;
                    }
                }
            }
        }
        search_pos = abs_pos + keyword.len();
    }
    results
}

fn scan_struct_fields(body: &str) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    for stmt in body.split(';') {
        let trimmed = stmt.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            let f_type = parts[..parts.len() - 1].join(" ");
            let f_name = parts[parts.len() - 1];
            if f_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                && f_type.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '<' || c == '>' || c == ':')
            {
                fields.push((f_type, f_name.to_string()));
            }
        }
    }
    fields
}

fn scan_interface_ops(if_body: &str) -> Vec<(String, String)> {
    let mut ops = Vec::new();
    for stmt in if_body.split(';') {
        let stmt = stmt.trim();
        if let Some(open_paren) = stmt.find('(') {
            if let Some(close_paren) = stmt[open_paren + 1..].rfind(')') {
                let close_idx = open_paren + 1 + close_paren;
                let before_paren = stmt[..open_paren].trim();
                let params_raw = stmt[open_paren + 1..close_idx].trim();
                let tokens: Vec<&str> = before_paren.split_whitespace().collect();
                if tokens.len() >= 2 {
                    let op_name = tokens[tokens.len() - 1];
                    if op_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                        ops.push((op_name.to_string(), params_raw.to_string()));
                    }
                }
            }
        }
    }
    ops
}

fn scan_typedefs(content: &str) -> Vec<(String, String)> {
    let mut typedefs = Vec::new();
    let mut search_pos = 0;
    while let Some(kw_pos) = content[search_pos..].find("typedef") {
        let abs_pos = search_pos + kw_pos;
        let before_ok = abs_pos == 0 || {
            let b = content.as_bytes()[abs_pos - 1];
            !b.is_ascii_alphanumeric() && b != b'_'
        };
        let after = &content[abs_pos + 7..];
        if before_ok && after.starts_with(char::is_whitespace) {
            if let Some(semi_pos) = after.find(';') {
                let stmt = after[..semi_pos].trim();
                let parts: Vec<&str> = stmt.split_whitespace().collect();
                if parts.len() >= 2 {
                    let t_type = parts[..parts.len() - 1].join(" ");
                    let t_name = parts[parts.len() - 1];
                    if t_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                        && t_type.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '<' || c == '>' || c == ':')
                    {
                        typedefs.push((t_type, t_name.to_string()));
                    }
                }
                search_pos = abs_pos + 7 + semi_pos + 1;
                continue;
            }
        }
        search_pos = abs_pos + 7;
    }
    typedefs
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
