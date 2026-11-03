//! [DOC-TIER: A]
//! 1. Purpose: OMG IDL v4 lossless Concrete Syntax Tree (`rowan` + `logos`) parser and SysML v2 HIR/AST translator.
//! 2. Scope: Tokenizes and parses OMG IDL into `ParsedIdlCst` and lowers directly to `deap_core::PackageDef`.
//! 3. Invariants: Lossless source reconstruction (`root.text() == source`) and deterministic `PartDef`/`ActionDef`/`AttributeDef` lowering.
//! 4. Concurrency: Stateless pure parser and translator; all data structures are `Send` + `Sync`.
//! 5. Error Handling: Error-resilient lexing and CST lowering without panics or `.unwrap()`.
//! 6. Security & Safety: Pure safe Rust, zero unsafe blocks, checked `.get(...)` slicing.
//! 7. Cross-Platform: Standard CRLF/LF normalized lexing; Realises: [REQ-0201], [REQ-0001/IDLTranslator].

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use crate::table::sanitize_identifier;
use deap_core::{ActionDef, AttributeDef, PackageDef, PartDef};
use logos::Logos;

/// IDL lexer tokens. Realises: [REQ-0201]
#[derive(logos::Logos, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdlToken {
    #[regex(r"[ \t\r\n]+")]
    Whitespace,
    #[regex(r"//[^\n]*", allow_greedy = true)]
    LineComment,
    #[regex(r"/\*([^*]|\*+[^*/])*\*+/", allow_greedy = true)]
    BlockComment,
    #[regex(r"module|interface|struct|enum|typedef|const|exception|sequence|attribute|readonly|in|out|inout")]
    Keyword,
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", priority = 1)]
    Ident,
    #[regex(r"[0-9]+(\.[0-9]+)?")]
    NumberLit,
    #[regex(r#""([^"\\]|\\.)*""#, allow_greedy = true)]
    StringLit,
    #[regex(r"::|[{}();,<>:=\[\]+*/%-]")]
    Punct,
    Error,
}

/// IDL syntax kind for Rowan CST. Realises: [REQ-0201]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum IdlSyntaxKind {
    Whitespace = 0,
    LineComment,
    BlockComment,
    Keyword,
    Ident,
    NumberLit,
    StringLit,
    Punct,
    ErrorToken,
    SourceFile,
}

/// IDL Rowan language definition. Realises: [REQ-0201]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdlLanguage {}

impl rowan::Language for IdlLanguage {
    type Kind = IdlSyntaxKind;

    /// Maps raw syntax kind to `IdlSyntaxKind`. Realises: [REQ-0201]
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        match raw.0 {
            0 => IdlSyntaxKind::Whitespace,
            1 => IdlSyntaxKind::LineComment,
            2 => IdlSyntaxKind::BlockComment,
            3 => IdlSyntaxKind::Keyword,
            4 => IdlSyntaxKind::Ident,
            5 => IdlSyntaxKind::NumberLit,
            6 => IdlSyntaxKind::StringLit,
            7 => IdlSyntaxKind::Punct,
            9 => IdlSyntaxKind::SourceFile,
            _ => IdlSyntaxKind::ErrorToken,
        }
    }

    /// Maps `IdlSyntaxKind` to raw syntax kind. Realises: [REQ-0201]
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}

/// Summary of an IDL construct. Realises: [REQ-0201]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdlNodeSummary {
    pub kind: String,
    pub name: String,
    pub span_start: usize,
    pub span_end: usize,
}

/// Parsed IDL CST output. Realises: [REQ-0201]
#[derive(Debug, Clone)]
pub struct ParsedIdlCst {
    pub green: rowan::GreenNode,
    pub nodes: Vec<IdlNodeSummary>,
    pub lossless_roundtrip_eq: bool,
}

const TARGET_KEYWORDS: &[&str] = &[
    "module", "interface", "struct", "enum", "typedef", "const", "exception", "attribute",
];

/// Maps token to syntax kind. Realises: [REQ-0201]
const fn token_to_syntax_kind(token: IdlToken) -> IdlSyntaxKind {
    match token {
        IdlToken::Whitespace => IdlSyntaxKind::Whitespace,
        IdlToken::LineComment => IdlSyntaxKind::LineComment,
        IdlToken::BlockComment => IdlSyntaxKind::BlockComment,
        IdlToken::Keyword => IdlSyntaxKind::Keyword,
        IdlToken::Ident => IdlSyntaxKind::Ident,
        IdlToken::NumberLit => IdlSyntaxKind::NumberLit,
        IdlToken::StringLit => IdlSyntaxKind::StringLit,
        IdlToken::Punct => IdlSyntaxKind::Punct,
        IdlToken::Error => IdlSyntaxKind::ErrorToken,
    }
}

/// Parses IDL source into lossless CST. Realises: [REQ-0201]
#[must_use]
pub fn parse_idl_cst(source: &str) -> ParsedIdlCst {
    let mut builder = rowan::GreenNodeBuilder::new();
    builder.start_node(<IdlLanguage as rowan::Language>::kind_to_raw(IdlSyntaxKind::SourceFile));
    let mut nodes = Vec::new();
    let mut pending_kw: Option<(String, usize)> = None;
    for (token_res, span) in logos::Logos::lexer(source).spanned() {
        let token = token_res.unwrap_or(IdlToken::Error);
        let syntax_kind = token_to_syntax_kind(token);
        let slice = source.get(span.clone()).unwrap_or("");
        builder.token(<IdlLanguage as rowan::Language>::kind_to_raw(syntax_kind), slice);
        match token {
            IdlToken::Keyword if TARGET_KEYWORDS.contains(&slice) => {
                pending_kw = Some((slice.to_owned(), span.start));
            }
            IdlToken::Ident => {
                if let Some((kw_kind, kw_start)) = pending_kw.take() {
                    nodes.push(IdlNodeSummary {
                        kind: kw_kind,
                        name: slice.to_owned(),
                        span_start: kw_start,
                        span_end: span.end,
                    });
                }
            }
            IdlToken::Whitespace | IdlToken::LineComment | IdlToken::BlockComment => {}
            _ => pending_kw = None,
        }
    }
    builder.finish_node();
    let green = builder.finish();
    let root = rowan::SyntaxNode::<IdlLanguage>::new_root(green.clone());
    let lossless_roundtrip_eq = root.text() == source;
    ParsedIdlCst { green, nodes, lossless_roundtrip_eq }
}

/// Deterministic CST-to-HIR translator for OMG IDL specifications. Realises: [REQ-0001/IDLTranslator]
#[derive(Debug, Default, Clone)]
pub struct IDLTranslator;

impl IDLTranslator {
    /// Creates a fresh instance of the IDL CST translator. Realises: [REQ-TRANSLATORS-IDL-IDLTRANSLATOR-NEW]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Translates OMG IDL source text into a canonical SysML v2 `PackageDef` via `parse_idl_cst`. Realises: [REQ-0001/IDLTranslator_translate]
    pub fn translate(&self, content: &str, default_name: &str) -> Result<PackageDef, String> {
        let cst = parse_idl_cst(content);
        let pkg_name = cst
            .nodes
            .iter()
            .find(|n| n.kind == "module")
            .map_or_else(
                || sanitize_identifier(default_name, "IDL_Package"),
                |n| sanitize_identifier(&n.name, default_name),
            );
        let mut pkg = PackageDef {
            name: pkg_name,
            doc: Some("Translated from OMG IDL schema".to_string()),
            ..Default::default()
        };
        for (s_raw_name, s_body) in scan_idl_blocks(content, &cst, "struct") {
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
        for (if_raw_name, if_body) in scan_idl_blocks(content, &cst, "interface") {
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
        for (type_name, t_raw_name) in scan_typedefs(content) {
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

/// Parses an IDL operation parameter list into `(in_params, out_params)` via `IdlToken::lexer`. Realises: [REQ-0001/IDLTranslator_translate]
fn parse_idl_op_params(params_raw: &str) -> (Vec<AttributeDef>, Vec<AttributeDef>) {
    let (mut in_params, mut out_params, mut cur, mut groups) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for (tok_res, span) in IdlToken::lexer(params_raw).spanned() {
        let Some(slice) = params_raw.get(span) else { continue };
        match tok_res {
            Ok(IdlToken::Whitespace | IdlToken::LineComment | IdlToken::BlockComment) | Err(()) => {}
            Ok(IdlToken::Punct) if slice == "," => {
                if !cur.is_empty() {
                    groups.push(std::mem::take(&mut cur));
                }
            }
            Ok(tok) => cur.push((tok, slice)),
        }
    }
    if !cur.is_empty() {
        groups.push(cur);
    }
    for group in groups {
        let Some((&(_, raw_name), prefix)) = group.split_last() else { continue };
        let (direction, type_tokens) = match prefix.split_first() {
            Some((&(_, dir @ ("in" | "out" | "inout")), rest)) if !rest.is_empty() => (dir, rest),
            Some(_) => ("in", prefix),
            None => continue,
        };
        let Some(type_name) = join_idl_type_tokens(type_tokens) else { continue };
        let attr = AttributeDef { name: sanitize_identifier(raw_name, "param"), type_name, default_value: None, doc: None };
        if direction == "out" { out_params.push(attr); } else { in_params.push(attr); }
    }
    (in_params, out_params)
}

/// Joins tokenized IDL type segments into a canonical type string. Realises: [REQ-TRANSLATORS-IDL-SCAN-STRUCT-FIELDS]
fn join_idl_type_tokens(tokens: &[(IdlToken, &str)]) -> Option<String> {
    let (mut out, mut prev_was_word) = (String::new(), false);
    for &(tok, slice) in tokens {
        let is_word = matches!(tok, IdlToken::Ident | IdlToken::Keyword);
        let is_punct = matches!(tok, IdlToken::Punct) && matches!(slice, "<" | ">" | "::" | "-");
        if !is_word && !is_punct { return None; }
        if prev_was_word && is_word { out.push(' '); }
        out.push_str(slice);
        prev_was_word = is_word;
    }
    (!out.is_empty()).then_some(out)
}

/// Extracts named `{ ... };` blocks for a target IDL keyword (`struct` or `interface`) from `ParsedIdlCst`. Realises: [REQ-TRANSLATORS-IDL-SCAN-IDL-BLOCKS]
fn scan_idl_blocks<'a>(content: &'a str, cst: &ParsedIdlCst, keyword: &str) -> Vec<(&'a str, &'a str)> {
    let mut results = Vec::new();
    for node in cst.nodes.iter().filter(|n| n.kind == keyword) {
        let name_start = node.span_end.saturating_sub(node.name.len());
        let (Some(block_name), Some(rest)) = (content.get(name_start..node.span_end).filter(|s| !s.is_empty()), content.get(node.span_end..)) else { continue };
        let (mut brace_open_end, mut brace_close_start, mut depth) = (None, None, 0usize);
        for (tok_res, span) in IdlToken::lexer(rest).spanned() {
            let Some(slice) = rest.get(span.clone()) else { continue };
            if let Some(b_end) = brace_close_start {
                if matches!(tok_res, Ok(IdlToken::Whitespace | IdlToken::LineComment | IdlToken::BlockComment)) { continue; }
                if matches!(tok_res, Ok(IdlToken::Punct)) && slice == ";" {
                    if let Some(b_start) = brace_open_end.and_then(|s| rest.get(s..b_end)) { results.push((block_name, b_start)); }
                }
                break;
            }
            if matches!(tok_res, Ok(IdlToken::Punct)) {
                match slice {
                    "{" => { if depth == 0 { brace_open_end = Some(span.end); } depth += 1; }
                    "}" if depth > 0 => { depth -= 1; if depth == 0 { brace_close_start = Some(span.start); } }
                    ";" if depth == 0 => break,
                    _ => {}
                }
            }
        }
    }
    results
}

/// Extracts `(type_name, field_name)` pairs from an IDL struct body. Realises: [REQ-TRANSLATORS-IDL-SCAN-STRUCT-FIELDS]
fn scan_struct_fields(body: &str) -> Vec<(String, String)> {
    let (mut fields, mut stmt_tokens) = (Vec::new(), Vec::new());
    for (tok_res, span) in IdlToken::lexer(body).spanned() {
        let Some(slice) = body.get(span) else { continue };
        match tok_res {
            Ok(IdlToken::Whitespace | IdlToken::LineComment | IdlToken::BlockComment) => {}
            Ok(IdlToken::Punct) if slice == ";" => {
                if let Some((&(IdlToken::Ident | IdlToken::Keyword, f_name), type_parts)) = stmt_tokens.split_last().filter(|(_, tp)| !tp.is_empty()) {
                    if let Some(f_type) = join_idl_type_tokens(type_parts) { fields.push((f_type, f_name.to_string())); }
                }
                stmt_tokens.clear();
            }
            Ok(tok) => stmt_tokens.push((tok, slice)),
            Err(()) => stmt_tokens.clear(),
        }
    }
    fields
}

/// Extracts `(operation_name, params_raw)` pairs from an IDL interface body. Realises: [REQ-TRANSLATORS-IDL-SCAN-INTERFACE-OPS]
fn scan_interface_ops(if_body: &str) -> Vec<(String, String)> {
    let (mut ops, mut stmt_tokens) = (Vec::new(), Vec::<(IdlToken, std::ops::Range<usize>, &str)>::new());
    for (tok_res, span) in IdlToken::lexer(if_body).spanned() {
        let Some(slice) = if_body.get(span.clone()) else { continue };
        match tok_res {
            Ok(IdlToken::Whitespace | IdlToken::LineComment | IdlToken::BlockComment) => {}
            Ok(IdlToken::Punct) if slice == ";" => {
                let open_pos = stmt_tokens.iter().position(|&(t, _, s)| t == IdlToken::Punct && s == "(");
                let close_pos = stmt_tokens.iter().rposition(|&(t, _, s)| t == IdlToken::Punct && s == ")");
                if let (Some(o_idx), Some(c_idx)) = (open_pos, close_pos) {
                    if let (Some(&(IdlToken::Ident | IdlToken::Keyword, _, op_name)), Some((_, o_sp, _)), Some((_, c_sp, _))) = (o_idx.checked_sub(1).filter(|&i| i >= 1 && c_idx > o_idx).and_then(|i| stmt_tokens.get(i)), stmt_tokens.get(o_idx), stmt_tokens.get(c_idx)) {
                        if let Some(params_raw) = if_body.get(o_sp.end..c_sp.start) { ops.push((op_name.to_string(), params_raw.trim().to_string())); }
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

/// Extracts `(type_name, alias_name)` pairs from top-level IDL `typedef` statements. Realises: [REQ-TRANSLATORS-IDL-SCAN-TYPEDEFS]
fn scan_typedefs(content: &str) -> Vec<(String, String)> {
    let (mut typedefs, mut in_typedef, mut stmt_tokens) = (Vec::new(), false, Vec::new());
    for (tok_res, span) in IdlToken::lexer(content).spanned() {
        let Some(slice) = content.get(span) else { continue };
        match tok_res {
            Ok(IdlToken::Whitespace | IdlToken::LineComment | IdlToken::BlockComment) => {}
            Ok(IdlToken::Keyword) if !in_typedef && slice == "typedef" => { in_typedef = true; stmt_tokens.clear(); }
            Ok(IdlToken::Punct) if in_typedef && slice == ";" => {
                if let Some((&(IdlToken::Ident | IdlToken::Keyword, t_name), type_parts)) = stmt_tokens.split_last().filter(|(_, tp)| !tp.is_empty()) {
                    if let Some(t_type) = join_idl_type_tokens(type_parts) { typedefs.push((t_type, t_name.to_string())); }
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
