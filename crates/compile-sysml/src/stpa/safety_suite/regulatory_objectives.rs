#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Domain-agnostic regulatory safety objectives and integrity level assessment renderer.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! regulatory objective traceability objectives for module `crates/compile-sysml/src/stpa/safety_suite/regulatory_objectives.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Synthesizes `06_REGULATORY_OBJECTIVES_ASSESSMENT.md` deterministically from SysML v2 AST
//!   `RequirementDef`, `ConstraintDef`, and `CalcDef` declarations alongside parameter tokens.
//!
//! ## 3. Preconditions
//! - Caller supplies a parsed `PackageDef` and extracted `BTreeMap<String, String>` parameter token map.
//!
//! ## 4. Postconditions
//! - Returns the rendered Markdown assessment table and safety objective roster.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O((T + R + C + K) * log O) over tokens T, requirements R, constraints C, calcs K, and objectives O.
//! - Space Complexity: O(O) bounded `BTreeMap` and Markdown string buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Emits a clean empty-package summary when no objectives or formal AST obligations exist.
//! - State: Pure functional string synthesis.
//! - Action: Uses `.strip_prefix()`, `.strip_suffix()`, and `.saturating_add(1)` with zero panics or regexes.
//! - Resource: Bounded heap allocations proportional to AST objective count.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-SAFETY-SUITE-RENDER-REGULATORY-OBJECTIVES-ASSESSMENT] | SysML v2: PackageDef(regulatory_objectives)

use deap_core::sysml_cst::CalcDef;

use super::pillars::{
    collect_declared_token_objectives, collect_token_classification_rows, RegulatoryObjectiveRow,
};
use super::*;

/// Appends uncovered `RequirementDef` safety objectives into `rows_by_idx`.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-REGULATORY-OBJECTIVES-ASSESSMENT]
fn append_requirement_objectives(
    pkg: &PackageDef,
    display_prefix: &str,
    default_assurance: &str,
    rows_by_idx: &mut BTreeMap<usize, RegulatoryObjectiveRow>,
) {
    let mut next_free_idx = 1usize;
    let mut seen_reqs = BTreeSet::new();
    for sub_pkg in pkg.get_all_packages() {
        for req in &sub_pkg.requirement_defs {
            if !seen_reqs.insert(req.name.clone()) {
                continue;
            }
            let citation = format!("RequirementDef({})", sanitize_markdown_cell(&req.name));
            if rows_by_idx
                .values()
                .any(|r| r.evidence.contains(&citation) || r.objective.contains(&req.name))
            {
                continue;
            }
            while rows_by_idx.contains_key(&next_free_idx) {
                next_free_idx = next_free_idx.saturating_add(1);
            }
            let obj_text = req
                .doc
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(sanitize_markdown_cell)
                .unwrap_or_else(|| {
                    format!("Satisfy requirement {}", sanitize_markdown_cell(&req.name))
                });
            rows_by_idx.insert(
                next_free_idx,
                RegulatoryObjectiveRow {
                    objective_id: format!("{}{:02}", display_prefix, next_free_idx),
                    objective: obj_text,
                    robustness: "Verified".to_string(),
                    integrity: "Requirement Bound".to_string(),
                    assurance_level: default_assurance.to_string(),
                    evidence: citation,
                },
            );
            next_free_idx = next_free_idx.saturating_add(1);
        }
    }
}

/// Appends uncovered `ConstraintDef` safety objectives into `rows_by_idx`.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-REGULATORY-OBJECTIVES-ASSESSMENT]
fn append_constraint_objectives(
    all_constraints: &[ConstraintDef],
    display_prefix: &str,
    default_assurance: &str,
    rows_by_idx: &mut BTreeMap<usize, RegulatoryObjectiveRow>,
) {
    let mut next_free_idx = 1usize;
    for con in all_constraints {
        let citation = format!("ConstraintDef({})", sanitize_markdown_cell(&con.name));
        if rows_by_idx
            .values()
            .any(|r| r.evidence.contains(&citation) || r.objective.contains(&con.name))
        {
            continue;
        }
        while rows_by_idx.contains_key(&next_free_idx) {
            next_free_idx = next_free_idx.saturating_add(1);
        }
        let obj_text = if con.expression.trim().is_empty() {
            format!("Enforce constraint {}", sanitize_markdown_cell(&con.name))
        } else {
            format!(
                "Enforce constraint {}: {}",
                sanitize_markdown_cell(&con.name),
                sanitize_markdown_cell(&con.expression)
            )
        };
        rows_by_idx.insert(
            next_free_idx,
            RegulatoryObjectiveRow {
                objective_id: format!("{}{:02}", display_prefix, next_free_idx),
                objective: obj_text,
                robustness: "Run-Time Monitored".to_string(),
                integrity: "Formal Invariant".to_string(),
                assurance_level: default_assurance.to_string(),
                evidence: citation,
            },
        );
        next_free_idx = next_free_idx.saturating_add(1);
    }
}

/// Appends uncovered `CalcDef` quantitative safety objectives into `rows_by_idx`.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-REGULATORY-OBJECTIVES-ASSESSMENT]
fn append_calc_objectives(
    all_calcs: &[CalcDef],
    display_prefix: &str,
    default_assurance: &str,
    rows_by_idx: &mut BTreeMap<usize, RegulatoryObjectiveRow>,
) {
    let mut next_free_idx = 1usize;
    for calc in all_calcs {
        let citation = format!("CalcDef({})", sanitize_markdown_cell(&calc.name));
        if rows_by_idx
            .values()
            .any(|r| r.evidence.contains(&citation) || r.objective.contains(&calc.name))
        {
            continue;
        }
        while rows_by_idx.contains_key(&next_free_idx) {
            next_free_idx = next_free_idx.saturating_add(1);
        }
        let obj_text = calc
            .expression
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map_or_else(
                || format!("Evaluate calculation {}", sanitize_markdown_cell(&calc.name)),
                |expr| {
                    format!(
                        "Evaluate calculation {}: {}",
                        sanitize_markdown_cell(&calc.name),
                        sanitize_markdown_cell(expr)
                    )
                },
            );
        rows_by_idx.insert(
            next_free_idx,
            RegulatoryObjectiveRow {
                objective_id: format!("{}{:02}", display_prefix, next_free_idx),
                objective: obj_text,
                robustness: "Analytical".to_string(),
                integrity: "Quantitative Bound".to_string(),
                assurance_level: default_assurance.to_string(),
                evidence: citation,
            },
        );
        next_free_idx = next_free_idx.saturating_add(1);
    }
}

/// Resolves the default assurance level string from `primary_integrity`, `tokens`, or AST formal bounds.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-REGULATORY-OBJECTIVES-ASSESSMENT]
fn resolve_default_assurance_level(
    primary_integrity: Option<&str>,
    tokens: &BTreeMap<String, String>,
    all_constraints: &[ConstraintDef],
    all_calcs: &[CalcDef],
) -> String {
    primary_integrity
        .or_else(|| find_token_ci(tokens, "AssuranceLevel"))
        .or_else(|| find_token_ci(tokens, "DefaultAssuranceLevel"))
        .or_else(|| find_token_ci(tokens, "IntegrityLevel"))
        .or_else(|| find_token_prefix_ci(tokens, "IntegrityLevel_"))
        .map(sanitize_markdown_cell)
        .unwrap_or_else(|| {
            if all_constraints.iter().any(|c| c.is_assertion) {
                "SIL-3 / High Assurance (Derived: AST Assert Invariant)".to_string()
            } else if !all_constraints.is_empty() || !all_calcs.is_empty() {
                "SIL-2 / Elevated Assurance (Derived: AST Formal Bound)".to_string()
            } else {
                "SIL-1 / Baseline Assurance (Derived: AST Schema)".to_string()
            }
        })
}

/// Appends the regulatory assessment summary header table onto `out`.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-REGULATORY-OBJECTIVES-ASSESSMENT]
fn render_assessment_header(
    base_pkg: &str,
    part_count: usize,
    constraint_count: usize,
    calc_count: usize,
    classification_rows: &[(String, String)],
) -> String {
    let mut out = String::new();
    out.push_str("# Domain Regulatory Objectives Assessment & Safety Objective Roster\n\n");
    out.push_str("| Assessment Field | Value |\n| :--- | :--- |\n");
    out.push_str(&format!(
        "| Target System Package | {} |\n",
        sanitize_markdown_cell(base_pkg)
    ));
    out.push_str(&format!("| Declared Architectural Components | {} |\n", part_count));
    out.push_str(&format!(
        "| Declared Formal Constraints & Calculations | {} constraints, {} calculations |\n",
        constraint_count, calc_count
    ));
    for (field, value) in classification_rows {
        out.push_str(&format!("| {} | {} |\n", field, value));
    }
    out.push('\n');
    out
}

/// Appends the `Declared Regulatory Safety Objectives` Markdown table or empty-package note onto `out`.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-REGULATORY-OBJECTIVES-ASSESSMENT]
fn append_objectives_table(
    base_pkg: &str,
    rows_by_idx: &BTreeMap<usize, RegulatoryObjectiveRow>,
    out: &mut String,
) {
    out.push_str("## Declared Regulatory Safety Objectives\n\n");
    if rows_by_idx.is_empty() {
        out.push_str(&format!(
            "No regulatory safety objectives, `requirement def`, `constraint def`, or `calc def` declarations present in package `{}`.\n\n",
            sanitize_markdown_cell(base_pkg)
        ));
    } else {
        out.push_str("| Objective ID | Objective | Robustness | Integrity | Assurance Level | Evidence |\n| :--- | :--- | :--- | :--- | :--- | :--- |\n");
        for row in rows_by_idx.values() {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                row.objective_id,
                row.objective,
                row.robustness,
                row.integrity,
                row.assurance_level,
                row.evidence
            ));
        }
        out.push('\n');
    }
}

/// Renders the domain-agnostic regulatory objectives and integrity level assessment (`06_REGULATORY_OBJECTIVES_ASSESSMENT.md`).
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-REGULATORY-OBJECTIVES-ASSESSMENT]
pub(super) fn render_regulatory_objectives_assessment(
    pkg: &PackageDef,
    tokens: &BTreeMap<String, String>,
) -> String {
    let base_pkg = if pkg.name.trim().is_empty() {
        "System"
    } else {
        pkg.name.trim()
    };
    let part_count = pkg
        .get_all_parts()
        .iter()
        .filter(|p| !p.name.starts_with('_'))
        .count();
    let all_constraints = pkg.get_all_constraints();
    let all_calcs = pkg.get_all_calcs();

    let raw_prefix = tokens
        .get("REGULATORY_OBJECTIVE_PREFIX")
        .map_or("OBJ-", String::as_str);
    let trimmed_stem = raw_prefix.trim().trim_end_matches(['-', '_']);
    let prefix_stem = if trimmed_stem.is_empty() {
        "OBJ"
    } else {
        trimmed_stem
    };
    let display_prefix = format!("{}-", prefix_stem);
    let (classification_rows, primary_integrity) = collect_token_classification_rows(tokens);

    let mut out = render_assessment_header(
        base_pkg,
        part_count,
        all_constraints.len(),
        all_calcs.len(),
        &classification_rows,
    );

    let default_assurance = resolve_default_assurance_level(
        primary_integrity,
        tokens,
        &all_constraints,
        &all_calcs,
    );

    let mut rows_by_idx = collect_declared_token_objectives(
        tokens,
        prefix_stem,
        &display_prefix,
        &default_assurance,
    );
    append_requirement_objectives(pkg, &display_prefix, &default_assurance, &mut rows_by_idx);
    append_constraint_objectives(
        &all_constraints,
        &display_prefix,
        &default_assurance,
        &mut rows_by_idx,
    );
    append_calc_objectives(
        &all_calcs,
        &display_prefix,
        &default_assurance,
        &mut rows_by_idx,
    );

    append_objectives_table(base_pkg, &rows_by_idx, &mut out);
    out
}
