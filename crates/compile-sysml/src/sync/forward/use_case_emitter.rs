#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # Use Case Specification Markdown Synthesis
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
//! Projects each allocated `UseCaseMeta` into a canonical `docs/use-cases/UC-*.md` specification
//! with Check 13B compliant `graph TD` diagrams, actors, preconditions, triggers, main success
//! scenario steps, alternate/exception flows, postconditions, and realization matrices.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: `index.uc_metas` contains allocated Use Case metadata entries.
//! - **Postconditions**: Populates `generated_files` with `use-cases/UC-XX-*.md` entries.
//! - **Invariants**: Every Use Case links to its subject Feature and (when present) paired User Story.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Use Cases lacking explicit alternate flows or postconditions synthesize deterministic fallback clauses.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Single-threaded `O(U * S)` iteration over allocated Use Cases and User Stories.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Consumes `use_case_renderer` and `use_case_diagrams`; invoked by `forward::sync_forward_to_map`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Verified by Use Case emission tests in `sync/forward/tests*.rs`.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-USE-CASE-EMITTER]

use super::*;

/// Resolves `(alternate_flows, postconditions)` Markdown blocks for `ctx`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
fn resolve_use_case_alternates_and_postconditions(ctx: &UseCaseClauseCtx<'_>) -> (String, String) {
    let subject = &ctx.uc_m.engine_name;
    let primary_actor = ctx.primary_actor;
    let clean_display = ctx.clean_display;

    let alternate_flows = if !ctx.parsed_doc.alternate_flows.is_empty() {
        ctx.parsed_doc
            .alternate_flows
            .iter()
            .map(|f| {
                if f.starts_with('-') {
                    f.clone()
                } else {
                    format!("- {}", f)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else if let Some(obj) = ctx.objective_opt {
        format!(
            "- Precondition guard failure in `{subject}`:\n\
             \x20 - `{subject}` rejects the `{clean_display}` request without partial state mutation and reports diagnostic status to `{primary_actor}`.\n\
             - Invariant violation while enforcing `{obj}`:\n\
             \x20 - `{subject}` aborts `{clean_display}` execution, preserves prior verified state, and notifies `{primary_actor}`."
        )
    } else {
        format!(
            "- Precondition guard failure in `{subject}`:\n\
             \x20 - `{subject}` rejects the `{clean_display}` request without partial state mutation and reports diagnostic status to `{primary_actor}`.\n\
             - Invariant or constraint violation during `{clean_display}` execution:\n\
             \x20 - `{subject}` aborts the active operation, preserves prior verified state, and emits diagnostic notification to `{primary_actor}`."
        )
    };

    let postconditions = if !ctx.parsed_doc.postconditions.is_empty() {
        ctx.parsed_doc
            .postconditions
            .iter()
            .map(|p| format!("- {}", p))
            .collect::<Vec<_>>()
            .join("\n")
    } else if !ctx.uc_m.uc.postconditions.is_empty() {
        ctx.uc_m
            .uc
            .postconditions
            .iter()
            .map(|p| format!("- {}", p))
            .collect::<Vec<_>>()
            .join("\n")
    } else if let Some(obj) = ctx.objective_opt {
        format!(
            "- `{subject}` satisfies declared objective `{obj}` with all subsystem invariants intact.\n\
             - Verified `{clean_display}` completion status is recorded for `{primary_actor}`."
        )
    } else if !ctx.parsed_doc.alternate_flows.is_empty() {
        let diag_summary = ctx.parsed_doc.alternate_flows.join(", ");
        format!(
            "- `{subject}` preserves declared subsystem state invariants for `{clean_display}` or fails closed with `{diag_summary}`.\n\
             - Verified `{clean_display}` outcome is recorded for `{primary_actor}`."
        )
    } else {
        format!(
            "- `{subject}` completes `{clean_display}` with all declared subsystem state invariants satisfied.\n\
             - Verified `{clean_display}` outcome is recorded for `{primary_actor}`."
        )
    };

    (alternate_flows, postconditions)
}

/// Resolves `(spec_source, source_refs)` Markdown links for a Use Case based on modular vs single-file schema layout.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
fn resolve_use_case_sources(has_modular_schema: bool, subsys_dir: &str) -> (String, String) {
    let spec_source = if !has_modular_schema {
        "[schema/model.sysml](../../schema/model.sysml)".to_string()
    } else if subsys_dir == "conops" {
        "[schema/conops/activities.sysml](../../schema/conops/activities.sysml)".to_string()
    } else {
        format!(
            "[schema/subsystems/{subsys_dir}/behavior.sysml](../../schema/subsystems/{subsys_dir}/behavior.sysml)"
        )
    };

    let source_refs = if !has_modular_schema {
        r#"- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/model.sysml` pursuant to ISO/IEC/IEEE 29148."#
            .to_string()
    } else if subsys_dir == "conops" {
        r#"- ConOps Operational Activities: `schema/conops/activities.sysml`
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/conops/activities.sysml` pursuant to ISO/IEC/IEEE 29148."#
            .to_string()
    } else {
        format!(
            r#"- Subsystem Behavior Specification: `schema/subsystems/{subsys_dir}/behavior.sysml`
- ConOps Operational Activities: `schema/conops/activities.sysml`
- System Architecture Model: `schema/model.sysml`
- Normative Systems Engineering Standard: ISO/IEC/IEEE 29148:2018 §6.4.2 Concept of Operations

Operational use case realization and traceability links derive from `schema/subsystems/{subsys_dir}/behavior.sysml` pursuant to ISO/IEC/IEEE 29148."#
        )
    };

    (spec_source, source_refs)
}

/// Builds `(realization_matrix_table, spec_source, source_refs)` for `uc_m`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
fn build_use_case_realization_and_sources(
    uc_m: &UseCaseMeta<'_>,
    index: &ForwardSyncIndex<'_>,
) -> (String, String, String) {
    let feat_link_num = uc_m.engine_feat_num;
    let feat_part = &uc_m.engine_name;
    let feat_subsys = to_display_name(&uc_m.pkg_name);
    let feat_issue = format_issue_id_cell(resolve_verified_issue_id(
        index,
        &format!("features/FEAT-{:02}-{}.md", feat_link_num, feat_part),
    ));

    let matching_story = uc_m.story_s_num.and_then(|s_num| {
        index
            .story_metas
            .iter()
            .find(|s| s.s_num == s_num && s.pkg_name == uc_m.pkg_name)
    });

    let us_row = if let Some(story) = matching_story {
        let us_link_num = story.s_num;
        let us_slug = sanitize_id(&story.action_name);
        let us_display = index
            .story_display_titles
            .get(&us_link_num)
            .cloned()
            .unwrap_or_else(|| to_display_name(&story.action_name));
        let us_issue = format_issue_id_cell(resolve_verified_issue_id(
            index,
            &format!("user-stories/US-{:02}-{}.md", us_link_num, us_slug),
        ));
        format!(
            "| **{}** | User Story | [User Story {:02}: {}](../user-stories/US-{:02}-{}.md) |\n",
            us_issue, us_link_num, us_display, us_link_num, us_slug
        )
    } else {
        String::new()
    };

    let feat_row = format!(
        "| **{}** | Feature | [Feature {:02}: [{}] {}](../features/FEAT-{:02}-{}.md) |\n",
        feat_issue, feat_link_num, feat_subsys, feat_part, feat_link_num, feat_part
    );

    let realization_matrix_table = format!(
        "| Specification Item | Type | Link / Reference |\n\
| :--- | :--- | :--- |\n\
{}{}",
        us_row, feat_row
    );
    let (spec_source, source_refs) =
        resolve_use_case_sources(index.has_modular_schema, &uc_m.subsys_dir);
    (realization_matrix_table, spec_source, source_refs)
}

/// Renders a single `UC-XX-*.md` specification document and returns `(rel_path, content)`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
fn render_single_use_case_spec(
    uc_m: &UseCaseMeta<'_>,
    index: &ForwardSyncIndex<'_>,
) -> (String, String) {
    let u_num = uc_m.u_num;
    let clean_name = strip_use_case_prefix(&uc_m.uc.name);
    let clean_display = to_display_name(clean_name);
    let uc_slug = sanitize_id(clean_name);
    let rel_path = format!("use-cases/UC-{:02}-{}.md", u_num, uc_slug);
    let _uc_date = resolve_date(&index.target_out_dir, &rel_path, index.today_iso);
    let uc_issue_id = format_issue_id_cell(resolve_verified_issue_id(index, &rel_path));
    let title = format!("Use Case {:02}: {}", u_num, clean_display);

    let parsed_doc = parse_use_case_doc(uc_m.uc.doc.as_deref());
    let objective_opt = uc_m
        .uc
        .objective
        .as_deref()
        .map(|o| o.trim().trim_matches('"').trim())
        .filter(|o| !o.is_empty());

    let (primary_actor, actors_markdown) = resolve_use_case_actors(uc_m, &parsed_doc);
    let clause_ctx = UseCaseClauseCtx {
        uc_m,
        parsed_doc: &parsed_doc,
        primary_actor: &primary_actor,
        clean_display: &clean_display,
        objective_opt,
    };

    let (preconditions, trigger) = resolve_use_case_preconditions_and_trigger(&clause_ctx);
    let resolved_scenario_steps = resolve_use_case_scenario_steps(&clause_ctx);
    let main_scenario = resolved_scenario_steps.join("\n");
    let (alternate_flows, postconditions) =
        resolve_use_case_alternates_and_postconditions(&clause_ctx);
    let (realization_matrix_table, spec_source, source_refs) =
        build_use_case_realization_and_sources(uc_m, index);

    let subject = &uc_m.engine_name;
    let uc_diagram = generate_use_case_diagram(
        &primary_actor,
        subject,
        &clean_display,
        &resolved_scenario_steps,
    );

    let content = format!(
        "| Attribute | Specification Detail |\n\
| :--- | :--- |\n\
| **Issue ID** | {uc_issue_id} |\n\
| **Title** | {title} |\n\
| **Type** | use-case |\n\
| **Primary Actor** | {primary_actor} |\n\
| **Scope** | {subject} |\n\
| **Level** | User Goal |\n\
| **Generation Mode** | subagent |\n\
| **Specification Source** | {spec_source} |\n\n\
# {title}\n\n\
## UML Diagrams\n\n```mermaid\n{uc_diagram}\n```\n\n\
## 1. Actors\n{actors_markdown}\n\n\
## 2. Preconditions\n{preconditions}\n\n\
## 3. Trigger\n{trigger}\n\n\
## 4. Main Success Scenario\n{main_scenario}\n\n\
## 5. Alternate and Exception Flows\n{alternate_flows}\n\n\
## 6. Postconditions\n{postconditions}\n\n\
## 8. Realization Matrix\n{realization_matrix_table}\
## Source References\n\
Use case operational flows and lifecycle activity references:\n\
{source_refs}\n"
    );
    (rel_path, content)
}

/// Synthesizes all `use-cases/UC-XX-*.md` specifications from `index.uc_metas` into `generated_files`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
pub(crate) fn emit_use_case_specs(
    index: &ForwardSyncIndex<'_>,
    generated_files: &mut BTreeMap<String, String>,
) {
    for uc_m in &index.uc_metas {
        let (rel_path, content) = render_single_use_case_spec(uc_m, index);
        generated_files.insert(rel_path, content);
    }
}
