//! # Markdown Table Parsing, Column Normalization & Cell Sanitization Engine
//!
//! Deterministic parsing, column normalization, cell value sanitization, and classification
//! for tabular specifications derived from Level 0 OEM engineering requirements and ICDs.
//!
//! Realises: [REQ-SYSML-INGEST-TABLE]

use std::collections::{BTreeMap, BTreeSet};

/// SysML v2 reserved keywords that must not be used as bare identifiers in AST generation.
/// Realises: [REQ-INGEST-SYSML-TABLE-RESERVED-KEYWORDS]
pub const RESERVED_SYSML_KEYWORDS: &[&str] = &[
    "package", "part", "def", "port", "attribute", "action", "flow", "item", "connect",
    "connection", "interface", "doc", "assert", "constraint", "state", "requirement", "use",
    "case", "test", "in", "out", "inout", "import", "public", "private", "alias", "perform",
    "subsystem", "first", "then", "step", "entry", "exit", "do", "transition", "assume",
    "require", "verify", "satisfy", "text",
];

/// Keywords identifying structural or functional section headers that are not component names.
/// Realises: [REQ-INGEST-SYSML-TABLE-NON-COMPONENT-KEYWORDS]
pub const NON_COMPONENT_SECTION_KEYWORDS: &[&str] = &[
    "bill of materials", "bom", "components", "parts", "subsystems", "interfaces", "ports",
    "signals", "signal interface", "interconnects", "parameters", "parametric limits", "limits",
    "constraints", "invariants", "assertions", "requirements", "specifications", "overview",
    "introduction", "scope", "architecture", "notes", "glossary", "definitions", "summary",
    "system interfaces", "system parameters", "system limits", "system constraints",
];

/// Authoritative provenance column header names recognized during tabular extraction.
/// Realises: [REQ-INGEST-SYSML-TABLE-PROVENANCE-COLUMNS]
pub const PROVENANCE_COLUMNS: &[&str] = &[
    "provenance_citation", "provenance", "source_reference", "oem_source_reference",
    "oem_document_citation", "citation", "reference", "source", "clause", "specification_reference",
];

/// Classification archetype for Markdown tables based on structural schema patterns.
/// Realises: [REQ-INGEST-SYSML-TABLE-TABLETYPE]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableType {
    Bom,
    Ports,
    Constraints,
    Properties,
    Generic,
}

/// Parsed, normalized in-memory representation of a Markdown table.
/// Realises: [REQ-INGEST-SYSML-TABLE-MARKDOWNTABLE]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MarkdownTable {
    pub raw_headers: Vec<String>,
    pub normalized_headers: Vec<String>,
    pub units: Vec<String>,
    pub rows: Vec<BTreeMap<String, String>>,
}

/// Sanitizes arbitrary text into a valid, deterministic SysML v2 / KerML identifier.
/// Realises: [REQ-SYSML-INGEST-TABLE/sanitize_identifier]
pub fn sanitize_identifier(text: &str, default_name: &str) -> String {
    if text.is_empty() {
        return default_name.to_string();
    }
    let stripped = strip_bracketed_content(text);
    let clean_cand = if !stripped.is_empty() {
        stripped
    } else {
        text.to_string()
    };
    let formatted: String = clean_cand
        .chars()
        .map(|c| if matches!(c, '*' | '`' | '_' | '#') { ' ' } else { c })
        .collect();
    let mut clean = formatted
        .trim()
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != '_', "_");
    clean = collapse_underscores(&clean).trim_matches('_').to_string();
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

/// Strips parenthesized, bracketed, or braced content from `text`.
/// Realises: [REQ-INGEST-SYSML-TABLE-STRIP-BRACKETED-CONTENT]
fn strip_bracketed_content(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
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

/// Collapses consecutive underscore characters into a single underscore.
/// Realises: [REQ-INGEST-SYSML-TABLE-COLLAPSE-UNDERSCORES]
fn collapse_underscores(s: &str) -> String {
    let mut res = String::with_capacity(s.len());
    let mut prev_under = false;
    for c in s.chars() {
        if c == '_' {
            if !prev_under {
                res.push('_');
                prev_under = true;
            }
        } else {
            res.push(c);
            prev_under = false;
        }
    }
    res
}

/// Extracts measurement physical units enclosed in parentheses or brackets.
/// Realises: [REQ-SYSML-INGEST-TABLE/extract_unit_from_header_or_val]
pub fn extract_unit_from_header_or_val(header_or_val: &str) -> (String, String) {
    let mut unit = String::new();
    let mut cleaned = String::with_capacity(header_or_val.len());
    let mut chars = header_or_val.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '(' || c == '[' {
            let mut inner = String::new();
            let mut closed = false;
            for next_c in chars.by_ref() {
                if next_c == ')' || next_c == ']' {
                    closed = true;
                    break;
                }
                inner.push(next_c);
            }
            if closed && unit.is_empty() {
                let is_valid_unit = !inner.is_empty()
                    && inner.chars().all(|ch| {
                        ch.is_ascii_alphanumeric()
                            || matches!(ch, '_' | '/' | '°' | '^' | '%' | '-')
                    });
                if is_valid_unit {
                    unit = inner.trim().to_string();
                }
            }
        } else {
            cleaned.push(c);
        }
    }
    (cleaned.trim().to_string(), unit)
}

/// Normalizes a column header into a canonical snake_case lookup key.
/// Realises: [REQ-SYSML-INGEST-TABLE/normalize_col]
pub fn normalize_col(name: &str) -> String {
    let (cleaned, _) = extract_unit_from_header_or_val(name);
    let mut clean = String::with_capacity(cleaned.len());
    let mut prev_under = false;
    for c in cleaned.chars() {
        let lower = c.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            clean.push(lower);
            prev_under = false;
        } else if !prev_under {
            clean.push('_');
            prev_under = true;
        }
    }
    clean.trim_matches('_').to_string()
}

/// Strips Markdown hyperlinks, HTML tags, and inline formatting from a table cell value.
/// Realises: [REQ-SYSML-INGEST-TABLE/clean_cell_value]
pub fn clean_cell_value(val: &str) -> String {
    if val.is_empty() {
        return String::new();
    }
    let step1 = strip_markdown_links(val);
    strip_html_tags(&step1).trim().to_string()
}

/// Strips Markdown `[label](url)` link syntax while retaining `label`.
/// Realises: [REQ-INGEST-SYSML-TABLE-STRIP-MARKDOWN-LINKS]
fn strip_markdown_links(val: &str) -> String {
    let mut out = String::with_capacity(val.len());
    let mut rest = val;
    while let Some(start_bracket) = rest.find('[') {
        out.push_str(&rest[..start_bracket]);
        let after_start = &rest[start_bracket + 1..];
        if let Some(end_bracket) = after_start.find(']') {
            let visible_text = &after_start[..end_bracket];
            let after_end_bracket = &after_start[end_bracket + 1..];
            if after_end_bracket.starts_with('(') {
                if let Some(end_paren) = after_end_bracket.find(')') {
                    out.push_str(visible_text);
                    rest = &after_end_bracket[end_paren + 1..];
                    continue;
                }
            }
        }
        out.push('[');
        rest = after_start;
    }
    out.push_str(rest);
    out
}

/// Strips inline HTML tags while replacing `<br>` / `<br/>` tags with single spaces.
/// Realises: [REQ-INGEST-SYSML-TABLE-STRIP-HTML-TAGS]
fn strip_html_tags(val: &str) -> String {
    let mut out = String::with_capacity(val.len());
    let mut rest = val;
    while let Some(start_tag) = rest.find('<') {
        out.push_str(&rest[..start_tag]);
        let after_start = &rest[start_tag + 1..];
        if let Some(end_tag) = after_start.find('>') {
            let lower = after_start[..end_tag].trim().to_ascii_lowercase();
            if lower == "br"
                || lower.starts_with("br ")
                || lower.starts_with("br/")
                || lower == "br/"
            {
                out.push(' ');
            }
            rest = &after_start[end_tag + 1..];
        } else {
            out.push('<');
            rest = after_start;
        }
    }
    out.push_str(rest);
    out
}

/// Parses a scalar string into numeric float, unit, and optional signed integer representation.
/// Realises: [REQ-SYSML-INGEST-TABLE/parse_numeric_with_unit]
pub fn parse_numeric_with_unit(raw_val: &str) -> (Option<f64>, Option<String>, Option<i64>) {
    let clean: String = raw_val.chars().filter(|&c| c != '*' && c != '`').collect();
    let clean = clean.trim();
    if (clean.starts_with("0x") || clean.starts_with("0X")) && clean.len() > 2 {
        let hex_str = &clean[2..];
        if hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
            if let Ok(val_int) = i64::from_str_radix(hex_str, 16) {
                return (Some(val_int as f64), Some(String::new()), Some(val_int));
            }
        }
    }
    if let Some((val_str, unit_str)) = split_number_and_unit(clean) {
        let unit_str = unit_str.to_string();
        if val_str.contains('.') || val_str.to_ascii_lowercase().contains('e') {
            if let Ok(val_float) = val_str.parse::<f64>() {
                return (Some(val_float), Some(unit_str), None);
            }
        } else if let Ok(val_int) = val_str.parse::<i64>() {
            return (Some(val_int as f64), Some(unit_str), Some(val_int));
        }
    }
    (None, None, None)
}

/// Splits a trimmed cell into `(numeric_prefix, unit_suffix)` if valid.
/// Realises: [REQ-INGEST-SYSML-TABLE-SPLIT-NUMBER-AND-UNIT]
fn split_number_and_unit(s: &str) -> Option<(&str, &str)> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let mut idx = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let mut has_digits = false;
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
        if !has_digits && !has_dec {
            return None;
        }
        has_digits = true;
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
    let num_part = s.get(..idx)?;
    let rest = s.get(idx..)?.trim_start();
    rest.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '/' | '°' | '^' | '%' | '-'))
        .then_some((num_part, rest))
}

/// Parses range bounds string e.g. `[0.0, 100.0]`, `-20.0 to 60.0`, `18.0 .. 25.2`.
/// Realises: [REQ-SYSML-INGEST-TABLE/parse_range_bounds]
pub fn parse_range_bounds(range_text: &str) -> (Option<f64>, Option<f64>) {
    let clean: String = range_text
        .chars()
        .filter(|&c| !matches!(c, '*' | '`' | '[' | ']' | '(' | ')'))
        .collect();
    let parts = split_range_parts(clean.trim());
    if let (Some(&p0), Some(&p1)) = (parts.first(), parts.get(1)) {
        if let (Some(v1), Some(v2)) = (parse_numeric_with_unit(p0).0, parse_numeric_with_unit(p1).0)
        {
            return (Some(v1.min(v2)), Some(v1.max(v2)));
        }
    }
    (None, None)
}

/// Splits a range string across `,`, `:`, `..`, or `to` delimiters.
/// Realises: [REQ-INGEST-SYSML-TABLE-SPLIT-RANGE-PARTS]
fn split_range_parts(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let (mut start, mut i, s_bytes) = (0, 0, s.as_bytes());
    while i < s.len() {
        let cur = s_bytes.get(i).copied();
        let rest = s.get(i..).unwrap_or("");
        if matches!(cur, Some(b',' | b':')) {
            if let Some(seg) = s.get(start..i) {
                parts.push(seg.trim());
            }
            i += 1;
            start = i;
        } else if rest.starts_with("..")
            || (rest.starts_with("to")
                && (i == 0
                    || s_bytes
                        .get(i.saturating_sub(1))
                        .is_some_and(u8::is_ascii_whitespace))
                && (i + 2 == s.len()
                    || s_bytes.get(i + 2).is_some_and(u8::is_ascii_whitespace)))
        {
            if let Some(seg) = s.get(start..i) {
                parts.push(seg.trim());
            }
            i += 2;
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

/// Extracts authoritative provenance citation from a row dictionary.
/// Realises: [REQ-SYSML-INGEST-TABLE/extract_provenance_citation]
pub fn extract_provenance_citation(row: &BTreeMap<String, String>, headers: &[String]) -> String {
    for h in headers {
        if PROVENANCE_COLUMNS.contains(&h.as_str()) {
            if let Some(val) = row.get(h).map(|v| v.trim()).filter(|v| !v.is_empty()) {
                return val.to_string();
            }
        }
    }
    String::new()
}

/// Composes a grounded SysML v2 documentation string preserving description, unit, and OEM citation.
/// Realises: [REQ-SYSML-INGEST-TABLE/compose_grounded_doc]
pub fn compose_grounded_doc(description: &str, citation: &str, unit: &str) -> String {
    let mut tokens = Vec::new();
    if !description.trim().is_empty() {
        tokens.push(description.trim().to_string());
    }
    if !unit.trim().is_empty() {
        tokens.push(format!("[unit: {}]", unit.trim()));
    }
    if !citation.trim().is_empty() {
        tokens.push(format!("[Source: {}]", citation.trim()));
    }
    tokens.join(" ").trim().to_string()
}

/// Classifies table type based purely on structural column header patterns.
/// Realises: [REQ-SYSML-INGEST-TABLE/classify_table]
pub fn classify_table(headers: &[String]) -> TableType {
    let h_set: BTreeSet<&str> = headers.iter().map(String::as_str).collect();
    let has_bom_name = [
        "component", "component_name", "part", "part_name", "subsystem", "item", "module",
        "assembly", "lru",
    ]
    .iter()
    .any(|k| h_set.contains(k));
    let has_bom_attr = [
        "part_number", "pn", "sku", "mass", "weight", "power", "qty", "quantity", "cost",
    ]
    .iter()
    .any(|k| h_set.contains(k));
    if (has_bom_name && has_bom_attr)
        || (has_bom_name
            && headers.len() >= 3
            && !["direction", "dir", "port", "signal"]
                .iter()
                .any(|h| h_set.contains(h)))
    {
        return TableType::Bom;
    }
    let has_port_name = [
        "port", "port_name", "signal", "signal_name", "signal_id", "interface", "interface_name",
        "flow",
    ]
    .iter()
    .any(|k| h_set.contains(k));
    let has_dir = ["direction", "dir", "flow_direction"]
        .iter()
        .any(|k| h_set.contains(k));
    let has_type = ["type", "data_type", "payload", "rate", "protocol"]
        .iter()
        .any(|k| h_set.contains(k));
    if has_port_name
        || (has_dir && has_type)
        || (h_set.contains("source_port") && h_set.contains("target_port"))
        || (h_set.contains("source") && h_set.contains("target"))
    {
        return TableType::Ports;
    }
    let has_constraint = [
        "min", "max", "lower_bound", "upper_bound", "range", "valid_range", "limit", "bound",
        "bounds", "tolerance", "constraint", "assertion",
    ]
    .iter()
    .any(|k| h_set.contains(k));
    let has_param = ["parameter", "param", "property", "metric", "variable", "name"]
        .iter()
        .any(|k| h_set.contains(k));
    if has_constraint && (has_param || headers.len() >= 2) {
        return TableType::Constraints;
    }
    if let Some(first) = headers.first().filter(|_| (2..=3).contains(&headers.len())) {
        if ["property", "parameter", "attribute", "key", "field", "name"]
            .iter()
            .any(|k| first.contains(k))
        {
            return TableType::Properties;
        }
    }
    TableType::Generic
}

/// Parses lines of Markdown text into a sequence of structured [`MarkdownTable`] instances.
/// Realises: [REQ-SYSML-INGEST-TABLE/parse_markdown_tables]
pub fn parse_markdown_tables(lines: &[&str]) -> Vec<MarkdownTable> {
    let mut tables = Vec::with_capacity(
        deap_core::markdown_ast::parse_markdown_ast(&lines.join("\n"))
            .tables
            .len(),
    );
    let mut in_table = false;
    let (mut raw_headers, mut norm_headers, mut units, mut rows) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut flush_table = |in_tbl: &mut bool,
                           raw_h: &mut Vec<String>,
                           norm_h: &mut Vec<String>,
                           u: &mut Vec<String>,
                           r: &mut Vec<BTreeMap<String, String>>| {
        if *in_tbl && !raw_h.is_empty() && !r.is_empty() {
            tables.push(MarkdownTable {
                raw_headers: raw_h.clone(),
                normalized_headers: norm_h.clone(),
                units: u.clone(),
                rows: r.clone(),
            });
        }
        *in_tbl = false;
        raw_h.clear();
        norm_h.clear();
        u.clear();
        r.clear();
    };
    let (mut i, n) = (0, lines.len());
    while i < n {
        let line = lines.get(i).copied().unwrap_or("").trim();
        if line.contains('|') {
            let cells: Vec<String> = line
                .trim_matches('|')
                .split('|')
                .map(|c| c.trim().to_string())
                .collect();
            let is_delimiter =
                !cells.is_empty() && cells.iter().all(|c| is_table_delimiter_cell(c));
            if is_delimiter && !in_table {
                if i > 0 {
                    let prev_line = lines.get(i - 1).copied().unwrap_or("").trim();
                    if prev_line.contains('|') {
                        let prev_cells: Vec<String> = prev_line
                            .trim_matches('|')
                            .split('|')
                            .map(|c| c.trim().to_string())
                            .collect();
                        raw_headers = prev_cells.clone();
                        norm_headers = prev_cells.iter().map(|c| normalize_col(c)).collect();
                        units = prev_cells
                            .iter()
                            .map(|c| extract_unit_from_header_or_val(c).1)
                            .collect();
                        in_table = true;
                        i += 1;
                        continue;
                    }
                }
            } else if is_delimiter && in_table {
                i += 1;
                continue;
            } else if in_table {
                let mut row_dict = BTreeMap::new();
                for (idx, norm_col) in norm_headers.iter().enumerate() {
                    let val = cells
                        .get(idx)
                        .map(|c| clean_cell_value(c))
                        .unwrap_or_default();
                    row_dict.insert(norm_col.clone(), val);
                }
                rows.push(row_dict);
                i += 1;
                continue;
            } else if let Some(raw_next) = lines.get(i + 1).copied().filter(|nl| nl.contains('|')) {
                let next_cells: Vec<String> = raw_next
                    .trim()
                    .trim_matches('|')
                    .split('|')
                    .map(|c| c.trim().to_string())
                    .collect();
                if !next_cells.is_empty() && next_cells.iter().all(|c| is_table_delimiter_cell(c))
                {
                    flush_table(
                        &mut in_table,
                        &mut raw_headers,
                        &mut norm_headers,
                        &mut units,
                        &mut rows,
                    );
                    raw_headers = cells.clone();
                    norm_headers = cells.iter().map(|c| normalize_col(c)).collect();
                    units = cells
                        .iter()
                        .map(|c| extract_unit_from_header_or_val(c).1)
                        .collect();
                    in_table = true;
                    i += 2;
                    continue;
                }
            }
        } else if in_table {
            flush_table(
                &mut in_table,
                &mut raw_headers,
                &mut norm_headers,
                &mut units,
                &mut rows,
            );
        }
        i += 1;
    }
    flush_table(
        &mut in_table,
        &mut raw_headers,
        &mut norm_headers,
        &mut units,
        &mut rows,
    );
    tables
}

/// Checks if `s` is a GFM table alignment delimiter cell (`---`, `:---`, `:---:`, `---:`).
/// Realises: [REQ-INGEST-SYSML-TABLE-IS-TABLE-DELIMITER-CELL]
pub(crate) fn is_table_delimiter_cell(s: &str) -> bool {
    let trimmed = s.trim();
    let without_left = trimmed.strip_prefix(':').unwrap_or(trimmed);
    let without_colons = without_left.strip_suffix(':').unwrap_or(without_left);
    !without_colons.is_empty() && without_colons.chars().all(|c| c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(sanitize_identifier("Signal_Alpha (Hz)", "Item"), "Signal_Alpha");
        assert_eq!(sanitize_identifier("[REQ-0001] Abstract Spec", "Item"), "Abstract_Spec");
        assert_eq!(sanitize_identifier("1. Normative Statement", "Item"), "_1_Normative_Statement");
        assert_eq!(sanitize_identifier("subsystem", "Item"), "subsystem_item");
        assert_eq!(sanitize_identifier("AC-01: Pure Schema-Driven", "Item"), "AC_01_Pure_Schema_Driven");
        assert_eq!(sanitize_identifier("port", "Item"), "port_item");
        assert_eq!(sanitize_identifier("text", "Item"), "text_item");
        assert_eq!(sanitize_identifier("", "Fallback"), "Fallback");
    }

    #[test]
    fn test_extract_unit_and_normalize_col() {
        assert_eq!(extract_unit_from_header_or_val("Param_Alpha (kg)"), ("Param_Alpha".to_string(), "kg".to_string()));
        assert_eq!(extract_unit_from_header_or_val("Rate_Beta [MHz]"), ("Rate_Beta".to_string(), "MHz".to_string()));
        assert_eq!(normalize_col("Requirement ID"), "requirement_id");
        assert_eq!(normalize_col("**Deterministic UUIDv5**"), "deterministic_uuidv5");
        assert_eq!(normalize_col("Param_Alpha (kg)"), "param_alpha");
    }

    #[test]
    fn test_clean_cell_value() {
        assert_eq!(clean_cell_value("[Standard 101](https://example.com)"), "Standard 101");
        assert_eq!(clean_cell_value("Line 1<br/>Line 2"), "Line 1 Line 2");
        assert_eq!(clean_cell_value("<b>Bold</b> text"), "Bold text");
    }

    #[test]
    fn test_parse_numeric_with_unit() {
        assert_eq!(parse_numeric_with_unit("1800.0 kg"), (Some(1800.0), Some("kg".to_string()), None));
        assert_eq!(parse_numeric_with_unit("42"), (Some(42.0), Some(String::new()), Some(42)));
        assert_eq!(parse_numeric_with_unit("0x1A").2, Some(26));
    }

    #[test]
    fn test_parse_range_bounds() {
        assert_eq!(parse_range_bounds("[0.0, 100.0]"), (Some(0.0), Some(100.0)));
        assert_eq!(parse_range_bounds("-20.0 to 60.0"), (Some(-20.0), Some(60.0)));
        assert_eq!(parse_range_bounds("18.0 .. 25.2"), (Some(18.0), Some(25.2)));
    }
}
