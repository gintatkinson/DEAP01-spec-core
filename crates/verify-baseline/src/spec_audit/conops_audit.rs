//! Native Rust ConOps Completeness & Mission Intent Integrity Engine.
//!
//! Conforms to DO-178C Level A and ISO 26262 ASIL D safety standards.
//!
//! Validates:
//! 1. Hierarchical ConOps and Tactical Mission Intent workspace presence:
//!    - Checks whether `docs/conops/` exists; if absent in downstream workspace, gracefully skip or report exempt.
//! 2. Open Schema Contract ($N \ge N_{\mathrm{min}}$) audits when present:
//!    - Emergency Decision Matrix ($N \ge 7$ canonical triggers `EMG-01`..`EMG-07`).
//!    - PACE C2 Plan ($N \ge 4$ tiers: Primary, Alternate, Contingency, Emergency).
//!    - Multi-Domain Threat Matrix (covers at least 9 threat domains: Kinetic, Mechanical,
//!      Power/Thermal, Environmental, EW/Cyber, Optical, Signature/Acoustic, Human Factors, CBRN).
//!    - MoE / MoP formulations with Threshold / Objective value pairs and SI units.
//!    - Operational allocation tags (`/// OperationalAllocation: [OA-XX]` or `[MET-XX]`).
//!
//! Strictly zero regex, zero unwrap/expect/panic in non-test code, zero Unicode em dashes (uses ASCII -- exclusively).

use crate::spec_audit::markdown_ast::{parse_markdown, MarkdownDoc};
use crate::spec_audit::{SpecAuditError, SpecAuditFinding};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Canonical emergency contingency triggers required in Section 12 Emergency Decision Matrix.
pub const CANONICAL_EMERGENCY_TRIGGERS: &[&str] = &[
    "EMG-01", // Lost C2 Link
    "EMG-02", // GNSS Navigation Loss
    "EMG-03", // Propulsion / Power Failure
    "EMG-04", // Critical Sensor Fault
    "EMG-05", // Geofence Breach / Airspace Conflict
    "EMG-06", // Structural / Actuation Anomaly
    "EMG-07", // Emergency Termination Command
];

/// Mandatory C2 communication tiers required in PACE plan.
pub const PACE_TIERS: &[&str] = &["primary", "alternate", "contingency", "emergency"];

/// Threat domain specification containing domain name and matching keywords.
#[derive(Debug, Clone, Copy)]
pub struct ThreatDomainSpec {
    pub name: &'static str,
    pub keywords: &'static [&'static str],
}

/// 9 canonical operational threat domains required in Multi-Domain Threat Matrix.
pub const MANDATORY_THREAT_DOMAINS: &[ThreatDomainSpec] = &[
    ThreatDomainSpec {
        name: "Kinetic",
        keywords: &["kinetic", "thr-kin", "ballistic", "projectile", "collision"],
    },
    ThreatDomainSpec {
        name: "Mechanical",
        keywords: &["mechanical", "structural", "thr-mec", "actuator jam", "flutter"],
    },
    ThreatDomainSpec {
        name: "Power/Thermal",
        keywords: &["power/thermal", "power and thermal", "power", "thermal", "thr-pwr", "thr-thm"],
    },
    ThreatDomainSpec {
        name: "Environmental",
        keywords: &["environmental", "atmospheric", "weather", "icing", "precipitation", "gust", "thr-env"],
    },
    ThreatDomainSpec {
        name: "EW/Cyber",
        keywords: &[
            "ew", "electronic warfare", "electromagnetic", "rf jamming", "gnss jamming",
            "cyber", "cybersecurity", "packet injection", "firmware tampering", "thr-ew", "thr-cyb", "thr-ewc",
        ],
    },
    ThreatDomainSpec {
        name: "Optical",
        keywords: &["optical", "laser", "dazzling", "camera saturation", "thr-opt"],
    },
    ThreatDomainSpec {
        name: "Signature/Acoustic",
        keywords: &["signature", "acoustic", "infrared", "rcs", "radar cross-section", "thr-sig", "thr-ac"],
    },
    ThreatDomainSpec {
        name: "Human Factors",
        keywords: &["human factors", "human", "operator fatigue", "pilot", "input disparity", "thr-hum"],
    },
    ThreatDomainSpec {
        name: "CBRN",
        keywords: &["cbrn", "chemical", "biological", "radiological", "nuclear", "toxic", "contamination", "thr-cbrn"],
    },
];

/// Recognized SI and engineering units for MoE / MoP formulations.
pub const MOE_MOP_RECOGNIZED_UNITS: &[&str] = &[
    "s", "sec", "second", "seconds", "ms", "us", "µs", "ns", "min", "h", "hr",
    "hz", "khz", "mhz", "ghz",
    "m", "km", "cm", "mm", "nm", "um", "µm",
    "m/s", "km/h", "m/s^2", "m/s2", "fps", "rpm",
    "kg", "g", "mg",
    "j", "kj", "mj", "wh", "kwh",
    "w", "kw", "mw",
    "v", "mv", "kv",
    "a", "ma",
    "deg", "rad", "degree", "degrees", "radian", "radians",
    "db", "dbm", "dbi",
    "%", "percent", "ratio", "dimensionless", "count",
    "bps", "kbps", "mbps", "gbps", "b/s", "kb/s", "mb/s", "gb/s",
    "b", "kb", "mb", "gb", "bytes", "ppm",
];

/// Represents an individual parsed Markdown table row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow {
    pub cells: Vec<String>,
    pub line_no: usize,
}

/// Represents an extracted Markdown table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownTable {
    pub headers: Vec<String>,
    pub rows: Vec<TableRow>,
    pub start_line: usize,
}

/// Extracts all CommonMark pipe tables from content without using regular expressions.
///
/// Preconditions: None.
/// Postconditions: Returns list of parsed Markdown tables preserving column order and line numbers.
/// Algorithmic Complexity: O(L * C) where L is lines count and C is cells count.
pub fn extract_markdown_tables(content: &str) -> Vec<MarkdownTable> {
    let mut tables = Vec::new();
    let mut in_code_block = false;
    let mut current_headers: Option<Vec<String>> = None;
    let mut current_rows: Vec<TableRow> = Vec::new();
    let mut table_start_line = 0;
    let mut delimiter_expected = false;

    for (idx, line) in content.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            if current_headers.is_some() {
                if let Some(headers) = current_headers.take() {
                    tables.push(MarkdownTable {
                        headers,
                        rows: current_rows.clone(),
                        start_line: table_start_line,
                    });
                    current_rows.clear();
                }
            }
            continue;
        }

        if in_code_block {
            continue;
        }

        if trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.len() >= 2 {
            let inner = match trimmed.strip_prefix('|').and_then(|s| s.strip_suffix('|')) {
                Some(s) => s,
                None => "",
            };
            let cells: Vec<String> = inner.split('|').map(|c| c.trim().to_string()).collect();

            // Check if this is a delimiter row (| :--- | :--- |)
            let is_delimiter = cells.iter().all(|c| {
                !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':' || ch.is_whitespace())
            });

            if is_delimiter && delimiter_expected {
                delimiter_expected = false;
                continue;
            } else if is_delimiter {
                continue;
            }

            if current_headers.is_none() {
                // Header row
                current_headers = Some(cells);
                table_start_line = line_no;
                delimiter_expected = true;
            } else if !delimiter_expected {
                // Data row
                current_rows.push(TableRow { cells, line_no });
            }
        } else {
            // End of table
            if let Some(headers) = current_headers.take() {
                tables.push(MarkdownTable {
                    headers,
                    rows: current_rows.clone(),
                    start_line: table_start_line,
                });
                current_rows.clear();
            }
            delimiter_expected = false;
        }
    }

    if let Some(headers) = current_headers.take() {
        tables.push(MarkdownTable {
            headers,
            rows: current_rows,
            start_line: table_start_line,
        });
    }

    tables
}

/// Checks whether `docs/conops/` exists on disk.
///
/// Preconditions: `repo_root` is an accessible path.
/// Postconditions: Returns true if `docs/conops/` directory exists.
/// Algorithmic Complexity: O(1) filesystem metadata check.
pub fn check_conops_dir_exists(repo_root: &Path) -> bool {
    repo_root.join("docs").join("conops").is_dir()
}

/// Audits Section 12 Emergency Decision Matrix for complete coverage of canonical emergency triggers ($N \ge 7$).
///
/// Preconditions: `content` is valid UTF-8.
/// Postconditions: Returns findings if any canonical trigger is missing or count is less than 7.
/// Algorithmic Complexity: O(T + L) where T is tables and L is lines count.
pub fn validate_emergency_decision_matrix(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let lower_doc = content.to_lowercase();

    // Check if this document includes an Emergency Decision section
    let has_emergency_section = lower_doc.contains("emergency decision")
        || lower_doc.contains("contingency matrix")
        || lower_doc.contains("7-row emergency decision")
        || lower_doc.contains("emg-01");

    if !has_emergency_section {
        return findings;
    }

    // Extract all EMG-XX triggers mentioned
    let mut found_triggers = HashSet::new();
    let upper_doc = content.to_ascii_uppercase();

    for &canonical in CANONICAL_EMERGENCY_TRIGGERS {
        if upper_doc.contains(canonical) {
            found_triggers.insert(canonical.to_string());
        }
    }

    // Also scan for any other EMG-XX triggers
    let mut search_from = 0;
    while let Some(slice) = upper_doc.get(search_from..) {
        let Some(pos) = slice.find("EMG-") else {
            break;
        };
        let abs_pos = search_from + pos;
        let suffix = upper_doc.get(abs_pos + 4..).unwrap_or("");
        let mut num_end = 0;
        let bytes = suffix.as_bytes();
        while num_end < bytes.len() && bytes[num_end].is_ascii_digit() {
            num_end += 1;
        }
        if num_end > 0 {
            if let Some(digits) = suffix.get(..num_end) {
                let trigger_id = format!("EMG-{}", digits);
                found_triggers.insert(trigger_id);
            }
        }
        search_from = abs_pos + 4;
    }

    // Validate N >= 7 triggers
    if found_triggers.len() < 7 {
        let missing: Vec<&str> = CANONICAL_EMERGENCY_TRIGGERS
            .iter()
            .copied()
            .filter(|&t| !found_triggers.contains(t))
            .collect();

        findings.push(SpecAuditFinding::new(
            "conops-emergency-matrix-incomplete",
            format!(
                "Section 12 Emergency Decision Matrix in '{}' defines {}/7 canonical emergency triggers; missing: {}.",
                rel_path,
                found_triggers.len(),
                if missing.is_empty() { "none (count underflow)".to_string() } else { missing.join(", ") }
            ),
            rel_path,
            None,
        ));
    }

    findings
}

/// Audits Section 5 PACE C2 Link Communications Plan for coverage of 4 mandatory tiers ($N \ge 4$).
///
/// Preconditions: `content` is valid UTF-8.
/// Postconditions: Returns findings if any PACE tier (Primary, Alternate, Contingency, Emergency) is missing.
/// Algorithmic Complexity: O(T + L) where T is tables and L is lines count.
pub fn validate_pace_plan(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let lower_doc = content.to_lowercase();

    // Check if this document includes a PACE plan section
    let has_pace_section = lower_doc.contains("pace c2")
        || lower_doc.contains("pace communications")
        || lower_doc.contains("c2 link communications plan")
        || lower_doc.contains("pace plan")
        || lower_doc.contains("## 5. pace");

    if !has_pace_section {
        return findings;
    }

    let mut found_tiers = HashSet::new();

    // Scan for PACE tiers in tables or lines
    let tables = extract_markdown_tables(content);
    for tbl in &tables {
        let header_str = tbl.headers.join(" ").to_lowercase();
        if header_str.contains("pace") || header_str.contains("tier") || header_str.contains("link medium") {
            for row in &tbl.rows {
                let row_str = row.cells.join(" ").to_lowercase();
                for &tier in PACE_TIERS {
                    if row_str.contains(tier) {
                        found_tiers.insert(tier);
                    }
                }
            }
        }
    }

    // Fallback scan across full text if not found in tables
    if found_tiers.len() < PACE_TIERS.len() {
        for &tier in PACE_TIERS {
            if lower_doc.contains(tier) {
                found_tiers.insert(tier);
            }
        }
    }

    if found_tiers.len() < 4 {
        let missing: Vec<&str> = PACE_TIERS
            .iter()
            .copied()
            .filter(|&t| !found_tiers.contains(t))
            .collect();

        findings.push(SpecAuditFinding::new(
            "conops-pace-plan-incomplete",
            format!(
                "Section 5 PACE C2 Link Communications Plan in '{}' defines {}/4 mandatory tiers; missing: {}.",
                rel_path,
                found_tiers.len(),
                missing.join(", ")
            ),
            rel_path,
            None,
        ));
    }

    findings
}

/// Audits Section 4 Multi-Domain Operational Threat & Contested Environment Matrix across 9 threat domains.
///
/// Preconditions: `content` is valid UTF-8.
/// Postconditions: Returns findings if any mandatory threat domain is missing.
/// Algorithmic Complexity: O(D * L) where D is threat domains and L is lines count.
pub fn validate_threat_matrix(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let lower_doc = content.to_lowercase();

    // Check if this document includes a threat matrix section
    let has_threat_section = lower_doc.contains("threat matrix")
        || lower_doc.contains("contested environment")
        || lower_doc.contains("operational threat")
        || lower_doc.contains("## 4. multi-domain");

    if !has_threat_section {
        return findings;
    }

    let mut found_domains = HashSet::new();

    for domain in MANDATORY_THREAT_DOMAINS {
        let matched = domain.keywords.iter().any(|&kw| lower_doc.contains(kw));
        if matched {
            found_domains.insert(domain.name);
        }
    }

    if found_domains.len() < MANDATORY_THREAT_DOMAINS.len() {
        let missing: Vec<&str> = MANDATORY_THREAT_DOMAINS
            .iter()
            .map(|d| d.name)
            .filter(|&name| !found_domains.contains(name))
            .collect();

        findings.push(SpecAuditFinding::new(
            "conops-threat-matrix-missing-domain",
            format!(
                "Multi-Domain Operational Threat Matrix in '{}' defines {}/9 mandatory domains; missing: {}.",
                rel_path,
                found_domains.len(),
                missing.join(", ")
            ),
            rel_path,
            None,
        ));
    }

    findings
}

/// Audits Section 3 Measures of Effectiveness (MoE) & Measures of Performance (MoP) for
/// Threshold / Objective value pairs and explicit SI / engineering units.
///
/// Preconditions: `content` is valid UTF-8.
/// Postconditions: Returns findings for missing value pairs or invalid/missing SI units.
/// Algorithmic Complexity: O(T * R) where T is tables and R is rows count.
pub fn validate_moe_mop_metrics(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let lower_doc = content.to_lowercase();

    // Check if this document contains MoE / MoP specifications
    let has_moe_mop = lower_doc.contains("measures of effectiveness")
        || lower_doc.contains("measures of performance")
        || lower_doc.contains("moe")
        || lower_doc.contains("mop");

    if !has_moe_mop {
        return findings;
    }

    let tables = extract_markdown_tables(content);

    for tbl in &tables {
        let header_str = tbl.headers.join(" ").to_lowercase();
        let has_metric_cols = header_str.contains("metric") || header_str.contains("moe") || header_str.contains("mop");
        let has_thresh_obj = header_str.contains("threshold") || header_str.contains("objective");

        if has_metric_cols && has_thresh_obj {
            // Map column indices
            let mut thresh_idx = None;
            let mut obj_idx = None;
            let mut unit_idx = None;
            let mut id_idx = 0;

            for (col_idx, h) in tbl.headers.iter().enumerate() {
                let h_lower = h.to_lowercase();
                if h_lower.contains("threshold") {
                    thresh_idx = Some(col_idx);
                } else if h_lower.contains("objective") {
                    obj_idx = Some(col_idx);
                } else if h_lower.contains("unit") {
                    unit_idx = Some(col_idx);
                } else if h_lower.contains("id") {
                    id_idx = col_idx;
                }
            }

            for row in &tbl.rows {
                let metric_id = row.cells.get(id_idx).map(|s| s.trim()).unwrap_or("Metric");
                let metric_id_clean = metric_id.trim_matches(&['`', '*', '_'][..]);

                // Check Threshold and Objective value pairs
                let thresh_val = thresh_idx.and_then(|idx| row.cells.get(idx)).map(|s| s.trim()).unwrap_or("");
                let obj_val = obj_idx.and_then(|idx| row.cells.get(idx)).map(|s| s.trim()).unwrap_or("");

                let is_thresh_invalid = thresh_val.is_empty()
                    || thresh_val.contains("{{")
                    || thresh_val.eq_ignore_ascii_case("tbd")
                    || thresh_val.eq_ignore_ascii_case("n/a");

                let is_obj_invalid = obj_val.is_empty()
                    || obj_val.contains("{{")
                    || obj_val.eq_ignore_ascii_case("tbd")
                    || obj_val.eq_ignore_ascii_case("n/a");

                if is_thresh_invalid || is_obj_invalid {
                    findings.push(SpecAuditFinding::new(
                        "conops-moe-mop-missing-threshold-objective",
                        format!(
                            "Metric '{}' on line {} in '{}' is missing valid Threshold and/or Objective value pair (Threshold='{}', Objective='{}').",
                            metric_id_clean, row.line_no, rel_path, thresh_val, obj_val
                        ),
                        rel_path,
                        Some(row.line_no),
                    ));
                }

                // Check SI / engineering unit
                if let Some(uidx) = unit_idx {
                    let unit_val = row.cells.get(uidx).map(|s| s.trim()).unwrap_or("");
                    let unit_clean = unit_val
                        .trim_matches(&['`', '*', '_', '[', ']'][..])
                        .to_lowercase();

                    let is_unit_valid = !unit_clean.is_empty()
                        && !unit_clean.contains("{{")
                        && !unit_clean.eq_ignore_ascii_case("tbd")
                        && !unit_clean.eq_ignore_ascii_case("n/a")
                        && MOE_MOP_RECOGNIZED_UNITS.iter().any(|&u| u == unit_clean);

                    if !is_unit_valid {
                        findings.push(SpecAuditFinding::new(
                            "conops-moe-mop-missing-unit",
                            format!(
                                "Metric '{}' on line {} in '{}' specifies invalid or missing SI/engineering unit: '{}'.",
                                metric_id_clean, row.line_no, rel_path, unit_val
                            ),
                            rel_path,
                            Some(row.line_no),
                        ));
                    }
                }
            }
        }
    }

    findings
}

/// Audits operational allocation tags (`/// OperationalAllocation: [OA-XX]` or `[MET-XX]`)
/// across ConOps and Mission Intent specifications.
///
/// Preconditions: `content` is valid UTF-8.
/// Postconditions: Returns findings for missing or malformed operational allocation tags.
/// Algorithmic Complexity: O(L) where L is lines count.
pub fn validate_operational_allocation_tags(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let mut declared_oa_tasks = HashSet::new();
    let mut declared_met_tasks = HashSet::new();

    // 1. Scan for declared OA-XX and MET-XX identifiers outside code blocks
    let mut in_code_block = false;
    for (idx, line) in content.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        // Check for malformed allocation tags e.g. "/// OperationalAllocation:" without square brackets
        if trimmed.starts_with("///") && trimmed.to_lowercase().contains("operationalallocation") {
            if !trimmed.contains('[') || !trimmed.contains(']') {
                findings.push(SpecAuditFinding::new(
                    "conops-operational-allocation-malformed",
                    format!(
                        "Operational allocation tag on line {} in '{}' is malformed; expected '/// OperationalAllocation: [ID]' format.",
                        line_no, rel_path
                    ),
                    rel_path,
                    Some(line_no),
                ));
            }
        }

        // Extract OA-XX and MET-XX declarations from tables or list items
        let upper_line = trimmed.to_ascii_uppercase();

        // Check for OA-XX tokens
        let mut search_from = 0;
        while let Some(slice) = upper_line.get(search_from..) {
            let Some(pos) = slice.find("OA-") else {
                break;
            };
            let abs_pos = search_from + pos;
            let suffix = upper_line.get(abs_pos + 3..).unwrap_or("");
            let mut num_end = 0;
            let bytes = suffix.as_bytes();
            while num_end < bytes.len() && bytes[num_end].is_ascii_digit() {
                num_end += 1;
            }
            if num_end > 0 {
                if let Some(digits) = suffix.get(..num_end) {
                    let id = format!("OA-{}", digits);
                    declared_oa_tasks.insert(id);
                }
            }
            search_from = abs_pos + 3;
        }

        // Check for MET-XX tokens
        let mut search_from_met = 0;
        while let Some(slice) = upper_line.get(search_from_met..) {
            let Some(pos) = slice.find("MET-") else {
                break;
            };
            let abs_pos = search_from_met + pos;
            let suffix = upper_line.get(abs_pos + 4..).unwrap_or("");
            let mut num_end = 0;
            let bytes = suffix.as_bytes();
            while num_end < bytes.len() && bytes[num_end].is_ascii_digit() {
                num_end += 1;
            }
            if num_end > 0 {
                if let Some(digits) = suffix.get(..num_end) {
                    let id = format!("MET-{}", digits);
                    declared_met_tasks.insert(id);
                }
            }
            search_from_met = abs_pos + 4;
        }
    }

    // 2. Verify that each declared OA or MET task has an OperationalAllocation tag
    let upper_content = content.to_ascii_uppercase();

    for oa_id in declared_oa_tasks {
        let expected_tag = format!("[{}]", oa_id);
        if !upper_content.contains("OPERATIONALALLOCATION") || !upper_content.contains(&expected_tag) {
            findings.push(SpecAuditFinding::new(
                "conops-operational-allocation-missing",
                format!(
                    "Operational activity '{}' declared in '{}' has no valid Gate 24 allocation tag ('/// OperationalAllocation: [{}]').",
                    oa_id, rel_path, oa_id
                ),
                rel_path,
                None,
            ));
        }
    }

    for met_id in declared_met_tasks {
        let expected_tag = format!("[{}]", met_id);
        if !upper_content.contains("OPERATIONALALLOCATION") || !upper_content.contains(&expected_tag) {
            findings.push(SpecAuditFinding::new(
                "conops-operational-allocation-missing",
                format!(
                    "Mission essential task '{}' declared in '{}' has no valid Gate 24 allocation tag ('/// OperationalAllocation: [{}]').",
                    met_id, rel_path, met_id
                ),
                rel_path,
                None,
            ));
        }
    }

    findings
}

/// Audits Section 4.1 / 8.1 Super-System Operational Architecture and Segment Boundaries (ISO/IEC/IEEE 29148:2018 §6.4.2 / IEEE Std 1362-1998 §5.3).
///
/// Preconditions: `content` is valid UTF-8.
/// Postconditions: Returns findings if a ConOps document omits super-system operational architecture, operational segment definitions, or boundary diagram.
/// Algorithmic Complexity: O(L) where L is lines count.
pub fn validate_super_system_architecture(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let lower_doc = content.to_lowercase();
    let lower_path = rel_path.to_lowercase();

    let in_conops_dir = lower_path.contains("docs/conops/") || lower_path.starts_with("docs/conops");
    let is_conops_spec = lower_path.ends_with("/conops.md")
        || lower_path == "conops.md"
        || lower_doc.contains("# concept of operations");

    let is_full_conops = is_conops_spec || (in_conops_dir && lower_path.ends_with("conops.md"));
    let is_arch_unit = in_conops_dir
        && (lower_path.contains("04_") || lower_path.contains("08_") || lower_path.contains("architecture"));

    if !is_full_conops && !is_arch_unit {
        return findings;
    }

    // 1. Must define Section 4.1 / 8.1 / 4.7 Super-System Architecture
    let has_super_arch = (lower_doc.contains("super-system") || lower_doc.contains("super system"))
        && (lower_doc.contains("operational architecture")
            || lower_doc.contains("segment boundaries")
            || lower_doc.contains("### 4.1")
            || lower_doc.contains("### 8.1")
            || lower_doc.contains("### 4.7"));

    if !has_super_arch {
        findings.push(SpecAuditFinding::new(
            "conops-super-system-architecture-missing",
            format!(
                "ConOps specification in '{}' omits required Section 4.1 / 8.1 / 4.7 Super-System Operational Architecture and Segment Boundaries (ISO/IEC/IEEE 29148:2018 §6.4.2 / IEEE Std 1362-1998 §5.3).",
                rel_path
            ),
            rel_path,
            None,
        ));
        return findings;
    }

    let is_template_placeholder = content.contains("{{SUPER_SYSTEM_ARCHITECTURE}}");

    // 2. Must contain Mermaid boundary diagram formalizing segment boundaries
    let has_diagram = is_template_placeholder
        || (content.contains("```mermaid")
            && (lower_doc.contains("flowchart") || lower_doc.contains("graph ")));

    if !has_diagram {
        findings.push(SpecAuditFinding::new(
            "conops-super-system-boundary-diagram-missing",
            format!(
                "Section 4.1 / 8.1 / 4.7 in '{}' omits required Mermaid boundary diagram formalizing system operational segment boundaries.",
                rel_path
            ),
            rel_path,
            None,
        ));
    }

    // 3. Must declare operational segments
    let has_segments = lower_doc.contains("segment");
    if !has_segments {
        findings.push(SpecAuditFinding::new(
            "conops-super-system-segments-missing",
            format!(
                "Section 4.1 / 8.1 / 4.7 in '{}' omits required operational segment definitions and segment allocation matrix.",
                rel_path
            ),
            rel_path,
            None,
        ));
    }

    // 4. Must enforce IEEE 1362 and DoDAF SV-1 standards governance (Issue #267)
    let has_ieee1362 = lower_doc.contains("ieee 1362") || lower_doc.contains("ieee std 1362") || lower_doc.contains("ieee:1362");
    let has_dodaf_sv1 = (lower_doc.contains("dodaf") && (lower_doc.contains("sv-1") || lower_doc.contains("sv1")))
        || lower_doc.contains("dodaf 2.02 sv-1")
        || lower_doc.contains("dodaf v2.02 sv-1");
    if !has_ieee1362 || !has_dodaf_sv1 {
        findings.push(SpecAuditFinding::new(
            "conops-super-system-standards-governance-missing",
            format!(
                "ConOps Section 4 / 8 Super-System Architecture in '{}' lacks explicit IEEE 1362 (§5.3) and DoDAF SV-1 standards governance.",
                rel_path
            ),
            rel_path,
            None,
        ));
    }

    // 5. Must enforce discrete port definitions in architecture diagram (DoDAF SV-1 / SysML v2 Port Taxonomy)
    let has_diagram_ports = if is_template_placeholder {
        true
    } else if let Some(m_idx) = content.find("```mermaid") {
        let m_slice = &content[m_idx..];
        let end_idx = m_slice[10..].find("```").map(|i| m_idx + 10 + i).unwrap_or(content.len());
        let diag_text = &content[m_idx..end_idx];
        diag_text.contains("PORT-") || diag_text.contains("PORT_") || diag_text.to_lowercase().contains("port")
    } else {
        false
    };

    if !has_diagram_ports {
        findings.push(SpecAuditFinding::new(
            "conops-super-system-ports-missing",
            format!(
                "Section 4 / 8 Super-System Architecture diagram in '{}' omits discrete port definitions (PORT-<SUBSYS>-<NAME>) mandated by DoDAF SV-1 and SysML v2 Port Taxonomy.",
                rel_path
            ),
            rel_path,
            None,
        ));
    }

    // 6. Must enforce traceable connection identifiers (CONN-01..CONN-N)
    let has_diagram_conns = if is_template_placeholder {
        true
    } else if let Some(m_idx) = content.find("```mermaid") {
        let m_slice = &content[m_idx..];
        let end_idx = m_slice[10..].find("```").map(|i| m_idx + 10 + i).unwrap_or(content.len());
        let diag_text = &content[m_idx..end_idx];
        diag_text.contains("CONN-") || diag_text.contains("conn-")
    } else {
        false
    };

    if !has_diagram_conns {
        findings.push(SpecAuditFinding::new(
            "conops-super-system-connections-missing",
            format!(
                "Section 4 / 8 Super-System Architecture diagram in '{}' omits traceable connection identifiers (CONN-01..CONN-N) mandated by ISO/IEC/IEEE 29148:2018 §6.4.3 and IEEE 1362 §5.3.",
                rel_path
            ),
            rel_path,
            None,
        ));
    }

    // 7. Must enforce Rule E5: Modularized Subgraph Boundary Edge Routing (refs #277)
    if !is_template_placeholder {
        if let Some(m_idx) = content.find("```mermaid") {
            let m_slice = &content[m_idx..];
            let end_idx = m_slice[10..].find("```").map(|i| m_idx + 10 + i).unwrap_or(content.len());
            let diag_text = &content[m_idx..end_idx];

            if let Some(err_msg) = audit_modular_subgraph_edge_routing(diag_text) {
                findings.push(SpecAuditFinding::new(
                    "conops-super-system-modular-edge-routing-missing",
                    format!(
                        "Section 4 / 8 Super-System Architecture diagram in '{}' {}. Decouple super-system segment context from internal subsystem breakdown views and route external edges cleanly to boundary ports (Rule E5, refs #277).",
                        rel_path, err_msg
                    ),
                    rel_path,
                    None,
                ));
            }
        }
    }

    findings
}

/// Audits Mermaid diagram for Rule E5 modularized boundary edge routing.
/// Detects monolithic nested subgraphs (depth >= 3) and cross-hierarchy edges penetrating into nested subgraphs.
pub fn audit_modular_subgraph_edge_routing(diag_text: &str) -> Option<String> {
    let mut subgraph_stack: Vec<String> = Vec::new();
    let mut max_nesting_depth = 0usize;
    let mut nested_nodes: HashSet<String> = HashSet::new();
    let mut node_subgraph: HashMap<String, String> = HashMap::new();
    let mut cross_edges: Vec<(String, String, String)> = Vec::new();

    const ARROWS: &[&str] = &[
        "<-->", "<-.->", "<==>", "-->", "-.->", "==>", "---", "-.-", "===",
    ];

    for raw_line in diag_text.lines() {
        let line = strip_mermaid_comment_local(raw_line).trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("subgraph") {
            let after = line["subgraph".len()..].trim();
            let sub_id = if let Some(bracket_pos) = after.find('[') {
                after[..bracket_pos].trim().to_string()
            } else if let Some(space_pos) = after.find(' ') {
                after[..space_pos].trim().to_string()
            } else {
                after.trim_matches('"').to_string()
            };
            subgraph_stack.push(sub_id);
            if subgraph_stack.len() > max_nesting_depth {
                max_nesting_depth = subgraph_stack.len();
            }
            continue;
        }

        if line == "end" || line.starts_with("end ") {
            subgraph_stack.pop();
            continue;
        }

        // Check if edge
        let mut is_edge = false;
        for &arrow in ARROWS {
            if let Some(pos) = line.find(arrow) {
                is_edge = true;
                let from_part = line[..pos].trim();
                let after_arrow = line[pos + arrow.len()..].trim();
                let to_part = if after_arrow.starts_with('|') {
                    if let Some(second_pipe) = after_arrow[1..].find('|') {
                        after_arrow[second_pipe + 2..].trim()
                    } else {
                        after_arrow
                    }
                } else {
                    after_arrow
                };
                let from_id = extract_node_id_local(from_part);
                let to_id = extract_node_id_local(to_part);
                if !from_id.is_empty() && !to_id.is_empty() {
                    cross_edges.push((from_id, to_id, line.to_string()));
                }
                break;
            }
        }

        // If not edge, check if node declaration inside subgraph
        if !is_edge && !subgraph_stack.is_empty() {
            let node_id = extract_node_id_local(line);
            if !node_id.is_empty()
                && node_id != "direction"
                && node_id != "style"
                && node_id != "class"
                && node_id != "classDef"
                && !node_id.contains("~~~")
            {
                if let Some(current_sub) = subgraph_stack.last() {
                    node_subgraph.insert(node_id.clone(), current_sub.clone());
                }
                if subgraph_stack.len() >= 2 {
                    nested_nodes.insert(node_id);
                }
            }
        }
    }

    if max_nesting_depth >= 3 {
        return Some(format!(
            "contains monolithic deeply nested subgraphs (nesting depth {} >= 3), violating Rule E5 (modularized boundary edge routing)",
            max_nesting_depth
        ));
    }

    if !nested_nodes.is_empty() {
        for (from, to, edge_line) in &cross_edges {
            if nested_nodes.contains(to) || nested_nodes.contains(from) {
                let from_sub = node_subgraph.get(from).map(|s| s.as_str()).unwrap_or("");
                let to_sub = node_subgraph.get(to).map(|s| s.as_str()).unwrap_or("");
                if from_sub != to_sub {
                    return Some(format!(
                        "routes cross-hierarchy edge '{}' across nested subgraph boundaries, violating Rule E5 (modularized boundary edge routing)",
                        edge_line
                    ));
                }
            }
        }
    }

    None
}

fn extract_node_id_local(s: &str) -> String {
    let delimiters = ['[', '(', '{', '>', '-', '.', '='];
    if let Some(idx) = s.find(|c| delimiters.contains(&c)) {
        s[..idx].trim().to_string()
    } else {
        s.trim().to_string()
    }
}

fn strip_mermaid_comment_local(line: &str) -> &str {
    if let Some(pos) = line.find("%%") {
        &line[..pos]
    } else {
        line
    }
}

/// Audits Section 4.3 / 8.3 Subsystem Architecture (100% AST part definition coverage) (ISO/IEC/IEEE 29148:2018 §6.4.3).
///
/// Preconditions: `content` is valid UTF-8.
/// Postconditions: Returns findings if a ConOps document omits subsystem architecture or AST part bindings.
/// Algorithmic Complexity: O(L) where L is lines count.
pub fn validate_subsystem_architecture(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let lower_doc = content.to_lowercase();
    let lower_path = rel_path.to_lowercase();

    let in_conops_dir = lower_path.contains("docs/conops/") || lower_path.starts_with("docs/conops");
    let is_conops_spec = lower_path.ends_with("/conops.md")
        || lower_path == "conops.md"
        || lower_doc.contains("# concept of operations");

    let is_full_conops = is_conops_spec || (in_conops_dir && lower_path.ends_with("conops.md"));
    let is_arch_unit = in_conops_dir
        && (lower_path.contains("04_") || lower_path.contains("08_") || lower_path.contains("architecture"));

    if !is_full_conops && !is_arch_unit {
        return findings;
    }

    // 1. Must define Section 4.3 / 8.3 Subsystem Architecture
    let has_subsys = lower_doc.contains("subsystem architecture")
        || lower_doc.contains("physical subsystem decomposition")
        || lower_doc.contains("### 4.3")
        || lower_doc.contains("### 8.3")
        || lower_doc.contains("### 4.8");

    if !has_subsys {
        findings.push(SpecAuditFinding::new(
            "conops-subsystem-architecture-missing",
            format!(
                "ConOps specification in '{}' omits required Section 4.3 / 8.3 Subsystem Architecture (100% AST Part Coverage) in accordance with ISO/IEC/IEEE 29148:2018 §6.4.3.",
                rel_path
            ),
            rel_path,
            None,
        ));
        return findings;
    }

    // 2. Must contain AST part allocations / ground truth citations
    let has_ast_binding = lower_doc.contains("part def")
        || lower_doc.contains("ast part")
        || lower_doc.contains("ast_part_ref")
        || lower_doc.contains("ssot ground truth")
        || lower_doc.contains("subsystem part")
        || lower_doc.contains("functional purpose")
        || lower_doc.contains("schema/");

    if !has_ast_binding {
        findings.push(SpecAuditFinding::new(
            "conops-subsystem-ast-coverage-missing",
            format!(
                "Section 4.3 / 8.3 in '{}' omits deterministic AST part def coverage and SSOT ground truth citations.",
                rel_path
            ),
            rel_path,
            None,
        ));
    }

    findings
}

/// Audits Section 4.4 / 8.4 System Interface Topology and Port Taxonomy (ISO/IEC/IEEE 29148:2018 §6.4.3).
///
/// Preconditions: `content` is valid UTF-8.
/// Postconditions: Returns findings if a ConOps document omits port taxonomy or interface topology.
/// Algorithmic Complexity: O(L) where L is lines count.
pub fn validate_system_interface_topology(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let lower_doc = content.to_lowercase();
    let lower_path = rel_path.to_lowercase();

    let in_conops_dir = lower_path.contains("docs/conops/") || lower_path.starts_with("docs/conops");
    let is_conops_spec = lower_path.ends_with("/conops.md")
        || lower_path == "conops.md"
        || lower_doc.contains("# concept of operations");

    let is_full_conops = is_conops_spec || (in_conops_dir && lower_path.ends_with("conops.md"));
    let is_arch_unit = in_conops_dir
        && (lower_path.contains("04_") || lower_path.contains("08_") || lower_path.contains("architecture"));

    if !is_full_conops && !is_arch_unit {
        return findings;
    }

    let has_topo = lower_doc.contains("port taxonomy")
        || lower_doc.contains("interface interconnect")
        || lower_doc.contains("system interface topology")
        || lower_doc.contains("### 4.4")
        || lower_doc.contains("### 8.4");

    if !has_topo {
        findings.push(SpecAuditFinding::new(
            "conops-system-interface-topology-missing",
            format!(
                "ConOps specification in '{}' omits required Section 4.4 / 8.4 System Interface Topology and Port Taxonomy in accordance with ISO/IEC/IEEE 29148:2018 §6.4.3.",
                rel_path
            ),
            rel_path,
            None,
        ));
    }

    findings
}

/// Audits top-level schema contract in .pipeline/schemas/conops_specification_schema.json (Issue #256).
///
/// Preconditions: `schema` is valid parsed JSON.
/// Postconditions: Returns findings if schema contract omits super_system_architecture, subsystem_architecture, or system_interface_topology.
/// Algorithmic Complexity: O(P) where P is schema properties count.
pub fn validate_conops_schema_contract(schema: &Value, schema_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();

    let req_array = match schema.get("required").and_then(|r| r.as_array()) {
        Some(arr) => arr,
        None => {
            findings.push(SpecAuditFinding::new(
                "conops-schema-required-array-missing",
                format!("Top-level schema in '{}' has no required array.", schema_path),
                schema_path,
                None,
            ));
            return findings;
        }
    };

    let required_props: HashSet<&str> = req_array
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    // 1. Verify required array presence
    if !required_props.contains("super_system_architecture") {
        findings.push(SpecAuditFinding::new(
            "conops-schema-super-system-architecture-missing",
            format!("Top-level schema contract required array in '{}' omits 'super_system_architecture'.", schema_path),
            schema_path,
            None,
        ));
    }

    if !required_props.contains("subsystem_architecture") {
        findings.push(SpecAuditFinding::new(
            "conops-schema-subsystem-architecture-missing",
            format!("Top-level schema contract required array in '{}' omits 'subsystem_architecture'.", schema_path),
            schema_path,
            None,
        ));
    }

    if !required_props.contains("system_interface_topology") {
        findings.push(SpecAuditFinding::new(
            "conops-schema-interface-topology-missing",
            format!("Top-level schema contract required array in '{}' omits 'system_interface_topology'.", schema_path),
            schema_path,
            None,
        ));
    }

    if !required_props.contains("user_classes") {
        findings.push(SpecAuditFinding::new(
            "conops-schema-user-classes-missing",
            format!("Top-level schema contract required array in '{}' omits 'user_classes'.", schema_path),
            schema_path,
            None,
        ));
    }


    // 2. Verify properties definitions
    let props = match schema.get("properties").and_then(|p| p.as_object()) {
        Some(p) => p,
        None => {
            findings.push(SpecAuditFinding::new(
                "conops-schema-properties-missing",
                format!("Top-level schema in '{}' has no properties object.", schema_path),
                schema_path,
                None,
            ));
            return findings;
        }
    };

    // 2a. super_system_architecture property structure
    if let Some(super_sys) = props.get("super_system_architecture") {
        let is_obj = super_sys.get("type").and_then(|t| t.as_str()).map_or(false, |t| t == "object");
        let super_req: HashSet<&str> = super_sys
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        let has_segments = super_req.contains("segments");
        let has_diagram = super_req.contains("boundary_diagram_mermaid");

        let seg_items_req: HashSet<&str> = super_sys
            .get("properties")
            .and_then(|p| p.get("segments"))
            .and_then(|s| s.get("items"))
            .and_then(|i| i.get("required"))
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        let has_seg_fields = seg_items_req.contains("segment_id")
            && seg_items_req.contains("segment_name")
            && seg_items_req.contains("operational_role");

        if !is_obj || !has_segments || !has_diagram || !has_seg_fields {
            findings.push(SpecAuditFinding::new(
                "conops-schema-super-system-architecture-incomplete",
                format!("Property 'super_system_architecture' in '{}' is incomplete; requires object with segments (segment_id, segment_name, operational_role) and boundary_diagram_mermaid.", schema_path),
                schema_path,
                None,
            ));
        }
    } else {
        findings.push(SpecAuditFinding::new(
            "conops-schema-super-system-architecture-missing",
            format!("Top-level schema properties in '{}' omits 'super_system_architecture'.", schema_path),
            schema_path,
            None,
        ));
    }

    // 2b. subsystem_architecture property structure
    if let Some(subsys) = props.get("subsystem_architecture") {
        let is_obj = subsys.get("type").and_then(|t| t.as_str()).map_or(false, |t| t == "object");
        let subsys_req: HashSet<&str> = subsys
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        let has_subsystems = subsys_req.contains("subsystems");
        let has_allocations = subsys_req.contains("ast_part_allocations");

        let subsys_items_req: HashSet<&str> = subsys
            .get("properties")
            .and_then(|p| p.get("subsystems"))
            .and_then(|s| s.get("items"))
            .and_then(|i| i.get("required"))
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        let has_subsys_fields = subsys_items_req.contains("subsystem_id")
            && subsys_items_req.contains("subsystem_name")
            && subsys_items_req.contains("ast_part_ref")
            && subsys_items_req.contains("functional_description");

        if !is_obj || !has_subsystems || !has_allocations || !has_subsys_fields {
            findings.push(SpecAuditFinding::new(
                "conops-schema-subsystem-architecture-incomplete",
                format!("Property 'subsystem_architecture' in '{}' is incomplete; requires object with subsystems (subsystem_id, subsystem_name, ast_part_ref, functional_description) and ast_part_allocations.", schema_path),
                schema_path,
                None,
            ));
        }
    } else {
        findings.push(SpecAuditFinding::new(
            "conops-schema-subsystem-architecture-missing",
            format!("Top-level schema properties in '{}' omits 'subsystem_architecture'.", schema_path),
            schema_path,
            None,
        ));
    }

    // 2c. system_interface_topology property structure
    if let Some(topo) = props.get("system_interface_topology") {
        let is_obj = topo.get("type").and_then(|t| t.as_str()).map_or(false, |t| t == "object");
        let topo_req: HashSet<&str> = topo
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        let has_links = topo_req.contains("interface_links");

        let link_items_req: HashSet<&str> = topo
            .get("properties")
            .and_then(|p| p.get("interface_links"))
            .and_then(|l| l.get("items"))
            .and_then(|i| i.get("required"))
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        let has_link_fields = link_items_req.contains("link_id")
            && link_items_req.contains("source_subsystem")
            && link_items_req.contains("target_subsystem")
            && link_items_req.contains("protocol_type");

        if !is_obj || !has_links || !has_link_fields {
            findings.push(SpecAuditFinding::new(
                "conops-schema-interface-topology-incomplete",
                format!("Property 'system_interface_topology' in '{}' is incomplete; requires object with interface_links (link_id, source_subsystem, target_subsystem, protocol_type).", schema_path),
                schema_path,
                None,
            ));
        }
    } else {
        findings.push(SpecAuditFinding::new(
            "conops-schema-interface-topology-missing",
            format!("Top-level schema properties in '{}' omits 'system_interface_topology'.", schema_path),
            schema_path,
            None,
        ));
    }

    // 2d. user_classes property structure (ISO 29148 §5.2.4, refs #291)
    if let Some(user_classes) = props.get("user_classes") {
        let is_arr = user_classes.get("type").and_then(|t| t.as_str()).map_or(false, |t| t == "array");
        let min_items = user_classes.get("minItems").and_then(|m| m.as_u64()).unwrap_or(0);

        let items_req: HashSet<&str> = user_classes
            .get("items")
            .and_then(|i| i.get("required"))
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        let has_req_fields = items_req.contains("id")
            && items_req.contains("title")
            && items_req.contains("player_or_operator")
            && items_req.contains("interfacing_stakeholder")
            && items_req.contains("characteristics")
            && items_req.contains("training_type")
            && items_req.contains("constraint_source");

        let id_pattern = user_classes
            .get("items")
            .and_then(|i| i.get("properties"))
            .and_then(|p| p.get("id"))
            .and_then(|id| id.get("pattern"))
            .and_then(|p| p.as_str());

        let has_ucl_pattern = id_pattern.map_or(false, |p| p == "^UCL-[0-9]{2}$");

        if !is_arr || min_items < 1 || !has_req_fields {
            findings.push(SpecAuditFinding::new(
                "conops-schema-user-classes-incomplete",
                format!("Property 'user_classes' in '{}' is incomplete; requires array with minItems >= 1 and items required fields (id, title, player_or_operator, interfacing_stakeholder, characteristics, training_type, constraint_source).", schema_path),
                schema_path,
                None,
            ));
        }

        if !has_ucl_pattern {
            findings.push(SpecAuditFinding::new(
                "conops-schema-user-classes-taxonomy-collision",
                format!("Property 'user_classes.items.properties.id.pattern' in '{}' must be '^UCL-[0-9]{{2}}$' to enforce unambiguous User Class taxonomy and prevent collision with Level 2 System Use Cases (refs #291).", schema_path),
                schema_path,
                None,
            ));
        }
    } else {
        findings.push(SpecAuditFinding::new(
            "conops-schema-user-classes-missing",
            format!("Top-level schema properties in '{}' omits 'user_classes'.", schema_path),
            schema_path,
            None,
        ));
    }

    findings
}


/// Audits top-level schema contract file on disk.
///
/// Preconditions: `schema_path` exists on disk.
/// Postconditions: Returns Ok(Vec<SpecAuditFinding>) detailing schema contract violations.
/// Algorithmic Complexity: O(B) where B is file byte length.
pub fn validate_conops_schema_file(schema_path: &Path) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    if !schema_path.is_file() {
        return Ok(Vec::new());
    }
    let content = match fs::read_to_string(schema_path) {
        Ok(c) => c,
        Err(e) => return Err(SpecAuditError::IoError(e.to_string())),
    };
    let json_val: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => return Err(SpecAuditError::RuleError(e.to_string())),
    };
    let path_str = schema_path.to_string_lossy().to_string();
    Ok(validate_conops_schema_contract(&json_val, &path_str))
}

/// Audits ConOps documents for User Class taxonomy disambiguation (`UCL-xx`) per ISO/IEC/IEEE 29148:2018 §5.2.4 (refs #291).
///
/// Preconditions: `content` is valid UTF-8.
/// Postconditions: Returns findings if any User Class uses legacy `UC-xx` prefix or collides with System Use Cases.
/// Algorithmic Complexity: O(T + L) where T is tables and L is lines count.
pub fn validate_user_classes(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();

    // 1. Audit Markdown Tables defining User Classes
    let tables = extract_markdown_tables(content);
    for tbl in &tables {
        let mut id_col_idx = None;
        for (col_idx, header) in tbl.headers.iter().enumerate() {
            let h_lower = header.to_lowercase();
            if h_lower.contains("user class id") || h_lower.contains("user class") {
                id_col_idx = Some(col_idx);
                break;
            }
        }

        if let Some(col) = id_col_idx {
            for row in &tbl.rows {
                if col < row.cells.len() {
                    let cell = &row.cells[col];
                    let raw_id = cell.trim()
                        .trim_matches('*')
                        .trim_matches('_')
                        .trim_matches('`')
                        .trim();
                    if raw_id.is_empty() {
                        continue;
                    }
                    if (raw_id.starts_with("UC-") || raw_id.starts_with("uc-"))
                        && !raw_id.starts_with("UCL-")
                        && !raw_id.starts_with("ucl-")
                    {
                        findings.push(SpecAuditFinding::new(
                            "conops-user-class-taxonomy-collision",
                            format!(
                                "User Class '{}' in '{}' at line {} uses legacy 'UC-xx' prefix colliding with Level 2 System Use Cases. User Classes must use unambiguous 'UCL-xx' prefix per ISO/IEC/IEEE 29148:2018 §5.2.4 (refs #291).",
                                raw_id, rel_path, row.line_no
                            ),
                            rel_path,
                            Some(row.line_no),
                        ));
                    }
                }
            }
        }
    }

    // 2. Scan prose and diagrams for colliding Level 2 System Use Case citations or legacy User Class citations (UC-xx)
    for (line_idx, line) in content.lines().enumerate() {
        let line_no = line_idx + 1;
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            continue;
        }

        // Avoid re-flagging rows already processed in table step
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            continue;
        }

        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let is_uc = (bytes[i] == b'U' || bytes[i] == b'u')
                && i + 2 < bytes.len()
                && (bytes[i + 1] == b'C' || bytes[i + 1] == b'c')
                && bytes[i + 2] == b'-';

            if is_uc {
                let left_ok = i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_' || bytes[i - 1] == b'-');
                if left_ok {
                    let digit_start = i + 3;
                    let mut j = digit_start;
                    while j < bytes.len() && bytes[j].is_ascii_digit() {
                        j += 1;
                    }
                    if j > digit_start {
                        let right_ok = j == bytes.len() || !(bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'-');
                        if right_ok {
                            let matched = &line[i..j];
                            findings.push(SpecAuditFinding::new(
                                "conops-user-class-taxonomy-collision",
                                format!(
                                    "Found colliding prefix '{}' in '{}' at line {}. ConOps documents must not cite legacy 'UC-xx' User Classes or Level 2 System Use Cases; use 'UCL-xx' for User Classes per ISO/IEC/IEEE 29148:2018 §5.2.4 (refs #291).",
                                    matched, rel_path, line_no
                                ),
                                rel_path,
                                Some(line_no),
                            ));
                            i = j;
                            continue;
                        }
                    }
                }
            }
            i += 1;
        }
    }

    findings
}

/// Audits an individual ConOps or Mission Intent specification against all Open Schema Contract requirements.
///
/// Preconditions: `doc` has been parsed, `content` is valid UTF-8.
/// Postconditions: Returns Ok(Vec<SpecAuditFinding>) containing all detected ConOps anomalies.
/// Algorithmic Complexity: O(N) where N is content byte length.
pub fn validate_conops(
    _doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let mut findings = Vec::new();

    // 1. Emergency Decision Matrix
    findings.append(&mut validate_emergency_decision_matrix(content, rel_path));

    // 2. PACE C2 Link Communications Plan
    findings.append(&mut validate_pace_plan(content, rel_path));

    // 3. Multi-Domain Threat Matrix
    findings.append(&mut validate_threat_matrix(content, rel_path));

    // 4. MoE / MoP formulations
    findings.append(&mut validate_moe_mop_metrics(content, rel_path));

    // 5. Operational Allocation Tags
    findings.append(&mut validate_operational_allocation_tags(content, rel_path));

    // 6. Super-System Operational Architecture (ISO 29148 §6.4.2 / IEEE 1362 §5.3)
    findings.append(&mut validate_super_system_architecture(content, rel_path));

    // 7. Subsystem Architecture (100% AST part coverage, ISO 29148 §6.4.3)
    findings.append(&mut validate_subsystem_architecture(content, rel_path));

    // 8. System Interface Topology & Port Taxonomy (ISO 29148 §6.4.3)
    findings.append(&mut validate_system_interface_topology(content, rel_path));

    // 9. User Class Taxonomy Disambiguation (ISO 29148 §5.2.4, refs #291)
    findings.append(&mut validate_user_classes(content, rel_path));

    Ok(findings)
}


/// Audits all ConOps and Mission Intent specifications under `docs/conops/` in the workspace.
///
/// If `docs/conops/` does not exist in the workspace, gracefully skips and returns Ok(Vec::new()).
///
/// Preconditions: `repo_root` is an accessible workspace path.
/// Postconditions: Returns Ok(Vec<SpecAuditFinding>) detailing any ConOps compliance violations across workspace.
/// Algorithmic Complexity: O(F * N) where F is file count and N is file byte length.
pub fn validate_conops_workspace(repo_root: &Path) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let conops_dir = repo_root.join("docs").join("conops");
    if !conops_dir.is_dir() {
        // Downstream workspace without ConOps directory: gracefully exempt
        return Ok(Vec::new());
    }

    let mut all_findings = Vec::new();

    fn scan_dir(dir: &Path, repo_root: &Path, findings: &mut Vec<SpecAuditFinding>) -> Result<(), SpecAuditError> {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => return Err(SpecAuditError::IoError(e.to_string())),
        };

        let mut paths = Vec::new();
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => return Err(SpecAuditError::IoError(e.to_string())),
            };
            paths.push(entry.path());
        }
        paths.sort();

        for path in paths {
            if path.is_dir() {
                scan_dir(&path, repo_root, findings)?;
            } else if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                let content = match fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => return Err(SpecAuditError::IoError(e.to_string())),
                };
                let rel_path = path.strip_prefix(repo_root).unwrap_or(&path).to_string_lossy().to_string();
                let doc = parse_markdown(&content)?;
                let mut f = validate_conops(&doc, &content, &rel_path)?;
                findings.append(&mut f);
            }
        }
        Ok(())
    }

    scan_dir(&conops_dir, repo_root, &mut all_findings)?;
    Ok(all_findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_extraction() {
        let content = r#"# Section Title

| Metric ID | Threshold | Objective | Unit |
| :--- | :--- | :--- | :--- |
| MoE-01 | 100.0 | 50.0 | ms |
| MoP-01 | 250.0 | 500.0 | Hz |
"#;
        let tables = extract_markdown_tables(content);
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].headers.len(), 4);
        assert_eq!(tables[0].rows.len(), 2);
        assert_eq!(tables[0].rows[0].cells[0], "MoE-01");
        assert_eq!(tables[0].rows[0].cells[3], "ms");
    }

    #[test]
    fn test_emergency_decision_matrix_valid_passes() {
        let content = r#"## 12. 7-Row Emergency Decision & Contingency Matrix
| Trigger ID | Contingency Trigger | Failsafe State |
| :--- | :--- | :--- |
| `EMG-01` | Lost C2 Link | Failsafe_Hold |
| `EMG-02` | GNSS Loss | Dead_Reckoning |
| `EMG-03` | Power Failure | Safe_Divert |
| `EMG-04` | Sensor Fault | Secondary_Observer |
| `EMG-05` | Geofence Breach | Return_To_Base |
| `EMG-06` | Actuation Anomaly | Emergency_Stop |
| `EMG-07` | Abort Command | Impact_Safe |
"#;
        let findings = validate_emergency_decision_matrix(content, "docs/conops/CONOPS.md");
        assert!(findings.is_empty(), "Expected passing emergency matrix, got: {:?}", findings);
    }

    #[test]
    fn test_emergency_decision_matrix_missing_triggers_fails() {
        let content = r#"## 12. 7-Row Emergency Decision & Contingency Matrix
| Trigger ID | Contingency Trigger |
| :--- | :--- |
| `EMG-01` | Lost C2 Link |
| `EMG-02` | GNSS Loss |
"#;
        let findings = validate_emergency_decision_matrix(content, "docs/conops/CONOPS.md");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "conops-emergency-matrix-incomplete");
        assert!(findings[0].message.contains("EMG-03"));
        assert!(findings[0].message.contains("EMG-07"));
    }

    #[test]
    fn test_pace_plan_valid_passes() {
        let content = r#"## 5. PACE C2 Link Communications Plan
| PACE Tier | Link Medium |
| :--- | :--- |
| **Primary** | 2.4 GHz COFDM |
| **Alternate** | 5.8 GHz Telemetry |
| **Contingency** | 915 MHz Long-Range |
| **Emergency** | 433 MHz Independent Kill Switch |
"#;
        let findings = validate_pace_plan(content, "docs/conops/MISSION_INTENT.md");
        assert!(findings.is_empty(), "Expected passing PACE plan, got: {:?}", findings);
    }

    #[test]
    fn test_pace_plan_missing_tiers_fails() {
        let content = r#"## 5. PACE C2 Link Communications Plan
| PACE Tier | Link Medium |
| :--- | :--- |
| **Primary** | 2.4 GHz COFDM |
| **Alternate** | 5.8 GHz Telemetry |
"#;
        let findings = validate_pace_plan(content, "docs/conops/MISSION_INTENT.md");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "conops-pace-plan-incomplete");
        assert!(findings[0].message.contains("contingency"));
        assert!(findings[0].message.contains("emergency"));
    }

    #[test]
    fn test_threat_matrix_all_domains_passes() {
        let content = r#"## 4. Multi-Domain Operational Threat & Contested Environment Matrix
- Kinetic: Ballistic projectile and collision mitigation
- Mechanical: Structural flutter and actuator jam detection
- Power/Thermal: Battery over-temperature and thermal runaway isolation
- Environmental: Atmospheric icing and gust response
- EW/Cyber: RF jamming and cybersecurity packet injection defenses
- Optical: Laser blinding and camera saturation filters
- Signature/Acoustic: Low-noise propeller and acoustic signature suppression
- Human Factors: Operator fatigue and command disparity validation
- CBRN: Toxic particulate filter and hazardous contamination ingress sealing
"#;
        let findings = validate_threat_matrix(content, "docs/conops/MISSION_INTENT.md");
        assert!(findings.is_empty(), "Expected all threat domains present, got: {:?}", findings);
    }

    #[test]
    fn test_threat_matrix_missing_domain_fails() {
        let content = r#"## 4. Multi-Domain Operational Threat & Contested Environment Matrix
- Kinetic: Ballistic projectile
- Mechanical: Structural vibration
- Power/Thermal: Battery thermal
- Environmental: Weather gust
"#;
        let findings = validate_threat_matrix(content, "docs/conops/MISSION_INTENT.md");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "conops-threat-matrix-missing-domain");
        assert!(findings[0].message.contains("CBRN"));
        assert!(findings[0].message.contains("Optical"));
    }

    #[test]
    fn test_moe_mop_metrics_validation() {
        let valid_content = r#"## 3. Measures of Effectiveness (MoE) & Measures of Performance (MoP) Metrics
| Metric ID | Formulation | Threshold | Objective | Unit |
| :--- | :--- | :--- | :--- | :--- |
| MoE-01 | Latency equation | 100.0 | 50.0 | ms |
| MoP-01 | Loop frequency | 250.0 | 500.0 | Hz |
"#;
        let findings = validate_moe_mop_metrics(valid_content, "docs/conops/MISSION_INTENT.md");
        assert!(findings.is_empty(), "Valid MoE/MoP should pass, got: {:?}", findings);

        let invalid_content = r#"## 3. Measures of Effectiveness (MoE) & Measures of Performance (MoP) Metrics
| Metric ID | Formulation | Threshold | Objective | Unit |
| :--- | :--- | :--- | :--- | :--- |
| MoE-01 | Latency equation | {{TODO}} | 50.0 | ms |
| MoP-01 | Loop frequency | 250.0 | TBD | unknown_unit |
"#;
        let invalid_findings = validate_moe_mop_metrics(invalid_content, "docs/conops/MISSION_INTENT.md");
        assert!(invalid_findings.iter().any(|f| f.rule_id == "conops-moe-mop-missing-threshold-objective"));
        assert!(invalid_findings.iter().any(|f| f.rule_id == "conops-moe-mop-missing-unit"));
    }

    #[test]
    fn test_operational_allocation_tags_validation() {
        let valid_content = r#"# Tactical Mission Intent
## Operational Activities
| Activity ID | Name |
| :--- | :--- |
| OA-01 | Autonomous Trajectory Control |
| MET-01 | Execute Precision Ingress |

/// OperationalAllocation: [OA-01]
/// OperationalAllocation: [MET-01]
"#;
        let findings = validate_operational_allocation_tags(valid_content, "docs/conops/MISSION_INTENT.md");
        assert!(findings.is_empty(), "Valid operational allocations should pass: {:?}", findings);

        let unallocated_content = r#"# Tactical Mission Intent
## Operational Activities
| Activity ID | Name |
| :--- | :--- |
| OA-01 | Autonomous Trajectory Control |
| OA-02 | Failsafe Containment |

/// OperationalAllocation: [OA-01]
"#;
        let unallocated_findings = validate_operational_allocation_tags(unallocated_content, "docs/conops/MISSION_INTENT.md");
        assert_eq!(unallocated_findings.len(), 1);
        assert_eq!(unallocated_findings[0].rule_id, "conops-operational-allocation-missing");
        assert!(unallocated_findings[0].message.contains("OA-02"));

        let malformed_content = r#"# Tactical Mission Intent
## Operational Activities
| Activity ID | Name |
| :--- | :--- |
| OA-01 | Autonomous Trajectory Control |

/// OperationalAllocation: OA-01
"#;
        let malformed_findings = validate_operational_allocation_tags(malformed_content, "docs/conops/MISSION_INTENT.md");
        assert!(malformed_findings.iter().any(|f| f.rule_id == "conops-operational-allocation-malformed"));
    }

    #[test]
    fn test_issue_256_conops_schema_rejects_missing_super_system_and_subsystem_architecture() {
        let invalid_schema = serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "required": [
                "metadata",
                "operational_context",
                "deficiencies",
                "proposed_capabilities",
                "user_classes"
            ],
            "properties": {
                "metadata": { "type": "object" },
                "operational_context": { "type": "object" }
            }
        });
        let findings = validate_conops_schema_contract(&invalid_schema, ".pipeline/schemas/conops_specification_schema.json");
        assert!(!findings.is_empty(), "Schema contract must reject omission of super_system_architecture and subsystem_architecture");
        assert!(findings.iter().any(|f| f.rule_id == "conops-schema-super-system-architecture-missing"));
        assert!(findings.iter().any(|f| f.rule_id == "conops-schema-subsystem-architecture-missing"));
        assert!(findings.iter().any(|f| f.rule_id == "conops-schema-interface-topology-missing"));
    }

    #[test]
    fn test_issue_256_conops_doc_rejects_missing_super_system_and_subsystem_architecture() {
        let content = r#"# Concept of Operations (ConOps)
## 1. Operational Context
System boundaries and context.

## 2. Operational Deficiencies
Current gaps and issues.
"#;
        let super_findings = validate_super_system_architecture(content, "docs/conops/CONOPS.md");
        assert!(!super_findings.is_empty(), "Must reject ConOps doc missing super-system architecture");
        assert_eq!(super_findings[0].rule_id, "conops-super-system-architecture-missing");

        let subsys_findings = validate_subsystem_architecture(content, "docs/conops/CONOPS.md");
        assert!(!subsys_findings.is_empty(), "Must reject ConOps doc missing subsystem architecture");
        assert_eq!(subsys_findings[0].rule_id, "conops-subsystem-architecture-missing");

        let topo_findings = validate_system_interface_topology(content, "docs/conops/CONOPS.md");
        assert!(!topo_findings.is_empty(), "Must reject ConOps doc missing interface topology");
        assert_eq!(topo_findings[0].rule_id, "conops-system-interface-topology-missing");
    }

    #[test]
    fn test_issue_256_conops_schema_valid_contract_passes() {
        let valid_schema = serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "required": [
                "metadata",
                "super_system_architecture",
                "subsystem_architecture",
                "system_interface_topology",
                "user_classes"
            ],
            "properties": {
                "super_system_architecture": {
                    "type": "object",
                    "required": ["segments", "boundary_diagram_mermaid"],
                    "properties": {
                        "segments": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["segment_id", "segment_name", "operational_role"]
                            }
                        },
                        "boundary_diagram_mermaid": { "type": "string" }
                    }
                },
                "subsystem_architecture": {
                    "type": "object",
                    "required": ["subsystems", "ast_part_allocations"],
                    "properties": {
                        "subsystems": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["subsystem_id", "subsystem_name", "ast_part_ref", "functional_description"]
                            }
                        },
                        "ast_part_allocations": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    }
                },
                "system_interface_topology": {
                    "type": "object",
                    "required": ["interface_links"],
                    "properties": {
                        "interface_links": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["link_id", "source_subsystem", "target_subsystem", "protocol_type"]
                            }
                        }
                    }
                },
                "user_classes": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": [
                            "id",
                            "title",
                            "player_or_operator",
                            "interfacing_stakeholder",
                            "characteristics",
                            "training_type",
                            "constraint_source"
                        ],
                        "properties": {
                            "id": {
                                "type": "string",
                                "pattern": "^UCL-[0-9]{2}$"
                            }
                        }
                    }
                }
            }
        });

        let findings = validate_conops_schema_contract(&valid_schema, ".pipeline/schemas/conops_specification_schema.json");
        assert!(findings.is_empty(), "Valid schema contract must pass with zero findings, got: {:?}", findings);
    }

    #[test]
    fn test_issue_256_conops_doc_valid_architecture_passes() {
        let valid_conops = r#"# Concept of Operations (ConOps)
## 4. System Operational Architecture & Physical Subsystem Decomposition

### 4.1 Super-System Operational Architecture & Segment Boundaries
In accordance with IEEE Std 1362-1998 §5.3, ISO/IEC/IEEE 29148:2018 §6.4.3, and DoDAF 2.02 SV-1, the operational system is partitioned across major physical segments:
- Primary Operational Segment (SEG-01)
- Supervisory Control Segment (SEG-02)
- Auxiliary Support Segment (SEG-03)

```mermaid
flowchart TD
    subgraph External_Actors["External Operating Environment & Actors (IEEE 1362 §5.1)"]
        direction TB
        Operator["Operator"]
    end
    subgraph Supervisory_Segment["Supervisory Control Segment (IEEE 1362 §5.3)"]
        direction TB
        GCS["Ground Control Station<br/>• PORT-SUP-C2 (INOUT)"]
    end
    subgraph Core_Execution_Segment["Core System Execution Segment (DoDAF SV-1)"]
        direction TB
        Platform["Core Platform<br/>• PORT-CTRL-C2 (INOUT)"]
    end
    Operator -->|"CONN-01: Supervisory Command"| GCS
    GCS -->|"CONN-02: System Datalink"| Platform
```

### 4.3 Subsystem Architecture (100% AST Part Coverage)
Decomposition of subsystems derived deterministically from AST part definitions:
- Primary Controller | SysML v2: `part def FlightComputer` | SSOT Ground Truth Mapping: schema/model.sysml
Functional Purpose & Scope: Core flight guidance.

### 4.4 Port Taxonomy & Interface Interconnects
Taxonomy of interface interconnects:
- PORT-C2-IN (IN)
- PORT-TLM-OUT (OUT)
"#;
        let super_f = validate_super_system_architecture(valid_conops, "docs/conops/CONOPS.md");
        assert!(super_f.is_empty(), "Valid super-system architecture must pass: {:?}", super_f);

        let subsys_f = validate_subsystem_architecture(valid_conops, "docs/conops/CONOPS.md");
        assert!(subsys_f.is_empty(), "Valid subsystem architecture must pass: {:?}", subsys_f);

        let topo_f = validate_system_interface_topology(valid_conops, "docs/conops/CONOPS.md");
        assert!(topo_f.is_empty(), "Valid interface topology must pass: {:?}", topo_f);
    }

    #[test]
    fn test_issue_256_live_conops_schema_file_passes() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir.parent().and_then(|p| p.parent()).unwrap();
        let schema_path = root.join(".pipeline").join("schemas").join("conops_specification_schema.json");
        if schema_path.is_file() {
            let findings = validate_conops_schema_file(&schema_path).expect("Failed to parse live schema file");
            assert!(findings.is_empty(), "Live .pipeline/schemas/conops_specification_schema.json must pass contract validation: {:?}", findings);
        }
    }

    #[test]
    fn test_issue_267_unconstrained_diagram_lacking_ieee_1362_and_dodaf_sv1_fails() {
        let unconstrained_conops = r#"# Concept of Operations (ConOps)
## 4. System Operational Architecture & Physical Subsystem Decomposition

### 4.1 Super-System Operational Architecture & Segment Boundaries
Operational system is partitioned across major physical segments:
- Primary Operational Segment (SEG-01)
- Supervisory Control Segment (SEG-02)

```mermaid
flowchart TD
    GCS["Ground Control Station"] --> Platform["Core Platform"]
```

### 4.3 Subsystem Architecture (100% AST Part Coverage)
Decomposition of subsystems derived deterministically from AST part definitions:
- Primary Controller | SysML v2: `part def FlightComputer` | SSOT Ground Truth Mapping: schema/model.sysml
Functional Purpose & Scope: Core flight guidance.

### 4.4 Port Taxonomy & Interface Interconnects
Taxonomy of interface interconnects:
- PORT-C2-IN (IN)
- PORT-TLM-OUT (OUT)
"#;
        let findings = validate_super_system_architecture(unconstrained_conops, "docs/conops/CONOPS.md");
        assert!(
            !findings.is_empty(),
            "Unconstrained architecture diagram lacking IEEE 1362 and DoDAF SV-1 must fail quality gate"
        );
        let codes: Vec<&str> = findings.iter().map(|f| f.rule_id.as_str()).collect();
        assert!(
            codes.contains(&"conops-super-system-standards-governance-missing"),
            "Expected finding 'conops-super-system-standards-governance-missing', got: {:?}",
            codes
        );
        assert!(
            codes.contains(&"conops-super-system-ports-missing"),
            "Expected finding 'conops-super-system-ports-missing', got: {:?}",
            codes
        );
        assert!(
            codes.contains(&"conops-super-system-connections-missing"),
            "Expected finding 'conops-super-system-connections-missing', got: {:?}",
            codes
        );
    }

    #[test]
    fn test_issue_267_compliant_diagram_with_ieee_1362_and_dodaf_sv1_passes() {
        let compliant_conops = r#"# Concept of Operations (ConOps)
## 4. System Operational Architecture & Physical Subsystem Decomposition

### 4.1 Super-System Operational Architecture & Segment Boundaries
In accordance with IEEE Std 1362-1998 §5.3, ISO/IEC/IEEE 29148:2018 §6.4.3, and DoDAF 2.02 SV-1, the operational system is partitioned across major physical segments:
- Primary Operational Segment (SEG-01)
- Supervisory Control Segment (SEG-02)

```mermaid
flowchart TD
    subgraph External_Actors["External Operating Environment & Actors (IEEE 1362 §5.1)"]
        direction TB
        Operator["Operator"]
    end
    subgraph Supervisory_Segment["Supervisory Segment (IEEE 1362 §5.3)"]
        direction TB
        GCS["Ground Control Station<br/>• PORT-SUP-C2 (INOUT)"]
    end
    subgraph Core_Execution_Segment["Core System Execution Segment (DoDAF SV-1)"]
        direction TB
        Platform["Core Platform<br/>• PORT-CTRL-C2 (INOUT)"]
    end
    Operator -->|"CONN-01: Supervisory Command"| GCS
    GCS -->|"CONN-02: Datalink"| Platform
```

### 4.3 Subsystem Architecture (100% AST Part Coverage)
Decomposition of subsystems derived deterministically from AST part definitions:
- Primary Controller | SysML v2: `part def FlightComputer` | SSOT Ground Truth Mapping: schema/model.sysml
Functional Purpose & Scope: Core flight guidance.

### 4.4 Port Taxonomy & Interface Interconnects
Taxonomy of interface interconnects:
- PORT-C2-IN (IN)
- PORT-TLM-OUT (OUT)
"#;
        let findings = validate_super_system_architecture(compliant_conops, "docs/conops/CONOPS.md");
        assert!(
            findings.is_empty(),
            "Compliant architecture diagram with IEEE 1362 and DoDAF SV-1 must pass quality gate, got: {:?}",
            findings
        );
    }

    #[test]
    fn test_issue_277_monolithic_nested_subgraph_cross_hierarchy_edges_detected_and_rejected() {
        let hairball_conops = r#"# Concept of Operations (ConOps)
## 4. System Operational Architecture & Physical Subsystem Decomposition

### 4.1 Super-System Operational Architecture & Segment Boundaries
In accordance with IEEE Std 1362-1998 §5.3, ISO/IEC/IEEE 29148:2018 §6.4.3, and DoDAF 2.02 SV-1:

```mermaid
flowchart TD
    subgraph Super_System["Super-System (IEEE 1362)"]
        direction TB
        subgraph External_Actors["External Actors"]
            direction TB
            Operator["Operator<br/>• PORT_OP (OUT)"]
        end
        subgraph Supervisory_Segment["Supervisory Segment"]
            direction TB
            GCS["Ground Control Station<br/>• PORT_SUP (INOUT)"]
        end
        subgraph Platform_Segment["Platform Segment"]
            direction TB
            subgraph Tier_1["Tier 1 - Primary Flight"]
                direction TB
                Controller["Flight Computer<br/>• PORT_CTRL (INOUT)"]
            end
            subgraph Tier_2["Tier 2 - Actuation"]
                direction TB
                Actuator["Actuator Core<br/>• PORT_ACT (IN)"]
            end
        end
    end
    Operator -->|"CONN-01: Command"| GCS
    GCS -->|"CONN-02: Datalink"| Controller
    Controller -->|"CONN-03: Actuation Demand"| Actuator
```

### 4.3 Subsystem Architecture (100% AST Part Coverage)
Decomposition of subsystems:
- Controller | SysML v2: `part def FlightComputer` | SSOT Ground Truth Mapping: schema/model.sysml
Functional Purpose & Scope: Core guidance.

### 4.4 Port Taxonomy & Interface Interconnects
Taxonomy of interface interconnects:
- PORT-C2-IN (IN)
- PORT-TLM-OUT (OUT)
"#;
        let findings = validate_super_system_architecture(hairball_conops, "docs/conops/CONOPS.md");
        assert!(
            findings.iter().any(|f| f.rule_id == "conops-super-system-modular-edge-routing-missing"),
            "Monolithic SV-1 diagram with 3-level nested subgraphs and cross-hierarchy edge routing into internal tiers must be rejected with 'conops-super-system-modular-edge-routing-missing' finding. Got findings: {:?}",
            findings
        );
    }

    #[test]
    fn test_issue_277_modularized_boundary_edge_routing_passes() {
        let modular_conops = r#"# Concept of Operations (ConOps)
## 4. System Operational Architecture & Physical Subsystem Decomposition

### 4.1 Super-System Operational Architecture & Segment Boundaries
In accordance with IEEE Std 1362-1998 §5.3, ISO/IEC/IEEE 29148:2018 §6.4.3, and DoDAF 2.02 SV-1:

```mermaid
flowchart TD
    subgraph Supervisory_Segment["Supervisory Segment"]
        direction TB
        GCS["Ground Control Station<br/>• PORT_SUP (INOUT)"]
    end
    subgraph Platform_Segment["Platform Segment"]
        direction TB
        Controller["Flight Computer<br/>• PORT_CTRL (INOUT)"]
        Actuator["Actuator Core<br/>• PORT_ACT (IN)"]
    end
    GCS -->|"CONN-01: Datalink"| Controller
    Controller -->|"CONN-02: Actuation Demand"| Actuator
```
"#;
        let findings = validate_super_system_architecture(modular_conops, "docs/conops/CONOPS.md");
        assert!(
            !findings.iter().any(|f| f.rule_id == "conops-super-system-modular-edge-routing-missing"),
            "Modular SV-1 diagram with single-level segment subgraphs must pass Rule E5 validation. Got findings: {:?}",
            findings
        );
    }

    #[test]
    fn test_issue_291_conops_schema_rejects_user_class_uc_pattern_collision() {
        let invalid_schema = serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "required": [
                "metadata",
                "super_system_architecture",
                "subsystem_architecture",
                "system_interface_topology",
                "user_classes"
            ],
            "properties": {
                "super_system_architecture": {
                    "type": "object",
                    "required": ["segments", "boundary_diagram_mermaid"],
                    "properties": {
                        "segments": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["segment_id", "segment_name", "operational_role"]
                            }
                        },
                        "boundary_diagram_mermaid": { "type": "string" }
                    }
                },
                "subsystem_architecture": {
                    "type": "object",
                    "required": ["subsystems", "ast_part_allocations"],
                    "properties": {
                        "subsystems": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["subsystem_id", "subsystem_name", "ast_part_ref", "functional_description"]
                            }
                        },
                        "ast_part_allocations": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    }
                },
                "system_interface_topology": {
                    "type": "object",
                    "required": ["interface_links"],
                    "properties": {
                        "interface_links": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["link_id", "source_subsystem", "target_subsystem", "protocol_type"]
                            }
                        }
                    }
                },
                "user_classes": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": [
                            "id", "title", "player_or_operator", "interfacing_stakeholder",
                            "characteristics", "training_type", "constraint_source"
                        ],
                        "properties": {
                            "id": {
                                "type": "string",
                                "pattern": "^UC-[0-9]{2}$"
                            }
                        }
                    }
                }
            }
        });
        let findings = validate_conops_schema_contract(&invalid_schema, ".pipeline/schemas/conops_specification_schema.json");
        assert!(
            findings.iter().any(|f| f.rule_id == "conops-schema-user-classes-taxonomy-collision"),
            "Schema with legacy ^UC-[0-9]{{2}}$ pattern must be rejected with 'conops-schema-user-classes-taxonomy-collision' (refs #291)"
        );
    }

    #[test]
    fn test_issue_291_conops_doc_rejects_user_class_uc_collision() {
        let colliding_conops = r#"# Concept of Operations (ConOps)
## 4. User Classes & Stakeholder Taxonomy
| User Class ID | Title | Player or Operator | Interfacing Stakeholder | Characteristics & Responsibilities | Training & Qualification | Constraint Source |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **UC-01** | System Operator (SO) | Direct Operator | Safety Authority | Supervision oversight | Type Qualification | ISO 29148 |
| **UC-04** | Maintenance Technician (MT) | Support Operator | Maintenance Depot | Pre-sortie inspections | Certified Tech | ISO 29148 |

Operational handover is executed between incoming operator and outgoing operator (UC-01).
"#;
        let findings = validate_user_classes(colliding_conops, "docs/conops/04_USER_CLASSES.md");
        assert!(
            findings.iter().any(|f| f.rule_id == "conops-user-class-taxonomy-collision" && f.message.contains("UC-01")),
            "ConOps declaring User Class UC-01 must be rejected with taxonomy collision finding (refs #291)"
        );
        assert!(
            findings.iter().any(|f| f.rule_id == "conops-user-class-taxonomy-collision" && f.message.contains("UC-04")),
            "ConOps declaring User Class UC-04 must be rejected with taxonomy collision finding (refs #291)"
        );
    }

    #[test]
    fn test_issue_291_conops_doc_ucl_taxonomy_passes() {
        let compliant_conops = r#"# Concept of Operations (ConOps)
## 4. User Classes & Stakeholder Taxonomy
| User Class ID | Title | Player or Operator | Interfacing Stakeholder | Characteristics & Responsibilities | Training & Qualification | Constraint Source |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **UCL-01** | System Operator (SO) | Direct Operator | Safety Authority | Supervision oversight | Type Qualification | ISO 29148 |
| **UCL-04** | Maintenance Technician (MT) | Support Operator | Maintenance Depot | Pre-sortie inspections | Certified Tech | ISO 29148 |

Operational handover is executed between incoming operator and outgoing operator (UCL-01).
"#;
        let findings = validate_user_classes(compliant_conops, "docs/conops/04_USER_CLASSES.md");
        assert!(
            findings.is_empty(),
            "ConOps with unambiguous UCL-xx taxonomy must pass with zero findings, got: {:?}",
            findings
        );
    }
}



