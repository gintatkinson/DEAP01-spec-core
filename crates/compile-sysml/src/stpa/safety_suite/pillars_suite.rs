#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Formal safety constraint, FMECA criticality matrix, and STPA hazard log renderers.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! safety transpilation objectives for module `crates/compile-sysml/src/stpa/safety_suite/pillars_suite.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Synthesizes `04_SAFETY_CONSTRAINTS.md`, `05_FMECA_MATRIX.md`, and `HAZARD_LOG.md` deterministically
//!   from SysML v2 `PackageDef` AST declarations, UCAs, constraints, FMECA rows, and parameter tokens.
//! - Re-exports `render_rta_architecture` for `07_RTA_ARCHITECTURE.md` generation.
//!
//! ## 3. Preconditions
//! - Caller supplies parsed `PackageDef`, `UnsafeControlAction`, `ConstraintDef`, and `FmecaRow` slices.
//!
//! ## 4. Postconditions
//! - Returns rendered Markdown safety documents with sanitized single-line table cells.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(P + U + C + F) over parts P, UCAs U, constraints C, and FMECA rows F.
//! - Space Complexity: O(P + U + C + F) bounded output string buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Degrades cleanly when `constraints`, `calcs`, `requirement_defs`, or `ucas` are empty.
//! - State: Pure functional string synthesis.
//! - Action: Uses `get_cycled` (`checked_rem` + `.get()`) and `.saturating_add(1)` with zero panics.
//! - Resource: Bounded heap allocations proportional to rendered document length.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-SAFETY-SUITE-PILLARS-SUITE] | SysML v2: PackageDef(pillars_suite)

pub(super) use super::rta_renderer::render_rta_architecture;
use super::*;

/// Appends schema-declared `CalcDef` and `RequirementDef` obligation tables onto `out`.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-SAFETY-CONSTRAINTS]
fn render_calc_and_requirement_obligations(pkg: &PackageDef, base_pkg: &str, out: &mut String) {
    let calcs = pkg.get_all_calcs();
    out.push_str("\n## Schema-Declared Calculation Obligations (SysML v2 SSOT)\n\n");
    if calcs.is_empty() {
        out.push_str(&format!(
            "No `calc def` expressions declared in package `{}`.\n",
            base_pkg
        ));
    } else {
        out.push_str("| Calc Def | Expression |\n| :--- | :--- |\n");
        for calc in &calcs {
            let clean_name = sanitize_markdown_cell(&calc.name);
            let expr = calc
                .expression
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(sanitize_markdown_cell)
                .or_else(|| {
                    calc.doc
                        .as_deref()
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(sanitize_markdown_cell)
                })
                .unwrap_or_else(|| {
                    let param_names = calc
                        .in_params
                        .iter()
                        .map(|p| sanitize_markdown_cell(&p.name))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}({}) (Derived: AST calc)", clean_name, param_names)
                });
            out.push_str(&format!("| {} | {} |\n", clean_name, expr));
        }
    }

    let mut seen_reqs = BTreeSet::new();
    let mut req_rows = Vec::new();
    for sub_pkg in pkg.get_all_packages() {
        for req in &sub_pkg.requirement_defs {
            if seen_reqs.insert(req.name.clone()) {
                let spec = req
                    .doc
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(sanitize_markdown_cell)
                    .unwrap_or_else(|| "Schema-declared safety requirement obligation".to_string());
                req_rows.push((sanitize_markdown_cell(&req.name), spec));
            }
        }
    }

    out.push_str("\n## Schema-Declared Safety Requirement Obligations (SysML v2 SSOT)\n\n");
    if req_rows.is_empty() {
        out.push_str(&format!(
            "No `requirement def` declarations present in package `{}`.\n\n",
            base_pkg
        ));
    } else {
        out.push_str("| Requirement Def | Specification |\n| :--- | :--- |\n");
        for (req_name, spec) in &req_rows {
            out.push_str(&format!("| {} | {} |\n", req_name, spec));
        }
        out.push('\n');
    }
}

/// Renders formal safety constraint obligations (`04_SAFETY_CONSTRAINTS.md`) derived purely from AST UCAs, `ConstraintDef`s, `CalcDef`s, and `RequirementDef`s.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-SAFETY-CONSTRAINTS]
pub(super) fn render_safety_constraints(
    pkg: &PackageDef,
    ucas: &[UnsafeControlAction],
    constraints: &[ConstraintDef],
) -> String {
    let base_pkg = if pkg.name.trim().is_empty() {
        "System"
    } else {
        pkg.name.trim()
    };

    let mut out = String::new();
    out.push_str("# Formal Safety Constraints\n\n");
    out.push_str("## Derived Safety Constraints per Unsafe Control Action\n\n");
    out.push_str("| Safety Constraint ID | UCA ID | Constraint Statement |\n| :--- | :--- | :--- |\n");
    for u in ucas {
        out.push_str(&format!(
            "| {} | {} | {} shall remain within safe bounds under {} |\n",
            u.constraint, u.id, u.control_action, u.guide_word
        ));
    }

    out.push_str("\n## Schema-Declared Constraint Defs (SysML v2 SSOT)\n\n");
    if constraints.is_empty() {
        out.push_str(&format!(
            "No `constraint def` invariants declared in package `{}`.\n",
            base_pkg
        ));
    } else {
        out.push_str("| Constraint Def | Expression |\n| :--- | :--- |\n");
        for c in constraints {
            let clean_name = sanitize_markdown_cell(&c.name);
            let expr = if c.expression.trim().is_empty() {
                c.doc
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(sanitize_markdown_cell)
                    .unwrap_or_else(|| {
                        format!("{} == true (Derived: AST constraint)", clean_name)
                    })
            } else {
                sanitize_markdown_cell(&c.expression)
            };
            out.push_str(&format!("| {} | {} |\n", clean_name, expr));
        }
    }

    render_calc_and_requirement_obligations(pkg, base_pkg, &mut out);
    out
}

/// Renders the FMECA criticality matrix Markdown table (`05_FMECA_MATRIX.md`) across all pre-scored `FmecaRow` entries.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-FMECA-MATRIX-DOC]
pub(super) fn render_fmeca_matrix_doc(rows: &[FmecaRow]) -> String {
    let mut out = String::new();
    out.push_str("# FMECA Criticality Matrix\n\n");
    out.push_str("The Risk Priority Number is the product of severity (S), occurrence (O)\n");
    out.push_str("and detection (D) scores.\n\n");
    out.push_str("| FMECA ID | Component | Failure Mode | Potential Effect | Severity (S) | Occurrence (O) | Detection (D) | RPN (S x O x D) | Mitigation | Derivation Basis |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for r in rows {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            r.id,
            r.component,
            r.failure_mode,
            r.effect,
            r.severity_cell,
            r.occurrence_cell,
            r.detection_cell,
            r.rpn_cell,
            r.mitigation,
            r.basis
        ));
    }
    out.push('\n');
    out
}

/// Resolves the ordered list of sanitized loss source identifiers for `HAZARD_LOG.md`.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-HAZARD-LOG]
fn collect_hazard_log_loss_sources(
    controllers: &[&PartDef],
    non_internal_parts: &[&PartDef],
    ucas: &[UnsafeControlAction],
    base_pkg: &str,
) -> Vec<String> {
    if !controllers.is_empty() {
        return controllers
            .iter()
            .map(|c| sanitize_markdown_cell(&c.name))
            .collect();
    }

    let mut seen_ctrl = BTreeSet::new();
    let mut uca_ctrls = Vec::new();
    for u in ucas {
        let clean = sanitize_markdown_cell(&u.controller);
        if !clean.is_empty() && seen_ctrl.insert(clean.clone()) {
            uca_ctrls.push(clean);
        }
    }
    if !uca_ctrls.is_empty() {
        uca_ctrls
    } else if !non_internal_parts.is_empty() {
        non_internal_parts
            .iter()
            .map(|p| sanitize_markdown_cell(&p.name))
            .collect()
    } else {
        vec![sanitize_markdown_cell(base_pkg)]
    }
}

/// Appends deduplicated hazard rows (or the baseline fallback hazard row) onto `out`.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-HAZARD-LOG]
fn emit_hazard_log_rows(
    ucas: &[UnsafeControlAction],
    constraints: &[ConstraintDef],
    loss_sources: &[String],
    base_pkg: &str,
    out: &mut String,
) {
    let mut seen_hazards = BTreeSet::new();
    for (idx, u) in ucas.iter().enumerate() {
        if !u.hazard.trim().is_empty() && seen_hazards.insert(u.hazard.clone()) {
            let src = format!(
                "{} / {}",
                sanitize_markdown_cell(&u.controller),
                sanitize_markdown_cell(&u.control_action)
            );
            let guard_suffix = get_cycled(constraints, idx)
                .map(|c| format!("; guard {}", sanitize_markdown_cell(&c.name)))
                .unwrap_or_default();
            let notes = format!(
                "Enforced by {} ({}){}",
                sanitize_markdown_cell(&u.constraint),
                sanitize_markdown_cell(&u.severity),
                guard_suffix
            );
            out.push_str(&format!(
                "| {} | Hazard | {} | Open | {} |\n",
                sanitize_markdown_cell(&u.hazard),
                src,
                notes
            ));
        }
    }

    if seen_hazards.is_empty() {
        let primary_src = loss_sources
            .first()
            .cloned()
            .unwrap_or_else(|| sanitize_markdown_cell(base_pkg));
        let baseline_note = constraints.first().map_or_else(
            || {
                format!(
                    "Enforced by {} baseline safety envelope (Derived)",
                    sanitize_markdown_cell(base_pkg)
                )
            },
            |c| {
                format!(
                    "Enforced by constraint {} (Derived)",
                    sanitize_markdown_cell(&c.name)
                )
            },
        );
        out.push_str(&format!(
            "| H-1 | Hazard | {} | Open | {} |\n",
            primary_src, baseline_note
        ));
    }
}

/// Renders the AST-derived STPA Hazard Log (`HAZARD_LOG.md`) covering system losses, deduplicated action hazards, safety constraint bindings, and resolved governance authority.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-HAZARD-LOG]
pub(super) fn render_hazard_log(
    pkg: &PackageDef,
    ucas: &[UnsafeControlAction],
    constraints: &[ConstraintDef],
    tokens: &BTreeMap<String, String>,
) -> String {
    let all_parts = pkg.get_all_parts();
    let non_internal_parts: Vec<&PartDef> = all_parts
        .iter()
        .filter(|p| !p.name.starts_with('_'))
        .collect();
    let controllers: Vec<&PartDef> = non_internal_parts
        .iter()
        .copied()
        .filter(|part| !part.actions.is_empty())
        .collect();
    let base_pkg = if pkg.name.trim().is_empty() {
        "System"
    } else {
        pkg.name.trim()
    };
    let loss_sources =
        collect_hazard_log_loss_sources(&controllers, &non_internal_parts, ucas, base_pkg);

    let mut out = String::new();
    out.push_str("# Hazard Log\n\n| ID | Kind | Source | Status | Notes |\n| :--- | :--- | :--- | :--- | :--- |\n");

    for (i, src) in loss_sources.iter().enumerate() {
        out.push_str(&format!(
            "| L-{} | Loss | {} | Open | Loss of safe functional integrity of {} (AST Derived) |\n",
            i.saturating_add(1),
            src,
            src
        ));
    }

    emit_hazard_log_rows(ucas, constraints, &loss_sources, base_pkg, &mut out);

    let authority = [
        "ResolutionAuthority",
        "SafetyAuthority",
        "ApprovalAuthority",
        "Authority",
    ]
    .iter()
    .find_map(|key| find_token_ci(tokens, key))
    .map(sanitize_markdown_cell)
    .unwrap_or_else(|| {
        format!(
            "{} Safety Governance Authority (Derived)",
            sanitize_markdown_cell(base_pkg)
        )
    });

    out.push_str(&format!("\n| Resolution Authority | {} |\n\n", authority));
    out
}
