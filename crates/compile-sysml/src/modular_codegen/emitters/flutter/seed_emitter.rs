//! SQLite DDL, EAV property, and modular package seed Dart code emission for Flutter.
//!
//! Safety & Traceability: Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
//! Contract: Synthesizes injection-safe Dart SQLite DDL, metamodel metadata, instance rows, and modular package barrels ordered deterministically via `BTreeMap` and `BTreeSet` from `SysmlModel` or `PackageDef` trees and `TopologyNode` slices.
//! Bounds & Faults: Total panic-free string synthesis degrading unparsed attribute defaults safely to escaped text literals in `O(P * A + N * K log K)` time.

use super::contracts::TopologyNode;
use super::naming::{
    escape_dart_single_quoted, extract_bounds_from_doc, format_display_name, to_pascal_case,
};
use super::topology::{collect_package_slices, strip_surrounding_quotes};
use deap_core::sysml_ast::{PackageDef, PartDef, SysmlModel};
use std::collections::{BTreeMap, BTreeSet};

/// Maps a SysML primitive attribute type identifier onto `"double"`, `"int"`, `"bool"`, or `"text"`.
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
fn map_sysml_attr_field_type(type_name: &str) -> &'static str {
    match type_name {
        "Real" | "Float" | "Double" => "double",
        "Integer" | "Int" => "int",
        "Boolean" | "Bool" => "bool",
        _ => "text",
    }
}

/// Infers the SQLite `type_attributes` field category and Dart SQL default literal from a JSON value.
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
fn format_json_prop_type_and_default(val: &serde_json::Value) -> (&'static str, String) {
    let num_res = val.as_f64().map(|num| ("double", format!("'{num}'")));
    let bool_res = val.as_bool().map(|b| ("bool", format!("'{b}'")));
    let str_res = val
        .as_str()
        .map(|s| ("text", format!("'{}'", escape_dart_single_quoted(s))));
    num_res
        .or(bool_res)
        .or(str_res)
        .unwrap_or(("text", "null".to_string()))
}

/// Parses a raw SysML attribute default expression into a typed `serde_json::Value` for EAV seeding.
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
fn parse_attr_default_json(type_name: &str, raw_def: &str) -> serde_json::Value {
    let trimmed = raw_def.trim();
    match map_sysml_attr_field_type(type_name) {
        "double" | "int" => trimmed
            .parse::<f64>()
            .map_or(serde_json::Value::String(trimmed.to_string()), |n| {
                serde_json::json!(n)
            }),
        "bool" => trimmed.to_ascii_lowercase().parse::<bool>().map_or(
            serde_json::Value::String(trimmed.to_string()),
            serde_json::Value::Bool,
        ),
        _ => serde_json::Value::String(strip_surrounding_quotes(trimmed).to_string()),
    }
}

/// Appends the five canonical DEAP SQLite table DDL statements and three lookup index definitions.
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
fn append_seed_ddl(out: &mut String) {
    for stmt in [
        "CREATE TABLE IF NOT EXISTS type_definitions (type_name TEXT PRIMARY KEY, display_name TEXT NOT NULL, icon_symbol TEXT NOT NULL)",
        "CREATE TABLE IF NOT EXISTS type_attributes (id INTEGER PRIMARY KEY AUTOINCREMENT, type_name TEXT NOT NULL REFERENCES type_definitions(type_name) ON DELETE CASCADE, key TEXT NOT NULL, label TEXT NOT NULL, field_type TEXT NOT NULL, section_label TEXT, section_order INTEGER DEFAULT 0, is_required INTEGER DEFAULT 0, min_value REAL, max_value REAL, regex_pattern TEXT, enum_options TEXT, enum_display_names TEXT, default_value TEXT, input_formatters TEXT, UNIQUE(type_name, key))",
        "CREATE TABLE IF NOT EXISTS type_relations (id INTEGER PRIMARY KEY AUTOINCREMENT, parent_type TEXT NOT NULL REFERENCES type_definitions(type_name) ON DELETE CASCADE, relation_name TEXT NOT NULL, child_type TEXT NOT NULL REFERENCES type_definitions(type_name) ON DELETE CASCADE, display_label TEXT NOT NULL)",
        "CREATE TABLE IF NOT EXISTS instances (id TEXT PRIMARY KEY, type_name TEXT NOT NULL REFERENCES type_definitions(type_name) ON DELETE RESTRICT, label TEXT NOT NULL, parent_id TEXT REFERENCES instances(id) ON DELETE SET NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL)",
        "CREATE TABLE IF NOT EXISTS properties (id INTEGER PRIMARY KEY AUTOINCREMENT, instance_id TEXT NOT NULL REFERENCES instances(id) ON DELETE CASCADE, key TEXT NOT NULL, value_text TEXT, value_num REAL, value_type TEXT NOT NULL, UNIQUE(instance_id, key))",
        "CREATE INDEX IF NOT EXISTS idx_properties_lookup ON properties(instance_id, key)",
        "CREATE INDEX IF NOT EXISTS idx_instances_type ON instances(type_name)",
        "CREATE INDEX IF NOT EXISTS idx_instances_parent ON instances(parent_id)",
    ] {
        out.push_str("    await txn.execute('''\n      ");
        out.push_str(stmt);
        out.push_str("\n    ''');\n");
    }
}

/// Appends deduplicated `type_definitions` and structural `type_relations` SQLite batch inserts.
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
fn append_seed_types_and_relations(out: &mut String, parts: &[PartDef], nodes: &[TopologyNode]) {
    let mut seen_types = BTreeSet::new();
    let candidate_types = parts
        .iter()
        .map(|p| p.name.trim())
        .chain(nodes.iter().map(|n| n.type_name.trim()))
        .filter(|name| !name.is_empty());
    for t_name in candidate_types {
        if seen_types.insert(t_name.to_string()) {
            let esc_type = escape_dart_single_quoted(t_name);
            let esc_display = escape_dart_single_quoted(&format_display_name(t_name));
            out.push_str(&format!("    batch.rawInsert('INSERT OR REPLACE INTO type_definitions (type_name, display_name, icon_symbol) VALUES (?, ?, ?)', ['{esc_type}', '{esc_display}', 'layers']);\n"));
        }
    }
    for (part, child) in parts
        .iter()
        .flat_map(|p| p.parts.iter().map(move |c| (p, c)))
    {
        let esc_part = escape_dart_single_quoted(&part.name);
        let esc_child = escape_dart_single_quoted(&child.name);
        let esc_label = escape_dart_single_quoted(&format_display_name(&child.name));
        out.push_str(&format!("    batch.rawInsert('INSERT OR REPLACE INTO type_relations (parent_type, relation_name, child_type, display_label) VALUES (?, ?, ?, ?)', ['{esc_part}', 'contains', '{esc_child}', '{esc_label}']);\n"));
    }
}

/// Appends `type_attributes` metadata rows derived from SysML `PartDef` attributes and dynamic node properties.
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
fn append_seed_type_attributes(out: &mut String, parts: &[PartDef], nodes: &[TopologyNode]) {
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut next_order: BTreeMap<String, usize> = BTreeMap::new();
    for (part, order, attr) in parts.iter().flat_map(|p| {
        p.attributes
            .iter()
            .enumerate()
            .map(move |(ord, a)| (p, ord, a))
    }) {
        seen.insert((part.name.clone(), attr.name.clone()));
        next_order.insert(part.name.clone(), order.saturating_add(1));
        let field_type = map_sysml_attr_field_type(&attr.type_name);
        let (min_opt, max_opt) = extract_bounds_from_doc(attr.doc.as_deref());
        let min_str = min_opt.map_or("null".to_string(), |v| format!("{v:.1}"));
        let max_str = max_opt.map_or("null".to_string(), |v| format!("{v:.1}"));
        let default_str = attr
            .default_value
            .as_deref()
            .map_or("null".to_string(), |raw| {
                format!(
                    "'{}'",
                    escape_dart_single_quoted(strip_surrounding_quotes(raw))
                )
            });
        let (esc_t, esc_k, esc_l) = (
            escape_dart_single_quoted(&part.name),
            escape_dart_single_quoted(&attr.name),
            escape_dart_single_quoted(&format_display_name(&attr.name)),
        );
        out.push_str(&format!("    batch.rawInsert('INSERT OR REPLACE INTO type_attributes (type_name, key, label, field_type, section_label, section_order, is_required, min_value, max_value, default_value) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)', ['{esc_t}', '{esc_k}', '{esc_l}', '{field_type}', 'General', {order}, 1, {min_str}, {max_str}, {default_str}]);\n"));
    }
    let node_props = nodes
        .iter()
        .map(|n| (n.type_name.trim(), &n.properties))
        .filter(|(t, _)| !t.is_empty())
        .flat_map(|(t, props)| props.iter().map(move |(k, v)| (t, k.trim(), v)))
        .filter(|(_, key, _)| !key.is_empty());
    for (t_name, key, val) in node_props {
        if seen.insert((t_name.to_string(), key.to_string())) {
            let order = next_order.get(t_name).copied().unwrap_or(0);
            next_order.insert(t_name.to_string(), order.saturating_add(1));
            let (field_type, default_str) = format_json_prop_type_and_default(val);
            let (esc_t, esc_k, esc_l) = (
                escape_dart_single_quoted(t_name),
                escape_dart_single_quoted(key),
                escape_dart_single_quoted(&format_display_name(key)),
            );
            out.push_str(&format!("    batch.rawInsert('INSERT OR REPLACE INTO type_attributes (type_name, key, label, field_type, section_label, section_order, is_required, min_value, max_value, default_value) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)', ['{esc_t}', '{esc_k}', '{esc_l}', '{field_type}', 'General', {order}, 1, null, null, {default_str}]);\n"));
        }
    }
}

/// Appends merged instance and `PartDef`-default EAV `properties` rows for a single `TopologyNode`.
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
fn append_node_eav_properties(out: &mut String, parts: &[PartDef], node: &TopologyNode) {
    let node_id_esc = escape_dart_single_quoted(&node.id);
    let mut merged = node.properties.clone();
    let defaults = parts
        .iter()
        .find(|p| p.name == node.type_name)
        .into_iter()
        .flat_map(|part| &part.attributes)
        .filter_map(|a| {
            a.default_value
                .as_deref()
                .map(|d| (&a.name, &a.type_name, d))
        });
    for (name, t_name, raw_def) in defaults {
        merged
            .entry(name.clone())
            .or_insert(parse_attr_default_json(t_name, raw_def));
    }
    for (k, v) in &merged {
        let key_esc = escape_dart_single_quoted(k);
        let num_tail = v.as_f64().map(|num| format!("null, {num}, 'double'"));
        let bool_tail = v
            .as_bool()
            .map(|b| format!("'{b}', {}, 'bool'", u8::from(b)));
        let str_tail = v
            .as_str()
            .map(|s| format!("'{}', null, 'text'", escape_dart_single_quoted(s)));
        if let Some(tail) = num_tail.or(bool_tail).or(str_tail) {
            out.push_str(&format!("    batch.rawInsert('INSERT OR REPLACE INTO properties (instance_id, key, value_text, value_num, value_type) VALUES (?, ?, ?, ?, ?)', ['{node_id_esc}', '{key_esc}', {tail}]);\n"));
        }
    }
}

/// Synthesizes the complete `seedGeneratedDomainDatabase` Dart SQLite seed source for a `SysmlModel`.
///
/// Safety & Traceability: Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
/// Contract: Wraps DDL execution and batched metamodel/instance inserts inside one atomic `db.transaction` block exporting `kDeapSchemaHash` aliases and `seedGeneratedDomainDatabase`.
/// Bounds & Faults: Empty models and empty node slices produce valid transactional Dart DDL seed code in `O(P * A + N * (P + K log K))` time without panicking.
pub fn generate_domain_seed_dart(
    model: &SysmlModel,
    nodes: &[TopologyNode],
    schema_hash: u64,
) -> String {
    let mut out = format!(
        "// GENERATED CODE - DO NOT MODIFY BY HAND\nimport 'package:sqflite_common_ffi/sqflite_ffi.dart';\n\nconst int kDeapSchemaHash = 0x{schema_hash:016X};\nconst int deapSchemaHash = kDeapSchemaHash;\nconst int DEAP_SCHEMA_HASH = kDeapSchemaHash;\n\nFuture<void> seedGeneratedDomainDatabase(Database db) async {{\n  await db.transaction((txn) async {{\n"
    );
    append_seed_ddl(&mut out);
    out.push_str("    final batch = txn.batch();\n\n");
    append_seed_types_and_relations(&mut out, &model.parts, nodes);
    append_seed_type_attributes(&mut out, &model.parts, nodes);
    for node in nodes {
        let (id_esc, type_esc, label_esc) = (
            escape_dart_single_quoted(&node.id),
            escape_dart_single_quoted(node.type_name.trim()),
            escape_dart_single_quoted(&node.label),
        );
        let parent_str = node
            .parent_id
            .as_deref()
            .filter(|p| !p.is_empty())
            .map_or("null".to_string(), |p| {
                format!("'{}'", escape_dart_single_quoted(p))
            });
        out.push_str(&format!("    batch.rawInsert('INSERT OR REPLACE INTO instances (id, type_name, label, parent_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)', ['{id_esc}', '{type_esc}', '{label_esc}', {parent_str}, {}, {}]);\n", node.created_at, node.updated_at));
        append_node_eav_properties(&mut out, &model.parts, node);
    }
    out.push_str("    await batch.commit(noResult: true);\n  });\n}\n");
    out
}

/// Emits modular per-package seed `.g.dart` units, shared canonical DDL, and the re-exporting barrel file.
///
/// Safety & Traceability: Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
/// Contract: Deduplicates `snake_case` package slugs via `BTreeSet` and emits `(relative_path, dart_source)` pairs for `ddl_schema.g.dart`, `<slug>_seed.g.dart`, and `generated_domain_seed.g.dart`.
/// Bounds & Faults: Duplicate package stems are skipped deterministically after first emission in pre-order traversal with zero panics.
pub fn emit_modular_seed_files(
    pkg: &PackageDef,
    nodes: &[TopologyNode],
    schema_hash: u64,
) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let mut ddl = format!("// GENERATED CODE - DO NOT MODIFY BY HAND\nimport 'package:sqflite_common_ffi/sqflite_ffi.dart';\nconst int kDeapSchemaHash = 0x{schema_hash:016X};\nFuture<void> applyCanonicalDeapDdl(DatabaseExecutor txn) async {{\n");
    append_seed_ddl(&mut ddl);
    ddl.push_str("}\n");
    files.push((
        "lib/src/data/seeds/generated/ddl_schema.g.dart".to_string(),
        ddl,
    ));
    let mut barrel = String::from(
        "// GENERATED CODE - DO NOT MODIFY BY HAND\nexport 'generated/ddl_schema.g.dart';\n",
    );
    let mut seen_slugs = BTreeSet::new();
    for (slug, sub_pkg) in collect_package_slices(pkg) {
        if seen_slugs.insert(slug.clone()) {
            let sub_model = SysmlModel {
                package_name: Some(sub_pkg.name.clone()),
                parts: sub_pkg.part_defs.clone(),
                ..SysmlModel::default()
            };
            let fn_name = format!("seed{}PackageDatabase", to_pascal_case(&slug));
            let content = generate_domain_seed_dart(&sub_model, nodes, schema_hash)
                .replace("seedGeneratedDomainDatabase", &fn_name);
            files.push((
                format!("lib/src/data/seeds/generated/{slug}_seed.g.dart"),
                content,
            ));
            barrel.push_str(&format!("export 'generated/{slug}_seed.g.dart';\n"));
        }
    }
    files.push((
        "lib/src/data/seeds/generated_domain_seed.g.dart".to_string(),
        barrel,
    ));
    files
}
