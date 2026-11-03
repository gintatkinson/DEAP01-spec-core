//! Workspace discovery and multi-format schema/Markdown AST ingestion for `SysMLParameterBindingEngine`.

use std::fs;
use std::path::{Path, PathBuf};

use deap_core::markdown_ast::parse_markdown_ast;
use serde_json::Value;

use super::schema_extractor::{
    extract_subsystem_parts_from_interface_schema, extract_subsystem_parts_from_json,
    extract_subsystem_parts_from_markdown, extract_subsystem_parts_from_sysml,
    extract_subsystem_parts_from_yaml, find_all_attributes, find_first_package,
};
use super::SysMLParameterBindingEngine;

impl SysMLParameterBindingEngine {
    /// Discovers parameter and schema sources across ancestor directories up to 5 levels.
    /// Inputs: `root_dir: &Path` | Outputs: `()` | Invariants: deterministic ancestor traversal
    ///Errors: silently skips inaccessible paths | Complexity: O(D) over directory depth
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-AUTO-DISCOVER-SOURCES] | SysML v2: ActionDef(SysMLParameterBindingEngine::auto_discover_sources)
    pub fn auto_discover_sources(&mut self, root_dir: &Path) {
        let mut search_dirs = Vec::new();
        let mut curr = root_dir.to_path_buf();
        for _ in 0..5 {
            search_dirs.push(curr.clone());
            if let Some(parent) = curr.parent() {
                if parent == curr {
                    break;
                }
                curr = parent.to_path_buf();
            } else {
                break;
            }
        }
        self.auto_detect_workspace_parameters(&search_dirs);
    }

    /// Scans candidate workspace directories for `.sysml`, `.json`, `.yaml`, `.md`, and interface schemas.
    /// Inputs: `search_dirs: &[PathBuf]` | Outputs: `()` | Invariants: respects upstream firewall guard
    ///Errors: skips unreadable files | Complexity: O(F) over workspace schema files
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-AUTO-DETECT-WORKSPACE-PARAMETERS] | SysML v2: ActionDef(SysMLParameterBindingEngine::auto_detect_workspace_parameters)
    pub fn auto_detect_workspace_parameters(&mut self, search_dirs: &[PathBuf]) {
        let supported_exts = deap_core::UnifiedSchemaModelExtractor::SUPPORTED_EXTENSIONS;

        for sdir in search_dirs {
            if sdir.join(".pipeline").join("upstream").is_dir() {
                continue;
            }

            let schema_sysml = sdir.join(".pipeline").join("schema.sysml");
            if schema_sysml.is_file() {
                self.ingest_file(&schema_sysml);
            }
            let schema_digest = sdir.join(".pipeline").join("schema-digest.json");
            if schema_digest.is_file() {
                self.ingest_file(&schema_digest);
            }

            for rel_dir in &["schema", "docs/architecture", "docs/research"] {
                let cdir = sdir.join(rel_dir);
                if cdir.is_dir() {
                    for entry in walkdir::WalkDir::new(&cdir)
                        .sort_by_file_name()
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        let path = entry.path();
                        if path.is_file() {
                            let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                            if fname.starts_with('.') {
                                continue;
                            }
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

    /// Dispatches file ingestion by file extension and updates derived architecture bindings.
    /// Inputs: `path: &Path` | Outputs: `bool` | Invariants: idempotent AST part registration
    ///Errors: returns `false` on unsupported or missing file | Complexity: O(N) over file size
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-FILE] | SysML v2: ActionDef(SysMLParameterBindingEngine::ingest_file)
    pub fn ingest_file(&mut self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

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

    /// Ingests subsystem parts and scalar key-value bindings from a YAML file.
    /// Inputs: `path: &Path` | Outputs: `bool` | Invariants: preserves existing part order
    ///Errors: returns `false` if file cannot be read | Complexity: O(N) over YAML lines
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-YAML-FILE] | SysML v2: ActionDef(SysMLParameterBindingEngine::ingest_yaml_file)
    pub fn ingest_yaml_file(&mut self, path: &Path) -> bool {
        if let Ok(content) = fs::read_to_string(path) {
            let fpath = path.to_string_lossy().to_string();
            let yaml_parts = extract_subsystem_parts_from_yaml(&content, &fpath);
            for p in yaml_parts {
                self.ast_part_names.insert(p.name.clone());
                self.explicit_keys.insert(p.name.clone());
                self.explicit_keys.insert(p.name.to_uppercase());
                if !self.ast_parts.iter().any(|existing| existing.name == p.name) {
                    self.ast_parts.push(p);
                }
            }

            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('-') {
                    continue;
                }
                if let Some((k, v)) = trimmed.split_once(':') {
                    let k_clean = k.trim().replace(['"', '\'', '`'], "").to_string();
                    let v_clean = v
                        .trim()
                        .trim_matches(['"', '\'', '`', ';'])
                        .trim()
                        .to_string();
                    if !k_clean.is_empty()
                        && !v_clean.is_empty()
                        && !v_clean.starts_with('[')
                        && !v_clean.starts_with('{')
                    {
                        self.parameter_bindings
                            .insert(k_clean.clone(), v_clean.clone());
                        self.parameter_bindings
                            .insert(k_clean.to_uppercase(), v_clean.clone());
                        self._map_semantic_aliases(&k_clean, &v_clean);
                    }
                }
            }
            return true;
        }
        false
    }

    /// Ingests subsystem parts from `.proto`, `.idl`, or `.arxml` interface schema files.
    /// Inputs: `path: &Path` | Outputs: `bool` | Invariants: preserves existing part order
    ///Errors: returns `false` if file cannot be read | Complexity: O(N) over file size
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-INTERFACE-SCHEMA-FILE] | SysML v2: ActionDef(SysMLParameterBindingEngine::ingest_interface_schema_file)
    pub fn ingest_interface_schema_file(&mut self, path: &Path) -> bool {
        if let Ok(content) = fs::read_to_string(path) {
            let fpath = path.to_string_lossy().to_string();
            let schema_parts = extract_subsystem_parts_from_interface_schema(&content, &fpath);
            for p in schema_parts {
                self.ast_part_names.insert(p.name.clone());
                self.explicit_keys.insert(p.name.clone());
                self.explicit_keys.insert(p.name.to_uppercase());
                if !self.ast_parts.iter().any(|existing| existing.name == p.name) {
                    self.ast_parts.push(p);
                }
            }
            return true;
        }
        false
    }

    /// Ingests subsystem parts and JSON object parameters from a `.json` file.
    /// Inputs: `path: &Path` | Outputs: `bool` | Invariants: preserves existing part order
    ///Errors: returns `false` on read or JSON parse failure | Complexity: O(N) over JSON payload
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-JSON-FILE] | SysML v2: ActionDef(SysMLParameterBindingEngine::ingest_json_file)
    pub fn ingest_json_file(&mut self, path: &Path) -> bool {
        if let Ok(content) = fs::read_to_string(path) {
            let fpath = path.to_string_lossy().to_string();
            let json_parts = extract_subsystem_parts_from_json(&content, &fpath);
            for p in json_parts {
                self.ast_part_names.insert(p.name.clone());
                self.explicit_keys.insert(p.name.clone());
                self.explicit_keys.insert(p.name.to_uppercase());
                if !self.ast_parts.iter().any(|existing| existing.name == p.name) {
                    self.ast_parts.push(p);
                }
            }

            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                self.ingest_dictionary(&val);
                return true;
            }
        }
        false
    }

    /// Ingests SysML v2 parts, ports, attributes, and package identifier from a `.sysml` file.
    /// Inputs: `path: &Path` | Outputs: `bool` | Invariants: preserves source provenance
    ///Errors: returns `false` if file cannot be read | Complexity: O(N) over SysML source bytes
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-SYSML-FILE] | SysML v2: ActionDef(SysMLParameterBindingEngine::ingest_sysml_file)
    pub fn ingest_sysml_file(&mut self, path: &Path) -> bool {
        if let Ok(content) = fs::read_to_string(path) {
            let fpath = path.to_string_lossy().to_string();
            let parsed_parts = extract_subsystem_parts_from_sysml(&content, &fpath);
            for p in &parsed_parts {
                self.ast_part_names.insert(p.name.clone());
                self.explicit_keys.insert(p.name.clone());
                self.explicit_keys.insert(p.name.to_uppercase());
                if !self.ast_parts.iter().any(|existing| existing.name == p.name) {
                    self.ast_parts.push(p.clone());
                }
            }
            return self.ingest_sysml_text(&content);
        }
        false
    }

    /// Ingests SysML v2 package, parts, and attributes from raw SysML text.
    /// Inputs: `text: &str` | Outputs: `bool` | Invariants: maps semantic aliases for all attributes
    ///Errors: never panics on malformed text | Complexity: O(N) over text length
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-SYSML-TEXT] | SysML v2: ActionDef(SysMLParameterBindingEngine::ingest_sysml_text)
    pub fn ingest_sysml_text(&mut self, text: &str) -> bool {
        if let Some(pkg_name) = find_first_package(text) {
            let pkg_name = pkg_name.trim();
            if !["Package", "Model", "Root"].contains(&pkg_name) {
                self.inferred_system_identifier = Some(pkg_name.to_string());
                self.parameter_bindings
                    .insert("SYSTEM_IDENTIFIER".to_string(), pkg_name.to_string());
                self.parameter_bindings
                    .insert("MISSION_SYSTEM_NAME".to_string(), pkg_name.to_string());
                self.explicit_keys.insert("SYSTEM_IDENTIFIER".to_string());
                self.explicit_keys.insert("MISSION_SYSTEM_NAME".to_string());
            }
        }

        let parts =
            deap_core::UnifiedSchemaModelExtractor::extract_subsystem_parts_from_text(
                ".sysml", text,
            );
        for p_name in parts {
            let p_name = p_name.trim().to_string();
            self.ast_part_names.insert(p_name.clone());
            self.explicit_keys.insert(p_name.clone());
            self.explicit_keys.insert(p_name.to_uppercase());
        }

        if self.ast_parts.is_empty() {
            let fallback_parts = extract_subsystem_parts_from_sysml(text, "schema/DEAP_MODEL.sysml");
            for p in fallback_parts {
                if !self.ast_parts.iter().any(|existing| existing.name == p.name) {
                    self.ast_parts.push(p);
                }
            }
        }

        for (attr_name, raw_val) in find_all_attributes(text) {
            self.explicit_keys.insert(attr_name.to_string());
            self.explicit_keys.insert(attr_name.to_uppercase());
            self.parameter_bindings
                .insert(attr_name.to_string(), raw_val.to_string());
            self.parameter_bindings
                .insert(attr_name.to_uppercase(), raw_val.to_string());
            self._map_semantic_aliases(attr_name, raw_val);
        }

        true
    }

    /// Ingests subsystem parts, H1 system title, and GFM table key-value pairs via `deap_core::markdown_ast`.
    /// Inputs: `path: &Path` | Outputs: `bool` | Invariants: uses `pulldown-cmark` AST sections and tables
    ///Errors: returns `false` if file cannot be read | Complexity: O(N) over Markdown document size
    /// Realises: [REQ-ASSEMBLE-CONOPS-PARAMS-SYSMLPARAMETERBINDINGENGINE-INGEST-MARKDOWN-FILE] | SysML v2: ActionDef(SysMLParameterBindingEngine::ingest_markdown_file)
    pub fn ingest_markdown_file(&mut self, path: &Path) -> bool {
        if let Ok(content) = fs::read_to_string(path) {
            let fpath = path.to_string_lossy().to_string();
            let md_parts = extract_subsystem_parts_from_markdown(&content, &fpath);
            for p in md_parts {
                self.ast_part_names.insert(p.name.clone());
                self.explicit_keys.insert(p.name.clone());
                self.explicit_keys.insert(p.name.to_uppercase());
                if !self.ast_parts.iter().any(|existing| existing.name == p.name) {
                    self.ast_parts.push(p);
                }
            }

            let ast = parse_markdown_ast(&content);
            if let Some(h1_sec) = ast.sections.iter().find(|s| s.level == 1) {
                let title_clean = h1_sec.heading.replace(['*', '_', '`'], "").trim().to_string();
                let generic = [
                    "technical specifications",
                    "specifications",
                    "table of contents",
                    "overview",
                    "system specifications",
                    "concept of operations",
                    "conops",
                    "mission intent",
                    "requirements",
                    "architecture",
                    "metadata",
                ];
                if !generic.contains(&title_clean.to_lowercase().as_str())
                    && !title_clean.is_empty()
                    && self.inferred_system_identifier.is_none()
                {
                    self.inferred_system_identifier = Some(title_clean.clone());
                    self.parameter_bindings
                        .insert("SYSTEM_IDENTIFIER".to_string(), title_clean.clone());
                    self.parameter_bindings
                        .insert("MISSION_SYSTEM_NAME".to_string(), title_clean);
                    self.explicit_keys.insert("SYSTEM_IDENTIFIER".to_string());
                    self.explicit_keys.insert("MISSION_SYSTEM_NAME".to_string());
                }
            }

            for table in &ast.tables {
                for row in std::iter::once(&table.headers).chain(table.rows.iter()) {
                    if let (Some(c0), Some(c1)) = (row.first(), row.get(1)) {
                        let k = c0
                            .replace(['*', '`'], "")
                            .trim_matches('_')
                            .trim()
                            .to_string();
                        let v = c1.trim().to_string();
                        if !k.is_empty() && !v.is_empty() && !k.starts_with("---") {
                            self.parameter_bindings.insert(k.clone(), v.clone());
                            self.parameter_bindings.insert(k.to_uppercase(), v.clone());
                            self._map_semantic_aliases(&k, &v);
                        }
                    }
                }
            }
            return true;
        }
        false
    }
}
