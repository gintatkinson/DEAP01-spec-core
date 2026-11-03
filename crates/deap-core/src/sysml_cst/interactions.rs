//! Behavioral interaction sequence, system use-case, and verification test-case AST definitions and parsers for SysML v2.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-INTERACTIONS], [DO-330-TQL-4-SYSML-AST]
//! Contract: Defines `InteractionDef`, `UseCaseDef`, `TestCaseDef`, and `parse_test_case_defs` for behavioral and verification modeling.
//! Bounds & Faults: All structural AST records and parsers execute deterministically in linear O(N) time and space over input size with zero panics.

use super::identifiers::{
    extract_doc_comment_before, extract_ident, find_matching_brace, is_word_boundary_after,
    is_word_boundary_before, mask_string_literals, skip_ws,
};
use super::model_parser::strip_sysml_comments;
use serde::{Deserialize, Serialize};

/// Represents a behavioral interaction sequence of lifelines, message flows, and triggers in SysML v2.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-INTERACTIONDEF]
/// Contract: Captures the interaction `name`, participating component `lifelines`, ordered `messages` exchanged across lifelines, associated state-machine `triggers`, and optional `doc` metadata.
/// Bounds & Faults: O(1) default construction and O(L + M + T) clone/serialization cost over lifeline, message, and trigger counts with deterministic `serde` empty-vector defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct InteractionDef {
    pub name: String,
    #[serde(default)]
    pub lifelines: Vec<String>,
    #[serde(default)]
    pub messages: Vec<String>,
    #[serde(default)]
    pub triggers: Vec<String>,
    pub doc: Option<String>,
}

/// Represents a behavioral system use-case definition and actor realization contract in SysML v2.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-SYSML-AST-USECASEDEF]
/// Contract: Preserves use-case `name`, optional `doc` block, system `subject`, primary `actor`, complete `actors` binding list, `objective`, `includes` and `extends` use-case references, ordered scenario `steps`, `preconditions`, and `postconditions`.
/// Bounds & Faults: O(1) default construction and linear O(A + S + C) clone/serialization cost over actor, step, and condition vectors with deterministic `serde` defaults when optional collections are omitted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UseCaseDef {
    pub name: String,
    pub doc: Option<String>,
    pub subject: Option<String>,
    pub actor: Option<String>,
    #[serde(default)]
    pub actors: Vec<String>,
    pub objective: Option<String>,
    #[serde(default)]
    pub includes: Vec<String>,
    #[serde(default)]
    pub extends: Vec<String>,
    #[serde(default)]
    pub steps: Vec<String>,
    #[serde(default)]
    pub preconditions: Vec<String>,
    #[serde(default)]
    pub postconditions: Vec<String>,
}

/// Represents a verification test case definition in SysML v2.
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C.
/// Realises: [REQ-DEAP-CORE-SYSML-AST-TESTCASEDEF] | SysML v2: PartDef(TestCaseDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TestCaseDef {
    pub name: String,
    pub subject_part: Option<String>,
    #[serde(default)]
    pub verified_requirements: Vec<String>,
    pub objective: Option<String>,
    #[serde(default)]
    pub test_steps: Vec<String>,
    pub doc: Option<String>,
}

/// Parsed elements extracted from a test case body block.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-TEST-CASE-DEFS]
#[derive(Default)]
struct TestCaseBodyContent {
    subject_part: Option<String>,
    verified_requirements: Vec<String>,
    objective: Option<String>,
    test_steps: Vec<String>,
}

/// Parses the inner body statements of a test case block.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-TEST-CASE-DEFS]
fn parse_test_case_body(body: &str) -> TestCaseBodyContent {
    let mut content = TestCaseBodyContent::default();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("subject") && is_word_boundary_after(trimmed, 7) {
            let val = trimmed.get(7..).unwrap_or("").trim().trim_end_matches(';').trim();
            if !val.is_empty() {
                content.subject_part = Some(val.to_string());
            }
        } else if trimmed.starts_with("verify") && is_word_boundary_after(trimmed, 6) {
            let mut rest = trimmed.get(6..).unwrap_or("").trim();
            if rest.starts_with("assert") && is_word_boundary_after(rest, 6) {
                rest = rest.get(6..).unwrap_or("").trim();
            }
            if rest.starts_with("requirement") && is_word_boundary_after(rest, 11) {
                rest = rest.get(11..).unwrap_or("").trim();
            } else if rest.starts_with("constraint") && is_word_boundary_after(rest, 10) {
                rest = rest.get(10..).unwrap_or("").trim();
            }
            for target in rest.trim_end_matches(';').trim().split(',') {
                let t = target.trim();
                if !t.is_empty() {
                    content.verified_requirements.push(t.to_string());
                }
            }
        } else if trimmed.starts_with("objective") && is_word_boundary_after(trimmed, 9) {
            let val = trimmed.get(9..).unwrap_or("").trim().trim_end_matches(';').trim().trim_matches('"');
            if !val.is_empty() {
                content.objective = Some(val.to_string());
            }
        } else if trimmed.starts_with("step") && is_word_boundary_after(trimmed, 4) {
            let val = trimmed.get(4..).unwrap_or("").trim().trim_end_matches(';').trim();
            if !val.is_empty() {
                content.test_steps.push(val.to_string());
            }
        }
    }
    content
}

/// Parse test case definitions from SysML v2 source text.
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-TEST-CASE-DEFS]
pub fn parse_test_case_defs(content: &str) -> Vec<TestCaseDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut test_cases = Vec::new();
    let mut idx = 0;
    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("test")) {
        let test_pos = idx + pos;
        if !is_word_boundary_before(&masked, test_pos)
            || !is_word_boundary_after(&masked, test_pos + 4)
        {
            idx = test_pos + 4;
            continue;
        }
        let doc = extract_doc_comment_before(&clean, test_pos);
        let mut curr = test_pos + 4;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if !masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("case") && is_word_boundary_after(s, 4))
        {
            idx = test_pos + 4;
            continue;
        }
        curr += 4;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = test_pos + 4;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));

        let rem_body = masked.get(curr..).unwrap_or("");
        let (body_content, next_idx) = if rem_body.starts_with('{') {
            if let Some(close_pos) = find_matching_brace(&clean, curr) {
                let body = clean.get(curr + 1..close_pos).unwrap_or("");
                (parse_test_case_body(body), close_pos + 1)
            } else {
                idx = test_pos + 4;
                continue;
            }
        } else if rem_body.starts_with(';') {
            (TestCaseBodyContent::default(), curr + 1)
        } else {
            idx = test_pos + 4;
            continue;
        };

        idx = next_idx;
        test_cases.push(TestCaseDef {
            name: name.to_string(),
            subject_part: body_content.subject_part,
            verified_requirements: body_content.verified_requirements,
            objective: body_content.objective,
            test_steps: body_content.test_steps,
            doc,
        });
    }
    test_cases
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies `InteractionDef` defaults, omitted-vector JSON deserialization, and lifeline/message/trigger round-trip fidelity.
    #[test]
    fn interaction_def_default_and_serde_roundtrip() -> Result<(), serde_json::Error> {
        let empty = InteractionDef::default();
        assert!(empty.name.is_empty() && empty.lifelines.is_empty() && empty.messages.is_empty() && empty.triggers.is_empty() && empty.doc.is_none());
        let minimal: InteractionDef = serde_json::from_str(r#"{"name":"TelemetryHandshake","doc":null}"#)?;
        assert!(minimal.name == "TelemetryHandshake" && minimal.lifelines.is_empty());
        let populated = InteractionDef {
            name: "ModeTransitionSequence".to_string(),
            lifelines: vec!["FlightDirector".to_string(), "ActuatorNode".to_string()],
            messages: vec!["arm_actuators".to_string(), "reply_arm_ack".to_string()],
            triggers: vec!["on_arm_confirmed".to_string()],
            doc: Some("Nominal arming sequence between flight director and actuator.".to_string()),
        };
        assert_eq!(serde_json::from_str::<InteractionDef>(&serde_json::to_string(&populated)?)?, populated);
        Ok(())
    }

    /// Verifies `UseCaseDef` defaults, omitted-collection JSON deserialization, and actor/scenario clause round-trip fidelity.
    #[test]
    fn use_case_def_actor_bindings_and_clauses_roundtrip() -> Result<(), serde_json::Error> {
        let empty = UseCaseDef::default();
        assert!(empty.name.is_empty() && empty.subject.is_none() && empty.actors.is_empty() && empty.steps.is_empty());
        let minimal: UseCaseDef = serde_json::from_str(r#"{"name":"UC_01_ExecuteGuidance","doc":null,"subject":null,"actor":null,"objective":null}"#)?;
        assert!(minimal.name == "UC_01_ExecuteGuidance" && minimal.actors.is_empty());
        let populated = UseCaseDef {
            name: "UC_02_AbortAndSafeLatch".to_string(),
            doc: Some("Executes deterministic abort and actuator safe-latch.".to_string()),
            subject: Some("FlightControlSystem".to_string()),
            actor: Some("SafetyMonitor".to_string()),
            actors: vec!["SafetyMonitor".to_string(), "FlightCrew".to_string()],
            objective: Some("Transition actuators to safe-latched state within 5 ms.".to_string()),
            includes: vec!["UC_LogFaultFrame".to_string()],
            extends: vec!["UC_NominalFlightLoop".to_string()],
            steps: vec!["Detect envelope exceedance".to_string(), "Command safe latch".to_string()],
            preconditions: vec!["System power bus nominal".to_string()],
            postconditions: vec!["Actuators latched and fault telemetry recorded".to_string()],
        };
        assert_eq!(serde_json::from_str::<UseCaseDef>(&serde_json::to_string(&populated)?)?, populated);
        Ok(())
    }
}
