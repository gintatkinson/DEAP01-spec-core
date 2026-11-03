#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Checks 4 and 5 (Derivation-Annotation Contract table and Section 2 Component tables) for `blueprint_gate`.
//! Realises: [REQ-BLUEPRINT-CHECK-4-DERIVATION], [REQ-BLUEPRINT-CHECK-5-COMPONENTS]

use crate::blueprint_gate::{ComponentPort, ComponentProperty, ComponentSpec, DerivationRow};
use crate::bp_gate_tables::{
    is_empty_or_placeholder_cell, is_pascal_case_identifier, is_table_separator_row,
    is_valid_sysml_property_type, split_markdown_table_row, strip_markdown_decorations,
};

/// Check 4 (Derivation-Annotation Contract Gate): Validates the 5-column Derivation-Annotation Contract table in Section 1.
/// Realises: [REQ-BLUEPRINT-CHECK-4-DERIVATION]
pub fn check_4_derivation_table(section_1_lines: &[&str]) -> Result<Vec<DerivationRow>, Vec<String>> {
    let mut errors = Vec::new();
    let mut in_fence = false;
    let mut header_idx: Option<usize> = None;
    for (idx, line) in section_1_lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            if let Some(cols) = split_markdown_table_row(trimmed) {
                let c0 = strip_markdown_decorations(cols.first().map_or("", String::as_str)).to_ascii_lowercase();
                if c0.contains("claim") || c0.contains("requirement") {
                    if cols.len() != 5 {
                        errors.push(format!("Check 4 failed: Derivation-Annotation Contract table header has {} columns; expected 5", cols.len()));
                        return Err(errors);
                    }
                    let c1 = strip_markdown_decorations(cols.get(1).map_or("", String::as_str)).to_ascii_lowercase();
                    let c2 = strip_markdown_decorations(cols.get(2).map_or("", String::as_str)).to_ascii_lowercase();
                    let c3 = strip_markdown_decorations(cols.get(3).map_or("", String::as_str)).to_ascii_lowercase();
                    let c4 = strip_markdown_decorations(cols.get(4).map_or("", String::as_str)).to_ascii_lowercase();
                    if (c1.contains("ssot") || c1.contains("anchor"))
                        && (c2.contains("methodology") || c2.contains("citation"))
                        && c3.contains("grade")
                        && (c4.contains("caveat") || c4.contains("impact"))
                    {
                        header_idx = Some(idx);
                        break;
                    }
                }
            }
        }
    }
    let Some(h_idx) = header_idx else {
        return Err(vec!["Check 4 failed: missing 5-column Derivation-Annotation Contract table in Section 1".to_string()]);
    };
    let sep_line = section_1_lines.get(h_idx.saturating_add(1)).copied().unwrap_or("");
    if !split_markdown_table_row(sep_line).is_some_and(|cols| is_table_separator_row(&cols, 5)) {
        errors.push("Check 4 failed: Derivation-Annotation Contract table header must be followed by a 5-column separator row".to_string());
    }
    let rows = parse_derivation_table_rows(section_1_lines, h_idx, &mut errors);
    if rows.is_empty() {
        errors.push("Check 4 failed: Derivation-Annotation Contract table in Section 1 must contain at least 1 data row".to_string());
    }
    if errors.is_empty() { Ok(rows) } else { Err(errors) }
}

/// Parses data rows following the Section 1 derivation table header and separator.
/// Realises: [REQ-BLUEPRINT-CHECK-4-DERIVATION]
fn parse_derivation_table_rows(
    section_1_lines: &[&str],
    h_idx: usize,
    errors: &mut Vec<String>,
) -> Vec<DerivationRow> {
    let mut rows = Vec::new();
    for (offset, line) in section_1_lines.iter().skip(h_idx.saturating_add(2)).enumerate() {
        let row_num = offset.saturating_add(1);
        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.starts_with('|') {
            break;
        }
        let Some(cols) = split_markdown_table_row(trimmed) else {
            break;
        };
        if cols.len() != 5 {
            errors.push(format!(
                "Check 4 failed: derivation table row {row_num} has {} columns; expected 5",
                cols.len()
            ));
            continue;
        }
        for (col_idx, cell) in cols.iter().enumerate() {
            if is_empty_or_placeholder_cell(cell) {
                errors.push(format!(
                    "Check 4 failed: derivation table row {row_num} column {} is empty or contains a placeholder ('{cell}')",
                    col_idx.saturating_add(1)
                ));
            }
        }
        let raw_grade = cols.get(3).map_or("", String::as_str);
        let clean_grade = strip_markdown_decorations(raw_grade);
        if !matches!(
            clean_grade.as_str(),
            "Directly Evidenced" | "Analytically Derived" | "Declared Assumption"
        ) {
            errors.push(format!(
                "Check 4 failed: derivation table row {row_num} has invalid Derivation Grade '{clean_grade}'; must be 'Directly Evidenced', 'Analytically Derived', or 'Declared Assumption'"
            ));
        }
        rows.push(DerivationRow {
            claim: cols.first().cloned().unwrap_or_default(),
            ssot_anchor: cols.get(1).cloned().unwrap_or_default(),
            methodology: cols.get(2).cloned().unwrap_or_default(),
            grade: clean_grade,
            caveat: cols.get(4).cloned().unwrap_or_default(),
        });
    }
    rows
}

/// Parses a 5-column Component Property table starting at `header_idx`.
/// Realises: [REQ-BLUEPRINT-PARSE-SINGLE-COMPONENT]
fn parse_component_property_rows(
    comp_name: &str,
    comp_lines: &[&str],
    header_idx: usize,
    properties: &mut Vec<ComponentProperty>,
    errors: &mut Vec<String>,
) -> usize {
    let sep_line = comp_lines.get(header_idx.saturating_add(1)).copied().unwrap_or("");
    if !split_markdown_table_row(sep_line).is_some_and(|s| is_table_separator_row(&s, 5)) {
        errors.push(format!(
            "Check 5 failed: Component '{comp_name}' property table header must be followed by a 5-column separator row"
        ));
    }
    let mut idx = header_idx.saturating_add(2);
    while let Some(&row_line) = comp_lines.get(idx) {
        let r_trim = row_line.trim();
        if r_trim.is_empty() || !r_trim.starts_with('|') {
            break;
        }
        let Some(r_cols) = split_markdown_table_row(r_trim) else {
            break;
        };
        if r_cols.len() != 5 {
            errors.push(format!(
                "Check 5 failed: Component '{comp_name}' property row has {} columns; expected 5",
                r_cols.len()
            ));
            idx = idx.saturating_add(1);
            continue;
        }
        for (c_i, cell) in r_cols.iter().enumerate() {
            if is_empty_or_placeholder_cell(cell) {
                errors.push(format!(
                    "Check 5 failed: Component '{comp_name}' property row column {} is empty or placeholder",
                    c_i.saturating_add(1)
                ));
            }
        }
        let attr = strip_markdown_decorations(r_cols.first().map_or("", String::as_str));
        let p_type = strip_markdown_decorations(r_cols.get(1).map_or("", String::as_str));
        let def_val = strip_markdown_decorations(r_cols.get(2).map_or("", String::as_str));
        let unit = strip_markdown_decorations(r_cols.get(3).map_or("", String::as_str));
        let desc = r_cols.get(4).cloned().unwrap_or_default();
        if !is_valid_sysml_property_type(&p_type) {
            errors.push(format!(
                "Check 5 failed: Component '{comp_name}' attribute '{attr}' has invalid SysML type '{p_type}'; expected Integer, Boolean, Real, String, or PascalCase classifier"
            ));
        }
        properties.push(ComponentProperty {
            attribute: attr,
            prop_type: p_type,
            default_val: def_val,
            unit,
            description: desc,
        });
        idx = idx.saturating_add(1);
    }
    idx
}

/// Parses a 5-column Component Port table starting at `header_idx`.
/// Realises: [REQ-BLUEPRINT-PARSE-SINGLE-COMPONENT]
fn parse_component_port_rows(
    comp_name: &str,
    comp_lines: &[&str],
    header_idx: usize,
    ports: &mut Vec<ComponentPort>,
    errors: &mut Vec<String>,
) -> usize {
    let sep_line = comp_lines.get(header_idx.saturating_add(1)).copied().unwrap_or("");
    if !split_markdown_table_row(sep_line).is_some_and(|s| is_table_separator_row(&s, 5)) {
        errors.push(format!(
            "Check 5 failed: Component '{comp_name}' port table header must be followed by a 5-column separator row"
        ));
    }
    let mut idx = header_idx.saturating_add(2);
    while let Some(&row_line) = comp_lines.get(idx) {
        let r_trim = row_line.trim();
        if r_trim.is_empty() || !r_trim.starts_with('|') {
            break;
        }
        let Some(r_cols) = split_markdown_table_row(r_trim) else {
            break;
        };
        if r_cols.len() != 5 {
            errors.push(format!(
                "Check 5 failed: Component '{comp_name}' port row has {} columns; expected 5",
                r_cols.len()
            ));
            idx = idx.saturating_add(1);
            continue;
        }
        for (c_i, cell) in r_cols.iter().enumerate() {
            if is_empty_or_placeholder_cell(cell) {
                errors.push(format!(
                    "Check 5 failed: Component '{comp_name}' port row column {} is empty or placeholder",
                    c_i.saturating_add(1)
                ));
            }
        }
        let port_name = strip_markdown_decorations(r_cols.first().map_or("", String::as_str));
        let direction = strip_markdown_decorations(r_cols.get(1).map_or("", String::as_str));
        let port_type = strip_markdown_decorations(r_cols.get(2).map_or("", String::as_str));
        let protocol = strip_markdown_decorations(r_cols.get(3).map_or("", String::as_str));
        let desc = r_cols.get(4).cloned().unwrap_or_default();
        if !matches!(direction.as_str(), "in" | "out" | "inout") {
            errors.push(format!(
                "Check 5 failed: Component '{comp_name}' port '{port_name}' has invalid Direction '{direction}'; must be 'in', 'out', or 'inout'"
            ));
        }
        ports.push(ComponentPort {
            port: port_name,
            direction,
            port_type,
            protocol,
            description: desc,
        });
        idx = idx.saturating_add(1);
    }
    idx
}

/// Validates and parses the 5-column Property table and 5-column Port table inside a `### Component:` subsection.
/// Realises: [REQ-BLUEPRINT-PARSE-SINGLE-COMPONENT]
fn parse_single_component(
    comp_name: &str,
    comp_lines: &[&str],
    errors: &mut Vec<String>,
) -> ComponentSpec {
    let mut properties = Vec::new();
    let mut ports = Vec::new();
    let mut found_prop_table = false;
    let mut found_port_table = false;
    let mut idx = 0usize;
    let mut in_fence = false;

    while let Some(&line) = comp_lines.get(idx) {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            idx = idx.saturating_add(1);
            continue;
        }
        if !in_fence {
            if let Some(cols) = split_markdown_table_row(trimmed) {
                if cols.len() == 5 {
                    let c0 = strip_markdown_decorations(cols.first().map_or("", String::as_str)).to_ascii_lowercase();
                    let c1 = strip_markdown_decorations(cols.get(1).map_or("", String::as_str)).to_ascii_lowercase();
                    let c2 = strip_markdown_decorations(cols.get(2).map_or("", String::as_str)).to_ascii_lowercase();
                    let c3 = strip_markdown_decorations(cols.get(3).map_or("", String::as_str)).to_ascii_lowercase();
                    let c4 = strip_markdown_decorations(cols.get(4).map_or("", String::as_str)).to_ascii_lowercase();

                    if c0 == "attribute" && c1 == "type" && c2 == "default" && c3 == "unit" && c4 == "description" {
                        found_prop_table = true;
                        idx = parse_component_property_rows(comp_name, comp_lines, idx, &mut properties, errors);
                        continue;
                    } else if c0 == "port" && c1 == "direction" && c2 == "type" && c3 == "protocol" && c4 == "description" {
                        found_port_table = true;
                        idx = parse_component_port_rows(comp_name, comp_lines, idx, &mut ports, errors);
                        continue;
                    }
                }
            }
        }
        idx = idx.saturating_add(1);
    }

    if !found_prop_table {
        errors.push(format!(
            "Check 5 failed: Component '{comp_name}' is missing required 5-column '| Attribute | Type | Default | Unit | Description |' table"
        ));
    } else if properties.is_empty() {
        errors.push(format!("Check 5 failed: Component '{comp_name}' property table must contain at least 1 row"));
    }
    if !found_port_table {
        errors.push(format!(
            "Check 5 failed: Component '{comp_name}' is missing required 5-column '| Port | Direction | Type | Protocol | Description |' table"
        ));
    } else if ports.is_empty() {
        errors.push(format!("Check 5 failed: Component '{comp_name}' port table must contain at least 1 row"));
    }

    ComponentSpec {
        name: comp_name.to_string(),
        properties,
        ports,
    }
}

/// Check 5 (Component & 5-Column Property/Port Table Gate): Validates Section 2 component subsections.
/// Realises: [REQ-BLUEPRINT-CHECK-5-COMPONENTS]
pub fn check_5_components(section_2_lines: &[&str]) -> Result<Vec<ComponentSpec>, Vec<String>> {
    let mut errors = Vec::new();
    let mut component_starts: Vec<(usize, String)> = Vec::new();
    let mut in_fence = false;

    for (idx, line) in section_2_lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            if let Some(rest) = trimmed.strip_prefix("### Component:") {
                let ident = rest.trim().to_string();
                if !is_pascal_case_identifier(&ident) {
                    errors.push(format!(
                        "Check 5 failed: Component identifier '{ident}' is not valid PascalCase (must be non-empty ASCII alphanumeric starting with uppercase letter, with no underscores or spaces)"
                    ));
                }
                component_starts.push((idx, ident));
            }
        }
    }

    if component_starts.is_empty() {
        return Err(vec![
            "Check 5 failed: Section 2 must contain at least one '### Component: <PascalCaseIdentifier>' subsection"
                .to_string(),
        ]);
    }

    let total_lines = section_2_lines.len();
    let mut specs = Vec::with_capacity(component_starts.len());
    for (i, (start_idx, comp_name)) in component_starts.iter().enumerate() {
        let end_idx = component_starts
            .get(i.saturating_add(1))
            .map_or(total_lines, |(next_start, _)| *next_start);
        let sub_lines = section_2_lines.get(start_idx.saturating_add(1)..end_idx).unwrap_or(&[]);
        specs.push(parse_single_component(comp_name, sub_lines, &mut errors));
    }

    if errors.is_empty() {
        Ok(specs)
    } else {
        Err(errors)
    }
}
