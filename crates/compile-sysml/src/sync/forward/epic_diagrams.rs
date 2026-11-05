#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # Epic Mermaid Statechart and Class Diagram Lowering
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-EPIC-STATECHART]
//! Extracts normalized `TransitionDef` source, target, and trigger/guard/effect labels and
//! synthesizes Check 13B compliant Mermaid `stateDiagram-v2` and `classDiagram` blocks for
//! `EPIC-*.md` package specifications without phantom package classes.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Input `PackageDef`, `StateDef`, and `TransitionDef` references are valid.
//! - **Postconditions**: Emits non-empty Mermaid `stateDiagram-v2` and `classDiagram` blocks.
//! - **Invariants**: State and operation identifiers are deduplicated deterministically via `BTreeSet`.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Transitions lacking resolvable source/target endpoints return `None` without panicking.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Pure single-threaded `O(N)` traversal over package parts, states, and transitions.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Consumed by `epic_emitter` and `story_layers`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Verified by diagram tests in `sync/forward/tests_diagrams_and_stories.rs`.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-EPIC-STATECHART]

use super::*;

/// Resolves raw `(source, target)` state identifiers from `TransitionDef::source`/`target` or `first ... then ...` syntax.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-EXTRACT-TRANSITION-ENDPOINTS]
fn resolve_raw_transition_endpoints(trans: &TransitionDef) -> Option<(String, String)> {
    match (
        trans
            .source
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty()),
        trans
            .target
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty()),
    ) {
        (Some(src), Some(tgt)) => Some((src.to_string(), tgt.to_string())),
        _ => {
            let t_name = trans.name.as_deref().unwrap_or("");
            let (_, after_first) = t_name.split_once("first ")?;
            let src_candidate = after_first.split_whitespace().next().unwrap_or("");
            let (_, after_then) = after_first.split_once("then ")?;
            let tgt_candidate = after_then.split_whitespace().next().unwrap_or("");
            if src_candidate.is_empty() || tgt_candidate.is_empty() {
                None
            } else {
                Some((src_candidate.to_string(), tgt_candidate.to_string()))
            }
        }
    }
}

/// Extracts transition source, target, and optional formatted trigger/guard/effect label from a `TransitionDef`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-EXTRACT-TRANSITION-ENDPOINTS]
pub(crate) fn extract_transition_endpoints(
    trans: &TransitionDef,
) -> Option<(String, String, Option<String>)> {
    let (raw_src, raw_tgt) = resolve_raw_transition_endpoints(trans)?;
    let src = if raw_src == "Initial" || raw_src == "entry" || raw_src == "[*]" {
        "[*]".to_string()
    } else {
        raw_src
    };
    let tgt = if raw_tgt == "Terminal" || raw_tgt == "exit" || raw_tgt == "[*]" {
        "[*]".to_string()
    } else {
        raw_tgt
    };

    let mut label_parts = Vec::new();
    if let Some(trig) = trans
        .trigger
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        label_parts.push(trig.to_string());
    }
    if let Some(grd) = trans
        .guard
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        label_parts.push(format!("[{}]", grd));
    }
    if let Some(eff) = trans
        .effect
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        label_parts.push(format!("/ {}", eff));
    }
    let label = if label_parts.is_empty() {
        None
    } else {
        Some(label_parts.join(" "))
    };

    Some((src, tgt, label))
}

/// Lowers an explicit `StateDef` transition table into a Mermaid `stateDiagram-v2` block when non-empty.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-EPIC-STATECHART]
fn render_declared_epic_statechart(st_def: &StateDef) -> Option<String> {
    let mut lines = vec!["stateDiagram-v2".to_string()];
    let mut source_states = BTreeSet::new();
    let mut target_states = BTreeSet::new();

    for trans in &st_def.transitions {
        if let Some((src, tgt, label)) = extract_transition_endpoints(trans) {
            if src == "[*]" {
                lines.push(format!("    [*] --> {}", tgt));
                if tgt != "[*]" {
                    target_states.insert(tgt);
                }
            } else if tgt == "[*]" {
                lines.push(format!("    {} --> [*]", src));
                source_states.insert(src);
            } else {
                if let Some(lbl) = label {
                    lines.push(format!("    {} --> {} : {}", src, tgt, lbl));
                } else {
                    lines.push(format!("    {} --> {}", src, tgt));
                }
                source_states.insert(src);
                target_states.insert(tgt);
            }
        }
    }

    if !lines.iter().any(|l| l.ends_with("--> [*]")) {
        for tgt in &target_states {
            if !source_states.contains(tgt) {
                lines.push(format!("    {} --> [*]", tgt));
            }
        }
    }

    if lines.len() > 1 {
        Some(lines.join("\n"))
    } else {
        None
    }
}

/// Lowers `ConOps` `action_defs` or the canonical mission lifecycle fallback into a Mermaid `stateDiagram-v2` block.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-EPIC-STATECHART]
fn render_conops_statechart(p: &PackageDef) -> String {
    if !p.action_defs.is_empty() {
        let mut lines = vec![
            "stateDiagram-v2".to_string(),
            "    [*] --> Standby".to_string(),
        ];
        let mut prev = "Standby".to_string();
        for act in &p.action_defs {
            let act_id = sanitize_id(&act.name);
            lines.push(format!(
                "    {} --> {} : execute_{}",
                prev,
                act_id,
                to_snake_case(&act.name)
            ));
            prev = act_id;
        }
        lines.push(format!("    {} --> [*]", prev));
        lines.join("\n")
    } else {
        r#"stateDiagram-v2
    [*] --> Standby
    Standby --> Operational : mission_authorized
    Operational --> Verification : evaluate_mission
    Verification --> NominalComplete : mission_complete
    Verification --> SafeFallback : contingency_abort
    NominalComplete --> [*]
    SafeFallback --> [*]"#
            .to_string()
    }
}

/// Generates a Mermaid `stateDiagram-v2` lowered from `PackageDef::state_defs` or canonical package lifecycle modes.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-EPIC-STATECHART]
pub(crate) fn generate_epic_statechart(p: &PackageDef) -> String {
    if let Some(rendered) = p
        .state_defs
        .first()
        .and_then(render_declared_epic_statechart)
    {
        return rendered;
    }

    if p.name == "ConOps" {
        return render_conops_statechart(p);
    }

    let pkg_prefix = sanitize_id(&p.name);
    if !pkg_prefix.is_empty() && (!p.part_defs.is_empty() || !p.action_defs.is_empty()) {
        let primary_part = p
            .part_defs
            .first()
            .map(|pt| sanitize_id(&pt.name))
            .unwrap_or_else(|| pkg_prefix.clone());
        let primary_action = p
            .action_defs
            .first()
            .map(|act| sanitize_id(&act.name))
            .or_else(|| {
                p.part_defs
                    .iter()
                    .flat_map(|pt| pt.actions.iter())
                    .next()
                    .map(|act| sanitize_id(&act.name))
            })
            .unwrap_or_else(|| format!("Execute{}", primary_part));

        return format!(
            "stateDiagram-v2\n\
             \x20   [*] --> {pkg_prefix}Initialization\n\
             \x20   {pkg_prefix}Initialization --> {primary_part}Ready : configure_{subsys}\n\
             \x20   {primary_part}Ready --> {primary_action}Active : dispatch_{subsys}\n\
             \x20   {primary_action}Active --> {pkg_prefix}Verification : validate_invariants\n\
             \x20   {pkg_prefix}Verification --> {pkg_prefix}NominalComplete : invariants_satisfied\n\
             \x20   {pkg_prefix}Verification --> {pkg_prefix}SafeFallback : constraint_violation\n\
             \x20   {pkg_prefix}NominalComplete --> [*]\n\
             \x20   {pkg_prefix}SafeFallback --> [*]",
            pkg_prefix = pkg_prefix,
            primary_part = primary_part,
            primary_action = primary_action,
            subsys = to_snake_case(&p.name),
        );
    }

    let sys_label = if pkg_prefix.is_empty() {
        "System".to_string()
    } else {
        pkg_prefix
    };
    format!(
        "stateDiagram-v2\n\
         \x20   [*] --> {sys_label}ColdStart\n\
         \x20   {sys_label}ColdStart --> {sys_label}Initialization : initialize\n\
         \x20   {sys_label}Initialization --> {sys_label}Operational : system_ready\n\
         \x20   {sys_label}Operational --> {sys_label}Verification : evaluate_state\n\
         \x20   {sys_label}Verification --> {sys_label}Operational : nominal\n\
         \x20   {sys_label}Verification --> {sys_label}DegradedRecovery : fault_detected\n\
         \x20   {sys_label}DegradedRecovery --> {sys_label}Operational : fault_cleared\n\
         \x20   {sys_label}DegradedRecovery --> {sys_label}SafeShutdown : critical_fault\n\
         \x20   {sys_label}SafeShutdown --> [*]"
    )
}

/// Generates an expanded Mermaid `classDiagram` for an Epic across its constituent `PartDef` classifiers and composition edges.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-EPIC-CLASS-DIAGRAM]
pub(crate) fn generate_epic_class_diagram(
    p: &PackageDef,
    primary_engine: &str,
    is_root: bool,
) -> String {
    let mut lines = vec!["classDiagram".to_string()];

    if is_root {
        let root_label = sanitize_id(&p.name);
        if let Some(root_part) = p.part_defs.iter().find(|part| part.name == root_label) {
            append_part_class_block(&mut lines, root_part);
            for subpart in &p.part_defs {
                if subpart.name != root_label {
                    append_part_class_block(&mut lines, subpart);
                    lines.push(format!(
                        "    {} --> {} : contains",
                        root_label, subpart.name
                    ));
                }
            }
            return lines.join("\n");
        }
    }

    let parent_label = p
        .parent_package
        .as_deref()
        .map(sanitize_id)
        .filter(|lbl| p.part_defs.iter().any(|pt| pt.name == *lbl))
        .unwrap_or_default();

    for part in &p.part_defs {
        append_part_class_block(&mut lines, part);
    }

    let has_primary = p.part_defs.iter().any(|pt| pt.name == primary_engine);

    if !parent_label.is_empty() && has_primary && parent_label != primary_engine {
        lines.push(format!(
            "    {} --> {} : contains",
            parent_label, primary_engine
        ));
    }

    for part in &p.part_defs {
        if part.name != primary_engine && has_primary {
            lines.push(format!(
                "    {} --> {} : coordinates",
                primary_engine, part.name
            ));
        }
        for subpart in &part.parts {
            let target_type = subpart.type_name.as_deref().unwrap_or(&subpart.name);
            if p.part_defs.iter().any(|pt| pt.name == target_type) {
                lines.push(format!(
                    "    {} --> {} : {}",
                    part.name, target_type, subpart.name
                ));
            }
        }
    }

    lines.join("\n")
}
