//! Multi-format schema part extraction (YAML, JSON, Markdown `pulldown-cmark` AST, interface schemas) and JSON dictionary ingestion.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use deap_core::markdown_ast::parse_markdown_ast;
use serde_json::Value;

use super::sysml_extractor::extract_subsystem_parts_from_sysml;
use super::sysml_lexer::find_line_number;
use super::{SubsystemPart, SysMLParameterBindingEngine};

/// Builds `SubsystemPart` records with line-number provenance from extracted part names.
/// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-SUBSYSTEM-PARTS] | SysML v2: ActionDef(parts_from_names)
fn parts_from_names(
    content: &str,
    filepath: &str,
    part_names: BTreeSet<String>,
) -> Vec<SubsystemPart> {
    part_names
        .into_iter()
        .map(|name| {
            let line_num = content
                .lines()
                .enumerate()
                .find(|(_, l)| l.contains(&name))
                .map(|(idx, _)| idx + 1);
            SubsystemPart {
                name,
                doc: String::new(),
                ports: Vec::new(),
                mass_kg: None,
                power_w: None,
                actions: Vec::new(),
                attributes: BTreeMap::new(),
                constraints: Vec::new(),
                source_file: filepath.to_string(),
                source_line: line_num,
                epistemic_tier: "[TIER-1: OEM]".to_string(),
            }
        })
        .collect()
}

/// [DOC-TIER: A] Extracts subsystem parts from a YAML schema document.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-SUBSYSTEM-PARTS-FROM-YAML] | SysML v2: ActionDef(extract_subsystem_parts_from_yaml)
/// Contract: Delegates part name extraction to `UnifiedSchemaModelExtractor` for `.yaml` and attaches source provenance.
/// Bounds & Faults: O(N) time and space over `content.len()`; infallible extraction.
pub fn extract_subsystem_parts_from_yaml(content: &str, filepath: &str) -> Vec<SubsystemPart> {
    let names =
        deap_core::sysml_ast::UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(
            ".yaml", content,
        );
    parts_from_names(content, filepath, names)
}

/// [DOC-TIER: A] Extracts subsystem parts from a JSON schema document.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-SUBSYSTEM-PARTS-FROM-JSON] | SysML v2: ActionDef(extract_subsystem_parts_from_json)
/// Contract: Delegates part name extraction to `UnifiedSchemaModelExtractor` for `.json` and attaches source provenance.
/// Bounds & Faults: O(N) time and space over `content.len()`; infallible extraction.
pub fn extract_subsystem_parts_from_json(content: &str, filepath: &str) -> Vec<SubsystemPart> {
    let names =
        deap_core::sysml_ast::UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(
            ".json", content,
        );
    parts_from_names(content, filepath, names)
}

/// [DOC-TIER: A] Extracts subsystem parts from a Markdown specification using `deap_core::markdown_ast` (`pulldown-cmark` AST).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-SUBSYSTEM-PARTS-FROM-MARKDOWN] | SysML v2: ActionDef(extract_subsystem_parts_from_markdown)
/// Contract: Extracts subsystem part names and enriches descriptions/line offsets from `parse_markdown_ast(content)` tables.
/// Bounds & Faults: O(N) time and space over `content.len()`; attaches 1-indexed `source_line` and `[TIER-1: OEM]` tier.
pub fn extract_subsystem_parts_from_markdown(content: &str, filepath: &str) -> Vec<SubsystemPart> {
    let ast = parse_markdown_ast(content);
    let part_names =
        deap_core::sysml_ast::UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(
            ".md", content,
        );
    let mut parts = parts_from_names(content, filepath, part_names);
    for part in &mut parts {
        for table in &ast.tables {
            for row in &table.rows {
                if row.first().is_some_and(|c0| c0.contains(&part.name)) {
                    if let Some(desc) = row.get(1) {
                        if part.doc.is_empty() && !desc.trim().is_empty() {
                            part.doc = desc.trim().to_string();
                        }
                    }
                    if part.source_line.is_none() {
                        part.source_line = find_line_number(content, table.start_offset);
                    }
                }
            }
        }
    }
    parts
}

/// [DOC-TIER: A] Extracts subsystem parts from an interface schema file (`.proto`, `.idl`, `.arxml`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-SUBSYSTEM-PARTS-FROM-INTERFACE-SCHEMA] | SysML v2: ActionDef(extract_subsystem_parts_from_interface_schema)
/// Contract: Dispatches by file extension to `UnifiedSchemaModelExtractor` and attaches line-number provenance.
/// Bounds & Faults: O(N) time and space over `content.len()`; infallible extraction.
pub fn extract_subsystem_parts_from_interface_schema(
    content: &str,
    filepath: &str,
) -> Vec<SubsystemPart> {
    let ext = Path::new(filepath)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let dot_ext = format!(".{ext}");
    let names =
        deap_core::sysml_ast::UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(
            &dot_ext, content,
        );
    parts_from_names(content, filepath, names)
}

/// [DOC-TIER: A] Extracts subsystem parts from a schema file or directory tree.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-EXTRACT-SUBSYSTEM-PARTS] | SysML v2: ActionDef(extract_subsystem_parts)
/// Contract: Inspects `target: &Path` (single file or directory of schema files) and returns all discovered `Vec<SubsystemPart>`.
/// Bounds & Faults: O(F * N) time and space over discovered schema files; ignores unreadable files gracefully.
pub fn extract_subsystem_parts(target: &Path) -> Vec<SubsystemPart> {
    let mut parts = Vec::new();
    if target.is_file() {
        if let Ok(content) = fs::read_to_string(target) {
            let fpath = target.to_string_lossy().to_string();
            let ext = target
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            match ext.as_str() {
                "sysml" => parts.extend(extract_subsystem_parts_from_sysml(&content, &fpath)),
                "yaml" | "yml" => parts.extend(extract_subsystem_parts_from_yaml(&content, &fpath)),
                "json" => parts.extend(extract_subsystem_parts_from_json(&content, &fpath)),
                "md" | "markdown" => {
                    parts.extend(extract_subsystem_parts_from_markdown(&content, &fpath))
                }
                _ => parts.extend(extract_subsystem_parts_from_interface_schema(
                    &content, &fpath,
                )),
            }
        }
    } else if target.is_dir() {
        let schema_files =
            deap_core::sysml_ast::UnifiedSchemaModelExtractor::discover_schema_files(&[target]);
        for sf in schema_files {
            parts.extend(extract_subsystem_parts(&sf));
        }
    }
    parts
}

impl SysMLParameterBindingEngine {
    /// Binds a single JSON key-value pair into `parameter_bindings` and maps semantic aliases (`[REQ-ASSEMBLE-CONOPS]`).
    fn ingest_json_entry(&mut self, k: &str, v: &Value) {
        self.explicit_keys.insert(k.to_uppercase());
        match v {
            Value::String(s) => {
                self.parameter_bindings.insert(k.to_string(), s.clone());
                self.parameter_bindings.insert(k.to_uppercase(), s.clone());
                self._map_semantic_aliases(k, s);
            }
            Value::Number(n) => {
                let str_val = n.to_string();
                self.parameter_bindings.insert(k.to_string(), str_val.clone());
                self.parameter_bindings.insert(k.to_uppercase(), str_val.clone());
                self._map_semantic_aliases(k, &str_val);
            }
            Value::Bool(b) => {
                let str_val = b.to_string();
                self.parameter_bindings.insert(k.to_string(), str_val.clone());
                self.parameter_bindings.insert(k.to_uppercase(), str_val.clone());
                self._map_semantic_aliases(k, &str_val);
            }
            Value::Array(arr) => {
                let lower_k = k.to_lowercase();
                if [
                    "core_mission_capabilities",
                    "core_capabilities",
                    "capabilities",
                    "mission_capabilities",
                ]
                .contains(&lower_k.as_str())
                {
                    let mut lines = Vec::new();
                    for (idx, item) in arr.iter().enumerate() {
                        let item_str = match item {
                            Value::String(s) => s.trim().to_string(),
                            other => other.to_string(),
                        };
                        if item_str.starts_with(&format!("{}.", idx + 1)) || item_str.starts_with('-') {
                            lines.push(format!("  {item_str}"));
                        } else {
                            lines.push(format!("  {}. {item_str}", idx + 1));
                        }
                    }
                    let formatted = lines.join("\n");
                    self.parameter_bindings.insert(k.to_string(), formatted.clone());
                    self.parameter_bindings.insert(k.to_uppercase(), formatted.clone());
                    self.parameter_bindings.insert("CORE_MISSION_CAPABILITIES".to_string(), formatted);
                }
            }
            _ => {}
        }
    }

    /// Extracts package/system identifier and AST part names from `schema_nodes` (`[REQ-ASSEMBLE-CONOPS]`).
    fn ingest_schema_nodes(&mut self, nodes: &[Value]) {
        for node in nodes {
            if let Value::String(s) = node {
                if let Some((kind, name)) = s.split_once(':') {
                    if (kind == "package" || kind == "system")
                        && self.inferred_system_identifier.is_none()
                    {
                        self.inferred_system_identifier = Some(name.trim().to_string());
                    }
                    if kind == "part" {
                        self.ast_part_names.insert(name.trim().to_string());
                    }
                }
            }
        }
    }

    /// Recursively ingests nested parameter dictionaries and `schema_nodes` from a JSON `Value`.
    /// Inputs: `data: &Value` | Outputs: `()` | Invariants: maps semantic aliases for all scalar entries
    ///Errors: ignores non-object JSON values | Complexity: O(N) over JSON tree nodes
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-DICTIONARY] | SysML v2: ActionDef(SysMLParameterBindingEngine::ingest_dictionary)
    pub fn ingest_dictionary(&mut self, data: &Value) {
        if let Value::Object(map) = data {
            let containers = [
                "parameters", "domain_parameters", "domain_params",
                "specs", "attributes", "metadata",
            ];
            for container_key in &containers {
                if let Some(sub) = map.get(*container_key) {
                    self.ingest_dictionary(sub);
                }
            }

            for name_key in &[
                "system_identifier", "system_name", "system",
                "name", "SYSTEM_IDENTIFIER", "MISSION_SYSTEM_NAME",
            ] {
                if let Some(Value::String(s)) = map.get(*name_key) {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        self.inferred_system_identifier = Some(trimmed.to_string());
                    }
                }
            }

            for (k, v) in map {
                if !containers.contains(&k.as_str()) && k != "schema_nodes" {
                    self.ingest_json_entry(k, v);
                }
            }

            if let Some(Value::Array(nodes)) = map.get("schema_nodes") {
                self.ingest_schema_nodes(nodes);
            }
        }
    }
}
