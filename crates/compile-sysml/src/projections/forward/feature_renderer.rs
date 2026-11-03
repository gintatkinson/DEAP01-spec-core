#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # Feature Layer 2/3 Renderers and BDD Acceptance Criteria Synthesis
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-FEATURE-BDD]
//! Projects SysML v2 `PartDef` states, ports, operations, actions, constraints, and
//! `RequirementDef` bindings into Feature Layers 2..=3 and lossless `3..=10` positive/negative
//! BDD Acceptance Criteria (`AC-01`..`AC-10`).
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Input `PartDef` and `RequirementDef` slices are valid AST nodes.
//! - **Postconditions**: Emits 3-layer architectural prose and `3..=10` BDD criteria lines.
//! - **Invariants**: When obligations exceed 4, remaining obligations are folded into a composite guard without truncation.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Classifiers with zero obligations emit 4 structural state/interface BDD criteria.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Pure single-threaded `O(N)` formatting over obligations and interface members.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Consumes `feature_schema` and re-exports `feature_diagrams` operation codecs.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Verified by Feature BDD and obligation tests in `sync/forward/tests*.rs`.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-FEATURE-BDD]

pub use super::feature_diagrams::{
    format_markdown_operation_bullet, format_mermaid_operation_signature,
    parse_markdown_operation_bullet,
};
use super::*;

/// Projects `PartDef` states, constraints, and requirements into `FEAT-*.md` `### Layer 2: Logic & Safety State Management`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-FEAT-LAYER2]
pub(crate) fn render_feature_layer2_logic_management(
    part: &PartDef,
    reqs: &[&RequirementDef],
) -> String {
    let obligations = collect_feature_constraint_obligations(part, reqs);
    let states_desc = if part.states.is_empty() {
        "stateless structural transition evaluation".to_string()
    } else {
        let items: Vec<String> = part
            .states
            .iter()
            .map(|st| {
                let transitions: Vec<String> = st
                    .transitions
                    .iter()
                    .filter_map(|tr| match (tr.source.as_deref(), tr.target.as_deref()) {
                        (Some(s), Some(t)) if !s.trim().is_empty() && !t.trim().is_empty() => {
                            Some(format!("{} -> {}", s.trim(), t.trim()))
                        }
                        _ => None,
                    })
                    .collect();
                if transitions.is_empty() {
                    format!("`{}`", st.name.trim())
                } else {
                    format!("`{}` (`{}`)", st.name.trim(), transitions.join(", "))
                }
            })
            .collect();
        format!("lifecycle state machines {}", items.join(", "))
    };

    let inv_desc = if obligations.is_empty() {
        "no additional invariant guards declared in schema".to_string()
    } else {
        let names: Vec<String> = obligations
            .iter()
            .map(|ob| format!("`{}`", ob.name))
            .collect();
        format!("invariant obligations {}", names.join(", "))
    };

    format!(
        "Classifier `{}` governs {} enforcing {}.",
        part.name, states_desc, inv_desc
    )
}

/// Projects `PartDef` ports, operations, and actions into `FEAT-*.md` `### Layer 3: Presentation & Actuator Interface Binding`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-FEAT-LAYER3]
pub(crate) fn render_feature_layer3_interface_binding(part: &PartDef) -> String {
    let mut bindings: Vec<String> = Vec::new();
    for port in &part.ports {
        let dir = if port.direction.trim().is_empty() {
            "inout"
        } else {
            port.direction.trim()
        };
        let pt = if port.type_name.trim().is_empty() {
            "Port"
        } else {
            port.type_name.trim()
        };
        bindings.push(format!("port `{} {} : {}`", dir, port.name.trim(), pt));
    }
    for op in &part.operations {
        let ret = op.return_type.as_deref().unwrap_or("void");
        bindings.push(format!("operation `{}() : {}`", op.name.trim(), ret));
    }
    for act in &part.actions {
        bindings.push(format!("action `{}`", act.name.trim()));
    }

    if bindings.is_empty() {
        format!(
            "Classifier `{}` exposes an internal structural boundary with no external ports, operations, or actions declared in schema.",
            part.name
        )
    } else {
        format!(
            "Classifier `{}` binds external and behavioral interfaces across {}.",
            part.name,
            bindings.join(", ")
        )
    }
}

/// Formats a `FeatureConstraintObligation` into a provenance-safe inline clause for Feature BDD criteria.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORMAT-FEATURE-OBLIGATION]
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
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-SUMMARIZE-PART-INTERFACES]
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
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-FEATURE-BDD]
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
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-FEATURE-BDD]
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
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-FEATURE-BDD]
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
