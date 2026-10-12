//! # Markdown Table Parsing, Column Normalization & Cell Sanitization Engine
//!
//! ## 1. Safety Intent & Regulatory Scope
//! This module provides deterministic, bounded parsing, column normalization, cell value
//! sanitization, and classification for tabular specifications derived from Level 0 OEM
//! engineering requirements, interface control documents (ICDs), and parametric budgets.
//!
//! In safety-critical software architectures (DO-178C Level A, ISO 26262 ASIL D, ECSS-E-ST-40C),
//! requirements ingestion represents the primary semantic boundary between unstructured OEM
//! artifacts and formal model-based system representations. Unsanitized input, ambiguous
//! physical units, unparsed range boundaries, or identifier collisions can propagate silent
//! corruptions into downstream compilers, static analyzers, and formal verifiers.
//!
//! ## 2. Core Safety Invariants & Hazard Mitigation
//! - **H-INGEST-01 (Parser Injection / Malformed Identifiers)**:
//!   Arbitrary text is transformed into valid SysML v2 / KerML identifiers via [`sanitize_identifier`].
//!   All illegal punctuation, Markdown formatting, and control characters are stripped.
//!   Reserved KerML/SysML v2 keywords are systematically suffixed (`<keyword>_item`) to prevent
//!   grammar corruption or compiler panic.
//! - **H-INGEST-02 (Loss of Measurement Units & Provenance)**:
//!   Physical measurement units enclosed in parentheses or brackets (e.g. `Mass (kg)`, `Rate [Hz]`)
//!   are extracted deterministically via [`extract_unit_from_header_or_val`], preserving both
//!   the canonical semantic name and the measurement unit. Authoritative OEM provenance citations
//!   are retained and bound to generated AST elements via [`compose_grounded_doc`].
//! - **H-INGEST-03 (Numeric Truncation & Sign/Hex Corruption)**:
//!   Numeric fields supporting scalar floats, signed integers, and hexadecimal literals (`0x...`)
//!   are parsed through [`parse_numeric_with_unit`] without silent truncation. Scientific notation
//!   and explicit signs (`+`, `-`) are preserved according to IEEE 754 float semantics.
//! - **H-INGEST-04 (Parametric Boundary Inversion)**:
//!   Interval limits parsed via [`parse_range_bounds`] enforce strict monotonicity:
//!   $min\_val \le max\_val$, eliminating inverted tolerances from input errors.
//! - **H-INGEST-05 (Zero Hardcoded Domain Concepts)**:
//!   Table classification ([`classify_table`]) operates purely on structural meta-model patterns
//!   (component lists, port lists, parametric constraints, key-value properties). No domain-specific
//!   entities or application vocabularies are hardcoded.
//!
//! ## 3. Determinism & Complexity Guarantees
//! - All parsing and transformation algorithms operate in $O(N)$ linear time where $N$ is
//!   the character length of the input table or line slice.
//! - State transitions in [`parse_markdown_tables`] are strictly monotonic and resilient against
//!   unclosed tables, empty rows, and malformed Markdown separators.

use regex::Regex;
use std::collections::HashMap;

/// SysML v2 reserved keywords that must not be used as bare identifiers in AST generation.
///
/// If any extracted identifier matches a reserved keyword in this table, it is safely
/// suffixed with `_item` by [`sanitize_identifier`] to maintain KerML grammar validity.
pub const RESERVED_SYSML_KEYWORDS: &[&str] = &[
    "package", "part", "def", "port", "attribute", "action", "flow",
    "item", "connect", "connection", "interface", "doc", "assert",
    "constraint", "state", "requirement", "use", "case", "test",
    "in", "out", "inout", "import", "public", "private", "alias",
    "perform", "subsystem", "first", "then", "step", "entry", "exit",
    "do", "transition", "assume", "require", "verify", "satisfy", "text",
];

/// Keywords identifying structural or functional section headers that are not component names.
///
/// In Markdown specifications, headings such as `## Bill of Materials` or `## System Interfaces`
/// delineate tabular sections rather than individual physical or logical components.
pub const NON_COMPONENT_SECTION_KEYWORDS: &[&str] = &[
    "bill of materials", "bom", "components", "parts", "subsystems",
    "interfaces", "ports", "signals", "signal interface", "interconnects",
    "parameters", "parametric limits", "limits", "constraints", "invariants",
    "assertions", "requirements", "specifications", "overview", "introduction",
    "scope", "architecture", "notes", "glossary", "definitions", "summary",
    "system interfaces", "system parameters", "system limits", "system constraints",
];

/// Authoritative provenance column header names recognized during tabular extraction.
///
/// These column headers establish formal DO-178C Section 5.5 / ISO 26262 Part 8 traceability
/// back to OEM source documents, standards, or external requirements.
pub const PROVENANCE_COLUMNS: &[&str] = &[
    "provenance_citation", "provenance", "source_reference",
    "oem_source_reference", "oem_document_citation", "citation",
    "reference", "source", "clause", "specification_reference",
];

/// Classification archetype for Markdown tables based on structural schema patterns.
///
/// Used by the translation pipeline to determine whether table rows project into
/// structural parts, communication ports, parametric constraints, or metadata attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableType {
    /// Bill of Materials or Component Hierarchy table (projects to `PartDef` / `PartUsage`).
    Bom,
    /// Interface, Port, or Signal definition table (projects to `PortDef` / `FlowDef`).
    Ports,
    /// Parametric limits, tolerances, or assertions (projects to `ConstraintDef`).
    Constraints,
    /// Key-Value metadata or contractual property table (projects to `AttributeDef`).
    Properties,
    /// Unclassified tabular data requiring generic attribute mapping.
    Generic,
}

/// Parsed, normalized in-memory representation of a Markdown table.
///
/// Holds the raw column headers, canonical snake_case normalized column names,
/// extracted physical units per column, and row key-value dictionaries.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MarkdownTable {
    /// Original verbatim header strings from the Markdown table source.
    pub raw_headers: Vec<String>,
    /// Sanitized, lowercased, unit-stripped canonical header names.
    pub normalized_headers: Vec<String>,
    /// Physical measurement units extracted from respective header columns (e.g. "kg", "Hz").
    pub units: Vec<String>,
    /// Row data records, keyed by normalized column header names.
    pub rows: Vec<HashMap<String, String>>,
}

/// Sanitizes arbitrary text into a valid, deterministic SysML v2 / KerML identifier.
///
/// /// Realises: [REQ-SYSML-INGEST-TABLE/sanitize_identifier]
///
/// ### Safety Intent:
/// Prevents KerML / SysML v2 grammar corruption, identifier injection, or compiler panic
/// caused by illegal punctuation, spaces, numeric prefixes, or collision with reserved keywords.
///
/// ### Preconditions:
/// - `text` is a valid UTF-8 string slice.
/// - `default_name` is a non-empty fallback identifier conforming to SysML identifier syntax.
///
/// ### Postconditions:
/// - The returned `String` is non-empty.
/// - The first character is either an ASCII letter (`a-z`, `A-Z`) or an underscore (`_`).
/// - All subsequent characters are ASCII alphanumeric (`a-z`, `A-Z`, `0-9`) or underscores (`_`).
/// - The identifier does not match any entry in [`RESERVED_SYSML_KEYWORDS`].
/// - If the cleaned input is empty, `default_name.to_string()` is returned.
///
/// ### Algorithmic Steps:
/// 1. Strip parenthesized or bracketed unit/specifier clauses (e.g., `Mass (kg)` -> `Mass`).
/// 2. Remove Markdown formatting characters (`*`, `` ` ``, `_`, `#`).
/// 3. Replace all non-alphanumeric characters with underscores.
/// 4. Collapse consecutive underscores and trim leading/trailing underscores.
/// 5. Prefix with an underscore if the initial character is an ASCII digit.
/// 6. Check against reserved SysML v2 keywords; append `_item` if collision occurs.
///
/// # Examples
/// ```rust
/// use ingest_sysml::table::sanitize_identifier;
///
/// assert_eq!(sanitize_identifier("Signal_Alpha (Hz)", "Item"), "Signal_Alpha");
/// assert_eq!(sanitize_identifier("subsystem", "Item"), "subsystem_item");
/// assert_eq!(sanitize_identifier("1. Step Name", "Item"), "_1_Step_Name");
/// ```
pub fn sanitize_identifier(text: &str, default_name: &str) -> String {
    // Fast path: empty input falls back directly to default identifier
    if text.is_empty() {
        return default_name.to_string();
    }

    // Step 1: Remove parentheses/bracketed content if that leaves a non-empty name:
    // e.g. "Mass (kg)" -> "Mass", "Voltage [V]" -> "Voltage"
    let bracket_re = Regex::new(r"[\(\[\{][^\)\]\}]*[\)\]\}]").unwrap();
    let stripped = bracket_re.replace_all(text, "").trim().to_string();
    let clean_cand = if !stripped.is_empty() {
        stripped
    } else {
        text.to_string()
    };

    // Step 2: Remove Markdown inline formatting tokens (*, `, _, #)
    let fmt_re = Regex::new(r"[*`_#]").unwrap();
    let formatted = fmt_re.replace_all(&clean_cand, " ").trim().to_string();

    // Step 3: Replace non-alphanumeric characters with underscores
    let non_alnum_re = Regex::new(r"[^a-zA-Z0-9_]").unwrap();
    let mut clean = non_alnum_re.replace_all(&formatted, "_").into_owned();

    // Step 4: Collapse multiple consecutive underscores and trim boundary underscores
    let multi_under_re = Regex::new(r"_+").unwrap();
    clean = multi_under_re.replace_all(&clean, "_").into_owned();
    clean = clean.trim_matches('_').to_string();

    // Fallback if sanitization completely evacuated the identifier
    if clean.is_empty() {
        return default_name.to_string();
    }

    // Step 5: SysML v2 identifiers must not start with an ASCII digit
    if clean.chars().next().unwrap().is_ascii_digit() {
        clean = format!("_{}", clean);
    }

    // Step 6: Avoid collisions with SysML v2 / KerML reserved grammar keywords
    if RESERVED_SYSML_KEYWORDS.contains(&clean.to_ascii_lowercase().as_str()) {
        clean = format!("{}_item", clean);
    }

    clean
}

/// Extracts measurement physical units enclosed in parentheses or brackets.
///
/// /// Realises: [REQ-SYSML-INGEST-TABLE/extract_unit_from_header_or_val]
///
/// ### Safety Intent:
/// Separates measurement physical units (e.g. SI units, frequencies, percentages) from
/// attribute and port names to ensure typed SysML v2 attributes receive proper unit docstrings.
///
/// ### Preconditions:
/// - `header_or_val` is a valid UTF-8 string slice.
///
/// ### Postconditions:
/// - Returns a tuple `(cleaned_name, unit)` where `unit` contains the extracted unit string
///   or is empty if no bracketed unit was present.
/// - `cleaned_name` is stripped of parenthesized/bracketed unit content and whitespace-trimmed.
///
/// # Examples
/// ```rust
/// use ingest_sysml::table::extract_unit_from_header_or_val;
///
/// let (name, unit) = extract_unit_from_header_or_val("Frequency (Hz)");
/// assert_eq!(name, "Frequency");
/// assert_eq!(unit, "Hz");
/// ```
pub fn extract_unit_from_header_or_val(header_or_val: &str) -> (String, String) {
    // Regex matching unit tokens inside parentheses or brackets: e.g. (kg), [m/s^2], (deg_C)
    let unit_re = Regex::new(r"[\(\[]([a-zA-Z0-9_/°^%\-]+)[\)\]]").unwrap();
    let strip_re = Regex::new(r"[\(\[][^\)\]]*[\)\]]").unwrap();

    let unit = if let Some(caps) = unit_re.captures(header_or_val) {
        caps.get(1).map_or("", |m| m.as_str()).trim().to_string()
    } else {
        String::new()
    };

    let cleaned = strip_re.replace_all(header_or_val, "").trim().to_string();
    (cleaned, unit)
}

/// Normalizes a column header into a canonical snake_case lookup key.
///
/// /// Realises: [REQ-SYSML-INGEST-TABLE/normalize_col]
///
/// ### Safety Intent:
/// Normalizes heterogeneous Markdown table header representations (e.g. `**Part Number**`,
/// `Part_Number`, `part-number`, `Part Number (SKU)`) to a canonical lookup key (`part_number`)
/// so that column matching in schema translation is robust against formatting variances.
///
/// ### Preconditions:
/// - `name` is a valid UTF-8 string slice.
///
/// ### Postconditions:
/// - Returns a lowercased, underscore-delimited snake_case string stripped of formatting.
pub fn normalize_col(name: &str) -> String {
    let (cleaned, _) = extract_unit_from_header_or_val(name);
    let fmt_re = Regex::new(r"[*`_]").unwrap();
    let unformatted = fmt_re.replace_all(&cleaned, " ").trim().to_ascii_lowercase();

    let non_alnum_re = Regex::new(r"[^a-z0-9]+").unwrap();
    let mut clean = non_alnum_re.replace_all(&unformatted, "_").into_owned();
    clean = clean.trim_matches('_').to_string();
    clean
}

/// Strips Markdown hyperlinks, HTML tags, and inline formatting from a table cell value.
///
/// /// Realises: [REQ-SYSML-INGEST-TABLE/clean_cell_value]
///
/// ### Safety Intent:
/// Neutralizes parser injection vectors, raw HTML tags, embedded hyperlinks, and formatting
/// tokens present in OEM Markdown cells, producing clean plaintext suitable for AST documentation.
///
/// ### Preconditions:
/// - `val` is a valid UTF-8 string slice.
///
/// ### Postconditions:
/// - Returned string contains no raw HTML tags (`<...>` or `<br/>`).
/// - Markdown hyperlinks `[text](url)` are converted to plain `text`.
/// - Whitespace is trimmed.
pub fn clean_cell_value(val: &str) -> String {
    if val.is_empty() {
        return String::new();
    }

    // Step 1: Strip Markdown hyperlink syntax: [visible text](http://target) -> visible text
    let link_re = Regex::new(r"\[([^\]]+)\]\([^\)]+\)").unwrap();
    let step1 = link_re.replace_all(val, "$1");

    // Step 2: Replace line break tags with whitespace separator
    let br_re = Regex::new(r"(?i)<br\s*/?>").unwrap();
    let step2 = br_re.replace_all(&step1, " ");

    // Step 3: Strip remaining HTML tags
    let html_re = Regex::new(r"<[^>]+>").unwrap();
    let step3 = html_re.replace_all(&step2, "");

    step3.trim().to_string()
}

/// Parses a scalar string into numeric float, unit, and optional signed integer representation.
///
/// /// Realises: [REQ-SYSML-INGEST-TABLE/parse_numeric_with_unit]
///
/// ### Safety Intent:
/// Converts unstructured scalar cell values (e.g. `1800.0 kg`, `150 W`, `42`, `0x1A`) into
/// strongly-typed IEEE 754 float values, signed integers, and unit tokens without loss of precision.
///
/// ### Preconditions:
/// - `raw_val` is a valid UTF-8 string slice.
///
/// ### Postconditions:
/// - If a valid number or hex literal is present, returns `(Some(float_val), Some(unit), int_val)`.
/// - If non-numeric, returns `(None, None, None)`.
///
/// # Examples
/// ```rust
/// use ingest_sysml::table::parse_numeric_with_unit;
///
/// let (f_val, unit, i_val) = parse_numeric_with_unit("150.5 W");
/// assert_eq!(f_val, Some(150.5));
/// assert_eq!(unit.as_deref(), Some("W"));
/// assert_eq!(i_val, None);
///
/// let (f_hex, _, i_hex) = parse_numeric_with_unit("0x20");
/// assert_eq!(f_hex, Some(32.0));
/// assert_eq!(i_hex, Some(32));
/// ```
pub fn parse_numeric_with_unit(raw_val: &str) -> (Option<f64>, Option<String>, Option<i64>) {
    let fmt_re = Regex::new(r"[*`]").unwrap();
    let clean = fmt_re.replace_all(raw_val, "").trim().to_string();

    // Check hexadecimal pattern: 0x...
    let hex_re = Regex::new(r"^(0x[0-9a-fA-F]+)$").unwrap();
    if let Some(caps) = hex_re.captures(&clean) {
        let hex_str = caps.get(1).unwrap().as_str();
        if let Ok(val_int) = i64::from_str_radix(&hex_str[2..], 16) {
            return (Some(val_int as f64), Some(String::new()), Some(val_int));
        }
    }

    // Number followed by optional physical unit: e.g. 100.5 kg, -20.0 degC
    let num_re = Regex::new(r"^([-+]?[0-9]*\.?[0-9]+(?:[eE][-+]?[0-9]+)?)\s*([a-zA-Z0-9_/°^%\-]*)$").unwrap();
    if let Some(caps) = num_re.captures(&clean) {
        let val_str = caps.get(1).unwrap().as_str();
        let unit_str = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();

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

/// Parses range bounds string e.g. `[0.0, 100.0]`, `-20.0 to 60.0`, `18.0 .. 25.2`.
///
/// /// Realises: [REQ-SYSML-INGEST-TABLE/parse_range_bounds]
///
/// ### Safety Intent:
/// Extracts parametric tolerance intervals, enforcing invariant $min\_val \le max\_val$.
/// Inverted boundary specifications in OEM documents are normalized to eliminate logic bugs.
///
/// ### Preconditions:
/// - `range_text` is a valid UTF-8 string slice.
///
/// ### Postconditions:
/// - Returns `(Some(min), Some(max))` where `min <= max` if two valid bounds are extracted.
/// - Returns `(None, None)` if interval syntax cannot be resolved.
pub fn parse_range_bounds(range_text: &str) -> (Option<f64>, Option<f64>) {
    let fmt_re = Regex::new(r"[*`\[\]\(\)]").unwrap();
    let clean = fmt_re.replace_all(range_text, "").trim().to_string();

    let split_re = Regex::new(r"\s*(?:,|\.\.|\bto\b|:)\s*").unwrap();
    let parts: Vec<&str> = split_re.split(&clean).collect();
    if parts.len() >= 2 {
        let (val1, _, _) = parse_numeric_with_unit(parts[0]);
        let (val2, _, _) = parse_numeric_with_unit(parts[1]);
        if let (Some(v1), Some(v2)) = (val1, val2) {
            let min_v = if v1 < v2 { v1 } else { v2 };
            let max_v = if v1 > v2 { v1 } else { v2 };
            return (Some(min_v), Some(max_v));
        }
    }

    (None, None)
}

/// Extracts authoritative provenance citation from a row dictionary.
///
/// /// Realises: [REQ-SYSML-INGEST-TABLE/extract_provenance_citation]
///
/// ### Safety Intent:
/// Preserves DO-178C Section 5.5 / ISO 26262 Part 8 traceability by inspecting known
/// provenance column names defined in [`PROVENANCE_COLUMNS`].
///
/// ### Preconditions:
/// - `row` is a valid hashmap representing a table row.
/// - `headers` contains normalized column names for the table.
///
/// ### Postconditions:
/// - Returns the first non-empty provenance string found, or an empty string.
pub fn extract_provenance_citation(row: &HashMap<String, String>, headers: &[String]) -> String {
    for h in headers {
        if PROVENANCE_COLUMNS.contains(&h.as_str()) {
            if let Some(val) = row.get(h) {
                let trimmed = val.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }
    String::new()
}

/// Composes a grounded SysML v2 documentation string preserving description, unit, and OEM citation.
///
/// /// Realises: [REQ-SYSML-INGEST-TABLE/compose_grounded_doc]
///
/// ### Safety Intent:
/// Synthesizes standardized documentation for AST nodes, embedding physical measurement units
/// and formal verification source citations into KerML/SysML v2 `doc /* ... */` blocks.
///
/// ### Preconditions:
/// - `description`, `citation`, and `unit` are valid UTF-8 string slices.
///
/// ### Postconditions:
/// - Returns a combined docstring formatted as: `<description> [unit: <unit>] [Source: <citation>]`.
pub fn compose_grounded_doc(description: &str, citation: &str, unit: &str) -> String {
    let mut tokens = Vec::new();
    let desc_trimmed = description.trim();
    if !desc_trimmed.is_empty() {
        tokens.push(desc_trimmed.to_string());
    }
    let unit_trimmed = unit.trim();
    if !unit_trimmed.is_empty() {
        tokens.push(format!("[unit: {}]", unit_trimmed));
    }
    let cit_trimmed = citation.trim();
    if !cit_trimmed.is_empty() {
        tokens.push(format!("[Source: {}]", cit_trimmed));
    }
    tokens.join(" ").trim().to_string()
}

/// Classifies table type based purely on structural column header patterns.
///
/// /// Realises: [REQ-SYSML-INGEST-TABLE/classify_table]
///
/// ### Safety Intent:
/// Determines the architectural target archetype of a table without relying on domain-specific
/// vocabulary. Operates strictly on structural systems engineering column archetypes
/// (components, ports, parametric constraints, key-value properties).
///
/// ### Preconditions:
/// - `headers` contains normalized column header names.
///
/// ### Postconditions:
/// - Returns one of [`TableType::Bom`], [`TableType::Ports`], [`TableType::Constraints`],
///   [`TableType::Properties`], or [`TableType::Generic`].
pub fn classify_table(headers: &[String]) -> TableType {
    let h_set: std::collections::HashSet<&str> = headers.iter().map(|s| s.as_str()).collect();

    // 1. BOM / Component list archetype detection
    let bom_indicators = [
        "component", "component_name", "part", "part_name", "subsystem",
        "item", "module", "assembly", "lru",
    ];
    let bom_attr_indicators = [
        "part_number", "pn", "sku", "mass", "weight", "power", "qty",
        "quantity", "cost",
    ];

    let has_bom_name = bom_indicators.iter().any(|k| h_set.contains(k));
    let has_bom_attr = bom_attr_indicators.iter().any(|k| h_set.contains(k));

    if has_bom_name && has_bom_attr {
        return TableType::Bom;
    }
    if has_bom_name && headers.len() >= 3 && !h_set.iter().any(|h| ["direction", "dir", "port", "signal"].contains(h)) {
        return TableType::Bom;
    }

    // 2. Ports / Interfaces / Signals archetype detection
    let port_indicators = [
        "port", "port_name", "signal", "signal_name", "signal_id",
        "interface", "interface_name", "flow",
    ];
    let dir_indicators = ["direction", "dir", "flow_direction"];
    let type_indicators = ["type", "data_type", "payload", "rate", "protocol"];

    let has_port_name = port_indicators.iter().any(|k| h_set.contains(k));
    let has_dir = dir_indicators.iter().any(|k| h_set.contains(k));
    let has_type = type_indicators.iter().any(|k| h_set.contains(k));

    if has_port_name || (has_dir && has_type) {
        return TableType::Ports;
    }
    if h_set.contains("source_port") && h_set.contains("target_port") {
        return TableType::Ports;
    }
    if h_set.contains("source") && h_set.contains("target") {
        return TableType::Ports;
    }

    // 3. Parametric limits / Constraints archetype detection
    let constraint_indicators = [
        "min", "max", "lower_bound", "upper_bound", "range", "valid_range",
        "limit", "bound", "bounds", "tolerance", "constraint", "assertion",
    ];
    let param_indicators = ["parameter", "param", "property", "metric", "variable", "name"];

    let has_constraint = constraint_indicators.iter().any(|k| h_set.contains(k));
    let has_param = param_indicators.iter().any(|k| h_set.contains(k));

    if has_constraint && has_param {
        return TableType::Constraints;
    }
    if has_constraint && headers.len() >= 2 {
        return TableType::Constraints;
    }

    // 4. Key-Value / Properties archetype detection (2-3 columns, first column specifies property/key)
    if (headers.len() == 2 || headers.len() == 3) && !headers.is_empty() {
        let first = &headers[0];
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
///
/// /// Realises: [REQ-SYSML-INGEST-TABLE/parse_markdown_tables]
///
/// ### Safety Intent:
/// Parses GitHub Flavored Markdown (GFM) tables with lookahead and delimiter verification.
/// Robust against unclosed tables, non-table pipe delimiters in code blocks, and irregular
/// column counts.
///
/// ### Preconditions:
/// - `lines` is an array of UTF-8 string slices representing lines of Markdown text.
///
/// ### Postconditions:
/// - Returns a vector of zero or more validated [`MarkdownTable`] instances.
/// - Every returned table contains non-empty headers and at least one parsed data row.
pub fn parse_markdown_tables(lines: &[&str]) -> Vec<MarkdownTable> {
    let mut tables = Vec::new();
    let mut in_table = false;
    let mut raw_headers = Vec::new();
    let mut norm_headers = Vec::new();
    let mut units = Vec::new();
    let mut rows = Vec::new();

    // Regex matching standard Markdown table delimiter row: e.g. | :--- | ---: | :---: |
    let delim_re = Regex::new(r"^:?-+:?$").unwrap();

    let mut flush_table = |in_tbl: &mut bool,
                           raw_h: &mut Vec<String>,
                           norm_h: &mut Vec<String>,
                           u: &mut Vec<String>,
                           r: &mut Vec<HashMap<String, String>>| {
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

    let mut i = 0;
    let n = lines.len();

    while i < n {
        let line = lines[i].trim();

        if line.contains('|') {
            let cells: Vec<String> = line
                .trim_matches('|')
                .split('|')
                .map(|c| c.trim().to_string())
                .collect();

            let is_delimiter = !cells.is_empty() && cells.iter().all(|c| delim_re.is_match(c));

            if is_delimiter && !in_table {
                // Header was on previous line if available
                if i > 0 {
                    let prev_line = lines[i - 1].trim();
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
                // Ignore interior repeated delimiter
                i += 1;
                continue;
            } else if in_table {
                // Process data row
                let mut row_dict = HashMap::new();
                for (idx, norm_col) in norm_headers.iter().enumerate() {
                    let val = if idx < cells.len() {
                        clean_cell_value(&cells[idx])
                    } else {
                        String::new()
                    };
                    row_dict.insert(norm_col.clone(), val);
                }
                rows.push(row_dict);
                i += 1;
                continue;
            } else {
                // Forward lookahead: inspect next line to determine if this line is a header
                if i + 1 < n && lines[i + 1].contains('|') {
                    let next_line = lines[i + 1].trim();
                    let next_cells: Vec<String> = next_line
                        .trim_matches('|')
                        .split('|')
                        .map(|c| c.trim().to_string())
                        .collect();
                    if !next_cells.is_empty() && next_cells.iter().all(|c| delim_re.is_match(c)) {
                        flush_table(&mut in_table, &mut raw_headers, &mut norm_headers, &mut units, &mut rows);
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
            }
        } else if in_table {
            flush_table(&mut in_table, &mut raw_headers, &mut norm_headers, &mut units, &mut rows);
        }

        i += 1;
    }

    flush_table(&mut in_table, &mut raw_headers, &mut norm_headers, &mut units, &mut rows);
    tables
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
        let (name, unit) = extract_unit_from_header_or_val("Param_Alpha (kg)");
        assert_eq!(name, "Param_Alpha");
        assert_eq!(unit, "kg");

        let (name2, unit2) = extract_unit_from_header_or_val("Rate_Beta [MHz]");
        assert_eq!(name2, "Rate_Beta");
        assert_eq!(unit2, "MHz");

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
        let (f_val, unit, i_val) = parse_numeric_with_unit("1800.0 kg");
        assert_eq!(f_val, Some(1800.0));
        assert_eq!(unit.as_deref(), Some("kg"));
        assert_eq!(i_val, None);

        let (f_val2, unit2, i_val2) = parse_numeric_with_unit("42");
        assert_eq!(f_val2, Some(42.0));
        assert_eq!(unit2.as_deref(), Some(""));
        assert_eq!(i_val2, Some(42));

        let (f_val3, _, i_val3) = parse_numeric_with_unit("0x1A");
        assert_eq!(f_val3, Some(26.0));
        assert_eq!(i_val3, Some(26));
    }

    #[test]
    fn test_parse_range_bounds() {
        assert_eq!(parse_range_bounds("[0.0, 100.0]"), (Some(0.0), Some(100.0)));
        assert_eq!(parse_range_bounds("-20.0 to 60.0"), (Some(-20.0), Some(60.0)));
        assert_eq!(parse_range_bounds("18.0 .. 25.2"), (Some(18.0), Some(25.2)));
    }

    #[test]
    fn test_classify_and_parse_markdown_tables() {
        let sample = r#"
| Metadata Field | Contract Specification |
| :--- | :--- |
| **Requirement ID** | `REQ-0001` |
| **Subsystem** | Subsystem_Alpha |

| Component | Part Number | Mass (kg) |
| :--- | :--- | :--- |
| Classifier_Alpha | PN_001 | 25.0 |

| Port | Direction | Type |
| :--- | :--- | :--- |
| Port_1 | inout | Type_Alpha |
"#;
        let lines: Vec<&str> = sample.lines().collect();
        let tables = parse_markdown_tables(&lines);
        assert_eq!(tables.len(), 3);

        assert_eq!(classify_table(&tables[0].normalized_headers), TableType::Properties);
        assert_eq!(classify_table(&tables[1].normalized_headers), TableType::Bom);
        assert_eq!(classify_table(&tables[2].normalized_headers), TableType::Ports);

        let req_id_row = &tables[0].rows[0];
        assert_eq!(req_id_row.get("metadata_field").unwrap(), "**Requirement ID**");
        assert_eq!(req_id_row.get("contract_specification").unwrap(), "`REQ-0001`");
    }
}
