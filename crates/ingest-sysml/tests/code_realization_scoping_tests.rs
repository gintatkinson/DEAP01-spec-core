//! Integration tests for Issue #472: Deterministic realization package naming and architectural
//! `part def` scoping during Rust (`src/`) and Dart/Flutter (`lib/`) source code ingestion.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C software
//! architecture extraction and traceability verification rules. Verifies that ingesting
//! `<component>/src` or `<component>/lib` produces parent-qualified `<Parent>_<src|lib>_Realization`
//! package names instead of raw `package src` / `package lib`, supports `--package <NAME>`
//! overrides, scopes code `part def` extraction when combined with `--schema <DOMAIN_SCHEMA>`,
//! and preserves Check 17 Pillar 7 FMECA bidirectional closure across realization schemas.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Ingesting `<parent>/src` or `<parent>/lib` without `--package` never emits `package src`
//!   or `package lib`.
//! - Combining `--source <DIR>` with `--schema <DOMAIN_SCHEMA>` retains only `PartDef` items
//!   matching declared architectural `part def` components or carrying explicit `Realises:`
//!   traceability bindings.
//! - Check 17 (`check_safety_integrity`) Pillar 7 FMECA closure evaluates architectural domain
//!   `part def` declarations without failing on realization-only code structs in `*_realization.sysml`.
//!
//! ## 3. Preconditions
//! - Temporary test workspaces are created in the OS temporary directory and cleaned up on drop.
//!
//! ## 4. Postconditions
//! - Asserts deterministic package naming, realization `PartDef` scoping, and Check 17 Pillar 7
//!   FMECA closure compliance.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N \log N)$ bounded by temporary fixture size $N$.
//! - Space Complexity: $O(N)$ for parsed SysML v2 AST structures and temporary test files.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Temporary directories are unconditionally removed in `Drop` even if an assertion panics.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-ISSUE-472-REALIZATION-SCOPING-TESTS]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use compile_sysml::docstring::run_extract_source;
use compile_sysml::parse_sysml;
use deap_core::source_ast::{ingest_source_directory, SourceLanguage};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use verify_baseline::checks::safety_stpa::{check_safety_integrity, SafetyCheckOutcome};

/// RAII temporary workspace guard for Issue #472 realization scoping tests.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Ensures isolated filesystem fixtures in `std::env::temp_dir()` for deterministic testing
/// without polluting the repository workspace.
///
/// ## 2. Invariants & Mathematical Properties
/// - Each instance receives a unique directory path combining process ID, nanosecond timestamp,
///   and an atomic counter.
///
/// ## 3. Preconditions
/// - OS temporary directory is writable.
///
/// ## 4. Postconditions
/// - `self.root` exists on disk during test execution and is recursively deleted on drop.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ creation, $O(F)$ cleanup for $F$ created files.
/// - Space Complexity: $O(1)$ heap path allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Ignores cleanup errors on drop to prevent double-panic abort during unwinding.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ISSUE-472-TEMP-DIR-GUARD]
struct ScopedTempDir {
    root: PathBuf,
}

impl ScopedTempDir {
    /// Creates a new isolated temporary workspace directory with the given `label`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Provides deterministic test isolation for filesystem-backed CLI and AST verification tests.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Monotonically incrementing atomic counter prevents collisions across parallel test threads.
    ///
    /// ## 3. Preconditions
    /// - `label` is a valid path segment string.
    ///
    /// ## 4. Postconditions
    /// - Returns a `ScopedTempDir` whose `root` directory exists on disk.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$ directory creation.
    /// - Space Complexity: $O(1)$ `PathBuf` allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Panics with a diagnostic message if the OS temporary directory is unwritable.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-ISSUE-472-TEMP-DIR-NEW]
    fn new(label: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!(
            "deap_issue472_{}_{}_{}_{}",
            label,
            std::process::id(),
            nanos,
            seq
        ));
        fs::create_dir_all(&root).expect("create temp dir");
        Self { root }
    }
}

impl Drop for ScopedTempDir {
    /// Recursively removes the temporary workspace directory when the guard leaves scope.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Enforces zero leftover temporary test artifacts on the host filesystem.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Unconditionally invokes `fs::remove_dir_all(&self.root)`.
    ///
    /// ## 3. Preconditions
    /// - `self` is being dropped.
    ///
    /// ## 4. Postconditions
    /// - Temporary directory `self.root` is removed from disk.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(F)$ where $F$ is the number of files created inside `self.root`.
    /// - Space Complexity: $O(1)$ auxiliary stack space.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Swallows `io::Error` during drop so test failure panics are never masked.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-ISSUE-472-TEMP-DIR-DROP]
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Verifies that ingesting `<component>/src` and `<component>/lib` derives parent-qualified
/// `<Parent>_<src|lib>_Realization` package names instead of raw `package src` / `package lib`,
/// and that `--package <NAME>` explicitly overrides the target package identifier.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Prevents root namespace pollution (`package src`, `package lib`) across multi-component
/// Rust and Flutter workspaces during Ground 0.0 source ingestion and docstring extraction.
///
/// ## 2. Invariants & Mathematical Properties
/// - `crates/solver/src` -> `solver_src_Realization`
/// - `app_flutter/lib` -> `app_flutter_lib_Realization`
/// - `--package CustomSolverPkg` -> `CustomSolverPkg`
///
/// ## 3. Preconditions
/// - `CARGO_BIN_EXE_ingest-sysml` binary is built and accessible.
///
/// ## 4. Postconditions
/// - Emitted SysML v2 packages match expected parent-qualified realization names or explicit overrides.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ over fixture source files.
/// - Space Complexity: $O(N)$ for parsed `PackageDef` AST.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Fails with explicit diagnostic output if raw `package src` or `package lib` is emitted.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ISSUE-472-PACKAGE-QUALIFICATION]
#[test]
fn test_issue_472_parent_qualified_realization_package_naming_and_override() {
    let ws = ScopedTempDir::new("pkg_naming");
    let solver_src = ws.root.join("crates").join("solver").join("src");
    let flutter_lib = ws.root.join("app_flutter").join("lib");
    let schema_dir = ws.root.join("schema");
    fs::create_dir_all(&solver_src).expect("create crates/solver/src");
    fs::create_dir_all(&flutter_lib).expect("create app_flutter/lib");
    fs::create_dir_all(&schema_dir).expect("create schema/");

    fs::write(
        solver_src.join("lib.rs"),
        "/// Realises: [REQ-SOLVER-01]\npub struct TacticalSolver {\n    pub horizon_sec: u32,\n}\n",
    )
    .expect("write solver lib.rs");

    fs::write(
        flutter_lib.join("cockpit_hud.dart"),
        "/// Realises: [REQ-HUD-01]\nclass CockpitHud {\n  final String mode;\n  CockpitHud(this.mode);\n}\n",
    )
    .expect("write cockpit_hud.dart");

    let bin = env!("CARGO_BIN_EXE_ingest-sysml");

    // 1. Ingest crates/solver/src without --package -> solver_src_Realization
    let solver_out = schema_dir.join("solver_realization.sysml");
    let solver_digest = schema_dir.join("solver_digest.json");
    let status_solver = Command::new(bin)
        .arg("--source")
        .arg(&solver_src)
        .arg("--out")
        .arg(&solver_out)
        .arg("--digest")
        .arg(&solver_digest)
        .output()
        .expect("run ingest-sysml on crates/solver/src");
    assert!(
        status_solver.status.success(),
        "ingest-sysml failed on crates/solver/src: {}",
        String::from_utf8_lossy(&status_solver.stderr)
    );
    let solver_sysml = fs::read_to_string(&solver_out).expect("read solver_realization.sysml");
    let solver_pkg = parse_sysml(&solver_sysml).expect("parse solver_realization.sysml");
    assert_eq!(
        solver_pkg.name, "solver_src_Realization",
        "Expected parent-qualified 'solver_src_Realization' instead of raw 'src', got:\n{}",
        solver_sysml
    );
    assert!(
        !solver_sysml.contains("package src "),
        "Emitted SysML must never contain unscoped 'package src'"
    );

    // 2. Ingest app_flutter/lib without --package -> app_flutter_lib_Realization
    let cockpit_out = schema_dir.join("cockpit_realization.sysml");
    let cockpit_digest = schema_dir.join("cockpit_digest.json");
    let status_cockpit = Command::new(bin)
        .arg("--source")
        .arg(&flutter_lib)
        .arg("--out")
        .arg(&cockpit_out)
        .arg("--digest")
        .arg(&cockpit_digest)
        .output()
        .expect("run ingest-sysml on app_flutter/lib");
    assert!(
        status_cockpit.status.success(),
        "ingest-sysml failed on app_flutter/lib: {}",
        String::from_utf8_lossy(&status_cockpit.stderr)
    );
    let cockpit_sysml = fs::read_to_string(&cockpit_out).expect("read cockpit_realization.sysml");
    let cockpit_pkg = parse_sysml(&cockpit_sysml).expect("parse cockpit_realization.sysml");
    assert_eq!(
        cockpit_pkg.name, "app_flutter_lib_Realization",
        "Expected parent-qualified 'app_flutter_lib_Realization' instead of raw 'lib', got:\n{}",
        cockpit_sysml
    );
    assert!(
        !cockpit_sysml.contains("package lib "),
        "Emitted SysML must never contain unscoped 'package lib'"
    );

    // 3. Explicit --package CustomSolverPkg overrides default derivation
    let custom_out = schema_dir.join("custom_solver.sysml");
    let custom_digest = schema_dir.join("custom_digest.json");
    let status_custom = Command::new(bin)
        .arg("--source")
        .arg(&solver_src)
        .arg("--package")
        .arg("CustomSolverPkg")
        .arg("--out")
        .arg(&custom_out)
        .arg("--digest")
        .arg(&custom_digest)
        .output()
        .expect("run ingest-sysml with --package CustomSolverPkg");
    assert!(
        status_custom.status.success(),
        "ingest-sysml --package failed: {}",
        String::from_utf8_lossy(&status_custom.stderr)
    );
    let custom_sysml = fs::read_to_string(&custom_out).expect("read custom_solver.sysml");
    let custom_pkg = parse_sysml(&custom_sysml).expect("parse custom_solver.sysml");
    assert_eq!(custom_pkg.name, "CustomSolverPkg");

    // 4. Verify run_extract_source and ingest_source_directory also qualify raw "src" / "lib"
    let extracted_pkg = run_extract_source(
        &solver_src,
        &schema_dir.join("extract.sysml"),
        &schema_dir.join("extract_digest.json"),
        true,
    )
    .expect("run_extract_source on crates/solver/src");
    assert_eq!(extracted_pkg.name, "solver_src_Realization");

    let direct_pkg = ingest_source_directory(&flutter_lib, SourceLanguage::Dart, "lib")
        .expect("ingest_source_directory with raw 'lib'");
    assert_eq!(direct_pkg.name, "app_flutter_lib_Realization");
}

/// Verifies that combining `--source <DIR>` with `--schema <DOMAIN_SCHEMA>` (without manual
/// `--allowed-parts`) automatically scopes extracted code `PartDef` items to architectural
/// `part def` components declared in `--schema` or source items carrying explicit `Realises:`
/// requirement bindings, pruning unannotated internal UI/helper structs.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Prevents 140+ internal implementation helper structs, DTOs, and private widget state classes
/// from polluting architectural `part def` namespaces when extracting code realization models.
///
/// ## 2. Invariants & Mathematical Properties
/// - Retains `PartDef` $P$ iff $P.\text{name} \in \text{SchemaParts} \lor P.\text{satisfied\_requirements} \neq \emptyset$.
/// - Prunes unannotated internal helper structs and enums not present in the reference domain schema.
///
/// ## 3. Preconditions
/// - Domain schema and multi-item source tree exist in the temporary workspace.
///
/// ## 4. Postconditions
/// - Output realization package contains `TacticalSolver` and `RealizedTrackFilter`, and excludes
///   `InternalScratchBuffer` and `PrivateUiHelperEnum`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N)$ in source and schema file size.
/// - Space Complexity: $O(P)$ where $P$ is the number of AST part definitions.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Fails closed if `--source` and `--schema` are rejected as mutually exclusive or if unannotated
///   helper types leak into `pkg.part_defs`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ISSUE-472-SCHEMA-SCOPED-CODE-INGESTION]
#[test]
fn test_issue_472_source_with_reference_schema_scopes_to_architectural_and_realized_parts() {
    let ws = ScopedTempDir::new("schema_scoping");
    let solver_src = ws.root.join("crates").join("solver").join("src");
    let schema_dir = ws.root.join("schema");
    fs::create_dir_all(&solver_src).expect("create crates/solver/src");
    fs::create_dir_all(&schema_dir).expect("create schema/");

    let domain_schema_path = schema_dir.join("domain.sysml");
    let domain_sysml = r#"package TacticalDomain {
    requirement def REQ_SOLVER_01 {
        doc /* Real-time track correlation */
    }
    part def TacticalSolver;
    part def CockpitHud;
}
"#;
    fs::write(&domain_schema_path, domain_sysml).expect("write schema/domain.sysml");

    // Source file contains:
    // 1. TacticalSolver (matches domain schema part def)
    // 2. RealizedTrackFilter (carries explicit Realises: [REQ-SOLVER-01])
    // 3. InternalScratchBuffer (unannotated internal helper struct -> must be pruned)
    // 4. PrivateUiHelperEnum (unannotated internal helper enum -> must be pruned from part_defs)
    let rust_source = r#"pub struct TacticalSolver {
    pub cycle_ms: u32,
}

/// Realises: [REQ-SOLVER-01]
pub struct RealizedTrackFilter {
    pub alpha_gain: f64,
}

pub struct InternalScratchBuffer {
    pub capacity: usize,
}

pub enum PrivateUiHelperEnum {
    Hidden,
    Visible,
}
"#;
    fs::write(solver_src.join("lib.rs"), rust_source).expect("write crates/solver/src/lib.rs");

    let out_sysml_path = schema_dir.join("solver_realization.sysml");
    let out_digest_path = schema_dir.join("solver_digest.json");

    let bin = env!("CARGO_BIN_EXE_ingest-sysml");
    let output = Command::new(bin)
        .arg("--source")
        .arg(&solver_src)
        .arg("--schema")
        .arg(&domain_schema_path)
        .arg("--out")
        .arg(&out_sysml_path)
        .arg("--digest")
        .arg(&out_digest_path)
        .output()
        .expect("run ingest-sysml with --source and --schema");

    assert!(
        output.status.success(),
        "Expected ingest-sysml --source <DIR> --schema <SCHEMA> to succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let generated = fs::read_to_string(&out_sysml_path).expect("read solver_realization.sysml");
    let pkg = parse_sysml(&generated).expect("parse generated realization SysML");
    let part_names: Vec<&str> = pkg.part_defs.iter().map(|p| p.name.as_str()).collect();

    assert!(
        part_names.contains(&"TacticalSolver"),
        "Expected architectural part 'TacticalSolver' to be retained, got: {:?}",
        part_names
    );
    assert!(
        part_names.contains(&"RealizedTrackFilter"),
        "Expected realized part 'RealizedTrackFilter' to be retained, got: {:?}",
        part_names
    );
    assert!(
        !part_names.contains(&"InternalScratchBuffer"),
        "Expected unannotated helper 'InternalScratchBuffer' to be pruned, got: {:?}",
        part_names
    );
    assert!(
        !part_names.contains(&"PrivateUiHelperEnum"),
        "Expected unannotated helper enum 'PrivateUiHelperEnum' to be pruned from part_defs, got: {:?}",
        part_names
    );
}

/// Verifies that Check 17 (`check_safety_integrity`) Pillar 7 FMECA bidirectional AST closure
/// passes when `schema/` contains both an architectural domain model (`schema/domain.sysml`)
/// and code realization schema files (`schema/solver_realization.sysml` / `*_Realization` packages),
/// without failing on realization-only code structs.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Ensures system-level STPA/FMECA verification enforces 100% coverage of architectural
/// `part def` components while excluding implementation-level realization schemas from
/// requiring redundant system hazard rows for internal code structs.
///
/// ## 2. Invariants & Mathematical Properties
/// - `discover_architectural_sysml_model_text` excludes `*_realization.sysml` files (and
///   `*_Realization` packages) when domain `.sysml` files are present in `schema/`.
/// - All architectural `part def` components in `schema/domain.sysml` remain strictly enforced
///   by Check 17 Pillar 7 FMECA closure.
///
/// ## 3. Preconditions
/// - Temporary workspace contains `schema/domain.sysml`, `schema/solver_realization.sysml`,
///   and a complete 8-pillar `docs/safety/STPA_MATRIX.md` covering the domain components.
///
/// ## 4. Postconditions
/// - `check_safety_integrity` returns `Ok(SafetyCheckOutcome::DownstreamVerified)`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(M + R)$ where $M$ is schema text length and $R$ is FMECA row count.
/// - Space Complexity: $O(M + R)$ for parsed AST part definitions and FMECA rows.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Fails closed if realization-only structs trigger false-positive Pillar 7 missing-component errors.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ISSUE-472-CHECK17-REALIZATION-CLOSURE]
#[test]
fn test_issue_472_check17_fmeca_closure_ignores_realization_only_code_parts() {
    let ws = ScopedTempDir::new("check17_realization");
    let schema_dir = ws.root.join("schema");
    let safety_dir = ws.root.join("docs").join("safety");
    fs::create_dir_all(&schema_dir).expect("create schema/");
    fs::create_dir_all(&safety_dir).expect("create docs/safety/");

    // Domain model declares two architectural components: TacticalSolver and CockpitHud
    let domain_model = r#"package TacticalDomain {
    part def TacticalSolver;
    part def CockpitHud;
}
"#;
    fs::write(schema_dir.join("domain.sysml"), domain_model).expect("write schema/domain.sysml");

    // Code realization schema in schema/ contains implementation-level structs alongside TacticalSolver
    let realization_model = r#"package solver_src_Realization {
    part def TacticalSolver;
    part def InternalDtoHelper;
    part def WidgetStateHelper;
}
"#;
    fs::write(
        schema_dir.join("solver_realization.sysml"),
        realization_model,
    )
    .expect("write schema/solver_realization.sysml");

    let mut stpa = String::from("# System Safety & STPA Matrix\n\n");
    stpa.push_str("## 1. System Losses ($L-1..N$)\n- Loss L-1: Loss of Tactical Picture\n\n");
    stpa.push_str("## 2. System Hazards ($H-1..N$)\n- Hazard H-1: Stale Track Solution\n\n");
    stpa.push_str("## 3. Control Structure Topology\n```mermaid\ngraph TD\n  TacticalSolver --> CockpitHud\n```\n\n");
    stpa.push_str("## 4. Unsafe Control Actions ($UCA-1..N$)\n- UCA-1: Not providing track update causes hazard\n- UCA-2: Providing corrupted track update causes hazard\n- UCA-3: Providing update too late or out of order\n- UCA-4: Alert stopped too soon or applied too long\n\n");
    stpa.push_str("## 5. Loss Scenarios ($LS-1..N$)\n- Scenario LS-1: Solver queue saturation delays track update\n\n");
    stpa.push_str("## 6. Formal Safety Constraints ($SC-1..N$)\n- SC-1: Solver shall bound track fusion latency under 50ms\n\n");
    stpa.push_str("## 7. FMECA Criticality Matrix\n\n");
    stpa.push_str("| FMECA ID | Component | Failure Mode | Potential Effect | Severity (S) | Occurrence (O) | Detection (D) | RPN (S x O x D) | Mitigation | Derivation Basis |\n");
    stpa.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for i in 1..=8 {
        let dim = match i % 4 {
            0 => "Interface Γ",
            1 => "State Φ",
            2 => "Action Ω",
            _ => "Resource Ψ",
        };
        stpa.push_str(&format!(
            "| FMECA-{:03} | TacticalSolver | Solver Mode {} ({}) | Degraded track | 5 | 2 | 2 | 20 | Watchdog | SSOT |\n",
            i, i, dim
        ));
    }
    for i in 9..=16 {
        let dim = match i % 4 {
            0 => "Interface Γ",
            1 => "State Φ",
            2 => "Action Ω",
            _ => "Resource Ψ",
        };
        stpa.push_str(&format!(
            "| FMECA-{:03} | CockpitHud | Hud Mode {} ({}) | Stale display | 4 | 2 | 2 | 16 | Redraw | Derived |\n",
            i, i, dim
        ));
    }
    stpa.push_str("\n## 8. Domain Regulatory Objectives & Integrity Mapping\n- Pillar 8 DO-178C DAL A and SORA SAIL II objectives\n\n");
    stpa.push_str("## Commercial Toolchain Hooks\n- MATLAB / Simulink / Stateflow verification hooks\n");

    fs::write(safety_dir.join("STPA_MATRIX.md"), stpa).expect("write STPA_MATRIX.md");

    let res = check_safety_integrity(&ws.root, false, true);
    assert_eq!(
        res,
        Ok(SafetyCheckOutcome::DownstreamVerified),
        "Expected Check 17 to pass when domain components are covered and realization-only structs are in *_realization.sysml, got: {:?}",
        res
    );
}
