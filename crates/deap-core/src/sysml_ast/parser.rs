#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! SysML v2 source parsing and `logos::Lexer<SysmlToken>` comment/string stripping utilities.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-MODULE], [DO-330-TQL-4-SYSML-AST]
//! Contract: Provides `parse_sysml`, `parse_sysml_model`, `lower_to_hir_db`, and `SysmlToken` lexer-backed comment/string stripping for SysML v2 source text.
//! Bounds & Faults: Executes in linear O(N) time and space over source length with zero hand-rolled character state machines and zero panicking slice operations.

use logos::Logos;

use super::actions::parse_action_defs;
use super::connections_and_flows::{parse_connection_defs, parse_port_defs};
use super::expressions::parse_constraint_defs;
use super::identifiers::{
    extract_doc_comment_before, extract_ident, find_matching_brace, is_word_boundary_after,
    is_word_boundary_before, mask_string_literals, skip_ws,
};
use super::interfaces_and_items::{parse_attribute_defs, parse_part_defs};
use super::package_model::SysmlModel;
use super::requirements::parse_requirement_defs;
use super::safety_and_fmeca::parse_hazard_defs;
use super::test_cases::parse_test_case_defs;
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
