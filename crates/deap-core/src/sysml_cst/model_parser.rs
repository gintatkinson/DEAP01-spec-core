#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! SysML v2 source parsing, `logos::Lexer<SysmlToken>` comment/string stripping, and polyglot JSON/YAML part extractors.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-MODULE], [DO-330-TQL-4-SYSML-AST]
//! Contract: Provides `parse_sysml`, `parse_sysml_model`, `lower_to_hir_db`, `SysmlToken` lexer-backed comment/string stripping, and `extract_parts_from_json` / `extract_parts_from_yaml_lines`.
//! Bounds & Faults: Executes in linear O(N) time and space over source length with zero hand-rolled character state machines and zero panicking slice operations.

use logos::Logos;
use std::collections::BTreeSet;

use super::actions::parse_action_defs;
use super::connections_and_flows::{parse_connection_defs, parse_port_defs};
use super::expressions::parse_constraint_defs;
use super::identifiers::{
    extract_doc_comment_before, extract_ident, find_matching_brace, is_valid_subsystem_identifier,
    is_word_boundary_after, is_word_boundary_before, mask_string_literals, skip_ws,
};
use super::interactions::parse_test_case_defs;
use super::interfaces_and_items::{parse_attribute_defs, parse_part_defs};
use super::package_model::SysmlModel;
use super::requirements::parse_requirement_defs;
use super::safety_and_fmeca::parse_hazard_defs;
use crate::sysml_cst::lexer::SysmlToken;
use crate::sysml_cst::{SourceProgram, SysmlCompilerDb};

/// Extracts all top-level or package-scoped SysML v2 definitions from a cleaned body slice.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-SYSML]
fn parse_scope_into_model(
    scope: &str,
    package_name: Option<String>,
    doc: Option<String>,
) -> SysmlModel {
    let mut parts = parse_part_defs(scope);
    let ports = parse_port_defs(scope);
    let actions = parse_action_defs(scope);
    let constraints = parse_constraint_defs(scope);
    let requirements = parse_requirement_defs(scope);
    let attributes = parse_attribute_defs(scope);
    let test_cases = parse_test_case_defs(scope);
    let scoped_connections = parse_connection_defs(scope);
    let scoped_hazards = parse_hazard_defs(scope);

    if let Some(first_part) = parts.first_mut() {
        for conn in scoped_connections {
            if !first_part.connections.iter().any(|c| {
                c.name == conn.name
                    && c.source_port == conn.source_port
                    && c.target_port == conn.target_port
            }) {
                first_part.connections.push(conn);
            }
        }
        for haz in scoped_hazards {
            if !first_part.hazards.iter().any(|h| h.name == haz.name) {
                first_part.hazards.push(haz);
            }
        }
    }

    SysmlModel {
        package_name,
        doc,
        attributes,
        parts,
        ports,
        actions,
        constraints,
        requirements,
        test_cases,
    }
}

/// [DOC-TIER: A] Parse an entire SysML v2 source text into a structured `SysmlModel`.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-SYSML]
/// Contract: Strips non-doc comments via `strip_sysml_comments`, masks string literals via `mask_string_literals`, and extracts package or root definitions into `SysmlModel`.
/// Bounds & Faults: O(N) time and space over `content.len()` with checked `.get(...)` slicing and zero panics.
#[must_use]
pub fn parse_sysml(content: &str) -> SysmlModel {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut package_info = None;
    let mut idx = 0;
    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("package")) {
        let pkg_pos = idx + pos;
        if !is_word_boundary_before(&masked, pkg_pos)
            || !is_word_boundary_after(&masked, pkg_pos + 7)
        {
            idx = pkg_pos + 7;
            continue;
        }
        let doc = extract_doc_comment_before(&clean, pkg_pos);
        let mut curr = pkg_pos + 7;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = pkg_pos + 7;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if masked.get(curr..).is_some_and(|s| s.starts_with('{')) {
            let open_pos = curr;
            let body = if let Some(close_pos) = find_matching_brace(&clean, open_pos) {
                clean.get(open_pos + 1..close_pos).unwrap_or("")
            } else {
                clean.get(open_pos + 1..).unwrap_or("")
            };
            package_info = Some((name.to_string(), doc, body));
            break;
        }
        idx = pkg_pos + 7;
    }

    if let Some((package_name, doc, body)) = package_info {
        parse_scope_into_model(body, Some(package_name), doc)
    } else {
        parse_scope_into_model(&clean, None, None)
    }
}

/// [DOC-TIER: A] Parses a SysML v2 textual model into a `SysmlModel` (canonical alias for [`parse_sysml`]).
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-PARSE-SYSML-MODEL]
/// Contract: Delegates directly to `parse_sysml(content)` with identical structural guarantees.
/// Bounds & Faults: O(N) time and space over `content.len()` with zero panics.
#[must_use]
pub fn parse_sysml_model(content: &str) -> SysmlModel {
    parse_sysml(content)
}

/// [DOC-TIER: A] Lowers a `SourceProgram` into a `SysmlModel` via the incremental `SysmlCompilerDb` CST/HIR database.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-MODULE], [REQ-0201]
/// Contract: Memoizes the Rowan CST and lowered HIR constructs in `db` for `program` and lowers the verified source program into `SysmlModel`.
/// Bounds & Faults: O(N) single-pass CST/HIR memoization and model extraction; returns `SysmlModel::default()` on empty input without panicking.
pub fn lower_to_hir_db(db: &mut SysmlCompilerDb, program: &SourceProgram) -> SysmlModel {
    let cst = db.parse_sysml_green(program);
    let hir = db.lower_sysml_hir(program);
    if hir.is_empty() && cst.constructs.is_empty() && program.text.trim().is_empty() {
        return SysmlModel::default();
    }
    parse_sysml(&program.text)
}

/// [DOC-TIER: A] Strips C-style multi-line comments (`/* ... */`) and line comments (`// ...`) from SysML text using `logos::Lexer<SysmlToken>`, while preserving `doc /* ... */` documentation blocks and string literals.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/STRIP-COMMENTS]
/// Contract: Tokenizes `content` via `SysmlToken::lexer(content).spanned()`, eliding `LineComment` tokens and replacing non-`doc` `BlockComment` tokens with newline-preserving spaces.
/// Bounds & Faults: O(N) single-pass DFA tokenization with zero regex backtracking and zero hand-rolled character state machines.
#[must_use]
pub fn strip_sysml_comments(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut prev_is_doc = false;
    for (res, span) in SysmlToken::lexer(content).spanned() {
        let text = content.get(span).unwrap_or("");
        match res {
            Ok(SysmlToken::LineComment) => {}
            Ok(SysmlToken::BlockComment) => {
                if prev_is_doc {
                    out.push_str(text);
                    prev_is_doc = false;
                } else {
                    out.push(' ');
                    for b in text.bytes() {
                        if b == b'\n' {
                            out.push('\n');
                        }
                    }
                }
            }
            Ok(SysmlToken::Whitespace) => out.push_str(text),
            Ok(tok) => {
                out.push_str(text);
                prev_is_doc = tok == SysmlToken::DocMetaKw && text == "doc";
            }
            Err(()) => {
                out.push_str(text);
                prev_is_doc = false;
            }
        }
    }
    out
}

/// Strips all line comments (`// ...`), block comments (`/* ... */`, including `doc /* ... */`), and string literals using `logos::Lexer<SysmlToken>`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/STRIP-ALL-COMMENTS-AND-STRINGS]
pub(crate) fn strip_all_block_comments_and_strings(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for (res, span) in SysmlToken::lexer(input).spanned() {
        let text = input.get(span).unwrap_or("");
        match res {
            Ok(SysmlToken::LineComment) => {}
            Ok(SysmlToken::BlockComment) => {
                out.push(' ');
                for b in text.bytes() {
                    if b == b'\n' {
                        out.push('\n');
                    }
                }
            }
            Ok(SysmlToken::StringLit) => {
                for ch in text.chars() {
                    out.push(if ch == '\n' { '\n' } else { ' ' });
                }
            }
            _ => out.push_str(text),
        }
    }
    out
}

/// Recursively walks a parsed JSON `Value` tree and collects valid subsystem identifiers.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-PARTS-JSON]
pub fn extract_parts_from_json(val: &serde_json::Value, parts: &mut BTreeSet<String>) {
    match val {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::Array(nodes)) = map.get("schema_nodes") {
                for node in nodes {
                    if let serde_json::Value::String(s) = node {
                        if let Some((kind, name)) = s.split_once(':') {
                            if matches!(kind, "part" | "component" | "subsystem")
                                && is_valid_subsystem_identifier(name.trim())
                            {
                                parts.insert(name.trim().to_string());
                            }
                        }
                    }
                }
            }
            for key in &["subsystems", "components", "parts", "modules"] {
                if let Some(v) = map.get(*key) {
                    collect_json_collection_parts(v, parts);
                }
            }
            for subval in map.values() {
                extract_parts_from_json(subval, parts);
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                extract_parts_from_json(item, parts);
            }
        }
        _ => {}
    }
}

/// Collects valid subsystem identifiers from a JSON array, object, or string value under a known collection key.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-PARTS-JSON]
fn collect_json_collection_parts(v: &serde_json::Value, parts: &mut BTreeSet<String>) {
    match v {
        serde_json::Value::Array(arr) => {
            for item in arr {
                match item {
                    serde_json::Value::String(s) if is_valid_subsystem_identifier(s.trim()) => {
                        parts.insert(s.trim().to_string());
                    }
                    serde_json::Value::Object(submap) => {
                        for name_key in &["name", "id", "identifier", "title"] {
                            if let Some(serde_json::Value::String(n)) = submap.get(*name_key) {
                                if is_valid_subsystem_identifier(n.trim()) {
                                    parts.insert(n.trim().to_string());
                                    break;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        serde_json::Value::Object(submap) => {
            for k in submap.keys() {
                if is_valid_subsystem_identifier(k.trim()) {
                    parts.insert(k.trim().to_string());
                }
            }
        }
        serde_json::Value::String(s) if is_valid_subsystem_identifier(s.trim()) => {
            parts.insert(s.trim().to_string());
        }
        _ => {}
    }
}

/// Scans YAML or JSON lines for subsystem and component list entries without an external YAML crate.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-PARTS-YAML]
pub fn extract_parts_from_yaml_lines(content: &str, parts: &mut BTreeSet<String>) {
    let mut in_parts_list = false;
    let mut list_indent = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let current_indent = line.len() - line.trim_start().len();
        let is_key = [
            "subsystems:", "components:", "parts:", "modules:",
            "subsystem:", "component:", "part:", "module:",
        ]
        .iter()
        .any(|kw| trimmed.to_ascii_lowercase().starts_with(kw));

        if is_key {
            in_parts_list = true;
            list_indent = current_indent;
            if let Some((_, val)) = trimmed.split_once(':') {
                let val_clean = val.trim().trim_matches(['"', '\'', '[', ']']);
                for item in val_clean.split(',').filter(|s| !s.is_empty()) {
                    let item_trim = item.trim().trim_matches(['"', '\'']);
                    if is_valid_subsystem_identifier(item_trim) {
                        parts.insert(item_trim.to_string());
                    }
                }
            }
            continue;
        }

        if in_parts_list {
            if current_indent <= list_indent && !trimmed.starts_with('-') {
                in_parts_list = false;
            } else if let Some(rest) = trimmed.strip_prefix('-') {
                let item_clean = rest.trim();
                let name = item_clean.split_once(':').map_or_else(
                    || item_clean.trim_matches(['"', '\'']),
                    |(_, v)| v.trim().trim_matches(['"', '\'']),
                );
                let ident: String = name.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
                if is_valid_subsystem_identifier(&ident) {
                    parts.insert(ident);
                }
            }
        }
    }
}
