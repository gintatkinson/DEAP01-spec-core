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

/// Verifies that `check_41_solver_bridge_and_sqlite_seed_parity_gate` fails when `topology_data.json`
/// is malformed or when `app_flutter/lib/data/seeds/` is missing or contains no non-empty `.dart` files.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A fail-closed persistence verification mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Malformed `topology_data.json` or missing `"nodes"` array produces a `[Check 41]` violation.
/// - Missing `app_flutter/lib/data/seeds/` or empty `.dart` seed files when `topology_data.json` exists produces a `[Check 41]` violation.
///
/// ## 3. Preconditions
/// - Temporary workspace directory contains `app_flutter/assets/topology_data.json`.
///
/// ## 4. Postconditions
/// - Returns `Err(violations)` for malformed topology JSON, missing `seeds/` directory, and empty `seeds/` directory.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ linear in topology JSON size.
/// - Space Complexity: $O(V)$ for violation strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Panics in test if any negative probe silently returns `Ok(())`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-41-BIJECTIVE-SEED-PARITY]
#[test]
fn test_check_41_rejects_malformed_topology_and_missing_or_empty_seeds_dir() {
    // Case 1: Malformed topology_data.json
    let tmp_malformed = ScopedTempDir::new("c41_malformed");
    let assets_malformed = tmp_malformed.path.join("app_flutter/assets");
    fs::create_dir_all(&assets_malformed).expect("create assets dir");
    fs::write(assets_malformed.join("topology_data.json"), "{ invalid json ")
        .expect("write malformed json");
    let res_malformed = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp_malformed.path);
    assert!(
        res_malformed.is_err(),
        "Malformed topology_data.json must fail Check 41"
    );

    // Case 2: Valid topology_data.json but missing app_flutter/lib/data/seeds/ directory
    let tmp_no_seeds = ScopedTempDir::new("c41_no_seeds");
    let assets_no_seeds = tmp_no_seeds.path.join("app_flutter/assets");
    fs::create_dir_all(&assets_no_seeds).expect("create assets dir");
    fs::write(
        assets_no_seeds.join("topology_data.json"),
        r#"{"nodes":[{"id":"NODE_ALPHA_01","typeName":"AlphaComponent","properties":{"custom_metric":1.0}}]}"#,
    )
    .expect("write topology_data.json");
    let res_no_seeds = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp_no_seeds.path);
    assert!(
        res_no_seeds.is_err(),
        "Missing app_flutter/lib/data/seeds/ when topology_data.json exists must fail Check 41"
    );

    // Case 3: Empty seeds directory (no non-empty .dart files)
    let seeds_empty = tmp_no_seeds.path.join("app_flutter/lib/data/seeds");
    fs::create_dir_all(&seeds_empty).expect("create seeds dir");
    fs::write(seeds_empty.join("empty_seed.dart"), "   \n").expect("write whitespace dart file");
    let res_empty_seeds = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp_no_seeds.path);
    assert!(
        res_empty_seeds.is_err(),
        "Empty .dart seed files when topology_data.json exists must fail Check 41"
    );
}

/// Verifies that `check_41_solver_bridge_and_sqlite_seed_parity_gate` rejects an un-delegated or
/// placeholder-polluted `domain_seed_strategy.dart` even when `generated_domain_seed.g.dart` is complete,
/// rejects missing explicit topology `typeName` or `properties` keys, and passes when properly wired.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D bijective SQLite persistence parity standards.
///
/// ## 2. Invariants & Mathematical Properties
/// - `generated_domain_seed.g.dart` cannot mask an un-delegated `domain_seed_strategy.dart` or synthetic placeholders (`space_$`, `field_1`).
/// - Explicit topology node `typeName` and `properties` keys must be present in the SQLite seed.
///
/// ## 3. Preconditions
/// - Temporary workspace directory contains `topology_data.json` and SQLite seed files under `app_flutter/lib/data/seeds/`.
///
/// ## 4. Postconditions
/// - Returns `Err(violations)` for un-delegated/placeholder `domain_seed_strategy.dart` and missing `typeName`/`properties` keys; returns `Ok(())` when complete and delegated.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(E + S)$ linear in topology nodes and seed file size.
/// - Space Complexity: $O(E + V)$ for entity sets and violation diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Panics in test if any negative probe passes or positive probe fails.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-41-BIJECTIVE-SEED-PARITY]
#[test]
fn test_check_41_rejects_unwired_strategy_placeholder_loops_and_missing_types_or_props() {
    let tmp = ScopedTempDir::new("c41_strategy_and_props");
    let assets_dir = tmp.path.join("app_flutter/assets");
    let seeds_dir = tmp.path.join("app_flutter/lib/data/seeds");
    fs::create_dir_all(&assets_dir).expect("create assets dir");
    fs::create_dir_all(&seeds_dir).expect("create seeds dir");

    let topology_json = r#"{
  "nodes": [
    {
      "id": "NODE_ALPHA_01",
      "typeName": "AlphaComponent",
      "properties": {
        "custom_metric_value": 42.5
      }
    },
    {
      "id": "NODE_BETA_02",
      "typeName": "BetaComponent",
      "properties": {
        "status_label_key": "NOMINAL"
      }
    }
  ]
}"#;
    fs::write(assets_dir.join("topology_data.json"), topology_json)
        .expect("write topology_data.json");

    // Probe A: Seed file has entity IDs and 5 tables, but is missing explicit typeName 'BetaComponent' and property 'custom_metric_value'
    let incomplete_seed = r#"
Future<void> seedGeneratedDomainDatabase(Database db) async {
  // type_definitions, type_attributes, type_relations, instances, properties
  // AlphaComponent, NODE_ALPHA_01, NODE_BETA_02, status_label_key
}
"#;
    fs::write(
        seeds_dir.join("generated_domain_seed.g.dart"),
        incomplete_seed,
    )
    .expect("write incomplete seed");
    let res_incomplete = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp.path);
    assert!(
        res_incomplete.is_err(),
        "Seed missing explicit typeName or property key must fail Check 41"
    );
    let errs_incomplete = res_incomplete.expect_err("must return errors");
    assert!(
        errs_incomplete.iter().any(|e| e.contains("BetaComponent")),
        "Must report missing typeName BetaComponent: {:?}",
        errs_incomplete
    );
    assert!(
        errs_incomplete
            .iter()
            .any(|e| e.contains("custom_metric_value")),
        "Must report missing property key custom_metric_value: {:?}",
        errs_incomplete
    );

    // Probe B: Complete generated_domain_seed.g.dart, but sibling domain_seed_strategy.dart does not delegate and uses synthetic space_$ / field_1 placeholders
    let complete_generated_seed = r#"
Future<void> seedGeneratedDomainDatabase(Database db) async {
  // type_definitions, type_attributes, type_relations, instances, properties
  // AlphaComponent, BetaComponent, NODE_ALPHA_01, NODE_BETA_02
  // custom_metric_value, status_label_key
}
"#;
    fs::write(
        seeds_dir.join("generated_domain_seed.g.dart"),
        complete_generated_seed,
    )
    .expect("write complete generated seed");

    let defective_strategy = r#"
class DomainSeedStrategy {
  Future<void> seed(Database db) async {
    for (int i = 0; i < 10; i++) {
      final id = 'space_$i';
      final prop = 'field_1';
    }
  }
}
"#;
    fs::write(
        seeds_dir.join("domain_seed_strategy.dart"),
        defective_strategy,
    )
    .expect("write defective domain_seed_strategy.dart");

    let res_strategy = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp.path);
    assert!(
        res_strategy.is_err(),
        "Un-delegated or placeholder-polluted domain_seed_strategy.dart must fail Check 41 even when generated_domain_seed.g.dart is present"
    );

    // Probe C: Clean domain_seed_strategy.dart delegating to seedGeneratedDomainDatabase
    let clean_strategy = r#"
class DomainSeedStrategy {
  Future<void> seed(Database db) async {
    await seedGeneratedDomainDatabase(db);
  }
}
"#;
    fs::write(seeds_dir.join("domain_seed_strategy.dart"), clean_strategy)
        .expect("write clean domain_seed_strategy.dart");

    let res_clean = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp.path);
    assert!(
        res_clean.is_ok(),
        "Complete generated seed with delegating domain_seed_strategy.dart must pass Check 41: {:?}",
        res_clean.err()
    );
}
