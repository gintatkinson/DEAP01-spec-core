//! Use Case specification (`docs/use-cases/UC-*.md`) synthesis from indexed SysML v2 `UseCaseDef` nodes.
//!
//! Safety & Traceability: Realises: [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
//! Contract: Projects each allocated `UseCaseMeta` into a canonical UML Use Case Markdown specification with Check 13B compliant `graph TD` diagrams, actors, preconditions, triggers, main success scenario steps, alternate/exception flows, postconditions, and realization matrices.
//! Bounds & Faults: Enforces checked UTF-8 slicing (`.get`, `.first`), saturating step numbering (`saturating_add`), and provenance-safe fallback prose free of `ENGINEERING_PARAMETER_TERMS` with zero panics.

use super::*;

/// Consolidated borrowed context for resolving UML Use Case behavioral sections for a single `UseCaseMeta`.
/// Realises: [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
struct UseCaseClauseCtx<'a> {
    uc_m: &'a UseCaseMeta<'a>,
    parsed_doc: &'a ParsedUseCaseDoc,
    primary_actor: &'a str,
    clean_display: &'a str,
    objective_opt: Option<&'a str>,
}

/// Resolves `(primary_actor, actors_markdown)` for `uc_m` from `parsed_doc`, AST actors, package interaction lifelines, or `"Operator"`.
/// Realises: [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
fn resolve_use_case_actors(
    uc_m: &UseCaseMeta<'_>,
    parsed_doc: &ParsedUseCaseDoc,
) -> (String, String) {
    let cleaned_ast_actors: Vec<String> = uc_m
        .uc
        .actors
        .iter()
        .map(|a| {
            a.trim()
                .trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | '[' | ']'))
                .trim()
                .to_string()
        })
        .filter(|a| !a.is_empty())
        .collect();
    let cleaned_singular_actor: Option<String> = uc_m
        .uc
        .actor
        .as_deref()
        .map(|a| {
            a.trim()
                .trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | '[' | ']'))
                .trim()
                .to_string()
        })
        .filter(|a| !a.is_empty());

    let actors: Vec<String> = if !parsed_doc.actors.is_empty() {
        parsed_doc.actors.clone()
    } else if !cleaned_ast_actors.is_empty() {
        cleaned_ast_actors
    } else if let Some(a) = cleaned_singular_actor {
        vec![a]
    } else if let Some(pkg_init_actor) = extract_package_interaction_initiator_actor(uc_m.pkg) {
        vec![pkg_init_actor]
    } else {
        vec!["Operator".to_string()]
    };

    let primary_actor = actors
        .first()
        .cloned()
        .unwrap_or_else(|| "Operator".to_string());
    let actors_markdown = actors
        .iter()
        .map(|a| format!("- `{a}`"))
        .collect::<Vec<_>>()
        .join("\n");
    (primary_actor, actors_markdown)
}

/// Resolves `(preconditions, trigger)` Markdown blocks for `ctx`.
/// Realises: [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
fn resolve_use_case_preconditions_and_trigger(ctx: &UseCaseClauseCtx<'_>) -> (String, String) {
    let subject = &ctx.uc_m.engine_name;
    let primary_actor = ctx.primary_actor;
    let clean_display = ctx.clean_display;

    let preconditions = if !ctx.parsed_doc.preconditions.is_empty() {
        ctx.parsed_doc
            .preconditions
            .iter()
            .map(|p| format!("- {}", p))
            .collect::<Vec<_>>()
            .join("\n")
    } else if !ctx.uc_m.uc.preconditions.is_empty() {
        ctx.uc_m
            .uc
            .preconditions
            .iter()
            .map(|p| format!("- {}", p))
            .collect::<Vec<_>>()
            .join("\n")
    } else if let Some(obj) = ctx.objective_opt {
        format!(
            "- `{subject}` is initialized and ready to enforce `{obj}`.\n\
             - `{primary_actor}` holds authorized operational access to `{subject}`."
        )
    } else {
        format!(
            "- `{subject}` is initialized in its nominal operational state for `{clean_display}`.\n\
             - `{primary_actor}` holds authorized operational access to `{subject}`."
        )
    };

    let trigger = if !ctx.parsed_doc.trigger.is_empty() {
        format!("- {}", ctx.parsed_doc.trigger)
    } else if let Some(obj) = ctx.objective_opt {
        format!("- {obj}")
    } else {
        format!("- `{primary_actor}` initiates `{clean_display}` request on `{subject}`.")
    };

    (preconditions, trigger)
}

/// Resolves numbered `## 4. Main Success Scenario` steps for `ctx`.
/// Realises: [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
fn resolve_use_case_scenario_steps(ctx: &UseCaseClauseCtx<'_>) -> Vec<String> {
    let subject = &ctx.uc_m.engine_name;
    let primary_actor = ctx.primary_actor;
    let clean_display = ctx.clean_display;

    if !ctx.parsed_doc.main_scenario.is_empty() {
        ctx.parsed_doc.main_scenario.clone()
    } else if !ctx.uc_m.uc.steps.is_empty() {
        ctx.uc_m
            .uc
            .steps
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let trimmed = s.trim().trim_matches('"');
                let d_len = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
                let has_num =
                    d_len > 0 && trimmed.get(d_len..).is_some_and(|r| r.starts_with(". "));
                if has_num {
                    trimmed.to_string()
                } else {
                    format!("{}. {}", i.saturating_add(1), trimmed)
                }
            })
            .collect()
    } else if let Some(obj) = ctx.objective_opt {
        if !ctx.parsed_doc.alternate_flows.is_empty() {
            let diag_summary = ctx.parsed_doc.alternate_flows.join(", ");
            vec![
                format!(
                    "1. `{primary_actor}` invokes `{clean_display}` on `{subject}` targeting objective `{obj}`."
                ),
                format!(
                    "2. `{subject}` verifies boundary preconditions and evaluates `{obj}` invariants (guarding `{diag_summary}`)."
                ),
                format!(
                    "3. `{subject}` records verified `{clean_display}` outcome and returns status to `{primary_actor}`."
                ),
            ]
        } else {
            vec![
                format!(
                    "1. `{primary_actor}` invokes `{clean_display}` on `{subject}` targeting objective `{obj}`."
                ),
                format!(
                    "2. `{subject}` verifies boundary preconditions and executes `{obj}`."
                ),
                format!(
                    "3. `{subject}` confirms declared postconditions for `{clean_display}` and returns status to `{primary_actor}`."
                ),
            ]
        }
    } else if !ctx.parsed_doc.alternate_flows.is_empty() {
        let diag_summary = ctx.parsed_doc.alternate_flows.join(", ");
        vec![
            format!("1. `{primary_actor}` invokes `{clean_display}` on `{subject}`."),
            format!(
                "2. `{subject}` evaluates declared preconditions and invariant guards for `{clean_display}` (`{diag_summary}`)."
            ),
            format!(
                "3. `{subject}` completes `{clean_display}` state transition and returns verified status to `{primary_actor}`."
            ),
        ]
    } else {
        vec![
            format!("1. `{primary_actor}` invokes `{clean_display}` request on `{subject}`."),
            format!(
                "2. `{subject}` verifies declared preconditions and structural boundary guards for `{clean_display}`."
            ),
            format!(
                "3. `{subject}` executes `{clean_display}` behavioral transformation while preserving subsystem invariants."
            ),
            format!(
                "4. `{subject}` confirms postcondition satisfaction and returns completion status to `{primary_actor}`."
            ),
        ]
    }
}

/// Resolves `(alternate_flows, postconditions)` Markdown blocks for `ctx`.
/// Realises: [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
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

/// Builds `(realization_matrix_table, spec_source, source_refs)` for `uc_m`.
/// Realises: [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
fn build_use_case_realization_and_sources(
    uc_m: &UseCaseMeta<'_>,
    index: &ForwardSyncIndex<'_>,
) -> (String, String, String) {
    let feat_link_num = uc_m.engine_feat_num;
    let resolved_feat = index.part_contexts.get(feat_link_num.saturating_sub(1));
    let feat_part = resolved_feat
        .map(|ctx| ctx.part.name.as_str())
        .unwrap_or(uc_m.engine_name.as_str());
    let fallback_subsys = to_display_name(&uc_m.pkg_name);
    let feat_subsys = resolved_feat
        .map(|ctx| ctx.subsys_display.as_str())
        .unwrap_or(fallback_subsys.as_str());
    let feat_issue = format_issue_id_cell(resolve_existing_issue_id(
        &index.target_out_dir,
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
        let us_issue = format_issue_id_cell(resolve_existing_issue_id(
            &index.target_out_dir,
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

    let subsys_dir = &uc_m.subsys_dir;
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

    (realization_matrix_table, spec_source, source_refs)
}

/// Synthesizes all `use-cases/UC-XX-*.md` specifications from `index.uc_metas` into `generated_files`.
/// Realises: [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
pub(crate) fn emit_use_case_specs(
    index: &ForwardSyncIndex<'_>,
    generated_files: &mut BTreeMap<String, String>,
) {
    for uc_m in &index.uc_metas {
        let u_num = uc_m.u_num;
        let clean_name = strip_use_case_prefix(&uc_m.uc.name);
        let clean_display = to_display_name(clean_name);
        let uc_slug = sanitize_id(clean_name);
        let rel_path = format!("use-cases/UC-{:02}-{}.md", u_num, uc_slug);
        let _uc_date = resolve_date(&index.target_out_dir, &rel_path, index.today_iso);
        let uc_issue_id =
            format_issue_id_cell(resolve_existing_issue_id(&index.target_out_dir, &rel_path));
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
            r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | {uc_issue_id} |
| **Title** | {title} |
| **Type** | use-case |
| **Primary Actor** | {primary_actor} |
| **Scope** | {subject} |
| **Level** | User Goal |
| **Generation Mode** | subagent |
| **Specification Source** | {spec_source} |

# {title}

## UML Diagrams

```mermaid
{uc_diagram}
```

## 1. Actors
{actors_markdown}

## 2. Preconditions
{preconditions}

## 3. Trigger
{trigger}

## 4. Main Success Scenario
{main_scenario}

## 5. Alternate and Exception Flows
{alternate_flows}

## 6. Postconditions
{postconditions}

## 8. Realization Matrix
{realization_matrix_table}
## Source References
Use case operational flows and lifecycle activity references:
{source_refs}
"#
        );

        generated_files.insert(rel_path, content);
    }
}
