//! Mermaid state, sequence, flowchart, ER, XY, and generic diagram parsers.
//! Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]

use std::collections::BTreeSet;

use crate::spec_audit::diagram_ast::{
    ErDiagramAst, FlowEdgeAst, FlowNodeAst, FlowchartAst, GenericDiagramAst,
    SequenceActivationAst, SequenceDiagramAst, SequenceMessageAst, SequenceParticipantAst,
    StateDefAst, StateDiagramAst, StateNoteAst, StateTransitionAst, XyChartAst,
};
use crate::spec_audit::diagram_invariants::{strip_mermaid_comment, ARROWS};
use crate::spec_audit::markdown_ast::MermaidBlock;
use crate::spec_audit::SpecAuditError;

/// Parses a Mermaid `stateDiagram` / `stateDiagram-v2` block into [`StateDiagramAst`].
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-STATE-DIAGRAM]
pub(crate) fn parse_state_diagram(block: &MermaidBlock) -> Result<StateDiagramAst, SpecAuditError> {
    let mut states_set: BTreeSet<String> = BTreeSet::new();
    let mut states = Vec::new();
    let mut transitions = Vec::new();
    let mut notes = Vec::new();

    for (idx, line) in block.raw.lines().enumerate() {
        let line_num = block.start_line + idx;
        let clean = strip_mermaid_comment(line).trim();
        if clean.is_empty()
            || clean.eq_ignore_ascii_case("stateDiagram")
            || clean.eq_ignore_ascii_case("stateDiagram-v2")
            || clean == "}"
        {
            continue;
        }
        if let Some(after_note_raw) = clean.strip_prefix("note ") {
            let after_note = after_note_raw.trim();
            let (target_state, text) =
                if let Some((dir_and_target, note_body)) = after_note.split_once(':') {
                    let target = dir_and_target.split_whitespace().last().unwrap_or("").to_string();
                    (Some(target), note_body.trim().to_string())
                } else {
                    (None, after_note.to_string())
                };
            notes.push(StateNoteAst { target_state, text, line: line_num });
            continue;
        }
        if let Some(after_state_raw) = clean.strip_prefix("state ") {
            let after_state = after_state_raw.trim().trim_end_matches('{').trim();
            let is_comp = clean.ends_with('{');
            let (id, label) = if let Some((label_part, id_part)) = after_state.split_once(" as ") {
                (id_part.trim().to_string(), Some(label_part.trim().trim_matches('"').to_string()))
            } else {
                (after_state.to_string(), None)
            };
            if !id.is_empty() && states_set.insert(id.clone()) {
                states.push(StateDefAst { id, label, is_composite: is_comp, sub_states: Vec::new(), line: line_num });
            }
            continue;
        }
        if let Some((left, right_full)) = clean.split_once("-->") {
            let source = left.trim().to_string();
            let (target, label_opt) = if let Some((t, l)) = right_full.split_once(':') {
                (t.trim().to_string(), Some(l.trim().to_string()))
            } else {
                (right_full.trim().to_string(), None)
            };
            for st in [&source, &target] {
                if st != "[*]" && !st.is_empty() && states_set.insert(st.clone()) {
                    states.push(StateDefAst { id: st.clone(), label: None, is_composite: false, sub_states: Vec::new(), line: line_num });
                }
            }
            let (event, guard, action) = parse_state_transition_label(label_opt.as_deref());
            transitions.push(StateTransitionAst { source, target, event, guard, action, raw_label: label_opt, line: line_num });
        }
    }

    Ok(StateDiagramAst { states, transitions, notes })
}

/// Extracts `(event, guard, action)` components from a state transition label.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-STATE-DIAGRAM]
fn parse_state_transition_label(
    label_opt: Option<&str>,
) -> (Option<String>, Option<String>, Option<String>) {
    let Some(raw_label) = label_opt else {
        return (None, None, None);
    };
    let unquoted = raw_label.trim_matches('"').trim();
    let mut guard = None;
    let mut label_without_guard = unquoted.to_string();
    if let Some(open_b) = unquoted.find('[') {
        if let Some(close_b) = unquoted[open_b..].find(']') {
            let actual_close = open_b + close_b;
            guard = Some(unquoted[open_b + 1..actual_close].trim().to_string());
            label_without_guard =
                format!("{}{}", &unquoted[..open_b], &unquoted[actual_close + 1..]);
        }
    }
    let mut event = None;
    let mut action = None;
    if let Some((evt_part, act_part)) = label_without_guard.split_once('/') {
        let a = act_part.trim().to_string();
        if !a.is_empty() {
            action = Some(a);
        }
        let e = evt_part.trim().to_string();
        if !e.is_empty() {
            event = Some(e);
        }
    } else {
        let e = label_without_guard.trim().to_string();
        if !e.is_empty() {
            event = Some(e);
        }
    }
    (event, guard, action)
}

/// Parses a Mermaid `sequenceDiagram` block into [`SequenceDiagramAst`].
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-SEQUENCE-DIAGRAM]
pub(crate) fn parse_sequence_diagram(
    block: &MermaidBlock,
) -> Result<SequenceDiagramAst, SpecAuditError> {
    let mut participants: Vec<SequenceParticipantAst> = Vec::new();
    let mut messages: Vec<SequenceMessageAst> = Vec::new();
    let mut activations: Vec<SequenceActivationAst> = Vec::new();

    for (idx, line) in block.raw.lines().enumerate() {
        let line_num = block.start_line + idx;
        let clean = strip_mermaid_comment(line).trim();
        if clean.is_empty()
            || clean.eq_ignore_ascii_case("sequenceDiagram")
            || clean.eq_ignore_ascii_case("autonumber")
        {
            continue;
        }
        if clean.starts_with("participant ") || clean.starts_with("actor ") {
            let is_actor = clean.starts_with("actor ");
            let rest = if is_actor {
                clean.strip_prefix("actor ").unwrap_or("")
            } else {
                clean.strip_prefix("participant ").unwrap_or("")
            }
            .trim();
            let (name, alias) = if let Some((name_part, alias_part)) = rest.split_once(" as ") {
                (name_part.trim().to_string(), Some(alias_part.trim().trim_matches('"').to_string()))
            } else {
                (rest.to_string(), None)
            };
            participants.push(SequenceParticipantAst { name, alias, is_actor, line: line_num });
            continue;
        }
        if let Some(part_raw) = clean.strip_prefix("activate ") {
            activations.push(SequenceActivationAst {
                participant: part_raw.trim().to_string(),
                is_activate: true,
                line: line_num,
            });
            continue;
        }
        if let Some(part_raw) = clean.strip_prefix("deactivate ") {
            activations.push(SequenceActivationAst {
                participant: part_raw.trim().to_string(),
                is_activate: false,
                line: line_num,
            });
            continue;
        }
        parse_sequence_message_line(clean, line_num, &mut participants, &mut messages);
    }

    Ok(SequenceDiagramAst { participants, messages, activations })
}

/// Parses a single sequence message line (`A ->> B : msg()`) if present.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-SEQUENCE-DIAGRAM]
fn parse_sequence_message_line(
    clean: &str,
    line_num: usize,
    participants: &mut Vec<SequenceParticipantAst>,
    messages: &mut Vec<SequenceMessageAst>,
) {
    const SEQ_ARROWS: &[&str] = &[
        "-->>+", "-->>-", "-->>", "->>+", "->>-", "->>", "-->+", "-->-", "-->", "->+", "->-", "->",
        "--x", "-x", "--)", "-)",
    ];
    for &arrow in SEQ_ARROWS {
        if let Some(pos) = clean.find(arrow) {
            let from = clean[..pos].trim().to_string();
            let right_full = clean[pos + arrow.len()..].trim();
            let (to, msg_text) = if let Some((target_part, text)) = right_full.split_once(':') {
                (target_part.trim().to_string(), text.trim().to_string())
            } else {
                (right_full.to_string(), String::new())
            };
            let is_reply = arrow.starts_with("--");
            let mut op_name = None;
            let mut params = Vec::new();
            let unquoted_msg = msg_text.trim_matches('"').trim();
            if let Some(open_p) = unquoted_msg.find('(') {
                if let Some(close_p) = unquoted_msg[open_p..].find(')') {
                    let actual_close = open_p + close_p;
                    let name_candidate = unquoted_msg[..open_p].trim();
                    if !name_candidate.is_empty() {
                        op_name = Some(name_candidate.to_string());
                    }
                    let inside = unquoted_msg[open_p + 1..actual_close].trim();
                    if !inside.is_empty() {
                        for p in inside.split(',') {
                            let trimmed_p = p.trim();
                            if !trimmed_p.is_empty() {
                                params.push(trimmed_p.to_string());
                            }
                        }
                    }
                }
            } else if !unquoted_msg.is_empty() {
                let first_word = unquoted_msg.split_whitespace().next().unwrap_or("");
                if !first_word.is_empty() {
                    op_name = Some(first_word.to_string());
                }
            }

            for party in [&from, &to] {
                if !party.is_empty() && !participants.iter().any(|p| p.name == *party) {
                    participants.push(SequenceParticipantAst { name: party.clone(), alias: None, is_actor: false, line: line_num });
                }
            }
            messages.push(SequenceMessageAst {
                from, to, arrow: arrow.to_string(), message: msg_text,
                operation_name: op_name, parameters: params, is_reply, line: line_num,
            });
            break;
        }
    }
}

/// Parses a Mermaid `flowchart` or `graph` block into [`FlowchartAst`].
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-FLOWCHART]
pub(crate) fn parse_flowchart(block: &MermaidBlock) -> Result<FlowchartAst, SpecAuditError> {
    let mut direction = None;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut node_ids: BTreeSet<String> = BTreeSet::new();

    for (idx, line) in block.raw.lines().enumerate() {
        let line_num = block.start_line + idx;
        let clean = strip_mermaid_comment(line).trim();
        if clean.is_empty() {
            continue;
        }
        if clean.starts_with("flowchart ") || clean.starts_with("graph ") {
            let dir = clean.split_whitespace().nth(1).unwrap_or("TD").to_string();
            direction = Some(dir);
            continue;
        }
        for &arrow in ARROWS {
            if let Some(pos) = clean.find(arrow) {
                let from = clean.get(..pos).unwrap_or("").trim();
                let after_arrow = clean.get(pos + arrow.len()..).unwrap_or("").trim();
                let (label, to) = if let Some(after_pipe) = after_arrow.strip_prefix('|') {
                    if let Some(second_pipe) = after_pipe.find('|') {
                        let lbl = after_pipe.get(..second_pipe).unwrap_or("").trim();
                        let target = after_pipe.get(second_pipe + 1..).unwrap_or("").trim();
                        (Some(lbl.trim_matches('"').to_string()), target)
                    } else {
                        (None, after_arrow)
                    }
                } else {
                    (None, after_arrow)
                };
                let from_id = extract_node_id(from);
                let to_id = extract_node_id(to);
                if node_ids.insert(from_id.clone()) {
                    nodes.push(FlowNodeAst { id: from_id.clone(), label: extract_node_label(from), shape: None, line: line_num });
                }
                if node_ids.insert(to_id.clone()) {
                    nodes.push(FlowNodeAst { id: to_id.clone(), label: extract_node_label(to), shape: None, line: line_num });
                }
                edges.push(FlowEdgeAst { from: from_id, to: to_id, arrow: arrow.to_string(), label, line: line_num });
                break;
            }
        }
    }

    Ok(FlowchartAst { direction, nodes, edges })
}

/// Extracts the bare node identifier preceding `[`, `(`, `{`, or `>`.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-EXTRACT-NODE-ID]
fn extract_node_id(s: &str) -> String {
    let delimiters = ['[', '(', '{', '>'];
    if let Some(idx) = s.find(|c| delimiters.contains(&c)) {
        s[..idx].trim().to_string()
    } else {
        s.trim().to_string()
    }
}

/// Extracts the inner label text from a flowchart node definition.
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-EXTRACT-NODE-LABEL]
fn extract_node_label(s: &str) -> Option<String> {
    let start_delims = ['[', '(', '{'];
    if let Some(start) = s.find(|c| start_delims.contains(&c)) {
        let close_delims = [']', ')', '}'];
        if let Some(end) = s.rfind(|c| close_delims.contains(&c)) {
            if end > start {
                let inner = s[start + 1..end].trim().trim_matches('"');
                return Some(inner.to_string());
            }
        }
    }
    None
}

/// Parses a Mermaid `erDiagram` block into [`ErDiagramAst`].
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-ER-DIAGRAM]
pub(crate) fn parse_er_diagram(block: &MermaidBlock) -> Result<ErDiagramAst, SpecAuditError> {
    let mut entities = Vec::new();
    let mut relationships = Vec::new();
    for line in block.raw.lines() {
        let clean = strip_mermaid_comment(line).trim();
        if clean.is_empty() || clean.eq_ignore_ascii_case("erDiagram") {
            continue;
        }
        if clean.contains("||") || clean.contains("}|") || clean.contains("|{") {
            relationships.push(clean.to_string());
        } else {
            let entity = clean.split_whitespace().next().unwrap_or("").to_string();
            if !entity.is_empty() && !entities.contains(&entity) {
                entities.push(entity);
            }
        }
    }
    Ok(ErDiagramAst { entities, relationships })
}

/// Parses a Mermaid `xychart-beta` block into [`XyChartAst`].
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-XYCHART]
pub(crate) fn parse_xychart(block: &MermaidBlock) -> Result<XyChartAst, SpecAuditError> {
    let mut title = None;
    let mut x_axis = None;
    let mut y_axis = None;
    for line in block.raw.lines() {
        let clean = strip_mermaid_comment(line).trim();
        if let Some(rest) = clean.strip_prefix("title ") {
            title = Some(rest.trim().trim_matches('"').to_string());
        } else if let Some(rest) = clean.strip_prefix("x-axis ") {
            x_axis = Some(rest.trim().to_string());
        } else if let Some(rest) = clean.strip_prefix("y-axis ") {
            y_axis = Some(rest.trim().to_string());
        }
    }
    Ok(XyChartAst { title, x_axis, y_axis })
}

/// Parses a generic Mermaid block into [`GenericDiagramAst`].
/// Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST-PARSE-GENERIC-DIAGRAM]
pub(crate) fn parse_generic_diagram(
    block: &MermaidBlock,
) -> Result<GenericDiagramAst, SpecAuditError> {
    let mut lines = Vec::new();
    for line in block.raw.lines() {
        let clean = strip_mermaid_comment(line).trim();
        if !clean.is_empty() {
            lines.push(clean.to_string());
        }
    }
    Ok(GenericDiagramAst { header: block.header.clone(), lines })
}
