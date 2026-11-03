//! Check 24 `ICD_01` connection helpers and `ICD_02` Master Signal Flow Dictionary auditor.
//!
//! Safety & Traceability: Realises: [REQ-PARITY-GATE-24]
//! Contract: Validates nested connection qualification, port directionality, and `ICD_02` signal dictionary metrology.
//! Bounds & Faults: Deterministic BTreeSet deduplication; fails closed on unmapped signals or placeholder metrology.

use super::parity_check_icd_ast::{
    extract_port_identifier, extract_single_signal_id, is_metrology_placeholder,
    parse_markdown_table_rows, PortMeta,
};
use std::collections::{BTreeMap, BTreeSet};

/// State accumulator for `ICD_01` Section 5 connection binding table auditing.
/// Realises: [REQ-PARITY-GATE-24]
pub(crate) struct Icd01ConnAuditState<'a> {
    pub(crate) declared_ports: &'a BTreeMap<String, PortMeta>,
    pub(crate) ast_nested_subsystems: &'a BTreeSet<String>,
    pub(crate) ast_nested_ports: &'a BTreeSet<String>,
    pub(crate) bound_ports: BTreeSet<String>,
    pub(crate) valid_connections: BTreeSet<(String, String)>,
    pub(crate) seen_conn_ids: BTreeSet<String>,
}

/// Validates hierarchical `(Subsystem.Path.port)` qualification on nested connections.
/// Realises: [REQ-PARITY-GATE-24]
pub(crate) fn check_nested_connection_qualification(
    conn_id: &str,
    raw_endpoints: (&str, &str),
    port_ids: (&Option<String>, &Option<String>),
    state: &Icd01ConnAuditState<'_>,
    errors: &mut Vec<String>,
) {
    let (src_raw, dst_raw) = raw_endpoints;
    let src_subsys = port_ids
        .0
        .as_ref()
        .and_then(|id| state.declared_ports.get(id))
        .map(|p| p.subsystem.as_str())
        .unwrap_or("");
    let dst_subsys = port_ids
        .1
        .as_ref()
        .and_then(|id| state.declared_ports.get(id))
        .map(|p| p.subsystem.as_str())
        .unwrap_or("");
    let src_port_name = port_ids
        .0
        .as_ref()
        .and_then(|id| state.declared_ports.get(id))
        .map(|p| p.name.as_str())
        .unwrap_or(src_raw);
    let dst_port_name = port_ids
        .1
        .as_ref()
        .and_then(|id| state.declared_ports.get(id))
        .map(|p| p.name.as_str())
        .unwrap_or(dst_raw);
    let has_multi_subsystems =
        !src_subsys.is_empty() && !dst_subsys.is_empty() && src_subsys != dst_subsys;
    let is_ast_nested = state
        .ast_nested_subsystems
        .contains(&src_subsys.to_ascii_lowercase())
        || state
            .ast_nested_subsystems
            .contains(&dst_subsys.to_ascii_lowercase())
        || state
            .ast_nested_ports
            .contains(&src_port_name.to_ascii_lowercase())
        || state
            .ast_nested_ports
            .contains(&dst_port_name.to_ascii_lowercase());
    let is_nested = is_ast_nested
        || src_raw.contains('.')
        || dst_raw.contains('.')
        || src_subsys.contains('.')
        || dst_subsys.contains('.')
        || has_multi_subsystems;
    if !is_nested {
        return;
    }
    if !(src_raw.contains('(') && src_raw.contains('.') && src_raw.contains(')')) {
        errors.push(format!(
            "Check 24 failed: Connection '{}' source port '{}' lacks hierarchical subsystem qualification for nested port (expected format 'PORT-ID (Subsystem.Path.port)')",
            conn_id, src_raw
        ));
    }
    if !(dst_raw.contains('(') && dst_raw.contains('.') && dst_raw.contains(')')) {
        errors.push(format!(
            "Check 24 failed: Connection '{}' dest port '{}' lacks hierarchical subsystem qualification for nested port (expected format 'PORT-ID (Subsystem.Path.port)')",
            conn_id, dst_raw
        ));
    }
}

/// Validates port existence and directionality for a source (`is_src = true`) or destination endpoint.
/// Realises: [REQ-PARITY-GATE-24]
pub(crate) fn bind_connection_endpoint(
    conn_id: &str,
    port_id_opt: &Option<String>,
    is_src: bool,
    state: &mut Icd01ConnAuditState<'_>,
    errors: &mut Vec<String>,
) {
    let Some(ref pid) = port_id_opt else {
        return;
    };
    let Some(port) = state.declared_ports.get(pid) else {
        let role = if is_src { "source" } else { "dest" };
        errors.push(format!(
            "Check 24 failed: Connection '{}' references undeclared {} port '{}'",
            conn_id, role, pid
        ));
        return;
    };
    if is_src {
        if port.direction != "OUT" && port.direction != "INOUT" {
            errors.push(format!(
                "Check 24 failed: Connection '{}' binds port '{}' with direction '{}' as source (expected OUT or INOUT)",
                conn_id, pid, port.direction
            ));
        }
    } else if port.direction != "IN" && port.direction != "INOUT" {
        errors.push(format!(
            "Check 24 failed: Connection '{}' binds port '{}' with direction '{}' as destination (expected IN or INOUT)",
            conn_id, pid, port.direction
        ));
    }
    state.bound_ports.insert(pid.clone());
}

/// Harvests `(src_port_id, dst_port_id)` pairs from `ICD_02` Section 2 into connection audit state.
/// Realises: [REQ-PARITY-GATE-24]
pub(crate) fn harvest_icd02_signal_port_bindings(
    icd02_content: &str,
    state: &mut Icd01ConnAuditState<'_>,
) {
    let (sig_headers, sig_rows) =
        parse_markdown_table_rows(icd02_content, "signal flow dictionary");
    let src_idx = sig_headers
        .iter()
        .position(|h| h.contains("source"))
        .unwrap_or(2);
    let dst_idx = sig_headers
        .iter()
        .position(|h| h.contains("dest"))
        .unwrap_or(3);
    for row in &sig_rows {
        let src_opt = row.get(src_idx).and_then(|s| extract_port_identifier(s));
        let dst_opt = row.get(dst_idx).and_then(|s| extract_port_identifier(s));
        if let Some(ref s) = src_opt {
            state.bound_ports.insert(s.clone());
        }
        if let Some(ref d) = dst_opt {
            state.bound_ports.insert(d.clone());
        }
        if let (Some(s), Some(d)) = (src_opt, dst_opt) {
            state.valid_connections.insert((s, d));
        }
    }
}

/// Audits `ICD_02` Section 2 Master Signal Flow Dictionary rows against `ICD_01` ports and connections.
/// Realises: [REQ-PARITY-GATE-24]
pub(crate) fn audit_icd02_signals(
    icd02_content: &str,
    declared_ports: &BTreeMap<String, PortMeta>,
    valid_connections: &BTreeSet<(String, String)>,
    errors: &mut Vec<String>,
) {
    let (sig_headers, sig_rows) =
        parse_markdown_table_rows(icd02_content, "signal flow dictionary");
    if sig_rows.is_empty() {
        errors.push(
            "Check 24 failed: ICD_02 Master Signal Flow Dictionary Table (Section 2) is missing or contains 0 rows.".to_string(),
        );
    }

    let idxs = [
        sig_headers
            .iter()
            .position(|h| h.contains("signal id") || h.contains("id"))
            .unwrap_or(0),
        sig_headers
            .iter()
            .position(|h| h.contains("signal name") || h.contains("name"))
            .unwrap_or(1),
        sig_headers
            .iter()
            .position(|h| h.contains("source"))
            .unwrap_or(2),
        sig_headers
            .iter()
            .position(|h| h.contains("dest"))
            .unwrap_or(3),
        sig_headers
            .iter()
            .position(|h| h.contains("data type") || h.contains("type"))
            .unwrap_or(4),
        sig_headers
            .iter()
            .position(|h| h.contains("unit"))
            .unwrap_or(5),
        sig_headers
            .iter()
            .position(|h| h.contains("range"))
            .unwrap_or(6),
        sig_headers
            .iter()
            .position(|h| h.contains("rate"))
            .unwrap_or(7),
        sig_headers
            .iter()
            .position(|h| h.contains("default"))
            .unwrap_or(8),
        sig_headers
            .iter()
            .position(|h| h.contains("citation") || h.contains("schema") || h.contains("source"))
            .unwrap_or(9),
    ];

    let mut seen_sig_ids = BTreeSet::new();
    for row in &sig_rows {
        audit_single_icd02_signal_row(
            row,
            &idxs,
            (declared_ports, valid_connections),
            &mut seen_sig_ids,
            errors,
        );
    }
}

/// Audits a single `ICD_02` signal dictionary row for port binding, connection mapping, and metrology.
/// Realises: [REQ-PARITY-GATE-24]
fn audit_single_icd02_signal_row(
    row: &[String],
    idxs: &[usize; 10],
    tables: (&BTreeMap<String, PortMeta>, &BTreeSet<(String, String)>),
    seen_sig_ids: &mut BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let (declared_ports, valid_connections) = tables;
    let raw_sig_id = row.get(idxs[0]).map(|s| s.as_str()).unwrap_or("");
    let sig_id = extract_single_signal_id(raw_sig_id)
        .unwrap_or_else(|| raw_sig_id.trim_matches(|c| c == '`' || c == ' ' || c == '*').to_string());
    let sig_name = row.get(idxs[1]).map(|s| s.as_str()).unwrap_or("").trim_matches(|c| c == '`' || c == ' ' || c == '*');
    let src_raw = row.get(idxs[2]).map(|s| s.as_str()).unwrap_or("");
    let dst_raw = row.get(idxs[3]).map(|s| s.as_str()).unwrap_or("");
    let src_port = extract_port_identifier(src_raw)
        .unwrap_or_else(|| src_raw.trim_matches(|c| c == '`' || c == ' ' || c == '*').to_string());
    let dst_port = extract_port_identifier(dst_raw)
        .unwrap_or_else(|| dst_raw.trim_matches(|c| c == '`' || c == ' ' || c == '*').to_string());

    if sig_id.is_empty() || !sig_id.starts_with("SIG-") {
        errors.push(format!(
            "Check 24 failed: Invalid Signal ID '{}' in Section 2 (must start with 'SIG-')",
            raw_sig_id
        ));
    } else if !seen_sig_ids.insert(sig_id.clone()) {
        errors.push(format!(
            "Check 24 failed: Duplicate Signal ID '{}' in Section 2",
            sig_id
        ));
    }

    if !declared_ports.contains_key(&src_port) {
        errors.push(format!(
            "Check 24 failed: Signal '{}' ({}) references undeclared source port '{}'",
            sig_id, sig_name, src_port
        ));
    }
    if !declared_ports.contains_key(&dst_port) {
        errors.push(format!(
            "Check 24 failed: Signal '{}' ({}) references undeclared dest port '{}'",
            sig_id, sig_name, dst_port
        ));
    }
    if !valid_connections.contains(&(src_port.clone(), dst_port.clone())) {
        errors.push(format!(
            "Check 24 failed: Unmapped signal '{}': no connection in ICD_01 connects source '{}' to dest '{}'",
            sig_id, src_port, dst_port
        ));
    }

    let fields = [
        ("data type", idxs[4]),
        ("SI units", idxs[5]),
        ("valid range", idxs[6]),
        ("update rate", idxs[7]),
        ("safe default value", idxs[8]),
        ("schema citation", idxs[9]),
    ];
    for (fname, col_idx) in fields {
        let val = row.get(col_idx).map(|s| s.as_str()).unwrap_or("").trim();
        if val.is_empty() || is_metrology_placeholder(val) {
            errors.push(format!(
                "Check 24 failed: Signal '{}' field '{}' has incomplete metrology or placeholder: '{}'",
                sig_id, fname, val
            ));
        }
    }
}
