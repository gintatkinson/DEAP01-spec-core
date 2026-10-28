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
    ActionDef, AttributeDef, CalcDef, ConstraintDef, OperationDef, PackageDef, PartDef, PortDef,
    StateDef, UseCaseDef,
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
