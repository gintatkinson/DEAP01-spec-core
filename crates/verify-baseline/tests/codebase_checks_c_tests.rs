//! Integration tests for Check 38 (presentation anti-hardcoding AST gate) and Check 17 (STPA/FMECA closure) (`codebase.rs` and `safety_stpa.rs`).

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use verify_baseline::checks::codebase::check_38_presentation_anti_hardcoding_ast_gate;
use verify_baseline::checks::safety_stpa::{check_safety_integrity, SafetyCheckOutcome};

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
            "deap_verify_codebase_c_it_{}_{}_{}",
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

/// Verifies that Check 38 (`check_38_presentation_anti_hardcoding_ast_gate`) rejects all 19 structural
/// presentation hardcoding patterns.
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

/// Verifies that Check 17 (`check_safety_integrity`) Pillar 7 FMECA bidirectional AST closure
/// passes when `schema/` contains both an architectural domain model (`schema/domain.sysml`)
/// and code realization schema files (`schema/solver_realization.sysml` / `*_Realization` packages),
/// without failing on realization-only code structs.
#[test]
fn test_issue_472_check17_fmeca_closure_ignores_realization_only_code_parts(
) -> Result<(), Box<dyn std::error::Error>> {
    let ws = ScopedTempDir::new("check17_realization");
    let schema_dir = ws.path.join("schema");
    let safety_dir = ws.path.join("docs").join("safety");
    fs::create_dir_all(&schema_dir)?;
    fs::create_dir_all(&safety_dir)?;

    let domain_model = r#"package TacticalDomain {
    part def TacticalSolver;
    part def CockpitHud;
}
"#;
    fs::write(schema_dir.join("domain.sysml"), domain_model)?;

    let realization_model = r#"package solver_src_Realization {
    part def TacticalSolver;
    part def InternalDtoHelper;
    part def WidgetStateHelper;
}
"#;
    fs::write(
        schema_dir.join("solver_realization.sysml"),
        realization_model,
    )?;

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
    stpa.push_str(
        "## Commercial Toolchain Hooks\n- MATLAB / Simulink / Stateflow verification hooks\n",
    );

    fs::write(safety_dir.join("STPA_MATRIX.md"), stpa)?;

    let res = check_safety_integrity(&ws.path, false, true);
    assert_eq!(
        res,
        Ok(SafetyCheckOutcome::DownstreamVerified),
        "Expected Check 17 to pass when domain components are covered and realization-only structs are in *_realization.sysml, got: {:?}",
        res
    );
    Ok(())
}
