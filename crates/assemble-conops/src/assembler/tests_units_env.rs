//! Unit tests for ConOps Unit 08 (environmental limits), governance rule decontamination, and Mission Intent Unit 09.

use std::fs;
use std::path::PathBuf;

use super::*;
use crate::params::SysMLParameterBindingEngine;

#[test]
fn test_canonical_unit_08_environmental_integrity_and_decontamination(
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .ok_or("no p1")?
        .parent()
        .ok_or("no p2")?;
    let conops_units_dir =
        repo_root.join("skills/spec-conops-engineering/resources/units/conops");
    let unit_08_path = conops_units_dir.join("08_ENVIRONMENTAL_MIL_STD_810H.md");

    if !unit_08_path.is_file() {
        return Ok(());
    }

    let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
    let (is_valid, errors) =
        validate_unit_integrity(std::slice::from_ref(&unit_08_path), Some(&engine));
    assert!(is_valid, "Unit 08 integrity validation failed: {errors:?}");
    assert!(errors.is_empty());

    let raw = fs::read_to_string(&unit_08_path)?;
    for forbidden in [
        "MIL-STD-810H",
        "MIL-STD-461G",
        "MIL-STD-1472H",
        "Air Transport",
        "Air Carriage",
        "Rapid Decompression",
        "altitude ceiling",
        "cargo hold",
        "radome",
        "telemetry mast",
        "Ground Support Equipment",
        "Red China Clay",
        "gunfire vibration",
        "\u{2014}",
        "\u{2013}",
    ] {
        assert!(!raw.contains(forbidden), "Unit 08 must not contain {forbidden}");
    }

    for required in ["ISO/IEC/IEEE 15288:2023", "ISO/IEC/IEEE 29148:2018", "IEC 60529"] {
        assert!(raw.contains(required), "Unit 08 must cite {required}");
    }

    for env_id in ["ENV-01", "ENV-02", "ENV-03", "ENV-04", "ENV-05", "ENV-06", "ENV-07"] {
        assert!(
            raw.contains(&format!("**{env_id}**")),
            "Unit 08 must contain abstract environmental matrix row {env_id}"
        );
    }

    let substituted = engine.substitute(&raw);
    assert!(
        !substituted.contains("{{"),
        "Unit 08 substituted output must contain zero unresolved placeholders"
    );
    Ok(())
}

#[test]
fn test_governance_rule_conops_mission_intent_integrity_decontamination(
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .ok_or("no p1")?
        .parent()
        .ok_or("no p2")?;
    let rule_path = repo_root.join("rules/conops-mission-intent-integrity.md");

    assert!(
        rule_path.is_file(),
        "rules/conops-mission-intent-integrity.md must exist"
    );
    let content = fs::read_to_string(&rule_path)?;

    let forbidden_concepts = [
        "civil airborne",
        "bird strike",
        "mid-air",
        "airframe limits",
        "pitot probe",
        "propeller delamination",
        "pilot fatigue",
        "DO-178C",
        "ARP4761",
        "ARP4754A",
        "h_max",
    ];

    let mut found_violations = Vec::new();
    for concept in &forbidden_concepts {
        if content.to_lowercase().contains(&concept.to_lowercase()) {
            found_violations.push(*concept);
        }
    }

    assert!(
        found_violations.is_empty(),
        "rules/conops-mission-intent-integrity.md contains forbidden aerospace domain concepts: {:?}",
        found_violations
    );
    Ok(())
}

fn assert_unit_09_jargon_and_unit_decontamination(raw: &str) {
    let lower = raw.to_lowercase();
    for term in [
        "bingo",
        "lz-primary",
        "lz-divert",
        "landing zone",
        "elevation",
        "obstacle clearance",
        "flight level",
        "flight ceiling",
        "airfield",
        "aerodrome",
        "runway",
        "airspeed",
        "glide slope",
    ] {
        assert!(!lower.contains(term), "Unit 09 contains forbidden aviation jargon: {term}");
    }

    for unit in ["knots", "knot", "kts", "feet", "nautical mile", "fpm"] {
        for line in raw.lines() {
            for word in line.to_lowercase().split_whitespace() {
                let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric());
                assert_ne!(cleaned, unit, "Unit 09 contains forbidden non-SI unit: {unit}");
            }
        }
    }
    assert!(!raw.contains('\u{2014}'), "Unit 09 must not contain Unicode em dash");
    assert!(!raw.contains('\u{2013}'), "Unit 09 must not contain Unicode en dash");
}

#[test]
fn test_canonical_mission_intent_unit_09_resource_math_decontamination(
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .ok_or("no p1")?
        .parent()
        .ok_or("no p2")?;
    let unit_09_path = repo_root.join(
        "skills/spec-conops-engineering/resources/units/mission_intent/09_BINGO_ENERGY_MATH.md",
    );

    assert!(unit_09_path.is_file(), "09_BINGO_ENERGY_MATH.md must exist on disk");

    let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
    let (is_valid, errors) =
        validate_unit_integrity(std::slice::from_ref(&unit_09_path), Some(&engine));
    assert!(is_valid, "Unit 09 integrity validation failed: {errors:?}");
    assert!(errors.is_empty());

    let raw = fs::read_to_string(&unit_09_path)?;
    for required in [
        "## 9. Critical Energy Threshold Mathematics & Secondary Divert Protocols",
        "### 9.1 Parametric Closed-Loop Resource Reserve Formulation",
        "### 9.2 Secondary and Tertiary Divert Recovery Protocols",
        "R_{\\mathrm{threshold}}(t)",
    ] {
        assert!(raw.contains(required), "Unit 09 must contain '{required}'");
    }
    assert!(
        raw.contains("INCOSE Systems Engineering Handbook v5.0 (§3.2)")
            || raw.contains("INCOSE SEH v5.0 §3.2")
    );
    assert!(
        raw.contains(r"\text{Ratio}_{\text{reserve\_min}}")
            || raw.contains(r"Ratio_{\text{reserve\_min}}")
    );

    assert_unit_09_jargon_and_unit_decontamination(&raw);

    let substituted = engine.substitute(&raw);
    assert!(
        !substituted.contains("{{"),
        "Unit 09 substituted output must contain zero unresolved placeholders"
    );
    Ok(())
}
