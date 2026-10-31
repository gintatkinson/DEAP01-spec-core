//! Feature JSON Schema synthesis, constraint obligation deduplication, and BDD Acceptance Criteria rendering.
//!
//! Safety & Traceability: Realises: [REQ-SYNC-FORWARD-GENERATE-FEATURE-BDD]
//! Contract: Projects SysML v2 `PartDef` attributes, ports, defaults, constraints, and `RequirementDef` bindings into Draft 2020-12 JSON Schemas and lossless `3..=10` positive/negative BDD Acceptance Criteria (`AC-01`..`AC-10`), and re-exports Feature class diagram and operation bullet codecs.
//! Bounds & Faults: Enforces RFC 8259 string escaping, `BTreeMap`-ordered obligation deduplication without truncation, bounds-checked slice accessors, and saturating index arithmetic with zero runtime panics.

pub use super::feature_diagrams::{
    format_markdown_operation_bullet, format_mermaid_operation_signature,
    parse_markdown_operation_bullet,
};
use super::*;

/// Formats a single `AttributeDef` into a JSON Schema property block, recording non-defaulted attributes in `required`.
/// Realises: [REQ-SYNC-FORWARD-GENERATE-JSON-SCHEMA]
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
/// Realises: [REQ-SYNC-FORWARD-GENERATE-JSON-SCHEMA]
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
/// Realises: [REQ-SYNC-FORWARD-FEATURE-CONSTRAINT-OBLIGATION]
#[derive(Debug, Clone)]
pub(crate) struct FeatureConstraintObligation {
    pub(crate) name: String,
    pub(crate) safe_expr: Option<String>,
    pub(crate) req_trace: Option<String>,
}

/// Inserts or enriches a `RequirementDef`-scoped `ConstraintDef` obligation in `obligations` using `index_by_name`.
/// Realises: [REQ-SYNC-FORWARD-COLLECT-FEATURE-OBLIGATIONS]
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
/// Realises: [REQ-SYNC-FORWARD-COLLECT-FEATURE-OBLIGATIONS]
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

/// Formats a `FeatureConstraintObligation` into a provenance-safe inline clause for Feature BDD criteria.
/// Realises: [REQ-SYNC-FORWARD-FORMAT-FEATURE-OBLIGATION]
pub(crate) fn format_feature_obligation_clause(ob: &FeatureConstraintObligation) -> String {
    let base = match &ob.safe_expr {
        Some(expr) => format!("`{}` (`{}`)", ob.name, sanitize_prose_links(expr)),
        None => format!("`{}`", ob.name),
    };
    match &ob.req_trace {
        Some(rt) if rt != &ob.name => format!("{base} [Req `{rt}`]"),
        _ => base,
    }
}

/// Summarizes a `PartDef`'s declared attributes, ports, and diagnostic rejection channel for Feature BDD prose.
/// Realises: [REQ-SYNC-FORWARD-SUMMARIZE-PART-INTERFACES]
pub(crate) fn summarize_part_interface_identifiers(part: &PartDef) -> (String, String, String) {
    let attr_summary = if part.attributes.is_empty() {
        format!("`{}` state representation", part.name)
    } else {
        let items: Vec<String> = part
            .attributes
            .iter()
            .map(|a| {
                let t = if a.type_name.trim().is_empty() {
                    "String"
                } else {
                    a.type_name.trim()
                };
                format!("`{}` (`{}`)", a.name.trim(), t)
            })
            .collect();
        items.join(", ")
    };

    let port_summary = if part.ports.is_empty() {
        format!("`{}` structural boundary", part.name)
    } else {
        let items: Vec<String> = part
            .ports
            .iter()
            .map(|p| {
                let dir = if p.direction.trim().is_empty() {
                    "inout"
                } else {
                    p.direction.trim()
                };
                let t = if p.type_name.trim().is_empty() {
                    "Port"
                } else {
                    p.type_name.trim()
                };
                format!("`{}` (`{} {}`)", p.name.trim(), dir, t)
            })
            .collect();
        items.join(", ")
    };

    let rejection_channel = part
        .ports
        .iter()
        .find(|p| matches!(p.direction.trim(), "out" | "inout"))
        .map(|p| format!("diagnostic port `{}`", p.name.trim()))
        .unwrap_or_else(|| "deterministic error diagnostic output".to_string());

    (attr_summary, port_summary, rejection_channel)
}

/// Appends paired positive satisfaction and negative boundary-violation BDD criteria for `ob` to `criteria`.
/// Realises: [REQ-SYNC-FORWARD-GENERATE-FEATURE-BDD]
fn push_invariant_bdd_pair(
    criteria: &mut Vec<String>,
    part_name: &str,
    ob: &FeatureConstraintObligation,
    rejection_channel: &str,
) {
    let clause = format_feature_obligation_clause(ob);
    let pos_num = criteria.len().saturating_add(1);
    criteria.push(format!(
        "- [ ] AC-{pos_num:02}: Given valid `{part_name}` inputs satisfying {clause}, When `{part_name}` evaluates invariant `{}`, Then the invariant holds and verified state is committed.",
        ob.name
    ));
    let neg_num = criteria.len().saturating_add(1);
    criteria.push(format!(
        "- [ ] AC-{neg_num:02}: Given boundary inputs that violate {clause}, When `{part_name}` evaluates invariant `{}`, Then `{part_name}` rejects the invalid transition and routes a diagnostic via {rejection_channel}.",
        ob.name
    ));
}

/// Appends positive and (when below the 10-AC ceiling) negative structural interface BDD criteria to `criteria`.
/// Realises: [REQ-SYNC-FORWARD-GENERATE-FEATURE-BDD]
fn push_interface_bdd_criteria(
    criteria: &mut Vec<String>,
    part_name: &str,
    attr_summary: &str,
    port_summary: &str,
    rejection_channel: &str,
) {
    let iface_pos_num = criteria.len().saturating_add(1);
    criteria.push(format!(
        "- [ ] AC-{iface_pos_num:02}: Given nominal typed values for {attr_summary} across {port_summary}, When `{part_name}` executes structural interface processing, Then state and output ports conform to declared schema types."
    ));
    if criteria.len() < 10 {
        let iface_neg_num = criteria.len().saturating_add(1);
        criteria.push(format!(
            "- [ ] AC-{iface_neg_num:02}: Given malformed input frames across {attr_summary} or {port_summary}, When `{part_name}` boundary validation executes, Then `{part_name}` rejects the malformed input via {rejection_channel}."
        ));
    }
}

/// Generates lossless `3..=10` positive/negative BDD Acceptance Criteria from `PartDef` obligations and interfaces.
/// Realises: [REQ-SYNC-FORWARD-GENERATE-FEATURE-BDD]
pub(crate) fn generate_feature_bdd(part: &PartDef, reqs: &[&RequirementDef]) -> String {
    let obligations = collect_feature_constraint_obligations(part, reqs);
    let (attr_summary, port_summary, rejection_channel) =
        summarize_part_interface_identifiers(part);
    let mut criteria: Vec<String> = Vec::new();

    if obligations.is_empty() {
        criteria.push(format!(
            "- [ ] AC-01: Given valid `{}` initialization inputs for {} (no additional constraints specified in schema), When `{}` commits typed domain state, Then declared state attributes hold verified values.",
            part.name, attr_summary, part.name
        ));
        criteria.push(format!(
            "- [ ] AC-02: Given malformed or out-of-domain input values for {}, When `{}` validates incoming state updates, Then `{}` rejects the invalid state update and emits a diagnostic via {}.",
            attr_summary, part.name, part.name, rejection_channel
        ));
        criteria.push(format!(
            "- [ ] AC-03: Given nominal signal arrival across {}, When `{}` processes the declared interface transaction, Then output state conforms to the structural classifier contract.",
            port_summary, part.name
        ));
        criteria.push(format!(
            "- [ ] AC-04: Given an invalid or unrecognized interface frame across {}, When `{}` boundary validation executes, Then `{}` refuses the malformed frame via {} and preserves prior verified state.",
            port_summary, part.name, part.name, rejection_channel
        ));
    } else if obligations.len() <= 4 {
        for ob in &obligations {
            push_invariant_bdd_pair(&mut criteria, &part.name, ob, &rejection_channel);
        }
        push_interface_bdd_criteria(
            &mut criteria,
            &part.name,
            &attr_summary,
            &port_summary,
            &rejection_channel,
        );
    } else {
        for ob in obligations.iter().take(3) {
            push_invariant_bdd_pair(&mut criteria, &part.name, ob, &rejection_channel);
        }

        let remaining_clauses: Vec<String> = obligations
            .iter()
            .skip(3)
            .map(format_feature_obligation_clause)
            .collect();
        let composite_list = remaining_clauses.join("; ");

        let comp_pos_num = criteria.len().saturating_add(1);
        criteria.push(format!(
            "- [ ] AC-{comp_pos_num:02}: Given valid `{}` state satisfying composite invariants ({composite_list}), When `{}` evaluates the remaining invariant conjunction, Then all declared invariants hold and verified state is committed.",
            part.name, part.name
        ));
        let comp_neg_num = criteria.len().saturating_add(1);
        criteria.push(format!(
            "- [ ] AC-{comp_neg_num:02}: Given input state that violates any invariant in ({composite_list}), When `{}` evaluates the composite guard, Then `{}` rejects the invalid state transition and routes a diagnostic via {rejection_channel}.",
            part.name, part.name
        ));
        push_interface_bdd_criteria(
            &mut criteria,
            &part.name,
            &attr_summary,
            &port_summary,
            &rejection_channel,
        );
    }

    criteria.join("\n")
}
