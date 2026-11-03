//! ConOps matrix and metric validators (Emergency Decision, PACE, Threat Matrix, MoE/MoP, Operational Allocation).

use crate::spec_audit::conops_audit::{
    extract_markdown_tables, MarkdownTable, CANONICAL_EMERGENCY_TRIGGERS, MANDATORY_THREAT_DOMAINS,
    MOE_MOP_RECOGNIZED_UNITS, PACE_TIERS,
};
use crate::spec_audit::SpecAuditFinding;
use std::collections::BTreeSet;

/// Extracts all tokens matching `PREFIX<digits>` (e.g., `EMG-01`, `OA-01`, `MET-01`) from `upper_text`.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-EMERGENCY-DECISION-MATRIX]
fn extract_prefixed_ids(upper_text: &str, prefix: &str, out: &mut BTreeSet<String>) {
    let mut search_from = 0;
    while let Some(slice) = upper_text.get(search_from..) {
        let Some(pos) = slice.find(prefix) else {
            break;
        };
        let abs_pos = search_from + pos;
        let suffix = upper_text.get(abs_pos + prefix.len()..).unwrap_or("");
        let num_end = suffix
            .as_bytes()
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count();
        if num_end > 0 {
            if let Some(digits) = suffix.get(..num_end) {
                out.insert(format!("{}{}", prefix, digits));
            }
        }
        search_from = abs_pos + prefix.len();
    }
}

/// Audits Section 12 Emergency Decision Matrix for complete coverage of canonical emergency triggers ($N \ge 7$).
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-EMERGENCY-DECISION-MATRIX]
pub fn validate_emergency_decision_matrix(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let lower_doc = content.to_lowercase();

    let has_emergency_section = lower_doc.contains("emergency decision")
        || lower_doc.contains("contingency matrix")
        || lower_doc.contains("7-row emergency decision")
        || lower_doc.contains("emg-01");
    if !has_emergency_section {
        return findings;
    }

    let mut found_triggers = BTreeSet::new();
    let upper_doc = content.to_ascii_uppercase();
    for &canonical in CANONICAL_EMERGENCY_TRIGGERS {
        if upper_doc.contains(canonical) {
            found_triggers.insert(canonical.to_string());
        }
    }
    extract_prefixed_ids(&upper_doc, "EMG-", &mut found_triggers);

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
                if missing.is_empty() {
                    "none (count underflow)".to_string()
                } else {
                    missing.join(", ")
                }
            ),
            rel_path,
            None,
        ));
    }

    findings
}

/// Audits Section 5 PACE C2 Link Communications Plan for coverage of 4 mandatory tiers ($N \ge 4$).
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-PACE-PLAN]
pub fn validate_pace_plan(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let lower_doc = content.to_lowercase();

    let has_pace_section = lower_doc.contains("pace c2")
        || lower_doc.contains("pace communications")
        || lower_doc.contains("c2 link communications plan")
        || lower_doc.contains("pace plan")
        || lower_doc.contains("## 5. pace");
    if !has_pace_section {
        return findings;
    }

    let mut found_tiers = BTreeSet::new();
    let tables = extract_markdown_tables(content);
    for tbl in &tables {
        let header_str = tbl.headers.join(" ").to_lowercase();
        if header_str.contains("pace")
            || header_str.contains("tier")
            || header_str.contains("link medium")
        {
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
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-THREAT-MATRIX]
pub fn validate_threat_matrix(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let lower_doc = content.to_lowercase();

    let has_threat_section = lower_doc.contains("threat matrix")
        || lower_doc.contains("contested environment")
        || lower_doc.contains("operational threat")
        || lower_doc.contains("## 4. multi-domain");
    if !has_threat_section {
        return findings;
    }

    let mut found_domains = BTreeSet::new();
    for domain in MANDATORY_THREAT_DOMAINS {
        if domain.keywords.iter().any(|&kw| lower_doc.contains(kw)) {
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

/// Validates a single MoE/MoP table for Threshold/Objective value pairs and SI units.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-MOE-MOP-METRICS]
fn validate_moe_mop_table(
    tbl: &MarkdownTable,
    rel_path: &str,
    findings: &mut Vec<SpecAuditFinding>,
) {
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

        let thresh_val = thresh_idx
            .and_then(|idx| row.cells.get(idx))
            .map(|s| s.trim())
            .unwrap_or("");
        let obj_val = obj_idx
            .and_then(|idx| row.cells.get(idx))
            .map(|s| s.trim())
            .unwrap_or("");

        let is_cell_invalid = |v: &str| {
            v.is_empty()
                || v.contains("{{")
                || v.eq_ignore_ascii_case("tbd")
                || v.eq_ignore_ascii_case("n/a")
        };

        if is_cell_invalid(thresh_val) || is_cell_invalid(obj_val) {
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

        if let Some(uidx) = unit_idx {
            let unit_val = row.cells.get(uidx).map(|s| s.trim()).unwrap_or("");
            let unit_clean = unit_val
                .trim_matches(&['`', '*', '_', '[', ']'][..])
                .to_lowercase();
            let is_unit_valid = !is_cell_invalid(&unit_clean)
                && MOE_MOP_RECOGNIZED_UNITS.contains(&unit_clean.as_str());
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

/// Audits Section 3 Measures of Effectiveness (MoE) & Measures of Performance (MoP) metrics.
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-MOE-MOP-METRICS]
pub fn validate_moe_mop_metrics(content: &str, rel_path: &str) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let lower_doc = content.to_lowercase();

    let has_moe_mop = lower_doc.contains("measures of effectiveness")
        || lower_doc.contains("measures of performance")
        || lower_doc.contains("moe")
        || lower_doc.contains("mop");
    if !has_moe_mop {
        return findings;
    }

    for tbl in extract_markdown_tables(content) {
        let header_str = tbl.headers.join(" ").to_lowercase();
        let has_metric_cols = header_str.contains("metric")
            || header_str.contains("moe")
            || header_str.contains("mop");
        let has_thresh_obj = header_str.contains("threshold") || header_str.contains("objective");
        if has_metric_cols && has_thresh_obj {
            validate_moe_mop_table(&tbl, rel_path, &mut findings);
        }
    }

    findings
}

/// Audits operational allocation tags (`/// OperationalAllocation: [OA-XX]` or `[MET-XX]`).
/// Realises: [REQ-SPEC-AUDIT-CONOPS-AUDIT-VALIDATE-OPERATIONAL-ALLOCATION-TAGS]
pub fn validate_operational_allocation_tags(
    content: &str,
    rel_path: &str,
) -> Vec<SpecAuditFinding> {
    let mut findings = Vec::new();
    let mut declared_oa_tasks = BTreeSet::new();
    let mut declared_met_tasks = BTreeSet::new();

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

        if trimmed.starts_with("///")
            && trimmed.to_lowercase().contains("operationalallocation")
            && (!trimmed.contains('[') || !trimmed.contains(']'))
        {
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

        let upper_line = trimmed.to_ascii_uppercase();
        extract_prefixed_ids(&upper_line, "OA-", &mut declared_oa_tasks);
        extract_prefixed_ids(&upper_line, "MET-", &mut declared_met_tasks);
    }

    let upper_content = content.to_ascii_uppercase();
    for oa_id in declared_oa_tasks {
        let expected_tag = format!("[{}]", oa_id);
        if !upper_content.contains("OPERATIONALALLOCATION")
            || !upper_content.contains(&expected_tag)
        {
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
        if !upper_content.contains("OPERATIONALALLOCATION")
            || !upper_content.contains(&expected_tag)
        {
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
