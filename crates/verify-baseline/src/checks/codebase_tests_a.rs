//! Unit tests for Checks 38, 39, and 40 (presentation anti-hardcoding, zero mocking, and UI contracts).
//! Realises: [REQ-VERIFY-BASELINE-CODEBASE-CHECKS]

use std::fs;
use std::path::PathBuf;

use super::codebase::*;

pub(super) struct ScopedTempDir {
    pub(super) path: PathBuf,
}

impl ScopedTempDir {
    pub(super) fn new(prefix: &str) -> Result<Self, std::io::Error> {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("test_codebase_{}_{}_{}", prefix, nanos, count));
        fs::create_dir_all(&dir)?;
        Ok(Self { path: dir })
    }
}

impl Drop for ScopedTempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_check_38_positive_clean_presentation() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("check_38_pos")?;
    let feature_dir = tmp.path.join("app_flutter/lib/features/workspace");
    fs::create_dir_all(&feature_dir)?;

    let clean_dart = r#"
class TacticalWorkspaceView extends StatelessWidget {
  final WorkspaceViewModel vm;
  const TacticalWorkspaceView({required this.vm, super.key});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(vm.displayTitle),
        Text(vm.activeTelemetrySummary),
        EntityRow(
          label: vm.selectedEntityLabel,
          isActive: vm.isEntityActive,
        ),
      ],
    );
  }
}
"#;
    fs::write(feature_dir.join("workspace_view.dart"), clean_dart)?;

    let res = check_38_presentation_anti_hardcoding_ast_gate(&tmp.path);
    assert!(
        res.is_ok(),
        "Positive clean presentation must pass: {:?}",
        res.err()
    );
    Ok(())
}

#[test]
fn test_check_38_negative_hardcoded_literals() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("check_38_neg")?;
    let assets_dir = tmp.path.join("app_flutter/assets");
    let feature_dir = tmp.path.join("app_flutter/lib/features/tactical");
    fs::create_dir_all(&assets_dir)?;
    fs::create_dir_all(&feature_dir)?;

    let topology_json = r#"{
  "nodes": [
    {"id": "NODE_ALPHA_01", "name": "Alpha Node", "type": "Sensor"}
  ]
}"#;
    fs::write(assets_dir.join("topology_data.json"), topology_json)?;

    let defective_dart = r#"
class DefectiveView extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        const Text('NODE_ALPHA_01'),
        const Text('RULES: 0 VIOLATIONS'),
        EntityItem(isActive: true),
      ],
    );
  }
}
"#;
    fs::write(feature_dir.join("defective_view.dart"), defective_dart)?;

    let res = check_38_presentation_anti_hardcoding_ast_gate(&tmp.path);
    assert!(
        res.is_err(),
        "Defective hardcoded presentation must fail Check 38"
    );
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs.iter().any(|e| e.contains("NODE_ALPHA_01")));
    assert!(errs.iter().any(|e| e.contains("RULES: 0 VIOLATIONS")));
    assert!(errs.iter().any(|e| e.contains("isActive: true")));
    Ok(())
}

#[test]
fn test_check_39_positive_clean_persistence_test() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("check_39_pos")?;
    let test_dir = tmp.path.join("app_flutter/test/data");
    fs::create_dir_all(&test_dir)?;

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

  test('valid live persistence test', () async {
    final rows = await db.query('instances');
    expect(rows, isNotEmpty);
  });
}
"#;
    fs::write(test_dir.join("live_db_test.dart"), clean_test)?;

    let res = check_39_zero_mocking_test_ast_gate(&tmp.path);
    assert!(
        res.is_ok(),
        "Clean persistence test must pass Check 39: {:?}",
        res.err()
    );
    Ok(())
}

#[test]
fn test_check_39_negative_mocking_and_leaking_db() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("check_39_neg")?;
    let test_dir = tmp.path.join("app_flutter/test/features");
    fs::create_dir_all(&test_dir)?;

    let defective_test = r#"
class MockDataRepository extends Mock implements TreeRepository {}
class _FakeTacticalThemeService extends Fake {}

void main() {
  setUpAll(() async {
    await openDatabase(':memory:');
    // Notice missing tearDownAll!
  });

  test('mocked test', () {
    final emptyRes = const Result.success([]);
    expect(emptyRes, isNotNull);
  });
}
"#;
    fs::write(test_dir.join("mock_test.dart"), defective_test)?;

    let res = check_39_zero_mocking_test_ast_gate(&tmp.path);
    assert!(
        res.is_err(),
        "Mocking and leaking SQLite must fail Check 39"
    );
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs
        .iter()
        .any(|e| e.contains("Mock") || e.contains("mocking")));
    assert!(errs.iter().any(|e| e.contains("Result.success")));
    assert!(errs.iter().any(|e| e.contains("tearDownAll")));
    Ok(())
}

#[test]
fn test_check_40_positive_ui_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("check_40_pos")?;
    let test_gen_dir = tmp.path.join("app_flutter/test/generated");
    let test_dir = tmp.path.join("app_flutter/test/widgets");
    fs::create_dir_all(&test_gen_dir)?;
    fs::create_dir_all(&test_dir)?;

    let generated_contract = r#"
// Generated schema UI contract test
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
  testWidgets('assert zero RenderFlex overflow across all viewports', (tester) async {
    final rect = tester.getRect(find.byKey(const ValueKey('hud')));
    assertPairwiseHudNonOverlap([rect, rect]);
    await tester.tap(find.byKey(const ValueKey('row')));
  });
}
"#;
    fs::write(
        test_gen_dir.join("schema_ui_contract_test.g.dart"),
        generated_contract,
    )?;

    let interactive_test = r#"
void main() {
  testWidgets('interactive tap test with HUD non-overlap', (tester) async {
    await tester.tap(find.byKey(const Key('tactical_control_button')));
    await tester.pumpAndSettle();
    assertPairwiseHudNonOverlap(tester);
  });
}
"#;
    fs::write(test_dir.join("interactive_test.dart"), interactive_test)?;

    let res = check_40_interactive_gesture_and_hud_non_overlap_gate(&tmp.path);
    assert!(
        res.is_ok(),
        "Positive contract test must pass Check 40: {:?}",
        res.err()
    );
    Ok(())
}

#[test]
fn test_check_40_negative_shallow_probing_and_direct_bridge_mutation(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("check_40_neg")?;
    let test_gen_dir = tmp.path.join("app_flutter/test/generated");
    let test_dir = tmp.path.join("app_flutter/test/widgets");
    fs::create_dir_all(&test_gen_dir)?;
    fs::create_dir_all(&test_dir)?;

    let incomplete_contract = r#"
const kViewports = [
  'FHD_1080P_1X',
  'QHD_1440P_1X',
  'UHD_4K_1X',
  'UHD_4K_2X',
  'XDR_6K_1X',
];
"#;
    fs::write(
        test_gen_dir.join("schema_ui_contract_test.g.dart"),
        incomplete_contract,
    )?;

    let defective_test = r#"
void probeWidgetKey(Key key) {
  find.byKey(key).evaluate().isNotEmpty;
}

void main() {
  testWidgets('defective test mounting TacticalHud without non-overlap assert', (tester) async {
    await tester.pumpWidget(const TacticalHud());
    bridge.relockInterlock();
  });
}
"#;
    fs::write(test_dir.join("defective_widget_test.dart"), defective_test)?;

    let res = check_40_interactive_gesture_and_hud_non_overlap_gate(&tmp.path);
    assert!(
        res.is_err(),
        "Incomplete viewports and shallow probing must fail Check 40"
    );
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs.iter().any(|e| e.contains("XDR_6K_2X")));
    assert!(errs.iter().any(|e| e.contains("probeWidgetKey")));
    assert!(errs.iter().any(|e| e.contains("bridge.relockInterlock")));
    assert!(errs
        .iter()
        .any(|e| e.contains("assertPairwiseHudNonOverlap")));
    Ok(())
}
