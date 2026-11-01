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
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Represents a connection or topological flow link between ports in SysML v2.
///
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
    #[serde(default = "default_one")]
    pub severity: i32,
    pub item_flow_ref: Option<String>,
    pub protocol: Option<String>,
    pub latency_ms: Option<f64>,
    #[serde(default)]
    pub is_flow: bool,
}

/// Checks whether `masked` at byte offset `idx` begins a standalone `connection` or `flow` keyword.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CONNECTION-DEFS]
fn match_connection_keyword(masked: &str, idx: usize) -> Option<(bool, usize)> {
    let rem = masked.get(idx..)?;
    if !is_word_boundary_before(masked, idx) {
        return None;
    }
    if rem.starts_with("connection") && is_word_boundary_after(masked, idx.saturating_add(10)) {
        Some((false, 10))
    } else if rem.starts_with("flow") && is_word_boundary_after(masked, idx.saturating_add(4)) {
        Some((true, 4))
    } else {
        None
    }
}

/// Extracts either a brace-delimited `{ ... }` connection body or an inline `connect ...;` clause starting at `curr`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CONNECTION-DEFS]
fn extract_connection_body<'a>(
    clean: &'a str,
    masked: &str,
    curr: usize,
) -> Option<(&'a str, usize)> {
    let rem_conn = masked.get(curr..)?;
    if rem_conn.starts_with('{') {
        let close_pos = find_matching_brace(clean, curr)?;
        let body = clean.get(curr.saturating_add(1)..close_pos).unwrap_or("");
        Some((body, close_pos.saturating_add(1)))
    } else if rem_conn.starts_with("connect") && is_word_boundary_after(rem_conn, 7) {
        let semi_rel = rem_conn.find(';')?;
        let end_pos = curr.saturating_add(semi_rel).saturating_add(1);
        let body = clean.get(curr..end_pos).unwrap_or("");
        Some((body, end_pos))
    } else {
        None
    }
}

/// Scans lines inside a connection or flow block for the first `connect <src> to <tgt>` or `flow from <src> to <tgt>` clause.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CONNECTION-DEFS]
fn extract_connection_endpoints(body: &str) -> (String, String) {
    for line in body.lines() {
        let trimmed = line.trim();
        let after_conn = if trimmed.starts_with("connect") && is_word_boundary_after(trimmed, 7) {
            Some(trimmed.get(7..).unwrap_or("").trim_start())
        } else if trimmed.starts_with("flow") && is_word_boundary_after(trimmed, 4) {
            let after_flow = trimmed.get(4..).unwrap_or("").trim_start();
            if after_flow.starts_with("from") && is_word_boundary_after(after_flow, 4) {
                Some(after_flow.get(4..).unwrap_or("").trim_start())
            } else {
                None
            }
        } else {
            None
        };

        if let Some(after) = after_conn {
            if let Some((raw_src, raw_tgt)) = after.split_once(" to ") {
                let src = raw_src.trim().to_string();
                let tgt = raw_tgt.trim().trim_end_matches(';').trim().to_string();
                return (src, tgt);
            }
        }
    }
    (String::new(), String::new())
}

/// Extracts the leading owning part identifier from a dotted `<part>.<port>` endpoint reference.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CONNECTION-DEFS]
fn extract_part_prefix_from_port(port_ref: &str) -> Option<String> {
    port_ref.split_once('.').map(|(part, _)| part.to_string())
}

/// Parses topological `connection` and `flow` definitions from SysML v2 source text.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-CONNECTION-DEFS]
/// Contract: Extracts every `connection [def]` and top-level `flow [def]` block or inline `connect <src> to <tgt>;` declaration in appearance order while ignoring inner `flow from ...` body statements.
/// Bounds & Faults: Executes in O(N) time and O(N) space over `content.len()` with checked `.get` slicing; malformed or unclosed declarations are skipped without panicking.
pub fn parse_connection_defs(content: &str) -> Vec<ConnectionDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut conns = Vec::new();
    let mut idx: usize = 0;

    while idx < masked.len() {
        let Some((is_flow, kw_len)) = match_connection_keyword(&masked, idx) else {
            let rem = masked.get(idx..).unwrap_or("");
            let step = rem.chars().next().map(char::len_utf8).unwrap_or(1);
            idx = idx.saturating_add(step);
            continue;
        };

        let kw_pos = idx;
        let after_kw = kw_pos.saturating_add(kw_len);
        let mut curr = after_kw;
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));

        // Skip `flow from <src> to <tgt>;` statements residing inside a container body.
        if is_flow
            && masked
                .get(curr..)
                .is_some_and(|s| s.starts_with("from") && is_word_boundary_after(s, 4))
        {
            idx = after_kw;
            continue;
        }

        let doc = extract_doc_comment_before(&clean, kw_pos);
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            curr = curr.saturating_add(3);
            curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        }

        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = after_kw;
            continue;
        }
        curr = curr.saturating_add(name_len);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));

        let Some((body, next_idx)) = extract_connection_body(&clean, &masked, curr) else {
            idx = after_kw;
            continue;
        };
        idx = next_idx;

        let (source_port, target_port) = extract_connection_endpoints(body);
        let source_part = extract_part_prefix_from_port(&source_port);
        let target_part = extract_part_prefix_from_port(&target_port);

        conns.push(ConnectionDef {
            name: name.to_string(),
            source_port,
            target_port,
            source_part,
            target_part,
            doc,
            severity: 1,
            item_flow_ref: None,
            protocol: None,
            latency_ms: None,
            is_flow,
        });
    }
    conns
}

/// Parses a single item-flow declaration line inside a SysML v2 port block into a `FlowDef`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PORT-FLOW-LINE]
fn parse_port_flow_line(line: &str) -> Option<FlowDef> {
    let trimmed = line.trim().trim_end_matches(';').trim();
    let is_attr = trimmed.starts_with("attribute") && is_word_boundary_after(trimmed, 9);
    if trimmed.is_empty() || is_attr {
        return None;
    }

    let mut rest = trimmed;
    let mut direction = None;
    let mut saw_flow_kw = false;

    if rest.starts_with("flow") && is_word_boundary_after(rest, 4) {
        saw_flow_kw = true;
        rest = rest.get(4..).unwrap_or("").trim_start();
    }
    for dir_kw in ["inout", "out", "in"] {
        if rest.starts_with(dir_kw) && is_word_boundary_after(rest, dir_kw.len()) {
            direction = Some(dir_kw.to_string());
            rest = rest.get(dir_kw.len()..).unwrap_or("").trim_start();
            break;
        }
    }
    let has_payload_kw = rest.starts_with("flow") || rest.starts_with("item");
    if has_payload_kw && is_word_boundary_after(rest, 4) {
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
    let after_name = rest.get(name_len..).unwrap_or("").trim_start();
    let item_type = if let Some(after_colon_raw) = after_name.strip_prefix(':') {
        let (t_name, _) = extract_type_ident(after_colon_raw.trim_start());
        if t_name.is_empty() {
            "Item".to_string()
        } else {
            t_name.to_string()
        }
    } else {
        "Item".to_string()
    };

    Some(FlowDef {
        name: name.to_string(),
        direction: direction.unwrap_or_else(|| "out".to_string()),
        item_type,
        doc: None,
        ..Default::default()
    })
}

/// Inspects tokens immediately preceding `port_pos` for a leading `inout`, `out`, or `in` direction modifier.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PORT-DEFS]
fn extract_leading_port_direction(masked: &str, port_pos: usize) -> (Option<&'static str>, usize) {
    let before_port = masked.get(..port_pos).unwrap_or("").trim_end();
    for dir_kw in ["inout", "out", "in"] {
        let dir_pos = before_port.len().saturating_sub(dir_kw.len());
        if before_port.ends_with(dir_kw) && is_word_boundary_before(before_port, dir_pos) {
            return (Some(dir_kw), dir_pos);
        }
    }
    (None, port_pos)
}

/// Consumes optional `def` and inline direction (`inout`, `out`, `in`) modifiers following the `port` keyword.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PORT-DEFS]
fn consume_optional_direction_and_def(masked: &str, start: usize) -> (Option<&'static str>, usize) {
    let mut curr = start;
    let mut dir2 = None;
    for _ in 0..2 {
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            curr = curr.saturating_add(3);
            curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        }
        if dir2.is_none() {
            let rem_dir = masked.get(curr..).unwrap_or("");
            for dir_kw in ["inout", "out", "in"] {
                if rem_dir.starts_with(dir_kw) && is_word_boundary_after(rem_dir, dir_kw.len()) {
                    dir2 = Some(dir_kw);
                    curr = curr.saturating_add(dir_kw.len());
                    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
                    break;
                }
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
        let sep_len = if rem_type.starts_with(":>") { 2 } else { 1 };
        curr = curr.saturating_add(sep_len);
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

/// Aggregated attributes and item flows extracted from a `{ ... }` port declaration body.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PORT-DEFS]
struct ParsedPortBody {
    protocol_family: String,
    port_category: String,
    electrical_attributes: BTreeMap<String, String>,
    item_flows: Vec<FlowDef>,
}

/// Extracts `protocol_family`, `port_category`, electrical attributes, and `FlowDef` lines from an optional port body.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PORT-DEFS]
fn parse_port_body_members(port_body_opt: Option<&str>) -> ParsedPortBody {
    let mut protocol_family = String::new();
    let mut port_category = default_port_category();
    let mut electrical_attributes = BTreeMap::new();
    let mut item_flows = Vec::new();

    if let Some(port_body) = port_body_opt {
        for attr in parse_attribute_defs(port_body) {
            let val = attr
                .default_value
                .as_deref()
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .to_string();
            if attr.name == "protocol_family" {
                protocol_family = val;
            } else if attr.name == "port_category" {
                if !val.is_empty() {
                    port_category = val;
                }
            } else {
                electrical_attributes.insert(attr.name, val);
            }
        }
        for line in port_body.lines() {
            if let Some(flow) = parse_port_flow_line(line) {
                item_flows.push(flow);
            }
        }
    }

    ParsedPortBody {
        protocol_family,
        port_category,
        electrical_attributes,
        item_flows,
    }
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
        if let Some(close_brace) = find_matching_brace(clean, curr) {
            Some((
                clean.get(curr.saturating_add(1)..close_brace),
                close_brace.saturating_add(1),
            ))
        } else if let Some(close_rel) = rem_body.find('}') {
            let close_abs = curr.saturating_add(close_rel);
            Some((
                clean.get(curr.saturating_add(1)..close_abs),
                close_abs.saturating_add(1),
            ))
        } else {
            None
        }
    } else if rem_body.starts_with(';') {
        Some((None, curr.saturating_add(1)))
    } else {
        None
    }
}

/// Parses `PortDef` interface definitions and usages from SysML v2 source text.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-PORT-DEFS]
/// Contract: Extracts every `port` and `port def` declaration in appearance order, resolving leading or inline directionality, `~` conjugation, protocol/category metadata, `BTreeMap` electrical attributes, and nested `FlowDef` item flows.
/// Bounds & Faults: Executes in O(N) time and O(N) space over `content.len()` with checked `.get` slicing; malformed port declarations are skipped safely without panicking.
pub fn parse_port_defs(content: &str) -> Vec<PortDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut ports = Vec::new();
    let mut idx: usize = 0;

    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("port")) {
        let port_pos = idx.saturating_add(pos);
        let after_port = port_pos.saturating_add(4);
        if !is_word_boundary_before(&masked, port_pos)
            || !is_word_boundary_after(&masked, after_port)
        {
            idx = after_port;
            continue;
        }

        let (dir1, doc_end) = extract_leading_port_direction(&masked, port_pos);
        let doc = extract_doc_comment_before(&clean, doc_end);

        let mut curr = after_port;
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        let (dir2, after_dir) = consume_optional_direction_and_def(&masked, curr);
        curr = after_dir;

        let mut conj1 = false;
        if masked.get(curr..).is_some_and(|s| s.starts_with('~')) {
            conj1 = true;
            curr = curr.saturating_add(1);
            curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        }

        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = after_port;
            continue;
        }
        curr = curr.saturating_add(name_len);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));

        let (type_name, conj2, after_type) = parse_port_type_annotation(&masked, curr);
        curr = after_type;

        let Some((port_body_opt, next_idx)) =
            extract_port_terminator_or_body(&clean, &masked, curr)
        else {
            idx = after_port;
            continue;
        };
        idx = next_idx;

        let body_members = parse_port_body_members(port_body_opt);

        ports.push(PortDef {
            name: name.to_string(),
            type_name: type_name.unwrap_or_else(|| name.to_string()),
            direction: dir1.or(dir2).unwrap_or("inout").to_string(),
            is_conjugated: conj1 || conj2,
            doc,
            port_category: body_members.port_category,
            protocol_family: body_members.protocol_family,
            electrical_attributes: body_members.electrical_attributes,
            item_flows: body_members.item_flows,
        });
    }
    ports
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies `parse_connection_defs` across block `connection def`, inline `connect`, and `flow def` variants.
    #[test]
    fn parse_connection_defs_extracts_connections_flows_and_owner_parts(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let src = r#"
            doc /* Redundant avionics bus link */
            connection def SensorToController {
                connect sensor.out_port to ctrl.in_port;
            }
            connection inline_bus connect nav_out to guidance_in;
            flow def TelemetryStream {
                flow from recorder.tx_port to downlink.rx_port;
            }
            // Malformed or inner flow statement must not produce spurious ConnectionDef entries
            flow from orphan_src to orphan_tgt;
            connection broken_unclosed {
        "#;

        let conns = parse_connection_defs(src);
        assert_eq!(conns.len(), 3);

        let c0 = conns.first().ok_or("missing SensorToController")?;
        assert_eq!(c0.name, "SensorToController");
        assert_eq!(c0.doc.as_deref(), Some("Redundant avionics bus link"));
        assert_eq!(c0.source_port, "sensor.out_port");
        assert_eq!(c0.target_port, "ctrl.in_port");
        assert_eq!(c0.source_part.as_deref(), Some("sensor"));
        assert_eq!(c0.target_part.as_deref(), Some("ctrl"));
        assert!(!c0.is_flow);
        assert_eq!(c0.severity, 1);

        let c1 = conns.get(1).ok_or("missing inline_bus")?;
        assert_eq!(c1.name, "inline_bus");
        assert_eq!(c1.source_port, "nav_out");
        assert_eq!(c1.target_port, "guidance_in");
        assert!(c1.source_part.is_none());
        assert!(c1.target_part.is_none());
        assert!(!c1.is_flow);

        let c2 = conns.get(2).ok_or("missing TelemetryStream")?;
        assert_eq!(c2.name, "TelemetryStream");
        assert_eq!(c2.source_port, "recorder.tx_port");
        assert_eq!(c2.target_port, "downlink.rx_port");
        assert_eq!(c2.source_part.as_deref(), Some("recorder"));
        assert_eq!(c2.target_part.as_deref(), Some("downlink"));
        assert!(c2.is_flow);
        Ok(())
    }

    /// Verifies `parse_port_defs` handles conjugation, protocol/category attributes, item flows, and malformed inputs.
    #[test]
    fn parse_port_defs_extracts_conjugation_attributes_and_item_flows(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let src = r#"
            doc /* Primary ARINC 429 receiver port */
            in port rx_bus : ~Arinc429Port {
                attribute protocol_family : String = "ARINC-429";
                attribute port_category : String = "AvionicsBus";
                attribute impedance_ohm : Real = 75.0;
                in item nav_frame : NavPacket;
                flow out ack_signal : AckPulse;
                inout status_word;
            }
            port ~conj_inline;
            port unclosed_port : BrokenType {
        "#;

        let ports = parse_port_defs(src);
        assert_eq!(ports.len(), 2);

        let p0 = ports.first().ok_or("missing rx_bus")?;
        assert_eq!(p0.name, "rx_bus");
        assert_eq!(p0.type_name, "Arinc429Port");
        assert_eq!(p0.direction, "in");
        assert!(p0.is_conjugated);
        assert_eq!(p0.doc.as_deref(), Some("Primary ARINC 429 receiver port"));
        assert_eq!(p0.protocol_family, "ARINC-429");
        assert_eq!(p0.port_category, "AvionicsBus");
        assert_eq!(
            p0.electrical_attributes
                .get("impedance_ohm")
                .map(String::as_str),
            Some("75.0")
        );
        assert_eq!(p0.item_flows.len(), 3);
        let f0 = p0.item_flows.first().ok_or("missing flow 0")?;
        assert_eq!(f0.name, "nav_frame");
        assert_eq!(f0.direction, "in");
        assert_eq!(f0.item_type, "NavPacket");
        let f1 = p0.item_flows.get(1).ok_or("missing flow 1")?;
        assert_eq!(f1.name, "ack_signal");
        assert_eq!(f1.direction, "out");
        assert_eq!(f1.item_type, "AckPulse");
        let f2 = p0.item_flows.get(2).ok_or("missing flow 2")?;
        assert_eq!(f2.name, "status_word");
        assert_eq!(f2.direction, "inout");
        assert_eq!(f2.item_type, "Item");

        let p1 = ports.get(1).ok_or("missing conj_inline")?;
        assert_eq!(p1.name, "conj_inline");
        assert_eq!(p1.type_name, "conj_inline");
        assert_eq!(p1.direction, "inout");
        assert!(p1.is_conjugated);
        Ok(())
    }
}
