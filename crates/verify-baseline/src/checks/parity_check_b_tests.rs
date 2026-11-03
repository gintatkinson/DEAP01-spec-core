//! Unit tests (part B) for `parity_gates`: Check 27 canonical views, Check 23 citation fraud, Check 24 ICD, and `SectionSlicer`.
//!
//! Safety & Traceability: Realises: [REQ-PARITY-GATE-23], [REQ-PARITY-GATE-24], [REQ-PARITY-GATE-27]
//! Contract: Exercises upstream SSOT architecture views, Check 23 citation fraud fixtures, Check 24 nested ports, and `SectionSlicer`.
//! Bounds & Faults: Uses auto-cleaned temporary directories (`TempDir`); zero shared mutable state.

use super::parity_check_a_tests::TempDir;
use super::parity_gates::*;
use std::fs;

#[test]
fn test_defect_276_upstream_missing_architecture_views_library_fails(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("check27_upstream_missing_arch_views")?;
    let pipeline_upstream = tmp.path.join(".pipeline").join("upstream");
    fs::create_dir_all(&pipeline_upstream)?;

    let res = check_cross_document_diagram_parity(&tmp.path);
    assert!(
        res.is_err(),
        "Upstream compiler workspace must fail when canonical SSOT architecture views library (docs/architecture/views/) is missing"
    );
    Ok(())
}

#[test]
fn test_defect_276_canonical_architecture_views_parity_passes(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("check27_canonical_arch_views_matching")?;
    let pipeline_upstream = tmp.path.join(".pipeline").join("upstream");
    let arch_views_dir = tmp.path.join("docs").join("architecture").join("views");
    let conops_dir = tmp.path.join("docs").join("conops");
    let interfaces_dir = tmp.path.join("docs").join("interfaces");
    let reports_dir = tmp.path.join("docs").join("reports");

    fs::create_dir_all(&pipeline_upstream)?;
    fs::create_dir_all(&arch_views_dir)?;
    fs::create_dir_all(&conops_dir)?;
    fs::create_dir_all(&interfaces_dir)?;
    fs::create_dir_all(&reports_dir)?;

    let sv1_content = r#"flowchart TD
    Operator -->|"CONN-01: Supervisory Command"| SupervisoryStation
    SupervisoryStation -->|"CONN-02: System Datalink"| Controller
"#;
    fs::write(arch_views_dir.join("sv1_super_system.mmd"), sv1_content)?;

    let conops_content = r#"## 4. System Operational Architecture
```mermaid
flowchart TD
    Operator -->|"CONN-01: Supervisory Command"| SupervisoryStation
    SupervisoryStation -->|"CONN-02: System Datalink"| Controller
```
"#;
    fs::write(conops_dir.join("04_SYSTEM_ARCHITECTURE.md"), conops_content)?;

    let icd_content = r#"# Level 1C: System Interface Matrix
| Connection ID | Source Port | Dest Port |
| `CONN-01` | PORT-SUP-CMD | PORT-CTRL-CMD |
| `CONN-02` | PORT-SUP-TLM | PORT-CTRL-TLM |
"#;
    fs::write(
        interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md"),
        icd_content,
    )?;

    let report_content = r#"# Executive Engineering Deliverables
```mermaid
flowchart TD
    Operator -->|"CONN-01: Supervisory Command"| SupervisoryStation
    SupervisoryStation -->|"CONN-02: System Datalink"| Controller
```
"#;
    fs::write(
        reports_dir.join("PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"),
        report_content,
    )?;

    let res = check_cross_document_diagram_parity(&tmp.path);
    assert_eq!(res, Ok(ParityOutcome::Verified));
    Ok(())
}

#[test]
fn test_reproduce_defect_286_synthetic_mocks_blind_compiler_to_citation_fraud(
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::checks::safety_stpa::check_safety_integrity;

    let tmp = TempDir::new("check23_reproduce_defect_286")?;
    let schema_dir = tmp.path.join("schema");
    let features_dir = tmp.path.join("docs").join("features");
    let safety_dir = tmp.path.join("docs").join("safety");
    fs::create_dir_all(&schema_dir)?;
    fs::create_dir_all(&features_dir)?;
    fs::create_dir_all(&safety_dir)?;

    let model_fixture =
        include_str!("../../../../tests/fixtures/safety/ground_truth_model.sysml");
    fs::write(schema_dir.join("model.sysml"), model_fixture)?;

    let oem_manual_fixture = include_str!("../../../../tests/fixtures/safety/oem_user_manual.md");
    fs::write(schema_dir.join("oem_user_manual.md"), oem_manual_fixture)?;

    let missing_file_fixture =
        include_str!("../../../../tests/fixtures/safety/missing_file_citation_feature.md");
    fs::write(
        features_dir.join("feat_missing_file.md"),
        missing_file_fixture,
    )?;

    let fraudulent_cit_fixture =
        include_str!("../../../../tests/fixtures/safety/fraudulent_citation_feature.md");
    fs::write(
        features_dir.join("feat_fraudulent_cit.md"),
        fraudulent_cit_fixture,
    )?;

    let section_fraud_fixture =
        include_str!("../../../../tests/fixtures/safety/section_symbol_fraud_feature.md");
    fs::write(
        features_dir.join("feat_section_fraud.md"),
        section_fraud_fixture,
    )?;

    let ungrounded_dshot_fixture =
        include_str!("../../../../tests/fixtures/safety/ungrounded_dshot_feature.md");
    fs::write(
        features_dir.join("feat_ungrounded_dshot.md"),
        ungrounded_dshot_fixture,
    )?;

    let mut stpa_content = String::from(include_str!(
        "../../../../tests/fixtures/safety/complete_stpa_matrix.md"
    ));
    stpa_content.push_str(
        "\n\n<!-- Source: schema/phantom-safety-spec.md §9.9.9 -->\nActuator servos operate via DShot600 ungrounded bus.\n",
    );
    fs::write(safety_dir.join("STPA_MATRIX.md"), stpa_content)?;

    let check23_res = check_factual_grounding(&tmp.path, false, true);
    assert!(
        check23_res.is_err(),
        "Check 23 failed to reject downstream citation fraud and ungrounded protocols!"
    );

    let check17_res = check_safety_integrity(&tmp.path, false, true);
    assert!(
        check17_res.is_err(),
        "Check 17 failed to reject fraudulent safety citations to phantom files!"
    );
    Ok(())
}

#[test]
fn test_defect_333_icd_unqualified_nested_gnss_fix_fails(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("check24_unqualified_gnss_fix")?;
    let interfaces_dir = tmp.path.join("docs").join("interfaces");
    fs::create_dir_all(&interfaces_dir)?;

    let icd01_content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #333 |
| **Title** | System Interface Matrix |
| **Type** | icd |

# Level 1C: System Interface Matrix

## 4. Port Definition Roster Table
| Port ID | Subsystem | Port Name | Direction | Port Type | Multiplicity | Protocol Profile |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `PORT-FC-GNSS_FIX_IN` | FlightController | gnssFixIn | IN | DataPort | 1 | PeriodicStream |
| `PORT-NS-GNSS_FIX_OUT` | NavigationSensorSuite | gnssFixOut | OUT | DataPort | 1 | PeriodicStream |

## 5. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port | Flow Behavior | Latency Max ms | Reliability Req | Item Flows Conveyed |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `CONN-05` | gnssFixOut | gnssFixIn | Periodic update (100 Hz) | 10.0 | High | GnssFix |
"#;
    fs::write(
        interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md"),
        icd01_content,
    )?;

    let icd02_content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #333 |
| **Title** | Master Signal Flow Dictionary |
| **Type** | icd |

# Level 1C: Master Signal Flow Dictionary

## 2. Master Signal Flow Dictionary Table
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-NS-FC-001` | GnssFixData | `PORT-NS-GNSS_FIX_OUT` | `PORT-FC-GNSS_FIX_IN` | GnssFix | dimensionless | [0, 1] | 100 Hz | 0 | schema/model.sysml |
"#;
    fs::write(
        interfaces_dir.join("ICD_02_MASTER_SIGNAL_DICTIONARY.md"),
        icd02_content,
    )?;

    let res = check_icd_completeness(&tmp.path);
    assert!(
        res.is_err(),
        "Check 24 must fail when gnssFix connection lacks hierarchical qualification: {:?}",
        res
    );
    Ok(())
}

#[test]
fn test_defect_333_icd_complete_level1c_suite_passes() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("check24_complete_level1c")?;
    let interfaces_dir = tmp.path.join("docs").join("interfaces");
    fs::create_dir_all(&interfaces_dir)?;

    let icd01_content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #333 |
| **Title** | System Interface Matrix |
| **Type** | icd |

# Level 1C: System Interface Matrix

## 4. Port Definition Roster Table
| Port ID | Subsystem | Port Name | Direction | Port Type | Multiplicity | Protocol Profile |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `PORT-FC-GNSS_FIX_IN` | FlightController | gnssFixIn | IN | DataPort | 1 | PeriodicStream |
| `PORT-NS-GNSS_FIX_OUT` | NavigationSensorSuite | gnssFixOut | OUT | DataPort | 1 | PeriodicStream |

## 5. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port | Flow Behavior | Latency Max ms | Reliability Req | Item Flows Conveyed |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `CONN-05` | `PORT-NS-GNSS_FIX_OUT` (FlightController.NavigationSensorSuite.gnssFixOut) | `PORT-FC-GNSS_FIX_IN` (FlightController.gnssFixIn) | Periodic update (100 Hz) | 10.0 | High | GnssFix |
"#;
    fs::write(
        interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md"),
        icd01_content,
    )?;

    let icd02_content = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #333 |
| **Title** | Master Signal Flow Dictionary |
| **Type** | icd |

# Level 1C: Master Signal Flow Dictionary

## 2. Master Signal Flow Dictionary Table
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value | Schema Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SIG-NS-FC-001` | GnssFixData | `PORT-NS-GNSS_FIX_OUT` | `PORT-FC-GNSS_FIX_IN` | GnssFix | dimensionless | [0, 1] | 100 Hz | 0 | schema/model.sysml |
"#;
    fs::write(
        interfaces_dir.join("ICD_02_MASTER_SIGNAL_DICTIONARY.md"),
        icd02_content,
    )?;

    let res = check_icd_completeness(&tmp.path);
    assert_eq!(
        res,
        Ok(ParityOutcome::Verified),
        "Expected complete ICD suite to pass, but got: {:?}",
        res
    );
    Ok(())
}

#[test]
fn test_unicode_before_citation_does_not_panic() {
    let line = "* İnterface timing Ω µs as defined in schema/model.sysml §2.1";
    let citations = extract_citations_from_line(line, 1);
    assert_eq!(citations.len(), 1);
    if let Some(first) = citations.first() {
        assert_eq!(first.target_file, "schema/model.sysml");
        assert_eq!(first.section.as_deref(), Some("2.1"));
    }
}

#[test]
fn test_multiple_citations_same_file_different_sections_on_single_line() {
    let line = "per schema/oem_user_manual.md §1.1 and per schema/oem_user_manual.md §99.9";
    let citations = extract_citations_from_line(line, 1);
    assert_eq!(
        citations.len(),
        2,
        "Both citations on the same line must be preserved"
    );
    let sec1 = citations
        .iter()
        .any(|c| c.section.as_deref() == Some("1.1"));
    let sec2 = citations
        .iter()
        .any(|c| c.section.as_deref() == Some("99.9"));
    assert!(sec1 && sec2);
}

#[test]
fn test_section_slicer_rejects_non_heading_body_number_match(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("slicer_non_heading")?;
    let doc_path = tmp.path.join("spec.md");
    let content = "# 1. Overview\nSome body text mentioning 7.2.3 in passing without heading.\n## 2. Details\nMore text.\n";
    fs::write(&doc_path, content)?;

    let slicer = SectionSlicer::new(&tmp.path);
    let extracted = slicer.extract_section("spec.md", "§7.2.3");
    assert!(
        extracted.is_none(),
        "SectionSlicer must not return whole-file fallback when section is not a heading"
    );
    Ok(())
}
