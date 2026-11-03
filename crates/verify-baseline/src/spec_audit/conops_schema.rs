//! ConOps JSON Schema Contract and User Class Taxonomy (`UCL-xx`) validators.

use crate::spec_audit::conops_audit::extract_markdown_tables;
use crate::spec_audit::{SpecAuditError, SpecAuditFinding};
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Helper to extract a `required` string set from a JSON Schema node.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-CONOPS-SCHEMA-CONTRACT]
fn extract_required_set(val: &Value) -> BTreeSet<&str> {
    val.get("required")
        .and_then(|r| r.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default()
}

/// Extracts the `required` string set from `obj.properties[key].items`.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-EXTRACT-REQUIRED-SET]
fn extract_nested_item_required_set<'a>(obj: &'a Value, key: &str) -> BTreeSet<&'a str> {
    obj.get("properties")
        .and_then(|p| p.get(key))
        .and_then(|s| s.get("items"))
        .map(extract_required_set)
        .unwrap_or_default()
}

/// Validates `super_system_architecture`, `subsystem_architecture`, and `system_interface_topology` in `props`.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-CONOPS-SCHEMA-CONTRACT]
fn validate_schema_arch_props(
    props: &Map<String, Value>,
    schema_path: &str,
    findings: &mut Vec<SpecAuditFinding>,
) {
    if let Some(super_sys) = props.get("super_system_architecture") {
        let is_obj = super_sys.get("type").and_then(|t| t.as_str()) == Some("object");
        let super_req = extract_required_set(super_sys);
        let seg_items_req = extract_nested_item_required_set(super_sys, "segments");
        let ok = is_obj
            && super_req.contains("segments")
            && super_req.contains("boundary_diagram_mermaid")
            && ["segment_id", "segment_name", "operational_role"].iter().all(|k| seg_items_req.contains(k));
        if !ok {
            findings.push(SpecAuditFinding::new(
                "conops-schema-super-system-architecture-incomplete",
                format!("Property 'super_system_architecture' in '{}' is incomplete; requires object with segments (segment_id, segment_name, operational_role) and boundary_diagram_mermaid.", schema_path),
                schema_path, None,
            ));
        }
    } else {
        findings.push(SpecAuditFinding::new(
            "conops-schema-super-system-architecture-missing",
            format!("Top-level schema properties in '{}' omits 'super_system_architecture'.", schema_path),
            schema_path, None,
        ));
    }

    if let Some(subsys) = props.get("subsystem_architecture") {
        let is_obj = subsys.get("type").and_then(|t| t.as_str()) == Some("object");
        let subsys_req = extract_required_set(subsys);
        let subsys_items_req = extract_nested_item_required_set(subsys, "subsystems");
        let ok = is_obj
            && subsys_req.contains("subsystems")
            && subsys_req.contains("ast_part_allocations")
            && ["subsystem_id", "subsystem_name", "ast_part_ref", "functional_description"].iter().all(|k| subsys_items_req.contains(k));
        if !ok {
            findings.push(SpecAuditFinding::new(
                "conops-schema-subsystem-architecture-incomplete",
                format!("Property 'subsystem_architecture' in '{}' is incomplete; requires object with subsystems (subsystem_id, subsystem_name, ast_part_ref, functional_description) and ast_part_allocations.", schema_path),
                schema_path, None,
            ));
        }
    } else {
        findings.push(SpecAuditFinding::new(
            "conops-schema-subsystem-architecture-missing",
            format!("Top-level schema properties in '{}' omits 'subsystem_architecture'.", schema_path),
            schema_path, None,
        ));
    }

    if let Some(topo) = props.get("system_interface_topology") {
        let is_obj = topo.get("type").and_then(|t| t.as_str()) == Some("object");
        let topo_req = extract_required_set(topo);
        let link_items_req = extract_nested_item_required_set(topo, "interface_links");
        let ok = is_obj
            && topo_req.contains("interface_links")
            && ["link_id", "source_subsystem", "target_subsystem", "protocol_type"].iter().all(|k| link_items_req.contains(k));
        if !ok {
            findings.push(SpecAuditFinding::new(
                "conops-schema-interface-topology-incomplete",
                format!("Property 'system_interface_topology' in '{}' is incomplete; requires object with interface_links (link_id, source_subsystem, target_subsystem, protocol_type).", schema_path),
                schema_path, None,
            ));
        }
    } else {
        findings.push(SpecAuditFinding::new(
            "conops-schema-interface-topology-missing",
            format!("Top-level schema properties in '{}' omits 'system_interface_topology'.", schema_path),
            schema_path, None,
        ));
    }
}

/// Validates `user_classes` property structure in `props`.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-CONOPS-SCHEMA-CONTRACT]
fn validate_schema_user_classes_prop(
    props: &Map<String, Value>,
    schema_path: &str,
    findings: &mut Vec<SpecAuditFinding>,
) {
    let Some(user_classes) = props.get("user_classes") else {
        findings.push(SpecAuditFinding::new(
            "conops-schema-user-classes-missing",
            format!("Top-level schema properties in '{}' omits 'user_classes'.", schema_path),
            schema_path,
            None,
        ));
        return;
    };

    let is_arr = user_classes.get("type").and_then(|t| t.as_str()) == Some("array");
    let min_items = user_classes
        .get("minItems")
        .and_then(|m| m.as_u64())
        .unwrap_or(0);
    let items_req = user_classes
        .get("items")
        .map(extract_required_set)
        .unwrap_or_default();
    let has_req_fields = [
        "id",
        "title",
        "player_or_operator",
        "interfacing_stakeholder",
        "characteristics",
        "training_type",
        "constraint_source",
    ]
    .iter()
    .all(|f| items_req.contains(f));

    if !is_arr || min_items < 1 || !has_req_fields {
        findings.push(SpecAuditFinding::new(
            "conops-schema-user-classes-incomplete",
            format!("Property 'user_classes' in '{}' is incomplete; requires array with minItems >= 1 and items required fields (id, title, player_or_operator, interfacing_stakeholder, characteristics, training_type, constraint_source).", schema_path),
            schema_path,
            None,
        ));
    }

    let id_pattern = user_classes
        .get("items")
        .and_then(|i| i.get("properties"))
        .and_then(|p| p.get("id"))
        .and_then(|id| id.get("pattern"))
        .and_then(|p| p.as_str());
    if id_pattern != Some("^UCL-[0-9]{2}$") {
        findings.push(SpecAuditFinding::new(
            "conops-schema-user-classes-taxonomy-collision",
            format!("Property 'user_classes.items.properties.id.pattern' in '{}' must be '^UCL-[0-9]{{2}}$' to enforce unambiguous User Class taxonomy and prevent collision with Level 2 System Use Cases (refs #291).", schema_path),
            schema_path,
            None,
        ));
    }
}

/// Audits top-level schema contract in `.pipeline/schemas/conops_specification_schema.json`.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-CONOPS-SCHEMA-CONTRACT]
pub fn validate_conops_schema_contract(schema: &Value, schema_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();

    if schema.get("required").and_then(|r| r.as_array()).is_none() {
        findings.push(SpecAuditFinding::new(
            "conops-schema-required-array-missing",
            format!("Top-level schema in '{}' has no required array.", schema_path),
            schema_path,
            None,
        ));
        return findings;
    }

    let required_props = extract_required_set(schema);
    for (prop, rule_id) in [
        ("super_system_architecture", "conops-schema-super-system-architecture-missing"),
        ("subsystem_architecture", "conops-schema-subsystem-architecture-missing"),
        ("system_interface_topology", "conops-schema-interface-topology-missing"),
        ("user_classes", "conops-schema-user-classes-missing"),
    ] {
        if !required_props.contains(prop) {
            findings.push(SpecAuditFinding::new(
                rule_id,
                format!("Top-level schema contract required array in '{}' omits '{}'.", schema_path, prop),
                schema_path,
                None,
            ));
        }
    }

    let Some(props) = schema.get("properties").and_then(|p| p.as_object()) else {
        findings.push(SpecAuditFinding::new(
            "conops-schema-properties-missing",
            format!("Top-level schema in '{}' has no properties object.", schema_path),
            schema_path,
            None,
        ));
        return findings;
    };

    validate_schema_arch_props(props, schema_path, &mut findings);
    validate_schema_user_classes_prop(props, schema_path, &mut findings);
    findings
}

/// Audits top-level schema contract file on disk.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-CONOPS-SCHEMA-FILE]
pub fn validate_conops_schema_file(
    schema_path: &Path,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    if !schema_path.is_file() {
        return Ok(Vec::new());
    }
    let content =
        fs::read_to_string(schema_path).map_err(|e| SpecAuditError::IoError(e.to_string()))?;
    let json_val: Value =
        serde_json::from_str(&content).map_err(|e| SpecAuditError::RuleError(e.to_string()))?;
    let path_str = schema_path.to_string_lossy().to_string();
    Ok(validate_conops_schema_contract(&json_val, &path_str))
}

/// Scans prose lines for legacy `UC-xx` tokens colliding with Level 2 System Use Cases.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-USER-CLASSES]
fn scan_prose_uc_collisions(
    content: &str,
    rel_path: &str,
    findings: &mut Vec<SpecAuditFinding>,
) {
    for (line_idx, line) in content.lines().enumerate() {
        let line_no = line_idx + 1;
        let trimmed = line.trim();
        if trimmed.starts_with('#') || (trimmed.starts_with('|') && trimmed.ends_with('|')) {
            continue;
        }

        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let is_uc = bytes.get(i).is_some_and(|&b0| b0 == b'U' || b0 == b'u')
                && bytes.get(i + 1).is_some_and(|&b1| b1 == b'C' || b1 == b'c')
                && bytes.get(i + 2) == Some(&b'-');
            let left_ok = i == 0
                || bytes
                    .get(i.saturating_sub(1))
                    .is_none_or(|&b| !(b.is_ascii_alphanumeric() || b == b'_' || b == b'-'));
            if is_uc && left_ok {
                let digit_start = i + 3;
                let mut j = digit_start;
                while bytes.get(j).is_some_and(|b| b.is_ascii_digit()) {
                    j += 1;
                }
                let right_ok = j == bytes.len()
                    || bytes
                        .get(j)
                        .is_none_or(|&b| !(b.is_ascii_alphanumeric() || b == b'_' || b == b'-'));
                if j > digit_start && right_ok {
                    if let Some(matched) = line.get(i..j) {
                        findings.push(SpecAuditFinding::new(
                            "conops-user-class-taxonomy-collision",
                            format!(
                                "Found colliding prefix '{}' in '{}' at line {}. ConOps documents must not cite legacy 'UC-xx' User Classes or Level 2 System Use Cases; use 'UCL-xx' for User Classes per ISO/IEC/IEEE 29148:2018 §5.2.4 (refs #291).",
                                matched, rel_path, line_no
                            ),
                            rel_path,
                            Some(line_no),
                        ));
                        i = j;
                        continue;
                    }
                }
            }
            i += 1;
        }
    }
}

/// Audits ConOps documents for User Class taxonomy disambiguation (`UCL-xx`) per ISO/IEC/IEEE 29148:2018 §5.2.4.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-USER-CLASSES]
pub fn validate_user_classes(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();

    for tbl in extract_markdown_tables(content) {
        let id_col_idx = tbl.headers.iter().position(|h| {
            let h_lower = h.to_lowercase();
            h_lower.contains("user class id") || h_lower.contains("user class")
        });

        if let Some(col) = id_col_idx {
            for row in &tbl.rows {
                if let Some(cell) = row.cells.get(col) {
                    let raw_id = cell
                        .trim()
                        .trim_matches(|c| matches!(c, '*' | '_' | '`'))
                        .trim();
                    if (raw_id.starts_with("UC-") || raw_id.starts_with("uc-"))
                        && !raw_id.starts_with("UCL-")
                        && !raw_id.starts_with("ucl-")
                    {
                        findings.push(SpecAuditFinding::new(
                            "conops-user-class-taxonomy-collision",
                            format!(
                                "User Class '{}' in '{}' at line {} uses legacy 'UC-xx' prefix colliding with Level 2 System Use Cases. User Classes must use unambiguous 'UCL-xx' prefix per ISO/IEC/IEEE 29148:2018 §5.2.4 (refs #291).",
                                raw_id, rel_path, row.line_no
                            ),
                            rel_path,
                            Some(row.line_no),
                        ));
                    }
                }
            }
        }
    }

    scan_prose_uc_collisions(content, rel_path, &mut findings);
    findings
}
