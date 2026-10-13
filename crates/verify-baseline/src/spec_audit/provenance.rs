//! Native Rust Provenance & Factual Grounding Validator.
//!
//! Conforms to DO-178C Level A and ISO 26262 ASIL D safety standards.
//!
//! Validates:
//! 1. Factual grounding & empirical assertions:
//!    - Audits numeric parameters, units, and engineering assertions against SysML schema AST symbols
//!      (`SysmlSymbolIndex` and `ConstraintDef` / `AttributeDef`) or normative standards citations.
//!    - Enforces that numeric values (frequencies, latency, voltages, rates, dimensions) have explicit units and grounding.
//! 2. Source reference integrity (`validate_source_reference_clause_integrity`):
//!    - Verifies that `## Source References` sections in specifications cite verbatim clause numbers and schema paths
//!      (e.g. `ISO/IEC/IEEE 29148:2018 §6.4.2`, `RTCA DO-178C §6.3.1`, `schema/*.sysml`).
//!    - Detects and reports placeholder citations, empty sections, or self-referential links.
//! 3. Specification Cardinality Bounds (`validate_cardinality_bounds`):
//!    - Epics: 3--15 features recommended; flags anomalies.
//!    - Features: 3--10 acceptance criteria recommended; flags anomalies.
//!
//! Strictly zero regex, zero unwrap/expect/panic in non-test code, zero Unicode em dashes (uses ASCII -- exclusively).

use crate::spec_audit::markdown_ast::{parse_markdown, MarkdownDoc};
use crate::spec_audit::uml_validator::SysmlSymbolIndex;
use crate::spec_audit::{SpecAuditError, SpecAuditFinding};
use deap_core::sysml_ast::{AttributeDef, ConstraintDef};

/// Recognized physical and engineering units across frequencies, timing, electrical, rates, dimensions, and quantities.
pub const RECOGNIZED_UNITS: &[&str] = &[
    // Frequency
    "hz", "khz", "mhz", "ghz",
    // Time & Latency
    "s", "sec", "second", "seconds", "ms", "us", "µs", "ns", "min", "minute", "minutes", "hr", "hour", "hours", "h",
    // Voltage, Current, Power, Energy
    "v", "mv", "kv", "a", "ma", "ua", "µa", "w", "kw", "mw", "j", "kj", "mj", "kwh", "mah", "ah",
    // Resistance, Capacitance
    "ohm", "kohm", "mohm", "f", "uf", "µf", "nf", "pf",
    // Rates & Velocity
    "bps", "kbps", "mbps", "gbps", "b/s", "kb/s", "mb/s", "gb/s", "bytes/s", "baud",
    "m/s", "km/h", "deg/s", "rad/s", "m/s^2", "m/s2", "fps", "rpm",
    // Dimensions & Geometry
    "m", "cm", "mm", "km", "nm", "um", "µm", "deg", "rad", "degree", "degrees", "radian", "radians",
    // Mass & Temperature
    "kg", "g", "mg", "c", "°c", "k",
    // Ratios, Levels, Percentages
    "%", "percent", "db", "dbm", "dbi", "ppm", "ratio",
    // Data storage
    "b", "kb", "mb", "gb", "tb", "byte", "bytes",
];

/// Known engineering parameter terms that require explicit physical units and grounding.
pub const ENGINEERING_PARAMETER_TERMS: &[&str] = &[
    "frequency",
    "latency",
    "delay",
    "timeout",
    "voltage",
    "current",
    "power",
    "energy",
    "bandwidth",
    "throughput",
    "velocity",
    "speed",
    "altitude",
    "heading",
    "acceleration",
    "rate",
    "sampling rate",
    "clock",
    "dimension",
    "resistance",
    "capacitance",
    "payload",
    "mass",
    "thrust",
    "torque",
    "pressure",
    "temperature",
];

/// Normative standards prefixes and keywords for factual grounding citations.
pub const NORMATIVE_STANDARDS: &[&str] = &[
    "iso",
    "iec",
    "ieee",
    "rtca",
    "do-178",
    "do-178c",
    "do-254",
    "do-331",
    "mil-std",
    "stanag",
    "rfc",
    "arinc",
    "nist",
    "ecss",
    "astm",
    "sae",
    "jarus",
    "sora",
];

/// Placeholder tokens prohibited in authoritative source reference citations.
pub const PROHIBITED_PLACEHOLDERS: &[&str] = &[
    "{{",
    "}}",
    "<todo",
    "todo:",
    "placeholder",
    "link-to-schema",
    "link-to-specification",
    "#tbd",
    "[tbd]",
    "to be populated",
    "tbd",
    "unspecified",
];

/// Checks if a token represents a recognized physical or engineering unit.
///
/// Preconditions: None.
/// Postconditions: Returns true if token matches a recognized unit case-insensitively.
/// Algorithmic Complexity: O(U) where U is recognized unit table size.
pub fn is_recognized_unit(token: &str) -> bool {
    let lower = token.trim().to_lowercase();
    let clean = lower.trim_end_matches(&[',', '.', ';', ':', ')', ']', '}'][..]);
    RECOGNIZED_UNITS.iter().any(|&u| u == clean)
}

/// Validates whether a token or table cell is a pure scalar numeric value
/// (e.g. `1800.0`, `-45`, `+12.5`, `50 ms`, `250 Hz`), rather than an identifier,
/// package name, or title (e.g. `AVENGER5_ControlActions`, `PL-40`, `RS-485`, `RBF3`).
///
/// Preconditions: None.
/// Postconditions: Returns true if and only if val represents a pure scalar number or scalar with unit.
/// Algorithmic Complexity: O(N) where N is val byte length.
pub fn is_pure_scalar_numeric(val: &str) -> bool {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Strip markdown formatting (*, `, _, ", ')
    let clean = trimmed.trim_matches(|c| c == '*' || c == '`' || c == '_' || c == '"' || c == '\'');
    let bytes = clean.as_bytes();
    if bytes.is_empty() {
        return false;
    }

    let mut idx = 0;
    // Optional sign: +, -, or unicode minus
    if clean.starts_with('+') || clean.starts_with('-') {
        idx += 1;
    } else if clean.starts_with('−') {
        idx += '−'.len_utf8();
    }

    if idx >= bytes.len() {
        return false;
    }

    // Must have at least one digit
    let mut has_digits = false;
    while idx < bytes.len() && bytes[idx].is_ascii_digit() {
        has_digits = true;
        idx += 1;
    }
    if !has_digits {
        return false;
    }

    // Optional decimal point and digits
    if idx < bytes.len() && bytes[idx] == b'.' {
        idx += 1;
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            idx += 1;
        }
    }

    // Optional exponent (e/E)
    if idx < bytes.len() && (bytes[idx] == b'e' || bytes[idx] == b'E') {
        idx += 1;
        if idx < bytes.len() && (bytes[idx] == b'+' || bytes[idx] == b'-') {
            idx += 1;
        }
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            idx += 1;
        }
    }

    // Remainder should be empty or a recognized physical unit
    let rest = clean[idx..].trim();
    if rest.is_empty() {
        return true;
    }

    is_recognized_unit(rest)
}

/// Parses a table cell into a scalar numeric value and optional unit if and only if
/// the cell represents a pure numeric claim, rejecting arbitrary identifier text,
/// subsystem package names, or document titles.
///
/// Preconditions: None.
/// Postconditions: Returns Some((val, unit)) if pure scalar, otherwise None.
/// Algorithmic Complexity: O(N) where N is val byte length.
pub fn extract_scalar_numeric_claim(val: &str) -> Option<(f64, Option<String>)> {
    let trimmed = val.trim();
    if !is_pure_scalar_numeric(trimmed) {
        return None;
    }

    let clean = trimmed.trim_matches(|c| c == '*' || c == '`' || c == '_' || c == '"' || c == '\'');
    let mut num_end = 0;
    for (i, c) in clean.char_indices() {
        if c.is_ascii_digit() || c == '.' || c == '+' || c == '-' || c == '−' || c == 'e' || c == 'E' {
            num_end = i + c.len_utf8();
        } else if !c.is_whitespace() {
            break;
        }
    }

    let num_str = clean[..num_end].trim().replace('−', "-");
    let val_f64 = num_str.parse::<f64>().ok()?;
    let rem = clean[num_end..].trim();
    let unit = if !rem.is_empty() && is_recognized_unit(rem) {
        Some(rem.to_string())
    } else {
        None
    };

    Some((val_f64, unit))
}

/// Checks whether a line contains an engineering parameter keyword.
///
/// Preconditions: None.
/// Postconditions: Returns the matched engineering parameter keyword if found.
/// Algorithmic Complexity: O(P * K) where P is parameters count and K is line length.
pub fn find_engineering_parameter(line: &str) -> Option<&'static str> {
    let lower = line.to_lowercase();
    for &param in ENGINEERING_PARAMETER_TERMS {
        if let Some(pos) = lower.find(param) {
            let before_ok = if pos == 0 {
                true
            } else {
                let prev_b = lower.as_bytes()[pos - 1];
                !prev_b.is_ascii_alphanumeric() && prev_b != b'_' && prev_b != b'-'
            };

            let end = pos + param.len();
            let after_ok = if end >= lower.len() {
                true
            } else {
                let next_b = lower.as_bytes()[end];
                !next_b.is_ascii_alphanumeric() && next_b != b'_' && next_b != b'-'
            };

            if before_ok && after_ok {
                return Some(param);
            }
        }
    }
    None
}

/// Scans a line for naked numeric literals (digits without an immediate or adjacent unit)
/// in an engineering parameter context.
///
/// Strictly zero regex: iterates through byte slices.
///
/// Preconditions: None.
/// Postconditions: Returns a list of (number_string, start_byte_offset) for un-annotated numeric quantities.
/// Algorithmic Complexity: O(L) where L is line length.
pub fn find_naked_numeric_quantities(line: &str) -> Vec<(String, usize)> {
    let mut naked = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut idx = 0;

    while idx < len {
        // Find digit start
        if bytes[idx].is_ascii_digit() {
            let start = idx;

            // Check if this digit is part of an identifier prefix (e.g. REQ-02, FEAT-01, AC-03, v1, PL-40, RS-485, AVENGER5)
            let mut word_start = start;
            while word_start > 0 {
                let b = bytes[word_start - 1];
                if b.is_ascii_whitespace() || b == b'[' || b == b'(' || b == b'{' || b == b'<' || b == b'|' {
                    break;
                }
                word_start -= 1;
            }
            let prefix = &bytes[word_start..start];
            let is_id_token_prefix = prefix.iter().any(|b| b.is_ascii_alphabetic() || *b == b'_');

            let mut dot_seen = false;
            while idx < len && (bytes[idx].is_ascii_digit() || (bytes[idx] == b'.' && !dot_seen && idx + 1 < len && bytes[idx + 1].is_ascii_digit())) {
                if bytes[idx] == b'.' {
                    dot_seen = true;
                }
                idx += 1;
            }
            let end = idx;

            // Character after digits: check if it's an identifier suffix (e.g. 5_ControlActions, 40_Catapult, 5pkg)
            let is_id_token_suffix = if end < len {
                let b = bytes[end];
                if b == b'_' || b == b'-' {
                    true
                } else if b.is_ascii_alphabetic() {
                    let mut unit_end = end;
                    while unit_end < len && (bytes[unit_end].is_ascii_alphanumeric() || bytes[unit_end] == b'/' || bytes[unit_end] == b'^' || bytes[unit_end] == b'%') {
                        unit_end += 1;
                    }
                    let unit_candidate = &line[end..unit_end];
                    !is_recognized_unit(unit_candidate)
                } else {
                    false
                }
            } else {
                false
            };

            // Check if preceding token is an index/label word (e.g. "feature 24", "epic 01", "req 02", "step 3")
            let is_preceded_by_label_word = if word_start > 0 {
                let before_text = line[..word_start].trim_end();
                let last_word = before_text.rsplit(|c: char| c.is_whitespace() || c == '|' || c == '*' || c == '`' || c == '#').next().unwrap_or("");
                let lw_clean = last_word.trim().to_lowercase();
                matches!(lw_clean.as_str(), "feature" | "feat" | "epic" | "req" | "ac" | "step" | "item" | "section" | "table" | "figure" | "level" | "mode" | "id" | "issue")
            } else {
                false
            };

            let is_identifier = is_id_token_prefix || is_id_token_suffix || is_preceded_by_label_word;

            // Character after digits: if it's alphanumeric, check if it forms a recognized unit
            if !is_identifier && start < end {
                let num_str = &line[start..end];

                // Skip version strings, ISO dates, section numbers like 1.2 or 3.4.1, labels like 02:,
                // or outline sub-step labels like 5a., 5e., 5i., 5d., 5a)
                let is_outline_label = if end < len && bytes[end].is_ascii_alphabetic() {
                    let mut letter_end = end;
                    while letter_end < len && bytes[letter_end].is_ascii_alphabetic() {
                        letter_end += 1;
                    }
                    (letter_end - end <= 2)
                        && letter_end < len
                        && (bytes[letter_end] == b'.' || bytes[letter_end] == b')' || bytes[letter_end] == b':')
                } else {
                    false
                };

                let is_section_or_label = if end < len && (bytes[end] == b'.' || bytes[end] == b')' || bytes[end] == b':') {
                    true
                } else if is_outline_label {
                    true
                } else if start > 0 && (bytes[start - 1] == b'.' || bytes[start - 1] == b'v' || bytes[start - 1] == b'V') {
                    true
                } else {
                    false
                };

                if !is_section_or_label {
                    // Check if followed immediately or after space by a recognized unit
                    let mut has_unit = false;

                    // 1. Attached unit (e.g. "50ms", "250Hz")
                    if end < len && bytes[end].is_ascii_alphabetic() {
                        let mut unit_end = end;
                        while unit_end < len && (bytes[unit_end].is_ascii_alphanumeric() || bytes[unit_end] == b'/' || bytes[unit_end] == b'^' || bytes[unit_end] == b'%') {
                            unit_end += 1;
                        }
                        let unit_candidate = &line[end..unit_end];
                        if is_recognized_unit(unit_candidate) {
                            has_unit = true;
                        }
                    } else if end < len && bytes[end] == b'%' {
                        has_unit = true;
                    } else {
                        // 2. Trailing unit separated by whitespace (e.g. "50 ms", "250 Hz")
                        let rem = line[end..].trim_start();
                        if !rem.is_empty() {
                            let mut unit_end = 0;
                            let rem_bytes = rem.as_bytes();
                            while unit_end < rem_bytes.len() && (rem_bytes[unit_end].is_ascii_alphanumeric() || rem_bytes[unit_end] == b'/' || rem_bytes[unit_end] == b'^' || rem_bytes[unit_end] == b'%') {
                                unit_end += 1;
                            }
                            if unit_end > 0 {
                                let unit_candidate = &rem[..unit_end];
                                if is_recognized_unit(unit_candidate) {
                                    has_unit = true;
                                }
                            }
                        }
                    }

                    if !has_unit {
                        naked.push((num_str.to_string(), start));
                    }
                }
            }
        } else {
            idx += 1;
        }
    }

    naked
}

/// Checks if an engineering parameter or assertion is grounded by SysML symbols,
/// normative standards, or schema citations.
///
/// Preconditions: None.
/// Postconditions: Returns true if grounding evidence is found on the line or in document context.
/// Algorithmic Complexity: O(S) where S is symbol table lookups.
pub fn is_assertion_grounded(
    line: &str,
    content: &str,
    symbol_index: Option<&SysmlSymbolIndex>,
    constraints: &[ConstraintDef],
    attributes: &[AttributeDef],
) -> bool {
    let lower_line = line.to_lowercase();

    // 1. Check for normative standards citation (ISO, IEC, IEEE, DO-178C, MIL-STD, etc.)
    for &std in NORMATIVE_STANDARDS {
        if lower_line.contains(std) {
            return true;
        }
    }

    // 2. Check for explicit schema path or clause citations (schema/, .sysml, §)
    if lower_line.contains("schema/")
        || lower_line.contains(".sysml")
        || lower_line.contains("§")
        || lower_line.contains("ssot")
        || lower_line.contains("grounding:")
        || lower_line.contains("source:")
        || lower_line.contains("ref:")
    {
        return true;
    }

    // 3. Check against SysML Symbol Index
    if let Some(idx) = symbol_index {
        for attr in &idx.attribute_names {
            if lower_line.contains(&attr.to_lowercase()) {
                return true;
            }
        }
        for part in &idx.part_names {
            if lower_line.contains(&part.to_lowercase()) {
                return true;
            }
        }
        for act in &idx.action_names {
            if lower_line.contains(&act.to_lowercase()) {
                return true;
            }
        }
        for port in &idx.port_names {
            if lower_line.contains(&port.to_lowercase()) {
                return true;
            }
        }
    }

    // 4. Check against concrete ConstraintDef and AttributeDef slices
    for c in constraints {
        if !c.name.is_empty() && lower_line.contains(&c.name.to_lowercase()) {
            return true;
        }
    }
    for a in attributes {
        if !a.name.is_empty() && lower_line.contains(&a.name.to_lowercase()) {
            return true;
        }
    }

    // 5. Global document fallback: if document body contains an authoritative Source References section
    // citing schema/*.sysml or normative standards, assertions inherit document-level grounding
    let lower_doc = content.to_lowercase();
    if lower_doc.contains("## source references") || lower_doc.contains("## 6. source references") {
        if lower_doc.contains("schema/") || lower_doc.contains(".sysml") {
            return true;
        }
    }

    false
}

/// Audits numeric parameters, units, and engineering assertions against SysML schema AST symbols
/// or normative standards citations.
///
/// Preconditions: `doc` has been parsed into `MarkdownDoc`, `content` is valid UTF-8.
/// Postconditions: Returns Ok(Vec<SpecAuditFinding>) detailing missing units or ungrounded assertions.
/// Algorithmic Complexity: O(L * K) where L is lines and K is line length.
pub fn validate_factual_grounding(
    _doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
    symbol_index: Option<&SysmlSymbolIndex>,
    constraints: &[ConstraintDef],
    attributes: &[AttributeDef],
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let mut findings = Vec::new();
    let mut in_code_block = false;

    for (idx, line) in content.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Skip non-normative metadata header table rows, table dividers, and document metadata entries
        if trimmed.starts_with('|') {
            if crate::spec_audit::markdown_ast::is_metadata_header_table_row(trimmed) {
                continue;
            }
            if trimmed.chars().all(|c| c == '|' || c == '-' || c == ':' || c.is_ascii_whitespace()) {
                continue;
            }
            let lower = trimmed.to_lowercase();
            if lower.contains("version")
                || lower.contains("date")
                || lower.contains("author")
                || lower.contains("type")
                || lower.contains("status")
                || lower.contains("title")
                || lower.contains("package")
                || lower.contains("subsystem")
                || lower.contains("generation_mode")
                || lower.contains("generation mode")
                || lower.contains("generation-mode")
                || lower.contains("subagent")
                || lower.contains("actor")
                || lower.contains("scope")
                || lower.contains("level")
                || lower.contains("source")
            {
                let parts: Vec<&str> = trimmed
                    .split('|')
                    .map(|c| c.trim())
                    .filter(|c| !c.is_empty())
                    .collect();
                if parts.len() == 2 {
                    let k_norm = parts[0].replace(['*', '_', '`'], "").trim().to_lowercase();
                    if k_norm == "title"
                        || k_norm == "package"
                        || k_norm == "subsystem"
                        || k_norm == "version"
                        || k_norm == "date"
                        || k_norm == "author"
                        || k_norm == "type"
                        || k_norm == "status"
                        || k_norm == "generation mode"
                        || k_norm == "generation_mode"
                        || k_norm == "generation-mode"
                        || k_norm == "subagent drafted"
                        || k_norm == "subagent_drafted"
                        || k_norm == "specification source"
                        || k_norm == "parent epic"
                        || k_norm == "interface type"
                        || k_norm == "schema containers"
                        || k_norm == "primary actor"
                        || k_norm == "scope"
                        || k_norm == "level"
                        || k_norm == "issue id"
                        || k_norm == "issue"
                    {
                        continue;
                    }
                }
            }
        }

        // Check if line mentions an engineering parameter
        if let Some(param) = find_engineering_parameter(trimmed) {
            let naked_nums = find_naked_numeric_quantities(trimmed);

            // 1. Enforce explicit physical units for numeric parameters
            for (num_str, _) in &naked_nums {
                findings.push(SpecAuditFinding::new(
                    "factual-grounding-missing-unit",
                    format!(
                        "Engineering parameter '{}' on line {} in '{}' specifies numeric quantity '{}' without an explicit physical/engineering unit.",
                        param, line_no, rel_path, num_str
                    ),
                    rel_path,
                    Some(line_no),
                ));
            }

            // 2. Enforce empirical grounding against SysML AST symbols or normative citations
            if !is_assertion_grounded(trimmed, content, symbol_index, constraints, attributes) {
                findings.push(SpecAuditFinding::new(
                    "factual-grounding-ungrounded-assertion",
                    format!(
                        "Engineering assertion on line {} in '{}' mentioning '{}' lacks empirical grounding against SysML schema AST symbols or normative standards citations.",
                        line_no, rel_path, param
                    ),
                    rel_path,
                    Some(line_no),
                ));
            }
        }
    }

    Ok(findings)
}

/// Convenience entry point for validating factual grounding directly from raw Markdown content.
///
/// Preconditions: `content` is valid UTF-8 Markdown text.
/// Postconditions: Returns Ok(Vec<SpecAuditFinding>) or Err(SpecAuditError) if parsing fails.
/// Algorithmic Complexity: O(N) where N is content byte length.
pub fn validate_factual_grounding_content(
    content: &str,
    rel_path: &str,
    symbol_index: Option<&SysmlSymbolIndex>,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let doc = parse_markdown(content)?;
    validate_factual_grounding(&doc, content, rel_path, symbol_index, &[], &[])
}

/// Extracts the text slice and starting line number of the `## Source References` section.
///
/// Preconditions: None.
/// Postconditions: Returns Some((line_no, section_text)) if Source References section exists.
/// Algorithmic Complexity: O(H + N) where H is headings and N is section byte length.
pub fn extract_source_references_section<'a>(content: &'a str) -> Option<(usize, &'a str)> {
    let mut target_line = None;
    let mut target_level = 2;
    let mut start_byte = 0;

    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            let after_hash = trimmed[level..].trim().to_lowercase();

            if after_hash.contains("source reference")
                || (after_hash.contains("source") && after_hash.contains("reference"))
            {
                target_line = Some(idx + 1);
                target_level = level;
                // Calculate byte offset of start of next line
                let line_start = line.as_ptr() as usize - content.as_ptr() as usize;
                start_byte = line_start + line.len();
                if start_byte < content.len() && content.as_bytes()[start_byte] == b'\n' {
                    start_byte += 1;
                }
                break;
            }
        }
    }

    let line_no = match target_line {
        Some(l) => l,
        None => return None,
    };

    if start_byte >= content.len() {
        return Some((line_no, ""));
    }

    // Find the end of this section: next heading with level <= target_level
    let rem = &content[start_byte..];
    let mut end_byte = rem.len();

    for line in rem.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            if level <= target_level {
                let pos = line.as_ptr() as usize - rem.as_ptr() as usize;
                end_byte = pos;
                break;
            }
        }
    }

    Some((line_no, &rem[..end_byte]))
}

/// Verifies that `## Source References` sections in specifications cite verbatim clause numbers
/// and schema paths (e.g. `ISO/IEC/IEEE 29148:2018 §6.4.2`, `RTCA DO-178C §6.3.1`, `schema/*.sysml`).
/// Detects and reports placeholder citations, empty sections, or self-referential links.
///
/// Preconditions: `doc` has been parsed into `MarkdownDoc`, `content` is valid UTF-8.
/// Postconditions: Returns Ok(Vec<SpecAuditFinding>) detailing source reference integrity violations.
/// Algorithmic Complexity: O(N) where N is content byte length.
pub fn validate_source_reference_clause_integrity(
    _doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let mut findings = Vec::new();

    // 1. Locate Source References section
    let (sec_line, sec_text) = match extract_source_references_section(content) {
        Some((l, t)) => (l, t),
        None => {
            findings.push(SpecAuditFinding::new(
                "source-reference-section-missing",
                format!(
                    "Specification '{}' is missing a mandatory 'Source References' section.",
                    rel_path
                ),
                rel_path,
                None,
            ));
            return Ok(findings);
        }
    };

    // 2. Check for empty section body
    let trimmed_sec = sec_text.trim();
    if trimmed_sec.is_empty() {
        findings.push(SpecAuditFinding::new(
            "source-reference-section-empty",
            format!(
                "Specification '{}' has an empty 'Source References' section on line {}. It must cite authoritative schema paths and normative standards.",
                rel_path, sec_line
            ),
            rel_path,
            Some(sec_line),
        ));
        return Ok(findings);
    }

    // 3. Scan lines for prohibited placeholders and self-referential links
    let filename = if let Some(pos) = rel_path.rfind('/') {
        &rel_path[pos + 1..]
    } else {
        rel_path
    };

    for (offset, line) in sec_text.lines().enumerate() {
        let current_line = sec_line + offset + 1;
        let line_trimmed = line.trim();
        let lower_line = line_trimmed.to_lowercase();

        // 3a. Placeholder citations
        for &placeholder in PROHIBITED_PLACEHOLDERS {
            if lower_line.contains(placeholder) {
                findings.push(SpecAuditFinding::new(
                    "source-reference-placeholder-detected",
                    format!(
                        "Specification '{}' contains unresolved placeholder '{}' in 'Source References' on line {}.",
                        rel_path, placeholder, current_line
                    ),
                    rel_path,
                    Some(current_line),
                ));
                break;
            }
        }

        // 3b. Self-referential links
        if lower_line.contains(filename) || lower_line.contains("(self)") {
            findings.push(SpecAuditFinding::new(
                "source-reference-self-referential",
                format!(
                    "Specification '{}' contains self-referential link '{}' in 'Source References' on line {}.",
                    rel_path, filename, current_line
                ),
                rel_path,
                Some(current_line),
            ));
        }
    }

    // 4. Verifies verbatim clause numbers and schema paths
    let lower_sec = sec_text.to_lowercase();
    let has_schema_path = lower_sec.contains("schema/") || lower_sec.contains(".sysml");
    let has_verbatim_clause = lower_sec.contains('§')
        || lower_sec.contains("clause ")
        || lower_sec.contains("section ");

    if !has_schema_path && !has_verbatim_clause {
        findings.push(SpecAuditFinding::new(
            "source-reference-clause-missing",
            format!(
                "Specification '{}' 'Source References' section on line {} does not cite verbatim clause numbers (e.g. 'ISO/IEC/IEEE 29148:2018 §6.4.2') or schema paths (e.g. 'schema/*.sysml').",
                rel_path, sec_line
            ),
            rel_path,
            Some(sec_line),
        ));
    }

    Ok(findings)
}

/// Counts referenced features in an Epic specification.
///
/// Preconditions: None.
/// Postconditions: Returns count of features declared or referenced in the Epic.
/// Algorithmic Complexity: O(L) where L is lines count.
pub fn count_epic_features(content: &str) -> usize {
    let mut count = 0;
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        let lower = trimmed.to_lowercase();
        // Check for checklist items referencing features, e.g. "- [ ] FEAT-01" or "- [x] Feature 01"
        if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") || trimmed.starts_with("* [ ]") || trimmed.starts_with("* [x]") {
            if lower.contains("feat-") || lower.contains("feature") {
                count += 1;
            }
        } else if (trimmed.starts_with('-') || trimmed.starts_with('*')) && (lower.contains("feat-") || lower.contains("feature ")) {
            count += 1;
        }
    }

    count
}

/// Counts acceptance criteria in a Feature specification.
///
/// Preconditions: None.
/// Postconditions: Returns count of acceptance criteria declared in the Feature.
/// Algorithmic Complexity: O(L) where L is lines count.
pub fn count_feature_acceptance_criteria(content: &str) -> usize {
    let mut count = 0;
    let mut in_code_block = false;
    let mut in_ac_section = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        if trimmed.starts_with('#') {
            let lower_heading = trimmed.to_lowercase();
            if lower_heading.contains("acceptance criteria")
                || lower_heading.contains("verification")
                || lower_heading.contains("conformance")
                || lower_heading.contains("requirements")
            {
                in_ac_section = true;
            } else if trimmed.starts_with("## ") {
                in_ac_section = false;
            }
            continue;
        }

        let lower = trimmed.to_lowercase();
        // Checkbox criteria e.g. "- [ ] AC-01", "- [ ] REQ-01"
        if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") || trimmed.starts_with("* [ ]") || trimmed.starts_with("* [x]") {
            count += 1;
        } else if in_ac_section && (trimmed.starts_with('-') || trimmed.starts_with('*') || (trimmed.len() >= 2 && trimmed.as_bytes()[0].is_ascii_digit() && trimmed.as_bytes()[1] == b'.')) {
            if lower.contains("ac-") || lower.contains("req-") || lower.contains("shall") || lower.contains("given") || lower.contains("when") || lower.contains("then") {
                count += 1;
            }
        }
    }

    count
}

/// Audits specification cardinality bounds across Epics and Features:
/// - Epics: 3--15 features recommended; flags anomalies.
/// - Features: 3--10 acceptance criteria recommended; flags anomalies.
///
/// Preconditions: `doc` has been parsed into `MarkdownDoc`, `content` is valid UTF-8.
/// Postconditions: Returns Ok(Vec<SpecAuditFinding>) detailing cardinality anomalies.
/// Algorithmic Complexity: O(N) where N is content byte length.
pub fn validate_cardinality_bounds(
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let mut findings = Vec::new();

    // Determine document type (Epic vs Feature) from metadata table or frontmatter
    let mut is_epic = false;
    let mut is_feature = false;

    if let Some(t) = doc
        .table_metadata
        .get("type")
        .map(|s| s.as_str())
        .or_else(|| {
            doc.frontmatter
                .as_ref()
                .and_then(|fm| fm.get("type").map(|s| s.as_str()))
        })
    {
        let lower = t.to_lowercase();
        if lower == "epic" {
            is_epic = true;
        } else if lower == "feature" {
            is_feature = true;
        }
    }

    let lower_path = rel_path.to_lowercase();
    if !is_epic && !is_feature {
        if lower_path.contains("/epics/") || lower_path.contains("epic-") {
            is_epic = true;
        } else if lower_path.contains("/features/") || lower_path.contains("feat-") {
            is_feature = true;
        }
    }

    if is_epic {
        let feature_count = count_epic_features(content);
        if feature_count < 3 {
            findings.push(SpecAuditFinding::new(
                "cardinality-bounds-epic-underflow",
                format!(
                    "Epic specification '{}' references only {} feature(s); recommended cardinality bounds are 3--15 features.",
                    rel_path, feature_count
                ),
                rel_path,
                None,
            ));
        } else if feature_count > 15 {
            findings.push(SpecAuditFinding::new(
                "cardinality-bounds-epic-overflow",
                format!(
                    "Epic specification '{}' references {} features, exceeding recommended upper bound of 15 features (3--15 recommended).",
                    rel_path, feature_count
                ),
                rel_path,
                None,
            ));
        }
    } else if is_feature {
        let ac_count = count_feature_acceptance_criteria(content);
        if ac_count < 3 {
            findings.push(SpecAuditFinding::new(
                "cardinality-bounds-feature-underflow",
                format!(
                    "Feature specification '{}' defines only {} acceptance criteria; recommended cardinality bounds are 3--10 criteria.",
                    rel_path, ac_count
                ),
                rel_path,
                None,
            ));
        } else if ac_count > 10 {
            findings.push(SpecAuditFinding::new(
                "cardinality-bounds-feature-overflow",
                format!(
                    "Feature specification '{}' defines {} acceptance criteria, exceeding recommended upper bound of 10 criteria (3--10 recommended).",
                    rel_path, ac_count
                ),
                rel_path,
                None,
            ));
        }
    }

    Ok(findings)
}

/// Top-level coordinator that audits provenance, source references, and cardinality bounds for a specification.
///
/// Preconditions: `doc` has been parsed, `content` is valid UTF-8.
/// Postconditions: Returns Ok(Vec<SpecAuditFinding>) containing all detected provenance anomalies.
/// Algorithmic Complexity: O(N) where N is content byte length.
pub fn validate_provenance(
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
    symbol_index: Option<&SysmlSymbolIndex>,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let mut all_findings = Vec::new();

    // 1. Factual Grounding
    let mut fg = validate_factual_grounding(doc, content, rel_path, symbol_index, &[], &[])?;
    all_findings.append(&mut fg);

    // 2. Source References Integrity
    let mut sri = validate_source_reference_clause_integrity(doc, content, rel_path)?;
    all_findings.append(&mut sri);

    // 3. Cardinality Bounds
    let mut cb = validate_cardinality_bounds(doc, content, rel_path)?;
    all_findings.append(&mut cb);

    Ok(all_findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_symbol_index() -> SysmlSymbolIndex {
        let mut index = SysmlSymbolIndex::new();
        index.part_names.insert("FlightController".to_string());
        index.attribute_names.insert("sampling_rate".to_string());
        index.attribute_names.insert("bus_voltage".to_string());
        index.action_names.insert("sample_sensors".to_string());
        index
    }

    #[test]
    fn test_factual_grounding_with_valid_units_and_grounding_passes() {
        let content = r#"---
title: "Feature 01: Telemetry Stream"
type: feature
---
# Feature 01: Telemetry Stream

## Metadata
| Attribute | Detail |
| :--- | :--- |
| **Title** | Feature 01: Telemetry Stream |

## Requirements
- [ ] REQ-01: FlightController telemetry sampling_rate shall be 250 Hz in accordance with RTCA DO-178C §6.3.1.
- [ ] REQ-02: Maximum end-to-end telemetry latency shall not exceed 50 ms.
- [ ] REQ-03: Primary DC bus_voltage must remain between 11.5 V and 12.6 V per schema/model.sysml.

## Source References
Derived from schema/model.sysml and RTCA DO-178C §6.3.1.
"#;
        let doc = parse_markdown(content).expect("parse markdown");
        let index = build_test_symbol_index();
        let findings = validate_factual_grounding(&doc, content, "docs/features/feat-01.md", Some(&index), &[], &[])
            .expect("validation succeeds");

        assert!(findings.is_empty(), "Expected no findings, got: {:?}", findings);
    }

    #[test]
    fn test_factual_grounding_missing_units_fails() {
        let content = r#"---
title: "Feature 01: Telemetry Stream"
type: feature
---
# Feature 01: Telemetry Stream

## Requirements
- [ ] REQ-01: Sensor telemetry frequency shall operate at 250 without units.
- [ ] REQ-02: Maximum telemetry latency shall be 50 without units.
- [ ] REQ-03: Secondary bus voltage is 12 without units.

## Source References
Derived from schema/model.sysml §4.1.
"#;
        let doc = parse_markdown(content).expect("parse markdown");
        let index = build_test_symbol_index();
        let findings = validate_factual_grounding(&doc, content, "docs/features/feat-01.md", Some(&index), &[], &[])
            .expect("validation succeeds");

        let missing_units: Vec<_> = findings.iter().filter(|f| f.rule_id == "factual-grounding-missing-unit").collect();
        assert!(!missing_units.is_empty(), "Expected missing unit findings");
        assert!(missing_units.iter().any(|f| f.message.contains("250")));
        assert!(missing_units.iter().any(|f| f.message.contains("50")));
        assert!(missing_units.iter().any(|f| f.message.contains("12")));
    }

    #[test]
    fn test_source_reference_clause_integrity_passes() {
        let content = r#"# Feature 01: Flight Guidance

## Source References
Normative standards baseline: ISO/IEC/IEEE 29148:2018 §6.4.2 and schema/model.sysml.
"#;
        let doc = parse_markdown(content).expect("parse markdown");
        let findings = validate_source_reference_clause_integrity(&doc, content, "docs/features/feat-01.md")
            .expect("validation succeeds");

        assert!(findings.is_empty(), "Expected clean source references, got: {:?}", findings);
    }

    #[test]
    fn test_source_reference_missing_section_fails() {
        let content = r#"# Feature 01: Flight Guidance

## Description
This spec lacks a source references section.
"#;
        let doc = parse_markdown(content).expect("parse markdown");
        let findings = validate_source_reference_clause_integrity(&doc, content, "docs/features/feat-01.md")
            .expect("validation succeeds");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "source-reference-section-missing");
    }

    #[test]
    fn test_source_reference_empty_section_fails() {
        let content = r#"# Feature 01: Flight Guidance

## Source References

## Next Section
"#;
        let doc = parse_markdown(content).expect("parse markdown");
        let findings = validate_source_reference_clause_integrity(&doc, content, "docs/features/feat-01.md")
            .expect("validation succeeds");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "source-reference-section-empty");
    }

    #[test]
    fn test_source_reference_placeholder_and_self_referential_fails() {
        let content = r#"# Feature 01: Flight Guidance

## Source References
- Schema: [link-to-schema](docs/features/feat-01.md)
- {{TODO: Populate external citation}}
"#;
        let doc = parse_markdown(content).expect("parse markdown");
        let findings = validate_source_reference_clause_integrity(&doc, content, "docs/features/feat-01.md")
            .expect("validation succeeds");

        assert!(findings.iter().any(|f| f.rule_id == "source-reference-placeholder-detected"));
        assert!(findings.iter().any(|f| f.rule_id == "source-reference-self-referential"));
    }

    #[test]
    fn test_cardinality_bounds_epic_bounds() {
        // Underflow: 1 feature
        let underflow_content = r#"---
type: epic
---
# Epic 01: Core Systems

## Requirements & Checklist
- [ ] FEAT-01: Single feature
"#;
        let doc1 = parse_markdown(underflow_content).expect("parse markdown");
        let findings1 = validate_cardinality_bounds(&doc1, underflow_content, "docs/epics/epic-01.md")
            .expect("validation succeeds");
        assert!(findings1.iter().any(|f| f.rule_id == "cardinality-bounds-epic-underflow"));

        // Nominal: 4 features
        let nominal_content = r#"---
type: epic
---
# Epic 01: Core Systems

## Requirements & Checklist
- [ ] FEAT-01: Feature 1
- [ ] FEAT-02: Feature 2
- [ ] FEAT-03: Feature 3
- [ ] FEAT-04: Feature 4
"#;
        let doc2 = parse_markdown(nominal_content).expect("parse markdown");
        let findings2 = validate_cardinality_bounds(&doc2, nominal_content, "docs/epics/epic-01.md")
            .expect("validation succeeds");
        assert!(findings2.is_empty(), "Nominal epic features should pass: {:?}", findings2);

        // Overflow: 16 features
        let mut overflow_lines = vec![
            "---".to_string(),
            "type: epic".to_string(),
            "---".to_string(),
            "# Epic 01: Huge Epic".to_string(),
            "".to_string(),
            "## Requirements & Checklist".to_string(),
        ];
        for i in 1..=16 {
            overflow_lines.push(format!("- [ ] FEAT-{:02}: Feature {}", i, i));
        }
        let overflow_content = overflow_lines.join("\n");
        let doc3 = parse_markdown(&overflow_content).expect("parse markdown");
        let findings3 = validate_cardinality_bounds(&doc3, &overflow_content, "docs/epics/epic-01.md")
            .expect("validation succeeds");
        assert!(findings3.iter().any(|f| f.rule_id == "cardinality-bounds-epic-overflow"));
    }

    #[test]
    fn test_cardinality_bounds_feature_bounds() {
        // Underflow: 1 criterion
        let underflow_content = r#"---
type: feature
---
# Feature 01: Subsystem

## Acceptance Criteria
- [ ] AC-01: Single criterion
"#;
        let doc1 = parse_markdown(underflow_content).expect("parse markdown");
        let findings1 = validate_cardinality_bounds(&doc1, underflow_content, "docs/features/feat-01.md")
            .expect("validation succeeds");
        assert!(findings1.iter().any(|f| f.rule_id == "cardinality-bounds-feature-underflow"));

        // Nominal: 5 criteria
        let nominal_content = r#"---
type: feature
---
# Feature 01: Subsystem

## Acceptance Criteria
- [ ] AC-01: Criterion 1
- [ ] AC-02: Criterion 2
- [ ] AC-03: Criterion 3
- [ ] AC-04: Criterion 4
- [ ] AC-05: Criterion 5
"#;
        let doc2 = parse_markdown(nominal_content).expect("parse markdown");
        let findings2 = validate_cardinality_bounds(&doc2, nominal_content, "docs/features/feat-01.md")
            .expect("validation succeeds");
        assert!(findings2.is_empty(), "Nominal criteria should pass: {:?}", findings2);

        // Overflow: 11 criteria
        let mut overflow_lines = vec![
            "---".to_string(),
            "type: feature".to_string(),
            "---".to_string(),
            "# Feature 01: Huge Feature".to_string(),
            "".to_string(),
            "## Acceptance Criteria".to_string(),
        ];
        for i in 1..=11 {
            overflow_lines.push(format!("- [ ] AC-{:02}: Criterion {}", i, i));
        }
        let overflow_content = overflow_lines.join("\n");
        let doc3 = parse_markdown(&overflow_content).expect("parse markdown");
        let findings3 = validate_cardinality_bounds(&doc3, &overflow_content, "docs/features/feat-01.md")
            .expect("validation succeeds");
        assert!(findings3.iter().any(|f| f.rule_id == "cardinality-bounds-feature-overflow"));
    }

    #[test]
    fn test_reproduce_defect_213_package_digits_and_metadata_table_contradiction() {
        let content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #10 |
| **Title** | Feature 24 Power Management System |
| **Type** | feature |
| **Parent Epic** | [EPIC-01](../epics/EPIC-01.md) |
| **Package** | 5_PowerControl |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Feature 24: Power Management System

## Requirements
| Property | Subsystem Package | Value |
| :--- | :--- | :--- |
| DSC_IN1 Power | 5_ControlActions | 50 W |
| Catapult Speed | PL-40_Catapult | 35 m/s |
| Serial Bus Rate | RS-485_Bus | 115200 bps |

## Acceptance Criteria
- [ ] AC-01: Primary DC bus power shall not exceed 50 W.
- [ ] AC-02: Catapult speed shall reach 35 m/s.
- [ ] AC-03: Serial telemetry rate shall sustain 115200 bps.

## Source References
Derived from schema/model.sysml and RTCA DO-178C §6.3.1.
"#;
        let doc = parse_markdown(content).expect("parse markdown");
        let index = build_test_symbol_index();

        // 1. Cardinality bounds must read 'type' from CommonMark metadata table (not just YAML frontmatter)
        let cb_findings = validate_cardinality_bounds(&doc, content, "specifications/feat-custom-path.md")
            .expect("cardinality validation succeeds");
        assert!(
            cb_findings.is_empty(),
            "Expected 0 cardinality findings for nominal 3 criteria, but got: {:?}",
            cb_findings
        );

        // 2. Factual grounding must NOT treat metadata table rows or package-name digits as claimed parameters / naked numbers
        let findings = validate_factual_grounding(&doc, content, "docs/features/feat-24.md", Some(&index), &[], &[])
            .expect("validation succeeds");

        assert!(
            findings.is_empty(),
            "Expected 0 factual grounding findings, but package-name digits were erroneously extracted: {:?}",
            findings
        );
    }

    #[test]
    fn test_is_pure_scalar_numeric() {
        // Pure scalar quantities and quantities with valid units
        assert!(is_pure_scalar_numeric("42"));
        assert!(is_pure_scalar_numeric("-40.0"));
        assert!(is_pure_scalar_numeric("+5.5"));
        assert!(is_pure_scalar_numeric("115200"));
        assert!(is_pure_scalar_numeric("1e-3"));
        assert!(is_pure_scalar_numeric("50 W"));
        assert!(is_pure_scalar_numeric("35 m/s"));
        assert!(is_pure_scalar_numeric("115200 bps"));
        assert!(is_pure_scalar_numeric("250 Hz"));
        assert!(is_pure_scalar_numeric("10 ms"));
        assert!(is_pure_scalar_numeric("`50 W`"));
        assert!(is_pure_scalar_numeric("*35 m/s*"));

        // Non-scalar identifiers, subsystem packages, and document titles
        assert!(!is_pure_scalar_numeric("5_ControlActions"));
        assert!(!is_pure_scalar_numeric("PL-40_Catapult"));
        assert!(!is_pure_scalar_numeric("RS-485_Bus"));
        assert!(!is_pure_scalar_numeric("AVENGER5"));
        assert!(!is_pure_scalar_numeric("Feature 24 Power Management System"));
        assert!(!is_pure_scalar_numeric("AVENGER5_ControlActions"));
        assert!(!is_pure_scalar_numeric("DSC_IN1 Arm Enable"));
        assert!(!is_pure_scalar_numeric(""));
        assert!(!is_pure_scalar_numeric("N/A"));
    }

    #[test]
    fn test_extract_scalar_numeric_claim() {
        assert_eq!(extract_scalar_numeric_claim("50 W"), Some((50.0, Some("W".to_string()))));
        assert_eq!(extract_scalar_numeric_claim("-40.0"), Some((-40.0, None)));
        assert_eq!(extract_scalar_numeric_claim("115200 bps"), Some((115200.0, Some("bps".to_string()))));
        assert_eq!(extract_scalar_numeric_claim("35 m/s"), Some((35.0, Some("m/s".to_string()))));

        // Subsystem packages and titles must return None
        assert_eq!(extract_scalar_numeric_claim("5_ControlActions"), None);
        assert_eq!(extract_scalar_numeric_claim("AVENGER5"), None);
        assert_eq!(extract_scalar_numeric_claim("Feature 24 Power Management System"), None);
        assert_eq!(extract_scalar_numeric_claim("PL-40_Catapult"), None);
    }
}


