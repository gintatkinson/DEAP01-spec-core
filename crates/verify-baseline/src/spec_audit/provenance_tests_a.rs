use super::*;

pub(super) fn build_test_symbol_index() -> SysmlSymbolIndex {
    let mut index = SysmlSymbolIndex::new();
    index.part_names.insert("FlightController".to_string());
    index.attribute_names.insert("sampling_rate".to_string());
    index.attribute_names.insert("bus_voltage".to_string());
    index.attribute_names.insert("latency".to_string());
    index.action_names.insert("sample_sensors".to_string());
    index
}

#[test]
fn test_factual_grounding_with_valid_units_and_grounding_passes(
) -> Result<(), Box<dyn std::error::Error>> {
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
    let doc = parse_markdown(content)?;
    let index = build_test_symbol_index();
    let findings = validate_factual_grounding(
        &doc,
        content,
        "docs/features/feat-01.md",
        Some(&index),
        (&[], &[]),
    )?;

    assert!(
        findings.is_empty(),
        "Expected no findings, got: {:?}",
        findings
    );
    Ok(())
}

#[test]
fn test_factual_grounding_missing_units_fails() -> Result<(), Box<dyn std::error::Error>> {
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
    let doc = parse_markdown(content)?;
    let index = build_test_symbol_index();
    let findings = validate_factual_grounding(
        &doc,
        content,
        "docs/features/feat-01.md",
        Some(&index),
        (&[], &[]),
    )?;

    let missing_units: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_id == "factual-grounding-missing-unit")
        .collect();
    assert!(!missing_units.is_empty(), "Expected missing unit findings");
    assert!(missing_units.iter().any(|f| f.message.contains("250")));
    assert!(missing_units.iter().any(|f| f.message.contains("50")));
    assert!(missing_units.iter().any(|f| f.message.contains("12")));
    Ok(())
}

#[test]
fn test_source_reference_clause_integrity_passes() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"# Feature 01: Flight Guidance

## Source References
Normative standards baseline: ISO/IEC/IEEE 29148:2018 §6.4.2 and schema/model.sysml.
"#;
    let doc = parse_markdown(content)?;
    let findings =
        validate_source_reference_clause_integrity(&doc, content, "docs/features/feat-01.md")?;

    assert!(
        findings.is_empty(),
        "Expected clean source references, got: {:?}",
        findings
    );
    Ok(())
}

#[test]
fn test_source_reference_missing_section_fails() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"# Feature 01: Flight Guidance

## Description
This spec lacks a source references section.
"#;
    let doc = parse_markdown(content)?;
    let findings =
        validate_source_reference_clause_integrity(&doc, content, "docs/features/feat-01.md")?;

    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings.first().map(|f| f.rule_id.as_str()),
        Some("source-reference-section-missing")
    );
    Ok(())
}

#[test]
fn test_source_reference_empty_section_fails() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"# Feature 01: Flight Guidance

## Source References

## Next Section
"#;
    let doc = parse_markdown(content)?;
    let findings =
        validate_source_reference_clause_integrity(&doc, content, "docs/features/feat-01.md")?;

    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings.first().map(|f| f.rule_id.as_str()),
        Some("source-reference-section-empty")
    );
    Ok(())
}

#[test]
fn test_source_reference_placeholder_and_self_referential_fails(
) -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"# Feature 01: Flight Guidance

## Source References
- Schema: [link-to-schema](docs/features/feat-01.md)
- {{TODO: Populate external citation}}
"#;
    let doc = parse_markdown(content)?;
    let findings =
        validate_source_reference_clause_integrity(&doc, content, "docs/features/feat-01.md")?;

    assert!(findings
        .iter()
        .any(|f| f.rule_id == "source-reference-placeholder-detected"));
    assert!(findings
        .iter()
        .any(|f| f.rule_id == "source-reference-self-referential"));
    Ok(())
}

#[test]
fn test_cardinality_bounds_epic_bounds() -> Result<(), Box<dyn std::error::Error>> {
    let underflow_content = r#"---
type: epic
---
# Epic 01: Core Systems

## Requirements & Checklist
- [ ] #10 - [Feature 01: Single feature](../features/FEAT-01.md) (traceable requirement)
"#;
    let doc1 = parse_markdown(underflow_content)?;
    let findings1 =
        validate_cardinality_bounds(&doc1, underflow_content, "docs/epics/epic-01.md")?;
    assert!(findings1
        .iter()
        .any(|f| f.rule_id == "cardinality-bounds-epic-underflow"));

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
    let doc2 = parse_markdown(nominal_content)?;
    let findings2 =
        validate_cardinality_bounds(&doc2, nominal_content, "docs/epics/epic-01.md")?;
    assert!(
        findings2.is_empty(),
        "Nominal epic features should pass: {:?}",
        findings2
    );

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
    let doc3 = parse_markdown(&overflow_content)?;
    let findings3 =
        validate_cardinality_bounds(&doc3, &overflow_content, "docs/epics/epic-01.md")?;
    assert!(findings3
        .iter()
        .any(|f| f.rule_id == "cardinality-bounds-epic-overflow"));
    Ok(())
}

#[test]
fn test_cardinality_bounds_feature_bounds() -> Result<(), Box<dyn std::error::Error>> {
    let underflow_content = r#"---
type: feature
---
# Feature 01: Subsystem

## Acceptance Criteria
- [ ] AC-01: Single criterion
"#;
    let doc1 = parse_markdown(underflow_content)?;
    let findings1 =
        validate_cardinality_bounds(&doc1, underflow_content, "docs/features/feat-01.md")?;
    assert!(findings1
        .iter()
        .any(|f| f.rule_id == "cardinality-bounds-feature-underflow"));

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
    let doc2 = parse_markdown(nominal_content)?;
    let findings2 =
        validate_cardinality_bounds(&doc2, nominal_content, "docs/features/feat-01.md")?;
    assert!(
        findings2.is_empty(),
        "Nominal criteria should pass: {:?}",
        findings2
    );

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
    let doc3 = parse_markdown(&overflow_content)?;
    let findings3 =
        validate_cardinality_bounds(&doc3, &overflow_content, "docs/features/feat-01.md")?;
    assert!(findings3
        .iter()
        .any(|f| f.rule_id == "cardinality-bounds-feature-overflow"));
    Ok(())
}

#[test]
fn test_reproduce_defect_213_package_digits_and_metadata_table_contradiction(
) -> Result<(), Box<dyn std::error::Error>> {
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
    let doc = parse_markdown(content)?;
    let index = build_test_symbol_index();

    let cb_findings =
        validate_cardinality_bounds(&doc, content, "specifications/feat-custom-path.md")?;
    assert!(
        cb_findings.is_empty(),
        "Expected 0 cardinality findings for nominal 3 criteria, but got: {:?}",
        cb_findings
    );

    let findings = validate_factual_grounding(
        &doc,
        content,
        "docs/features/feat-24.md",
        Some(&index),
        (&[], &[]),
    )?;

    assert!(
        findings.is_empty(),
        "Expected 0 factual grounding findings, but package-name digits were erroneously extracted: {:?}",
        findings
    );
    Ok(())
}
