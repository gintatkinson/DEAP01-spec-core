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
    check_38_presentation_anti_hardcoding_ast_gate, check_39_zero_mocking_test_ast_gate,
    check_40_interactive_gesture_and_hud_non_overlap_gate,
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
fn test_check_no_duplicate_dart_solver_math_rejects_bloated_bridge_and_markers(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("dup_math");
    let solver_dir = tmp.path.join("crates/domain-solver/src");
    let data_dir = tmp.path.join("app_flutter/lib/data");
    fs::create_dir_all(&solver_dir)?;
    fs::create_dir_all(&data_dir)?;

    fs::write(solver_dir.join("solver.rs"), "// native rust solver")?;

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

    fs::write(data_dir.join("custom_solver_bridge.dart"), &bloated_bridge)?;

    let res = check_no_duplicate_dart_solver_math(&tmp.path);
    assert!(res.is_err(), "Bloated bridge and duplicate math markers must fail");
    let errs = res.err().ok_or("must return violation list")?;
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
    Ok(())
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
fn test_check_41_rejects_malformed_topology_and_missing_or_empty_seeds_dir(
) -> Result<(), Box<dyn std::error::Error>> {
    // Case 1: Malformed topology_data.json
    let tmp_malformed = ScopedTempDir::new("c41_malformed");
    let assets_malformed = tmp_malformed.path.join("app_flutter/assets");
    fs::create_dir_all(&assets_malformed)?;
    fs::write(assets_malformed.join("topology_data.json"), "{ invalid json ")?;
    let res_malformed = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp_malformed.path);
    assert!(
        res_malformed.is_err(),
        "Malformed topology_data.json must fail Check 41"
    );

    // Case 2: Valid topology_data.json but missing app_flutter/lib/data/seeds/ directory
    let tmp_no_seeds = ScopedTempDir::new("c41_no_seeds");
    let assets_no_seeds = tmp_no_seeds.path.join("app_flutter/assets");
    fs::create_dir_all(&assets_no_seeds)?;
    fs::write(
        assets_no_seeds.join("topology_data.json"),
        r#"{"nodes":[{"id":"NODE_ALPHA_01","typeName":"AlphaComponent","properties":{"custom_metric":1.0}}]}"#,
    )?;
    let res_no_seeds = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp_no_seeds.path);
    assert!(
        res_no_seeds.is_err(),
        "Missing app_flutter/lib/data/seeds/ when topology_data.json exists must fail Check 41"
    );

    // Case 3: Empty seeds directory (no non-empty .dart files)
    let seeds_empty = tmp_no_seeds.path.join("app_flutter/lib/data/seeds");
    fs::create_dir_all(&seeds_empty)?;
    fs::write(seeds_empty.join("empty_seed.dart"), "   \n")?;
    let res_empty_seeds = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp_no_seeds.path);
    assert!(
        res_empty_seeds.is_err(),
        "Empty .dart seed files when topology_data.json exists must fail Check 41"
    );
    Ok(())
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
fn test_check_41_rejects_unwired_strategy_placeholder_loops_and_missing_types_or_props(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("c41_strategy_and_props");
    let assets_dir = tmp.path.join("app_flutter/assets");
    let seeds_dir = tmp.path.join("app_flutter/lib/data/seeds");
    fs::create_dir_all(&assets_dir)?;
    fs::create_dir_all(&seeds_dir)?;

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
    fs::write(assets_dir.join("topology_data.json"), topology_json)?;

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
    )?;
    let res_incomplete = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp.path);
    assert!(
        res_incomplete.is_err(),
        "Seed missing explicit typeName or property key must fail Check 41"
    );
    let errs_incomplete = res_incomplete.err().ok_or("must return errors")?;
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
    )?;

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
    )?;

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
    fs::write(seeds_dir.join("domain_seed_strategy.dart"), clean_strategy)?;

    let res_clean = check_41_solver_bridge_and_sqlite_seed_parity_gate(&tmp.path);
    assert!(
        res_clean.is_ok(),
        "Complete generated seed with delegating domain_seed_strategy.dart must pass Check 41: {:?}",
        res_clean.err()
    );
    Ok(())
}

/// Verifies that Check 39 flags Dart 3 class modifiers on mocks/fakes/stubs, generic `*Repository`/`*Service`
/// `implements` clauses, commented-out `tearDownAll`, tautological `expect(true, isTrue)`, and direct
/// controller/bridge mutations inside `testWidgets` without a `tester.` gesture.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Section 1.9 Zero-Mocking Live Persistence Mandate.
///
/// ## 2. Invariants & Mathematical Properties
/// - Comment stripping prevents `// tearDownAll(() => db.close());` from satisfying SQLite disposal.
/// - Dart 3 modifiers (`final class Mock...`, `sealed class Fake...`, `base class Stub...`) are caught.
/// - `implements *Repository` and `implements *Service` are rejected.
/// - Tautological assertions and gesture-free `controller.`/`bridge.` mutations in `testWidgets` are rejected.
///
/// ## 3. Preconditions
/// - Temporary workspace contains defective Dart test file under `app_flutter/test/`.
///
/// ## 4. Postconditions
/// - `check_39_zero_mocking_test_ast_gate` returns `Err` containing all expected violation diagnostics.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ linear in test file lines.
/// - Space Complexity: $O(V)$ for violation strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Panics in test if any violation class is missed.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-39-ZERO-MOCKING-TEST-AST-GATE]
#[test]
fn test_check_39_negative_probe_flags_dart3_mock_modifiers_commented_teardown_and_direct_mutation_without_tester_gesture(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("c39_neg_probe");
    let test_dir = tmp.path.join("app_flutter/test/features");
    fs::create_dir_all(&test_dir)?;

    let defective_dart = r#"
final class MockEntityStore {}
base class StubTelemetryClient implements AlphaEntityRepository {}
class CustomFakeAdapter implements BetaDomainService {}

void main() {
  setUpAll(() async {
    final db = await openDatabase(inMemoryDatabasePath);
    await DomainSeedStrategy.seed(db);
  });
  // tearDownAll(() async { await db.close(); });

  testWidgets('mutates controller directly without tester gesture', (tester) async {
    await tester.pumpWidget(const SizedBox());
    controller.mutateEntityState('NODE_01');
    expect(true, isTrue);
  });
}
"#;
    fs::write(test_dir.join("defective_persistence_test.dart"), defective_dart)?;

    let res = check_39_zero_mocking_test_ast_gate(&tmp.path);
    assert!(res.is_err(), "Check 39 must reject Dart 3 mocks, commented teardown, tautologies, and gesture-free mutations");
    let errs = res.err().ok_or("expected Check 39 violations")?;
    assert!(errs.iter().any(|e| e.contains("MockEntityStore")), "Must flag final class MockEntityStore: {:?}", errs);
    assert!(errs.iter().any(|e| e.contains("AlphaEntityRepository")), "Must flag implements AlphaEntityRepository: {:?}", errs);
    assert!(errs.iter().any(|e| e.contains("BetaDomainService")), "Must flag implements BetaDomainService: {:?}", errs);
    assert!(errs.iter().any(|e| e.contains("tearDownAll")), "Must flag commented-out tearDownAll: {:?}", errs);
    assert!(errs.iter().any(|e| e.contains("expect(true, isTrue)")), "Must flag tautological expect(true, isTrue): {:?}", errs);
    assert!(errs.iter().any(|e| e.contains("controller.mutateEntityState")), "Must flag gesture-free controller mutation: {:?}", errs);
    Ok(())
}

/// Verifies that Check 39 passes a clean SQLite test suite with uncommented `tearDownAll` and real `tester.tap` gestures.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A live persistence verification standards.
///
/// ## 2. Invariants & Mathematical Properties
/// - Clean `openDatabase` + `DomainSeedStrategy` with active `tearDownAll` and `tester.tap` yields zero violations.
///
/// ## 3. Preconditions
/// - Temporary workspace contains a compliant Dart test file under `app_flutter/test/`.
///
/// ## 4. Postconditions
/// - `check_39_zero_mocking_test_ast_gate` returns `Ok(())`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ linear in test file lines.
/// - Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Panics in test if false-positive violations are reported.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-39-ZERO-MOCKING-TEST-AST-GATE]
#[test]
fn test_check_39_positive_probe_passes_real_sqlite_and_tester_gesture(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("c39_pos_probe");
    let test_dir = tmp.path.join("app_flutter/test/features");
    fs::create_dir_all(&test_dir)?;

    let clean_dart = r#"
void main() {
  late Database db;
  setUpAll(() async {
    db = await openDatabase(inMemoryDatabasePath);
    await seedGeneratedDomainDatabase(db);
  });

  tearDownAll(() async {
    await db.close();
  });

  testWidgets('actuates UI via tester.tap before verifying state', (tester) async {
    await tester.pumpWidget(const SizedBox());
    await tester.tap(find.byKey(const ValueKey('entity_row_NODE_01')));
    await tester.pumpAndSettle();
    expect(find.byKey(const ValueKey('entity_inspector_NODE_01')), findsOneWidget);
  });
}
"#;
    fs::write(test_dir.join("clean_persistence_test.dart"), clean_dart)?;

    let res = check_39_zero_mocking_test_ast_gate(&tmp.path);
    assert!(res.is_ok(), "Compliant SQLite + gesture test must pass Check 39: {:?}", res.err());
    Ok(())
}

/// Verifies that Check 40 fails closed when `schema_ui_contract_test.g.dart` is missing or has commented-out
/// `tester.tap`/`Rect.fromLTWH`, and flags domain-agnostic direct `bridge.<method>(` mutations without `tester.` gestures.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H spatial and interactive verification gates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Missing `app_flutter/test/generated/schema_ui_contract_test.g.dart` fails closed when `app_flutter/test` exists.
/// - Commented-out `// await tester.tap(...)` or synthetic `Rect.fromLTWH(` in contract test is rejected.
/// - Direct `bridge.<method>(` call in `testWidgets` without a `tester.` gesture is rejected without hardcoded domain names.
///
/// ## 3. Preconditions
/// - Temporary workspace contains `app_flutter/test/`.
///
/// ## 4. Postconditions
/// - `check_40_interactive_gesture_and_hud_non_overlap_gate` returns `Err` for both missing and commented-out contracts.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ linear in scanned test lines.
/// - Space Complexity: $O(V)$ for violation strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Panics in test if missing file or commented contract passes.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-40-INTERACTIVE-GESTURE-HUD-NON-OVERLAP]
#[test]
fn test_check_40_negative_probe_fail_closed_on_missing_or_commented_contract_and_direct_bridge_mutation(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("c40_neg_probe");
    let test_dir = tmp.path.join("app_flutter/test/widgets");
    fs::create_dir_all(&test_dir)?;

    // Case 1: Missing schema_ui_contract_test.g.dart must fail closed
    let res_missing = check_40_interactive_gesture_and_hud_non_overlap_gate(&tmp.path);
    assert!(res_missing.is_err(), "Missing schema_ui_contract_test.g.dart must fail closed");
    let errs_missing = res_missing.err().ok_or("expected missing contract error")?;
    assert!(
        errs_missing.iter().any(|e| e.contains("schema_ui_contract_test.g.dart")),
        "Must report missing schema_ui_contract_test.g.dart: {:?}",
        errs_missing
    );

    // Case 2: Contract file present but uses commented-out tester.tap and synthetic Rect.fromLTWH, plus direct bridge mutation in widget test
    let gen_dir = tmp.path.join("app_flutter/test/generated");
    fs::create_dir_all(&gen_dir)?;
    let commented_contract = r#"
// FHD_1080P_1X QHD_1440P_1X UHD_4K_1X UHD_4K_2X XDR_6K_1X XDR_6K_2X
void main() {
  testWidgets('placeholder contract', (tester) async {
    final rect = const Rect.fromLTWH(0, 0, 1920, 1080);
    // await tester.tap(find.byKey(SchemaWidgetKeys.entityRow('NODE_01')));
    // tester.getRect(find.byKey(SchemaWidgetKeys.hudOverlay('primary')));
  });
}
"#;
    fs::write(gen_dir.join("schema_ui_contract_test.g.dart"), commented_contract)?;

    let widget_test = r#"
void main() {
  testWidgets('direct generic bridge mutation without gesture', (tester) async {
    await tester.pumpWidget(const HudOverlay());
    bridge.applyGenericParameterMutation(42);
  });
}
"#;
    fs::write(test_dir.join("generic_bridge_test.dart"), widget_test)?;

    let res_commented = check_40_interactive_gesture_and_hud_non_overlap_gate(&tmp.path);
    assert!(res_commented.is_err(), "Commented contract and direct bridge mutation must fail Check 40");
    let errs = res_commented.err().ok_or("expected Check 40 violations")?;
    assert!(errs.iter().any(|e| e.contains("tester.tap(")), "Must flag missing uncommented tester.tap(: {:?}", errs);
    assert!(errs.iter().any(|e| e.contains("tester.getRect(")), "Must flag missing uncommented tester.getRect(: {:?}", errs);
    assert!(errs.iter().any(|e| e.contains("Rect.fromLTWH(")), "Must flag synthetic Rect.fromLTWH(: {:?}", errs);
    assert!(
        errs.iter().any(|e| e.contains("bridge.applyGenericParameterMutation(")),
        "Must flag generic direct bridge mutation: {:?}",
        errs
    );
    Ok(())
}

/// Verifies that Check 40 passes when `schema_ui_contract_test.g.dart` contains all 6 viewports,
/// all 4 physical resolutions, live `tester.getRect(`, `tester.tap(`, `assertPairwiseHudNonOverlap`, and `.overlaps(`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H multi-viewport verification standards.
///
/// ## 2. Invariants & Mathematical Properties
/// - Complete uncommented contract test and gesture-driven widget test pass Check 40 with zero violations.
///
/// ## 3. Preconditions
/// - Temporary workspace contains compliant `schema_ui_contract_test.g.dart` and widget test.
///
/// ## 4. Postconditions
/// - `check_40_interactive_gesture_and_hud_non_overlap_gate` returns `Ok(())`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ linear in contract and test lines.
/// - Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Panics in test if false-positive violations occur.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-40-INTERACTIVE-GESTURE-HUD-NON-OVERLAP]
#[test]
fn test_check_40_positive_probe_passes_complete_4_viewport_contract(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("c40_pos_probe");
    let gen_dir = tmp.path.join("app_flutter/test/generated");
    fs::create_dir_all(&gen_dir)?;

    let valid_contract = r#"
const kCanonicalViewports = [
  CanonicalViewport(identifier: 'FHD_1080P_1X', physicalWidth: 1920, physicalHeight: 1080),
  CanonicalViewport(identifier: 'QHD_1440P_1X', physicalWidth: 2560, physicalHeight: 1440),
  CanonicalViewport(identifier: 'UHD_4K_1X', physicalWidth: 3840, physicalHeight: 2160),
  CanonicalViewport(identifier: 'UHD_4K_2X', physicalWidth: 3840, physicalHeight: 2160),
  CanonicalViewport(identifier: 'XDR_6K_1X', physicalWidth: 6016, physicalHeight: 3384),
  CanonicalViewport(identifier: 'XDR_6K_2X', physicalWidth: 6016, physicalHeight: 3384),
];
void assertPairwiseHudNonOverlap(List<Rect> rects) {
  final overlaps = rects[0].overlaps(rects[1]);
}
void main() {
  testWidgets('live contract', (tester) async {
    await tester.pumpWidget(const SizedBox());
    final r = tester.getRect(find.byKey(const ValueKey('hud')));
    assertPairwiseHudNonOverlap([r, r]);
    await tester.tap(find.byKey(const ValueKey('row')));
  });
}
"#;
    fs::write(gen_dir.join("schema_ui_contract_test.g.dart"), valid_contract)?;

    let res = check_40_interactive_gesture_and_hud_non_overlap_gate(&tmp.path);
    assert!(res.is_ok(), "Complete 4-resolution, 6-viewport contract must pass Check 40: {:?}", res.err());
    Ok(())
}

/// Verifies that Check 38 (`check_38_presentation_anti_hardcoding_ast_gate`) rejects all 19 structural
/// presentation hardcoding patterns (inner parentheses in `Text(...)`, trailing `style:` parameters,
/// multi-line `Text(...)`, named `id:`/`label:` widget arguments, arbitrary whitespace after `isActive:`/`isSelected:`,
/// parentId/sub-stem entity references, percentage telemetry, slash-ratio counts, colon-prefixed telemetry
/// readouts, bracketed badges, and numeric unit/count tokens).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H human-machine interface anti-hardcoding verification mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Every individual hardcoded presentation symptom pattern produces `Err(violations)` when placed in `app_flutter/lib/features/**/*.dart`.
/// - Zero Regex and Zero Hardcoded Domain Concepts: all detections rely on dynamic topology/schema entity extraction and structural token predicates.
///
/// ## 3. Preconditions
/// - Temporary workspace contains `app_flutter/assets/topology_data.json` and `app_flutter/lib/features/panel/view.dart`.
///
/// ## 4. Postconditions
/// - Every negative probe returns `Err` from `check_38_presentation_anti_hardcoding_ast_gate`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(P \times L)$ where $P$ is the probe count and $L$ is file length.
/// - Space Complexity: $O(E + V)$ for extracted entity identifiers and violation strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Panics in test if any of the 20 negative probes silently passes Check 38.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-38-PRESENTATION-ANTI-HARDCODING]
#[test]
fn test_check_38_rejects_all_19_hardcoded_presentation_telemetry_and_entity_patterns(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("c38_all_19_patterns");
    let assets_dir = tmp.path.join("app_flutter/assets");
    let feature_dir = tmp.path.join("app_flutter/lib/features/panel");
    fs::create_dir_all(&assets_dir)?;
    fs::create_dir_all(&feature_dir)?;

    let topology_json = r#"{
  "nodes": [
    {
      "id": "UNIT_ALPHA_01",
      "label": "Unit-Alpha (P1)",
      "parentId": "ZONE-NORTH-01",
      "typeName": "AlphaUnit",
      "properties": {
        "zone_code": "ZONE-ALPHA-01"
      }
    },
    {
      "id": "UNIT_BRAVO_02",
      "label": "Unit-Bravo (P2)",
      "parentId": "ZONE-NORTH-01",
      "typeName": "AlphaUnit",
      "properties": {}
    }
  ]
}"#;
    fs::write(assets_dir.join("topology_data.json"), topology_json)?;

    let probes = [
        "const Text('Unit-Alpha (P1)', style: TextStyle())",
        "const Text('Unit-Bravo (P2)')",
        "EntityRow(isActive:   true)",
        "EntityRow(isActive:   false)",
        "ZoneSelector(id: 'ZONE-ALPHA-01')",
        "const Text('2 NODES')",
        "const Text('75% Reserve * Primary First')",
        "const Text('FOCUSED NODE: Unit-Alpha')",
        "const Text('NORTH-01 v')",
        "const Text('WAVE 02 INGRESS (T-01:09) * 12x CLASS-2NODE')",
        "const Text('RULES: 0 VIOLATIONS')",
        "const Text('SENSOR-01: 15km', style: TextStyle())",
        "const Text('ACTUATOR-02: ENGAGED', style: TextStyle())",
        "const Text('[AIR-GAP]')",
        "const Text('LOW (0.0%)')",
        "const Text('2 / 8 ALLOCATED')",
        "const Text('6 / 8 ALLOCATED')",
        "const Text('(ENGINE CONF: 99.2%)')",
        "const Text('WAVE N+1: PASS')",
        "const Text(\n  'Unit-Alpha (P1)',\n)",
    ];

    let view_file = feature_dir.join("view.dart");
    for (idx, snippet) in probes.iter().enumerate() {
        let dart_src = format!(
            "class PanelView {{\n  void build() {{\n    {};\n  }}\n}}\n",
            snippet
        );
        fs::write(&view_file, &dart_src)?;
        let res = check_38_presentation_anti_hardcoding_ast_gate(&tmp.path);
        assert!(
            res.is_err(),
            "Check 38 probe #{} failed to reject hardcoded snippet: {}",
            idx + 1,
            snippet
        );
    }
    Ok(())
}

/// Verifies that Check 38 strips `/* ... */` block comments and `//` line comments before scanning,
/// and permits `Key(...)`/`ValueKey(...)`, `import` directives, and dynamically interpolated `$` strings.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 zero false-positive comment and key disambiguation mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Commented-out hardcoded literals inside `/* ... */` or `//` do not trigger violations.
/// - `Key('...')`, `ValueKey('...')`, `import '...'`, and interpolated `'${vm.value}'` strings pass cleanly.
///
/// ## 3. Preconditions
/// - Temporary workspace contains `topology_data.json` and a clean Dart feature file with comments, keys, and interpolations.
///
/// ## 4. Postconditions
/// - `check_38_presentation_anti_hardcoding_ast_gate` returns `Ok(())`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ linear in Dart file length.
/// - Space Complexity: $O(L)$ for comment-stripped buffer.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Panics in test if false-positive violations are emitted.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-38-PRESENTATION-ANTI-HARDCODING]
#[test]
fn test_check_38_ignores_block_comments_keys_and_interpolated_dynamic_bindings(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("c38_clean_comments_and_keys");
    let assets_dir = tmp.path.join("app_flutter/assets");
    let feature_dir = tmp.path.join("app_flutter/lib/features/panel");
    fs::create_dir_all(&assets_dir)?;
    fs::create_dir_all(&feature_dir)?;

    let topology_json = r#"{
  "nodes": [
    {
      "id": "UNIT_ALPHA_01",
      "label": "Unit-Alpha (P1)",
      "parentId": "ZONE-NORTH-01",
      "typeName": "AlphaUnit"
    }
  ]
}"#;
    fs::write(assets_dir.join("topology_data.json"), topology_json)?;

    let clean_dart = r#"
import 'package:app_flutter/features/zone-north-01/panel_model.dart';

/*
  const Text('Unit-Alpha (P1)');
  const Text('LOW (0.0%)');
  EntityRow(isActive: true);
*/
class CleanPanelView {
  void build(PanelViewModel vm) {
    // const Text('RULES: 0 VIOLATIONS');
    final k1 = const Key('ZONE-NORTH-01');
    final k2 = const ValueKey('UNIT_ALPHA_01');
    final t1 = Text('${vm.unitLabel} (P1)');
    final t2 = Text('${vm.ratioText}');
    final row = EntityRow(isActive: vm.isActive, isSelected: vm.isSelected);
  }
}
"#;
    fs::write(feature_dir.join("clean_view.dart"), clean_dart)?;

    let res = check_38_presentation_anti_hardcoding_ast_gate(&tmp.path);
    assert!(
        res.is_ok(),
        "Clean view with block comments, keys, and dynamic interpolations must pass Check 38: {:?}",
        res.err()
    );
    Ok(())
}
