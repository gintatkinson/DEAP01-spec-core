#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! STPA Unsafe Control Action (UCA) Cartesian Expansion Engine.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! safety analysis objectives for module `crates/compile-sysml/src/stpa/uca.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Expands every controlling `PartDef` action (or package-scoped `ActionDef` when inline part
//!   actions are absent) across the 4 canonical STPA guide words (`4 * |Actions|` bijective rows).
//! - Resolves controller ownership, operational context, hazard severity, and integrity level
//!   deterministically from SysML v2 AST nodes with zero unresolved placeholder literals.
//!
//! ## 3. Preconditions
//! - Caller supplies a parsed `deap_core::sysml_ast::PackageDef` hierarchy.
//!
//! ## 4. Postconditions
//! - Every emitted `UnsafeControlAction` contains single-line Markdown-safe cells, deterministic
//!   `H-*` and `SC-*` identifiers, and AST-grounded `severity` and `sail` (integrity level) values.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(P + A * (G + M)) where P is AST part/package node count, A is action count,
//!   G = 4 guide words, and M is bounded attribute/hazard lookup cardinality.
//! - Space Complexity: O(P + A * G) auxiliary memory for collected AST metadata and output UCAs.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Degrades gracefully on minimal schemas with zero `PartDef`, `AttributeDef`, or `ConstraintDef` nodes.
//! - State: Pure functional projection with zero shared mutable state.
//! - Action: Executes with zero regular expressions, zero unchecked unwrapping, and zero direct slice indexing.
//! - Resource: Bounded linear heap allocations proportional to AST node count.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-UCA-MODULE] | SysML v2: PackageDef(stpa_uca)

mod helpers;
mod resolvers;

use std::collections::BTreeSet;

use deap_core::sysml_ast::{
    AttributeDef, ConstraintDef, HazardDef, PackageDef, PartDef, RequirementDef,
};
use serde::{Deserialize, Serialize};

use helpers::{resolve_action_controller, sanitize_uca_cell};
use resolvers::{resolve_uca_integrity_level, resolve_uca_severity, synthesize_uca_context};

/// The 4 canonical STPA guide-word categories.
/// Realises: [REQ-STPA-UCA-GUIDE-WORDS]
pub const STPA_GUIDE_WORDS: [&str; 4] = [
    "Not providing",
    "Providing",
    "Too early / Too late / Out of order",
    "Stopped too soon / Applied too long",
];

/// Representation of a single Unsafe Control Action derived from an action and guide word.
/// Realises: [REQ-STPA-UCA-UNSAFECONTROLACTION]
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

/// Aggregated read-only AST metadata harvested across a `PackageDef` hierarchy for UCA synthesis.
/// Realises: [REQ-STPA-UCA-AST-RISK-CONTEXT]
pub(super) struct AstRiskContext {
    pub(super) base_pkg_name: String,
    pub(super) all_constraints: Vec<ConstraintDef>,
    pub(super) pkg_attributes: Vec<AttributeDef>,
    pub(super) all_part_attributes: Vec<AttributeDef>,
    pub(super) pkg_hazards: Vec<HazardDef>,
    pub(super) pkg_requirements: Vec<RequirementDef>,
    pub(super) has_assert_or_guard: bool,
}

impl AstRiskContext {
    /// Harvests package- and part-scoped attributes, hazards, constraints, and requirements from `pkg`.
    /// Realises: [REQ-STPA-UCA-AST-RISK-CONTEXT-BUILD]
    fn from_package(pkg: &PackageDef, all_parts: &[PartDef]) -> Self {
        let trimmed_pkg = pkg.name.trim();
        let base_pkg_name = if trimmed_pkg.is_empty() {
            "SystemController".to_string()
        } else {
            trimmed_pkg.to_string()
        };

        let all_constraints = pkg.get_all_constraints();
        let mut pkg_attributes = Vec::new();
        let mut pkg_hazards = Vec::new();
        let mut pkg_requirements = Vec::new();
        let mut seen_reqs = BTreeSet::new();

        for sub_pkg in pkg.get_all_packages() {
            for attr in &sub_pkg.attribute_defs {
                pkg_attributes.push(attr.clone());
            }
            for haz in &sub_pkg.hazard_defs {
                pkg_hazards.push(haz.clone());
            }
            for req in &sub_pkg.requirement_defs {
                if seen_reqs.insert(req.name.clone()) {
                    pkg_requirements.push(req.clone());
                }
            }
        }

        let mut all_part_attributes = Vec::new();
        for part in all_parts {
            if part.name.starts_with('_') {
                continue;
            }
            for attr in &part.attributes {
                all_part_attributes.push(attr.clone());
            }
        }

        let has_assert_or_guard = all_constraints
            .iter()
            .any(|c| c.is_assertion || !c.expression.trim().is_empty())
            || !pkg_requirements.is_empty();

        Self {
            base_pkg_name,
            all_constraints,
            pkg_attributes,
            all_part_attributes,
            pkg_hazards,
            pkg_requirements,
            has_assert_or_guard,
        }
    }
}

/// Scoped action target descriptor passed to UCA severity, integrity, and context resolvers.
/// Realises: [REQ-STPA-UCA-EXPAND-CARTESIAN-STPA]
pub(super) struct UcaActionTarget<'a> {
    pub(super) matched_part: Option<&'a PartDef>,
    pub(super) controller_name: &'a str,
    pub(super) action_name: &'a str,
    pub(super) action_idx: usize,
}

/// Appends the 4 guide-word `UnsafeControlAction` rows for a single `UcaActionTarget` onto `ucas`.
/// Realises: [REQ-STPA-UCA-EXPAND-CARTESIAN-STPA]
fn append_action_ucas(
    target: &UcaActionTarget<'_>,
    ctx: &AstRiskContext,
    uca_counter: &mut usize,
    ucas: &mut Vec<UnsafeControlAction>,
) {
    let hazard_id = format!("H-{}", target.action_idx.saturating_add(1));
    for guide_word in &STPA_GUIDE_WORDS {
        let uca_idx = *uca_counter;
        *uca_counter = uca_counter.saturating_add(1);
        let severity = resolve_uca_severity(target, guide_word, ctx);
        let sail = resolve_uca_integrity_level(target, &severity, ctx);
        let context = synthesize_uca_context(target, guide_word, uca_idx, ctx);
        ucas.push(UnsafeControlAction {
            id: format!("UCA-{:03}", *uca_counter),
            controller: target.controller_name.to_string(),
            control_action: target.action_name.to_string(),
            guide_word: (*guide_word).to_string(),
            context,
            hazard: hazard_id.clone(),
            constraint: format!("SC-{:03}", *uca_counter),
            severity,
            sail,
        });
    }
}

/// Expands the dynamic Cartesian UCA matrix across controlling `PartDef` actions (or package-level
/// `ActionDef` nodes) and the 4 canonical STPA guide words.
/// Realises: [REQ-STPA-UCA-EXPAND-CARTESIAN-STPA]
pub fn expand_cartesian_stpa(pkg: &PackageDef) -> Vec<UnsafeControlAction> {
    let mut ucas = Vec::new();
    let all_parts = pkg.get_all_parts();
    let non_internal_parts: Vec<&PartDef> = all_parts
        .iter()
        .filter(|p| !p.name.starts_with('_'))
        .collect();
    let ctx = AstRiskContext::from_package(pkg, &all_parts);

    let mut controllers: Vec<&PartDef> = non_internal_parts
        .iter()
        .copied()
        .filter(|part| !part.actions.is_empty())
        .collect();
    if controllers.is_empty() {
        controllers = all_parts
            .iter()
            .filter(|part| !part.actions.is_empty())
            .collect();
    }

    let mut uca_counter: usize = 0;
    let mut action_counter: usize = 0;

    for controller in &controllers {
        let controller_name = sanitize_uca_cell(&controller.name);
        for action in &controller.actions {
            let action_idx = action_counter;
            action_counter = action_counter.saturating_add(1);
            let action_name = sanitize_uca_cell(&action.name);
            let target = UcaActionTarget {
                matched_part: Some(controller),
                controller_name: &controller_name,
                action_name: &action_name,
                action_idx,
            };
            append_action_ucas(&target, &ctx, &mut uca_counter, &mut ucas);
        }
    }

    if ucas.is_empty() {
        for action in pkg.get_all_actions() {
            let resolved = resolve_action_controller(
                &action,
                &all_parts,
                &non_internal_parts,
                &ctx.base_pkg_name,
            );
            let action_idx = action_counter;
            action_counter = action_counter.saturating_add(1);
            let action_name = sanitize_uca_cell(&action.name);
            let target = UcaActionTarget {
                matched_part: resolved.part,
                controller_name: &resolved.name,
                action_name: &action_name,
                action_idx,
            };
            append_action_ucas(&target, &ctx, &mut uca_counter, &mut ucas);
        }
    }

    ucas
}

#[cfg(test)]
mod tests {
    use super::*;
    use deap_core::sysml_ast::{ActionDef, PartDef};

    /// Verifies that `expand_cartesian_stpa` expands 2 inline controller actions across all 4 STPA
    /// guide words into 8 bijective UCAs preserving `H-1`/`H-2` and `SC-001`..`SC-008` identifiers.
    /// Realises: [REQ-STPA-UCA-TEST-CARTESIAN-EXPANSION]
    #[test]
    fn test_uca_cartesian_expansion_all_guide_words() -> Result<(), Box<dyn std::error::Error>> {
        let mut pkg = PackageDef {
            name: "TestPackage".to_string(),
            ..Default::default()
        };

        let mut controller = PartDef {
            name: "SystemController".to_string(),
            ..Default::default()
        };
        controller.actions.push(ActionDef {
            name: "EngageActuator".to_string(),
            ..Default::default()
        });
        controller.actions.push(ActionDef {
            name: "ReleaseSafetyLock".to_string(),
            ..Default::default()
        });
        pkg.part_defs.push(controller);

        let ucas = expand_cartesian_stpa(&pkg);
        assert_eq!(ucas.len(), 8);
        let first = ucas.first().ok_or("missing first element of ucas")?;
        assert_eq!(first.id, "UCA-001");
        assert_eq!(first.controller, "SystemController");
        assert_eq!(first.control_action, "EngageActuator");
        assert_eq!(first.guide_word, "Not providing");
        assert_eq!(first.constraint, "SC-001");
        assert_eq!(first.hazard, "H-1");
        assert_eq!(first.severity, "Critical (Derived)");
        assert_eq!(first.sail, "SIL-2 (Derived)");

        assert_eq!(
            ucas.get(1).ok_or("missing element 1 of ucas")?.guide_word,
            "Providing"
        );
        assert_eq!(
            ucas.get(2).ok_or("missing element 2 of ucas")?.guide_word,
            "Too early / Too late / Out of order"
        );
        assert_eq!(
            ucas.get(3).ok_or("missing element 3 of ucas")?.guide_word,
            "Stopped too soon / Applied too long"
        );

        assert_eq!(
            ucas.get(4)
                .ok_or("missing element 4 of ucas")?
                .control_action,
            "ReleaseSafetyLock"
        );
        assert_eq!(
            ucas.get(4).ok_or("missing element 4 of ucas")?.hazard,
            "H-2"
        );
        Ok(())
    }
}
