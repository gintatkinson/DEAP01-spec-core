//! Markdown table AST processors for Ports/Interfaces, Parametric Constraints, and Key-Value Properties.

use std::collections::BTreeMap;

use crate::table::{
    extract_provenance_citation, parse_numeric_with_unit, parse_range_bounds, sanitize_identifier,
    MarkdownTable, PROVENANCE_COLUMNS,
};
use deap_core::sysml_ast::{
    AttributeDef, ConnectionDef, ConstraintDef, FlowDef, PackageDef, PartDef, PortDef,
};

use super::md_common::{compose_sanitized_doc, escape_sysml_string_literal};

/// Maps each normalized table header column name to its extracted engineering unit string. Realises: [REQ-INGEST-MD-PARAMETRIC]
fn build_unit_map(t: &MarkdownTable) -> BTreeMap<String, String> {
    t.normalized_headers
        .iter()
        .enumerate()
        .map(|(i, h)| (h.clone(), t.units.get(i).cloned().unwrap_or_default()))
        .collect()
}

/// Returns the trimmed cell string stripped of surrounding backticks and asterisks for `col`. Realises: [REQ-INGEST-MD-PARAMETRIC]
fn cell_clean<'a>(row: &'a BTreeMap<String, String>, col: Option<&String>) -> &'a str {
    col.and_then(|c| row.get(c))
        .map_or("", |v| v.trim().trim_matches(['`', '*', ' ']))
}

/// Appends `attr` to `target_part` (or `pkg.attribute_defs`) when no attribute with `attr.name` exists. Realises: [REQ-INGEST-MD-PARAMETRIC]
fn push_unique_attr(
    target_part: &mut Option<&mut PartDef>,
    pkg: &mut PackageDef,
    attr: AttributeDef,
) {
    let list = target_part.as_deref_mut().map_or(&mut pkg.attribute_defs, |p| &mut p.attributes);
    if !list.iter().any(|a| a.name == attr.name) {
        list.push(attr);
    }
}

/// Resolves canonical SysML v2 `(type_name, default_value)` from explicit type and raw cell value.
/// Realises: [REQ-SYSML-INGEST-MD/process_properties_table]
pub fn resolve_typed_default(
    raw_type: &str,
    raw_def: &str,
    int_v: Option<i64>,
    num_v: Option<f64>,
    empty_type_override: Option<&str>,
) -> (String, Option<String>) {
    let type_name = if !raw_type.is_empty() {
        match raw_type.to_ascii_lowercase().as_str() {
            "int" | "integer" | "usize" | "u64" | "u32" | "u16" | "u8" | "isize" | "i64"
            | "i32" | "i16" | "i8" => "Integer".to_string(),
            "real" | "float" | "float32" | "float64" | "double" | "f64" | "f32" => "Real".to_string(),
            "bool" | "boolean" => "Boolean".to_string(),
            "str" | "string" => "String".to_string(),
            _ => sanitize_identifier(raw_type, empty_type_override.unwrap_or("String")),
        }
    } else if let Some(ov) = empty_type_override {
        ov.to_string()
    } else if int_v.is_some() && !raw_def.contains('.') {
        "Integer".to_string()
    } else if num_v.is_some() {
        "Real".to_string()
    } else if raw_def.eq_ignore_ascii_case("true") || raw_def.eq_ignore_ascii_case("false") {
        "Boolean".to_string()
    } else {
        "String".to_string()
    };

    let def_val = if raw_def.is_empty() {
        None
    } else if type_name == "Integer" {
        int_v.or_else(|| raw_def.parse::<i64>().ok()).map(|v| v.to_string())
    } else if type_name == "Real" {
        num_v.or_else(|| raw_def.parse::<f64>().ok()).map(|v| v.to_string())
    } else if type_name == "Boolean" {
        (raw_def.eq_ignore_ascii_case("true") || raw_def.eq_ignore_ascii_case("false"))
            .then(|| raw_def.to_ascii_lowercase())
    } else {
        Some(escape_sysml_string_literal(raw_def))
    };
    (type_name, def_val)
}

/// Normalizes a raw interface latency cell string and unit into milliseconds (`f64`).
/// Realises: [REQ-SYSML-INGEST-MD/parse_latency_to_ms]
pub fn parse_latency_to_ms(raw: &str, col_unit: &str) -> Option<f64> {
    let cleaned = raw
        .trim()
        .trim_matches(['`', '*', ' '])
        .trim_start_matches(['<', '>', '=', '~', '\u{2264}', '\u{2265}', ' ']);
    let (val, val_unit, _) = parse_numeric_with_unit(cleaned);
    let num = val?;
    let unit = val_unit.as_deref().filter(|u| !u.is_empty()).unwrap_or(col_unit).to_ascii_lowercase();
    if matches!(unit.as_str(), "s" | "sec" | "secs" | "second" | "seconds") {
        Some(num * 1000.0)
    } else if matches!(unit.as_str(), "us" | "\u{00b5}s" | "\u{03bc}s" | "usec" | "microseconds") {
        Some(num / 1000.0)
    } else {
        Some(num)
    }
}

/// Extracts a `ConnectionDef` from a port table row when source and target port columns are populated.
/// Realises: [REQ-SYSML-INGEST-MD/process_ports_table]
fn extract_row_connection(row: &BTreeMap<String, String>, headers: &[String], unit_map: &BTreeMap<String, String>, port_name: &str, protocol: &str) -> Option<ConnectionDef> {
    let find_col = |keys: &[&str]| headers.iter().find(|h| keys.contains(&h.as_str()));
    let (s_col, t_col) = (find_col(&["source_port", "source", "source_part"])?, find_col(&["target_port", "target", "target_part"])?);
    let (src_val, tgt_val) = (cell_clean(row, Some(s_col)), cell_clean(row, Some(t_col)));
    if src_val.is_empty() || tgt_val.is_empty() {
        return None;
    }
    let doc_col = find_col(&["description", "doc", "notes"]);
    let latency_col = find_col(&["latency", "latency_ms", "latency_budget", "max_latency", "propagation_latency"]);
    let cit = extract_provenance_citation(row, headers);
    let raw_desc = doc_col.and_then(|c| row.get(c)).map_or("", |v| v.trim());
    let conn_doc = compose_sanitized_doc(raw_desc, &cit, "");
    let latency_ms = latency_col.and_then(|c| parse_latency_to_ms(row.get(c)?, unit_map.get(c).map_or("", String::as_str)));
    Some(ConnectionDef {
        name: format!("conn_{port_name}"),
        source_port: src_val.to_string(),
        target_port: tgt_val.to_string(),
        source_part: src_val.split_once('.').map(|(p, _)| p.trim_matches(['`', '*', ' ']).to_string()),
        target_part: tgt_val.split_once('.').map(|(p, _)| p.trim_matches(['`', '*', ' ']).to_string()),
        doc: (!conn_doc.is_empty()).then_some(conn_doc),
        severity: 1,
        protocol: (!protocol.is_empty()).then(|| protocol.to_string()),
        latency_ms,
        ..Default::default()
    })
}

/// Processes Port/Signal/Interface tables into `PortDef`, `FlowDef`, and `ConnectionDef` AST nodes.
/// Realises: [REQ-SYSML-INGEST-MD/process_ports_table]
pub fn process_ports_table(table: &MarkdownTable, mut target_part: Option<&mut PartDef>, pkg: &mut PackageDef) {
    let headers = &table.normalized_headers;
    let unit_map = build_unit_map(table);
    let find_col = |keys: &[&str]| headers.iter().find(|h| keys.contains(&h.as_str()));
    let name_col = find_col(&["port", "port_name", "signal", "signal_name", "signal_id", "interface", "name", "flow"])
        .or_else(|| headers.first())
        .cloned()
        .unwrap_or_else(|| "port".to_string());
    let dir_col = find_col(&["direction", "dir", "flow_direction"]);
    let type_col = find_col(&["type", "data_type", "item_type", "payload", "port_type"]);
    let protocol_col = find_col(&["protocol", "protocol_family", "bus"]);
    let rate_col = find_col(&["rate", "rate_hz", "frequency", "update_rate"]);
    let range_col = find_col(&["range", "valid_range"]);
    let unit_col = find_col(&["unit", "units", "engineering_units"]);
    let def_col = find_col(&["default", "default_value"]);
    let doc_col = find_col(&["description", "doc", "notes"]);

    for row in &table.rows {
        let raw_name = row.get(&name_col).map_or("", |v| v.trim());
        if raw_name.is_empty() {
            continue;
        }
        let port_name = sanitize_identifier(raw_name, "port");
        let direction = match cell_clean(row, dir_col).to_ascii_lowercase().as_str() {
            "in" | "input" | "rx" => "in",
            "out" | "output" | "tx" => "out",
            _ => "inout",
        }
        .to_string();
        let (type_name, _) = resolve_typed_default(cell_clean(row, type_col), "", None, None, Some("Port"));
        let protocol = cell_clean(row, protocol_col).to_string();
        let cit = extract_provenance_citation(row, headers);
        let raw_desc = doc_col.and_then(|c| row.get(c)).map_or("", |v| v.trim());
        let doc = compose_sanitized_doc(raw_desc, &cit, "");
        let rate_hz = rate_col.and_then(|c| row.get(c)).and_then(|v| parse_numeric_with_unit(v).0);
        let eff_unit = unit_col.and_then(|c| row.get(c)).map(String::as_str).map(str::trim).filter(|u| !u.is_empty()).map_or_else(|| unit_map.get(&name_col).cloned().unwrap_or_default(), str::to_string);
        let val_range = range_col.and_then(|c| row.get(c)).map(|v| v.trim().to_string());
        let def_val = def_col.and_then(|c| row.get(c)).map(|v| v.trim().to_string());

        let mut item_flows = Vec::new();
        if rate_hz.is_some() || !eff_unit.is_empty() || val_range.is_some() || (type_name != "Port" && direction != "inout") {
            let flow_doc = compose_sanitized_doc(raw_desc, &cit, &eff_unit);
            item_flows.push(FlowDef {
                name: format!("{port_name}_flow"),
                direction: if direction != "inout" { direction.clone() } else { "out".to_string() },
                item_type: if type_name != "Port" { type_name.clone() } else { "Item".to_string() },
                doc: (!flow_doc.is_empty()).then_some(flow_doc),
                rate_hz,
                unit: (!eff_unit.is_empty()).then_some(eff_unit),
                valid_range: val_range,
                default_value: def_val,
            });
        }
        let port_def = PortDef { name: port_name.clone(), type_name: type_name.clone(), direction, doc: (!doc.is_empty()).then_some(doc), port_category: "DataPort".to_string(), protocol_family: protocol.clone(), item_flows, ..Default::default() };
        let port_list = target_part.as_deref_mut().map_or(&mut pkg.port_defs, |p| &mut p.ports);
        if !port_list.iter().any(|p| p.name == port_name) {
            port_list.push(port_def);
        }
        if let Some(conn) = extract_row_connection(row, headers, &unit_map, &port_name, &protocol) {
            if !pkg.connection_defs.iter().any(|c| c.name == conn.name) {
                pkg.connection_defs.push(conn);
            }
        }
    }
}

/// Resolves the effective engineering unit and boolean constraint expression for a constraint table row.
/// Realises: [REQ-SYSML-INGEST-MD/process_constraints_table]
fn extract_constraint_unit_and_expr(
    row: &BTreeMap<String, String>,
    headers: &[String],
    unit_map: &BTreeMap<String, String>,
    param_col: &str,
    param_name: &str,
) -> (String, String) {
    let find_col = |keys: &[&str]| headers.iter().find(|h| keys.contains(&h.as_str()));
    let min_col = find_col(&["min", "minimum", "lower_bound", "min_val"]);
    let max_col = find_col(&["max", "maximum", "upper_bound", "max_val"]);
    let range_col = find_col(&["range", "valid_range", "bounds", "limit", "bound"]);
    let unit_col = find_col(&["unit", "units", "engineering_units"]);
    let constraint_col = find_col(&["constraint", "condition", "invariant", "assertion"]);

    let mut eff_unit = unit_col.and_then(|c| row.get(c)).map(String::as_str).map(str::trim).filter(|u| !u.is_empty()).map_or_else(|| unit_map.get(param_col).cloned().unwrap_or_default(), str::to_string);
    let (mut min_val, mut max_val) = (None, None);
    for (col_opt, dst) in [(min_col, &mut min_val), (max_col, &mut max_val)] {
        if let Some(v_str) = col_opt.and_then(|c| row.get(c)) {
            let (v, u, _) = parse_numeric_with_unit(v_str);
            *dst = v;
            if let Some(us) = u.filter(|_| eff_unit.is_empty()) {
                eff_unit = us;
            }
        }
    }
    if let Some(v_str) = (min_val.is_none() || max_val.is_none()).then_some(range_col).flatten().and_then(|rc| row.get(rc)) {
        let (r_min, r_max) = parse_range_bounds(v_str);
        min_val = min_val.or(r_min);
        max_val = max_val.or(r_max);
    }
    let expression = if let Some(c) = constraint_col {
        row.get(c).map_or("", |v| v.trim().trim_matches('`').trim()).trim_end_matches(';').trim().to_string()
    } else if let (Some(mn), Some(mx)) = (min_val, max_val) {
        format!("{param_name} >= {mn} and {param_name} <= {mx}")
    } else if let Some(mn) = min_val {
        format!("{param_name} >= {mn}")
    } else if let Some(mx) = max_val {
        format!("{param_name} <= {mx}")
    } else {
        String::new()
    };
    (eff_unit, expression)
}

/// Processes Parametric Limits and Constraints into `ConstraintDef` and `AttributeDef` AST nodes.
/// Realises: [REQ-SYSML-INGEST-MD/process_constraints_table]
pub fn process_constraints_table(
    table: &MarkdownTable,
    mut target_part: Option<&mut PartDef>,
    pkg: &mut PackageDef,
) {
    let headers = &table.normalized_headers;
    let unit_map = build_unit_map(table);
    let find_col = |keys: &[&str]| headers.iter().find(|h| keys.contains(&h.as_str()));
    let con_name_col = find_col(&["constraint_name", "constraint_id", "rule_name", "invariant_name"]);
    let param_col = find_col(&["parameter", "param", "property", "metric", "variable", "name"])
        .or_else(|| headers.iter().find(|h| Some(*h) != con_name_col))
        .or_else(|| headers.first())
        .cloned()
        .unwrap_or_else(|| "parameter".to_string());
    let type_col = find_col(&["type", "data_type"]);
    let def_col = find_col(&["default", "default_value", "nominal", "value"]);
    let doc_col = find_col(&["description", "doc", "notes"]);

    for row in &table.rows {
        let raw_param = row.get(&param_col).map_or("", |v| v.trim());
        if raw_param.is_empty() {
            continue;
        }
        let param_name = sanitize_identifier(raw_param, "param");
        let cit = extract_provenance_citation(row, headers);
        let raw_desc = doc_col.and_then(|c| row.get(c)).map_or("", |v| v.trim());
        let (eff_unit, expression) = extract_constraint_unit_and_expr(row, headers, &unit_map, &param_col, &param_name);
        if !expression.is_empty() {
            let raw_cn = cell_clean(row, con_name_col);
            let con_name = if !raw_cn.is_empty() { sanitize_identifier(raw_cn, "Constraint") } else { format!("assert_{param_name}_range") };
            let con_doc = compose_sanitized_doc(raw_desc, &cit, &eff_unit);
            let con_def = ConstraintDef {
                name: con_name.clone(),
                expression,
                is_assertion: true,
                doc: (!con_doc.is_empty()).then_some(con_doc),
                ..Default::default()
            };
            let con_list = target_part.as_deref_mut().map_or(&mut pkg.constraint_defs, |p| &mut p.constraints);
            if !con_list.iter().any(|c| c.name == con_name) {
                con_list.push(con_def);
            }
        }
        let (raw_type, raw_def) = (cell_clean(row, type_col), cell_clean(row, def_col));
        let (num_v, _, int_v) = parse_numeric_with_unit(raw_def);
        let (type_name, default_value) = resolve_typed_default(raw_type, raw_def, int_v, num_v, Some("Real"));
        let attr_doc = compose_sanitized_doc(raw_desc, &cit, &eff_unit);
        push_unique_attr(&mut target_part, pkg, AttributeDef { name: param_name, type_name, default_value, doc: (!attr_doc.is_empty()).then_some(attr_doc) });
    }
}

/// Processes 2-3 column Key-Value property tables into `AttributeDef` AST nodes.
/// Realises: [REQ-SYSML-INGEST-MD/process_properties_table]
pub fn process_properties_table(
    table: &MarkdownTable,
    mut target_part: Option<&mut PartDef>,
    pkg: &mut PackageDef,
) {
    let headers = &table.normalized_headers;
    if headers.is_empty()
        || headers.iter().any(|h| {
            ["derivation_grade", "ssot_input_anchor", "methodology_citation", "claim_requirement", "caveat_downstream_impact"]
                .iter()
                .any(|k| h.contains(k))
        })
    {
        return;
    }
    let Some(k_col) = headers.first() else { return };
    let find_col = |keys: &[&str]| headers.iter().find(|h| keys.contains(&h.as_str()));
    let type_col = find_col(&["type", "data_type", "attribute_type", "value_type"]);
    let def_col = find_col(&["default", "default_bound", "default_value", "bound", "bounds", "value", "nominal", "initial_value", "limit", "range", "valid_range"]);
    let mut d_col = find_col(&["description", "doc", "notes", "comment", "details"]);
    let unit_col = find_col(&["unit", "units", "engineering_units"]);
    let v_col = def_col.or_else(|| {
        headers.iter().skip(1).find(|h| Some(*h) != type_col && Some(*h) != unit_col && Some(*h) != d_col && !PROVENANCE_COLUMNS.contains(&h.as_str()))
    });
    if d_col.is_none() {
        d_col = headers.iter().skip(2).find(|h| !PROVENANCE_COLUMNS.contains(&h.as_str()) && ![unit_col, type_col, def_col, v_col].contains(&Some(*h)));
    }

    for row in &table.rows {
        let raw_k = row.get(k_col).map_or("", |v| v.trim());
        let raw_v = v_col.and_then(|c| row.get(c)).map_or("", |v| v.trim());
        if raw_k.is_empty() {
            continue;
        }
        let attr_name = sanitize_identifier(raw_k, "property");
        let cit = extract_provenance_citation(row, headers);
        let raw_desc = d_col.and_then(|c| row.get(c)).map_or("", |v| v.trim());
        let clean_v = raw_v.trim_matches('`').trim();
        let (nv, pu, iv) = parse_numeric_with_unit(clean_v);
        let is_non_unit_ident = pu.as_deref().is_some_and(|u| {
            !u.is_empty() && !clean_v.contains(char::is_whitespace) && (u.contains('-') || (!u.starts_with('^') && !u.starts_with('/') && u.chars().any(|c| c.is_ascii_digit())))
        });
        let (num_v, parsed_unit, int_v) = if is_non_unit_ident { (None, None, None) } else { (nv, pu, iv) };
        let row_unit = cell_clean(row, unit_col);
        let unit = if !row_unit.is_empty() { row_unit.to_string() } else { parsed_unit.unwrap_or_default() };
        let (type_name, default_value) = resolve_typed_default(cell_clean(row, type_col), clean_v, int_v, num_v, None);
        let eff_doc = compose_sanitized_doc(raw_desc, &cit, &unit);
        push_unique_attr(&mut target_part, pkg, AttributeDef { name: attr_name, type_name, default_value, doc: (!eff_doc.is_empty()).then_some(eff_doc) });
    }
}

/// Generic table processor falling back to property-style processing.
/// Realises: [REQ-SYSML-INGEST-MD/process_generic_table]
pub fn process_generic_table(
    table: &MarkdownTable,
    target_part: Option<&mut PartDef>,
    pkg: &mut PackageDef,
) {
    let is_prop_header = (2..=6).contains(&table.normalized_headers.len())
        && table.normalized_headers.first().is_some_and(|first| {
            ["property", "parameter", "attribute", "key", "field", "name"].iter().any(|k| first.contains(k))
        });
    if (2..=3).contains(&table.normalized_headers.len()) || is_prop_header {
        process_properties_table(table, target_part, pkg);
    }
}
