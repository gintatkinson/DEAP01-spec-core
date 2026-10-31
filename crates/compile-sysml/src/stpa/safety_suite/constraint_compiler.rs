//! STPA and FMECA Markdown matrix to SysML v2 `ConstraintDef` package compiler.
//!
//! Safety & Traceability: Realises: [REQ-STPA-SAFETY-SUITE-CONSTRAINT-COMPILER]
//! Contract: Parses UCA and FMECA Markdown tables into formal SysML v2 `PackageDef` constraint assertions while enforcing negative safety model invariants and AST `valid_parts` scope filtering.
//! Bounds & Faults: Performs single-pass line-by-line checked column lookups (`.get()`, `.strip_prefix()`, `.saturating_add()`) with zero regular expressions, panics, or unchecked slice indexing.

use super::*;

/// Parses an STPA Markdown document and compiles its UCA and FMECA matrices into a SysML v2 constraint package.
///
/// Safety & Traceability: Realises: [REQ-STPA-SAFETY-SUITE-COMPILE-STPA-TO-CONSTRAINTS]
/// Contract: Delegates to `compile_stpa_to_constraints_filtered` with unfiltered part scope (`None, None`) to synthesize a deterministic `PackageDef` named `package_name`.
/// Bounds & Faults: Linear `O(L)` scan over input text length `L` with zero panics, regular expressions, or direct slice indexing.
pub fn compile_stpa_to_constraints(content: &str, package_name: &str) -> PackageDef {
    compile_stpa_to_constraints_filtered(content, package_name, None, None)
}

/// Parses an STPA Markdown document into a SysML v2 constraint package filtered by negative AST invariants and valid part sets.
///
/// Safety & Traceability: Realises: [REQ-STPA-SAFETY-SUITE-COMPILE-STPA-TO-CONSTRAINTS-FILTERED]
/// Contract: Compiles `UCA-###` rows linked to `SC-###` identifiers into assertion `ConstraintDef` nodes and compiles `FMECA-###` rows into component health `ConstraintDef` nodes only when the component is non-negated and matches `valid_parts` or `model` parts.
/// Bounds & Faults: Bounded `O(L * P)` traversal over Markdown lines `L` and candidate parts `P` using `.get()` and `.strip_prefix()` with zero panics or unchecked indexing.
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

    let mut negated_components = deap_core::sysml_ast::extract_negated_components(content);
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

    // Match UCA table rows: | UCA-### | Controller | Action | Guide Word | ... | Hazard | Constraint | ...
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let cols: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
        let (Some(&uca_id), Some(&action), Some(&guide_word), Some(&sc_col)) =
            (cols.get(1), cols.get(3), cols.get(4), cols.get(7))
        else {
            continue;
        };
        if !uca_id
            .strip_prefix("UCA-")
            .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
        {
            continue;
        }
        let Some(sc_id) = extract_sc_id_token(sc_col) else {
            continue;
        };

        if !action.is_empty() {
            let constraint_name =
                format!("{}_{}", sc_id.replace('-', "_"), action.replace(' ', "_"));
            let doc = format!(
                "{}: {} shall not violate safety under {} (ref {})",
                sc_id, action, guide_word, uca_id
            );
            let expression = format!("{}SafeState == true", action.replace(' ', ""));

            pkg.constraint_defs.push(ConstraintDef {
                name: constraint_name,
                doc: Some(doc),
                expression,
                is_assertion: true,
                ..Default::default()
            });
        }
    }

    // Match FMECA table rows: | FMECA-### | Component | Failure Mode | Potential Effect | ... | Mitigation |
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let cols: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
        let (Some(&fmeca_id), Some(&component), Some(&failure_mode), Some(_)) =
            (cols.get(1), cols.get(2), cols.get(3), cols.get(4))
        else {
            continue;
        };
        if !fmeca_id
            .strip_prefix("FMECA-")
            .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
        {
            continue;
        }

        if !component.is_empty() && !failure_mode.is_empty() {
            let comp_clean = component.trim_matches(|c| c == '`' || c == '*').trim();

            let comp_alnum: String = comp_clean
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_ascii_lowercase();

            // If negative AST constraints negate the component, forbid synthesis
            if negated_components.iter().any(|neg| {
                comp_alnum.contains(neg.as_str())
                    || deap_core::sysml_ast::component_matches(comp_clean, neg)
            }) {
                continue;
            }

            // If valid parts are provided or derived from model, verify component existence in AST
            if let Some(parts) = valid_parts {
                if !parts
                    .iter()
                    .any(|p| deap_core::sysml_ast::component_matches(comp_clean, p))
                {
                    continue;
                }
            } else if let Some(ref parts) = model_part_set {
                if !parts
                    .iter()
                    .any(|p| deap_core::sysml_ast::component_matches(comp_clean, p))
                {
                    continue;
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

            pkg.constraint_defs.push(ConstraintDef {
                name: constraint_name,
                doc: Some(doc),
                expression,
                is_assertion: false,
                ..Default::default()
            });
        }
    }

    pkg
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
