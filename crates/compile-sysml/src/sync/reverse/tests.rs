#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Unit tests for reverse-sync frontmatter extraction, subdirectory scoping (`Issue #534`), and blueprint `SC-*` safety constraint extraction (`Issue #535`).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical
//! verification standards for `crates/compile-sysml/src/sync/reverse.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Verifies YAML frontmatter extraction, non-Agile `docs/designs/` table isolation, and deduplicated `SC-*` safety constraint extraction.
//!
//! ## 3. Preconditions
//! - Host temporary directory (`std::env::temp_dir()`) is writable.
//!
//! ## 4. Postconditions
//! - Temporary test workspaces are removed prior to test assertions.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(1)$ over bounded synthetic fixtures.
//! - Space Complexity: $O(1)$ auxiliary test memory.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Returns descriptive `Err(String)` if fixture setup or reverse synchronization fails.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0604] [REQ-SYSML-SYNC-REVERSE/SUBDIR-SCOPING] [REQ-SYSML-SYNC-REVERSE/BLUEPRINT-SC-EXTRACTION]

use std::path::Path;

use deap_core::sysml_cst::PartDef;

use super::*;

#[test]
fn test_parse_frontmatter_extraction() {
    let md = r#"---
title: "Feature 01: CoreSubsystem"
type: feature
part: CoreSubsystem
---

# Content
"#;
    let (fm, body) = parse_frontmatter(md);
    assert_eq!(
        fm.get("title").map(String::as_str),
        Some("Feature 01: CoreSubsystem")
    );
    assert_eq!(fm.get("type").map(String::as_str), Some("feature"));
    assert_eq!(fm.get("part").map(String::as_str), Some("CoreSubsystem"));
    assert!(body.contains("# Content"));
}

/// Regression test for Issue #534 verifying that non-Agile documents under `docs/designs/`
/// with 2-column Markdown tables containing `| Type | feature |` do not pollute `.pipeline/schema.sysml`.
///
/// Realises: [REQ-0604] [REQ-SYSML-SYNC-REVERSE/SUBDIR-SCOPING]
#[test]
fn test_reverse_sync_ignores_docs_designs_type_tables() -> Result<(), String> {
    let tmp_dir =
        std::env::temp_dir().join(format!("deap_issue_534_rev_sync_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp_dir);

    let pipeline_dir = tmp_dir.join(".pipeline");
    let designs_dir = tmp_dir.join("docs").join("designs");
    let features_dir = tmp_dir.join("docs").join("features");
    fs::create_dir_all(&pipeline_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&designs_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&features_dir).map_err(|e| e.to_string())?;

    let schema_path = pipeline_dir.join("schema.sysml");
    let digest_path = pipeline_dir.join("schema-digest.json");
    fs::write(
        &schema_path,
        "package TestSys {\n    part def CoreController;\n}\n",
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        designs_dir.join("blueprint-01-arch.md"),
        "# Blueprint 01\n\n| Attribute | Value |\n| :--- | :--- |\n| Type | feature |\n",
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        features_dir.join("fea-01.md"),
        "---\ntitle: \"Feature 01: CoreController\"\ntype: feature\npart: CoreController\n---\n\n# Feature 01\n\n- `+computeControl() : void`\n",
    )
    .map_err(|e| e.to_string())?;

    let base_pkg = PackageDef {
        name: "TestSys".to_string(),
        part_defs: vec![PartDef {
            name: "CoreController".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let opts = ReverseSyncOptions {
        docs_dir: tmp_dir.join("docs"),
        schema_path: Some(schema_path.clone()),
        output_path: schema_path.clone(),
        digest_path,
        allow_overwrite: true,
    };

    let (pkg, _) = reverse_sync_specs_to_sysml(Some(&base_pkg), &opts)?;
    let emitted_sysml = fs::read_to_string(&schema_path).map_err(|e| e.to_string())?;
    let _ = fs::remove_dir_all(&tmp_dir);

    assert!(!pkg.part_defs.iter().any(|p| p.name == "Blueprint01Arch"));
    assert!(!emitted_sysml.contains("Blueprint01Arch"));
    assert!(pkg.part_defs.iter().any(|p| p.name == "CoreController"));
    Ok(())
}

/// Writes the temporary blueprint and STPA matrix fixture files for Issue #535.
/// Realises: [REQ-0604] [REQ-SYSML-SYNC-REVERSE/BLUEPRINT-SC-EXTRACTION]
fn setup_issue_535_blueprint_sc_fixture(tmp_dir: &Path) -> Result<ReverseSyncOptions, String> {
    let pipeline_dir = tmp_dir.join(".pipeline");
    let designs_dir = tmp_dir.join("docs").join("designs");
    let safety_dir = tmp_dir.join("docs").join("architecture").join("safety");
    fs::create_dir_all(&pipeline_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&designs_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&safety_dir).map_err(|e| e.to_string())?;

    let schema_path = pipeline_dir.join("schema.sysml");
    let digest_path = pipeline_dir.join("schema-digest.json");
    fs::write(
        &schema_path,
        "package TestSys {\n    part def CoreController;\n}\n",
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        designs_dir.join("blueprint-01-core.md"),
        "---\ntitle: \"Blueprint 01: Core\"\ntype: blueprint\n---\n\n# Blueprint 01\n\n| SC ID | Safety Constraint | UCA | Hazard | Mitigation |\n| :--- | :--- | :--- | :--- | :--- |\n| `SC-01` | Controller must enforce bounds | UCA-01 | H-01 | Clamp output |\n| **SC-02** | Controller must reject stale telemetry | UCA-02 | H-02 | Check timestamp |\n| `SC-03` | Controller must validate checksum | UCA-03 | H-03 | Verify CRC |\n",
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        safety_dir.join("STPA_MATRIX.md"),
        "# STPA Matrix\n\n| SC ID | Safety Constraint | UCA | Hazard |\n| :--- | :--- | :--- | :--- |\n| **SC-01** | Controller must enforce bounds | UCA-01 | H-01 |\n",
    )
    .map_err(|e| e.to_string())?;

    Ok(ReverseSyncOptions {
        docs_dir: tmp_dir.join("docs"),
        schema_path: Some(schema_path.clone()),
        output_path: schema_path,
        digest_path,
        allow_overwrite: true,
    })
}

/// Regression test for Issue #535 verifying that `reverse_sync_specs_to_sysml` extracts
/// `SC-*` safety constraints from `docs/designs/blueprint-*.md` and deduplicates across `STPA_MATRIX.md`.
///
/// Realises: [REQ-0604] [REQ-SYSML-SYNC-REVERSE/BLUEPRINT-SC-EXTRACTION]
#[test]
fn test_reverse_sync_extracts_blueprint_sc_constraints() -> Result<(), String> {
    let tmp_dir =
        std::env::temp_dir().join(format!("deap_issue_535_rev_sync_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp_dir);
    let opts = setup_issue_535_blueprint_sc_fixture(&tmp_dir)?;

    let base_pkg = PackageDef {
        name: "TestSys".to_string(),
        part_defs: vec![PartDef {
            name: "CoreController".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let (pkg, _) = reverse_sync_specs_to_sysml(Some(&base_pkg), &opts)?;
    let emitted_sysml = fs::read_to_string(&opts.output_path).map_err(|e| e.to_string())?;
    let _ = fs::remove_dir_all(&tmp_dir);

    assert!(emitted_sysml.contains("assert constraint SC_01_SafetyConstraint"));
    assert!(emitted_sysml.contains("assert constraint SC_02_SafetyConstraint"));
    assert!(emitted_sysml.contains("assert constraint SC_03_SafetyConstraint"));
    assert_eq!(emitted_sysml.matches("SC_01_SafetyConstraint").count(), 1);
    assert_eq!(
        pkg.constraint_defs
            .iter()
            .filter(|c| c.name == "SC_01_SafetyConstraint")
            .count(),
        1
    );
    Ok(())
}
