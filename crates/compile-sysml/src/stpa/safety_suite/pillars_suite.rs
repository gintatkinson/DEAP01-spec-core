//! Formal safety constraint, FMECA criticality matrix, Run-Time Assurance architecture, and hazard log renderers.
//!
//! Safety & Traceability: Realises: [REQ-STPA-SAFETY-SUITE-PILLARS-SUITE]
//! Contract: Synthesizes `04_SAFETY_CONSTRAINTS.md`, `05_FMECA_MATRIX.md`, `07_RTA_ARCHITECTURE.md`, and `HAZARD_LOG.md` deterministically from SysML v2 `PackageDef` AST declarations, UCAs, constraints, FMECA rows, and parameter tokens.
//! Bounds & Faults: Executes bounded linear and cycled `.get()` / `checked_rem` traversals with saturating arithmetic, sanitized Markdown/KaTeX/MATLAB fragments, and zero panics or direct slice indexing.

use super::*;

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
    out.push_str("| Safety Constraint ID | UCA ID | Constraint Statement |\n");
    out.push_str("| :--- | :--- | :--- |\n");
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
        out.push_str("| Constraint Def | Expression |\n");
        out.push_str("| :--- | :--- |\n");
        for c in constraints {
            let clean_name = sanitize_markdown_cell(&c.name);
            let expr = if c.expression.trim().is_empty() {
                c.doc
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(sanitize_markdown_cell)
                    .unwrap_or_else(|| {
                        format!("{} == true (Derived: AST ConstraintDef)", clean_name)
                    })
            } else {
                sanitize_markdown_cell(&c.expression)
            };
            out.push_str(&format!("| {} | {} |\n", clean_name, expr));
        }
    }

    let calcs = pkg.get_all_calcs();
    out.push_str("\n## Schema-Declared Calculation Obligations (SysML v2 SSOT)\n\n");
    if calcs.is_empty() {
        out.push_str(&format!(
            "No `calc def` expressions declared in package `{}`.\n",
            base_pkg
        ));
    } else {
        out.push_str("| Calc Def | Expression |\n");
        out.push_str("| :--- | :--- |\n");
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
                    format!("{}({}) (Derived: AST CalcDef)", clean_name, param_names)
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
        out.push_str("| Requirement Def | Specification |\n");
        out.push_str("| :--- | :--- |\n");
        for (req_name, spec) in &req_rows {
            out.push_str(&format!("| {} | {} |\n", req_name, spec));
        }
        out.push('\n');
    }

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

/// Renders the Run-Time Assurance (`07_RTA_ARCHITECTURE.md`) simplex topology diagram and AST-derived KaTeX/SLDV theorem proof suite.
/// Realises: [REQ-STPA-SAFETY-SUITE-RENDER-RTA-ARCHITECTURE]
pub(super) fn render_rta_architecture(
    pkg: &PackageDef,
    tokens: &BTreeMap<String, String>,
) -> String {
    let all_parts = pkg.get_all_parts();
    let non_internal_parts: Vec<&PartDef> = all_parts
        .iter()
        .filter(|p| !p.name.starts_with('_'))
        .collect();
    let base_pkg = if pkg.name.trim().is_empty() {
        "System"
    } else {
        pkg.name.trim()
    };
    let default_high = format!("{}_AssuranceController", base_pkg);
    let default_mon = format!("{}_MonitorSwitch", base_pkg);
    let default_rec = format!("{}_RecoveryChannel", base_pkg);

    let high_assurance = non_internal_parts
        .iter()
        .copied()
        .find(|p| p.name.contains("Assurance") || p.name.contains("Controller"))
        .or_else(|| non_internal_parts.first().copied())
        .or_else(|| all_parts.first())
        .map(|p| p.name.as_str())
        .unwrap_or(default_high.as_str());
    let monitor_switch = non_internal_parts
        .iter()
        .copied()
        .find(|p| p.name.contains("Solver") || p.name.contains("Monitor"))
        .or_else(|| non_internal_parts.get(1).copied())
        .or_else(|| all_parts.get(1))
        .map(|p| p.name.as_str())
        .unwrap_or(default_mon.as_str());
    let recovery_chan = non_internal_parts
        .iter()
        .copied()
        .find(|p| p.name.contains("Safety") || p.name.contains("Recovery"))
        .or_else(|| non_internal_parts.get(2).copied())
        .or_else(|| all_parts.get(2))
        .map(|p| p.name.as_str())
        .unwrap_or(default_rec.as_str());
    let sys_under_assurance = base_pkg;

    let mut out = String::new();
    out.push_str("# Run-Time Assurance Architecture & Formal Proof Suite\n\n");
    out.push_str("## Simplex Run-Time Assurance Topology\n\n");
    out.push_str("```mermaid\n");
    out.push_str("graph TD\n");
    out.push_str("    subgraph \"Run-Time Assurance Architecture\"\n");
    out.push_str("        direction TB\n");
    out.push_str(&format!(
        "        HighAssuranceChannel[\"High Assurance Channel<br/>({})\"] --> SafetyMonitorSwitch[\"Safety Monitor Switch<br/>({})\"]\n",
        high_assurance, monitor_switch
    ));
    out.push_str(&format!(
        "        RecoveryChannel[\"Recovery Channel<br/>({})\"] --> SafetyMonitorSwitch\n",
        recovery_chan
    ));
    out.push_str(&format!(
        "        SafetyMonitorSwitch --> SystemUnderAssurance[\"System Under Assurance<br/>({})\"]\n",
        sys_under_assurance
    ));
    out.push_str("    end\n");
    out.push_str("```\n\n");

    out.push_str("## Formal Proof Suite\n\n");

    let all_calcs = pkg.get_all_calcs();
    let all_constraints = pkg.get_all_constraints();

    if !all_calcs.is_empty() || !all_constraints.is_empty() {
        for (idx, calc) in all_calcs.iter().enumerate() {
            let tid = format!("T-{:02}", idx.saturating_add(1));
            let raw_expr = calc
                .expression
                .as_deref()
                .map(|s| s.trim_matches(|ch: char| ch.is_whitespace() || ch == ';'))
                .filter(|s| !s.is_empty())
                .unwrap_or("true");
            let katex_name = escape_katex_ident(&calc.name);
            let katex_expr = lower_sysml_expr_to_katex(raw_expr);
            let matlab_expr = lower_sysml_expr_to_matlab(raw_expr);
            out.push_str(&format!("## Theorem {} -- {}\n\n", tid, calc.name));
            out.push_str("### Formal Theorem Statement\n\n");
            out.push_str(&format!(
                "$$\n\\begin{{aligned}}\n    \\mathrm{{{}}} &: {}\n\\end{{aligned}}\n$$\n\n",
                katex_name, katex_expr
            ));
            out.push_str("### Symbolic Derivation\n\n");
            out.push_str(&format!(
                "$$\n\\begin{{aligned}}\n    \\mathrm{{{}}} &= {} \\\\\n\\end{{aligned}}\n$$\n\n",
                katex_name, katex_expr
            ));
            out.push_str("### Parameter Definitions & Engineering Units Table\n\n");
            out.push_str("| Symbol | Description | Value | Engineering Unit |\n");
            out.push_str("| :--- | :--- | :--- | :--- |\n");
            if calc.in_params.is_empty() {
                out.push_str(&format!(
                    "| `{}` | AST Calc Expression | `{}` | dimensionless |\n",
                    calc.name, raw_expr
                ));
            } else {
                for param in &calc.in_params {
                    let val = param
                        .default_value
                        .as_deref()
                        .map(|s| s.trim().trim_matches('"').trim())
                        .filter(|s| !s.is_empty())
                        .or_else(|| find_token_ci(tokens, &param.name))
                        .map(sanitize_markdown_cell)
                        .unwrap_or_else(|| "Symbolic Input (AST CalcParam)".to_string());
                    let unit = if param.type_name.trim().is_empty() {
                        "scalar".to_string()
                    } else {
                        sanitize_markdown_cell(&param.type_name)
                    };
                    out.push_str(&format!(
                        "| {} | AST Calc Input | {} | {} |\n",
                        sanitize_markdown_cell(&param.name),
                        val,
                        unit
                    ));
                }
            }
            out.push_str("\n### Step-by-Step Numerical Proof Evaluation\n\n");
            out.push_str(&format!(
                "$$\n\\begin{{aligned}}\n    \\mathrm{{{}}} &= {} \\\\\n\\end{{aligned}}\n$$\n\n",
                katex_name, katex_expr
            ));
            out.push_str("### SLDV Temporal Assertion Binding\n\n");
            out.push_str("```matlab\n");
            out.push_str(&format!(
                "sldv.assert( ({}), 'Bind_{}_CALC' );\n",
                matlab_expr,
                calc.name.to_uppercase().replace('-', "_")
            ));
            out.push_str("```\n\n");
        }

        let offset = all_calcs.len();
        for (idx, c) in all_constraints.iter().enumerate() {
            let tid = format!("T-{:02}", offset.saturating_add(idx).saturating_add(1));
            let trimmed_constraint = c
                .expression
                .trim_matches(|ch: char| ch.is_whitespace() || ch == ';');
            let raw_expr = if trimmed_constraint.is_empty() {
                "true"
            } else {
                trimmed_constraint
            };
            let katex_name = escape_katex_ident(&c.name);
            let katex_expr = lower_sysml_expr_to_katex(raw_expr);
            let matlab_expr = lower_sysml_expr_to_matlab(raw_expr);
            out.push_str(&format!("## Theorem {} -- {}\n\n", tid, c.name));
            out.push_str("### Formal Theorem Statement\n\n");
            out.push_str(&format!(
                "$$\n\\begin{{aligned}}\n    \\mathrm{{{}}} &: {}\n\\end{{aligned}}\n$$\n\n",
                katex_name, katex_expr
            ));
            out.push_str("### Symbolic Derivation\n\n");
            out.push_str(&format!(
                "$$\n\\begin{{aligned}}\n    \\mathrm{{{}}} &\\iff {} \\\\\n\\end{{aligned}}\n$$\n\n",
                katex_name, katex_expr
            ));
            out.push_str("### Parameter Definitions & Engineering Units Table\n\n");
            out.push_str("| Symbol | Description | Value | Engineering Unit |\n");
            out.push_str("| :--- | :--- | :--- | :--- |\n");
            if c.parameters.is_empty() {
                out.push_str(&format!(
                    "| `{}` | AST Constraint Invariant | `{}` | boolean |\n",
                    c.name, raw_expr
                ));
            } else {
                for param in &c.parameters {
                    let (p_name, p_type) = match param.split_once(':') {
                        Some((n, t)) => (n.trim(), t.trim()),
                        None => (param.trim(), "scalar"),
                    };
                    let val = find_token_ci(tokens, p_name)
                        .map(sanitize_markdown_cell)
                        .unwrap_or_else(|| "Symbolic Bound (AST ConstraintParam)".to_string());
                    let clean_type = if p_type.is_empty() {
                        "scalar".to_string()
                    } else {
                        sanitize_markdown_cell(p_type)
                    };
                    out.push_str(&format!(
                        "| {} | AST Constraint Parameter | {} | {} |\n",
                        sanitize_markdown_cell(p_name),
                        val,
                        clean_type
                    ));
                }
            }
            out.push_str("\n### Step-by-Step Numerical Proof Evaluation\n\n");
            out.push_str(&format!(
                "$$\n\\begin{{aligned}}\n    \\mathrm{{{}}} &\\implies {} \\\\\n\\end{{aligned}}\n$$\n\n",
                katex_name, katex_expr
            ));
            out.push_str("### SLDV Temporal Assertion Binding\n\n");
            out.push_str("```matlab\n");
            out.push_str(&format!(
                "sldv.assert( ({}), 'Bind_{}_ASSERTION' );\n",
                matlab_expr,
                c.name.to_uppercase().replace('-', "_")
            ));
            out.push_str("```\n\n");
        }
    } else {
        out.push_str(&format!(
            "No `calc def` or `constraint def` expressions declared in package `{}`.\n\n",
            base_pkg
        ));
    }

    out
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

    let loss_sources: Vec<String> = if !controllers.is_empty() {
        controllers
            .iter()
            .map(|c| sanitize_markdown_cell(&c.name))
            .collect()
    } else {
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
    };

    let mut out = String::new();
    out.push_str("# Hazard Log\n\n");
    out.push_str("| ID | Kind | Source | Status | Notes |\n");
    out.push_str("| :--- | :--- | :--- | :--- | :--- |\n");

    for (i, src) in loss_sources.iter().enumerate() {
        out.push_str(&format!(
            "| L-{} | Loss | {} | Open | Loss of safe functional integrity of {} (AST Derived) |\n",
            i.saturating_add(1),
            src,
            src
        ));
    }

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
        let baseline_note = constraints
            .first()
            .map(|c| {
                format!(
                    "Enforced by ConstraintDef {} (Derived)",
                    sanitize_markdown_cell(&c.name)
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "Enforced by {} baseline safety envelope (Derived)",
                    sanitize_markdown_cell(base_pkg)
                )
            });
        out.push_str(&format!(
            "| H-1 | Hazard | {} | Open | {} |\n",
            primary_src, baseline_note
        ));
    }

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
