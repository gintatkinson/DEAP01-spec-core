//! Regression and reproduction test suite for `compile-sysml` forward synchronization and AST verification.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
//! objectives for `crates/compile-sysml/tests/reproducer_tests.rs`. Provides deterministic
//! regression tests for adversarial audit findings in `compile-sysml`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Unallocated forward-synced specification files (`EPIC-XX-*.md`, `FEAT-XX-*.md`, `US-XX-*.md`,
//!   `UC-XX-*.md`) never synthesize hardcoded arithmetic fallback tracker issue IDs (`#10`, `#89`,
//!   `#95`, `#101`) that could collide with existing historical GitHub issues; unallocated
//!   specifications emit `TBD` until a real tracker issue is allocated, while existing allocated
//!   tracker issue IDs on disk are preserved across re-synchronization.
//!
//! ## 3. Preconditions
//! - `compile_sysml::sync::forward::forward_sync_sysml_to_specs` is accessible and
//!   `std::env::temp_dir()` is writable for ephemeral test fixtures.
//!
//! ## 4. Postconditions
//! - All temporary test fixtures are removed prior to test completion and all assertions hold.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ over the AST node count $N$ of synthetic SysML package fixtures.
//! - Space Complexity: $O(N)$ bounded auxiliary memory for generated Markdown specification buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Validates boundary inputs and file paths passed to `forward_sync_sysml_to_specs`.
//! - State: Ensures zero cross-test state contamination via process- and timestamp-scoped temp directories.
//! - Action: Executes deterministically with zero unbounded recursion, panics, or `.unwrap()` calls.
//! - Resource: Cleans up temporary filesystem directories deterministically before returning.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYNC-FORWARD-RESOLVE-ISSUE-ID] | SysML v2: VerificationCaseDef(reproducer_tests)

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use compile_sysml::sync::forward::{forward_sync_sysml_to_specs, ForwardSyncOptions};
use deap_core::sysml_ast::{ActionDef, PackageDef, PartDef, UseCaseDef};

/// Verifies that `forward_sync_sysml_to_specs` does not emit hardcoded colliding fallback issue IDs
/// (`#101`, `#10`, `#89`, `#95`) when generating new specification files in a fresh directory, and
/// preserves existing tracker-allocated issue IDs (e.g., `#560`) across subsequent re-syncs.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
/// for fn `test_issue_552_forward_sync_does_not_emit_hardcoded_colliding_issue_ids` in
/// `crates/compile-sysml/tests/reproducer_tests.rs` (refs #552).
///
/// ## 2. Invariants & Mathematical Properties
/// - Bijective Issue-ID Traceability Invariant: For any specification artifact $S$ with no prior
///   persisted tracker allocation on disk, $\text{IssueID}(S) = \text{TBD}$ rather than a synthetic
///   integer offset ($f + 9$, $s + 88$, $u + 94$, $e + 100$). For any specification artifact $S$
///   with an existing persisted tracker issue ID $\#k$ ($k > 0$) on disk, re-running forward sync
///   preserves $\text{IssueID}(S) = \#k$ in both its metadata table and parent checklists.
///
/// ## 3. Preconditions
/// - A synthetic `PackageDef` containing a subsystem `PartDef`, `ActionDef`, and `UseCaseDef` is
///   constructed and synchronized to an initially empty temporary output directory.
///
/// ## 4. Postconditions
/// - Freshly generated `EPIC-01-*.md`, `FEAT-01-*.md`, `US-01-*.md`, and `UC-01-*.md` files contain
///   `| **Issue ID** | TBD |` and checklist entries `- [ ] TBD - [...]`, with zero occurrences of
///   synthetic `#101`, `#10`, `#89`, or `#95`.
/// - After updating `FEAT-01-NavigationController.md` on disk to `| **Issue ID** | #560 |`, re-running
///   `forward_sync_sysml_to_specs` preserves `| **Issue ID** | #560 |` in the Feature file and emits
///   `- [ ] #560 - [Feature 01: ...]` in the parent Epic checklist.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ over the synthetic `PackageDef` AST and generated Markdown files.
/// - Space Complexity: $O(N)$ auxiliary memory for the generated file map.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Returns `Result<(), Box<dyn std::error::Error>>` on any I/O or sync failure.
/// - State: Uses a unique timestamped directory under `std::env::temp_dir()`.
/// - Action: Cleans up temporary files unconditionally before returning.
/// - Resource: Bounded to 5 small Markdown files in temporary storage.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYNC-FORWARD-RESOLVE-ISSUE-ID] | SysML v2: VerificationCaseDef(test_issue_552_forward_sync_does_not_emit_hardcoded_colliding_issue_ids)
#[test]
fn test_issue_552_forward_sync_does_not_emit_hardcoded_colliding_issue_ids(
) -> Result<(), Box<dyn std::error::Error>> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp_dir = std::env::temp_dir().join(format!(
        "deap_issue_552_forward_sync_{}_{}",
        std::process::id(),
        nanos
    ));
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir)?;
    }
    fs::create_dir_all(&tmp_dir)?;

    let mut pkg = PackageDef {
        name: "NavigationSubsystem".to_string(),
        ..Default::default()
    };
    pkg.part_defs.push(PartDef {
        name: "NavigationController".to_string(),
        ..Default::default()
    });
    pkg.action_defs.push(ActionDef {
        name: "ComputeRouteAction".to_string(),
        ..Default::default()
    });
    pkg.use_case_defs.push(UseCaseDef {
        name: "UC_Compute_Route".to_string(),
        ..Default::default()
    });

    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/navigation.sysml"),
        docs_dir: tmp_dir.clone(),
        out_dir: Some(tmp_dir.clone()),
        dry_run: false,
        force: true,
    };

    // Part 1: Fresh output directory with no prior .md files on disk
    let generated_fresh = forward_sync_sysml_to_specs(&pkg, &opts)?;

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

    // Assert zero hardcoded colliding fallback issue IDs in fresh specs
    assert!(
        !epic_md.contains("| **Issue ID** | #101 |"),
        "Fresh EPIC-01 must not emit hardcoded colliding fallback Issue ID #101:\n{}",
        epic_md
    );
    assert!(
        !feat_md.contains("| **Issue ID** | #10 |"),
        "Fresh FEAT-01 must not emit hardcoded colliding fallback Issue ID #10:\n{}",
        feat_md
    );
    assert!(
        !us_md.contains("| **Issue ID** | #89 |"),
        "Fresh US-01 must not emit hardcoded colliding fallback Issue ID #89:\n{}",
        us_md
    );
    assert!(
        !uc_md.contains("| **Issue ID** | #95 |"),
        "Fresh UC-01 must not emit hardcoded colliding fallback Issue ID #95:\n{}",
        uc_md
    );

    assert!(
        !epic_md.contains("- [ ] #10 -"),
        "Fresh EPIC-01 checklist must not reference synthetic #10:\n{}",
        epic_md
    );
    assert!(
        !epic_md.contains("- [ ] #89 -"),
        "Fresh EPIC-01 checklist must not reference synthetic #89:\n{}",
        epic_md
    );
    assert!(
        !epic_md.contains("- [ ] #95 -"),
        "Fresh EPIC-01 checklist must not reference synthetic #95:\n{}",
        epic_md
    );

    // Assert unallocated specs emit `| **Issue ID** | TBD |` and `- [ ] TBD - [...]`
    assert!(
        epic_md.contains("| **Issue ID** | TBD |"),
        "Fresh EPIC-01 must emit '| **Issue ID** | TBD |':\n{}",
        epic_md
    );
    assert!(
        feat_md.contains("| **Issue ID** | TBD |"),
        "Fresh FEAT-01 must emit '| **Issue ID** | TBD |':\n{}",
        feat_md
    );
    assert!(
        us_md.contains("| **Issue ID** | TBD |"),
        "Fresh US-01 must emit '| **Issue ID** | TBD |':\n{}",
        us_md
    );
    assert!(
        uc_md.contains("| **Issue ID** | TBD |"),
        "Fresh UC-01 must emit '| **Issue ID** | TBD |':\n{}",
        uc_md
    );
    assert!(
        epic_md.contains("- [ ] TBD - [Feature 01: [System Architecture] NavigationController](../features/FEAT-01-NavigationController.md)"),
        "Fresh EPIC-01 checklist must emit '- [ ] TBD - [Feature 01: ...]':\n{}",
        epic_md
    );

    // Part 2: Simulate tracker issue allocation (#560) on FEAT-01 on disk and re-run forward sync
    let feat_disk_path = tmp_dir.join("features/FEAT-01-NavigationController.md");
    let allocated_feat_md = feat_md.replace("| **Issue ID** | TBD |", "| **Issue ID** | #560 |");
    fs::write(&feat_disk_path, &allocated_feat_md)?;

    let generated_resync = forward_sync_sysml_to_specs(&pkg, &opts)?;
    let _ = fs::remove_dir_all(&tmp_dir);

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

/// Verifies that `resolve_schema_targets` and `SCHEMA_REMEDIATION_MESSAGE` honor `.pipeline/schema.sysml`
/// as the fallback Single Source of Truth (SSOT) when `schema/` is absent on disk, when `schema/`
/// contains only `.gitkeep`, and when `explicit_path` is `None`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
/// for fn `test_issue_546_resolve_schema_targets_falls_back_to_pipeline_schema` in
/// `crates/compile-sysml/tests/reproducer_tests.rs` (refs #546).
///
/// ## 2. Invariants & Mathematical Properties
/// - Fallback Resolution Invariant: When `<ws>/.pipeline/schema.sysml` exists on disk and `<ws>/schema`
///   either does not exist or contains zero `.sysml` files, `resolve_schema_targets(Some(<ws>/schema))`
///   and `resolve_schema_targets(None)` deterministically resolve to `SchemaTarget::SingleFile(.pipeline/schema.sysml)`.
/// - Remediation Diagnostic Invariant: `SCHEMA_REMEDIATION_MESSAGE` explicitly references both `schema/`
///   and `.pipeline/schema.sysml` and instructs operators to emit `--out .pipeline/schema.sysml`.
///
/// ## 3. Preconditions
/// - `std::env::temp_dir()` is writable for creating an isolated temporary workspace directory `temp_ws`.
///
/// ## 4. Postconditions
/// - All fallback resolution assertions and remediation message assertions hold, the original working
///   directory is restored, and `temp_ws` is removed before returning.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ over the bounded temporary directory entries.
/// - Space Complexity: $O(1)$ auxiliary memory.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates `Some(<ws>/schema)` (both absent and `.gitkeep`-only) and `None` inputs to `resolve_schema_targets`.
/// - State: Restores process working directory unconditionally before asserting `res_none`.
/// - Action: Executes deterministically with zero panics on valid I/O state.
/// - Resource: Cleans up `temp_ws` unconditionally prior to function exit.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-RESOLVE-TARGETS] | SysML v2: VerificationCaseDef(test_issue_546_resolve_schema_targets_falls_back_to_pipeline_schema)
#[test]
fn test_issue_546_resolve_schema_targets_falls_back_to_pipeline_schema(
) -> Result<(), Box<dyn std::error::Error>> {
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

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_ws = std::env::temp_dir().join(format!(
        "deap_issue_546_resolve_schema_{}_{}",
        std::process::id(),
        nanos
    ));
    if temp_ws.exists() {
        fs::remove_dir_all(&temp_ws)?;
    }
    let pipeline_dir = temp_ws.join(".pipeline");
    fs::create_dir_all(&pipeline_dir)?;
    let pipeline_schema = pipeline_dir.join("schema.sysml");
    fs::write(&pipeline_schema, "package Issue546FallbackPkg {}\n")?;

    // Case 1: temp_ws/schema does NOT exist on disk, but temp_ws/.pipeline/schema.sysml exists
    let res_missing_schema_dir = compile_sysml::resolve_schema_targets(Some(temp_ws.join("schema")));
    assert_eq!(
        res_missing_schema_dir,
        Ok(compile_sysml::SchemaTarget::SingleFile(
            pipeline_schema.clone()
        )),
        "resolve_schema_targets(Some(<ws>/schema)) must fall back to <ws>/.pipeline/schema.sysml even when <ws>/schema directory does not exist"
    );

    // Case 2: temp_ws/schema exists on disk with only .gitkeep inside it
    let schema_dir = temp_ws.join("schema");
    fs::create_dir_all(&schema_dir)?;
    fs::write(schema_dir.join(".gitkeep"), "")?;
    let res_empty_schema_dir = compile_sysml::resolve_schema_targets(Some(schema_dir));
    assert_eq!(
        res_empty_schema_dir,
        Ok(compile_sysml::SchemaTarget::SingleFile(
            pipeline_schema.clone()
        )),
        "resolve_schema_targets(Some(<ws>/schema)) must fall back to <ws>/.pipeline/schema.sysml when <ws>/schema has only .gitkeep"
    );

    // Case 3: resolve_schema_targets(None) from inside temp_ws
    let orig_cwd = std::env::current_dir()?;
    std::env::set_current_dir(&temp_ws)?;
    let res_none = compile_sysml::resolve_schema_targets(None);
    std::env::set_current_dir(&orig_cwd)?;
    let _ = fs::remove_dir_all(&temp_ws);

    assert_eq!(
        res_none,
        Ok(compile_sysml::SchemaTarget::SingleFile(PathBuf::from(
            ".pipeline/schema.sysml"
        ))),
        "resolve_schema_targets(None) must fall back to .pipeline/schema.sysml"
    );

    Ok(())
}

/// Verifies that `run_compilation_gate` compiles each `blueprint-*.sysml` file inside a
/// `docs/designs` blueprint triplet directory in place and emits per-file `blueprint-*.digest.json`
/// sidecars when invoked with the default compilation paths (`.pipeline/schema.sysml` and
/// `.pipeline/schema-digest.json`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
/// for fn `test_issue_547_run_compilation_gate_compiles_blueprint_triplet_dir_in_place` in
/// `crates/compile-sysml/tests/reproducer_tests.rs` (refs #547).
///
/// ## 2. Invariants & Mathematical Properties
/// - Per-Blueprint Triplet Isolation Invariant: When compiling a directory $D$ of `blueprint-*.sysml`
///   files with default output/digest paths, each $f_i \in D$ is validated, canonicalized in place
///   at $f_i$, and paired with its cryptographic SHA-256 digest at $f_i\text{.with\_extension("digest.json")}$
///   without cross-merging sibling blueprint packages into $f_i$.
///
/// ## 3. Preconditions
/// - `std::env::temp_dir()` is writable for creating an ephemeral `docs/designs` fixture containing
///   `blueprint-01-alpha.sysml` (`AlphaPkg`) and `blueprint-02-beta.sysml` (`BetaPkg`).
///
/// ## 4. Postconditions
/// - `run_compilation_gate` returns exit code `0`.
/// - Both `blueprint-01-alpha.digest.json` and `blueprint-02-beta.digest.json` exist on disk.
/// - `blueprint-01-alpha.sysml` preserves `package AlphaPkg` without `BetaActuator`, and
///   `blueprint-02-beta.sysml` preserves `package BetaPkg` without `AlphaController`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N)$ over the two synthetic blueprint AST fixtures.
/// - Space Complexity: $O(N)$ auxiliary memory for serialized SysML and digest strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates directory-target invocation of `run_compilation_gate` with default output paths.
/// - State: Uses a unique process- and timestamp-scoped directory under `std::env::temp_dir()`.
/// - Action: Executes deterministically with zero panics or `.unwrap()` calls.
/// - Resource: Removes the temporary directory prior to assertions so failed assertions leave zero debris.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-BLUEPRINT-TRIPLET-DIR] | SysML v2: VerificationCaseDef(test_issue_547_run_compilation_gate_compiles_blueprint_triplet_dir_in_place)
#[test]
fn test_issue_547_run_compilation_gate_compiles_blueprint_triplet_dir_in_place(
) -> Result<(), Box<dyn std::error::Error>> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let temp_ws = std::env::temp_dir().join(format!(
        "deap_issue_547_blueprint_triplet_{}_{}",
        std::process::id(),
        nanos
    ));
    let _ = fs::remove_dir_all(&temp_ws);
    let designs_dir = temp_ws.join("docs").join("designs");
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
    let _ = fs::remove_dir_all(&temp_ws);

    assert_eq!(
        code, 0,
        "run_compilation_gate on docs/designs blueprint triplet directory must return 0"
    );
    assert!(
        has_alpha_digest,
        "blueprint-01-alpha.digest.json sidecar must be created in docs/designs"
    );
    assert!(
        has_beta_digest,
        "blueprint-02-beta.digest.json sidecar must be created in docs/designs"
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
