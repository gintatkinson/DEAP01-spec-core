//! JSON and YAML subsystem/component data extractors.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-MODULE], [DO-330-TQL-4-SYSML-AST]
//! Contract: Provides `extract_parts_from_json` and `extract_parts_from_yaml_lines` for structured schema part extraction.
//! Bounds & Faults: O(N) traversal over JSON nodes and YAML lines with zero panics.

use super::identifiers::is_valid_subsystem_identifier;
use std::collections::BTreeSet;

/// Recursively walks a parsed JSON `Value` tree and collects valid subsystem identifiers.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-PARTS-JSON]
pub fn extract_parts_from_json(val: &serde_json::Value, parts: &mut BTreeSet<String>) {
    match val {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::Array(nodes)) = map.get("schema_nodes") {
                for node in nodes {
                    if let serde_json::Value::String(s) = node {
                        if let Some((kind, name)) = s.split_once(':') {
                            if kind == "part" || kind == "component" || kind == "subsystem" {
                                let n = name.trim();
                                if is_valid_subsystem_identifier(n) {
                                    parts.insert(n.to_string());
                                }
                            }
                        }
                    }
                }
            }
            for key in &["subsystems", "components", "parts", "modules"] {
                if let Some(v) = map.get(*key) {
                    match v {
                        serde_json::Value::Array(arr) => {
                            for item in arr {
                                match item {
                                    serde_json::Value::String(s) => {
                                        let s_clean = s.trim();
                                        if is_valid_subsystem_identifier(s_clean) {
                                            parts.insert(s_clean.to_string());
                                        }
                                    }
                                    serde_json::Value::Object(submap) => {
                                        for name_key in &["name", "id", "identifier", "title"] {
                                            if let Some(serde_json::Value::String(n)) =
                                                submap.get(*name_key)
                                            {
                                                let n_clean = n.trim();
                                                if is_valid_subsystem_identifier(n_clean) {
                                                    parts.insert(n_clean.to_string());
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        serde_json::Value::Object(submap) => {
                            for k in submap.keys() {
                                let k_clean = k.trim();
                                if is_valid_subsystem_identifier(k_clean) {
                                    parts.insert(k_clean.to_string());
                                }
                            }
                        }
                        serde_json::Value::String(s) => {
                            let s_clean = s.trim();
                            if is_valid_subsystem_identifier(s_clean) {
                                parts.insert(s_clean.to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
            for (_, subval) in map {
                extract_parts_from_json(subval, parts);
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                extract_parts_from_json(item, parts);
            }
        }
        _ => {}
    }
}

/// Scans YAML or JSON lines for subsystem and component list entries without an external YAML crate.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-PARTS-YAML]
pub fn extract_parts_from_yaml_lines(content: &str, parts: &mut BTreeSet<String>) {
    let mut in_parts_list = false;
    let mut list_indent = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let current_indent = line.len() - line.trim_start().len();

        let is_key = [
            "subsystems:",
            "components:",
            "parts:",
            "modules:",
            "subsystem:",
            "component:",
            "part:",
            "module:",
        ]
        .iter()
        .any(|kw| trimmed.to_ascii_lowercase().starts_with(kw));

        if is_key {
            in_parts_list = true;
            list_indent = current_indent;
            if let Some((_, val)) = trimmed.split_once(':') {
                let val_clean = val.trim().trim_matches(['"', '\'', '[', ']']);
                if !val_clean.is_empty() {
                    for item in val_clean.split(',') {
                        let item_trim = item.trim().trim_matches(['"', '\'']);
                        if is_valid_subsystem_identifier(item_trim) {
                            parts.insert(item_trim.to_string());
                        }
                    }
                }
            }
            continue;
        }

        if in_parts_list {
            if current_indent <= list_indent && !trimmed.starts_with('-') {
                in_parts_list = false;
            } else if let Some(rest) = trimmed.strip_prefix('-') {
                let item_clean = rest.trim();
                let name = if let Some((_, v)) = item_clean.split_once(':') {
                    v.trim().trim_matches(['"', '\''])
                } else {
                    item_clean.trim_matches(['"', '\''])
                };
                let ident: String = name
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                if is_valid_subsystem_identifier(&ident) {
                    parts.insert(ident);
                }
            }
        }
    }
}
