//! Native Rust Provenance & Factual Grounding Validator.
//!
//! Conforms to DO-178C Level A and ISO 26262 ASIL D safety standards.

#[cfg(test)]
use crate::spec_audit::markdown_ast::parse_markdown;
use crate::spec_audit::markdown_ast::MarkdownDoc;
use crate::spec_audit::uml_validator::SysmlSymbolIndex;
use crate::spec_audit::{SpecAuditError, SpecAuditFinding};

pub use crate::spec_audit::provenance_grounding::{
    find_naked_numeric_quantities, is_assertion_grounded, validate_factual_grounding,
    validate_factual_grounding_content,
};
pub use crate::spec_audit::provenance_refs::{
    count_epic_features, count_feature_acceptance_criteria, extract_source_references_section,
    is_valid_epic_feature_checklist_item, validate_cardinality_bounds,
    validate_source_reference_clause_integrity,
};
pub use crate::spec_audit::provenance_temporal::validate_sequence_diagram_temporal_safety;

/// Recognized physical and engineering units across frequencies, timing, electrical, rates, dimensions, and quantities.
pub const RECOGNIZED_UNITS: &[&str] = &[
    "hz", "khz", "mhz", "ghz", "s", "sec", "second", "seconds", "ms", "us", "µs", "ns", "min",
    "minute", "minutes", "hr", "hour", "hours", "h", "v", "mv", "kv", "a", "ma", "ua", "µa", "w",
    "kw", "mw", "j", "kj", "mj", "kwh", "mah", "ah", "ohm", "kohm", "mohm", "f", "uf", "µf", "nf",
    "pf", "bps", "kbps", "mbps", "gbps", "b/s", "kb/s", "mb/s", "gb/s", "bytes/s", "baud", "m/s",
    "km/h", "deg/s", "rad/s", "m/s^2", "m/s2", "fps", "rpm", "m", "cm", "mm", "km", "nm", "um",
    "µm", "deg", "rad", "degree", "degrees", "radian", "radians", "kg", "g", "mg", "c", "°c", "k",
    "%", "percent", "db", "dbm", "dbi", "ppm", "ratio", "bar", "mbar", "pa", "kpa", "mpa", "psi",
    "n", "kn", "b", "kb", "mb", "gb", "tb", "byte", "bytes",
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
    "iso", "iec", "ieee", "rtca", "do-178", "do-178c", "do-254", "do-331", "mil-std", "stanag",
    "rfc", "arinc", "nist", "ecss", "astm", "sae", "jarus", "sora",
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
    "actuator",
    "motor",
    "safetyswitch",
    "drive",
    "powerstage",
    "highvoltage",
    "emitter",
    "propulsionenable",
    "effector",
    "valve",
    "pump",
];

/// Human C2 / HITL participant tokens in sequence diagrams.
pub const HITL_SENDER_TOKENS: &[&str] = &[
    "operator",
    "operators",
    "pilot",
    "pilots",
    "human",
    "commander",
    "supervisor",
    "controller",
    "technician",
    "safetyofficer",
    "user",
    "c2",
    "gcsoperator",
    "gcs",
    "groundcontrolstation",
    "groundstation",
    "console",
];

/// Checks if a token represents a recognized physical or engineering unit.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-RECOGNIZED-UNIT]
pub fn is_recognized_unit(token: &str) -> bool {
    let lower = token.trim().to_lowercase();
    let clean = lower.trim_end_matches(&[',', '.', ';', ':', ')', ']', '}'][..]);
    RECOGNIZED_UNITS.contains(&clean)
}

/// Validates whether a token or table cell is a pure scalar numeric value.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-IS-PURE-SCALAR-NUMERIC]
pub fn is_pure_scalar_numeric(val: &str) -> bool {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return false;
    }

    let clean = trimmed.trim_matches(|c| matches!(c, '*' | '`' | '_' | '"' | '\''));
    let bytes = clean.as_bytes();
    if bytes.is_empty() {
        return false;
    }

    let mut idx = 0;
    if clean.starts_with('+') || clean.starts_with('-') {
        idx += 1;
    } else if clean.starts_with('−') {
        idx += '−'.len_utf8();
    }
    if idx >= bytes.len() {
        return false;
    }

    let mut has_digits = false;
    while bytes.get(idx).is_some_and(|b| b.is_ascii_digit()) {
        has_digits = true;
        idx += 1;
    }
    if !has_digits {
        return false;
    }

    if bytes.get(idx) == Some(&b'.') {
        idx += 1;
        while bytes.get(idx).is_some_and(|b| b.is_ascii_digit()) {
            idx += 1;
        }
    }

    if bytes.get(idx).is_some_and(|&b| b == b'e' || b == b'E') {
        idx += 1;
        if bytes.get(idx).is_some_and(|&b| b == b'+' || b == b'-') {
            idx += 1;
        }
        while bytes.get(idx).is_some_and(|b| b.is_ascii_digit()) {
            idx += 1;
        }
    }

    let rest = clean[idx..].trim();
    rest.is_empty() || is_recognized_unit(rest)
}

/// Parses a table cell into a scalar numeric value and optional unit.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-EXTRACT-SCALAR-NUMERIC-CLAIM]
pub fn extract_scalar_numeric_claim(val: &str) -> Option<(f64, Option<String>)> {
    let trimmed = val.trim();
    if !is_pure_scalar_numeric(trimmed) {
        return None;
    }

    let clean = trimmed.trim_matches(|c| matches!(c, '*' | '`' | '_' | '"' | '\''));
    let mut num_end = 0;
    let mut byte_offset = 0;
    for c in clean.chars() {
        let next_offset = byte_offset + c.len_utf8();
        if c.is_ascii_digit() || matches!(c, '.' | '+' | '-' | '−' | 'e' | 'E') {
            num_end = next_offset;
        } else if !c.is_whitespace() {
            break;
        }
        byte_offset = next_offset;
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
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-FIND-ENGINEERING-PARAMETER]
pub fn find_engineering_parameter(line: &str) -> Option<&'static str> {
    let lower = line.to_ascii_lowercase();
    for &param in ENGINEERING_PARAMETER_TERMS {
        if let Some(pos) = lower.find(param) {
            let before_ok = pos == 0
                || lower.as_bytes().get(pos - 1).is_none_or(|&prev_b| {
                    !prev_b.is_ascii_alphanumeric() && prev_b != b'_' && prev_b != b'-'
                });

            let end = pos + param.len();
            let after_ok = end >= lower.len()
                || lower.as_bytes().get(end).is_none_or(|&next_b| {
                    !next_b.is_ascii_alphanumeric() && next_b != b'_' && next_b != b'-'
                });

            if before_ok && after_ok {
                return Some(param);
            }
        }
    }
    None
}

/// Normalizes an identifier or token by filtering alphanumeric characters and lowercasing.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-NORMALIZE-TOKEN]
pub fn normalize_token(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

/// Splits a string by non-alphanumeric characters into lowercase words.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-TOKENIZE-WORDS]
pub fn tokenize_words(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_ascii_lowercase())
        .collect()
}

/// Top-level coordinator that audits provenance, source references, and cardinality bounds for a specification.
/// Realises: [REQ-SPEC-AUDIT-PROVENANCE-VALIDATE-PROVENANCE]
pub fn validate_provenance(
    doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
    symbol_index: Option<&SysmlSymbolIndex>,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let mut all_findings = Vec::new();
    all_findings.extend(validate_factual_grounding(
        doc,
        content,
        rel_path,
        symbol_index,
        (&[], &[]),
    )?);
    all_findings.extend(validate_source_reference_clause_integrity(
        doc, content, rel_path,
    )?);
    all_findings.extend(validate_cardinality_bounds(doc, content, rel_path)?);
    Ok(all_findings)
}

#[cfg(test)]
#[path = "provenance_tests_a.rs"]
mod tests_a;
#[cfg(test)]
#[path = "provenance_tests_b.rs"]
mod tests_b;
