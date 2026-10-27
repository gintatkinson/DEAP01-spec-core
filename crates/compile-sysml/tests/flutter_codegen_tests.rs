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

use compile_sysml::flutter_codegen::{emit_dart_solver_bridge, generate_solver_bridge_dart};
use deap_core::sysml_ast::{AttributeDef, ItemDef, PartDef, PortDef, SysmlModel};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

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
    /// Realises: [REQ-FLUTTER-CODEGEN-TEST-FIXTURE]
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Verifies that `emit_dart_solver_bridge` writes `solver_bridge.g.dart` with typed DTO fields and `--solve-json` transport.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D Zero-Trust FFI and JSON solver transport contracts.
///
/// ## 2. Invariants & Mathematical Properties
/// - Port and Item definitions with attributes generate typed fields (`double`, `int`, `bool`, `String`) and `toJson`/`fromJson`.
/// - `DeapNativeSolverIsolate` includes `solveJson` invoking `--solve-json`.
///
/// ## 3. Preconditions
/// - `SysmlModel` contains `ItemDef`, `PartDef`, and `PortDef` declarations.
///
/// ## 4. Postconditions
/// - `solver_bridge.g.dart` exists on disk and satisfies all structural assertions.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ in generated source size.
/// - Space Complexity: $O(N)$ for in-memory source verification.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Fails test if typed fields or `--solve-json` transport are absent.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
#[test]
fn test_emit_dart_solver_bridge_typed_dtos_and_solve_json_transport() {
    let tmp = ScopedTempDir::new("solver_bridge");
    let mut model = SysmlModel::default();
    model.package_name = Some("OptimizationKernelPackage".to_string());

    let mut item_def = ItemDef::default();
    item_def.name = "SolverTelemetryPayload".to_string();
    item_def.attributes.push(AttributeDef {
        name: "composite_score".to_string(),
        type_name: "Real".to_string(),
        default_value: Some("0.0".to_string()),
        doc: None,
    });
    item_def.attributes.push(AttributeDef {
        name: "iteration_count".to_string(),
        type_name: "Integer".to_string(),
        default_value: Some("0".to_string()),
        doc: None,
    });
    item_def.attributes.push(AttributeDef {
        name: "feasible_flag".to_string(),
        type_name: "Boolean".to_string(),
        default_value: Some("true".to_string()),
        doc: None,
    });
    item_def.attributes.push(AttributeDef {
        name: "status_code".to_string(),
        type_name: "String".to_string(),
        default_value: Some("READY".to_string()),
        doc: None,
    });
    let mut port_def = PortDef::default();
    port_def.name = "telemetry_out_port".to_string();
    port_def.type_name = "SolverTelemetryPayload".to_string();
    model.ports.push(port_def);

    let mut part_def = PartDef::default();
    part_def.name = "ConstraintEvaluatorUnit".to_string();
    part_def.is_def = true;
    part_def.item_defs.push(item_def);
    part_def.attributes.push(AttributeDef {
        name: "margin_ratio".to_string(),
        type_name: "Real".to_string(),
        default_value: Some("1.0".to_string()),
        doc: None,
    });
    let mut part_port = PortDef::default();
    part_port.name = "evaluator_port".to_string();
    part_port.type_name = "ConstraintEvaluatorUnit".to_string();
    part_def.ports.push(part_port);
    model.parts.push(part_def);

    let bridge_path = emit_dart_solver_bridge(&tmp.path, &model, 0xCAFE_BABE_1234_5678)
        .expect("emit_dart_solver_bridge must succeed");
    assert!(bridge_path.exists(), "solver_bridge.g.dart must be written to disk");

    let dart_src = fs::read_to_string(&bridge_path).expect("read solver_bridge.g.dart");
    let direct_src = generate_solver_bridge_dart(&model, 0xCAFE_BABE_1234_5678);
    assert_eq!(dart_src, direct_src);

    // 1. Imports for JSON & CLI process transport
    assert!(dart_src.contains("import 'dart:convert';"));
    assert!(dart_src.contains("import 'dart:io';"));

    // 2. Typed DTO for ItemDef and PortDef referencing ItemDef
    assert!(dart_src.contains("class SolverTelemetryPayloadDto"));
    assert!(dart_src.contains("class TelemetryOutPortDto"));
    assert!(dart_src.contains("final double compositeScore;"));
    assert!(dart_src.contains("final int iterationCount;"));
    assert!(dart_src.contains("final bool feasibleFlag;"));
    assert!(dart_src.contains("final String statusCode;"));
    assert!(dart_src.contains("'composite_score': compositeScore"));
    assert!(dart_src.contains("json['composite_score']"));

    // 3. Typed DTO for PortDef referencing PartDef attributes
    assert!(dart_src.contains("class EvaluatorPortDto"));
    assert!(dart_src.contains("final double marginRatio;"));

    // 4. --solve-json transport method on DeapNativeSolverIsolate
    assert!(dart_src.contains("Future<Map<String, dynamic>> solveJson("));
    assert!(dart_src.contains("'--solve-json'"));
}
