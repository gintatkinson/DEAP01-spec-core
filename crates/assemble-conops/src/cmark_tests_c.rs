//! Unit tests for ConOps & Mission Intent unit integrity, link relativization, and resource decontamination (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).

use std::fs;
use std::path::{Path, PathBuf};

use super::*;

#[test]
fn test_relativize_markdown_links_and_unit_integrity() -> Result<(), Box<dyn std::error::Error>> {
    let text = "Check out [Schema](../../schema/platform.sysml) and [Anchor](#heading) and [Ext](https://example.com).";
    let out_file = Path::new("/workspace/docs/conops/CONOPS.md");
    let ws = Path::new("/workspace");
    let relativized = relativize_markdown_links(text, out_file, ws);
    assert!(relativized.contains("[Anchor](#heading)"));
    assert!(relativized.contains("[Ext](https://example.com)"));

    let temp_dir = std::env::temp_dir().join("test_conops_units");
    fs::create_dir_all(&temp_dir)?;
    let unit_file = temp_dir.join("01_TEST.md");
    fs::write(&unit_file, "# Unit 1\nClean text without tokens.")?;
    let (is_valid, errors) = validate_unit_integrity(&[unit_file], None);
    assert!(is_valid);
    assert!(errors.is_empty());
    fs::remove_dir_all(&temp_dir)?;

    let temp_bad = std::env::temp_dir().join("test_conops_units_bad");
    fs::create_dir_all(&temp_bad)?;
    let bad_file = temp_bad.join("01_BAD.md");
    fs::write(&bad_file, "# Unit 1\nUnresolved {{NONEXISTENT_TOKEN_12345}}.")?;
    let (is_valid_bad, errors_bad) = validate_unit_integrity(&[bad_file], None);
    assert!(!is_valid_bad);
    assert!(errors_bad.first().ok_or("missing err")?.contains("NONEXISTENT_TOKEN_12345"));
    fs::remove_dir_all(&temp_bad)?;
    Ok(())
}

#[test]
fn test_assemble_document_extracts_metadata_via_pulldown_cmark_ast(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join(format!("test_cmark_meta_{}", std::process::id()));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }
    fs::create_dir_all(&temp_dir)?;
    let unit_file = temp_dir.join("01_METADATA_AND_OVERVIEW.md");
    fs::write(
        &unit_file,
        "| Attribute | Value |\n| :--- | :--- |\n| **Title** | AST Extracted ConOps |\n| **Version** | 2.4.0 |\n| **Date** | 2026-11-01 |\n\n# Primary Heading\n\n## 1. Scope\n\n```python\n# code comment not demoted\n```\n",
    )?;
    let mut engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
    let (assembled, errors) = assemble_document(&temp_dir, None, (None, None), &mut engine, None);
    assert!(errors.is_empty(), "Unexpected errors: {errors:?}");
    assert!(assembled.contains("| **Title** | AST Extracted ConOps |"));
    assert!(assembled.contains("| **Version** | 2.4.0 |"));
    assert!(assembled.contains("| **Date** | 2026-11-01 |"));
    assert!(assembled.contains("# code comment not demoted"));
    fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

fn repo_root_path() -> Result<PathBuf, &'static str> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest_dir.parent().ok_or("no p1")?.parent().ok_or("no p2")?.to_path_buf())
}

fn assert_unit_file_decontaminated(
    path: &Path,
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
    assert!(!raw.contains('\u{2014}'));
    assert!(!raw.contains('\u{2013}'));
    Ok(())
}

#[test]
fn test_canonical_unit_05_integrity_and_alias_probe() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo_root_path()?;
    let unit_05_path = repo_root
        .join("skills/spec-conops-engineering/resources/units/conops/05_OPERATIONAL_STATE_SPACE_AND_RISK.md");
    if unit_05_path.is_file() {
        assert_unit_file_decontaminated(
            &unit_05_path,
            &[
                "AIR_DENSITY", "t_{\\mathrm{fall}}", "d_{\\mathrm{wind}}", "FRONTAL_AREA",
                "DRAG_COEFFICIENT", "Ground Risk Buffer", "Ground Risk Class",
                "Under ISO/IEC/IEEE 15026-3 / ISO/IEC/IEEE 15288:2023",
                "containment geometry (ISO/IEC/IEEE 15026-3",
            ],
        )?;
        let raw_content = fs::read_to_string(&unit_05_path)?;
        for required in ["ISO/IEC/IEEE 15026-3", "E_{\\mathrm{boundary\\_impact}}", "kinematic trajectory dynamics", "boundary containment geometry"] {
            assert!(raw_content.contains(required));
        }
        let lower = raw_content.to_lowercase();
        for concept in ["sora", "jarus", "bvlos", "vlos", "airspace", "flight termination", "aerodynamic drag", "ballistic fall", "wind drift"] {
            assert!(!lower.contains(concept));
        }
    }
    let temp_dir = std::env::temp_dir().join(format!("test_alias_05_{}", std::process::id()));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }
    let conops_temp = temp_dir.join("conops");
    fs::create_dir_all(&conops_temp)?;
    fs::write(
        conops_temp.join("05_AIRSPACE_AND_SORA_RISK.md"),
        "## 5. Operational State Space\n\nContent via historical alias.",
    )?;
    let mut engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
    let (assembled_text, errors) = assemble_document(&conops_temp, Some("CONOPS"), (Some("1.0.0"), Some("2026-10-14")), &mut engine, None);
    assert!(errors.is_empty());
    assert!(assembled_text.contains("Content via historical alias"));
    fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

#[test]
fn test_canonical_architecture_pace_and_env_units_decontamination(
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo_root_path()?;
    let conops_units_dir = repo_root.join("skills/spec-conops-engineering/resources/units/conops");
    assert_unit_file_decontaminated(
        &conops_units_dir.join("07_OPTX_EXCHANGES.md"),
        &["ARINC", "avionic", "DO-178", "DO-254", "DAL-A", "DAL-B", "Control Surface Deflections", "Barometric Static Pressure", "Dynamic Bingo", "ROE Arming Keys", "ASTM F3411", "ASD-STAN", "ballistic recovery", "Squib"],
    )?;
    assert_unit_file_decontaminated(&conops_units_dir.join("04_SYSTEM_ARCHITECTURE.md"), &["Encrypted PACE Communications", "MATLAB"])?;
    assert_unit_file_decontaminated(&conops_units_dir.join("08_SYSTEM_ARCHITECTURE.md"), &["ARINC", "STANAG", "MIL-STD-1553"])?;

    let unit_12_path = conops_units_dir.join("12_EMERGENCY_DECISION_MATRIX.md");
    if unit_12_path.is_file() {
        let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
        let text = fs::read_to_string(&unit_12_path)?;
        let substituted = engine.substitute(&text);
        assert!(!substituted.contains("{{"));
        assert!(!substituted.to_lowercase().contains("parachute"));
    }

    let pace_path = repo_root.join("skills/spec-conops-engineering/resources/units/mission_intent/05_PACE_C2_PLAN.md");
    if pace_path.is_file() {
        assert_unit_file_decontaminated(&pace_path, &["MIL-STD-188-220", "MIL-STD-882E", "STANAG", "CJCSI", "5.0s", "2.0s", "15.0s", "30.0s", "tau_loss = 5.0"])?;
    }

    let unit_08_path = conops_units_dir.join("08_ENVIRONMENTAL_MIL_STD_810H.md");
    if unit_08_path.is_file() {
        assert_unit_file_decontaminated(
            &unit_08_path,
            &["MIL-STD-810H", "MIL-STD-461G", "MIL-STD-1472H", "Air Transport", "Air Carriage", "Rapid Decompression", "altitude ceiling", "cargo hold", "radome", "telemetry mast", "Ground Support Equipment", "Red China Clay", "gunfire vibration"],
        )?;
    }
    Ok(())
}

#[test]
fn test_canonical_units_01_09_10_and_governance_rule() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = repo_root_path()?;
    let rule_path = repo_root.join("rules/conops-mission-intent-integrity.md");
    if rule_path.is_file() {
        let content = fs::read_to_string(&rule_path)?.to_lowercase();
        for concept in ["civil airborne", "bird strike", "mid-air", "airframe limits", "pitot probe", "propeller delamination", "pilot fatigue", "do-178c", "arp4761", "arp4754a", "h_max"] {
            assert!(!content.contains(concept));
        }
    }

    let mi_09 = repo_root.join("skills/spec-conops-engineering/resources/units/mission_intent/09_BINGO_ENERGY_MATH.md");
    if mi_09.is_file() {
        assert_unit_file_decontaminated(&mi_09, &["bingo", "lz-primary", "lz-divert", "landing zone", "airfield", "aerodrome", "runway", "airspeed", "glide slope"])?;
    }

    let conops_dir = repo_root.join("skills/spec-conops-engineering/resources/units/conops");
    let u01 = conops_dir.join("01_METADATA_AND_OVERVIEW.md");
    if u01.is_file() {
        assert_unit_file_decontaminated(&u01, &["subgraph AuxSupport"])?;
    }
    let u09 = conops_dir.join("09_SCENARIOS_AND_TIMELINES.md");
    if u09.is_file() {
        assert_unit_file_decontaminated(&u09, &["Phase_Ingress_StationKeeping", "Phase_PID_InterlockCheck", "Phase_DualConsent_ArmingExecution", "Phase_PostAction_AssessmentDump"])?;
    }
    let u10 = conops_dir.join("10_MAINTENANCE_AND_GSE_SUPPORT.md");
    if u10.is_file() {
        assert_unit_file_decontaminated(&u10, &["Step 1: System Ingress and<br/>Safe-State Disarm"])?;
    }
    Ok(())
}
