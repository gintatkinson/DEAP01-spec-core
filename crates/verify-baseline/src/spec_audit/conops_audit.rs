//! Native Rust ConOps Completeness & Mission Intent Integrity Engine.
//!
//! Conforms to DO-178C Level A and ISO 26262 ASIL D safety standards.

use crate::spec_audit::markdown_ast::{parse_markdown, MarkdownDoc};
use crate::spec_audit::{SpecAuditError, SpecAuditFinding};
use std::fs;
use std::path::Path;

pub use crate::spec_audit::conops_architecture::{
    audit_modular_subgraph_edge_routing, validate_subsystem_architecture,
    validate_super_system_architecture, validate_system_interface_topology,
};
pub use crate::spec_audit::conops_matrices::{
    validate_emergency_decision_matrix, validate_moe_mop_metrics,
    validate_operational_allocation_tags, validate_pace_plan, validate_threat_matrix,
};
pub use crate::spec_audit::conops_schema::{
    validate_conops_schema_contract, validate_conops_schema_file, validate_user_classes,
};

/// Canonical emergency contingency triggers required in Section 12 Emergency Decision Matrix.
pub const CANONICAL_EMERGENCY_TRIGGERS: &[&str] = &[
    "EMG-01", "EMG-02", "EMG-03", "EMG-04", "EMG-05", "EMG-06", "EMG-07",
];

/// Mandatory C2 communication tiers required in PACE plan.
pub const PACE_TIERS: &[&str] = &["primary", "alternate", "contingency", "emergency"];

/// Threat domain specification containing domain name and matching keywords.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-THREATDOMAINSPEC]
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
        keywords: &[
            "power/thermal",
            "power and thermal",
            "power",
            "thermal",
            "thr-pwr",
            "thr-thm",
        ],
    },
    ThreatDomainSpec {
        name: "Environmental",
        keywords: &[
            "environmental",
            "atmospheric",
            "weather",
            "icing",
            "precipitation",
            "gust",
            "thr-env",
        ],
    },
    ThreatDomainSpec {
        name: "EW/Cyber",
        keywords: &[
            "ew",
            "electronic warfare",
            "electromagnetic",
            "rf jamming",
            "gnss jamming",
            "cyber",
            "cybersecurity",
            "packet injection",
            "firmware tampering",
            "thr-ew",
            "thr-cyb",
            "thr-ewc",
        ],
    },
    ThreatDomainSpec {
        name: "Optical",
        keywords: &["optical", "laser", "dazzling", "camera saturation", "thr-opt"],
    },
    ThreatDomainSpec {
        name: "Signature/Acoustic",
        keywords: &[
            "signature",
            "acoustic",
            "infrared",
            "rcs",
            "radar cross-section",
            "thr-sig",
            "thr-ac",
        ],
    },
    ThreatDomainSpec {
        name: "Human Factors",
        keywords: &[
            "human factors",
            "human",
            "operator fatigue",
            "pilot",
            "input disparity",
            "thr-hum",
        ],
    },
    ThreatDomainSpec {
        name: "CBRN",
        keywords: &[
            "cbrn",
            "chemical",
            "biological",
            "radiological",
            "nuclear",
            "toxic",
            "contamination",
            "thr-cbrn",
        ],
    },
];

/// Recognized SI and engineering units for MoE / MoP formulations.
pub const MOE_MOP_RECOGNIZED_UNITS: &[&str] = &[
    "s", "sec", "second", "seconds", "ms", "us", "µs", "ns", "min", "h", "hr", "hz", "khz", "mhz",
    "ghz", "m", "km", "cm", "mm", "nm", "um", "µm", "m/s", "km/h", "m/s^2", "m/s2", "fps", "rpm",
    "kg", "g", "mg", "j", "kj", "mj", "wh", "kwh", "w", "kw", "mw", "v", "mv", "kv", "a", "ma",
    "deg", "rad", "degree", "degrees", "radian", "radians", "k", "db", "dbm", "dbi", "%",
    "percent", "ratio", "dimensionless", "count", "bps", "kbps", "mbps", "gbps", "b/s", "kb/s",
    "mb/s", "gb/s", "b", "kb", "mb", "gb", "bytes", "ppm",
];

/// Represents an individual parsed Markdown table row.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-TABLEROW]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow {
    pub cells: Vec<String>,
    pub line_no: usize,
}

/// Represents an extracted Markdown table.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-MARKDOWNTABLE]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownTable {
    pub headers: Vec<String>,
    pub rows: Vec<TableRow>,
    pub start_line: usize,
}

/// Extracts all CommonMark pipe tables from content without using regular expressions.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-EXTRACT-MARKDOWN-TABLES]
pub fn extract_markdown_tables(content: &str) -> Vec<MarkdownTable> {
    crate::cmark_gates::spec_audit::extract_conops_tables_cmark(content)
}

/// Checks whether `docs/conops/` exists on disk.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-CHECK-CONOPS-DIR-EXISTS]
pub fn check_conops_dir_exists(repo_root: &Path) -> bool {
    repo_root.join("docs").join("conops").is_dir()
}

/// Audits an individual ConOps or Mission Intent specification against all Open Schema Contract requirements.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-CONOPS]
pub fn validate_conops(
    _doc: &MarkdownDoc,
    content: &str,
    rel_path: &str,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let mut findings = Vec::new();
    findings.extend(validate_emergency_decision_matrix(content, rel_path));
    findings.extend(validate_pace_plan(content, rel_path));
    findings.extend(validate_threat_matrix(content, rel_path));
    findings.extend(validate_moe_mop_metrics(content, rel_path));
    findings.extend(validate_operational_allocation_tags(content, rel_path));
    findings.extend(validate_super_system_architecture(content, rel_path));
    findings.extend(validate_subsystem_architecture(content, rel_path));
    findings.extend(validate_system_interface_topology(content, rel_path));
    findings.extend(validate_user_classes(content, rel_path));
    Ok(findings)
}

/// Audits all ConOps and Mission Intent specifications under `docs/conops/` in the workspace.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-CONOPS-WORKSPACE]
pub fn validate_conops_workspace(
    repo_root: &Path,
) -> Result<Vec<SpecAuditFinding>, SpecAuditError> {
    let conops_dir = repo_root.join("docs").join("conops");
    if !conops_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut all_findings = Vec::new();
    scan_conops_dir(&conops_dir, repo_root, &mut all_findings)?;
    Ok(all_findings)
}

/// Recursively scans `dir` for `.md` ConOps documents.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-SCAN-DIR]
fn scan_conops_dir(
    dir: &Path,
    repo_root: &Path,
    findings: &mut Vec<SpecAuditFinding>,
) -> Result<(), SpecAuditError> {
    let entries = fs::read_dir(dir).map_err(|e| SpecAuditError::IoError(e.to_string()))?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| SpecAuditError::IoError(e.to_string()))?;
        paths.push(entry.path());
    }
    paths.sort();

    for path in paths {
        if path.is_dir() {
            scan_conops_dir(&path, repo_root, findings)?;
        } else if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            let content =
                fs::read_to_string(&path).map_err(|e| SpecAuditError::IoError(e.to_string()))?;
            let rel_path = path
                .strip_prefix(repo_root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            let doc = parse_markdown(&content)?;
            findings.extend(validate_conops(&doc, &content, &rel_path)?);
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "conops_tests_a.rs"]
mod tests_a;
#[cfg(test)]
#[path = "conops_tests_b.rs"]
mod tests_b;
#[cfg(test)]
#[path = "conops_tests_c.rs"]
mod tests_c;
