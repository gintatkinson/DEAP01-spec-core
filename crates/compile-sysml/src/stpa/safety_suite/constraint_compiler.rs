#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! STPA and FMECA Markdown matrix to SysML v2 `ConstraintDef` package compiler.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, IEC 61508, and ECSS-E-ST-40C
//! safety transpilation objectives for module `crates/compile-sysml/src/stpa/safety_suite/constraint_compiler.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Parses UCA and FMECA Markdown tables into formal SysML v2 `PackageDef` constraint assertions
//!   while enforcing negative safety model invariants and AST `valid_parts` scope filtering.
//!
//! ## 3. Preconditions
//! - Caller supplies UTF-8 Markdown content, target `package_name`, and optional `valid_parts` or `model`.
//!
//! ## 4. Postconditions
//! - Returns a deterministic `PackageDef` populated with `ConstraintDef` nodes for valid UCA and FMECA rows.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(L * P) over Markdown lines L and candidate parts P.
//! - Space Complexity: O(C + P) over synthesized constraints C and part identifier sets P.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Ignores malformed table rows, missing columns, and non-digit `UCA-`/`FMECA-`/`SC-` prefixes.
//! - State: Pure functional compilation with zero shared state.
//! - Action: Uses `.get()`, `.strip_prefix()`, and `.saturating_add()` with zero panics or regexes.
//! - Resource: Bounded allocations proportional to valid UCA and FMECA table rows.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603], [REQ-STPA-SAFETY-SUITE-CONSTRAINT-COMPILER] | SysML v2: PackageDef(constraint_compiler)

use super::*;

/// Parses an STPA Markdown document and compiles its UCA and FMECA matrices into a SysML v2 constraint package.
/// Realises: [REQ-STPA-SAFETY-SUITE-COMPILE-STPA-TO-CONSTRAINTS]
pub fn compile_stpa_to_constraints(content: &str, package_name: &str) -> PackageDef {
    compile_stpa_to_constraints_filtered(content, package_name, None, None)
}

/// Parses an STPA Markdown document into a SysML v2 constraint package filtered by negative AST invariants and valid part sets.
/// Realises: [REQ-STPA-SAFETY-SUITE-COMPILE-STPA-TO-CONSTRAINTS-FILTERED]
pub fn compile_stpa_to_constraints_filtered(
    content: &str,
    package_name: &str,
    valid_parts: Option<&BTreeSet<String>>,
    model: Option<&PackageDef>,
) -> PackageDef {
    let mut pkg = PackageDef {
        name: package_name.to_string(),
        doc: Some("STPA and FMECA Safety Invariants compiled for Run-Time Assurance (RTA) & SLDV Verification".to_string()),
        ..Default::default()
    };

    let mut negated_components: BTreeSet<String> =
        deap_core::sysml_ast::extract_negated_components(content)
            .into_iter()
            .collect();
    if let Some(m) = model {
        let sysml = crate::semantic::serializer::to_sysml(m);
        negated_components.extend(deap_core::sysml_ast::extract_negated_components(&sysml));
        negated_components.extend(m.extract_negated_components());
    }

    let model_part_set: Option<BTreeSet<String>> = model.map(|m| {
        m.get_all_parts()
            .into_iter()
            .map(|p| p.name)
            .filter(|n| !n.starts_with('_'))
            .collect()
    });
    let allowed_parts = valid_parts.or(model_part_set.as_ref());

    for line in content.lines() {
        if let Some(con) = compile_uca_row_to_constraint(line) {
            pkg.constraint_defs.push(con);
        }
    }

    for line in content.lines() {
        if let Some(con) =
            compile_fmeca_row_to_constraint(line, &negated_components, allowed_parts)
        {
            pkg.constraint_defs.push(con);
        }
    }

    pkg
}

/// Parses a single `| UCA-### | ... |` table row into an assertion `ConstraintDef`.
/// Realises: [REQ-STPA-SAFETY-SUITE-COMPILE-STPA-TO-CONSTRAINTS-FILTERED]
fn compile_uca_row_to_constraint(line: &str) -> Option<ConstraintDef> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return None;
    }
    let cols: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
    let (&uca_id, &action, &guide_word, &sc_col) =
        (cols.get(1)?, cols.get(3)?, cols.get(4)?, cols.get(7)?);
    if !uca_id
        .strip_prefix("UCA-")
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
    {
        return None;
    }
    let sc_id = extract_sc_id_token(sc_col)?;
    if action.is_empty() {
        return None;
    }

    let constraint_name = format!("{}_{}", sc_id.replace('-', "_"), action.replace(' ', "_"));
    let doc = format!(
        "{}: {} shall not violate safety under {} (ref {})",
        sc_id, action, guide_word, uca_id
    );
    let expression = format!("{}SafeState == true", action.replace(' ', ""));

    Some(ConstraintDef {
        name: constraint_name,
        doc: Some(doc),
        expression,
        is_assertion: true,
        ..Default::default()
    })
}

/// Parses a single `| FMECA-### | ... |` table row into a component integrity `ConstraintDef`.
/// Realises: [REQ-STPA-SAFETY-SUITE-COMPILE-STPA-TO-CONSTRAINTS-FILTERED]
fn compile_fmeca_row_to_constraint(
    line: &str,
    negated_components: &BTreeSet<String>,
    allowed_parts: Option<&BTreeSet<String>>,
) -> Option<ConstraintDef> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return None;
    }
    let cols: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
    let (&fmeca_id, &component, &failure_mode, _) =
        (cols.get(1)?, cols.get(2)?, cols.get(3)?, cols.get(4)?);
    if !fmeca_id
        .strip_prefix("FMECA-")
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
    {
        return None;
    }
    if component.is_empty() || failure_mode.is_empty() {
        return None;
    }

    let comp_clean = component.trim_matches(|c| c == '`' || c == '*').trim();
    let comp_alnum: String = comp_clean
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();

    if negated_components.iter().any(|neg| {
        comp_alnum.contains(neg.as_str())
            || deap_core::sysml_ast::component_matches(comp_clean, neg)
    }) {
        return None;
    }

    if let Some(parts) = allowed_parts {
        if !parts
            .iter()
            .any(|p| deap_core::sysml_ast::component_matches(comp_clean, p))
        {
            return None;
        }
    }

    let constraint_name = format!(
        "{}_{}_Integrity",
        fmeca_id.replace('-', "_"),
        comp_clean.replace(' ', "_")
    );
    let doc = format!(
        "{}: {} shall mitigate {}",
        fmeca_id, comp_clean, failure_mode
    );
    let expression = format!("{}HealthStatus >= 1", comp_clean.replace(' ', ""));

    Some(ConstraintDef {
        name: constraint_name,
        doc: Some(doc),
        expression,
        is_assertion: false,
        ..Default::default()
    })
}

/// Extracts the first `SC-<digits>` safety constraint identifier from a table cell using `.strip_prefix()` and `.get()`.
/// Realises: [REQ-STPA-SAFETY-SUITE-COMPILE-STPA-TO-CONSTRAINTS-FILTERED]
fn extract_sc_id_token(sc_col: &str) -> Option<&str> {
    let mut remaining = sc_col;
    while let Some(pos) = remaining.find("SC-") {
        let candidate = remaining.get(pos..)?;
        let after_prefix = candidate.strip_prefix("SC-")?;
        let digit_len = after_prefix
            .as_bytes()
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count();
        if digit_len > 0 {
            return candidate.get(..3usize.saturating_add(digit_len));
        }
        remaining = after_prefix;
    }
    None
}
