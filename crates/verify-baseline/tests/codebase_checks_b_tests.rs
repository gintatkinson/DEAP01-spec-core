//! Integration tests for Check 39 (zero-mocking live SQLite persistence) and Check 40 (interactive gesture and HUD non-overlap) gates (`codebase.rs`).

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use verify_baseline::checks::codebase::{
    check_39_zero_mocking_test_ast_gate, check_40_interactive_gesture_and_hud_non_overlap_gate,
};

static COUNTER: AtomicU64 = AtomicU64::new(1);

/// Scoped temporary directory that removes itself on drop.
struct ScopedTempDir {
    path: PathBuf,
}

impl ScopedTempDir {
    /// Creates a new isolated temporary directory for test execution.
    fn new(prefix: &str) -> Self {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "deap_verify_codebase_b_it_{}_{}_{}",
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
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Verifies that Check 39 flags Dart 3 class modifiers on mocks/fakes/stubs, generic `*Repository`/`*Service`
/// `implements` clauses, commented-out `tearDownAll`, tautological `expect(true, isTrue)`, and direct
/// controller/bridge mutations inside `testWidgets` without a `tester.` gesture.
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
    fs::write(
        test_dir.join("defective_persistence_test.dart"),
        defective_dart,
    )?;

    let res = check_39_zero_mocking_test_ast_gate(&tmp.path);
    assert!(res.is_err(), "Check 39 must reject Dart 3 mocks, commented teardown, tautologies, and gesture-free mutations");
    let errs = res.err().ok_or("expected Check 39 violations")?;
    assert!(
        errs.iter().any(|e| e.contains("MockEntityStore")),
        "Must flag final class MockEntityStore: {:?}",
        errs
    );
    assert!(
        errs.iter().any(|e| e.contains("AlphaEntityRepository")),
        "Must flag implements AlphaEntityRepository: {:?}",
        errs
    );
    assert!(
        errs.iter().any(|e| e.contains("BetaDomainService")),
        "Must flag implements BetaDomainService: {:?}",
        errs
    );
    assert!(
        errs.iter().any(|e| e.contains("tearDownAll")),
        "Must flag commented-out tearDownAll: {:?}",
        errs
    );
    assert!(
        errs.iter().any(|e| e.contains("expect(true, isTrue)")),
        "Must flag tautological expect(true, isTrue): {:?}",
        errs
    );
    assert!(
        errs.iter()
            .any(|e| e.contains("controller.mutateEntityState")),
        "Must flag gesture-free controller mutation: {:?}",
        errs
    );
    Ok(())
}

/// Verifies that Check 39 passes a clean SQLite test suite with uncommented `tearDownAll` and real `tester.tap` gestures.
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
    assert!(
        res.is_ok(),
        "Compliant SQLite + gesture test must pass Check 39: {:?}",
        res.err()
    );
    Ok(())
}

/// Verifies that Check 40 fails closed when `schema_ui_contract_test.g.dart` is missing or has commented-out
/// `tester.tap`/`Rect.fromLTWH`, and flags domain-agnostic direct `bridge.<method>(` mutations without `tester.` gestures.
#[test]
fn test_check_40_negative_probe_fail_closed_on_missing_or_commented_contract_and_direct_bridge_mutation(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("c40_neg_probe");
    let test_dir = tmp.path.join("app_flutter/test/widgets");
    fs::create_dir_all(&test_dir)?;

    let res_missing = check_40_interactive_gesture_and_hud_non_overlap_gate(&tmp.path);
    assert!(
        res_missing.is_err(),
        "Missing schema_ui_contract_test.g.dart must fail closed"
    );
    let errs_missing = res_missing.err().ok_or("expected missing contract error")?;
    assert!(
        errs_missing
            .iter()
            .any(|e| e.contains("schema_ui_contract_test.g.dart")),
        "Must report missing schema_ui_contract_test.g.dart: {:?}",
        errs_missing
    );

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
    fs::write(
        gen_dir.join("schema_ui_contract_test.g.dart"),
        commented_contract,
    )?;

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
    assert!(
        res_commented.is_err(),
        "Commented contract and direct bridge mutation must fail Check 40"
    );
    let errs = res_commented.err().ok_or("expected Check 40 violations")?;
    assert!(
        errs.iter().any(|e| e.contains("tester.tap(")),
        "Must flag missing uncommented tester.tap(: {:?}",
        errs
    );
    assert!(
        errs.iter().any(|e| e.contains("tester.getRect(")),
        "Must flag missing uncommented tester.getRect(: {:?}",
        errs
    );
    assert!(
        errs.iter().any(|e| e.contains("Rect.fromLTWH(")),
        "Must flag synthetic Rect.fromLTWH(: {:?}",
        errs
    );
    assert!(
        errs.iter()
            .any(|e| e.contains("bridge.applyGenericParameterMutation(")),
        "Must flag generic direct bridge mutation: {:?}",
        errs
    );
    Ok(())
}

/// Verifies that Check 40 passes when `schema_ui_contract_test.g.dart` contains all 6 viewports,
/// all 4 physical resolutions, live `tester.getRect(`, `tester.tap(`, `assertPairwiseHudNonOverlap`, and `.overlaps(`.
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
    fs::write(
        gen_dir.join("schema_ui_contract_test.g.dart"),
        valid_contract,
    )?;

    let res = check_40_interactive_gesture_and_hud_non_overlap_gate(&tmp.path);
    assert!(
        res.is_ok(),
        "Complete 4-resolution, 6-viewport contract must pass Check 40: {:?}",
        res.err()
    );
    Ok(())
}
