//! Strongly-typed Dart extension-type view and modular schema model code emission.
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A and DO-330 TQL-4 deterministic Flutter model synthesis controls.
//! ## 2. Invariants & Mathematical Properties
//! - Emits zero-allocation `InstanceRecord` extension views in deterministic `BTreeSet` order.
//! ## 3. Preconditions
//! - Callers supply parsed `SysmlModel` or `PackageDef` AST nodes and normalized `TopologyNode` slices.
//! ## 4. Postconditions
//! - Produces valid Dart 3 extension types, `SchemaEntityIds`, `SchemaWidgetKeys`, and barrel exports.
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O((P + N) \cdot A \log A)$ over parts $P$, nodes $N$, and attributes $A$.
//! - Space Complexity: $O(P \cdot A + N)$ bounded by synthesized Dart source buffers and seen sets.
//! ## 6. Failure Modes & Fault Tolerance
//! - Total panic-free emission substituting safe typed defaults for malformed or omitted literals.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SCHEMA-MODELS]

use std::collections::BTreeSet;

use deap_core::sysml_ast::{AttributeDef, PackageDef, PartDef, SysmlModel};

use super::contracts::TopologyNode;
use super::naming::{escape_dart_single_quoted, extract_bounds_from_doc, to_camel_case, to_pascal_case, to_snake_case_dart};
use super::topology::{collect_package_slices, strip_surrounding_quotes};

/// Appends a strongly-typed Dart property getter and numeric range validator for a SysML attribute.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A type-safe model projection and SysML invariant range enforcement.
/// ## 2. Invariants & Mathematical Properties
/// - Maps `Real`/`Float`/`Double`, `Integer`/`Int`, `Boolean`/`Bool`, and fallback string attributes.
/// ## 3. Preconditions
/// - `out` is an open Dart extension-type body; `camel_attr` is a non-empty Dart member identifier.
/// ## 4. Postconditions
/// - Appends a null-coalescing `_record` getter and, for floating-point types, `validate<Attr>()`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ linear in attribute name, default literal, and doc string byte length $L$.
/// - Space Complexity: $O(L)$ bounded temporary formatting buffers appended to `out`.
/// ## 6. Failure Modes & Fault Tolerance
/// - Unparseable numeric or boolean defaults fall back deterministically to `0.0`, `0`, or `false`.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SCHEMA-MODELS]
fn append_part_attr_getter(out: &mut String, attr: &AttributeDef, camel_attr: &str) {
    let pascal_attr = to_pascal_case(&attr.name);
    let attr_esc = escape_dart_single_quoted(&attr.name);
    match attr.type_name.as_str() {
        "Real" | "Float" | "Double" => {
            let parsed = attr.default_value.as_deref()
                .and_then(|v| strip_surrounding_quotes(v).parse::<f64>().ok()).unwrap_or(0.0);
            let default_val = if parsed.fract() == 0.0 {
                format!("{parsed:.1}")
            } else {
                format!("{parsed}")
            };
            out.push_str(&format!("  double get {camel_attr} => _record.getPropertyDouble('{attr_esc}') ?? {default_val};\n"));
            let (min_opt, max_opt) = extract_bounds_from_doc(attr.doc.as_deref());
            let (min_val, max_val) = (min_opt.unwrap_or(0.0), max_opt.unwrap_or(1_000_000.0));
            out.push_str(&format!("\n  /// Validates SysML inv constraint for {camel_attr}\n  bool validate{pascal_attr}() {{\n    final v = {camel_attr};\n    return v >= {min_val:.1} && v <= {max_val:.1};\n  }}\n"));
        }
        "Integer" | "Int" => {
            let parsed = attr.default_value.as_deref()
                .and_then(|v| strip_surrounding_quotes(v).parse::<i64>().ok()).unwrap_or(0);
            out.push_str(&format!("  int get {camel_attr} => _record.getPropertyInt('{attr_esc}') ?? {parsed};\n"));
        }
        "Boolean" | "Bool" => {
            let is_true = attr.default_value.as_deref()
                .is_some_and(|v| strip_surrounding_quotes(v).eq_ignore_ascii_case("true"));
            let default_val = if is_true { "true" } else { "false" };
            out.push_str(&format!("  bool get {camel_attr} => _record.getPropertyBool('{attr_esc}') ?? {default_val};\n"));
        }
        _ => {
            let raw_def = attr.default_value.as_deref().map(strip_surrounding_quotes).unwrap_or("");
            out.push_str(&format!("  String get {camel_attr} => _record.getPropertyString('{attr_esc}') ?? '{}';\n", escape_dart_single_quoted(raw_def)));
        }
    }
}

/// Appends deduplicated Dart getters inferred from dynamic topology node JSON property bags.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A schema-topology reconciliation without duplicate member collisions.
/// ## 2. Invariants & Mathematical Properties
/// - Retains `BTreeMap` lexical key order per node and skips getters already recorded in `seen_getters`.
/// ## 3. Preconditions
/// - `out` holds an in-progress Dart extension body; `type_name` is a trimmed entity type identifier.
/// ## 4. Postconditions
/// - Appends `double`, `bool`, or `String` getters for unseen properties of matching `TopologyNode` items.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \cdot K \log G)$ across $N$ nodes, $K$ properties, and $G$ seen getters.
/// - Space Complexity: $O(G)$ additional entries inserted into `seen_getters`.
/// ## 6. Failure Modes & Fault Tolerance
/// - Whitespace-only or empty normalized keys are filtered out without emitting malformed Dart syntax.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SCHEMA-MODELS]
fn append_dynamic_node_getters(
    out: &mut String, nodes: &[TopologyNode], type_name: &str, seen_getters: &mut BTreeSet<String>,
) {
    let props = nodes.iter().filter(|n| n.type_name.trim() == type_name)
        .flat_map(|n| n.properties.iter())
        .map(|(k, v)| (k.trim(), to_camel_case(k.trim()), v))
        .filter(|(_, camel_attr, _)| !camel_attr.is_empty());
    for (key_trimmed, camel_attr, v) in props {
        if seen_getters.insert(camel_attr.clone()) {
            let attr_esc = escape_dart_single_quoted(key_trimmed);
            let line = match (v.is_number(), v.is_boolean()) {
                (true, _) => format!("  double get {camel_attr} => _record.getPropertyDouble('{attr_esc}') ?? 0.0;\n"),
                (false, true) => format!("  bool get {camel_attr} => _record.getPropertyBool('{attr_esc}') ?? false;\n"),
                (false, false) => format!("  String get {camel_attr} => _record.getPropertyString('{attr_esc}') ?? '';\n"),
            };
            out.push_str(&line);
        }
    }
}

/// Synthesizes a complete Dart 3 `extension type const <PascalType>View` declaration block.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A zero-cost typed wrapper synthesis over `InstanceRecord`.
/// ## 2. Invariants & Mathematical Properties
/// - Precedence invariant: declared `PartDef` attributes always precede inferred topology properties.
/// ## 3. Preconditions
/// - `type_name` identifies a candidate SysML part or topology entity type; `nodes` is initialized.
/// ## 4. Postconditions
/// - Returns `Some((view_name, dart_block))` when `to_pascal_case(type_name)` is non-empty, else `None`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O((A + N \cdot K) \log G)$ over part attributes $A$ and node properties $N \cdot K$.
/// - Space Complexity: $O(A + G)$ for the rendered Dart block and getter deduplication set.
/// ## 6. Failure Modes & Fault Tolerance
/// - Delimiter-only or empty `type_name` inputs safely return `None` without emitting invalid types.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SCHEMA-MODELS]
fn render_single_extension_view(
    type_name: &str, part_opt: Option<&PartDef>, nodes: &[TopologyNode],
) -> Option<(String, String)> {
    let pascal_type = to_pascal_case(type_name);
    if pascal_type.is_empty() {
        return None;
    }
    let view_name = format!("{pascal_type}View");
    let mut block = format!("/// Strongly-typed zero-allocation view projection over an InstanceRecord.\nextension type const {view_name}(InstanceRecord _record) implements InstanceRecord {{\n");
    let mut seen_getters = BTreeSet::new();
    let attrs = part_opt.into_iter().flat_map(|part| part.attributes.iter())
        .map(|attr| (attr, to_camel_case(&attr.name)))
        .filter(|(_, camel_attr)| !camel_attr.is_empty());
    for (attr, camel_attr) in attrs {
        if seen_getters.insert(camel_attr.clone()) {
            append_part_attr_getter(&mut block, attr, &camel_attr);
        }
    }
    append_dynamic_node_getters(&mut block, nodes, type_name, &mut seen_getters);
    block.push_str("}\n\n");
    Some((view_name, block))
}

/// Appends `SchemaEntityIds` compile-time constants and `SchemaWidgetKeys` UI test key factories.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A UI contract verification and zero-string-drift entity addressing.
/// ## 2. Invariants & Mathematical Properties
/// - Deduplicates `camelCase` entity and parent identifiers via `BTreeSet` while preserving first-seen order.
/// ## 3. Preconditions
/// - `out` is a mutable Dart source buffer importing `package:flutter/widgets.dart`.
/// ## 4. Postconditions
/// - Appends closed `abstract final class SchemaEntityIds` and `SchemaWidgetKeys` declarations to `out`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N)$ over $N$ topology nodes and optional parent references.
/// - Space Complexity: $O(N)$ bounded by `seen_ids` and appended constant declarations.
/// ## 6. Failure Modes & Fault Tolerance
/// - Escapes single quotes and `$` characters in raw IDs and ignores empty normalized identifiers.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SCHEMA-MODELS]
fn append_entity_ids_and_widget_keys(out: &mut String, nodes: &[TopologyNode]) {
    out.push_str("/// Authoritative compile-time schema entity IDs guaranteeing zero string drift.\nabstract final class SchemaEntityIds {\n");
    let mut seen_ids = BTreeSet::new();
    let raw_ids = nodes.iter()
        .flat_map(|node| [Some(node.id.as_str()), node.parent_id.as_deref()])
        .flatten().map(str::trim)
        .map(|trimmed| (trimmed, to_camel_case(trimmed)))
        .filter(|(_, camel_id)| !camel_id.is_empty());
    for (trimmed, camel_id) in raw_ids {
        if seen_ids.insert(camel_id.clone()) {
            out.push_str(&format!("  static const String {camel_id} = '{}';\n", escape_dart_single_quoted(trimmed)));
        }
    }
    out.push_str("}\n\n/// Standardized widget keys generated for infallible UI interaction testing.\nabstract final class SchemaWidgetKeys {\n  static Key entityRow(String id) => Key('entity_row_$id');\n  static Key entityInspector(String id) => Key('entity_inspector_$id');\n  static Key entityPropertyField(String id, String key) => Key('prop_${id}_$key');\n  static Key hudOverlay(String overlayId) => Key('hud_overlay_$overlayId');\n  static Key splitPaneSash(String sashId) => Key('split_sash_$sashId');\n}\n");
}

/// Generates the monolithic `generated_schema_models.g.dart` view, entity ID, and widget key contract.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 deterministic SysML-to-Flutter model compilation.
/// ## 2. Invariants & Mathematical Properties
/// - Emits each distinct `<PascalType>View` at most once across the union of `model.parts` and `nodes`.
/// ## 3. Preconditions
/// - `model` holds parsed SysML definitions and `nodes` holds resolved topology entity records.
/// ## 4. Postconditions
/// - Returns a complete, self-contained Dart library string with imports, views, IDs, and widget keys.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O((P + N) \cdot (P + A + N \cdot K) \log G)$ over parts $P$ and nodes $N$.
/// - Space Complexity: $O(P \cdot A + N \cdot K)$ for the assembled Dart source buffer.
/// ## 6. Failure Modes & Fault Tolerance
/// - Empty models and empty node slices still emit valid Dart imports and empty constant classes.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SCHEMA-MODELS]
pub fn generate_schema_models_dart(
    model: &SysmlModel, nodes: &[TopologyNode], _schema_hash: u64,
) -> String {
    let mut out = String::from("// GENERATED CODE - DO NOT MODIFY BY HAND\n// Generated by compile-sysml from SysML schema\n// Strongly-Typed Zero-Cost View Projections over InstanceRecord\n\nimport 'package:flutter/widgets.dart';\nimport 'package:app_flutter/models/instance_record.dart';\n\n");
    let mut seen_views = BTreeSet::new();
    let type_names = model.parts.iter().map(|p| p.name.trim())
        .chain(nodes.iter().map(|n| n.type_name.trim()))
        .filter(|t| !t.is_empty());
    for type_name in type_names {
        let part_opt = model.parts.iter().find(|p| p.name.trim() == type_name);
        if let Some((view_name, block)) = render_single_extension_view(type_name, part_opt, nodes) {
            if seen_views.insert(view_name) {
                out.push_str(&block);
            }
        }
    }
    append_entity_ids_and_widget_keys(&mut out, nodes);
    out
}

/// Emits package-scoped `.g.dart` view units, `schema_keys.g.dart`, and the re-export barrel file.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A modular artifact partitioning and collision-free barrel synthesis.
/// ## 2. Invariants & Mathematical Properties
/// - Deduplicates relative output paths via `BTreeSet` and guarantees barrel exports match emitted units.
/// ## 3. Preconditions
/// - `pkg` references a parsed SysML root `PackageDef` hierarchy and `nodes` holds topology entities.
/// ## 4. Postconditions
/// - Returns `(relative_path, dart_source)` pairs ending with `schema_keys.g.dart` and the barrel file.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(S \cdot P_s \cdot (A + N \cdot K) \log G)$ across $S$ slices and $P_s$ parts.
/// - Space Complexity: $O(U + N)$ proportional to total emitted unit bytes $U$ and path set size.
/// ## 6. Failure Modes & Fault Tolerance
/// - Packages without valid part names still emit `schema_keys.g.dart` and a valid barrel export file.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-SCHEMA-MODELS]
pub fn emit_modular_model_files(
    pkg: &PackageDef, nodes: &[TopologyNode], _schema_hash: u64,
) -> Vec<(String, String)> {
    let (mut files, mut exports, mut seen_paths) = (Vec::new(), Vec::new(), BTreeSet::new());
    let slice_parts = collect_package_slices(pkg).into_iter()
        .flat_map(|(slug, sub_pkg)| sub_pkg.part_defs.iter().map(move |part| (slug.clone(), part)));
    for (pkg_slug, part) in slice_parts {
        if let Some((_, view_block)) = render_single_extension_view(part.name.trim(), Some(part), nodes) {
            let rel_under_models = format!("generated/{pkg_slug}/{}_view.g.dart", to_snake_case_dart(&part.name));
            let full_rel = format!("lib/src/models/{rel_under_models}");
            if seen_paths.insert(full_rel.clone()) {
                let unit_src = format!("// GENERATED CODE - DO NOT MODIFY BY HAND\nimport 'package:app_flutter/models/instance_record.dart';\n\n{view_block}");
                files.push((full_rel, unit_src));
                exports.push(format!("export '{rel_under_models}';\n"));
            }
        }
    }
    let mut keys_src = String::from("// GENERATED CODE - DO NOT MODIFY BY HAND\nimport 'package:flutter/widgets.dart';\n\n");
    append_entity_ids_and_widget_keys(&mut keys_src, nodes);
    files.push(("lib/src/models/generated/schema_keys.g.dart".to_string(), keys_src));
    exports.push("export 'generated/schema_keys.g.dart';\n".to_string());
    files.push(("lib/src/models/generated_schema_models.g.dart".to_string(), format!("// GENERATED CODE - DO NOT MODIFY BY HAND\n{}", exports.concat())));
    files
}
