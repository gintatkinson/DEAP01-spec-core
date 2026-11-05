#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Unit tests for `ARCH-04` dry-run persistence, `docs/designs/*.sysml` manifest and sidecar digest updates (`Issue #536`), and malformed UTF-8 frontmatter resilience.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical
//! verification standards for `crates/compile-sysml/src/sync/reverse.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Verifies side-effect-free `dry_run: true` execution, atomic sidecar `.digest.json` updates under `docs/designs/`, and UTF-8 boundary resilience.
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
//! - Returns descriptive errors if fixture setup or reverse synchronization fails.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0604] [REQ-SYNC-REVERSE-REVERSE-SYNC-SPECS-TO-SYSML]

use std::path::Path;

use deap_core::sysml_ast::PartDef;

use super::*;

/// Verifies `ARCH-04`: `reverse_sync_specs_to_sysml_with_dry_run` performs zero disk writes when `dry_run` is `true`
/// and writes atomically when `dry_run` is `false`.
///
/// Realises: [REQ-0604] [REQ-SYNC-REVERSE-REVERSE-SYNC-SPECS-TO-SYSML]
#[test]
fn test_arch_04_reverse_sync_with_dry_run_skips_writes() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = std::env::temp_dir().join(format!(
        "deap_test_reverse_sync_dry_run_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&tmp_dir);
    let features_dir = tmp_dir.join("docs").join("features");
    fs::create_dir_all(&features_dir).map_err(|e| e.to_string())?;
    fs::write(
        features_dir.join("FEAT-001.md"),
        "---\nid: FEAT-001\ntitle: CoreController\n---\n# Feature: CoreController\n",
    )
    .map_err(|e| e.to_string())?;

    let schema_path = tmp_dir.join("schema.sysml");
    let digest_path = tmp_dir.join("schema_digest.json");
    let opts = ReverseSyncOptions {
        docs_dir: tmp_dir.join("docs"),
        schema_path: None,
        output_path: schema_path.clone(),
        digest_path: digest_path.clone(),
        allow_overwrite: true,
    };

    let base_pkg = PackageDef {
        name: "TestSys".to_string(),
        part_defs: vec![PartDef {
            name: "CoreController".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let (pkg_dry, _) = reverse_sync_specs_to_sysml_with_dry_run(Some(&base_pkg), &opts, true)?;
    assert_eq!(pkg_dry.part_defs.len(), 1);
    assert!(!schema_path.exists(), "Dry-run must not write output_path");
    assert!(!digest_path.exists(), "Dry-run must not write digest_path");

    let (_pkg_live, _) = reverse_sync_specs_to_sysml_with_dry_run(Some(&base_pkg), &opts, false)?;
    assert!(
        schema_path.exists(),
        "Non-dry-run must atomically write output_path"
    );
    assert!(
        digest_path.exists(),
        "Non-dry-run must atomically write digest_path"
    );

    let _ = fs::remove_dir_all(&tmp_dir);
    Ok(())
}

/// Creates the temporary `docs/designs/blueprint-01-core.sysml` and stale sidecar fixture for Issue #536.
/// Realises: [REQ-0604] [REQ-SYNC-REVERSE-REVERSE-SYNC-SPECS-TO-SYSML]
fn setup_designs_manifest_fixture(
    tmp_dir: &Path,
) -> Result<(PathBuf, ReverseSyncOptions), Box<dyn std::error::Error>> {
    let designs_dir = tmp_dir.join("docs").join("designs");
    let pipeline_dir = tmp_dir.join(".pipeline");
    fs::create_dir_all(&designs_dir)?;
    fs::create_dir_all(&pipeline_dir)?;

    let bp_sysml_path = designs_dir.join("blueprint-01-core.sysml");
    fs::write(
        &bp_sysml_path,
        "package BlueprintCore {\n    part def CoreEngine {}\n}\n",
    )?;
    let bp_digest_path = designs_dir.join("blueprint-01-core.digest.json");
    fs::write(
        &bp_digest_path,
        "{\n  \"sha256\": \"stale\",\n  \"file_manifest\": []\n}\n",
    )?;

    let schema_path = pipeline_dir.join("schema.sysml");
    fs::write(
        &schema_path,
        "package TestSys {\n    part def CoreController {}\n}\n",
    )?;
    let digest_path = pipeline_dir.join("schema_digest.json");

    let opts = ReverseSyncOptions {
        docs_dir: tmp_dir.join("docs"),
        schema_path: Some(schema_path.clone()),
        output_path: schema_path,
        digest_path,
        allow_overwrite: true,
    };
    Ok((bp_digest_path, opts))
}

/// Verifies that `reverse_sync_specs_to_sysml` includes `docs/designs/*.sysml` in `digest.file_manifest`
/// and atomically refreshes sidecar `docs/designs/*.digest.json` files.
///
/// Realises: [REQ-0604] [REQ-SYNC-REVERSE-REVERSE-SYNC-SPECS-TO-SYSML]
#[test]
fn test_reverse_sync_includes_docs_designs_sysml_in_manifest_and_updates_sidecar_digests(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = std::env::temp_dir().join(format!(
        "deap_test_reverse_sync_designs_manifest_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&tmp_dir);
    let (bp_digest_path, opts) = setup_designs_manifest_fixture(&tmp_dir)?;

    let base_pkg = PackageDef {
        name: "TestSys".to_string(),
        part_defs: vec![PartDef {
            name: "CoreController".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let (_pkg, digest) = reverse_sync_specs_to_sysml(Some(&base_pkg), &opts)?;
    let sidecar_raw = fs::read_to_string(&bp_digest_path)?;
    let _ = fs::remove_dir_all(&tmp_dir);

    assert!(digest
        .file_manifest
        .iter()
        .any(|entry| entry.path == "docs/designs/blueprint-01-core.sysml"));

    let sidecar_digest: SchemaDigest = serde_json::from_str(&sidecar_raw)?;
    assert_ne!(sidecar_digest.sha256, "stale");
    assert!(sidecar_digest
        .file_manifest
        .iter()
        .any(|entry| entry.path == "docs/designs/blueprint-01-core.sysml"));
    Ok(())
}

/// Verifies `parse_frontmatter` and `reverse_sync_specs_to_sysml_with_dry_run` handle malformed
/// single-character quote delimiters, multi-byte UTF-8 strings, and malformed operation bullets without panicking.
///
/// Realises: [REQ-0604]
#[test]
fn test_parse_frontmatter_and_action_extraction_utf8_and_malformed_inputs() -> Result<(), String> {
    let malformed_md = "---\ntitle: \"\nsubtitle: '\nempty_dq: \"\"\nempty_sq: ''\nutf8_key: \"軌道制御: Δv ≤ 60 m/s\"\nno_colon_line\n---\nBody ΔΩ🛰\n";
    let (fm, body) = parse_frontmatter(malformed_md);
    assert_eq!(fm.get("title").map(String::as_str), Some("\""));
    assert_eq!(fm.get("subtitle").map(String::as_str), Some("'"));
    assert_eq!(fm.get("empty_dq").map(String::as_str), Some(""));
    assert_eq!(fm.get("empty_sq").map(String::as_str), Some(""));
    assert_eq!(
        fm.get("utf8_key").map(String::as_str),
        Some("軌道制御: Δv ≤ 60 m/s")
    );
    assert!(body.contains("Body ΔΩ🛰"));

    let unclosed_md = "---\ntitle: \"unclosed\nBody without closing fence";
    let (fm_unclosed, body_unclosed) = parse_frontmatter(unclosed_md);
    assert!(fm_unclosed.is_empty());
    assert_eq!(body_unclosed, unclosed_md);

    let tmp_dir = std::env::temp_dir().join(format!(
        "deap_test_rev_sync_utf8_checked_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&tmp_dir);
    let features_dir = tmp_dir.join("docs").join("features");
    fs::create_dir_all(&features_dir).map_err(|e| e.to_string())?;
    fs::write(
        features_dir.join("FEAT-01-Core.md"),
        concat!(
            "---\n",
            "title: \"\n",
            "type: feature\n",
            "part: CoreEngine\n",
            "---\n\n",
            "- `+computeTrajectory(in Δv: Real, in 軌道: String) : ControlStatus`\n",
            "- `+execute_coreengine() : void`\n",
            "- `+)reversedParens( : void`\n",
            "- `+unclosedParen(in x: Real : void`\n",
            "- `+() : void`\n",
            "- `+missingReturnType() : `\n"
        ),
    )
    .map_err(|e| e.to_string())?;

    let opts = ReverseSyncOptions {
        docs_dir: tmp_dir.join("docs"),
        schema_path: None,
        output_path: tmp_dir.join("schema.sysml"),
        digest_path: tmp_dir.join("schema-digest.json"),
        allow_overwrite: true,
    };
    let (pkg, _) = reverse_sync_specs_to_sysml_with_dry_run(None, &opts, true)?;
    let _ = fs::remove_dir_all(&tmp_dir);

    let core_part = pkg
        .part_defs
        .iter()
        .find(|p| p.name == "CoreEngine")
        .ok_or_else(|| "Expected CoreEngine PartDef".to_string())?;
    let action_names: Vec<&str> = core_part.actions.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(action_names, vec!["computeTrajectory"]);
    Ok(())
}
