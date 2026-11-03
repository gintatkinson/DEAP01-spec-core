#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! STPA Class-1 and Class-2 loss scenario anchor resolution and Markdown table synthesis.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! safety transpilation objectives for module `crates/compile-sysml/src/stpa/safety_suite/loss_scenarios.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Cycles across AST ports, states, attributes, connections, and constraints/requirements at `idx`
//!   via `get_cycled` (`checked_rem` + `.get()`) to ground Class-1 and Class-2 STPA loss scenarios.
//! - Emits 2 deterministic loss scenarios (`LS-###`) per UCA row (or baseline envelope scenarios when UCAs are empty).
//!
//! ## 3. Preconditions
//! - Caller supplies a parsed `PackageDef`, `UnsafeControlAction` slice, and `ConstraintDef` slice.
//!
//! ## 4. Postconditions
//! - Returns the rendered `03_LOSS_SCENARIOS.md` document with sanitized single-line Markdown table cells.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(P + U * C) over AST parts P, UCAs U, and connections C.
//! - Space Complexity: O(P + U) bounded output string buffer.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Degrades gracefully to baseline control envelope scenarios when `ucas` is empty.
//! - State: Pure functional string synthesis.
//! - Action: Uses `get_cycled`, `.saturating_add()`, and `.saturating_mul()` with zero panics.
//! - Resource: Bounded heap allocations proportional to UCA count.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-SAFETY-SUITE-RENDER-LOSS-SCENARIOS] | SysML v2: PackageDef(loss_scenarios)

use super::tokens::sanitize_markdown_cell;
use super::topology_uca::{
    collect_pkg_attrs_and_reqs, format_attribute_anchor, resolve_guard_anchor, LossScenarioAnchors,
    LossScenarioPkgScope,
};
use super::*;
use deap_core::sysml_ast::ConnectionDef;

/// Cycles across AST ports, states, attributes, connections, and constraints/requirements at `idx` to ground STPA loss scenarios.
/// Realises: [REQ-STPA-SAFETY-SUITE-RESOLVE-LOSS-ANCHORS]
pub(super) fn resolve_loss_scenario_anchors(
    idx: usize,
    controller_name: &str,
    part: Option<&PartDef>,
    scope: &LossScenarioPkgScope<'_>,
) -> LossScenarioAnchors {
    let (ingress_port, path_port) = match part.filter(|p| !p.ports.is_empty()) {
        Some(p) => {
            let first = get_cycled(&p.ports, idx)
                .map(|port| format!("port {}", sanitize_markdown_cell(&port.name)))
                .unwrap_or_else(|| "controller input boundary".to_string());
            let second = get_cycled(&p.ports, idx.saturating_add(1))
                .map(|port| format!("port {}", sanitize_markdown_cell(&port.name)))
                .unwrap_or_else(|| "actuation output channel".to_string());
            (first, second)
        }
        None => (
            "controller input boundary".to_string(),
            "actuation output channel".to_string(),
        ),
    };

    let state = part
        .and_then(|p| get_cycled(&p.states, idx))
        .map(|st| format!("state {}", sanitize_markdown_cell(&st.name)))
        .unwrap_or_else(|| "active operational mode".to_string());

    let attribute = if let Some(a) = part.and_then(|p| get_cycled(&p.attributes, idx)) {
        format_attribute_anchor(a)
    } else if let Some(a) = get_cycled(scope.pkg_attrs, idx) {
        format_attribute_anchor(a)
    } else {
        "process-model state variable".to_string()
    };

    let incident_conns: Vec<&ConnectionDef> = scope
        .connections
        .iter()
        .filter(|c| {
            c.source_part.as_deref() == Some(controller_name)
                || c.target_part.as_deref() == Some(controller_name)
        })
        .collect();

    let connection = if let Some(c) = get_cycled(&incident_conns, idx)
        .copied()
        .or_else(|| get_cycled(scope.connections, idx))
    {
        let proto = c.protocol.as_deref().unwrap_or(c.name.as_str());
        format!(
            "connection {} ({})",
            sanitize_markdown_cell(&c.name),
            sanitize_markdown_cell(proto)
        )
    } else {
        format!(
            "{} interface {}",
            sanitize_markdown_cell(controller_name),
            path_port
        )
    };

    LossScenarioAnchors {
        ingress_port,
        path_port,
        state,
        attribute,
        connection,
        guard: resolve_guard_anchor(part, scope, idx),
    }
}

/// Synthesizes Class-1 and Class-2 scenario tuples for `"Not providing"` or `"Providing"` guide words.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-LOSS-SCENARIOS]
fn synthesize_omission_or_commission(
    is_omission: bool,
    ctrl: &str,
    act: &str,
    sc: &str,
    anchors: &LossScenarioAnchors,
) -> (String, String, String, String, String, String) {
    if is_omission {
        (
            format!(
                "{} omits {} while in {} with {} under {}",
                ctrl, act, anchors.state, anchors.attribute, anchors.guard
            ),
            format!(
                "Stale process-model belief in {} during {} due to unrefreshed {} on {} (Derived: STPA Class-1)",
                ctrl, anchors.state, anchors.attribute, anchors.ingress_port
            ),
            format!(
                "Enforce {} and {} with state-consistency check on {} before suppressing {}",
                sc, anchors.guard, anchors.ingress_port, act
            ),
            format!(
                "{} command or feedback for {} drops across {} and {} while {} requires dispatch",
                ctrl, act, anchors.path_port, anchors.connection, anchors.guard
            ),
            format!(
                "Feedback dropout or execution path stall across {} and {} masking {} threshold (Derived: STPA Class-2)",
                anchors.path_port, anchors.connection, anchors.attribute
            ),
            format!(
                "Enforce {} and {} via closed-loop delivery confirmation across {}",
                sc, anchors.guard, anchors.path_port
            ),
        )
    } else {
        (
            format!(
                "{} erroneously issues {} while in {} violating {} and {}",
                ctrl, act, anchors.state, anchors.attribute, anchors.guard
            ),
            format!(
                "Corrupted process-model transition into {} or invalid {} evaluation on {} in {} (Derived: STPA Class-1)",
                anchors.state, anchors.attribute, anchors.ingress_port, ctrl
            ),
            format!(
                "Enforce {} precondition gate on {} and {} prior to {}",
                sc, anchors.guard, anchors.state, act
            ),
            format!(
                "Spurious or corrupted {} execution propagates from {} across {} and {} against {}",
                act, ctrl, anchors.path_port, anchors.connection, anchors.guard
            ),
            format!(
                "Corrupted ingress feedback on {} or unguarded channel transit over {} distorting {} (Derived: STPA Class-2)",
                anchors.ingress_port, anchors.connection, anchors.attribute
            ),
            format!(
                "Enforce {} runtime boundary monitor for {} across {}",
                sc, anchors.guard, anchors.path_port
            ),
        )
    }
}

/// Synthesizes Class-1 and Class-2 scenario tuples for temporal sequence or actuation duration guide words.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-LOSS-SCENARIOS]
fn synthesize_temporal_or_duration(
    is_temporal: bool,
    ctrl: &str,
    act: &str,
    sc: &str,
    anchors: &LossScenarioAnchors,
) -> (String, String, String, String, String, String) {
    if is_temporal {
        (
            format!(
                "{} dispatches {} out of temporal sequence relative to {} and {} under {}",
                ctrl, act, anchors.state, anchors.attribute, anchors.guard
            ),
            format!(
                "Premature or delayed mode transition across {} and {} triggered by out-of-order input on {} (Derived: STPA Class-1)",
                anchors.state, anchors.attribute, anchors.ingress_port
            ),
            format!(
                "Enforce {} temporal ordering invariant from {} in {}",
                sc, anchors.guard, ctrl
            ),
            format!(
                "{} dispatch of {} experiences transport skew across {} and {} violating {}",
                ctrl, act, anchors.path_port, anchors.connection, anchors.guard
            ),
            format!(
                "Queueing delay, jitter, or message reordering across {} and {} exceeding {} (Derived: STPA Class-2)",
                anchors.path_port, anchors.connection, anchors.attribute
            ),
            format!(
                "Enforce {} bounded-skew interlock for {} across {}",
                sc, anchors.guard, anchors.path_port
            ),
        )
    } else {
        (
            format!(
                "{} terminates {} prematurely or latches execution in {} against {} and {}",
                ctrl, act, anchors.state, anchors.attribute, anchors.guard
            ),
            format!(
                "Miscalibrated completion predicate on {} in {} due to stale feedback on {} (Derived: STPA Class-1)",
                anchors.attribute, anchors.state, anchors.ingress_port
            ),
            format!(
                "Enforce {} duration and termination predicate bound via {} in {}",
                sc, anchors.guard, ctrl
            ),
            format!(
                "{} execution of {} stalls early or remains latched across {} and {} beyond {}",
                ctrl, act, anchors.path_port, anchors.connection, anchors.guard
            ),
            format!(
                "Lost completion acknowledgment or stuck execution channel on {} and {} drifting {} (Derived: STPA Class-2)",
                anchors.path_port, anchors.connection, anchors.attribute
            ),
            format!(
                "Enforce {} bounded-hold cutoff monitor for {} on {}",
                sc, anchors.guard, anchors.path_port
            ),
        )
    }
}

/// Appends the Class-1 and Class-2 loss-scenario table rows for `ucas` onto `out`.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-LOSS-SCENARIOS]
fn emit_uca_loss_scenario_rows(
    all_parts: &[PartDef],
    non_internal_parts: &[&PartDef],
    ucas: &[UnsafeControlAction],
    scope: &LossScenarioPkgScope<'_>,
    out: &mut String,
) {
    for (i, u) in ucas.iter().enumerate() {
        let part = all_parts
            .iter()
            .find(|p| p.name == u.controller)
            .or_else(|| get_cycled(non_internal_parts, i).copied())
            .or_else(|| get_cycled(all_parts, i));
        let anchors = resolve_loss_scenario_anchors(i, &u.controller, part, scope);
        let ctrl = sanitize_markdown_cell(&u.controller);
        let act = sanitize_markdown_cell(&u.control_action);
        let uca_id = sanitize_markdown_cell(&u.id);
        let sc = sanitize_markdown_cell(&u.constraint);

        let (c1_scen, c1_cause, c1_mit, c2_scen, c2_cause, c2_mit) = match u.guide_word.as_str() {
            "Not providing" => synthesize_omission_or_commission(true, &ctrl, &act, &sc, &anchors),
            "Providing" => synthesize_omission_or_commission(false, &ctrl, &act, &sc, &anchors),
            "Too early / Too late / Out of order" => {
                synthesize_temporal_or_duration(true, &ctrl, &act, &sc, &anchors)
            }
            _ => synthesize_temporal_or_duration(false, &ctrl, &act, &sc, &anchors),
        };

        let ls1_id = format!("LS-{:03}", i.saturating_mul(2).saturating_add(1));
        let ls2_id = format!("LS-{:03}", i.saturating_mul(2).saturating_add(2));
        out.push_str(&format!(
            "| {} | {} | Class-1 (Controller / Process-Model Flaw) | {} | {} | {} | {} | {} |\n",
            ls1_id, uca_id, ctrl, act, c1_scen, c1_cause, c1_mit
        ));
        out.push_str(&format!(
            "| {} | {} | Class-2 (Control-Path / Feedback / Execution Degradation) | {} | {} | {} | {} | {} |\n",
            ls2_id, uca_id, ctrl, act, c2_scen, c2_cause, c2_mit
        ));
    }
    out.push('\n');
}

/// Renders AST-grounded Class-1 and Class-2 STPA loss scenarios (`03_LOSS_SCENARIOS.md`).
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-LOSS-SCENARIOS]
pub(super) fn render_loss_scenarios(
    pkg: &PackageDef,
    ucas: &[UnsafeControlAction],
    constraints: &[ConstraintDef],
) -> String {
    let all_parts = pkg.get_all_parts();
    let non_internal_parts: Vec<&PartDef> = all_parts
        .iter()
        .filter(|p| !p.name.starts_with('_'))
        .collect();
    let connections = pkg.get_all_connections();
    let base_pkg = if pkg.name.trim().is_empty() {
        "System"
    } else {
        pkg.name.trim()
    };
    let (pkg_attrs, all_reqs) = collect_pkg_attrs_and_reqs(pkg);
    let scope = LossScenarioPkgScope {
        pkg_attrs: &pkg_attrs,
        connections: &connections,
        constraints,
        reqs: &all_reqs,
        base_pkg,
    };

    let mut out = String::new();
    out.push_str("# Loss Scenarios & Causal Factors\n\n");
    out.push_str("STPA control-loop loss scenario analysis synthesizing Class-1 controller process-model flaws and Class-2 control-path, feedback, and execution degradations from SysML v2 AST structure.\n\n");
    out.push_str("| Loss Scenario ID | UCA ID | Scenario Class | Controller | Control Action | Scenario | Causal Factor | Safety Constraint Mitigation |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");

    if ucas.is_empty() {
        let primary_part = non_internal_parts
            .first()
            .copied()
            .or_else(|| all_parts.first());
        let ctrl = primary_part.map_or_else(
            || sanitize_markdown_cell(base_pkg),
            |p| sanitize_markdown_cell(&p.name),
        );
        let a = resolve_loss_scenario_anchors(0, &ctrl, primary_part, &scope);
        out.push_str(&format!(
            "| LS-001 | UCA-BASE | Class-1 (Controller / Process-Model Flaw) | {ctrl} | BaselineControlEnvelope | {ctrl} process-model divergence in {} across {} with {} under {} | Unrefreshed {} or stale mode belief on {} (Derived: STPA Class-1) | Enforce SC-001 and {} across {} in {ctrl} |\n",
            a.state, a.ingress_port, a.attribute, a.guard, a.attribute, a.ingress_port, a.guard, a.ingress_port
        ));
        out.push_str(&format!(
            "| LS-002 | UCA-BASE | Class-2 (Control-Path / Feedback / Execution Degradation) | {ctrl} | BaselineControlEnvelope | Control-path or feedback degradation across {} and {} violating {} | Transport delay, dropout, or execution stall across {} distorting {} (Derived: STPA Class-2) | Enforce SC-001 and {} via closed-loop integrity verification on {} |\n\n",
            a.path_port, a.connection, a.guard, a.connection, a.attribute, a.guard, a.path_port
        ));
        return out;
    }

    emit_uca_loss_scenario_rows(&all_parts, &non_internal_parts, ucas, &scope, &mut out);
    out
}
