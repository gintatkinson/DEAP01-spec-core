//! Modular Flutter contract synthesis orchestrator and legacy public symbol facade.
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D deterministic codegen controls.
//! ## 2. Invariants & Mathematical Properties
//! - Deterministic `BTreeMap`/`BTreeSet` emission and atomic temp-rename persistence across targets.
//! ## 3. Preconditions
//! - Callers provide parsed `SysmlModel` or `PackageDef` ASTs and a writable workspace directory.
//! ## 4. Postconditions
//! - Emits monolithic and package-sliced Dart seeds, views, FFI solver bridges, and UI test suites.
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(S \cdot (P \cdot A + N \cdot K \log K))$ across $S$ slices and $N$ nodes.
//! - Space Complexity: $O(U)$ proportional to total generated `.g.dart` batch byte volume $U$.
//! ## 6. Failure Modes & Fault Tolerance
//! - Fail-closed `FlutterCodegenError` propagation on I/O, JSON serialization, or topology defects.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607], [REQ-EMIT-FLUTTER-CONTRACTS], [REQ-COMPILER-FLUTTER-CODEGEN]

pub mod bridge_emitter;
pub mod contracts;
pub mod models_emitter;
pub mod naming;
pub mod seed_emitter;
pub mod topology;
pub mod ui_test_emitter;

use std::fs;
use std::path::{Path, PathBuf};

use deap_core::sysml_ast::{PackageDef, SysmlModel};

pub use bridge_emitter::generate_solver_bridge_dart;
pub use contracts::{FlutterCodegenError, FlutterContractOptions, FlutterContractReport, TopologyNode};
pub use models_emitter::generate_schema_models_dart;
pub use naming::{
    compute_schema_hash, escape_dart_single_quoted, extract_bounds_from_doc, format_display_name,
    split_ident_into_words, to_camel_case, to_pascal_case, to_snake_case_dart,
};
pub use seed_emitter::generate_domain_seed_dart;
pub use topology::parse_topology_json;
pub use ui_test_emitter::generate_ui_contract_test_dart;

/// Resolves the target Flutter project root, preferring `<out_dir>/app_flutter` when present.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A workspace directory boundary resolution for generated Dart units.
/// ## 2. Invariants & Mathematical Properties
/// - Selects `out_dir.join("app_flutter")` iff `is_dir()` holds; otherwise preserves `out_dir`.
/// ## 3. Preconditions
/// - `out_dir` references a workspace root or direct Flutter package directory path.
/// ## 4. Postconditions
/// - Returns an owned `PathBuf` anchoring `lib/` and `test/` generated contract subdirectories.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ single directory metadata check.
/// - Space Complexity: $O(L)$ proportional to path byte length $L$.
/// ## 6. Failure Modes & Fault Tolerance
/// - Total panic-free resolution falling back to `out_dir` when `app_flutter` is absent.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-EMIT-FLUTTER-CONTRACTS]
fn resolve_flutter_base_dir(out_dir: &Path) -> PathBuf {
    let candidate = out_dir.join("app_flutter");
    if candidate.is_dir() { candidate } else { out_dir.to_path_buf() }
}

/// Persists a batch of `(relative_path, dart_source)` tuples atomically under `base_dir`.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A crash-consistent file persistence via atomic temp-file rename.
/// ## 2. Invariants & Mathematical Properties
/// - Ensures parent directories are created prior to `write_atomic` in deterministic slice order.
/// ## 3. Preconditions
/// - `base_dir` designates a writable directory and `files` holds relative POSIX-style `.g.dart` paths.
/// ## 4. Postconditions
/// - Every `(rel_path, content)` entry in `files` is durably materialized under `base_dir`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(F + B)$ over $F$ output files and $B$ total UTF-8 source bytes.
/// - Space Complexity: $O(L)$ per joined path buffer of byte length $L$.
/// ## 6. Failure Modes & Fault Tolerance
/// - Aborts immediately and propagates `FlutterCodegenError::Io` on directory or atomic write failure.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-EMIT-FLUTTER-CONTRACTS]
fn write_generated_dart_batch(base_dir: &Path, files: &[(String, String)]) -> Result<(), FlutterCodegenError> {
    for (rel_path, content) in files {
        let full_path = base_dir.join(rel_path);
        if let Some(parent) = full_path.parent() { fs::create_dir_all(parent)?; }
        crate::semantic::digest::write_atomic(&full_path, content)?;
    }
    Ok(())
}

/// Lifts a flat `SysmlModel` into a single-level `PackageDef` for legacy caller compatibility.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A lossless AST adapter construction between flat and modular pipelines.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves `parts`, `ports`, and `attributes` verbatim; defaults missing `package_name` to `"root"`.
/// ## 3. Preconditions
/// - `model` references any valid parsed `SysmlModel` instance.
/// ## 4. Postconditions
/// - Returns an owned `PackageDef` suitable for `emit_modular_flutter_contracts` slice traversal.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(P + Q + A)$ cloning $P$ parts, $Q$ ports, and $A$ top-level attributes.
/// - Space Complexity: $O(P + Q + A)$ for the synthesized `PackageDef` node.
/// ## 6. Failure Modes & Fault Tolerance
/// - Total panic-free pure transformation across empty or unnamed `SysmlModel` inputs.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-EMIT-FLUTTER-CONTRACTS]
fn synth_package_from_flat_model(model: &SysmlModel) -> PackageDef {
    let name = match &model.package_name { Some(n) => n.clone(), None => "root".to_string() };
    PackageDef {
        name,
        part_defs: model.parts.clone(),
        port_defs: model.ports.clone(),
        attribute_defs: model.attributes.clone(),
        ..PackageDef::default()
    }
}

/// Synthesizes and atomically writes the standalone `solver_bridge.g.dart` C-ABI FFI contract.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D deterministic C-ABI FFI bridge persistence.
/// ## 2. Invariants & Mathematical Properties
/// - Embeds `schema_hash` verbatim and writes `lib/data/generated/solver_bridge.g.dart` atomically.
/// ## 3. Preconditions
/// - `out_dir` is writable, `model` holds SysML port definitions, and `schema_hash` is precomputed.
/// ## 4. Postconditions
/// - Materializes `solver_bridge.g.dart` on disk and returns its canonical `PathBuf`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(Q \cdot A \log A)$ over $Q$ ports and $A$ attributes per DTO contract.
/// - Space Complexity: $O(Q \cdot A)$ for the rendered Dart bridge buffer prior to atomic flush.
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates `FlutterCodegenError::Io` when parent directory creation or atomic write fails.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-EMIT-FLUTTER-CONTRACTS]
pub fn emit_dart_solver_bridge(out_dir: &Path, model: &SysmlModel, schema_hash: u64) -> Result<PathBuf, FlutterCodegenError> {
    let base_dir = resolve_flutter_base_dir(out_dir);
    let solver_bridge_path = base_dir.join("lib").join("data").join("generated").join("solver_bridge.g.dart");
    if let Some(parent) = solver_bridge_path.parent() { fs::create_dir_all(parent)?; }
    let content = generate_solver_bridge_dart(model, schema_hash);
    crate::semantic::digest::write_atomic(&solver_bridge_path, &content)?;
    Ok(solver_bridge_path)
}

/// Orchestrates full monolithic plus modular package-sliced Flutter `.g.dart` contract emission.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 end-to-end schema-to-Flutter synthesis governance.
/// ## 2. Invariants & Mathematical Properties
/// - Stamps identical 64-bit `schema_hash` across seeds, models, and bridges; skips I/O on `dry_run`.
/// ## 3. Preconditions
/// - `out_dir` designates workspace root; `pkg` and `model` supply coherent parsed SysML definitions.
/// ## 4. Postconditions
/// - Emits all `.g.dart` artifacts when `!dry_run` and returns `FlutterContractReport` telemetry.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(S \cdot (P \cdot A + N \cdot K \log K))$ across $S$ slices and $N$ nodes.
/// - Space Complexity: $O(U)$ bounded by accumulated `(relative_path, dart_source)` batch size $U$.
/// ## 6. Failure Modes & Fault Tolerance
/// - Fails closed via `FlutterCodegenError` on topology parsing, model hashing, or batch I/O faults.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-EMIT-FLUTTER-CONTRACTS]
pub fn emit_modular_flutter_contracts(
    out_dir: &Path,
    pkg: &PackageDef,
    model: &SysmlModel,
    topology_json: Option<&str>,
    dry_run: bool,
) -> Result<FlutterContractReport, FlutterCodegenError> {
    let base_dir = resolve_flutter_base_dir(out_dir);
    let (resolved_topology_str, nodes) = topology::resolve_topology_nodes(out_dir, &base_dir, &model.parts, topology_json)?;
    let schema_str = serde_json::to_string(model).map_err(|e| FlutterCodegenError::InvalidModel(format!("Failed to serialize SysmlModel for hash: {e}")))?;
    let schema_hash = compute_schema_hash(&schema_str, &resolved_topology_str);
    let seed_path = base_dir.join("lib").join("data").join("seeds").join("generated_domain_seed.g.dart");
    let models_path = base_dir.join("lib").join("models").join("generated_schema_models.g.dart");
    let solver_bridge_path = base_dir.join("lib").join("data").join("generated").join("solver_bridge.g.dart");
    let ui_contract_test_path = base_dir.join("test").join("generated").join("schema_ui_contract_test.g.dart");
    let mut batch: Vec<(String, String)> = vec![
        ("lib/data/seeds/generated_domain_seed.g.dart".to_string(), generate_domain_seed_dart(model, &nodes, schema_hash)),
        ("lib/models/generated_schema_models.g.dart".to_string(), generate_schema_models_dart(model, &nodes, schema_hash)),
        ("lib/data/generated/solver_bridge.g.dart".to_string(), generate_solver_bridge_dart(model, schema_hash)),
        ("test/generated/schema_ui_contract_test.g.dart".to_string(), generate_ui_contract_test_dart(model, &nodes)),
    ];
    batch.extend(seed_emitter::emit_modular_seed_files(pkg, &nodes, schema_hash));
    batch.extend(models_emitter::emit_modular_model_files(pkg, &nodes, schema_hash));
    batch.extend(bridge_emitter::emit_modular_bridge_files(pkg, schema_hash));
    batch.extend(ui_test_emitter::emit_modular_ui_test_files(pkg, &nodes));
    if !dry_run { write_generated_dart_batch(&base_dir, &batch)?; }
    Ok(FlutterContractReport {
        seed_path, models_path, solver_bridge_path, ui_contract_test_path,
        schema_hash, entities_count: nodes.len(), types_count: model.parts.len(),
    })
}

/// Emits monolithic and modular Flutter contracts directly from a flat `SysmlModel` descriptor.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A backward-compatible public API governance for flat model callers.
/// ## 2. Invariants & Mathematical Properties
/// - Delegates deterministically to `emit_modular_flutter_contracts` with `dry_run` set to `false`.
/// ## 3. Preconditions
/// - `out_dir` is writable, `model` is a valid `SysmlModel`, and `topology_json` is optional UTF-8 JSON.
/// ## 4. Postconditions
/// - Persists all Flutter contract `.g.dart` outputs and returns the resulting `FlutterContractReport`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(P \cdot A + N \cdot K \log K)$ over $P$ parts and $N$ topology instances.
/// - Space Complexity: $O(U)$ bounded by synthesized `PackageDef` and `.g.dart` batch buffers.
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates any `FlutterCodegenError` raised during topology decoding, hashing, or disk writes.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-EMIT-FLUTTER-CONTRACTS]
pub fn emit_flutter_contracts(out_dir: &Path, model: &SysmlModel, topology_json: Option<&str>) -> Result<FlutterContractReport, FlutterCodegenError> {
    let pkg = synth_package_from_flat_model(model);
    emit_modular_flutter_contracts(out_dir, &pkg, model, topology_json, false)
}
