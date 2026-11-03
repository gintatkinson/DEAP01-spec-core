//! Unit tests for CommonMark AST parameter binding, schema ingestion, and Level 1B sanitization (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).

use std::collections::BTreeMap;

use super::*;

#[test]
fn test_parameter_substitution_and_inline_default() {
    let mut bindings = BTreeMap::new();
    bindings.insert("SYSTEM_IDENTIFIER".to_string(), "AutonomousPlatform".to_string());
    let engine = SysMLParameterBindingEngine::new(Some(bindings), None, None, false, None);
    let input = "System: {{SYSTEM_IDENTIFIER}} with baud {{BAUD_RATE:57600}} and default {{UNKNOWN_KEY:FallbackValue}}";
    let output = engine.substitute(input);
    assert!(output.contains("AutonomousPlatform"));
    assert!(output.contains("FallbackValue"));
}

#[test]
fn test_domain_sanitization_civilian_and_medical() {
    let engine = SysMLParameterBindingEngine::new(None, None, None, false, Some("medical"));
    let input = "The parachute deployment is authorized under ROE-01 for weapons release.";
    let output = engine.substitute(input);
    assert!(output.contains("failsafe joint brake"));
    assert!(output.contains("INT-01"));
    assert!(output.contains("high-consequence actuation"));
}

#[test]
fn test_default_domain_ontology_pure_schema_driven_no_kinetic_bias() {
    let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
    assert_eq!(engine.resolve_token("HIGH_CONSEQUENCE_ACTION", None), "high-consequence actuation");
    assert_eq!(engine.resolve_token("COLLATERAL_RISK_PHRASE", None), "adjacent operational risk");
    assert_eq!(engine.resolve_token("INTERLOCK_PREFIX", None), "INT");
    assert_eq!(engine.resolve_token("TARGET_VERIFICATION_ACRONYM", None), "PCV");
    assert_eq!(engine.resolve_token("INTERLOCK_POLICY_NAME", None), "Operational Safety Interlocks");
    assert_eq!(engine.resolve_token("INTERLOCK_SECTION_TITLE", None), "Safety Constraints & Subsystem Interlocks");
}

#[test]
fn test_interlock_prefix_saf_compatibility() {
    let mut bindings = BTreeMap::new();
    bindings.insert("INTERLOCK_PREFIX".to_string(), "SAF".to_string());
    let engine = SysMLParameterBindingEngine::new(Some(bindings), None, None, false, None);
    assert_eq!(engine.resolve_token("INTERLOCK_PREFIX", None), "SAF");
    assert_eq!(engine.resolve_token("INTERLOCK_01_TAG", None), "SAF-01");
    assert!(engine.substitute("Authorized under ROE-01.").contains("SAF-01"));
}

#[test]
fn test_mass_and_physics_derivation() {
    let mut bindings = BTreeMap::new();
    bindings.insert("TOTAL_MTOW_KG".to_string(), "100.0".to_string());
    bindings.insert("V_MAX_MPS".to_string(), "20.0".to_string());
    let engine = SysMLParameterBindingEngine::new(Some(bindings), None, None, false, None);
    assert_eq!(engine.resolve_token("TOTAL_MTOW_KG", None), "100.0");
    assert_eq!(engine.resolve_token("V_BOUNDARY_MAX_MPS", None), "20.0");
    assert_eq!(engine.resolve_token("E_BOUNDARY_IMPACT_JOULES", None), "20000.0");
    assert_eq!(engine.resolve_token("E_K_UNMITIGATED_JOULES", None), "20000.0");
    assert_eq!(engine.resolve_token("V_RESIDUAL_MPS", None), "1.5");
    assert_eq!(engine.resolve_token("E_K_MITIGATED_JOULES", None), "112.5");
    assert_ne!(engine.resolve_token("V_TERMINAL_UNMITIGATED_MPS", None), "N/A");
    assert!(!engine.resolve_token("DOMAIN_RISK_ASSESSMENT_CLAUSES", None).is_empty());
    assert_eq!(engine.resolve_token("SORA_GROUND_RISK_CLASS", None), "N/A");
    assert_eq!(engine.resolve_token("SORA_AIR_RISK_CLASS", None), "N/A");
    assert_eq!(engine.resolve_token("SAIL_LEVEL", None), "N/A");
}

#[test]
fn test_super_system_and_subsystem_architecture_synthesis_contract() -> Result<(), String> {
    let mut engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
    engine.ast_part_names.insert("Avionics".to_string());
    engine.ast_part_names.insert("Actuation".to_string());
    engine._derive_subsystem_architecture();
    let super_arch = engine
        .parameter_bindings
        .get("SUPER_SYSTEM_ARCHITECTURE")
        .ok_or_else(|| "missing SUPER_SYSTEM_ARCHITECTURE".to_string())?;
    for req in ["External_Actors", "Supervisory_Segment", "Support_Segment", "~~~", "CONN-", "• PORT-"] {
        assert!(super_arch.contains(req), "Super-system architecture must include {req}");
    }
    assert!(!super_arch.contains("Core_Tier"));
    let sub_arch = engine
        .parameter_bindings
        .get("SUBSYSTEM_ARCHITECTURE_SECTION")
        .ok_or_else(|| "missing SUBSYSTEM_ARCHITECTURE_SECTION".to_string())?;
    for bullet in [
        "- **Functional Purpose & Scope:**",
        "- **Physical & Logical Interface / Port Allocations:**",
        "- **Power, Mass & Resource Envelopes:**",
        "- **Operational Role & Statechart Integration:**",
        "- **Safety Invariants, Containment Interlocks & FMECA Linkage:**",
        "- **SSOT Ground Truth Mapping:**",
        "SysML v2: `part def Avionics`",
    ] {
        assert!(sub_arch.contains(bullet), "Subsystem architecture must contain {bullet}");
    }
    let seg_matrix = engine
        .parameter_bindings
        .get("SEGMENT_ALLOCATION_MATRIX")
        .ok_or_else(|| "missing SEGMENT_ALLOCATION_MATRIX".to_string())?;
    assert!(seg_matrix.contains("SSOT Ground Truth Source & Citation"));
    assert!(seg_matrix.contains("SysML v2: `part def Avionics`"));
    Ok(())
}

#[test]
fn test_issue_248_pure_schema_driven_containment_and_zero_fallback_recovery(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut exp_bindings = BTreeMap::new();
    exp_bindings.insert("IS_EXPENDABLE".to_string(), "true".to_string());
    let exp_engine = SysMLParameterBindingEngine::new(Some(exp_bindings), None, None, false, None);
    assert_ne!(exp_engine.resolve_token("PRIMARY_TERMINAL_TARGET", None), "Primary Recovery Base");
    assert_ne!(exp_engine.resolve_token("PRIMARY_RECOVERY_FACILITY", None), "Primary Recovery Base");
    assert_ne!(exp_engine.resolve_token("LIFECYCLE_TRANSIT_MODE", None), "Autonomous_RTB_Transit");
    assert_ne!(exp_engine.resolve_token("LIFECYCLE_FAILSAFE_SEQUENCE", None), "N/A");
    assert_ne!(exp_engine.resolve_token("CONTAINMENT_MITIGATION_TYPE", None), "N/A");

    let generic_engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
    let substituted = generic_engine.substitute("Emergency containment initiates parachute deployment for recovery.");
    assert!(!substituted.to_lowercase().contains("parachute"));

    let mut chute_bindings = BTreeMap::new();
    chute_bindings.insert("TOTAL_MTOW_KG".to_string(), "200.0".to_string());
    chute_bindings.insert("AIR_DENSITY_KGM3".to_string(), "1.2".to_string());
    chute_bindings.insert("S_CANOPY".to_string(), "50.0".to_string());
    chute_bindings.insert("PARACHUTE_DRAG_COEFFICIENT".to_string(), "1.5".to_string());
    let mut chute_engine = SysMLParameterBindingEngine::new(Some(chute_bindings), None, None, false, None);
    chute_engine.ast_part_names.insert("ParachuteRecoveryUnit".to_string());
    chute_engine._derive_quadratic_physics();
    let v_mit: f64 = chute_engine.resolve_token("V_TERMINAL_MITIGATED_MPS", None).parse()?;
    assert!((v_mit - 6.60).abs() < 0.1);
    Ok(())
}

#[test]
fn test_extract_subsystem_parts_preserves_source_provenance() -> Result<(), String> {
    let sysml_code = "package AutonomousPlatform {\n    part def PrimaryOBC {\n        doc /* Core onboard computer */\n        inout port c2_link : C2Interface;\n        out port actuator_bus : MotorInterface;\n    }\n}\n";
    let parts = extract_subsystem_parts_from_sysml(sysml_code, "schema/Platform.sysml");
    assert_eq!(parts.len(), 1);
    let obc = parts.first().ok_or_else(|| "missing obc".to_string())?;
    assert_eq!(obc.name, "PrimaryOBC");
    assert_eq!(obc.source_file, "schema/Platform.sysml");
    assert_eq!(obc.source_line, Some(2));
    assert_eq!(obc.epistemic_tier, "[TIER-1: OEM]");
    assert_eq!(obc.ports.len(), 2);
    let port0 = obc.ports.first().ok_or_else(|| "missing port0".to_string())?;
    assert_eq!(port0.name, "c2_link");
    assert_eq!(port0.direction, "INOUT");
    assert_eq!(port0.source_line, Some(4));
    let port1 = obc.ports.get(1).ok_or_else(|| "missing port1".to_string())?;
    assert_eq!(port1.name, "actuator_bus");
    assert_eq!(port1.direction, "OUT");
    assert_eq!(port1.source_line, Some(5));
    Ok(())
}

#[test]
fn test_conops_scenarios_synthesize_from_ast_statechart() -> Result<(), Box<dyn std::error::Error>> {
    let sysml_model = "package MedicalInfusionPlatform {\n    part def InfusionPumpUnit {\n        doc /* Core infusion pump subsystem */\n        out port fluid_line : FlowInterface;\n    }\n    part def OcclusionSensor {\n        doc /* Upstream/downstream pressure monitoring */\n        in port pressure_tap : PressureInterface;\n    }\n    part def FlowController {\n        doc /* High-precision delivery controller */\n    }\n}\n";
    let temp_sysml = std::env::temp_dir().join("test_schema_317.sysml");
    std::fs::write(&temp_sysml, sysml_model)?;
    let mut engine = SysMLParameterBindingEngine::new(None, None, None, false, Some("medical"));
    engine.ingest_file(&temp_sysml);
    if temp_sysml.exists() {
        std::fs::remove_file(&temp_sysml)?;
    }
    for part_name in &["InfusionPumpUnit", "OcclusionSensor", "FlowController"] {
        assert!(engine.ast_part_names.contains(*part_name));
    }
    assert!(!engine.ast_part_names.contains("Phase_Ingress_StationKeeping"));
    let substituted = engine.substitute("Operational execution under ROE-01 commands weapons release.");
    assert!(!substituted.contains("ROE-01"));
    assert!(!substituted.contains("weapons release"));
    assert!(substituted.contains("INT-01"));
    assert!(substituted.contains("high-consequence actuation"));
    Ok(())
}

#[test]
fn test_ingest_markdown_file_via_pulldown_cmark_ast() -> Result<(), Box<dyn std::error::Error>> {
    let md_content = "# **AegisGuard Autonomous Sentinel**\n\n```markdown\n# Fake Heading Inside Code Fence\n| FakeKey | FakeVal |\n|---|---|\n| TOTAL_MTOW_KG | 9999.0 |\n```\n\n| Parameter | Value |\n| :--- | :--- |\n| **TOTAL_MTOW_KG** | 85.0 |\n| *V_MAX_MPS* | 42.0 |\n";
    let temp_md = std::env::temp_dir().join(format!("test_md_ast_ingest_{}.md", std::process::id()));
    std::fs::write(&temp_md, md_content)?;
    let mut engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
    let ok = engine.ingest_markdown_file(&temp_md);
    if temp_md.exists() {
        std::fs::remove_file(&temp_md)?;
    }
    assert!(ok);
    assert_eq!(engine.resolve_token("SYSTEM_IDENTIFIER", None), "AegisGuard Autonomous Sentinel");
    assert_eq!(engine.resolve_token("TOTAL_MTOW_KG", None), "85.0");
    assert_eq!(engine.resolve_token("V_MAX_MPS", None), "42.0");
    assert!(!engine.parameter_bindings.contains_key("FakeKey"));
    Ok(())
}

#[test]
fn test_sanitize_level_1b_polynomial_baud_opcodes_and_use_cases() {
    let raw = "Data link at 115200 baud with x^16 + x^12 + x^5 + 1 CRC-16 polynomial formula.";
    let sanitized = sanitize_level_1b_operational_text(raw);
    assert!(!sanitized.contains("115200"));
    assert!(!sanitized.contains("x^16"));
    assert!(sanitized.contains("Frame Integrity Check"));

    let raw_op = "Triggered by opcode 0x12 and UC-01 (UC-02 and UC-03) command.";
    let sanitized_op = sanitize_level_1b_operational_text(raw_op);
    assert!(!sanitized_op.contains("0x12"));
    assert!(!sanitized_op.contains("UC-01"));
    assert!(!sanitized_op.contains("UC-02"));
    assert!(!sanitized_op.contains("opcode"));

    let wrapped = wrap_mermaid_label("This is a very long subsystem descriptive label that exceeds maximum width limit", 25);
    assert!(wrapped.contains("<br/>"));

    let legacy_conops = "Maintenance Technician (UC-04) performing pre-sortie inspection under UC-01.";
    let sanitized_legacy = sanitize_level_1b_operational_text(legacy_conops);
    assert!(!sanitized_legacy.contains("UC-04"));
    assert!(!sanitized_legacy.contains("UC-01"));

    let compliant_conops = "Maintenance Technician (UCL-04) performing pre-sortie inspection under UCL-01.";
    let sanitized_compliant = sanitize_level_1b_operational_text(compliant_conops);
    assert!(sanitized_compliant.contains("UCL-04"));
    assert!(sanitized_compliant.contains("UCL-01"));
}
