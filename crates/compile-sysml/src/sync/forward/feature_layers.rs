#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # Feature Markdown Validation Table, Topology, and Layer 1 Renderers
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-FEAT-VALIDATION]
//! Projects SysML v2 `PartDef` attributes, boundary ports, nested subparts, constraints, and
//! `RequirementDef` traces into CommonMark `FEAT-*.md` Section 2, Section 3, and Layer 1 sections.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Input `PartDef` and `RequirementDef` slices are valid AST nodes.
//! - **Postconditions**: Emits escaped CommonMark validation tables and topology prose.
//! - **Invariants**: `BTreeSet` deduplication ensures deterministic ordering and zero duplicate rows.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Classifiers with zero attributes, ports, constraints, or requirements emit a fallback sentence.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Pure single-threaded `O(N log N)` deduplication and formatting over part members.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Consumed by `feature_emitter`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Verified by Feature validation and layer tests in `sync/forward/tests*.rs`.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-FEAT-VALIDATION]

use super::*;

/// Borrowed `ConstraintDef` paired with an optional owning `RequirementDef` traceability label.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-SCOPED-CONSTRAINT-REF]
pub(crate) struct ScopedConstraintRef<'a> {
    pub(crate) constraint: &'a ConstraintDef,
    pub(crate) req_trace: Option<String>,
}

/// Deduplicates allocated `RequirementDef` nodes and scoped `ConstraintDef` references for `part`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-FEAT-VALIDATION]
fn collect_deduped_feature_reqs_and_constraints<'a>(
    part: &'a PartDef,
    reqs: &[&'a RequirementDef],
) -> (Vec<&'a RequirementDef>, Vec<ScopedConstraintRef<'a>>) {
    let mut all_reqs: Vec<&RequirementDef> = Vec::new();
    let mut seen_req_keys: BTreeSet<String> = BTreeSet::new();
    for req in part.requirements.iter().chain(reqs.iter().copied()) {
        let label = format_requirement_label(req);
        if !label.is_empty() && seen_req_keys.insert(label) {
            all_reqs.push(req);
        }
    }

    let mut all_constraints: Vec<ScopedConstraintRef<'_>> = Vec::new();
    let mut seen_constraints: BTreeSet<String> = BTreeSet::new();
    for c in &part.constraints {
        let cname = c.name.trim();
        if !cname.is_empty() && seen_constraints.insert(cname.to_string()) {
            all_constraints.push(ScopedConstraintRef {
                constraint: c,
                req_trace: None,
            });
        }
    }
    for req in &all_reqs {
        let req_label = format_requirement_label(req);
        let trace = if req_label.is_empty() {
            None
        } else {
            Some(req_label)
        };
        for c in &req.constraints {
            let cname = c.name.trim();
            if !cname.is_empty() && seen_constraints.insert(cname.to_string()) {
                all_constraints.push(ScopedConstraintRef {
                    constraint: c,
                    req_trace: trace.clone(),
                });
            }
        }
    }

    (all_reqs, all_constraints)
}

/// Formats a single `AttributeDef` row for the Feature Section 2 CommonMark validation table.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-FEAT-VALIDATION]
fn format_validation_attribute_row(attr: &AttributeDef) -> String {
    let a_name = escape_markdown_table_cell(attr.name.trim());
    let a_type = if attr.type_name.trim().is_empty() {
        "String".to_string()
    } else {
        escape_markdown_table_cell(attr.type_name.trim())
    };
    let default_col = match attr
        .default_value
        .as_deref()
        .map(str::trim)
        .filter(|d| !d.is_empty())
    {
        Some(dv) => {
            let probe = format!("{} {}", attr.name.trim(), dv);
            if is_provenance_safe_constraint_expr(&probe) {
                format!(
                    "Default `{}`",
                    escape_markdown_table_cell(&sanitize_prose_links(dv))
                )
            } else {
                "Declared default in JSON Schema".to_string()
            }
        }
        None => "Required (no default)".to_string(),
    };
    format!("| Attribute | `{a_name}` | `{a_type}` | {default_col} |")
}

/// Formats a single `ScopedConstraintRef` row for the Feature Section 2 CommonMark validation table.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-FEAT-VALIDATION]
fn format_validation_constraint_row(sc: &ScopedConstraintRef<'_>) -> String {
    let c = sc.constraint;
    let c_name = escape_markdown_table_cell(c.name.trim());
    let type_col = match &sc.req_trace {
        Some(rt) => format!("Invariant (`{}`)", escape_markdown_table_cell(rt)),
        None => "Invariant".to_string(),
    };
    let raw_expr = c.expression.trim();
    let expr_col = if raw_expr.is_empty() {
        "Declared structural invariant".to_string()
    } else if is_provenance_safe_constraint_expr(raw_expr) {
        format!(
            "`{}`",
            escape_markdown_table_cell(&sanitize_prose_links(raw_expr))
        )
    } else {
        "See verbatim SysML invariant block below".to_string()
    };
    format!("| Constraint | `{c_name}` | {type_col} | {expr_col} |")
}

/// Appends the verbatim ```sysml constraint fence to `lines` when `all_constraints` is non-empty.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-FEAT-VALIDATION]
fn append_sysml_constraints_fence(
    lines: &mut Vec<String>,
    all_constraints: &[ScopedConstraintRef<'_>],
) {
    if all_constraints.is_empty() {
        return;
    }
    lines.push(String::new());
    lines.push("```sysml".to_string());
    for sc in all_constraints {
        let cname = sc.constraint.name.trim();
        let cexpr = sanitize_prose_links(sc.constraint.expression.trim());
        if cexpr.is_empty() {
            lines.push(format!("constraint {cname} {{}}"));
        } else {
            lines.push(format!("constraint {cname} {{ {cexpr} }}"));
        }
    }
    lines.push("```".to_string());
}

/// Projects `PartDef` attributes, ports, constraints, and requirements into `FEAT-*.md` Section 2 (`### 2. Validation & Constraints`).
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-FEAT-VALIDATION]
pub(crate) fn render_feature_validation_and_constraints(
    part: &PartDef,
    reqs: &[&RequirementDef],
) -> String {
    let (all_reqs, all_constraints) = collect_deduped_feature_reqs_and_constraints(part, reqs);

    if part.attributes.is_empty()
        && part.ports.is_empty()
        && all_constraints.is_empty()
        && all_reqs.is_empty()
    {
        return format!(
            "Classifier `{}` declares no additional constraints specified in schema.",
            part.name
        );
    }

    let mut lines: Vec<String> = vec![
        format!(
            "Declared structural attributes, boundary ports, invariants, and allocated requirements for classifier `{}`:",
            part.name
        ),
        String::new(),
        "| AST Member Kind | Identifier | Declared Type / Direction | Default / Invariant Contract |"
            .to_string(),
        "| :--- | :--- | :--- | :--- |".to_string(),
    ];

    for attr in &part.attributes {
        lines.push(format_validation_attribute_row(attr));
    }
    for port in &part.ports {
        let p_name = escape_markdown_table_cell(port.name.trim());
        let p_dir = if port.direction.trim().is_empty() {
            "inout".to_string()
        } else {
            escape_markdown_table_cell(port.direction.trim())
        };
        let p_type = if port.type_name.trim().is_empty() {
            "Port".to_string()
        } else {
            escape_markdown_table_cell(port.type_name.trim())
        };
        lines.push(format!(
            "| Port | `{p_name}` | `{p_dir} {p_type}` | Boundary interface contract |"
        ));
    }
    for sc in &all_constraints {
        lines.push(format_validation_constraint_row(sc));
    }
    for req in &all_reqs {
        let req_id_col = escape_markdown_table_cell(&format_requirement_label(req));
        let clean_text = sanitize_prose_links(req.text.trim());
        let contract_col =
            if !clean_text.is_empty() && is_provenance_safe_constraint_expr(&clean_text) {
                escape_markdown_table_cell(&clean_text)
            } else {
                "Allocated requirement obligation".to_string()
            };
        lines.push(format!(
            "| Requirement | `{req_id_col}` | Allocated Requirement | {contract_col} |"
        ));
    }

    append_sysml_constraints_fence(&mut lines, &all_constraints);
    lines.join("\n")
}

/// Projects `PartDef` nested subparts and boundary ports into domain-neutral prose for `FEAT-*.md` Section 3.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-FEAT-ARRANGEMENT]
pub(crate) fn render_feature_logical_arrangement(part: &PartDef) -> String {
    let subparts_clause = if part.parts.is_empty() {
        "no nested subparts declared in schema".to_string()
    } else {
        let items: Vec<String> = part
            .parts
            .iter()
            .map(|sp| {
                let sp_type = sp.type_name.as_deref().map(str::trim).unwrap_or("");
                if sp_type.is_empty() {
                    format!("`{}`", sp.name.trim())
                } else {
                    format!("`{} : {}`", sp.name.trim(), sp_type)
                }
            })
            .collect();
        format!("nested subparts {}", items.join(", "))
    };

    let ports_clause = if part.ports.is_empty() {
        "no external boundary ports declared in schema".to_string()
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
                let pt = if p.type_name.trim().is_empty() {
                    "Port"
                } else {
                    p.type_name.trim()
                };
                format!("`{} {} : {}`", dir, p.name.trim(), pt)
            })
            .collect();
        format!("boundary ports {}", items.join(", "))
    };

    format!(
        "Structural composition and boundary topology for classifier `{}` organize {} paired with {}.",
        part.name, subparts_clause, ports_clause
    )
}

/// Projects `PartDef::attributes` into `FEAT-*.md` `### Layer 1: Domain State & Signal Model`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-FEAT-LAYER1]
pub(crate) fn render_feature_layer1_state_model(part: &PartDef) -> String {
    if part.attributes.is_empty() {
        return format!(
            "Classifier `{}` maintains typed lifecycle status (`nominal`, `degraded`, `fault`) with no additional value attributes declared in schema.",
            part.name
        );
    }
    let attrs: Vec<String> = part
        .attributes
        .iter()
        .map(|a| {
            let t = if a.type_name.trim().is_empty() {
                "String"
            } else {
                a.type_name.trim()
            };
            format!("`{} : {}`", a.name.trim(), t)
        })
        .collect();
    format!(
        "Classifier `{}` encapsulates typed domain state attributes {} alongside operational lifecycle status.",
        part.name,
        attrs.join(", ")
    )
}
