#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # User Story 3-Layer Definition of Done Architectural Renderers
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-US-3LAYER]
//! Projects owning `PartDef` attributes and boundary ports, `PackageDef` attribute/port
//! definitions, `ActionDef` typed `in`/`out` parameters, `StateDef` lifecycle models and
//! transition guards, `ConstraintDef`/`RequirementDef` invariants, and `InteractionDef`
//! bindings into `US-*.md` Layers 1..=3 with zero `"N/A"` placeholders.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Input `UserStoryProjectionCtx` references valid AST slices.
//! - **Postconditions**: Emits non-empty Layer 1, Layer 2, and Layer 3 Markdown bullet blocks.
//! - **Invariants**: Zero `"N/A"` placeholders are emitted across any layer.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Missing attributes, states, transitions, invariants, or ports emit deterministic structural bindings.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Pure single-threaded `O(N)` formatting over AST attributes, states, constraints, and ports.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Consumed by `story_emitter`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Verified by User Story 3-layer tests in `sync/forward/tests*.rs`.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-US-3LAYER]

use super::*;

/// Renders the Layer 1 domain state attribute bullet for a User Story.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-US-3LAYER]
fn render_story_layer1_attrs(ctx: &UserStoryProjectionCtx<'_>) -> String {
    let engine_name = ctx.engine_name;
    let pkg_name = ctx.pkg_name;

    let mut attr_items: Vec<String> = Vec::new();
    if let Some(pt) = ctx.target_part {
        for attr in &pt.attributes {
            let n = attr.name.trim();
            let t = attr.type_name.trim();
            if !n.is_empty() {
                if t.is_empty() {
                    attr_items.push(format!("`{n}`"));
                } else {
                    attr_items.push(format!("`{n}: {t}`"));
                }
            }
        }
    }
    if let Some(cp) = ctx.curr_pkg {
        for attr in &cp.attribute_defs {
            let n = attr.name.trim();
            let t = attr.type_name.trim();
            if !n.is_empty() {
                let formatted = if t.is_empty() {
                    format!("`{n}`")
                } else {
                    format!("`{n}: {t}`")
                };
                if !attr_items.contains(&formatted) {
                    attr_items.push(formatted);
                }
            }
        }
    }
    if attr_items.is_empty() {
        format!(
            "- Domain state record: `{engine_name}` structural state representation within `{pkg_name}`."
        )
    } else {
        format!(
            "- Domain state attributes on `{engine_name}`: {}.",
            attr_items.join(", ")
        )
    }
}

/// Renders the Layer 1 typed `ActionDef` parameter contract bullet for a User Story.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-US-3LAYER]
fn render_story_layer1_params(ctx: &UserStoryProjectionCtx<'_>) -> String {
    let engine_name = ctx.engine_name;
    let action_name = ctx.action_name;

    if let Some(act) = ctx
        .act_opt
        .filter(|a| !a.in_params.is_empty() || !a.out_params.is_empty())
    {
        let ins: Vec<String> = act
            .in_params
            .iter()
            .map(|p| {
                let n = p.name.trim();
                let t = p.type_name.trim();
                if t.is_empty() {
                    format!("`{n}`")
                } else {
                    format!("`{n}: {t}`")
                }
            })
            .collect();
        let outs: Vec<String> = act
            .out_params
            .iter()
            .map(|p| {
                let n = p.name.trim();
                let t = p.type_name.trim();
                if t.is_empty() {
                    format!("`{n}`")
                } else {
                    format!("`{n}: {t}`")
                }
            })
            .collect();
        let in_desc = if ins.is_empty() {
            "none".to_string()
        } else {
            ins.join(", ")
        };
        let out_desc = if outs.is_empty() {
            "void".to_string()
        } else {
            outs.join(", ")
        };
        format!(
            "- Action `{action_name}` typed data contract: input arguments ({in_desc}), output values ({out_desc})."
        )
    } else {
        format!(
            "- Action `{action_name}` invocation message envelope bound to `{engine_name}` typed domain schema."
        )
    }
}

/// Renders the Layer 1 declared lifecycle state enumeration bullet across `ctx.states`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-US-3LAYER]
fn render_story_layer1_states(ctx: &UserStoryProjectionCtx<'_>) -> String {
    let mut state_names: Vec<String> = Vec::new();
    for st in ctx.states {
        let st_name = st.name.trim();
        if !st_name.is_empty() {
            let entry = format!("`{st_name}`");
            if !state_names.contains(&entry) {
                state_names.push(entry);
            }
        }
        for tr in &st.transitions {
            if let Some((src, tgt, _)) = extract_transition_endpoints(tr) {
                if src != "[*]" {
                    let s_entry = format!("`{src}`");
                    if !state_names.contains(&s_entry) {
                        state_names.push(s_entry);
                    }
                }
                if tgt != "[*]" {
                    let t_entry = format!("`{tgt}`");
                    if !state_names.contains(&t_entry) {
                        state_names.push(t_entry);
                    }
                }
            }
        }
    }
    if state_names.is_empty() {
        format!(
            "- Declared lifecycle state model: `{}` discrete operational state enumeration.",
            ctx.engine_name
        )
    } else {
        format!(
            "- Declared lifecycle state model: {}.",
            state_names.join(", ")
        )
    }
}

/// Renders the Layer 2 state-transition management bullet from `ctx.matched_transition`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-US-3LAYER]
fn render_story_layer2_transition(ctx: &UserStoryProjectionCtx<'_>) -> String {
    let Some(m) = ctx.matched_transition else {
        return format!(
            "- State transition management: enforces deterministic state progression and rollback safety across `{}`.",
            ctx.engine_name
        );
    };
    let src_disp = if m.source == "[*]" {
        "InitialEntry"
    } else {
        m.source.as_str()
    };
    let tgt_disp = if m.target == "[*]" {
        "TerminalExit"
    } else {
        m.target.as_str()
    };
    let mut details: Vec<String> = Vec::new();
    if let Some(trig) = m
        .transition
        .trigger
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        details.push(format!("trigger `{trig}`"));
    }
    if let Some(grd) = m
        .transition
        .guard
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let clean_g = grd.trim_start_matches('[').trim_end_matches(']').trim();
        if !clean_g.is_empty() && !clean_g.contains("](") {
            details.push(format!("guard `[{clean_g}]`"));
        }
    }
    if let Some(eff) = m
        .transition
        .effect
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        details.push(format!("effect `{eff}`"));
    }
    if details.is_empty() {
        format!("- State transition management: `{src_disp} -> {tgt_disp}`.")
    } else {
        format!(
            "- State transition management: `{src_disp} -> {tgt_disp}` ({}).",
            details.join(", ")
        )
    }
}

/// Renders the Layer 2 deduplicated constraint and requirement invariant bullet for a User Story.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-US-3LAYER]
fn render_story_layer2_invariants(ctx: &UserStoryProjectionCtx<'_>) -> String {
    let mut inv_items: Vec<String> = Vec::new();
    for c in ctx.constraints {
        let lbl = format_constraint_label(c);
        if !lbl.is_empty() {
            let formatted = format!("`{lbl}`");
            if !inv_items.contains(&formatted) {
                inv_items.push(formatted);
            }
        }
    }
    for r in ctx.reqs {
        let lbl = format_requirement_label(r);
        if !lbl.is_empty() {
            let formatted = format!("`{lbl}`");
            if !inv_items.contains(&formatted) {
                inv_items.push(formatted);
            }
        }
    }
    if inv_items.is_empty() {
        format!(
            "- Invariant enforcement: preserves `{}` structural invariants without partial state mutation on rejection.",
            ctx.engine_name
        )
    } else {
        format!(
            "- Formal constraints and requirements enforced: {}.",
            inv_items.join(", ")
        )
    }
}

/// Renders the Layer 3 component port binding, interaction routing, and diagnostic fault surface bullets.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-US-3LAYER]
fn render_story_layer3_interface_binding(ctx: &UserStoryProjectionCtx<'_>) -> String {
    let engine_name = ctx.engine_name;
    let action_name = ctx.action_name;
    let story_actor = ctx.story_actor;
    let reply_msg = ctx.reply_msg;
    let interaction_name = ctx.interaction_name;

    let mut port_items: Vec<String> = Vec::new();
    let ports_slice = ctx
        .target_part
        .filter(|pt| !pt.ports.is_empty())
        .map(|pt| pt.ports.as_slice())
        .or_else(|| ctx.curr_pkg.map(|cp| cp.port_defs.as_slice()))
        .unwrap_or(&[]);
    for port in ports_slice {
        let n = port.name.trim();
        let t = port.type_name.trim();
        let d = if port.direction.trim().is_empty() { "inout" } else { port.direction.trim() };
        if !n.is_empty() {
            let formatted = if t.is_empty() {
                format!("`{n} ({d})`")
            } else {
                format!("`{n}: {t} ({d})`")
            };
            port_items.push(formatted);
        }
    }

    let layer3_ports = if port_items.is_empty() {
        format!(
            "- Component interface endpoints: binds `{engine_name}` command ingress and completion notification egress channels."
        )
    } else {
        format!(
            "- Component port bindings on `{engine_name}`: {}.",
            port_items.join(", ")
        )
    };
    let layer3_interaction = format!(
        "- Interaction binding (`{interaction_name}`): routes `{action_name}` invocation from `{story_actor}` to `{engine_name}` and returns `{reply_msg}`."
    );
    let layer3_fault_surface = format!(
        "- Diagnostic fault surface: reports deterministic rejection diagnostics to `{story_actor}` across the declared interface boundary when guard or invariant checks fail."
    );

    format!("{layer3_ports}\n{layer3_interaction}\n{layer3_fault_surface}")
}

/// Synthesizes the explicit 3-layer Definition of Done architectural binding sections for a User Story.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-US-3LAYER]
pub(crate) fn generate_user_story_three_layer_sections(ctx: &UserStoryProjectionCtx<'_>) -> String {
    let layer1_attrs = render_story_layer1_attrs(ctx);
    let layer1_params = render_story_layer1_params(ctx);
    let layer1_states = render_story_layer1_states(ctx);
    let layer2_exec = format!(
        "- Behavioral execution contract: invokes `{}` on `{}` with atomic precondition verification.",
        ctx.call_sig, ctx.engine_name
    );
    let layer2_transition = render_story_layer2_transition(ctx);
    let layer2_invariants = render_story_layer2_invariants(ctx);
    let layer3_block = render_story_layer3_interface_binding(ctx);

    format!(
        "### Layer 1: Domain State & Data Model\n\
         {layer1_attrs}\n\
         {layer1_params}\n\
         {layer1_states}\n\n\
         ### Layer 2: Logic & State Management\n\
         {layer2_exec}\n\
         {layer2_transition}\n\
         {layer2_invariants}\n\n\
         ### Layer 3: Presentation & Actuator Interface Binding\n\
         {layer3_block}"
    )
}
