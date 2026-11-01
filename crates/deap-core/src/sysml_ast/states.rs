//! Hierarchical state machine and transition AST definitions and depth-bounded parsers for SysML v2.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-STATES], [DO-330-TQL-4-SYSML-AST]
//! Contract: Defines `StateDef` and `TransitionDef` and provides deterministic, zero-regex `parse_state_defs` extraction of `entry`/`do`/`exit` lifecycle hooks, `first ... then ...` and `<Src>_to_<Tgt>` transitions, and nested `sub_states`.
//! Bounds & Faults: All character scans use bounds-checked `.get` slicing and saturating offset arithmetic; nested `sub_states` traversal is strictly bounded by `MAX_AST_DEPTH = 64` with zero panics or unbounded call-stack growth.

use super::identifiers::{
    extract_doc_comment_before, extract_ident, find_matching_brace, is_word_boundary_after,
    is_word_boundary_before, mask_string_literals, skip_ws,
};
use super::strip_sysml_comments;
use serde::{Deserialize, Serialize};

/// Maximum permitted recursive `StateDef` sub-state nesting depth per DEAP Rust Coding Standard Section 5.
/// Realises: [REQ-SYSML-AST-MAX-DEPTH-BOUND]
const MAX_AST_DEPTH: usize = 64;

/// Represents a directed state transition in a SysML v2 state machine.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-TRANSITIONDEF]
/// Contract: Captures optional transition `name`, `source` and `target` state endpoints, `trigger` event, boolean `guard` condition, transition `effect` action, and `doc` metadata.
/// Bounds & Faults: O(1) structural record construction with deterministic `Default` and `serde` round-trip behavior and zero fallible state mutations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TransitionDef {
    pub name: Option<String>,
    pub source: Option<String>,
    pub target: Option<String>,
    pub trigger: Option<String>,
    pub guard: Option<String>,
    pub effect: Option<String>,
    pub doc: Option<String>,
}

/// Represents a hierarchical state machine state definition or state usage in SysML v2.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-STATEDEF]
/// Contract: Preserves state `name`, optional `doc` metadata, `entry_action`, `do_action`, and `exit_action` lifecycle hooks, outgoing/internal `transitions`, and hierarchical `sub_states`.
/// Bounds & Faults: O(1) structural node construction; nested `sub_states` extraction is strictly bounded to `MAX_AST_DEPTH = 64` to guarantee bounded stack consumption.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StateDef {
    pub name: String,
    pub doc: Option<String>,
    pub entry_action: Option<String>,
    pub do_action: Option<String>,
    pub exit_action: Option<String>,
    #[serde(default)]
    pub transitions: Vec<TransitionDef>,
    #[serde(default)]
    pub sub_states: Vec<StateDef>,
}

/// Trims `raw` and returns `Some(String)` when non-empty or `None` when blank.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-STATE-DEFS]
fn non_empty_string(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Parses a SysML v2 `transition [<name>] first <source> [accept <trigger>] then <target>;` clause.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-STATE-DEFS]
fn parse_first_then_transition(raw_trans: &str) -> TransitionDef {
    let (t_name, after_first) = if let Some(rest) = raw_trans.strip_prefix("first ") {
        (None, rest.trim())
    } else if let Some((before_first, rest_first)) = raw_trans.split_once(" first ") {
        (non_empty_string(before_first), rest_first.trim())
    } else {
        (None, "")
    };

    let (before_then, tgt_str) = match after_first.split_once(" then ") {
        Some((b_then, a_then)) => (b_then.trim(), a_then.trim()),
        None => (after_first, ""),
    };

    let (src_str, trig_str) = match before_then.split_once(" accept ") {
        Some((b_acc, a_acc)) => (b_acc.trim(), a_acc.trim()),
        None => (before_then, ""),
    };

    TransitionDef {
        name: t_name,
        source: non_empty_string(src_str),
        target: non_empty_string(tgt_str),
        trigger: non_empty_string(trig_str),
        guard: None,
        effect: None,
        doc: None,
    }
}

/// Parses a shorthand `transition <Source>_to_<Target>[_on_<Trigger>];` clause into a `TransitionDef`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-STATE-DEFS]
fn parse_conventional_transition(raw_trans: &str) -> TransitionDef {
    let (source, target, trigger) = if let Some(to_idx) = raw_trans.find("_to_") {
        let src_part = raw_trans.get(..to_idx).unwrap_or("");
        let after_to = raw_trans
            .get(to_idx.saturating_add(4)..)
            .unwrap_or("")
            .trim();
        if let Some(on_idx) = after_to.find("_on_") {
            let tgt_part = after_to.get(..on_idx).unwrap_or("");
            let trig_part = after_to.get(on_idx.saturating_add(4)..).unwrap_or("");
            (
                non_empty_string(src_part),
                non_empty_string(tgt_part),
                non_empty_string(trig_part),
            )
        } else {
            (non_empty_string(src_part), non_empty_string(after_to), None)
        }
    } else {
        (None, None, None)
    };

    TransitionDef {
        name: Some(raw_trans.to_string()),
        source,
        target,
        trigger,
        guard: None,
        effect: None,
        doc: None,
    }
}

/// Dispatches a non-empty `transition` body clause between SysML v2 `first ... then ...` syntax and `<Src>_to_<Tgt>` shorthand.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-STATE-DEFS]
fn parse_transition_clause(raw_trans: &str) -> Option<TransitionDef> {
    if raw_trans.is_empty() {
        return None;
    }
    if raw_trans.starts_with("first ") || raw_trans.contains(" first ") {
        Some(parse_first_then_transition(raw_trans))
    } else {
        Some(parse_conventional_transition(raw_trans))
    }
}

/// Strips a leading state-body keyword `kw` at a word boundary and trims trailing `;` and whitespace.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-STATE-DEFS]
fn strip_state_clause<'a>(trimmed: &'a str, kw: &str) -> Option<&'a str> {
    if trimmed.starts_with(kw) && is_word_boundary_after(trimmed, kw.len()) {
        Some(
            trimmed
                .get(kw.len()..)
                .unwrap_or("")
                .trim()
                .trim_end_matches(';')
                .trim(),
        )
    } else {
        None
    }
}

/// Parsed header and optional inner `{ ... }` body slice for a `state` or `state def` declaration.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-STATE-DEFS]
struct ParsedStateSpan<'a> {
    name: &'a str,
    doc: Option<String>,
    body_str: Option<&'a str>,
    next_idx: usize,
}

/// Parses a `state [def] <name> ({ ... } | ;)` span starting at byte offset `st_pos`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-STATE-DEFS]
fn parse_state_span<'a>(
    clean: &'a str,
    masked: &'a str,
    st_pos: usize,
) -> Option<ParsedStateSpan<'a>> {
    let doc = extract_doc_comment_before(clean, st_pos);
    let mut curr = st_pos.saturating_add(5);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    if masked
        .get(curr..)
        .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
    {
        curr = curr.saturating_add(3);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    }
    let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
    if name.is_empty() {
        return None;
    }
    curr = curr.saturating_add(name_len);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));

    let rem_body = masked.get(curr..).unwrap_or("");
    let (body_str, next_idx) = if rem_body.starts_with('{') {
        let close_pos = find_matching_brace(clean, curr)?;
        (
            clean.get(curr.saturating_add(1)..close_pos),
            close_pos.saturating_add(1),
        )
    } else if rem_body.starts_with(';') {
        (None, curr.saturating_add(1))
    } else {
        return None;
    };

    Some(ParsedStateSpan {
        name,
        doc,
        body_str,
        next_idx,
    })
}

/// Builds a `StateDef` by extracting `entry`, `do`, `exit` hooks, `transition` clauses, and depth-bounded `sub_states`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-STATE-DEFS]
fn build_state_def_from_span(span: ParsedStateSpan<'_>, depth: usize) -> StateDef {
    let mut entry_action = None;
    let mut do_action = None;
    let mut exit_action = None;
    let mut transitions = Vec::new();

    let sub_states = if let Some(body) = span.body_str {
        for line in body.lines() {
            let trimmed = line.trim();
            if let Some(act) = strip_state_clause(trimmed, "entry") {
                entry_action = Some(act.to_string());
            } else if let Some(act) = strip_state_clause(trimmed, "do") {
                do_action = Some(act.to_string());
            } else if let Some(act) = strip_state_clause(trimmed, "exit") {
                exit_action = Some(act.to_string());
            } else if let Some(raw_trans) = strip_state_clause(trimmed, "transition") {
                if let Some(trans) = parse_transition_clause(raw_trans) {
                    transitions.push(trans);
                }
            }
        }
        parse_state_defs_at_depth(body, depth.saturating_add(1))
    } else {
        Vec::new()
    };

    StateDef {
        name: span.name.to_string(),
        doc: span.doc,
        entry_action,
        do_action,
        exit_action,
        transitions,
        sub_states,
    }
}

/// Depth-bounded helper extracting `StateDef` declarations up to `MAX_AST_DEPTH`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-STATE-DEFS]
fn parse_state_defs_at_depth(content: &str, depth: usize) -> Vec<StateDef> {
    if depth > MAX_AST_DEPTH {
        return Vec::new();
    }
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut states = Vec::new();
    let mut idx: usize = 0;

    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("state")) {
        let st_pos = idx.saturating_add(pos);
        let after_st = st_pos.saturating_add(5);
        if !is_word_boundary_before(&masked, st_pos) || !is_word_boundary_after(&masked, after_st) {
            idx = after_st;
            continue;
        }
        if let Some(span) = parse_state_span(&clean, &masked, st_pos) {
            idx = span.next_idx;
            states.push(build_state_def_from_span(span, depth));
        } else {
            idx = after_st;
        }
    }
    states
}

/// Parses hierarchical `StateDef` definitions and state usages from SysML v2 source text.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-STATE-DEFS]
/// Contract: Extracts every `state def` and `state` usage in appearance order, capturing `entry`/`do`/`exit` lifecycle hooks, `first ... then ...` and `<Src>_to_<Tgt>` transitions, and nested `sub_states` bounded by `MAX_AST_DEPTH = 64`.
/// Bounds & Faults: Executes in O(N * D) time where `D <= 64` using bounds-checked `.get` slicing and saturating offset arithmetic; exceeding `MAX_AST_DEPTH` truncates deeper sub-states cleanly without panicking.
pub fn parse_state_defs(content: &str) -> Vec<StateDef> {
    parse_state_defs_at_depth(content, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies `parse_state_defs` across `entry`/`do`/`exit` hooks, `first ... then ...` transitions, shorthand transitions, and malformed blocks.
    #[test]
    fn parse_state_defs_extracts_hooks_transitions_and_skips_malformed(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let src = r#"
            doc /* Flight mode supervisor state machine */
            state def FlightModeSupervisor {
                entry InitSupervisor;
                do RunModeVote;
                exit SafeLatchActuators;

                state Standby;
                state Armed;
                state Active;

                transition first Standby accept arm_cmd then Armed;
                transition engage_flow first Armed accept takeoff_trigger then Active;
                transition Active_to_Standby_on_disarm_cmd;
                transition EmergencyReset;
            }

            // Malformed declarations must be skipped without panicking
            state def ;
            state BrokenNoTerminator
            state BrokenUnclosed {
        "#;

        let states = parse_state_defs(src);
        assert_eq!(states.len(), 1);
        let sup = states.first().ok_or("missing FlightModeSupervisor")?;
        assert_eq!(sup.name, "FlightModeSupervisor");
        assert_eq!(
            sup.doc.as_deref(),
            Some("Flight mode supervisor state machine")
        );
        assert_eq!(sup.entry_action.as_deref(), Some("InitSupervisor"));
        assert_eq!(sup.do_action.as_deref(), Some("RunModeVote"));
        assert_eq!(sup.exit_action.as_deref(), Some("SafeLatchActuators"));
        assert_eq!(sup.sub_states.len(), 3);

        assert_eq!(sup.transitions.len(), 4);
        let t0 = sup.transitions.first().ok_or("missing t0")?;
        assert!(t0.name.is_none());
        assert_eq!(t0.source.as_deref(), Some("Standby"));
        assert_eq!(t0.trigger.as_deref(), Some("arm_cmd"));
        assert_eq!(t0.target.as_deref(), Some("Armed"));

        let t1 = sup.transitions.get(1).ok_or("missing t1")?;
        assert_eq!(t1.name.as_deref(), Some("engage_flow"));
        assert_eq!(t1.source.as_deref(), Some("Armed"));
        assert_eq!(t1.trigger.as_deref(), Some("takeoff_trigger"));
        assert_eq!(t1.target.as_deref(), Some("Active"));

        let t2 = sup.transitions.get(2).ok_or("missing t2")?;
        assert_eq!(t2.name.as_deref(), Some("Active_to_Standby_on_disarm_cmd"));
        assert_eq!(t2.source.as_deref(), Some("Active"));
        assert_eq!(t2.target.as_deref(), Some("Standby"));
        assert_eq!(t2.trigger.as_deref(), Some("disarm_cmd"));

        let t3 = sup.transitions.get(3).ok_or("missing t3")?;
        assert_eq!(t3.name.as_deref(), Some("EmergencyReset"));
        assert!(t3.source.is_none());
        assert!(t3.target.is_none());
        assert!(t3.trigger.is_none());
        Ok(())
    }

    /// Verifies `parse_state_defs` parses up to `MAX_AST_DEPTH` (64) and stops cleanly at `MAX_AST_DEPTH + 1` (65) and deep inputs.
    #[test]
    fn parse_state_defs_enforces_max_ast_depth_and_plus_one_bounds(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut nested_sysml = format!("state Leaf_{};", MAX_AST_DEPTH.saturating_add(1));
        for level in (0..=MAX_AST_DEPTH).rev() {
            nested_sysml = format!("state def StateNode_{level} {{ {nested_sysml} }}");
        }
        let parsed = parse_state_defs(&nested_sysml);
        let mut cursor = parsed.first().ok_or("missing root StateNode_0")?;
        for expected_level in 0..=MAX_AST_DEPTH {
            assert_eq!(cursor.name, format!("StateNode_{expected_level}"));
            if expected_level < MAX_AST_DEPTH {
                cursor = cursor
                    .sub_states
                    .first()
                    .ok_or("missing sub_state within MAX_AST_DEPTH")?;
            }
        }
        assert!(
            cursor.sub_states.is_empty(),
            "Sub-state at depth MAX_AST_DEPTH + 1 must be truncated cleanly without panicking"
        );

        let mut very_deep = String::from("state DeepLeaf;");
        for level in (0..160_usize).rev() {
            very_deep = format!("state def DeepState_{level} {{ {very_deep} }}");
        }
        let deep_parsed = parse_state_defs(&very_deep);
        assert_eq!(deep_parsed.len(), 1);
        Ok(())
    }
}
