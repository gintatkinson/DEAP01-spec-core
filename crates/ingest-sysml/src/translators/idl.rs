//! # OMG IDL to SysML v2 AST Translator Engine
//!
//! Deterministically translates Object Management Group (OMG) Interface Definition
//! Language (IDL) specifications into canonical `deap_core::sysml_ast::PackageDef` models.
//!
//! Realises: [REQ-0001/IDLTranslator]

use crate::table::sanitize_identifier;
use crate::translators::idl_cst::{parse_idl_cst, IdlToken};
use deap_core::sysml_ast::{ActionDef, AttributeDef, PackageDef, PartDef};
use logos::Logos;

/// Strips line (`//`) and block (`/* ... */`) comments from OMG IDL source via `idl_cst`.
/// Realises: [REQ-TRANSLATORS-IDL-STRIP-COMMENTS]
fn strip_idl_comments(content: &str) -> String {
    let cst = parse_idl_cst(content);
    let mut cleaned = String::with_capacity(content.len().saturating_add(cst.nodes.len()));
    let mut last_end = 0usize;
    for (tok_res, span) in IdlToken::lexer(content).spanned() {
        if let Some(gap) = content.get(last_end..span.start) {
            cleaned.push_str(gap);
        }
        match tok_res {
            Ok(IdlToken::LineComment | IdlToken::BlockComment) => {
                if let Some(slice) = content.get(span.clone()) {
                    for ch in slice.chars() {
                        cleaned.push(if ch == '\n' { '\n' } else { ' ' });
                    }
                }
            }
            _ => {
                if let Some(slice) = content.get(span.clone()) {
                    cleaned.push_str(slice);
                }
            }
        }
        last_end = span.end;
    }
    if let Some(tail) = content.get(last_end..) {
        cleaned.push_str(tail);
    }
    cleaned
}

/// Deterministic translator for OMG IDL specifications.
/// Realises: [REQ-0001/IDLTranslator]
#[derive(Debug, Default, Clone)]
pub struct IDLTranslator;

impl IDLTranslator {
    /// Creates a fresh instance of the IDL translator.
    /// Realises: [REQ-TRANSLATORS-IDL-IDLTRANSLATOR-NEW]
    pub fn new() -> Self {
        Self
    }

    /// Translates OMG IDL source text into a canonical SysML v2 `PackageDef`.
    /// Realises: [REQ-0001/IDLTranslator_translate]
    pub fn translate(&self, content: &str, default_name: &str) -> Result<PackageDef, String> {
        let stripped = strip_idl_comments(content);
        let pkg_name = if let Some(mod_name) = scan_module_name(&stripped) {
            sanitize_identifier(&mod_name, default_name)
        } else {
            sanitize_identifier(default_name, "IDL_Package")
        };

        let mut pkg = PackageDef {
            name: pkg_name,
            doc: Some("Translated from OMG IDL schema".to_string()),
            ..Default::default()
        };

        for (s_raw_name, s_body) in scan_idl_blocks(&stripped, "struct") {
            let s_name = sanitize_identifier(s_raw_name, "Struct");
            let mut attrs = Vec::new();
            for (f_type, f_raw_name) in scan_struct_fields(s_body) {
                attrs.push(AttributeDef {
                    name: sanitize_identifier(&f_raw_name, "field"),
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

        for (if_raw_name, if_body) in scan_idl_blocks(&stripped, "interface") {
            let if_name = sanitize_identifier(if_raw_name, "Interface");
            let mut actions = Vec::new();
            for (op_raw_name, params_raw) in scan_interface_ops(if_body) {
                let op_name = sanitize_identifier(&op_raw_name, "action");
                let params_raw = params_raw.trim();
                let mut in_params = Vec::new();
                let mut out_params = Vec::new();
                if !params_raw.is_empty() {
                    for p in params_raw.split(',').map(str::trim).filter(|p| !p.is_empty()) {
                        let p_parts: Vec<&str> = p.split_whitespace().collect();
                        if let (Some(&direction), Some(&raw_type), Some(&raw_name)) =
                            (p_parts.first(), p_parts.get(1), p_parts.get(2))
                        {
                            if ["in", "out", "inout"].contains(&direction) {
                                let attr = AttributeDef {
                                    name: sanitize_identifier(raw_name, "param"),
                                    type_name: raw_type.to_string(),
                                    default_value: None,
                                    doc: None,
                                };
                                if direction == "out" {
                                    out_params.push(attr);
                                } else {
                                    in_params.push(attr);
                                }
                                continue;
                            }
                        }
                        if let (Some(&raw_type), Some(&raw_name)) =
                            (p_parts.first(), p_parts.get(1))
                        {
                            in_params.push(AttributeDef {
                                name: sanitize_identifier(raw_name, "param"),
                                type_name: raw_type.to_string(),
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

        for (t_type, t_raw_name) in scan_typedefs(&stripped) {
            pkg.attribute_defs.push(AttributeDef {
                name: sanitize_identifier(&t_raw_name, "typedef"),
                type_name: t_type,
                default_value: None,
                doc: Some("IDL Typedef".to_string()),
            });
        }

        Ok(pkg)
    }
}

/// Extracts the primary IDL `module` identifier using `idl_cst` and token spans.
/// Realises: [REQ-TRANSLATORS-IDL-SCAN-MODULE-NAME]
fn scan_module_name(content: &str) -> Option<String> {
    let cst = parse_idl_cst(content);
    if let Some(node) = cst.nodes.iter().find(|n| n.kind == "module") {
        return Some(node.name.clone());
    }
    let mut saw_module = false;
    for (tok_res, span) in IdlToken::lexer(content).spanned() {
        let slice = content.get(span)?;
        match tok_res {
            Ok(IdlToken::Keyword) if slice == "module" => saw_module = true,
            Ok(IdlToken::Ident) if saw_module => return Some(slice.to_string()),
            Ok(IdlToken::Whitespace | IdlToken::LineComment | IdlToken::BlockComment) => {}
            _ => saw_module = false,
        }
    }
    None
}

/// Extracts named `{ ... };` blocks for a target IDL keyword (`struct` or `interface`).
/// Realises: [REQ-TRANSLATORS-IDL-SCAN-IDL-BLOCKS]
fn scan_idl_blocks<'a>(content: &'a str, keyword: &str) -> Vec<(&'a str, &'a str)> {
    let mut results = Vec::new();
    let mut search_pos = 0;
    while let Some(kw_pos) = content.get(search_pos..).and_then(|s| s.find(keyword)) {
        let abs_pos = search_pos + kw_pos;
        let before_ok = abs_pos == 0
            || content
                .as_bytes()
                .get(abs_pos.saturating_sub(1))
                .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_');
        let after = content.get(abs_pos + keyword.len()..).unwrap_or("");
        if before_ok && after.starts_with(char::is_whitespace) {
            if let Some(brace_open) = after.find('{') {
                let header = after.get(..brace_open).unwrap_or("").trim();
                let name_len = header
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                    .count();
                let block_name = header.get(..name_len).unwrap_or("");
                let body_and_rest = after.get(brace_open + 1..).unwrap_or("");
                if let Some(brace_close) = body_and_rest.find('}') {
                    let body = body_and_rest.get(..brace_close).unwrap_or("");
                    let after_close = body_and_rest
                        .get(brace_close + 1..)
                        .unwrap_or("")
                        .trim_start();
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

/// Extracts `(type_name, field_name)` pairs from an IDL struct body.
/// Realises: [REQ-TRANSLATORS-IDL-SCAN-STRUCT-FIELDS]
fn scan_struct_fields(body: &str) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    for stmt in body.split(';') {
        let trimmed = stmt.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if let Some((&f_name, type_parts)) = parts.split_last().filter(|(_, tp)| !tp.is_empty()) {
            let f_type = type_parts.join(" ");
            if f_name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                && f_type.chars().all(|c| {
                    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '<' | '>' | ':')
                })
            {
                fields.push((f_type, f_name.to_string()));
            }
        }
    }
    fields
}

/// Extracts `(operation_name, params_raw)` pairs from an IDL interface body.
/// Realises: [REQ-TRANSLATORS-IDL-SCAN-INTERFACE-OPS]
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
                if let Some((&op_name, _)) = tokens.split_last().filter(|(_, p)| !p.is_empty()) {
                    if op_name
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                    {
                        ops.push((op_name.to_string(), params_raw.to_string()));
                    }
                }
            }
        }
    }
    ops
}

/// Extracts `(type_name, alias_name)` pairs from top-level IDL `typedef` statements.
/// Realises: [REQ-TRANSLATORS-IDL-SCAN-TYPEDEFS]
fn scan_typedefs(content: &str) -> Vec<(String, String)> {
    let mut typedefs = Vec::new();
    let mut search_pos = 0;
    while let Some(kw_pos) = content.get(search_pos..).and_then(|s| s.find("typedef")) {
        let abs_pos = search_pos + kw_pos;
        let before_ok = abs_pos == 0
            || content
                .as_bytes()
                .get(abs_pos.saturating_sub(1))
                .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_');
        let after = content.get(abs_pos + 7..).unwrap_or("");
        if before_ok && after.starts_with(char::is_whitespace) {
            if let Some(semi_pos) = after.find(';') {
                let stmt = after.get(..semi_pos).unwrap_or("").trim();
                let parts: Vec<&str> = stmt.split_whitespace().collect();
                if let Some((&t_name, type_parts)) =
                    parts.split_last().filter(|(_, tp)| !tp.is_empty())
                {
                    let t_type = type_parts.join(" ");
                    if t_name
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                        && t_type.chars().all(|c| {
                            c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '<' | '>' | ':')
                        })
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
    fn test_idl_translation_synthetic() -> Result<(), String> {
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
        let pkg = translator.translate(idl_source, "Fallback")?;

        assert_eq!(pkg.name, "Module_Alpha");
        assert_eq!(pkg.attribute_defs.len(), 1);
        assert_eq!(
            pkg.attribute_defs.first().map(|a| a.name.as_str()),
            Some("Payload_Type")
        );
        assert_eq!(
            pkg.attribute_defs.first().map(|a| a.type_name.as_str()),
            Some("sequence<octet>")
        );

        assert_eq!(pkg.part_defs.len(), 2);
        let s0 = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "Struct_Zero")
            .ok_or("missing Struct_Zero")?;
        assert_eq!(s0.attributes.len(), 2);

        let ifp = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "Interface_Prime")
            .ok_or("missing Interface_Prime")?;
        assert_eq!(ifp.actions.len(), 1);
        let action = ifp.actions.first().ok_or("missing ExecuteCommand")?;
        assert_eq!(action.name, "ExecuteCommand");
        assert_eq!(action.in_params.len(), 1);
        assert_eq!(action.out_params.len(), 1);
        Ok(())
    }
}
