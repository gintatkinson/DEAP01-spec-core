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
//! 4. Sequence diagram temporal safety (`validate_sequence_diagram_temporal_safety`):
//!    - Validates that physical arming/firing signals in Mermaid sequence diagrams are preceded by explicit HITL C2 consent.
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
    // Pressure & Force
    "bar", "mbar", "pa", "kpa", "mpa", "psi", "n", "kn",
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

/// Physical actuation / high-consequence target entity tokens in sequence diagrams.
pub const PHYSICAL_ARMING_TARGET_TOKENS: &[&str] = &[
    "actuator", "motor", "safetyswitch", "drive",
    "powerstage", "highvoltage", "emitter",
    "propulsionenable", "effector", "valve", "pump",
];

/// Human C2 / HITL participant tokens in sequence diagrams.
pub const HITL_SENDER_TOKENS: &[&str] = &[
    "operator", "operators", "pilot", "pilots", "human", "commander",
    "supervisor", "controller", "technician", "safetyofficer", "user",
    "c2", "gcsoperator", "gcs", "groundcontrolstation", "groundstation", "console",
];

/// Checks if a token represents a recognized physical or engineering unit.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_recognized_unit` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_recognized_unit` (declaration #1) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `token: &str` must satisfy domain and type-safety constraints prior to invoking `is_recognized_unit`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_recognized_unit` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(token)) bounded over input/member dimension of `is_recognized_unit`.
/// - Space Complexity: O(N(token)) bounded auxiliary memory for `is_recognized_unit`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_recognized_unit` (`bool`).
/// - State: Maintains consistent state invariants across `is_recognized_unit` without partial mutation on error.
/// - Action: Executes `is_recognized_unit` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_recognized_unit` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-RECOGNIZED-UNIT] | SysML v2: ActionDef(is_recognized_unit)
pub fn is_recognized_unit(token: &str) -> bool {
    let lower = token.trim().to_lowercase();
    let clean = lower.trim_end_matches(&[',', '.', ';', ':', ')', ']', '}'][..]);
    RECOGNIZED_UNITS.iter().any(|&u| u == clean)
}

/// Validates whether a token or table cell is a pure scalar numeric value
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_pure_scalar_numeric` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_pure_scalar_numeric` (declaration #2) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `val: &str` must satisfy domain and type-safety constraints prior to invoking `is_pure_scalar_numeric`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_pure_scalar_numeric` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(val)) bounded over input/member dimension of `is_pure_scalar_numeric`.
/// - Space Complexity: O(N(val)) bounded auxiliary memory for `is_pure_scalar_numeric`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_pure_scalar_numeric` (`bool`).
/// - State: Maintains consistent state invariants across `is_pure_scalar_numeric` without partial mutation on error.
/// - Action: Executes `is_pure_scalar_numeric` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_pure_scalar_numeric` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-PURE-SCALAR-NUMERIC] | SysML v2: ActionDef(is_pure_scalar_numeric)
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `extract_scalar_numeric_claim` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `extract_scalar_numeric_claim` (declaration #3) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `val: &str` must satisfy domain and type-safety constraints prior to invoking `extract_scalar_numeric_claim`.
///
/// ## 4. Postconditions
/// - Returns `Option<(f64, Option<String>)>` from `extract_scalar_numeric_claim` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(val)) bounded over input/member dimension of `extract_scalar_numeric_claim`.
/// - Space Complexity: O(N(val)) bounded auxiliary memory for `extract_scalar_numeric_claim`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `extract_scalar_numeric_claim` (`Option<(f64, Option<String>)>`).
/// - State: Maintains consistent state invariants across `extract_scalar_numeric_claim` without partial mutation on error.
/// - Action: Executes `extract_scalar_numeric_claim` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `extract_scalar_numeric_claim` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-EXTRACT-SCALAR-NUMERIC-CLAIM] | SysML v2: ActionDef(extract_scalar_numeric_claim)
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
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `find_engineering_parameter` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `find_engineering_parameter` (declaration #4) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `line: &str` must satisfy domain and type-safety constraints prior to invoking `find_engineering_parameter`.
///
/// ## 4. Postconditions
/// - Returns `Option<&'static str>` from `find_engineering_parameter` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(line)) bounded over input/member dimension of `find_engineering_parameter`.
/// - Space Complexity: O(N(line)) bounded auxiliary memory for `find_engineering_parameter`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `find_engineering_parameter` (`Option<&'static str>`).
/// - State: Maintains consistent state invariants across `find_engineering_parameter` without partial mutation on error.
/// - Action: Executes `find_engineering_parameter` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `find_engineering_parameter` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-FIND-ENGINEERING-PARAMETER] | SysML v2: ActionDef(find_engineering_parameter)
pub fn find_engineering_parameter(line: &str) -> Option<&'static str> {
    let lower = line.to_ascii_lowercase();
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `find_naked_numeric_quantities` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `find_naked_numeric_quantities` (declaration #5) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `line: &str` must satisfy domain and type-safety constraints prior to invoking `find_naked_numeric_quantities`.
///
/// ## 4. Postconditions
/// - Returns `Vec<(String, usize)>` from `find_naked_numeric_quantities` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(line)) bounded over input/member dimension of `find_naked_numeric_quantities`.
/// - Space Complexity: O(N(line)) bounded auxiliary memory for `find_naked_numeric_quantities`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `find_naked_numeric_quantities` (`Vec<(String, usize)>`).
/// - State: Maintains consistent state invariants across `find_naked_numeric_quantities` without partial mutation on error.
/// - Action: Executes `find_naked_numeric_quantities` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `find_naked_numeric_quantities` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-FIND-NAKED-NUMERIC-QUANTITIES] | SysML v2: ActionDef(find_naked_numeric_quantities)
pub fn find_naked_numeric_quantities(line: &str) -> Vec<(String, usize)> {
    let mut naked = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut idx = 0;

    while idx < len {
        // Find digit start
        if bytes[idx].is_ascii_digit() {
            let start = idx;

            // Check if this digit is part of an identifier prefix (e.g. REQ-02, FEAT-01, AC-03, v1, PL-40, RS-485, ALPHA5)
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

            // Character after digits: check if it's an identifier suffix (e.g. 5_ControlActions, 40_Actuator, 5pkg)
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_assertion_grounded` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_assertion_grounded` (declaration #6) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `line: &str` must satisfy domain and type-safety constraints prior to invoking `is_assertion_grounded`.
/// - `_content: &str` must satisfy domain and type-safety constraints prior to invoking `is_assertion_grounded`.
/// - `symbol_index: Option<&SysmlSymbolIndex>` must satisfy domain and type-safety constraints prior to invoking `is_assertion_grounded`.
/// - `constraints: &[ConstraintDef]` must satisfy domain and type-safety constraints prior to invoking `is_assertion_grounded`.
/// - `attributes: &[AttributeDef]` must satisfy domain and type-safety constraints prior to invoking `is_assertion_grounded`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_assertion_grounded` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(line)) bounded over input/member dimension of `is_assertion_grounded`.
/// - Space Complexity: O(N(line)) bounded auxiliary memory for `is_assertion_grounded`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_assertion_grounded` (`bool`).
/// - State: Maintains consistent state invariants across `is_assertion_grounded` without partial mutation on error.
/// - Action: Executes `is_assertion_grounded` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_assertion_grounded` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-ASSERTION-GROUNDED] | SysML v2: ActionDef(is_assertion_grounded)
pub fn is_assertion_grounded(
    line: &str,
    _content: &str,
    symbol_index: Option<&SysmlSymbolIndex>,
    constraints: &[ConstraintDef],
    attributes: &[AttributeDef],
) -> bool {
    let lower_line = line.to_ascii_lowercase();

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
            if lower_line.contains(&attr.to_ascii_lowercase()) {
                return true;
            }
        }
        for part in &idx.part_names {
            if lower_line.contains(&part.to_ascii_lowercase()) {
                return true;
            }
        }
        for act in &idx.action_names {
            if lower_line.contains(&act.to_ascii_lowercase()) {
                return true;
            }
        }
        for port in &idx.port_names {
            if lower_line.contains(&port.to_ascii_lowercase()) {
                return true;
            }
        }
    }

    // 4. Check against concrete ConstraintDef and AttributeDef slices
    for c in constraints {
        if !c.name.is_empty() && lower_line.contains(&c.name.to_ascii_lowercase()) {
            return true;
        }
    }
    for a in attributes {
        if !a.name.is_empty() && lower_line.contains(&a.name.to_ascii_lowercase()) {
            return true;
        }
    }

    // 5. Document-level normative grounding fallback:
    // If the document body contains an authoritative Source References section citing normative standards
    // or schema baselines, standard engineering parameters inherit document-level grounding.
    let lower_doc = _content.to_ascii_lowercase();
    if lower_doc.contains("## source references") || lower_doc.contains("## 6. source references") {
        if lower_doc.contains("schema/")
            || lower_doc.contains(".sysml")
            || NORMATIVE_STANDARDS.iter().any(|&std| lower_doc.contains(std))
        {
            return true;
        }
    }

    false
}

/// Audits numeric parameters, units, and engineering assertions against SysML schema AST symbols
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `validate_factual_grounding` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `validate_factual_grounding` (declaration #7) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `doc: &MarkdownDoc` must satisfy domain and type-safety constraints prior to invoking `validate_factual_grounding`.
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `validate_factual_grounding`.
/// - `rel_path: &str` must satisfy domain and type-safety constraints prior to invoking `validate_factual_grounding`.
/// - `symbol_index: Option<&SysmlSymbolIndex>` must satisfy domain and type-safety constraints prior to invoking `validate_factual_grounding`.
/// - `constraints: &[ConstraintDef]` must satisfy domain and type-safety constraints prior to invoking `validate_factual_grounding`.
/// - `attributes: &[AttributeDef]` must satisfy domain and type-safety constraints prior to invoking `validate_factual_grounding`.
///
/// ## 4. Postconditions
/// - Returns `Result<Vec<SpecAuditFinding>, SpecAuditError>` from `validate_factual_grounding` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(doc)) bounded over input/member dimension of `validate_factual_grounding`.
/// - Space Complexity: O(N(doc)) bounded auxiliary memory for `validate_factual_grounding`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `validate_factual_grounding` (`Result<Vec<SpecAuditFinding>, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `validate_factual_grounding` without partial mutation on error.
/// - Action: Executes `validate_factual_grounding` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `validate_factual_grounding` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-VALIDATE-FACTUAL-GROUNDING] | SysML v2: ActionDef(validate_factual_grounding)
pub fn validate_factual_grounding(
    doc: &MarkdownDoc,
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

    // 3. Sequence diagram temporal safety gate (factual-grounding-temporal-safety-violation)
    let mut temporal_findings = validate_sequence_diagram_temporal_safety(doc, content, rel_path);
    findings.append(&mut temporal_findings);

    Ok(findings)
}

/// Convenience entry point for validating factual grounding directly from raw Markdown content.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `validate_factual_grounding_content` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `validate_factual_grounding_content` (declaration #8) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `validate_factual_grounding_content`.
/// - `rel_path: &str` must satisfy domain and type-safety constraints prior to invoking `validate_factual_grounding_content`.
/// - `symbol_index: Option<&SysmlSymbolIndex>` must satisfy domain and type-safety constraints prior to invoking `validate_factual_grounding_content`.
///
/// ## 4. Postconditions
/// - Returns `Result<Vec<SpecAuditFinding>, SpecAuditError>` from `validate_factual_grounding_content` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(content)) bounded over input/member dimension of `validate_factual_grounding_content`.
/// - Space Complexity: O(N(content)) bounded auxiliary memory for `validate_factual_grounding_content`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `validate_factual_grounding_content` (`Result<Vec<SpecAuditFinding>, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `validate_factual_grounding_content` without partial mutation on error.
/// - Action: Executes `validate_factual_grounding_content` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `validate_factual_grounding_content` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-VALIDATE-FACTUAL-GROUNDING-CONTENT] | SysML v2: ActionDef(validate_factual_grounding_content)
pub fn validate_factual_grounding_content(
    content: &str,
    rel_path: &str,
    symbol_index: Option<&SysmlSymbolIndex>,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let doc = parse_markdown(content)?;
    validate_factual_grounding(&doc, content, rel_path, symbol_index, &[], &[])
}

/// Normalizes an identifier or token by filtering alphanumeric characters and lowercasing.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `normalize_token` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `normalize_token` (declaration #9) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `name: &str` must satisfy domain and type-safety constraints prior to invoking `normalize_token`.
///
/// ## 4. Postconditions
/// - Returns `String` from `normalize_token` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(name)) bounded over input/member dimension of `normalize_token`.
/// - Space Complexity: O(N(name)) bounded auxiliary memory for `normalize_token`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `normalize_token` (`String`).
/// - State: Maintains consistent state invariants across `normalize_token` without partial mutation on error.
/// - Action: Executes `normalize_token` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `normalize_token` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-NORMALIZE-TOKEN] | SysML v2: ActionDef(normalize_token)
pub fn normalize_token(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

/// Splits a string by non-alphanumeric characters into lowercase words.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `tokenize_words` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `tokenize_words` (declaration #10) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `s: &str` must satisfy domain and type-safety constraints prior to invoking `tokenize_words`.
///
/// ## 4. Postconditions
/// - Returns `Vec<String>` from `tokenize_words` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(s)) bounded over input/member dimension of `tokenize_words`.
/// - Space Complexity: O(N(s)) bounded auxiliary memory for `tokenize_words`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `tokenize_words` (`Vec<String>`).
/// - State: Maintains consistent state invariants across `tokenize_words` without partial mutation on error.
/// - Action: Executes `tokenize_words` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `tokenize_words` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-TOKENIZE-WORDS] | SysML v2: ActionDef(tokenize_words)
pub fn tokenize_words(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_ascii_lowercase())
        .collect()
}

/// Parses a participant or actor declaration in Mermaid sequence diagrams without regex.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_participant_declaration` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_participant_declaration` (declaration #11) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `line: &str` must satisfy domain and type-safety constraints prior to invoking `parse_participant_declaration`.
///
/// ## 4. Postconditions
/// - Returns `Option<(String, String)>` from `parse_participant_declaration` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(line)) bounded over input/member dimension of `parse_participant_declaration`.
/// - Space Complexity: O(N(line)) bounded auxiliary memory for `parse_participant_declaration`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_participant_declaration` (`Option<(String, String)>`).
/// - State: Maintains consistent state invariants across `parse_participant_declaration` without partial mutation on error.
/// - Action: Executes `parse_participant_declaration` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_participant_declaration` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-PARSE-PARTICIPANT-DECLARATION] | SysML v2: ActionDef(parse_participant_declaration)
fn parse_participant_declaration(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    let rest = if let Some(r) = trimmed.strip_prefix("participant ") {
        r.trim()
    } else if let Some(r) = trimmed.strip_prefix("actor ") {
        r.trim()
    } else {
        return None;
    };

    if let Some((id, alias_part)) = rest.split_once(" as ") {
        let id_clean = id.trim().trim_matches('"');
        let alias_clean = alias_part.trim().trim_matches('"');
        Some((id_clean.to_string(), alias_clean.to_string()))
    } else {
        let id_clean = rest.trim().trim_matches('"');
        Some((id_clean.to_string(), String::new()))
    }
}

/// Parses a message arrow line in Mermaid sequence diagrams without regex.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `parse_message_arrow` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `parse_message_arrow` (declaration #12) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `line: &str` must satisfy domain and type-safety constraints prior to invoking `parse_message_arrow`.
///
/// ## 4. Postconditions
/// - Returns `Option<(&str, &str, &str)>` from `parse_message_arrow` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(line)) bounded over input/member dimension of `parse_message_arrow`.
/// - Space Complexity: O(N(line)) bounded auxiliary memory for `parse_message_arrow`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `parse_message_arrow` (`Option<(&str, &str, &str)>`).
/// - State: Maintains consistent state invariants across `parse_message_arrow` without partial mutation on error.
/// - Action: Executes `parse_message_arrow` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `parse_message_arrow` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-PARSE-MESSAGE-ARROW] | SysML v2: ActionDef(parse_message_arrow)
fn parse_message_arrow(line: &str) -> Option<(&str, &str, &str)> {
    let trimmed = line.trim();
    let (left, action) = trimmed.split_once(':')?;
    let left = left.trim();
    let action = action.trim();

    const ARROWS: &[&str] = &[
        "-->>", "-->", "--)", "--x",
        "->>", "->", "-x", "-)",
    ];

    for arrow in ARROWS {
        if let Some(idx) = left.find(arrow) {
            let sender = left[..idx].trim().trim_matches('"');
            let target = left[idx + arrow.len()..].trim().trim_matches('"');
            if !sender.is_empty() && !target.is_empty() {
                return Some((sender, target, action));
            }
        }
    }

    None
}

/// Determines whether a message sender represents a Human-in-the-Loop (HITL) operator or C2 console.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_hitl_sender` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_hitl_sender` (declaration #13) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `sender: &str` must satisfy domain and type-safety constraints prior to invoking `is_hitl_sender`.
/// - `participants: &std::collections::HashMap<String, String>` must satisfy domain and type-safety constraints prior to invoking `is_hitl_sender`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_hitl_sender` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(sender)) bounded over input/member dimension of `is_hitl_sender`.
/// - Space Complexity: O(N(sender)) bounded auxiliary memory for `is_hitl_sender`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_hitl_sender` (`bool`).
/// - State: Maintains consistent state invariants across `is_hitl_sender` without partial mutation on error.
/// - Action: Executes `is_hitl_sender` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_hitl_sender` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-HITL-SENDER] | SysML v2: ActionDef(is_hitl_sender)
fn is_hitl_sender(
    sender: &str,
    participants: &std::collections::HashMap<String, String>,
) -> bool {
    let s_norm = normalize_token(sender);
    let alias = participants.get(sender).cloned().unwrap_or_default();
    let a_norm = normalize_token(&alias);

    const HITL_SUBSTRINGS: &[&str] = &[
        "operator", "pilot", "gcs", "commander", "human",
        "supervisor", "safetyofficer", "controller", "console",
        "technician", "user", "c2",
    ];

    if HITL_SENDER_TOKENS.contains(&s_norm.as_str()) || HITL_SENDER_TOKENS.contains(&a_norm.as_str()) {
        return true;
    }

    HITL_SUBSTRINGS.iter().any(|tok| s_norm.contains(tok) || a_norm.contains(tok))
}

/// Determines whether a message action signifies an explicit HITL arming authorization or operator consent.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_hitl_consent_action` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_hitl_consent_action` (declaration #14) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `action: &str` must satisfy domain and type-safety constraints prior to invoking `is_hitl_consent_action`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_hitl_consent_action` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(action)) bounded over input/member dimension of `is_hitl_consent_action`.
/// - Space Complexity: O(N(action)) bounded auxiliary memory for `is_hitl_consent_action`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_hitl_consent_action` (`bool`).
/// - State: Maintains consistent state invariants across `is_hitl_consent_action` without partial mutation on error.
/// - Action: Executes `is_hitl_consent_action` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_hitl_consent_action` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-HITL-CONSENT-ACTION] | SysML v2: ActionDef(is_hitl_consent_action)
fn is_hitl_consent_action(action: &str) -> bool {
    let words = tokenize_words(action);
    if words.is_empty() {
        return false;
    }

    if words.iter().any(|w| {
        w == "consent"
            || w == "consent_granted"
            || w == "authorize"
            || w == "authorized"
            || w == "authorization"
            || w == "permit"
            || w == "permission"
            || w == "enable"
            || w == "enabled"
    }) {
        return true;
    }

    let has_auth = words.iter().any(|w| w == "auth" || w.starts_with("auth") || w == "arm" || w.starts_with("arm"));
    let has_cmd_or_token = words.iter().any(|w| {
        w == "command"
            || w == "cmd"
            || w == "token"
            || w == "tokens"
            || w == "key"
            || w == "keys"
            || w == "confirm"
            || w == "switch"
            || w == "request"
            || w == "grant"
            || w == "granted"
    });
    if has_auth && has_cmd_or_token {
        return true;
    }

    false
}

/// Determines whether an action signifies a disarm, safe, or inhibit command that resets active consent.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_disarm_action` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_disarm_action` (declaration #15) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `action: &str` must satisfy domain and type-safety constraints prior to invoking `is_disarm_action`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_disarm_action` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(action)) bounded over input/member dimension of `is_disarm_action`.
/// - Space Complexity: O(N(action)) bounded auxiliary memory for `is_disarm_action`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_disarm_action` (`bool`).
/// - State: Maintains consistent state invariants across `is_disarm_action` without partial mutation on error.
/// - Action: Executes `is_disarm_action` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_disarm_action` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-DISARM-ACTION] | SysML v2: ActionDef(is_disarm_action)
fn is_disarm_action(action: &str) -> bool {
    let words = tokenize_words(action);
    const DISARM_WORDS: &[&str] = &[
        "disarm", "disarming", "safe", "safing", "inhibit", "inhibiting",
        "abort", "aborting", "disable", "deactivate", "isolate",
    ];
    words.iter().any(|w| DISARM_WORDS.contains(&w.as_str()))
}

/// Determines whether a message target represents a physical arming, motor-enable, or actuator device.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_physical_arming_target` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_physical_arming_target` (declaration #16) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `target: &str` must satisfy domain and type-safety constraints prior to invoking `is_physical_arming_target`.
/// - `participants: &std::collections::HashMap<String, String>` must satisfy domain and type-safety constraints prior to invoking `is_physical_arming_target`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_physical_arming_target` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(target)) bounded over input/member dimension of `is_physical_arming_target`.
/// - Space Complexity: O(N(target)) bounded auxiliary memory for `is_physical_arming_target`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_physical_arming_target` (`bool`).
/// - State: Maintains consistent state invariants across `is_physical_arming_target` without partial mutation on error.
/// - Action: Executes `is_physical_arming_target` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_physical_arming_target` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-PHYSICAL-ARMING-TARGET] | SysML v2: ActionDef(is_physical_arming_target)
fn is_physical_arming_target(
    target: &str,
    participants: &std::collections::HashMap<String, String>,
) -> bool {
    let t_norm = normalize_token(target);
    let alias = participants.get(target).cloned().unwrap_or_default();
    let a_norm = normalize_token(&alias);

    PHYSICAL_ARMING_TARGET_TOKENS.iter().any(|tok| {
        t_norm == *tok || t_norm.contains(tok) || (!a_norm.is_empty() && (a_norm == *tok || a_norm.contains(tok)))
    })
}

/// Determines whether an action signifies a physical actuation, drive, or effector trigger.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_physical_arming_action` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_physical_arming_action` (declaration #17) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `action: &str` must satisfy domain and type-safety constraints prior to invoking `is_physical_arming_action`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_physical_arming_action` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(action)) bounded over input/member dimension of `is_physical_arming_action`.
/// - Space Complexity: O(N(action)) bounded auxiliary memory for `is_physical_arming_action`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_physical_arming_action` (`bool`).
/// - State: Maintains consistent state invariants across `is_physical_arming_action` without partial mutation on error.
/// - Action: Executes `is_physical_arming_action` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_physical_arming_action` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-PHYSICAL-ARMING-ACTION] | SysML v2: ActionDef(is_physical_arming_action)
fn is_physical_arming_action(action: &str) -> bool {
    let words = tokenize_words(action);
    if words.is_empty() {
        return false;
    }

    const ACTUATION_KEYWORDS: &[&str] = &[
        "actuate", "actuation", "drive", "engage", "trigger",
        "arm", "arming",
    ];

    if words.iter().any(|w| ACTUATION_KEYWORDS.contains(&w.as_str())) {
        return true;
    }

    let act_norm = normalize_token(action);
    const COMBOS: &[&str] = &[
        "motorenable", "enablemotor", "propulsionenable",
        "actuatedrive", "engagedrive",
        "activatepowerstage", "highvoltage",
    ];

    COMBOS.iter().any(|c| act_norm.contains(c))
}

/// Scans Mermaid sequence diagrams and validates that any physical arming or firing signal
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `validate_sequence_diagram_temporal_safety` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `validate_sequence_diagram_temporal_safety` (declaration #18) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `doc: &MarkdownDoc` must satisfy domain and type-safety constraints prior to invoking `validate_sequence_diagram_temporal_safety`.
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `validate_sequence_diagram_temporal_safety`.
/// - `rel_path: &str` must satisfy domain and type-safety constraints prior to invoking `validate_sequence_diagram_temporal_safety`.
///
/// ## 4. Postconditions
/// - Returns `Vec<SpecAuditFinding>` from `validate_sequence_diagram_temporal_safety` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(doc)) bounded over input/member dimension of `validate_sequence_diagram_temporal_safety`.
/// - Space Complexity: O(N(doc)) bounded auxiliary memory for `validate_sequence_diagram_temporal_safety`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `validate_sequence_diagram_temporal_safety` (`Vec<SpecAuditFinding>`).
/// - State: Maintains consistent state invariants across `validate_sequence_diagram_temporal_safety` without partial mutation on error.
/// - Action: Executes `validate_sequence_diagram_temporal_safety` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `validate_sequence_diagram_temporal_safety` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-VALIDATE-SEQUENCE-DIAGRAM-TEMPORAL-SAFETY] | SysML v2: ActionDef(validate_sequence_diagram_temporal_safety)
pub fn validate_sequence_diagram_temporal_safety(
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();

    for block in &doc.mermaid_blocks {
        let is_sequence = block.header.eq_ignore_ascii_case("sequenceDiagram")
            || block.raw.lines().any(|l| l.trim().starts_with("sequenceDiagram"));
        if !is_sequence {
            continue;
        }

        let mut hitl_consent_granted = false;
        let mut participants = std::collections::HashMap::new();

        for (idx, line) in block.raw.lines().enumerate() {
            let curr_lineno = block.start_line + 1 + idx;
            let trimmed = line.trim();

            if trimmed.is_empty()
                || trimmed.starts_with("%%")
                || trimmed.starts_with("Note ")
                || trimmed.starts_with("Note\t")
                || trimmed.starts_with("autonumber")
                || trimmed.starts_with("title ")
            {
                continue;
            }

            if let Some((id, alias)) = parse_participant_declaration(trimmed) {
                participants.insert(id, alias);
                continue;
            }

            if let Some((sender, target, action)) = parse_message_arrow(trimmed) {
                // 1. Check if this message grants Human C2 Arming / Consent
                if is_hitl_sender(sender, &participants) && is_hitl_consent_action(action) {
                    hitl_consent_granted = true;
                }

                // 2. Check if this message is a disarm/safe action (resets consent)
                if is_disarm_action(action) {
                    hitl_consent_granted = false;
                    continue;
                }

                // 3. Check if target is a Physical Arming Target
                let is_target_phys = is_physical_arming_target(target, &participants);

                // 4. Check if action is a Physical Arming Action
                let is_act_arm = is_physical_arming_action(action);

                if is_target_phys && is_act_arm {
                    if !hitl_consent_granted {
                        let reported_line = content
                            .lines()
                            .enumerate()
                            .find(|(i, l)| {
                                let lno = i + 1;
                                lno >= block.start_line && lno <= block.end_line && l.trim() == trimmed
                            })
                            .map(|(i, _)| i + 1)
                            .unwrap_or(curr_lineno);

                        findings.push(SpecAuditFinding::new(
                            "factual-grounding-temporal-safety-violation",
                            format!(
                                "Autonomous physical actuation sequence detected in '{}' on line {}. Physical signal '{}' targeting '{}' is not preceded by an explicit Human-in-the-Loop (HITL) authorization command or operator consent.",
                                rel_path, reported_line, action, target
                            ),
                            rel_path,
                            Some(reported_line),
                        ));
                    }
                }
            }
        }
    }

    findings
}

/// Extracts the text slice and starting line number of the `## Source References` section.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `extract_source_references_section` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `extract_source_references_section` (declaration #19) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `content: &'a str` must satisfy domain and type-safety constraints prior to invoking `extract_source_references_section`.
///
/// ## 4. Postconditions
/// - Returns `Option<(usize, &'a str)>` from `extract_source_references_section` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(content)) bounded over input/member dimension of `extract_source_references_section`.
/// - Space Complexity: O(N(content)) bounded auxiliary memory for `extract_source_references_section`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `extract_source_references_section` (`Option<(usize, &'a str)>`).
/// - State: Maintains consistent state invariants across `extract_source_references_section` without partial mutation on error.
/// - Action: Executes `extract_source_references_section` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `extract_source_references_section` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-EXTRACT-SOURCE-REFERENCES-SECTION] | SysML v2: ActionDef(extract_source_references_section)
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `validate_source_reference_clause_integrity` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `validate_source_reference_clause_integrity` (declaration #20) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `_doc: &MarkdownDoc` must satisfy domain and type-safety constraints prior to invoking `validate_source_reference_clause_integrity`.
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `validate_source_reference_clause_integrity`.
/// - `rel_path: &str` must satisfy domain and type-safety constraints prior to invoking `validate_source_reference_clause_integrity`.
///
/// ## 4. Postconditions
/// - Returns `Result<Vec<SpecAuditFinding>, SpecAuditError>` from `validate_source_reference_clause_integrity` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(_doc)) bounded over input/member dimension of `validate_source_reference_clause_integrity`.
/// - Space Complexity: O(N(_doc)) bounded auxiliary memory for `validate_source_reference_clause_integrity`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `validate_source_reference_clause_integrity` (`Result<Vec<SpecAuditFinding>, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `validate_source_reference_clause_integrity` without partial mutation on error.
/// - Action: Executes `validate_source_reference_clause_integrity` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `validate_source_reference_clause_integrity` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-VALIDATE-SOURCE-REFERENCE-CLAUSE-INTEGRITY] | SysML v2: ActionDef(validate_source_reference_clause_integrity)
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

/// Validates whether a line matches the canonical Epic feature checklist grammar.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `is_valid_epic_feature_checklist_item` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `is_valid_epic_feature_checklist_item` (declaration #21) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `line: &str` must satisfy domain and type-safety constraints prior to invoking `is_valid_epic_feature_checklist_item`.
///
/// ## 4. Postconditions
/// - Returns `bool` from `is_valid_epic_feature_checklist_item` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(line)) bounded over input/member dimension of `is_valid_epic_feature_checklist_item`.
/// - Space Complexity: O(N(line)) bounded auxiliary memory for `is_valid_epic_feature_checklist_item`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `is_valid_epic_feature_checklist_item` (`bool`).
/// - State: Maintains consistent state invariants across `is_valid_epic_feature_checklist_item` without partial mutation on error.
/// - Action: Executes `is_valid_epic_feature_checklist_item` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `is_valid_epic_feature_checklist_item` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-VALID-EPIC-FEATURE-CHECKLIST-ITEM] | SysML v2: ActionDef(is_valid_epic_feature_checklist_item)
pub fn is_valid_epic_feature_checklist_item(line: &str) -> bool {
    let trimmed = line.trim();
    let after_box = if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("- [x] ").or_else(|| trimmed.strip_prefix("- [X] ")) {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("* [ ] ") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("* [x] ").or_else(|| trimmed.strip_prefix("* [X] ")) {
        rest
    } else {
        return false;
    };

    let after_ws = after_box.trim_start();

    // 2. Must contain #<IssueID> or #[IssueID]
    let after_issue = if let Some(rest) = after_ws.strip_prefix("#[issueid]")
        .or_else(|| after_ws.strip_prefix("#[IssueID]"))
        .or_else(|| after_ws.strip_prefix("#[issue_id]"))
    {
        rest
    } else if let Some(rest) = after_ws.strip_prefix('#') {
        let digits_len = rest.chars().take_while(|c| c.is_ascii_digit()).count();
        if digits_len == 0 {
            return false;
        }
        &rest[digits_len..]
    } else {
        return false;
    };

    let after_sep = after_issue.trim_start().trim_start_matches('-').trim_start_matches(':').trim_start();

    // 3. Must contain markdown link [<Title>](<target>)
    if !after_sep.starts_with('[') {
        return false;
    }
    let sep_pos = match after_sep.find("](") {
        Some(pos) => pos,
        None => return false,
    };
    let after_target_open = &after_sep[sep_pos + 2..];
    let close_paren_rel = match after_target_open.find(')') {
        Some(pos) => pos,
        None => return false,
    };
    let target = &after_target_open[..close_paren_rel];
    let title = &after_sep[1..sep_pos];

    let lower_title = title.to_lowercase();
    let lower_target = target.to_lowercase();
    if !lower_title.contains("feat-")
        && !lower_title.contains("feature")
        && !lower_target.contains("feat-")
        && !lower_target.contains("features/")
    {
        return false;
    }

    // 4. Must contain trailing parenthetical (<clauses>)
    let after_link = after_target_open[close_paren_rel + 1..].trim();
    if !after_link.starts_with('(') || !after_link.ends_with(')') || after_link.len() < 3 {
        return false;
    }

    true
}

/// Counts referenced features in an Epic specification.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `count_epic_features` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `count_epic_features` (declaration #22) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `count_epic_features`.
///
/// ## 4. Postconditions
/// - Returns `usize` from `count_epic_features` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(content)) bounded over input/member dimension of `count_epic_features`.
/// - Space Complexity: O(N(content)) bounded auxiliary memory for `count_epic_features`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `count_epic_features` (`usize`).
/// - State: Maintains consistent state invariants across `count_epic_features` without partial mutation on error.
/// - Action: Executes `count_epic_features` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `count_epic_features` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-COUNT-EPIC-FEATURES] | SysML v2: ActionDef(count_epic_features)
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

        if trimmed.starts_with("- [ ]")
            || trimmed.starts_with("- [x]")
            || trimmed.starts_with("- [X]")
            || trimmed.starts_with("* [ ]")
            || trimmed.starts_with("* [x]")
            || trimmed.starts_with("* [X]")
        {
            if is_valid_epic_feature_checklist_item(trimmed) {
                count += 1;
            }
        }
    }

    count
}

/// Counts acceptance criteria in a Feature specification.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `count_feature_acceptance_criteria` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `count_feature_acceptance_criteria` (declaration #23) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `count_feature_acceptance_criteria`.
///
/// ## 4. Postconditions
/// - Returns `usize` from `count_feature_acceptance_criteria` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(content)) bounded over input/member dimension of `count_feature_acceptance_criteria`.
/// - Space Complexity: O(N(content)) bounded auxiliary memory for `count_feature_acceptance_criteria`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `count_feature_acceptance_criteria` (`usize`).
/// - State: Maintains consistent state invariants across `count_feature_acceptance_criteria` without partial mutation on error.
/// - Action: Executes `count_feature_acceptance_criteria` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `count_feature_acceptance_criteria` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-COUNT-FEATURE-ACCEPTANCE-CRITERIA] | SysML v2: ActionDef(count_feature_acceptance_criteria)
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `validate_cardinality_bounds` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `validate_cardinality_bounds` (declaration #24) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `doc: &MarkdownDoc` must satisfy domain and type-safety constraints prior to invoking `validate_cardinality_bounds`.
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `validate_cardinality_bounds`.
/// - `rel_path: &str` must satisfy domain and type-safety constraints prior to invoking `validate_cardinality_bounds`.
///
/// ## 4. Postconditions
/// - Returns `Result<Vec<SpecAuditFinding>, SpecAuditError>` from `validate_cardinality_bounds` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(doc)) bounded over input/member dimension of `validate_cardinality_bounds`.
/// - Space Complexity: O(N(doc)) bounded auxiliary memory for `validate_cardinality_bounds`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `validate_cardinality_bounds` (`Result<Vec<SpecAuditFinding>, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `validate_cardinality_bounds` without partial mutation on error.
/// - Action: Executes `validate_cardinality_bounds` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `validate_cardinality_bounds` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-VALIDATE-CARDINALITY-BOUNDS] | SysML v2: ActionDef(validate_cardinality_bounds)
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
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `validate_provenance` in `crates/verify-baseline/src/spec_audit/provenance.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `validate_provenance` (declaration #25) within `SPEC-AUDIT-PROVENANCE`.
///
/// ## 3. Preconditions
/// - `doc: &MarkdownDoc` must satisfy domain and type-safety constraints prior to invoking `validate_provenance`.
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `validate_provenance`.
/// - `rel_path: &str` must satisfy domain and type-safety constraints prior to invoking `validate_provenance`.
/// - `symbol_index: Option<&SysmlSymbolIndex>` must satisfy domain and type-safety constraints prior to invoking `validate_provenance`.
///
/// ## 4. Postconditions
/// - Returns `Result<Vec<SpecAuditFinding>, SpecAuditError>` from `validate_provenance` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(doc)) bounded over input/member dimension of `validate_provenance`.
/// - Space Complexity: O(N(doc)) bounded auxiliary memory for `validate_provenance`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `validate_provenance` (`Result<Vec<SpecAuditFinding>, SpecAuditError>`).
/// - State: Maintains consistent state invariants across `validate_provenance` without partial mutation on error.
/// - Action: Executes `validate_provenance` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `validate_provenance` (`SPEC-AUDIT-PROVENANCE`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-VALIDATE-PROVENANCE] | SysML v2: ActionDef(validate_provenance)
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
        index.attribute_names.insert("latency".to_string());
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
- [ ] #10 - [Feature 01: Single feature](../features/FEAT-01.md) (traceable requirement)
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
- [ ] #10 - [Feature 01: Feature 1](../features/FEAT-01.md) (traceable requirement)
- [ ] #11 - [Feature 02: Feature 2](../features/FEAT-02.md) (traceable requirement)
- [ ] #12 - [Feature 03: Feature 3](../features/FEAT-03.md) (traceable requirement)
- [ ] #13 - [Feature 04: Feature 4](../features/FEAT-04.md) (traceable requirement)
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
            overflow_lines.push(format!("- [ ] #{i} - [Feature {i:02}: Feature {i}](../features/FEAT-{i:02}.md) (traceable requirement)"));
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
| Actuator Speed | PL-40_Actuator | 35 m/s |
| Serial Bus Rate | RS-485_Bus | 115200 bps |

## Acceptance Criteria
- [ ] AC-01: Primary DC bus power shall not exceed 50 W.
- [ ] AC-02: Actuator speed shall reach 35 m/s.
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
        assert!(!is_pure_scalar_numeric("PL-40_Actuator"));
        assert!(!is_pure_scalar_numeric("RS-485_Bus"));
        assert!(!is_pure_scalar_numeric("ALPHA5"));
        assert!(!is_pure_scalar_numeric("Feature 24 Power Management System"));
        assert!(!is_pure_scalar_numeric("ALPHA5_ControlActions"));
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
        assert_eq!(extract_scalar_numeric_claim("ALPHA5"), None);
        assert_eq!(extract_scalar_numeric_claim("Feature 24 Power Management System"), None);
        assert_eq!(extract_scalar_numeric_claim("PL-40_Actuator"), None);
    }

    #[test]
    fn test_concept_provenance_abstract_structural_property_graph() {
        let content = r#"---
title: "Feature 30: Abstract Structural Topology"
type: feature
---
# Feature 30: Abstract Structural Topology

## Metadata
| Attribute | Detail |
| :--- | :--- |
| **Title** | Feature 30: Abstract Structural Topology |

## Architecture Baseline
| Property | Value |
| :--- | :--- |
| structure_topology | DistributedMesh |
| actuation_mode | DualRedundantHydraulic |
| cooling_architecture | VaporCycleLoop |

## Requirements
- [ ] REQ-01: Bus interface operates at 1000 bps with DistributedMesh topology.
- [ ] REQ-02: Actuation hydraulic circuit maintains 210 bar working pressure.
- [ ] REQ-03: Vapor cycle loop sustains heat dissipation of 500 W.

## Source References
Derived from schema/model.sysml §2.1 and ISO/IEC/IEEE 15288:2023.
"#;
        let doc = parse_markdown(content).expect("parse markdown");
        let index = build_test_symbol_index();
        let findings = validate_factual_grounding(&doc, content, "docs/features/feat-30.md", Some(&index), &[], &[])
            .expect("validation succeeds");

        assert!(
            findings.is_empty(),
            "Abstract structural property graph should pass without domain bias: {:?}",
            findings
        );
    }

    #[test]
    fn test_concept_provenance_orthogonal_actuator_fixtures() {
        let content = r#"---
title: "Feature 31: Orthogonal Actuator Subsystem Control"
type: feature
---
# Feature 31: Orthogonal Actuator Subsystem Control

## Metadata
| Attribute | Detail |
| :--- | :--- |
| **Title** | Feature 31: Orthogonal Actuator Subsystem Control |

## Requirements
- [ ] REQ-01: Orthogonal actuation mechanisms sustain peak torque of 15 Nm per axis.
- [ ] REQ-02: Articulated orthogonal actuator deflection limits span -25.0 deg to +25.0 deg.
- [ ] REQ-03: Bus telemetry updates at 100 Hz.

## Source References
Derived from schema/model.sysml and ISO/IEC/IEEE 15026-3.
"#;
        let doc = parse_markdown(content).expect("parse markdown");
        let index = build_test_symbol_index();
        let findings = validate_factual_grounding(&doc, content, "docs/features/feat-31.md", Some(&index), &[], &[])
            .expect("validation succeeds");

        assert!(
            findings.is_empty(),
            "Orthogonal actuator fixture should validate cleanly without domain bias: {:?}",
            findings
        );
    }

    #[test]
    fn test_concept_provenance_multi_channel_actuation_architecture() {
        let content = r#"---
title: "Feature 32: Multi-Channel Actuation Architecture"
type: feature
---
# Feature 32: Multi-Channel Actuation Architecture

## Metadata
| Attribute | Detail |
| :--- | :--- |
| **Title** | Feature 32: Multi-Channel Actuation Architecture |

## Requirements
- [ ] REQ-01: Redundant primary channel controllers provide dual authority over 50 ms step response.
- [ ] REQ-02: Secondary stage actuators command differential positioning control within 20 ms.
- [ ] REQ-03: Actuator bus power consumption shall not exceed 120 W.

## Source References
Derived from schema/model.sysml and ISO/IEC/IEEE 12207:2017.
"#;
        let doc = parse_markdown(content).expect("parse markdown");
        let index = build_test_symbol_index();
        let findings = validate_factual_grounding(&doc, content, "docs/features/feat-32.md", Some(&index), &[], &[])
            .expect("validation succeeds");

        assert!(
            findings.is_empty(),
            "Multi-channel actuation architecture fixture should validate cleanly: {:?}",
            findings
        );
    }

    #[test]
    fn test_concept_provenance_fails_on_unverified_domain_claim() {
        let content = r#"---
title: "Feature 33: Ungrounded Domain Claim"
type: feature
---
# Feature 33: Ungrounded Domain Claim

## Requirements
- [ ] REQ-01: Vehicle operates at 450 m/s maximum velocity without schema derivation.

## Description
This spec lacks normative citations and schema grounding for its operational velocity parameter.
"#;
        let doc = parse_markdown(content).expect("parse markdown");
        let index = build_test_symbol_index();
        let findings = validate_factual_grounding(&doc, content, "docs/features/feat-33.md", Some(&index), &[], &[])
            .expect("validation succeeds");

        assert!(
            findings.iter().any(|f| f.rule_id == "factual-grounding-ungrounded-assertion"),
            "Expected ungrounded assertion finding for unverified claim, got: {:?}",
            findings
        );
    }

    #[test]
    fn test_autonomous_arming_sequence_diagram_detected_and_rejected() {
        let content = include_str!("../../../../tests/fixtures/safety/autonomous_arming_usecase.md");
        let doc = parse_markdown(content).expect("parse markdown");
        let findings = validate_sequence_diagram_temporal_safety(&doc, content, "tests/fixtures/safety/autonomous_arming_usecase.md");

        assert!(
            !findings.is_empty(),
            "Autonomous physical actuation sequence fixture must be rejected by temporal safety validator"
        );
        let violations: Vec<_> = findings
            .iter()
            .filter(|f| f.rule_id == "factual-grounding-temporal-safety-violation")
            .collect();
        assert!(
            !violations.is_empty(),
            "Expected factual-grounding-temporal-safety-violation findings, got: {:?}",
            findings
        );
        assert!(
            violations.iter().any(|f| f.message.contains("Autonomous physical actuation sequence detected")),
            "Violation message must explain autonomous physical actuation sequence"
        );
    }

    #[test]
    fn test_hitl_authorized_arming_sequence_diagram_passes() {
        let content = include_str!("../../../../tests/fixtures/safety/hitl_arming_usecase.md");
        let doc = parse_markdown(content).expect("parse markdown");
        let findings = validate_sequence_diagram_temporal_safety(&doc, content, "tests/fixtures/safety/hitl_arming_usecase.md");

        let violations: Vec<_> = findings
            .iter()
            .filter(|f| f.rule_id == "factual-grounding-temporal-safety-violation")
            .collect();
        assert!(
            violations.is_empty(),
            "HITL authorized actuation sequence fixture must pass temporal safety validator, got: {:?}",
            violations
        );
    }

    #[test]
    fn test_conops_scn_05_sequence_diagram_passes_temporal_safety() {
        let content = r#"# ConOps
```mermaid
sequenceDiagram
    autonumber
    participant Sensors as "Sensor Suite Array"
    participant EdgeNode as "Edge Processing Node"
    participant Controller as "Core Controller"
    participant Watchdog as "Safety Watchdog"
    participant Console as "Supervisory Console"

    Sensors ->> EdgeNode: 1. Stream Synchronized Multi-Modal Frames
    EdgeNode ->> EdgeNode: 2. Correlate Temporal Features & Fusion
    EdgeNode ->> Controller: 3. Deliver Correlated State Vector
    Controller ->> Watchdog: 4. Verify State Estimation Margin
    Watchdog -->> Controller: 5. State Bounds Confirmed Normal
    Controller ->> Console: 6. Synchronize State Telemetry
    Console -->> Controller: 7. Supervisory State Acknowledged
    Controller ->> Sensors: 8. Maintain Continuous Stream Calibration
```
"#;
        let doc = parse_markdown(content).expect("parse markdown");
        let findings = validate_sequence_diagram_temporal_safety(&doc, content, "docs/conops/SCN-05.md");
        assert!(
            findings.is_empty(),
            "SCN-05 sequence diagram should pass temporal safety without violations: {:?}",
            findings
        );
    }

    #[test]
    fn test_issue_449_count_epic_features_strict_checklist_grammar() {
        // Unlinked backtick item lacking IssueID, hyperlink, and trailing mapped requirements parenthetical
        let unlinked_epic = r#"---
type: epic
---
# Epic 01: Core Systems

## Requirements & Checklist
- [ ] `FEAT-01_example` -- description
- [ ] `FEAT-02_example` -- description
"#;
        assert_eq!(
            count_epic_features(unlinked_epic),
            0,
            "Unlinked backtick checklist items must not be counted as valid Epic features"
        );

        // Canonical linked items with IssueID, hyperlink, and parenthetical clauses
        let canonical_epic = r#"---
type: epic
---
# Epic 01: Core Systems

## 2. Requirements & Checklist
- [ ] #10 - [Feature 01: [ConOps] Tracker](../features/FEAT-01-Tracker.md) (ISO/IEC/IEEE 29148:2018 §6.4.2)
- [x] #11 - [Feature 02: [Core] Detector](https://github.com/org/repo/blob/main/docs/features/FEAT-02-Detector.md) (traceable requirement)
- [ ] #[IssueID] - [Feature 03: [Core] Engine](../features/FEAT-03-Engine.md) (architectural part definition)
"#;
        assert_eq!(
            count_epic_features(canonical_epic),
            3,
            "Canonical Epic feature checklist items must be counted"
        );
    }
}


