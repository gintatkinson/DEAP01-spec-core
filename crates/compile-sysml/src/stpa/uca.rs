//! STPA Unsafe Control Action (UCA) Cartesian Expansion Engine.
//!
//! Generates Cartesian products of controlling part-def control actions across
//! the 4 universal STPA guide words.

use deap_core::sysml_ast::PackageDef;
use serde::{Deserialize, Serialize};

/// The 4 canonical STPA guide-word categories.
pub const STPA_GUIDE_WORDS: [&str; 4] = [
    "Not providing",
    "Providing",
    "Too early / Too late / Out of order",
    "Stopped too soon / Applied too long",
];

pub const PENDING_PARAMETER: &str = "PENDING_PARAMETER";

/// Representation of a single Unsafe Control Action derived from an action and guide word.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnsafeControlAction {
    pub id: String,
    pub controller: String,
    pub control_action: String,
    pub guide_word: String,
    pub context: String,
    pub hazard: String,
    pub constraint: String,
    pub severity: String,
    pub sail: String,
}

/// Expands the dynamic Cartesian UCA matrix as the union over controlling part defs
/// of |Actions(controller)| x 4 STPA guide words.
pub fn expand_cartesian_stpa(pkg: &PackageDef) -> Vec<UnsafeControlAction> {
    let mut ucas = Vec::new();
    let all_parts = pkg.get_all_parts();
    let controllers: Vec<_> = all_parts
        .iter()
        .filter(|part| !part.actions.is_empty())
        .collect();

    let mut uca_counter = 0;
    let mut action_counter = 0;

    for controller in &controllers {
        for action in &controller.actions {
            action_counter += 1;
            for guide_word in &STPA_GUIDE_WORDS {
                uca_counter += 1;
                let action_name = &action.name;
                ucas.push(UnsafeControlAction {
                    id: format!("UCA-{:03}", uca_counter),
                    controller: controller.name.clone(),
                    control_action: action_name.clone(),
                    guide_word: (*guide_word).to_string(),
                    context: format!("Context for {} under {}", action_name, guide_word),
                    hazard: format!("H-{}", action_counter),
                    constraint: format!("SC-{:03}", uca_counter),
                    severity: PENDING_PARAMETER.to_string(),
                    sail: PENDING_PARAMETER.to_string(),
                });
            }
        }
    }

    // Also check actions declared at package level (or across recursive subpackages) if no controller has actions
    if ucas.is_empty() {
        let all_actions = pkg.get_all_actions();
        if !all_actions.is_empty() {
            let default_controller = all_parts
                .iter()
                .find(|p| p.name.contains("System") || p.name.ends_with("Engine"))
                .map(|p| p.name.clone())
                .unwrap_or_else(|| {
                    if pkg.name == "SysML_Model" {
                        "DEAPCompilerSystem".to_string()
                    } else {
                        pkg.name.clone()
                    }
                });

            for action in &all_actions {
                action_counter += 1;
                for guide_word in &STPA_GUIDE_WORDS {
                    uca_counter += 1;
                    let action_name = &action.name;
                    ucas.push(UnsafeControlAction {
                        id: format!("UCA-{:03}", uca_counter),
                        controller: default_controller.clone(),
                        control_action: action_name.clone(),
                        guide_word: (*guide_word).to_string(),
                        context: format!("Context for {} under {}", action_name, guide_word),
                        hazard: format!("H-{}", action_counter),
                        constraint: format!("SC-{:03}", uca_counter),
                        severity: PENDING_PARAMETER.to_string(),
                        sail: PENDING_PARAMETER.to_string(),
                    });
                }
            }
        }
    }

    ucas
}

#[cfg(test)]
mod tests {
    use super::*;
    use deap_core::sysml_ast::{ActionDef, PartDef};

    #[test]
    fn test_uca_cartesian_expansion_all_guide_words() {
        let mut pkg = PackageDef::default();
        pkg.name = "TestPackage".to_string();

        let mut controller = PartDef::default();
        controller.name = "FlightComputer".to_string();
        let mut act1 = ActionDef::default();
        act1.name = "DeployParachute".to_string();
        controller.actions.push(act1);

        let mut act2 = ActionDef::default();
        act2.name = "IgniteRocket".to_string();
        controller.actions.push(act2);
        pkg.part_defs.push(controller);

        let ucas = expand_cartesian_stpa(&pkg);
        // 2 actions * 4 guide words = 8 UCAs
        assert_eq!(ucas.len(), 8);
        assert_eq!(ucas[0].id, "UCA-001");
        assert_eq!(ucas[0].controller, "FlightComputer");
        assert_eq!(ucas[0].control_action, "DeployParachute");
        assert_eq!(ucas[0].guide_word, "Not providing");
        assert_eq!(ucas[0].constraint, "SC-001");
        assert_eq!(ucas[0].hazard, "H-1");

        assert_eq!(ucas[1].guide_word, "Providing");
        assert_eq!(ucas[2].guide_word, "Too early / Too late / Out of order");
        assert_eq!(ucas[3].guide_word, "Stopped too soon / Applied too long");

        assert_eq!(ucas[4].control_action, "IgniteRocket");
        assert_eq!(ucas[4].hazard, "H-2");
    }
}
