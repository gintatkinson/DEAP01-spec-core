//! Deterministic Dart FFI C-ABI solver bridge, typed DTO, and modular barrel code generation.
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D C-ABI boundary controls.
//! ## 2. Invariants & Mathematical Properties
//! - Deterministic `BTreeMap` and `BTreeSet` ordering without `HashMap`, `HashSet`, or regex.
//! ## 3. Preconditions
//! - Callers provide parsed `SysmlModel` or `PackageDef` trees and a 64-bit `schema_hash`.
//! ## 4. Postconditions
//! - Synthesizes monolithic or package-sliced Dart FFI runtime, typed DTOs, and barrel exports.
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(P \cdot A \log A)$; Space Complexity: $O(P \cdot A)$ over $P$ ports and $A$ attrs.
//! ## 6. Failure Modes & Fault Tolerance
//! - Total panic-free emission defaulting to `SolverPortDto` when input schemas omit ports.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]

use std::collections::{BTreeMap, BTreeSet};
use deap_core::sysml_ast::{AttributeDef, PackageDef, PortDef, SysmlModel};
use super::naming::{escape_dart_single_quoted, to_camel_case, to_pascal_case, to_snake_case_dart};
use super::topology::{collect_package_slices, strip_surrounding_quotes};

/// Normalized descriptor pairing a SysML contract identifier with its Dart DTO class and fields.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A deterministic intermediate representation rules for DTO synthesis.
/// ## 2. Invariants & Mathematical Properties
/// - `dto_name` always carries a PascalCase stem suffixed with `Dto` derived from `source_name`.
/// ## 3. Preconditions
/// - Constructed from deduplicated `ItemDef`, `PortDef`, or fallback `SolverPort` metadata.
/// ## 4. Postconditions
/// - Supplies immutable attribute slices to `append_typed_dto_class` and modular file writers.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ move construction; Space Complexity: $O(A)$ for $A$ owned attributes.
/// ## 6. Failure Modes & Fault Tolerance
/// - Empty `attrs` vectors remain valid and trigger raw `payload` pass-through DTO emission.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
struct DtoSpec { source_name: String, dto_name: String, attrs: Vec<AttributeDef> }

/// Maps a SysML `AttributeDef` to its Dart scalar type, default literal, and JSON decode expression.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D null-safe deserialization boundary rules.
/// ## 2. Invariants & Mathematical Properties
/// - Total mapping partitioning SysML scalar names into `double`, `int`, `bool`, or `String`.
/// ## 3. Preconditions
/// - `attr` references a parsed `AttributeDef` with UTF-8 `name`, `type_name`, and `default_value`.
/// ## 4. Postconditions
/// - Returns `(dart_type, default_literal, from_json_expr)` with escaped single-quoted JSON keys.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(K + V)$; Space Complexity: $O(K + V)$ in key length $K$ and value length $V$.
/// ## 6. Failure Modes & Fault Tolerance
/// - Unparseable numeric or boolean defaults fall back deterministically to `0.0`, `0`, or `false`.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
fn resolve_dto_attribute_codec(attr: &AttributeDef) -> (&'static str, String, String) {
    let key_esc = escape_dart_single_quoted(&attr.name);
    let raw = attr.default_value.as_deref().map(strip_surrounding_quotes);
    match attr.type_name.as_str() {
        "Real" | "Float" | "Double" => {
            let lit = format!("{:.1}", raw.and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0));
            ("double", lit.clone(), format!("(json['{key_esc}'] as num?)?.toDouble() ?? {lit}"))
        }
        "Integer" | "Int" => {
            let lit = format!("{}", raw.and_then(|v| v.parse::<i64>().ok()).unwrap_or(0));
            ("int", lit.clone(), format!("(json['{key_esc}'] as num?)?.toInt() ?? {lit}"))
        }
        "Boolean" | "Bool" => {
            let lit = raw.is_some_and(|v| v.eq_ignore_ascii_case("true")).to_string();
            ("bool", lit.clone(), format!("(json['{key_esc}'] as bool?) ?? {lit}"))
        }
        _ => {
            let lit = format!("'{}'", escape_dart_single_quoted(raw.unwrap_or("")));
            ("String", lit.clone(), format!("(json['{key_esc}'] as String?) ?? {lit}"))
        }
    }
}

/// Appends a deterministic Dart DTO class with typed fields, `toJson`, and `fromJson` codecs.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A type-safe data transfer contract synthesis without reflection.
/// ## 2. Invariants & Mathematical Properties
/// - Deduplicates `camelCase` member names via `BTreeSet` while reserving `payload` collision-free.
/// ## 3. Preconditions
/// - `out` is a writable Dart buffer; `dto_name` is a valid Dart class identifier; `attrs` is a slice.
/// ## 4. Postconditions
/// - Appends one complete `class {dto_name}` declaration preserving extra map entries in `payload`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(A \log A)$; Space Complexity: $O(A)$ across $A$ input `AttributeDef` items.
/// ## 6. Failure Modes & Fault Tolerance
/// - Emits a positional map-backed `payload` DTO constructor when zero unique typed fields remain.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
fn append_typed_dto_class(out: &mut String, source_name: &str, dto_name: &str, attrs: &[AttributeDef]) {
    out.push_str(&format!("/// Data Transfer Object for SysML contract `{source_name}`.\nclass {dto_name} {{\n"));
    let mut seen = BTreeSet::from([String::new(), "payload".to_string()]);
    let unique: Vec<&AttributeDef> = attrs.iter().filter(|a| seen.insert(to_camel_case(&a.name))).collect();
    if unique.is_empty() {
        out.push_str(&format!(concat!(
            "  final Map<String, dynamic> payload;\n  const {0}([this.payload = const <String, dynamic>{{}}]);\n\n",
            "  Map<String, dynamic> toJson() => payload;\n",
            "  factory {0}.fromJson(Map<String, dynamic> json) => {0}(json);\n}}\n\n",
        ), dto_name));
        return;
    }
    let (mut fields, mut ctor, mut to_j, mut from_j) = (String::new(), String::new(), String::new(), String::new());
    for attr in unique {
        let (camel, key_esc) = (to_camel_case(&attr.name), escape_dart_single_quoted(&attr.name));
        let (dart_type, default_lit, from_json) = resolve_dto_attribute_codec(attr);
        fields.push_str(&format!("  final {dart_type} {camel};\n"));
        ctor.push_str(&format!("    this.{camel} = {default_lit},\n"));
        to_j.push_str(&format!("    '{key_esc}': {camel},\n"));
        from_j.push_str(&format!("    {camel}: {from_json},\n"));
    }
    out.push_str(&format!(concat!(
        "{0}  final Map<String, dynamic> payload;\n\n  const {1}({{\n{2}",
        "    this.payload = const <String, dynamic>{{}},\n  }});\n\n",
        "  Map<String, dynamic> toJson() => <String, dynamic>{{\n    ...payload,\n{3}  }};\n\n",
        "  factory {1}.fromJson(Map<String, dynamic> json) => {1}(\n{4}    payload: json,\n  );\n}}\n\n",
    ), fields, dto_name, ctor, to_j, from_j));
}

/// Appends the Dart FFI imports, ABI constants, `DeapFfiStatus` enum, and `DeapByteBuffer` struct.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ISO 26262 ASIL D C-ABI memory ceiling and CRC-32 framing rules.
/// ## 2. Invariants & Mathematical Properties
/// - Pins `kDeapAbiVersion = 1`, `kMaxFfiPayloadBytes = 16777216`, and 16-digit hex `kDeapSchemaHash`.
/// ## 3. Preconditions
/// - `out` is an initialized output buffer and `schema_hash` is the computed 64-bit schema digest.
/// ## 4. Postconditions
/// - Appends complete Dart FFI header declarations and unknown-code fail-safe status decoding.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ template formatting; Space Complexity: $O(1)$ fixed preamble growth.
/// ## 6. Failure Modes & Fault Tolerance
/// - Generated `DeapFfiStatus.fromCode` fails closed to `DeapFfiStatus.internalPanic` on unknown codes.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
fn append_ffi_runtime_preamble(out: &mut String, schema_hash: u64) {
    out.push_str(&format!(concat!(
        "// GENERATED CODE - DO NOT MODIFY BY HAND\n// Generated by compile-sysml from SysML schema\n",
        "// Zero-Trust Deterministic Rust C-ABI Bridge & DTO Contracts (DO-178C Level A / ISO 26262 ASIL D)\n\n",
        "import 'dart:async';\nimport 'dart:convert';\nimport 'dart:ffi';\n",
        "import 'dart:io';\nimport 'dart:isolate';\nimport 'dart:typed_data';\n\n",
        "const int kDeapAbiVersion = 1;\nconst int kDeapSchemaHash = 0x{:016X};\n",
        "const int kMaxFfiPayloadBytes = 16777216; // 16 MiB ceiling\n\n",
    ), schema_hash));
    out.push_str(concat!(
        "enum DeapFfiStatus {\n  ok(0), invalidArgument(1), schemaVersionMismatch(2), crcMismatch(3),\n",
        "  payloadTooLarge(4), internalPanic(5), allocationFailure(6);\n  final int code;\n  const DeapFfiStatus(this.code);\n",
        "  static DeapFfiStatus fromCode(int code) {\n    for (final s in values) { if (s.code == code) return s; }\n",
        "    return DeapFfiStatus.internalPanic;\n  }\n}\n\n",
        "final class DeapByteBuffer extends Struct {\n  external Pointer<Uint8> ptr;\n",
        "  @IntPtr() external int len;\n  @IntPtr() external int cap;\n",
        "  @Uint32() external int crc32;\n  @Uint32() external int status;\n}\n\n",
    ));
}

/// Appends the `DeapNativeSolverIsolate` worker-port and JSON subprocess execution bridge class.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A bounded-latency execution budgets keeping UI threads non-blocking.
/// ## 2. Invariants & Mathematical Properties
/// - Enforces a strict 500 ms `TimeoutException` deadline on isolate messages and CLI solver calls.
/// ## 3. Preconditions
/// - `out` already contains FFI runtime imports (`dart:async`, `dart:convert`, `dart:io`, `dart:isolate`).
/// ## 4. Postconditions
/// - Appends `DeapNativeSolverIsolate` exposing `evaluateRequest` and `solveJson` async methods.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ static Dart class emission; Space Complexity: $O(1)$ buffer append.
/// ## 6. Failure Modes & Fault Tolerance
/// - Generated methods close `ReceivePort` in `finally` and raise `StateError` on non-zero exit codes.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
fn append_solver_isolate_class(out: &mut String) {
    out.push_str(concat!(
        "class DeapNativeSolverIsolate {\n  final SendPort _sendPort;\n  DeapNativeSolverIsolate(this._sendPort);\n\n",
        "  Future<Uint8List> evaluateRequest(Uint8List requestPayload) async {\n",
        "    if (requestPayload.lengthInBytes > kMaxFfiPayloadBytes) throw ArgumentError('FFI payload > 16 MiB');\n",
        "    final ReceivePort responsePort = ReceivePort();\n    try {\n",
        "      _sendPort.send(<Object>[requestPayload, responsePort.sendPort]);\n",
        "      final Object? raw = await responsePort.first.timeout(const Duration(milliseconds: 500),\n",
        "        onTimeout: () => throw TimeoutException('Solver isolate timeout (500ms)'));\n",
        "      if (raw is Uint8List) return raw;\n      throw StateError('Unexpected solver isolate response');\n",
        "    } finally {\n      responsePort.close();\n    }\n  }\n\n",
        "  Future<Map<String, dynamic>> solveJson(String solverExecutablePath, Map<String, dynamic> requestJson) async {\n",
        "    final String encodedRequest = jsonEncode(requestJson);\n",
        "    final ProcessResult result = await Process.run(solverExecutablePath, <String>['--solve-json', encodedRequest])\n",
        "      .timeout(const Duration(milliseconds: 500), onTimeout: () => throw TimeoutException('Solver timeout (500ms)'));\n",
        "    if (result.exitCode != 0) throw StateError('Solver exited (${result.exitCode}): ${result.stderr}');\n",
        "    final Object? decoded = jsonDecode(result.stdout.toString());\n",
        "    if (decoded is Map<String, dynamic>) return decoded;\n    throw StateError('Invalid solver JSON output');\n  }\n}\n",
    ));
}

/// Converts `port.electrical_attributes` into `String`-typed `AttributeDef` records or returns `fallback`.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A interface port metadata preservation across generated DTO contracts.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves `BTreeMap` key ordering of `electrical_attributes` when non-empty.
/// ## 3. Preconditions
/// - `port` references a parsed `PortDef` and `fallback` holds enclosing part or empty attributes.
/// ## 4. Postconditions
/// - Returns synthesized `String` attributes for electrical ports, or `fallback.to_vec()` when empty.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(E + F)$; Space Complexity: $O(E + F)$ over $E$ electrical or $F$ fallback attrs.
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function returning an empty vector when both `electrical_attributes` and `fallback` are empty.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
fn port_electrical_attrs(port: &PortDef, fallback: &[AttributeDef]) -> Vec<AttributeDef> {
    if port.electrical_attributes.is_empty() {
        return fallback.to_vec();
    }
    port.electrical_attributes.iter().map(|(k, v)| AttributeDef {
        name: k.clone(), type_name: "String".to_string(), default_value: Some(v.clone()), doc: None,
    }).collect()
}

/// Extracts ordered, deduplicated `DtoSpec` definitions from a `SysmlModel`'s items, parts, and ports.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A deterministic schema-to-DTO derivation without domain hardcoding.
/// ## 2. Invariants & Mathematical Properties
/// - Emits each `dto_name` at most once via `BTreeSet` and synthesizes `SolverPortDto` if empty.
/// ## 3. Preconditions
/// - `model` references a parsed `SysmlModel` containing parts, item definitions, and ports.
/// ## 4. Postconditions
/// - Returns a non-empty `Vec<DtoSpec>` covering all item definitions, port names, and port types.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O((I + P) \log(I + P))$; Space Complexity: $O(I + P)$ over $I$ items and $P$ ports.
/// ## 6. Failure Modes & Fault Tolerance
/// - Schemas lacking both items and ports fall back safely to `SolverPortDto` with `model.attributes`.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
fn collect_model_dto_specs(model: &SysmlModel) -> Vec<DtoSpec> {
    let mut type_attrs: BTreeMap<String, Vec<AttributeDef>> = model.parts.iter()
        .filter(|p| !p.attributes.is_empty()).map(|p| (p.name.clone(), p.attributes.clone())).collect();
    type_attrs.extend(model.parts.iter().flat_map(|p| p.item_defs.iter()).map(|i| (i.name.clone(), i.attributes.clone())));
    let mut seen_dtos = BTreeSet::from(["Dto".to_string()]);
    let mut specs: Vec<DtoSpec> = model.parts.iter().flat_map(|p| p.item_defs.iter()).filter_map(|item| {
        let dto_name = format!("{}Dto", to_pascal_case(&item.name));
        seen_dtos.insert(dto_name.clone()).then(|| DtoSpec {
            source_name: item.name.clone(), dto_name, attrs: item.attributes.clone(),
        })
    }).collect();
    let mut all_ports: Vec<(String, String, Vec<AttributeDef>)> = model.ports.iter()
        .map(|p| (p.name.clone(), p.type_name.clone(), port_electrical_attrs(p, &[])))
        .chain(model.parts.iter().flat_map(|part| part.ports.iter().map(move |p| {
            (p.name.clone(), p.type_name.clone(), port_electrical_attrs(p, &part.attributes))
        }))).collect();
    if [all_ports.is_empty(), specs.is_empty()].into_iter().all(std::convert::identity) {
        all_ports.push(("SolverPort".to_string(), String::new(), model.attributes.clone()));
    }
    for (port_name, port_type, fallback_attrs) in all_ports {
        let key = Some(port_type.as_str()).filter(|s| !s.is_empty()).unwrap_or(&port_name);
        let attrs = type_attrs.get(key).cloned().unwrap_or(fallback_attrs);
        for candidate in [port_name, port_type] {
            let dto_name = format!("{}Dto", to_pascal_case(&candidate));
            if seen_dtos.insert(dto_name.clone()) {
                specs.push(DtoSpec { source_name: candidate, dto_name, attrs: attrs.clone() });
            }
        }
    }
    specs
}

/// Synthesizes a monolithic `solver_bridge.g.dart` source string containing FFI runtime and DTOs.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A single-unit C-ABI solver bridge contract compilation rules.
/// ## 2. Invariants & Mathematical Properties
/// - Deterministic pure function of `(model, schema_hash)` yielding byte-identical Dart output.
/// ## 3. Preconditions
/// - `model` is a valid `SysmlModel` reference and `schema_hash` is a 64-bit cryptographic digest.
/// ## 4. Postconditions
/// - Returns complete Dart source combining FFI preamble, all `DtoSpec` classes, and solver isolate.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(D \cdot A \log A)$; Space Complexity: $O(D \cdot A)$ for $D$ DTOs and $A$ attrs.
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible generation emitting `SolverPortDto` even when `model` contains zero declarations.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
pub fn generate_solver_bridge_dart(model: &SysmlModel, schema_hash: u64) -> String {
    let mut out = String::new();
    append_ffi_runtime_preamble(&mut out, schema_hash);
    for spec in collect_model_dto_specs(model) {
        append_typed_dto_class(&mut out, &spec.source_name, &spec.dto_name, &spec.attrs);
    }
    append_solver_isolate_class(&mut out);
    out
}

/// Emits modular FFI runtime, per-package DTO `.g.dart` slices, and the `solver_bridge.g.dart` barrel.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A modular compilation unit decomposition and barrel re-export parity.
/// ## 2. Invariants & Mathematical Properties
/// - Deduplicates relative `.g.dart` output paths via `BTreeSet` and guarantees `>= 1` DTO module.
/// ## 3. Preconditions
/// - `pkg` references the root `PackageDef` hierarchy and `schema_hash` holds the 64-bit digest.
/// ## 4. Postconditions
/// - Returns `(relative_path, dart_source)` tuples for FFI runtime, DTO slices, and barrel file.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(S \cdot D \log D)$; Space Complexity: $O(S \cdot D)$ over $S$ slices and $D$ DTOs.
/// ## 6. Failure Modes & Fault Tolerance
/// - Emits `lib/src/data/generated/dto/root/solver_port_dto.g.dart` when slices yield zero DTOs.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
pub fn emit_modular_bridge_files(pkg: &PackageDef, schema_hash: u64) -> Vec<(String, String)> {
    let mut runtime_src = String::new();
    append_ffi_runtime_preamble(&mut runtime_src, schema_hash);
    append_solver_isolate_class(&mut runtime_src);
    let mut files = vec![("lib/src/data/generated/ffi/solver_ffi_runtime.g.dart".to_string(), runtime_src)];
    let (mut dto_exports, mut seen_paths) = (Vec::new(), BTreeSet::new());
    for (pkg_slug, sub_pkg) in collect_package_slices(pkg) {
        let mut specs: Vec<DtoSpec> = sub_pkg.item_defs.iter()
            .filter(|i| !to_pascal_case(&i.name).is_empty())
            .map(|i| DtoSpec {
                source_name: i.name.clone(), dto_name: format!("{}Dto", to_pascal_case(&i.name)),
                attrs: i.attributes.clone(),
            }).collect();
        let has_items = !specs.is_empty();
        let has_defs = [!sub_pkg.part_defs.is_empty(), !sub_pkg.port_defs.is_empty()].into_iter().any(std::convert::identity);
        specs.extend(has_defs.then(|| collect_model_dto_specs(&SysmlModel {
            package_name: Some(sub_pkg.name.clone()), attributes: sub_pkg.attribute_defs.clone(),
            parts: sub_pkg.part_defs.clone(), ports: sub_pkg.port_defs.clone(), ..SysmlModel::default()
        })).into_iter().flatten().filter(|s| {
            [!has_items, s.source_name != "SolverPort"].into_iter().any(std::convert::identity)
        }));
        for spec in specs {
            let rel = format!("generated/dto/{pkg_slug}/{}_dto.g.dart", to_snake_case_dart(&spec.source_name));
            let full_path = format!("lib/src/data/{rel}");
            if seen_paths.insert(full_path.clone()) {
                let mut src = String::from("// GENERATED CODE - DO NOT MODIFY BY HAND\n\n");
                append_typed_dto_class(&mut src, &spec.source_name, &spec.dto_name, &spec.attrs);
                files.push((full_path, src));
                dto_exports.push(rel);
            }
        }
    }
    if dto_exports.is_empty() {
        let rel = "generated/dto/root/solver_port_dto.g.dart".to_string();
        let mut src = String::from("// GENERATED CODE - DO NOT MODIFY BY HAND\n\n");
        append_typed_dto_class(&mut src, "SolverPort", "SolverPortDto", &pkg.attribute_defs);
        files.push((format!("lib/src/data/{rel}"), src));
        dto_exports.push(rel);
    }
    let mut barrel = String::from("// GENERATED CODE - DO NOT MODIFY BY HAND\nexport 'generated/ffi/solver_ffi_runtime.g.dart';\n");
    dto_exports.iter().for_each(|rel| barrel.push_str(&format!("export '{rel}';\n")));
    files.push(("lib/src/data/solver_bridge.g.dart".to_string(), barrel));
    files
}
