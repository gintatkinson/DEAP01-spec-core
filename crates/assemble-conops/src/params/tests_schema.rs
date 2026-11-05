//! Unit tests for multi-format schema ingestion, provenance columns, and AST physical limits.

use std::collections::BTreeMap;

use super::*;

#[test]
fn test_defect_302_unified_schema_support_in_assemble_conops(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = SysMLParameterBindingEngine::new(None, None, None, false, None);

    let yaml_content =
        "subsystems:\n  - FlightComputer\n  - NavigationUnit\nOPERATIONAL_RANGE_KM: 500\n";
    let temp_yaml = std::env::temp_dir().join("test_schema_302.yaml");
    std::fs::write(&temp_yaml, yaml_content)?;
    let ingested_yaml = engine.ingest_file(&temp_yaml);
    std::fs::remove_file(&temp_yaml).ok();
    assert!(ingested_yaml, "YAML schema ingestion must succeed");
    assert!(engine.ast_part_names.contains("FlightComputer"));
    assert!(engine.ast_part_names.contains("NavigationUnit"));
    assert_eq!(
        engine
            .parameter_bindings
            .get("OPERATIONAL_RANGE_KM")
            .map(|s| s.as_str()),
        Some("500")
    );

    let md_content = "# Specifications\n\n### 4.3 Subsystem Architecture\n| Subsystem | Function |\n|---|---|\n| GuidanceComputer | Navigation and guidance |\n| SensorPayload | High-res electro-optical |\n";
    let temp_md = std::env::temp_dir().join("test_schema_302.md");
    std::fs::write(&temp_md, md_content)?;
    let ingested_md = engine.ingest_file(&temp_md);
    std::fs::remove_file(&temp_md).ok();
    assert!(
        ingested_md,
        "Markdown architecture table ingestion must succeed"
    );
    assert!(engine.ast_part_names.contains("GuidanceComputer"));
    assert!(engine.ast_part_names.contains("SensorPayload"));

    let proto_content = "syntax = \"proto3\";\nservice TelemetryGateway {\n  rpc Stream();\n}\n";
    let temp_proto = std::env::temp_dir().join("test_schema_302.proto");
    std::fs::write(&temp_proto, proto_content)?;
    let ingested_proto = engine.ingest_file(&temp_proto);
    std::fs::remove_file(&temp_proto).ok();
    assert!(ingested_proto, "Protobuf schema ingestion must succeed");
    assert!(engine.ast_part_names.contains("TelemetryGateway"));
    Ok(())
}

#[test]
fn test_defect_313_zero_weapon_crypto_zeroization_and_abstract_authority_handoff() {
    let mut exp_bindings = BTreeMap::new();
    exp_bindings.insert("IS_EXPENDABLE".to_string(), "true".to_string());
    let exp_engine = SysMLParameterBindingEngine::new(Some(exp_bindings), None, None, false, None);

    let failsafe = exp_engine.resolve_token("LIFECYCLE_FAILSAFE_SEQUENCE", None);
    assert!(
        !failsafe.contains("zeroization"),
        "LIFECYCLE_FAILSAFE_SEQUENCE must not contain zeroization: got '{failsafe}'"
    );
    assert!(
        !failsafe.contains("ditching"),
        "LIFECYCLE_FAILSAFE_SEQUENCE must not contain ditching: got '{failsafe}'"
    );

    let end_state = exp_engine.resolve_token("LIFECYCLE_END_STATE", None);
    assert!(
        !end_state.contains("terminal impact"),
        "LIFECYCLE_END_STATE must not contain terminal impact: got '{end_state}'"
    );
    assert!(
        !end_state.contains("ditching"),
        "LIFECYCLE_END_STATE must not contain ditching: got '{end_state}'"
    );
    assert!(
        !end_state.contains("cryptographic zeroization"),
        "LIFECYCLE_END_STATE must not contain cryptographic zeroization: got '{end_state}'"
    );

    let post_op = exp_engine.resolve_token("LIFECYCLE_POST_OP_STATE", None);
    assert!(
        !post_op.contains("kinetic impact"),
        "LIFECYCLE_POST_OP_STATE must not contain kinetic impact: got '{post_op}'"
    );
    assert!(
        !post_op.contains("zeroization"),
        "LIFECYCLE_POST_OP_STATE must not contain zeroization: got '{post_op}'"
    );

    let mit_type = exp_engine.resolve_token("CONTAINMENT_MITIGATION_TYPE", None);
    assert!(
        !mit_type.contains("ZEROIZATION"),
        "CONTAINMENT_MITIGATION_TYPE must not contain ZEROIZATION: got '{mit_type}'"
    );

    let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
    let handoff_section = engine.resolve_token("SUPERVISORY_AUTHORITY_HANDOFF_SECTION", None);
    assert_ne!(
        handoff_section, "N/A",
        "SUPERVISORY_AUTHORITY_HANDOFF_SECTION must be dynamically derived"
    );
    assert!(
        handoff_section.contains("sequenceDiagram"),
        "SUPERVISORY_AUTHORITY_HANDOFF_SECTION must contain Mermaid sequenceDiagram"
    );
    assert!(
        handoff_section.contains("ISO/IEC 27001"),
        "SUPERVISORY_AUTHORITY_HANDOFF_SECTION must cite ISO/IEC 27001 interface contract"
    );
    assert!(
        !handoff_section.contains("VehicleController"),
        "Must not contain hardcoded UAS VehicleController"
    );
    assert!(
        !handoff_section.contains("PrimaryConsole"),
        "Must not contain hardcoded PrimaryConsole"
    );
}

#[test]
fn test_defect_316_synthesize_subsystem_architecture_provenance_columns(
) -> Result<(), Box<dyn std::error::Error>> {
    let sysml_code = r#"package AutonomousPlatform {
    part def PrimaryOBC {
        doc /* Core onboard computer */
        inout port c2_link : C2Interface;
        out port actuator_bus : MotorInterface;
    }
}
"#;
    let temp_sysml = std::env::temp_dir().join("test_schema_316.sysml");
    std::fs::write(&temp_sysml, sysml_code)?;
    let mut engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
    engine.ingest_file(&temp_sysml);
    let subsys_sec = engine.resolve_token("SUBSYSTEM_ARCHITECTURE_SECTION", None);
    std::fs::remove_file(&temp_sysml).ok();

    assert!(
        subsys_sec.contains("| Port Name | Direction | Interface Type | Functional Binding / Interconnect | OEM / SSOT Source |"),
        "Interface table must contain '| OEM / SSOT Source |' column header: got:\n{subsys_sec}"
    );
    let expected_c2_link = format!("[c2_link]({}#L4) [TIER-1: OEM]", temp_sysml.display());
    assert!(
        subsys_sec.contains(&expected_c2_link),
        "Interface table must contain '{expected_c2_link}': got:\n{subsys_sec}"
    );
    let expected_actuator = format!("[actuator_bus]({}#L5) [TIER-1: OEM]", temp_sysml.display());
    assert!(
        subsys_sec.contains(&expected_actuator),
        "Interface table must contain '{expected_actuator}': got:\n{subsys_sec}"
    );

    assert!(
        subsys_sec.contains("| Resource Parameter | Nominal Allocation | Min (-15% Tolerance) | Max (+15% Tolerance) | Engineering Units | Allocation Description | OEM / SSOT Source |"),
        "Resource budgets table must contain '| OEM / SSOT Source |' column header: got:\n{subsys_sec}"
    );
    let expected_obc = format!("[PrimaryOBC]({}#L2) [TIER-1: OEM]", temp_sysml.display());
    assert!(
        subsys_sec.contains(&expected_obc),
        "Resource budgets table must contain '{expected_obc}': got:\n{subsys_sec}"
    );
    Ok(())
}

#[test]
fn test_cuas_coa_physical_limits_sysml_provenance_and_zero_placeholder_output(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_root = std::env::temp_dir().join(format!("deap_test_params_provenance_{}", std::process::id()));
    let schema_dir = temp_root.join("schema");
    let unit_dir = temp_root.join("docs/conops/units/conops");
    std::fs::create_dir_all(&schema_dir)?;
    std::fs::create_dir_all(&unit_dir)?;

    let schema_path = schema_dir.join("test_platform.sysml");
    let sysml_content = "package AutonomousPlatform {\n    attribute TOTAL_MTOW_KG = 25.0;\n    attribute V_MAX_MPS = 35.0;\n    attribute V_CRUISE_NOMINAL_MPS = 20.0;\n    attribute OPERATING_TEMPERATURE_MIN_C = -20.0;\n    attribute OPERATING_TEMPERATURE_MAX_C = +55.0;\n    part def GuidanceController {\n        doc /* Primary guidance and control computer */\n        inout port c2_bus : C2TelemetryPort;\n        out port actuator_cmd : ActuatorCommandPort;\n    }\n}\n";
    std::fs::write(&schema_path, sysml_content)?;

    let unit_path = unit_dir.join("01_METADATA_AND_OVERVIEW.md");
    let unit_template = "# 1. Metadata and Operational Overview\n- **System Identifier:** {{SYSTEM_IDENTIFIER}}\n- **Maximum Mass Envelope:** {{TOTAL_MTOW_KG}} kg\n- **Maximum Velocity Limit:** {{V_MAX_MPS}} m/s\n- **Nominal Cruise Velocity:** {{V_CRUISE_NOMINAL_MPS}} m/s\n- **Thermal Operating Envelope:** {{OPERATING_TEMPERATURE_MIN_C}} deg C to {{OPERATING_TEMPERATURE_MAX_C}} deg C\n\n{{SUBSYSTEM_ARCHITECTURE_SECTION}}\n";
    std::fs::write(&unit_path, unit_template)?;

    let mut engine = SysMLParameterBindingEngine::new(
        None,
        Some(schema_path.as_path()),
        Some(temp_root.as_path()),
        true,
        None,
    );
    engine.ingest_file(&schema_path);

    assert_eq!(engine.parameter_bindings.get("TOTAL_MTOW_KG").map(String::as_str), Some("25.0"));
    assert_eq!(engine.parameter_bindings.get("V_MAX_MPS").map(String::as_str), Some("35.0"));
    assert_eq!(engine.resolve_token("SYSTEM_IDENTIFIER", None), "AutonomousPlatform");

    let raw_unit = std::fs::read_to_string(&unit_path)?;
    let rendered = engine.substitute(&raw_unit);
    std::fs::remove_dir_all(&temp_root).ok();

    assert!(!rendered.contains("{{"), "Rendered ConOps unit must contain zero unresolved '{{{{' placeholders: got:\n{rendered}");
    assert!(rendered.contains("[GuidanceController]("), "Rendered ConOps unit must preserve SysML source provenance citation");
    assert!(rendered.contains("[TIER-1: OEM]"), "Rendered ConOps unit must preserve [TIER-1: OEM] epistemic provenance tag");
    Ok(())
}

#[test]
fn test_issue_446_cuas_schema_physical_limits_override_uav_defaults(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_root = std::env::temp_dir().join(format!("deap_test_issue_446_{}", std::process::id()));
    let schema_dir = temp_root.join("schema");
    let unit_dir = temp_root.join("docs/conops/units/conops");
    std::fs::create_dir_all(&schema_dir)?;
    std::fs::create_dir_all(&unit_dir)?;

    let schema_path = schema_dir.join("cuas_system.sysml");
    let sysml_content = "package CuasDefenseSystem {\n    attribute sector_radius_m : Real = 5000.0;\n    attribute max_detection_range_m : Real = 8000.0;\n    attribute engagement_range_m : Real = 3000.0;\n    attribute max_latency_ms : Real = 50.0;\n    attribute total_mass_kg : Real = 150.0;\n    attribute dim_length_m : Real = 4.0;\n    attribute dim_width_m : Real = 2.5;\n    attribute dim_height_m : Real = 2.2;\n    part def TargetTrackingRadar {\n        attribute detection_range_m : Real = 8000.0;\n        attribute mass_kg : Real = 75.0;\n    }\n    part def RfMitigationEffector {\n        attribute engagement_range_m : Real = 3000.0;\n        attribute mass_kg : Real = 45.0;\n    }\n}\n";
    std::fs::write(&schema_path, sysml_content)?;

    let unit_path = unit_dir.join("01_METADATA_AND_OVERVIEW.md");
    let unit_template = "### 1.3.3 Master Physical Limits Table\n| Parameter ID | Bounding Parameter Name | Parametric Symbol | Threshold (Boundary Limit) | Objective (Nominal Target) | Engineering Unit | Normative / Safety Basis |\n| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n| **PL-01** | Maximum Operating Gross Mass (M_max) | m_max | <= {{M_MAX_LIMIT_KG}} | {{M_MAX_NOMINAL_KG}} | kg | Certified maximum structural mass limit |\n| **PL-03** | Physical Dimensions (Length x Width x Height) | L_x, L_y, L_z | <= {{DIM_MAX_L_M}} x {{DIM_MAX_W_M}} x {{DIM_MAX_H_M}} | {{DIM_NOM_L_M}} x {{DIM_NOM_W_M}} x {{DIM_NOM_H_M}} | m | Spatial transport and operational clearance envelope |\n| **PL-07** | Maximum Operating Spatial Boundary | d_max | <= {{SPATIAL_LIMIT_MAX_M}} | {{SPATIAL_LIMIT_NOMINAL_M}} | {{SPATIAL_LIMIT_UNIT:m}} | Maximum certified operational boundary limit |\n| **PL-08** | Command & Control (C2) Datalink Range | Range_C2 | >= {{C2_RANGE_MIN_KM}} | {{C2_RANGE_NOMINAL_KM}} | km | Beyond-Line-of-Sight / Line-of-Sight C2 range margin |\n";
    std::fs::write(&unit_path, unit_template)?;

    let mut engine = SysMLParameterBindingEngine::new(
        None,
        Some(schema_path.as_path()),
        Some(temp_root.as_path()),
        true,
        None,
    );
    engine.ingest_file(&schema_path);
    engine.derive_master_physical_limits_from_ast();

    assert_eq!(engine.resolve_token("M_MAX_LIMIT_KG", None), "150.0");
    assert_eq!(engine.resolve_token("SPATIAL_LIMIT_MAX_M", None), "5000.0");
    assert_eq!(engine.resolve_token("DIM_MAX_L_M", None), "4.0");
    assert_eq!(engine.resolve_token("DIM_MAX_W_M", None), "2.5");
    assert_eq!(engine.resolve_token("DIM_MAX_H_M", None), "2.2");
    assert_ne!(engine.resolve_token("C2_RANGE_MIN_KM", None), "N/A");

    let raw_unit = std::fs::read_to_string(&unit_path)?;
    let rendered = engine.substitute(&raw_unit);
    std::fs::remove_dir_all(&temp_root).ok();

    for forbidden in ["<= 25.0", "<= 120.0", "10.0 x 10.0 x 10.0", ">= N/A"] {
        assert!(!rendered.contains(forbidden), "ConOps output must not contain '{forbidden}'");
    }
    Ok(())
}
