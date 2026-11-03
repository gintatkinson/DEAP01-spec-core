#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # User Story Specification Markdown Synthesis
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-EMITTER]
//! Projects each allocated `StoryMeta` into a canonical `docs/user-stories/US-*.md` specification
//! containing the behavioral statement, 3-Layer Definition of Done sections, Mermaid
//! `sequenceDiagram`, BDD acceptance criteria, required Feature links, and normative source references.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: `index.story_metas` and `index.story_ast_contexts` are aligned 1:1.
//! - **Postconditions**: Populates `generated_files` with `user-stories/US-XX-*.md` entries.
//! - **Invariants**: Every User Story emits a valid Mermaid `sequenceDiagram` with `autonumber`.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Stories without an explicit actor default deterministically to `"Operator"`.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Single-threaded `O(S * R)` iteration over allocated stories and requirements.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Consumes `story_resolver`, `story_layers`, and `story_renderer`; invoked by `forward::sync_forward_to_map`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Verified by User Story emission tests in `sync/forward/tests*.rs`.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-EMITTER]

use super::*;

/// Resolves the sanitized User Story actor display token and Mermaid `sequenceDiagram` block.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-EMITTER]
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

/// Synthesizes the `(user_story_statement, three_layer_sections, bdd_criteria, seq_diagram)` body sections for `s_meta`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-EMITTER]
fn render_user_story_body_sections(
    s_meta: &StoryMeta,
    global_uc_actor: Option<&str>,
    index: &ForwardSyncIndex<'_>,
) -> (String, String, String, String) {
    let handles = resolve_story_ast_handles(s_meta, index);
    let parsed_doc = merge_story_docs(
        parse_story_doc(handles.act_opt.and_then(|a| a.doc.as_deref())),
        parse_story_doc(handles.inter_opt.and_then(|i| i.doc.as_deref())),
    );
    let (story_actor, seq_diagram) = resolve_story_actors_and_sequence(
        s_meta,
        &parsed_doc,
        handles.curr_pkg,
        global_uc_actor,
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
        s_num: s_meta.s_num,
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
    (
        user_story_statement,
        three_layer_sections,
        bdd_criteria,
        seq_diagram,
    )
}

/// Renders a single `US-XX-*.md` specification document and returns `(rel_path, content)`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-EMITTER]
fn render_single_user_story_spec(
    root_pkg_name: &str,
    s_meta: &StoryMeta,
    global_uc_actor: Option<&str>,
    index: &ForwardSyncIndex<'_>,
) -> (String, String) {
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

    let (user_story_statement, three_layer_sections, bdd_criteria, seq_diagram) =
        render_user_story_body_sections(s_meta, global_uc_actor, index);
    let links = build_story_links_and_sources(root_pkg_name, s_meta, index);

    let content = format!(
        "| Attribute | Specification Detail |\n\
| :--- | :--- |\n\
| **Issue ID** | {us_issue_id} |\n\
| **Title** | {title} |\n\
| **Type** | user-story |\n\
| **Parent Epic** | {parent_epic_link} |\n\
| **SysML Interaction** | `{interaction_name}` |\n\
| **SysML Test Case** | `Test_{action_name}` |\n\
| **Generation Mode** | subagent |\n\
| **Specification Source** | {spec_source} |\n\n\
# {title}\n\n\
## User Story\n\n{user_story_statement}\n\n\
## Architectural Realization (3-Layer Definition of Done)\n\n{three_layer_sections}\n\n\
## UML Sequence Diagram\n\n```mermaid\n{seq_diagram}\n```\n\n\
## Acceptance Criteria (BDD)\n{bdd_criteria}\n\n\
## Required Features\n{req_features}\n\n\
## Source References\n\
Operational concept definitions and system sequence interaction models:\n\
{source_refs}\n",
        action_name = s_meta.action_name,
        interaction_name = s_meta.interaction_name,
        parent_epic_link = links.parent_epic_link,
        spec_source = links.spec_source,
        req_features = links.req_features,
        source_refs = links.source_refs,
    );
    (rel_path, content)
}

/// Synthesizes all `user-stories/US-XX-*.md` specifications from `index.story_metas` into `generated_files`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-EMITTER]
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
        let (rel_path, content) =
            render_single_user_story_spec(&pkg.name, s_meta, global_uc_actor.as_deref(), index);
        generated_files.insert(rel_path, content);
    }
}
