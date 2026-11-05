//! ICD port/connection/signal token extractors, SI unit validators, Markdown table parser, and nested AST port collector.
//!
//! Safety & Traceability: Realises: [REQ-PORT-ID-EXTRACTION], [REQ-MARKDOWN-TABLE-PARSER], [REQ-METROLOGY-SI-VALIDATOR]
//! Contract: Extracts `PORT-*`/`CONN-*`/`SIG-*` identifiers, validates SI units, parses Markdown tables, and collects nested AST ports.
//! Bounds & Faults: Pure non-panicking string and AST operations with zero regex.

use deap_core::sysml_ast::PartDef;
use std::collections::BTreeSet;

/// Checks if a metrology value is an unresolved placeholder string.
/// Realises: [REQ-METROLOGY-PLACEHOLDER-DETECTOR]
pub(crate) fn is_metrology_placeholder(val: &str) -> bool {
    let lower = val.to_ascii_lowercase();
    lower.contains("not declared in schema")
        || lower.contains("declared assumption")
        || lower.contains("tbd")
        || lower.contains("unknown")
        || lower.contains("todo")
        || lower.contains("unspecified")
}

/// Checks if a unit string matches recognized SI or dimensional metric standards.
/// Realises: [REQ-METROLOGY-SI-VALIDATOR]
pub(crate) fn is_valid_si_unit(unit: &str) -> bool {
    let u = unit.trim().to_ascii_lowercase();
    matches!(
        u.as_str(),
        "s" | "sec"
            | "second"
            | "seconds"
            | "ms"
            | "us"
            | "µs"
            | "ns"
            | "min"
            | "h"
            | "hr"
            | "hz"
            | "khz"
            | "mhz"
            | "ghz"
            | "m"
            | "km"
            | "cm"
            | "mm"
            | "nm"
            | "um"
            | "µm"
            | "m/s"
            | "km/h"
            | "m/s^2"
            | "m/s2"
            | "fps"
            | "rpm"
            | "kg"
            | "g"
            | "mg"
            | "j"
            | "kj"
            | "mj"
            | "wh"
            | "kwh"
            | "w"
            | "kw"
            | "mw"
            | "v"
            | "mv"
            | "kv"
            | "a"
            | "ma"
    ) || is_valid_digital_or_angle_unit(&u)
}

/// Checks angular, dimensionless, ratio, or digital throughput unit tokens.
/// Realises: [REQ-METROLOGY-SI-VALIDATOR]
fn is_valid_digital_or_angle_unit(u: &str) -> bool {
    matches!(
        u,
        "deg"
            | "rad"
            | "degree"
            | "degrees"
            | "radian"
            | "radians"
            | "db"
            | "dbm"
            | "dbi"
            | "%"
            | "percent"
            | "ratio"
            | "dimensionless"
            | "count"
            | "bps"
            | "kbps"
            | "mbps"
            | "gbps"
            | "b/s"
            | "kb/s"
            | "mb/s"
            | "gb/s"
            | "b"
            | "kb"
            | "mb"
            | "gb"
            | "bytes"
            | "ppm"
    )
}

/// Recursively collects part definition names into a deterministic set.
/// Realises: [REQ-AST-PART-DEF-NAME-COLLECTOR]
pub(crate) fn collect_part_def_names(part: &PartDef, set: &mut BTreeSet<String>) {
    if !part.name.is_empty() {
        set.insert(part.name.clone());
    }
    for sub in &part.parts {
        collect_part_def_names(sub, set);
    }
}

/// Extracts a declared port ID (e.g. `PORT-FC-GNSS_FIX_IN`) from text.
/// Realises: [REQ-PORT-ID-EXTRACTION]
pub fn extract_port_identifier(s: &str) -> Option<String> {
    let trimmed = s.trim_matches(|c| c == '`' || c == ' ' || c == '*');
    let pos = trimmed.find("PORT-")?;
    let rest = trimmed.get(pos..)?;
    let end = rest
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
        .unwrap_or(rest.len());
    if end > 5 {
        rest.get(..end).map(|slice| slice.to_ascii_uppercase())
    } else {
        None
    }
}

/// Extracts connection ID (e.g. `CONN-05`) from text.
/// Realises: [REQ-CONN-ID-EXTRACTION]
pub fn extract_single_conn_id(s: &str) -> Option<String> {
    let trimmed = s.trim_matches(|c| c == '`' || c == ' ' || c == '*');
    let lower = trimmed.to_ascii_lowercase();
    let pos = lower.find("conn-")?;
    let rest = trimmed.get(pos..)?;
    let end = rest
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
        .unwrap_or(rest.len());
    if end > 5 {
        rest.get(..end).map(|slice| slice.to_ascii_uppercase())
    } else {
        None
    }
}

/// Extracts signal ID (e.g. `SIG-NS-FC-001`) from text.
/// Realises: [REQ-SIGNAL-ID-EXTRACTION]
pub fn extract_single_signal_id(s: &str) -> Option<String> {
    let trimmed = s.trim_matches(|c| c == '`' || c == ' ' || c == '*');
    let lower = trimmed.to_ascii_lowercase();
    let pos = lower.find("sig-")?;
    let rest = trimmed.get(pos..)?;
    let end = rest
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
        .unwrap_or(rest.len());
    if end > 4 {
        rest.get(..end).map(|slice| slice.to_ascii_uppercase())
    } else {
        None
    }
}

/// Strict Markdown table separator row validator (`^:?-+:?$`).
/// Realises: [REQ-TABLE-SEPARATOR-VALIDATION]
pub fn is_table_separator_line(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return false;
    }
    let cols: Vec<&str> = trimmed
        .split('|')
        .map(|c| c.trim())
        .filter(|c| !c.is_empty())
        .collect();
    if cols.is_empty() {
        return false;
    }
    cols.iter().all(|col| {
        let c_trim = col.trim_matches(':');
        !c_trim.is_empty() && c_trim.chars().all(|ch| ch == '-')
    })
}

/// Parses Markdown table headers and rows under a section header containing `section_kw`.
/// Realises: [REQ-MARKDOWN-TABLE-PARSER]
pub fn parse_markdown_table_rows(
    content: &str,
    section_kw: &str,
) -> (Vec<String>, Vec<Vec<String>>) {
    let mut headers = Vec::new();
    let mut rows = Vec::new();
    let mut in_section = false;
    let mut in_table = false;
    let mut header_seen = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            if trimmed.to_ascii_lowercase().contains(section_kw) {
                in_section = true;
                in_table = false;
                header_seen = false;
                continue;
            } else if in_section {
                if !rows.is_empty() {
                    break;
                }
                in_section = false;
                in_table = false;
                header_seen = false;
            }
        }
        if !in_section {
            continue;
        }
        if trimmed.starts_with('|') {
            if is_table_separator_line(trimmed) {
                in_table = true;
                continue;
            }
            let raw_cols: Vec<&str> = trimmed.split('|').map(|c| c.trim()).collect();
            let col_count = raw_cols.len();
            let start = usize::from(trimmed.starts_with('|') && col_count > 1);
            let end = if trimmed.ends_with('|') && col_count > start {
                col_count.saturating_sub(1)
            } else {
                col_count
            };
            let row_cols: Vec<String> = raw_cols
                .get(start..end)
                .unwrap_or(&[])
                .iter()
                .map(|&s| s.to_string())
                .collect();
            if !header_seen {
                headers = row_cols.iter().map(|s| s.to_ascii_lowercase()).collect();
                header_seen = true;
                continue;
            }
            if in_table {
                rows.push(row_cols);
            }
        } else if in_table && trimmed.is_empty() {
            break;
        }
    }
    (headers, rows)
}

/// Metadata describing a declared port definition in `ICD_01`.
/// Realises: [REQ-CHECKS-PARITY-GATES-PORTMETA]
#[derive(Debug, Clone)]
pub(crate) struct PortMeta {
    pub(crate) subsystem: String,
    pub(crate) name: String,
    pub(crate) direction: String,
}

/// Recursively collects nested subsystem and port identifiers from parsed SysML `PartDef` AST nodes.
/// Realises: [REQ-PARITY-GATE-24]
pub(crate) fn collect_ast_nested_subsystems_and_ports(
    parts: &[deap_core::sysml_ast::PartDef],
    is_nested: bool,
    nested_subsystems: &mut BTreeSet<String>,
    nested_ports: &mut BTreeSet<String>,
) {
    for part in parts {
        if is_nested {
            if !part.name.is_empty() {
                nested_subsystems.insert(part.name.to_ascii_lowercase());
            }
            for port in &part.ports {
                if !port.name.is_empty() {
                    nested_ports.insert(port.name.to_ascii_lowercase());
                }
            }
        }
        if !part.parts.is_empty() {
            if !part.name.is_empty() {
                nested_subsystems.insert(part.name.to_ascii_lowercase());
            }
            collect_ast_nested_subsystems_and_ports(
                &part.parts,
                true,
                nested_subsystems,
                nested_ports,
            );
        }
    }
}
