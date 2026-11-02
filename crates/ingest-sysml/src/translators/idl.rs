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
    let mut cleaned = String::with_capacity(content.len());
    let mut last_end = 0usize;
    for (tok_res, span) in IdlToken::lexer(content).spanned() {
        if let Some(gap) = content.get(last_end..span.start) {
            cleaned.push_str(gap);
        }
        if let Some(slice) = content.get(span.clone()) {
            if matches!(tok_res, Ok(IdlToken::LineComment | IdlToken::BlockComment)) {
                cleaned.extend(slice.chars().map(|ch| if ch == '\n' { '\n' } else { ' ' }));
            } else {
                cleaned.push_str(slice);
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
        let pkg_name = scan_module_name(&stripped).map_or_else(
            || sanitize_identifier(default_name, "IDL_Package"),
            |mod_name| sanitize_identifier(&mod_name, default_name),
        );
        let mut pkg = PackageDef {
            name: pkg_name,
            doc: Some("Translated from OMG IDL schema".to_string()),
            ..Default::default()
        };
        for (s_raw_name, s_body) in scan_idl_blocks(&stripped, "struct") {
            let attributes = scan_struct_fields(s_body)
                .into_iter()
                .map(|(type_name, f_raw_name)| AttributeDef {
                    name: sanitize_identifier(&f_raw_name, "field"),
                    type_name,
                    default_value: None,
                    doc: None,
                })
                .collect();
            pkg.part_defs.push(PartDef {
                name: sanitize_identifier(s_raw_name, "Struct"),
                doc: Some("IDL Struct".to_string()),
                attributes,
                ..Default::default()
            });
        }
        for (if_raw_name, if_body) in scan_idl_blocks(&stripped, "interface") {
            let actions = scan_interface_ops(if_body)
                .into_iter()
                .map(|(op_raw_name, params_raw)| {
                    let (in_params, out_params) = parse_idl_op_params(&params_raw);
                    ActionDef {
                        name: sanitize_identifier(&op_raw_name, "action"),
                        in_params,
                        out_params,
                        ..Default::default()
                    }
                })
                .collect();
            pkg.part_defs.push(PartDef {
                name: sanitize_identifier(if_raw_name, "Interface"),
                doc: Some("IDL Interface".to_string()),
                actions,
                ..Default::default()
            });
        }
        for (type_name, t_raw_name) in scan_typedefs(&stripped) {
            pkg.attribute_defs.push(AttributeDef {
                name: sanitize_identifier(&t_raw_name, "typedef"),
                type_name,
                default_value: None,
                doc: Some("IDL Typedef".to_string()),
            });
        }
        Ok(pkg)
    }
}

/// Parses an IDL operation parameter list into `(in_params, out_params)` via `IdlToken::lexer`.
/// Realises: [REQ-0001/IDLTranslator_translate]
fn parse_idl_op_params(params_raw: &str) -> (Vec<AttributeDef>, Vec<AttributeDef>) {
    let (mut in_params, mut out_params, mut cur, mut groups) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for (tok_res, span) in IdlToken::lexer(params_raw).spanned() {
        let Some(slice) = params_raw.get(span) else { continue };
        match tok_res {
            Ok(IdlToken::Whitespace | IdlToken::LineComment | IdlToken::BlockComment) => {}
            Ok(IdlToken::Punct) if slice == "," => {
                if !cur.is_empty() {
                    groups.push(std::mem::take(&mut cur));
                }
            }
            Ok(tok) => cur.push((tok, slice)),
            Err(()) => {}
        }
    }
    if !cur.is_empty() {
        groups.push(cur);
    }
    for group in groups {
        let Some(&(_, raw_name)) = group.last() else { continue };
        let prefix = &group[..group.len().saturating_sub(1)];
        let (direction, type_tokens) = match prefix.first() {
            Some(&(_, dir @ ("in" | "out" | "inout"))) if prefix.len() >= 2 => (dir, &prefix[1..]),
            _ if !prefix.is_empty() => ("in", prefix),
            _ => continue,
        };
        let Some(type_name) = join_idl_type_tokens(type_tokens) else { continue };
        let attr = AttributeDef {
            name: sanitize_identifier(raw_name, "param"),
            type_name,
            default_value: None,
            doc: None,
        };
        if direction == "out" {
            out_params.push(attr);
        } else {
            in_params.push(attr);
        }
    }
    (in_params, out_params)
}

/// Joins tokenized IDL type segments into a canonical type string.
/// Realises: [REQ-TRANSLATORS-IDL-SCAN-STRUCT-FIELDS]
fn join_idl_type_tokens(tokens: &[(IdlToken, &str)]) -> Option<String> {
    let (mut out, mut prev_was_word) = (String::new(), false);
    for &(tok, slice) in tokens {
        let is_word = matches!(tok, IdlToken::Ident | IdlToken::Keyword);
        let is_punct = matches!(tok, IdlToken::Punct) && matches!(slice, "<" | ">" | "::" | "-");
        if !is_word && !is_punct {
            return None;
        }
        if prev_was_word && is_word {
            out.push(' ');
        }
        out.push_str(slice);
        prev_was_word = is_word;
    }
    (!out.is_empty()).then_some(out)
}

/// Extracts the primary IDL `module` identifier using `idl_cst` and token spans.
/// Realises: [REQ-TRANSLATORS-IDL-SCAN-MODULE-NAME]
fn scan_module_name(content: &str) -> Option<String> {
    parse_idl_cst(content)
        .nodes
        .into_iter()
        .find(|n| n.kind == "module")
        .map(|n| n.name)
}

/// Extracts named `{ ... };` blocks for a target IDL keyword (`struct` or `interface`).
/// Realises: [REQ-TRANSLATORS-IDL-SCAN-IDL-BLOCKS]
fn scan_idl_blocks<'a>(content: &'a str, keyword: &str) -> Vec<(&'a str, &'a str)> {
    let cst = parse_idl_cst(content);
    let mut results = Vec::new();
    for node in cst.nodes.iter().filter(|n| n.kind == keyword) {
        let name_start = node.span_end.saturating_sub(node.name.len());
        let (Some(block_name), Some(rest)) = (
            content.get(name_start..node.span_end).filter(|s| !s.is_empty()),
            content.get(node.span_end..),
        ) else {
            continue;
        };
        let (mut brace_open_end, mut brace_close_start, mut depth) = (None, None, 0usize);
        for (tok_res, span) in IdlToken::lexer(rest).spanned() {
            let Some(slice) = rest.get(span.clone()) else { continue };
            if let Some(b_end) = brace_close_start {
                if matches!(tok_res, Ok(IdlToken::Whitespace | IdlToken::LineComment | IdlToken::BlockComment)) {
                    continue;
                }
                if matches!(tok_res, Ok(IdlToken::Punct)) && slice == ";" {
                    if let Some(b_start) = brace_open_end.and_then(|s| rest.get(s..b_end)) {
                        results.push((block_name, b_start));
                    }
                }
                break;
            }
            if matches!(tok_res, Ok(IdlToken::Punct)) {
                match slice {
                    "{" => {
                        if depth == 0 {
                            brace_open_end = Some(span.end);
                        }
                        depth += 1;
                    }
                    "}" if depth > 0 => {
                        depth -= 1;
                        if depth == 0 {
                            brace_close_start = Some(span.start);
                        }
                    }
                    ";" if depth == 0 => break,
                    _ => {}
                }
            }
        }
    }
    results
}

/// Extracts `(type_name, field_name)` pairs from an IDL struct body.
/// Realises: [REQ-TRANSLATORS-IDL-SCAN-STRUCT-FIELDS]
fn scan_struct_fields(body: &str) -> Vec<(String, String)> {
    let (mut fields, mut stmt_tokens) = (Vec::new(), Vec::new());
    for (tok_res, span) in IdlToken::lexer(body).spanned() {
        let Some(slice) = body.get(span) else { continue };
        match tok_res {
            Ok(IdlToken::Whitespace | IdlToken::LineComment | IdlToken::BlockComment) => {}
            Ok(IdlToken::Punct) if slice == ";" => {
                if let Some((&(IdlToken::Ident | IdlToken::Keyword, f_name), type_parts)) =
                    stmt_tokens.split_last().filter(|(_, tp)| !tp.is_empty())
                {
                    if let Some(f_type) = join_idl_type_tokens(type_parts) {
                        fields.push((f_type, f_name.to_string()));
                    }
                }
                stmt_tokens.clear();
            }
            Ok(tok) => stmt_tokens.push((tok, slice)),
            Err(()) => stmt_tokens.clear(),
        }
    }
    fields
}

/// Extracts `(operation_name, params_raw)` pairs from an IDL interface body.
/// Realises: [REQ-TRANSLATORS-IDL-SCAN-INTERFACE-OPS]
fn scan_interface_ops(if_body: &str) -> Vec<(String, String)> {
    let mut ops = Vec::new();
    let mut stmt_tokens: Vec<(IdlToken, std::ops::Range<usize>, &str)> = Vec::new();
    for (tok_res, span) in IdlToken::lexer(if_body).spanned() {
        let Some(slice) = if_body.get(span.clone()) else { continue };
        match tok_res {
            Ok(IdlToken::Whitespace | IdlToken::LineComment | IdlToken::BlockComment) => {}
            Ok(IdlToken::Punct) if slice == ";" => {
                let open_pos = stmt_tokens.iter().position(|&(t, _, s)| t == IdlToken::Punct && s == "(");
                let close_pos = stmt_tokens.iter().rposition(|&(t, _, s)| t == IdlToken::Punct && s == ")");
                if let (Some(o_idx), Some(c_idx)) = (open_pos, close_pos) {
                    if o_idx >= 2 && c_idx > o_idx && matches!(stmt_tokens[o_idx - 1].0, IdlToken::Ident | IdlToken::Keyword) {
                        if let Some(params_raw) = if_body.get(stmt_tokens[o_idx].1.end..stmt_tokens[c_idx].1.start) {
                            ops.push((stmt_tokens[o_idx - 1].2.to_string(), params_raw.trim().to_string()));
                        }
                    }
                }
                stmt_tokens.clear();
            }
            Ok(tok) => stmt_tokens.push((tok, span, slice)),
            Err(()) => stmt_tokens.clear(),
        }
    }
    ops
}

/// Extracts `(type_name, alias_name)` pairs from top-level IDL `typedef` statements.
/// Realises: [REQ-TRANSLATORS-IDL-SCAN-TYPEDEFS]
fn scan_typedefs(content: &str) -> Vec<(String, String)> {
    let (mut typedefs, mut in_typedef, mut stmt_tokens) = (Vec::new(), false, Vec::new());
    for (tok_res, span) in IdlToken::lexer(content).spanned() {
        let Some(slice) = content.get(span) else { continue };
        match tok_res {
            Ok(IdlToken::Whitespace | IdlToken::LineComment | IdlToken::BlockComment) => {}
            Ok(IdlToken::Keyword) if !in_typedef && slice == "typedef" => {
                in_typedef = true;
                stmt_tokens.clear();
            }
            Ok(IdlToken::Punct) if in_typedef && slice == ";" => {
                if let Some((&(IdlToken::Ident | IdlToken::Keyword, t_name), type_parts)) =
                    stmt_tokens.split_last().filter(|(_, tp)| !tp.is_empty())
                {
                    if let Some(t_type) = join_idl_type_tokens(type_parts) {
                        typedefs.push((t_type, t_name.to_string()));
                    }
                }
                in_typedef = false;
                stmt_tokens.clear();
            }
            Ok(IdlToken::Punct) if in_typedef && matches!(slice, "{" | "}") => in_typedef = false,
            Ok(tok) if in_typedef => stmt_tokens.push((tok, slice)),
            Err(()) if in_typedef => in_typedef = false,
            _ => {}
        }
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
        assert_eq!(pkg.attribute_defs.first().map(|a| a.name.as_str()), Some("Payload_Type"));
        assert_eq!(pkg.attribute_defs.first().map(|a| a.type_name.as_str()), Some("sequence<octet>"));
        assert_eq!(pkg.part_defs.len(), 2);
        let s0 = pkg.part_defs.iter().find(|p| p.name == "Struct_Zero").ok_or("missing Struct_Zero")?;
        assert_eq!(s0.attributes.len(), 2);
        let ifp = pkg.part_defs.iter().find(|p| p.name == "Interface_Prime").ok_or("missing Interface_Prime")?;
        assert_eq!(ifp.actions.len(), 1);
        let action = ifp.actions.first().ok_or("missing ExecuteCommand")?;
        assert_eq!(action.name, "ExecuteCommand");
        assert_eq!(action.in_params.len(), 1);
        assert_eq!(action.out_params.len(), 1);
        Ok(())
    }
}
