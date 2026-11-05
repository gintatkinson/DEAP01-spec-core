//! Integration tests for deterministic Dart solver bridge codegen (`flutter_codegen.rs`).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D verification mandates.
//! Verifies that `emit_dart_solver_bridge` and `generate_solver_bridge_dart` synthesize
//! strongly-typed port and item DTOs (`toJson` / `fromJson`) and `--solve-json` transport.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Every `ItemDef` and `PortDef` with resolved attributes emits typed Dart fields (`double`, `int`, `bool`, `String`).
//! - `DeapNativeSolverIsolate` provides both `evaluateRequest` (C-ABI byte buffer) and `solveJson` (`--solve-json` CLI transport).
//!
//! ## 3. Preconditions
//! - Temporary test directories are writable and cleaned up on drop.
//!
//! ## 4. Postconditions
//! - Asserts that `solver_bridge.g.dart` is written and contains typed DTOs and `--solve-json` invocation.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(P + I + A)$ linear in AST ports, items, and attributes.
//! - Space Complexity: $O(P + I + A)$ bounded temporary test buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Panics only inside `#[test]` harnesses when contract assertions fail.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-COMPILER-FLUTTER-CODEGEN], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]

use compile_sysml::flutter_codegen::{
    emit_dart_solver_bridge, generate_domain_seed_dart, generate_schema_models_dart,
    generate_solver_bridge_dart, generate_ui_contract_test_dart, TopologyNode,
};
use deap_core::sysml_ast::{AttributeDef, ItemDef, PartDef, PortDef, SysmlModel};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(1);

/// Scoped temporary directory that removes itself on drop.
/// Realises: [REQ-FLUTTER-CODEGEN-TEST-FIXTURE]
struct ScopedTempDir {
    path: PathBuf,
}

impl ScopedTempDir {
    /// Creates a new isolated temporary directory for test execution.
    /// Realises: [REQ-FLUTTER-CODEGEN-TEST-FIXTURE]
    fn new(prefix: &str) -> Self {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "deap_flutter_codegen_it_{}_{}_{}",
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
    /// Realises: [REQ-FLUTTER-CODEGEN-TEST-FIXTURE]
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Constructs an `AttributeDef` with optional default value for Flutter codegen tests.
/// Realises: [REQ-COMPILER-FLUTTER-CODEGEN]
fn make_attr(name: &str, type_name: &str, default_value: Option<&str>) -> AttributeDef {
    AttributeDef {
        name: name.to_string(),
        type_name: type_name.to_string(),
        default_value: default_value.map(str::to_string),
        doc: None,
    }
}

/// Constructs a `TopologyNode` fixture instance with standard timestamps.
/// Realises: [REQ-COMPILER-FLUTTER-CODEGEN]
fn make_node(
    id: &str,
    type_name: &str,
    label: &str,
    parent_id: Option<&str>,
    properties: BTreeMap<String, serde_json::Value>,
) -> TopologyNode {
    TopologyNode {
        id: id.to_string(),
        type_name: type_name.to_string(),
        label: label.to_string(),
        parent_id: parent_id.map(str::to_string),
        properties,
        created_at: 1000,
        updated_at: 2000,
    }
}

#[test]
fn test_emit_dart_solver_bridge_typed_dtos_and_solve_json_transport(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempDir::new("solver_bridge");
    let item_def = ItemDef {
        name: "SolverTelemetryPayload".to_string(),
        attributes: vec![
            make_attr("composite_score", "Real", Some("0.0")),
            make_attr("iteration_count", "Integer", Some("0")),
            make_attr("feasible_flag", "Boolean", Some("true")),
            make_attr("status_code", "String", Some("READY")),
        ],
        ..Default::default()
    };
    let port_def = PortDef {
        name: "telemetry_out_port".to_string(),
        type_name: "SolverTelemetryPayload".to_string(),
        ..Default::default()
    };
    let part_def = PartDef {
        name: "ConstraintEvaluatorUnit".to_string(),
        is_def: true,
        item_defs: vec![item_def],
        attributes: vec![make_attr("margin_ratio", "Real", Some("1.0"))],
        ports: vec![PortDef {
            name: "evaluator_port".to_string(),
            type_name: "ConstraintEvaluatorUnit".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let model = SysmlModel {
        package_name: Some("OptimizationKernelPackage".to_string()),
        ports: vec![port_def],
        parts: vec![part_def],
        ..Default::default()
    };

    let bridge_path = emit_dart_solver_bridge(&tmp.path, &model, 0xCAFE_BABE_1234_5678)?;
    assert!(bridge_path.exists(), "solver_bridge.g.dart must be written to disk");

    let dart_src = fs::read_to_string(&bridge_path)?;
    let direct_src = generate_solver_bridge_dart(&model, 0xCAFE_BABE_1234_5678);
    assert_eq!(dart_src, direct_src);

    for needle in [
        "import 'dart:convert';",
        "import 'dart:io';",
        "class SolverTelemetryPayloadDto",
        "class TelemetryOutPortDto",
        "final double compositeScore;",
        "final int iterationCount;",
        "final bool feasibleFlag;",
        "final String statusCode;",
        "'composite_score': compositeScore",
        "json['composite_score']",
        "class EvaluatorPortDto",
        "final double marginRatio;",
        "Future<Map<String, dynamic>> solveJson(",
        "'--solve-json'",
    ] {
        assert!(dart_src.contains(needle), "missing in solver_bridge.g.dart: {needle}");
    }
    Ok(())
}

#[test]
fn test_generate_domain_seed_dart_unions_topology_types_attributes_and_part_defaults() {
    let alpha_part = PartDef {
        name: "AlphaComponent".to_string(),
        is_def: true,
        attributes: vec![
            make_attr("default_threshold", "Real", Some("42.5")),
            make_attr("override_metric", "Real", Some("10.0")),
            make_attr("active_flag", "Boolean", Some("true")),
            make_attr("mode_tag", "String", Some("NOMINAL")),
        ],
        ..Default::default()
    };
    let model = SysmlModel {
        parts: vec![alpha_part],
        ..Default::default()
    };

    let mut alpha_props = BTreeMap::new();
    alpha_props.insert("override_metric".to_string(), serde_json::json!(99.5));
    alpha_props.insert("dynamic_extra_metric".to_string(), serde_json::json!(7.25));

    let mut beta_props = BTreeMap::new();
    beta_props.insert("topology_only_prop".to_string(), serde_json::json!("READY"));
    beta_props.insert("topology_only_flag".to_string(), serde_json::json!(false));

    let nodes = vec![
        make_node("NODE_ALPHA_01", "AlphaComponent", "Alpha Node 01", None, alpha_props),
        make_node(
            "NODE_BETA_02",
            "BetaTopologyOnlyType",
            "Beta Node 02",
            Some("NODE_ALPHA_01"),
            beta_props,
        ),
    ];

    let seed_src = generate_domain_seed_dart(&model, &nodes, 0x1234_5678_9ABC_DEF0);
    for needle in [
        "['AlphaComponent', 'Alpha Component', 'layers']",
        "['BetaTopologyOnlyType', 'Beta Topology Only Type', 'layers']",
        "['AlphaComponent', 'dynamic_extra_metric'",
        "['BetaTopologyOnlyType', 'topology_only_prop'",
        "['BetaTopologyOnlyType', 'topology_only_flag'",
        "['NODE_ALPHA_01', 'override_metric', null, 99.5, 'double']",
        "['NODE_ALPHA_01', 'default_threshold', null, 42.5, 'double']",
        "['NODE_ALPHA_01', 'active_flag', 'true', 1, 'bool']",
        "['NODE_ALPHA_01', 'mode_tag', 'NOMINAL', null, 'text']",
    ] {
        assert!(seed_src.contains(needle), "missing in seed_src: {needle}");
    }
}

/// Strips line comments (`//`) from Dart source text for structural test assertions.
/// Realises: [REQ-FLUTTER-GENERATE-UI-CONTRACT-TEST]
fn strip_line_comments_for_test(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.lines() {
        let code = match line.split_once("//") {
            Some((before, _)) => before,
            None => line,
        };
        out.push_str(code);
        out.push('\n');
    }
    out
}

#[test]
fn test_generate_ui_contract_test_dart_emits_live_pump_getrect_tap_and_pairwise_overlaps() {
    let model = SysmlModel::default();
    let nodes = vec![
        make_node("NODE_ALPHA_01", "AlphaComponent", "Alpha Node 01", None, BTreeMap::new()),
        make_node(
            "NODE_BETA_02",
            "BetaComponent",
            "Beta Node 02",
            Some("NODE_ALPHA_01"),
            BTreeMap::new(),
        ),
    ];

    let raw_dart = generate_ui_contract_test_dart(&model, &nodes);
    let stripped = strip_line_comments_for_test(&raw_dart);

    for needle in [
        "DomainSeedStrategy",
        "a.overlaps(b)",
        "tester.pumpWidget(",
        "tester.getRect(",
        "await tester.tap(find.byKey(SchemaWidgetKeys.entityRow('NODE_ALPHA_01')));",
        "await tester.tap(find.byKey(SchemaWidgetKeys.entityRow('NODE_BETA_02')));",
        "1920",
        "1080",
        "2560",
        "1440",
        "3840",
        "2160",
        "6016",
        "3384",
    ] {
        assert!(stripped.contains(needle), "missing in ui contract test: {needle}");
    }
    assert!(!stripped.contains("Rect.fromLTWH("));
    assert!(!stripped.contains("'{}'"));
}

/// Builds the `SysmlModel` and `Vec<TopologyNode>` fixture for sanitized-defaults testing.
/// Realises: [REQ-FLUTTER-GENERATE-SCHEMA-MODELS]
fn build_sanitized_defaults_fixture() -> (SysmlModel, Vec<TopologyNode>) {
    let alpha_part = PartDef {
        name: "AlphaComponent".to_string(),
        is_def: true,
        attributes: vec![
            make_attr("quoted_mode", "String", Some("\"NOMINAL\"")),
            make_attr("single_quoted_tag", "String", Some("'STANDBY'")),
            make_attr("bad_real_default", "Real", Some("TBD")),
            make_attr("bad_int_default", "Integer", Some("N/A")),
            make_attr("bad_bool_default", "Boolean", Some("INVALID")),
        ],
        ports: vec![PortDef {
            name: "alpha_port".to_string(),
            type_name: "AlphaComponent".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let model = SysmlModel {
        parts: vec![alpha_part],
        ..Default::default()
    };

    let mut alpha_props = BTreeMap::new();
    alpha_props.insert("dynamic_extra_metric".to_string(), serde_json::json!(7.25));
    alpha_props.insert("dynamic_flag".to_string(), serde_json::json!(true));
    alpha_props.insert("dynamic_label".to_string(), serde_json::json!("ACTIVE"));

    let mut beta_props = BTreeMap::new();
    beta_props.insert("topology_only_prop".to_string(), serde_json::json!("READY"));
    beta_props.insert("topology_only_num".to_string(), serde_json::json!(15.0));

    let nodes = vec![
        make_node("NODE-ALPHA-01", "AlphaComponent", "Alpha Node 01", Some("ZONE-NORTH-01"), alpha_props),
        make_node(
            "NODE_ALPHA_01",
            "AlphaComponent",
            "Alpha Node 01 Duplicate Camel",
            Some("ZONE-NORTH-01"),
            BTreeMap::new(),
        ),
        make_node("NODE_BETA_02", "BetaTopologyOnlyType", "Beta Node 02", Some("ZONE-NORTH-01"), beta_props),
    ];
    (model, nodes)
}

#[test]
fn test_generate_schema_models_and_seed_union_topology_types_dedup_ids_and_sanitize_defaults() {
    let (model, nodes) = build_sanitized_defaults_fixture();
    let models_src = generate_schema_models_dart(&model, &nodes, 0x1234);
    let seed_src = generate_domain_seed_dart(&model, &nodes, 0x1234);
    let bridge_src = generate_solver_bridge_dart(&model, 0x1234);

    for needle in [
        "extension type const BetaTopologyOnlyTypeView(InstanceRecord _record) implements InstanceRecord",
        "double get dynamicExtraMetric => _record.getPropertyDouble('dynamic_extra_metric')",
        "bool get dynamicFlag => _record.getPropertyBool('dynamic_flag')",
        "String get dynamicLabel => _record.getPropertyString('dynamic_label')",
        "String get topologyOnlyProp => _record.getPropertyString('topology_only_prop')",
        "double get topologyOnlyNum => _record.getPropertyDouble('topology_only_num')",
        "static const String zoneNorth01 = 'ZONE-NORTH-01';",
        "String get quotedMode => _record.getPropertyString('quoted_mode') ?? 'NOMINAL';",
        "String get singleQuotedTag => _record.getPropertyString('single_quoted_tag') ?? 'STANDBY';",
        "double get badRealDefault => _record.getPropertyDouble('bad_real_default') ?? 0.0;",
        "int get badIntDefault => _record.getPropertyInt('bad_int_default') ?? 0;",
        "bool get badBoolDefault => _record.getPropertyBool('bad_bool_default') ?? false;",
    ] {
        assert!(models_src.contains(needle), "missing in models_src: {needle}\n{models_src}");
    }
    assert_eq!(models_src.matches("static const String nodeAlpha01 =").count(), 1);
    assert!(!seed_src.contains("'\"NOMINAL\"'"));
    assert!(!bridge_src.contains("'\"NOMINAL\"'"));
}
