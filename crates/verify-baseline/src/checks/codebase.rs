//! Baseline verification checks 38 through 41 for presentation anti-hardcoding, zero mocking, UI contracts, and solver bridge parity.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and MIL-STD-1472H standards.
//! Enforces deterministic static AST verification preventing hardcoded presentation strings,
//! mock/fake test bypasses, unverified viewport contracts, and duplicate solver logic across Flutter and Rust tiers.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Pure Schema & Topology Grounding: Extracts identifiers dynamically at runtime from schema and topology files.
//! - Zero Regex Usage: Enforces deterministic linear character and token traversal exclusively.
//! - Bijective Persistence & SSOT: Validates 1:1 entity parity in SQLite persistence seeds and thin C-ABI delegation.
//!
//! ## 3. Preconditions
//! - `repo_root` is an accessible filesystem path pointing to the root of the workspace.
//!
//! ## 4. Postconditions
//! - Returns `Ok(())` when all checked invariants hold.
//! - Returns `Err(Vec<String>)` containing formatted diagnostic violation messages upon any defect.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(F * L) where F is the file count and L is the average line length.
//! - Space Complexity: O(E + V) where E is the entity identifier count and V is the violation diagnostic count.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Gracefully returns `Ok(())` when optional directories (such as `app_flutter`) are absent.
//! - Collects all detected violations per gate rather than failing on the first violation.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-VERIFY-BASELINE-CODEBASE-CHECKS], [DO-330-TQL-4-CHECKS-CODEBASE]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use super::codebase_gesture_gate::{
    extract_uninterpolated_string_literals, is_hardcoded_telemetry_or_designator_literal,
};
pub use super::codebase_gesture_gate::check_40_interactive_gesture_and_hud_non_overlap_gate;
pub use super::codebase_mock_gate::{check_39_zero_mocking_test_ast_gate, strip_dart_comments};
pub use super::codebase_solver_gate::{
    check_41_solver_bridge_and_sqlite_seed_parity_gate, check_no_duplicate_dart_solver_math,
};

/// Check 38: Presentation Anti-Hardcoding AST Gate.
/// Realises: [REQ-CHECK-38-PRESENTATION-ANTI-HARDCODING]
pub fn check_38_presentation_anti_hardcoding_ast_gate(repo_root: &Path) -> Result<(), Vec<String>> {
    let features_dir = repo_root.join("app_flutter").join("lib").join("features");
    if !features_dir.is_dir() {
        return Ok(());
    }

    let mut violations = Vec::new();
    let entity_ids = extract_entity_identifiers_from_topology_and_schema(repo_root);

    scan_dart_features_for_hardcoding(repo_root, &entity_ids, &mut violations);

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        Err(violations)
    }
}

/// Dynamically extracts entity identifiers and names from topology JSON and SysML schema files.
/// Realises: [REQ-CHECK-38-EXTRACT-TOPOLOGY-IDENTIFIERS]
fn extract_entity_identifiers_from_topology_and_schema(repo_root: &Path) -> BTreeSet<String> {
    let mut entity_ids = BTreeSet::new();

    let topology_path = repo_root
        .join("app_flutter")
        .join("assets")
        .join("topology_data.json");
    if let Ok(content) = fs::read_to_string(&topology_path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            extract_strings_from_json_value(&val, &mut entity_ids);
        }
    }

    let schema_paths = [
        repo_root.join(".pipeline").join("schema.sysml"),
        repo_root.join("schema").join("schema.sysml"),
    ];

    for sp in &schema_paths {
        if let Ok(content) = fs::read_to_string(sp) {
            for line in content.lines() {
                let trimmed = line.trim();
                let decl_prefixes = [
                    "part def ",
                    "item def ",
                    "port def ",
                    "action def ",
                    "calc def ",
                    "attribute def ",
                    "state def ",
                    "part ",
                    "item ",
                    "port ",
                ];
                for prefix in &decl_prefixes {
                    if let Some(rest) = trimmed.strip_prefix(prefix) {
                        let ident_token: String = rest
                            .chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                            .collect();
                        let clean = ident_token.trim();
                        if clean.len() >= 3 && !is_generic_schema_keyword(clean) {
                            entity_ids.insert(clean.to_string());
                        }
                    }
                }
            }
        }
    }

    entity_ids
}

/// Iteratively harvests identifier string values from a JSON AST using an explicit work-stack
/// without regex or unbounded call-stack recursion.
/// Realises: [REQ-CHECK-38-EXTRACT-JSON-VALUES]
pub(crate) fn extract_strings_from_json_value(
    val: &serde_json::Value,
    entities: &mut BTreeSet<String>,
) {
    let mut stack: Vec<&serde_json::Value> = vec![val];
    while let Some(current) = stack.pop() {
        match current {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    let lower_k = k.to_lowercase();
                    let is_entity_key = matches!(
                        lower_k.as_str(),
                        "id" | "name"
                            | "label"
                            | "callsign"
                            | "title"
                            | "parentid"
                            | "parent_id"
                            | "source"
                            | "target"
                            | "sourceid"
                            | "targetid"
                    );
                    if is_entity_key {
                        if let Some(s) = v.as_str() {
                            insert_entity_and_stems(s, entities);
                        }
                    } else if lower_k == "properties" {
                        if let serde_json::Value::Object(prop_map) = v {
                            for prop_val in prop_map.values() {
                                if let Some(ps) = prop_val.as_str() {
                                    if ps.contains('-') || ps.contains('_') {
                                        insert_entity_and_stems(ps, entities);
                                    }
                                }
                            }
                        }
                    }
                }
                for child in map.values().rev() {
                    stack.push(child);
                }
            }
            serde_json::Value::Array(arr) => {
                for elem in arr.iter().rev() {
                    stack.push(elem);
                }
            }
            _ => {}
        }
    }
}

/// Inserts an entity identifier and its parenthetical/hyphenated base stems into `entities`.
/// Realises: [REQ-CHECK-38-EXTRACT-JSON-VALUES]
fn insert_entity_and_stems(raw: &str, entities: &mut BTreeSet<String>) {
    let trimmed = raw.trim();
    if trimmed.len() >= 3 && !is_generic_schema_keyword(trimmed) {
        entities.insert(trimmed.to_string());
    }
    let base = if let Some((before_paren, _)) = trimmed.split_once('(') {
        let stem = before_paren.trim();
        if stem.len() >= 3 && !is_generic_schema_keyword(stem) {
            entities.insert(stem.to_string());
        }
        stem
    } else {
        trimmed
    };
    if let Some((_, sub_stem)) = base.split_once('-') {
        let sub_clean = sub_stem.trim();
        if sub_clean.contains('-') && sub_clean.len() >= 4 && !is_generic_schema_keyword(sub_clean)
        {
            entities.insert(sub_clean.to_string());
        }
    }
}

/// Determines whether an identifier string is a generic language or schema keyword.
/// Realises: [REQ-CHECK-38-IS-GENERIC-SCHEMA-KEYWORD]
fn is_generic_schema_keyword(ident: &str) -> bool {
    let lower = ident.to_lowercase();
    matches!(
        lower.as_str(),
        "int"
            | "integer"
            | "bool"
            | "boolean"
            | "real"
            | "string"
            | "true"
            | "false"
            | "null"
            | "node"
            | "link"
            | "edge"
            | "type"
            | "item"
            | "part"
            | "port"
            | "def"
            | "action"
            | "calc"
            | "state"
            | "name"
    )
}

/// Scans Dart source files under `app_flutter/lib/features` for presentation hardcoding violations.
/// Realises: [REQ-CHECK-38-SCAN-DART-FEATURES]
fn scan_dart_features_for_hardcoding(
    repo_root: &Path,
    entity_ids: &BTreeSet<String>,
    violations: &mut Vec<String>,
) {
    let features_dir = repo_root.join("app_flutter").join("lib").join("features");
    for entry_res in WalkDir::new(&features_dir) {
        let entry = match entry_res {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.file_type().is_file() {
            let path = entry.path();
            if path.extension().map(|ext| ext == "dart").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(path) {
                    let rel = path
                        .strip_prefix(repo_root)
                        .unwrap_or(path)
                        .display()
                        .to_string();
                    let stripped_content = strip_dart_comments(&content);
                    for (idx, line) in stripped_content.lines().enumerate() {
                        let line_no = idx.saturating_add(1);
                        check_presentation_line(line, &rel, line_no, entity_ids, violations);
                    }
                }
            }
        }
    }
}

/// Checks static boolean flags (`isActive: true/false`, `isSelected: true/false`) on a Dart line.
/// Realises: [REQ-CHECK-38-CHECK-PRESENTATION-LINE]
fn check_static_boolean_flags(
    stripped: &str,
    file_rel: &str,
    line_no: usize,
    violations: &mut Vec<String>,
) {
    for flag_name in ["isActive", "isSelected"] {
        let key_colon = format!("{}:", flag_name);
        let mut search_from = 0usize;
        while let Some(rel_idx) = stripped.get(search_from..).and_then(|s| s.find(&key_colon)) {
            let after_colon_idx = search_from
                .saturating_add(rel_idx)
                .saturating_add(key_colon.len());
            let after_colon = stripped.get(after_colon_idx..).unwrap_or("").trim_start();
            for bool_lit in ["true", "false"] {
                if let Some(rest) = after_colon.strip_prefix(bool_lit) {
                    let boundary_ok = rest
                        .chars()
                        .next()
                        .map(|c| !c.is_ascii_alphanumeric() && c != '_')
                        .unwrap_or(true);
                    if boundary_ok {
                        violations.push(format!(
                            "Check 38 violation: Hardcoded static boolean flag '{}: {}' found in {}:{}. Row states must bind dynamically.",
                            flag_name, bool_lit, file_rel, line_no
                        ));
                    }
                }
            }
            search_from = after_colon_idx;
        }
    }
}

/// Checks an individual Dart source line for hardcoded presentation strings or static boolean flags.
/// Realises: [REQ-CHECK-38-CHECK-PRESENTATION-LINE]
fn check_presentation_line(
    line: &str,
    file_rel: &str,
    line_no: usize,
    entity_ids: &BTreeSet<String>,
    violations: &mut Vec<String>,
) {
    let stripped = line.split_once("//").map_or(line.trim(), |(c, _)| c.trim());
    if stripped.is_empty()
        || stripped.starts_with("import ")
        || stripped.starts_with("export ")
        || stripped.starts_with("part ")
        || stripped.starts_with("library ")
    {
        return;
    }

    check_static_boolean_flags(stripped, file_rel, line_no, violations);

    for (start_idx, inner_raw) in extract_uninterpolated_string_literals(stripped) {
        if stripped
            .get(..start_idx)
            .unwrap_or("")
            .trim_end()
            .ends_with("Key(")
        {
            continue;
        }
        let inner = inner_raw.trim();
        if inner.is_empty() {
            continue;
        }

        for entity in entity_ids {
            if inner == entity.as_str() || inner.contains(entity.as_str()) {
                violations.push(format!(
                    "Check 38 violation: Hardcoded un-interpolated domain entity literal '{}' in Text widget in {}:{}.",
                    entity, file_rel, line_no
                ));
                break;
            }
        }

        if is_hardcoded_telemetry_or_designator_literal(inner) {
            violations.push(format!(
                "Check 38 violation: Hardcoded static telemetry indicator found in {}:{}: '{}'. Telemetry status must bind to live solver or monitor state.",
                file_rel, line_no, stripped
            ));
        }
    }
}
