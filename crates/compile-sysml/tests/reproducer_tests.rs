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
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use compile_sysml::sync::forward::{forward_sync_sysml_to_specs, ForwardSyncOptions};
use compile_sysml::FmecaScoringConfig;
use deap_core::sysml_ast::{
    ActionDef, AttributeDef, CalcDef, ConstraintDef, InteractionDef, OperationDef, PackageDef,
    PartDef, PortDef, RequirementDef, StateDef, TransitionDef, UseCaseDef,
};

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

/// Monotonic sequence counter ensuring unique RAII temporary directory paths across concurrent test threads.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C test isolation
/// objectives for `TEMP_DIR_COUNTER` in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #693).
///
/// ## 2. Invariants & Mathematical Properties
/// - Monotonicity Invariant: Every `fetch_add(1, Ordering::Relaxed)` call yields a strictly distinct
///   `u64` identifier within the active test process.
///
/// ## 3. Preconditions
/// - Initialized statically to `0` prior to test harness execution.
///
/// ## 4. Postconditions
/// - Prevents directory name collisions even when concurrent threads sample identical wall-clock timestamps.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ atomic increment.
/// - Space Complexity: $O(1)$ static 64-bit word.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Accessed exclusively by `TempDirGuard::new`.
/// - State: Lock-free atomic state with zero contention deadlock risk.
/// - Action: Executes without panicking.
/// - Resource: Zero heap allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FMECA-TEMP-ISOLATION] | SysML v2: VerificationCaseDef(TEMP_DIR_COUNTER)
static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

/// RAII temporary directory guard that deterministically removes ephemeral filesystem fixtures on drop.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification hygiene
/// objectives for `TempDirGuard` in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #693).
///
/// ## 2. Invariants & Mathematical Properties
/// - Deterministic Cleanup Invariant: Regardless of whether a test exits normally (`Ok(())`) or
///   unwinds on an `assert!` failure (RED phase), `Drop::drop` removes `self.path` from `std::env::temp_dir()`.
///
/// ## 3. Preconditions
/// - `std::env::temp_dir()` is writable by the current test runner process.
///
/// ## 4. Postconditions
/// - `self.path` exists on disk throughout the lifetime of `TempDirGuard` and is removed when dropped.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(K)$ on drop, where $K$ is the bounded file count inside the temporary tree.
/// - Space Complexity: $O(1)$ heap storage for `PathBuf`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Exposes an immutable `&Path` borrow via `TempDirGuard::path`.
/// - State: Process-, nanosecond-, and atomic-sequence-scoped directory path prevents cross-test races.
/// - Action: Never panics during stack unwinding in `Drop::drop`.
/// - Resource: Reclaims all ephemeral directory entries automatically on scope exit.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FMECA-TEMP-ISOLATION] | SysML v2: VerificationCaseDef(TempDirGuard)
struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    /// Allocates and creates a fresh process- and sequence-unique temporary directory under `std::env::temp_dir()`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
    /// objectives for `TempDirGuard::new` in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #693).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Freshness Invariant: Upon `Ok(guard)`, `guard.path()` is a newly created empty directory.
    ///
    /// ## 3. Preconditions
    /// - `prefix` is a valid non-empty ASCII path component prefix.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(TempDirGuard)` with `self.path.is_dir() == true`, or propagates an `io::Error`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$ filesystem directory creation.
    /// - Space Complexity: $O(1)$ path buffer allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Propagates directory creation errors via `Result`.
    /// - State: Uses `TEMP_DIR_COUNTER`, PID, and epoch nanoseconds for collision-free naming.
    /// - Action: Zero `.unwrap()`, `.expect()`, or silent error discarding.
    /// - Resource: Pairs allocation immediately with RAII `TempDirGuard` ownership.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FMECA-TEMP-ISOLATION] | SysML v2: VerificationCaseDef(TempDirGuard_new)
    fn new(prefix: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let seq = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "{}_{}_{}_{}",
            prefix,
            std::process::id(),
            nanos,
            seq
        ));
        if path.exists() {
            fs::remove_dir_all(&path)?;
        }
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    /// Borrows the underlying temporary directory path managed by this RAII guard.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
    /// objectives for `TempDirGuard::path` in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #693).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Referential Stability Invariant: Returns the exact directory path created during `TempDirGuard::new`.
    ///
    /// ## 3. Preconditions
    /// - `self` is a live `TempDirGuard` instance.
    ///
    /// ## 4. Postconditions
    /// - Returns an immutable `&Path` slice without mutating filesystem state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$.
    /// - Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible reference borrow.
    /// - State: Read-only borrow.
    /// - Action: Zero allocations or side effects.
    /// - Resource: Tied to the lifetime of `&self`.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FMECA-TEMP-ISOLATION] | SysML v2: VerificationCaseDef(TempDirGuard_path)
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    /// Deterministically removes the temporary directory tree when `TempDirGuard` leaves scope.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C resource cleanup
    /// objectives for `TempDirGuard::drop` in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #693).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Unwind-Safe Teardown Invariant: Executes `fs::remove_dir_all(&self.path)` without panicking even
    ///   when invoked during assertion-failure stack unwinding.
    ///
    /// ## 3. Preconditions
    /// - `self.path` points to the process-scoped temporary directory owned by this guard.
    ///
    /// ## 4. Postconditions
    /// - `self.path` is removed from `std::env::temp_dir()`, or any OS teardown error is logged to stderr.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(K)$ over the $K$ ephemeral entries created inside `self.path`.
    /// - Space Complexity: $O(1)$ auxiliary stack space.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Standard `Drop::drop(&mut self)` contract.
    /// - State: Idempotent check via `self.path.exists()`.
    /// - Action: Logs teardown errors explicitly via `eprintln!` rather than panicking.
    /// - Resource: Guarantees zero orphaned temporary fixture directories after test runs.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FMECA-TEMP-ISOLATION] | SysML v2: VerificationCaseDef(TempDirGuard_drop)
    fn drop(&mut self) {
        if self.path.exists() {
            if let Err(err) = fs::remove_dir_all(&self.path) {
                eprintln!(
                    "TempDirGuard warning: failed to remove {}: {}",
                    self.path.display(),
                    err
                );
            }
        }
    }
}

/// Verifies that `generate_fmeca_matrix` over a pure software SysML v2 AST (`QueryEnginePkg` /
/// `QueryPlanner`) emits zero hardcoded physical avionics templates, zero unresearched `"SSOT"`
/// basis rows, full `[0..]` member enumeration across all four failure dimensions, and accurate
/// `(8, 3, 2)` (`RPN == 48`) default scores when `FmecaScoringConfig::default()` has empty scales.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
/// for fn `test_issue_693_fmeca_software_ast_eliminates_hardware_templates_fake_ssot_first_truncation_and_score_fallback`
/// in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #693).
///
/// ## 2. Invariants & Mathematical Properties
/// - Domain-Neutrality Invariant: Pure software AST declarations never synthesize physical hardware
///   or avionics prose (`"serial bus"`, `"CRC-32"`, `"Actuation Loop"`, `"deflection"`,
///   `"Thermal Headroom"`, `"Brownout"`, `"thermal sink"`, `"low-voltage cutoff"`, `"watchdog"`).
/// - Provenance Honesty Invariant: Every AST-derived row without an ingested `FAILURE_MODE_REGISTRY.md`
///   record carries `basis == "Derived"` and never falsifies `basis == "SSOT"`.
/// - Exhaustive `[0..]` Coverage Invariant: All declared ports (`sqlIngress`, `planEgress`), states
///   (`Idle`, `Optimizing`), actions/operations (`parseAst`, `emitPlan`, `validateBindings`), and
///   attributes (`maxRetries`, `cacheLimitMb`) are enumerated without `.first()` truncation, producing
///   at least 15 rows spanning `Interface`, `State`, `Action`, and `Resource`.
/// - Score Fallback Invariant: Passing `Some(&FmecaScoringConfig::default())` with empty override maps
///   preserves distinct `(default_s, default_o, default_d) = (8, 3, 2)` (`rpn == 48`) for `Interface`
///   rows rather than collapsing occurrence and detection to severity `(8, 8, 8)` (`rpn == 512`).
///
/// ## 3. Preconditions
/// - A pure software `PackageDef` (`QueryEnginePkg`) containing `PartDef` `QueryPlanner` with 2 ports,
///   2 states, 2 actions, 1 operation, and 2 attributes is constructed in memory.
///
/// ## 4. Postconditions
/// - All domain-neutrality, basis-provenance, exhaustive-enumeration, and score-resolution assertions hold.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(M)$ where $M$ is the total AST member count across `QueryPlanner`.
/// - Space Complexity: $O(M)$ for the synthesized `Vec<FmecaRow>`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Invokes `compile_sysml::stpa::fmeca::generate_fmeca_matrix` directly with `Some(&FmecaScoringConfig::default())`.
/// - State: Pure in-memory deterministic evaluation with zero global state or filesystem dependencies.
/// - Action: Executes with zero `.unwrap()`, `.expect()`, or range indexing panics.
/// - Resource: Bounded to a single synthetic `PackageDef` fixture in memory.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-FMECA-DOMAIN-NEUTRAL], [REQ-STPA-FMECA-FULL-ENUMERATION], [REQ-STPA-FMECA-SCORE-FALLBACK] | SysML v2: VerificationCaseDef(test_issue_693_fmeca_software_ast_eliminates_hardware_templates_fake_ssot_first_truncation_and_score_fallback)
#[test]
fn test_issue_693_fmeca_software_ast_eliminates_hardware_templates_fake_ssot_first_truncation_and_score_fallback(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut planner = PartDef {
        name: "QueryPlanner".to_string(),
        ..Default::default()
    };
    planner.ports.push(PortDef {
        name: "sqlIngress".to_string(),
        ..Default::default()
    });
    planner.ports.push(PortDef {
        name: "planEgress".to_string(),
        ..Default::default()
    });
    planner.states.push(StateDef {
        name: "Idle".to_string(),
        ..Default::default()
    });
    planner.states.push(StateDef {
        name: "Optimizing".to_string(),
        ..Default::default()
    });
    planner.actions.push(ActionDef {
        name: "parseAst".to_string(),
        ..Default::default()
    });
    planner.actions.push(ActionDef {
        name: "emitPlan".to_string(),
        ..Default::default()
    });
    planner.operations.push(OperationDef {
        name: "validateBindings".to_string(),
        ..Default::default()
    });
    planner.attributes.push(AttributeDef {
        name: "maxRetries".to_string(),
        ..Default::default()
    });
    planner.attributes.push(AttributeDef {
        name: "cacheLimitMb".to_string(),
        ..Default::default()
    });

    let mut pkg = PackageDef {
        name: "QueryEnginePkg".to_string(),
        ..Default::default()
    };
    pkg.part_defs.push(planner);

    let scoring_cfg = FmecaScoringConfig::default();
    let rows = compile_sysml::stpa::fmeca::generate_fmeca_matrix(&pkg, Some(&scoring_cfg));

    assert!(
        !rows.is_empty(),
        "generate_fmeca_matrix must produce non-empty FMECA rows for QueryPlanner"
    );

    // 1. Zero physical avionics / hardware substrings in pure software AST rows
    let forbidden_avionics_substrings = [
        "serial bus",
        "CRC-32",
        "Actuation Loop",
        "deflection",
        "Thermal Headroom",
        "Brownout",
        "thermal sink",
        "low-voltage cutoff",
        "watchdog",
    ];
    for row in &rows {
        for needle in forbidden_avionics_substrings {
            assert!(
                !row.failure_mode.contains(needle)
                    && !row.effect.contains(needle)
                    && !row.mitigation.contains(needle),
                "Pure software FMECA row must not contain hardcoded physical avionics substring {:?}: {:?}",
                needle,
                row
            );
        }
    }

    // 2. Zero falsified "SSOT" basis when no FAILURE_MODE_REGISTRY.md entry exists
    assert!(
        rows.iter().all(|r| r.basis == "Derived"),
        "All AST-derived FMECA rows without researched registry entries must carry basis == \"Derived\", got: {:?}",
        rows
    );

    // 3. Full [0..] member enumeration across all 4 dimensions (no .first() truncation)
    assert!(
        rows.len() >= 15,
        "Full [0..] enumeration across 9 AST members must yield at least 15 FMECA rows, got {}: {:?}",
        rows.len(),
        rows
    );
    for expected_dim in ["Interface", "State", "Action", "Resource"] {
        assert!(
            rows.iter().any(|r| r.dimension == expected_dim),
            "FMECA rows must cover dimension {:?}, got: {:?}",
            expected_dim,
            rows
        );
    }
    for expected_member in [
        "sqlIngress",
        "planEgress",
        "Idle",
        "Optimizing",
        "parseAst",
        "emitPlan",
        "validateBindings",
        "maxRetries",
        "cacheLimitMb",
    ] {
        assert!(
            rows.iter().any(|r| {
                r.failure_mode.contains(expected_member)
                    || r.effect.contains(expected_member)
                    || r.mitigation.contains(expected_member)
            }),
            "FMECA rows must enumerate AST member {:?} without .first() truncation, got: {:?}",
            expected_member,
            rows
        );
    }

    // 4. Empty-scale FmecaScoringConfig::default() resolves Interface row scores to (8, 3, 2) (rpn == 48), NOT (8, 8, 8) (rpn == 512)
    let interface_rows: Vec<_> = rows.iter().filter(|r| r.dimension == "Interface").collect();
    assert!(
        !interface_rows.is_empty(),
        "Expected non-empty Interface FMECA rows for sqlIngress and planEgress"
    );
    for row in &interface_rows {
        assert_ne!(
            (row.severity, row.occurrence, row.detection),
            (8, 8, 8),
            "Empty-scale FmecaScoringConfig::default() must not collapse Interface scores to (8, 8, 8) / RPN 512: {:?}",
            row
        );
        assert_ne!(
            row.rpn, 512,
            "Interface row RPN must not be inflated to 512 by resolve_scores bug: {:?}",
            row
        );
    }
    assert!(
        interface_rows
            .iter()
            .any(|r| (r.severity, r.occurrence, r.detection) == (8, 3, 2) && r.rpn == 48),
        "Empty-scale FmecaScoringConfig::default() must resolve Interface row scores to (8, 3, 2) with rpn == 48, got: {:?}",
        interface_rows
    );

    Ok(())
}

/// Verifies that `compile-sysml --schema .pipeline/upstream --stpa-transpile` scopes its default
/// safety output directory to `<subworkspace>/docs/safety` (without polluting `<root>/docs/safety`)
/// and ingests `<subworkspace>/docs/research/FAILURE_MODE_REGISTRY.md` rows with `SSOT` provenance
/// while marking unresearched AST-synthesized rows as `Derived`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
/// for fn `test_issue_693_stpa_transpile_scopes_subworkspace_out_dir_and_ingests_failure_mode_registry`
/// in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #693).
///
/// ## 2. Invariants & Mathematical Properties
/// - Subworkspace Output Scoping Invariant: Invoking `compile-sysml --schema .pipeline/upstream --stpa-transpile`
///   without an explicit `--out` flag resolves `docs/safety` relative to the `.pipeline/upstream`
///   subworkspace root, emitting `.pipeline/upstream/docs/safety/05_FMECA_MATRIX.md` and
///   `.pipeline/upstream/docs/safety/STPA_MATRIX.md` while leaving `<root>/docs/safety` untouched.
/// - Research Registry Ingestion Invariant: Researched failure modes recorded in
///   `<subworkspace>/docs/research/FAILURE_MODE_REGISTRY.md` are emitted into `05_FMECA_MATRIX.md`
///   with exact `(S, O, D, RPN)` (`7, 2, 2, 28`) and `| SSOT |` provenance, whereas AST-derived
///   rows lacking registry entries emit `| Derived |`.
///
/// ## 3. Preconditions
/// - An isolated RAII `TempDirGuard` workspace contains `.pipeline/upstream/.pipeline/schema.sysml`
///   and `.pipeline/upstream/docs/research/FAILURE_MODE_REGISTRY.md`, and `CARGO_BIN_EXE_compile-sysml`
///   is compiled by Cargo.
///
/// ## 4. Postconditions
/// - `<temp>/docs/safety/05_FMECA_MATRIX.md` does not exist.
/// - `<temp>/.pipeline/upstream/docs/safety/05_FMECA_MATRIX.md` and `STPA_MATRIX.md` exist.
/// - `05_FMECA_MATRIX.md` contains `"Planner Memoization Arena Exhaustion"` with `| 7 | 2 | 2 | 28 |`
///   and `| SSOT |`, while unresearched rows carry `| Derived |`.
/// - `TempDirGuard` removes `<temp>` deterministically on scope exit.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ over the subworkspace SysML schema and `FAILURE_MODE_REGISTRY.md` rows.
/// - Space Complexity: $O(N)$ for subprocess output buffers and generated safety Markdown artifacts.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Invokes `CARGO_BIN_EXE_compile-sysml` via `std::process::Command` with explicit working directory.
/// - State: Scoped exclusively to `TempDirGuard::path()` with zero process-global `set_current_dir` mutation.
/// - Action: Executes with zero `.unwrap()`, `.expect()`, or silent error discarding.
/// - Resource: `TempDirGuard` Drop guarantees deterministic filesystem cleanup even on RED assertion failure.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-SUBWORKSPACE-OUTDIR], [REQ-STPA-FMECA-REGISTRY-INGESTION] | SysML v2: VerificationCaseDef(test_issue_693_stpa_transpile_scopes_subworkspace_out_dir_and_ingests_failure_mode_registry)
#[test]
fn test_issue_693_stpa_transpile_scopes_subworkspace_out_dir_and_ingests_failure_mode_registry(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDirGuard::new("deap_issue_693_stpa_transpile")?;
    let subws = temp.path().join(".pipeline").join("upstream");
    let subws_pipeline = subws.join(".pipeline");
    let subws_research = subws.join("docs").join("research");
    fs::create_dir_all(&subws_pipeline)?;
    fs::create_dir_all(&subws_research)?;

    let schema_src = "\
package QueryEnginePkg {
    part def QueryPlanner {
        port sqlIngress;
        port planEgress;
        attribute maxRetries : Integer;
    }
}
";
    fs::write(subws_pipeline.join("schema.sysml"), schema_src)?;

    let registry_md = "\
# Failure Mode Registry

| Component | Dimension | Failure Mode | Effect | S | O | D | Mitigation | Basis |
| :--- | :--- | :--- | :--- | :---: | :---: | :---: | :--- | :--- |
| QueryPlanner | Resource | Planner Memoization Arena Exhaustion | Out-of-memory abort during join planning | 7 | 2 | 2 | Enforce per-query arena quota | SSOT |
";
    fs::write(subws_research.join("FAILURE_MODE_REGISTRY.md"), registry_md)?;

    let output = Command::new(env!("CARGO_BIN_EXE_compile-sysml"))
        .current_dir(temp.path())
        .args(["--schema", ".pipeline/upstream", "--stpa-transpile"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "compile-sysml --schema .pipeline/upstream --stpa-transpile failed (status {:?}):\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        stdout,
        stderr
    );

    let root_polluted_fmeca = temp.path().join("docs").join("safety").join("05_FMECA_MATRIX.md");
    assert!(
        !root_polluted_fmeca.exists(),
        "--schema .pipeline/upstream --stpa-transpile must NOT pollute workspace root docs/safety/05_FMECA_MATRIX.md"
    );

    let scoped_fmeca_path = subws.join("docs").join("safety").join("05_FMECA_MATRIX.md");
    let scoped_stpa_path = subws.join("docs").join("safety").join("STPA_MATRIX.md");
    assert!(
        scoped_fmeca_path.is_file(),
        "Expected subworkspace FMECA matrix at {}",
        scoped_fmeca_path.display()
    );
    assert!(
        scoped_stpa_path.is_file(),
        "Expected subworkspace STPA matrix at {}",
        scoped_stpa_path.display()
    );

    let fmeca_md = fs::read_to_string(&scoped_fmeca_path)?;
    assert!(
        fmeca_md.contains("Planner Memoization Arena Exhaustion")
            && fmeca_md.contains("| 7 | 2 | 2 | 28 |")
            && fmeca_md.contains("| SSOT |"),
        "Subworkspace 05_FMECA_MATRIX.md must ingest FAILURE_MODE_REGISTRY.md entry with | 7 | 2 | 2 | 28 | and | SSOT |:\n{}",
        fmeca_md
    );

    let unresearched_table_rows: Vec<&str> = fmeca_md
        .lines()
        .filter(|line| {
            line.starts_with('|')
                && line.contains("FMECA-")
                && !line.contains("Planner Memoization Arena Exhaustion")
        })
        .collect();
    assert!(
        !unresearched_table_rows.is_empty(),
        "Expected unresearched AST-derived FMECA table rows alongside ingested registry entry:\n{}",
        fmeca_md
    );
    assert!(
        unresearched_table_rows
            .iter()
            .all(|line| line.contains("| Derived |") && !line.contains("| SSOT |")),
        "All unresearched AST-derived rows in 05_FMECA_MATRIX.md must carry | Derived | and never | SSOT |:\n{}",
        fmeca_md
    );

    Ok(())
}

/// Verifies that transpiling and emitting the safety suite for a pure software `PackageDef`
/// (`DatabaseTransactionPkg`) with zero `CalcDef` and zero `ConstraintDef` declarations produces
/// `06_REGULATORY_OBJECTIVES_ASSESSMENT.md` (and never `06_SORA_SAIL_ASSESSMENT.md`), contains zero
/// hardcoded UAV flight-physics `PROOF_TEMPLATES` or drone SORA/SAIL/GRC/ARC/OSO substrings across
/// all 10 emitted safety artifacts, and emits zero `sldv.assert(` statements in `SLDV_FORMAL_PROOFS.m`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
/// for fn `test_issue_695_non_uav_safety_suite_eliminates_hardcoded_uav_physics_proofs_and_sora_tables`
/// in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #695).
///
/// ## 2. Invariants & Mathematical Properties
/// - Pure Schema-Driven Safety Suite Invariant: For any domain-neutral `PackageDef` $P$ containing
///   zero UAV aerodynamic concepts, zero `CalcDef`, and zero `ConstraintDef` nodes,
///   $\text{transpile\_safety\_suite}(P)$ emits exact cardinality $|\text{suite}| = 10$ keyed with
///   `06_REGULATORY_OBJECTIVES_ASSESSMENT.md` rather than `06_SORA_SAIL_ASSESSMENT.md`, contains
///   zero UAV flight-physics formulas or JARUS SORA/SAIL/GRC/ARC/OSO tables in any artifact value,
///   and emits zero fabricated `sldv.assert(` obligations in `SLDV_FORMAL_PROOFS.m`.
///
/// ## 3. Preconditions
/// - An in-memory `DatabaseTransactionPkg` containing `PartDef` `TransactionCoordinator` and
///   `ActionDef` `CommitWriteBatch` (with `calc_defs.is_empty()` and `constraint_defs.is_empty()`)
///   is constructed alongside an isolated `TempDirGuard` output directory.
///
/// ## 4. Postconditions
/// - `suite.len() == 10`, `!suite.contains_key("06_SORA_SAIL_ASSESSMENT.md")`,
///   `!temp.path().join("06_SORA_SAIL_ASSESSMENT.md").exists()`, and
///   `suite.contains_key("06_REGULATORY_OBJECTIVES_ASSESSMENT.md")`.
/// - All 10 artifacts contain zero UAV flight-physics substrings and zero SORA/SAIL/OSO substrings.
/// - `SLDV_FORMAL_PROOFS.m` contains zero `"sldv.assert("` occurrences.
/// - `TempDirGuard` cleans up the temporary directory deterministically on scope exit.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(A \cdot L)$ where $A = 10$ is the safety artifact count and $L$ is bounded artifact byte length.
/// - Space Complexity: $O(A \cdot L)$ for the in-memory `BTreeMap<String, String>` safety suite.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Invokes both `transpile_safety_suite(&pkg, None)` and `emit_safety_suite(&pkg, temp.path(), None)`.
/// - State: Isolated to in-memory buffers and an ephemeral `TempDirGuard` directory.
/// - Action: Executes with zero `.unwrap()`, `.expect()`, `panic!`, `regex`, or direct slice indexing.
/// - Resource: `TempDirGuard` Drop reclaims temporary files even when unwinding on RED failure.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-SAFETY-SUITE-NO-HARDCODED-UAV-PROOFS], [REQ-STPA-SAFETY-SUITE-NO-HARDCODED-SORA] | SysML v2: VerificationCaseDef(test_issue_695_non_uav_safety_suite_eliminates_hardcoded_uav_physics_proofs_and_sora_tables)
#[test]
fn test_issue_695_non_uav_safety_suite_eliminates_hardcoded_uav_physics_proofs_and_sora_tables(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pkg = PackageDef {
        name: "DatabaseTransactionPkg".to_string(),
        ..Default::default()
    };
    pkg.part_defs.push(PartDef {
        name: "TransactionCoordinator".to_string(),
        ..Default::default()
    });
    pkg.action_defs.push(ActionDef {
        name: "CommitWriteBatch".to_string(),
        ..Default::default()
    });

    let temp = TempDirGuard::new("deap_issue_695_non_uav_safety_suite")?;
    let suite = compile_sysml::stpa::safety_suite::transpile_safety_suite(&pkg, None);
    compile_sysml::stpa::safety_suite::emit_safety_suite(&pkg, temp.path(), None)?;

    assert_eq!(
        suite.len(),
        10,
        "transpile_safety_suite must emit 10 safety artifacts, got {}: {:?}",
        suite.len(),
        suite.keys().collect::<Vec<_>>()
    );
    assert!(
        !suite.contains_key("06_SORA_SAIL_ASSESSMENT.md"),
        "Safety suite must not contain drone-specific 06_SORA_SAIL_ASSESSMENT.md, got keys: {:?}",
        suite.keys().collect::<Vec<_>>()
    );
    assert!(
        !temp.path().join("06_SORA_SAIL_ASSESSMENT.md").exists(),
        "emit_safety_suite must not write 06_SORA_SAIL_ASSESSMENT.md to disk"
    );
    assert!(
        suite.contains_key("06_REGULATORY_OBJECTIVES_ASSESSMENT.md"),
        "Safety suite must contain domain-neutral 06_REGULATORY_OBJECTIVES_ASSESSMENT.md, got keys: {:?}",
        suite.keys().collect::<Vec<_>>()
    );
    assert!(
        temp.path()
            .join("06_REGULATORY_OBJECTIVES_ASSESSMENT.md")
            .is_file(),
        "emit_safety_suite must write 06_REGULATORY_OBJECTIVES_ASSESSMENT.md to disk"
    );

    let forbidden_uav_physics = [
        "LiftToDragRatio",
        "SinkRate",
        "StallSpeed",
        "V_stall",
        "Aerodynamic drag",
        "PropulsionPower",
        "AvionicsPower",
        "GlideDistance",
        "WellClearRadius",
        "HorizontalSeparationAtCPA",
        "Terminal dive mass",
        "Aeroelastic dynamic pressure",
        "Kinetic Energy Dissipation Bound",
        "Containment Reach Bound",
        "Energy Balance Separation Bound",
    ];

    let forbidden_sora_substrings = [
        "SORA",
        "Ground Risk Class",
        "GRC",
        "Air Risk Class",
        "ARC",
        "Specific Assurance and Integrity Level",
        "| SAIL |",
        "| OSO ID |",
        "Operational Safety Objectives",
        "SAE ARP4761",
    ];

    for (artifact_name, content) in &suite {
        for needle in forbidden_uav_physics {
            assert!(
                !content.contains(needle),
                "Non-UAV safety artifact {} must contain zero hardcoded UAV flight-physics substring {:?}:\n{}",
                artifact_name,
                needle,
                content
            );
        }
        for needle in forbidden_sora_substrings {
            assert!(
                !content.contains(needle),
                "Non-UAV safety artifact {} must contain zero hardcoded drone SORA substring {:?}:\n{}",
                artifact_name,
                needle,
                content
            );
        }
    }

    let sldv_content = suite
        .get("SLDV_FORMAL_PROOFS.m")
        .ok_or("Missing SLDV_FORMAL_PROOFS.m in transpiled safety suite")?;
    assert!(
        !sldv_content.contains("sldv.assert("),
        "SLDV_FORMAL_PROOFS.m must contain zero sldv.assert( calls when PackageDef has zero CalcDef and zero ConstraintDef nodes:\n{}",
        sldv_content
    );

    Ok(())
}

/// Verifies that `transpile_safety_suite` derives RTA mathematical proofs (`07_RTA_ARCHITECTURE.md`
/// and `STPA_MATRIX.md`), Simulink Design Verifier proof obligations (`SLDV_FORMAL_PROOFS.m`), and
/// domain regulatory objectives and integrity level mappings (`06_REGULATORY_OBJECTIVES_ASSESSMENT.md`
/// and `STPA_MATRIX.md` Section 8) purely and deterministically from AST `ConstraintDef`, `CalcDef`,
/// and `PartDef` `AttributeDef` declarations without fabricating UAV proofs or hardcoded OSO tables.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
/// for fn `test_issue_695_safety_suite_derives_rta_proofs_sldv_and_regulatory_objectives_purely_from_ast`
/// in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #695).
///
/// ## 2. Invariants & Mathematical Properties
/// - AST-Derived Proof Obligation Invariant: Declared `ConstraintDef("LatencyCeiling")` and
///   `CalcDef("HeadroomMargin")` nodes project deterministically into isolated `$$\n\begin{aligned}` ..
///   `\end{aligned}\n$$` KaTeX aligned blocks in `07_RTA_ARCHITECTURE.md` and `STPA_MATRIX.md`, and
///   into `sldv.assert(` obligations referencing `"LATENCYCEILING"` in `SLDV_FORMAL_PROOFS.m`, with
///   zero hardcoded UAV `PROOF_TEMPLATES`.
/// - AST-Derived Regulatory Mapping Invariant: Declared `AttributeDef` properties
///   `IntegrityLevel_Service_Tier = "SIL-3"` and `OBJ01_Objective = "Deterministic rollback under arena exhaustion"`
///   on `PartDef` `QuerySupervisor` project into `06_REGULATORY_OBJECTIVES_ASSESSMENT.md` and
///   `STPA_MATRIX.md` (`"## 8. Domain Regulatory Objectives & Integrity Level Mapping"`), preserving
///   `"Service Tier"`, `"SIL-3"`, `"OBJ-01"`, and `"Deterministic rollback under arena exhaustion"`
///   without fabricating `"OSO"` rows or unmodeled `"OBJ-04"`..`"OBJ-10"` padding rows.
///
/// ## 3. Preconditions
/// - An in-memory `QuerySafetyPkg` is constructed with `ConstraintDef` `LatencyCeiling`
///   (`"queryLatencyMs <= 50"`), `CalcDef` `HeadroomMargin` (`Some("quotaMb - activeUsageMb")`),
///   and `PartDef` `QuerySupervisor` carrying `IntegrityLevel_Service_Tier = "SIL-3"` and
///   `OBJ01_Objective = "Deterministic rollback under arena exhaustion"`.
///
/// ## 4. Postconditions
/// - `07_RTA_ARCHITECTURE.md` and `STPA_MATRIX.md` contain isolated aligned display-math blocks
///   referencing `HeadroomMargin` and `LatencyCeiling` and zero UAV `PROOF_TEMPLATES`.
/// - `SLDV_FORMAL_PROOFS.m` contains `"sldv.assert("` and `"LATENCYCEILING"` and zero UAV `PROOF_TEMPLATES`.
/// - `06_REGULATORY_OBJECTIVES_ASSESSMENT.md` and `STPA_MATRIX.md` contain `"Service Tier"`, `"SIL-3"`,
///   `"OBJ-01"`, and `"Deterministic rollback under arena exhaustion"`, `STPA_MATRIX.md` contains
///   `"## 8. Domain Regulatory Objectives & Integrity Level Mapping"`, and neither fabricates `"OSO"`
///   rows or `"OBJ-04"`..`"OBJ-10"`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(A \cdot L)$ over the 10 generated safety artifacts.
/// - Space Complexity: $O(A \cdot L)$ for the in-memory `BTreeMap<String, String>`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Invokes `compile_sysml::stpa::safety_suite::transpile_safety_suite(&pkg, None)` directly.
/// - State: Pure in-memory evaluation with zero global state or filesystem side effects.
/// - Action: Executes with zero `.unwrap()`, `.expect()`, `panic!`, `regex`, or direct slice indexing.
/// - Resource: Bounded to a single synthetic `PackageDef` and 10 in-memory Markdown/MATLAB strings.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-AST-DERIVED-RTA-SLDV-PROOFS], [REQ-STPA-AST-DERIVED-REGULATORY-OBJECTIVES] | SysML v2: VerificationCaseDef(test_issue_695_safety_suite_derives_rta_proofs_sldv_and_regulatory_objectives_purely_from_ast)
#[test]
fn test_issue_695_safety_suite_derives_rta_proofs_sldv_and_regulatory_objectives_purely_from_ast(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut supervisor = PartDef {
        name: "QuerySupervisor".to_string(),
        ..Default::default()
    };
    supervisor.attributes.push(AttributeDef {
        name: "IntegrityLevel_Service_Tier".to_string(),
        default_value: Some("SIL-3".to_string()),
        ..Default::default()
    });
    supervisor.attributes.push(AttributeDef {
        name: "OBJ01_Objective".to_string(),
        default_value: Some("Deterministic rollback under arena exhaustion".to_string()),
        ..Default::default()
    });

    let mut pkg = PackageDef {
        name: "QuerySafetyPkg".to_string(),
        ..Default::default()
    };
    pkg.part_defs.push(supervisor);
    pkg.constraint_defs.push(ConstraintDef {
        name: "LatencyCeiling".to_string(),
        expression: "queryLatencyMs <= 50".to_string(),
        ..Default::default()
    });
    pkg.calc_defs.push(CalcDef {
        name: "HeadroomMargin".to_string(),
        expression: Some("quotaMb - activeUsageMb".to_string()),
        ..Default::default()
    });

    let suite = compile_sysml::stpa::safety_suite::transpile_safety_suite(&pkg, None);

    let forbidden_uav_physics = [
        "LiftToDragRatio",
        "SinkRate",
        "StallSpeed",
        "V_stall",
        "Aerodynamic drag",
        "PropulsionPower",
        "AvionicsPower",
        "GlideDistance",
        "WellClearRadius",
        "HorizontalSeparationAtCPA",
        "Terminal dive mass",
        "Aeroelastic dynamic pressure",
        "Kinetic Energy Dissipation Bound",
        "Containment Reach Bound",
        "Energy Balance Separation Bound",
    ];

    for (artifact_name, content) in &suite {
        for needle in forbidden_uav_physics {
            assert!(
                !content.contains(needle),
                "AST-derived safety artifact {} must contain zero UAV PROOF_TEMPLATES substring {:?}:\n{}",
                artifact_name,
                needle,
                content
            );
        }
    }

    let rta_md = suite
        .get("07_RTA_ARCHITECTURE.md")
        .ok_or("Missing 07_RTA_ARCHITECTURE.md in transpiled safety suite")?;
    let stpa_md = suite
        .get("STPA_MATRIX.md")
        .ok_or("Missing STPA_MATRIX.md in transpiled safety suite")?;

    for (doc_name, doc_content) in [
        ("07_RTA_ARCHITECTURE.md", rta_md.as_str()),
        ("STPA_MATRIX.md", stpa_md.as_str()),
    ] {
        assert!(
            doc_content.contains("$$\n\\begin{aligned}")
                && doc_content.contains("\\end{aligned}\n$$"),
            "{} must emit isolated $$\\n\\begin{{aligned}} .. \\end{{aligned}}\\n$$ display math blocks:\n{}",
            doc_name,
            doc_content
        );
        assert!(
            doc_content.contains("HeadroomMargin") || doc_content.contains("HEADROOMMARGIN"),
            "{} must reference AST CalcDef HeadroomMargin:\n{}",
            doc_name,
            doc_content
        );
        assert!(
            doc_content.contains("LatencyCeiling") || doc_content.contains("LATENCYCEILING"),
            "{} must reference AST ConstraintDef LatencyCeiling:\n{}",
            doc_name,
            doc_content
        );
    }

    let sldv_m = suite
        .get("SLDV_FORMAL_PROOFS.m")
        .ok_or("Missing SLDV_FORMAL_PROOFS.m in transpiled safety suite")?;
    assert!(
        sldv_m.contains("sldv.assert(") && sldv_m.contains("LATENCYCEILING"),
        "SLDV_FORMAL_PROOFS.m must derive sldv.assert( proof obligations referencing LATENCYCEILING from AST ConstraintDef:\n{}",
        sldv_m
    );

    let reg_md = suite
        .get("06_REGULATORY_OBJECTIVES_ASSESSMENT.md")
        .ok_or("Missing 06_REGULATORY_OBJECTIVES_ASSESSMENT.md in transpiled safety suite")?;

    assert!(
        stpa_md.contains("## 8. Domain Regulatory Objectives & Integrity Level Mapping"),
        "STPA_MATRIX.md must contain '## 8. Domain Regulatory Objectives & Integrity Level Mapping':\n{}",
        stpa_md
    );

    for (doc_name, doc_content) in [
        ("06_REGULATORY_OBJECTIVES_ASSESSMENT.md", reg_md.as_str()),
        ("STPA_MATRIX.md", stpa_md.as_str()),
    ] {
        for expected in [
            "Service Tier",
            "SIL-3",
            "OBJ-01",
            "Deterministic rollback under arena exhaustion",
        ] {
            assert!(
                doc_content.contains(expected),
                "{} must contain AST-derived regulatory mapping substring {:?}:\n{}",
                doc_name,
                expected,
                doc_content
            );
        }
        assert!(
            !doc_content.contains("OSO-") && !doc_content.contains("| OSO ID |"),
            "{} must not contain hardcoded drone SORA OSO rows:\n{}",
            doc_name,
            doc_content
        );
        for unmodeled_obj in [
            "OBJ-04", "OBJ-05", "OBJ-06", "OBJ-07", "OBJ-08", "OBJ-09", "OBJ-10",
        ] {
            assert!(
                !doc_content.contains(unmodeled_obj),
                "{} must not fabricate unmodeled objective row {:?} beyond declared AST items:\n{}",
                doc_name,
                unmodeled_obj,
                doc_content
            );
        }
    }

    Ok(())
}

/// Verifies that `transpile_safety_suite` deterministically lowers SysML v2 relational, logical,
/// arithmetic-modulo, identifier, and whitespace-only expressions from `CalcDef` and `ConstraintDef`
/// AST nodes into valid KaTeX display-math blocks (`07_RTA_ARCHITECTURE.md`, `STPA_MATRIX.md`) and
/// valid MATLAB/Simulink Design Verifier `sldv.assert` proof obligations (`07_RTA_ARCHITECTURE.md`,
/// `STPA_MATRIX.md`, `SLDV_FORMAL_PROOFS.m`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C formal proof
/// transpilation objectives for fn `test_issue_698_stpa_lowers_sysml_operators_to_katex_and_matlab`
/// in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #698).
///
/// ## 2. Invariants & Mathematical Properties
/// - KaTeX Operator & Identifier Lowering Invariant: Every `$$ ... $$` display-math block in
///   `07_RTA_ARCHITECTURE.md` and `STPA_MATRIX.md` lowers SysML v2 operators (`and`, `or`, `not`,
///   `implies`, `xor`, `>=`, `<=`, `!=`, `==`, `%`) to canonical KaTeX tokens (`\land`, `\lor`,
///   `\lnot`, `\implies`, `\oplus`, `\ge`, `\le`, `\neq`, `=`, `\bmod`), escapes snake_case
///   identifiers inside `\mathrm{...}`, maps whitespace-only expressions to `\mathrm{true}`,
///   contains zero unescaped `%` comment characters, and preserves balanced `{` and `}` braces.
/// - MATLAB/SLDV Operator Lowering Invariant: Every `sldv.assert(` statement across
///   `07_RTA_ARCHITECTURE.md`, `STPA_MATRIX.md`, and `SLDV_FORMAL_PROOFS.m` lowers SysML v2
///   boolean, implication, exclusive-or, inequality, modulo, and whitespace-only expressions to
///   syntactically valid MATLAB constructs (`&&`, `||`, `~`, `~=`, `xor(`, `mod(`, `(true)`) with
///   zero raw SysML word operators, zero `!=` or `%` tokens, and zero empty `sldv.assert( (),` calls.
///
/// ## 3. Preconditions
/// - An in-memory `PackageDef` (`SafetyLoweringPkg`) is constructed containing `PartDef`
///   (`InvariantSupervisor`) with `ActionDef` (`EvaluateGuards`), `CalcDef` (`Margin_Check`),
///   `CalcDef` (`Whitespace_Calc`), and `ConstraintDef` (`Slice_Bound`).
///
/// ## 4. Postconditions
/// - `07_RTA_ARCHITECTURE.md` and `STPA_MATRIX.md` contain non-empty, brace-balanced `$$ ... $$`
///   math blocks free of raw SysML operators and unescaped `%`, with all expected KaTeX operators
///   and `\mathrm{...}` lowered identifiers present in `07_RTA_ARCHITECTURE.md`.
/// - All `sldv.assert(` lines across `07_RTA_ARCHITECTURE.md`, `STPA_MATRIX.md`, and
///   `SLDV_FORMAL_PROOFS.m` contain zero raw SysML operators or empty `()` predicates, and
///   `SLDV_FORMAL_PROOFS.m` contains `&&`, `||`, unary `~`, `~=`, `xor(`, `mod(`, and `(true)`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(A \cdot L)$ single linear scan over generated Markdown and MATLAB buffers.
/// - Space Complexity: $O(A \cdot L)$ bounded in-memory `BTreeMap<String, String>` storage.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Invokes `compile_sysml::stpa::safety_suite::transpile_safety_suite(&pkg, None)` directly.
/// - State: Pure in-memory AST fixture with zero shared mutable state or filesystem writes.
/// - Action: Executes with zero `.unwrap()`, `.expect()`, `panic!`, `regex`, or direct slice indexing.
/// - Resource: Bounded single-package AST allocation reclaimed at test scope exit.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING] | SysML v2: VerificationCaseDef(test_issue_698_stpa_lowers_sysml_operators_to_katex_and_matlab)
#[test]
fn test_issue_698_stpa_lowers_sysml_operators_to_katex_and_matlab(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut supervisor = PartDef {
        name: "InvariantSupervisor".to_string(),
        ..Default::default()
    };
    supervisor.actions.push(ActionDef {
        name: "EvaluateGuards".to_string(),
        ..Default::default()
    });

    let mut pkg = PackageDef {
        name: "SafetyLoweringPkg".to_string(),
        ..Default::default()
    };
    pkg.part_defs.push(supervisor);
    pkg.calc_defs.push(CalcDef {
        name: "Margin_Check".to_string(),
        expression: Some(
            "guard_active and not fault_latched implies (total_ticks % cycle_period) == 0"
                .to_string(),
        ),
        ..Default::default()
    });
    pkg.calc_defs.push(CalcDef {
        name: "Whitespace_Calc".to_string(),
        expression: Some("   ".to_string()),
        ..Default::default()
    });
    pkg.constraint_defs.push(ConstraintDef {
        name: "Slice_Bound".to_string(),
        expression: "direct_range_slice_count >= 0 and direct_range_slice_count <= 0 or (backup_ok xor override_flag != false)".to_string(),
        ..Default::default()
    });

    let suite = compile_sysml::stpa::safety_suite::transpile_safety_suite(&pkg, None);
    let rta_md = suite
        .get("07_RTA_ARCHITECTURE.md")
        .ok_or("Missing 07_RTA_ARCHITECTURE.md in transpiled safety suite")?;
    let stpa_md = suite
        .get("STPA_MATRIX.md")
        .ok_or("Missing STPA_MATRIX.md in transpiled safety suite")?;
    let sldv_m = suite
        .get("SLDV_FORMAL_PROOFS.m")
        .ok_or("Missing SLDV_FORMAL_PROOFS.m in transpiled safety suite")?;

    let forbidden_katex_tokens = [
        ">=",
        "<=",
        "!=",
        "==",
        "&&",
        "||",
        " and ",
        " or ",
        " not ",
        " implies ",
        " xor ",
    ];

    for (doc_name, doc_content) in [
        ("07_RTA_ARCHITECTURE.md", rta_md.as_str()),
        ("STPA_MATRIX.md", stpa_md.as_str()),
    ] {
        let split_count = doc_content.split("$$").count();
        assert!(
            split_count >= 3 && split_count % 2 == 1,
            "{} must contain at least one properly closed $$ ... $$ display-math block (split count = {}):\n{}",
            doc_name,
            split_count,
            doc_content
        );

        for math_block in doc_content.split("$$").skip(1).step_by(2) {
            for forbidden in forbidden_katex_tokens {
                assert!(
                    !math_block.contains(forbidden),
                    "{} KaTeX math block must not contain raw SysML operator {:?}:\n$${}$$",
                    doc_name,
                    forbidden,
                    math_block
                );
            }

            let mut prev_escaped = false;
            let mut has_unescaped_percent = false;
            let mut brace_depth: i64 = 0;
            let mut negative_brace_depth = false;
            for ch in math_block.chars() {
                if ch == '{' && !prev_escaped {
                    brace_depth += 1;
                } else if ch == '}' && !prev_escaped {
                    brace_depth -= 1;
                    if brace_depth < 0 {
                        negative_brace_depth = true;
                    }
                } else if ch == '%' && !prev_escaped {
                    has_unescaped_percent = true;
                }

                prev_escaped = ch == '\\' && !prev_escaped;
            }

            assert!(
                !has_unescaped_percent,
                "{} KaTeX math block must not contain unescaped '%' comment character:\n$${}$$",
                doc_name,
                math_block
            );
            assert!(
                !negative_brace_depth && brace_depth == 0,
                "{} KaTeX math block must have balanced '{{' and '}}' braces (final depth = {}, negative = {}):\n$${}$$",
                doc_name,
                brace_depth,
                negative_brace_depth,
                math_block
            );
        }
    }

    let expected_katex_tokens = [
        "\\land",
        "\\lor",
        "\\lnot",
        "\\implies",
        "\\oplus",
        "\\ge",
        "\\le",
        "\\neq",
        "\\bmod",
        "\\mathrm{direct\\_range\\_slice\\_count}",
        "\\mathrm{guard\\_active}",
        "\\mathrm{true}",
    ];
    for expected in expected_katex_tokens {
        assert!(
            rta_md.contains(expected),
            "07_RTA_ARCHITECTURE.md must contain lowered KaTeX token {:?}:\n{}",
            expected,
            rta_md
        );
    }

    let forbidden_matlab_tokens = [
        " and ",
        " or ",
        " not ",
        " implies ",
        " xor ",
        "!=",
        "%",
        "sldv.assert( (),",
    ];

    for (doc_name, doc_content) in [
        ("07_RTA_ARCHITECTURE.md", rta_md.as_str()),
        ("STPA_MATRIX.md", stpa_md.as_str()),
        ("SLDV_FORMAL_PROOFS.m", sldv_m.as_str()),
    ] {
        for line in doc_content.lines().filter(|l| l.contains("sldv.assert(")) {
            for forbidden in forbidden_matlab_tokens {
                assert!(
                    !line.contains(forbidden),
                    "{} sldv.assert line must not contain raw/invalid token {:?}:\n{}",
                    doc_name,
                    forbidden,
                    line
                );
            }
        }
    }

    assert!(
        sldv_m.lines().any(|l| l.contains("sldv.assert(")),
        "SLDV_FORMAL_PROOFS.m must contain sldv.assert( statements:\n{}",
        sldv_m
    );

    let expected_matlab_tokens = ["&&", "||", "~", "~=", "xor(", "mod(", "(true)"];
    for expected in expected_matlab_tokens {
        assert!(
            sldv_m.contains(expected),
            "SLDV_FORMAL_PROOFS.m must contain lowered MATLAB operator/predicate {:?}:\n{}",
            expected,
            sldv_m
        );
    }
    assert!(
        sldv_m
            .chars()
            .zip(sldv_m.chars().skip(1))
            .any(|(a, b)| a == '~' && b != '='),
        "SLDV_FORMAL_PROOFS.m must contain unary MATLAB negation '~' distinct from '~=':\n{}",
        sldv_m
    );

    Ok(())
}

/// Verifies that `expand_cartesian_stpa` and `transpile_safety_suite` derive UCA controller
/// ownership, severity/integrity levels, Class-1 and Class-2 loss scenarios, and hazard log rows
/// purely from SysML v2 AST nodes without emitting `PENDING_PARAMETER`, `DEAPCompilerSystem`, or
/// placeholder loss/hazard skeletons across both populated and minimal `PackageDef` models.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety analysis
/// derivation objectives for fn
/// `test_issue_697_stpa_derives_uca_loss_scenarios_and_hazard_log_from_ast_without_pending_parameter_or_deap_compiler_system`
/// in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #697).
///
/// ## 2. Invariants & Mathematical Properties
/// - AST Controller & Integrity Resolution Invariant: When a `PackageDef` declares a `PartDef`
///   (`TransactionSupervisor`) whose identifier neither contains `"System"` nor ends with
///   `"Engine"`, `expand_cartesian_stpa` binds all $4 \times |\text{ActionDef}| = 8$ UCAs to
///   `controller == "TransactionSupervisor"` (never `"DEAPCompilerSystem"`), resolves `sail` from
///   `IntegrityLevel_Tier = "SIL-3"`, and emits zero `"PENDING_PARAMETER"` substrings in `severity`
///   or `sail`. When `part_defs` is empty, `controller` falls back deterministically to the
///   enclosing `PackageDef` name (`"SysML_Model"`).
/// - Skeleton-Free Safety Suite Invariant: Across all 10 artifacts produced by
///   `transpile_safety_suite`, zero occurrences of `"PENDING_PARAMETER"`, `"DEAPCompilerSystem"`,
///   `"Loss scenario skeleton"`, `"Skeleton loss entry"`, or `"Compound hazard skeleton"` appear,
///   even in the presence of unmatched constraint parameters, expressionless `ConstraintDef` /
///   `CalcDef` nodes, and package-level `ActionDef` declarations.
/// - AST-Derived Loss Scenario & Hazard Log Invariant: `03_LOSS_SCENARIOS.md` and `STPA_MATRIX.md`
///   synthesize both `"Class-1"` and `"Class-2"` loss scenarios referencing
///   `"TransactionSupervisor"`, AST ports (`"cmdIngress"` or `"ackEgress"`), AST states
///   (`"Standby"` or `"Committing"`), `"WalCommitBound"`, and `"SC-001"`, while `HAZARD_LOG.md`
///   synthesizes `"H-1"` and `"H-2"` rows and a resolved `"Resolution Authority"` without
///   `"PENDING_PARAMETER"`.
///
/// ## 3. Preconditions
/// - Fixture A constructs an in-memory `PackageDef` (`"SysML_Model"`) with `PartDef`
///   `"TransactionSupervisor"` (2 ports, 2 states, 2 attributes, zero inline actions), 2
///   package-level `ActionDef`s, 2 `ConstraintDef`s (one with unmatched parameter, one
///   expressionless), 1 expressionless `CalcDef`, and 1 `RequirementDef`.
/// - Fixture B constructs a minimal `PackageDef` (`"SysML_Model"`) with zero `PartDef`s, zero
///   package attributes, and 1 package-level `ActionDef` (`"DispatchFrame"`).
///
/// ## 4. Postconditions
/// - Fixture A produces 8 UCAs bound to `"TransactionSupervisor"` with `"SIL-3"` integrity, 10
///   placeholder-free safety artifacts, AST-linked Class-1/Class-2 loss scenarios, and populated
///   `H-1`/`H-2` hazard log entries.
/// - Fixture B binds UCAs to `"SysML_Model"` and emits 10 safety artifacts with zero
///   `"PENDING_PARAMETER"` and zero `"DEAPCompilerSystem"` occurrences.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(U + A \cdot L)$ where $U$ is UCA cardinality and $A = 10$ is the safety
///   suite artifact count of bounded byte length $L$.
/// - Space Complexity: $O(U + A \cdot L)$ for in-memory UCA vectors and safety artifact buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Invokes `compile_sysml::stpa::uca::expand_cartesian_stpa` and
///   `compile_sysml::stpa::safety_suite::transpile_safety_suite` directly.
/// - State: Pure in-memory deterministic fixtures with zero shared mutable state or disk I/O.
/// - Action: Executes with zero fallible unwrapping, panics, regular expressions, or byte-range slicing.
/// - Resource: Bounded heap allocations reclaimed automatically at function exit.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-AST-DERIVED-UCA-LOSS-SCENARIOS], [REQ-STPA-AST-DERIVED-HAZARD-LOG] | SysML v2: VerificationCaseDef(test_issue_697_stpa_derives_uca_loss_scenarios_and_hazard_log_from_ast_without_pending_parameter_or_deap_compiler_system)
#[test]
fn test_issue_697_stpa_derives_uca_loss_scenarios_and_hazard_log_from_ast_without_pending_parameter_or_deap_compiler_system(
) -> Result<(), Box<dyn std::error::Error>> {
    // Fixture A: Rich domain-neutral SysML_Model with non-"System"/non-"Engine" PartDef,
    // package-level actions, unmatched constraint parameter, and expressionless constraint/calc.
    let mut supervisor = PartDef {
        name: "TransactionSupervisor".to_string(),
        ..Default::default()
    };
    supervisor.ports.push(PortDef {
        name: "cmdIngress".to_string(),
        ..Default::default()
    });
    supervisor.ports.push(PortDef {
        name: "ackEgress".to_string(),
        ..Default::default()
    });
    supervisor.states.push(StateDef {
        name: "Standby".to_string(),
        ..Default::default()
    });
    supervisor.states.push(StateDef {
        name: "Committing".to_string(),
        ..Default::default()
    });
    supervisor.attributes.push(AttributeDef {
        name: "maxCommitJitterMs".to_string(),
        type_name: "Integer".to_string(),
        default_value: Some("15".to_string()),
        doc: None,
    });
    supervisor.attributes.push(AttributeDef {
        name: "IntegrityLevel_Tier".to_string(),
        type_name: "String".to_string(),
        default_value: Some("SIL-3".to_string()),
        doc: None,
    });

    let mut pkg = PackageDef {
        name: "SysML_Model".to_string(),
        ..Default::default()
    };
    pkg.part_defs.push(supervisor);
    pkg.action_defs.push(ActionDef {
        name: "PrepareWalBatch".to_string(),
        ..Default::default()
    });
    pkg.action_defs.push(ActionDef {
        name: "AbortStaleEpoch".to_string(),
        ..Default::default()
    });
    pkg.constraint_defs.push(ConstraintDef {
        name: "WalCommitBound".to_string(),
        expression: "commitJitterMs <= 15".to_string(),
        is_assertion: true,
        parameters: vec!["unmatchedParam: Integer".to_string()],
        ..Default::default()
    });
    pkg.constraint_defs.push(ConstraintDef {
        name: "QuorumGate".to_string(),
        expression: String::new(),
        ..Default::default()
    });
    pkg.calc_defs.push(CalcDef {
        name: "EpochHeadroom".to_string(),
        expression: None,
        in_params: vec![AttributeDef {
            name: "windowTicks".to_string(),
            type_name: "Integer".to_string(),
            default_value: None,
            doc: None,
        }],
        ..Default::default()
    });
    pkg.requirement_defs.push(RequirementDef {
        name: "ReqAtomicWal".to_string(),
        req_id: "REQ-WAL-01".to_string(),
        text: "Atomic WAL durability".to_string(),
        ..Default::default()
    });

    let ucas = compile_sysml::stpa::uca::expand_cartesian_stpa(&pkg);
    assert_eq!(
        ucas.len(),
        8,
        "expand_cartesian_stpa must return 8 UCAs for 2 package-level ActionDefs, got {}: {:?}",
        ucas.len(),
        ucas
    );
    for uca in &ucas {
        assert_eq!(
            uca.controller, "TransactionSupervisor",
            "UCA controller must resolve to AST PartDef 'TransactionSupervisor' (never 'DEAPCompilerSystem'): {:?}",
            uca
        );
        assert!(
            !uca.controller.contains("DEAPCompilerSystem"),
            "UCA controller must never contain hardcoded 'DEAPCompilerSystem': {:?}",
            uca
        );
        assert!(
            !uca.severity.contains("PENDING_PARAMETER"),
            "UCA severity must not contain 'PENDING_PARAMETER': {:?}",
            uca
        );
        assert!(
            !uca.sail.contains("PENDING_PARAMETER"),
            "UCA sail/integrity must not contain 'PENDING_PARAMETER': {:?}",
            uca
        );
        assert!(
            uca.sail.contains("SIL-3"),
            "UCA sail/integrity must derive 'SIL-3' from AST IntegrityLevel_Tier attribute: {:?}",
            uca
        );
    }

    let suite = compile_sysml::stpa::safety_suite::transpile_safety_suite(&pkg, None);
    assert_eq!(
        suite.len(),
        10,
        "transpile_safety_suite must emit 10 safety artifacts, got {}: {:?}",
        suite.len(),
        suite.keys().collect::<Vec<_>>()
    );

    let forbidden_skeleton_placeholders = [
        "PENDING_PARAMETER",
        "DEAPCompilerSystem",
        "Loss scenario skeleton",
        "Skeleton loss entry",
        "Compound hazard skeleton",
    ];
    for (artifact_name, content) in &suite {
        for forbidden in forbidden_skeleton_placeholders {
            assert!(
                !content.contains(forbidden),
                "Safety artifact {} must contain zero placeholder/hardcoded substring {:?}:\n{}",
                artifact_name,
                forbidden,
                content
            );
        }
    }

    let loss_scenarios_md = suite
        .get("03_LOSS_SCENARIOS.md")
        .ok_or("Missing 03_LOSS_SCENARIOS.md in transpiled safety suite")?;
    let stpa_matrix_md = suite
        .get("STPA_MATRIX.md")
        .ok_or("Missing STPA_MATRIX.md in transpiled safety suite")?;

    for (doc_name, doc_content) in [
        ("03_LOSS_SCENARIOS.md", loss_scenarios_md.as_str()),
        ("STPA_MATRIX.md", stpa_matrix_md.as_str()),
    ] {
        for required in [
            "Class-1",
            "Class-2",
            "TransactionSupervisor",
            "WalCommitBound",
            "SC-001",
        ] {
            assert!(
                doc_content.contains(required),
                "{} must contain AST-derived loss scenario token {:?}:\n{}",
                doc_name,
                required,
                doc_content
            );
        }
        assert!(
            doc_content.contains("cmdIngress") || doc_content.contains("ackEgress"),
            "{} must reference AST PortDef 'cmdIngress' or 'ackEgress' in loss scenarios:\n{}",
            doc_name,
            doc_content
        );
        assert!(
            doc_content.contains("Standby") || doc_content.contains("Committing"),
            "{} must reference AST StateDef 'Standby' or 'Committing' in loss scenarios:\n{}",
            doc_name,
            doc_content
        );
    }

    let hazard_log_md = suite
        .get("HAZARD_LOG.md")
        .ok_or("Missing HAZARD_LOG.md in transpiled safety suite")?;
    assert!(
        hazard_log_md
            .lines()
            .any(|line| line.starts_with('|') && line.contains("H-1")),
        "HAZARD_LOG.md must contain an 'H-1' table row:\n{}",
        hazard_log_md
    );
    assert!(
        hazard_log_md
            .lines()
            .any(|line| line.starts_with('|') && line.contains("H-2")),
        "HAZARD_LOG.md must contain an 'H-2' table row:\n{}",
        hazard_log_md
    );
    assert!(
        hazard_log_md.contains("Resolution Authority")
            && !hazard_log_md.contains("PENDING_PARAMETER"),
        "HAZARD_LOG.md must include a 'Resolution Authority' entry without 'PENDING_PARAMETER':\n{}",
        hazard_log_md
    );

    // Fixture B: Minimal PackageDef with zero PartDefs, zero package attributes, and 1 ActionDef.
    let mut minimal_pkg = PackageDef {
        name: "SysML_Model".to_string(),
        part_defs: Vec::new(),
        attribute_defs: Vec::new(),
        ..Default::default()
    };
    minimal_pkg.action_defs.push(ActionDef {
        name: "DispatchFrame".to_string(),
        ..Default::default()
    });

    let minimal_ucas = compile_sysml::stpa::uca::expand_cartesian_stpa(&minimal_pkg);
    assert!(
        !minimal_ucas.is_empty(),
        "expand_cartesian_stpa on minimal PackageDef with 1 ActionDef must return non-empty UCAs"
    );
    for uca in &minimal_ucas {
        assert_eq!(
            uca.controller, "SysML_Model",
            "Minimal PackageDef without PartDefs must assign controller == 'SysML_Model' (never 'DEAPCompilerSystem'): {:?}",
            uca
        );
        assert!(
            !uca.controller.contains("DEAPCompilerSystem"),
            "Minimal PackageDef UCA controller must never contain 'DEAPCompilerSystem': {:?}",
            uca
        );
    }

    let minimal_suite =
        compile_sysml::stpa::safety_suite::transpile_safety_suite(&minimal_pkg, None);
    assert_eq!(
        minimal_suite.len(),
        10,
        "transpile_safety_suite on minimal PackageDef must emit 10 artifacts, got {}",
        minimal_suite.len()
    );
    for (artifact_name, content) in &minimal_suite {
        assert!(
            !content.contains("PENDING_PARAMETER"),
            "Minimal PackageDef safety artifact {} must contain zero 'PENDING_PARAMETER':\n{}",
            artifact_name,
            content
        );
        assert!(
            !content.contains("DEAPCompilerSystem"),
            "Minimal PackageDef safety artifact {} must contain zero 'DEAPCompilerSystem':\n{}",
            artifact_name,
            content
        );
    }

    Ok(())
}

/// Verifies that parsing the real upstream schema `.pipeline/upstream/schema/model.sysml` and
/// running `transpile_safety_suite` produces all 10 safety artifacts with zero
/// `"PENDING_PARAMETER"` placeholders and zero hardcoded `"DEAPCompilerSystem"` controller leaks.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C end-to-end
/// model-to-safety-suite verification objectives for fn
/// `test_issue_697_upstream_sysml_model_transpiles_safety_suite_with_zero_pending_parameter`
/// in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #697).
///
/// ## 2. Invariants & Mathematical Properties
/// - Upstream SSOT Complete Transpilation Invariant: For the canonical upstream SysML v2 package
///   parsed from `.pipeline/upstream/schema/model.sysml`, every artifact $(k, v)$ in
///   $\text{transpile\_safety\_suite}(P, \text{None})$ satisfies $|v| > 0$,
///   $\text{"PENDING\_PARAMETER"} \notin v$, and $\text{"DEAPCompilerSystem"} \notin v$.
///
/// ## 3. Preconditions
/// - `.pipeline/upstream/schema/model.sysml` exists relative to `CARGO_MANIFEST_DIR` and parses
///   cleanly via `compile_sysml::parse_sysml`.
///
/// ## 4. Postconditions
/// - `suite.len() == 10` and all 10 emitted safety artifacts contain zero `"PENDING_PARAMETER"`
///   and zero `"DEAPCompilerSystem"` substrings.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(|S| + A \cdot L)$ where $|S|$ is the byte size of `model.sysml` and
///   $A \cdot L$ is total emitted safety suite byte size.
/// - Space Complexity: $O(|S| + A \cdot L)$ bounded in-memory AST and Markdown/MATLAB buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Resolves `model.sysml` relative to `env!("CARGO_MANIFEST_DIR")` without hardcoded
///   workstation paths and propagates file/parse errors via `Result`.
/// - State: Read-only filesystem load with zero mutations to repository state.
/// - Action: Executes with zero fallible unwrapping, panics, regular expressions, or byte-range slicing.
/// - Resource: Reclaims parsed AST and generated suite buffers deterministically at scope exit.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-STPA-UPSTREAM-MODEL-ZERO-PENDING-PARAMETER] | SysML v2: VerificationCaseDef(test_issue_697_upstream_sysml_model_transpiles_safety_suite_with_zero_pending_parameter)
#[test]
fn test_issue_697_upstream_sysml_model_transpiles_safety_suite_with_zero_pending_parameter(
) -> Result<(), Box<dyn std::error::Error>> {
    let model_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.pipeline/upstream/schema/model.sysml");
    let model_src = fs::read_to_string(&model_path)?;
    let pkg = compile_sysml::parse_sysml(&model_src)
        .map_err(|err| format!("Failed to parse {}: {:?}", model_path.display(), err))?;

    let suite = compile_sysml::stpa::safety_suite::transpile_safety_suite(&pkg, None);
    assert_eq!(
        suite.len(),
        10,
        "transpile_safety_suite on upstream model.sysml package {} must emit 10 safety artifacts, got {}: {:?}",
        pkg.name,
        suite.len(),
        suite.keys().collect::<Vec<_>>()
    );

    for (artifact_name, content) in &suite {
        assert!(
            !content.contains("PENDING_PARAMETER"),
            "Upstream model.sysml safety artifact {} (package {}) must contain zero 'PENDING_PARAMETER' occurrences:\n{}",
            artifact_name,
            pkg.name,
            content
        );
        assert!(
            !content.contains("DEAPCompilerSystem"),
            "Upstream model.sysml safety artifact {} (package {}) must contain zero 'DEAPCompilerSystem' occurrences:\n{}",
            artifact_name,
            pkg.name,
            content
        );
    }

    Ok(())
}

/// Constructs a domain-neutral distributed transaction ledger `PackageDef` (`DistributedLedgerSystem`
/// -> `JournalCommitSubsystem`) exercising AST-derived User Story synthesis for GitHub Issue #696.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic
/// test fixture construction objectives for fn `build_issue_696_distributed_ledger_fixture` in
/// `crates/compile-sysml/tests/reproducer_tests.rs` (refs #696).
///
/// ## 2. Invariants & Mathematical Properties
/// - Domain-Neutrality Invariant: Every identifier, port, state, constraint, interaction, and
///   requirement belongs strictly to a distributed storage / transaction journal ledger domain
///   with zero domain-specific hardware or vehicle tokens.
/// - Dual-Action Coverage Invariant: Populates both a documented package-level `ActionDef`
///   (`CommitJournalBatchAction` with `doc: Some(...)`) and an undocumented inline `PartDef`
///   `ActionDef` (`VerifyReplicaQuorumAction` with `doc: None`) alongside `InteractionDef`,
///   `StateDef` transitions, `ConstraintDef`, and `RequirementDef` bindings.
///
/// ## 3. Preconditions
/// - Pure in-memory construction with zero external filesystem or environment prerequisites.
///
/// ## 4. Postconditions
/// - Returns a deterministic `PackageDef` rooted at `"DistributedLedgerSystem"` containing
///   subpackage `"JournalCommitSubsystem"` with `"JournalCoordinatorEngine"`,
///   `"CommitJournalBatchAction"`, `"VerifyReplicaQuorumAction"`, `"CommitJournalInteraction"`,
///   `"JournalEpochState"`, and `"DurableQuorumRequirement"` (`"REQ-JRN-01"`).
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ bounded AST struct construction.
/// - Space Complexity: $O(1)$ bounded heap allocations for fixed AST vectors.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Total pure constructor taking zero parameters and returning `PackageDef`.
/// - State: Allocates fresh owned `PackageDef` instances with zero shared mutable state.
/// - Action: Executes with zero fallible unwrapping, panics, regular expressions, or byte-range slicing.
/// - Resource: Bounded heap memory owned by the caller.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYNC-US-FIXTURE-LEDGER-AST] | SysML v2: VerificationCaseDef(build_issue_696_distributed_ledger_fixture)
fn build_issue_696_distributed_ledger_fixture() -> PackageDef {
    let coordinator = PartDef {
        name: "JournalCoordinatorEngine".to_string(),
        attributes: vec![AttributeDef {
            name: "active_epoch_seq".to_string(),
            type_name: "Integer".to_string(),
            default_value: None,
            doc: None,
        }],
        ports: vec![
            PortDef {
                name: "batch_req_in".to_string(),
                type_name: "BatchRequestPort".to_string(),
                direction: "in".to_string(),
                ..Default::default()
            },
            PortDef {
                name: "commit_ack_out".to_string(),
                type_name: "CommitReceiptPort".to_string(),
                direction: "out".to_string(),
                ..Default::default()
            },
        ],
        constraints: vec![ConstraintDef {
            name: "QuorumWatermarkBound".to_string(),
            expression: "replica_ack_count >= 3".to_string(),
            ..Default::default()
        }],
        actions: vec![ActionDef {
            name: "VerifyReplicaQuorumAction".to_string(),
            in_params: vec![AttributeDef {
                name: "candidate_batch".to_string(),
                type_name: "JournalBatch".to_string(),
                default_value: None,
                doc: None,
            }],
            out_params: vec![AttributeDef {
                name: "quorum_verified".to_string(),
                type_name: "Boolean".to_string(),
                default_value: None,
                doc: None,
            }],
            doc: None,
            ..Default::default()
        }],
        ..Default::default()
    };

    let subpkg = PackageDef {
        name: "JournalCommitSubsystem".to_string(),
        part_defs: vec![coordinator],
        action_defs: vec![ActionDef {
            name: "CommitJournalBatchAction".to_string(),
            in_params: vec![
                AttributeDef {
                    name: "epoch_id".to_string(),
                    type_name: "EpochNumber".to_string(),
                    default_value: None,
                    doc: None,
                },
                AttributeDef {
                    name: "entries".to_string(),
                    type_name: "JournalEntryList".to_string(),
                    default_value: None,
                    doc: None,
                },
            ],
            out_params: vec![AttributeDef {
                name: "commit_receipt".to_string(),
                type_name: "CommitToken".to_string(),
                default_value: None,
                doc: None,
            }],
            doc: Some(
                "Actor: ClusterAdministrator\nPreconditions: Quorum lease is active and write-ahead log is open\nOutcome: journal batch is durably committed across quorum replicas"
                    .to_string(),
            ),
            ..Default::default()
        }],
        interaction_defs: vec![InteractionDef {
            name: "CommitJournalInteraction".to_string(),
            lifelines: vec![
                "admin : ClusterAdministrator".to_string(),
                "coord : JournalCoordinatorEngine".to_string(),
            ],
            messages: vec![
                "DispatchBatch from admin to coord".to_string(),
                "CommitReceiptReady from coord to admin".to_string(),
            ],
            ..Default::default()
        }],
        state_defs: vec![StateDef {
            name: "JournalEpochState".to_string(),
            transitions: vec![TransitionDef {
                source: Some("EpochIdle".to_string()),
                target: Some("EpochCommitted".to_string()),
                trigger: Some("CommitJournalBatchAction".to_string()),
                guard: Some("quorum_lease_valid == true".to_string()),
                effect: Some("EmitCommitReceipt".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        }],
        requirement_defs: vec![RequirementDef {
            name: "DurableQuorumRequirement".to_string(),
            req_id: "REQ-JRN-01".to_string(),
            text: "Durable quorum commit".to_string(),
            satisfied_by: vec!["JournalCoordinatorEngine".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };

    PackageDef {
        name: "DistributedLedgerSystem".to_string(),
        packages: vec![subpkg],
        ..Default::default()
    }
}

/// Verifies that `forward_sync_sysml_to_specs` synthesizes User Stories (`US-01-CommitJournalBatchAction.md`
/// and `US-02-VerifyReplicaQuorumAction.md`) with a canonical `## User Story` clause (`As a ..., I want to ..., so that ...`),
/// complete 3-layer Definition of Done architecture bindings (`Layer 1`, `Layer 2`, `Layer 3`),
/// AST-derived positive and negative BDD acceptance criteria, and zero hardcoded compiler self-description
/// boilerplate (`"processes the AST"`, `"semantic validator"`, `"graceful recovery occurs without data loss"`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C specification
/// derivation objectives for fn
/// `test_issue_696_forward_sync_user_story_derives_ast_clause_3_layer_bdd_and_zero_compiler_boilerplate`
/// in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #696).
///
/// ## 2. Invariants & Mathematical Properties
/// - AST-Derived User Story Clause Invariant: Every emitted `US-XX-*.md` artifact contains
///   `"## User Story"` with `"As a "`, `", I want to "`, and `", so that "`, resolving explicit
///   `doc` actor/outcome metadata on `US-01` (`ClusterAdministrator`, `CommitJournalBatchAction`,
///   `epoch_id: EpochNumber`, `JournalCoordinatorEngine`,
///   `"journal batch is durably committed across quorum replicas"`) and falling back to AST
///   requirement/constraint bindings (`REQ-JRN-01`, `DurableQuorumRequirement`, or
///   `QuorumWatermarkBound`) in the `so that` outcome of `US-02` (`doc: None`).
/// - Three-Layer Architectural Binding Invariant: Both `US-01` and `US-02` declare
///   `"Layer 1: Domain State & Data Model"`, `"Layer 2: Logic & State Management"`, and
///   `"Layer 3: Presentation & Actuator Interface Binding"` referencing `"active_epoch_seq"`,
///   `"batch_req_in"`, `"commit_ack_out"`, and `"QuorumWatermarkBound"`.
/// - AST-Derived BDD & Zero-Boilerplate Invariant: `US-01` `## Acceptance Criteria (BDD)` cites
///   AST preconditions, parameters, transition guards, target states, constraints, ports, and
///   interaction messages alongside a negative boundary rejection criterion without partial state
///   mutation, and neither `US-01` nor `US-02` emits compiler boilerplate substrings.
///
/// ## 3. Preconditions
/// - `build_issue_696_distributed_ledger_fixture()` constructs the `DistributedLedgerSystem` AST
///   and `TempDirGuard::new` provides an isolated RAII workspace path.
///
/// ## 4. Postconditions
/// - All User Story clause, 3-layer binding, positive/negative BDD criterion, and zero-boilerplate
///   assertions hold across `US-01` and `US-02`, and `TempDirGuard` removes temporary paths on drop.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ linear scan over the generated Markdown specification buffers.
/// - Space Complexity: $O(N)$ bounded heap storage for the returned `BTreeMap<String, String>`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Invokes `forward_sync_sysml_to_specs` with `dry_run: true, force: true`.
/// - State: Isolated via RAII `TempDirGuard` with zero shared global state.
/// - Action: Executes with zero fallible unwrapping, panics, regular expressions, or byte-range slicing.
/// - Resource: `TempDirGuard` Drop guarantees deterministic cleanup even during RED assertion unwinding.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYNC-US-AST-CLAUSE-AND-3LAYER], [REQ-SYNC-US-BDD-ZERO-BOILERPLATE] | SysML v2: VerificationCaseDef(test_issue_696_forward_sync_user_story_derives_ast_clause_3_layer_bdd_and_zero_compiler_boilerplate)
#[test]
fn test_issue_696_forward_sync_user_story_derives_ast_clause_3_layer_bdd_and_zero_compiler_boilerplate(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDirGuard::new("deap_issue_696_us_ast_derivation")?;
    let docs_dir = temp.path().join("docs");
    fs::create_dir_all(&docs_dir)?;

    let pkg = build_issue_696_distributed_ledger_fixture();
    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/distributed_ledger.sysml"),
        docs_dir: docs_dir.clone(),
        out_dir: Some(docs_dir),
        dry_run: true,
        force: true,
    };

    let generated = forward_sync_sysml_to_specs(&pkg, &opts)?;
    let us01 = generated
        .get("user-stories/US-01-CommitJournalBatchAction.md")
        .ok_or("Missing user-stories/US-01-CommitJournalBatchAction.md in forward sync output")?;
    let us02 = generated
        .get("user-stories/US-02-VerifyReplicaQuorumAction.md")
        .ok_or("Missing user-stories/US-02-VerifyReplicaQuorumAction.md in forward sync output")?;

    // 1. Both US-01 and US-02 contain "## User Story", "As a ", ", I want to ", and ", so that ".
    for (us_name, us_md) in [
        ("US-01-CommitJournalBatchAction.md", us01.as_str()),
        ("US-02-VerifyReplicaQuorumAction.md", us02.as_str()),
    ] {
        for required_clause_token in ["## User Story", "As a ", ", I want to ", ", so that "] {
            assert!(
                us_md.contains(required_clause_token),
                "{} must contain canonical User Story token {:?}:\n{}",
                us_name,
                required_clause_token,
                us_md
            );
        }
    }

    // 2. US-01 contains AST doc actor/outcome, action, typed input parameter, and owning PartDef.
    for expected_us01_token in [
        "ClusterAdministrator",
        "CommitJournalBatchAction",
        "epoch_id: EpochNumber",
        "JournalCoordinatorEngine",
        "journal batch is durably committed across quorum replicas",
    ] {
        assert!(
            us01.contains(expected_us01_token),
            "US-01 must contain AST-derived token {:?}:\n{}",
            expected_us01_token,
            us01
        );
    }

    // 3. US-02 (doc: None) contains action, typed input parameter, and cites REQ-JRN-01,
    //    DurableQuorumRequirement, or QuorumWatermarkBound in its `so that` outcome.
    for expected_us02_token in ["VerifyReplicaQuorumAction", "candidate_batch: JournalBatch"] {
        assert!(
            us02.contains(expected_us02_token),
            "US-02 must contain AST-derived token {:?}:\n{}",
            expected_us02_token,
            us02
        );
    }
    let (_, us02_after_story_heading) = us02
        .split_once("## User Story")
        .ok_or("US-02 missing '## User Story' section")?;
    let us02_story_section =
        if let Some((story_body, _)) = us02_after_story_heading.split_once("\n## ") {
            story_body
        } else {
            us02_after_story_heading
        };
    let (_, us02_so_that_clause) = us02_story_section
        .split_once(", so that ")
        .ok_or("US-02 '## User Story' section missing ', so that ' outcome clause")?;
    assert!(
        us02_so_that_clause.contains("REQ-JRN-01")
            || us02_so_that_clause.contains("DurableQuorumRequirement")
            || us02_so_that_clause.contains("QuorumWatermarkBound"),
        "US-02 (doc: None) 'so that' outcome must cite 'REQ-JRN-01', 'DurableQuorumRequirement', or 'QuorumWatermarkBound', got:\n{}",
        us02_so_that_clause
    );

    // 4. Both US-01 and US-02 contain 3-layer sections referencing AST state, ports, and constraint.
    for (us_name, us_md) in [
        ("US-01-CommitJournalBatchAction.md", us01.as_str()),
        ("US-02-VerifyReplicaQuorumAction.md", us02.as_str()),
    ] {
        for layer_heading in [
            "Layer 1: Domain State & Data Model",
            "Layer 2: Logic & State Management",
            "Layer 3: Presentation & Actuator Interface Binding",
        ] {
            assert!(
                us_md.contains(layer_heading),
                "{} must contain 3-layer Definition of Done section {:?}:\n{}",
                us_name,
                layer_heading,
                us_md
            );
        }
        for ast_binding in [
            "active_epoch_seq",
            "batch_req_in",
            "commit_ack_out",
            "QuorumWatermarkBound",
        ] {
            assert!(
                us_md.contains(ast_binding),
                "{} must reference AST 3-layer member {:?}:\n{}",
                us_name,
                ast_binding,
                us_md
            );
        }
    }

    // 5. US-01 `## Acceptance Criteria (BDD)` cites AST precondition, parameter, transition guard,
    //    target state, constraint, output port, interaction response message, and negative boundary
    //    rejection criterion without partial state mutation.
    let (_, us01_after_ac_heading) = us01
        .split_once("## Acceptance Criteria (BDD)")
        .ok_or("US-01 missing '## Acceptance Criteria (BDD)' section")?;
    let us01_ac_section = if let Some((ac_body, _)) = us01_after_ac_heading.split_once("\n## ") {
        ac_body
    } else {
        us01_after_ac_heading
    };

    for expected_bdd_token in [
        "Quorum lease is active and write-ahead log is open",
        "epoch_id: EpochNumber",
        "quorum_lease_valid == true",
        "EpochCommitted",
        "QuorumWatermarkBound",
        "commit_ack_out",
        "CommitReceiptReady",
    ] {
        assert!(
            us01_ac_section.contains(expected_bdd_token),
            "US-01 '## Acceptance Criteria (BDD)' must cite AST token {:?}:\n{}",
            expected_bdd_token,
            us01_ac_section
        );
    }

    let has_negative_boundary_criterion = us01_ac_section.lines().any(|line| {
        let lower = line.to_ascii_lowercase();
        let cites_guard_or_constraint =
            line.contains("quorum_lease_valid == true") || line.contains("QuorumWatermarkBound");
        let cites_violation = lower.contains("violat")
            || lower.contains("false")
            || lower.contains("fails")
            || lower.contains("unsatisfied")
            || lower.contains("not satisfied");
        let cites_rejection_without_partial_mutation = lower.contains("reject")
            && (lower.contains("without partial state mutation")
                || (lower.contains("without") && lower.contains("state mutation"))
                || lower.contains("no partial state mutation"));
        cites_guard_or_constraint && cites_violation && cites_rejection_without_partial_mutation
    });
    assert!(
        has_negative_boundary_criterion,
        "US-01 '## Acceptance Criteria (BDD)' must include a negative boundary criterion asserting rejection without partial state mutation when guard quorum_lease_valid == true or QuorumWatermarkBound is violated:\n{}",
        us01_ac_section
    );

    // 6. Neither US-01 nor US-02 contains compiler self-description boilerplate.
    for (us_name, us_md) in [
        ("US-01-CommitJournalBatchAction.md", us01.as_str()),
        ("US-02-VerifyReplicaQuorumAction.md", us02.as_str()),
    ] {
        for forbidden_boilerplate in [
            "processes the AST",
            "semantic validator",
            "graceful recovery occurs without data loss",
        ] {
            assert!(
                !us_md.contains(forbidden_boilerplate),
                "{} must contain zero compiler boilerplate substring {:?}:\n{}",
                us_name,
                forbidden_boilerplate,
                us_md
            );
        }
    }

    Ok(())
}

/// Verifies that non-forced forward synchronization (`force: false, dry_run: false`) evicts legacy
/// compiler-boilerplate User Story acceptance criteria and checked `[x]` state on stale boilerplate
/// `AC-US-01-01` while preserving tracker-allocated `Issue ID` (`#419`) and non-boilerplate parent
/// feature checklist state (`- [x] [Feature 01: ...]`), and subsequently preserves genuine custom
/// non-boilerplate `## User Story` prose and checked `- [x] AC-US-01-01:` across re-synchronization.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C idempotent
/// synchronization and stale-artifact remediation objectives for fn
/// `test_issue_696_forward_sync_non_forced_evicts_stale_user_story_boilerplate_preserving_issue_id`
/// in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #696).
///
/// ## 2. Invariants & Mathematical Properties
/// - Stale Boilerplate Eviction Invariant: When an existing `US-01-*.md` on disk lacks
///   `"## User Story"` and carries legacy compiler boilerplate (`"processes the AST"`,
///   `"semantic validator"`, `"graceful recovery occurs without data loss"`), running
///   `forward_sync_sysml_to_specs` with `force: false` evicts 100% of the boilerplate criteria and
///   resets the stale boilerplate `- [x] AC-US-01-01:` checkbox to `- [ ] AC-US-01-01:`, injects
///   the AST-derived `## User Story` (`ClusterAdministrator`), and preserves both
///   `| **Issue ID** | #419 |` and `- [x] [Feature 01: ...]`.
/// - Custom Authoring Preservation Invariant: Once `US-01-*.md` contains non-boilerplate custom
///   `## User Story` prose (`StorageReliabilityOperator`) and a verified `- [x] AC-US-01-01:`
///   checkbox on AST-derived criteria, re-running `forward_sync_sysml_to_specs` with `force: false`
///   preserves `| **Issue ID** | #419 |`, `StorageReliabilityOperator`, and `- [x] AC-US-01-01:`.
///
/// ## 3. Preconditions
/// - An isolated RAII `TempDirGuard` workspace seeds `docs/user-stories/US-01-CommitJournalBatchAction.md`
///   with legacy defective boilerplate and `| **Issue ID** | #419 |`.
///
/// ## 4. Postconditions
/// - Pass 1 (`force: false, dry_run: false`) evicts all boilerplate strings and stale `AC-US-01-01`
///   completion mark while preserving `#419` and `- [x] [Feature 01:` and injecting `## User Story`.
/// - Pass 2 (`force: false, dry_run: false`) preserves `#419`, `StorageReliabilityOperator`, and
///   `- [x] AC-US-01-01:` on disk.
/// - `TempDirGuard` removes temporary directories deterministically on drop.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ per synchronization pass over the fixture AST and Markdown file.
/// - Space Complexity: $O(N)$ bounded auxiliary memory for Markdown serialization buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Exercises `forward_sync_sysml_to_specs` across two stateful `force: false` disk passes.
/// - State: Scoped exclusively to `TempDirGuard::path()` with zero repository state mutation.
/// - Action: Executes with zero fallible unwrapping, panics, regular expressions, or byte-range slicing.
/// - Resource: `TempDirGuard` Drop reclaims ephemeral files even when unwinding on RED failure.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYNC-US-EVICT-STALE-BOILERPLATE], [REQ-SYNC-US-PRESERVE-CUSTOM-EDITS] | SysML v2: VerificationCaseDef(test_issue_696_forward_sync_non_forced_evicts_stale_user_story_boilerplate_preserving_issue_id)
#[test]
fn test_issue_696_forward_sync_non_forced_evicts_stale_user_story_boilerplate_preserving_issue_id(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDirGuard::new("deap_issue_696_evict_boilerplate")?;
    let docs_dir = temp.path().join("docs");
    let user_stories_dir = docs_dir.join("user-stories");
    fs::create_dir_all(&user_stories_dir)?;

    let us01_disk_path = user_stories_dir.join("US-01-CommitJournalBatchAction.md");
    let legacy_boilerplate_us01 = "\
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #419 |
| **Title** | User Story 01: Commit Journal Batch Action |
| **Type** | user-story |
| **Parent Epic** | [EPIC-02-JournalCommitSubsystem](../epics/EPIC-02-JournalCommitSubsystem.md) |
| **SysML Interaction** | `CommitJournalInteraction` |
| **SysML Test Case** | `Test_CommitJournalBatchAction` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/distributed_ledger.sysml](../../schema/distributed_ledger.sysml) |

# User Story 01: Commit Journal Batch Action

## Acceptance Criteria (BDD)
- [x] AC-US-01-01: Given operational environment initialized, When CommitJournalBatchAction executes, Then JournalCoordinatorEngine processes the AST within defined latency envelope.
- [ ] AC-US-01-02: Given formal constraints, When semantic validator executes, Then asserts invariant satisfaction.
- [ ] AC-US-01-03: Given exceptional conditions, When error recovery activates, Then graceful recovery occurs without data loss.

## Required Features
- [x] [Feature 01: [Journal Commit Subsystem] JournalCoordinatorEngine](../features/FEAT-01-JournalCoordinatorEngine.md)
";
    fs::write(&us01_disk_path, legacy_boilerplate_us01)?;

    let pkg = build_issue_696_distributed_ledger_fixture();
    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/distributed_ledger.sysml"),
        docs_dir: docs_dir.clone(),
        out_dir: Some(docs_dir),
        dry_run: false,
        force: false,
    };

    // Pass 1: Non-forced sync must evict legacy boilerplate and stale AC-US-01-01 `[x]` while
    // preserving Issue ID #419 and parent Feature 01 `[x]`.
    forward_sync_sysml_to_specs(&pkg, &opts)?;
    let pass1_us01 = fs::read_to_string(&us01_disk_path)?;

    assert!(
        pass1_us01.contains("| **Issue ID** | #419 |"),
        "Non-forced forward sync must preserve tracker Issue ID #419:\n{}",
        pass1_us01
    );
    assert!(
        pass1_us01.contains("## User Story") && pass1_us01.contains("ClusterAdministrator"),
        "Non-forced forward sync must inject AST-derived '## User Story' referencing 'ClusterAdministrator':\n{}",
        pass1_us01
    );
    for forbidden_boilerplate in [
        "processes the AST",
        "semantic validator",
        "graceful recovery occurs without data loss",
    ] {
        assert!(
            !pass1_us01.contains(forbidden_boilerplate),
            "Non-forced forward sync must completely evict stale compiler boilerplate {:?}:\n{}",
            forbidden_boilerplate,
            pass1_us01
        );
    }
    assert!(
        pass1_us01.contains("- [ ] AC-US-01-01:") && !pass1_us01.contains("- [x] AC-US-01-01:"),
        "Non-forced forward sync must evict stale '[x]' from replaced boilerplate AC-US-01-01:\n{}",
        pass1_us01
    );
    assert!(
        pass1_us01.contains("- [x] [Feature 01:"),
        "Non-forced forward sync must preserve '- [x] [Feature 01:' parent traceability checkbox:\n{}",
        pass1_us01
    );

    // Pass 2: Replace `## User Story` body on disk with custom non-boilerplate prose and mark
    // `- [x] AC-US-01-01:`, then re-run non-forced forward sync and verify preservation.
    let (before_story, after_story_heading) = pass1_us01
        .split_once("## User Story\n")
        .ok_or("Pass 1 US-01 missing '## User Story\\n' heading")?;
    let custom_story_body = "As a `StorageReliabilityOperator`, I want to invoke `CommitJournalBatchAction` under maintenance fencing, so that quorum durability holds.";
    let rebuilt_us01 = if let Some((_, rest_from_next_heading)) =
        after_story_heading.split_once("\n## ")
    {
        format!(
            "{}## User Story\n\n{}\n\n## {}",
            before_story, custom_story_body, rest_from_next_heading
        )
    } else {
        format!("{}## User Story\n\n{}\n", before_story, custom_story_body)
    };
    let custom_us01 = rebuilt_us01.replacen("- [ ] AC-US-01-01:", "- [x] AC-US-01-01:", 1);
    fs::write(&us01_disk_path, &custom_us01)?;

    forward_sync_sysml_to_specs(&pkg, &opts)?;
    let pass2_us01 = fs::read_to_string(&us01_disk_path)?;

    assert!(
        pass2_us01.contains("| **Issue ID** | #419 |"),
        "Second non-forced forward sync must preserve '| **Issue ID** | #419 |':\n{}",
        pass2_us01
    );
    assert!(
        pass2_us01.contains("StorageReliabilityOperator"),
        "Second non-forced forward sync must preserve custom non-boilerplate User Story actor 'StorageReliabilityOperator':\n{}",
        pass2_us01
    );
    assert!(
        pass2_us01.contains("- [x] AC-US-01-01:"),
        "Second non-forced forward sync must preserve checked '- [x] AC-US-01-01:' on non-boilerplate AC:\n{}",
        pass2_us01
    );

    Ok(())
}

/// Extracts the body of a level-2 Markdown heading (`## ...`) up to the next `\n## ` boundary.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic
/// specification verification objectives for fn `extract_markdown_h2_section` in
/// `crates/compile-sysml/tests/reproducer_tests.rs` (refs #694).
///
/// ## 2. Invariants & Mathematical Properties
/// - Bounds-Free Structural Slice Invariant: Partitions `markdown` exclusively via UTF-8 safe
///   `str::split_once` delimiters without direct byte-range indexing or regular expressions,
///   returning the exact substring bounded by `heading` and the subsequent level-2 heading.
///
/// ## 3. Preconditions
/// - `markdown` is a valid UTF-8 Markdown specification buffer and `heading` is a level-2 heading
///   prefix such as `"## 2. Preconditions"`.
///
/// ## 4. Postconditions
/// - Returns `Ok(body)` containing all content under `heading` prior to the next `\n## ` heading,
///   or `Err(diagnostic)` if `heading` is absent from `markdown`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ where $L$ is the byte length of `markdown`.
/// - Space Complexity: $O(1)$ borrowed slice on success; $O(L)$ only when formatting an error message.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Propagates missing-heading diagnostics explicitly via `Result<&'a str, String>`.
/// - State: Pure read-only borrow with zero mutable state.
/// - Action: Executes with zero `.unwrap()`, `.expect()`, `panic!`, `regex`, or direct slice indexing.
/// - Resource: Zero heap allocation on the nominal `Ok` path.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYNC-UC-H2-SECTION-SLICE] | SysML v2: VerificationCaseDef(extract_markdown_h2_section)
fn extract_markdown_h2_section<'a>(markdown: &'a str, heading: &str) -> Result<&'a str, String> {
    let (_before_heading, after_heading) = markdown
        .split_once(heading)
        .ok_or_else(|| format!("Missing heading {:?} in Markdown:\n{}", heading, markdown))?;
    if let Some((section_body, _remaining_sections)) = after_heading.split_once("\n## ") {
        Ok(section_body)
    } else {
        Ok(after_heading)
    }
}

/// Verifies that a Markdown document contains at least one closed ```` ```mermaid ```` diagram fence
/// and that every Mermaid block contains zero backticks (`` ` ``) and zero semicolons (`;`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C diagram hygiene
/// verification objectives for fn `assert_mermaid_blocks_free_of_backticks_and_semicolons` in
/// `crates/compile-sysml/tests/reproducer_tests.rs` (refs #694).
///
/// ## 2. Invariants & Mathematical Properties
/// - Mermaid Lexical Purity Invariant: Every fenced ```` ```mermaid ... ``` ```` block $B$ inside
///   `markdown` is properly closed and satisfies $\text{'`'} \notin B \land \text{';'} \notin B$.
///
/// ## 3. Preconditions
/// - `doc_name` identifies the specification artifact under verification and `markdown` holds its
///   emitted UTF-8 content.
///
/// ## 4. Postconditions
/// - Confirms at least one closed ```` ```mermaid ```` block exists and asserts zero backticks and
///   zero semicolons inside every Mermaid block.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ single linear scan over `markdown` of byte length $L$.
/// - Space Complexity: $O(1)$ auxiliary stack space over borrowed slices.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Returns `Err(String)` if no ```` ```mermaid ```` block exists or if a fence is unclosed.
/// - State: Pure read-only inspection with zero global state.
/// - Action: Executes with zero `.unwrap()`, `.expect()`, `regex`, or direct slice indexing.
/// - Resource: Zero heap allocation on valid Mermaid blocks.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYNC-UC-MERMAID-TOKEN-SANITIZATION] | SysML v2: VerificationCaseDef(assert_mermaid_blocks_free_of_backticks_and_semicolons)
fn assert_mermaid_blocks_free_of_backticks_and_semicolons(
    doc_name: &str,
    markdown: &str,
) -> Result<(), String> {
    let mut block_count: usize = 0;
    for tail in markdown.split("```mermaid").skip(1) {
        block_count += 1;
        let (mermaid_body, _after_close) = tail.split_once("```").ok_or_else(|| {
            format!(
                "{} contains an unclosed ```mermaid fence:\n{}",
                doc_name, markdown
            )
        })?;
        assert!(
            !mermaid_body.contains('`'),
            "{} Mermaid diagram block must contain zero backticks ('`'):\n```mermaid{}```",
            doc_name,
            mermaid_body
        );
        assert!(
            !mermaid_body.contains(';'),
            "{} Mermaid diagram block must contain zero semicolons (';'):\n```mermaid{}```",
            doc_name,
            mermaid_body
        );
    }
    if block_count == 0 {
        return Err(format!(
            "{} must contain at least one ```mermaid diagram block:\n{}",
            doc_name, markdown
        ));
    }
    Ok(())
}

/// Verifies that non-forced forward synchronization (`force: false, dry_run: false`) projects
/// single-line bolded BDD clauses (`- **Given**: ... - **When**: ... - **Then**: ...`) and inline
/// parenthetical diagnostics (`(Diagnostic: E0453)`) from `UseCaseDef.doc` into `UC-01-ForbidAttributeBypass.md`,
/// sanitizes backticks and semicolons from its Mermaid block, preserves tracker-allocated `Issue ID`
/// (`#512`), and completely evicts legacy 5-step hardcoded compiler boilerplate on disk.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C specification
/// projection and stale-boilerplate eviction objectives for fn
/// `test_issue_694_forward_sync_projects_single_line_bold_bdd_and_evicts_stale_use_case_boilerplate`
/// in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #694).
///
/// ## 2. Invariants & Mathematical Properties
/// - Single-Line Bolded BDD Clause Projection Invariant: Given a `UseCaseDef` whose `doc` encodes
///   inline `- **Given**:`, `- **When**:` (with parenthetical ``(`Diagnostic: E0453`)``), and
///   `- **Then**:` clauses on a single line, `forward_sync_sysml_to_specs` deterministically projects
///   the `Given` predicate into `## 2. Preconditions`, the `When` stimulus into `## 3. Trigger` and
///   step `1.` of `## 4. Main Success Scenario`, the `Then` outcome into step `2.` of
///   `## 4. Main Success Scenario` and `## 6. Postconditions`, and `E0453` into
///   `## 5. Alternate and Exception Flows`.
/// - Stale Use Case Boilerplate Eviction & Issue-ID Preservation Invariant: Running with `force: false`
///   over a pre-seeded legacy 5-step boilerplate `UC-01-ForbidAttributeBypass.md` preserves
///   `| **Issue ID** | #512 |` while evicting 100% of `"The compiler execution environment"`,
///   `"submits input payload"`, and `"Downstream resource timeout"`, and emitting zero `` ` `` or `;`
///   characters inside ```` ```mermaid ```` blocks.
///
/// ## 3. Preconditions
/// - An isolated RAII `TempDirGuard` workspace pre-seeds `docs/use-cases/UC-01-ForbidAttributeBypass.md`
///   with `| **Issue ID** | #512 |` and legacy 5-step compiler boilerplate, paired with an in-memory
///   `StaticPurityWorkspace` -> `PuritySubsystem` `PackageDef` containing `ClippyPurityGate` and
///   `UC_AC_01_ForbidAttributeBypass`.
///
/// ## 4. Postconditions
/// - Re-synchronized `docs/use-cases/UC-01-ForbidAttributeBypass.md` preserves `#512`, populates
///   sections 2 through 6 from the single-line bolded BDD `doc`, contains zero backticks or
///   semicolons in its Mermaid block, contains zero legacy boilerplate substrings, and cleans up
///   its temporary directory via `TempDirGuard` on scope exit.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ linear pass over the synthetic `PackageDef` and emitted Markdown file.
/// - Space Complexity: $O(N)$ bounded heap memory for AST and Markdown buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Invokes `forward_sync_sysml_to_specs` with `force: false, dry_run: false`.
/// - State: Scoped strictly to `TempDirGuard::path()` with zero repository mutations.
/// - Action: Executes with zero `.unwrap()`, `.expect()`, `panic!`, `regex`, or direct slice indexing.
/// - Resource: `TempDirGuard` Drop guarantees deterministic cleanup even during RED assertion unwinding.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYNC-UC-SINGLE-LINE-BOLD-BDD], [REQ-SYNC-UC-EVICT-STALE-BOILERPLATE] | SysML v2: VerificationCaseDef(test_issue_694_forward_sync_projects_single_line_bold_bdd_and_evicts_stale_use_case_boilerplate)
#[test]
fn test_issue_694_forward_sync_projects_single_line_bold_bdd_and_evicts_stale_use_case_boilerplate(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDirGuard::new("deap_issue_694_uc_single_line_bdd")?;
    let docs_dir = temp.path().join("docs");
    let use_cases_dir = docs_dir.join("use-cases");
    fs::create_dir_all(&use_cases_dir)?;

    let uc01_disk_path = use_cases_dir.join("UC-01-ForbidAttributeBypass.md");
    let legacy_boilerplate_uc01 = "\
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #512 |
| **Title** | Use Case 01: Forbid Attribute Bypass |
| **Type** | use-case |
| **Parent Epic** | [EPIC-02-PuritySubsystem](../epics/EPIC-02-PuritySubsystem.md) |
| **SysML Use Case** | `UC_AC_01_ForbidAttributeBypass` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/static_purity.sysml](../../schema/static_purity.sysml) |

# Use Case 01: Forbid Attribute Bypass

## 1. Actors
- **Primary Actor**: `Operator`
- **Supporting System**: `ClippyPurityGate`

## 2. Preconditions
- The compiler execution environment is initialized in nominal operational state.
- Authoritative input schema files exist with read access.

## 3. Trigger
- Actor `Operator` initiates `Forbid Attribute Bypass` execution request.

## 4. Main Success Scenario
1. Actor `Operator` submits input payload for `Forbid Attribute Bypass`.
2. `ClippyPurityGate` validates structural schema and boundary preconditions.
3. `ClippyPurityGate` executes core operational transformation.
4. `ClippyPurityGate` verifies post-execution invariants.
5. `ClippyPurityGate` emits deterministic completion acknowledgment to `Operator`.

## 5. Alternate and Exception Flows
- **2a. Boundary validation failure**: `ClippyPurityGate` rejects malformed request without state mutation.
- **5c. Downstream resource timeout or communication fault:** `ClippyPurityGate` aborts transaction and logs diagnostic status.

## 6. Postconditions
- Target state invariants hold and audit receipt is persisted.
";
    fs::write(&uc01_disk_path, legacy_boilerplate_uc01)?;

    let subpkg = PackageDef {
        name: "PuritySubsystem".to_string(),
        part_defs: vec![PartDef {
            name: "ClippyPurityGate".to_string(),
            ..Default::default()
        }],
        use_case_defs: vec![UseCaseDef {
            name: "UC_AC_01_ForbidAttributeBypass".to_string(),
            doc: Some(
                "- **Given**: `[workspace.lints.clippy]` configures forbid rules across all crates. - **When**: A module introduces `#[allow(clippy::unwrap_used)]` (`Diagnostic: E0453`). - **Then**: `cargo clippy --workspace --all-targets` fails closed with non-zero exit status."
                    .to_string(),
            ),
            ..Default::default()
        }],
        ..Default::default()
    };
    let pkg = PackageDef {
        name: "StaticPurityWorkspace".to_string(),
        packages: vec![subpkg],
        ..Default::default()
    };

    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/static_purity.sysml"),
        docs_dir: docs_dir.clone(),
        out_dir: Some(docs_dir),
        dry_run: false,
        force: false,
    };

    forward_sync_sysml_to_specs(&pkg, &opts)?;
    let uc01_md = fs::read_to_string(&uc01_disk_path)?;

    // 1. Preserve allocated Issue ID #512.
    assert!(
        uc01_md.contains("| **Issue ID** | #512 |"),
        "UC-01 must preserve tracker Issue ID #512 across non-forced boilerplate eviction:\n{}",
        uc01_md
    );

    // 2. ## 2. Preconditions contains Given clause tokens.
    let preconditions = extract_markdown_h2_section(&uc01_md, "## 2. Preconditions")?;
    assert!(
        preconditions.contains("[workspace.lints.clippy]")
            && preconditions.contains("configures forbid rules across all crates"),
        "UC-01 '## 2. Preconditions' must contain '[workspace.lints.clippy]' and 'configures forbid rules across all crates':\n{}",
        preconditions
    );

    // 3. ## 3. Trigger contains When stimulus token.
    let trigger = extract_markdown_h2_section(&uc01_md, "## 3. Trigger")?;
    assert!(
        trigger.contains("#[allow(clippy::unwrap_used)]"),
        "UC-01 '## 3. Trigger' must contain '#[allow(clippy::unwrap_used)]':\n{}",
        trigger
    );

    // 4. ## 4. Main Success Scenario contains step 1 (When) and step 2 (Then).
    let main_scenario = extract_markdown_h2_section(&uc01_md, "## 4. Main Success Scenario")?;
    assert!(
        main_scenario
            .lines()
            .any(|line| line.trim_start().starts_with("1. ")
                && line.contains("#[allow(clippy::unwrap_used)]")),
        "UC-01 '## 4. Main Success Scenario' step '1. ' must contain '#[allow(clippy::unwrap_used)]':\n{}",
        main_scenario
    );
    assert!(
        main_scenario
            .lines()
            .any(|line| line.trim_start().starts_with("2. ")
                && line.contains("cargo clippy --workspace --all-targets")),
        "UC-01 '## 4. Main Success Scenario' step '2. ' must contain 'cargo clippy --workspace --all-targets':\n{}",
        main_scenario
    );

    // 5. ## 5. Alternate and Exception Flows contains inline parenthetical Diagnostic E0453.
    let alt_flows = extract_markdown_h2_section(&uc01_md, "## 5. Alternate and Exception Flows")?;
    assert!(
        alt_flows.contains("E0453"),
        "UC-01 '## 5. Alternate and Exception Flows' must contain extracted diagnostic code 'E0453':\n{}",
        alt_flows
    );

    // 6. ## 6. Postconditions contains Then outcome.
    let postconditions = extract_markdown_h2_section(&uc01_md, "## 6. Postconditions")?;
    assert!(
        postconditions.contains("fails closed with non-zero exit status"),
        "UC-01 '## 6. Postconditions' must contain 'fails closed with non-zero exit status':\n{}",
        postconditions
    );

    // 7. Mermaid block contains zero backticks and zero semicolons.
    assert_mermaid_blocks_free_of_backticks_and_semicolons(
        "UC-01-ForbidAttributeBypass.md",
        &uc01_md,
    )?;

    // 8. Entire file contains zero legacy hardcoded boilerplate occurrences.
    for forbidden_boilerplate in [
        "The compiler execution environment",
        "submits input payload",
        "Downstream resource timeout",
    ] {
        assert!(
            !uc01_md.contains(forbidden_boilerplate),
            "UC-01-ForbidAttributeBypass.md must contain zero legacy boilerplate substring {:?}:\n{}",
            forbidden_boilerplate,
            uc01_md
        );
    }

    Ok(())
}

/// Verifies multi-line unbolded/bulleted `UseCaseDef.doc` clause and actor extraction, package-local
/// actor inheritance into sibling `ActionDef` User Stories via `extract_first_uc_actor`, Mermaid
/// semicolon stripping on SysML statement syntax, and Format-2 trailing diagnostic-only (`Diagnostic: E0602).`)
/// plus `objective` fallback projection in `forward_sync_sysml_to_specs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic
/// model-to-specification traceability objectives for fn
/// `test_issue_694_multiline_bdd_actor_extraction_and_diagnostic_only_use_case_projection`
/// in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #694).
///
/// ## 2. Invariants & Mathematical Properties
/// - Multi-Line Clause & Package Actor Inheritance Invariant: For a subpackage containing
///   `UC_01_VerifyAuditDigest` with multi-line unbolded/bulleted `Actor:`, `Given:`, `When:` (carrying
///   a semicolon-terminated statement `first S1 accept trig then S2;`), `Then:`, and `Diagnostic:`
///   clauses alongside `ActionDef` `VerifyAuditDigestAction`, forward synchronization resolves
///   `"AuditComplianceOfficer"` in `UC-01` `## 1. Actors`, propagates `"AuditComplianceOfficer"` to
///   `US-01-VerifyAuditDigestAction.md` via `extract_first_uc_actor`, maps `Given`/`When`/`Then`/`Diagnostic`
///   across sections 2 through 6 of `UC-01`, and strips `;` from `UC-01`'s Mermaid diagram block.
/// - Diagnostic Punctuation Normalization & Objective Fallback Invariant: For `UC_02_EnforceSchemaParity`
///   carrying `objective: Some("Deterministic Dual Schema Digest Parity Enforcement")` and Format-2
///   trailing diagnostic-only `doc: Some("Diagnostic: E0602).")`, forward synchronization cleans
///   `"E0602)."` to `"E0602"` in `## 5. Alternate and Exception Flows`, incorporates the `objective`
///   text into its scenario/trigger, and emits zero legacy compiler boilerplate strings.
///
/// ## 3. Preconditions
/// - An in-memory `LedgerAuditSystem` -> `AuditGateSubsystem` `PackageDef` containing
///   `PartDef` `"AuditVerifierEngine"`, `ActionDef` `"VerifyAuditDigestAction"`,
///   `UseCaseDef` `"UC_01_VerifyAuditDigest"`, and `UseCaseDef` `"UC_02_EnforceSchemaParity"` is
///   synchronized into an isolated RAII `TempDirGuard` workspace with `force: true, dry_run: false`.
///
/// ## 4. Postconditions
/// - `UC-01-VerifyAuditDigest.md`, `US-01-VerifyAuditDigestAction.md`, and
///   `UC-02-EnforceSchemaParity.md` satisfy all actor, BDD section mapping, Mermaid sanitization,
///   diagnostic punctuation stripping, objective projection, and zero-boilerplate assertions, and
///   `TempDirGuard` removes temporary files on drop.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ linear pass over the synthetic `PackageDef` and generated Markdown artifacts.
/// - Space Complexity: $O(N)$ bounded heap memory for AST fixtures and Markdown buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Invokes `forward_sync_sysml_to_specs` with `force: true, dry_run: false`.
/// - State: Isolated to `TempDirGuard::path()` with zero shared global state.
/// - Action: Executes with zero `.unwrap()`, `.expect()`, `panic!`, `regex`, or direct slice indexing.
/// - Resource: `TempDirGuard` Drop reclaims ephemeral filesystem state even on RED assertion failure.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYNC-UC-MULTILINE-ACTOR-INHERITANCE], [REQ-SYNC-UC-DIAGNOSTIC-ONLY-OBJECTIVE] | SysML v2: VerificationCaseDef(test_issue_694_multiline_bdd_actor_extraction_and_diagnostic_only_use_case_projection)
#[test]
fn test_issue_694_multiline_bdd_actor_extraction_and_diagnostic_only_use_case_projection(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDirGuard::new("deap_issue_694_uc_multiline_and_diag")?;
    let docs_dir = temp.path().join("docs");
    fs::create_dir_all(&docs_dir)?;

    let subpkg = PackageDef {
        name: "AuditGateSubsystem".to_string(),
        part_defs: vec![PartDef {
            name: "AuditVerifierEngine".to_string(),
            ..Default::default()
        }],
        action_defs: vec![ActionDef {
            name: "VerifyAuditDigestAction".to_string(),
            ..Default::default()
        }],
        use_case_defs: vec![
            UseCaseDef {
                name: "UC_01_VerifyAuditDigest".to_string(),
                doc: Some(
                    "- Actor: AuditComplianceOfficer\n\
                     - Given: Audit journal digest manifest is loaded on disk\n\
                     - When: AuditComplianceOfficer invokes VerifyAuditDigestAction with statement first S1 accept trig then S2;\n\
                     - Then: AuditVerifierEngine confirms digest parity across all committed records\n\
                     - Diagnostic: E0601"
                        .to_string(),
                ),
                ..Default::default()
            },
            UseCaseDef {
                name: "UC_02_EnforceSchemaParity".to_string(),
                objective: Some("Deterministic Dual Schema Digest Parity Enforcement".to_string()),
                doc: Some("Diagnostic: E0602).".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let pkg = PackageDef {
        name: "LedgerAuditSystem".to_string(),
        packages: vec![subpkg],
        ..Default::default()
    };

    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/ledger_audit.sysml"),
        docs_dir: docs_dir.clone(),
        out_dir: Some(docs_dir.clone()),
        dry_run: false,
        force: true,
    };

    forward_sync_sysml_to_specs(&pkg, &opts)?;

    let uc01_md = fs::read_to_string(docs_dir.join("use-cases/UC-01-VerifyAuditDigest.md"))?;
    let us01_md =
        fs::read_to_string(docs_dir.join("user-stories/US-01-VerifyAuditDigestAction.md"))?;
    let uc02_md = fs::read_to_string(docs_dir.join("use-cases/UC-02-EnforceSchemaParity.md"))?;

    // 1. UC-01-VerifyAuditDigest.md resolves actor, maps Given/When/Then/Diagnostic, and strips ';' from Mermaid.
    let uc01_actors = extract_markdown_h2_section(&uc01_md, "## 1. Actors")?;
    assert!(
        uc01_actors.contains("AuditComplianceOfficer"),
        "UC-01 '## 1. Actors' must resolve actor 'AuditComplianceOfficer':\n{}",
        uc01_actors
    );

    let uc01_preconditions = extract_markdown_h2_section(&uc01_md, "## 2. Preconditions")?;
    assert!(
        uc01_preconditions.contains("Audit journal digest manifest is loaded on disk"),
        "UC-01 '## 2. Preconditions' must project Given clause:\n{}",
        uc01_preconditions
    );

    let uc01_trigger = extract_markdown_h2_section(&uc01_md, "## 3. Trigger")?;
    assert!(
        uc01_trigger.contains(
            "AuditComplianceOfficer invokes VerifyAuditDigestAction with statement first S1 accept trig then S2"
        ),
        "UC-01 '## 3. Trigger' must project When clause:\n{}",
        uc01_trigger
    );

    let uc01_mss = extract_markdown_h2_section(&uc01_md, "## 4. Main Success Scenario")?;
    assert!(
        uc01_mss.contains(
            "AuditComplianceOfficer invokes VerifyAuditDigestAction with statement first S1 accept trig then S2"
        ),
        "UC-01 '## 4. Main Success Scenario' must project When clause:\n{}",
        uc01_mss
    );
    assert!(
        uc01_mss.contains(
            "AuditVerifierEngine confirms digest parity across all committed records"
        ),
        "UC-01 '## 4. Main Success Scenario' must project Then clause:\n{}",
        uc01_mss
    );

    let uc01_alt = extract_markdown_h2_section(&uc01_md, "## 5. Alternate and Exception Flows")?;
    assert!(
        uc01_alt.contains("E0601"),
        "UC-01 '## 5. Alternate and Exception Flows' must project 'Diagnostic: E0601':\n{}",
        uc01_alt
    );

    let uc01_post = extract_markdown_h2_section(&uc01_md, "## 6. Postconditions")?;
    assert!(
        uc01_post.contains(
            "AuditVerifierEngine confirms digest parity across all committed records"
        ),
        "UC-01 '## 6. Postconditions' must project Then clause:\n{}",
        uc01_post
    );

    assert_mermaid_blocks_free_of_backticks_and_semicolons(
        "UC-01-VerifyAuditDigest.md",
        &uc01_md,
    )?;

    // 2. US-01-VerifyAuditDigestAction.md inherits package-local actor "AuditComplianceOfficer" via extract_first_uc_actor.
    let us01_story = extract_markdown_h2_section(&us01_md, "## User Story")?;
    assert!(
        us01_story.contains("AuditComplianceOfficer"),
        "US-01-VerifyAuditDigestAction.md '## User Story' must inherit package-local actor 'AuditComplianceOfficer' via extract_first_uc_actor:\n{}",
        us01_story
    );

    // 3. UC-02-EnforceSchemaParity.md cleans "E0602)." to "E0602" in ## 5. Alternate and Exception Flows,
    //    incorporates objective in its scenario/trigger, and contains zero legacy boilerplate.
    let uc02_alt = extract_markdown_h2_section(&uc02_md, "## 5. Alternate and Exception Flows")?;
    assert!(
        uc02_alt.contains("E0602")
            && !uc02_alt.contains("E0602).")
            && !uc02_alt.contains("E0602)"),
        "UC-02 '## 5. Alternate and Exception Flows' must clean 'E0602).' to 'E0602':\n{}",
        uc02_alt
    );

    let uc02_trigger = extract_markdown_h2_section(&uc02_md, "## 3. Trigger")?;
    let uc02_mss = extract_markdown_h2_section(&uc02_md, "## 4. Main Success Scenario")?;
    assert!(
        uc02_trigger.contains("Deterministic Dual Schema Digest Parity Enforcement")
            || uc02_mss.contains("Deterministic Dual Schema Digest Parity Enforcement"),
        "UC-02 must incorporate objective 'Deterministic Dual Schema Digest Parity Enforcement' in its scenario/trigger:\nTrigger:\n{}\nMain Success Scenario:\n{}",
        uc02_trigger,
        uc02_mss
    );

    assert_mermaid_blocks_free_of_backticks_and_semicolons(
        "UC-02-EnforceSchemaParity.md",
        &uc02_md,
    )?;

    for (uc_name, uc_content) in [
        ("UC-01-VerifyAuditDigest.md", uc01_md.as_str()),
        ("UC-02-EnforceSchemaParity.md", uc02_md.as_str()),
    ] {
        for forbidden_boilerplate in [
            "The compiler execution environment",
            "submits input payload",
            "Downstream resource timeout",
        ] {
            assert!(
                !uc_content.contains(forbidden_boilerplate),
                "{} must contain zero legacy boilerplate substring {:?}:\n{}",
                uc_name,
                forbidden_boilerplate,
                uc_content
            );
        }
    }

    Ok(())
}

/// Constructs a domain-neutral distributed ledger governance `PackageDef` (`LedgerCoreSystem` ->
/// `LedgerGovernance_SysML`) exercising acronym preservation, >8 constraint projection, AST-grounded
/// Feature/Epic synthesis, and stale boilerplate merge eviction for GitHub Issue #699.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic
/// test fixture construction objectives for fn `build_issue_699_ledger_governance_fixture` in
/// `crates/compile-sysml/tests/reproducer_tests.rs` (refs #699).
///
/// ## 2. Invariants & Mathematical Properties
/// - Domain-Neutrality Invariant: Every package, component, attribute, port, state, constraint,
///   action, use case, and requirement belongs exclusively to distributed ledger snapshot governance
///   and cryptographic digest verification with zero hardcoded domain-specific tokens.
/// - Cardinality & Acronym Coverage Invariant: Populates 9 inline `PartDef` constraints
///   (`C01_Bound`..`C09_NinthInvariant`) plus 1 satisfied `RequirementDef` constraint
///   (`C10_TenthReqInvariant` on `REQ-0699`) to prove zero `.truncate(8)` loss, alongside mixed-case
///   technical acronyms (`SysML`, `CommonMark`, `GitHub`, `OpenAPI`, `UUIDv5`, `STPA`, `FMECA`, `SHA256`).
///
/// ## 3. Preconditions
/// - Pure in-memory AST construction with zero external filesystem or environment prerequisites.
///
/// ## 4. Postconditions
/// - Returns an owned `PackageDef` (`LedgerCoreSystem`) containing subpackage `LedgerGovernance_SysML`
///   with `PartDef` `SnapshotIntegrityValidator`, 2 `ActionDef`s, 1 `UseCaseDef`, and 1 `RequirementDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ bounded construction of fixed-cardinality AST nodes.
/// - Space Complexity: $O(1)$ bounded heap allocations owned by the returned `PackageDef`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Total pure constructor returning `PackageDef`.
/// - State: Fresh owned allocations with zero shared mutable state.
/// - Action: Executes with zero `.unwrap()`, `.expect()`, `panic!`, `regex`, or direct slice indexing.
/// - Resource: Bounded heap memory reclaimed when the caller drops `PackageDef`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYNC-ISSUE-699-LEDGER-FIXTURE] | SysML v2: VerificationCaseDef(build_issue_699_ledger_governance_fixture)
fn build_issue_699_ledger_governance_fixture() -> PackageDef {
    let part_constraints = [
        ("C01_Bound", "c01_val >= 0"),
        ("C02_EpochFloor", "c02_epoch >= 1"),
        ("C03_ReplicaCount", "c03_replicas >= 3"),
        ("C04_DigestLen", "c04_bytes == 32"),
        ("C05_SkewLimit", "c05_skew_ms <= 50"),
        ("C06_RetryCap", "c06_retries <= 4"),
        ("C07_BatchQuota", "c07_items <= 1024"),
        ("C08_QuorumRatio", "c08_ratio > 0"),
        ("C09_NinthInvariant", "c09_margin > 1"),
    ]
    .into_iter()
    .map(|(name, expression)| ConstraintDef {
        name: name.to_string(),
        expression: expression.to_string(),
        ..Default::default()
    })
    .collect::<Vec<_>>();

    let validator = PartDef {
        name: "SnapshotIntegrityValidator".to_string(),
        attributes: vec![
            AttributeDef {
                name: "checkpoint_digest".to_string(),
                type_name: "CheckpointDigestVector".to_string(),
                default_value: Some("sha256-zero".to_string()),
                doc: None,
            },
            AttributeDef {
                name: "mode_label".to_string(),
                type_name: "String".to_string(),
                default_value: None,
                doc: None,
            },
        ],
        ports: vec![
            PortDef {
                name: "snapshot_in".to_string(),
                direction: "in".to_string(),
                type_name: "SnapshotEnvelope".to_string(),
                ..Default::default()
            },
            PortDef {
                name: "fault_out".to_string(),
                direction: "out".to_string(),
                type_name: "IntegrityDiagnostic".to_string(),
                ..Default::default()
            },
        ],
        constraints: part_constraints,
        states: vec![StateDef {
            name: "ValidatorLifecycleState".to_string(),
            transitions: vec![TransitionDef {
                source: Some("Idle".to_string()),
                target: Some("Verifying".to_string()),
                trigger: Some("CompileSysMLToCommonMark".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };

    let subpkg = PackageDef {
        name: "LedgerGovernance_SysML".to_string(),
        doc: Some("Authoritative distributed ledger replication governance subsystem.".to_string()),
        part_defs: vec![validator],
        action_defs: vec![
            ActionDef {
                name: "CompileSysMLToCommonMark".to_string(),
                ..Default::default()
            },
            ActionDef {
                name: "SyncGitHubOpenAPIWithUUIDv5".to_string(),
                ..Default::default()
            },
        ],
        use_case_defs: vec![UseCaseDef {
            name: "UC_01_VerifySTPAAndFMECAWithSHA256".to_string(),
            ..Default::default()
        }],
        requirement_defs: vec![RequirementDef {
            name: "IntegrityAuditRequirement".to_string(),
            req_id: "REQ-0699".to_string(),
            text: "Verify deterministic ledger snapshot digest and invariant satisfaction."
                .to_string(),
            satisfied_by: vec!["SnapshotIntegrityValidator".to_string()],
            constraints: vec![ConstraintDef {
                name: "C10_TenthReqInvariant".to_string(),
                expression: "c10_integrity == true".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };

    PackageDef {
        name: "LedgerCoreSystem".to_string(),
        packages: vec![subpkg],
        ..Default::default()
    }
}

/// Extracts the body of a Markdown heading line containing `title_keyword` up to the next heading
/// of equal or shallower ATX `#` depth, falling back to `fallback_h2_prefix` when `title_keyword`
/// appears inside the body of a numbered level-2 section rather than on an ATX heading line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C structural
/// specification verification objectives for fn `extract_markdown_section_by_keyword` in
/// `crates/compile-sysml/tests/reproducer_tests.rs` (refs #699).
///
/// ## 2. Invariants & Mathematical Properties
/// - Safe Delimiter Partitioning Invariant: Slices `markdown` strictly via `lines()` and UTF-8 safe
///   `str::split_once` delimiters with zero direct byte-range indexing and zero regular expressions.
///
/// ## 3. Preconditions
/// - `markdown` is a valid UTF-8 Markdown document, `title_keyword` is a non-empty section title
///   token, and `fallback_h2_prefix` is a level-2 heading prefix such as `"## 1."` or `"## 2."`.
///
/// ## 4. Postconditions
/// - Returns `Ok(section_slice)` bounded to the matching section, or `Err(String)` if neither an
///   ATX heading containing `title_keyword` nor `fallback_h2_prefix` containing `title_keyword` exists.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ linear scan over `markdown` of byte length $L$.
/// - Space Complexity: $O(1)$ borrowed slice on the nominal `Ok` path.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Returns descriptive `Err(String)` diagnostics when the requested section is absent.
/// - State: Pure read-only borrow with zero shared mutable state.
/// - Action: Executes with zero `.unwrap()`, `.expect()`, `panic!`, `regex`, or direct slice indexing.
/// - Resource: Zero heap allocation on success.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYNC-ISSUE-699-SECTION-SLICE] | SysML v2: VerificationCaseDef(extract_markdown_section_by_keyword)
fn extract_markdown_section_by_keyword<'a>(
    markdown: &'a str,
    title_keyword: &str,
    fallback_h2_prefix: &str,
) -> Result<&'a str, String> {
    if let Some(heading_line) = markdown.lines().find(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with('#') && trimmed.contains(title_keyword)
    }) {
        let heading_level = heading_line
            .trim_start()
            .chars()
            .take_while(|&ch| ch == '#')
            .count();
        let (_before, after_heading) = markdown.split_once(heading_line).ok_or_else(|| {
            format!(
                "Failed to partition Markdown at heading {:?} in:\n{}",
                heading_line, markdown
            )
        })?;
        let next_boundary = after_heading.lines().skip(1).find(|line| {
            let trimmed = line.trim_start();
            let hashes = trimmed.chars().take_while(|&ch| ch == '#').count();
            hashes > 0 && hashes <= heading_level && trimmed.chars().nth(hashes) == Some(' ')
        });
        if let Some(boundary_line) = next_boundary {
            if let Some((section_body, _rest)) = after_heading.split_once(boundary_line) {
                return Ok(section_body);
            }
        }
        return Ok(after_heading);
    }

    let h2_body = extract_markdown_h2_section(markdown, fallback_h2_prefix)?;
    if h2_body.contains(title_keyword) {
        Ok(h2_body)
    } else {
        Err(format!(
            "Section {:?} did not contain expected keyword {:?}:\n{}",
            fallback_h2_prefix, title_keyword, h2_body
        ))
    }
}

/// Verifies that `forward_sync_sysml_to_specs` preserves technical acronyms and mixed-case tokens
/// in generated titles, eliminates CSS container / compiler self-description boilerplate from
/// `FEAT-01-SnapshotIntegrityValidator.md` and `EPIC-02-LedgerGovernance_SysML.md`, projects all
/// 10 constraints without `.truncate(8)` data loss alongside `3..=10` positive and negative BDD
/// acceptance criteria, grounds Epic sections 1, 3, 4, and 5 in AST declarations, and evicts stale
/// boilerplate during non-forced (`force: false`) re-synchronization while preserving tracker
/// `Issue ID` (`#699`), checked `- [x] AC-01:`, and custom human-authored Layer 3 prose.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C model-driven
/// specification integrity objectives for fn
/// `test_issue_699_forward_sync_ast_grounded_features_epics_acronyms_and_merge_eviction`
/// in `crates/compile-sysml/tests/reproducer_tests.rs` (refs #699).
///
/// ## 2. Invariants & Mathematical Properties
/// - Technical Acronym Tokenization Invariant: Humanized Epic, User Story, and Use Case titles
///   preserve `"SysML"`, `"CommonMark"`, `"GitHub"`, `"OpenAPI"`, `"UUIDv5"`, `"STPA"`, `"FMECA"`,
///   and `"SHA256"` intact and never fracture them into `"Sys ML"`, `"Common Mark"`, `"Git Hub"`,
///   or `"Open API"`.
/// - Zero-Boilerplate & Zero-Loss Feature Projection Invariant: `FEAT-01-SnapshotIntegrityValidator.md`
///   emits zero occurrences of `"@container"`, `"viewport containment"`, `"AST references"`, or
///   `"operational_invariants"`; preserves `"CheckpointDigestVector"` and `"sha256-zero"` in Section 1
///   (`Test Data Shape` JSON Schema); projects `"CheckpointDigestVector"`, `"snapshot_in"`,
///   `"fault_out"`, `"REQ-0699"`, and all 10 constraints and expressions (`"C01_Bound"` through
///   `"C09_NinthInvariant"` and `"C10_TenthReqInvariant"`) in Section 2 (`Validation & Constraints`);
///   and emits `3..=10` `- [ ] AC-` BDD items referencing all 10 constraints with both positive
///   satisfaction and negative violation/rejection criteria.
/// - AST-Grounded Epic Synthesis Invariant: `EPIC-02-LedgerGovernance_SysML.md` emits zero
///   `"Deterministic compilation passes, error containment"` or `"zero hardcoded domain semantics"`
///   boilerplate, grounding `## 1. Context` in `PackageDef.doc`, `## 3. Architecture` in
///   `"SnapshotIntegrityValidator"` and `"snapshot_in"`, `## 4. Operational Considerations` in
///   `"ValidatorLifecycleState"` (`"Idle"` / `"Verifying"`), and `## 5. Security & Governance` in
///   `"REQ-0699"` and `"C09_NinthInvariant"` / `"C10_TenthReqInvariant"`.
/// - Selective Stale-Boilerplate Merge Eviction Invariant: Under `force: false, dry_run: false`,
///   stale `"@container"`, `"viewport containment"`, `"AST references"`, and
///   `"Deterministic compilation passes"` sections on disk are evicted and replaced with AST-derived
///   sections while `"| **Issue ID** | #699 |"`, `"- [x] AC-01:"`, and custom non-boilerplate
///   `### Layer 3: Presentation & Actuator Interface Binding` prose survive intact.
///
/// ## 3. Preconditions
/// - `build_issue_699_ledger_governance_fixture()` constructs the domain-neutral `LedgerCoreSystem`
///   AST and two isolated RAII `TempDirGuard` workspaces are writable under `std::env::temp_dir()`.
///
/// ## 4. Postconditions
/// - Both forced fresh-generation assertions and non-forced merge-eviction/preservation assertions
///   hold, and both `TempDirGuard` instances deterministically clean up temporary files on drop.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ linear pass over the synthetic `PackageDef` and emitted Markdown buffers.
/// - Space Complexity: $O(N)$ bounded heap memory for AST fixtures and Markdown strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Exercises `forward_sync_sysml_to_specs` under both `force: true` and `force: false`.
/// - State: Isolated strictly to two RAII `TempDirGuard` directories with zero repository mutations.
/// - Action: Executes with zero `.unwrap()`, `.expect()`, `panic!`, `regex`, or direct slice indexing.
/// - Resource: `TempDirGuard` Drop guarantees deterministic cleanup even during RED assertion unwinding.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYNC-FEAT-EPIC-AST-GROUNDED-699], [REQ-SYNC-ACRONYM-PRESERVATION-699], [REQ-SYNC-MERGE-BOILERPLATE-EVICTION-699] | SysML v2: VerificationCaseDef(test_issue_699_forward_sync_ast_grounded_features_epics_acronyms_and_merge_eviction)
#[test]
fn test_issue_699_forward_sync_ast_grounded_features_epics_acronyms_and_merge_eviction(
) -> Result<(), Box<dyn std::error::Error>> {
    let pkg = build_issue_699_ledger_governance_fixture();

    // Pass A: Forced fresh synchronization into first RAII TempDirGuard.
    let temp_fresh = TempDirGuard::new("deap_issue_699_fresh_sync")?;
    let docs_fresh = temp_fresh.path().join("docs");
    fs::create_dir_all(&docs_fresh)?;

    let opts_fresh = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/ledger_governance.sysml"),
        docs_dir: docs_fresh.clone(),
        out_dir: Some(docs_fresh.clone()),
        dry_run: false,
        force: true,
    };
    let generated = forward_sync_sysml_to_specs(&pkg, &opts_fresh)?;

    let epic_md = generated
        .get("epics/EPIC-02-LedgerGovernance_SysML.md")
        .ok_or("Missing epics/EPIC-02-LedgerGovernance_SysML.md")?;
    let feat_md = generated
        .get("features/FEAT-01-SnapshotIntegrityValidator.md")
        .ok_or("Missing features/FEAT-01-SnapshotIntegrityValidator.md")?;
    let us01_md = generated
        .get("user-stories/US-01-CompileSysMLToCommonMark.md")
        .ok_or("Missing user-stories/US-01-CompileSysMLToCommonMark.md")?;
    let us02_md = generated
        .get("user-stories/US-02-SyncGitHubOpenAPIWithUUIDv5.md")
        .ok_or("Missing user-stories/US-02-SyncGitHubOpenAPIWithUUIDv5.md")?;
    let uc01_md = generated
        .get("use-cases/UC-01-VerifySTPAAndFMECAWithSHA256.md")
        .ok_or("Missing use-cases/UC-01-VerifySTPAAndFMECAWithSHA256.md")?;

    // 1. Acronym & Mixed-Case Preservation across Epic / User Story / Use Case titles.
    for (doc_name, doc_md, expected_acronyms) in [
        (
            "EPIC-02-LedgerGovernance_SysML.md",
            epic_md.as_str(),
            ["SysML"].as_slice(),
        ),
        (
            "US-01-CompileSysMLToCommonMark.md",
            us01_md.as_str(),
            ["SysML", "CommonMark"].as_slice(),
        ),
        (
            "US-02-SyncGitHubOpenAPIWithUUIDv5.md",
            us02_md.as_str(),
            ["GitHub", "OpenAPI", "UUIDv5"].as_slice(),
        ),
        (
            "UC-01-VerifySTPAAndFMECAWithSHA256.md",
            uc01_md.as_str(),
            ["STPA", "FMECA", "SHA256"].as_slice(),
        ),
    ] {
        for forbidden_split in ["Sys ML", "Common Mark", "Git Hub", "Open API"] {
            assert!(
                !doc_md.contains(forbidden_split),
                "{} must never fracture acronym into {:?}:\n{}",
                doc_name,
                forbidden_split,
                doc_md
            );
        }
        let h1_title = doc_md
            .lines()
            .find(|line| line.starts_with("# "))
            .ok_or_else(|| format!("{} missing '# ' H1 title line:\n{}", doc_name, doc_md))?;
        let meta_title = doc_md
            .lines()
            .find(|line| line.starts_with("| **Title** |"))
            .ok_or_else(|| format!("{} missing '| **Title** |' row:\n{}", doc_name, doc_md))?;
        for acronym in expected_acronyms {
            let in_h1 = h1_title
                .split(|ch: char| !ch.is_alphanumeric())
                .any(|tok| tok == *acronym);
            let in_meta = meta_title
                .split(|ch: char| !ch.is_alphanumeric())
                .any(|tok| tok == *acronym);
            assert!(
                in_h1 && in_meta,
                "{} titles must preserve standalone acronym token {:?} intact (H1: {:?}, Meta: {:?})",
                doc_name,
                acronym,
                h1_title,
                meta_title
            );
        }
    }

    // 2. FEAT-01-SnapshotIntegrityValidator.md Purity, Zero-Loss Section 2 (>8 Constraints), & 3..=10 BDD.
    for forbidden_feat_boilerplate in [
        "@container",
        "viewport containment",
        "AST references",
        "operational_invariants",
    ] {
        assert!(
            !feat_md.contains(forbidden_feat_boilerplate),
            "FEAT-01 must contain zero boilerplate substring {:?}:\n{}",
            forbidden_feat_boilerplate,
            feat_md
        );
    }

    let feat_sec1 = extract_markdown_section_by_keyword(feat_md, "Test Data Shape", "## 1.")?;
    for expected_sec1_token in ["CheckpointDigestVector", "sha256-zero"] {
        assert!(
            feat_sec1.contains(expected_sec1_token),
            "FEAT-01 Section 1 (Test Data Shape JSON Schema) must preserve {:?}:\n{}",
            expected_sec1_token,
            feat_sec1
        );
    }

    let feat_sec2 =
        extract_markdown_section_by_keyword(feat_md, "Validation & Constraints", "## 2.")?;
    let all_ten_constraints = [
        ("C01_Bound", "c01_val >= 0"),
        ("C02_EpochFloor", "c02_epoch >= 1"),
        ("C03_ReplicaCount", "c03_replicas >= 3"),
        ("C04_DigestLen", "c04_bytes == 32"),
        ("C05_SkewLimit", "c05_skew_ms <= 50"),
        ("C06_RetryCap", "c06_retries <= 4"),
        ("C07_BatchQuota", "c07_items <= 1024"),
        ("C08_QuorumRatio", "c08_ratio > 0"),
        ("C09_NinthInvariant", "c09_margin > 1"),
        ("C10_TenthReqInvariant", "c10_integrity == true"),
    ];
    for expected_sec2_member in [
        "CheckpointDigestVector",
        "snapshot_in",
        "fault_out",
        "REQ-0699",
    ] {
        assert!(
            feat_sec2.contains(expected_sec2_member),
            "FEAT-01 Section 2 (Validation & Constraints) must project {:?}:\n{}",
            expected_sec2_member,
            feat_sec2
        );
    }
    for (constraint_name, constraint_expr) in all_ten_constraints {
        assert!(
            feat_sec2.contains(constraint_name) && feat_sec2.contains(constraint_expr),
            "FEAT-01 Section 2 (Validation & Constraints) must project constraint {:?} and expression {:?} without .truncate(8) loss:\n{}",
            constraint_name,
            constraint_expr,
            feat_sec2
        );
    }

    let feat_bdd = extract_markdown_h2_section(feat_md, "## Acceptance Criteria (BDD)")?;
    let ac_lines = feat_bdd
        .lines()
        .filter(|line| line.trim_start().starts_with("- [ ] AC-"))
        .collect::<Vec<_>>();
    assert!(
        (3..=10).contains(&ac_lines.len()),
        "FEAT-01 '## Acceptance Criteria (BDD)' must contain strictly 3..=10 '- [ ] AC-' items, got {}:\n{}",
        ac_lines.len(),
        feat_bdd
    );
    for (constraint_name, _constraint_expr) in all_ten_constraints {
        assert!(
            feat_bdd.contains(constraint_name),
            "FEAT-01 '## Acceptance Criteria (BDD)' must reference constraint {:?}:\n{}",
            constraint_name,
            feat_bdd
        );
    }
    let has_positive_bdd = ac_lines.iter().any(|line| {
        let lower = line.to_ascii_lowercase();
        let has_gwt =
            lower.contains("given") && lower.contains("when") && lower.contains("then");
        let is_positive = (lower.contains("satisf")
            || lower.contains("valid")
            || lower.contains("nominal")
            || lower.contains("holds")
            || lower.contains("conform")
            || lower.contains("verif"))
            && !lower.contains("violat");
        has_gwt && is_positive
    });
    let has_negative_bdd = ac_lines.iter().any(|line| {
        let lower = line.to_ascii_lowercase();
        let has_gwt =
            lower.contains("given") && lower.contains("when") && lower.contains("then");
        let cites_violation = lower.contains("violat")
            || lower.contains("invalid")
            || lower.contains("malformed")
            || lower.contains("unsatisfied")
            || lower.contains("fails");
        let cites_rejection = lower.contains("reject")
            || lower.contains("fault_out")
            || lower.contains("diagnostic")
            || lower.contains("abort")
            || lower.contains("refus")
            || lower.contains("fail");
        has_gwt && cites_violation && cites_rejection
    });
    assert!(
        has_positive_bdd && has_negative_bdd,
        "FEAT-01 '## Acceptance Criteria (BDD)' must include both positive satisfaction and negative violation/rejection Given/When/Then criteria:\n{}",
        feat_bdd
    );

    // 3. EPIC-02-LedgerGovernance_SysML.md AST-Grounded Sections 1, 3, 4, 5.
    for forbidden_epic_boilerplate in [
        "Deterministic compilation passes, error containment",
        "zero hardcoded domain semantics",
    ] {
        assert!(
            !epic_md.contains(forbidden_epic_boilerplate),
            "EPIC-02 must contain zero compiler boilerplate substring {:?}:\n{}",
            forbidden_epic_boilerplate,
            epic_md
        );
    }

    let epic_sec1 = extract_markdown_h2_section(epic_md, "## 1. Context")?;
    assert!(
        epic_sec1.contains("Authoritative distributed ledger replication governance subsystem."),
        "EPIC-02 '## 1. Context' must project PackageDef.doc:\n{}",
        epic_sec1
    );

    let epic_sec3 = extract_markdown_h2_section(epic_md, "## 3. Architecture")?;
    assert!(
        epic_sec3.contains("SnapshotIntegrityValidator") && epic_sec3.contains("snapshot_in"),
        "EPIC-02 '## 3. Architecture' must reference 'SnapshotIntegrityValidator' and 'snapshot_in':\n{}",
        epic_sec3
    );

    let epic_sec4 = extract_markdown_h2_section(epic_md, "## 4. Operational Considerations")?;
    assert!(
        epic_sec4.contains("ValidatorLifecycleState")
            || epic_sec4.contains("Idle")
            || epic_sec4.contains("Verifying"),
        "EPIC-02 '## 4. Operational Considerations' must reference AST lifecycle state 'ValidatorLifecycleState' ('Idle' / 'Verifying'):\n{}",
        epic_sec4
    );

    let epic_sec5 = extract_markdown_h2_section(epic_md, "## 5. Security & Governance")?;
    assert!(
        epic_sec5.contains("REQ-0699")
            && (epic_sec5.contains("C09_NinthInvariant")
                || epic_sec5.contains("C10_TenthReqInvariant")),
        "EPIC-02 '## 5. Security & Governance' must reference 'REQ-0699' and 'C09_NinthInvariant' (or 'C10_TenthReqInvariant'):\n{}",
        epic_sec5
    );

    // 4. Non-Forced (force: false) Stale Boilerplate Eviction + Issue ID & Custom Prose Preservation.
    let temp_merge = TempDirGuard::new("deap_issue_699_non_forced_merge")?;
    let docs_merge = temp_merge.path().join("docs");
    let features_merge_dir = docs_merge.join("features");
    let epics_merge_dir = docs_merge.join("epics");
    fs::create_dir_all(&features_merge_dir)?;
    fs::create_dir_all(&epics_merge_dir)?;

    let feat_merge_path = features_merge_dir.join("FEAT-01-SnapshotIntegrityValidator.md");
    let epic_merge_path = epics_merge_dir.join("EPIC-02-LedgerGovernance_SysML.md");

    let seeded_stale_feat = "\
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #699 |
| **Title** | Feature 01: [Ledger Governance Sys ML] SnapshotIntegrityValidator |
| **Type** | feature |
| **Parent Epic** | [EPIC-02-LedgerGovernance_SysML](../epics/EPIC-02-LedgerGovernance_SysML.md) |
| **Interface Type** | Physical / Structural / Internal Bus |
| **Schema Containers** | `LedgerGovernance_SysML::SnapshotIntegrityValidator` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/ledger_governance.sysml](../../schema/ledger_governance.sysml) |

# Feature 01: [Ledger Governance Sys ML] SnapshotIntegrityValidator

## Interface Requirements
### 1. Test Data Shape
```json
{
  \"type\": \"object\",
  \"properties\": {
    \"operational_invariants\": { \"type\": \"string\" }
  }
}
```

### 2. Validation & Constraints
Formal constraints and invariant guarantees enforced by SnapshotIntegrityValidator.

### 3. Visual Layout & Arrangement
Logical layout, viewport containment, and container structure for SnapshotIntegrityValidator conforming to platform design tokens. Container boundaries enforce @container isolation rules.

### 4. Interactive Flow & States
Operational lifecycle events and discrete state transitions for SnapshotIntegrityValidator.

### Layer 1: Domain State & Signal Model
Domain state vector representing SnapshotIntegrityValidator status, AST references, and typed signal models.

### Layer 2: Logic & Safety State Management
Deterministic state machine solver and safety monitor logic for SnapshotIntegrityValidator.

### Layer 3: Presentation & Actuator Interface Binding
Custom authored quorum transport diagnostic routing narrative.

## Acceptance Criteria (BDD)
- [x] AC-01: Given valid ledger snapshot input, When SnapshotIntegrityValidator evaluates C01_Bound, Then verification succeeds.
- [ ] AC-02: Given operational state, When SnapshotIntegrityValidator evaluates constraint 'C02_EpochFloor', Then asserts invariant satisfaction without violation.

## Source References
Formal component specifications and normative systems engineering requirements:
- System Architecture Model: `schema/ledger_governance.sysml`
";
    fs::write(&feat_merge_path, seeded_stale_feat)?;

    let seeded_stale_epic = "\
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #699 |
| **Title** | Epic 02: Ledger Governance Sys ML |
| **Type** | epic |
| **Package** | LedgerGovernance_SysML |
| **Subsystem** | Ledger Governance Sys ML |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/ledger_governance.sysml](../../schema/ledger_governance.sysml) |

# Epic 02: Ledger Governance Sys ML

## 1. Context
Authoritative distributed ledger replication governance subsystem.

## 2. Requirements & Checklist
- [ ] #699 - [Feature 01: [Ledger Governance Sys ML] SnapshotIntegrityValidator](../features/FEAT-01-SnapshotIntegrityValidator.md)

## 3. Architecture
Subsystem structural composition, port allocations, and directional data connectors realized by SnapshotIntegrityValidator.

## 4. Operational Considerations
Deterministic compilation passes, error containment, and provable polynomial complexity execution.

## 5. Security & Governance
Safety-critical invariant satisfaction, formal trace matrix closure, and zero hardcoded domain semantics.

## 6. Source References
- System Architecture Model: `schema/ledger_governance.sysml`
";
    fs::write(&epic_merge_path, seeded_stale_epic)?;

    let opts_merge = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/ledger_governance.sysml"),
        docs_dir: docs_merge.clone(),
        out_dir: Some(docs_merge),
        dry_run: false,
        force: false,
    };
    forward_sync_sysml_to_specs(&pkg, &opts_merge)?;

    let merged_feat = fs::read_to_string(&feat_merge_path)?;
    let merged_epic = fs::read_to_string(&epic_merge_path)?;

    // Issue ID #699, checked `- [x] AC-01:`, and custom Layer 3 narrative must survive.
    assert!(
        merged_feat.contains("| **Issue ID** | #699 |"),
        "Non-forced FEAT-01 sync must preserve '| **Issue ID** | #699 |':\n{}",
        merged_feat
    );
    assert!(
        merged_epic.contains("| **Issue ID** | #699 |"),
        "Non-forced EPIC-02 sync must preserve '| **Issue ID** | #699 |':\n{}",
        merged_epic
    );
    assert!(
        merged_feat.contains("- [x] AC-01:"),
        "Non-forced FEAT-01 sync must preserve checked '- [x] AC-01:':\n{}",
        merged_feat
    );
    assert!(
        merged_feat.contains("Custom authored quorum transport diagnostic routing narrative."),
        "Non-forced FEAT-01 sync must preserve custom non-boilerplate Layer 3 prose:\n{}",
        merged_feat
    );

    // Stale boilerplate must be completely evicted (0 occurrences) and replaced by AST-derived sections.
    for evicted_token in [
        "@container",
        "viewport containment",
        "AST references",
        "Deterministic compilation passes",
    ] {
        assert!(
            !merged_feat.contains(evicted_token),
            "Non-forced FEAT-01 sync must evict stale boilerplate {:?}:\n{}",
            evicted_token,
            merged_feat
        );
        assert!(
            !merged_epic.contains(evicted_token),
            "Non-forced EPIC-02 sync must evict stale boilerplate {:?}:\n{}",
            evicted_token,
            merged_epic
        );
    }

    assert!(
        merged_feat.contains("C09_NinthInvariant")
            && merged_feat.contains("C10_TenthReqInvariant")
            && merged_feat.contains("CheckpointDigestVector"),
        "Non-forced FEAT-01 sync must replace evicted boilerplate with AST-derived constraints and types:\n{}",
        merged_feat
    );
    assert!(
        merged_epic.contains("Authoritative distributed ledger replication governance subsystem.")
            && merged_epic.contains("SnapshotIntegrityValidator")
            && merged_epic.contains("REQ-0699"),
        "Non-forced EPIC-02 sync must replace evicted boilerplate with AST-derived sections:\n{}",
        merged_epic
    );

    Ok(())
}
