//! Centralized schema discovery and AST part extractor.
//!
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C.
//! Realises: [REQ-SYSML-AST-MODULE], [DO-330-TQL-4-SYSML-AST]

use super::identifiers::is_valid_subsystem_identifier;
use super::interfaces_and_items::parse_part_defs;
use super::parser::{strip_all_block_comments_and_strings, strip_sysml_comments};
use super::parts_and_ports::SupportedExtensionsCatalog;
use super::schema_extractor_data::{extract_parts_from_json, extract_parts_from_yaml_lines};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Centralized schema discovery and AST part extractor shared across compiler and validators.
///
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C.
/// Realises: [REQ-DEAP-CORE-SYSML-AST-UNIFIEDSCHEMAMODELEXTRACTOR] | SysML v2: PartDef(UnifiedSchemaModelExtractor)
#[derive(Debug, Clone, Default)]
pub struct UnifiedSchemaModelExtractor;

impl UnifiedSchemaModelExtractor {
    /// Supported schema extensions across all pipeline tools.
    /// Realises: [REQ-SYSML-AST-MOD-UNIFIEDSCHEMAMODELEXTRACTOR-SUPPORTED-EXTENSIONS]
    pub const SUPPORTED_EXTENSIONS: SupportedExtensionsCatalog = SupportedExtensionsCatalog;

    /// Recursively discovers all supported schema files across the provided search directories.
    /// Excludes hidden entries (`.`), editor swap files (`#`), and `README.md`.
    /// Results are lexicographically sorted to eliminate filesystem traversal jitter.
    ///
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C.
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/DISCOVER-SCHEMA-FILES]
    pub fn discover_schema_files<P: AsRef<Path>>(search_dirs: &[P]) -> Vec<PathBuf> {
        let mut schema_files = Vec::new();
        for sdir in search_dirs {
            let p = sdir.as_ref();
            if !p.is_dir() {
                continue;
            }
            for entry_res in WalkDir::new(p).into_iter().filter_entry(|e| {
                if e.depth() == 0 {
                    return true;
                }
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && !name.starts_with('#')
            }) {
                let Ok(entry) = entry_res else {
                    continue;
                };
                if entry.file_type().is_file() {
                    let fpath = entry.path();
                    let fname = fpath.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let fname_lower = fname.to_ascii_lowercase();
                    if fname.starts_with('.')
                        || fname.starts_with('#')
                        || fname_lower == "readme.md"
                        || fname_lower == "digest.json"
                        || fname_lower.ends_with(".digest.json")
                        || fname_lower.ends_with("-digest.json")
                        || fname_lower.ends_with("_digest.json")
                    {
                        continue;
                    }
                    if let Some(ext) = fpath.extension().and_then(|e| e.to_str()) {
                        let dot_ext = format!(".{}", ext.to_ascii_lowercase());
                        if Self::SUPPORTED_EXTENSIONS.contains(&dot_ext.as_str()) {
                            schema_files.push(fpath.to_path_buf());
                        }
                    }
                }
            }
        }
        schema_files.sort();
        schema_files.dedup();
        schema_files
    }

    /// Extracts all subsystem part and component names across the provided file paths.
    ///
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C.
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-SUBSYSTEM-PARTS]
    pub fn extract_subsystem_parts<P: AsRef<Path>>(file_paths: &[P]) -> BTreeSet<String> {
        let mut part_names = BTreeSet::new();
        for fp in file_paths {
            let p = fp.as_ref();
            let fname_lower = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if fname_lower == "digest.json"
                || fname_lower.ends_with(".digest.json")
                || fname_lower.ends_with("-digest.json")
                || fname_lower.ends_with("_digest.json")
            {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(p) {
                let ext = p
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                let dot_ext = format!(".{}", ext);
                let extracted = Self::extract_subsystem_parts_from_text(&dot_ext, &content);
                part_names.extend(extracted);
            }
        }
        part_names
    }

    /// Extracts subsystem and component names from raw text of a specific format.
    ///
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C.
    /// Realises: [REQ-SYSML-AST-SSOT-PARITY/EXTRACT-SUBSYSTEM-PARTS-TEXT]
    pub fn extract_subsystem_parts_from_text(ext: &str, content: &str) -> BTreeSet<String> {
        let mut part_names = BTreeSet::new();
        let norm_ext = ext.to_ascii_lowercase();

        if norm_ext == ".sysml" {
            let clean = strip_sysml_comments(content);
            for p in parse_part_defs(&clean) {
                if !p.name.is_empty() {
                    part_names.insert(p.name);
                }
            }
            let comment_free = strip_all_block_comments_and_strings(&clean);
            Self::scan_sysml_part_statements(&comment_free, &mut part_names);
        } else if norm_ext == ".json" {
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(content) {
                let is_schema_digest = json_val.as_object().is_some_and(|m| {
                    m.contains_key("format_version")
                        && m.contains_key("source_hash")
                        && !m.contains_key("schema_nodes")
                });
                if !is_schema_digest {
                    extract_parts_from_json(&json_val, &mut part_names);
                }
            }
        } else if norm_ext == ".yaml" || norm_ext == ".yml" {
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(content) {
                extract_parts_from_json(&json_val, &mut part_names);
            }
            extract_parts_from_yaml_lines(content, &mut part_names);
        } else if norm_ext == ".md" || norm_ext == ".markdown" {
            Self::scan_markdown_table_parts(content, &mut part_names);
        } else if norm_ext == ".proto" {
            Self::scan_proto_parts(content, &mut part_names);
        } else if norm_ext == ".idl" {
            Self::scan_idl_parts(content, &mut part_names);
        } else if norm_ext == ".arxml" {
            Self::scan_arxml_parts(content, &mut part_names);
        } else if norm_ext == ".yang" {
            Self::scan_yang_parts(content, &mut part_names);
        }

        part_names
    }

    fn scan_sysml_part_statements(comment_free: &str, part_names: &mut BTreeSet<String>) {
        for line in comment_free.lines() {
            let mut stmt = line.trim();
            for kw in ["public ", "private ", "protected ", "abstract "] {
                if let Some(rest) = stmt.strip_prefix(kw) {
                    stmt = rest.trim_start();
                }
            }
            if let Some(after_part) = stmt.strip_prefix("part ") {
                let after_part = after_part.trim_start();
                if let Some(after_def) = after_part.strip_prefix("def ") {
                    let ident: String = after_def
                        .trim_start()
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    if !ident.is_empty() && ident != "def" && is_valid_subsystem_identifier(&ident)
                    {
                        part_names.insert(ident);
                    }
                } else {
                    let ident: String = after_part
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    let remainder = after_part.get(ident.len()..).unwrap_or("").trim_start();
                    let is_part_usage = remainder.starts_with(':')
                        || remainder.starts_with('{')
                        || remainder.starts_with('[')
                        || remainder.starts_with(';');
                    if is_part_usage && !ident.is_empty() && ident != "def" && is_valid_subsystem_identifier(&ident) {
                        part_names.insert(ident);
                    }
                }
            }
        }
    }

    fn scan_markdown_table_parts(content: &str, part_names: &mut BTreeSet<String>) {
        let mut in_table = false;
        let mut col_subsys_idx = None;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('|') && trimmed.ends_with('|') {
                let cells: Vec<&str> = trimmed
                    .split('|')
                    .map(str::trim)
                    .filter(|c| !c.is_empty())
                    .collect();
                if cells.is_empty() {
                    continue;
                }
                if cells.iter().all(|c| c.chars().all(|ch| ch == '-' || ch == ':' || ch.is_whitespace())) {
                    in_table = true;
                    continue;
                }
                if !in_table {
                    col_subsys_idx = None;
                    for (idx, cell) in cells.iter().enumerate() {
                        let clower = cell.to_ascii_lowercase();
                        if clower.contains("subsystem")
                            || clower.contains("component")
                            || clower.contains("segment")
                            || clower.contains("part")
                            || clower.contains("unit")
                        {
                            col_subsys_idx = Some(idx);
                            break;
                        }
                    }
                } else if let Some(idx) = col_subsys_idx {
                    if let Some(raw_cand) = cells.get(idx).copied() {
                        let clean_cand = raw_cand
                            .replace(['`', '*', '_', '[', ']'], "")
                            .trim()
                            .to_string();
                        let clean_ident: String = clean_cand
                            .chars()
                            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                            .collect();
                        if is_valid_subsystem_identifier(&clean_ident) {
                            part_names.insert(clean_ident);
                        }
                    }
                }
            } else {
                in_table = false;
                col_subsys_idx = None;
            }
        }
    }

    fn scan_proto_parts(content: &str, part_names: &mut BTreeSet<String>) {
        for line in content.lines() {
            let trimmed = line.trim();
            for kw in &["service ", "message "] {
                if let Some(rest) = trimmed.strip_prefix(kw) {
                    let ident: String = rest
                        .trim_start()
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    if !ident.is_empty() && is_valid_subsystem_identifier(&ident) {
                        part_names.insert(ident);
                    }
                }
            }
        }
    }

    fn scan_idl_parts(content: &str, part_names: &mut BTreeSet<String>) {
        for line in content.lines() {
            let trimmed = line.trim();
            for kw in &["interface ", "module ", "struct "] {
                if let Some(rest) = trimmed.strip_prefix(kw) {
                    let ident: String = rest
                        .trim_start()
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .collect();
                    if !ident.is_empty() && is_valid_subsystem_identifier(&ident) {
                        part_names.insert(ident);
                    }
                }
            }
        }
    }

    fn scan_arxml_parts(content: &str, part_names: &mut BTreeSet<String>) {
        for line in content.lines() {
            if line.contains("<SHORT-NAME>") && line.contains("</SHORT-NAME>") {
                if let Some(start) = line.find("<SHORT-NAME>") {
                    let after = line.get(start + 12..).unwrap_or("");
                    if let Some(end) = after.find("</SHORT-NAME>") {
                        let ident = after.get(..end).unwrap_or("").trim();
                        if is_valid_subsystem_identifier(ident) {
                            part_names.insert(ident.to_string());
                        }
                    }
                }
            }
        }
    }

    fn scan_yang_parts(content: &str, part_names: &mut BTreeSet<String>) {
        for line in content.lines() {
            let trimmed = line.trim();
            for kw in &["container ", "grouping ", "list "] {
                if let Some(rest) = trimmed.strip_prefix(kw) {
                    let ident: String = rest
                        .trim_start()
                        .chars()
                        .take_while(|c| {
                            c.is_ascii_alphanumeric() || *c == '_' || *c == '-' || *c == '.'
                        })
                        .collect();
                    if !ident.is_empty() && is_valid_subsystem_identifier(&ident) {
                        part_names.insert(ident);
                    }
                }
            }
        }
    }
}
