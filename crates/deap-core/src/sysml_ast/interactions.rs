//! Behavioral interaction sequence and system use-case AST definitions for SysML v2.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-INTERACTIONS], [DO-330-TQL-4-SYSML-AST]
//! Contract: Defines `InteractionDef` (lifelines, ordered message flows, and state transition triggers) and `UseCaseDef` (system subject, primary and secondary actor bindings, objective, `include`/`extend` relationships, scenario steps, preconditions, and postconditions) for behavioral modeling.
//! Bounds & Faults: All structural AST records construct and serialize deterministically in linear O(M) time and space over declared member vectors with infallible `Default` and `serde` wire-default semantics.

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies `InteractionDef` defaults, omitted-vector JSON deserialization, and lifeline/message/trigger round-trip fidelity.
    #[test]
    fn interaction_def_default_and_serde_roundtrip() -> Result<(), serde_json::Error> {
        let empty = InteractionDef::default();
        assert!(empty.name.is_empty());
        assert!(empty.lifelines.is_empty());
        assert!(empty.messages.is_empty());
        assert!(empty.triggers.is_empty());
        assert!(empty.doc.is_none());

        let minimal: InteractionDef =
            serde_json::from_str(r#"{"name":"TelemetryHandshake","doc":null}"#)?;
        assert_eq!(minimal.name, "TelemetryHandshake");
        assert!(minimal.lifelines.is_empty());
        assert!(minimal.messages.is_empty());
        assert!(minimal.triggers.is_empty());

        let populated = InteractionDef {
            name: "ModeTransitionSequence".to_string(),
            lifelines: vec!["FlightDirector".to_string(), "ActuatorNode".to_string()],
            messages: vec!["arm_actuators".to_string(), "reply_arm_ack".to_string()],
            triggers: vec!["on_arm_confirmed".to_string()],
            doc: Some("Nominal arming sequence between flight director and actuator.".to_string()),
        };
        let encoded = serde_json::to_string(&populated)?;
        let decoded: InteractionDef = serde_json::from_str(&encoded)?;
        assert_eq!(decoded, populated);
        Ok(())
    }

    /// Verifies `UseCaseDef` defaults, omitted-collection JSON deserialization, and actor/scenario clause round-trip fidelity.
    #[test]
    fn use_case_def_actor_bindings_and_clauses_roundtrip() -> Result<(), serde_json::Error> {
        let empty = UseCaseDef::default();
        assert!(empty.name.is_empty());
        assert!(empty.subject.is_none());
        assert!(empty.actor.is_none());
        assert!(empty.actors.is_empty());
        assert!(empty.includes.is_empty());
        assert!(empty.extends.is_empty());
        assert!(empty.steps.is_empty());
        assert!(empty.preconditions.is_empty());
        assert!(empty.postconditions.is_empty());

        let minimal: UseCaseDef = serde_json::from_str(
            r#"{"name":"UC_01_ExecuteGuidance","doc":null,"subject":null,"actor":null,"objective":null}"#,
        )?;
        assert_eq!(minimal.name, "UC_01_ExecuteGuidance");
        assert!(minimal.actors.is_empty());
        assert!(minimal.steps.is_empty());

        let populated = UseCaseDef {
            name: "UC_02_AbortAndSafeLatch".to_string(),
            doc: Some("Executes deterministic abort and actuator safe-latch.".to_string()),
            subject: Some("FlightControlSystem".to_string()),
            actor: Some("SafetyMonitor".to_string()),
            actors: vec!["SafetyMonitor".to_string(), "FlightCrew".to_string()],
            objective: Some("Transition actuators to safe-latched state within 5 ms.".to_string()),
            includes: vec!["UC_LogFaultFrame".to_string()],
            extends: vec!["UC_NominalFlightLoop".to_string()],
            steps: vec![
                "Detect envelope exceedance".to_string(),
                "Command safe latch".to_string(),
            ],
            preconditions: vec!["System power bus nominal".to_string()],
            postconditions: vec!["Actuators latched and fault telemetry recorded".to_string()],
        };
        let encoded = serde_json::to_string(&populated)?;
        let decoded: UseCaseDef = serde_json::from_str(&encoded)?;
        assert_eq!(decoded, populated);
        Ok(())
    }
}
