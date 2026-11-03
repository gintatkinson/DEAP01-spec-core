//! Workspace discovery and multi-format schema/Markdown AST ingestion for `SysMLParameterBindingEngine` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).

use std::fs;
use std::path::{Path, PathBuf};

use deap_core::markdown_ast::parse_markdown_ast;
use serde_json::Value;

use super::cmark_engine::{SubsystemPart, SysMLParameterBindingEngine};
use super::cmark_schema::{
    extract_subsystem_parts_from_interface_schema, extract_subsystem_parts_from_json,
    extract_subsystem_parts_from_markdown, extract_subsystem_parts_from_sysml,
    extract_subsystem_parts_from_yaml, find_all_attributes, find_first_package,
};

impl SysMLParameterBindingEngine {
    /// Registers extracted `SubsystemPart` items into `ast_part_names`, `explicit_keys`, and `ast_parts` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
    fn register_extracted_parts(&mut self, parts: Vec<SubsystemPart>) {
        for p in parts {
            self.ast_part_names.insert(p.name.clone());
            self.explicit_keys.insert(p.name.clone());
            self.explicit_keys.insert(p.name.to_uppercase());
            if !self.ast_parts.iter().any(|existing| existing.name == p.name) {
                self.ast_parts.push(p);
            }
        }
    }

    /// [DOC-TIER: A] Discovers parameter and schema sources across ancestor directories up to 5 levels.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-AUTO-DISCOVER-SOURCES] [FLS: fls_conops_cmark]
    /// Contract: Collects up to 5 ancestor directories starting at `root_dir` and delegates to `auto_detect_workspace_parameters`.
    /// Bounds & Faults: O(D) over directory depth; infallible.
    pub fn auto_discover_sources(&mut self, root_dir: &Path) {
        let mut search_dirs = Vec::new();
        let mut curr = root_dir.to_path_buf();
        for _ in 0..5 {
            search_dirs.push(curr.clone());
            if let Some(parent) = curr.parent().filter(|&p| p != curr) {
                curr = parent.to_path_buf();
            } else {
                break;
            }
        }
        self.auto_detect_workspace_parameters(&search_dirs);
    }

    /// [DOC-TIER: A] Scans candidate workspace directories for `.sysml`, `.json`, `.yaml`, `.md`, and interface schemas.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-AUTO-DETECT-WORKSPACE-PARAMETERS] [FLS: fls_conops_cmark]
    /// Contract: Respects `.pipeline/upstream` firewall, ingests discovered schema files, and re-derives ontology and physics bindings.
    /// Bounds & Faults: O(F) over workspace schema files; skips unreadable entries without panicking.
    pub fn auto_detect_workspace_parameters(&mut self, search_dirs: &[PathBuf]) {
        let supported_exts = deap_core::UnifiedSchemaModelExtractor::SUPPORTED_EXTENSIONS;
        for sdir in search_dirs {
            if sdir.join(".pipeline").join("upstream").is_dir() {
                continue;
            }
            for f in ["schema.sysml", "schema-digest.json"] {
                let candidate = sdir.join(".pipeline").join(f);
                if candidate.is_file() {
                    self.ingest_file(&candidate);
                }
            }
            for rel_dir in &["schema", "docs/architecture", "docs/research"] {
                let cdir = sdir.join(rel_dir);
                if cdir.is_dir() {
                    for entry_res in walkdir::WalkDir::new(&cdir).sort_by_file_name() {
                        let Ok(entry) = entry_res else { continue };
                        let path = entry.path();
                        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if path.is_file() && !fname.starts_with('.') {
                            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                let dot_ext = format!(".{}", ext.to_lowercase());
                                if supported_exts.contains(&dot_ext.as_str()) {
                                    self.ingest_file(path);
                                }
                            }
                        }
                    }
                }
            }
        }
        self._derive_domain_ontology();
        self._derive_operational_intent();
        self._derive_domain_risk_assessment();
        self._derive_lifecycle_contract();
        self._derive_quadratic_physics();
        self._derive_subsystem_architecture();
        self.derive_master_physical_limits_from_ast();
    }

    /// [DOC-TIER: A] Dispatches file ingestion by file extension and updates derived architecture bindings.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-FILE] [FLS: fls_conops_cmark]
    /// Contract: Routes `path` by extension to format-specific ingestion and refreshes architecture/physical limit derivations.
    /// Bounds & Faults: O(N) over file size; returns `false` on unsupported or missing file.
    pub fn ingest_file(&mut self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let ok = match ext.as_str() {
            "json" => self.ingest_json_file(path),
            "yaml" | "yml" => self.ingest_yaml_file(path),
            "sysml" => self.ingest_sysml_file(path),
            "md" | "markdown" => self.ingest_markdown_file(path),
            "proto" | "idl" | "arxml" => self.ingest_interface_schema_file(path),
            _ => false,
        };
        if ok {
            self._derive_subsystem_architecture();
            self.derive_master_physical_limits_from_ast();
        }
        ok
    }

    /// [DOC-TIER: A] Ingests subsystem parts and scalar key-value bindings from a YAML file.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-YAML-FILE] [FLS: fls_conops_cmark]
    /// Contract: Extracts YAML parts and top-level `key: value` scalar bindings into `parameter_bindings`.
    /// Bounds & Faults: O(N) over YAML lines; returns `false` if file cannot be read.
    pub fn ingest_yaml_file(&mut self, path: &Path) -> bool {
        let Ok(content) = fs::read_to_string(path) else { return false };
        let fpath = path.to_string_lossy().to_string();
        self.register_extracted_parts(extract_subsystem_parts_from_yaml(&content, &fpath));
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') {
                continue;
            }
            if let Some((k, v)) = trimmed.split_once(':') {
                let k_clean = k.trim().replace(['"', '\'', '`'], "");
                let v_clean = v.trim().trim_matches(['"', '\'', '`', ';']).trim().to_string();
                if !k_clean.is_empty() && !v_clean.is_empty() && !v_clean.starts_with(['[', '{']) {
                    self.parameter_bindings.insert(k_clean.clone(), v_clean.clone());
                    self.parameter_bindings.insert(k_clean.to_uppercase(), v_clean.clone());
                    self._map_semantic_aliases(&k_clean, &v_clean);
                }
            }
        }
        true
    }

    /// [DOC-TIER: A] Ingests subsystem parts from `.proto`, `.idl`, or `.arxml` interface schema files.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-INTERFACE-SCHEMA-FILE] [FLS: fls_conops_cmark]
    /// Contract: Extracts interface schema parts and registers their names in `ast_part_names` and `ast_parts`.
    /// Bounds & Faults: O(N) over file size; returns `false` if file cannot be read.
    pub fn ingest_interface_schema_file(&mut self, path: &Path) -> bool {
        let Ok(content) = fs::read_to_string(path) else { return false };
        let fpath = path.to_string_lossy().to_string();
        self.register_extracted_parts(extract_subsystem_parts_from_interface_schema(&content, &fpath));
        true
    }

    /// [DOC-TIER: A] Ingests subsystem parts and JSON object parameters from a `.json` file.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-JSON-FILE] [FLS: fls_conops_cmark]
    /// Contract: Extracts JSON subsystem parts and recursively ingests JSON object parameters via `ingest_dictionary`.
    /// Bounds & Faults: O(N) over JSON payload; returns `false` on read or JSON parse failure.
    pub fn ingest_json_file(&mut self, path: &Path) -> bool {
        let Ok(content) = fs::read_to_string(path) else { return false };
        let fpath = path.to_string_lossy().to_string();
        self.register_extracted_parts(extract_subsystem_parts_from_json(&content, &fpath));
        if let Ok(val) = serde_json::from_str::<Value>(&content) {
            self.ingest_dictionary(&val);
            return true;
        }
        false
    }

    /// [DOC-TIER: A] Ingests SysML v2 parts, ports, attributes, and package identifier from a `.sysml` file.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-SYSML-FILE] [FLS: fls_conops_cmark]
    /// Contract: Parses `SubsystemPart` records with source provenance and delegates attribute extraction to `ingest_sysml_text`.
    /// Bounds & Faults: O(N) over SysML source bytes; returns `false` if file cannot be read.
    pub fn ingest_sysml_file(&mut self, path: &Path) -> bool {
        let Ok(content) = fs::read_to_string(path) else { return false };
        let fpath = path.to_string_lossy().to_string();
        self.register_extracted_parts(extract_subsystem_parts_from_sysml(&content, &fpath));
        self.ingest_sysml_text(&content)
    }

    /// [DOC-TIER: A] Ingests SysML v2 package, parts, and attributes from raw SysML text.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-SYSML-TEXT] [FLS: fls_conops_cmark]
    /// Contract: Extracts package identifier, part names, and `attribute` declarations via `deap_core::sysml_cst`.
    /// Bounds & Faults: O(N) over `text.len()`; never panics on malformed text.
    pub fn ingest_sysml_text(&mut self, text: &str) -> bool {
        if let Some(pkg_name) = find_first_package(text).map(str::trim) {
            if !["Package", "Model", "Root"].contains(&pkg_name) {
                self.inferred_system_identifier = Some(pkg_name.to_string());
                for k in ["SYSTEM_IDENTIFIER", "MISSION_SYSTEM_NAME"] {
                    self.parameter_bindings.insert(k.to_string(), pkg_name.to_string());
                    self.explicit_keys.insert(k.to_string());
                }
            }
        }
        for p_name in deap_core::UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(".sysml", text) {
            let p_name = p_name.trim().to_string();
            self.ast_part_names.insert(p_name.clone());
            self.explicit_keys.insert(p_name.clone());
            self.explicit_keys.insert(p_name.to_uppercase());
        }
        if self.ast_parts.is_empty() {
            self.register_extracted_parts(extract_subsystem_parts_from_sysml(text, "schema/DEAP_MODEL.sysml"));
        }
        for (attr_name, raw_val) in find_all_attributes(text) {
            self.explicit_keys.insert(attr_name.to_string());
            self.explicit_keys.insert(attr_name.to_uppercase());
            self.parameter_bindings.insert(attr_name.to_string(), raw_val.to_string());
            self.parameter_bindings.insert(attr_name.to_uppercase(), raw_val.to_string());
            self._map_semantic_aliases(attr_name, raw_val);
        }
        true
    }

    /// [DOC-TIER: A] Ingests subsystem parts, H1 system title, and GFM table key-value pairs via `deap_core::markdown_ast`.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-MARKDOWN-FILE] [FLS: fls_conops_cmark]
    /// Contract: Uses `parse_markdown_ast` sections and tables to extract system identifier and parameter bindings without line-by-line regex scanning.
    /// Bounds & Faults: O(N) over Markdown document size; ignores headings and tables inside fenced code blocks.
    pub fn ingest_markdown_file(&mut self, path: &Path) -> bool {
        let Ok(content) = fs::read_to_string(path) else { return false };
        let fpath = path.to_string_lossy().to_string();
        self.register_extracted_parts(extract_subsystem_parts_from_markdown(&content, &fpath));
        let ast = parse_markdown_ast(&content);
        if let Some(h1_sec) = ast.sections.iter().find(|s| s.level == 1) {
            let title_clean = h1_sec.heading.replace(['*', '_', '`'], "").trim().to_string();
            let generic = [
                "technical specifications", "specifications", "table of contents", "overview",
                "system specifications", "concept of operations", "conops", "mission intent",
                "requirements", "architecture", "metadata",
            ];
            if !generic.contains(&title_clean.to_lowercase().as_str())
                && !title_clean.is_empty()
                && self.inferred_system_identifier.is_none()
            {
                self.inferred_system_identifier = Some(title_clean.clone());
                for k in ["SYSTEM_IDENTIFIER", "MISSION_SYSTEM_NAME"] {
                    self.parameter_bindings.insert(k.to_string(), title_clean.clone());
                    self.explicit_keys.insert(k.to_string());
                }
            }
        }
        for table in &ast.tables {
            for row in std::iter::once(&table.headers).chain(table.rows.iter()) {
                if let (Some(c0), Some(c1)) = (row.first(), row.get(1)) {
                    let k = c0.replace(['*', '`'], "").trim_matches('_').trim().to_string();
                    let v = c1.trim().to_string();
                    if !k.is_empty() && !v.is_empty() && !k.starts_with("---") {
                        self.parameter_bindings.insert(k.clone(), v.clone());
                        self.parameter_bindings.insert(k.to_uppercase(), v.clone());
                        self._map_semantic_aliases(&k, &v);
                    }
                }
            }
        }
        true
    }

    /// Binds a single JSON key-value pair into `parameter_bindings` (`[REQ-0401]`, `[FLS: fls_conops_cmark]`).
    fn ingest_json_entry(&mut self, k: &str, v: &Value) {
        self.explicit_keys.insert(k.to_uppercase());
        let scalar_opt = match v {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            Value::Bool(b) => Some(b.to_string()),
            _ => None,
        };
        if let Some(s) = scalar_opt {
            self.parameter_bindings.insert(k.to_string(), s.clone());
            self.parameter_bindings.insert(k.to_uppercase(), s.clone());
            self._map_semantic_aliases(k, &s);
            return;
        }
        if let Value::Array(arr) = v {
            let lk = k.to_lowercase();
            if ["core_mission_capabilities", "core_capabilities", "capabilities", "mission_capabilities"].contains(&lk.as_str()) {
                let lines: Vec<String> = arr.iter().enumerate().map(|(idx, item)| {
                    let s = match item { Value::String(st) => st.trim().to_string(), other => other.to_string() };
                    if s.starts_with(&format!("{}.", idx + 1)) || s.starts_with('-') { format!("  {s}") } else { format!("  {}. {s}", idx + 1) }
                }).collect();
                let formatted = lines.join("\n");
                for key in [k.to_string(), k.to_uppercase(), "CORE_MISSION_CAPABILITIES".to_string()] {
                    self.parameter_bindings.insert(key, formatted.clone());
                }
            }
        }
    }

    /// [DOC-TIER: A] Recursively ingests nested parameter dictionaries and `schema_nodes` from a JSON `Value`.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-DICTIONARY] [FLS: fls_conops_cmark]
    /// Contract: Traverses nested JSON parameter containers, binds scalar/array entries, and extracts `schema_nodes`.
    /// Bounds & Faults: O(N) over JSON tree nodes; ignores non-object JSON values safely.
    pub fn ingest_dictionary(&mut self, data: &Value) {
        if let Value::Object(map) = data {
            let containers = ["parameters", "domain_parameters", "domain_params", "specs", "attributes", "metadata"];
            for ck in &containers {
                if let Some(sub) = map.get(*ck) {
                    self.ingest_dictionary(sub);
                }
            }
            for nk in &["system_identifier", "system_name", "system", "name", "SYSTEM_IDENTIFIER", "MISSION_SYSTEM_NAME"] {
                if let Some(Value::String(s)) = map.get(*nk) {
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
                for node in nodes {
                    if let Some((kind, name)) = node.as_str().and_then(|s| s.split_once(':')) {
                        if (kind == "package" || kind == "system") && self.inferred_system_identifier.is_none() {
                            self.inferred_system_identifier = Some(name.trim().to_string());
                        }
                        if kind == "part" {
                            self.ast_part_names.insert(name.trim().to_string());
                        }
                    }
                }
            }
        }
    }
}
