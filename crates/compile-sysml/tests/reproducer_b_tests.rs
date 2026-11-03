//! Reproducer test helpers for GitHub Issues #552, #546, and #547 (`compile-sysml`).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
//! objectives for `crates/compile-sysml/tests/reproducer_b_tests.rs`. Exercises Issue ID fallback
//! resolution (#552), `.pipeline/schema.sysml` target fallback (#546), and in-place blueprint
//! triplet compilation (#547).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Unallocated forward-synced specification files emit `TBD` rather than synthetic colliding
//!   issue IDs (`#101`, `#10`, `#89`, `#95`), while allocated issue IDs (`#560`) survive re-sync.
//! - `resolve_schema_targets` deterministically resolves `.pipeline/schema.sysml` when `schema/`
//!   is missing or contains only `.gitkeep`.
//! - `run_compilation_gate` compiles `blueprint-*.sysml` files in place with isolated `.digest.json`
//!   sidecars.
//!
//! ## 3. Preconditions
//! - `std::env::temp_dir()` is writable for isolated RAII `TempDirGuard` test fixtures.
//!
//! ## 4. Postconditions
//! - All temporary test fixtures are cleaned up deterministically and all assertions hold.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N \log N)$ bounded over synthetic AST fixtures.
//! - Space Complexity: $O(N)$ auxiliary memory for Markdown and SysML buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Validates boundary inputs to `forward_sync_sysml_to_specs`, `resolve_schema_targets`,
//!   and `run_compilation_gate`.
//! - State: Isolated per-test temporary workspaces via `TempDirGuard`.
//! - Action: Executes deterministically with zero panics or `.unwrap()` calls.
//! - Resource: Reclaims all ephemeral filesystem trees on scope exit.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYNC-FORWARD-RESOLVE-ISSUE-ID] | SysML v2: VerificationCaseDef(reproducer_b_tests)

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use compile_sysml::sync::forward::{forward_sync_sysml_to_specs, ForwardSyncOptions};
use deap_core::sysml_ast::{ActionDef, PackageDef, PartDef, UseCaseDef};

use crate::reproducer_tests::TempDirGuard;

fn assert_issue_552_fresh_specs(
    generated_fresh: &BTreeMap<String, String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let epic_md = generated_fresh
        .get("epics/EPIC-01-NavigationSubsystem.md")
        .ok_or("Missing EPIC-01-NavigationSubsystem.md")?;
    let feat_md = generated_fresh
        .get("features/FEAT-01-NavigationController.md")
        .ok_or("Missing FEAT-01-NavigationController.md")?;
    let us_md = generated_fresh
        .get("user-stories/US-01-ComputeRouteAction.md")
        .ok_or("Missing US-01-ComputeRouteAction.md")?;
    let uc_md = generated_fresh
        .get("use-cases/UC-01-UC_Compute_Route.md")
        .ok_or("Missing UC-01-UC_Compute_Route.md")?;

    for (doc_md, forbidden_id) in [
        (epic_md, "| **Issue ID** | #101 |"),
        (feat_md, "| **Issue ID** | #10 |"),
        (us_md, "| **Issue ID** | #89 |"),
        (uc_md, "| **Issue ID** | #95 |"),
    ] {
        assert!(
            !doc_md.contains(forbidden_id),
            "Fresh spec must not emit hardcoded colliding fallback {}:\n{}",
            forbidden_id,
            doc_md
        );
        assert!(
            doc_md.contains("| **Issue ID** | TBD |"),
            "Fresh spec must emit '| **Issue ID** | TBD |':\n{}",
            doc_md
        );
    }

    for forbidden_ref in ["- [ ] #10 -", "- [ ] #89 -", "- [ ] #95 -"] {
        assert!(
            !epic_md.contains(forbidden_ref),
            "Fresh EPIC-01 checklist must not reference synthetic {}:\n{}",
            forbidden_ref,
            epic_md
        );
    }
    assert!(
        epic_md.contains("- [ ] TBD - [Feature 01: [System Architecture] NavigationController](../features/FEAT-01-NavigationController.md)"),
        "Fresh EPIC-01 checklist must emit '- [ ] TBD - [Feature 01: ...]':\n{}",
        epic_md
    );
    Ok(feat_md.clone())
}

/// Executes Issue #552 regression checks for unallocated `TBD` and preserved `#560` tracker IDs.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when all Issue #552 invariants hold.
///
/// # Complexity
/// * Time: $O(N)$ over the synthetic `NavigationSubsystem` AST. Space: $O(N)$.
///
/// # Failure Modes
/// * Propagates any I/O or assertion failure while cleaning up via `TempDirGuard`.
pub fn run_issue_552_checks() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDirGuard::new("deap_issue_552_forward_sync")?;
    let pkg = PackageDef {
        name: "NavigationSubsystem".to_string(),
        part_defs: vec![PartDef {
            name: "NavigationController".to_string(),
            ..Default::default()
        }],
        action_defs: vec![ActionDef {
            name: "ComputeRouteAction".to_string(),
            ..Default::default()
        }],
        use_case_defs: vec![UseCaseDef {
            name: "UC_Compute_Route".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/navigation.sysml"),
        docs_dir: temp.path().to_path_buf(),
        out_dir: Some(temp.path().to_path_buf()),
        dry_run: false,
        force: true,
    };

    let generated_fresh = forward_sync_sysml_to_specs(&pkg, &opts)?;
    let feat_md = assert_issue_552_fresh_specs(&generated_fresh)?;

    let feat_disk_path = temp.path().join("features/FEAT-01-NavigationController.md");
    let allocated_feat_md = feat_md.replace("| **Issue ID** | TBD |", "| **Issue ID** | #560 |");
    fs::write(&feat_disk_path, &allocated_feat_md)?;

    let generated_resync = forward_sync_sysml_to_specs(&pkg, &opts)?;
    let resynced_feat_md = generated_resync
        .get("features/FEAT-01-NavigationController.md")
        .ok_or("Missing resynced FEAT-01-NavigationController.md")?;
    let resynced_epic_md = generated_resync
        .get("epics/EPIC-01-NavigationSubsystem.md")
        .ok_or("Missing resynced EPIC-01-NavigationSubsystem.md")?;

    assert!(
        resynced_feat_md.contains("| **Issue ID** | #560 |"),
        "Re-synced FEAT-01 must preserve real tracker-allocated Issue ID #560:\n{}",
        resynced_feat_md
    );
    assert!(
        resynced_epic_md.contains("- [ ] #560 - [Feature 01: [System Architecture] NavigationController](../features/FEAT-01-NavigationController.md)"),
        "Re-synced EPIC-01 checklist must preserve real tracker-allocated Issue ID #560:\n{}",
        resynced_epic_md
    );
    Ok(())
}

/// Executes Issue #546 regression checks for `.pipeline/schema.sysml` fallback target resolution.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when all Issue #546 assertions pass.
///
/// # Complexity
/// * Time: $O(1)$ directory and file lookups. Space: $O(1)$.
///
/// # Failure Modes
/// * Restores process working directory prior to asserting and cleans up via `TempDirGuard`.
pub fn run_issue_546_checks() -> Result<(), Box<dyn std::error::Error>> {
    assert!(
        compile_sysml::SCHEMA_REMEDIATION_MESSAGE
            .contains("Error: No .sysml schema file found in schema/ or .pipeline/schema.sysml."),
        "SCHEMA_REMEDIATION_MESSAGE must mention both schema/ and .pipeline/schema.sysml:\n{}",
        compile_sysml::SCHEMA_REMEDIATION_MESSAGE
    );
    assert!(
        compile_sysml::SCHEMA_REMEDIATION_MESSAGE.contains("--out .pipeline/schema.sysml"),
        "SCHEMA_REMEDIATION_MESSAGE must reference --out .pipeline/schema.sysml:\n{}",
        compile_sysml::SCHEMA_REMEDIATION_MESSAGE
    );

    let temp_ws = TempDirGuard::new("deap_issue_546_resolve_schema")?;
    let pipeline_dir = temp_ws.path().join(".pipeline");
    fs::create_dir_all(&pipeline_dir)?;
    let pipeline_schema = pipeline_dir.join("schema.sysml");
    fs::write(&pipeline_schema, "package Issue546FallbackPkg {}\n")?;

    let res_missing = compile_sysml::resolve_schema_targets(Some(temp_ws.path().join("schema")));
    assert_eq!(
        res_missing,
        Ok(compile_sysml::SchemaTarget::SingleFile(
            pipeline_schema.clone()
        )),
        "resolve_schema_targets(Some(<ws>/schema)) must fall back to <ws>/.pipeline/schema.sysml when <ws>/schema is absent"
    );

    let schema_dir = temp_ws.path().join("schema");
    fs::create_dir_all(&schema_dir)?;
    fs::write(schema_dir.join(".gitkeep"), "")?;
    let res_empty = compile_sysml::resolve_schema_targets(Some(schema_dir));
    assert_eq!(
        res_empty,
        Ok(compile_sysml::SchemaTarget::SingleFile(
            pipeline_schema.clone()
        )),
        "resolve_schema_targets(Some(<ws>/schema)) must fall back to <ws>/.pipeline/schema.sysml when <ws>/schema has only .gitkeep"
    );

    let orig_cwd = std::env::current_dir()?;
    std::env::set_current_dir(temp_ws.path())?;
    let res_none = compile_sysml::resolve_schema_targets(None);
    std::env::set_current_dir(&orig_cwd)?;

    assert_eq!(
        res_none,
        Ok(compile_sysml::SchemaTarget::SingleFile(PathBuf::from(
            ".pipeline/schema.sysml"
        ))),
        "resolve_schema_targets(None) must fall back to .pipeline/schema.sysml"
    );
    Ok(())
}

/// Executes Issue #547 regression checks for in-place `docs/designs` blueprint triplet compilation.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when per-file blueprint compilation succeeds.
///
/// # Complexity
/// * Time: $O(N \log N)$ over the two blueprint files. Space: $O(N)$.
///
/// # Failure Modes
/// * Propagates any I/O failure and cleans up via `TempDirGuard`.
pub fn run_issue_547_checks() -> Result<(), Box<dyn std::error::Error>> {
    let temp_ws = TempDirGuard::new("deap_issue_547_blueprint_triplet")?;
    let designs_dir = temp_ws.path().join("docs").join("designs");
    fs::create_dir_all(&designs_dir)?;

    let alpha_path = designs_dir.join("blueprint-01-alpha.sysml");
    let beta_path = designs_dir.join("blueprint-02-beta.sysml");
    fs::write(
        &alpha_path,
        "package AlphaPkg {\n    part def AlphaController {\n        attribute id : Integer;\n    }\n}\n",
    )?;
    fs::write(
        &beta_path,
        "package BetaPkg {\n    part def BetaActuator {\n        attribute speed : Real;\n    }\n}\n",
    )?;

    let code = compile_sysml::run_compilation_gate(
        Some(designs_dir.clone()),
        Path::new(".pipeline/schema.sysml"),
        Path::new(".pipeline/schema-digest.json"),
    );

    let has_alpha_digest = designs_dir.join("blueprint-01-alpha.digest.json").is_file();
    let has_beta_digest = designs_dir.join("blueprint-02-beta.digest.json").is_file();
    let alpha_content = fs::read_to_string(&alpha_path)?;
    let beta_content = fs::read_to_string(&beta_path)?;

    assert_eq!(
        code, 0,
        "run_compilation_gate on docs/designs blueprint triplet directory must return 0"
    );
    assert!(
        has_alpha_digest && has_beta_digest,
        "Both blueprint-01-alpha.digest.json and blueprint-02-beta.digest.json sidecars must be created"
    );
    assert!(
        alpha_content.contains("package AlphaPkg") && !alpha_content.contains("BetaActuator"),
        "blueprint-01-alpha.sysml must remain isolated to AlphaPkg:\n{}",
        alpha_content
    );
    assert!(
        beta_content.contains("package BetaPkg") && !beta_content.contains("AlphaController"),
        "blueprint-02-beta.sysml must remain isolated to BetaPkg:\n{}",
        beta_content
    );
    Ok(())
}
