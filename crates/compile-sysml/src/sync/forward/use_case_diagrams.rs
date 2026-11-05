#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # Use Case Mermaid Diagram Lowering and Actor/Scenario Clause Resolvers
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-USE-CASE-DIAGRAM]
//! Resolves external initiator actors from package `InteractionDef` lifelines, lowers sanitized
//! scenario steps into Check 13B compliant Mermaid `graph TD` Use Case diagrams, and resolves
//! actors, preconditions, triggers, and main scenario steps for `UC-XX-*.md` specifications.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Input `UseCaseMeta` and `ParsedUseCaseDoc` references are valid.
//! - **Postconditions**: Emits sanitized Mermaid labels (`<= 60` chars) and numbered scenario steps.
//! - **Invariants**: Step numbering uses saturating arithmetic (`saturating_add(1)`).
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Missing actors default to `"Operator"`; missing steps synthesize 3..=4 deterministic scenario steps.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Pure single-threaded `O(N)` formatting over Use Case steps and actors.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Consumed by `use_case_emitter`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Verified by Use Case diagram and scenario tests in `sync/forward/tests*.rs`.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-USE-CASE-DIAGRAM]

use super::*;

/// Consolidated borrowed context for resolving UML Use Case behavioral sections for a single `UseCaseMeta`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
pub(crate) struct UseCaseClauseCtx<'a> {
    pub(crate) uc_m: &'a UseCaseMeta<'a>,
    pub(crate) parsed_doc: &'a ParsedUseCaseDoc,
    pub(crate) primary_actor: &'a str,
    pub(crate) clean_display: &'a str,
    pub(crate) objective_opt: Option<&'a str>,
}

/// Extracts the first external initiator lifeline actor from `InteractionDef` nodes in `pkg` excluding internal `PartDef` classifiers.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PKG-INTERACTION-ACTOR]
pub(crate) fn extract_package_interaction_initiator_actor(pkg: &PackageDef) -> Option<String> {
    for inter in &pkg.interaction_defs {
        if let Some(l0) = inter.lifelines.first() {
            let raw_init = if let Some((_, type_name)) = l0.split_once(':') {
                type_name.trim()
            } else {
                l0.trim()
            };
            let clean_init = raw_init
                .trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | '[' | ']'))
                .trim();
            if !clean_init.is_empty() && !pkg.part_defs.iter().any(|pt| pt.name == clean_init) {
                return Some(clean_init.to_string());
            }
        }
    }
    None
}

/// Sanitizes a text label for safe embedding inside a Mermaid Use Case `graph TD` stadium, circle, or subgraph node within 60 characters.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-SANITIZE-MERMAID-UC-LABEL]
pub(crate) fn sanitize_mermaid_uc_label(raw: &str) -> String {
    let mut mapped = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '`' => {}
            '"' => mapped.push('\''),
            ';' => mapped.push(','),
            '\r' | '\n' | '\t' => mapped.push(' '),
            _ => mapped.push(ch),
        }
    }
    let mut collapsed = mapped
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("&lt;", "<")
        .replace("&gt;", ">");

    let digit_len = collapsed.chars().take_while(char::is_ascii_digit).count();
    if digit_len > 0 {
        if let Some(after_digits) = collapsed.get(digit_len..) {
            if let Some(rest) = after_digits.strip_prefix(". ") {
                collapsed = rest.trim().to_string();
            }
        }
    }

    while collapsed.contains("-->") {
        collapsed = collapsed.replace("-->", "->");
    }
    while collapsed.contains("---") {
        collapsed = collapsed.replace("---", "--");
    }

    if collapsed.chars().count() > 60 {
        let truncated: String = collapsed.chars().take(57).collect();
        format!("{truncated}...")
    } else {
        collapsed
    }
}

/// Generates a Check 13B compliant UML Use Case Mermaid `graph TD` diagram with system boundary, stadium use case nodes, and actor links.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-GENERATE-USE-CASE-DIAGRAM]
pub(crate) fn generate_use_case_diagram(
    primary_actor: &str,
    subject: &str,
    clean_title: &str,
    main_scenario: &[String],
) -> String {
    let clean_subject = sanitize_mermaid_uc_label(subject);
    let clean_main_title = sanitize_mermaid_uc_label(clean_title);
    let clean_actor = sanitize_mermaid_uc_label(primary_actor);

    let mut lines = vec![
        "graph TD".to_string(),
        format!("    subgraph \"{} System Boundary\"", clean_subject),
        format!("        UC_Main([\"{}\"])", clean_main_title),
    ];

    let mut step_ids = Vec::new();
    for (idx, step) in main_scenario.iter().enumerate().take(6) {
        let step_id = format!("UC_Step{:02}", idx.saturating_add(1));
        let clean_step = sanitize_mermaid_uc_label(step);
        lines.push(format!("        {}([\"{}\"])", step_id, clean_step));
        step_ids.push(step_id);
    }
    lines.push("    end".to_string());

    let actor_id = sanitize_id(&clean_actor);
    lines.push(format!(
        "    {}((\"{}\")) --- UC_Main",
        actor_id, clean_actor
    ));
    for step_id in step_ids {
        lines.push(format!("    UC_Main -. \"<<include>>\" .-> {}", step_id));
    }

    lines.join("\n")
}

/// Resolves `(primary_actor, actors_markdown)` for `uc_m` from `parsed_doc`, AST actors, package interaction lifelines, or `"Operator"`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
pub(crate) fn resolve_use_case_actors(
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
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
pub(crate) fn resolve_use_case_preconditions_and_trigger(
    ctx: &UseCaseClauseCtx<'_>,
) -> (String, String) {
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

/// Synthesizes fallback numbered `## 4. Main Success Scenario` steps when neither `parsed_doc` nor `uc.steps` is populated.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
fn resolve_fallback_scenario_steps(ctx: &UseCaseClauseCtx<'_>) -> Vec<String> {
    let subject = &ctx.uc_m.engine_name;
    let primary_actor = ctx.primary_actor;
    let clean_display = ctx.clean_display;

    if let Some(obj) = ctx.objective_opt {
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

/// Resolves numbered `## 4. Main Success Scenario` steps for `ctx`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-USE-CASE-EMITTER]
pub(crate) fn resolve_use_case_scenario_steps(ctx: &UseCaseClauseCtx<'_>) -> Vec<String> {
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
    } else {
        resolve_fallback_scenario_steps(ctx)
    }
}
