//! Unit tests (part E) for `parity_gates`: Issues #539, #540, #541, #478, and Check 23 dogfood grounding.
//!
//! Safety & Traceability: Realises: [REQ-DUAL-SCHEMA-SSOT-PARITY], [REQ-PARITY-GATE-21], [REQ-PARITY-GATE-23]
//! Contract: Exercises design digest sidecars, `TargetScanDir::Designs`, `.sysml`/`.json` section slicing, and `use case def` parity.
//! Bounds & Faults: Uses auto-cleaned temporary directories (`TempDir`); zero shared mutable state.

use super::parity_check_a_tests::TempDir;
use super::parity_gates::*;
use std::fs;
use std::path::Path;

#[test]
fn test_check_31_dual_schema_ssot_parity_validates_docs_designs_sysml_and_sidecars(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("issue_539_check_31_docs_designs")?;
    let schema_dir = tmp.path.join("schema");
    fs::create_dir_all(&schema_dir)?;
    fs::write(schema_dir.join(".gitkeep"), "")?;

    let designs_dir = tmp.path.join("docs").join("designs");
    fs::create_dir_all(&designs_dir)?;
    fs::write(
        designs_dir.join("blueprint-01-alpha.sysml"),
        "package Alpha { part def AlphaEngine; }\n",
    )?;

    let res_missing_sidecar = check_dual_schema_ssot_parity(&tmp.path);
    assert!(
        res_missing_sidecar.is_err(),
        "Check 31 must fail when docs/designs/blueprint-01-alpha.digest.json sidecar is missing"
    );
    let sidecar_errs = res_missing_sidecar.err().unwrap_or_default().join("\n");
    assert!(
        sidecar_errs.contains("Missing or empty companion digest sidecar"),
        "Expected missing sidecar error diagnostic, got: {}",
        sidecar_errs
    );

    fs::write(
        designs_dir.join("blueprint-01-alpha.digest.json"),
        "{\"blueprint_id\":\"blueprint-01-alpha\"}\n",
    )?;
    let res_single_clean = check_dual_schema_ssot_parity(&tmp.path);
    assert_eq!(
        res_single_clean,
        Ok(ParityOutcome::DualSchemaSingleOrClean),
        "Check 31 must return Ok(DualSchemaSingleOrClean) when .digest.json sidecars are valid and .pipeline/schema.sysml is absent"
    );

    let pipeline_dir = tmp.path.join(".pipeline");
    fs::create_dir_all(&pipeline_dir)?;
    fs::write(
        pipeline_dir.join("schema.sysml"),
        "package Alpha { part def DivergentEngine; }\n",
    )?;
    let res_divergent = check_dual_schema_ssot_parity(&tmp.path);
    assert!(
        res_divergent.is_err(),
        "Check 31 must fail when .pipeline/schema.sysml diverges from docs/designs/blueprint-01-alpha.sysml"
    );

    fs::write(
        pipeline_dir.join("schema.sysml"),
        "package Alpha { part def AlphaEngine; }\n",
    )?;
    let res_identical = check_dual_schema_ssot_parity(&tmp.path);
    assert_eq!(
        res_identical,
        Ok(ParityOutcome::DualSchemaIdentical),
        "Check 31 must return Ok(DualSchemaIdentical) when .pipeline/schema.sysml matches docs/designs/blueprint-01-alpha.sysml"
    );

    Ok(())
}

#[test]
fn test_target_scan_dir_includes_designs_for_check_21_and_check_23(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("issue_540_designs_scan_dir")?;
    let schema_dir = tmp.path.join("schema");
    fs::create_dir_all(&schema_dir)?;
    fs::write(
        schema_dir.join("model.sysml"),
        "package Sys {\n  part def DeclaredController;\n}\n",
    )?;

    let designs_dir = tmp.path.join("docs").join("designs");
    fs::create_dir_all(&designs_dir)?;
    fs::write(
        designs_dir.join("blueprint-01-alpha.md"),
        "# Blueprint 01 Alpha\n\n[Source: schema/nonexistent.sysml#§1]\n\n```mermaid\nclassDiagram\n    class DeclaredController\n    class UndeclaredGhost\n    DeclaredController --> UndeclaredGhost\n```\n",
    )?;

    let res_check21 = check_semantic_diagram_ast_parity(&tmp.path);
    assert!(
        res_check21.is_err(),
        "Check 21 must fail when docs/designs/blueprint-01-alpha.md references undeclared class UndeclaredGhost"
    );
    let errs21 = res_check21.err().unwrap_or_default().join("\n");
    assert!(
        errs21.contains("UndeclaredGhost") && errs21.contains("blueprint-01-alpha.md"),
        "Expected Check 21 diagnostic for UndeclaredGhost in blueprint-01-alpha.md, got: {}",
        errs21
    );

    let res_check23 = check_factual_grounding(&tmp.path, false, true);
    assert!(
        res_check23.is_err(),
        "Check 23 must fail when docs/designs/blueprint-01-alpha.md contains a fraudulent citation"
    );
    let errs23 = res_check23.err().unwrap_or_default().join("\n");
    assert!(
        errs23.contains("nonexistent.sysml") && errs23.contains("blueprint-01-alpha.md"),
        "Expected Check 23 diagnostic for nonexistent.sysml in blueprint-01-alpha.md, got: {}",
        errs23
    );

    Ok(())
}

#[test]
fn test_section_slicer_extract_section_supports_sysml_and_digest_json_citations(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("issue_541_sysml_and_digest_json_section_slicer")?;
    let designs_dir = tmp.path.join("docs").join("designs");
    fs::create_dir_all(&designs_dir)?;
    fs::write(
        designs_dir.join("blueprint-01-alpha.sysml"),
        "package Alpha { part def FlightController; }\n",
    )?;
    fs::write(
        designs_dir.join("blueprint-01-alpha.digest.json"),
        "{\"package_name\":\"Alpha\"}\n",
    )?;

    let slicer = SectionSlicer::new(&tmp.path);

    let sysml_text = slicer.extract_section("docs/designs/blueprint-01-alpha.sysml", "2.1");
    assert!(
        sysml_text
            .as_deref()
            .is_some_and(|t| t.contains("FlightController")),
        "Expected SectionSlicer::extract_section on .sysml with section '2.1' to return content containing 'FlightController', got: {:?}",
        sysml_text
    );

    let json_text = slicer.extract_section("docs/designs/blueprint-01-alpha.digest.json", "1.0");
    assert!(
        json_text.as_deref().is_some_and(|t| t.contains("Alpha")),
        "Expected SectionSlicer::extract_section on .digest.json with section '1.0' to return content containing 'Alpha', got: {:?}",
        json_text
    );

    Ok(())
}

#[test]
fn test_issue_478_check_30a_rejects_unrealized_schema_use_case_defs_and_check_31_detects_use_case_def_divergence(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("issue_478_use_case_def_parity")?;
    let schema_dir = tmp.path.join("schema");
    let pipe_dir = tmp.path.join(".pipeline");
    let uc_dir = tmp.path.join("docs").join("use-cases");
    fs::create_dir_all(&schema_dir)?;
    fs::create_dir_all(&pipe_dir)?;
    fs::create_dir_all(&uc_dir)?;

    let full_sysml =
        "package Ops {\n  use case def PerformPreFlightCheck;\n  use case def ExecuteEmergencyLanding;\n}\n";
    fs::write(schema_dir.join("ops.sysml"), full_sysml)?;
    fs::write(pipe_dir.join("schema.sysml"), full_sysml)?;

    fs::write(
        uc_dir.join("uc-001-perform-pre-flight-check.md"),
        "# UC-001: PerformPreFlightCheck\nRealizes `use case def PerformPreFlightCheck`.\n",
    )?;

    let res_30a_unrealized = check_coverage_digest(&tmp.path);
    assert!(
        res_30a_unrealized.is_err(),
        "Check 30A must fail when schema use case def ExecuteEmergencyLanding is not realized in docs/use-cases/"
    );
    let errs_30a = res_30a_unrealized.err().unwrap_or_default().join("\n");
    assert!(
        errs_30a.contains(
            "Check 30A failed: Schema use case def 'ExecuteEmergencyLanding' is not realized in docs/use-cases/"
        ),
        "Expected Check 30A diagnostic for unrealized ExecuteEmergencyLanding, got: {}",
        errs_30a
    );

    fs::write(
        uc_dir.join("uc-002-execute-emergency-landing.md"),
        "# UC-002: ExecuteEmergencyLanding\nRealizes `use case def ExecuteEmergencyLanding`.\n",
    )?;
    let res_30a_ok = check_coverage_digest(&tmp.path);
    assert_eq!(
        res_30a_ok,
        Ok(ParityOutcome::Verified),
        "Check 30A must pass when all schema use case defs are realized in docs/use-cases/"
    );

    fs::write(
        pipe_dir.join("schema.sysml"),
        "package Ops {\n  use case def PerformPreFlightCheck;\n}\n",
    )?;
    let res_31_divergent = check_dual_schema_ssot_parity(&tmp.path);
    assert!(
        res_31_divergent.is_err(),
        "Check 31 must fail when use case def diverges between schema/*.sysml and .pipeline/schema.sysml"
    );
    let errs_31 = res_31_divergent.err().unwrap_or_default().join("\n");
    assert!(
        errs_31.contains("use case def") && errs_31.contains("Ops::ExecuteEmergencyLanding"),
        "Expected Check 31 mismatch diagnostic for use case def Ops::ExecuteEmergencyLanding, got: {}",
        errs_31
    );

    fs::write(pipe_dir.join("schema.sysml"), full_sysml)?;
    let res_31_ok = check_dual_schema_ssot_parity(&tmp.path);
    assert_eq!(
        res_31_ok,
        Ok(ParityOutcome::DualSchemaIdentical),
        "Check 31 must return Ok(DualSchemaIdentical) when use case defs match across dual schemas"
    );

    Ok(())
}

#[test]
fn test_check23_ignores_fences_directories_links_and_constraint_ids_while_failing_closed_on_fraud(
) -> Result<(), Box<dyn std::error::Error>> {
    let root_tmp = TempDir::new("check23_dogfood_fences_dirs_links")?;
    let root_designs = root_tmp.path.join("docs").join("designs");
    fs::create_dir_all(&root_designs)?;
    fs::write(
        root_designs.join("blueprint-01-core.md"),
        "# Blueprint 01\n## 2.1 Core Contract\nDeterministic AST compilation rules.\n",
    )?;

    let dogfood = root_tmp.path.join(".pipeline").join("upstream");
    let schema_dir = dogfood.join("schema");
    let safety_dir = dogfood.join("docs").join("safety");
    let uc_dir = dogfood.join("docs").join("use-cases");
    fs::create_dir_all(&schema_dir)?;
    fs::create_dir_all(&safety_dir)?;
    fs::create_dir_all(&uc_dir)?;

    let model_sysml = r#"package CompilerCore {
  part def AstVerifier;
  requirement def REQ_0604 {
    doc /* Projects Logical Interface Control Documents (docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md) */
  }
  use case def UC_AC_01_Deterministic_Ast_Verification;
}
"#;
    fs::write(schema_dir.join("compiler.sysml"), model_sysml)?;
    fs::write(
        uc_dir.join("UC-06-Deterministic_Ast_Verification.md"),
        "# Use Case 06: Deterministic Ast Verification\n",
    )?;

    let clean_spec = r#"# Safety Hazard Log
Grounded in `docs/designs/blueprint-01-core.md` §2.1.
Clean landing zones `docs/epics/` and `docs/interfaces` remain intact.
Requirement projects Logical Interface Control Documents (docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md).
Traceable to [User Story 32: Aggregate Diagnostics Via CollectAll And](../user-stories/US-32-AggregateViaCollectAll.md).
Hazard mitigation enforced via SC-001 and UCA-002 and REQ-101.

```mermaid
flowchart TD
    UC_Step01(["Inspect docs/des..."]) -->|"via RS-485"| UC_Step02(["Emit report"])
```
"#;
    let hazard_log_path = safety_dir.join("HAZARD_LOG.md");
    fs::write(&hazard_log_path, clean_spec)?;

    let res_clean = check_factual_grounding(&dogfood, false, true);
    assert_eq!(res_clean, Ok(ParityOutcome::Verified));
    assert_eq!(check_coverage_digest(&dogfood), Ok(ParityOutcome::Verified));

    verify_check23_and_30a_mutations(&dogfood, &schema_dir, &hazard_log_path, model_sysml, clean_spec)?;
    Ok(())
}

fn verify_check23_and_30a_mutations(
    dogfood: &Path,
    schema_dir: &Path,
    hazard_log_path: &Path,
    model_sysml: &str,
    clean_spec: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let unrealized_uc_sysml = format!(
        "{}\npackage ExtraPkg {{\n  use case def UC_AC_99_Unrealized_Verification_Flow;\n}}\n",
        model_sysml
    );
    fs::write(schema_dir.join("compiler.sysml"), unrealized_uc_sysml)?;
    assert!(check_coverage_digest(dogfood).is_err());
    fs::write(schema_dir.join("compiler.sysml"), model_sysml)?;

    let fraud_file_spec = format!("{}\nSee `docs/nonexistent_fraud.md` for details.\n", clean_spec);
    fs::write(hazard_log_path, fraud_file_spec)?;
    let res_fraud_file = check_factual_grounding(dogfood, false, true);
    assert!(res_fraud_file.is_err());
    let errs_file = res_fraud_file.err().unwrap_or_default().join("\n");
    assert!(errs_file.contains("docs/nonexistent_fraud.md"));

    let ungrounded_proto_spec = format!(
        "{}\nFrames are transmitted via RS-485 across external link.\n",
        clean_spec
    );
    fs::write(hazard_log_path, ungrounded_proto_spec)?;
    let res_proto = check_factual_grounding(dogfood, false, true);
    assert!(res_proto.is_err());
    let errs_proto = res_proto.err().unwrap_or_default().join("\n");
    assert!(errs_proto.contains("RS-485"));
    Ok(())
}
