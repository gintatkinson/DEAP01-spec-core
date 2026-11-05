//! SysML v2 subsystem part and port extraction with OEM provenance.

use std::collections::{BTreeMap, BTreeSet};

use super::sysml_lexer::{
    check_direction_word, extract_doc_comment_before, extract_inner_doc, extract_numeric_substring,
    find_all_attributes, find_line_number, find_matching_brace, is_ident_char,
};
use super::{SubsystemPart, SubsystemPort};

/// Advances `curr` past ASCII whitespace and an optional `def` keyword (`[REQ-ASSEMBLE-CONOPS]`).
fn skip_ws_and_optional_def(text: &str, bytes: &[u8], mut curr: usize) -> usize {
    while bytes.get(curr).is_some_and(u8::is_ascii_whitespace) {
        curr += 1;
    }
    if text.get(curr..).is_some_and(|s| s.starts_with("def"))
        && !bytes.get(curr + 3).copied().is_some_and(is_ident_char)
    {
        curr += 3;
        while bytes.get(curr).is_some_and(u8::is_ascii_whitespace) {
            curr += 1;
        }
    }
    curr
}

/// Parses a single `port` declaration starting at `port_idx` inside `body` (`[REQ-ASSEMBLE-CONOPS]`).
fn parse_single_port_at(
    content: &str,
    body: &str,
    body_offset: usize,
    filepath: &str,
    port_idx: usize,
) -> Option<(SubsystemPort, usize)> {
    let bytes = body.as_bytes();
    let before = body.get(..port_idx).unwrap_or("");
    let (mut direction, stmt_offset) = check_direction_word(before)
        .map(|(dir, start)| (dir.to_string(), start))
        .unwrap_or_else(|| ("INOUT".to_string(), port_idx));

    let mut curr = skip_ws_and_optional_def(body, bytes, port_idx + 4);
    if direction == "INOUT" {
        for (kw, dir_label) in [("inout", "INOUT"), ("out", "OUT"), ("in", "IN")] {
            if body.get(curr..).is_some_and(|s| s.starts_with(kw))
                && !bytes.get(curr + kw.len()).copied().is_some_and(is_ident_char)
            {
                direction = dir_label.to_string();
                curr += kw.len();
                while bytes.get(curr).is_some_and(u8::is_ascii_whitespace) {
                    curr += 1;
                }
                break;
            }
        }
    }

    let name_start = curr;
    while bytes.get(curr).copied().is_some_and(is_ident_char) {
        curr += 1;
    }
    let port_name = body.get(name_start..curr).unwrap_or("").to_string();
    if port_name.is_empty() {
        return None;
    }

    while bytes.get(curr).is_some_and(u8::is_ascii_whitespace) {
        curr += 1;
    }
    let mut type_name = "Port".to_string();
    if bytes.get(curr) == Some(&b':') {
        curr += 1;
        while bytes.get(curr).is_some_and(u8::is_ascii_whitespace) {
            curr += 1;
        }
        let type_start = curr;
        while bytes.get(curr).copied().is_some_and(is_ident_char) {
            curr += 1;
        }
        let t = body.get(type_start..curr).unwrap_or("").trim().to_string();
        if !t.is_empty() {
            type_name = t;
        }
    }

    let source_line = find_line_number(content, body_offset + stmt_offset);
    Some((
        SubsystemPort {
            name: port_name,
            direction,
            type_name,
            doc: String::new(),
            source_file: filepath.to_string(),
            source_line,
            epistemic_tier: "[TIER-1: OEM]".to_string(),
        },
        curr,
    ))
}

/// [DOC-TIER: A] Parses subsystem port declarations from a SysML `part def` block body.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-PARSE-SUBSYSTEM-PORTS-FROM-BODY] | SysML v2: ActionDef(parse_subsystem_ports_from_body)
/// Contract: Extracts `SubsystemPort` entries with direction (`IN`, `OUT`, `INOUT`), type, and 1-indexed source line from `body: &str`.
/// Bounds & Faults: O(N) time and space over `body.len()`; preserves OEM provenance `[TIER-1: OEM]`.
pub fn parse_subsystem_ports_from_body(
    content: &str,
    body: &str,
    body_offset: usize,
    filepath: &str,
) -> Vec<SubsystemPort> {
    let mut ports = Vec::new();
    let bytes = body.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if let Some(rel_pos) = body.get(i..).and_then(|s| s.find("port")) {
            let port_idx = i + rel_pos;
            let left_ok =
                port_idx == 0 || !bytes.get(port_idx - 1).copied().is_some_and(is_ident_char);
            let right_ok = !bytes.get(port_idx + 4).copied().is_some_and(is_ident_char);
            if !left_ok || !right_ok {
                i = port_idx + 4;
                continue;
            }
            if let Some((port, next_i)) =
                parse_single_port_at(content, body, body_offset, filepath, port_idx)
            {
                ports.push(port);
                i = next_i;
            } else {
                i = port_idx + 4;
            }
        } else {
            break;
        }
    }
    ports
}

/// Extracts attributes, mass (`kg`), power (`W`), and action names from a SysML `part` body (`[REQ-ASSEMBLE-CONOPS]`).
fn extract_part_attributes_and_actions(
    body: &str,
) -> (BTreeMap<String, String>, Option<f64>, Option<f64>, Vec<String>) {
    let mut attributes = BTreeMap::new();
    let mut mass_kg = None;
    let mut power_w = None;
    let mut actions = Vec::new();
    if !body.is_empty() {
        for (attr_name, raw_val) in find_all_attributes(body) {
            attributes.insert(attr_name.to_string(), raw_val.to_string());
            let lower_attr = attr_name.to_ascii_lowercase();
            if let Some(val_f64) = extract_numeric_substring(raw_val).and_then(|s| s.parse::<f64>().ok()) {
                if (lower_attr.contains("mass") || lower_attr.contains("weight")) && mass_kg.is_none() {
                    mass_kg = Some(val_f64);
                } else if (lower_attr.contains("power") || lower_attr.contains("watt")) && power_w.is_none() {
                    power_w = Some(val_f64);
                }
            }
        }
        for act in deap_core::sysml_ast::parse_action_defs(body) {
            actions.push(act.name);
        }
    }
    (attributes, mass_kg, power_w, actions)
}

/// Parses a single `part` or `part def` declaration starting at `part_pos` (`[REQ-ASSEMBLE-CONOPS]`).
fn parse_single_part_at(
    content: &str,
    filepath: &str,
    part_pos: usize,
) -> Option<(SubsystemPart, usize)> {
    let bytes = content.as_bytes();
    let mut curr = skip_ws_and_optional_def(content, bytes, part_pos + 4);
    let name_start = curr;
    while bytes.get(curr).copied().is_some_and(is_ident_char) {
        curr += 1;
    }
    let name = content.get(name_start..curr).unwrap_or("").trim().to_string();
    if name.is_empty() || name == "def" {
        return None;
    }

    while bytes.get(curr).is_some_and(u8::is_ascii_whitespace) {
        curr += 1;
    }
    if bytes.get(curr) == Some(&b':') {
        curr += 1;
        while bytes.get(curr).is_some_and(u8::is_ascii_whitespace) {
            curr += 1;
        }
        while bytes.get(curr).copied().is_some_and(is_ident_char) {
            curr += 1;
        }
        while bytes.get(curr).is_some_and(u8::is_ascii_whitespace) {
            curr += 1;
        }
    }

    let (body, next_idx) = if bytes.get(curr) == Some(&b'{') {
        if let Some(close_pos) = find_matching_brace(content, curr) {
            (content.get(curr + 1..close_pos).unwrap_or(""), close_pos + 1)
        } else {
            (content.get(curr + 1..).unwrap_or(""), bytes.len())
        }
    } else if bytes.get(curr) == Some(&b';') {
        ("", curr + 1)
    } else {
        ("", part_pos + 4)
    };

    let mut doc = extract_inner_doc(body);
    if doc.is_empty() {
        doc = extract_doc_comment_before(content, part_pos);
    }
    let body_offset = if bytes.get(curr) == Some(&b'{') { curr + 1 } else { part_pos };
    let ports = if !body.is_empty() {
        parse_subsystem_ports_from_body(content, body, body_offset, filepath)
    } else {
        Vec::new()
    };
    let (attributes, mass_kg, power_w, actions) = extract_part_attributes_and_actions(body);

    Some((
        SubsystemPart {
            name,
            doc,
            ports,
            mass_kg,
            power_w,
            actions,
            attributes,
            constraints: Vec::new(),
            source_file: filepath.to_string(),
            source_line: find_line_number(content, part_pos),
            epistemic_tier: "[TIER-1: OEM]".to_string(),
        },
        next_idx.max(part_pos + 4),
    ))
}

/// [DOC-TIER: A] Extracts `SubsystemPart` definitions with ports, attributes, actions, and source provenance from SysML text.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-SUBSYSTEM-PARTS-FROM-SYSML] | SysML v2: ActionDef(extract_subsystem_parts_from_sysml)
/// Contract: Scans `content: &str` for `part` / `part def` declarations and returns deduplicated `Vec<SubsystemPart>`.
/// Bounds & Faults: O(N) time and space over `content.len()`; records 1-indexed `source_line` and `[TIER-1: OEM]` provenance.
pub fn extract_subsystem_parts_from_sysml(content: &str, filepath: &str) -> Vec<SubsystemPart> {
    let mut parts = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    let mut seen = BTreeSet::new();

    while i < bytes.len() {
        if let Some(rel_pos) = content.get(i..).and_then(|s| s.find("part")) {
            let part_pos = i + rel_pos;
            let left_ok =
                part_pos == 0 || !bytes.get(part_pos - 1).copied().is_some_and(is_ident_char);
            let right_ok = !bytes.get(part_pos + 4).copied().is_some_and(is_ident_char);
            if !left_ok || !right_ok {
                i = part_pos + 4;
                continue;
            }
            if let Some((part, next_idx)) = parse_single_part_at(content, filepath, part_pos) {
                if seen.insert(part.name.clone()) {
                    parts.push(part);
                }
                i = next_idx;
            } else {
                i = part_pos + 4;
            }
        } else {
            break;
        }
    }
    parts
}
