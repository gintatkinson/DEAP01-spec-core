//! User Story specification (`docs/user-stories/US-*.md`) synthesis from indexed SysML v2 `ActionDef` and `InteractionDef` behaviors.
//!
//! Safety & Traceability: Realises: [REQ-SYNC-FORWARD-STORY-EMITTER]
//! Contract: Projects each allocated `StoryMeta` into a canonical User Story Markdown specification containing the behavioral statement, 3-Layer Definition of Done sections, Mermaid `sequenceDiagram`, BDD acceptance criteria, required Feature links, and normative source references.
//! Bounds & Faults: Uses `BTreeSet` deduplication, checked slice accessors (`.get`, `.first`), and saturating index arithmetic (`saturating_sub`) with zero panics.

use super::*;

/// Resolved AST package, `ActionDef`, `InteractionDef`, and target `PartDef` handles for a single User Story.
/// Realises: [REQ-SYNC-FORWARD-STORY-EMITTER]
struct StoryAstHandles<'a> {
    curr_pkg: Option<&'a PackageDef>,
    act_opt: Option<&'a ActionDef>,
    inter_opt: Option<&'a InteractionDef>,
    target_part: Option<&'a PartDef>,
}

/// Trailing Markdown linkage blocks (`req_features`, `parent_epic_link`, `spec_source`, `source_refs`) for a User Story.
/// Realises: [REQ-SYNC-FORWARD-STORY-EMITTER]
struct StoryLinksAndSources {
    req_features: String,
    parent_epic_link: String,
    spec_source: String,
    source_refs: String,
}

/// Resolves borrowed AST package, action, interaction, and target part handles for `s_meta`.
/// Realises: [REQ-SYNC-FORWARD-STORY-EMITTER]
fn resolve_story_ast_handles<'a>(
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

/// Resolves the sanitized User Story actor display token and Mermaid `sequenceDiagram` block.
/// Realises: [REQ-SYNC-FORWARD-STORY-EMITTER]
fn resolve_story_actors_and_sequence(
    s_meta: &StoryMeta,
    parsed_doc: &ParsedStoryDoc,
    curr_pkg: Option<&PackageDef>,
    global_uc_actor: Option<&str>,
) -> (String, String) {
    let curr_pkg_uc_actor = curr_pkg.and_then(|cp| extract_first_uc_actor(&cp.use_case_defs));
    let raw_story_actor = parsed_doc
        .actor
        .as_deref()
        .or(s_meta.initiator_actor.as_deref())
        .or(curr_pkg_uc_actor.as_deref())
        .or_else(|| {
            if s_meta.caller_engine != s_meta.engine_name {
                Some(s_meta.caller_engine.as_str())
            } else {
                None
            }
        })
        .or(global_uc_actor)
        .unwrap_or("Operator");
    let trimmed_story_actor = raw_story_actor
        .trim()
        .trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | '[' | ']'))
        .trim();
    let story_actor = if trimmed_story_actor.is_empty() {
        "Operator".to_string()
    } else {
        trimmed_story_actor.to_string()
    };

    let raw_seq_actor = parsed_doc
        .actor
        .as_deref()
        .or(s_meta.initiator_actor.as_deref())
        .or(curr_pkg_uc_actor.as_deref())
        .or(global_uc_actor)
        .unwrap_or("Operator");
    let seq_actor = sanitize_id(raw_seq_actor);
    let seq_caller = sanitize_id(&s_meta.caller_engine);
    let seq_engine = sanitize_id(&s_meta.engine_name);
    let seq_action = sanitize_id(&s_meta.action_name);
    let seq_reply = sanitize_id(&s_meta.reply_msg);

    let seq_diagram = if s_meta.caller_engine == s_meta.engine_name {
        format!(
            r#"sequenceDiagram
    autonumber
    actor {seq_actor}
    participant {seq_engine}
    {seq_actor}->>{seq_engine}: {seq_action}()
    {seq_engine}-->>{seq_actor}: {seq_reply}"#
        )
    } else {
        format!(
            r#"sequenceDiagram
    autonumber
    actor {seq_actor}
    participant {seq_caller}
    participant {seq_engine}
    {seq_actor}->>{seq_caller}: {seq_action}()
    {seq_caller}->>{seq_engine}: {seq_action}()
    {seq_engine}-->>{seq_caller}: {seq_reply}
    {seq_caller}-->>{seq_actor}: StatusReport"#
        )
    };

    (story_actor, seq_diagram)
}

/// Collects deduplicated `RequirementDef` nodes governing `s_meta` across the target part, owning package, and global model.
/// Realises: [REQ-SYNC-FORWARD-STORY-EMITTER]
fn collect_story_requirements<'a>(
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
/// Realises: [REQ-SYNC-FORWARD-STORY-EMITTER]
fn collect_story_constraints<'a>(handles: &StoryAstHandles<'a>) -> Vec<&'a ConstraintDef> {
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

/// Builds the required Features checklist, Parent Epic link, Specification Source link, and Source References block for `s_meta`.
/// Realises: [REQ-SYNC-FORWARD-STORY-EMITTER]
fn build_story_links_and_sources(
    root_pkg_name: &str,
    s_meta: &StoryMeta,
    index: &ForwardSyncIndex<'_>,
) -> StoryLinksAndSources {
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
        req_features: req_features_list.join("\n"),
        parent_epic_link,
        spec_source,
        source_refs,
    }
}

/// Synthesizes all `user-stories/US-XX-*.md` specifications from `index.story_metas` into `generated_files`.
/// Realises: [REQ-SYNC-FORWARD-STORY-EMITTER]
pub(crate) fn emit_user_story_specs(
    pkg: &PackageDef,
    index: &ForwardSyncIndex<'_>,
    generated_files: &mut BTreeMap<String, String>,
) {
    let global_uc_actor = index
        .all_packages
        .iter()
        .find_map(|p| extract_first_uc_actor(&p.use_case_defs));

    for s_meta in &index.story_metas {
        let s_num = s_meta.s_num;
        let story_slug = sanitize_id(&s_meta.action_name);
        let rel_path = format!("user-stories/US-{:02}-{}.md", s_num, story_slug);
        let _us_date = resolve_date(&index.target_out_dir, &rel_path, index.today_iso);
        let us_issue_id =
            format_issue_id_cell(resolve_existing_issue_id(&index.target_out_dir, &rel_path));
        let us_display_title = index
            .story_display_titles
            .get(&s_num)
            .cloned()
            .unwrap_or_else(|| to_display_name(&s_meta.action_name));
        let title = format!("User Story {:02}: {}", s_num, us_display_title);

        let handles = resolve_story_ast_handles(s_meta, index);
        let parsed_doc = merge_story_docs(
            parse_story_doc(handles.act_opt.and_then(|a| a.doc.as_deref())),
            parse_story_doc(handles.inter_opt.and_then(|i| i.doc.as_deref())),
        );
        let (story_actor, seq_diagram) = resolve_story_actors_and_sequence(
            s_meta,
            &parsed_doc,
            handles.curr_pkg,
            global_uc_actor.as_deref(),
        );

        let story_reqs = collect_story_requirements(s_meta, &handles, &index.all_requirements);
        let story_constraints = collect_story_constraints(&handles);
        let mut pkg_states: Vec<&StateDef> = Vec::new();
        if let Some(pt) = handles.target_part {
            pkg_states.extend(pt.states.iter());
        }
        if let Some(cp) = handles.curr_pkg {
            pkg_states.extend(cp.state_defs.iter());
        }
        let matched_transition = resolve_story_transition(&s_meta.action_name, &pkg_states);
        let call_sig = format_action_call_with_params(&s_meta.action_name, handles.act_opt);

        let proj_ctx = UserStoryProjectionCtx {
            s_num,
            story_actor: &story_actor,
            engine_name: &s_meta.engine_name,
            action_name: &s_meta.action_name,
            interaction_name: &s_meta.interaction_name,
            reply_msg: &s_meta.reply_msg,
            pkg_name: &s_meta.pkg_name,
            call_sig: &call_sig,
            parsed_doc: &parsed_doc,
            act_opt: handles.act_opt,
            curr_pkg: handles.curr_pkg,
            target_part: handles.target_part,
            states: &pkg_states,
            matched_transition: matched_transition.as_ref(),
            constraints: &story_constraints,
            reqs: &story_reqs,
        };

        let user_story_statement = generate_user_story_statement(&proj_ctx);
        let three_layer_sections = generate_user_story_three_layer_sections(&proj_ctx);
        let bdd_criteria = generate_user_story_bdd(&proj_ctx);
        let links = build_story_links_and_sources(&pkg.name, s_meta, index);

        let content = format!(
            r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | {us_issue_id} |
| **Title** | {title} |
| **Type** | user-story |
| **Parent Epic** | {parent_epic_link} |
| **SysML Interaction** | `{interaction_name}` |
| **SysML Test Case** | `Test_{action_name}` |
| **Generation Mode** | subagent |
| **Specification Source** | {spec_source} |

# {title}

## User Story

{user_story_statement}

## Architectural Realization (3-Layer Definition of Done)

{three_layer_sections}

## UML Sequence Diagram

```mermaid
{seq_diagram}
```

## Acceptance Criteria (BDD)
{bdd_criteria}

## Required Features
{req_features}

## Source References
Operational concept definitions and system sequence interaction models:
{source_refs}
"#,
            action_name = s_meta.action_name,
            interaction_name = s_meta.interaction_name,
            parent_epic_link = links.parent_epic_link,
            spec_source = links.spec_source,
            req_features = links.req_features,
            source_refs = links.source_refs,
        );

        generated_files.insert(rel_path, content);
    }
}
