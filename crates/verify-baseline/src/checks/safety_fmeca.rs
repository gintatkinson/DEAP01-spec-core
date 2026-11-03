//! FMECA criticality table parser and Pillar 7 validator for Check 17.
//! Realises: [REQ-CHECKS-SAFETY-STPA]

use std::collections::BTreeMap;
use std::path::Path;

use crate::checks::safety_pillars::{check_failure_dimension_coverage, matches_dimension_lower};

/// Extracted FMECA table summary.
/// Realises: [REQ-CHECKS-SAFETY-STPA-FMECADATA]
#[derive(Debug, Default)]
pub struct FmecaData {
    pub total_rows: usize,
    pub has_rpn: bool,
    pub ssot_basis_count: usize,
    pub derived_basis_count: usize,
    pub rows: Vec<FmecaRow>,
}

/// Single parsed row from an FMECA criticality table.
/// Realises: [REQ-CHECKS-SAFETY-STPA-FMECAROW]
#[derive(Debug, Default)]
pub struct FmecaRow {
    pub id: String,
    pub component: String,
    pub failure_mode: String,
    pub s: Option<i32>,
    pub o: Option<i32>,
    pub d: Option<i32>,
    pub rpn: Option<i32>,
    pub basis: Option<String>,
}

/// Checks whether a Markdown table header line represents an FMECA table.
/// Realises: [REQ-CHECKS-SAFETY-STPA-IS-FMECA-HEADER]
fn is_fmeca_header(line: &str) -> bool {
    let lower = line.to_lowercase();
    let has_mode = lower.contains("failure mode")
        || (lower.contains("failure") && (lower.contains("mode") || lower.contains("mechanism")));
    let has_fmeca_col = lower.contains("fmeca id") || lower.contains("fmeca-id");
    let has_fmeca_general = lower.contains("fmeca")
        && (lower.contains("component") || lower.contains("subsystem") || lower.contains("item")
            || lower.contains("rpn") || lower.contains("severity") || lower.contains("criticality"));
    has_mode || has_fmeca_col || has_fmeca_general
}

/// Parses FMECA table(s) from Markdown content into [`FmecaData`].
/// Realises: [REQ-CHECKS-SAFETY-STPA-PARSE-FMECA-TABLE]
pub fn parse_fmeca_table(content: &str) -> FmecaData {
    let mut data = FmecaData::default();
    let lines: Vec<&str> = content.lines().collect();
    let n = lines.len();
    let mut i = 0;

    while i < n {
        let line = lines.get(i).copied().unwrap_or("").trim();
        if line.starts_with('|') && is_fmeca_header(line) {
            let next_line = lines.get(i + 1).copied().unwrap_or("").trim();
            if !next_line.starts_with('|') || !next_line.contains("---") {
                i += 1;
                continue;
            }
            let headers: Vec<String> =
                line.split('|').filter(|s| !s.trim().is_empty()).map(|s| s.trim().to_lowercase()).collect();
            let col_indices = locate_fmeca_column_indices(&headers);
            if col_indices[5].is_some() || headers.iter().any(|h| h.contains("rpn")) {
                data.has_rpn = true;
            }
            i += 1;
            while lines.get(i).map(|l| l.trim()).is_some_and(|s| s.starts_with('|') && s.contains("---")) {
                i += 1;
            }
            while let Some(row_line) = lines.get(i).map(|l| l.trim()) {
                if !row_line.starts_with('|') {
                    break;
                }
                parse_fmeca_row_line(row_line, &col_indices, &mut data);
                i += 1;
            }
        }
        i += 1;
    }

    data
}

/// Locates `[id, comp, mode, s, o, rpn, d, basis]` column indices in an FMECA header.
/// Realises: [REQ-CHECKS-SAFETY-STPA-PARSE-FMECA-TABLE]
fn locate_fmeca_column_indices(headers: &[String]) -> [Option<usize>; 8] {
    let id_idx = headers.iter().position(|h| {
        h.contains("id") || h.contains("code") || h.starts_with("fm#") || h.starts_with("item#")
    });
    let comp_idx = headers
        .iter()
        .position(|h| {
            (h.contains("component")
                || h.contains("subsystem")
                || h.contains("part")
                || h.contains("unit"))
                && !h.contains("effect")
                && !h.contains("mode")
        })
        .or_else(|| headers.iter().position(|h| h.contains("item") && !h.contains("id")));
    let mode_idx = headers
        .iter()
        .position(|h| {
            (h.contains("failure mode") || h.contains("failure mechanism") || h.contains("mode"))
                && !h.contains("dimension")
                && !h.contains("effect")
        })
        .or_else(|| {
            headers.iter().position(|h| h.contains("failure") && !h.contains("id") && !h.contains("effect"))
        });
    let s_idx =
        headers.iter().position(|h| h == "s" || h.starts_with("sev") || h.contains("severity"));
    let o_idx =
        headers.iter().position(|h| h == "o" || h.starts_with("occ") || h.contains("occurrence"));
    let rpn_idx = headers.iter().position(|h| h == "rpn" || h.contains("priority"));
    let d_idx =
        headers.iter().position(|h| h == "d" || h.starts_with("det") || h.contains("detection"));
    let basis_idx = headers.iter().position(|h| {
        h.contains("basis") || h.contains("derivation") || h.contains("provenance")
    });
    [id_idx, comp_idx, mode_idx, s_idx, o_idx, rpn_idx, d_idx, basis_idx]
}

/// Parses a single FMECA row line and appends it to `data`.
/// Realises: [REQ-CHECKS-SAFETY-STPA-PARSE-FMECA-TABLE]
fn parse_fmeca_row_line(row_line: &str, idx: &[Option<usize>; 8], data: &mut FmecaData) {
    let raw_cells: Vec<String> = row_line.split('|').map(|s| s.trim().to_string()).collect();
    let cells: Vec<String> = if raw_cells.len() >= 2
        && raw_cells.first().is_some_and(|c| c.is_empty())
        && raw_cells.last().is_some_and(|c| c.is_empty())
    {
        raw_cells.get(1..raw_cells.len() - 1).unwrap_or_default().to_vec()
    } else {
        raw_cells
    };
    if cells.is_empty() {
        return;
    }

    let mut row = FmecaRow::default();
    if let Some(c) = idx[0].and_then(|i| cells.get(i)) {
        row.id = c.clone();
    }
    if let Some(c) = idx[1].and_then(|i| cells.get(i)) {
        row.component = c.clone();
    }
    if let Some(c) = idx[2].and_then(|i| cells.get(i)) {
        row.failure_mode = c.clone();
    }
    let starts_fm = cells.first().is_some_and(|c0| {
        let u = c0.to_uppercase();
        u.starts_with("FM") || u.starts_with("FMECA")
    });
    if row.component.is_empty() && (idx[0] == Some(0) || starts_fm) {
        if let Some(c1) = cells.get(1) {
            row.component = c1.clone();
        }
    }
    if row.failure_mode.is_empty() && (idx[1] == Some(1) || starts_fm) {
        if let Some(c2) = cells.get(2) {
            row.failure_mode = c2.clone();
        }
    }
    row.s = idx[3].and_then(|i| cells.get(i)).and_then(|c| c.parse::<i32>().ok());
    row.o = idx[4].and_then(|i| cells.get(i)).and_then(|c| c.parse::<i32>().ok());
    row.rpn = idx[5].and_then(|i| cells.get(i)).and_then(|c| c.parse::<i32>().ok());
    row.d = idx[6].and_then(|i| cells.get(i)).and_then(|c| c.parse::<i32>().ok());

    let basis_str = idx[7]
        .and_then(|i| cells.get(i))
        .cloned()
        .unwrap_or_else(|| cells.join(" "))
        .to_uppercase();
    let found_basis = if basis_str.contains("SSOT") {
        data.ssot_basis_count += 1;
        Some("SSOT".to_string())
    } else if basis_str.contains("DERIVED") {
        data.derived_basis_count += 1;
        Some("Derived".to_string())
    } else {
        None
    };
    row.basis = found_basis;
    data.total_rows += 1;
    data.rows.push(row);
}

/// Audits Pillar 7 FMECA Criticality Matrix completeness, dimensions, and RPN math.
/// Realises: [REQ-CHECKS-SAFETY-STPA-CHECK-SAFETY-INTEGRITY]
pub(crate) fn audit_pillar_7_fmeca(
    repo_root: &Path,
    stpa_content: &str,
    stpa_lower: &str,
    combined_content: &str,
    errors: &mut Vec<String>,
) {
    if !(stpa_lower.contains("fmeca") || stpa_lower.contains("failure mode")) {
        errors.push(
            "Pillar 7 violation: Missing FMECA Criticality Matrix in STPA_MATRIX.md.".to_string(),
        );
        return;
    }
    let fmeca = parse_fmeca_table(stpa_content);
    if fmeca.total_rows == 0 {
        errors.push(
            "Pillar 7 violation: FMECA Criticality Matrix in STPA_MATRIX.md contains 0 rows."
                .to_string(),
        );
    } else if fmeca.total_rows < 15 {
        errors.push(format!(
            "Pillar 7 violation: FMECA Criticality Matrix in STPA_MATRIX.md contains {} rows (minimum 15+ rows required).",
            fmeca.total_rows
        ));
    }
    if !(fmeca.has_rpn
        || stpa_lower.contains("risk priority number")
        || matches_dimension_lower(stpa_lower, &["rpn"]))
    {
        errors.push(
            "Pillar 7 violation: FMECA table missing RPN (Risk Priority Number) calculation."
                .to_string(),
        );
    }
    let total_basis = fmeca.ssot_basis_count + fmeca.derived_basis_count;
    if fmeca.total_rows > 0 && total_basis == 0 {
        errors.push("Pillar 7 violation: FMECA Criticality Matrix missing explicit Derivation Basis classification ('SSOT' / 'Derived').".to_string());
    } else if fmeca.total_rows > 0 && total_basis < fmeca.total_rows {
        errors.push(format!(
            "Pillar 7 violation: FMECA Criticality Matrix contains {} row(s) missing explicit Derivation Basis classification ('SSOT' / 'Derived').",
            fmeca.total_rows - total_basis
        ));
    }
    if fmeca.total_rows > 0 {
        let missing_dims = check_failure_dimension_coverage(stpa_content, &fmeca);
        if !missing_dims.is_empty() {
            errors.push(format!(
                "Pillar 7 violation: FMECA table missing coverage for universal failure dimension(s): {} (expected Interface (Γ), State (Φ), Action (Ω), Resource (Ψ)).",
                missing_dims.join(", ")
            ));
        }
        audit_pillar_7_multi_mode_and_ast(repo_root, &fmeca, combined_content, errors);
    }
    for row in &fmeca.rows {
        if let (Some(s), Some(o), Some(d)) = (row.s, row.o, row.d) {
            if !(1..=10).contains(&s) || !(1..=10).contains(&o) || !(1..=10).contains(&d) {
                errors.push(format!(
                    "Pillar 7 violation: FMECA row '{}' ratings out of range [1, 10] (S={}, O={}, D={}).",
                    row.id, s, o, d
                ));
            }
            if let Some(rpn) = row.rpn {
                if rpn != s * o * d {
                    errors.push(format!(
                        "Pillar 7 violation: FMECA row '{}' has invalid RPN calculation -- expected S({}) * O({}) * D({}) = {}, but found RPN = {}.",
                        row.id, s, o, d, s * o * d, rpn
                    ));
                }
            }
        }
    }
}

/// Audits Pillar 7 multi-failure-mode per component and bidirectional SysML AST closure.
/// Realises: [REQ-CHECKS-SAFETY-STPA-CHECK-SAFETY-INTEGRITY]
fn audit_pillar_7_multi_mode_and_ast(
    repo_root: &Path,
    fmeca: &FmecaData,
    combined_content: &str,
    errors: &mut Vec<String>,
) {
    let mut component_modes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in &fmeca.rows {
        let comp = clean_fmeca_component_name(&row.component);
        let mode = row.failure_mode.trim();
        if !comp.is_empty() && !mode.is_empty() {
            component_modes.entry(comp.to_string()).or_default().push(mode.to_string());
        }
    }
    if !component_modes.is_empty() {
        let mut single_mode_components = Vec::new();
        for (comp, modes) in &component_modes {
            let mut distinct = modes.clone();
            distinct.sort_by_key(|m| m.to_lowercase());
            distinct.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
            if distinct.len() < 2 {
                single_mode_components.push(comp.clone());
            }
        }
        if single_mode_components.len() == component_modes.len() {
            errors.push(format!("Pillar 7 violation: FMECA matrix exhibits degenerate single-failure-mode-per-component coverage (all {} components have only 1 failure mode). Multi-failure-mode coverage per component is required (minimum >= 2 distinct failure modes per component).", component_modes.len()));
        } else if !single_mode_components.is_empty() {
            errors.push(format!("Pillar 7 violation: FMECA component(s) with degenerate single-failure-mode coverage (minimum >= 2 distinct failure modes required): {}.", single_mode_components.join(", ")));
        }
    } else {
        errors.push("Pillar 7 violation: FMECA Criticality Matrix contains rows but no valid components or failure modes could be resolved.".to_string());
    }

    let Some(text) = crate::checks::parity_gates::discover_architectural_sysml_model_text(repo_root) else {
        return;
    };
    let expected_parts: Vec<String> = deap_core::sysml_cst::parse_part_defs(&text)
        .into_iter()
        .map(|p| p.name)
        .filter(|n| !n.starts_with('_'))
        .collect();
    if expected_parts.is_empty() {
        return;
    }

    let mut table_components: Vec<String> = Vec::new();
    for row in &fmeca.rows {
        let comp = clean_fmeca_component_name(&row.component);
        if !comp.is_empty() && !table_components.iter().any(|c| c.eq_ignore_ascii_case(comp)) {
            table_components.push(comp.to_string());
        }
    }
    let mut missing_parts: Vec<String> = expected_parts
        .iter()
        .filter(|p| !table_components.iter().any(|c| deap_core::sysml_cst::component_matches(c, p)))
        .cloned()
        .collect();
    if !missing_parts.is_empty() {
        missing_parts.sort();
        errors.push(format!("Pillar 7 violation: FMECA table missing declared AST part def component(s): {}.", missing_parts.join(", ")));
    }
    let mut undeclared_parts: Vec<String> = table_components
        .iter()
        .filter(|c| !expected_parts.iter().any(|p| deap_core::sysml_cst::component_matches(c, p)))
        .cloned()
        .collect();
    if !undeclared_parts.is_empty() {
        undeclared_parts.sort();
        errors.push(format!("Pillar 7 violation: FMECA table references undeclared phantom component(s) not in AST: {}.", undeclared_parts.join(", ")));
    }
    for c in &table_components {
        if deap_core::sysml_cst::is_component_negated(c, &text)
            || deap_core::sysml_cst::is_component_negated(c, combined_content)
        {
            errors.push(format!("Pillar 7 violation: FMECA table references component '{}' contradicted by negative AST model constraint.", c));
        }
    }
}

/// Strips Markdown link brackets and backticks/asterisks from an FMECA component cell.
/// Realises: [REQ-CHECKS-SAFETY-STPA-CHECK-SAFETY-INTEGRITY]
fn clean_fmeca_component_name(comp_raw: &str) -> &str {
    let trimmed = comp_raw.trim();
    let clean = if trimmed.starts_with('[') {
        trimmed.find(']').map(|idx| &trimmed[1..idx]).unwrap_or(trimmed)
    } else {
        trimmed
    };
    clean.trim_matches(|c| c == '`' || c == '*').trim()
}
