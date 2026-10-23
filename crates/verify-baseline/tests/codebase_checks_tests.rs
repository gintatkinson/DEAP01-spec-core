//! Integration tests for Baseline Verification Checks 38 through 41
//! (`crates/verify-baseline/src/checks/codebase.rs`) and Check 19 domain cleanliness.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and MIL-STD-1472H.
//! Verifies:
//! - Check 38: Presentation anti-hardcoding gate (flags hardcoded dynamic labels, numeric units, static row flags).
//! - Check 39: Zero-mocking test AST gate (flags direct controller mutations without preceding tester gestures, mocks, leaking SQLite).
//! - Check 40: Interactive gesture & 4-viewport non-overlap gate (fail-closed, requires real gestures and 4 viewports).
//! - Check 41: Solver bridge SSOT delegation & SQLite seed parity gate (requires solver_bridge.g.dart, rejects arithmetic in Dart bridge, verifies bijective seed parity).
//! - Check 19: Domain cleanliness of `codebase.rs` asserting zero customer domain tokens.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Negative Probes (RED): Every gate is verified to fail deterministically on defective fixtures.
//! - Positive Probes (GREEN): Compliant fixtures pass with `Ok(())`.
//! - Domain Purity: Zero customer domain tokens in `codebase.rs`.
//! - Zero Mocking: All tests execute against real temporary directory fixtures cleaned up via RAII.
//!
//! ## 3. Preconditions
//! - Standard temporary directory writable by the executing process.
//!
//! ## 4. Postconditions
//! - All integration tests pass deterministically with zero side effects or persistent artifacts.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(T * F) bounded linear test execution over fixtures.
//! - Space Complexity: O(M) bounded memory for temporary file buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - RAII Drop cleanup guarantees temporary workspaces are deleted even on assertion failure or unwind.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-WP4-CODEBASE-CHECKS-TESTS], [DO-330-TQL-4-CODEBASE-TESTS]

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use verify_baseline::checks::{
    check_38_presentation_anti_hardcoding_ast_gate,
    check_39_zero_mocking_test_ast_gate,
    check_40_interactive_gesture_and_hud_non_overlap_gate,
    check_41_solver_bridge_and_sqlite_seed_parity_gate,
};

/// Scoped RAII temporary workspace fixture for filesystem isolation.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A test isolation requirements. Prevents workspace pollution.
///
/// ## 2. Invariants & Mathematical Properties
/// - Creates a uniquely named directory under `std::env::temp_dir()`.
///
/// ## 3. Preconditions
/// - System temporary directory is writable.
///
/// ## 4. Postconditions
/// - Provides a valid workspace path on disk.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) directory creation.
/// - Space Complexity: O(1) path allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Panics on directory creation failure.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-TEST-WORKSPACE-ISOLATION-STRUCT]
struct ScopedTempWorkspace {
    root: PathBuf,
}

impl ScopedTempWorkspace {
    /// Creates a new scoped temporary workspace in the system temporary directory.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A test isolation requirements.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Uniquely named directory using PID, epoch nanoseconds, and monotonic counter.
    ///
    /// ## 3. Preconditions
    /// - `prefix: &str` is a non-empty alphanumeric label.
    ///
    /// ## 4. Postconditions
    /// - Returns a `ScopedTempWorkspace` instance with directory created on disk.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) directory creation.
    /// - Space Complexity: O(1) path allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Panics if directory creation fails.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-TEST-WORKSPACE-ISOLATION-CTOR]
    fn new(prefix: &str) -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!(
            "deap_codebase_{}_{}_{}_{}",
            prefix,
            std::process::id(),
            nanos,
            count
        ));
        fs::create_dir_all(&root).expect("create temp workspace");
        Self { root }
    }
}

impl Drop for ScopedTempWorkspace {
    /// Cleans up the temporary directory on drop.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A cleanup and resource management standards.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Removes the entire directory tree rooted at `self.root`.
    ///
    /// ## 3. Preconditions
    /// - `self` is an initialized `ScopedTempWorkspace`.
    ///
    /// ## 4. Postconditions
    /// - Temporary directory is deleted from disk.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(F) proportional to number of temporary files created.
    /// - Space Complexity: O(1) auxiliary memory.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Silently ignores removal failure during drop to avoid panicking during unwinding.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-TEST-WORKSPACE-ISOLATION-DROP]
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// =========================================================================
// Check 38 Tests: Presentation Anti-Hardcoding Gate
// =========================================================================

/// Negative probe test for Check 38: Verifies rejection of hardcoded dynamic entity labels and telemetry units.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H human-machine interface verification mandates.
/// Mechanically rejects hardcoded entity labels, telemetry units, and static row builder booleans.
///
/// ## 2. Invariants & Mathematical Properties
/// - Detects known entity IDs from topology JSON embedded in string literals.
/// - Detects hardcoded numeric telemetry units (`km`, `%`, `NODES`, `PASS`, `ENGAGED`, `[AIR-GAP]`).
/// - Detects static row builder booleans (`isActive: true`).
///
/// ## 3. Preconditions
/// - Temporary workspace initialized with topology JSON and defective presentation Dart file.
///
/// ## 4. Postconditions
/// - Check 38 returns `Err(Vec<String>)` containing diagnostics for all defective occurrences.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(F * L) over scanned Dart lines.
/// - Space Complexity: O(V) for diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts non-empty error list with expected diagnostic substrings.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-38-NEGATIVE-PROBE-LABELS-AND-TELEMETRY]
#[test]
fn test_check_38_negative_probe_flags_hardcoded_entity_labels_and_telemetry() {
    let ws = ScopedTempWorkspace::new("check_38_neg_labels");
    let assets_dir = ws.root.join("app_flutter/assets");
    let feature_dir = ws.root.join("app_flutter/lib/features/tactical");
    fs::create_dir_all(&assets_dir).expect("create assets dir");
    fs::create_dir_all(&feature_dir).expect("create feature dir");

    let topology_json = r#"{
  "nodes": [
    {"id": "SENSOR-NODE-ALPHA", "name": "Sensor Alpha", "type": "Radar"},
    {"id": "SECTOR-NORTH-01", "name": "Sector North", "type": "Sector"}
  ]
}"#;
    fs::write(assets_dir.join("topology_data.json"), topology_json).expect("write topology");

    let defective_dart = r#"
class TacticalCockpitView extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        const Text('SENSOR-NODE-ALPHA (Primary)'),
        const Text('RADAR-01: 15km'),
        const Text('2 / 8 INTERCEPTORS'),
        const Text('75% Reserve'),
        const Text('(SOLVER CONF: 99.2%)'),
        const Text('WAVE N+1: PASS'),
        const Text('EW-JAM-02: ENGAGED'),
        const Text('[AIR-GAP]'),
        const Text('2 NODES'),
        const Text('T-00:45'),
        EntityItem(id: 'SECTOR-NORTH-01', isActive: true),
      ],
    );
  }
}
"#;
    fs::write(feature_dir.join("tactical_cockpit_view.dart"), defective_dart).expect("write dart");

    let res = check_38_presentation_anti_hardcoding_ast_gate(&ws.root);
    assert!(
        res.is_err(),
        "Check 38 must fail on hardcoded dynamic entity labels and telemetry units"
    );
    let errs = res.unwrap_err().join("\n");
    assert!(errs.contains("SENSOR-NODE-ALPHA"), "Expected entity violation, got:\n{}", errs);
    assert!(errs.contains("isActive: true"), "Expected static boolean violation, got:\n{}", errs);
    assert!(errs.contains("15km") || errs.contains("telemetry"), "Expected telemetry violation, got:\n{}", errs);
}

/// Negative probe test for Check 38: Verifies rejection of multi-line Text literals containing entity names.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H anti-fabrication standards.
/// Ensures multi-line Text widgets do not evade presentation static scanning.
///
/// ## 2. Invariants & Mathematical Properties
/// - Detects entity identifiers split across multiple lines in `Text(...)` constructors.
///
/// ## 3. Preconditions
/// - Temporary workspace with topology declaring entity and multi-line Text call.
///
/// ## 4. Postconditions
/// - Check 38 returns `Err(Vec<String>)` citing the entity violation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) over lines.
/// - Space Complexity: O(V) for diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts error vector contains reference to the target entity identifier.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-38-NEGATIVE-PROBE-MULTILINE-TEXT]
#[test]
fn test_check_38_negative_probe_flags_multiline_text_literals() {
    let ws = ScopedTempWorkspace::new("check_38_neg_multiline");
    let assets_dir = ws.root.join("app_flutter/assets");
    let feature_dir = ws.root.join("app_flutter/lib/features/sidebar");
    fs::create_dir_all(&assets_dir).expect("create assets dir");
    fs::create_dir_all(&feature_dir).expect("create feature dir");

    let topology_json = r#"{
  "nodes": [
    {"id": "ASSET-RADAR-MAST-04", "name": "Radar Mast", "type": "Mast"}
  ]
}"#;
    fs::write(assets_dir.join("topology_data.json"), topology_json).expect("write topology");

    let defective_dart = r#"
class SidebarTreeItem extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(
          'ASSET-RADAR-MAST-04',
          style: TextStyle(fontWeight: FontWeight.bold),
        ),
      ],
    );
  }
}
"#;
    fs::write(feature_dir.join("sidebar_tree.dart"), defective_dart).expect("write dart");

    let res = check_38_presentation_anti_hardcoding_ast_gate(&ws.root);
    assert!(
        res.is_err(),
        "Check 38 must fail on multi-line Text widgets referencing known entities"
    );
    let errs = res.unwrap_err().join("\n");
    assert!(errs.contains("ASSET-RADAR-MAST-04"), "Expected multi-line entity violation, got:\n{}", errs);
}

/// Positive probe test for Check 38: Verifies that clean, dynamic ViewModel presentation passes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A verified presentation layer standards.
///
/// ## 2. Invariants & Mathematical Properties
/// - Presentation widgets reading from dynamic ViewModel properties pass cleanly.
///
/// ## 3. Preconditions
/// - Temporary workspace with properly bound presentation components.
///
/// ## 4. Postconditions
/// - Check 38 returns `Ok(())`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) scanning clean source.
/// - Space Complexity: O(1) zero violations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts `is_ok()`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-38-POSITIVE-PROBE-CLEAN-PRESENTATION]
#[test]
fn test_check_38_positive_probe_passes_clean_dynamic_presentation() {
    let ws = ScopedTempWorkspace::new("check_38_pos_clean");
    let feature_dir = ws.root.join("app_flutter/lib/features/workspace");
    fs::create_dir_all(&feature_dir).expect("create feature dir");

    let clean_dart = r#"
class CompliantWorkspaceView extends StatelessWidget {
  final WorkspaceViewModel vm;
  const CompliantWorkspaceView({required this.vm, super.key});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(vm.displayTitle),
        Text(vm.telemetryFormattedSummary),
        EntityRow(
          label: vm.selectedEntityLabel,
          isActive: vm.isEntityActive,
        ),
      ],
    );
  }
}
"#;
    fs::write(feature_dir.join("workspace_view.dart"), clean_dart).expect("write dart");

    let res = check_38_presentation_anti_hardcoding_ast_gate(&ws.root);
    assert!(
        res.is_ok(),
        "Check 38 must pass for clean, dynamic ViewModel data binding, got: {:?}",
        res.err()
    );
}

// =========================================================================
// Check 39 Tests: Zero-Mocking Test AST & SQLite Disposal Gate
// =========================================================================

/// Negative probe test for Check 39: Rejects direct controller mutation in testWidgets without preceding tester gestures.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Section 1.9 of the Project Constitution.
/// Verifies that test suites actuate controls via UI gestures rather than directly mutating controllers.
///
/// ## 2. Invariants & Mathematical Properties
/// - Rejects direct mutation method calls on `*Controller`, `*Bridge`, `*Notifier` prior to `expect`
///   when no `tester.tap`, `tester.drag`, etc. gestures were dispatched.
///
/// ## 3. Preconditions
/// - Temporary workspace containing a test file with direct controller mutation.
///
/// ## 4. Postconditions
/// - Check 39 returns `Err(Vec<String>)` flagging direct mutation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(T * L) over test lines.
/// - Space Complexity: O(V) for diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts error vector contains direct mutation diagnostic.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-39-NEGATIVE-PROBE-DIRECT-MUTATION]
#[test]
fn test_check_39_negative_probe_flags_direct_controller_mutation_without_tester_gesture() {
    let ws = ScopedTempWorkspace::new("check_39_neg_mutation");
    let test_dir = ws.root.join("app_flutter/test/features");
    fs::create_dir_all(&test_dir).expect("create test dir");

    let defective_test = r#"
void main() {
  testWidgets('defective test mutating controller directly without gesture', (tester) async {
    await tester.pumpWidget(const MyApp());
    tacticalController.setInterlockEngaged(true);
    expect(find.text('ENGAGED'), findsOneWidget);
  });
}
"#;
    fs::write(test_dir.join("tactical_widget_test.dart"), defective_test).expect("write test");

    let res = check_39_zero_mocking_test_ast_gate(&ws.root);
    assert!(
        res.is_err(),
        "Check 39 must reject direct controller/bridge mutation in testWidgets without preceding tester gestures"
    );
    let errs = res.unwrap_err().join("\n");
    assert!(
        errs.contains("mutation") || errs.contains("controller") || errs.contains("tacticalController"),
        "Expected direct mutation diagnostic, got:\n{}",
        errs
    );
}

/// Negative probe test for Check 39: Rejects mock classes, fake implementations, and leaking SQLite connections.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A live persistence testing mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Flags `class Mock*`, `extends Mock`, `class Fake*`, and `setUpAll` without `tearDownAll` disposal.
///
/// ## 3. Preconditions
/// - Temporary workspace with mock classes and unclosed SQLite test.
///
/// ## 4. Postconditions
/// - Check 39 returns `Err(Vec<String>)` citing mock classes and leaking SQLite teardown.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) over lines.
/// - Space Complexity: O(V) for diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts violations report both mock usage and teardown omission.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-39-NEGATIVE-PROBE-MOCKS-AND-LEAKS]
#[test]
fn test_check_39_negative_probe_flags_mock_classes_and_leaking_sqlite() {
    let ws = ScopedTempWorkspace::new("check_39_neg_mocks");
    let test_dir = ws.root.join("app_flutter/test/unit");
    fs::create_dir_all(&test_dir).expect("create test dir");

    let defective_test = r#"
class MockTacticalService extends Mock implements TacticalService {}
class FakeDataProvider extends Fake {}

void main() {
  setUpAll(() async {
    final db = await openDatabase(':memory:');
    // Notice: Missing tearDownAll (.close() / .dispose())
  });

  test('mocked verification', () {
    final res = const Result.success([]);
    expect(res, isNotNull);
  });
}
"#;
    fs::write(test_dir.join("mock_unit_test.dart"), defective_test).expect("write test");

    let res = check_39_zero_mocking_test_ast_gate(&ws.root);
    assert!(
        res.is_err(),
        "Check 39 must reject mock classes and leaking SQLite connections"
    );
    let errs = res.unwrap_err().join("\n");
    assert!(errs.contains("Mock"), "Expected Mock diagnostic, got:\n{}", errs);
    assert!(errs.contains("tearDownAll"), "Expected tearDownAll diagnostic, got:\n{}", errs);
}

/// Positive probe test for Check 39: Verifies that clean, gesture-actuated tests with proper teardown pass.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A live persistence and gesture-driven test mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Tests with real `tester.tap(...)` and proper SQLite `tearDownAll` pass cleanly.
///
/// ## 3. Preconditions
/// - Temporary workspace with compliant test files.
///
/// ## 4. Postconditions
/// - Check 39 returns `Ok(())`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) scan.
/// - Space Complexity: O(1) zero violations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts `is_ok()`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-39-POSITIVE-PROBE-CLEAN-TEST]
#[test]
fn test_check_39_positive_probe_passes_clean_gesture_actuated_tests() {
    let ws = ScopedTempWorkspace::new("check_39_pos_clean");
    let test_dir = ws.root.join("app_flutter/test/e2e");
    fs::create_dir_all(&test_dir).expect("create test dir");

    let clean_test = r#"
void main() {
  late Database db;
  setUpAll(() async {
    db = await openDatabase(inMemoryDatabasePath);
    await seedGeneratedDomainDatabase(db);
  });

  tearDownAll(() async {
    await db.close();
  });

  testWidgets('clean test with real gesture actuation', (tester) async {
    await tester.pumpWidget(const MyApp());
    await tester.tap(find.byKey(const Key('tactical_action_btn')));
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('tactical_status')), findsOneWidget);
  });
}
"#;
    fs::write(test_dir.join("clean_e2e_test.dart"), clean_test).expect("write test");

    let res = check_39_zero_mocking_test_ast_gate(&ws.root);
    assert!(
        res.is_ok(),
        "Check 39 must pass for gesture-actuated tests with proper teardown, got: {:?}",
        res.err()
    );
}

// =========================================================================
// Check 40 Tests: Interactive Gesture & Viewport Contract Gate
// =========================================================================

/// Negative probe test for Check 40: Proves fail-closed behavior when generated contract test is missing.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H fail-closed verification standards.
/// When Flutter and SysML schema exist, the generated UI contract test MUST exist.
///
/// ## 2. Invariants & Mathematical Properties
/// - When `app_flutter/lib/` and `.pipeline/schema.sysml` exist, missing
///   `app_flutter/test/generated/schema_ui_contract_test.g.dart` is a hard error.
///
/// ## 3. Preconditions
/// - Temporary workspace with `app_flutter/lib/` and `schema.sysml`, but no contract test.
///
/// ## 4. Postconditions
/// - Check 40 returns `Err(Vec<String>)` citing missing contract test.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) filesystem existence check.
/// - Space Complexity: O(1) diagnostic.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts error vector mentions missing contract test file.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-40-NEGATIVE-PROBE-FAIL-CLOSED-MISSING-CONTRACT]
#[test]
fn test_check_40_negative_probe_fail_closed_on_missing_contract_test() {
    let ws = ScopedTempWorkspace::new("check_40_neg_fail_closed");
    let lib_dir = ws.root.join("app_flutter/lib");
    let pipeline_dir = ws.root.join(".pipeline");
    let test_dir = ws.root.join("app_flutter/test");
    fs::create_dir_all(&lib_dir).expect("create lib dir");
    fs::create_dir_all(&pipeline_dir).expect("create pipeline dir");
    fs::create_dir_all(&test_dir).expect("create test dir");

    fs::write(pipeline_dir.join("schema.sysml"), "package Model {}").expect("write schema");
    fs::write(lib_dir.join("main.dart"), "void main() {}").expect("write main");

    // schema_ui_contract_test.g.dart is deliberately NOT created!
    let res = check_40_interactive_gesture_and_hud_non_overlap_gate(&ws.root);
    assert!(
        res.is_err(),
        "Check 40 must fail-closed when schema_ui_contract_test.g.dart is missing in downstream workspace"
    );
    let errs = res.unwrap_err().join("\n");
    assert!(
        errs.contains("schema_ui_contract_test.g.dart") || errs.contains("missing"),
        "Expected missing contract file diagnostic, got:\n{}",
        errs
    );
}

/// Negative probe test for Check 40: Rejects missing viewports, fake rectangles, commented gestures, and shallow probing.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to MIL-STD-1472H multi-resolution spatial layout verification mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Rejects contracts missing canonical viewports (such as 3840 or 6016).
/// - Rejects commented-out gestures (`// await tester.tap`) and fake `Rect.fromLTWH` constants.
/// - Rejects shallow `evaluate().isNotEmpty` and `probeWidgetKey`.
/// - Rejects HUD overlay mounting without `assertPairwiseHudNonOverlap` or `.overlaps(`.
///
/// ## 3. Preconditions
/// - Temporary workspace with contract test and widget test containing violations.
///
/// ## 4. Postconditions
/// - Check 40 returns `Err(Vec<String>)` detailing the violations.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(T * L) over scanned test lines.
/// - Space Complexity: O(V) for diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts error vector cites viewports, shallow probing, and missing HUD assertions.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-40-NEGATIVE-PROBE-DEFECTIVE-CONTRACT-AND-GESTURES]
#[test]
fn test_check_40_negative_probe_flags_missing_viewports_and_fake_rectangles() {
    let ws = ScopedTempWorkspace::new("check_40_neg_viewports");
    let gen_dir = ws.root.join("app_flutter/test/generated");
    let test_dir = ws.root.join("app_flutter/test/widgets");
    fs::create_dir_all(&gen_dir).expect("create gen dir");
    fs::create_dir_all(&test_dir).expect("create test dir");

    // Defective contract test: missing 6016 viewport, has commented-out tap and fake Rect.fromLTWH
    let defective_contract = r#"
const kViewports = [
  Size(1920, 1080),
  Size(2560, 1440),
  Size(3840, 2160),
  // Missing Size(6016, 3384) / XDR_6K_2X!
];

void main() {
  testWidgets('contract test', (tester) async {
    const fakeRect = Rect.fromLTWH(16, 16, 320, 64);
    // await tester.tap(find.byKey(Key('deap_action_btn')));
  });
}
"#;
    fs::write(gen_dir.join("schema_ui_contract_test.g.dart"), defective_contract).expect("write contract");

    let defective_widget_test = r#"
void probeWidgetKey(Key key) {
  find.byKey(key).evaluate().isNotEmpty;
}

void main() {
  testWidgets('mounts HUD without non-overlap assert', (tester) async {
    await tester.pumpWidget(const TacticalHud());
    // Missing assertPairwiseHudNonOverlap or overlaps!
  });
}
"#;
    fs::write(test_dir.join("hud_widget_test.dart"), defective_widget_test).expect("write test");

    let res = check_40_interactive_gesture_and_hud_non_overlap_gate(&ws.root);
    assert!(
        res.is_err(),
        "Check 40 must fail on missing viewports, fake rectangles, and shallow probing"
    );
    let errs = res.unwrap_err().join("\n");
    assert!(errs.contains("6K") || errs.contains("6016") || errs.contains("viewport"), "Expected viewport violation, got:\n{}", errs);
    assert!(errs.contains("evaluate().isNotEmpty") || errs.contains("probeWidgetKey"), "Expected shallow probing violation, got:\n{}", errs);
    assert!(errs.contains("assertPairwiseHudNonOverlap") || errs.contains("overlaps"), "Expected HUD non-overlap violation, got:\n{}", errs);
}

/// Positive probe test for Check 40: Verifies that complete 4-viewport contract test with real gestures passes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H verified UI contract standards.
///
/// ## 2. Invariants & Mathematical Properties
/// - Contract test testing all 4 canonical viewports with real `tester.getRect` and `await tester.tap` passes.
///
/// ## 3. Preconditions
/// - Temporary workspace with complete contract test and compliant HUD test.
///
/// ## 4. Postconditions
/// - Check 40 returns `Ok(())`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) scan.
/// - Space Complexity: O(1) zero violations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts `is_ok()`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-40-POSITIVE-PROBE-COMPLETE-CONTRACT]
#[test]
fn test_check_40_positive_probe_passes_complete_4_viewport_contract() {
    let ws = ScopedTempWorkspace::new("check_40_pos_complete");
    let gen_dir = ws.root.join("app_flutter/test/generated");
    let test_dir = ws.root.join("app_flutter/test/widgets");
    fs::create_dir_all(&gen_dir).expect("create gen dir");
    fs::create_dir_all(&test_dir).expect("create test dir");

    let complete_contract = r#"
// Complete 4-viewport contract test
const kCanonicalViewports = [
  'FHD_1080P_1X',
  'QHD_1440P_1X',
  'UHD_4K_1X',
  'UHD_4K_2X',
  'XDR_6K_1X',
  'XDR_6K_2X',
];

void main() {
  testWidgets('assert zero RenderFlex overflow across all viewports', (tester) async {
    final rect = tester.getRect(find.byKey(const Key('deap_hud_panel')));
    await tester.tap(find.byKey(const Key('deap_action_button')));
    await tester.pump();
  });
}
"#;
    fs::write(gen_dir.join("schema_ui_contract_test.g.dart"), complete_contract).expect("write contract");

    let clean_hud_test = r#"
void main() {
  testWidgets('assert HUD non-overlap', (tester) async {
    await tester.pumpWidget(const TacticalHud());
    assertPairwiseHudNonOverlap(tester);
  });
}
"#;
    fs::write(test_dir.join("clean_hud_test.dart"), clean_hud_test).expect("write test");

    let res = check_40_interactive_gesture_and_hud_non_overlap_gate(&ws.root);
    assert!(
        res.is_ok(),
        "Check 40 must pass for complete 4-viewport contract and HUD non-overlap assertions, got: {:?}",
        res.err()
    );
}

// =========================================================================
// Check 41 Tests: Solver Bridge SSOT & SQLite Seed Parity Gate
// =========================================================================

/// Negative probe test for Check 41: Proves rejection when domain solver crate exists but generated bridge is missing.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D single source of truth kernel integrity mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - When a domain solver crate exists in `crates/` and `app_flutter/lib/` exists,
///   `app_flutter/lib/data/generated/solver_bridge.g.dart` MUST exist on disk.
///
/// ## 3. Preconditions
/// - Temporary workspace with domain solver crate and Flutter lib, but missing solver bridge.
///
/// ## 4. Postconditions
/// - Check 41 returns `Err(Vec<String>)` citing missing generated solver bridge.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) filesystem existence checks.
/// - Space Complexity: O(1) diagnostic.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts error vector mentions missing solver bridge file.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-41-NEGATIVE-PROBE-MISSING-SOLVER-BRIDGE]
#[test]
fn test_check_41_negative_probe_flags_missing_solver_bridge_when_solver_crate_exists() {
    let ws = ScopedTempWorkspace::new("check_41_neg_missing_bridge");
    let solver_crate_dir = ws.root.join("crates/domain-solver-engine/src");
    let app_data_dir = ws.root.join("app_flutter/lib/data");
    fs::create_dir_all(&solver_crate_dir).expect("create solver crate dir");
    fs::create_dir_all(&app_data_dir).expect("create app data dir");

    fs::write(solver_crate_dir.join("lib.rs"), "// solver crate").expect("write lib.rs");
    fs::write(solver_crate_dir.join("solver.rs"), "// solver kernel").expect("write solver.rs");

    // solver_bridge.g.dart is deliberately NOT created!
    let res = check_41_solver_bridge_and_sqlite_seed_parity_gate(&ws.root);
    assert!(
        res.is_err(),
        "Check 41 must fail when domain solver crate exists but solver_bridge.g.dart is missing"
    );
    let errs = res.unwrap_err().join("\n");
    assert!(
        errs.contains("solver_bridge.g.dart") || errs.contains("solver"),
        "Expected missing solver bridge diagnostic, got:\n{}",
        errs
    );
}

/// Negative probe test for Check 41: Rejects client-side duplicate solver arithmetic in hand-written Dart bridge.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A SSOT computational kernel requirements.
/// Prevents mathematical drift between Rust solver algorithms and Dart UI layers.
///
/// ## 2. Invariants & Mathematical Properties
/// - Rejects arithmetic operations (`+`, `-`, `*`, `/`, `.clamp(`) in candidate/scoring methods
///   in Dart bridge files that do not delegate to `dart:ffi`.
///
/// ## 3. Preconditions
/// - Temporary workspace containing a hand-written Dart bridge with client-side solver math.
///
/// ## 4. Postconditions
/// - Check 41 returns `Err(Vec<String>)` citing duplicate solver mathematics.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(B * L) across Dart bridge files.
/// - Space Complexity: O(V) for diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts error vector cites duplicate solver mathematics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-41-NEGATIVE-PROBE-DUPLICATE-SOLVER-ARITHMETIC]
#[test]
fn test_check_41_negative_probe_flags_client_side_arithmetic_in_dart_bridge() {
    let ws = ScopedTempWorkspace::new("check_41_neg_duplicate_math");
    let solver_crate_dir = ws.root.join("crates/domain-solver-engine/src");
    let app_data_dir = ws.root.join("app_flutter/lib/data");
    let app_gen_dir = ws.root.join("app_flutter/lib/data/generated");
    fs::create_dir_all(&solver_crate_dir).expect("create solver crate dir");
    fs::create_dir_all(&app_data_dir).expect("create app data dir");
    fs::create_dir_all(&app_gen_dir).expect("create app gen dir");

    fs::write(solver_crate_dir.join("lib.rs"), "// solver crate").expect("write lib.rs");
    fs::write(solver_crate_dir.join("solver.rs"), "// solver kernel").expect("write solver.rs");
    fs::write(app_gen_dir.join("solver_bridge.g.dart"), "// generated bridge").expect("write gen bridge");

    let duplicate_math_bridge = r#"
class TacticalSolverBridge {
  List<PlanCandidate> generateCandidates() {
    // Defective client-side arithmetic synthesis without FFI delegation:
    final rawScore = alphaWeight * 0.75 + penaltyFactor * 0.25;
    final clamped = rawScore.clamp(0.0, 1.0);
    return [PlanCandidate(score: clamped)];
  }
}
"#;
    fs::write(app_data_dir.join("custom_tactical_bridge.dart"), duplicate_math_bridge).expect("write bridge");

    let res = check_41_solver_bridge_and_sqlite_seed_parity_gate(&ws.root);
    assert!(
        res.is_err(),
        "Check 41 must fail on duplicate solver arithmetic in Dart bridge without FFI delegation"
    );
    let errs = res.unwrap_err().join("\n");
    assert!(
        errs.contains("Duplicate solver mathematics") || errs.contains("arithmetic") || errs.contains("clamp"),
        "Expected duplicate solver math diagnostic, got:\n{}",
        errs
    );
}

/// Negative probe test for Check 41: Rejects missing entity seed parity and legacy telecom types in domain seed strategy.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D bijective data persistence standards.
///
/// ## 2. Invariants & Mathematical Properties
/// - Every node ID in `topology_data.json` must be seeded.
/// - Scans `domain_seed_strategy.dart` even when `seeds/` directory is absent (proves bypass fix).
/// - Rejects legacy telecom types (`CellSector`, `BtsSite`, `NetworkElement`, `CoreRouter`).
/// - Rejects missing canonical tables (`type_relations`, etc.).
///
/// ## 3. Preconditions
/// - Temporary workspace with topology declaring entities, and seed strategy omitting an entity.
///
/// ## 4. Postconditions
/// - Check 41 returns `Err(Vec<String>)` detailing parity and telecom violations.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(E * log S) over topology entities and seed lines.
/// - Space Complexity: O(V) for diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts error vector reports missing entity and telecom type violations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-41-NEGATIVE-PROBE-SEED-PARITY-AND-TELECOM]
#[test]
fn test_check_41_negative_probe_flags_missing_sqlite_seed_parity_and_telecom_types() {
    let ws = ScopedTempWorkspace::new("check_41_neg_seed_parity");
    let assets_dir = ws.root.join("app_flutter/assets");
    let data_dir = ws.root.join("app_flutter/lib/data");
    // Notice: seeds/ directory is deliberately NOT created to prove the old bypass bug is dead!
    fs::create_dir_all(&assets_dir).expect("create assets dir");
    fs::create_dir_all(&data_dir).expect("create data dir");

    let topology_json = r#"{
  "nodes": [
    {"id": "NODE_ALPHA_01", "name": "Alpha Node"},
    {"id": "NODE_BRAVO_MISSING_02", "name": "Bravo Node"}
  ]
}"#;
    fs::write(assets_dir.join("topology_data.json"), topology_json).expect("write topology");

    // Defective seed strategy: seeds NODE_ALPHA_01, misses NODE_BRAVO_MISSING_02,
    // contains legacy telecom type CellSector, and misses table type_relations!
    let defective_seed_strategy = r#"
class DomainSeedStrategy {
  Future<void> seed(Database db) async {
    await db.execute("INSERT INTO type_definitions VALUES ('CellSector')");
    await db.execute("INSERT INTO type_attributes VALUES (...)");
    await db.execute("INSERT INTO instances (id) VALUES ('NODE_ALPHA_01')");
    await db.execute("INSERT INTO properties VALUES (...)");
    // Missing NODE_BRAVO_MISSING_02, missing type_relations table!
  }
}
"#;
    fs::write(data_dir.join("domain_seed_strategy.dart"), defective_seed_strategy).expect("write seed");

    let res = check_41_solver_bridge_and_sqlite_seed_parity_gate(&ws.root);
    assert!(
        res.is_err(),
        "Check 41 must fail when topology entity is missing from seed or legacy telecom types are present"
    );
    let errs = res.unwrap_err().join("\n");
    assert!(errs.contains("NODE_BRAVO_MISSING_02"), "Expected missing entity violation, got:\n{}", errs);
    assert!(errs.contains("CellSector") || errs.contains("telecom"), "Expected telecom violation, got:\n{}", errs);
    assert!(errs.contains("type_relations"), "Expected missing table violation, got:\n{}", errs);
}

/// Positive probe test for Check 41: Verifies that clean C-ABI delegation and bijective seed parity pass.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D persistence and SSOT standards.
///
/// ## 2. Invariants & Mathematical Properties
/// - Thin FFI delegation bridge and 100% bijective seed across all 5 canonical tables pass.
///
/// ## 3. Preconditions
/// - Temporary workspace with compliant FFI bridge and complete SQLite seed.
///
/// ## 4. Postconditions
/// - Check 41 returns `Ok(())`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(E * log S) scan.
/// - Space Complexity: O(1) zero violations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts `is_ok()`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-41-POSITIVE-PROBE-CLEAN-SEED-AND-BRIDGE]
#[test]
fn test_check_41_positive_probe_passes_clean_thin_bridge_and_bijective_seed() {
    let ws = ScopedTempWorkspace::new("check_41_pos_clean");
    let assets_dir = ws.root.join("app_flutter/assets");
    let seeds_dir = ws.root.join("app_flutter/lib/data/seeds");
    let gen_dir = ws.root.join("app_flutter/lib/data/generated");
    fs::create_dir_all(&assets_dir).expect("create assets dir");
    fs::create_dir_all(&seeds_dir).expect("create seeds dir");
    fs::create_dir_all(&gen_dir).expect("create gen dir");

    let topology_json = r#"{
  "nodes": [
    {"id": "SENSOR_01", "name": "Sensor 1"},
    {"id": "CONTROLLER_02", "name": "Controller 2"}
  ]
}"#;
    fs::write(assets_dir.join("topology_data.json"), topology_json).expect("write topology");

    let complete_seed_dart = r#"
Future<void> seedDatabase(Database db) async {
  // 5 canonical tables:
  await db.execute("INSERT INTO type_definitions VALUES (...)");
  await db.execute("INSERT INTO type_attributes VALUES (...)");
  await db.execute("INSERT INTO type_relations VALUES (...)");
  await db.execute("INSERT INTO properties VALUES (...)");
  await db.execute("INSERT INTO instances (id) VALUES ('SENSOR_01'), ('CONTROLLER_02')");
}
"#;
    fs::write(seeds_dir.join("generated_domain_seed.g.dart"), complete_seed_dart).expect("write seed");

    let thin_bridge_dart = r#"
class SolverBridge {
  final DynamicLibrary _dylib;
  SolverBridge(this._dylib);

  Uint8List evaluate(Uint8List payload) {
    final fn = _dylib.lookupFunction<NativeFunction<...>, ...>('deap_evaluate_port');
    return fn(payload);
  }
}
"#;
    fs::write(gen_dir.join("solver_bridge.g.dart"), thin_bridge_dart).expect("write bridge");

    let res = check_41_solver_bridge_and_sqlite_seed_parity_gate(&ws.root);
    assert!(
        res.is_ok(),
        "Check 41 must pass for thin FFI bridge and bijective seed parity, got: {:?}",
        res.err()
    );
}

// =========================================================================
// Check 19 Test: Domain Cleanliness of codebase.rs
// =========================================================================

/// Check 19 domain cleanliness test for `crates/verify-baseline/src/checks/codebase.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to Check 19 (Pure Schema-Driven Compiler Invariant) and Section 1.3 of the Project Constitution.
/// Enforces that upstream verification tooling in `crates/verify-baseline/src/checks/codebase.rs`
/// contains zero hardcoded C-UAS or customer domain tokens.
///
/// ## 2. Invariants & Mathematical Properties
/// - Linear line-by-line inspection of `codebase.rs`.
/// - Rejects customer domain strings outside comments:
///   `"ASSET-"`, `"SECTOR-"`, `"INTERCEPTORS"`, `"RADAR-"`, `"EW-JAM-"`, `"12x CLASS-"`,
///   `"_buildCandidates"`, `"alphaCovOffset"`, `"reserveBoostPct"`, `"extraDensityPenalty"`,
///   `"paretoScore"`, `"violationCount: 0"`, `"bridge.relockInterlock("`,
///   `"bridge.triggerHotlRfJamRetasking("`, `"bridge.restoreStrategicLink("`,
///   `"bridge.setMobileAtakEudMode("`, `"bridge.toggleRadarDegraded("`,
///   `"bridge.setSwarmDensityMultiplier("`, `"bridge.setMinMagazineReserveRatio("`, `"cuas"`.
///
/// ## 3. Preconditions
/// - `crates/verify-baseline/src/checks/codebase.rs` exists on disk.
///
/// ## 4. Postconditions
/// - Returns with zero assertion failures when zero customer domain tokens exist in `codebase.rs`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L * K) where L is lines in `codebase.rs` and K is forbidden token count.
/// - Space Complexity: O(V) for collected violations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts that the violation collection is empty; fails with line numbers if any token is found.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-19-CODEBASE-RS-DOMAIN-CLEANLINESS]
#[test]
fn test_check_19_codebase_rs_zero_customer_domain_tokens() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let codebase_path = manifest_dir.join("src").join("checks").join("codebase.rs");
    let content = fs::read_to_string(&codebase_path)
        .unwrap_or_else(|err| panic!("Unable to read {:?}: {}", codebase_path, err));

    let forbidden_tokens = [
        "ASSET-",
        "SECTOR-",
        "INTERCEPTORS",
        "RADAR-",
        "EW-JAM-",
        "12x CLASS-",
        "_buildCandidates",
        "alphaCovOffset",
        "reserveBoostPct",
        "extraDensityPenalty",
        "paretoScore",
        "violationCount: 0",
        "bridge.relockInterlock(",
        "bridge.triggerHotlRfJamRetasking(",
        "bridge.restoreStrategicLink(",
        "bridge.setMobileAtakEudMode(",
        "bridge.toggleRadarDegraded(",
        "bridge.setSwarmDensityMultiplier(",
        "bridge.setMinMagazineReserveRatio(",
        "cuas",
    ];

    let mut violations = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = line.trim();

        // Skip docstrings and normal comments when evaluating domain token presence
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }

        let lower = trimmed.to_lowercase();
        for &token in &forbidden_tokens {
            let lower_token = token.to_lowercase();
            if lower.contains(&lower_token) {
                violations.push(format!(
                    "codebase.rs:{}: Hardcoded customer domain token '{}' found: '{}'",
                    line_no, token, trimmed
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Check 19 violation: codebase.rs contains hardcoded customer domain tokens:\n{}",
        violations.join("\n")
    );
}

// =========================================================================
// Anti-Regression Unit Tests for Checks 38, 39, 40, and 41
// =========================================================================

/// Positive probe test for Check 38: Verifies that Dart imports and package directives are not falsely flagged.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H lexical precision mandates.
/// Verifies that module import paths containing tokens like 'layout' or 'out' are never misidentified as domain entities.
///
/// ## 2. Invariants & Mathematical Properties
/// - Package imports and library directives starting with `import `, `export `, or `part ` are strictly bypassed.
/// - Generic schema keywords (`in`, `out`, `layout`, `model`) are not harvested as domain entity tokens.
///
/// ## 3. Preconditions
/// - Temporary workspace containing SysML schema with `port out` and a presentation Dart file with layout imports.
///
/// ## 4. Postconditions
/// - Check 38 returns `Ok(())` with zero false-positive diagnostics.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) scanning lines of clean Dart source.
/// - Space Complexity: O(1) zero violations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts `is_ok()`; prints any unexpected violation strings if regression occurs.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-38-IMPORT-DIRECTIVES-BYPASS]
#[test]
fn test_check_38_positive_probe_does_not_fail_on_dart_imports() {
    let ws = ScopedTempWorkspace::new("check_38_pos_imports");
    let pipeline_dir = ws.root.join(".pipeline");
    let feature_dir = ws.root.join("app_flutter/lib/features/layout");
    fs::create_dir_all(&pipeline_dir).expect("create pipeline dir");
    fs::create_dir_all(&feature_dir).expect("create feature dir");

    let schema_sysml = r#"
package Model {
    part def Controller {
        port in cmd_in;
        port out telemetry_out;
    }
}
"#;
    fs::write(pipeline_dir.join("schema.sysml"), schema_sysml).expect("write schema");

    let clean_imports_dart = r#"
import 'package:flutter/material.dart';
import 'package:cuas_coa_app/features/layout/breadcrumbs.dart';
import 'package:app_flutter/features/layout/split_workspace.dart';
export 'package:app_flutter/features/layout/split_workspace.dart';
part 'workspace_layout_helpers.dart';

class SplitWorkspaceView extends StatelessWidget {
  final WorkspaceViewModel vm;
  const SplitWorkspaceView({required this.vm, super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      child: Text(vm.displayTitle),
    );
  }
}
"#;
    fs::write(feature_dir.join("split_workspace_view.dart"), clean_imports_dart).expect("write dart");

    let res = check_38_presentation_anti_hardcoding_ast_gate(&ws.root);
    assert!(
        res.is_ok(),
        "Check 38 must not fail on Dart import/export/part directives with 'layout' or 'out', got: {:?}",
        res.err()
    );
}

/// Negative probe test for Check 38: Flags interpolated strings embedding dynamic entity stems or telemetry units.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H anti-hardcoding mandates.
/// Prevents evasion of presentation gates via string interpolation syntax (e.g. `'Asset-Alpha ($index)'`).
///
/// ## 2. Invariants & Mathematical Properties
/// - String literals containing dynamic entity stems (e.g. `SENSOR-NODE-ALPHA`) or hardcoded telemetry (e.g. `15km`, `75%`)
///   are rejected even when Dart string interpolation (`$var` or `${expr}`) is present in the literal.
///
/// ## 3. Preconditions
/// - Temporary workspace containing topology JSON with known entity IDs and Dart presentation using interpolated hardcoding.
///
/// ## 4. Postconditions
/// - Check 38 returns `Err(Vec<String>)` identifying both the hardcoded entity stem and the hardcoded telemetry unit.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) scanning lines.
/// - Space Complexity: O(V) for diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts error vector contains references to both `SENSOR-NODE-ALPHA` and the telemetry pattern.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-38-NEGATIVE-PROBE-INTERPOLATED-STRINGS]
#[test]
fn test_check_38_negative_probe_flags_interpolated_strings_with_entity_stems_or_telemetry() {
    let ws = ScopedTempWorkspace::new("check_38_neg_interpolated");
    let assets_dir = ws.root.join("app_flutter/assets");
    let feature_dir = ws.root.join("app_flutter/lib/features/tactical");
    fs::create_dir_all(&assets_dir).expect("create assets dir");
    fs::create_dir_all(&feature_dir).expect("create feature dir");

    let topology_json = r#"{
  "nodes": [
    {"id": "SENSOR-NODE-ALPHA", "name": "Sensor Alpha", "type": "Radar"}
  ]
}"#;
    fs::write(assets_dir.join("topology_data.json"), topology_json).expect("write topology");

    let defective_interpolated_dart = r#"
class InterpolatedTacticalView extends StatelessWidget {
  final int index;
  final String status;
  const InterpolatedTacticalView({required this.index, required this.status, super.key});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text('SENSOR-NODE-ALPHA ($index)'),
        Text('Target Range: 15km ($status)'),
        Text('Battery: 75% (${status.toUpperCase()})'),
      ],
    );
  }
}
"#;
    fs::write(feature_dir.join("interpolated_tactical_view.dart"), defective_interpolated_dart).expect("write dart");

    let res = check_38_presentation_anti_hardcoding_ast_gate(&ws.root);
    assert!(
        res.is_err(),
        "Check 38 must flag interpolated strings containing entity stems or telemetry units"
    );
    let errs = res.unwrap_err().join("\n");
    assert!(
        errs.contains("SENSOR-NODE-ALPHA"),
        "Expected entity stem diagnostic in interpolated string, got:\n{}",
        errs
    );
    assert!(
        errs.contains("15km") || errs.contains("distance unit"),
        "Expected distance telemetry diagnostic in interpolated string, got:\n{}",
        errs
    );
    assert!(
        errs.contains("75%") || errs.contains("percentage unit"),
        "Expected percentage telemetry diagnostic in interpolated string, got:\n{}",
        errs
    );
}

/// Negative probe test for Check 39: Catches direct controller mutations following an inner conditional block.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Section 1.9 of the Project Constitution.
/// Ensures AST block tracking maintains `testWidgets` scope across nested control flow blocks (`if`, `for`, `while`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Direct mutator calls on `*Controller`, `*Bridge`, or `*Notifier` occurring after an inner closing brace `}`
///   must not escape `testWidgets` scope detection.
///
/// ## 3. Preconditions
/// - Temporary workspace containing a test file where an inner `if (...) { ... }` block precedes a direct controller mutation.
///
/// ## 4. Postconditions
/// - Check 39 returns `Err(Vec<String>)` flagging the direct mutation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) scanning test lines.
/// - Space Complexity: O(V) for diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts error vector contains direct mutation diagnostic for the statement after the inner block.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-39-NEGATIVE-PROBE-MUTATION-AFTER-INNER-BLOCK]
#[test]
fn test_check_39_negative_probe_catches_direct_mutations_after_inner_if_block() {
    let ws = ScopedTempWorkspace::new("check_39_neg_inner_block");
    let test_dir = ws.root.join("app_flutter/test/features");
    fs::create_dir_all(&test_dir).expect("create test dir");

    let defective_test = r#"
void main() {
  testWidgets('test with inner block preceding direct mutation', (tester) async {
    await tester.pumpWidget(const MyApp());
    final shouldInitialize = true;
    if (shouldInitialize) {
      debugPrint('Initializing inner state');
    }
    // Direct mutation after inner if block closing brace:
    tacticalController.setInterlockEngaged(true);
    expect(find.text('ENGAGED'), findsOneWidget);
  });
}
"#;
    fs::write(test_dir.join("inner_block_mutation_test.dart"), defective_test).expect("write test");

    let res = check_39_zero_mocking_test_ast_gate(&ws.root);
    assert!(
        res.is_err(),
        "Check 39 must catch direct controller mutations occurring after an inner if block"
    );
    let errs = res.unwrap_err().join("\n");
    assert!(
        errs.contains("tacticalController") || errs.contains("setInterlockEngaged"),
        "Expected direct mutation diagnostic after inner if block, got:\n{}",
        errs
    );
}

/// Negative probe test for Check 40: Rejects UI contract tests lacking `tester.getRect` bounds queries or `tester.tap` gestures.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H anti-facade verification mandates.
/// Mechanically rejects hollow contract tests that define viewports but omit live RenderBox queries or interactive gestures.
///
/// ## 2. Invariants & Mathematical Properties
/// - Contract test lacking `tester.getRect(` is rejected with missing RenderBox bounds query diagnostic.
/// - Contract test lacking `tester.tap(` is rejected with missing interactive gesture diagnostic.
///
/// ## 3. Preconditions
/// - Temporary workspace containing valid Flutter lib, SysML schema, and hollow contract test file.
///
/// ## 4. Postconditions
/// - Check 40 returns `Err(Vec<String>)` detailing absence of both `tester.getRect` and `tester.tap`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) scanning contract test file lines.
/// - Space Complexity: O(V) for diagnostics.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts error vector contains diagnostics for both missing `tester.getRect` and missing `tester.tap`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-40-NEGATIVE-PROBE-MISSING-GESTURES-AND-RECTS]
#[test]
fn test_check_40_negative_probe_rejects_contract_test_lacking_get_rect_or_tap() {
    let ws = ScopedTempWorkspace::new("check_40_neg_no_gestures");
    let gen_dir = ws.root.join("app_flutter/test/generated");
    let lib_dir = ws.root.join("app_flutter/lib");
    let pipeline_dir = ws.root.join(".pipeline");
    fs::create_dir_all(&gen_dir).expect("create gen dir");
    fs::create_dir_all(&lib_dir).expect("create lib dir");
    fs::create_dir_all(&pipeline_dir).expect("create pipeline dir");

    fs::write(pipeline_dir.join("schema.sysml"), "package Model {}").expect("write schema");
    fs::write(lib_dir.join("main.dart"), "void main() {}").expect("write main");

    // Hollow contract test: has canonical viewports, but zero tester.getRect and zero tester.tap!
    let hollow_contract = r#"
const kCanonicalViewports = [
  'FHD_1080P_1X',
  'QHD_1440P_1X',
  'UHD_4K_1X',
  'UHD_4K_2X',
  'XDR_6K_1X',
  'XDR_6K_2X',
];

void main() {
  testWidgets('hollow contract test lacking live geometry and gestures', (tester) async {
    await tester.pumpWidget(const Placeholder());
    expect(1, equals(1));
  });
}
"#;
    fs::write(gen_dir.join("schema_ui_contract_test.g.dart"), hollow_contract).expect("write contract");

    let res = check_40_interactive_gesture_and_hud_non_overlap_gate(&ws.root);
    assert!(
        res.is_err(),
        "Check 40 must reject contract test lacking live tester.getRect queries and tester.tap gestures"
    );
    let errs = res.unwrap_err().join("\n");
    assert!(
        errs.contains("tester.getRect"),
        "Expected missing tester.getRect diagnostic, got:\n{}",
        errs
    );
    assert!(
        errs.contains("tester.tap"),
        "Expected missing tester.tap diagnostic, got:\n{}",
        errs
    );
}

/// Positive probe test for Check 41: Verifies that client-side comments mentioning arithmetic do not trigger false violations.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D single source of truth kernel integrity mandates.
/// Verifies that explanatory line comments containing arithmetic tokens are cleanly ignored.
///
/// ## 2. Invariants & Mathematical Properties
/// - Discards `//` line comments before scanning for client-side arithmetic (`.clamp(`, float multiplication, etc.).
///
/// ## 3. Preconditions
/// - Temporary workspace with thin C-ABI bridge containing explanatory comments mentioning `.clamp` and float math.
///
/// ## 4. Postconditions
/// - Check 41 returns `Ok(())`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) scanning bridge lines.
/// - Space Complexity: O(1) zero violations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts `is_ok()`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-41-LINE-COMMENT-STRIPPING]
#[test]
fn test_check_41_positive_probe_ignores_arithmetic_in_line_comments() {
    let ws = ScopedTempWorkspace::new("check_41_pos_comments");
    let solver_crate_dir = ws.root.join("crates/domain-solver-engine/src");
    let assets_dir = ws.root.join("app_flutter/assets");
    let seeds_dir = ws.root.join("app_flutter/lib/data/seeds");
    let gen_dir = ws.root.join("app_flutter/lib/data/generated");
    let data_dir = ws.root.join("app_flutter/lib/data");
    fs::create_dir_all(&solver_crate_dir).expect("create solver crate dir");
    fs::create_dir_all(&assets_dir).expect("create assets dir");
    fs::create_dir_all(&seeds_dir).expect("create seeds dir");
    fs::create_dir_all(&gen_dir).expect("create gen dir");
    fs::create_dir_all(&data_dir).expect("create data dir");

    fs::write(solver_crate_dir.join("lib.rs"), "// solver crate").expect("write lib.rs");
    fs::write(solver_crate_dir.join("solver.rs"), "// solver kernel").expect("write solver.rs");
    fs::write(gen_dir.join("solver_bridge.g.dart"), "// generated bridge").expect("write gen bridge");

    let topology_json = r#"{
  "nodes": [
    {"id": "NODE_ALPHA", "name": "Node Alpha"}
  ]
}"#;
    fs::write(assets_dir.join("topology_data.json"), topology_json).expect("write topology");

    let seed_dart = r#"
Future<void> seedDatabase(Database db) async {
  await db.execute("INSERT INTO type_definitions VALUES (...)");
  await db.execute("INSERT INTO type_attributes VALUES (...)");
  await db.execute("INSERT INTO type_relations VALUES (...)");
  await db.execute("INSERT INTO properties VALUES (...)");
  await db.execute("INSERT INTO instances (id) VALUES ('NODE_ALPHA')");
}
"#;
    fs::write(seeds_dir.join("generated_domain_seed.g.dart"), seed_dart).expect("write seed");

    let bridge_with_arithmetic_comments = r#"
import 'dart:ffi';

class SolverBridge {
  final DynamicLibrary _dylib;
  SolverBridge(this._dylib);

  // In native solver kernel, raw score is clamped via .clamp(0.0, 1.0) and multiplied by 0.8
  // Note: generateCandidates is performed entirely in native Rust
  Uint8List evaluate(Uint8List payload) {
    final fn = _dylib.lookupFunction<NativeFunction<Pointer<Uint8> Function(Pointer<Uint8>, Int32)>, Pointer<Uint8> Function(Pointer<Uint8>, int)>('deap_evaluate_port');
    return Uint8List(0);
  }
}
"#;
    fs::write(data_dir.join("tactical_solver_bridge.dart"), bridge_with_arithmetic_comments).expect("write bridge");

    let res = check_41_solver_bridge_and_sqlite_seed_parity_gate(&ws.root);
    assert!(
        res.is_ok(),
        "Check 41 must ignore arithmetic tokens inside line comments, got: {:?}",
        res.err()
    );
}

// =========================================================================
// Meta-Test: DO-178C Level A 7-Section Safety Docstring Completeness
// =========================================================================

/// Meta-test asserting that every function in `codebase_checks_tests.rs` carries a complete 7-section docstring.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and Check 36 docstring governance rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Every `fn ` declaration must be preceded by docstring lines containing:
///   `## 1. Safety Intent`, `## 2. Invariants`, `## 3. Preconditions`, `## 4. Postconditions`,
///   `## 5. Algorithmic Complexity`, `## 6. Failure Modes`, `## 7. Realization & Traceability`,
///   and `Realises: [`.
///
/// ## 3. Preconditions
/// - `codebase_checks_tests.rs` exists on disk.
///
/// ## 4. Postconditions
/// - Asserts zero missing sections across all test functions and structs.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) scanning source lines.
/// - Space Complexity: O(V) for collected violations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Asserts empty violation list; reports missing sections with function names and line numbers.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CODEBASE-TESTS-DOCSTRING-COMPLETENESS]
#[test]
fn test_codebase_checks_tests_docstring_completeness() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_file = manifest_dir.join("tests").join("codebase_checks_tests.rs");
    let content = fs::read_to_string(&test_file)
        .unwrap_or_else(|err| panic!("Unable to read {:?}: {}", test_file, err));

    let required_sections = [
        "## 1. Safety Intent & Regulatory Scope",
        "## 2. Invariants & Mathematical Properties",
        "## 3. Preconditions",
        "## 4. Postconditions",
        "## 5. Algorithmic Complexity Bounds",
        "## 6. Failure Modes & Fault Tolerance",
        "## 7. Realization & Traceability",
    ];

    let lines: Vec<&str> = content.lines().collect();
    let mut violations = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if (trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ")) && !trimmed.contains("main()") {
            let fn_name = trimmed.split('(').next().unwrap_or(trimmed);
            let line_no = idx.saturating_add(1);

            // Look back for docstring comments
            let mut doc_lines = Vec::new();
            let mut lookback = idx;
            while lookback > 0 {
                lookback = lookback.saturating_sub(1);
                let prev = lines[lookback].trim();
                if prev.starts_with("///") || prev.starts_with("//!") {
                    doc_lines.push(prev);
                } else if prev.starts_with("#[") {
                    // Skip attributes like #[test]
                    continue;
                } else {
                    break;
                }
            }

            let full_doc = doc_lines.join("\n");
            for section in &required_sections {
                if !full_doc.contains(section) {
                    violations.push(format!(
                        "codebase_checks_tests.rs:{}: function '{}' missing section '{}'",
                        line_no, fn_name, section
                    ));
                }
            }

            if !full_doc.contains("Realises: [") {
                violations.push(format!(
                    "codebase_checks_tests.rs:{}: function '{}' missing non-empty 'Realises: [...]' tag",
                    line_no, fn_name
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Docstring completeness violations in codebase_checks_tests.rs:\n{}",
        violations.join("\n")
    );
}
