#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # Epic Markdown Section Prose Renderers
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-EPIC-CONTEXT]
//! Projects SysML v2 `PackageDef` documentation, structural classifiers, ports, connectors,
//! state machines, actions, interactions, requirements, and constraints into `EPIC-*.md`
//! Sections 1, 3, 4, and 5 under ISO/IEC/IEEE 15288:2023.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Input `PackageDef` and `RequirementDef` slices are valid AST nodes.
//! - **Postconditions**: Returns sanitized, deterministic Markdown prose for each Epic section.
//! - **Invariants**: `BTreeSet` deduplication guarantees stable ordering with zero duplicate clauses.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Packages lacking explicit documentation or AST members emit deterministic structural summaries.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Pure single-threaded `O(N log N)` deduplication and formatting over package members.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Consumed by `epic_emitter`; re-exports parent Epic link and count helpers from `story_allocator`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Verified by Epic prose and count tests in `sync/forward/tests*.rs`.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-EPIC-CONTEXT]

pub use super::story_allocator::{
    collect_package_story_numbers, count_total_epics_inclusive, resolve_parent_epic_link,
};
use super::*;

/// Projects `PackageDef::doc` (sanitized of broken link syntax) or an AST member inventory into `EPIC-*.md` `## 1. Context`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-EPIC-CONTEXT]
pub(crate) fn render_epic_context(p: &PackageDef, subsys_display: &str) -> String {
    if let Some(doc) = p.doc.as_deref().map(str::trim).filter(|d| !d.is_empty()) {
        return sanitize_prose_links(doc);
    }

    let mut summary_clauses: Vec<String> = Vec::new();
    if !p.part_defs.is_empty() {
        let parts: Vec<String> = p
            .part_defs
            .iter()
            .map(|pt| format!("`{}`", pt.name.trim()))
            .collect();
        summary_clauses.push(format!("structural classifiers ({})", parts.join(", ")));
    }
    if !p.action_defs.is_empty() {
        let acts: Vec<String> = p
            .action_defs
            .iter()
            .map(|a| format!("`{}`", a.name.trim()))
            .collect();
        summary_clauses.push(format!("operational actions ({})", acts.join(", ")));
    }
    if !p.use_case_defs.is_empty() {
        let ucs: Vec<String> = p
            .use_case_defs
            .iter()
            .map(|u| format!("`{}`", u.name.trim()))
            .collect();
        summary_clauses.push(format!("system use cases ({})", ucs.join(", ")));
    }

    if summary_clauses.is_empty() {
        format!(
            "Package `{}` (`{}`) defines the top-level structural namespace scope declared in the SysML v2 model.",
            p.name, subsys_display
        )
    } else {
        format!(
            "Package `{}` (`{}`) defines {} within the SysML v2 model.",
            p.name,
            subsys_display,
            summary_clauses.join(", ")
        )
    }
}

/// Formats a `PartDef` classifier and its declared `PortDef` allocations for `EPIC-*.md` `## 3. Architecture`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-EPIC-ARCHITECTURE]
fn format_epic_part_classifier(pt: &PartDef) -> String {
    if pt.ports.is_empty() {
        format!("`{}`", pt.name.trim())
    } else {
        let port_list: Vec<String> = pt
            .ports
            .iter()
            .map(|port| {
                let dir = if port.direction.trim().is_empty() {
                    "inout"
                } else {
                    port.direction.trim()
                };
                let ptype = if port.type_name.trim().is_empty() {
                    "Port"
                } else {
                    port.type_name.trim()
                };
                format!("`{} {} : {}`", dir, port.name.trim(), ptype)
            })
            .collect();
        format!("`{}` (ports: {})", pt.name.trim(), port_list.join(", "))
    }
}

/// Projects `PackageDef` structural classifiers, boundary ports, and connectors into `EPIC-*.md` `## 3. Architecture`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-EPIC-ARCHITECTURE]
pub(crate) fn render_epic_architecture(p: &PackageDef) -> String {
    let mut clauses: Vec<String> = Vec::new();

    if !p.part_defs.is_empty() {
        let part_entries: Vec<String> = p
            .part_defs
            .iter()
            .map(format_epic_part_classifier)
            .collect();
        clauses.push(format!("Classifiers: {}.", part_entries.join("; ")));
    }

    if !p.port_defs.is_empty() {
        let pkg_ports: Vec<String> = p
            .port_defs
            .iter()
            .map(|port| format!("`{}`", port.name.trim()))
            .collect();
        clauses.push(format!(
            "Package port definitions: {}.",
            pkg_ports.join(", ")
        ));
    }

    if !p.connection_defs.is_empty() {
        let conns: Vec<String> = p
            .connection_defs
            .iter()
            .map(|conn| {
                let src = match &conn.source_part {
                    Some(sp) if !sp.trim().is_empty() => {
                        format!("{}.{}", sp.trim(), conn.source_port.trim())
                    }
                    _ => conn.source_port.trim().to_string(),
                };
                let dst = match &conn.target_part {
                    Some(tp) if !tp.trim().is_empty() => {
                        format!("{}.{}", tp.trim(), conn.target_port.trim())
                    }
                    _ => conn.target_port.trim().to_string(),
                };
                if conn.name.trim().is_empty() {
                    format!("`{src} -> {dst}`")
                } else {
                    format!("`{}` (`{src} -> {dst}`)", conn.name.trim())
                }
            })
            .collect();
        clauses.push(format!("Structural connectors: {}.", conns.join(", ")));
    }

    if clauses.is_empty() {
        format!(
            "Package `{}` serves as a structural namespace container with no direct `PartDef`, `PortDef`, or `ConnectionDef` declarations.",
            p.name
        )
    } else {
        clauses.join(" ")
    }
}

/// Collects deduplicated `StateDef` lifecycle summaries across package-level and `PartDef`-scoped state machines.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-EPIC-OPERATIONS]
fn collect_epic_state_items(p: &PackageDef) -> Vec<String> {
    let mut state_items: Vec<String> = Vec::new();
    let mut seen_states: BTreeSet<String> = BTreeSet::new();
    for st in p
        .state_defs
        .iter()
        .chain(p.part_defs.iter().flat_map(|pt| pt.states.iter()))
    {
        let st_name = st.name.trim();
        if st_name.is_empty() || !seen_states.insert(st_name.to_string()) {
            continue;
        }
        let transitions: Vec<String> = st
            .transitions
            .iter()
            .filter_map(|tr| match (tr.source.as_deref(), tr.target.as_deref()) {
                (Some(s), Some(t)) if !s.trim().is_empty() && !t.trim().is_empty() => {
                    match tr
                        .trigger
                        .as_deref()
                        .map(str::trim)
                        .filter(|g| !g.is_empty())
                    {
                        Some(trig) => Some(format!("{} -> {} on {}", s.trim(), t.trim(), trig)),
                        None => Some(format!("{} -> {}", s.trim(), t.trim())),
                    }
                }
                _ => tr
                    .name
                    .as_deref()
                    .map(str::trim)
                    .filter(|n| !n.is_empty())
                    .map(str::to_string),
            })
            .collect();
        if transitions.is_empty() {
            state_items.push(format!("`{st_name}`"));
        } else {
            state_items.push(format!("`{st_name}` ({})", transitions.join("; ")));
        }
    }
    state_items
}

/// Projects `PackageDef` and `PartDef` state machines, actions, and interactions into `EPIC-*.md` `## 4. Operational Considerations`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-EPIC-OPERATIONS]
pub(crate) fn render_epic_operational_considerations(p: &PackageDef) -> String {
    let state_items = collect_epic_state_items(p);

    let mut action_items: Vec<String> = Vec::new();
    let mut seen_actions: BTreeSet<String> = BTreeSet::new();
    for act in p
        .action_defs
        .iter()
        .chain(p.part_defs.iter().flat_map(|pt| pt.actions.iter()))
    {
        let act_name = act.name.trim();
        if !act_name.is_empty() && seen_actions.insert(act_name.to_string()) {
            action_items.push(format!("`{act_name}`"));
        }
    }

    let inter_items: Vec<String> = p
        .interaction_defs
        .iter()
        .filter(|i| !i.name.trim().is_empty())
        .map(|i| format!("`{}`", i.name.trim()))
        .collect();

    let mut clauses: Vec<String> = Vec::new();
    if !state_items.is_empty() {
        clauses.push(format!(
            "Lifecycle state machines: {}.",
            state_items.join(", ")
        ));
    }
    if !action_items.is_empty() {
        clauses.push(format!("Operational actions: {}.", action_items.join(", ")));
    }
    if !inter_items.is_empty() {
        clauses.push(format!(
            "Interaction sequences: {}.",
            inter_items.join(", ")
        ));
    }

    if clauses.is_empty() {
        format!(
            "Package `{}` declares structural definitions with no direct `StateDef`, `ActionDef`, or `InteractionDef` behavioral nodes.",
            p.name
        )
    } else {
        clauses.join(" ")
    }
}

/// Projects `PackageDef` and `PartDef` requirements and constraints into `EPIC-*.md` `## 5. Security & Governance`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RENDER-EPIC-GOVERNANCE]
pub(crate) fn render_epic_security_and_governance(
    p: &PackageDef,
    all_requirements: &[RequirementDef],
) -> String {
    let part_names: BTreeSet<&str> = p.part_defs.iter().map(|pt| pt.name.as_str()).collect();

    let mut req_labels: Vec<String> = Vec::new();
    let mut seen_reqs: BTreeSet<String> = BTreeSet::new();
    let mut matched_reqs: Vec<&RequirementDef> = Vec::new();

    for req in p
        .requirement_defs
        .iter()
        .chain(p.part_defs.iter().flat_map(|pt| pt.requirements.iter()))
        .chain(all_requirements.iter().filter(|r| {
            r.satisfied_by
                .iter()
                .any(|s| part_names.contains(s.as_str()))
        }))
    {
        let label = format_requirement_label(req);
        if !label.is_empty() && seen_reqs.insert(label.clone()) {
            req_labels.push(format!("`{label}`"));
            matched_reqs.push(req);
        }
    }

    let mut constraint_labels: Vec<String> = Vec::new();
    let mut seen_constraints: BTreeSet<String> = BTreeSet::new();
    for c in p
        .constraint_defs
        .iter()
        .chain(p.part_defs.iter().flat_map(|pt| pt.constraints.iter()))
        .chain(matched_reqs.iter().flat_map(|r| r.constraints.iter()))
    {
        let cname = c.name.trim();
        if cname.is_empty() || !seen_constraints.insert(cname.to_string()) {
            continue;
        }
        let formatted = format_constraint_label(c);
        if !formatted.is_empty() {
            constraint_labels.push(format!("`{}`", sanitize_prose_links(&formatted)));
        }
    }

    let mut clauses: Vec<String> = Vec::new();
    if !req_labels.is_empty() {
        clauses.push(format!(
            "Allocated requirements: {}.",
            req_labels.join(", ")
        ));
    }
    if !constraint_labels.is_empty() {
        clauses.push(format!(
            "Enforced invariant constraints: {}.",
            constraint_labels.join(", ")
        ));
    }

    if clauses.is_empty() {
        format!(
            "Package `{}` declares no package-scoped `requirement` or `constraint` governance invariants in schema.",
            p.name
        )
    } else {
        clauses.join(" ")
    }
}
