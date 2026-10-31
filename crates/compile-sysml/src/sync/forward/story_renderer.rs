//! User Story metadata model, constitutional User Story statement projection, and 3-layer Given-When-Then BDD Acceptance Criteria synthesis.
//!
//! Safety & Traceability: Realises: [REQ-SYNC-FORWARD-GENERATE-US-BDD]
//! Contract: Defines `StoryMeta` for package-scoped behavioral traceability, synthesizes constitutional `As a ..., I want to ..., so that ...` User Story statements, and projects `4..=10` AST-grounded 3-layer positive and negative BDD Acceptance Criteria (`AC-US-XX-01..N`).
//! Bounds & Faults: Uses `BTreeSet` invariant deduplication, provenance-safe constraint and requirement formatting, and `saturating_add` criterion indexing with zero regular expressions, direct indexing, or runtime panics.

use super::*;

/// Behavioral traceability metadata connecting a projected User Story (`US-XX-*.md`) to its owning `PartDef` Feature, caller lifeline, and `PackageDef` namespace.
///
/// Safety & Traceability: Realises: [REQ-SYNC-FORWARD-STORYMETA]
/// Contract: Preserves canonical action, target engine, caller engine, optional external lifeline `initiator_actor`, reply message, interaction name, subsystem display/directory identifiers, 1-based Feature numbers, and 1-based `s_num` allocation index across Epic, Feature, User Story, and Use Case synthesis passes.
/// Bounds & Faults: Owns bounded heap strings and scalar indices (`usize`) with deterministic `Default`/`Clone`/`Eq` value semantics and zero fallible operations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StoryMeta {
    pub action_name: String,
    pub engine_name: String,
    pub caller_engine: String,
    pub initiator_actor: Option<String>,
    pub reply_msg: String,
    pub interaction_name: String,
    pub subsys_display: String,
    pub pkg_name: String,
    pub subsys_dir: String,
    pub engine_feat_num: usize,
    pub caller_feat_num: Option<usize>,
    pub s_num: usize,
}

/// Synthesizes the constitutional `As a ..., I want to ..., so that ...` User Story statement from AST nodes.
/// Realises: [REQ-SYNC-FORWARD-GENERATE-US-STATEMENT]
pub(crate) fn generate_user_story_statement(ctx: &UserStoryProjectionCtx<'_>) -> String {
    let engine_name = ctx.engine_name;
    let action_name = ctx.action_name;
    let story_actor = ctx.story_actor;
    let call_sig = ctx.call_sig;

    let first_req_label = ctx
        .reqs
        .iter()
        .map(|r| format_requirement_label(r))
        .find(|lbl| !lbl.is_empty());
    let first_constraint_label = ctx
        .constraints
        .iter()
        .map(|c| format_constraint_label(c))
        .find(|lbl| !lbl.is_empty());

    let outcome_clause = if let Some(ref outcome) = ctx.parsed_doc.outcome {
        outcome.trim().trim_end_matches('.').to_string()
    } else if let Some(req_label) = first_req_label {
        if let Some(c_label) = first_constraint_label {
            format!(
                "`{engine_name}` satisfies formal requirement `{req_label}` while enforcing `{c_label}`"
            )
        } else {
            format!("`{engine_name}` satisfies formal requirement `{req_label}`")
        }
    } else if let Some(c_label) = first_constraint_label {
        format!("`{engine_name}` enforces declared invariant `{c_label}`")
    } else if let Some(out_p) = ctx.act_opt.and_then(|a| a.out_params.first()) {
        let p_name = out_p.name.trim();
        let p_type = out_p.type_name.trim();
        if p_type.is_empty() {
            format!("`{engine_name}` produces verified output `{p_name}`")
        } else {
            format!("`{engine_name}` produces verified output `{p_name}: {p_type}`")
        }
    } else if let Some(tgt) = ctx
        .matched_transition
        .map(|m| m.target.as_str())
        .filter(|t| *t != "[*]")
    {
        format!("`{engine_name}` transitions deterministically to declared state `{tgt}`")
    } else {
        format!(
            "`{engine_name}` completes `{action_name}` deterministically while preserving declared subsystem state invariants"
        )
    };

    format!(
        "As a `{story_actor}`, I want to invoke `{call_sig}` on `{engine_name}`, so that {outcome_clause}."
    )
}

/// Renders `AC-US-XX-01` (Layer 1 nominal data-model & precondition binding) and returns it alongside formatted `in_params` signatures.
/// Realises: [REQ-SYNC-FORWARD-GENERATE-US-BDD]
fn render_story_bdd_layer1_criterion(
    ctx: &UserStoryProjectionCtx<'_>,
    clean_src: Option<&str>,
    clean_tgt: Option<&str>,
) -> (String, Vec<String>) {
    let s_num = ctx.s_num;
    let story_actor = ctx.story_actor;
    let engine_name = ctx.engine_name;
    let action_name = ctx.action_name;

    let pre_state = match (&ctx.parsed_doc.precondition, clean_src) {
        (Some(pre), Some(src)) => {
            format!("{pre} and `{engine_name}` is in declared state `{src}`")
        }
        (Some(pre), None) => pre.clone(),
        (None, Some(src)) => format!("`{engine_name}` is in declared state `{src}`"),
        (None, None) => format!("`{engine_name}` is initialized in nominal operational state"),
    };

    let in_param_sigs: Vec<String> = ctx
        .act_opt
        .map(|a| {
            a.in_params
                .iter()
                .filter_map(|p| {
                    let n = p.name.trim();
                    let t = p.type_name.trim();
                    if n.is_empty() {
                        None
                    } else if t.is_empty() {
                        Some(format!("`{n}`"))
                    } else {
                        Some(format!("`{n}: {t}`"))
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let part_attr_sigs: Vec<String> = ctx
        .target_part
        .map(|pt| {
            pt.attributes
                .iter()
                .filter_map(|attr| {
                    let n = attr.name.trim();
                    let t = attr.type_name.trim();
                    if n.is_empty() {
                        None
                    } else if t.is_empty() {
                        Some(format!("`{n}`"))
                    } else {
                        Some(format!("`{n}: {t}`"))
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let in_data_clause = if !in_param_sigs.is_empty() {
        format!(" with typed input arguments ({})", in_param_sigs.join(", "))
    } else if !part_attr_sigs.is_empty() {
        format!(
            " with domain state attributes ({})",
            part_attr_sigs.join(", ")
        )
    } else {
        String::new()
    };

    let bind_clause = if !part_attr_sigs.is_empty() {
        format!(
            "validates and binds the request against `{engine_name}` domain state ({})",
            part_attr_sigs.join(", ")
        )
    } else {
        format!("validates and binds the typed request data model in `{engine_name}`")
    };

    let post_state_clause = if let Some(tgt) = clean_tgt {
        format!(" and transitions to `{tgt}`")
    } else if let Some(ref outcome) = ctx.parsed_doc.outcome {
        format!(" so that {}", outcome.trim().trim_end_matches('.'))
    } else {
        String::new()
    };

    let ac_line = format!(
        "- [ ] AC-US-{s_num:02}-01: Given {pre_state}{in_data_clause}, When `{story_actor}` invokes `{action_name}` on `{engine_name}`, Then `{engine_name}` {bind_clause}{post_state_clause}."
    );
    (ac_line, in_param_sigs)
}

/// Collects `1..=5` deduplicated Layer 2 transition-guard, `ConstraintDef`, and `RequirementDef` BDD criterion bodies plus all deduplicated constraint labels.
/// Realises: [REQ-SYNC-FORWARD-GENERATE-US-BDD]
fn collect_story_bdd_layer2_bodies(
    ctx: &UserStoryProjectionCtx<'_>,
    clean_src: Option<&str>,
    clean_tgt: Option<&str>,
    clean_guard: Option<&str>,
) -> (Vec<String>, Vec<String>) {
    let engine_name = ctx.engine_name;
    let action_name = ctx.action_name;

    let mut layer2_bodies: Vec<String> = Vec::new();
    if let Some(guard) = clean_guard {
        let guard_body = match (clean_src, clean_tgt) {
            (Some(src), Some(tgt)) => format!(
                "Given `{engine_name}` evaluates transition guard `[{guard}]` in state `{src}`, When guard condition `[{guard}]` holds true for `{action_name}`, Then `{engine_name}` authorizes state transition `{src} -> {tgt}` without invariant violation."
            ),
            (_, Some(tgt)) => format!(
                "Given `{engine_name}` evaluates transition guard `[{guard}]` prior to `{action_name}`, When guard condition `[{guard}]` holds true, Then `{engine_name}` authorizes transition to `{tgt}` without invariant violation."
            ),
            _ => format!(
                "Given `{engine_name}` evaluates transition guard `[{guard}]` prior to `{action_name}`, When guard condition `[{guard}]` holds true, Then `{engine_name}` authorizes state transition execution without invariant violation."
            ),
        };
        layer2_bodies.push(guard_body);
    }

    let mut seen_invariants: BTreeSet<String> = BTreeSet::new();
    let mut constraint_labels: Vec<String> = Vec::new();
    for c in ctx.constraints {
        let label = format_constraint_label(c);
        if !label.is_empty() && seen_invariants.insert(label.clone()) {
            constraint_labels.push(label.clone());
            if layer2_bodies.len() < 5 {
                layer2_bodies.push(format!(
                    "Given `{engine_name}` active state management during `{action_name}`, When `{engine_name}` evaluates formal constraint `{label}`, Then `{engine_name}` verifies constraint satisfaction before committing state updates."
                ));
            }
        }
    }

    for req in ctx.reqs {
        for c in &req.constraints {
            let label = format_constraint_label(c);
            if !label.is_empty() && seen_invariants.insert(label.clone()) {
                constraint_labels.push(label.clone());
                if layer2_bodies.len() < 5 {
                    layer2_bodies.push(format!(
                        "Given `{engine_name}` active state management during `{action_name}`, When `{engine_name}` evaluates formal constraint `{label}`, Then `{engine_name}` verifies constraint satisfaction before committing state updates."
                    ));
                }
            }
        }
        let req_label = format_requirement_label(req);
        if !req_label.is_empty()
            && seen_invariants.insert(req_label.clone())
            && layer2_bodies.len() < 5
        {
            layer2_bodies.push(format!(
                "Given `{engine_name}` requirement allocation for `{action_name}`, When `{engine_name}` verifies formal requirement `{req_label}`, Then `{engine_name}` confirms normative requirement satisfaction across execution."
            ));
        }
    }

    if layer2_bodies.is_empty() {
        if let (Some(src), Some(tgt)) = (clean_src, clean_tgt) {
            layer2_bodies.push(format!(
                "Given `{engine_name}` state machine lifecycle for `{action_name}`, When `{engine_name}` executes transition `{src} -> {tgt}`, Then `{engine_name}` preserves deterministic state conservation across execution."
            ));
        } else {
            layer2_bodies.push(format!(
                "Given `{engine_name}` behavioral contract for `{action_name}`, When `{engine_name}` executes state management logic, Then `{engine_name}` preserves deterministic state conservation without invariant violation."
            ));
        }
    }

    (layer2_bodies, constraint_labels)
}

/// Partitions `target_part` ports into formatted ingress (`in`/`inout`) and egress (`out`/`inout`) lists.
/// Realises: [REQ-SYNC-FORWARD-GENERATE-US-BDD]
fn partition_story_bdd_ports(target_part: Option<&PartDef>) -> (Vec<String>, Vec<String>) {
    let mut in_ports: Vec<String> = Vec::new();
    let mut out_ports: Vec<String> = Vec::new();
    if let Some(pt) = target_part {
        for port in &pt.ports {
            let n = port.name.trim();
            if n.is_empty() {
                continue;
            }
            let t = port.type_name.trim();
            let formatted = if t.is_empty() {
                format!("`{n}`")
            } else {
                format!("`{n}: {t}`")
            };
            let dir = port.direction.trim();
            if dir == "out" {
                out_ports.push(formatted);
            } else if dir == "in" {
                in_ports.push(formatted);
            } else {
                in_ports.push(formatted.clone());
                out_ports.push(formatted);
            }
        }
    }
    (in_ports, out_ports)
}

/// Renders the Layer 3 interface delivery BDD criterion at index `l3_idx`.
/// Realises: [REQ-SYNC-FORWARD-GENERATE-US-BDD]
fn render_story_bdd_layer3_criterion(
    ctx: &UserStoryProjectionCtx<'_>,
    l3_idx: usize,
    in_ports: &[String],
    out_ports: &[String],
) -> String {
    let s_num = ctx.s_num;
    let story_actor = ctx.story_actor;
    let engine_name = ctx.engine_name;
    let action_name = ctx.action_name;
    let interaction_name = ctx.interaction_name;
    let reply_msg = ctx.reply_msg;

    let out_param_sigs: Vec<String> = ctx
        .act_opt
        .map(|a| {
            a.out_params
                .iter()
                .filter_map(|p| {
                    let n = p.name.trim();
                    let t = p.type_name.trim();
                    if n.is_empty() {
                        None
                    } else if t.is_empty() {
                        Some(format!("`{n}`"))
                    } else {
                        Some(format!("`{n}: {t}`"))
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let given_interface = if !in_ports.is_empty() {
        format!(
            "`{engine_name}` ingress port binding ({})",
            in_ports.join(", ")
        )
    } else if !out_ports.is_empty() {
        format!(
            "`{engine_name}` declared port interface ({})",
            out_ports.join(", ")
        )
    } else {
        format!("`{engine_name}` interface binding for `{interaction_name}`")
    };

    let out_value_clause = if !out_param_sigs.is_empty() {
        format!(
            "materializes verified output ({}) and ",
            out_param_sigs.join(", ")
        )
    } else {
        String::new()
    };

    let egress_port_clause = if !out_ports.is_empty() {
        format!(" across egress port ({})", out_ports.join(", "))
    } else {
        String::new()
    };

    format!(
        "- [ ] AC-US-{s_num:02}-{l3_idx:02}: Given {given_interface}, When `{engine_name}` completes `{action_name}`, Then `{engine_name}` {out_value_clause}delivers `{reply_msg}`{egress_port_clause} to `{story_actor}`."
    )
}

/// Synthesizes `4..=10` AST-grounded 3-layer Given-When-Then BDD acceptance criteria (`AC-US-XX-01..N`) including a mandatory negative boundary / fault-containment criterion.
/// Realises: [REQ-SYNC-FORWARD-GENERATE-US-BDD]
pub(crate) fn generate_user_story_bdd(ctx: &UserStoryProjectionCtx<'_>) -> String {
    let s_num = ctx.s_num;
    let story_actor = ctx.story_actor;
    let engine_name = ctx.engine_name;
    let action_name = ctx.action_name;

    let mut criteria: Vec<String> = Vec::new();

    let clean_src = ctx
        .matched_transition
        .map(|m| m.source.as_str())
        .filter(|s| *s != "[*]");
    let clean_tgt = ctx
        .matched_transition
        .map(|m| m.target.as_str())
        .filter(|t| *t != "[*]");
    let clean_guard = ctx
        .matched_transition
        .and_then(|m| m.transition.guard.as_deref())
        .map(str::trim)
        .map(|g| g.trim_start_matches('[').trim_end_matches(']').trim())
        .filter(|g| !g.is_empty() && !g.contains("]("));

    let (ac_01, in_param_sigs) = render_story_bdd_layer1_criterion(ctx, clean_src, clean_tgt);
    criteria.push(ac_01);

    let (layer2_bodies, constraint_labels) =
        collect_story_bdd_layer2_bodies(ctx, clean_src, clean_tgt, clean_guard);
    for body in &layer2_bodies {
        let ac_idx = criteria.len().saturating_add(1);
        criteria.push(format!("- [ ] AC-US-{s_num:02}-{ac_idx:02}: {body}"));
    }

    let (in_ports, out_ports) = partition_story_bdd_ports(ctx.target_part);
    let l3_idx = criteria.len().saturating_add(1);
    criteria.push(render_story_bdd_layer3_criterion(
        ctx, l3_idx, &in_ports, &out_ports,
    ));

    let neg_trigger = if let Some(guard) = clean_guard {
        if let Some(first_c) = constraint_labels.first() {
            format!("unsatisfied transition guard `[{guard}]` or violated constraint `{first_c}`")
        } else {
            format!("unsatisfied transition guard `[{guard}]`")
        }
    } else if let Some(first_c) = constraint_labels.first() {
        format!("violated formal constraint `{first_c}`")
    } else if let Some(first_in_param) = in_param_sigs.first() {
        format!("invalid or out-of-bounds input argument {first_in_param}")
    } else if let Some(first_in_port) = in_ports.first() {
        format!("malformed ingress signal on port {first_in_port}")
    } else {
        format!(
            "unsatisfied operational precondition or malformed request frame on `{action_name}`"
        )
    };

    let neg_idx = criteria.len().saturating_add(1);
    criteria.push(format!(
        "- [ ] AC-US-{s_num:02}-{neg_idx:02}: Given {neg_trigger}, When `{story_actor}` invokes `{action_name}` on `{engine_name}`, Then `{engine_name}` rejects the request without partial state mutation, preserves prior verified state, and emits a deterministic fault diagnostic across the interface binding."
    ));

    criteria.join("\n")
}
