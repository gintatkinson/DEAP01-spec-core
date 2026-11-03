//! Behavioral action AST definitions and zero-regex action parsers for SysML v2.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-ACTIONS], [DO-330-TQL-4-SYSML-AST]
//! Contract: Defines `ActionDef` for both `action def` definitions and `action` usages and provides deterministic `parse_action_defs` extraction of directed `in`/`out` parameters, `perform` performer allocations, and ordered execution `step` sequences.
//! Bounds & Faults: Executes in linear O(N) time and O(N) space over comment-stripped buffers using bounds-checked `.get` slicing, saturating offset arithmetic, and zero panics on malformed or multi-byte UTF-8 inputs.

use super::identifiers::{
    default_true, extract_doc_comment_before, extract_ident, extract_type_ident,
    find_matching_brace, is_word_boundary_after, is_word_boundary_before, mask_string_literals,
    skip_ws, AttributeDef,
};
use super::strip_sysml_comments;
use serde::{Deserialize, Serialize};

/// Represents a behavioral action definition or action usage in SysML v2.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-ACTIONDEF]
/// Contract: Captures the action `name`, optional `doc` metadata, directed `in_params` and `out_params`, undirected `parameters`, optional `performer` allocation, ordered execution `steps`, and `is_def` (`true` for `action def`, `false` for `action` usages).
/// Bounds & Faults: O(1) structural record construction with deterministic `serde` defaults (`is_def` defaulting to `true`) and zero fallible state mutations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionDef {
    pub name: String,
    pub doc: Option<String>,
    #[serde(default)]
    pub in_params: Vec<AttributeDef>,
    #[serde(default)]
    pub out_params: Vec<AttributeDef>,
    #[serde(default)]
    pub parameters: Vec<AttributeDef>,
    pub performer: Option<String>,
    #[serde(default)]
    pub steps: Vec<String>,
    #[serde(default = "default_true")]
    pub is_def: bool,
}

impl Default for ActionDef {
    /// Constructs a default `ActionDef` with empty parameter and step vectors and `is_def` set to `true`.
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/ACTION-DEF-DEFAULT]
    fn default() -> Self {
        Self {
            name: String::new(),
            doc: None,
            in_params: Vec::new(),
            out_params: Vec::new(),
            parameters: Vec::new(),
            performer: None,
            steps: Vec::new(),
            is_def: true,
        }
    }
}

/// Parses a single `in` or `out` action parameter declaration into `(is_in, AttributeDef)`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-ACTION-PARAM]
fn parse_action_param(s: &str) -> Option<(bool, AttributeDef)> {
    let trimmed = s.trim();
    let (is_in, after_dir) = if trimmed.starts_with("in") && is_word_boundary_after(trimmed, 2) {
        (true, trimmed.get(2..).unwrap_or("").trim_start())
    } else if trimmed.starts_with("out") && is_word_boundary_after(trimmed, 3) {
        (false, trimmed.get(3..).unwrap_or("").trim_start())
    } else {
        return None;
    };
    let after_kw = if after_dir.starts_with("attribute") && is_word_boundary_after(after_dir, 9) {
        after_dir.get(9..).unwrap_or("").trim_start()
    } else if after_dir.starts_with("item") && is_word_boundary_after(after_dir, 4) {
        after_dir.get(4..).unwrap_or("").trim_start()
    } else {
        after_dir
    };
    let (name, name_len) = extract_ident(after_kw);
    if name.is_empty() {
        return None;
    }
    let after_name = after_kw.get(name_len..).unwrap_or("").trim_start();
    let after_colon = after_name.strip_prefix(':')?.trim_start();
    let (type_name, type_len) = extract_type_ident(after_colon);
    if type_len == 0 {
        return None;
    }
    Some((
        is_in,
        AttributeDef {
            name: name.to_string(),
            type_name: type_name.to_string(),
            default_value: None,
            doc: None,
        },
    ))
}

/// Parsed header, optional parameter list slice, and optional body slice for an `action` declaration.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-ACTION-DEFS]
struct ParsedActionSpan<'a> {
    name: &'a str,
    doc: Option<String>,
    is_def: bool,
    params_str: Option<&'a str>,
    body_str: Option<&'a str>,
    next_idx: usize,
}

/// Parses an `action [def] <name> [(...)] ({ ... } | ;)` span starting at byte offset `act_pos`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-ACTION-DEFS]
fn parse_action_span<'a>(
    clean: &'a str,
    masked: &'a str,
    act_pos: usize,
) -> Option<ParsedActionSpan<'a>> {
    let doc = extract_doc_comment_before(clean, act_pos);
    let mut curr = act_pos.saturating_add(6);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));

    let is_def = if masked
        .get(curr..)
        .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
    {
        curr = curr.saturating_add(3);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
        true
    } else {
        false
    };

    let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
    if name.is_empty() {
        return None;
    }
    curr = curr.saturating_add(name_len);
    curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));

    let mut params_str = None;
    if masked.get(curr..).is_some_and(|s| s.starts_with('(')) {
        let param_start = curr.saturating_add(1);
        let close_paren = masked.get(param_start..).and_then(|s| s.find(')'))?;
        let param_end = param_start.saturating_add(close_paren);
        params_str = clean.get(param_start..param_end);
        curr = param_end.saturating_add(1);
        curr = curr.saturating_add(skip_ws(masked.get(curr..).unwrap_or("")));
    }

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

    Some(ParsedActionSpan {
        name,
        doc,
        is_def,
        params_str,
        body_str,
        next_idx,
    })
}

/// Appends a parsed directed action parameter to `in_params` or `out_params`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-ACTION-PARAM]
fn record_directed_param(
    is_in: bool,
    attr: AttributeDef,
    in_params: &mut Vec<AttributeDef>,
    out_params: &mut Vec<AttributeDef>,
) {
    if is_in {
        in_params.push(attr);
    } else {
        out_params.push(attr);
    }
}

/// Builds an `ActionDef` by extracting directed parameters, performer allocation, and execution steps from `span`.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-ACTION-DEFS]
fn build_action_def_from_span(span: ParsedActionSpan<'_>) -> ActionDef {
    let mut in_params = Vec::new();
    let mut out_params = Vec::new();
    let mut steps = Vec::new();
    let mut performer = None;

    if let Some(params) = span.params_str {
        for part in params.split(',') {
            if let Some((is_in, attr)) = parse_action_param(part) {
                record_directed_param(is_in, attr, &mut in_params, &mut out_params);
            }
        }
    }

    if let Some(body) = span.body_str {
        for line in body.lines() {
            let trimmed = line.trim();
            if let Some((is_in, attr)) = parse_action_param(trimmed) {
                record_directed_param(is_in, attr, &mut in_params, &mut out_params);
            } else if trimmed.starts_with("perform") && is_word_boundary_after(trimmed, 7) {
                let perf = trimmed
                    .get(7..)
                    .unwrap_or("")
                    .trim()
                    .trim_end_matches(';')
                    .trim();
                if !perf.is_empty() {
                    performer = Some(perf.to_string());
                }
            } else if trimmed.starts_with("step") && is_word_boundary_after(trimmed, 4) {
                let step_name = trimmed
                    .get(4..)
                    .unwrap_or("")
                    .trim()
                    .trim_end_matches(';')
                    .trim();
                if !step_name.is_empty() {
                    steps.push(step_name.to_string());
                }
            }
        }
    }

    ActionDef {
        name: span.name.to_string(),
        doc: span.doc,
        in_params,
        out_params,
        performer,
        steps,
        is_def: span.is_def,
        ..Default::default()
    }
}

/// Parses behavioral `ActionDef` definitions and action usages from SysML v2 source text.
///
/// Safety & Traceability: Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-ACTION-DEFS]
/// Contract: Extracts every `action def` and `action` usage in appearance order, preserving `doc /* ... */` blocks, signature and body `in`/`out` parameters, `perform` performer bindings, and `step` declarations while ignoring action tokens inside comments or string literals.
/// Bounds & Faults: Executes in O(N) time and O(N) space over `content.len()` using bounds-checked `.get` slicing and saturating offset arithmetic; malformed or unclosed declarations are skipped safely without panicking.
pub fn parse_action_defs(content: &str) -> Vec<ActionDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut actions = Vec::new();
    let mut idx: usize = 0;

    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("action")) {
        let act_pos = idx.saturating_add(pos);
        let after_act = act_pos.saturating_add(6);
        if !is_word_boundary_before(&masked, act_pos) || !is_word_boundary_after(&masked, after_act)
        {
            idx = after_act;
            continue;
        }
        if let Some(span) = parse_action_span(&clean, &masked, act_pos) {
            idx = span.next_idx;
            actions.push(build_action_def_from_span(span));
        } else {
            idx = after_act;
        }
    }
    actions
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies `ActionDef::default` and serde JSON round-trip defaults (`is_def == true`).
    #[test]
    fn action_def_default_and_serde_roundtrip() -> Result<(), serde_json::Error> {
        let default_act = ActionDef::default();
        assert!(default_act.is_def);
        assert!(default_act.name.is_empty());
        assert!(default_act.in_params.is_empty());
        assert!(default_act.out_params.is_empty());
        assert!(default_act.steps.is_empty());

        let decoded: ActionDef = serde_json::from_str(r#"{"name":"ComputeGuidance"}"#)?;
        assert_eq!(decoded.name, "ComputeGuidance");
        assert!(decoded.is_def);

        let encoded = serde_json::to_string(&decoded)?;
        let roundtrip: ActionDef = serde_json::from_str(&encoded)?;
        assert_eq!(roundtrip, decoded);
        Ok(())
    }

    /// Verifies `parse_action_defs` across `action def`, `action` usages, `item` parameters, steps, and malformed inputs.
    #[test]
    fn parse_action_defs_extracts_params_performers_steps_and_skips_malformed(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let src = r#"
            /* Ignore action def CommentedAction; and multi-byte UTF-8: Δt >= 0 */
            attribute literalNote : String = "action def FakeInString;";

            doc /* Primary flight envelope action */
            action def ExecuteEnvelopeCheck(in item airspeed : ScalarValues::Real, out status : FlagType) {
                in attribute altitude_m : Real;
                out item alert_frame : AlertPacket;
                perform FlightEnvelopeMonitor;
                step sampleSensors;
                step evaluateEnvelope;
            }

            action triggerWatchdog;

            // Malformed declarations must be skipped without panicking
            action def ;
            action BrokenUnclosedParen(in x : Real;
            action BrokenNoTerminator
            action BrokenUnclosedBrace {
        "#;

        let actions = parse_action_defs(src);
        assert_eq!(actions.len(), 2);

        let a0 = actions.first().ok_or("missing ExecuteEnvelopeCheck")?;
        assert_eq!(a0.name, "ExecuteEnvelopeCheck");
        assert!(a0.is_def);
        assert_eq!(a0.doc.as_deref(), Some("Primary flight envelope action"));
        assert_eq!(a0.in_params.len(), 2);
        assert_eq!(
            a0.in_params
                .first()
                .map(|p| (p.name.as_str(), p.type_name.as_str())),
            Some(("airspeed", "ScalarValues::Real"))
        );
        assert_eq!(
            a0.in_params
                .get(1)
                .map(|p| (p.name.as_str(), p.type_name.as_str())),
            Some(("altitude_m", "Real"))
        );
        assert_eq!(a0.out_params.len(), 2);
        assert_eq!(
            a0.out_params
                .first()
                .map(|p| (p.name.as_str(), p.type_name.as_str())),
            Some(("status", "FlagType"))
        );
        assert_eq!(
            a0.out_params
                .get(1)
                .map(|p| (p.name.as_str(), p.type_name.as_str())),
            Some(("alert_frame", "AlertPacket"))
        );
        assert_eq!(a0.performer.as_deref(), Some("FlightEnvelopeMonitor"));
        assert_eq!(a0.steps, vec!["sampleSensors", "evaluateEnvelope"]);

        let a1 = actions.get(1).ok_or("missing triggerWatchdog")?;
        assert_eq!(a1.name, "triggerWatchdog");
        assert!(!a1.is_def);
        assert!(a1.doc.is_none());
        Ok(())
    }
}
