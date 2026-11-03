#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # User Story Metadata, AST Handle Resolution, and Statement Projection
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORYMETA]
//! Defines `StoryMeta`, synthesizes constitutional `As a ..., I want to ..., so that ...`
//! User Story statements, and resolves borrowed AST handles, requirements, constraints, and
//! feature/source links for `US-XX-*.md` specifications.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Input `StoryMeta`, `ForwardSyncIndex`, and `UserStoryProjectionCtx` are valid.
//! - **Postconditions**: Emits AST-grounded User Story statements and resolved traceability links.
//! - **Invariants**: `BTreeSet` deduplication ensures zero duplicate requirements or constraints.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Stories without explicit doc outcomes derive outcomes from requirements, constraints, or state transitions.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Pure single-threaded `O(N log N)` deduplication and formatting.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Consumed by `story_emitter`, `story_renderer`, `epic_emitter`, and `use_case_emitter`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Verified by User Story statement and traceability tests in `sync/forward/tests*.rs`.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORYMETA]

use super::*;

/// Behavioral traceability metadata connecting a projected User Story (`US-XX-*.md`) to its owning `PartDef` Feature, caller lifeline, and `PackageDef` namespace.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORYMETA]
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
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-US-STATEMENT]
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

/// Resolved AST package, `ActionDef`, `InteractionDef`, and target `PartDef` handles for a single User Story.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-EMITTER]
pub(crate) struct StoryAstHandles<'a> {
    pub(crate) curr_pkg: Option<&'a PackageDef>,
    pub(crate) act_opt: Option<&'a ActionDef>,
    pub(crate) inter_opt: Option<&'a InteractionDef>,
    pub(crate) target_part: Option<&'a PartDef>,
}

/// Trailing Markdown linkage blocks (`req_features`, `parent_epic_link`, `spec_source`, `source_refs`) for a User Story.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-EMITTER]
pub(crate) struct StoryLinksAndSources {
    pub(crate) req_features: String,
    pub(crate) parent_epic_link: String,
    pub(crate) spec_source: String,
    pub(crate) source_refs: String,
}

/// Resolves borrowed AST package, action, interaction, and target part handles for `s_meta`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-EMITTER]
pub(crate) fn resolve_story_ast_handles<'a>(
    s_meta: &StoryMeta,
    index: &'a ForwardSyncIndex<'a>,
) -> StoryAstHandles<'a> {
    let story_ctx = index.story_ast_contexts.get(s_meta.s_num.saturating_sub(1));
    let curr_pkg = story_ctx.map(|c| c.pkg).or_else(|| {
        index
            .part_packages
            .iter()
            .copied()
            .find(|p| p.name == s_meta.pkg_name)
            .or_else(|| {
                index
                    .all_packages
                    .iter()
                    .copied()
                    .find(|p| p.name == s_meta.pkg_name)
            })
    });
    let act_opt = story_ctx.and_then(|c| c.act_opt).or_else(|| {
        curr_pkg.and_then(|cp| {
            cp.action_defs
                .iter()
                .find(|a| strip_use_case_prefix(&a.name).eq_ignore_ascii_case(&s_meta.action_name))
                .or_else(|| {
                    cp.part_defs
                        .iter()
                        .flat_map(|pt| pt.actions.iter())
                        .find(|a| {
                            strip_use_case_prefix(&a.name).eq_ignore_ascii_case(&s_meta.action_name)
                        })
                })
        })
    });
    let inter_opt = story_ctx.and_then(|c| c.inter_opt);
    let target_part = curr_pkg
        .and_then(|cp| {
            cp.part_defs
                .iter()
                .find(|pt| pt.name == s_meta.engine_name)
                .or_else(|| cp.part_defs.first())
        })
        .or_else(|| {
            index
                .part_contexts
                .iter()
                .find(|ctx| ctx.part.name == s_meta.engine_name)
                .map(|ctx| ctx.part)
        });

    StoryAstHandles {
        curr_pkg,
        act_opt,
        inter_opt,
        target_part,
    }
}

/// Collects deduplicated `RequirementDef` nodes governing `s_meta` across the target part, owning package, and global model.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-EMITTER]
pub(crate) fn collect_story_requirements<'a>(
    s_meta: &StoryMeta,
    handles: &StoryAstHandles<'a>,
    all_requirements: &'a [RequirementDef],
) -> Vec<&'a RequirementDef> {
    let mut story_reqs: Vec<&'a RequirementDef> = Vec::new();
    let mut seen_reqs: BTreeSet<(&'a str, &'a str)> = BTreeSet::new();
    if let Some(pt) = handles.target_part {
        for r in &pt.requirements {
            if seen_reqs.insert((r.req_id.as_str(), r.name.as_str())) {
                story_reqs.push(r);
            }
        }
    }
    if let Some(cp) = handles.curr_pkg {
        for r in &cp.requirement_defs {
            let satisfies_target = r
                .satisfied_by
                .iter()
                .any(|s| s == &s_meta.engine_name || s == &s_meta.action_name);
            if satisfies_target && seen_reqs.insert((r.req_id.as_str(), r.name.as_str())) {
                story_reqs.push(r);
            }
        }
        for r in &cp.requirement_defs {
            if seen_reqs.insert((r.req_id.as_str(), r.name.as_str())) {
                story_reqs.push(r);
            }
        }
    }
    for r in all_requirements {
        let satisfies_target = r
            .satisfied_by
            .iter()
            .any(|s| s == &s_meta.engine_name || s == &s_meta.action_name);
        if satisfies_target && seen_reqs.insert((r.req_id.as_str(), r.name.as_str())) {
            story_reqs.push(r);
        }
    }
    story_reqs
}

/// Collects deduplicated `ConstraintDef` invariants across `handles.target_part` and `handles.curr_pkg`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-EMITTER]
pub(crate) fn collect_story_constraints<'a>(
    handles: &StoryAstHandles<'a>,
) -> Vec<&'a ConstraintDef> {
    let mut story_constraints: Vec<&'a ConstraintDef> = Vec::new();
    let mut seen_constraints: BTreeSet<(&'a str, &'a str)> = BTreeSet::new();
    if let Some(pt) = handles.target_part {
        for c in &pt.constraints {
            if seen_constraints.insert((c.name.as_str(), c.expression.as_str())) {
                story_constraints.push(c);
            }
        }
    }
    if let Some(cp) = handles.curr_pkg {
        for c in &cp.constraint_defs {
            if seen_constraints.insert((c.name.as_str(), c.expression.as_str())) {
                story_constraints.push(c);
            }
        }
    }
    story_constraints
}

/// Builds the required Features checklist for `s_meta`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-EMITTER]
fn build_story_required_features(s_meta: &StoryMeta, index: &ForwardSyncIndex<'_>) -> String {
    let mut req_features_list = Vec::new();
    if let Some(c_num) = s_meta.caller_feat_num {
        let c_subsys = index
            .part_contexts
            .get(c_num.saturating_sub(1))
            .map(|c| c.subsys_display.as_str())
            .unwrap_or(s_meta.subsys_display.as_str());
        req_features_list.push(format!(
            "- [ ] [Feature {:02}: [{}] {}](../features/FEAT-{:02}-{}.md)",
            c_num, c_subsys, s_meta.caller_engine, c_num, s_meta.caller_engine
        ));
    }
    if Some(s_meta.engine_feat_num) != s_meta.caller_feat_num {
        let e_subsys = index
            .part_contexts
            .get(s_meta.engine_feat_num.saturating_sub(1))
            .map(|c| c.subsys_display.as_str())
            .unwrap_or(s_meta.subsys_display.as_str());
        req_features_list.push(format!(
            "- [ ] [Feature {:02}: [{}] {}](../features/FEAT-{:02}-{}.md)",
            s_meta.engine_feat_num,
            e_subsys,
            s_meta.engine_name,
            s_meta.engine_feat_num,
            s_meta.engine_name
        ));
    }
    if req_features_list.is_empty() {
        req_features_list.push(format!(
            "- [ ] [Feature {:02}: [{}] {}](../features/FEAT-{:02}-{}.md)",
            s_meta.engine_feat_num,
            s_meta.subsys_display,
            s_meta.engine_name,
            s_meta.engine_feat_num,
            s_meta.engine_name
        ));
    }
    req_features_list.join("\n")
}

/// Builds the required Features checklist, Parent Epic link, Specification Source link, and Source References block for `s_meta`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-EMITTER]
pub(crate) fn build_story_links_and_sources(
    root_pkg_name: &str,
    s_meta: &StoryMeta,
    index: &ForwardSyncIndex<'_>,
) -> StoryLinksAndSources {
    let req_features = build_story_required_features(s_meta, index);
    let parent_epic_link = if s_meta.pkg_name == root_pkg_name {
        format!("[EPIC-01-{root_pkg_name}](../epics/EPIC-01-{root_pkg_name}.md)")
    } else {
        resolve_parent_epic_link(&index.ordered_packages, &s_meta.pkg_name)
    };

    let subsys_dir = &s_meta.subsys_dir;
    let spec_source = if !index.has_modular_schema {
        "[schema/model.sysml](../../schema/model.sysml)".to_string()
    } else if subsys_dir == "conops" {
        "[schema/conops/activities.sysml](../../schema/conops/activities.sysml)".to_string()
    } else {
        format!(
            "[schema/subsystems/{subsys_dir}/behavior.sysml](../../schema/subsystems/{subsys_dir}/behavior.sysml)"
        )
    };

    let source_refs = if !index.has_modular_schema {
        r#"- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Authoritative user story flows and interaction lifelines derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148."#
            .to_string()
    } else if subsys_dir == "conops" {
        r#"- ConOps Operational Activities: `schema/conops/activities.sysml`
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Authoritative user story flows and interaction lifelines derive from `schema/conops/activities.sysml` pursuant to ISO/IEC/IEEE 29148."#
            .to_string()
    } else {
        format!(
            r#"- Subsystem Behavior Specification: `schema/subsystems/{subsys_dir}/behavior.sysml`
- ConOps Operational Activities: `schema/conops/activities.sysml`
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Authoritative user story flows and interaction lifelines derive from `schema/subsystems/{subsys_dir}/behavior.sysml` pursuant to ISO/IEC/IEEE 29148."#
        )
    };

    StoryLinksAndSources {
        req_features,
        parent_epic_link,
        spec_source,
        source_refs,
    }
}
