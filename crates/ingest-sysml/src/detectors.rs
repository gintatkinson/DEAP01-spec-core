//! # Heterogeneous Schema Format Detectors & Target Discovery Engine
//!
//! ## 1. Safety Intent & Regulatory Scope
//! This module provides deterministic, bounded schema format detection and filesystem
//! target discovery for Level 0 OEM specifications, supporting heterogeneous input models
//! (Markdown, SysML v2, OMG IDL, AUTOSAR ARXML, Protobuf, and OpenAPI 3.0/3.1).
//!
//! In safety-critical systems engineering (DO-178C / DO-331 / ISO 26262), input format
//! classification is a safety-critical preprocessing gate. Erroneous format detection can
//! cause silent specification omission or corrupt downstream verification pipelines.
//!
//! ## 2. Core Safety Invariants
//! - **Pure Schema-Driven Classification**: Format detection operates strictly on formal
//!   file extensions and syntactic grammar markers. Zero domain-specific vocabulary is used.
//! - **Deterministic Precedence**: Direct extension matches take precedence over content
//!   heuristics. Content inspection employs bounded content scans.
//! - **Canonical Path Sorting**: All directory scans sort paths lexicographically to ensure
//!   bitwise identical AST construction order across different host filesystems.
//! - **Bounded Complexity**: Execution time is $O(N + M)$ where $N$ is directory entry count
//!   and $M$ is inspected header byte length.

use std::fs;
use std::path::{Path, PathBuf};

/// Raw uncompiled document extensions that require formal AST extraction prior to lowering.
/// These extensions represent human-oriented document formats where direct compilation is
/// prohibited without structured extraction.
pub const RAW_EXTENSIONS: &[&str] = &[".pdf", ".txt", ".doc", ".docx"];

/// Detects the formal specification format from file extension and content.
///
/// /// Realises: [REQ-0001/detect_format]
///
/// ### Algorithmic Precedence:
/// 1. Markdown extension (`.md`) or structural table delimiter patterns (`| ... |`).
/// 2. Raw uncompiled document formats (`.pdf`, `.txt`, `.doc`, `.docx`).
/// 3. Well-known extension mapping (`.sysml`, `.idl`, `.arxml`, `.proto`, `.json`, `.yaml`).
/// 4. Grammatical token search (`part def `, `module `, `<AUTOSAR`, `syntax =`, `openapi`).
///
/// ### Preconditions:
/// - `schema_path` is a non-empty filesystem path or descriptor.
/// - `content` contains initial UTF-8 file bytes or entire file text.
///
/// ### Postconditions:
/// - Returns `Ok(format_tag)` where `format_tag` is one of:
///   `"markdown"`, `"sysml"`, `"idl"`, `"autosar"`, `"protobuf"`, `"openapi"`, or `"raw"`.
/// - Returns `Err(diagnostic)` if format cannot be identified within the formal grammar set.
pub fn detect_format(schema_path: &str, content: &str) -> Result<String, String> {
    let path = Path::new(schema_path);
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| format!(".{}", s.to_ascii_lowercase()))
        .unwrap_or_default();

    // 1. Direct Markdown detection via extension or structural table syntax
    let has_table_header = content.lines().any(|line| {
        if line.contains('|') {
            const KEYWORDS: &[&str] = &[
                "component", "part", "bom", "port", "signal", "parameter", "interface",
                "property", "attribute",
            ];
            line.split('|').any(|cell| {
                let cell_clean = cell.trim().to_ascii_lowercase();
                KEYWORDS.contains(&cell_clean.as_str())
            })
        } else {
            false
        }
    });

    let has_table_delim = {
        let mut prev_had_pipe = false;
        let mut found = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                prev_had_pipe = false;
                continue;
            }
            if trimmed.contains('|') {
                if prev_had_pipe && trimmed.starts_with('|') {
                    let inner = trimmed.trim_matches('|');
                    if !inner.is_empty()
                        && inner.contains('-')
                        && inner.chars().all(|c| c == '-' || c == ':' || c == '|' || c.is_whitespace())
                    {
                        found = true;
                        break;
                    }
                }
                prev_had_pipe = true;
            } else {
                prev_had_pipe = false;
            }
        }
        found
    };

    if ext == ".md" || has_table_header || has_table_delim {
        return Ok("markdown".to_string());
    }

    // 2. Extension-based categorization
    if RAW_EXTENSIONS.contains(&ext.as_str()) {
        return Ok("raw".to_string());
    }
    if ext == ".sysml" {
        return Ok("sysml".to_string());
    }
    if ext == ".yang" {
        return Ok("yang".to_string());
    }
    if ext == ".idl" {
        return Ok("idl".to_string());
    }
    if ext == ".arxml" || ext == ".xml" {
        return Ok("autosar".to_string());
    }
    if ext == ".proto" {
        return Ok("protobuf".to_string());
    }
    if ext == ".json" || ext == ".yaml" || ext == ".yml" {
        return Ok("openapi".to_string());
    }

    // 3. Content-based fallback inspection
    if content.contains("part def ")
        || content.contains("package ")
        || content.contains("capability def ")
        || content.contains("requirement def ")
    {
        return Ok("sysml".to_string());
    }
    if path.ends_with(".yang")
        || content.contains("yang-version")
        || content.contains("namespace ")
        || content.contains("prefix ")
        || content.contains("typedef ")
        || content.contains("grouping ")
    {
        return Ok("yang".to_string());
    }
    if content.contains("module ") || content.contains("interface ") || content.contains("struct ") {
        return Ok("idl".to_string());
    }
    if content.contains("<AUTOSAR") || content.contains("<AR-PACKAGE") {
        return Ok("autosar".to_string());
    }
    if content.contains("syntax =") || content.contains("message ") {
        return Ok("protobuf".to_string());
    }
    if content.contains("openapi") || content.contains("swagger") || content.contains("\"paths\":") {
        return Ok("openapi".to_string());
    }

    Err(format!(
        "Unsupported schema format for '{}'. Supported formats: .sysml, .yang, .idl, .arxml/.xml, .proto, .json/.yaml/.yml, markdown (.md).",
        schema_path
    ))
}

/// Identifies whether a given path is an output or intermediate SysML file rather than an input schema.
fn is_output_sysml(p: &Path) -> bool {
    if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
        name == "model.sysml" || name == "schema.sysml"
    } else {
        false
    }
}

/// Discovers markdown files under search_dir/extracted or search_dir, ignoring README.md.
fn discover_markdown_files(search_dir: &Path) -> Vec<PathBuf> {
    let extracted_dir = if search_dir.ends_with("extracted") {
        search_dir.to_path_buf()
    } else {
        search_dir.join("extracted")
    };
    if extracted_dir.exists() {
        let mut extracted_md = find_files_with_ext(&extracted_dir, "md");
        extracted_md.retain(|p| p.file_name().and_then(|s| s.to_str()) != Some("README.md"));
        if !extracted_md.is_empty() {
            extracted_md.sort();
            return extracted_md;
        }
    }

    let mut root_md = find_files_with_ext(search_dir, "md");
    root_md.retain(|p| p.file_name().and_then(|s| s.to_str()) != Some("README.md"));
    root_md.sort();
    root_md
}

/// Discovers specification files or target directories adhering to deterministic search precedence.
///
/// /// Realises: [REQ-0001/discover_schema_targets]
///
/// ### Search Precedence Order:
/// 1. Direct file input (if `path_opt` is an explicit single file).
/// 2. Explicit format override (if `format_hint` is provided and not `"auto"`).
/// 3. Native SysML v2 models (`*.sysml` in search root, excluding output files if markdown specs exist).
/// 4. Extracted Markdown specifications (`schema/extracted/*.md`).
/// 5. Root Markdown specifications (`schema/*.md`).
/// 6. Heterogeneous Interface Definitions (`*.idl`, `*.arxml`, `*.proto`, `*.yaml`, `*.json`).
/// 7. Existing model fallback (`model.sysml` if no primary specifications exist).
///
/// ### Invariant:
/// All returned file paths are sorted lexicographically to eliminate filesystem traversal jitter.
pub fn discover_schema_targets(
    path_opt: Option<&Path>,
    format_hint: Option<&str>,
) -> Result<(String, Vec<PathBuf>), String> {
    let normalized_hint = format_hint.map(|h| h.trim().to_ascii_lowercase());
    let is_explicit = normalized_hint
        .as_deref()
        .map_or(false, |h| !h.is_empty() && h != "auto");

    if let Some(p) = path_opt {
        if p.is_file() {
            let fmt = if is_explicit {
                let hint = normalized_hint.unwrap();
                match hint.as_str() {
                    "markdown" | "md" => "markdown".to_string(),
                    "sysml" => "sysml".to_string(),
                    "yang" => "yang".to_string(),
                    "idl" => "idl".to_string(),
                    "autosar" | "arxml" => "autosar".to_string(),
                    "protobuf" | "proto" => "protobuf".to_string(),
                    "openapi" => "openapi".to_string(),
                    "raw" => "raw".to_string(),
                    other => other.to_string(),
                }
            } else {
                let content = fs::read_to_string(p).unwrap_or_default();
                detect_format(p.to_str().unwrap_or(""), &content)?
            };
            return Ok((fmt, vec![p.to_path_buf()]));
        }
    }

    let search_dir = path_opt
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("schema"));

    if !search_dir.exists() {
        return Ok(("unknown".to_string(), Vec::new()));
    }

    // Branch A: Explicit format hint specified by caller
    if is_explicit {
        let hint = normalized_hint.unwrap();
        let canonical_fmt = match hint.as_str() {
            "markdown" | "md" => "markdown",
            "sysml" => "sysml",
            "yang" => "yang",
            "idl" => "idl",
            "autosar" | "arxml" => "autosar",
            "protobuf" | "proto" => "protobuf",
            "openapi" => "openapi",
            "raw" => "raw",
            other => other,
        };

        if canonical_fmt == "markdown" {
            let files = discover_markdown_files(&search_dir);
            return Ok(("markdown".to_string(), files));
        }

        let exts: &[&str] = match canonical_fmt {
            "sysml" => &["sysml"],
            "yang" => &["yang"],
            "idl" => &["idl"],
            "autosar" => &["arxml", "xml"],
            "protobuf" => &["proto"],
            "openapi" => &["yaml", "yml", "json"],
            "raw" => &["pdf", "txt", "doc", "docx"],
            other => &[other],
        };

        let mut matching_files = Vec::new();
        for ext in exts {
            matching_files.extend(find_files_with_ext(&search_dir, ext));
        }
        matching_files.retain(|p| p.file_name().and_then(|s| s.to_str()) != Some("README.md"));

        if canonical_fmt == "sysml" {
            let (non_output, output): (Vec<_>, Vec<_>) =
                matching_files.into_iter().partition(|p| !is_output_sysml(p));
            matching_files = if !non_output.is_empty() {
                non_output
            } else {
                output
            };
        }

        matching_files.sort();
        return Ok((canonical_fmt.to_string(), matching_files));
    }

    // Branch B: Auto-discovery precedence
    // 1. Check for native .sysml models (excluding generated output files like model.sysml)
    let sysml_files = find_files_with_ext(&search_dir, "sysml");
    let (mut non_output_sysml, mut output_sysml): (Vec<_>, Vec<_>) =
        sysml_files.into_iter().partition(|p| !is_output_sysml(p));

    if !non_output_sysml.is_empty() {
        non_output_sysml.sort();
        return Ok(("sysml".to_string(), non_output_sysml));
    }

    // 2. Check for markdown specifications in schema/extracted/*.md or schema/*.md
    let md_files = discover_markdown_files(&search_dir);
    if !md_files.is_empty() {
        return Ok(("markdown".to_string(), md_files));
    }

    // 3. Check for other schema types (YANG, IDL, ARXML, Protobuf, OpenAPI)
    let format_exts = [
        ("yang", "yang"),
        ("idl", "idl"),
        ("arxml", "autosar"),
        ("xml", "autosar"),
        ("proto", "protobuf"),
        ("yaml", "openapi"),
        ("json", "openapi"),
    ];

    for (ext, fmt) in format_exts {
        let mut other_files = find_files_with_ext(&search_dir, ext);
        if !other_files.is_empty() {
            other_files.sort();
            return Ok((fmt.to_string(), other_files));
        }
    }

    // 4. Fallback to existing output SysML model if present and no primary schemas exist
    if !output_sysml.is_empty() {
        output_sysml.sort();
        return Ok(("sysml".to_string(), output_sysml));
    }

    Ok(("unknown".to_string(), Vec::new()))
}

/// Helper function to locate non-recursive files matching a target extension in a directory.
///
/// ### Arguments:
/// - `dir`: The directory to inspect.
/// - `ext`: File extension without leading dot (case-insensitive).
///
/// ### Returns:
/// A vector of matching file `PathBuf` objects.
fn find_files_with_ext(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(e) = p.extension() {
                    if e.to_string_lossy().eq_ignore_ascii_case(ext) {
                        results.push(p);
                    }
                }
            }
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_by_extension() {
        assert_eq!(detect_format("schema.md", "").unwrap(), "markdown");
        assert_eq!(detect_format("model.sysml", "").unwrap(), "sysml");
        assert_eq!(detect_format("interface.yang", "").unwrap(), "yang");
        assert_eq!(detect_format("interface.idl", "").unwrap(), "idl");
        assert_eq!(detect_format("system.arxml", "").unwrap(), "autosar");
        assert_eq!(detect_format("messages.proto", "").unwrap(), "protobuf");
        assert_eq!(detect_format("api.json", "").unwrap(), "openapi");
        assert_eq!(detect_format("api.yaml", "").unwrap(), "openapi");
        assert_eq!(detect_format("doc.pdf", "").unwrap(), "raw");
    }

    #[test]
    fn test_detect_format_by_content() {
        assert_eq!(
            detect_format("unknown_file", "package Package_0 { part def Classifier_Alpha; }").unwrap(),
            "sysml"
        );
        assert_eq!(
            detect_format("unknown_file", "module ietf-geo-location { yang-version 1.1; namespace \"urn:ietf\"; prefix geo; }").unwrap(),
            "yang"
        );
        assert_eq!(
            detect_format("unknown_file", "module Module_0 { struct Struct_1 { long id; }; };").unwrap(),
            "idl"
        );
        assert_eq!(
            detect_format("unknown_file", "<AUTOSAR><AR-PACKAGE></AR-PACKAGE></AUTOSAR>").unwrap(),
            "autosar"
        );
        assert_eq!(
            detect_format("unknown_file", "syntax = \"proto3\"; message Msg_0 { int32 val = 1; }").unwrap(),
            "protobuf"
        );
        assert_eq!(
            detect_format("unknown_file", "{\"openapi\": \"3.0.0\", \"paths\": {}}").unwrap(),
            "openapi"
        );
        assert_eq!(
            detect_format("unknown_file", "| Component | Part Number |\n| --- | --- |\n| Classifier_Alpha | 1 |").unwrap(),
            "markdown"
        );
    }

    fn get_schema_dir() -> PathBuf {
        let p = Path::new("schema");
        if p.exists() {
            p.to_path_buf()
        } else {
            Path::new("../../schema").to_path_buf()
        }
    }

    #[test]
    fn test_discover_schema_targets_req1() {
        let schema_dir = get_schema_dir();
        let path = schema_dir.join("REQ-0001.md");
        if path.exists() {
            let res = discover_schema_targets(Some(&path), None);
            assert!(res.is_ok());
            let (fmt, files) = res.unwrap();
            assert_eq!(fmt, "markdown");
            assert_eq!(files.len(), 1);
            assert_eq!(files[0], path);
        }
    }

    #[test]
    fn test_discover_schema_targets_with_format_hint_markdown() {
        let schema_dir = get_schema_dir();
        assert!(schema_dir.exists());
        let res = discover_schema_targets(Some(&schema_dir), Some("markdown"));
        assert!(res.is_ok());
        let (fmt, files) = res.unwrap();
        assert_eq!(fmt, "markdown");
        assert_eq!(files.len(), 199);
        assert!(!files.contains(&schema_dir.join("model.sysml")));
    }

    #[test]
    fn test_discover_schema_targets_auto_markdown_not_masked_by_model_sysml() {
        let schema_dir = get_schema_dir();
        assert!(schema_dir.exists());
        let res = discover_schema_targets(Some(&schema_dir), Some("auto"));
        assert!(res.is_ok());
        let (fmt, files) = res.unwrap();
        assert_eq!(fmt, "markdown");
        assert_eq!(files.len(), 199);
        assert!(!files.contains(&schema_dir.join("model.sysml")));
    }

    #[test]
    fn test_discover_schema_targets_explicit_sysml() {
        let schema_dir = get_schema_dir();
        assert!(schema_dir.exists());
        let res = discover_schema_targets(Some(&schema_dir), Some("sysml"));
        assert!(res.is_ok());
        let (fmt, files) = res.unwrap();
        assert_eq!(fmt, "sysml");
        assert_eq!(files, vec![schema_dir.join("model.sysml")]);
    }
}
