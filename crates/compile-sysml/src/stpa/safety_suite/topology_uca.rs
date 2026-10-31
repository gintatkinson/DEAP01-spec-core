//! STPA losses, hazards, control structure topology, UCA matrix, and loss scenario renderers.
//!
//! Safety & Traceability: Realises: [REQ-STPA-SAFETY-SUITE-TOPOLOGY-UCA]
//! Contract: Synthesizes `01_LOSSES_HAZARDS_TOPOLOGY.md`, `02_UCA_COMBINATORIAL_MATRIX.md`, and `03_LOSS_SCENARIOS.md` deterministically from SysML v2 `PackageDef` AST declarations and expanded `UnsafeControlAction` vectors.
//! Bounds & Faults: Executes bounded linear and cycled `.get()` / `checked_rem` AST traversals with saturating arithmetic, sanitized Markdown/Mermaid cells, and zero panics or direct slice indexing.

use super::tokens::{sanitize_markdown_cell, sanitize_mermaid_id};
use super::*;
use deap_core::sysml_ast::{AttributeDef, ConnectionDef, RequirementDef};

/// Renders the STPA system losses, deduplicated system hazards, and Mermaid hierarchical safety control structure topology.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-LOSSES-HAZARDS-TOPOLOGY]
pub(super) fn render_losses_hazards_topology(
    pkg: &PackageDef,
    ucas: &[UnsafeControlAction],
) -> String {
    let all_parts = pkg.get_all_parts();
    let non_internal_parts: Vec<&PartDef> = all_parts
        .iter()
        .filter(|p| !p.name.starts_with('_'))
        .collect();
    let controllers: Vec<&PartDef> = non_internal_parts
        .iter()
        .copied()
        .filter(|part| !part.actions.is_empty())
        .collect();

    let mut out = String::new();
    out.push_str("# System Losses, Hazards & Control Structure Topology\n\n");
    out.push_str("## System Losses\n\n");
    out.push_str("| Loss ID | Description |\n");
    out.push_str("| :--- | :--- |\n");

    let controller_names: Vec<String> = if !controllers.is_empty() {
        controllers.iter().map(|c| c.name.clone()).collect()
    } else {
        let mut seen_ctrl = BTreeSet::new();
        let mut uca_controllers = Vec::new();
        for u in ucas {
            if !u.controller.is_empty() && seen_ctrl.insert(u.controller.clone()) {
                uca_controllers.push(u.controller.clone());
            }
        }
        if !uca_controllers.is_empty() {
            uca_controllers
        } else if !non_internal_parts.is_empty() {
            non_internal_parts.iter().map(|p| p.name.clone()).collect()
        } else {
            all_parts.iter().map(|p| p.name.clone()).collect()
        }
    };

    let base_pkg = if pkg.name.trim().is_empty() {
        "System"
    } else {
        pkg.name.trim()
    };

    if controller_names.is_empty() {
        out.push_str(&format!(
            "| L-1 | Loss of safe function of {} |\n",
            base_pkg
        ));
    } else {
        for (i, c_name) in controller_names.iter().enumerate() {
            out.push_str(&format!(
                "| L-{} | Loss of safe function of {} |\n",
                i.saturating_add(1),
                c_name
            ));
        }
    }

    let default_ctrl = format!("{}_Controller", base_pkg);
    let default_act = format!("{}_Actuator", base_pkg);
    let default_proc = format!("{}_Process", base_pkg);
    let default_sens = format!("{}_Sensors", base_pkg);

    let primary_controller = controller_names.first().cloned().unwrap_or(default_ctrl);

    let actuator_name = non_internal_parts
        .get(1)
        .map(|p| p.name.as_str())
        .unwrap_or(default_act.as_str());
    let process_name = non_internal_parts
        .get(2)
        .map(|p| p.name.as_str())
        .unwrap_or(default_proc.as_str());
    let sensor_name = non_internal_parts
        .get(3)
        .map(|p| p.name.as_str())
        .unwrap_or(default_sens.as_str());

    out.push_str("\n## System Hazards\n\n");
    out.push_str("| Hazard ID | Associated Control Action | Controller |\n");
    out.push_str("| :--- | :--- | :--- |\n");
    let mut seen_hazards = BTreeSet::new();
    for uca in ucas {
        if !uca.hazard.is_empty() && seen_hazards.insert(uca.hazard.clone()) {
            out.push_str(&format!(
                "| {} | {} | {} |\n",
                uca.hazard, uca.control_action, uca.controller
            ));
        }
    }
    if seen_hazards.is_empty() {
        out.push_str(&format!(
            "| H-1 | Hazardous state transition or containment breach | {} |\n",
            primary_controller
        ));
    }

    out.push_str("\n## Hierarchical Control Structure Topology\n\n");
    out.push_str("```mermaid\n");
    out.push_str("graph TD\n");
    out.push_str("    subgraph \"Hierarchical Safety Control Structure\"\n");
    out.push_str("        direction TB\n");
    out.push_str(&format!(
        "        CompilerController[\"{}<br/>(Supervisory Controller)\"]\n",
        primary_controller
    ));
    out.push_str(&format!(
        "        ActuatorEngine[\"{}<br/>(Actuator Engine)\"]\n",
        actuator_name
    ));
    out.push_str(&format!(
        "        ControlledProcess[\"{}<br/>(Execution Environment)\"]\n",
        process_name
    ));
    out.push_str(&format!(
        "        DiagnosticSensors[\"{}<br/>(Diagnostic Sensors)\"]\n\n",
        sensor_name
    ));
    out.push_str("        CompilerController -->|\"supervisory dispatch: trigger_lowering - invoke_codegen\"| ActuatorEngine\n");
    out.push_str("        ActuatorEngine -->|\"artifact synthesis: lower_ast - write_binaries\"| ControlledProcess\n");
    out.push_str("        ControlledProcess -->|\"compilation artifacts and execution traces\"| DiagnosticSensors\n");
    out.push_str("        DiagnosticSensors -->|\"sensor feedback: diagnostic_errors - verification_metrics\"| CompilerController\n");
    for conn in pkg.get_all_connections() {
        if let (Some(src), Some(tgt)) = (&conn.source_part, &conn.target_part) {
            let src_id = sanitize_mermaid_id(src);
            let tgt_id = sanitize_mermaid_id(tgt);
            let label = conn.protocol.as_deref().unwrap_or(&conn.name);
            out.push_str(&format!(
                "        {}[\"{}\"] -->|\"{}\"| {}[\"{}\"]\n",
                src_id, src, label, tgt_id, tgt
            ));
        }
    }
    out.push_str("    end\n");
    out.push_str("```\n\n");

    out
}

/// Renders the 9-column combinatorial Unsafe Control Action (UCA) Markdown matrix across the four STPA guide-word categories.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-UCA-MATRIX]
pub(super) fn render_uca_matrix(ucas: &[UnsafeControlAction]) -> String {
    let mut out = String::new();
    out.push_str("# Unsafe Control Action Combinatorial Matrix\n\n");
    out.push_str("Cartesian product of controlling part-def control actions across the\n");
    out.push_str("four canonical STPA guide-word categories.\n\n");
    out.push_str("| UCA ID | Controller | Control Action | Guide Word | Context | Hazard | Safety Constraint | Severity | Integrity Level |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for u in ucas {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            u.id,
            u.controller,
            u.control_action,
            u.guide_word,
            u.context,
            u.hazard,
            u.constraint,
            u.severity,
            u.sail
        ));
    }
    out.push('\n');
    out
}

/// Resolved single-line sanitized AST control-loop anchors cycled per UCA index for Class-1 and Class-2 STPA loss scenarios.
/// Realises: [REQ-STPA-SAFETY-SUITE-LOSS-SCENARIO-ANCHORS]
pub(super) struct LossScenarioAnchors {
    pub(super) ingress_port: String,
    pub(super) path_port: String,
    pub(super) state: String,
    pub(super) attribute: String,
    pub(super) connection: String,
    pub(super) guard: String,
}

/// Formats an AST `AttributeDef` into a single-line Markdown-safe anchor label including any non-empty default value.
/// Realises: [REQ-STPA-SAFETY-SUITE-FORMAT-ATTR-ANCHOR]
pub(super) fn format_attribute_anchor(attr: &AttributeDef) -> String {
    let clean_name = sanitize_markdown_cell(&attr.name);
    if let Some(ref val) = attr.default_value {
        let trimmed = val.trim().trim_matches('"').trim();
        if !trimmed.is_empty() {
            return format!(
                "attribute {}={}",
                clean_name,
                sanitize_markdown_cell(trimmed)
            );
        }
    }
    format!("attribute {}", clean_name)
}

/// Borrowed package-wide AST slices shared across loss-scenario anchor resolution iterations to bound parameter arity.
/// Realises: [REQ-STPA-SAFETY-SUITE-LOSS-PKG-SCOPE]
pub(super) struct LossScenarioPkgScope<'a> {
    pub(super) pkg_attrs: &'a [AttributeDef],
    pub(super) connections: &'a [ConnectionDef],
    pub(super) constraints: &'a [ConstraintDef],
    pub(super) reqs: &'a [RequirementDef],
    pub(super) base_pkg: &'a str,
}

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

    let connection = if let Some(c) = get_cycled(&incident_conns, idx) {
        let proto = c.protocol.as_deref().unwrap_or(c.name.as_str());
        format!(
            "connection {} ({})",
            sanitize_markdown_cell(&c.name),
            sanitize_markdown_cell(proto)
        )
    } else if let Some(c) = get_cycled(scope.connections, idx) {
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

    let active_constraints: &[ConstraintDef] = if !scope.constraints.is_empty() {
        scope.constraints
    } else if let Some(p) = part {
        &p.constraints
    } else {
        &[]
    };

    let guard = if let Some(c) = get_cycled(active_constraints, idx) {
        if c.expression.trim().is_empty() {
            format!("ConstraintDef {}", sanitize_markdown_cell(&c.name))
        } else {
            format!(
                "ConstraintDef {} ({})",
                sanitize_markdown_cell(&c.name),
                sanitize_markdown_cell(&c.expression)
            )
        }
    } else if let Some(r) = get_cycled(scope.reqs, idx) {
        if r.req_id.trim().is_empty() {
            format!("RequirementDef {}", sanitize_markdown_cell(&r.name))
        } else {
            format!(
                "RequirementDef {} ({})",
                sanitize_markdown_cell(&r.name),
                sanitize_markdown_cell(&r.req_id)
            )
        }
    } else {
        format!(
            "{} operational invariant",
            sanitize_markdown_cell(scope.base_pkg)
        )
    };

    LossScenarioAnchors {
        ingress_port,
        path_port,
        state,
        attribute,
        connection,
        guard,
    }
}

/// Renders AST-grounded Class-1 (Controller / Process-Model Flaw) and Class-2 (Control-Path / Feedback / Execution Degradation) STPA loss scenarios.
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

    let mut pkg_attrs = Vec::new();
    let mut all_reqs = Vec::new();
    let mut seen_reqs = BTreeSet::new();
    for sub_pkg in pkg.get_all_packages() {
        for attr in &sub_pkg.attribute_defs {
            pkg_attrs.push(attr.clone());
        }
        for req in &sub_pkg.requirement_defs {
            if seen_reqs.insert(req.name.clone()) {
                all_reqs.push(req.clone());
            }
        }
    }

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
        let ctrl = primary_part
            .map(|p| sanitize_markdown_cell(&p.name))
            .unwrap_or_else(|| sanitize_markdown_cell(base_pkg));
        let LossScenarioAnchors {
            ingress_port,
            path_port,
            state,
            attribute,
            connection,
            guard,
        } = resolve_loss_scenario_anchors(0, &ctrl, primary_part, &scope);
        out.push_str(&format!(
            "| LS-001 | UCA-BASE | Class-1 (Controller / Process-Model Flaw) | {ctrl} | BaselineControlEnvelope | {ctrl} process-model divergence in {state} across {ingress_port} with {attribute} under {guard} | Unrefreshed {attribute} or stale mode belief on {ingress_port} (Derived: STPA Class-1) | Enforce SC-001 and {guard} across {ingress_port} in {ctrl} |\n"
        ));
        out.push_str(&format!(
            "| LS-002 | UCA-BASE | Class-2 (Control-Path / Feedback / Execution Degradation) | {ctrl} | BaselineControlEnvelope | Control-path or feedback degradation across {path_port} and {connection} violating {guard} | Transport delay, dropout, or execution stall across {connection} distorting {attribute} (Derived: STPA Class-2) | Enforce SC-001 and {guard} via closed-loop integrity verification on {path_port} |\n"
        ));
        out.push('\n');
        return out;
    }

    for (i, u) in ucas.iter().enumerate() {
        let part = all_parts
            .iter()
            .find(|p| p.name == u.controller)
            .or_else(|| get_cycled(&non_internal_parts, i).copied())
            .or_else(|| get_cycled(&all_parts, i));

        let anchors = resolve_loss_scenario_anchors(i, &u.controller, part, &scope);

        let ctrl = sanitize_markdown_cell(&u.controller);
        let act = sanitize_markdown_cell(&u.control_action);
        let uca_id = sanitize_markdown_cell(&u.id);
        let sc = sanitize_markdown_cell(&u.constraint);

        let (c1_scen, c1_cause, c1_mit, c2_scen, c2_cause, c2_mit) = match u.guide_word.as_str() {
            "Not providing" => (
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
            ),
            "Providing" => (
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
            ),
            "Too early / Too late / Out of order" => (
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
            ),
            _ => (
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
            ),
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
    out
}
