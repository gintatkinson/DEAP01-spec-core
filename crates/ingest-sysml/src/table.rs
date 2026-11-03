//! # Markdown Table Parsing, Column Normalization & Cell Sanitization Engine
//!
//! Deterministic parsing, column normalization, cell value sanitization, and classification
//! for tabular specifications derived from Level 0 OEM engineering requirements and ICDs.
//!
//! Realises: [REQ-SYSML-INGEST-TABLE]

use deap_core::markdown_ast::parse_markdown_ast;
use std::collections::{BTreeMap, BTreeSet};

/// SysML v2 reserved keywords that must not be used as bare identifiers in AST generation. Realises: [REQ-INGEST-SYSML-TABLE-RESERVED-KEYWORDS]
pub const RESERVED_SYSML_KEYWORDS: &[&str] = &[
    "package", "part", "def", "port", "attribute", "action", "flow", "item", "connect", "connection",
    "interface", "doc", "assert", "constraint", "state", "requirement", "use", "case", "test", "in",
    "out", "inout", "import", "public", "private", "alias", "perform", "subsystem", "first", "then",
    "step", "entry", "exit", "do", "transition", "assume", "require", "verify", "satisfy", "text",
];

/// Keywords identifying structural or functional section headers that are not component names. Realises: [REQ-INGEST-SYSML-TABLE-NON-COMPONENT-KEYWORDS]
pub const NON_COMPONENT_SECTION_KEYWORDS: &[&str] = &[
    "bill of materials", "bom", "components", "parts", "subsystems", "interfaces", "ports", "signals",
    "signal interface", "interconnects", "parameters", "parametric limits", "limits", "constraints",
    "invariants", "assertions", "requirements", "specifications", "overview", "introduction", "scope",
    "architecture", "notes", "glossary", "definitions", "summary", "system interfaces",
    "system parameters", "system limits", "system constraints",
];

/// Authoritative provenance column header names recognized during tabular extraction. Realises: [REQ-INGEST-SYSML-TABLE-PROVENANCE-COLUMNS]
pub const PROVENANCE_COLUMNS: &[&str] = &[
    "provenance_citation", "provenance", "source_reference", "oem_source_reference",
    "oem_document_citation", "citation", "reference", "source", "clause", "specification_reference",
];

/// Classification archetype for Markdown tables based on structural schema patterns. Realises: [REQ-INGEST-SYSML-TABLE-TABLETYPE]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableType {
    Bom,
    Ports,
    Constraints,
    Properties,
    Generic,
}

/// Parsed, normalized in-memory representation of a Markdown table. Realises: [REQ-INGEST-SYSML-TABLE-MARKDOWNTABLE]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MarkdownTable {
    pub raw_headers: Vec<String>,
    pub normalized_headers: Vec<String>,
    pub units: Vec<String>,
    pub rows: Vec<BTreeMap<String, String>>,
}

/// Sanitizes arbitrary text into a valid, deterministic SysML v2 / KerML identifier. Realises: [REQ-SYSML-INGEST-TABLE/sanitize_identifier]
pub fn sanitize_identifier(text: &str, default_name: &str) -> String {
    if text.is_empty() {
        return default_name.to_string();
    }
    let stripped = strip_bracketed_content(text);
    let clean_cand = if stripped.is_empty() { text } else { &stripped };
    let clean_raw = clean_cand
        .chars()
        .map(|c| if matches!(c, '*' | '`' | '_' | '#') { ' ' } else { c })
        .collect::<String>()
        .trim()
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != '_', "_");
    let mut clean: String = clean_raw.split('_').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("_");
    if clean.is_empty() {
        return default_name.to_string();
    }
    if clean.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        clean = format!("_{clean}");
    }
    if RESERVED_SYSML_KEYWORDS.contains(&clean.to_ascii_lowercase().as_str()) {
        clean = format!("{clean}_item");
    }
    clean
}

/// Strips parenthesized, bracketed, or braced content from `text`. Realises: [REQ-INGEST-SYSML-TABLE-STRIP-BRACKETED-CONTENT]
fn strip_bracketed_content(text: &str) -> String {
    let (mut out, mut chars) = (String::with_capacity(text.len()), text.chars());
    while let Some(c) = chars.next() {
        if matches!(c, '(' | '[' | '{') {
            for inner in chars.by_ref() {
                if matches!(inner, ')' | ']' | '}') {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out.trim().to_string()
}

/// Extracts measurement physical units enclosed in parentheses or brackets. Realises: [REQ-SYSML-INGEST-TABLE/extract_unit_from_header_or_val]
pub fn extract_unit_from_header_or_val(header_or_val: &str) -> (String, String) {
    let (mut unit, mut cleaned, mut chars) = (String::new(), String::with_capacity(header_or_val.len()), header_or_val.chars());
    while let Some(c) = chars.next() {
        if c == '(' || c == '[' {
            let (mut inner, mut closed) = (String::new(), false);
            for next_c in chars.by_ref() {
                if next_c == ')' || next_c == ']' {
                    closed = true;
                    break;
                }
                inner.push(next_c);
            }
            if closed && unit.is_empty() && !inner.is_empty() && inner.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '/' | '°' | '^' | '%' | '-')) {
                unit = inner.trim().to_string();
            }
        } else {
            cleaned.push(c);
        }
    }
    (cleaned.trim().to_string(), unit)
}

/// Normalizes a column header into a canonical snake_case lookup key. Realises: [REQ-SYSML-INGEST-TABLE/normalize_col]
pub fn normalize_col(name: &str) -> String {
    let (cleaned, _) = extract_unit_from_header_or_val(name);
    cleaned
        .to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// Replaces inline `<br>` / `<br/>` HTML line-break tags with spaces prior to CommonMark AST parsing. Realises: [REQ-INGEST-SYSML-TABLE-STRIP-HTML-TAGS]
fn replace_br_tags_with_space(input: &str) -> String {
    ["<br/>", "<br />", "<br>", "<BR/>", "<BR />", "<BR>"].into_iter().fold(input.to_string(), |acc, pat| acc.replace(pat, " "))
}

/// Strips Markdown hyperlinks, HTML tags, and inline formatting from a table cell value. Realises: [REQ-SYSML-INGEST-TABLE/clean_cell_value]
pub fn clean_cell_value(val: &str) -> String {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let normalized = replace_br_tags_with_space(trimmed);
    parse_markdown_ast(&format!("| {normalized} |\n| --- |\n"))
        .tables
        .first()
        .and_then(|t| t.headers.first())
        .map_or_else(|| normalized.trim().to_string(), |c| c.trim().to_string())
}

/// Parses a scalar string into numeric float, unit, and optional signed integer representation. Realises: [REQ-SYSML-INGEST-TABLE/parse_numeric_with_unit]
pub fn parse_numeric_with_unit(raw_val: &str) -> (Option<f64>, Option<String>, Option<i64>) {
    let clean: String = raw_val.chars().filter(|&c| c != '*' && c != '`').collect();
    let clean = clean.trim();
    if let Some(hex) = clean.strip_prefix("0x").or_else(|| clean.strip_prefix("0X")).filter(|h| !h.is_empty() && h.chars().all(|c| c.is_ascii_hexdigit())) {
        if let Ok(vi) = i64::from_str_radix(hex, 16) {
            return (Some(vi as f64), Some(String::new()), Some(vi));
        }
    }
    if let Some((val_str, unit_str)) = split_number_and_unit(clean) {
        let unit_str = unit_str.to_string();
        if val_str.contains('.') || val_str.to_ascii_lowercase().contains('e') {
            if let Ok(vf) = val_str.parse::<f64>() {
                return (Some(vf), Some(unit_str), None);
            }
        } else if let Ok(vi) = val_str.parse::<i64>() {
            return (Some(vi as f64), Some(unit_str), Some(vi));
        }
    }
    (None, None, None)
}

/// Splits a trimmed cell into `(numeric_prefix, unit_suffix)` if valid. Realises: [REQ-INGEST-SYSML-TABLE-SPLIT-NUMBER-AND-UNIT]
fn split_number_and_unit(s: &str) -> Option<(&str, &str)> {
    let bytes = s.as_bytes();
    let (mut idx, mut has_digits) = (usize::from(matches!(bytes.first(), Some(b'+' | b'-'))), false);
    while bytes.get(idx).is_some_and(u8::is_ascii_digit) {
        has_digits = true;
        idx += 1;
    }
    if bytes.get(idx) == Some(&b'.') {
        idx += 1;
        let mut has_dec = false;
        while bytes.get(idx).is_some_and(u8::is_ascii_digit) {
            has_dec = true;
            idx += 1;
        }
        has_digits = has_digits || has_dec;
    }
    if !has_digits {
        return None;
    }
    if matches!(bytes.get(idx), Some(b'e' | b'E')) {
        let exp_start = idx;
        idx += 1 + usize::from(matches!(bytes.get(idx + 1), Some(b'+' | b'-')));
        let mut has_exp = false;
        while bytes.get(idx).is_some_and(u8::is_ascii_digit) {
            has_exp = true;
            idx += 1;
        }
        if !has_exp {
            idx = exp_start;
        }
    }
    let (num_part, rest) = (s.get(..idx)?, s.get(idx..)?.trim_start());
    rest.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '/' | '°' | '^' | '%' | '-'))
        .then_some((num_part, rest))
}

/// Parses range bounds string e.g. `[0.0, 100.0]`, `-20.0 to 60.0`, `18.0 .. 25.2`. Realises: [REQ-SYSML-INGEST-TABLE/parse_range_bounds]
pub fn parse_range_bounds(range_text: &str) -> (Option<f64>, Option<f64>) {
    let clean: String = range_text.chars().filter(|&c| !matches!(c, '*' | '`' | '[' | ']' | '(' | ')')).collect();
    let parts = split_range_parts(clean.trim());
    if let (Some(&p0), Some(&p1)) = (parts.first(), parts.get(1)) {
        if let (Some(v1), Some(v2)) = (parse_numeric_with_unit(p0).0, parse_numeric_with_unit(p1).0) {
            return (Some(v1.min(v2)), Some(v1.max(v2)));
        }
    }
    (None, None)
}

/// Splits a range string across `,`, `:`, `..`, or `to` delimiters. Realises: [REQ-INGEST-SYSML-TABLE-SPLIT-RANGE-PARTS]
fn split_range_parts(s: &str) -> Vec<&str> {
    let (mut parts, mut start, mut i, s_bytes) = (Vec::new(), 0, 0, s.as_bytes());
    while i < s.len() {
        let (cur, rest) = (s_bytes.get(i).copied(), s.get(i..).unwrap_or(""));
        let is_to = rest.starts_with("to")
            && (i == 0 || s_bytes.get(i.saturating_sub(1)).is_some_and(u8::is_ascii_whitespace))
            && (i + 2 == s.len() || s_bytes.get(i + 2).is_some_and(u8::is_ascii_whitespace));
        if matches!(cur, Some(b',' | b':')) || rest.starts_with("..") || is_to {
            if let Some(seg) = s.get(start..i) {
                parts.push(seg.trim());
            }
            i += if matches!(cur, Some(b',' | b':')) { 1 } else { 2 };
            start = i;
        } else {
            i += 1;
        }
    }
    if let Some(seg) = s.get(start..).filter(|seg| !seg.is_empty()) {
        parts.push(seg.trim());
    }
    parts
}

/// Extracts authoritative provenance citation from a row dictionary. Realises: [REQ-SYSML-INGEST-TABLE/extract_provenance_citation]
pub fn extract_provenance_citation(row: &BTreeMap<String, String>, headers: &[String]) -> String {
    headers
        .iter()
        .filter(|h| PROVENANCE_COLUMNS.contains(&h.as_str()))
        .find_map(|h| row.get(h).map(|v| v.trim()).filter(|v| !v.is_empty()))
        .unwrap_or_default()
        .to_string()
}

/// Composes a grounded SysML v2 documentation string preserving description, unit, and OEM citation. Realises: [REQ-SYSML-INGEST-TABLE/compose_grounded_doc]
pub fn compose_grounded_doc(description: &str, citation: &str, unit: &str) -> String {
    let mut parts = Vec::with_capacity(3);
    if !description.trim().is_empty() { parts.push(description.trim().to_string()); }
    if !unit.trim().is_empty() { parts.push(format!("[unit: {}]", unit.trim())); }
    if !citation.trim().is_empty() { parts.push(format!("[Source: {}]", citation.trim())); }
    parts.join(" ")
}

/// Classifies table type based purely on structural column header patterns. Realises: [REQ-SYSML-INGEST-TABLE/classify_table]
pub fn classify_table(headers: &[String]) -> TableType {
    let h_set: BTreeSet<&str> = headers.iter().map(String::as_str).collect();
    let has_any = |keys: &[&str]| keys.iter().any(|k| h_set.contains(k));
    let has_bom_name = has_any(&["component", "component_name", "part", "part_name", "subsystem", "item", "module", "assembly", "lru"]);
    let has_bom_attr = has_any(&["part_number", "pn", "sku", "mass", "weight", "power", "qty", "quantity", "cost"]);
    if (has_bom_name && has_bom_attr) || (has_bom_name && headers.len() >= 3 && !has_any(&["direction", "dir", "port", "signal"])) {
        return TableType::Bom;
    }
    if has_any(&["port", "port_name", "signal", "signal_name", "signal_id", "interface", "interface_name", "flow"])
        || (has_any(&["direction", "dir", "flow_direction"]) && has_any(&["type", "data_type", "payload", "rate", "protocol"]))
        || (h_set.contains("source_port") && h_set.contains("target_port"))
        || (h_set.contains("source") && h_set.contains("target"))
    {
        return TableType::Ports;
    }
    if has_any(&["min", "max", "lower_bound", "upper_bound", "range", "valid_range", "limit", "bound", "bounds", "tolerance", "constraint", "assertion"])
        && (has_any(&["parameter", "param", "property", "metric", "variable", "name"]) || headers.len() >= 2)
    {
        return TableType::Constraints;
    }
    if headers.first().filter(|_| (2..=3).contains(&headers.len())).is_some_and(|f| ["property", "parameter", "attribute", "key", "field", "name"].iter().any(|k| f.contains(k))) {
        return TableType::Properties;
    }
    TableType::Generic
}

/// Normalizes input lines into a GFM table-ready Markdown buffer. Realises: [REQ-SYSML-INGEST-TABLE/parse_markdown_tables]
fn prepare_table_markdown(lines: &[&str]) -> String {
    let (mut buf, mut in_tbl) = (String::new(), false);
    for (i, &raw) in lines.iter().enumerate() {
        let trimmed = raw.trim();
        let next_is_delim = lines.get(i + 1).copied().is_some_and(|nl| {
            let t = nl.trim();
            t.contains('|') && t.trim_matches('|').split('|').all(is_table_delimiter_cell)
        });
        if trimmed.contains('|') && next_is_delim && !in_tbl {
            if !buf.is_empty() && !buf.ends_with("\n\n") {
                buf.push('\n');
            }
            in_tbl = true;
        } else if !trimmed.contains('|') {
            in_tbl = false;
        }
        buf.push_str(if in_tbl { trimmed } else { raw });
        buf.push('\n');
    }
    buf
}

/// Parses lines of Markdown text into a sequence of structured [`MarkdownTable`] instances. Realises: [REQ-SYSML-INGEST-TABLE/parse_markdown_tables]
pub fn parse_markdown_tables(lines: &[&str]) -> Vec<MarkdownTable> {
    let markdown = replace_br_tags_with_space(&prepare_table_markdown(lines));
    let ast = parse_markdown_ast(&markdown);
    let mut tables = Vec::with_capacity(ast.tables.len());
    for tbl in ast.tables {
        if tbl.headers.is_empty() || tbl.rows.is_empty() {
            continue;
        }
        let raw_headers = tbl.headers;
        let normalized_headers: Vec<String> = raw_headers.iter().map(|c| normalize_col(c)).collect();
        let units: Vec<String> = raw_headers.iter().map(|c| extract_unit_from_header_or_val(c).1).collect();
        let mut rows = Vec::with_capacity(tbl.rows.len());
        for raw_row in tbl.rows {
            if raw_row.is_empty() || raw_row.iter().all(|c| is_table_delimiter_cell(c)) {
                continue;
            }
            let row_dict = normalized_headers
                .iter()
                .enumerate()
                .map(|(idx, col)| (col.clone(), raw_row.get(idx).map(|c| clean_cell_value(c)).unwrap_or_default()))
                .collect();
            rows.push(row_dict);
        }
        if !rows.is_empty() {
            tables.push(MarkdownTable { raw_headers, normalized_headers, units, rows });
        }
    }
    tables
}

/// Checks if `s` is a GFM table alignment delimiter cell (`---`, `:---`, `:---:`, `---:`). Realises: [REQ-INGEST-SYSML-TABLE-IS-TABLE-DELIMITER-CELL]
pub(crate) fn is_table_delimiter_cell(s: &str) -> bool {
    let w = s.trim().trim_matches(':');
    !w.is_empty() && w.chars().all(|c| c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_and_table_helpers() {
        for (inp, fb, exp) in [("Signal_Alpha (Hz)", "Item", "Signal_Alpha"), ("[REQ-0001] Abstract Spec", "Item", "Abstract_Spec"), ("1. Normative Statement", "Item", "_1_Normative_Statement"), ("subsystem", "Item", "subsystem_item"), ("AC-01: Pure Schema-Driven", "Item", "AC_01_Pure_Schema_Driven"), ("port", "Item", "port_item"), ("text", "Item", "text_item"), ("", "Fallback", "Fallback")] {
            assert_eq!(sanitize_identifier(inp, fb), exp);
        }
        assert_eq!(extract_unit_from_header_or_val("Param_Alpha (kg)"), ("Param_Alpha".to_string(), "kg".to_string()));
        assert_eq!(extract_unit_from_header_or_val("Rate_Beta [MHz]"), ("Rate_Beta".to_string(), "MHz".to_string()));
        assert_eq!(normalize_col("Requirement ID"), "requirement_id");
        assert_eq!(normalize_col("**Deterministic UUIDv5**"), "deterministic_uuidv5");
        assert_eq!(normalize_col("Param_Alpha (kg)"), "param_alpha");
        assert_eq!(clean_cell_value("[Standard 101](https://example.com)"), "Standard 101");
        assert_eq!(clean_cell_value("Line 1<br/>Line 2"), "Line 1 Line 2");
        assert_eq!(clean_cell_value("<b>Bold</b> text"), "Bold text");
        assert_eq!(parse_numeric_with_unit("1800.0 kg"), (Some(1800.0), Some("kg".to_string()), None));
        assert_eq!(parse_numeric_with_unit("42"), (Some(42.0), Some(String::new()), Some(42)));
        assert_eq!(parse_numeric_with_unit("0x1A").2, Some(26));
        assert_eq!(parse_range_bounds("[0.0, 100.0]"), (Some(0.0), Some(100.0)));
        assert_eq!(parse_range_bounds("-20.0 to 60.0"), (Some(-20.0), Some(60.0)));
        assert_eq!(parse_range_bounds("18.0 .. 25.2"), (Some(18.0), Some(25.2)));
    }
}
