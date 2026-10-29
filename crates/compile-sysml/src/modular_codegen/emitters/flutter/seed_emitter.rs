//! SQLite DDL, EAV property, and modular package seed Dart code emission for Flutter.
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A and DO-330 TQL-4 deterministic database seed synthesis controls.
//! ## 2. Invariants & Mathematical Properties
//! - Emits injection-safe Dart SQLite batches ordered deterministically via `BTreeMap` and `BTreeSet`.
//! ## 3. Preconditions
//! - Callers supply parsed `SysmlModel` or `PackageDef` trees, normalized `TopologyNode` slices, and a 64-bit hash.
//! ## 4. Postconditions
//! - Produces canonical SQLite schema DDL, typed metamodel metadata, instance rows, and modular package barrels.
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(P \cdot A + N \cdot K \log K)$ over $P$ parts, $A$ attributes, and $N$ nodes with $K$ properties.
//! - Space Complexity: $O(P \cdot A + N \cdot K)$ bounded by output Dart source buffers and ordered tracking sets.
//! ## 6. Failure Modes & Fault Tolerance
//! - Total panic-free string synthesis degrading unparsed attribute defaults safely to escaped text literals.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]

use std::collections::{BTreeMap, BTreeSet};
use deap_core::sysml_ast::{PackageDef, PartDef, SysmlModel};
use super::contracts::TopologyNode;
use super::naming::{escape_dart_single_quoted, extract_bounds_from_doc, format_display_name, to_pascal_case};
use super::topology::{collect_package_slices, strip_surrounding_quotes};

/// Maps a SysML primitive attribute type identifier onto the canonical Flutter inspector field type.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A deterministic metamodel-to-UI scalar type projection standards.
/// ## 2. Invariants & Mathematical Properties
/// - Total many-to-one mapping collapsing numeric, integral, boolean, and fallback textual domains.
/// ## 3. Preconditions
/// - `type_name` is a UTF-8 slice naming a SysML attribute type reference.
/// ## 4. Postconditions
/// - Returns one of `"double"`, `"int"`, `"bool"`, or `"text"`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ bounded lexical literal comparison.
/// - Space Complexity: $O(1)$ static string slice return with zero heap allocation.
/// ## 6. Failure Modes & Fault Tolerance
/// - Unrecognized or domain-defined types fall back deterministically to `"text"`.
/// ## 7. Realization & Traceability
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
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A dynamic topology property normalization and literal escaping rules.
/// ## 2. Invariants & Mathematical Properties
/// - Prioritizes numeric scalars, then booleans, then escaped single-quoted strings, else SQL `null`.
/// ## 3. Preconditions
/// - `val` references any valid `serde_json::Value` attached to a `TopologyNode`.
/// ## 4. Postconditions
/// - Returns `(field_type, dart_sql_default_literal)` safe for embedding in generated `rawInsert` lists.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ linear in string value byte length $L$, or $O(1)$ for scalar primitives.
/// - Space Complexity: $O(L)$ bounded by the formatted literal buffer.
/// ## 6. Failure Modes & Fault Tolerance
/// - Arrays, objects, and JSON nulls degrade cleanly to `("text", "null")`.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
fn format_json_prop_type_and_default(val: &serde_json::Value) -> (&'static str, String) {
    let num_res = val.as_f64().map(|num| ("double", format!("'{num}'")));
    let bool_res = val.as_bool().map(|b| ("bool", format!("'{b}'")));
    let str_res = val.as_str().map(|s| ("text", format!("'{}'", escape_dart_single_quoted(s))));
    num_res.or(bool_res).or(str_res).unwrap_or(("text", "null".to_string()))
}

/// Parses a raw SysML attribute default expression into a typed `serde_json::Value` for EAV seeding.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A lossless schema default hydration for entity-attribute-value rows.
/// ## 2. Invariants & Mathematical Properties
/// - Coerces numeric and ASCII case-insensitive boolean literals; strips symmetric quotes on text defaults.
/// ## 3. Preconditions
/// - `type_name` designates the SysML attribute type and `raw_def` holds the unparsed default slice.
/// ## 4. Postconditions
/// - Returns a `serde_json::Value` ready for deterministic EAV property table emission.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(L)$ linear in the byte length $L$ of `raw_def`.
/// - Space Complexity: $O(L)$ bounded by owned string conversion buffers.
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-numeric or symbolic expressions preserve their trimmed textual representation without panicking.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
fn parse_attr_default_json(type_name: &str, raw_def: &str) -> serde_json::Value {
    let trimmed = raw_def.trim();
    match map_sysml_attr_field_type(type_name) {
        "double" | "int" => trimmed.parse::<f64>().map_or(serde_json::Value::String(trimmed.to_string()), |n| serde_json::json!(n)),
        "bool" => trimmed.to_ascii_lowercase().parse::<bool>().map_or(serde_json::Value::String(trimmed.to_string()), serde_json::Value::Bool),
        _ => serde_json::Value::String(strip_surrounding_quotes(trimmed).to_string()),
    }
}

/// Appends the five canonical DEAP SQLite table DDL statements and three lookup index definitions.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A relational schema integrity and cascading foreign-key enforcement.
/// ## 2. Invariants & Mathematical Properties
/// - Idempotent `IF NOT EXISTS` DDL covering metamodel types, attributes, relations, instances, and EAV properties.
/// ## 3. Preconditions
/// - `out` is a mutable Dart source buffer positioned inside an active `txn` asynchronous block.
/// ## 4. Postconditions
/// - Appends eight `await txn.execute(''' ... ''');` statements defining tables and secondary indexes.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ fixed eight-statement constant emission.
/// - Space Complexity: $O(1)$ constant DDL byte footprint appended to `out`.
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible in-memory string concatenation over static SQLite schema literals.
/// ## 7. Realization & Traceability
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
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A structural containment traceability between SysML `PartDef` blocks.
/// ## 2. Invariants & Mathematical Properties
/// - Tracks visited type identifiers in `BTreeSet` so each distinct type emits at most one definition row.
/// ## 3. Preconditions
/// - `out` is initialized; `parts` and `nodes` hold the active model parts and resolved topology entities.
/// ## 4. Postconditions
/// - Appends `batch.rawInsert` statements for all non-empty part/node types and nested `contains` relations.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O((P + N) \log(P + N) + R)$ where $R$ is total nested child-part usages.
/// - Space Complexity: $O(P + N)$ for the ordered `seen_types` deduplication set.
/// ## 6. Failure Modes & Fault Tolerance
/// - Blank or whitespace-only topology `type_name` fields are filtered out safely before emission.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
fn append_seed_types_and_relations(out: &mut String, parts: &[PartDef], nodes: &[TopologyNode]) {
    let mut seen_types = BTreeSet::new();
    let candidate_types = parts.iter().map(|p| p.name.trim())
        .chain(nodes.iter().map(|n| n.type_name.trim())).filter(|name| !name.is_empty());
    for t_name in candidate_types {
        if seen_types.insert(t_name.to_string()) {
            let esc_type = escape_dart_single_quoted(t_name);
            let esc_display = escape_dart_single_quoted(&format_display_name(t_name));
            out.push_str(&format!("    batch.rawInsert('INSERT OR REPLACE INTO type_definitions (type_name, display_name, icon_symbol) VALUES (?, ?, ?)', ['{esc_type}', '{esc_display}', 'layers']);\n"));
        }
    }
    for (part, child) in parts.iter().flat_map(|p| p.parts.iter().map(move |c| (p, c))) {
        let esc_part = escape_dart_single_quoted(&part.name);
        let esc_child = escape_dart_single_quoted(&child.name);
        let esc_label = escape_dart_single_quoted(&format_display_name(&child.name));
        out.push_str(&format!("    batch.rawInsert('INSERT OR REPLACE INTO type_relations (parent_type, relation_name, child_type, display_label) VALUES (?, ?, ?, ?)', ['{esc_part}', 'contains', '{esc_child}', '{esc_label}']);\n"));
    }
}

/// Appends `type_attributes` metadata rows derived from SysML `PartDef` attributes and dynamic node properties.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and MIL-STD-1472H bounded UI inspector metadata synthesis controls.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves 0-based `section_order` monotonicity per type while deduplicating `(type_name, key)` pairs.
/// ## 3. Preconditions
/// - `parts` provides formal SysML attributes with optional `[min, max]` docs; `nodes` holds instance property bags.
/// ## 4. Postconditions
/// - Appends `batch.rawInsert` calls populating `type_attributes` bounds, defaults, labels, and field types.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O((P \cdot A + N \cdot K) \log(P \cdot A + N \cdot K))$ across schema and node attributes.
/// - Space Complexity: $O(P \cdot A + N \cdot K)$ for `seen` pairs and `next_order` index tracking.
/// ## 6. Failure Modes & Fault Tolerance
/// - Missing numeric bounds or omitted default values emit SQL `null` tokens without failing generation.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
fn append_seed_type_attributes(out: &mut String, parts: &[PartDef], nodes: &[TopologyNode]) {
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut next_order: BTreeMap<String, usize> = BTreeMap::new();
    for (part, order, attr) in parts.iter().flat_map(|p| p.attributes.iter().enumerate().map(move |(ord, a)| (p, ord, a))) {
        seen.insert((part.name.clone(), attr.name.clone()));
        next_order.insert(part.name.clone(), order.saturating_add(1));
        let field_type = map_sysml_attr_field_type(&attr.type_name);
        let (min_opt, max_opt) = extract_bounds_from_doc(attr.doc.as_deref());
        let min_str = min_opt.map_or("null".to_string(), |v| format!("{v:.1}"));
        let max_str = max_opt.map_or("null".to_string(), |v| format!("{v:.1}"));
        let default_str = attr.default_value.as_deref().map_or("null".to_string(), |raw| format!("'{}'", escape_dart_single_quoted(strip_surrounding_quotes(raw))));
        let (esc_t, esc_k, esc_l) = (escape_dart_single_quoted(&part.name), escape_dart_single_quoted(&attr.name), escape_dart_single_quoted(&format_display_name(&attr.name)));
        out.push_str(&format!("    batch.rawInsert('INSERT OR REPLACE INTO type_attributes (type_name, key, label, field_type, section_label, section_order, is_required, min_value, max_value, default_value) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)', ['{esc_t}', '{esc_k}', '{esc_l}', '{field_type}', 'General', {order}, 1, {min_str}, {max_str}, {default_str}]);\n"));
    }
    let node_props = nodes.iter().map(|n| (n.type_name.trim(), &n.properties)).filter(|(t, _)| !t.is_empty())
        .flat_map(|(t, props)| props.iter().map(move |(k, v)| (t, k.trim(), v))).filter(|(_, key, _)| !key.is_empty());
    for (t_name, key, val) in node_props {
        if seen.insert((t_name.to_string(), key.to_string())) {
            let order = next_order.get(t_name).copied().unwrap_or(0);
            next_order.insert(t_name.to_string(), order.saturating_add(1));
            let (field_type, default_str) = format_json_prop_type_and_default(val);
            let (esc_t, esc_k, esc_l) = (escape_dart_single_quoted(t_name), escape_dart_single_quoted(key), escape_dart_single_quoted(&format_display_name(key)));
            out.push_str(&format!("    batch.rawInsert('INSERT OR REPLACE INTO type_attributes (type_name, key, label, field_type, section_label, section_order, is_required, min_value, max_value, default_value) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)', ['{esc_t}', '{esc_k}', '{esc_l}', '{field_type}', 'General', {order}, 1, null, null, {default_str}]);\n"));
        }
    }
}

/// Appends merged instance and `PartDef`-default EAV `properties` rows for a single `TopologyNode`.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A deterministic EAV instance hydration and schema default inheritance.
/// ## 2. Invariants & Mathematical Properties
/// - Explicit node properties override `PartDef` attribute defaults; keys emit in sorted `BTreeMap` order.
/// ## 3. Preconditions
/// - `parts` holds parsed `PartDef` items and `node` designates the target topology instance record.
/// ## 4. Postconditions
/// - Appends `batch.rawInsert` statements into `properties` for every numeric, boolean, or string entry.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O((A + K) \log(A + K))$ where $A$ is matching part attributes and $K$ is node properties.
/// - Space Complexity: $O(A + K)$ for the cloned and default-augmented `BTreeMap`.
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-scalar JSON values (arrays, objects, null) are skipped cleanly without emitting malformed SQL.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
fn append_node_eav_properties(out: &mut String, parts: &[PartDef], node: &TopologyNode) {
    let node_id_esc = escape_dart_single_quoted(&node.id);
    let mut merged = node.properties.clone();
    let defaults = parts.iter().find(|p| p.name == node.type_name).into_iter().flat_map(|part| &part.attributes)
        .filter_map(|a| a.default_value.as_deref().map(|d| (&a.name, &a.type_name, d)));
    for (name, t_name, raw_def) in defaults {
        merged.entry(name.clone()).or_insert(parse_attr_default_json(t_name, raw_def));
    }
    for (k, v) in &merged {
        let key_esc = escape_dart_single_quoted(k);
        let num_tail = v.as_f64().map(|num| format!("null, {num}, 'double'"));
        let bool_tail = v.as_bool().map(|b| format!("'{b}', {}, 'bool'", u8::from(b)));
        let str_tail = v.as_str().map(|s| format!("'{}', null, 'text'", escape_dart_single_quoted(s)));
        if let Some(tail) = num_tail.or(bool_tail).or(str_tail) {
            out.push_str(&format!("    batch.rawInsert('INSERT OR REPLACE INTO properties (instance_id, key, value_text, value_num, value_type) VALUES (?, ?, ?, ?, ?)', ['{node_id_esc}', '{key_esc}', {tail}]);\n"));
        }
    }
}

/// Synthesizes the complete `seedGeneratedDomainDatabase` Dart SQLite seed source for a `SysmlModel`.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A transactional database seeding and cryptographic schema hash stamping.
/// ## 2. Invariants & Mathematical Properties
/// - Wraps DDL execution and batched metamodel/instance inserts inside one atomic `db.transaction` block.
/// ## 3. Preconditions
/// - `model` references a parsed `SysmlModel`, `nodes` holds topology entities, and `schema_hash` is computed.
/// ## 4. Postconditions
/// - Returns complete `.g.dart` source exporting `kDeapSchemaHash` aliases and `seedGeneratedDomainDatabase`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(P \cdot A + N \cdot (P + K \log K))$ over $P$ parts and $N$ topology instances.
/// - Space Complexity: $O(P \cdot A + N \cdot K)$ proportional to the synthesized Dart source string.
/// ## 6. Failure Modes & Fault Tolerance
/// - Empty models and empty node slices still produce valid, compilable transactional Dart DDL seed code.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
pub fn generate_domain_seed_dart(model: &SysmlModel, nodes: &[TopologyNode], schema_hash: u64) -> String {
    let mut out = format!(
        "// GENERATED CODE - DO NOT MODIFY BY HAND\nimport 'package:sqflite_common_ffi/sqflite_ffi.dart';\n\nconst int kDeapSchemaHash = 0x{schema_hash:016X};\nconst int deapSchemaHash = kDeapSchemaHash;\nconst int DEAP_SCHEMA_HASH = kDeapSchemaHash;\n\nFuture<void> seedGeneratedDomainDatabase(Database db) async {{\n  await db.transaction((txn) async {{\n"
    );
    append_seed_ddl(&mut out);
    out.push_str("    final batch = txn.batch();\n\n");
    append_seed_types_and_relations(&mut out, &model.parts, nodes);
    append_seed_type_attributes(&mut out, &model.parts, nodes);
    for node in nodes {
        let (id_esc, type_esc, label_esc) = (escape_dart_single_quoted(&node.id), escape_dart_single_quoted(node.type_name.trim()), escape_dart_single_quoted(&node.label));
        let parent_str = node.parent_id.as_deref().filter(|p| !p.is_empty()).map_or("null".to_string(), |p| format!("'{}'", escape_dart_single_quoted(p)));
        out.push_str(&format!("    batch.rawInsert('INSERT OR REPLACE INTO instances (id, type_name, label, parent_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)', ['{id_esc}', '{type_esc}', '{label_esc}', {parent_str}, {}, {}]);\n", node.created_at, node.updated_at));
        append_node_eav_properties(&mut out, &model.parts, node);
    }
    out.push_str("    await batch.commit(noResult: true);\n  });\n}\n");
    out
}

/// Emits modular per-package seed `.g.dart` units, shared canonical DDL, and the re-exporting barrel file.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A modular artifact partitioning for multi-package SysML workspace architectures.
/// ## 2. Invariants & Mathematical Properties
/// - Deduplicates `snake_case` package slugs via `BTreeSet` to prevent output filename collisions.
/// ## 3. Preconditions
/// - `pkg` references the root `PackageDef` hierarchy, `nodes` holds topology entities, and `schema_hash` is valid.
/// ## 4. Postconditions
/// - Returns `(relative_path, dart_source)` pairs for `ddl_schema.g.dart`, `<slug>_seed.g.dart`, and the barrel.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(S \cdot (P_s \cdot A + N \cdot K \log K))$ across $S$ discovered non-empty package slices.
/// - Space Complexity: $O(S \cdot (P_s \cdot A + N \cdot K))$ for all emitted file path and content buffers.
/// ## 6. Failure Modes & Fault Tolerance
/// - Duplicate package stems are skipped deterministically after first emission in pre-order traversal.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607], [REQ-FLUTTER-GENERATE-DOMAIN-SEED]
pub fn emit_modular_seed_files(pkg: &PackageDef, nodes: &[TopologyNode], schema_hash: u64) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let mut ddl = format!("// GENERATED CODE - DO NOT MODIFY BY HAND\nimport 'package:sqflite_common_ffi/sqflite_ffi.dart';\nconst int kDeapSchemaHash = 0x{schema_hash:016X};\nFuture<void> applyCanonicalDeapDdl(DatabaseExecutor txn) async {{\n");
    append_seed_ddl(&mut ddl);
    ddl.push_str("}\n");
    files.push(("lib/src/data/seeds/generated/ddl_schema.g.dart".to_string(), ddl));
    let mut barrel = String::from("// GENERATED CODE - DO NOT MODIFY BY HAND\nexport 'generated/ddl_schema.g.dart';\n");
    let mut seen_slugs = BTreeSet::new();
    for (slug, sub_pkg) in collect_package_slices(pkg) {
        if seen_slugs.insert(slug.clone()) {
            let sub_model = SysmlModel { package_name: Some(sub_pkg.name.clone()), parts: sub_pkg.part_defs.clone(), ..SysmlModel::default() };
            let fn_name = format!("seed{}PackageDatabase", to_pascal_case(&slug));
            let content = generate_domain_seed_dart(&sub_model, nodes, schema_hash).replace("seedGeneratedDomainDatabase", &fn_name);
            files.push((format!("lib/src/data/seeds/generated/{slug}_seed.g.dart"), content));
            barrel.push_str(&format!("export 'generated/{slug}_seed.g.dart';\n"));
        }
    }
    files.push(("lib/src/data/seeds/generated_domain_seed.g.dart".to_string(), barrel));
    files
}
