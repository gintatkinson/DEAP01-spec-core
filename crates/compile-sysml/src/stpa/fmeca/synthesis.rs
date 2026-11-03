#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! FMECA Criticality Matrix Synthesis and Row Emission Engine.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! safety objectives for module `crates/compile-sysml/src/stpa/fmeca/synthesis.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Excludes negated components (`!Ident` and negative attribute/constraint stems) and `_`-prefixed internal helpers.
//! - Emits researched `registry_entries` with verified `"SSOT"` (or `"Derived"`) provenance and explicit component-referenced
//!   `HazardDef` rows with `"SSOT"` provenance, while stamping `"Derived"` on all compiler-synthesized AST member and padding rows.
//! - Enumerates 100% of AST members across `[0..]` and pads non-empty models to `>= 15` rows.
//!
//! ## 3. Preconditions
//! - Caller supplies a parsed `PackageDef`, optional `FmecaScoringConfig`, and slice of `RegistryFailureEntry` records.
//!
//! ## 4. Postconditions
//! - Returns `Vec<FmecaRow>` (`Vec::new()` iff no non-negated `PartDef` exists; otherwise `len() >= 15`) with sequential `FMECA-001..` IDs.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(|sysml| + P * (R + H + M)) over serialized SysML size, parts P, registry entries R, hazards H, and AST members M.
//! - Space Complexity: O(P + M + R) auxiliary sets and output row vector.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Accepts empty packages, empty scoring scales, and empty registry slices gracefully.
//! - State: Pure deterministic synthesis with `BTreeSet` negation precomputation.
//! - Action: Bounded loops with `.checked_rem` and `.get(..)` preventing division-by-zero or index panics.
//! - Resource: Bounded vector growth proportional to AST member and registry cardinality.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-FMECA-GENERATE-WITH-REGISTRY] | SysML v2: PackageDef(fmeca_synthesis)

use deap_core::sysml_cst::{HazardDef, PackageDef, PartDef};

use super::members::{
    classify_hazard_dimension, collect_dimension_member_triples, compute_negated_part_names,
    hazard_references_part, registry_entry_matches_part, resolve_scores,
};
use super::{
    DimensionTemplate, FmecaRow, FmecaScoringConfig, RegistryFailureEntry,
    ABSTRACT_STRUCTURAL_SUB_MODES, DIMENSION_TEMPLATES,
};

/// Shared read-only synthesis inputs passed to per-part dimension emission helpers.
/// Realises: [REQ-STPA-FMECA-EMIT-PART-DIMENSION]
struct PartDimensionContext<'a> {
    scoring_config: Option<&'a FmecaScoringConfig>,
    registry_entries: &'a [RegistryFailureEntry],
    all_hazards: &'a [HazardDef],
}

/// Unscored candidate FMECA row payload bundled to bound parameter arity in `push_scored_row`.
/// Realises: [REQ-STPA-FMECA-PUSH-SCORED-ROW]
struct PendingFmecaRow<'a> {
    component: &'a str,
    tmpl: &'a DimensionTemplate,
    basis_str: &'a str,
    triple: (String, String, String),
}

/// Generates the FMECA matrix covering 100% of declared AST `part def` components without external registry records.
/// Realises: [REQ-STPA-FMECA-GENERATE-FMECA-MATRIX]
pub fn generate_fmeca_matrix(
    pkg: &PackageDef,
    scoring_config: Option<&FmecaScoringConfig>,
) -> Vec<FmecaRow> {
    generate_fmeca_matrix_with_registry(pkg, scoring_config, &[])
}

/// Generates the FMECA matrix integrating researched `FAILURE_MODE_REGISTRY.md` records, explicit `HazardDef` declarations, and full `[0..]` AST member synthesis.
/// Realises: [REQ-STPA-FMECA-GENERATE-WITH-REGISTRY]
pub fn generate_fmeca_matrix_with_registry(
    pkg: &PackageDef,
    scoring_config: Option<&FmecaScoringConfig>,
    registry_entries: &[RegistryFailureEntry],
) -> Vec<FmecaRow> {
    let mut rows = Vec::new();
    let all_parts = pkg.get_all_parts();
    let negated_parts = compute_negated_part_names(pkg, &all_parts);

    let mut filtered_parts: Vec<&PartDef> = all_parts
        .iter()
        .filter(|p| !p.name.starts_with('_'))
        .filter(|p| !negated_parts.contains(&p.name))
        .collect();

    if filtered_parts.is_empty() {
        filtered_parts = all_parts
            .iter()
            .filter(|p| !negated_parts.contains(&p.name))
            .collect();
    }

    if filtered_parts.is_empty() {
        return rows;
    }

    let all_hazards = pkg.get_all_hazards();
    let ctx = PartDimensionContext {
        scoring_config,
        registry_entries,
        all_hazards: &all_hazards,
    };

    for part in &filtered_parts {
        for tmpl in &DIMENSION_TEMPLATES {
            emit_part_dimension_rows(part, tmpl, &ctx, &mut rows);
        }
    }

    pad_minimum_derived_rows(&filtered_parts, scoring_config, &mut rows);

    rows
}

/// Appends pre-researched `RegistryFailureEntry` records matching `(part_name, tmpl.dimension)` onto `rows`.
/// Realises: [REQ-STPA-FMECA-EMIT-PART-DIMENSION]
fn emit_matching_registry_rows(
    part_name: &str,
    tmpl: &DimensionTemplate,
    matching: &[&RegistryFailureEntry],
    rows: &mut Vec<FmecaRow>,
) {
    for entry in matching {
        let s = entry.severity.clamp(1, 10);
        let o = entry.occurrence.clamp(1, 10);
        let d = entry.detection.clamp(1, 10);
        let rpn = s.saturating_mul(o).saturating_mul(d);
        let effect = if entry.effect.is_empty() {
            format!("{}: {}", part_name, tmpl.effect_template)
        } else {
            entry.effect.clone()
        };
        let mitigation = if entry.mitigation.is_empty() {
            format!("{}: {}", part_name, tmpl.mitigation_template)
        } else {
            entry.mitigation.clone()
        };
        let basis = if entry.is_ssot_evidenced {
            "SSOT"
        } else {
            "Derived"
        };
        let row_id = format!("FMECA-{:03}", rows.len().saturating_add(1));
        rows.push(FmecaRow {
            id: row_id,
            component: part_name.to_string(),
            dimension: tmpl.dimension.as_str().to_string(),
            failure_mode: entry.failure_mode.clone(),
            effect,
            severity: s,
            occurrence: o,
            detection: d,
            rpn,
            severity_cell: format!("{}", s),
            occurrence_cell: format!("{}", o),
            detection_cell: format!("{}", d),
            rpn_cell: format!("{}", rpn),
            mitigation,
            basis: basis.to_string(),
        });
    }
}

/// Emits FMECA rows for a single `(PartDef, FailureDimension)` pair using registry precedence, explicit `HazardDef` matching, or `[0..]` AST member enumeration.
/// Realises: [REQ-STPA-FMECA-EMIT-PART-DIMENSION]
fn emit_part_dimension_rows(
    part: &PartDef,
    tmpl: &DimensionTemplate,
    ctx: &PartDimensionContext<'_>,
    rows: &mut Vec<FmecaRow>,
) {
    let part_name = part.name.as_str();
    let matching_registry: Vec<&RegistryFailureEntry> = ctx
        .registry_entries
        .iter()
        .filter(|entry| registry_entry_matches_part(entry, part_name, tmpl.dimension))
        .collect();

    if !matching_registry.is_empty() {
        emit_matching_registry_rows(part_name, tmpl, &matching_registry, rows);
        return;
    }

    let matching_hazards: Vec<&HazardDef> = ctx
        .all_hazards
        .iter()
        .filter(|haz| {
            hazard_references_part(haz, part_name)
                && classify_hazard_dimension(haz) == tmpl.dimension
        })
        .collect();

    if !matching_hazards.is_empty() {
        for haz in matching_hazards {
            let triple = (
                format!("{}: {}", part_name, haz.name),
                format!(
                    "{}: {}",
                    part_name,
                    haz.doc.as_deref().unwrap_or(tmpl.effect_template)
                ),
                format!(
                    "{}: Enforce declared hazard constraint mitigating '{}'",
                    part_name, haz.name
                ),
            );
            push_scored_row(
                rows,
                ctx.scoring_config,
                PendingFmecaRow {
                    component: part_name,
                    tmpl,
                    basis_str: "SSOT",
                    triple,
                },
            );
        }
        return;
    }

    let triples = collect_dimension_member_triples(part, tmpl);
    for triple in triples {
        push_scored_row(
            rows,
            ctx.scoring_config,
            PendingFmecaRow {
                component: part_name,
                tmpl,
                basis_str: "Derived",
                triple,
            },
        );
    }
}

/// Pads `rows` up to a minimum of 15 entries using domain-agnostic abstract structural sub-modes with `"Derived"` basis.
/// Realises: [REQ-STPA-FMECA-PAD-DERIVED-ROWS]
fn pad_minimum_derived_rows(
    filtered_parts: &[&PartDef],
    scoring_config: Option<&FmecaScoringConfig>,
    rows: &mut Vec<FmecaRow>,
) {
    let mut add_idx: usize = 0;
    while rows.len() < 15 {
        let sub_tmpl = add_idx
            .checked_rem(ABSTRACT_STRUCTURAL_SUB_MODES.len())
            .and_then(|i| ABSTRACT_STRUCTURAL_SUB_MODES.get(i));
        let part = add_idx
            .checked_rem(filtered_parts.len())
            .and_then(|i| filtered_parts.get(i));
        let (Some(sub_tmpl), Some(&part)) = (sub_tmpl, part) else {
            break;
        };
        let part_name = part.name.as_str();
        let triple = (
            format!("{} {}", part_name, sub_tmpl.mode_suffix),
            format!("{}: {}", part_name, sub_tmpl.effect_template),
            format!("{}: {}", part_name, sub_tmpl.mitigation_template),
        );
        push_scored_row(
            rows,
            scoring_config,
            PendingFmecaRow {
                component: part_name,
                tmpl: sub_tmpl,
                basis_str: "Derived",
                triple,
            },
        );
        add_idx = add_idx.saturating_add(1);
    }
}

/// Resolves `(S, O, D, RPN)` via `resolve_scores` and appends a formatted `FmecaRow` to `rows`.
/// Realises: [REQ-STPA-FMECA-PUSH-SCORED-ROW]
fn push_scored_row(
    rows: &mut Vec<FmecaRow>,
    scoring_config: Option<&FmecaScoringConfig>,
    pending: PendingFmecaRow<'_>,
) {
    let row_index = rows.len();
    let (s, o, d, s_cell, o_cell, d_cell) = resolve_scores(scoring_config, row_index, pending.tmpl);
    let rpn = s.saturating_mul(o).saturating_mul(d);
    let (mode_str, effect_str, mit_str) = pending.triple;
    rows.push(FmecaRow {
        id: format!("FMECA-{:03}", row_index.saturating_add(1)),
        component: pending.component.to_string(),
        dimension: pending.tmpl.dimension.as_str().to_string(),
        failure_mode: mode_str,
        effect: effect_str,
        severity: s,
        occurrence: o,
        detection: d,
        rpn,
        severity_cell: s_cell,
        occurrence_cell: o_cell,
        detection_cell: d_cell,
        rpn_cell: format!("{}", rpn),
        mitigation: mit_str,
        basis: pending.basis_str.to_string(),
    });
}
