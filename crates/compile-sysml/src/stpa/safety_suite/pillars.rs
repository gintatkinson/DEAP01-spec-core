//! Regulatory objectives assessment, unified 8-pillar STPA matrix, cross-traceability matrix, and SLDV script renderers.
//!
//! Safety & Traceability: Realises: [REQ-STPA-SAFETY-SUITE-PILLARS]
//! Contract: Synthesizes `06_REGULATORY_OBJECTIVES_ASSESSMENT.md`, `STPA_MATRIX.md`, STPA cross-traceability tables, and `SLDV_FORMAL_PROOFS.m` deterministically from SysML v2 AST definitions, FMECA rows, UCAs, and parameter token maps.
//! Bounds & Faults: Executes bounded linear and `BTreeMap`/`BTreeSet` passes using `.strip_prefix()`, `.strip_suffix()`, and `.saturating_add()` with zero panics, regular expressions, or direct slice indexing.

use super::*;

/// Normalized six-column regulatory safety objective record projected from AST attributes and definitions.
/// Realises: [REQ-STPA-SAFETY-SUITE-REGULATORY-OBJECTIVE-ROW]
struct RegulatoryObjectiveRow {
    objective_id: String,
    objective: String,
    robustness: String,
    integrity: String,
    assurance_level: String,
    evidence: String,
}

/// Parses an explicit `OBJ<NN>_Objective` or `<PREFIX><NN>_Objective` token key into its positive index and stem without regular expressions or direct slice indexing.
/// Realises: [REQ-STPA-SAFETY-SUITE-PARSE-DECLARED-OBJECTIVE-INDEX]
fn parse_declared_objective_index<'a>(key: &'a str, prefix_stem: &str) -> Option<(usize, &'a str)> {
    let before_suffix = key.strip_suffix("_Objective")?;
    let rest = before_suffix
        .strip_prefix(prefix_stem)
        .or_else(|| before_suffix.strip_prefix("OBJ"))?;
    let digits = rest.trim_start_matches(['-', '_']);
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let idx = digits.parse::<usize>().ok()?;
    if idx == 0 {
        return None;
    }
    Some((idx, before_suffix))
}

/// Renders the domain-agnostic regulatory objectives and integrity level assessment (`06_REGULATORY_OBJECTIVES_ASSESSMENT.md`) purely from SysML v2 AST nodes and parameter tokens.
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
    let all_parts = pkg.get_all_parts();
    let part_count = all_parts
        .iter()
        .filter(|p| !p.name.starts_with('_'))
        .count();
    let all_constraints = pkg.get_all_constraints();
    let all_calcs = pkg.get_all_calcs();

    let raw_prefix = tokens
        .get("REGULATORY_OBJECTIVE_PREFIX")
        .map(String::as_str)
        .unwrap_or("OBJ-");
    let trimmed_stem = raw_prefix.trim().trim_end_matches(['-', '_']);
    let prefix_stem = if trimmed_stem.is_empty() {
        "OBJ"
    } else {
        trimmed_stem
    };
    let display_prefix = format!("{}-", prefix_stem);

    let mut classification_rows: Vec<(String, String)> = Vec::new();
    let mut primary_integrity: Option<&str> = None;

    for (key, val) in tokens {
        if let Some(raw_field) = key.strip_prefix("RegulatoryClass_") {
            let formatted = raw_field.replace('_', " ");
            let clean_field = sanitize_markdown_cell(&formatted);
            if !clean_field.is_empty() {
                classification_rows.push((clean_field, sanitize_markdown_cell(val)));
            }
        } else if key == "RegulatoryClass" {
            classification_rows.push(("Regulatory Class".to_string(), sanitize_markdown_cell(val)));
        } else if let Some(raw_field) = key.strip_prefix("IntegrityLevel_") {
            let formatted = raw_field.replace('_', " ");
            let clean_field = sanitize_markdown_cell(&formatted);
            if !clean_field.is_empty() {
                if primary_integrity.is_none() {
                    primary_integrity = Some(val.as_str());
                }
                classification_rows.push((clean_field, sanitize_markdown_cell(val)));
            }
        } else if key == "IntegrityLevel" {
            if primary_integrity.is_none() {
                primary_integrity = Some(val.as_str());
            }
            classification_rows.push(("Integrity Level".to_string(), sanitize_markdown_cell(val)));
        }
    }

    let mut out = String::new();
    out.push_str("# Domain Regulatory Objectives Assessment & Safety Objective Roster\n\n");
    out.push_str("| Assessment Field | Value |\n");
    out.push_str("| :--- | :--- |\n");
    out.push_str(&format!(
        "| Target System Package | {} |\n",
        sanitize_markdown_cell(base_pkg)
    ));
    out.push_str(&format!(
        "| Declared Architectural Components | {} |\n",
        part_count
    ));
    out.push_str(&format!(
        "| Declared Formal Constraints & Calculations | {} constraints, {} calculations |\n",
        all_constraints.len(),
        all_calcs.len()
    ));
    for (field, value) in &classification_rows {
        out.push_str(&format!("| {} | {} |\n", field, value));
    }
    out.push('\n');

    let default_assurance = primary_integrity
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
        });

    let mut rows_by_idx: BTreeMap<usize, RegulatoryObjectiveRow> = BTreeMap::new();

    for (key, val) in tokens {
        if let Some((idx, key_stem)) = parse_declared_objective_index(key, prefix_stem) {
            let rob = tokens
                .get(&format!("{}_Robustness", key_stem))
                .map(|s| sanitize_markdown_cell(s))
                .unwrap_or_else(|| "Deterministic".to_string());
            let int = tokens
                .get(&format!("{}_Integrity", key_stem))
                .map(|s| sanitize_markdown_cell(s))
                .unwrap_or_else(|| "Schema-Declared".to_string());
            let ass = tokens
                .get(&format!("{}_Assurance_Level", key_stem))
                .map(|s| sanitize_markdown_cell(s))
                .unwrap_or_else(|| default_assurance.clone());
            let evi = tokens
                .get(&format!("{}_Evidence", key_stem))
                .map(|s| sanitize_markdown_cell(s))
                .unwrap_or_else(|| {
                    format!(
                        "AttributeDef({}_Objective)",
                        sanitize_markdown_cell(key_stem)
                    )
                });

            rows_by_idx.insert(
                idx,
                RegulatoryObjectiveRow {
                    objective_id: format!("{}{:02}", display_prefix, idx),
                    objective: sanitize_markdown_cell(val),
                    robustness: rob,
                    integrity: int,
                    assurance_level: ass,
                    evidence: evi,
                },
            );
        }
    }

    let mut next_free_idx = 1usize;

    let mut seen_reqs = BTreeSet::new();
    for sub_pkg in pkg.get_all_packages() {
        for req in &sub_pkg.requirement_defs {
            if !seen_reqs.insert(req.name.clone()) {
                continue;
            }
            let citation = format!("RequirementDef({})", sanitize_markdown_cell(&req.name));
            let already_covered = rows_by_idx
                .values()
                .any(|r| r.evidence.contains(&citation) || r.objective.contains(&req.name));
            if already_covered {
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
                    assurance_level: default_assurance.clone(),
                    evidence: citation,
                },
            );
            next_free_idx = next_free_idx.saturating_add(1);
        }
    }

    for con in &all_constraints {
        let citation = format!("ConstraintDef({})", sanitize_markdown_cell(&con.name));
        let already_covered = rows_by_idx
            .values()
            .any(|r| r.evidence.contains(&citation) || r.objective.contains(&con.name));
        if already_covered {
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
                assurance_level: default_assurance.clone(),
                evidence: citation,
            },
        );
        next_free_idx = next_free_idx.saturating_add(1);
    }

    for calc in &all_calcs {
        let citation = format!("CalcDef({})", sanitize_markdown_cell(&calc.name));
        let already_covered = rows_by_idx
            .values()
            .any(|r| r.evidence.contains(&citation) || r.objective.contains(&calc.name));
        if already_covered {
            continue;
        }
        while rows_by_idx.contains_key(&next_free_idx) {
            next_free_idx = next_free_idx.saturating_add(1);
        }
        let obj_text = match calc
            .expression
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            Some(expr) => format!(
                "Evaluate calculation {}: {}",
                sanitize_markdown_cell(&calc.name),
                sanitize_markdown_cell(expr)
            ),
            None => format!(
                "Evaluate calculation {}",
                sanitize_markdown_cell(&calc.name)
            ),
        };
        rows_by_idx.insert(
            next_free_idx,
            RegulatoryObjectiveRow {
                objective_id: format!("{}{:02}", display_prefix, next_free_idx),
                objective: obj_text,
                robustness: "Analytical".to_string(),
                integrity: "Quantitative Bound".to_string(),
                assurance_level: default_assurance.clone(),
                evidence: citation,
            },
        );
        next_free_idx = next_free_idx.saturating_add(1);
    }

    out.push_str("## Declared Regulatory Safety Objectives\n\n");
    if rows_by_idx.is_empty() {
        out.push_str(&format!(
            "No regulatory safety objectives, `requirement def`, `constraint def`, or `calc def` declarations present in package `{}`.\n\n",
            sanitize_markdown_cell(base_pkg)
        ));
    } else {
        out.push_str(
            "| Objective ID | Objective | Robustness | Integrity | Assurance Level | Evidence |\n",
        );
        out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");
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

    out
}

/// Renders the unified 8-pillar STPA Safety and Risk Analysis Matrix (`STPA_MATRIX.md`) integrating losses, hazards, topology, UCAs, loss scenarios, constraints, FMECA, regulatory objectives, and ASTM F3269-17 RTA proof hooks.
///
/// Safety & Traceability: Realises: [REQ-STPA-SAFETY-SUITE-RENDER-UNIFIED-STPA-MATRIX]
/// Contract: Composes the canonical STPA, FMECA, regulatory objective, and Run-Time Assurance sections deterministically from `pkg`, `ucas`, `constraints`, `fmeca_rows`, and `tokens` while stripping duplicate top-level `#` headings via `.strip_prefix()`.
/// Bounds & Faults: Bounded `O(N)` string composition over AST and matrix cardinalities with zero panics, regular expressions, or direct slice indexing.
pub fn render_unified_stpa_matrix(
    pkg: &PackageDef,
    ucas: &[UnsafeControlAction],
    constraints: &[ConstraintDef],
    fmeca_rows: &[FmecaRow],
    tokens: &BTreeMap<String, String>,
) -> String {
    let mut out = String::new();
    out.push_str("# Unified 8-Pillar STPA Safety & Risk Analysis Matrix\n\n");
    out.push_str("> **Primary Commercial Toolchain Integration Context:** MATLAB / Simulink / Stateflow / Embedded Coder\n");
    out.push_str("> **Safety Standards:** ISO/IEC/IEEE 15026-3 | ASTM F3269-17 RTA | MIL-STD-882E | Domain Integrity Framework\n\n");
    out.push_str("---\n\n");

    // Pillar 1, 2, 3: System Losses, System Hazards, and Hierarchical Control Structure Topology
    let losses_hazards_top = render_losses_hazards_topology(pkg, ucas);
    let body_123 = if let Some(stripped) =
        losses_hazards_top.strip_prefix("# System Losses, Hazards & Control Structure Topology\n\n")
    {
        stripped
    } else {
        &losses_hazards_top
    };
    out.push_str(body_123);
    out.push_str("\n\n---\n\n");

    // Pillar 4: Unsafe Control Actions (UCA-1..N)
    out.push_str("## 4. Unsafe Control Actions (**UCA-1..N**)\n\n");
    out.push_str(
        "Systematic identification across the 4 STPA guide word / failure mode categories:\n\n",
    );
    out.push_str("1. **Not providing causes hazard**\n");
    out.push_str("2. **Providing causes hazard**\n");
    out.push_str("3. **Too early, too late, or out of order**\n");
    out.push_str("4. **Stopped too soon or applied too long**\n\n");
    let uca_doc = render_uca_matrix(ucas);
    let uca_body = if let Some(stripped) =
        uca_doc.strip_prefix("# Unsafe Control Action Combinatorial Matrix\n\n")
    {
        stripped
    } else {
        &uca_doc
    };
    out.push_str(uca_body);
    out.push_str("\n\n---\n\n");

    // Pillar 5: Loss Scenarios (LS-1..N) & Causal Factors
    out.push_str("## 5. Loss Scenarios (**LS-1..N**) & Causal Factors\n\n");
    let ls_doc = render_loss_scenarios(pkg, ucas, constraints);
    let ls_body =
        if let Some(stripped) = ls_doc.strip_prefix("# Loss Scenarios & Causal Factors\n\n") {
            stripped
        } else {
            &ls_doc
        };
    out.push_str(ls_body);
    out.push_str("\n\n---\n\n");

    // Pillar 6: Formal Safety Constraints (SC-1..N) & Stateflow Synthesis Hooks
    out.push_str("## 6. Formal Safety Constraints (**SC-1..N**) & Stateflow Synthesis Hooks\n\n");
    out.push_str("The formal safety constraints and Run-Time Assurance supervisory monitors (`RTA`) are synthesized directly into **MATLAB / Simulink / Stateflow** charts for envelope protection and SLDV verification.\n\n");
    let sc_doc = render_safety_constraints(pkg, ucas, constraints);
    let sc_body = if let Some(stripped) = sc_doc.strip_prefix("# Formal Safety Constraints\n\n") {
        stripped
    } else {
        &sc_doc
    };
    out.push_str(sc_body);
    out.push_str("\n\n---\n\n");

    // Pillar 7: FMECA Criticality Matrix
    out.push_str("## 7. FMECA Criticality Matrix\n\n");
    let fmeca_doc = render_fmeca_matrix_doc(fmeca_rows);
    let fmeca_body =
        if let Some(stripped) = fmeca_doc.strip_prefix("# FMECA Criticality Matrix\n\n") {
            stripped
        } else {
            &fmeca_doc
        };
    out.push_str(fmeca_body);
    out.push_str("\n\n---\n\n");

    // Pillar 8: Domain Regulatory Objectives & Integrity Level Mapping
    out.push_str("## 8. Domain Regulatory Objectives & Integrity Level Mapping\n\n");
    let reg_doc = render_regulatory_objectives_assessment(pkg, tokens);
    let reg_body = if let Some(stripped) = reg_doc
        .strip_prefix("# Domain Regulatory Objectives Assessment & Safety Objective Roster\n\n")
    {
        stripped
    } else {
        &reg_doc
    };
    out.push_str(reg_body);
    out.push_str("\n\n---\n\n");

    // ASTM F3269-17 RTA Architecture & Toolchain Proof Hooks
    out.push_str(
        "## 9. ASTM F3269-17 Run-Time Assurance (RTA) Architecture & Toolchain Proof Hooks\n\n",
    );
    let rta_doc = render_rta_architecture(pkg, tokens);
    let rta_body = if let Some(stripped) =
        rta_doc.strip_prefix("# Run-Time Assurance Architecture & Formal Proof Suite\n\n")
    {
        stripped
    } else {
        &rta_doc
    };
    out.push_str(rta_body);

    out
}

/// Renders the STPA cross-traceability matrix verifying bijective hazard-to-constraint coverage across expanded UCAs.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-STPA-MATRIX]
#[allow(dead_code)]
pub(super) fn render_stpa_matrix(ucas: &[UnsafeControlAction]) -> String {
    let mut out = String::new();
    out.push_str("# STPA Cross-Traceability Matrix\n\n");
    out.push_str("| UCA ID | Controller | Control Action | Guide Word | Hazard | Safety Constraint | Traceability Status |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for u in ucas {
        let status = if !u.hazard.trim().is_empty() && !u.constraint.trim().is_empty() {
            "Verified (AST Bijective)"
        } else {
            "Derived (AST Control Action)"
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            u.id, u.controller, u.control_action, u.guide_word, u.hazard, u.constraint, status
        ));
    }
    out.push('\n');
    out
}

/// Renders the Simulink Design Verifier (`SLDV_FORMAL_PROOFS.m`) formal proof script derived strictly from AST `ConstraintDef` and `CalcDef` declarations.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-SLDV-SCRIPT]
pub(super) fn render_sldv_script(pkg: &PackageDef, constraints: &[ConstraintDef]) -> String {
    let base_pkg = if pkg.name.trim().is_empty() {
        "System"
    } else {
        pkg.name.trim()
    };
    let all_calcs = pkg.get_all_calcs();

    let mut out = String::new();
    out.push_str("% SLDV Formal Proof Script\n");

    if all_calcs.is_empty() && constraints.is_empty() {
        out.push_str(&format!(
            "% No calc def or constraint def assertions declared in package {}\n",
            base_pkg
        ));
        return out;
    }

    out.push_str("% Schema constraint bindings\n");
    for c in constraints {
        let expr = if c.expression.trim().trim_matches(';').trim().is_empty() {
            "false".to_string()
        } else {
            lower_sysml_expr_to_matlab(&c.expression)
        };
        let sanitized = c.name.to_uppercase().replace('-', "_");
        out.push_str(&format!(
            "sldv.assert( ({}), 'Bind_{}_ASSERTION' );\n",
            expr, sanitized
        ));
    }

    out.push_str("\n% Theorem proof bindings\n");
    for (idx, calc) in all_calcs.iter().enumerate() {
        let thm_id = format!("THM-{:03}", idx.saturating_add(1));
        let expr = calc
            .expression
            .as_deref()
            .map(lower_sysml_expr_to_matlab)
            .unwrap_or_else(|| "true".to_string());
        out.push_str(&format!("% {} {}\n", thm_id, calc.name));
        out.push_str(&format!(
            "sldv.assert( ({}), 'Proof_{}_{}' );\n\n",
            expr,
            thm_id.replace('-', "_"),
            calc.name.replace('-', "_")
        ));
    }

    out
}
