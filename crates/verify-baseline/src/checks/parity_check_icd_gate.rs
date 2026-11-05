//! Check 24: Level 1C ICD Completeness & Signal Flow Parity Gate (`check_icd_completeness`).
//!
//! Safety & Traceability: Realises: [REQ-PARITY-GATE-24]
//! Contract: Enforces port roster completeness, connection directionality, nested port qualification, and signal metrology.
//! Bounds & Faults: Deterministic BTreeMap/BTreeSet evaluation; fails closed on dangling ports or unmapped signals.

use super::parity_check_discovery::{discover_sysml_model_text, TargetScanDir};
use super::parity_check_icd_ast::{
    collect_ast_nested_subsystems_and_ports, extract_port_identifier, extract_single_conn_id,
    parse_markdown_table_rows, PortMeta,
};
use super::parity_check_icd_signals::{
    audit_icd02_signals, bind_connection_endpoint, check_nested_connection_qualification,
    harvest_icd02_signal_port_bindings, Icd01ConnAuditState,
};
use super::parity_gates::ParityOutcome;
use deap_core::sysml_ast::parse_part_defs;
use deap_core::workspace::is_upstream_compiler;
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Check 24: Level 1C ICD Completeness & Signal Flow Parity Gate.
/// Realises: [REQ-PARITY-GATE-24]
pub fn check_icd_completeness(repo_root: &Path) -> Result<ParityOutcome, Vec<String>> {
    if is_upstream_compiler(repo_root) {
        return Ok(ParityOutcome::Verified);
    }

    let iface_dir = TargetScanDir::Interfaces.resolve(repo_root);
    let icd_dir = TargetScanDir::Icd.resolve(repo_root);
    let interfaces_dir = if iface_dir.is_dir() || !icd_dir.is_dir() { iface_dir } else { icd_dir };

    let model_text = discover_sysml_model_text(repo_root);
    let has_extracted = repo_root.join("schema").join("extracted").is_dir();

    if !interfaces_dir.is_dir() {
        return Ok(if model_text.is_none() && !has_extracted {
            ParityOutcome::PendingOrClean
        } else {
            ParityOutcome::InterfacesDirMissing
        });
    }

    let (icd01_path, icd02_path) = discover_icd_files(&interfaces_dir);
    if icd01_path.is_none() && icd02_path.is_none() {
        return Ok(if model_text.is_none() && !has_extracted {
            ParityOutcome::PendingOrClean
        } else {
            ParityOutcome::IcdSpecsPending
        });
    }

    let Some(icd01_file) = icd01_path else {
        return Err(vec!["Check 24 failed: System Interface Matrix 'docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md' is missing.".to_string()]);
    };
    let Some(icd02_file) = icd02_path else {
        return Err(vec!["Check 24 failed: Master Signal Flow Dictionary 'docs/interfaces/ICD_02_MASTER_SIGNAL_DICTIONARY.md' is missing.".to_string()]);
    };

    let icd01_content = fs::read_to_string(&icd01_file)
        .map_err(|e| vec![format!("Check 24 failed: Failed to read ICD_01: {}", e)])?;
    let icd02_content = fs::read_to_string(&icd02_file)
        .map_err(|e| vec![format!("Check 24 failed: Failed to read ICD_02: {}", e)])?;

    let mut ast_nested_subsystems = BTreeSet::new();
    let mut ast_nested_ports = BTreeSet::new();
    if let Some(ref mtext) = model_text {
        let ast_parts = parse_part_defs(mtext);
        collect_ast_nested_subsystems_and_ports(&ast_parts, false, &mut ast_nested_subsystems, &mut ast_nested_ports);
    }

    let mut errors = Vec::new();
    let declared_ports = parse_icd01_declared_ports(&icd01_content, &mut errors);
    let valid_connections = audit_icd01_connections(
        (&icd01_content, &icd02_content),
        &declared_ports,
        (&ast_nested_subsystems, &ast_nested_ports),
        &mut errors,
    );
    audit_icd02_signals(&icd02_content, &declared_ports, &valid_connections, &mut errors);

    if errors.is_empty() { Ok(ParityOutcome::Verified) } else { Err(errors) }
}

/// Discovers `ICD_01` and `ICD_02` Markdown files in `interfaces_dir`.
/// Realises: [REQ-PARITY-GATE-24]
fn discover_icd_files(interfaces_dir: &Path) -> (Option<PathBuf>, Option<PathBuf>) {
    let mut icd01_path = None;
    let mut icd02_path = None;
    if let Ok(entries) = fs::read_dir(interfaces_dir) {
        let mut entry_list: Vec<_> = entries.flatten().collect();
        entry_list.sort_by_key(|e| e.file_name());
        for entry in entry_list {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") {
                if name.contains("ICD_01") || name.contains("SYSTEM_INTERFACE_MATRIX") {
                    icd01_path = Some(entry.path());
                } else if name.contains("ICD_02") || name.contains("MASTER_SIGNAL_DICTIONARY") {
                    icd02_path = Some(entry.path());
                }
            }
        }
    }
    (icd01_path, icd02_path)
}

/// Parses and validates `ICD_01` Section 4 Port Definition Roster rows.
/// Realises: [REQ-PARITY-GATE-24]
fn parse_icd01_declared_ports(
    icd01_content: &str,
    errors: &mut Vec<String>,
) -> BTreeMap<String, PortMeta> {
    let (port_headers, port_rows) = parse_markdown_table_rows(icd01_content, "port definition");
    if port_rows.is_empty() {
        errors.push(
            "Check 24 failed: ICD_01 Port Definition Roster Table (Section 4) is missing or contains 0 rows.".to_string(),
        );
    }

    let port_id_idx = port_headers
        .iter()
        .position(|h| h.contains("port id") || h.contains("id"))
        .unwrap_or(0);
    let subsys_idx = port_headers
        .iter()
        .position(|h| h.contains("subsystem"))
        .unwrap_or(1);
    let port_name_idx = port_headers
        .iter()
        .position(|h| h.contains("port name") || h.contains("name"))
        .unwrap_or(2);
    let dir_idx = port_headers
        .iter()
        .position(|h| h.contains("direction"))
        .unwrap_or(3);

    let mut declared_ports: BTreeMap<String, PortMeta> = BTreeMap::new();
    for row in &port_rows {
        let raw_id = row.get(port_id_idx).map(|s| s.as_str()).unwrap_or("");
        let port_id = extract_port_identifier(raw_id).unwrap_or_else(|| {
            raw_id
                .trim_matches(|c| c == '`' || c == ' ' || c == '*')
                .to_string()
        });
        let subsys = row
            .get(subsys_idx)
            .map(|s| s.as_str())
            .unwrap_or("")
            .trim_matches(|c| c == '`' || c == ' ' || c == '*')
            .to_string();
        let name = row
            .get(port_name_idx)
            .map(|s| s.as_str())
            .unwrap_or("")
            .trim_matches(|c| c == '`' || c == ' ' || c == '*')
            .to_string();
        let dir = row
            .get(dir_idx)
            .map(|s| s.as_str())
            .unwrap_or("")
            .trim_matches(|c| c == '`' || c == ' ' || c == '*')
            .to_ascii_uppercase();

        if port_id.is_empty() || !port_id.starts_with("PORT-") {
            errors.push(format!(
                "Check 24 failed: Invalid Port ID '{}' in Section 4 (must start with 'PORT-')",
                raw_id
            ));
        } else {
            match declared_ports.entry(port_id) {
                Entry::Occupied(e) => errors.push(format!(
                    "Check 24 failed: Duplicate Port ID '{}' declared in Section 4",
                    e.key()
                )),
                Entry::Vacant(e) => {
                    e.insert(PortMeta {
                        subsystem: subsys,
                        name,
                        direction: dir,
                    });
                }
            }
        }
    }
    declared_ports
}

/// Audits `ICD_01` Section 5 Connection Binding Roster rows and dangling ports.
/// Realises: [REQ-PARITY-GATE-24]
fn audit_icd01_connections(
    icd_contents: (&str, &str),
    declared_ports: &BTreeMap<String, PortMeta>,
    ast_nested: (&BTreeSet<String>, &BTreeSet<String>),
    errors: &mut Vec<String>,
) -> BTreeSet<(String, String)> {
    let (icd01_content, icd02_content) = icd_contents;
    let (conn_headers, conn_rows) = parse_markdown_table_rows(icd01_content, "connection binding");
    if conn_rows.is_empty() {
        errors.push(
            "Check 24 failed: ICD_01 Connection Binding Roster Table (Section 5) is missing or contains 0 rows.".to_string(),
        );
    }

    let conn_id_idx = conn_headers
        .iter()
        .position(|h| h.contains("connection id") || h.contains("id"))
        .unwrap_or(0);
    let src_idx = conn_headers
        .iter()
        .position(|h| h.contains("source"))
        .unwrap_or(1);
    let dst_idx = conn_headers
        .iter()
        .position(|h| h.contains("dest"))
        .unwrap_or(2);

    let mut state = Icd01ConnAuditState {
        declared_ports,
        ast_nested_subsystems: ast_nested.0,
        ast_nested_ports: ast_nested.1,
        bound_ports: BTreeSet::new(),
        valid_connections: BTreeSet::new(),
        seen_conn_ids: BTreeSet::new(),
    };

    for row in &conn_rows {
        audit_single_icd01_connection_row(
            row,
            (conn_id_idx, src_idx, dst_idx),
            &mut state,
            errors,
        );
    }

    if conn_rows
        .iter()
        .any(|r| r.first().is_some_and(|c| c.contains("Connection")))
    {
        harvest_icd02_signal_port_bindings(icd02_content, &mut state);
    }

    for (port_id, port) in declared_ports {
        if !state.bound_ports.contains(port_id) {
            errors.push(format!(
                "Check 24 failed: Dangling port detected: declared port '{}' ({}) in subsystem '{}' is not bound to any connection in Section 5",
                port_id, port.name, port.subsystem
            ));
        }
    }
    state.valid_connections
}

/// Audits a single connection row from `ICD_01` Section 5.
/// Realises: [REQ-PARITY-GATE-24]
fn audit_single_icd01_connection_row(
    row: &[String],
    cols: (usize, usize, usize),
    state: &mut Icd01ConnAuditState<'_>,
    errors: &mut Vec<String>,
) {
    let raw_conn_id = row.get(cols.0).map(|s| s.as_str()).unwrap_or("");
    let conn_id = extract_single_conn_id(raw_conn_id).unwrap_or_else(|| {
        raw_conn_id
            .trim_matches(|c| c == '`' || c == ' ' || c == '*')
            .to_string()
    });
    let src_raw = row.get(cols.1).map(|s| s.as_str()).unwrap_or("");
    let dst_raw = row.get(cols.2).map(|s| s.as_str()).unwrap_or("");

    if conn_id.is_empty() || (!conn_id.starts_with("CONN-") && !conn_id.ends_with("Connection")) {
        errors.push(format!(
            "Check 24 failed: Invalid Connection ID '{}' in Section 5 (must start with 'CONN-')",
            raw_conn_id
        ));
    } else if !state.seen_conn_ids.insert(conn_id.clone()) {
        errors.push(format!(
            "Check 24 failed: Duplicate Connection ID '{}' in Section 5",
            conn_id
        ));
    }

    let src_port_id = extract_port_identifier(src_raw);
    let dst_port_id = extract_port_identifier(dst_raw);
    if src_port_id.is_none() {
        errors.push(format!(
            "Check 24 failed: Connection '{}' source port '{}' lacks valid PORT-* identifier",
            conn_id, src_raw
        ));
    }
    if dst_port_id.is_none() {
        errors.push(format!(
            "Check 24 failed: Connection '{}' dest port '{}' lacks valid PORT-* identifier",
            conn_id, dst_raw
        ));
    }

    check_nested_connection_qualification(
        &conn_id,
        (src_raw, dst_raw),
        (&src_port_id, &dst_port_id),
        state,
        errors,
    );
    bind_connection_endpoint(&conn_id, &src_port_id, true, state, errors);
    bind_connection_endpoint(&conn_id, &dst_port_id, false, state, errors);

    if let (Some(s), Some(d)) = (src_port_id, dst_port_id) {
        state.valid_connections.insert((s, d));
    }
}
