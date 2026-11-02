//! SysML v2 source parsing and comment stripping utilities.
//!
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C.
//! Realises: [REQ-SYSML-AST-MODULE], [DO-330-TQL-4-SYSML-AST]

use super::actions::parse_action_defs;
use super::connections_and_flows::{parse_connection_defs, parse_port_defs};
use super::expressions::parse_constraint_defs;
use super::identifiers::{
    extract_doc_comment_before, extract_ident, find_matching_brace, is_ident_char,
    is_word_boundary_after, is_word_boundary_before, mask_string_literals, skip_ws,
};
use super::interfaces_and_items::{parse_attribute_defs, parse_part_defs};
use super::package_model::SysmlModel;
use super::requirements::parse_requirement_defs;
use super::safety_and_fmeca::parse_hazard_defs;
use super::test_cases::parse_test_case_defs;

/// Parse an entire SysML v2 source text into a structured SysmlModel.
///
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-SYSML]
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
        } else {
            idx = pkg_pos + 7;
            continue;
        }
    }

    if let Some((package_name, doc, body)) = package_info {
        let mut parts = parse_part_defs(body);
        let ports = parse_port_defs(body);
        let actions = parse_action_defs(body);
        let constraints = parse_constraint_defs(body);
        let requirements = parse_requirement_defs(body);
        let attributes = parse_attribute_defs(body);
        let test_cases = parse_test_case_defs(body);
        let pkg_connections = parse_connection_defs(body);
        let pkg_hazards = parse_hazard_defs(body);

        if let Some(first_part) = parts.first_mut() {
            for conn in pkg_connections {
                if !first_part.connections.iter().any(|c| {
                    c.name == conn.name
                        && c.source_port == conn.source_port
                        && c.target_port == conn.target_port
                }) {
                    first_part.connections.push(conn);
                }
            }
            for haz in pkg_hazards {
                if !first_part.hazards.iter().any(|h| h.name == haz.name) {
                    first_part.hazards.push(haz);
                }
            }
        }

        SysmlModel {
            package_name: Some(package_name),
            doc,
            attributes,
            parts,
            ports,
            actions,
            constraints,
            requirements,
            test_cases,
        }
    } else {
        let mut parts = parse_part_defs(&clean);
        let ports = parse_port_defs(&clean);
        let actions = parse_action_defs(&clean);
        let constraints = parse_constraint_defs(&clean);
        let requirements = parse_requirement_defs(&clean);
        let attributes = parse_attribute_defs(&clean);
        let test_cases = parse_test_case_defs(&clean);
        let root_connections = parse_connection_defs(&clean);
        let root_hazards = parse_hazard_defs(&clean);

        if let Some(first_part) = parts.first_mut() {
            for conn in root_connections {
                if !first_part.connections.iter().any(|c| {
                    c.name == conn.name
                        && c.source_port == conn.source_port
                        && c.target_port == conn.target_port
                }) {
                    first_part.connections.push(conn);
                }
            }
            for haz in root_hazards {
                if !first_part.hazards.iter().any(|h| h.name == haz.name) {
                    first_part.hazards.push(haz);
                }
            }
        }

        SysmlModel {
            package_name: None,
            doc: None,
            attributes,
            parts,
            ports,
            actions,
            constraints,
            requirements,
            test_cases,
        }
    }
}

/// Parses a SysML v2 textual model into a `SysmlModel` (canonical alias for [`parse_sysml`]).
///
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C.
/// Realises: [REQ-SYSML-AST-PARSE-SYSML-MODEL]
pub fn parse_sysml_model(content: &str) -> SysmlModel {
    parse_sysml(content)
}

/// Strips C-style multi-line comments (`/* ... */`) and line comments (`// ...`) from SysML text without regex,
/// while preserving `doc /* ... */` documentation blocks and string literals.
///
/// Conforms to DO-178C Level A and ECSS-E-ST-40C.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/STRIP-COMMENTS]
pub fn strip_sysml_comments(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_str = false;
    let mut in_escape = false;
    while let Some(c) = chars.next() {
        if in_str {
            out.push(c);
            if in_escape {
                in_escape = false;
            } else if c == '\\' {
                in_escape = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(c);
            continue;
        }
        if c == '/' {
            if let Some(&'/') = chars.peek() {
                chars.next();
                for next_c in chars.by_ref() {
                    if next_c == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            } else if let Some(&'*') = chars.peek() {
                let trimmed_before = out.trim_end();
                let is_doc_block = trimmed_before.ends_with("doc")
                    && (trimmed_before.len() == 3
                        || !trimmed_before
                            .as_bytes()
                            .get(trimmed_before.len() - 4)
                            .copied()
                            .is_some_and(is_ident_char));
                chars.next();
                if is_doc_block {
                    out.push_str("/*");
                    while let Some(next_c) = chars.next() {
                        out.push(next_c);
                        if next_c == '*' && chars.peek() == Some(&'/') {
                            out.push('/');
                            chars.next();
                            break;
                        }
                    }
                } else {
                    out.push(' ');
                    while let Some(next_c) = chars.next() {
                        if next_c == '\n' {
                            out.push('\n');
                        }
                        if next_c == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                }
            } else {
                out.push(c);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Strips all line comments (`// ...`), block comments (`/* ... */`, including `doc /* ... */`),
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/STRIP-ALL-COMMENTS-AND-STRINGS]
pub(crate) fn strip_all_block_comments_and_strings(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_str = false;
    let mut in_escape = false;
    while let Some(c) = chars.next() {
        if in_str {
            if c == '\n' {
                out.push('\n');
            } else {
                out.push(' ');
            }
            if in_escape {
                in_escape = false;
            } else if c == '\\' {
                in_escape = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(' ');
            continue;
        }
        if c == '/' {
            if let Some(&'/') = chars.peek() {
                chars.next();
                for next_c in chars.by_ref() {
                    if next_c == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            } else if let Some(&'*') = chars.peek() {
                chars.next();
                out.push(' ');
                while let Some(next_c) = chars.next() {
                    if next_c == '\n' {
                        out.push('\n');
                    }
                    if next_c == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        break;
                    }
                }
            } else {
                out.push(c);
            }
        } else {
            out.push(c);
        }
    }
    out
}
