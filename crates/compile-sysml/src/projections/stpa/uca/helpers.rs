#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! STPA UCA Cell Sanitization, Attribute Lookup, and Controller Attribution Helpers.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! safety analysis objectives for module `crates/compile-sysml/src/stpa/uca/helpers.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Sanitizes arbitrary AST text into single-line Markdown table cells free of pipe delimiters and newlines.
//! - Resolves controller attribution deterministically from `action.performer`, user-declared `PartDef` nodes,
//!   or enclosing `PackageDef` names without hardcoded compiler tokens.
//!
//! ## 3. Preconditions
//! - Caller supplies valid slices of `AttributeDef` and `PartDef` nodes from a parsed `PackageDef`.
//!
//! ## 4. Postconditions
//! - Returns non-empty sanitized cell values and controller bindings.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) linear scan over candidate attributes or parts.
//! - Space Complexity: O(L) bounded by matched identifier or attribute string byte length L.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Returns `None` on absent or blank attribute defaults.
//! - State: Pure stateless helpers with zero shared mutable state.
//! - Action: Executes with zero regular expressions, panics, `.unwrap()`, or direct slice indexing.
//! - Resource: Minimal short-lived string buffers.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-UCA-MODULE] | SysML v2: PackageDef(stpa_uca_helpers)

use deap_core::sysml_cst::{ActionDef, AttributeDef, PartDef};

/// Resolved controller binding pairing an optional AST `PartDef` reference with its sanitized controller identifier.
/// Realises: [REQ-STPA-UCA-RESOLVED-CONTROLLER]
pub(super) struct ResolvedController<'a> {
    pub(super) part: Option<&'a PartDef>,
    pub(super) name: String,
}

/// Sanitizes arbitrary AST text into a single-line Markdown table cell free of pipe delimiters and newlines.
/// Realises: [REQ-STPA-UCA-SANITIZE-CELL]
pub(super) fn sanitize_uca_cell(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.trim().chars() {
        match ch {
            '\r' | '\n' => out.push(' '),
            '|' => out.push('/'),
            _ => out.push(ch),
        }
    }
    out.trim().to_string()
}

/// Normalizes an optional AST `AttributeDef` default value by stripping surrounding whitespace and quotes.
/// Realises: [REQ-STPA-UCA-CLEAN-ATTR-VALUE]
pub(super) fn clean_attr_value(raw: &str) -> Option<String> {
    let unquoted = raw.trim().trim_matches('"').trim();
    if unquoted.is_empty() {
        return None;
    }
    let clean = sanitize_uca_cell(unquoted);
    if clean.is_empty() {
        None
    } else {
        Some(clean)
    }
}

/// Searches an `AttributeDef` slice case-insensitively for an exact identifier match with a non-empty default value.
/// Realises: [REQ-STPA-UCA-FIND-ATTR-EXACT]
pub(super) fn find_attr_exact_ci(attrs: &[AttributeDef], target: &str) -> Option<String> {
    for attr in attrs {
        if attr.name.eq_ignore_ascii_case(target) {
            if let Some(ref val) = attr.default_value {
                if let Some(clean) = clean_attr_value(val) {
                    return Some(clean);
                }
            }
        }
    }
    None
}

/// Searches an `AttributeDef` slice case-insensitively for an identifier starting with `prefix` carrying a non-empty default value.
/// Realises: [REQ-STPA-UCA-FIND-ATTR-PREFIX]
pub(super) fn find_attr_prefix_ci(attrs: &[AttributeDef], prefix: &str) -> Option<String> {
    let lower_prefix = prefix.to_ascii_lowercase();
    for attr in attrs {
        if attr.name.to_ascii_lowercase().starts_with(&lower_prefix) {
            if let Some(ref val) = attr.default_value {
                if let Some(clean) = clean_attr_value(val) {
                    return Some(clean);
                }
            }
        }
    }
    None
}

/// Maps an integer AST `HazardDef.severity` score (`i32`) to its canonical SSOT severity classification label.
/// Realises: [REQ-STPA-UCA-MAP-HAZARD-SEVERITY]
pub(super) fn map_hazard_severity_score(score: i32) -> Option<String> {
    match score {
        s if s >= 4 => Some("Catastrophic (SSOT)".to_string()),
        3 => Some("Critical (SSOT)".to_string()),
        2 => Some("Marginal (SSOT)".to_string()),
        1 => Some("Minor (SSOT)".to_string()),
        _ => None,
    }
}

/// Resolves the deterministic controller binding for a package-scoped `ActionDef` without compiler self-reference injection.
/// Realises: [REQ-STPA-UCA-RESOLVE-CONTROLLER]
pub(super) fn resolve_action_controller<'a>(
    action: &ActionDef,
    all_parts: &'a [PartDef],
    non_internal_parts: &[&'a PartDef],
    base_pkg_name: &str,
) -> ResolvedController<'a> {
    if let Some(ref perf) = action.performer {
        let perf_trimmed = perf.trim();
        if !perf_trimmed.is_empty() {
            if let Some(matched) = non_internal_parts
                .iter()
                .copied()
                .find(|p| p.name == perf_trimmed || p.name.eq_ignore_ascii_case(perf_trimmed))
                .or_else(|| {
                    all_parts.iter().find(|p| {
                        p.name == perf_trimmed || p.name.eq_ignore_ascii_case(perf_trimmed)
                    })
                })
            {
                return ResolvedController {
                    part: Some(matched),
                    name: sanitize_uca_cell(&matched.name),
                };
            }
        }
    }

    if let Some(part) = non_internal_parts
        .first()
        .copied()
        .or_else(|| all_parts.first())
    {
        ResolvedController {
            part: Some(part),
            name: sanitize_uca_cell(&part.name),
        }
    } else {
        ResolvedController {
            part: None,
            name: sanitize_uca_cell(base_pkg_name),
        }
    }
}
