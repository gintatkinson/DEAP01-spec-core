//! Unit tests (part C) for `parity_gates`: Check 24/27/28/31 edge cases, `TargetScanDir`, and workspace integrity.
//!
//! Safety & Traceability: Realises: [REQ-PARITY-GATE-24], [REQ-PARITY-GATE-27], [REQ-DUAL-SCHEMA-SSOT-PARITY]
//! Contract: Exercises non-GNSS nested port qualification, asymmetric connections, cross-package drift, and workspace integrity.
//! Bounds & Faults: Uses auto-cleaned temporary directories (`TempDir`); zero shared mutable state.

use super::parity_check_a_tests::TempDir;
use super::parity_check_discovery::TargetScanDir;
use super::parity_gates::*;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[test]
fn test_check24_non_gnss_nested_port_requires_qualification(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("check24_nested_non_gnss")?;
    let interfaces_dir = tmp.path.join("docs").join("interfaces");
    fs::create_dir_all(&interfaces_dir)?;

    let icd01_content = r#"# Level 1C: System Interface Matrix
## 4. Port Definition Roster Table
| Port ID | Subsystem | Port Name | Direction | Port Type | Multiplicity | Protocol Profile |
| `PORT-FC-IMU_IN` | FlightController | imuIn | IN | DataPort | 1 | PeriodicStream |
| `PORT-IMU-ACCEL_OUT` | InertialSensorSuite | accelOut | OUT | DataPort | 1 | PeriodicStream |

## 5. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port | Flow Behavior | Latency Max ms | Reliability Req | Item Flows Conveyed |
| `CONN-01` | PORT-IMU-ACCEL_OUT | PORT-FC-IMU_IN | Periodic update (200 Hz) | 5.0 | High | AccelData |
"#;
    fs::write(
        interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md"),
        icd01_content,
    )?;

    let icd02_content = r#"# Level 1C: Master Signal Flow Dictionary
## 2. Master Signal Flow Dictionary Table
| Signal ID | Signal Name | Source Port | Dest Port | Data Type | SI Units | Valid Range | Update Rate | Safe Default Value | Schema Citation |
| `SIG-IMU-FC-001` | AccelData | `PORT-IMU-ACCEL_OUT` | `PORT-FC-IMU_IN` | Accel | m/s^2 | [-100, 100] | 200 Hz | 0 | schema/model.sysml |
"#;
    fs::write(
        interfaces_dir.join("ICD_02_MASTER_SIGNAL_DICTIONARY.md"),
        icd02_content,
    )?;

    let res = check_icd_completeness(&tmp.path);
    assert!(
        res.is_err(),
        "Check 24 must fail when nested IMU connection lacks hierarchical qualification: {:?}",
        res
    );
    Ok(())
}

#[test]
fn test_check27_empty_conops_connections_with_populated_icd_fails(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("check27_asymmetric_empty")?;
    let conops_dir = tmp.path.join("docs").join("conops");
    let interfaces_dir = tmp.path.join("docs").join("interfaces");
    fs::create_dir_all(&conops_dir)?;
    fs::create_dir_all(&interfaces_dir)?;

    let conops_content = r#"## 4. System Operational Architecture
```mermaid
flowchart TD
    Operator --> SupervisoryStation
```
"#;
    fs::write(conops_dir.join("04_SYSTEM_ARCHITECTURE.md"), conops_content)?;

    let icd_content = r#"# Level 1C: System Interface Matrix
## 5. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port |
| `CONN-01` | PORT-SUP-CMD | PORT-CTRL-CMD |
"#;
    fs::write(
        interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md"),
        icd_content,
    )?;

    let res = check_cross_document_diagram_parity(&tmp.path);
    assert!(
        res.is_err(),
        "Check 27 must fail when ConOps has 0 connections while ICD has CONN-01"
    );
    Ok(())
}

#[test]
fn test_check31_detects_attribute_or_body_divergence_with_identical_part_names(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("check31_body_divergence")?;
    let schema_dir = tmp.path.join("schema");
    let pipe_dir = tmp.path.join(".pipeline");
    fs::create_dir_all(&schema_dir)?;
    fs::create_dir_all(&pipe_dir)?;

    let sysml_with_port =
        "package Core {\n  part def FlightController {\n    port cmdIn;\n  }\n}\n";
    let sysml_empty = "package Core {\n  part def FlightController;\n}\n";

    fs::write(schema_dir.join("domain.sysml"), sysml_with_port)?;
    fs::write(pipe_dir.join("schema.sysml"), sysml_empty)?;

    let res = check_dual_schema_ssot_parity(&tmp.path);
    assert!(
        res.is_err(),
        "Check 31 must detect structural divergence inside part def even when names are identical: {:?}",
        res
    );
    Ok(())
}

#[test]
fn test_issue_443_check31_detects_requirement_constraint_state_and_cross_package_divergence(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("check31_issue443_divergence")?;
    let schema_dir = tmp.path.join("schema");
    let pipe_dir = tmp.path.join(".pipeline");
    fs::create_dir_all(&schema_dir)?;
    fs::create_dir_all(&pipe_dir)?;

    let sysml_schema = "package Core {\n  part def Controller;\n  requirement def ReqSafety {\n    doc /* Must safely halt */\n  }\n}\n";
    let sysml_pipeline = "package Core {\n  part def Controller;\n}\n";

    fs::write(schema_dir.join("domain.sysml"), sysml_schema)?;
    fs::write(pipe_dir.join("schema.sysml"), sysml_pipeline)?;

    let res = check_dual_schema_ssot_parity(&tmp.path);
    assert!(
        res.is_err(),
        "Check 31 must detect requirement def divergence: {:?}",
        res
    );

    let sysml_schema_c =
        "package Core {\n  part def Controller;\n  constraint def ThermalLimit;\n}\n";
    fs::write(schema_dir.join("domain.sysml"), sysml_schema_c)?;
    let res_c = check_dual_schema_ssot_parity(&tmp.path);
    assert!(
        res_c.is_err(),
        "Check 31 must detect constraint def divergence: {:?}",
        res_c
    );

    let sysml_multi_pkg_a = "package SubA {\n  part def Engine {\n    port in1;\n  }\n}\npackage SubB {\n  part def Engine;\n}\n";
    let sysml_multi_pkg_b =
        "package SubA {\n  part def Engine;\n}\npackage SubB {\n  part def Engine;\n}\n";
    fs::write(schema_dir.join("domain.sysml"), sysml_multi_pkg_a)?;
    fs::write(pipe_dir.join("schema.sysml"), sysml_multi_pkg_b)?;
    let res_coll = check_dual_schema_ssot_parity(&tmp.path);
    assert!(
        res_coll.is_err(),
        "Check 31 must detect divergence in colliding part name across packages: {:?}",
        res_coll
    );

    Ok(())
}

#[test]
fn test_check28_propagates_schema_and_io_errors_fail_closed(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("check28_fail_closed")?;
    let schemas_dir = tmp.path.join(".pipeline").join("schemas");
    fs::create_dir_all(&schemas_dir)?;

    fs::write(
        schemas_dir.join("conops_specification_schema.json"),
        "{ invalid_json: true }",
    )?;

    let res = check_conops_and_mission_intent_completeness(&tmp.path, false, true);
    assert!(
        res.is_err(),
        "Check 28 must fail closed on malformed JSON schema: {:?}",
        res
    );
    Ok(())
}

#[test]
fn test_target_scan_dir_taxonomy_unification_option_b() {
    let set: BTreeSet<TargetScanDir> = TargetScanDir::ALL.iter().copied().collect();
    assert_eq!(
        set.len(),
        12,
        "TargetScanDir::ALL must contain 12 unique variants"
    );
    assert_eq!(TargetScanDir::ALL.len(), 12);
    assert_eq!(TargetScanDir::Designs.docs_subdir(), "designs");
    assert!(TargetScanDir::Conops.requires_factual_grounding());
    assert!(TargetScanDir::Safety.requires_factual_grounding());
    assert!(TargetScanDir::Interfaces.requires_factual_grounding());
    assert!(TargetScanDir::Icd.requires_factual_grounding());
    assert!(TargetScanDir::Architecture.requires_factual_grounding());
    assert!(TargetScanDir::Designs.requires_factual_grounding());
    assert!(TargetScanDir::Epics.requires_factual_grounding());
    assert!(TargetScanDir::Features.requires_factual_grounding());
    assert!(TargetScanDir::UserStories.requires_factual_grounding());
    assert!(TargetScanDir::UseCases.requires_factual_grounding());
    assert!(TargetScanDir::Reports.requires_factual_grounding());
    assert!(!TargetScanDir::Management.requires_factual_grounding());
}

#[test]
fn test_workspace_integrity_no_fake_predicates_or_fixture_hardcoding(
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("missing workspace root")?;
    let crates_dir = root.join("crates");

    let global_forbidden = vec![
        ["\"UNDEC", "LARED\""].concat(),
        ["\"DIVER", "GENT\""].concat(),
        ["\"PHAN", "TOM\""].concat(),
        ["let _ =", " repo_root"].concat(),
        ["\"VTOL", "Motor\""].concat(),
        ["\"dshot", "600\""].concat(),
        ["\"50 ", "hz\""].concat(),
        ["\"stanag ", "4586\""].concat(),
        ["\"stanag", "4586\""].concat(),
    ];
    let gate_specific_forbidden = vec![
        ["\"dsh", "ot\""].concat(),
        ["\"GN", "SS\""].concat(),
        ["\"IM", "U\""].concat(),
        ["\"Flight", "Controller\""].concat(),
    ];

    let mut violations = Vec::new();
    for entry_res in WalkDir::new(&crates_dir).sort_by_file_name().into_iter() {
        let entry = entry_res?;
        let path = entry.path();
        if !entry.file_type().is_file() || path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        if !rel.contains("/src/") || rel.ends_with("_tests.rs") {
            continue;
        }
        let content = fs::read_to_string(path)?;
        audit_prod_file_for_forbidden_literals(
            &content,
            &rel,
            &global_forbidden,
            &gate_specific_forbidden,
            &mut violations,
        );
    }

    assert!(
        violations.is_empty(),
        "Workspace integrity violations found ({}):\n{}",
        violations.len(),
        violations.join("\n")
    );
    Ok(())
}

fn audit_prod_file_for_forbidden_literals(
    content: &str,
    rel: &str,
    global_forbidden: &[String],
    gate_specific_forbidden: &[String],
    violations: &mut Vec<String>,
) {
    let prod = content.split("#[cfg(test)]").next().unwrap_or(content);
    let check_gate_specific = rel.ends_with("parity_gates.rs")
        || rel.contains("parity_check_")
        || rel.ends_with("safety_stpa.rs")
        || rel.ends_with("provenance.rs");

    for (idx, line) in prod.lines().enumerate() {
        let line_no = idx + 1;
        let code = line.split("//").next().unwrap_or(line);
        for needle in global_forbidden {
            if code.contains(needle) {
                violations.push(format!(
                    "{}:{}: contains forbidden predicate or fixture literal {}",
                    rel, line_no, needle
                ));
            }
        }
        if check_gate_specific {
            for needle in gate_specific_forbidden {
                if code.contains(needle) {
                    violations.push(format!(
                        "{}:{}: contains hardcoded fixture literal {}",
                        rel, line_no, needle
                    ));
                }
            }
        }
    }

    if rel.ends_with("safety_stpa.rs") {
        let corrupted_space_entry = ["\"", " ", "\","].concat();
        for kw_const in [
            "INTERFACE_KEYWORDS",
            "STATE_KEYWORDS",
            "ACTION_KEYWORDS",
            "RESOURCE_KEYWORDS",
        ] {
            if let Some(block) = prod
                .find(kw_const)
                .and_then(|pos| prod.get(pos..))
                .and_then(|after| after.find("];").and_then(|end| after.get(..end)))
            {
                if block.contains(&corrupted_space_entry) {
                    violations.push(format!(
                        "{}: {} contains corrupted single-space entry",
                        rel, kw_const
                    ));
                }
            }
        }
    }
}
