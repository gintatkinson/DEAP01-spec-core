#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! STPA UCA Severity, Integrity Level, and Operational Context Resolvers.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! safety analysis objectives for module `crates/compile-sysml/src/stpa/uca/resolvers.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Resolves UCA hazard severity and safety integrity levels deterministically from AST `HazardDef`
//!   and `AttributeDef` SSOT declarations or guide-word rules.
//! - Synthesizes guide-word-differentiated operational contexts cycled across AST states, ports,
//!   constraints, and requirements via checked `.checked_rem` and `.get(..)`.
//!
//! ## 3. Preconditions
//! - Caller supplies a valid `UcaActionTarget`, `guide_word`, and pre-harvested `AstRiskContext`.
//!
//! ## 4. Postconditions
//! - Returns non-empty single-line Markdown-safe severity, integrity level (`sail`), and context strings.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(|hazards| + |attributes| + L) per UCA resolution.
//! - Space Complexity: O(L) bounded by formatted string byte length L.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Degrades gracefully to deterministic derived classifications when SSOT nodes are absent.
//! - State: Pure read-only AST inspection.
//! - Action: Uses `.checked_rem` and `.get(..)` exclusively with zero panics or `.unwrap()`.
//! - Resource: Bounded single-line string allocations.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-UCA-MODULE] | SysML v2: PackageDef(stpa_uca_resolvers)

use deap_core::sysml_cst::{ConstraintDef, PartDef};

use super::helpers::{
    find_attr_exact_ci, find_attr_prefix_ci, map_hazard_severity_score, sanitize_uca_cell,
};
use super::{AstRiskContext, UcaActionTarget};

/// Evaluates explicit AST `HazardDef` scores on `target.matched_part` and `ctx.pkg_hazards`.
/// Realises: [REQ-STPA-UCA-RESOLVE-SEVERITY]
fn lookup_ssot_hazard_severity(
    target: &UcaActionTarget<'_>,
    ctx: &AstRiskContext,
) -> Option<String> {
    let action_lower = target.action_name.to_ascii_lowercase();
    if let Some(part) = target.matched_part {
        if !part.hazards.is_empty() {
            let chosen = part
                .hazards
                .iter()
                .find(|h| {
                    h.severity >= 1
                        && (h.name.eq_ignore_ascii_case(target.action_name)
                            || h.name.to_ascii_lowercase().contains(&action_lower))
                })
                .or_else(|| {
                    target
                        .action_idx
                        .checked_rem(part.hazards.len())
                        .and_then(|i| part.hazards.get(i))
                });
            if let Some(mapped) = chosen.and_then(|h| map_hazard_severity_score(h.severity)) {
                return Some(mapped);
            }
        }
    }

    if !ctx.pkg_hazards.is_empty() {
        let chosen = ctx
            .pkg_hazards
            .iter()
            .find(|h| {
                h.severity >= 1
                    && (h.name.eq_ignore_ascii_case(target.action_name)
                        || h.name.eq_ignore_ascii_case(target.controller_name)
                        || h.name.to_ascii_lowercase().contains(&action_lower))
            })
            .or_else(|| {
                target
                    .action_idx
                    .checked_rem(ctx.pkg_hazards.len())
                    .and_then(|i| ctx.pkg_hazards.get(i))
            });
        if let Some(mapped) = chosen.and_then(|h| map_hazard_severity_score(h.severity)) {
            return Some(mapped);
        }
    }

    None
}

/// Derives UCA hazard severity from explicit AST `HazardDef` / `AttributeDef` SSOT declarations or deterministic STPA guide-word rules.
/// Realises: [REQ-STPA-UCA-RESOLVE-SEVERITY]
pub(super) fn resolve_uca_severity(
    target: &UcaActionTarget<'_>,
    guide_word: &str,
    ctx: &AstRiskContext,
) -> String {
    if let Some(mapped) = lookup_ssot_hazard_severity(target, ctx) {
        return mapped;
    }

    let candidate_keys = [
        format!("{}_Severity", target.action_name),
        format!("{}_Severity", target.controller_name),
        "HazardSeverity".to_string(),
        "DefaultSeverity".to_string(),
        "Severity".to_string(),
    ];

    for key in &candidate_keys {
        let found = target
            .matched_part
            .and_then(|p| find_attr_exact_ci(&p.attributes, key))
            .or_else(|| find_attr_exact_ci(&ctx.pkg_attributes, key))
            .or_else(|| find_attr_exact_ci(&ctx.all_part_attributes, key));
        if let Some(val) = found {
            if let Ok(parsed_score) = val.parse::<i32>() {
                if let Some(mapped) = map_hazard_severity_score(parsed_score) {
                    return mapped;
                }
            }
            return val;
        }
    }

    match guide_word {
        "Not providing" | "Providing" => {
            if ctx.has_assert_or_guard {
                "Catastrophic (Derived)".to_string()
            } else {
                "Critical (Derived)".to_string()
            }
        }
        _ => {
            if ctx.has_assert_or_guard {
                "Critical (Derived)".to_string()
            } else {
                "Marginal (Derived)".to_string()
            }
        }
    }
}

/// Derives UCA safety integrity level from scoped AST `AttributeDef` declarations or deterministic IEC 61508 / ISO 15026-3 severity mapping.
/// Realises: [REQ-STPA-UCA-RESOLVE-INTEGRITY]
pub(super) fn resolve_uca_integrity_level(
    target: &UcaActionTarget<'_>,
    severity: &str,
    ctx: &AstRiskContext,
) -> String {
    let integrity_keys = [
        format!("{}_IntegrityLevel", target.action_name),
        format!("{}_IntegrityLevel", target.controller_name),
        "IntegrityLevel".to_string(),
    ];
    let assurance_keys = [
        format!("{}_AssuranceLevel", target.action_name),
        format!("{}_AssuranceLevel", target.controller_name),
        "AssuranceLevel".to_string(),
    ];

    if let Some(part) = target.matched_part {
        for key in &integrity_keys {
            if let Some(val) = find_attr_exact_ci(&part.attributes, key) {
                return val;
            }
        }
        if let Some(val) = find_attr_prefix_ci(&part.attributes, "IntegrityLevel_") {
            return val;
        }
        for key in &assurance_keys {
            if let Some(val) = find_attr_exact_ci(&part.attributes, key) {
                return val;
            }
        }
    }

    for attrs in [&ctx.pkg_attributes, &ctx.all_part_attributes] {
        for key in &integrity_keys {
            if let Some(val) = find_attr_exact_ci(attrs, key) {
                return val;
            }
        }
        if let Some(val) = find_attr_prefix_ci(attrs, "IntegrityLevel_") {
            return val;
        }
        for key in &assurance_keys {
            if let Some(val) = find_attr_exact_ci(attrs, key) {
                return val;
            }
        }
    }

    if severity.starts_with("Catastrophic") {
        "SIL-3 (Derived)".to_string()
    } else if severity.starts_with("Critical") {
        "SIL-2 (Derived)".to_string()
    } else {
        "SIL-1 (Derived)".to_string()
    }
}

/// Resolves the cycled constraint or requirement guard anchor for `uca_idx` across part and package scopes.
/// Realises: [REQ-STPA-UCA-SYNTHESIZE-CONTEXT]
fn resolve_uca_guard_anchor(
    matched_part: Option<&PartDef>,
    uca_idx: usize,
    ctx: &AstRiskContext,
) -> String {
    let active_constraints: &[ConstraintDef] = match matched_part {
        Some(p) if !p.constraints.is_empty() => &p.constraints,
        _ => &ctx.all_constraints,
    };

    if let Some(c) = uca_idx
        .checked_rem(active_constraints.len())
        .and_then(|i| active_constraints.get(i))
    {
        let expr = sanitize_uca_cell(&c.expression);
        if expr.is_empty() {
            return format!("constraint {}", sanitize_uca_cell(&c.name));
        }
        return format!("constraint {} ({})", sanitize_uca_cell(&c.name), expr);
    }

    if let Some(r) = uca_idx
        .checked_rem(ctx.pkg_requirements.len())
        .and_then(|i| ctx.pkg_requirements.get(i))
    {
        let rid = sanitize_uca_cell(&r.req_id);
        if rid.is_empty() {
            return format!("requirement {}", sanitize_uca_cell(&r.name));
        }
        return format!("requirement {} ({})", sanitize_uca_cell(&r.name), rid);
    }

    format!(
        "package {} control envelope",
        sanitize_uca_cell(&ctx.base_pkg_name)
    )
}

/// Synthesizes a single-line, guide-word-differentiated STPA operational context anchored to AST states, ports, constraints, and requirements.
/// Realises: [REQ-STPA-UCA-SYNTHESIZE-CONTEXT]
pub(super) fn synthesize_uca_context(
    target: &UcaActionTarget<'_>,
    guide_word: &str,
    uca_idx: usize,
    ctx: &AstRiskContext,
) -> String {
    let state_anchor = target
        .matched_part
        .and_then(|p| {
            uca_idx
                .checked_rem(p.states.len())
                .and_then(|i| p.states.get(i))
        })
        .map(|s| format!("state {}", sanitize_uca_cell(&s.name)))
        .unwrap_or_else(|| "active operational state".to_string());

    let port_anchor = target
        .matched_part
        .and_then(|p| {
            uca_idx
                .checked_rem(p.ports.len())
                .and_then(|i| p.ports.get(i))
        })
        .map(|port| format!("port {}", sanitize_uca_cell(&port.name)))
        .unwrap_or_else(|| "control interface".to_string());

    let guard_anchor = resolve_uca_guard_anchor(target.matched_part, uca_idx, ctx);
    let ctrl = sanitize_uca_cell(target.controller_name);
    let act = sanitize_uca_cell(target.action_name);

    let raw = match guide_word {
        "Not providing" => format!(
            "{} omits {} in {} across {} while {} demands intervention",
            ctrl, act, state_anchor, port_anchor, guard_anchor
        ),
        "Providing" => format!(
            "{} issues {} in {} across {} violating {}",
            ctrl, act, state_anchor, port_anchor, guard_anchor
        ),
        "Too early / Too late / Out of order" => format!(
            "{} dispatches {} out of temporal sequence in {} via {} relative to {}",
            ctrl, act, state_anchor, port_anchor, guard_anchor
        ),
        _ => format!(
            "{} prematurely halts or latches {} in {} on {} against {}",
            ctrl, act, state_anchor, port_anchor, guard_anchor
        ),
    };

    sanitize_uca_cell(&raw)
}
