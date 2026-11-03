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
    assert_eq!(tables.first().map(|t| t.headers.len()), Some(4));
    assert_eq!(tables.first().map(|t| t.rows.len()), Some(2));
    assert_eq!(
        tables
            .first()
            .and_then(|t| t.rows.first())
            .and_then(|r| r.cells.first())
            .map(|s| s.as_str()),
        Some("MoE-01")
    );
    assert_eq!(
        tables
            .first()
            .and_then(|t| t.rows.first())
            .and_then(|r| r.cells.get(3))
            .map(|s| s.as_str()),
        Some("ms")
    );
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
    assert!(
        findings.is_empty(),
        "Expected passing emergency matrix, got: {:?}",
        findings
    );
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
    assert_eq!(
        findings.first().map(|f| f.rule_id.as_str()),
        Some("conops-emergency-matrix-incomplete")
    );
    assert!(findings
        .first()
        .is_some_and(|f| f.message.contains("EMG-03")));
    assert!(findings
        .first()
        .is_some_and(|f| f.message.contains("EMG-07")));
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
    assert!(
        findings.is_empty(),
        "Expected passing PACE plan, got: {:?}",
        findings
    );
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
    assert_eq!(
        findings.first().map(|f| f.rule_id.as_str()),
        Some("conops-pace-plan-incomplete")
    );
    assert!(findings
        .first()
        .is_some_and(|f| f.message.contains("contingency")));
    assert!(findings
        .first()
        .is_some_and(|f| f.message.contains("emergency")));
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
    assert!(
        findings.is_empty(),
        "Expected all threat domains present, got: {:?}",
        findings
    );
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
    assert_eq!(
        findings.first().map(|f| f.rule_id.as_str()),
        Some("conops-threat-matrix-missing-domain")
    );
    assert!(findings.first().is_some_and(|f| f.message.contains("CBRN")));
    assert!(findings
        .first()
        .is_some_and(|f| f.message.contains("Optical")));
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
    assert!(
        findings.is_empty(),
        "Valid MoE/MoP should pass, got: {:?}",
        findings
    );

    let invalid_content = r#"## 3. Measures of Effectiveness (MoE) & Measures of Performance (MoP) Metrics
| Metric ID | Formulation | Threshold | Objective | Unit |
| :--- | :--- | :--- | :--- | :--- |
| MoE-01 | Latency equation | {{TODO}} | 50.0 | ms |
| MoP-01 | Loop frequency | 250.0 | TBD | unknown_unit |
"#;
    let invalid_findings =
        validate_moe_mop_metrics(invalid_content, "docs/conops/MISSION_INTENT.md");
    assert!(invalid_findings
        .iter()
        .any(|f| f.rule_id == "conops-moe-mop-missing-threshold-objective"));
    assert!(invalid_findings
        .iter()
        .any(|f| f.rule_id == "conops-moe-mop-missing-unit"));
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
    let findings =
        validate_operational_allocation_tags(valid_content, "docs/conops/MISSION_INTENT.md");
    assert!(
        findings.is_empty(),
        "Valid operational allocations should pass: {:?}",
        findings
    );

    let unallocated_content = r#"# Tactical Mission Intent
## Operational Activities
| Activity ID | Name |
| :--- | :--- |
| OA-01 | Autonomous Trajectory Control |
| OA-02 | Failsafe Containment |

/// OperationalAllocation: [OA-01]
"#;
    let unallocated_findings = validate_operational_allocation_tags(
        unallocated_content,
        "docs/conops/MISSION_INTENT.md",
    );
    assert_eq!(unallocated_findings.len(), 1);
    assert_eq!(
        unallocated_findings.first().map(|f| f.rule_id.as_str()),
        Some("conops-operational-allocation-missing")
    );
    assert!(unallocated_findings
        .first()
        .is_some_and(|f| f.message.contains("OA-02")));

    let malformed_content = r#"# Tactical Mission Intent
## Operational Activities
| Activity ID | Name |
| :--- | :--- |
| OA-01 | Autonomous Trajectory Control |

/// OperationalAllocation: OA-01
"#;
    let malformed_findings = validate_operational_allocation_tags(
        malformed_content,
        "docs/conops/MISSION_INTENT.md",
    );
    assert!(malformed_findings
        .iter()
        .any(|f| f.rule_id == "conops-operational-allocation-malformed"));
}
