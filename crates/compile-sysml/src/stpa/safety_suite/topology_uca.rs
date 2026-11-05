#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! STPA losses, hazards, control structure topology, UCA matrix, and loss-scenario anchor helpers.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! safety transpilation objectives for module `crates/compile-sysml/src/stpa/safety_suite/topology_uca.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Synthesizes `01_LOSSES_HAZARDS_TOPOLOGY.md` and `02_UCA_COMBINATORIAL_MATRIX.md` deterministically
//!   from SysML v2 `PackageDef` AST declarations and expanded `UnsafeControlAction` vectors.
//! - Provides shared AST anchor formatting and guard resolution helpers for `loss_scenarios.rs`.
//!
//! ## 3. Preconditions
//! - Caller supplies a parsed `PackageDef` and slice of `UnsafeControlAction` rows.
//!
//! ## 4. Postconditions
//! - Emits well-formed Markdown tables and Mermaid `graph TD` hierarchical control structure diagrams.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(P + U + C) over AST parts P, UCAs U, and connections C.
//! - Space Complexity: O(P + U + C) bounded output string buffer.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Degrades gracefully on schemas with zero controllers, hazards, or connections.
//! - State: Pure functional string synthesis.
//! - Action: Uses `.get()`, `.first()`, and `.saturating_add(1)` with zero panics or direct slice indexing.
//! - Resource: Bounded heap allocations proportional to rendered document length.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-SAFETY-SUITE-TOPOLOGY-UCA] | SysML v2: PackageDef(topology_uca)

pub(super) use super::loss_scenarios::*;
use super::tokens::{sanitize_markdown_cell, sanitize_mermaid_id};
use super::*;
use deap_core::sysml_ast::{AttributeDef, ConnectionDef, RequirementDef};

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

/// Borrowed package-wide AST slices shared across loss-scenario anchor resolution iterations to bound parameter arity.
/// Realises: [REQ-STPA-SAFETY-SUITE-LOSS-PKG-SCOPE]
pub(super) struct LossScenarioPkgScope<'a> {
    pub(super) pkg_attrs: &'a [AttributeDef],
    pub(super) connections: &'a [ConnectionDef],
    pub(super) constraints: &'a [ConstraintDef],
    pub(super) reqs: &'a [RequirementDef],
    pub(super) base_pkg: &'a str,
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

/// Resolves the cycled constraint or requirement guard anchor for `idx` across part and package scopes.
/// Realises: [REQ-STPA-SAFETY-SUITE-RESOLVE-LOSS-ANCHORS]
pub(super) fn resolve_guard_anchor(
    part: Option<&PartDef>,
    scope: &LossScenarioPkgScope<'_>,
    idx: usize,
) -> String {
    let active_constraints: &[ConstraintDef] = if !scope.constraints.is_empty() {
        scope.constraints
    } else if let Some(p) = part {
        &p.constraints
    } else {
        &[]
    };

    if let Some(c) = get_cycled(active_constraints, idx) {
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
    }
}

/// Harvests package-scoped attributes and deduplicated requirements across `pkg.get_all_packages()`.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-LOSS-SCENARIOS]
pub(super) fn collect_pkg_attrs_and_reqs(
    pkg: &PackageDef,
) -> (Vec<AttributeDef>, Vec<RequirementDef>) {
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
    (pkg_attrs, all_reqs)
}

/// Resolves the ordered list of controller names from inline action parts, UCA rows, or package parts.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-LOSSES-HAZARDS-TOPOLOGY]
fn resolve_controller_names(
    all_parts: &[PartDef],
    non_internal_parts: &[&PartDef],
    ucas: &[UnsafeControlAction],
) -> Vec<String> {
    let controllers: Vec<&PartDef> = non_internal_parts
        .iter()
        .copied()
        .filter(|part| !part.actions.is_empty())
        .collect();
    if !controllers.is_empty() {
        return controllers.iter().map(|c| c.name.clone()).collect();
    }

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
}

/// Appends the Mermaid `Hierarchical Safety Control Structure` diagram block onto `out`.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-LOSSES-HAZARDS-TOPOLOGY]
fn render_topology_mermaid(
    pkg: &PackageDef,
    non_internal_parts: &[&PartDef],
    base_pkg: &str,
    primary_controller: &str,
    out: &mut String,
) {
    let default_act = format!("{}_Actuator", base_pkg);
    let default_proc = format!("{}_Process", base_pkg);
    let default_sens = format!("{}_Sensors", base_pkg);

    let actuator_name = non_internal_parts
        .get(1)
        .map_or(default_act.as_str(), |p| p.name.as_str());
    let process_name = non_internal_parts
        .get(2)
        .map_or(default_proc.as_str(), |p| p.name.as_str());
    let sensor_name = non_internal_parts
        .get(3)
        .map_or(default_sens.as_str(), |p| p.name.as_str());

    out.push_str("\n## Hierarchical Control Structure Topology\n\n```mermaid\ngraph TD\n");
    out.push_str("    subgraph \"Hierarchical Safety Control Structure\"\n        direction TB\n");
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
    out.push_str("    end\n```\n\n");
}

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
    let controller_names = resolve_controller_names(&all_parts, &non_internal_parts, ucas);

    let mut out = String::new();
    out.push_str("# System Losses, Hazards & Control Structure Topology\n\n");
    out.push_str("## System Losses\n\n| Loss ID | Description |\n| :--- | :--- |\n");

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
    let primary_controller = controller_names.first().cloned().unwrap_or(default_ctrl);

    out.push_str("\n## System Hazards\n\n| Hazard ID | Associated Control Action | Controller |\n| :--- | :--- | :--- |\n");
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

    render_topology_mermaid(
        pkg,
        &non_internal_parts,
        base_pkg,
        &primary_controller,
        &mut out,
    );
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
