#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # User Story AST Context Descriptors and Provenance-Safe Label Formatters
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-US-PROJECTION-CTX]
//! Screens `ConstraintDef` expressions against `ENGINEERING_PARAMETER_TERMS` for unitless numeric
//! literals, formats provenance-safe constraint and requirement labels, parses `ActionDef` and
//! `InteractionDef` doc clauses, and resolves `StateDef` transitions.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Input AST nodes and doc comments are valid UTF-8 strings.
//! - **Postconditions**: Emits provenance-safe labels and transition bindings.
//! - **Invariants**: Zero unitless numeric literals paired with engineering parameter terms are emitted into prose.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Expressions containing unitless engineering parameter literals fall back to the constraint identifier.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Pure single-threaded `O(|expr|)` linear scans without regular expressions.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Consumed by `epic_renderer`, `feature_layers`, `feature_schema`, `story_emitter`, and `story_layers`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Verified by provenance and User Story tests in `sync/forward/tests*.rs`.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-US-PROJECTION-CTX]

use super::*;

/// Canonical 26-term engineering parameter vocabulary checked when projecting `ConstraintDef` expressions into prose.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-ENG-PARAM-TERMS]
pub(crate) const ENGINEERING_PARAMETER_TERMS: &[&str] = &[
    "payload",
    "current",
    "latency",
    "delay",
    "timeout",
    "frequency",
    "voltage",
    "power",
    "energy",
    "bandwidth",
    "throughput",
    "velocity",
    "speed",
    "altitude",
    "heading",
    "acceleration",
    "rate",
    "clock",
    "dimension",
    "resistance",
    "capacitance",
    "mass",
    "thrust",
    "torque",
    "pressure",
    "temperature",
];

/// Checks whether a `ConstraintDef` expression can be safely embedded into specification prose without pairing an engineering parameter term with a unitless numeric literal or broken link token.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PROVENANCE-SAFE-EXPR]
pub(crate) fn is_provenance_safe_constraint_expr(expr: &str) -> bool {
    let trimmed = expr.trim();
    if trimmed.is_empty() || trimmed.contains("](") {
        return false;
    }
    let has_digit = trimmed.chars().any(|c| c.is_ascii_digit());
    if !has_digit {
        return true;
    }
    let has_eng_term = trimmed
        .split(|c: char| !c.is_alphabetic())
        .filter(|w| !w.is_empty())
        .any(|word| {
            ENGINEERING_PARAMETER_TERMS
                .iter()
                .any(|term| word.eq_ignore_ascii_case(term))
        });
    !has_eng_term
}

/// Formats a `ConstraintDef` into a provenance-safe display label, omitting expressions that contain unitless engineering parameter literals.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORMAT-CONSTRAINT-LABEL]
pub(crate) fn format_constraint_label(c: &ConstraintDef) -> String {
    let name = c.name.trim();
    if name.is_empty() {
        return String::new();
    }
    let expr = c.expression.trim();
    if is_provenance_safe_constraint_expr(expr) {
        format!("{name} ({expr})")
    } else {
        name.to_string()
    }
}

/// Formats a `RequirementDef` identifier and name into a canonical traceability label (`"{req_id} ({name})"` when distinct).
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORMAT-REQUIREMENT-LABEL]
pub(crate) fn format_requirement_label(req: &RequirementDef) -> String {
    let id = req.req_id.trim();
    let name = req.name.trim();
    if !id.is_empty() && !name.is_empty() && id != name {
        format!("{id} ({name})")
    } else if !id.is_empty() {
        id.to_string()
    } else {
        name.to_string()
    }
}

/// Structured User Story doc-comment clauses extracted deterministically from an `ActionDef` or `InteractionDef`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSED-STORY-DOC]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ParsedStoryDoc {
    pub(crate) actor: Option<String>,
    pub(crate) precondition: Option<String>,
    pub(crate) outcome: Option<String>,
}

/// Parses optional `Actor:`, `Preconditions:`/`Precondition:`, and `Outcome:`/`Postconditions:`/`Postcondition:`/`Objective:` clauses from a SysML doc comment without regular expressions.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSE-STORY-DOC]
pub(crate) fn parse_story_doc(doc_opt: Option<&str>) -> ParsedStoryDoc {
    let mut parsed = ParsedStoryDoc::default();
    let Some(raw) = doc_opt.map(str::trim).filter(|s| !s.is_empty()) else {
        return parsed;
    };

    for line in raw.lines() {
        let trimmed = line
            .trim()
            .trim_start_matches('*')
            .trim_start_matches('-')
            .trim();
        if trimmed.is_empty() || is_stale_user_story_boilerplate(trimmed) {
            continue;
        }
        if let Some(rest) = strip_prefix_ignore_ascii_case(trimmed, "Actor:")
            .or_else(|| strip_prefix_ignore_ascii_case(trimmed, "Actors:"))
        {
            let first_actor = rest
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | '[' | ']'))
                .trim();
            if !first_actor.is_empty() {
                parsed.actor = Some(first_actor.to_string());
            }
        } else if let Some(rest) = strip_prefix_ignore_ascii_case(trimmed, "Preconditions:")
            .or_else(|| strip_prefix_ignore_ascii_case(trimmed, "Precondition:"))
        {
            let val = rest.trim().trim_end_matches('.').trim();
            if !val.is_empty() && !val.contains("](") {
                parsed.precondition = Some(val.to_string());
            }
        } else if let Some(rest) = strip_prefix_ignore_ascii_case(trimmed, "Outcome:")
            .or_else(|| strip_prefix_ignore_ascii_case(trimmed, "Postconditions:"))
            .or_else(|| strip_prefix_ignore_ascii_case(trimmed, "Postcondition:"))
            .or_else(|| strip_prefix_ignore_ascii_case(trimmed, "Objective:"))
        {
            let val = rest.trim().trim_end_matches('.').trim();
            if !val.is_empty() && !val.contains("](") {
                parsed.outcome = Some(val.to_string());
            }
        } else if parsed.outcome.is_none()
            && !trimmed.contains("](")
            && is_provenance_safe_constraint_expr(trimmed)
        {
            let val = trimmed.trim_end_matches('.').trim();
            if !val.is_empty() {
                parsed.outcome = Some(val.to_string());
            }
        }
    }

    parsed
}

/// Merges primary `ActionDef` doc clauses with fallback `InteractionDef` doc clauses field-by-field.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-MERGE-STORY-DOCS]
pub(crate) fn merge_story_docs(
    primary: ParsedStoryDoc,
    fallback: ParsedStoryDoc,
) -> ParsedStoryDoc {
    ParsedStoryDoc {
        actor: primary.actor.or(fallback.actor),
        precondition: primary.precondition.or(fallback.precondition),
        outcome: primary.outcome.or(fallback.outcome),
    }
}

/// Extracts the first non-empty sanitized actor declared across a slice of `UseCaseDef` nodes.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-EXTRACT-FIRST-UC-ACTOR]
pub(crate) fn extract_first_uc_actor(use_cases: &[UseCaseDef]) -> Option<String> {
    for uc in use_cases {
        let parsed = parse_use_case_doc(uc.doc.as_deref());
        for candidate in parsed
            .actors
            .iter()
            .chain(uc.actors.iter())
            .chain(uc.actor.iter())
        {
            let clean = candidate
                .trim()
                .trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | '[' | ']'))
                .trim();
            if !clean.is_empty() {
                return Some(clean.to_string());
            }
        }
    }
    None
}

/// Formats typed `in` and `out` parameters of an `ActionDef` as `ActionName(in p: T, ..., out q: U)` (or `ActionName()` when parameterless).
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORMAT-ACTION-CALL]
pub(crate) fn format_action_call_with_params(
    action_name: &str,
    act_opt: Option<&ActionDef>,
) -> String {
    let Some(act) = act_opt else {
        return format!("{action_name}()");
    };
    let mut args = Vec::new();
    for p in &act.in_params {
        let name = p.name.trim();
        let ty = p.type_name.trim();
        if ty.is_empty() {
            args.push(format!("in {name}"));
        } else {
            args.push(format!("in {name}: {ty}"));
        }
    }
    for p in &act.out_params {
        let name = p.name.trim();
        let ty = p.type_name.trim();
        if ty.is_empty() {
            args.push(format!("out {name}"));
        } else {
            args.push(format!("out {name}: {ty}"));
        }
    }
    format!("{action_name}({})", args.join(", "))
}

/// Resolved `StateDef` transition endpoints and AST reference matched to a User Story action.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-MATCHED-STORY-TRANSITION]
pub(crate) struct MatchedStoryTransition<'a> {
    pub(crate) transition: &'a TransitionDef,
    pub(crate) source: String,
    pub(crate) target: String,
}

/// Resolves the most specific `StateDef` transition governing `action_name` across `states`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RESOLVE-STORY-TRANSITION]
pub(crate) fn resolve_story_transition<'a>(
    action_name: &str,
    states: &[&'a StateDef],
) -> Option<MatchedStoryTransition<'a>> {
    let action_lower = action_name.to_ascii_lowercase();
    for st in states {
        for tr in &st.transitions {
            let matches_action = tr
                .trigger
                .as_deref()
                .is_some_and(|t| t.to_ascii_lowercase().contains(&action_lower))
                || tr
                    .effect
                    .as_deref()
                    .is_some_and(|e| e.to_ascii_lowercase().contains(&action_lower))
                || tr
                    .name
                    .as_deref()
                    .is_some_and(|n| n.to_ascii_lowercase().contains(&action_lower));
            if matches_action {
                if let Some((source, target, _)) = extract_transition_endpoints(tr) {
                    return Some(MatchedStoryTransition {
                        transition: tr,
                        source,
                        target,
                    });
                }
            }
        }
    }
    for st in states {
        for tr in &st.transitions {
            if let Some((source, target, _)) = extract_transition_endpoints(tr) {
                if source != "[*]" && target != "[*]" {
                    return Some(MatchedStoryTransition {
                        transition: tr,
                        source,
                        target,
                    });
                }
            }
        }
    }
    for st in states {
        for tr in &st.transitions {
            if let Some((source, target, _)) = extract_transition_endpoints(tr) {
                return Some(MatchedStoryTransition {
                    transition: tr,
                    source,
                    target,
                });
            }
        }
    }
    None
}

/// Consolidated borrowed AST projection context for rendering a single User Story (`US-XX-*.md`).
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-US-PROJECTION-CTX]
pub(crate) struct UserStoryProjectionCtx<'a> {
    pub(crate) s_num: usize,
    pub(crate) story_actor: &'a str,
    pub(crate) engine_name: &'a str,
    pub(crate) action_name: &'a str,
    pub(crate) interaction_name: &'a str,
    pub(crate) reply_msg: &'a str,
    pub(crate) pkg_name: &'a str,
    pub(crate) call_sig: &'a str,
    pub(crate) parsed_doc: &'a ParsedStoryDoc,
    pub(crate) act_opt: Option<&'a ActionDef>,
    pub(crate) curr_pkg: Option<&'a PackageDef>,
    pub(crate) target_part: Option<&'a PartDef>,
    pub(crate) states: &'a [&'a StateDef],
    pub(crate) matched_transition: Option<&'a MatchedStoryTransition<'a>>,
    pub(crate) constraints: &'a [&'a ConstraintDef],
    pub(crate) reqs: &'a [&'a RequirementDef],
}

/// Metadata associating a `PartDef` with its owning package display name and directory slug.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARTCONTEXT]
pub(crate) struct PartContext<'a> {
    pub(crate) part: &'a PartDef,
    pub(crate) pkg_name: String,
    pub(crate) subsys_display: String,
    pub(crate) subsys_dir: String,
}
