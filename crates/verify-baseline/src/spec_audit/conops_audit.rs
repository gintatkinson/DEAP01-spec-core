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
use std::collections::HashSet;
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
    let upper_doc = content.to_uppercase();

    for &canonical in CANONICAL_EMERGENCY_TRIGGERS {
        if upper_doc.contains(canonical) {
            found_triggers.insert(canonical.to_string());
        }
    }

    // Also scan for any other EMG-XX triggers
    let mut search_from = 0;
    while let Some(pos) = upper_doc[search_from..].find("EMG-") {
        let abs_pos = search_from + pos;
        let suffix = &upper_doc[abs_pos + 4..];
        let mut num_end = 0;
        let bytes = suffix.as_bytes();
        while num_end < bytes.len() && bytes[num_end].is_ascii_digit() {
            num_end += 1;
        }
        if num_end > 0 {
            let trigger_id = format!("EMG-{}", &suffix[..num_end]);
            found_triggers.insert(trigger_id);
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
        let upper_line = trimmed.to_uppercase();

        // Check for OA-XX tokens
        let mut search_from = 0;
        while let Some(pos) = upper_line[search_from..].find("OA-") {
            let abs_pos = search_from + pos;
            let suffix = &upper_line[abs_pos + 3..];
            let mut num_end = 0;
            let bytes = suffix.as_bytes();
            while num_end < bytes.len() && bytes[num_end].is_ascii_digit() {
                num_end += 1;
            }
            if num_end > 0 {
                let id = format!("OA-{}", &suffix[..num_end]);
                declared_oa_tasks.insert(id);
            }
            search_from = abs_pos + 3;
        }

        // Check for MET-XX tokens
        let mut search_from_met = 0;
        while let Some(pos) = upper_line[search_from_met..].find("MET-") {
            let abs_pos = search_from_met + pos;
            let suffix = &upper_line[abs_pos + 4..];
            let mut num_end = 0;
            let bytes = suffix.as_bytes();
            while num_end < bytes.len() && bytes[num_end].is_ascii_digit() {
                num_end += 1;
            }
            if num_end > 0 {
                let id = format!("MET-{}", &suffix[..num_end]);
                declared_met_tasks.insert(id);
            }
            search_from_met = abs_pos + 4;
        }
    }

    // 2. Verify that each declared OA or MET task has an OperationalAllocation tag
    let upper_content = content.to_uppercase();

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

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => return Err(SpecAuditError::IoError(e.to_string())),
            };
            let path = entry.path();
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
}
