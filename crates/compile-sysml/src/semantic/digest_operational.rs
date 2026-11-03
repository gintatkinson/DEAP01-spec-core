#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Operational activity, exchange, scenario, and node extraction for `SchemaDigest`.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for UAF operational projections.
//! ## 2. Invariants & Mathematical Properties
//! Deterministically assigns sequential `OA-XX`, `OpTx-XX`, and `SCN-XX` identifiers using saturating counters.
//! ## 3. Preconditions
//! Caller supplies a valid borrowed `PackageDef` root AST node.
//! ## 4. Postconditions
//! Returns ordered vectors of operational activities, exchanges, scenarios, and nodes without mutating the AST.
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(V) over AST elements. Space Complexity: O(V) output vectors.
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Total pure extraction with zero panics or unbounded recursion.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603, REQ-SEMANTIC-DIGEST-EXTRACT-OPERATIONAL-ACTIVITIES, REQ-SEMANTIC-DIGEST-EXTRACT-OPERATIONAL-EXCHANGES]

use super::digest::{
    OperationalActivity, OperationalExchange, OperationalNode, OperationalScenario,
};
use deap_core::sysml_ast::PackageDef;

/// Extracts `OperationalActivity` records from package-level and part-level `ActionDef` declarations.
/// Realises: [REQ-SEMANTIC-DIGEST-EXTRACT-OPERATIONAL-ACTIVITIES] | SysML v2: ActionDef(extract_operational_activities)
pub(crate) fn extract_operational_activities(pkg: &PackageDef) -> Vec<OperationalActivity> {
    let mut activities = Vec::new();
    let mut idx: usize = 1;

    for p in pkg.get_all_packages() {
        for act in &p.action_defs {
            activities.push(OperationalActivity {
                id: format!("OA-{:02}", idx),
                name: act.name.clone(),
                doc: act.doc.clone().unwrap_or_default(),
                performer: act.performer.clone().unwrap_or_default(),
                inputs: act
                    .in_params
                    .iter()
                    .map(|param| param.name.clone())
                    .collect(),
                outputs: act
                    .out_params
                    .iter()
                    .map(|param| param.name.clone())
                    .collect(),
            });
            idx = idx.saturating_add(1);
        }
    }

    for part in pkg.get_all_parts() {
        for act in &part.actions {
            activities.push(OperationalActivity {
                id: format!("OA-{:02}", idx),
                name: act.name.clone(),
                doc: act.doc.clone().unwrap_or_default(),
                performer: act.performer.clone().unwrap_or_else(|| part.name.clone()),
                inputs: act
                    .in_params
                    .iter()
                    .map(|param| param.name.clone())
                    .collect(),
                outputs: act
                    .out_params
                    .iter()
                    .map(|param| param.name.clone())
                    .collect(),
            });
            idx = idx.saturating_add(1);
        }
    }

    activities
}

/// Extracts `OperationalExchange` records from package-level and part-level `ConnectionDef` declarations.
/// Realises: [REQ-SEMANTIC-DIGEST-EXTRACT-OPERATIONAL-EXCHANGES] | SysML v2: ActionDef(extract_operational_exchanges)
pub(crate) fn extract_operational_exchanges(pkg: &PackageDef) -> Vec<OperationalExchange> {
    let mut exchanges = Vec::new();
    let mut idx: usize = 1;

    for p in pkg.get_all_packages() {
        for conn in &p.connection_defs {
            exchanges.push(OperationalExchange {
                id: format!("OpTx-{:02}", idx),
                name: conn.name.clone(),
                source_port: conn.source_port.clone(),
                target_port: conn.target_port.clone(),
                source_part: conn.source_part.clone().unwrap_or_default(),
                target_part: conn.target_part.clone().unwrap_or_default(),
                protocol: conn.protocol.clone().unwrap_or_default(),
            });
            idx = idx.saturating_add(1);
        }
    }

    for part in pkg.get_all_parts() {
        for conn in &part.connections {
            exchanges.push(OperationalExchange {
                id: format!("OpTx-{:02}", idx),
                name: conn.name.clone(),
                source_port: conn.source_port.clone(),
                target_port: conn.target_port.clone(),
                source_part: conn
                    .source_part
                    .clone()
                    .unwrap_or_else(|| part.name.clone()),
                target_part: conn.target_part.clone().unwrap_or_default(),
                protocol: conn.protocol.clone().unwrap_or_default(),
            });
            idx = idx.saturating_add(1);
        }
    }

    exchanges
}

/// Extracts `OperationalScenario` records from all `UseCaseDef` declarations in `pkg`.
/// Realises: [REQ-SEMANTIC-DIGEST-EXTRACT-OPERATIONAL-SCENARIOS] | SysML v2: ActionDef(extract_operational_scenarios)
pub(crate) fn extract_operational_scenarios(pkg: &PackageDef) -> Vec<OperationalScenario> {
    let mut scenarios = Vec::new();
    for (idx, uc) in (1..).zip(pkg.get_all_use_cases()) {
        scenarios.push(OperationalScenario {
            id: format!("SCN-{:02}", idx),
            name: uc.name.clone(),
            actor: uc.actor.clone().unwrap_or_default(),
            objective: uc.objective.clone().unwrap_or_default(),
            steps: uc.steps.clone(),
        });
    }
    scenarios
}

/// Extracts `OperationalNode` records from all `PartDef` declarations in `pkg`.
/// Realises: [REQ-SEMANTIC-DIGEST-EXTRACT-OPERATIONAL-NODES] | SysML v2: ActionDef(extract_operational_nodes)
pub(crate) fn extract_operational_nodes(pkg: &PackageDef) -> Vec<OperationalNode> {
    let mut nodes = Vec::new();
    for part in pkg.get_all_parts() {
        nodes.push(OperationalNode {
            name: part.name.clone(),
            node_type: "Part".to_string(),
            doc: part.doc.clone().unwrap_or_default(),
            allocated_activities: part.actions.iter().map(|a| a.name.clone()).collect(),
        });
    }
    nodes
}
