use super::tests_a::build_test_symbol_index;
use super::*;

#[test]
fn test_is_pure_scalar_numeric() {
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

    assert!(!is_pure_scalar_numeric("5_ControlActions"));
    assert!(!is_pure_scalar_numeric("PL-40_Actuator"));
    assert!(!is_pure_scalar_numeric("RS-485_Bus"));
    assert!(!is_pure_scalar_numeric("ALPHA5"));
    assert!(!is_pure_scalar_numeric(
        "Feature 24 Power Management System"
    ));
    assert!(!is_pure_scalar_numeric("ALPHA5_ControlActions"));
    assert!(!is_pure_scalar_numeric("DSC_IN1 Arm Enable"));
    assert!(!is_pure_scalar_numeric(""));
    assert!(!is_pure_scalar_numeric("N/A"));
}

#[test]
fn test_extract_scalar_numeric_claim() {
    assert_eq!(
        extract_scalar_numeric_claim("50 W"),
        Some((50.0, Some("W".to_string())))
    );
    assert_eq!(extract_scalar_numeric_claim("-40.0"), Some((-40.0, None)));
    assert_eq!(
        extract_scalar_numeric_claim("115200 bps"),
        Some((115200.0, Some("bps".to_string())))
    );
    assert_eq!(
        extract_scalar_numeric_claim("35 m/s"),
        Some((35.0, Some("m/s".to_string())))
    );

    assert_eq!(extract_scalar_numeric_claim("5_ControlActions"), None);
    assert_eq!(extract_scalar_numeric_claim("ALPHA5"), None);
    assert_eq!(
        extract_scalar_numeric_claim("Feature 24 Power Management System"),
        None
    );
    assert_eq!(extract_scalar_numeric_claim("PL-40_Actuator"), None);
}

#[test]
fn test_concept_provenance_abstract_structural_property_graph(
) -> Result<(), Box<dyn std::error::Error>> {
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
    let doc = parse_markdown(content)?;
    let index = build_test_symbol_index();
    let findings = validate_factual_grounding(
        &doc,
        content,
        "docs/features/feat-30.md",
        Some(&index),
        (&[], &[]),
    )?;

    assert!(
        findings.is_empty(),
        "Abstract structural property graph should pass without domain bias: {:?}",
        findings
    );
    Ok(())
}

#[test]
fn test_concept_provenance_orthogonal_actuator_fixtures(
) -> Result<(), Box<dyn std::error::Error>> {
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
    let doc = parse_markdown(content)?;
    let index = build_test_symbol_index();
    let findings = validate_factual_grounding(
        &doc,
        content,
        "docs/features/feat-31.md",
        Some(&index),
        (&[], &[]),
    )?;

    assert!(
        findings.is_empty(),
        "Orthogonal actuator fixture should validate cleanly without domain bias: {:?}",
        findings
    );
    Ok(())
}

#[test]
fn test_concept_provenance_multi_channel_actuation_architecture(
) -> Result<(), Box<dyn std::error::Error>> {
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
    let doc = parse_markdown(content)?;
    let index = build_test_symbol_index();
    let findings = validate_factual_grounding(
        &doc,
        content,
        "docs/features/feat-32.md",
        Some(&index),
        (&[], &[]),
    )?;

    assert!(
        findings.is_empty(),
        "Multi-channel actuation architecture fixture should validate cleanly: {:?}",
        findings
    );
    Ok(())
}

#[test]
fn test_concept_provenance_fails_on_unverified_domain_claim(
) -> Result<(), Box<dyn std::error::Error>> {
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
    let doc = parse_markdown(content)?;
    let index = build_test_symbol_index();
    let findings = validate_factual_grounding(
        &doc,
        content,
        "docs/features/feat-33.md",
        Some(&index),
        (&[], &[]),
    )?;

    assert!(
        findings
            .iter()
            .any(|f| f.rule_id == "factual-grounding-ungrounded-assertion"),
        "Expected ungrounded assertion finding for unverified claim, got: {:?}",
        findings
    );
    Ok(())
}

#[test]
fn test_autonomous_arming_sequence_diagram_detected_and_rejected(
) -> Result<(), Box<dyn std::error::Error>> {
    let content =
        include_str!("../../../../tests/fixtures/safety/autonomous_arming_usecase.md");
    let doc = parse_markdown(content)?;
    let findings = validate_sequence_diagram_temporal_safety(
        &doc,
        content,
        "tests/fixtures/safety/autonomous_arming_usecase.md",
    );

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
        violations.iter().any(|f| f
            .message
            .contains("Autonomous physical actuation sequence detected")),
        "Violation message must explain autonomous physical actuation sequence"
    );
    Ok(())
}

#[test]
fn test_hitl_authorized_arming_sequence_diagram_passes(
) -> Result<(), Box<dyn std::error::Error>> {
    let content = include_str!("../../../../tests/fixtures/safety/hitl_arming_usecase.md");
    let doc = parse_markdown(content)?;
    let findings = validate_sequence_diagram_temporal_safety(
        &doc,
        content,
        "tests/fixtures/safety/hitl_arming_usecase.md",
    );

    let violations: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_id == "factual-grounding-temporal-safety-violation")
        .collect();
    assert!(
        violations.is_empty(),
        "HITL authorized actuation sequence fixture must pass temporal safety validator, got: {:?}",
        violations
    );
    Ok(())
}

#[test]
fn test_conops_scn_05_sequence_diagram_passes_temporal_safety(
) -> Result<(), Box<dyn std::error::Error>> {
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
    let doc = parse_markdown(content)?;
    let findings =
        validate_sequence_diagram_temporal_safety(&doc, content, "docs/conops/SCN-05.md");
    assert!(
        findings.is_empty(),
        "SCN-05 sequence diagram should pass temporal safety without violations: {:?}",
        findings
    );
    Ok(())
}

#[test]
fn test_issue_449_count_epic_features_strict_checklist_grammar() {
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
