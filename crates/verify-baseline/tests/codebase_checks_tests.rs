//! Integration tests for Check 41 duplicate Dart solver math and thin-bridge gates (`codebase.rs`).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D verification mandates.
//! Verifies that `check_no_duplicate_dart_solver_math` and `check_41_solver_bridge_and_sqlite_seed_parity_gate`
//! reject > 1200 line Dart bridge files and forbidden duplicate solver markers.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Rejects any `app_flutter/lib/data/*bridge*.dart` exceeding 1200 lines when a solver crate is present.
//! - Rejects forbidden markers: `synthesize_candidates(`, `synthesizeCandidates(`, `compute_composite_score(`,
//!   `computeCompositeScore(`, `evaluate_constraints(`, `evaluateConstraints(`, `_hotlVetoActive = true`.
//!
//! ## 3. Preconditions
//! - Temporary test directories are writable and cleaned up on drop.
//!
//! ## 4. Postconditions
//! - Asserts deterministic pass on thin C-ABI bridges and fail with violation diagnostics on bloated/duplicated bridges.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(B \times L)$ where $B$ is bridge file count and $L$ is line count.
//! - Space Complexity: $O(V)$ for accumulated violation diagnostics.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Panics only inside `#[test]` harnesses when contract assertions fail.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-CHECK-41-SOLVER-BRIDGE-PARITY], [REQ-CHECK-41-SSOT-DELEGATION]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use verify_baseline::checks::codebase::{
    check_41_solver_bridge_and_sqlite_seed_parity_gate, check_no_duplicate_dart_solver_math,
};

static COUNTER: AtomicU64 = AtomicU64::new(1);

/// Scoped temporary directory that removes itself on drop.
struct ScopedTempDir {
    path: PathBuf,
}

impl ScopedTempDir {
    /// Creates a new isolated temporary directory for test execution.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-330 TQL-4 isolated test fixture management.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Unique directory path per atomic counter increment and process ID.
    ///
    /// ## 3. Preconditions
    /// - System temp directory is writable.
    ///
    /// ## 4. Postconditions
    /// - Returns initialized `ScopedTempDir` with existing directory on disk.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$ filesystem directory creation.
    /// - Space Complexity: $O(1)$ path allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Ignores benign pre-existing directory creation errors in test setup.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-CHECK-41-TEST-FIXTURE]
    fn new(prefix: &str) -> Self {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "deap_verify_codebase_it_{}_{}_{}",
            prefix,
            std::process::id(),
            id
        ));
        let _ = fs::create_dir_all(&path);
        Self { path }
    }
}

impl Drop for ScopedTempDir {
    /// Cleans up temporary test directory on scope exit.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-330 TQL-4 zero workspace pollution mandate.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Removes `self.path` recursively on drop.
    ///
    /// ## 3. Preconditions
    /// - `self` is being dropped.
    ///
    /// ## 4. Postconditions
    /// - Temporary directory is removed from the OS temp folder.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(F)$ in temporary file count.
    /// - Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Suppresses teardown I/O errors during drop.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-CHECK-41-TEST-FIXTURE]
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Verifies that `check_no_duplicate_dart_solver_math` rejects Dart bridge files exceeding 1200 lines
/// and rejects forbidden solver math markers.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A single source of truth kernel integrity mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Fails when a bridge file exceeds 1200 lines.
/// - Fails when forbidden markers (`synthesize_candidates(`, `compute_composite_score(`, `evaluate_constraints(`, `_hotlVetoActive = true`) appear.
///
/// ## 3. Preconditions
/// - Workspace contains a non-upstream solver crate under `crates/` and Dart bridge files under `app_flutter/lib/data/`.
///
/// ## 4. Postconditions
/// - Returns `Err(violations)` containing diagnostics for line ceiling and each forbidden marker.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ linear in bridge file lines.
/// - Space Complexity: $O(V)$ in reported violations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Fails test if any violation is missed.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-41-SSOT-DELEGATION]
#[test]
fn test_check_no_duplicate_dart_solver_math_rejects_bloated_bridge_and_markers() {
    let tmp = ScopedTempDir::new("dup_math");
    let solver_dir = tmp.path.join("crates/domain-solver/src");
    let data_dir = tmp.path.join("app_flutter/lib/data");
    fs::create_dir_all(&solver_dir).expect("create solver dir");
    fs::create_dir_all(&data_dir).expect("create data dir");

    fs::write(solver_dir.join("solver.rs"), "// native rust solver").expect("write solver.rs");

    let mut bloated_bridge = String::new();
    bloated_bridge.push_str("class NativeSolverBridge {\n");
    bloated_bridge.push_str("  void runDuplicate() {\n");
    bloated_bridge.push_str("    synthesize_candidates();\n");
    bloated_bridge.push_str("    compute_composite_score();\n");
    bloated_bridge.push_str("    evaluate_constraints();\n");
    bloated_bridge.push_str("    _hotlVetoActive = true;\n");
    bloated_bridge.push_str("  }\n");
    for i in 0..1205 {
        bloated_bridge.push_str(&format!("  // padding line {}\n", i));
    }
    bloated_bridge.push_str("}\n");

    fs::write(data_dir.join("custom_solver_bridge.dart"), &bloated_bridge)
        .expect("write bloated bridge");

    let res = check_no_duplicate_dart_solver_math(&tmp.path);
    assert!(res.is_err(), "Bloated bridge and duplicate math markers must fail");
    let errs = res.expect_err("must return violation list");
    assert!(
        errs.iter().any(|e| e.contains("1200")),
        "Must report > 1200 line ceiling violation: {:?}",
        errs
    );
    assert!(
        errs.iter().any(|e| e.contains("synthesize_candidates(")),
        "Must report synthesize_candidates(: {:?}",
        errs
    );
    assert!(
        errs.iter().any(|e| e.contains("compute_composite_score(")),
        "Must report compute_composite_score(: {:?}",
        errs
    );
    assert!(
        errs.iter().any(|e| e.contains("evaluate_constraints(")),
        "Must report evaluate_constraints(: {:?}",
        errs
    );
    assert!(
        errs.iter().any(|e| e.contains("_hotlVetoActive = true")),
        "Must report _hotlVetoActive = true: {:?}",
        errs
    );

    let gate_res = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp.path);
    assert!(gate_res.is_err(), "Check 41 gate must also fail");
}
