//! Unit tests for ConOps architecture (Units 04/07/08), emergency decision matrix (Unit 12), and PACE C2 (Unit 05).

use std::fs;
use std::path::PathBuf;

use super::*;
use crate::params::SysMLParameterBindingEngine;

fn assert_unit_file_decontaminated(
    path: &std::path::Path,
    forbidden: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    if !path.is_file() {
        return Ok(());
    }
    let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
    let (is_valid, errors) =
        validate_unit_integrity(std::slice::from_ref(&path.to_path_buf()), Some(&engine));
    assert!(is_valid, "Unit integrity validation failed: {errors:?}");
    assert!(errors.is_empty());

    let raw = fs::read_to_string(path)?;
    for term in forbidden {
        assert!(!raw.contains(term), "Unit {:?} must not contain '{term}'", path.file_name());
    }
    assert!(!raw.contains('\u{2014}'), "Unit must not contain em dash");
    assert!(!raw.contains('\u{2013}'), "Unit must not contain en dash");
    Ok(())
}

#[test]
fn test_canonical_unit_system_architecture_and_optx_decontamination(
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .ok_or("no p1")?
        .parent()
        .ok_or("no p2")?;
    let conops_units_dir =
        repo_root.join("skills/spec-conops-engineering/resources/units/conops");

    assert_unit_file_decontaminated(
        &conops_units_dir.join("07_OPTX_EXCHANGES.md"),
        &[
            "ARINC",
            "avionic",
            "DO-178",
            "DO-254",
            "DAL-A",
            "DAL-B",
            "Control Surface Deflections",
            "Barometric Static Pressure",
            "Dynamic Bingo",
            "ROE Arming Keys",
            "ASTM F3411",
            "ASD-STAN",
            "ballistic recovery",
            "Squib",
        ],
    )?;

    assert_unit_file_decontaminated(
        &conops_units_dir.join("04_SYSTEM_ARCHITECTURE.md"),
        &["Encrypted PACE Communications", "MATLAB"],
    )?;

    assert_unit_file_decontaminated(
        &conops_units_dir.join("08_SYSTEM_ARCHITECTURE.md"),
        &["ARINC", "STANAG", "MIL-STD-1553"],
    )?;
    Ok(())
}

#[test]
fn test_canonical_unit_12_and_lifecycle_containment_decontamination(
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .ok_or("no p1")?
        .parent()
        .ok_or("no p2")?;
    let conops_units_dir =
        repo_root.join("skills/spec-conops-engineering/resources/units/conops");

    let unit_12_path = conops_units_dir.join("12_EMERGENCY_DECISION_MATRIX.md");
    if unit_12_path.is_file() {
        let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
        let (is_valid, errors) =
            validate_unit_integrity(std::slice::from_ref(&unit_12_path), Some(&engine));
        assert!(
            is_valid,
            "Unit 12 integrity validation failed: {:?}",
            errors
        );
        assert!(errors.is_empty());

        let text = fs::read_to_string(&unit_12_path)?;
        let substituted = engine.substitute(&text);
        assert!(
            !substituted.contains("{{"),
            "Unit 12 must have zero unresolved tokens"
        );
        assert!(
            !substituted.contains('\u{2014}'),
            "Unit 12 must not contain em dash"
        );
        assert!(
            !substituted.to_lowercase().contains("parachute"),
            "Unit 12 must not contain parachute"
        );
    }

    let mut exp_bindings = std::collections::BTreeMap::new();
    exp_bindings.insert("IS_EXPENDABLE".to_string(), "true".to_string());
    let exp_engine =
        SysMLParameterBindingEngine::new(Some(exp_bindings), None, None, false, None);

    let unit_05_path = conops_units_dir.join("05_OPERATIONAL_STATE_SPACE_AND_RISK.md");
    if unit_05_path.is_file() {
        let (is_valid, errors) =
            validate_unit_integrity(std::slice::from_ref(&unit_05_path), Some(&exp_engine));
        assert!(
            is_valid,
            "Unit 05 expendable integrity failed: {:?}",
            errors
        );
        let text = fs::read_to_string(&unit_05_path)?;
        let substituted = exp_engine.substitute(&text);
        assert!(
            !substituted.to_lowercase().contains("parachute"),
            "Expendable Unit 05 must not contain parachute"
        );
    }
    Ok(())
}

#[test]
fn test_canonical_unit_05_pace_c2_plan_decontamination() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .ok_or("no p1")?
        .parent()
        .ok_or("no p2")?;
    let pace_path = repo_root.join(
        "skills/spec-conops-engineering/resources/units/mission_intent/05_PACE_C2_PLAN.md",
    );

    if !pace_path.is_file() {
        return Ok(());
    }

    assert_unit_file_decontaminated(
        &pace_path,
        &[
            "MIL-STD-188-220",
            "MIL-STD-882E",
            "STANAG",
            "CJCSI",
            "5.0s",
            "2.0s",
            "15.0s",
            "30.0s",
            "tau_loss = 5.0",
        ],
    )?;

    let raw = fs::read_to_string(&pace_path)?;
    for required in [
        "IEEE Std 1558-2020",
        "ISO/IEC/IEEE 15288:2023",
        "tau_timeout_Primary",
        "tau_hysteresis_Primary",
        "OPERATOR_CONFIGURED",
    ] {
        assert!(raw.contains(required), "Unit 05 PACE C2 Plan must contain {required}");
    }
    Ok(())
}
