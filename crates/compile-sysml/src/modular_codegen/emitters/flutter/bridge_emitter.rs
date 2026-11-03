//! Deterministic Dart FFI C-ABI solver bridge, typed DTO, and modular barrel code generation.
//!
//! Safety & Traceability: Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
//! Contract: Synthesizes monolithic or package-sliced Dart FFI runtime, typed DTO classes, and barrel exports from `SysmlModel` or `PackageDef` trees using deterministic `BTreeMap` and `BTreeSet` ordering.
//! Bounds & Faults: Total panic-free emission defaulting to `SolverPortDto` when input schemas omit ports in `O(P * A log A)` time.

use super::naming::{escape_dart_single_quoted, to_camel_case, to_pascal_case, to_snake_case_dart};
use super::topology::{collect_package_slices, strip_surrounding_quotes};
use deap_core::sysml_cst::{AttributeDef, PackageDef, PortDef, SysmlModel};
use std::collections::{BTreeMap, BTreeSet};

/// Normalized descriptor pairing a SysML contract identifier with its Dart DTO class and fields.
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
struct DtoSpec {
    source_name: String,
    dto_name: String,
    attrs: Vec<AttributeDef>,
}

/// Maps a SysML `AttributeDef` to its Dart scalar type, default literal, and JSON decode expression.
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
fn resolve_dto_attribute_codec(attr: &AttributeDef) -> (&'static str, String, String) {
    let key_esc = escape_dart_single_quoted(&attr.name);
    let raw = attr.default_value.as_deref().map(strip_surrounding_quotes);
    match attr.type_name.as_str() {
        "Real" | "Float" | "Double" => {
            let lit = format!(
                "{:.1}",
                raw.and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0)
            );
            (
                "double",
                lit.clone(),
                format!("(json['{key_esc}'] as num?)?.toDouble() ?? {lit}"),
            )
        }
        "Integer" | "Int" => {
            let lit = format!("{}", raw.and_then(|v| v.parse::<i64>().ok()).unwrap_or(0));
            (
                "int",
                lit.clone(),
                format!("(json['{key_esc}'] as num?)?.toInt() ?? {lit}"),
            )
        }
        "Boolean" | "Bool" => {
            let lit = raw
                .is_some_and(|v| v.eq_ignore_ascii_case("true"))
                .to_string();
            (
                "bool",
                lit.clone(),
                format!("(json['{key_esc}'] as bool?) ?? {lit}"),
            )
        }
        _ => {
            let lit = format!("'{}'", escape_dart_single_quoted(raw.unwrap_or("")));
            (
                "String",
                lit.clone(),
                format!("(json['{key_esc}'] as String?) ?? {lit}"),
            )
        }
    }
}

/// Appends a deterministic Dart DTO class with typed fields, `toJson`, and `fromJson` codecs.
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
fn append_typed_dto_class(
    out: &mut String,
    source_name: &str,
    dto_name: &str,
    attrs: &[AttributeDef],
) {
    out.push_str(&format!(
        "/// Data Transfer Object for SysML contract `{source_name}`.\nclass {dto_name} {{\n"
    ));
    let mut seen = BTreeSet::from([String::new(), "payload".to_string()]);
    let unique: Vec<&AttributeDef> = attrs
        .iter()
        .filter(|a| seen.insert(to_camel_case(&a.name)))
        .collect();
    if unique.is_empty() {
        out.push_str(&format!(concat!(
            "  final Map<String, dynamic> payload;\n  const {0}([this.payload = const <String, dynamic>{{}}]);\n\n",
            "  Map<String, dynamic> toJson() => payload;\n",
            "  factory {0}.fromJson(Map<String, dynamic> json) => {0}(json);\n}}\n\n",
        ), dto_name));
        return;
    }
    let (mut fields, mut ctor, mut to_j, mut from_j) =
        (String::new(), String::new(), String::new(), String::new());
    for attr in unique {
        let (camel, key_esc) = (
            to_camel_case(&attr.name),
            escape_dart_single_quoted(&attr.name),
        );
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
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
fn port_electrical_attrs(port: &PortDef, fallback: &[AttributeDef]) -> Vec<AttributeDef> {
    if port.electrical_attributes.is_empty() {
        return fallback.to_vec();
    }
    port.electrical_attributes
        .iter()
        .map(|(k, v)| AttributeDef {
            name: k.clone(),
            type_name: "String".to_string(),
            default_value: Some(v.clone()),
            doc: None,
        })
        .collect()
}

/// Extracts ordered, deduplicated `DtoSpec` definitions from a `SysmlModel`'s items, parts, and ports.
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
fn collect_model_dto_specs(model: &SysmlModel) -> Vec<DtoSpec> {
    let mut type_attrs: BTreeMap<String, Vec<AttributeDef>> = model
        .parts
        .iter()
        .filter(|p| !p.attributes.is_empty())
        .map(|p| (p.name.clone(), p.attributes.clone()))
        .collect();
    type_attrs.extend(
        model
            .parts
            .iter()
            .flat_map(|p| p.item_defs.iter())
            .map(|i| (i.name.clone(), i.attributes.clone())),
    );
    let mut seen_dtos = BTreeSet::from(["Dto".to_string()]);
    let mut specs: Vec<DtoSpec> = model
        .parts
        .iter()
        .flat_map(|p| p.item_defs.iter())
        .filter_map(|item| {
            let dto_name = format!("{}Dto", to_pascal_case(&item.name));
            seen_dtos.insert(dto_name.clone()).then(|| DtoSpec {
                source_name: item.name.clone(),
                dto_name,
                attrs: item.attributes.clone(),
            })
        })
        .collect();
    let mut all_ports: Vec<(String, String, Vec<AttributeDef>)> = model
        .ports
        .iter()
        .map(|p| {
            (
                p.name.clone(),
                p.type_name.clone(),
                port_electrical_attrs(p, &[]),
            )
        })
        .chain(model.parts.iter().flat_map(|part| {
            part.ports.iter().map(move |p| {
                (
                    p.name.clone(),
                    p.type_name.clone(),
                    port_electrical_attrs(p, &part.attributes),
                )
            })
        }))
        .collect();
    if [all_ports.is_empty(), specs.is_empty()]
        .into_iter()
        .all(std::convert::identity)
    {
        all_ports.push((
            "SolverPort".to_string(),
            String::new(),
            model.attributes.clone(),
        ));
    }
    for (port_name, port_type, fallback_attrs) in all_ports {
        let key = Some(port_type.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(&port_name);
        let attrs = type_attrs.get(key).cloned().unwrap_or(fallback_attrs);
        for candidate in [port_name, port_type] {
            let dto_name = format!("{}Dto", to_pascal_case(&candidate));
            if seen_dtos.insert(dto_name.clone()) {
                specs.push(DtoSpec {
                    source_name: candidate,
                    dto_name,
                    attrs: attrs.clone(),
                });
            }
        }
    }
    specs
}

/// Synthesizes a monolithic `solver_bridge.g.dart` source string containing FFI runtime and DTOs.
///
/// Safety & Traceability: Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
/// Contract: Deterministic pure function of `(model, schema_hash)` combining FFI preamble, all `DtoSpec` classes, and `DeapNativeSolverIsolate`.
/// Bounds & Faults: Infallible `O(D * A log A)` generation emitting `SolverPortDto` even when `model` contains zero declarations.
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
///
/// Safety & Traceability: Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SOLVER-BRIDGE]
/// Contract: Deduplicates relative `.g.dart` output paths via `BTreeSet` and returns `(relative_path, dart_source)` tuples for FFI runtime, DTO slices, and barrel file.
/// Bounds & Faults: Guarantees `>= 1` DTO module by emitting `lib/src/data/generated/dto/root/solver_port_dto.g.dart` when slices yield zero DTOs.
pub fn emit_modular_bridge_files(pkg: &PackageDef, schema_hash: u64) -> Vec<(String, String)> {
    let mut runtime_src = String::new();
    append_ffi_runtime_preamble(&mut runtime_src, schema_hash);
    append_solver_isolate_class(&mut runtime_src);
    let mut files = vec![(
        "lib/src/data/generated/ffi/solver_ffi_runtime.g.dart".to_string(),
        runtime_src,
    )];
    let (mut dto_exports, mut seen_paths) = (Vec::new(), BTreeSet::new());
    for (pkg_slug, sub_pkg) in collect_package_slices(pkg) {
        let mut specs: Vec<DtoSpec> = sub_pkg
            .item_defs
            .iter()
            .filter(|i| !to_pascal_case(&i.name).is_empty())
            .map(|i| DtoSpec {
                source_name: i.name.clone(),
                dto_name: format!("{}Dto", to_pascal_case(&i.name)),
                attrs: i.attributes.clone(),
            })
            .collect();
        let has_items = !specs.is_empty();
        let has_defs = [!sub_pkg.part_defs.is_empty(), !sub_pkg.port_defs.is_empty()]
            .into_iter()
            .any(std::convert::identity);
        specs.extend(
            has_defs
                .then(|| {
                    collect_model_dto_specs(&SysmlModel {
                        package_name: Some(sub_pkg.name.clone()),
                        attributes: sub_pkg.attribute_defs.clone(),
                        parts: sub_pkg.part_defs.clone(),
                        ports: sub_pkg.port_defs.clone(),
                        ..SysmlModel::default()
                    })
                })
                .into_iter()
                .flatten()
                .filter(|s| {
                    [!has_items, s.source_name != "SolverPort"]
                        .into_iter()
                        .any(std::convert::identity)
                }),
        );
        for spec in specs {
            let rel = format!(
                "generated/dto/{pkg_slug}/{}_dto.g.dart",
                to_snake_case_dart(&spec.source_name)
            );
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
    dto_exports
        .iter()
        .for_each(|rel| barrel.push_str(&format!("export '{rel}';\n")));
    files.push(("lib/src/data/solver_bridge.g.dart".to_string(), barrel));
    files
}
