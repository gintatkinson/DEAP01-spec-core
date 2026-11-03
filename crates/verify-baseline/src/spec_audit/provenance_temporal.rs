//! Sequence diagram temporal safety validation for `provenance.rs`.

use crate::spec_audit::markdown_ast::MarkdownDoc;
use crate::spec_audit::provenance::{
    normalize_token, tokenize_words, HITL_SENDER_TOKENS, PHYSICAL_ARMING_TARGET_TOKENS,
};
use crate::spec_audit::SpecAuditFinding;
use std::collections::BTreeMap;

/// Parses a participant or actor declaration in Mermaid sequence diagrams without regex.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-PARSE-PARTICIPANT-DECLARATION]
pub(crate) fn parse_participant_declaration(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    let rest = trimmed
        .strip_prefix("participant ")
        .or_else(|| trimmed.strip_prefix("actor "))?
        .trim();

    if let Some((id, alias_part)) = rest.split_once(" as ") {
        let id_clean = id.trim().trim_matches('"');
        let alias_clean = alias_part.trim().trim_matches('"');
        Some((id_clean.to_string(), alias_clean.to_string()))
    } else {
        let id_clean = rest.trim().trim_matches('"');
        Some((id_clean.to_string(), String::new()))
    }
}

/// Parses a message arrow line in Mermaid sequence diagrams without regex.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-PARSE-MESSAGE-ARROW]
pub(crate) fn parse_message_arrow(line: &str) -> Option<(&str, &str, &str)> {
    let trimmed = line.trim();
    let (left, action) = trimmed.split_once(':')?;
    let left = left.trim();
    let action = action.trim();

    const ARROWS: &[&str] = &["-->>", "-->", "--)", "--x", "->>", "->", "-x", "-)"];

    for arrow in ARROWS {
        if let Some(idx) = left.find(arrow) {
            let sender = left[..idx].trim().trim_matches('"');
            let target = left[idx + arrow.len()..].trim().trim_matches('"');
            if !sender.is_empty() && !target.is_empty() {
                return Some((sender, target, action));
            }
        }
    }
    None
}

/// Determines whether a message sender represents a Human-in-the-Loop (HITL) operator or C2 console.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-HITL-SENDER]
pub(crate) fn is_hitl_sender(sender: &str, participants: &BTreeMap<String, String>) -> bool {
    let s_norm = normalize_token(sender);
    let alias = participants.get(sender).cloned().unwrap_or_default();
    let a_norm = normalize_token(&alias);

    const HITL_SUBSTRINGS: &[&str] = &[
        "operator",
        "pilot",
        "gcs",
        "commander",
        "human",
        "supervisor",
        "safetyofficer",
        "controller",
        "console",
        "technician",
        "user",
        "c2",
    ];

    if HITL_SENDER_TOKENS.contains(&s_norm.as_str())
        || HITL_SENDER_TOKENS.contains(&a_norm.as_str())
    {
        return true;
    }

    HITL_SUBSTRINGS
        .iter()
        .any(|tok| s_norm.contains(tok) || a_norm.contains(tok))
}

/// Determines whether a message action signifies an explicit HITL arming authorization or operator consent.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-HITL-CONSENT-ACTION]
pub(crate) fn is_hitl_consent_action(action: &str) -> bool {
    let words = tokenize_words(action);
    if words.is_empty() {
        return false;
    }

    if words.iter().any(|w| {
        matches!(
            w.as_str(),
            "consent"
                | "consent_granted"
                | "authorize"
                | "authorized"
                | "authorization"
                | "permit"
                | "permission"
                | "enable"
                | "enabled"
        )
    }) {
        return true;
    }

    let has_auth = words
        .iter()
        .any(|w| w == "auth" || w.starts_with("auth") || w == "arm" || w.starts_with("arm"));
    let has_cmd_or_token = words.iter().any(|w| {
        matches!(
            w.as_str(),
            "command"
                | "cmd"
                | "token"
                | "tokens"
                | "key"
                | "keys"
                | "confirm"
                | "switch"
                | "request"
                | "grant"
                | "granted"
        )
    });
    has_auth && has_cmd_or_token
}

/// Determines whether an action signifies a disarm, safe, or inhibit command that resets active consent.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-DISARM-ACTION]
pub(crate) fn is_disarm_action(action: &str) -> bool {
    let words = tokenize_words(action);
    const DISARM_WORDS: &[&str] = &[
        "disarm",
        "disarming",
        "safe",
        "safing",
        "inhibit",
        "inhibiting",
        "abort",
        "aborting",
        "disable",
        "deactivate",
        "isolate",
    ];
    words.iter().any(|w| DISARM_WORDS.contains(&w.as_str()))
}

/// Determines whether a message target represents a physical arming, motor-enable, or actuator device.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-PHYSICAL-ARMING-TARGET]
pub(crate) fn is_physical_arming_target(
    target: &str,
    participants: &BTreeMap<String, String>,
) -> bool {
    let t_norm = normalize_token(target);
    let alias = participants.get(target).cloned().unwrap_or_default();
    let a_norm = normalize_token(&alias);

    PHYSICAL_ARMING_TARGET_TOKENS.iter().any(|tok| {
        t_norm == *tok
            || t_norm.contains(tok)
            || (!a_norm.is_empty() && (a_norm == *tok || a_norm.contains(tok)))
    })
}

/// Determines whether an action signifies a physical actuation, drive, or effector trigger.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-PHYSICAL-ARMING-ACTION]
pub(crate) fn is_physical_arming_action(action: &str) -> bool {
    let words = tokenize_words(action);
    if words.is_empty() {
        return false;
    }

    const ACTUATION_KEYWORDS: &[&str] = &[
        "actuate",
        "actuation",
        "drive",
        "engage",
        "trigger",
        "arm",
        "arming",
    ];

    if words
        .iter()
        .any(|w| ACTUATION_KEYWORDS.contains(&w.as_str()))
    {
        return true;
    }

    let act_norm = normalize_token(action);
    const COMBOS: &[&str] = &[
        "motorenable",
        "enablemotor",
        "propulsionenable",
        "actuatedrive",
        "engagedrive",
        "activatepowerstage",
        "highvoltage",
    ];

    COMBOS.iter().any(|c| act_norm.contains(c))
}

/// Scans Mermaid sequence diagrams and validates that physical arming signals have prior HITL consent.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-VALIDATE-SEQUENCE-DIAGRAM-TEMPORAL-SAFETY]
pub fn validate_sequence_diagram_temporal_safety(
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();

    for block in &doc.mermaid_blocks {
        let is_sequence = block.header.eq_ignore_ascii_case("sequenceDiagram")
            || block
                .raw
                .lines()
                .any(|l| l.trim().starts_with("sequenceDiagram"));
        if !is_sequence {
            continue;
        }

        let mut hitl_consent_granted = false;
        let mut participants = BTreeMap::new();

        for (idx, line) in block.raw.lines().enumerate() {
            let curr_lineno = block.start_line + 1 + idx;
            let trimmed = line.trim();

            if trimmed.is_empty()
                || trimmed.starts_with("%%")
                || trimmed.starts_with("Note ")
                || trimmed.starts_with("Note\t")
                || trimmed.starts_with("autonumber")
                || trimmed.starts_with("title ")
            {
                continue;
            }

            if let Some((id, alias)) = parse_participant_declaration(trimmed) {
                participants.insert(id, alias);
                continue;
            }

            if let Some((sender, target, action)) = parse_message_arrow(trimmed) {
                if is_hitl_sender(sender, &participants) && is_hitl_consent_action(action) {
                    hitl_consent_granted = true;
                }
                if is_disarm_action(action) {
                    hitl_consent_granted = false;
                    continue;
                }
                if is_physical_arming_target(target, &participants)
                    && is_physical_arming_action(action)
                    && !hitl_consent_granted
                {
                    let reported_line = content
                        .lines()
                        .enumerate()
                        .find(|(i, l)| {
                            let lno = i + 1;
                            lno >= block.start_line
                                && lno <= block.end_line
                                && l.trim() == trimmed
                        })
                        .map(|(i, _)| i + 1)
                        .unwrap_or(curr_lineno);

                    findings.push(SpecAuditFinding::new(
                        "factual-grounding-temporal-safety-violation",
                        format!(
                            "Autonomous physical actuation sequence detected in '{}' on line {}. Physical signal '{}' targeting '{}' is not preceded by an explicit Human-in-the-Loop (HITL) authorization command or operator consent.",
                            rel_path, reported_line, action, target
                        ),
                        rel_path,
                        Some(reported_line),
                    ));
                }
            }
        }
    }

    findings
}
