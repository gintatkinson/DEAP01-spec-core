//! Unit tests for link relativization, unit integrity, Unit 05 decontamination, and `pulldown-cmark` AST metadata extraction.

use std::fs;
use std::path::{Path, PathBuf};

use super::*;
use crate::params::SysMLParameterBindingEngine;

#[test]
fn test_relativize_markdown_links() {
    let text = "Check out [Schema](../../schema/platform.sysml) and [Anchor](#heading) and [Ext](https://example.com).";
    let out_file = Path::new("/workspace/docs/conops/CONOPS.md");
    let ws = Path::new("/workspace");
    let relativized = relativize_markdown_links(text, out_file, ws);
    assert!(relativized.contains("[Anchor](#heading)"));
    assert!(relativized.contains("[Ext](https://example.com)"));
}

#[test]
fn test_validate_unit_integrity_with_clean_content() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("test_conops_units");
    fs::create_dir_all(&temp_dir)?;
    let unit_file = temp_dir.join("01_TEST.md");
    fs::write(&unit_file, "# Unit 1\nClean text without tokens.")?;

    let (is_valid, errors) = validate_unit_integrity(&[unit_file], None);
    assert!(is_valid);
    assert!(errors.is_empty());
    fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

#[test]
fn test_validate_unit_integrity_with_unresolved_placeholder(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("test_conops_units_bad");
    fs::create_dir_all(&temp_dir)?;
    let unit_file = temp_dir.join("01_BAD.md");
    fs::write(
        &unit_file,
        "# Unit 1\nUnresolved {{NONEXISTENT_TOKEN_12345}}.",
    )?;

    let (is_valid, errors) = validate_unit_integrity(&[unit_file], None);
    assert!(!is_valid);
    assert!(!errors.is_empty());
    assert!(errors
        .first()
        .ok_or("missing err")?
        .contains("NONEXISTENT_TOKEN_12345"));
    fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

#[test]
fn test_assemble_document_extracts_metadata_via_pulldown_cmark_ast(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join(format!("test_cmark_meta_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
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

#[test]
fn test_canonical_unit_05_integrity_and_decontamination() -> Result<(), Box<dyn std::error::Error>>
{
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .ok_or("no p1")?
        .parent()
        .ok_or("no p2")?;
    let unit_05_path = repo_root
        .join("skills/spec-conops-engineering/resources/units/conops/05_OPERATIONAL_STATE_SPACE_AND_RISK.md");

    if !unit_05_path.is_file() {
        return Ok(());
    }

    let engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
    let (is_valid, errors) =
        validate_unit_integrity(std::slice::from_ref(&unit_05_path), Some(&engine));
    assert!(is_valid, "Unit 05 integrity validation failed: {errors:?}");
    assert!(errors.is_empty());

    let raw_content = fs::read_to_string(&unit_05_path)?;
    for forbidden in [
        "AIR_DENSITY",
        "t_{\\mathrm{fall}}",
        "d_{\\mathrm{wind}}",
        "FRONTAL_AREA",
        "DRAG_COEFFICIENT",
        "Ground Risk Buffer",
        "Ground Risk Class",
        "\u{2014}",
        "\u{2013}",
        "Under ISO/IEC/IEEE 15026-3 / ISO/IEC/IEEE 15288:2023",
        "containment geometry (ISO/IEC/IEEE 15026-3",
    ] {
        assert!(!raw_content.contains(forbidden), "Unit 05 must not contain {forbidden}");
    }
    for required in [
        "ISO/IEC/IEEE 15026-3",
        "E_{\\mathrm{boundary\\_impact}}",
        "kinematic trajectory dynamics",
        "boundary containment geometry",
    ] {
        assert!(raw_content.contains(required), "Unit 05 must contain {required}");
    }
    Ok(())
}

fn assert_unit_05_si_and_aviation_decontamination(raw_content: &str) {
    let lower = raw_content.to_lowercase();
    for unit in ["knots", "knot", "kts", "feet", "foot", "mph", "nautical mile"] {
        assert!(!lower.contains(unit), "Unit 05 must not contain non-SI aviation unit '{unit}'");
    }
    for line in raw_content.lines() {
        for word in line.split(|c: char| !c.is_alphanumeric() && c != '_') {
            assert!(word != "ft" && word != "FT", "Unit 05 must not contain 'ft' in line: {line}");
            assert!(word != "nm" && word != "NM", "Unit 05 must not contain 'nm' in line: {line}");
            assert!(
                word != "AGL" && word != "MSL",
                "Unit 05 must not contain altitude reference '{word}' in line: {line}"
            );
            assert!(
                !word.starts_with("FL")
                    || word.len() < 4
                    || !word[2..].chars().all(|c| c.is_ascii_digit()),
                "Unit 05 must not contain flight level '{word}' in line: {line}"
            );
        }
    }
    for concept in [
        "sora",
        "jarus",
        "bvlos",
        "vlos",
        "airspace",
        "flight termination",
        "aerodynamic drag",
        "ballistic fall",
        "wind drift",
    ] {
        assert!(!lower.contains(concept), "Unit 05 must not contain aviation concept '{concept}'");
    }
    assert!(raw_content.contains("ISO/IEC/IEEE 15288:2023 §6.4.4"));
    assert!(raw_content.contains("ISO/IEC/IEEE 15026-3"));
    for si in ["m/s", "m/s^2", "kg", "J", "s"] {
        assert!(raw_content.contains(si), "Unit 05 must define normalized SI unit '{si}'");
    }
}

#[test]
fn test_canonical_unit_05_non_si_aviation_units_and_alias_probe(
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .ok_or("no p1")?
        .parent()
        .ok_or("no p2")?;
    let unit_05_path = repo_root
        .join("skills/spec-conops-engineering/resources/units/conops/05_OPERATIONAL_STATE_SPACE_AND_RISK.md");

    if !unit_05_path.is_file() {
        return Ok(());
    }

    let raw_content = fs::read_to_string(&unit_05_path)?;
    assert_unit_05_si_and_aviation_decontamination(&raw_content);

    let skill_path = repo_root.join("skills/spec-conops-engineering/SKILL.md");
    if skill_path.is_file() {
        let skill_content = fs::read_to_string(&skill_path)?;
        assert!(
            !skill_content.contains("JARUS SORA"),
            "skills/spec-conops-engineering/SKILL.md must not contain hardcoded 'JARUS SORA'"
        );
    }

    let temp_dir = std::env::temp_dir().join(format!("test_alias_05_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    let conops_temp = temp_dir.join("conops");
    fs::create_dir_all(&conops_temp)?;

    let alias_file = conops_temp.join("05_AIRSPACE_AND_SORA_RISK.md");
    fs::write(
        &alias_file,
        "## 5. Operational State Space\n\nContent via historical alias.",
    )?;

    let mut engine = SysMLParameterBindingEngine::new(None, None, None, false, None);
    let (assembled_text, errors) = assemble_document(
        &conops_temp,
        Some("CONOPS"),
        (Some("1.0.0"), Some("2026-10-14")),
        &mut engine,
        None,
    );
    assert!(errors.is_empty(), "Assembly with alias failed: {errors:?}");
    assert!(
        assembled_text.contains("Content via historical alias"),
        "assemble_document must include 05_AIRSPACE_AND_SORA_RISK.md as fallback alias for Unit 05"
    );

    fs::remove_dir_all(&temp_dir)?;
    Ok(())
}
