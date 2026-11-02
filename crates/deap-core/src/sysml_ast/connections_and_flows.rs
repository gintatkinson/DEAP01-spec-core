//! Topological connection, item-flow, and port boundary AST definitions and parsers for SysML v2.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-CONNECTIONS-AND-FLOWS], [DO-330-TQL-4-SYSML-AST]
//! Contract: Defines `ConnectionDef` and provides deterministic, zero-regex AST extractors (`parse_connection_defs` and `parse_port_defs`) for topological connections, directed `FlowDef` payloads, and `PortDef` interface boundaries.
//! Bounds & Faults: All parsers execute in linear O(N) time over comment-stripped buffers using bounds-checked `.get` slicing, saturating offset arithmetic, `BTreeMap` attribute maps, and zero panics on malformed inputs.

use super::identifiers::{
    default_one, default_port_category, extract_doc_comment_before, extract_ident,
    extract_type_ident, find_matching_brace, is_word_boundary_after, is_word_boundary_before,
    mask_string_literals, skip_ws, FlowDef,
};
use super::{parse_attribute_defs, strip_sysml_comments, PortDef};
use crate::sysml_cst::lexer::SysmlToken;
use logos::Logos;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Represents a connection or topological flow link between ports in SysML v2.
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-CONNECTIONDEF]
/// Contract: Captures connection identifier, `source_port` and `target_port` endpoints, optional owning `source_part` and `target_part` prefixes, `severity` (defaulting to `1`), `item_flow_ref`, `protocol`, `latency_ms`, and `is_flow`.
/// Bounds & Faults: O(1) structural record construction and deterministic `serde` round-trip defaults with zero fallible state mutations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ConnectionDef {
    pub name: String,
    pub source_port: String,
    pub target_port: String,
    pub source_part: Option<String>,
    pub target_part: Option<String>,
    pub doc: Option<String>,
    #[serde(default = "default_one")] pub severity: i32,
    pub item_flow_ref: Option<String>,
    pub protocol: Option<String>,
    pub latency_ms: Option<f64>,
    #[serde(default)] pub is_flow: bool,
}

/// Extracts either a brace-delimited `{ ... }` connection body or an inline `connect ...;` clause starting at `curr`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CONNECTION-DEFS]
fn extract_connection_body<'a>(clean: &'a str, masked: &str, curr: usize) -> Option<(&'a str, usize)> {
    let rem_conn = masked.get(curr..)?;
    if rem_conn.starts_with('{') {
        let close_pos = find_matching_brace(clean, curr)?;
        Some((clean.get(curr.saturating_add(1)..close_pos).unwrap_or(""), close_pos.saturating_add(1)))
    } else if rem_conn.starts_with("connect") && is_word_boundary_after(rem_conn, 7) {
        let end_pos = curr.saturating_add(rem_conn.find(';')?).saturating_add(1);
        Some((clean.get(curr..end_pos).unwrap_or(""), end_pos))
    } else {
        None
    }
}

/// Scans lines inside a connection or flow block for the first `connect <src> to <tgt>` or `flow from <src> to <tgt>` clause.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CONNECTION-DEFS]
fn extract_connection_endpoints(body: &str) -> (String, String) {
    for line in body.lines() {
        let mut iter = SysmlToken::lexer(line).spanned().filter(|(t, _)| !matches!(t, Ok(SysmlToken::Whitespace)));
        let Some((_, first_sp)) = iter.next() else { continue };
        let after_start = match line.get(first_sp.clone()) {
            Some("connect") => Some(first_sp.end),
            Some("flow") => iter.next().filter(|(_, sp)| line.get(sp.clone()) == Some("from")).map(|(_, sp)| sp.end),
            _ => None,
        };
        if let Some((raw_src, raw_tgt)) = after_start.and_then(|pos| line.get(pos..)).and_then(|s| s.split_once(" to ")) {
            return (raw_src.trim().to_string(), raw_tgt.trim().trim_end_matches(';').trim().to_string());
        }
    }
    (String::new(), String::new())
}

/// Parses topological `connection` and `flow` definitions from SysML v2 source text.
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CONNECTION-DEFS]
/// Contract: Extracts every `connection [def]` and top-level `flow [def]` block or inline `connect <src> to <tgt>;` declaration in appearance order while ignoring inner `flow from ...` body statements.
/// Bounds & Faults: Executes in O(N) time and O(N) space over `content.len()` with checked `.get` slicing; malformed or unclosed declarations are skipped without panicking.
pub fn parse_connection_defs(content: &str) -> Vec<ConnectionDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut conns = Vec::new();
    let mut skip_until: usize = 0;

    for (res, span) in SysmlToken::lexer(&masked).spanned() {
        if span.start < skip_until || res.is_err() {
            continue;
        }
        let kw = masked.get(span.clone()).unwrap_or("");
        if kw != "connection" && kw != "flow" {
            continue;
        }
        let is_flow = kw == "flow";
        let mut curr = span.end.saturating_add(skip_ws(masked.get(span.end..).unwrap_or("")));
        if is_flow && masked.get(curr..).is_some_and(|s| s.starts_with("from") && is_word_boundary_after(s, 4)) {
            continue;
        }
        let doc = extract_doc_comment_before(&clean, span.start);
        if masked.get(curr..).is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3)) {
            curr = curr.saturating_add(3);
            curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        }
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            continue;
        }
        curr = curr.saturating_add(name_len);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        let Some((body, next_idx)) = extract_connection_body(&clean, &masked, curr) else { continue };
        skip_until = next_idx;
        let (source_port, target_port) = extract_connection_endpoints(body);
        let source_part = source_port.split_once('.').map(|(p, _)| p.to_string());
        let target_part = target_port.split_once('.').map(|(p, _)| p.to_string());
        conns.push(ConnectionDef {
            name: name.to_string(),
            source_port,
            target_port,
            source_part,
            target_part,
            doc,
            severity: 1,
            is_flow,
            ..ConnectionDef::default()
        });
    }
    conns
}

/// Parses a single item-flow declaration line inside a SysML v2 port block into a `FlowDef`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PORT-FLOW-LINE]
fn parse_port_flow_line(line: &str) -> Option<FlowDef> {
    let trimmed = line.trim().trim_end_matches(';').trim();
    if trimmed.is_empty() || (trimmed.starts_with("attribute") && is_word_boundary_after(trimmed, 9)) {
        return None;
    }
    let mut rest = trimmed;
    let mut direction = None;
    let mut saw_flow_kw = false;
    if rest.starts_with("flow") && is_word_boundary_after(rest, 4) {
        saw_flow_kw = true;
        rest = rest.get(4..).unwrap_or("").trim_start();
    }
    if let Some(dir_kw) = ["inout", "out", "in"]
        .into_iter()
        .find(|kw| rest.starts_with(kw) && is_word_boundary_after(rest, kw.len()))
    {
        direction = Some(dir_kw.to_string());
        rest = rest.get(dir_kw.len()..).unwrap_or("").trim_start();
    }
    if (rest.starts_with("flow") || rest.starts_with("item")) && is_word_boundary_after(rest, 4) {
        saw_flow_kw = true;
        rest = rest.get(4..).unwrap_or("").trim_start();
    }
    if !saw_flow_kw && direction.is_none() {
        return None;
    }
    let (name, name_len) = extract_ident(rest);
    if name.is_empty() {
        return None;
    }
    let item_type = rest
        .get(name_len..)
        .unwrap_or("")
        .trim_start()
        .strip_prefix(':')
        .map(|s| extract_type_ident(s.trim_start()).0)
        .filter(|t| !t.is_empty())
        .unwrap_or("Item")
        .to_string();
    Some(FlowDef {
        name: name.to_string(),
        direction: direction.unwrap_or_else(|| "out".to_string()),
        item_type,
        ..FlowDef::default()
    })
}

/// Inspects tokens immediately preceding `port_pos` for a leading `inout`, `out`, or `in` direction modifier.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PORT-DEFS]
fn extract_leading_port_direction(masked: &str, port_pos: usize) -> (Option<&'static str>, usize) {
    let before = masked.get(..port_pos).unwrap_or("").trim_end();
    ["inout", "out", "in"]
        .into_iter()
        .find_map(|kw| {
            let pos = before.len().saturating_sub(kw.len());
            (before.ends_with(kw) && is_word_boundary_before(before, pos)).then_some((Some(kw), pos))
        })
        .unwrap_or((None, port_pos))
}

/// Consumes optional `def` and inline direction (`inout`, `out`, `in`) modifiers following the `port` keyword.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PORT-DEFS]
fn consume_optional_direction_and_def(masked: &str, start: usize) -> (Option<&'static str>, usize) {
    let mut curr = start;
    let mut dir2 = None;
    for _ in 0..2 {
        if masked.get(curr..).is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3)) {
            curr = curr.saturating_add(3);
            curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        }
        if dir2.is_none() {
            let rem_dir = masked.get(curr..).unwrap_or("");
            if let Some(dir_kw) = ["inout", "out", "in"]
                .into_iter()
                .find(|kw| rem_dir.starts_with(kw) && is_word_boundary_after(rem_dir, kw.len()))
            {
                dir2 = Some(dir_kw);
                curr = curr.saturating_add(dir_kw.len());
                curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
            }
        }
    }
    (dir2, curr)
}

/// Parses an optional `: [~]<PortType>` or `:> [~]<PortType>` clause after a port identifier.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PORT-DEFS]
fn parse_port_type_annotation(masked: &str, start: usize) -> (Option<String>, bool, usize) {
    let mut curr = start;
    let mut type_name = None;
    let mut conj2 = false;
    let rem_type = masked.get(curr..).unwrap_or("");
    if rem_type.starts_with(":>") || rem_type.starts_with(':') {
        curr = curr.saturating_add(if rem_type.starts_with(":>") { 2 } else { 1 });
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        if masked.get(curr..).is_some_and(|s| s.starts_with('~')) {
            conj2 = true;
            curr = curr.saturating_add(1);
            curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        }
        let (t, t_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if !t.is_empty() {
            type_name = Some(t.to_string());
            curr = curr.saturating_add(t_len);
            curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        }
    }
    (type_name, conj2, curr)
}

/// Extracts `protocol_family`, `port_category`, electrical attributes, and `FlowDef` lines from an optional port body.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PORT-DEFS]
fn parse_port_body_members(
    port_body_opt: Option<&str>,
) -> (String, String, BTreeMap<String, String>, Vec<FlowDef>) {
    let mut protocol_family = String::new();
    let mut port_category = default_port_category();
    let mut electrical_attributes = BTreeMap::new();
    let mut item_flows = Vec::new();
    if let Some(port_body) = port_body_opt {
        for attr in parse_attribute_defs(port_body) {
            let val = attr.default_value.as_deref().unwrap_or("").trim().trim_matches('"').to_string();
            match attr.name.as_str() {
                "protocol_family" => protocol_family = val,
                "port_category" if !val.is_empty() => port_category = val,
                "port_category" => {}
                _ => {
                    electrical_attributes.insert(attr.name, val);
                }
            }
        }
        item_flows.extend(port_body.lines().filter_map(parse_port_flow_line));
    }
    (protocol_family, port_category, electrical_attributes, item_flows)
}

/// Extracts either a brace-delimited `{ ... }` port body or a `;` terminator starting at `curr`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PORT-DEFS]
fn extract_port_terminator_or_body<'a>(
    clean: &'a str,
    masked: &str,
    curr: usize,
) -> Option<(Option<&'a str>, usize)> {
    let rem_body = masked.get(curr..)?;
    if rem_body.starts_with('{') {
        let close = find_matching_brace(clean, curr)
            .or_else(|| rem_body.find('}').map(|r| curr.saturating_add(r)))?;
        Some((clean.get(curr.saturating_add(1)..close), close.saturating_add(1)))
    } else if rem_body.starts_with(';') {
        Some((None, curr.saturating_add(1)))
    } else {
        None
    }
}

/// Parses `PortDef` interface definitions and usages from SysML v2 source text.
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PORT-DEFS]
/// Contract: Extracts every `port` and `port def` declaration in appearance order, resolving leading or inline directionality, `~` conjugation, protocol/category metadata, `BTreeMap` electrical attributes, and nested `FlowDef` item flows.
/// Bounds & Faults: Executes in O(N) time and O(N) space over `content.len()` with checked `.get` slicing; malformed port declarations are skipped safely without panicking.
pub fn parse_port_defs(content: &str) -> Vec<PortDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut ports = Vec::new();
    let mut skip_until: usize = 0;

    for (res, span) in SysmlToken::lexer(&masked).spanned() {
        if span.start < skip_until || res.is_err() || masked.get(span.clone()) != Some("port") {
            continue;
        }
        let (dir1, doc_end) = extract_leading_port_direction(&masked, span.start);
        let doc = extract_doc_comment_before(&clean, doc_end);
        let curr = span.end.saturating_add(skip_ws(masked.get(span.end..).unwrap_or("")));
        let (dir2, mut curr) = consume_optional_direction_and_def(&masked, curr);
        let mut conj1 = false;
        if masked.get(curr..).is_some_and(|s| s.starts_with('~')) {
            conj1 = true;
            curr = curr.saturating_add(1);
            curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        }
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            continue;
        }
        curr = curr.saturating_add(name_len);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        let (type_name, conj2, curr) = parse_port_type_annotation(&masked, curr);
        let Some((port_body_opt, next_idx)) = extract_port_terminator_or_body(&clean, &masked, curr) else { continue };
        skip_until = next_idx;
        let (protocol_family, port_category, electrical_attributes, item_flows) =
            parse_port_body_members(port_body_opt);
        ports.push(PortDef {
            name: name.to_string(),
            type_name: type_name.unwrap_or_else(|| name.to_string()),
            direction: dir1.or(dir2).unwrap_or("inout").to_string(),
            is_conjugated: conj1 || conj2,
            doc,
            port_category,
            protocol_family,
            electrical_attributes,
            item_flows,
        });
    }
    ports
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_connection_defs_extracts_connections_flows_and_owner_parts() -> Result<(), Box<dyn std::error::Error>> {
        let src = "doc /* Redundant avionics bus link */\nconnection def SensorToController {\n    connect sensor.out_port to ctrl.in_port;\n}\nconnection inline_bus connect nav_out to guidance_in;\nflow def TelemetryStream {\n    flow from recorder.tx_port to downlink.rx_port;\n}\nflow from orphan_src to orphan_tgt;\nconnection broken_unclosed {";
        let conns = parse_connection_defs(src);
        let (c0, c1, c2) = (conns.first().ok_or("c0")?, conns.get(1).ok_or("c1")?, conns.get(2).ok_or("c2")?);
        assert_eq!((conns.len(), c0.name.as_str(), c0.doc.as_deref(), c0.source_port.as_str(), c0.target_port.as_str(), c0.source_part.as_deref(), c0.target_part.as_deref(), c0.is_flow, c0.severity), (3, "SensorToController", Some("Redundant avionics bus link"), "sensor.out_port", "ctrl.in_port", Some("sensor"), Some("ctrl"), false, 1));
        assert_eq!((c1.name.as_str(), c1.source_port.as_str(), c1.target_port.as_str(), c1.source_part.is_none() && c1.target_part.is_none() && !c1.is_flow), ("inline_bus", "nav_out", "guidance_in", true));
        assert_eq!((c2.name.as_str(), c2.source_port.as_str(), c2.target_port.as_str(), c2.source_part.as_deref(), c2.target_part.as_deref(), c2.is_flow), ("TelemetryStream", "recorder.tx_port", "downlink.rx_port", Some("recorder"), Some("downlink"), true));
        Ok(())
    }

    #[test]
    fn parse_port_defs_extracts_conjugation_attributes_and_item_flows() -> Result<(), Box<dyn std::error::Error>> {
        let src = "doc /* Primary ARINC 429 receiver port */\nin port rx_bus : ~Arinc429Port {\n    attribute protocol_family : String = \"ARINC-429\";\n    attribute port_category : String = \"AvionicsBus\";\n    attribute impedance_ohm : Real = 75.0;\n    in item nav_frame : NavPacket;\n    flow out ack_signal : AckPulse;\n    inout status_word;\n}\nport ~conj_inline;\nport unclosed_port : BrokenType {";
        let ports = parse_port_defs(src);
        let (p0, p1) = (ports.first().ok_or("p0")?, ports.get(1).ok_or("p1")?);
        assert_eq!((ports.len(), p0.name.as_str(), p0.type_name.as_str(), p0.direction.as_str(), p0.is_conjugated, p0.doc.as_deref(), p0.protocol_family.as_str(), p0.port_category.as_str()), (2, "rx_bus", "Arinc429Port", "in", true, Some("Primary ARINC 429 receiver port"), "ARINC-429", "AvionicsBus"));
        assert_eq!(p0.electrical_attributes.get("impedance_ohm").map(String::as_str), Some("75.0"));
        let flows: Vec<(&str, &str, &str)> = p0.item_flows.iter().map(|f| (f.name.as_str(), f.direction.as_str(), f.item_type.as_str())).collect();
        assert_eq!(flows, vec![("nav_frame", "in", "NavPacket"), ("ack_signal", "out", "AckPulse"), ("status_word", "inout", "Item")]);
        assert_eq!((p1.name.as_str(), p1.type_name.as_str(), p1.direction.as_str(), p1.is_conjugated), ("conj_inline", "conj_inline", "inout", true));
        Ok(())
    }
}
