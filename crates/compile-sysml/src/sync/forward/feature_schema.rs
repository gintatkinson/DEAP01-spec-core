#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # Feature JSON Schema Synthesis and Constraint Obligation Deduplication
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-JSON-SCHEMA]
//! Projects SysML v2 `PartDef` attributes, ports, defaults, constraints, and `RequirementDef`
//! bindings into Draft 2020-12 JSON Schemas and deduplicated `FeatureConstraintObligation` lists.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Input `PartDef` and `RequirementDef` slices are valid AST nodes.
//! - **Postconditions**: Emits valid RFC 8259 JSON Schema payloads and deduplicated obligations.
//! - **Invariants**: 100% of constraint and requirement obligations are preserved in insertion order.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Non-provenance-safe expressions omit inline safe expression strings without losing obligation IDs.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Pure single-threaded `O(N log N)` deduplication via `BTreeMap`.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Consumed by `feature_emitter` and `feature_renderer`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Verified by Feature JSON Schema and obligation tests in `sync/forward/tests*.rs`.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-JSON-SCHEMA]

use super::*;

/// Formats a single `AttributeDef` into a JSON Schema property block, recording non-defaulted attributes in `required`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-JSON-SCHEMA]
fn render_json_schema_attribute_prop(attr: &AttributeDef, required: &mut Vec<String>) -> String {
    let raw_type = attr.type_name.trim();
    let json_type = match raw_type {
        "Integer" | "int" | "Nat" | "usize" | "u64" | "u32" | "i64" | "i32" => "integer",
        "Real" | "Float" | "float" | "double" | "f64" | "f32" => "number",
        "Boolean" | "bool" => "boolean",
        _ => "string",
    };
    let escaped_attr_name = escape_json_string(&attr.name);
    let display_sysml_type = if raw_type.is_empty() {
        "String"
    } else {
        raw_type
    };
    let escaped_sysml_type = escape_json_string(display_sysml_type);
    let desc_raw = attr
        .doc
        .as_deref()
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            format!(
                "Declared attribute {} ({})",
                attr.name.trim(),
                display_sysml_type
            )
        });
    let escaped_desc = escape_json_string(&desc_raw);

    let mut fields = vec![
        format!("      \"type\": \"{json_type}\""),
        format!("      \"x-sysml-type\": \"{escaped_sysml_type}\""),
        format!("      \"description\": \"{escaped_desc}\""),
    ];

    let default_opt = attr
        .default_value
        .as_deref()
        .map(str::trim)
        .filter(|d| !d.is_empty());
    if let Some(dv) = default_opt {
        let default_lit = format_json_default_literal(dv, json_type);
        fields.push(format!("      \"default\": {default_lit}"));
    } else {
        required.push(format!("\"{escaped_attr_name}\""));
    }

    format!(
        "    \"{escaped_attr_name}\": {{\n{}\n    }}",
        fields.join(",\n")
    )
}

/// Generates a Draft 2020-12 JSON Schema from `PartDef` attributes, ports, defaults, and constraints.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-JSON-SCHEMA]
pub(crate) fn generate_json_schema(part: &PartDef) -> String {
    let mut props = Vec::new();
    let mut required = Vec::new();
    let escaped_part_name = escape_json_string(&part.name);

    props.push(format!(
        "    \"part\": {{\n      \"type\": \"string\",\n      \"const\": \"{escaped_part_name}\"\n    }}"
    ));
    required.push("\"part\"".to_string());

    props.push(
        "    \"status\": {\n      \"type\": \"string\",\n      \"enum\": [\"nominal\", \"degraded\", \"fault\"]\n    }".to_string(),
    );
    required.push("\"status\"".to_string());

    for attr in &part.attributes {
        props.push(render_json_schema_attribute_prop(attr, &mut required));
    }

    for port in &part.ports {
        let escaped_port_name = escape_json_string(&port.name);
        let escaped_dir = escape_json_string(if port.direction.trim().is_empty() {
            "inout"
        } else {
            port.direction.trim()
        });
        let escaped_port_type = escape_json_string(if port.type_name.trim().is_empty() {
            "Port"
        } else {
            port.type_name.trim()
        });
        props.push(format!(
            "    \"{escaped_port_name}\": {{\n      \"type\": \"object\",\n      \"properties\": {{\n        \"direction\": {{ \"type\": \"string\", \"const\": \"{escaped_dir}\" }},\n        \"port_type\": {{ \"type\": \"string\", \"const\": \"{escaped_port_type}\" }}\n      }},\n      \"required\": [\"direction\", \"port_type\"]\n    }}"
        ));
        required.push(format!("\"{escaped_port_name}\""));
    }

    let constraints_extension = if part.constraints.is_empty() {
        String::new()
    } else {
        let entries: Vec<String> = part
            .constraints
            .iter()
            .filter(|c| !c.name.trim().is_empty())
            .map(|c| {
                format!(
                    "    {{ \"name\": \"{}\", \"expression\": \"{}\" }}",
                    escape_json_string(c.name.trim()),
                    escape_json_string(c.expression.trim())
                )
            })
            .collect();
        if entries.is_empty() {
            String::new()
        } else {
            format!(
                ",\n  \"x-sysml-constraints\": [\n{}\n  ]",
                entries.join(",\n")
            )
        }
    };

    format!(
        "{{\n  \"$schema\": \"https://json-schema.org/draft/2020-12/schema\",\n  \"title\": \"{name}PayloadSchema\",\n  \"type\": \"object\",\n  \"properties\": {{\n{props}\n  }},\n  \"required\": [\n    {reqs}\n  ]{constraints_ext},\n  \"additionalProperties\": false\n}}",
        name = escaped_part_name,
        props = props.join(",\n"),
        reqs = required.join(",\n    "),
        constraints_ext = constraints_extension,
    )
}

/// Deduplicated AST invariant or requirement obligation bound to a `PartDef` Feature specification.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-FEATURE-CONSTRAINT-OBLIGATION]
#[derive(Debug, Clone)]
pub(crate) struct FeatureConstraintObligation {
    pub(crate) name: String,
    pub(crate) safe_expr: Option<String>,
    pub(crate) req_trace: Option<String>,
}

/// Inserts or enriches a `RequirementDef`-scoped `ConstraintDef` obligation in `obligations` using `index_by_name`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-COLLECT-FEATURE-OBLIGATIONS]
fn upsert_scoped_constraint_obligation(
    c: &ConstraintDef,
    req_trace: &Option<String>,
    obligations: &mut Vec<FeatureConstraintObligation>,
    index_by_name: &mut BTreeMap<String, usize>,
) {
    let name = c.name.trim();
    if name.is_empty() {
        return;
    }
    if let Some(&existing_idx) = index_by_name.get(name) {
        if let Some(existing) = obligations.get_mut(existing_idx) {
            if existing.req_trace.is_none() {
                existing.req_trace.clone_from(req_trace);
            }
            if existing.safe_expr.is_none() {
                let expr = c.expression.trim();
                if is_provenance_safe_constraint_expr(expr) {
                    existing.safe_expr = Some(expr.to_string());
                }
            }
        }
        return;
    }
    let expr = c.expression.trim();
    let safe_expr = if is_provenance_safe_constraint_expr(expr) {
        Some(expr.to_string())
    } else {
        None
    };
    let idx = obligations.len();
    obligations.push(FeatureConstraintObligation {
        name: name.to_string(),
        safe_expr,
        req_trace: req_trace.clone(),
    });
    index_by_name.insert(name.to_string(), idx);
}

/// Collects 100% of deduplicated constraint and requirement obligations for a `PartDef` without truncation loss.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-COLLECT-FEATURE-OBLIGATIONS]
pub(crate) fn collect_feature_constraint_obligations(
    part: &PartDef,
    reqs: &[&RequirementDef],
) -> Vec<FeatureConstraintObligation> {
    let mut obligations: Vec<FeatureConstraintObligation> = Vec::new();
    let mut index_by_name: BTreeMap<String, usize> = BTreeMap::new();

    for c in &part.constraints {
        let name = c.name.trim();
        if name.is_empty() || index_by_name.contains_key(name) {
            continue;
        }
        let expr = c.expression.trim();
        let safe_expr = if is_provenance_safe_constraint_expr(expr) {
            Some(expr.to_string())
        } else {
            None
        };
        let idx = obligations.len();
        obligations.push(FeatureConstraintObligation {
            name: name.to_string(),
            safe_expr,
            req_trace: None,
        });
        index_by_name.insert(name.to_string(), idx);
    }

    for req in part.requirements.iter().chain(reqs.iter().copied()) {
        let req_label = format_requirement_label(req);
        let req_trace = if req_label.is_empty() {
            None
        } else {
            Some(req_label.clone())
        };

        if !req.constraints.is_empty() {
            for c in &req.constraints {
                upsert_scoped_constraint_obligation(
                    c,
                    &req_trace,
                    &mut obligations,
                    &mut index_by_name,
                );
            }
        } else if !req_label.is_empty() && !index_by_name.contains_key(&req_label) {
            let safe_expr = if is_provenance_safe_constraint_expr(&req.text) {
                Some(req.text.trim().to_string())
            } else {
                None
            };
            let idx = obligations.len();
            obligations.push(FeatureConstraintObligation {
                name: req_label.clone(),
                safe_expr,
                req_trace,
            });
            index_by_name.insert(req_label, idx);
        }
    }

    obligations
}
