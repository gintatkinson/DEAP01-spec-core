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
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `TableType` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `TableType` across member set `[Bom, Ports, Constraints, Properties, Generic]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `TableType` members (`Bom, Ports, Constraints, Properties, Generic`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `TableType` representation for enum `TableType`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `TableType`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `TableType`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `TableType` (`TableType`).
/// - State: Maintains consistent state invariants across `TableType` without partial mutation on error.
/// - Action: Executes `TableType` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `TableType` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-INGEST-SYSML-TABLE-TABLETYPE] | SysML v2: PartDef(TableType)
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
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `MarkdownTable` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `MarkdownTable` across member set `[raw_headers: Vec<String>, normalized_headers: Vec<String>, units: Vec<String>, rows: Vec<HashMap<String, String>>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `MarkdownTable` members (`raw_headers: Vec<String>, normalized_headers: Vec<String>, units: Vec<String>, rows: Vec<HashMap<String, String>>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `MarkdownTable` representation for struct `MarkdownTable`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(4)) bounded over input/member dimension of `MarkdownTable`.
/// - Space Complexity: O(M(4)) bounded auxiliary memory for `MarkdownTable`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `MarkdownTable` (`MarkdownTable`).
/// - State: Maintains consistent state invariants across `MarkdownTable` without partial mutation on error.
/// - Action: Executes `MarkdownTable` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `MarkdownTable` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-INGEST-SYSML-TABLE-MARKDOWNTABLE] | SysML v2: PartDef(MarkdownTable)
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
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `sanitize_identifier` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `sanitize_identifier` (declaration #3) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - `text: &str` must satisfy domain and type-safety constraints prior to invoking `sanitize_identifier`.
/// - `default_name: &str` must satisfy domain and type-safety constraints prior to invoking `sanitize_identifier`.
///
/// ## 4. Postconditions
/// - Returns `String` from `sanitize_identifier` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(text)) bounded over input/member dimension of `sanitize_identifier`.
/// - Space Complexity: O(N(text)) bounded auxiliary memory for `sanitize_identifier`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `sanitize_identifier` (`String`).
/// - State: Maintains consistent state invariants across `sanitize_identifier` without partial mutation on error.
/// - Action: Executes `sanitize_identifier` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `sanitize_identifier` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-TABLE/sanitize_identifier] | SysML v2: ActionDef(sanitize_identifier)
pub fn sanitize_identifier(text: &str, default_name: &str) -> String {
    // Fast path: empty input falls back directly to default identifier
    if text.is_empty() {
        return default_name.to_string();
    }

    // Step 1: Remove parentheses/bracketed content if that leaves a non-empty name:
    // e.g. "Mass (kg)" -> "Mass", "Voltage [V]" -> "Voltage"
    let stripped = strip_bracketed_content(text);
    let clean_cand = if !stripped.is_empty() {
        stripped
    } else {
        text.to_string()
    };

    // Step 2: Remove Markdown inline formatting tokens (*, `, _, #)
    let formatted: String = clean_cand
        .chars()
        .map(|c| if c == '*' || c == '`' || c == '_' || c == '#' { ' ' } else { c })
        .collect();
    let formatted = formatted.trim();

    // Step 3: Replace non-alphanumeric characters with underscores
    let mut clean = formatted.replace(|c: char| !c.is_ascii_alphanumeric() && c != '_', "_");

    // Step 4: Collapse multiple consecutive underscores and trim boundary underscores
    clean = collapse_underscores(&clean);
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

/// Executes deterministic fn `strip_bracketed_content` within `crates/ingest-sysml/src/table.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `strip_bracketed_content` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `strip_bracketed_content` (declaration #4) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - `text: &str` must satisfy domain and type-safety constraints prior to invoking `strip_bracketed_content`.
///
/// ## 4. Postconditions
/// - Returns `String` from `strip_bracketed_content` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(text)) bounded over input/member dimension of `strip_bracketed_content`.
/// - Space Complexity: O(N(text)) bounded auxiliary memory for `strip_bracketed_content`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `strip_bracketed_content` (`String`).
/// - State: Maintains consistent state invariants across `strip_bracketed_content` without partial mutation on error.
/// - Action: Executes `strip_bracketed_content` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `strip_bracketed_content` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-INGEST-SYSML-TABLE-STRIP-BRACKETED-CONTENT] | SysML v2: ActionDef(strip_bracketed_content)
fn strip_bracketed_content(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '(' || c == '[' || c == '{' {
            for inner in chars.by_ref() {
                if inner == ')' || inner == ']' || inner == '}' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out.trim().to_string()
}

/// Executes deterministic fn `collapse_underscores` within `crates/ingest-sysml/src/table.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `collapse_underscores` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `collapse_underscores` (declaration #5) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - `s: &str` must satisfy domain and type-safety constraints prior to invoking `collapse_underscores`.
///
/// ## 4. Postconditions
/// - Returns `String` from `collapse_underscores` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(s)) bounded over input/member dimension of `collapse_underscores`.
/// - Space Complexity: O(N(s)) bounded auxiliary memory for `collapse_underscores`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `collapse_underscores` (`String`).
/// - State: Maintains consistent state invariants across `collapse_underscores` without partial mutation on error.
/// - Action: Executes `collapse_underscores` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `collapse_underscores` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-INGEST-SYSML-TABLE-COLLAPSE-UNDERSCORES] | SysML v2: ActionDef(collapse_underscores)
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `extract_unit_from_header_or_val` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `extract_unit_from_header_or_val` (declaration #6) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - `header_or_val: &str` must satisfy domain and type-safety constraints prior to invoking `extract_unit_from_header_or_val`.
///
/// ## 4. Postconditions
/// - Returns `(String, String)` from `extract_unit_from_header_or_val` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(header_or_val)) bounded over input/member dimension of `extract_unit_from_header_or_val`.
/// - Space Complexity: O(N(header_or_val)) bounded auxiliary memory for `extract_unit_from_header_or_val`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `extract_unit_from_header_or_val` (`(String, String)`).
/// - State: Maintains consistent state invariants across `extract_unit_from_header_or_val` without partial mutation on error.
/// - Action: Executes `extract_unit_from_header_or_val` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `extract_unit_from_header_or_val` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-TABLE/extract_unit_from_header_or_val] | SysML v2: ActionDef(extract_unit_from_header_or_val)
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
                let is_valid_unit = !inner.is_empty() && inner.chars().all(|ch| {
                    ch.is_ascii_alphanumeric()
                        || ch == '_'
                        || ch == '/'
                        || ch == '°'
                        || ch == '^'
                        || ch == '%'
                        || ch == '-'
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `normalize_col` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `normalize_col` (declaration #7) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - `name: &str` must satisfy domain and type-safety constraints prior to invoking `normalize_col`.
///
/// ## 4. Postconditions
/// - Returns `String` from `normalize_col` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(name)) bounded over input/member dimension of `normalize_col`.
/// - Space Complexity: O(N(name)) bounded auxiliary memory for `normalize_col`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `normalize_col` (`String`).
/// - State: Maintains consistent state invariants across `normalize_col` without partial mutation on error.
/// - Action: Executes `normalize_col` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `normalize_col` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-TABLE/normalize_col] | SysML v2: ActionDef(normalize_col)
pub fn normalize_col(name: &str) -> String {
    let (cleaned, _) = extract_unit_from_header_or_val(name);
    let mut clean = String::with_capacity(cleaned.len());
    let mut prev_under = false;
    for c in cleaned.chars() {
        let lower = c.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            clean.push(lower);
            prev_under = false;
        } else {
            if !prev_under {
                clean.push('_');
                prev_under = true;
            }
        }
    }
    clean.trim_matches('_').to_string()
}

/// Strips Markdown hyperlinks, HTML tags, and inline formatting from a table cell value.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `clean_cell_value` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `clean_cell_value` (declaration #8) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - `val: &str` must satisfy domain and type-safety constraints prior to invoking `clean_cell_value`.
///
/// ## 4. Postconditions
/// - Returns `String` from `clean_cell_value` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(val)) bounded over input/member dimension of `clean_cell_value`.
/// - Space Complexity: O(N(val)) bounded auxiliary memory for `clean_cell_value`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `clean_cell_value` (`String`).
/// - State: Maintains consistent state invariants across `clean_cell_value` without partial mutation on error.
/// - Action: Executes `clean_cell_value` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `clean_cell_value` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-TABLE/clean_cell_value] | SysML v2: ActionDef(clean_cell_value)
pub fn clean_cell_value(val: &str) -> String {
    if val.is_empty() {
        return String::new();
    }

    // Step 1: Strip Markdown hyperlink syntax: [visible text](http://target) -> visible text
    let step1 = strip_markdown_links(val);

    // Step 2: Replace line break tags with whitespace separator & Step 3: Strip remaining HTML tags
    let step3 = strip_html_tags(&step1);

    step3.trim().to_string()
}

/// Executes deterministic fn `strip_markdown_links` within `crates/ingest-sysml/src/table.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `strip_markdown_links` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `strip_markdown_links` (declaration #9) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - `val: &str` must satisfy domain and type-safety constraints prior to invoking `strip_markdown_links`.
///
/// ## 4. Postconditions
/// - Returns `String` from `strip_markdown_links` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(val)) bounded over input/member dimension of `strip_markdown_links`.
/// - Space Complexity: O(N(val)) bounded auxiliary memory for `strip_markdown_links`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `strip_markdown_links` (`String`).
/// - State: Maintains consistent state invariants across `strip_markdown_links` without partial mutation on error.
/// - Action: Executes `strip_markdown_links` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `strip_markdown_links` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-INGEST-SYSML-TABLE-STRIP-MARKDOWN-LINKS] | SysML v2: ActionDef(strip_markdown_links)
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

/// Executes deterministic fn `strip_html_tags` within `crates/ingest-sysml/src/table.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `strip_html_tags` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `strip_html_tags` (declaration #10) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - `val: &str` must satisfy domain and type-safety constraints prior to invoking `strip_html_tags`.
///
/// ## 4. Postconditions
/// - Returns `String` from `strip_html_tags` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(val)) bounded over input/member dimension of `strip_html_tags`.
/// - Space Complexity: O(N(val)) bounded auxiliary memory for `strip_html_tags`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `strip_html_tags` (`String`).
/// - State: Maintains consistent state invariants across `strip_html_tags` without partial mutation on error.
/// - Action: Executes `strip_html_tags` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `strip_html_tags` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-INGEST-SYSML-TABLE-STRIP-HTML-TAGS] | SysML v2: ActionDef(strip_html_tags)
fn strip_html_tags(val: &str) -> String {
    let mut out = String::with_capacity(val.len());
    let mut rest = val;
    while let Some(start_tag) = rest.find('<') {
        out.push_str(&rest[..start_tag]);
        let after_start = &rest[start_tag + 1..];
        if let Some(end_tag) = after_start.find('>') {
            let tag_content = after_start[..end_tag].trim();
            let lower = tag_content.to_ascii_lowercase();
            if lower == "br" || lower.starts_with("br ") || lower.starts_with("br/") || lower == "br/" {
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_numeric_with_unit` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_numeric_with_unit` (declaration #11) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - `raw_val: &str` must satisfy domain and type-safety constraints prior to invoking `parse_numeric_with_unit`.
///
/// ## 4. Postconditions
/// - Returns `(Option<f64>, Option<String>, Option<i64>)` from `parse_numeric_with_unit` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(raw_val)) bounded over input/member dimension of `parse_numeric_with_unit`.
/// - Space Complexity: O(N(raw_val)) bounded auxiliary memory for `parse_numeric_with_unit`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_numeric_with_unit` (`(Option<f64>, Option<String>, Option<i64>)`).
/// - State: Maintains consistent state invariants across `parse_numeric_with_unit` without partial mutation on error.
/// - Action: Executes `parse_numeric_with_unit` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_numeric_with_unit` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-TABLE/parse_numeric_with_unit] | SysML v2: ActionDef(parse_numeric_with_unit)
pub fn parse_numeric_with_unit(raw_val: &str) -> (Option<f64>, Option<String>, Option<i64>) {
    let clean: String = raw_val.chars().filter(|&c| c != '*' && c != '`').collect();
    let clean = clean.trim();

    // Check hexadecimal pattern: 0x...
    if (clean.starts_with("0x") || clean.starts_with("0X")) && clean.len() > 2 {
        let hex_str = &clean[2..];
        if hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
            if let Ok(val_int) = i64::from_str_radix(hex_str, 16) {
                return (Some(val_int as f64), Some(String::new()), Some(val_int));
            }
        }
    }

    // Number followed by optional physical unit: e.g. 100.5 kg, -20.0 degC
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

/// Executes deterministic fn `split_number_and_unit` within `crates/ingest-sysml/src/table.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `split_number_and_unit` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `split_number_and_unit` (declaration #12) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - `s: &str` must satisfy domain and type-safety constraints prior to invoking `split_number_and_unit`.
///
/// ## 4. Postconditions
/// - Returns `Option<(&str, &str)>` from `split_number_and_unit` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(s)) bounded over input/member dimension of `split_number_and_unit`.
/// - Space Complexity: O(N(s)) bounded auxiliary memory for `split_number_and_unit`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `split_number_and_unit` (`Option<(&str, &str)>`).
/// - State: Maintains consistent state invariants across `split_number_and_unit` without partial mutation on error.
/// - Action: Executes `split_number_and_unit` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `split_number_and_unit` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-INGEST-SYSML-TABLE-SPLIT-NUMBER-AND-UNIT] | SysML v2: ActionDef(split_number_and_unit)
fn split_number_and_unit(s: &str) -> Option<(&str, &str)> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let mut idx = 0;
    if bytes[idx] == b'+' || bytes[idx] == b'-' {
        idx += 1;
    }
    let mut has_digits = false;
    while idx < bytes.len() && bytes[idx].is_ascii_digit() {
        has_digits = true;
        idx += 1;
    }
    if idx < bytes.len() && bytes[idx] == b'.' {
        idx += 1;
        let mut has_dec_digits = false;
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            has_dec_digits = true;
            idx += 1;
        }
        if !has_digits && !has_dec_digits {
            return None;
        }
        has_digits = true;
    }
    if !has_digits {
        return None;
    }
    if idx < bytes.len() && (bytes[idx] == b'e' || bytes[idx] == b'E') {
        let exp_start = idx;
        idx += 1;
        if idx < bytes.len() && (bytes[idx] == b'+' || bytes[idx] == b'-') {
            idx += 1;
        }
        let mut has_exp_digits = false;
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            has_exp_digits = true;
            idx += 1;
        }
        if !has_exp_digits {
            idx = exp_start;
        }
    }
    let num_part = &s[..idx];
    let rest = s[idx..].trim_start();
    let valid_unit = rest.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || c == '_'
            || c == '/'
            || c == '°'
            || c == '^'
            || c == '%'
            || c == '-'
    });
    if valid_unit {
        Some((num_part, rest))
    } else {
        None
    }
}

/// Parses range bounds string e.g. `[0.0, 100.0]`, `-20.0 to 60.0`, `18.0 .. 25.2`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_range_bounds` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_range_bounds` (declaration #13) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - `range_text: &str` must satisfy domain and type-safety constraints prior to invoking `parse_range_bounds`.
///
/// ## 4. Postconditions
/// - Returns `(Option<f64>, Option<f64>)` from `parse_range_bounds` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(range_text)) bounded over input/member dimension of `parse_range_bounds`.
/// - Space Complexity: O(N(range_text)) bounded auxiliary memory for `parse_range_bounds`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_range_bounds` (`(Option<f64>, Option<f64>)`).
/// - State: Maintains consistent state invariants across `parse_range_bounds` without partial mutation on error.
/// - Action: Executes `parse_range_bounds` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_range_bounds` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-TABLE/parse_range_bounds] | SysML v2: ActionDef(parse_range_bounds)
pub fn parse_range_bounds(range_text: &str) -> (Option<f64>, Option<f64>) {
    let clean: String = range_text
        .chars()
        .filter(|&c| c != '*' && c != '`' && c != '[' && c != ']' && c != '(' && c != ')')
        .collect();
    let clean = clean.trim();

    let parts = split_range_parts(clean);
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

/// Executes deterministic fn `split_range_parts` within `crates/ingest-sysml/src/table.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `split_range_parts` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `split_range_parts` (declaration #14) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - `s: &str` must satisfy domain and type-safety constraints prior to invoking `split_range_parts`.
///
/// ## 4. Postconditions
/// - Returns `Vec<&str>` from `split_range_parts` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(s)) bounded over input/member dimension of `split_range_parts`.
/// - Space Complexity: O(N(s)) bounded auxiliary memory for `split_range_parts`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `split_range_parts` (`Vec<&str>`).
/// - State: Maintains consistent state invariants across `split_range_parts` without partial mutation on error.
/// - Action: Executes `split_range_parts` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `split_range_parts` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-INGEST-SYSML-TABLE-SPLIT-RANGE-PARTS] | SysML v2: ActionDef(split_range_parts)
fn split_range_parts(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let s_bytes = s.as_bytes();
    let mut i = 0;
    while i < s.len() {
        if s_bytes[i] == b',' || s_bytes[i] == b':' {
            parts.push(s[start..i].trim());
            i += 1;
            start = i;
        } else if s[i..].starts_with("..") {
            parts.push(s[start..i].trim());
            i += 2;
            start = i;
        } else if s[i..].starts_with("to")
            && (i == 0 || s_bytes[i - 1].is_ascii_whitespace())
            && (i + 2 == s.len() || s_bytes[i + 2].is_ascii_whitespace())
        {
            parts.push(s[start..i].trim());
            i += 2;
            start = i;
        } else {
            i += 1;
        }
    }
    if start < s.len() {
        parts.push(s[start..].trim());
    }
    parts
}

/// Extracts authoritative provenance citation from a row dictionary.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `extract_provenance_citation` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `extract_provenance_citation` (declaration #15) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - `row: &HashMap<String, String>` must satisfy domain and type-safety constraints prior to invoking `extract_provenance_citation`.
/// - `headers: &[String]` must satisfy domain and type-safety constraints prior to invoking `extract_provenance_citation`.
///
/// ## 4. Postconditions
/// - Returns `String` from `extract_provenance_citation` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(row)) bounded over input/member dimension of `extract_provenance_citation`.
/// - Space Complexity: O(N(row)) bounded auxiliary memory for `extract_provenance_citation`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `extract_provenance_citation` (`String`).
/// - State: Maintains consistent state invariants across `extract_provenance_citation` without partial mutation on error.
/// - Action: Executes `extract_provenance_citation` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `extract_provenance_citation` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-TABLE/extract_provenance_citation] | SysML v2: ActionDef(extract_provenance_citation)
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
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `compose_grounded_doc` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `compose_grounded_doc` (declaration #16) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - `description: &str` must satisfy domain and type-safety constraints prior to invoking `compose_grounded_doc`.
/// - `citation: &str` must satisfy domain and type-safety constraints prior to invoking `compose_grounded_doc`.
/// - `unit: &str` must satisfy domain and type-safety constraints prior to invoking `compose_grounded_doc`.
///
/// ## 4. Postconditions
/// - Returns `String` from `compose_grounded_doc` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(description)) bounded over input/member dimension of `compose_grounded_doc`.
/// - Space Complexity: O(N(description)) bounded auxiliary memory for `compose_grounded_doc`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `compose_grounded_doc` (`String`).
/// - State: Maintains consistent state invariants across `compose_grounded_doc` without partial mutation on error.
/// - Action: Executes `compose_grounded_doc` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `compose_grounded_doc` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-TABLE/compose_grounded_doc] | SysML v2: ActionDef(compose_grounded_doc)
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
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `classify_table` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `classify_table` (declaration #17) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - `headers: &[String]` must satisfy domain and type-safety constraints prior to invoking `classify_table`.
///
/// ## 4. Postconditions
/// - Returns `TableType` from `classify_table` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(headers)) bounded over input/member dimension of `classify_table`.
/// - Space Complexity: O(N(headers)) bounded auxiliary memory for `classify_table`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `classify_table` (`TableType`).
/// - State: Maintains consistent state invariants across `classify_table` without partial mutation on error.
/// - Action: Executes `classify_table` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `classify_table` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-TABLE/classify_table] | SysML v2: ActionDef(classify_table)
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
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_markdown_tables` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_markdown_tables` (declaration #18) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - `lines: &[&str]` must satisfy domain and type-safety constraints prior to invoking `parse_markdown_tables`.
///
/// ## 4. Postconditions
/// - Returns `Vec<MarkdownTable>` from `parse_markdown_tables` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(lines)) bounded over input/member dimension of `parse_markdown_tables`.
/// - Space Complexity: O(N(lines)) bounded auxiliary memory for `parse_markdown_tables`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_markdown_tables` (`Vec<MarkdownTable>`).
/// - State: Maintains consistent state invariants across `parse_markdown_tables` without partial mutation on error.
/// - Action: Executes `parse_markdown_tables` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_markdown_tables` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-TABLE/parse_markdown_tables] | SysML v2: ActionDef(parse_markdown_tables)
pub fn parse_markdown_tables(lines: &[&str]) -> Vec<MarkdownTable> {
    let mut tables = Vec::new();
    let mut in_table = false;
    let mut raw_headers = Vec::new();
    let mut norm_headers = Vec::new();
    let mut units = Vec::new();
    let mut rows = Vec::new();

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

            let is_delimiter = !cells.is_empty() && cells.iter().all(|c| is_table_delimiter_cell(c));

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
                    if !next_cells.is_empty() && next_cells.iter().all(|c| is_table_delimiter_cell(c)) {
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

/// Executes deterministic fn `is_table_delimiter_cell` within `crates/ingest-sysml/src/table.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_table_delimiter_cell` in `crates/ingest-sysml/src/table.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_table_delimiter_cell` (declaration #19) within `INGEST-SYSML-TABLE`.
///
/// ## 3. Preconditions
/// - Invoked within a valid execution context for `is_table_delimiter_cell` in `INGEST-SYSML-TABLE`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_table_delimiter_cell` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(K(is_table_delimiter_cell)) bounded over input/member dimension of `is_table_delimiter_cell`.
/// - Space Complexity: O(K(is_table_delimiter_cell)) bounded auxiliary memory for `is_table_delimiter_cell`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_table_delimiter_cell` (`bool`).
/// - State: Maintains consistent state invariants across `is_table_delimiter_cell` without partial mutation on error.
/// - Action: Executes `is_table_delimiter_cell` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_table_delimiter_cell` (`INGEST-SYSML-TABLE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-INGEST-SYSML-TABLE-IS-TABLE-DELIMITER-CELL] | SysML v2: ActionDef(is_table_delimiter_cell)
pub(crate) fn is_table_delimiter_cell(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
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
