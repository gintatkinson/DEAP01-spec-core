#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! FMECA AST Member Enumeration, Negated-Component Filtering, and Score Resolution Helpers.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! safety objectives for module `crates/compile-sysml/src/stpa/fmeca/members.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Zero `.first()` truncation: visits 100% of `part.ports` (`Interface`), `part.states` (`State`),
//!   `part.actions` chained with `part.operations` (`Action`), and `part.attributes` (`Resource`).
//! - Precomputes negated component names (`!Ident` and negative attribute/constraint stems) and resolves clamped `[1, 10]` `(S, O, D)` scores.
//!
//! ## 3. Preconditions
//! - Caller supplies valid AST `PackageDef`, `PartDef`, `HazardDef`, `RegistryFailureEntry`, and `DimensionTemplate` references.
//!
//! ## 4. Postconditions
//! - Returns deterministic component matching predicates, hazard dimension classifications, member triples, and `(S, O, D)` score tuples.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(|sysml| + P + M_d) over serialized SysML size, parts P, and AST member count M_d.
//! - Space Complexity: O(P + M_d) bounded sets and `(String, String, String)` triple vectors.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Empty member vectors automatically emit a component-scoped abstract fallback triple.
//! - State: Pure functional read-only projection over AST slices.
//! - Action: Finite iteration with `.checked_rem` and `.get(..)` preventing division-by-zero or index panics.
//! - Resource: Bounded heap allocations proportional to AST member count.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-FMECA-FULL-ENUMERATION] | SysML v2: PackageDef(fmeca_members)

use std::collections::BTreeSet;

use deap_core::sysml_ast::{HazardDef, PackageDef, PartDef};

use super::{
    DimensionTemplate, FailureDimension, FmecaScoringConfig, RegistryFailureEntry,
    ScoringScaleEntry,
};

/// Scans serialized SysML text for `!Ident` negation assertions and records matched identifiers into `negated_parts`.
/// Realises: [REQ-STPA-FMECA-NEGATED-PARTS]
fn scan_bang_negated_identifiers(pkg_sysml: &str, negated_parts: &mut BTreeSet<String>) {
    let sysml_bytes = pkg_sysml.as_bytes();
    let mut byte_idx = 0usize;
    while byte_idx < sysml_bytes.len() {
        if sysml_bytes.get(byte_idx) == Some(&b'!') {
            let mut start = byte_idx.saturating_add(1);
            while start < sysml_bytes.len()
                && sysml_bytes.get(start).is_some_and(u8::is_ascii_whitespace)
            {
                start = start.saturating_add(1);
            }
            if sysml_bytes
                .get(start)
                .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
            {
                let mut end = start.saturating_add(1);
                while end < sysml_bytes.len()
                    && sysml_bytes
                        .get(end)
                        .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
                {
                    end = end.saturating_add(1);
                }
                if let Some(ident) = pkg_sysml.get(start..end) {
                    negated_parts.insert(ident.to_string());
                }
                byte_idx = end;
                continue;
            }
        }
        byte_idx = byte_idx.saturating_add(1);
    }
}

/// Precomputes the `BTreeSet` of `PartDef` names excluded by `!Ident` assertions or negative constraint stems.
/// Realises: [REQ-STPA-FMECA-NEGATED-PARTS]
pub(super) fn compute_negated_part_names(
    pkg: &PackageDef,
    all_parts: &[PartDef],
) -> BTreeSet<String> {
    let pkg_sysml = crate::semantic::serializer::to_sysml(pkg);
    let mut negated_parts = BTreeSet::new();
    scan_bang_negated_identifiers(&pkg_sysml, &mut negated_parts);

    let mut negated_stems: BTreeSet<String> =
        pkg.extract_negated_components().into_iter().collect();
    negated_stems.extend(deap_core::sysml_ast::extract_negated_components(&pkg_sysml));
    for part in all_parts {
        let comp_clean: String = part
            .name
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        if !comp_clean.is_empty()
            && negated_stems.iter().any(|stem| {
                comp_clean.contains(stem.as_str())
                    || deap_core::sysml_ast::component_matches(&part.name, stem)
            })
        {
            negated_parts.insert(part.name.clone());
        }
    }

    negated_parts
}

/// Resolves `(S, O, D)` numeric scores and formatted cell strings for `index` from `scoring_config` or `tmpl`.
/// Realises: [REQ-STPA-FMECA-RESOLVE-SCORES]
pub(super) fn resolve_scores(
    scoring_config: Option<&FmecaScoringConfig>,
    index: usize,
    tmpl: &DimensionTemplate,
) -> (u32, u32, u32, String, String, String) {
    if let Some(cfg) = scoring_config {
        let pick =
            |scale: &[ScoringScaleEntry], offset: usize, default_score: u32| -> (u32, String) {
                match index
                    .saturating_add(offset)
                    .checked_rem(scale.len())
                    .and_then(|i| scale.get(i))
                {
                    Some(entry) => {
                        let clamped = entry.score.clamp(1, 10);
                        (clamped, format!("{} ({})", entry.label, clamped))
                    }
                    None => (default_score, format!("{}", default_score)),
                }
            };

        let (s, s_cell) = pick(&cfg.severity_scale, 0, tmpl.default_s);
        let (o, o_cell) = pick(&cfg.occurrence_scale, 1, tmpl.default_o);
        let (d, d_cell) = pick(&cfg.detection_scale, 2, tmpl.default_d);
        (s, o, d, s_cell, o_cell, d_cell)
    } else {
        (
            tmpl.default_s,
            tmpl.default_o,
            tmpl.default_d,
            format!("{}", tmpl.default_s),
            format!("{}", tmpl.default_o),
            format!("{}", tmpl.default_d),
        )
    }
}

/// Checks whether a `RegistryFailureEntry` targets `(part_name, dim)` via exact case-insensitive match or `component_matches`.
/// Realises: [REQ-STPA-FMECA-REGISTRY-COMPONENT-MATCH]
pub(super) fn registry_entry_matches_part(
    entry: &RegistryFailureEntry,
    part_name: &str,
    dim: FailureDimension,
) -> bool {
    let comp = entry.component.trim();
    !comp.is_empty()
        && entry.dimension == dim
        && (comp.eq_ignore_ascii_case(part_name)
            || deap_core::sysml_ast::component_matches(part_name, comp))
}

/// Determines whether an explicit `HazardDef` directly references `part_name` in its identifier or doc block.
/// Realises: [REQ-STPA-FMECA-HAZARD-PART-MATCH]
pub(super) fn hazard_references_part(haz: &HazardDef, part_name: &str) -> bool {
    let trimmed = part_name.trim();
    if trimmed.is_empty() {
        return false;
    }
    let needle = trimmed.to_ascii_lowercase();
    if haz.name.to_ascii_lowercase().contains(&needle) {
        return true;
    }
    haz.doc
        .as_deref()
        .is_some_and(|doc| doc.to_ascii_lowercase().contains(&needle))
}

/// Classifies an explicit `HazardDef` into one of the four universal `FailureDimension` buckets.
/// Realises: [REQ-STPA-FMECA-CLASSIFY-HAZARD-DIM]
pub(super) fn classify_hazard_dimension(haz: &HazardDef) -> FailureDimension {
    let combined =
        format!("{} {}", haz.name, haz.doc.as_deref().unwrap_or("")).to_ascii_lowercase();
    if combined.contains("interface")
        || combined.contains("port")
        || combined.contains("protocol")
        || combined.contains("ingress")
        || combined.contains("egress")
    {
        FailureDimension::Interface
    } else if combined.contains("state")
        || combined.contains("transition")
        || combined.contains("deadlock")
    {
        FailureDimension::State
    } else if combined.contains("resource")
        || combined.contains("capacity")
        || combined.contains("quota")
        || combined.contains("memory")
        || combined.contains("attribute")
    {
        FailureDimension::Resource
    } else {
        FailureDimension::Action
    }
}

/// Enumerates all `[0..]` AST members of `part` belonging to `tmpl.dimension` (or 1 component-level abstract fallback triple when empty).
/// Realises: [REQ-STPA-FMECA-FULL-ENUMERATION]
pub(super) fn collect_dimension_member_triples(
    part: &PartDef,
    tmpl: &DimensionTemplate,
) -> Vec<(String, String, String)> {
    let part_name = part.name.as_str();
    let mut triples = Vec::new();

    match tmpl.dimension {
        FailureDimension::Interface => {
            for port in &part.ports {
                triples.push((
                    format!("{} Port '{}' {}", part_name, port.name, tmpl.mode_suffix),
                    format!(
                        "{}: {} on port '{}'",
                        part_name, tmpl.effect_template, port.name
                    ),
                    format!(
                        "{}: {} for port '{}'",
                        part_name, tmpl.mitigation_template, port.name
                    ),
                ));
            }
        }
        FailureDimension::State => {
            for st in &part.states {
                triples.push((
                    format!("{} State '{}' {}", part_name, st.name, tmpl.mode_suffix),
                    format!(
                        "{}: {} in state '{}'",
                        part_name, tmpl.effect_template, st.name
                    ),
                    format!(
                        "{}: {} for state '{}'",
                        part_name, tmpl.mitigation_template, st.name
                    ),
                ));
            }
        }
        FailureDimension::Action => {
            collect_action_and_operation_triples(part, tmpl, &mut triples);
        }
        FailureDimension::Resource => {
            for attr in &part.attributes {
                triples.push((
                    format!(
                        "{} Attribute '{}' {}",
                        part_name, attr.name, tmpl.mode_suffix
                    ),
                    format!(
                        "{}: {} affecting attribute '{}'",
                        part_name, tmpl.effect_template, attr.name
                    ),
                    format!(
                        "{}: {} for attribute '{}'",
                        part_name, tmpl.mitigation_template, attr.name
                    ),
                ));
            }
        }
    }

    if triples.is_empty() {
        triples.push((
            format!("{} {}", part_name, tmpl.mode_suffix),
            format!("{}: {}", part_name, tmpl.effect_template),
            format!("{}: {}", part_name, tmpl.mitigation_template),
        ));
    }

    triples
}

/// Appends behavioral `Action` and `Operation` failure triples for `part` onto `triples`.
/// Realises: [REQ-STPA-FMECA-FULL-ENUMERATION]
fn collect_action_and_operation_triples(
    part: &PartDef,
    tmpl: &DimensionTemplate,
    triples: &mut Vec<(String, String, String)>,
) {
    let part_name = part.name.as_str();
    for act in &part.actions {
        triples.push((
            format!("{} Action '{}' {}", part_name, act.name, tmpl.mode_suffix),
            format!(
                "{}: {} during action '{}'",
                part_name, tmpl.effect_template, act.name
            ),
            format!(
                "{}: {} for action '{}'",
                part_name, tmpl.mitigation_template, act.name
            ),
        ));
    }
    for op in &part.operations {
        triples.push((
            format!("{} Operation '{}' {}", part_name, op.name, tmpl.mode_suffix),
            format!(
                "{}: {} during operation '{}'",
                part_name, tmpl.effect_template, op.name
            ),
            format!(
                "{}: {} for operation '{}'",
                part_name, tmpl.mitigation_template, op.name
            ),
        ));
    }
}
