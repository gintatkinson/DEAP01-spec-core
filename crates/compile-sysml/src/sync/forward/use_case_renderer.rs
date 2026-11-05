#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # Structured Use Case AST Documentation Parser
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSE-USE-CASE-DOC]
//! Parses single-line and multi-line `UseCaseDef.doc` UML and BDD behavioral clauses into
//! `ParsedUseCaseDoc` under ISO/IEC/IEEE 29148:2018.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Input `doc_opt` is an optional UTF-8 string slice.
//! - **Postconditions**: Produces a `ParsedUseCaseDoc` with 1-based numbered `main_scenario` steps.
//! - **Invariants**: All step numbering uses saturating arithmetic (`saturating_add(1)`).
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Empty or absent doc comments return `ParsedUseCaseDoc::default()` without panicking.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Pure single-threaded `O(|doc|)` linear scanning over lines and inline clauses.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Consumes `use_case_clauses`; consumed by `model_indexer`, `story_context`, and `use_case_emitter`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Verified by Use Case doc parsing tests in `sync/forward/tests*.rs`.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSE-USE-CASE-DOC]

use super::*;

/// Structured Use Case sections parsed from `UseCaseDef.doc` comments.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSEDUSECASEDOC]
#[derive(Debug, Default)]
pub(crate) struct ParsedUseCaseDoc {
    pub(crate) actors: Vec<String>,
    pub(crate) subject: Option<String>,
    pub(crate) preconditions: Vec<String>,
    pub(crate) trigger: String,
    pub(crate) main_scenario: Vec<String>,
    pub(crate) alternate_flows: Vec<String>,
    pub(crate) postconditions: Vec<String>,
}

/// Active clause section tracked across multi-line `UseCaseDef.doc` continuations.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSE-USE-CASE-DOC]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum UseCaseDocSection {
    #[default]
    None,
    Actors,
    Subject,
    Preconditions,
    Trigger,
    When,
    Main,
    Alt,
    Diagnostic,
    Postconditions,
    Then,
}

/// Mutable state accumulator used while projecting `UseCaseDef.doc` clauses into `ParsedUseCaseDoc`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSE-USE-CASE-DOC]
#[derive(Debug, Default)]
struct UseCaseDocAccumulator {
    parsed: ParsedUseCaseDoc,
    current_section: UseCaseDocSection,
    explicit_main_header: bool,
    bdd_when_steps: Vec<String>,
    bdd_then_steps: Vec<String>,
    freeform_lines: Vec<String>,
}

/// Checks whether a trimmed `UseCaseDef.doc` segment begins with an unordered bullet or numbered list marker.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSE-USE-CASE-DOC]
fn has_leading_list_marker(trimmed_seg: &str) -> bool {
    if ["- ", "* ", "+ "].iter().any(|p| trimmed_seg.starts_with(p)) {
        return true;
    }
    let d_len = trimmed_seg.chars().take_while(char::is_ascii_digit).count();
    d_len > 0 && trimmed_seg.get(d_len..).is_some_and(|r| r.starts_with(". ") || r.starts_with(") "))
}

/// Splits comma-separated actor declarations, strips quote/backtick/bracket wrappers, and appends unique non-empty names.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSE-USE-CASE-DOC]
fn push_actor_csv_tokens(actors: &mut Vec<String>, raw_csv: &str) {
    for raw_actor in raw_csv.split(',') {
        let clean = raw_actor.trim().trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | '[' | ']')).trim();
        push_unique_uc_clause(actors, clean);
    }
}

/// Strips outer whitespace, backticks, quotes, and brackets from a `Subject:` value, returning `None` when empty.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSE-USE-CASE-DOC]
fn sanitize_subject_token(raw_subj: &str) -> Option<String> {
    let clean = raw_subj.trim().trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | '[' | ']')).trim();
    (!clean.is_empty()).then(|| clean.to_string())
}

/// Appends `clause` onto `trigger`, joining with `sep` when `trigger` is already non-empty.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSE-USE-CASE-DOC]
fn append_trigger_clause(trigger: &mut String, clause: &str, sep: &str) {
    if trigger.is_empty() {
        *trigger = clause.to_string();
    } else {
        *trigger = format!("{trigger}{sep}{clause}");
    }
}

/// Normalizes `raw_diag` and records `"Diagnostic: <code>"` in `alternate_flows` when non-empty.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSE-USE-CASE-DOC]
fn push_cleaned_diagnostic_clause(alternate_flows: &mut Vec<String>, raw_diag: &str) {
    let cleaned_diag = clean_diagnostic_value(raw_diag);
    if !cleaned_diag.is_empty() {
        let formatted = format!("Diagnostic: {cleaned_diag}");
        push_unique_uc_clause(alternate_flows, &formatted);
    }
}

/// Concatenates `clean` onto the trailing step in `steps` unless `starts_new_list_item` starts a distinct clause.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSE-USE-CASE-DOC]
fn append_or_push_step_continuation(
    steps: &mut Vec<String>,
    clean: &str,
    starts_new_list_item: bool,
) {
    if !starts_new_list_item {
        if let Some(last) = steps.last_mut() {
            *last = format!("{last} {clean}");
            return;
        }
    }
    push_unique_uc_clause(steps, clean);
}

/// Dispatches `Actors:`, `Subject:`, `Preconditions:`/`Given:`, `Trigger:`, and `When:` headers into `acc`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSE-USE-CASE-DOC]
fn apply_actor_or_trigger_uc_clause(acc: &mut UseCaseDocAccumulator, clean: &str) -> bool {
    if let Some(act_str) = match_case_insensitive_uc_label(clean, "Actors:")
        .or_else(|| match_case_insensitive_uc_label(clean, "Actor:"))
    {
        push_actor_csv_tokens(&mut acc.parsed.actors, act_str);
        acc.current_section = UseCaseDocSection::Actors;
        return true;
    }
    if let Some(subj) = match_case_insensitive_uc_label(clean, "Subject:") {
        if let Some(clean_subj) = sanitize_subject_token(subj) {
            acc.parsed.subject = Some(clean_subj);
        }
        acc.current_section = UseCaseDocSection::Subject;
        return true;
    }
    if let Some(pre) = match_case_insensitive_uc_label(clean, "Preconditions:")
        .or_else(|| match_case_insensitive_uc_label(clean, "Precondition:"))
        .or_else(|| match_case_insensitive_uc_label(clean, "Given:"))
    {
        extract_inline_diagnostics(pre, &mut acc.parsed.alternate_flows);
        push_unique_uc_clause(&mut acc.parsed.preconditions, pre);
        acc.current_section = UseCaseDocSection::Preconditions;
        return true;
    }
    if let Some(trig) = match_case_insensitive_uc_label(clean, "Trigger:") {
        if !trig.is_empty() {
            extract_inline_diagnostics(trig, &mut acc.parsed.alternate_flows);
            append_trigger_clause(&mut acc.parsed.trigger, trig, "; ");
        }
        acc.current_section = UseCaseDocSection::Trigger;
        return true;
    }
    if let Some(when_clause) = match_case_insensitive_uc_label(clean, "When:") {
        if !when_clause.is_empty() {
            extract_inline_diagnostics(when_clause, &mut acc.parsed.alternate_flows);
            append_trigger_clause(&mut acc.parsed.trigger, when_clause, "; ");
            push_unique_uc_clause(&mut acc.bdd_when_steps, when_clause);
        }
        acc.current_section = UseCaseDocSection::When;
        return true;
    }
    false
}

/// Dispatches `Main Success Scenario:`, `Alternate/Exception Flows:`, `Diagnostic:`, `Postconditions:`, and `Then:` headers into `acc`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSE-USE-CASE-DOC]
fn apply_scenario_or_outcome_uc_clause(acc: &mut UseCaseDocAccumulator, clean: &str) -> bool {
    if let Some(inline_main) = match_case_insensitive_uc_label(clean, "Main Success Scenario:") {
        acc.explicit_main_header = true;
        extract_inline_diagnostics(inline_main, &mut acc.parsed.alternate_flows);
        push_unique_uc_clause(&mut acc.parsed.main_scenario, inline_main);
        acc.current_section = UseCaseDocSection::Main;
        return true;
    }
    if let Some(inline_alt) =
        match_case_insensitive_uc_label(clean, "Alternate and Exception Flows:")
            .or_else(|| match_case_insensitive_uc_label(clean, "Alternate Flows:"))
            .or_else(|| match_case_insensitive_uc_label(clean, "Exception Flows:"))
    {
        push_unique_uc_clause(&mut acc.parsed.alternate_flows, inline_alt);
        acc.current_section = UseCaseDocSection::Alt;
        return true;
    }
    if let Some(diag_clause) = match_case_insensitive_uc_label(clean, "Diagnostic:") {
        push_cleaned_diagnostic_clause(&mut acc.parsed.alternate_flows, diag_clause);
        acc.current_section = UseCaseDocSection::Diagnostic;
        return true;
    }
    if let Some(post) = match_case_insensitive_uc_label(clean, "Postconditions:")
        .or_else(|| match_case_insensitive_uc_label(clean, "Postcondition:"))
    {
        extract_inline_diagnostics(post, &mut acc.parsed.alternate_flows);
        push_unique_uc_clause(&mut acc.parsed.postconditions, post);
        acc.current_section = UseCaseDocSection::Postconditions;
        return true;
    }
    if let Some(then_clause) = match_case_insensitive_uc_label(clean, "Then:") {
        extract_inline_diagnostics(then_clause, &mut acc.parsed.alternate_flows);
        push_unique_uc_clause(&mut acc.parsed.postconditions, then_clause);
        push_unique_uc_clause(&mut acc.bdd_then_steps, then_clause);
        acc.current_section = UseCaseDocSection::Then;
        return true;
    }
    false
}

/// Appends an unlabeled continuation segment to the currently active `UseCaseDocSection` in `acc`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSE-USE-CASE-DOC]
fn apply_continuation_uc_clause(
    acc: &mut UseCaseDocAccumulator,
    clean: &str,
    starts_new_list_item: bool,
) {
    match acc.current_section {
        UseCaseDocSection::Actors => {
            push_actor_csv_tokens(&mut acc.parsed.actors, clean);
        }
        UseCaseDocSection::Subject => {
            if acc.parsed.subject.is_none() {
                acc.parsed.subject = sanitize_subject_token(clean);
            }
        }
        UseCaseDocSection::Trigger => {
            extract_inline_diagnostics(clean, &mut acc.parsed.alternate_flows);
            append_trigger_clause(&mut acc.parsed.trigger, clean, " ");
        }
        UseCaseDocSection::When => {
            extract_inline_diagnostics(clean, &mut acc.parsed.alternate_flows);
            let sep = if starts_new_list_item { "; " } else { " " };
            append_trigger_clause(&mut acc.parsed.trigger, clean, sep);
            append_or_push_step_continuation(&mut acc.bdd_when_steps, clean, starts_new_list_item);
        }
        UseCaseDocSection::Main => {
            extract_inline_diagnostics(clean, &mut acc.parsed.alternate_flows);
            push_unique_uc_clause(&mut acc.parsed.main_scenario, clean);
        }
        UseCaseDocSection::Alt => {
            push_unique_uc_clause(&mut acc.parsed.alternate_flows, clean);
        }
        UseCaseDocSection::Diagnostic => {
            push_cleaned_diagnostic_clause(&mut acc.parsed.alternate_flows, clean);
        }
        UseCaseDocSection::Preconditions => {
            extract_inline_diagnostics(clean, &mut acc.parsed.alternate_flows);
            push_unique_uc_clause(&mut acc.parsed.preconditions, clean);
        }
        UseCaseDocSection::Postconditions => {
            extract_inline_diagnostics(clean, &mut acc.parsed.alternate_flows);
            push_unique_uc_clause(&mut acc.parsed.postconditions, clean);
        }
        UseCaseDocSection::Then => {
            extract_inline_diagnostics(clean, &mut acc.parsed.alternate_flows);
            append_or_push_step_continuation(
                &mut acc.parsed.postconditions,
                clean,
                starts_new_list_item,
            );
            append_or_push_step_continuation(&mut acc.bdd_then_steps, clean, starts_new_list_item);
        }
        UseCaseDocSection::None => {
            extract_inline_diagnostics(clean, &mut acc.parsed.alternate_flows);
            push_unique_uc_clause(&mut acc.freeform_lines, clean);
        }
    }
}

/// Synthesizes fallback `main_scenario` steps from BDD `When`/`Then` or freeform clauses and enforces 1-based step numbering.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSE-USE-CASE-DOC]
fn finalize_parsed_use_case_doc(mut acc: UseCaseDocAccumulator) -> ParsedUseCaseDoc {
    if !acc.explicit_main_header && acc.parsed.main_scenario.is_empty() {
        for step in acc.bdd_when_steps.iter().chain(acc.bdd_then_steps.iter()) {
            push_unique_uc_clause(&mut acc.parsed.main_scenario, step);
        }
        if acc.parsed.main_scenario.is_empty() {
            for line in &acc.freeform_lines {
                push_unique_uc_clause(&mut acc.parsed.main_scenario, line);
            }
        }
    }

    acc.parsed.main_scenario = acc
        .parsed
        .main_scenario
        .into_iter()
        .enumerate()
        .map(|(idx, step)| {
            let s = step.trim();
            let digit_len = s.chars().take_while(char::is_ascii_digit).count();
            let has_number =
                digit_len > 0 && s.get(digit_len..).is_some_and(|r| r.starts_with(". "));
            if has_number {
                s.to_string()
            } else {
                format!("{}. {}", idx.saturating_add(1), s)
            }
        })
        .collect();

    acc.parsed
}

/// Parses structured UML or BDD-formatted `UseCaseDef.doc` comments across single-line and multi-line forms into `ParsedUseCaseDoc`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARSE-USE-CASE-DOC]
pub(crate) fn parse_use_case_doc(doc_opt: Option<&str>) -> ParsedUseCaseDoc {
    let doc_str = match doc_opt {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => return ParsedUseCaseDoc::default(),
    };

    let mut acc = UseCaseDocAccumulator::default();
    for raw_line in doc_str.lines() {
        let trimmed_line = raw_line.trim();
        if trimmed_line.is_empty() {
            continue;
        }
        for raw_seg in split_inline_use_case_clauses(trimmed_line) {
            let trimmed_seg = raw_seg.trim();
            if trimmed_seg.is_empty() {
                continue;
            }
            let starts_new_list_item = has_leading_list_marker(trimmed_seg);
            let clean = normalize_doc_clause_head(trimmed_seg);
            if clean.is_empty() {
                continue;
            }
            if !apply_actor_or_trigger_uc_clause(&mut acc, &clean)
                && !apply_scenario_or_outcome_uc_clause(&mut acc, &clean)
            {
                apply_continuation_uc_clause(&mut acc, &clean, starts_new_list_item);
            }
        }
    }

    finalize_parsed_use_case_doc(acc)
}
