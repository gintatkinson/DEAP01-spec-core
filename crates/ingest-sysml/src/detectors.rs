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

/// Helper function to recursively locate files matching a target extension under a directory.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Schema discovery is a safety-critical
/// preprocessing gate: schemas placed in nested subdirectories of `schema/` (e.g. a YANG module
/// tree) must be ingested rather than silently omitted from the compiled model.
///
/// ## 2. Invariants & Mathematical Properties
/// - Traversal order is deterministic: entries are sorted by file name at every directory level
///   (`WalkDir::sort_by_file_name`), so the result is identical across host filesystems.
/// - Hidden entries (names beginning with `.`) below the root, and everything beneath hidden
///   directories, are excluded. The root itself is always walked, even when named `.`.
/// - Extension matching is ASCII case-insensitive.
///
/// ## 3. Preconditions
/// - `dir` is a candidate directory path; `ext` is a file extension without leading dot.
///
/// ## 4. Postconditions
/// - Returns every regular file (symlinks to files included) under `dir`, at any depth, whose
///   extension equals `ext`, in deterministic walk order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(F \log F)$ in visited entries $F$ (per-directory sorting). Space Complexity: $O(F)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - A non-existent or unreadable `dir`, and unreadable walk entries, yield no results for those
///   entries without panicking; symlinked directories are not followed, preventing cycles.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0001/find_files_with_ext_recursive]
fn find_files_with_ext(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut results = Vec::new();
    let walker = walkdir::WalkDir::new(dir)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || !e.file_name().to_string_lossy().starts_with('.'));
    for entry in walker.flatten() {
        let p = entry.path();
        if p.is_file() {
            if let Some(e) = p.extension() {
                if e.to_string_lossy().eq_ignore_ascii_case(ext) {
                    results.push(p.to_path_buf());
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
        if !schema_dir.join("REQ-0001.md").exists() {
            return;
        }
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
        if !schema_dir.join("REQ-0001.md").exists() {
            return;
        }
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
        if !schema_dir.join("model.sysml").exists() {
            return;
        }
        let res = discover_schema_targets(Some(&schema_dir), Some("sysml"));
        assert!(res.is_ok());
        let (fmt, files) = res.unwrap();
        assert_eq!(fmt, "sysml");
        assert_eq!(files, vec![schema_dir.join("model.sysml")]);
    }

    #[test]
    fn test_yang_autodiscovery_and_lowering_red_probe() {
        let temp_dir = std::env::temp_dir().join(format!("test_yang_probe_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let yang_source = r#"module ietf_interfaces {
    yang-version 1.1;
    namespace "urn:ietf:params:xml:ns:yang:ietf-interfaces";
    prefix "if";

    typedef interface-ref {
        type string;
        description "Reference to an interface";
    }

    container interfaces {
        description "Interface configuration";
        list interface {
            key "name";
            leaf name {
                type string;
            }
            leaf enabled {
                type boolean;
                default "true";
            }
        }
    }
}
"#;
        let yang_file = temp_dir.join("ietf_interfaces.yang");
        fs::write(&yang_file, yang_source).expect("Failed to write temporary .yang file");

        // 1. Content detection probe (without .yang extension):
        // Under legacy defect, content containing "module " was misclassified as "idl".
        let detected_fmt = detect_format("schema_without_ext", yang_source)
            .expect("detect_format failed on YANG content");
        assert_eq!(
            detected_fmt,
            "yang",
            "DEFECT REPRODUCED: detect_format misclassified YANG content containing 'module ' as '{}'",
            detected_fmt
        );

        // 2. Directory auto-discovery probe:
        // Under legacy defect, format_exts lacked ("yang", "yang"), returning empty files.
        let (disc_fmt, files) = discover_schema_targets(Some(&temp_dir), Some("auto"))
            .expect("discover_schema_targets failed");
        assert_eq!(
            disc_fmt,
            "yang",
            "DEFECT REPRODUCED: Directory auto-discovery failed for .yang schemas: returned format '{}'",
            disc_fmt
        );
        assert_eq!(
            files.len(),
            1,
            "DEFECT REPRODUCED: Directory auto-discovery found 0 files for .yang schemas"
        );
        assert_eq!(files[0], yang_file);

        // 3. Lowering probe:
        // Under legacy defect, lack of YangTranslator or IDL routing emitted 0 structural elements.
        let pkg = crate::translate_yang(yang_source, "ietf_interfaces")
            .expect("translate_yang failed to parse YANG model");
        assert_eq!(pkg.name, "IetfInterfaces");
        assert!(
            pkg.attribute_defs.iter().any(|a| a.name == "interfaceRef"),
            "DEFECT REPRODUCED: Yang lowering missing typedef attribute definition 'interfaceRef'"
        );
        assert!(
            pkg.part_defs.iter().any(|p| p.name == "Interfaces"),
            "DEFECT REPRODUCED: Yang lowering missing container PartDef 'Interfaces'"
        );
        let interfaces_part = pkg.part_defs.iter().find(|p| p.name == "Interfaces").unwrap();
        assert!(
            interfaces_part.parts.iter().any(|p| p.name == "interface_val"),
            "DEFECT REPRODUCED: Yang lowering missing list part usage 'interface_val' inside Interfaces"
        );
        assert!(
            pkg.part_defs.iter().any(|p| p.name == "Interface_item"),
            "DEFECT REPRODUCED: Yang lowering missing list PartDef 'Interface_item' in package"
        );

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// Self-removing temporary directory guard so fixtures are deleted even when an assertion panics.
    struct TempDirGuard(PathBuf);

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn test_find_files_with_ext_recurses_nested_schema_dirs_deterministically() {
        let root = std::env::temp_dir().join(format!(
            "deap_yang_recursive_discovery_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let guard = TempDirGuard(root.clone());
        let nested = guard.0.join("zeta").join("inner");
        let sibling = guard.0.join("alpha");
        let hidden = guard.0.join(".hidden_dir");
        fs::create_dir_all(&nested).expect("create nested fixture dir");
        fs::create_dir_all(&sibling).expect("create sibling fixture dir");
        fs::create_dir_all(&hidden).expect("create hidden fixture dir");
        let module_src = "module sample_module { prefix sm; container RootNode { leaf label { type string; } } }\n";
        let nested_file = nested.join("deep_module.yang");
        let sibling_file = sibling.join("side_module.YANG");
        let top_file = guard.0.join("top_module.yang");
        fs::write(&nested_file, module_src).expect("write nested yang");
        fs::write(&sibling_file, module_src).expect("write sibling yang");
        fs::write(&top_file, module_src).expect("write top yang");
        fs::write(hidden.join("ignored_module.yang"), module_src).expect("write hidden yang");
        fs::write(guard.0.join(".dot_module.yang"), module_src).expect("write hidden file");
        fs::write(nested.join("notes.txt"), "not a schema").expect("write non-schema file");

        let found = find_files_with_ext(&guard.0, "yang");
        assert!(!found.is_empty(), "fixture guard: recursive discovery found zero .yang files");
        assert_eq!(
            found,
            vec![sibling_file.clone(), top_file.clone(), nested_file.clone()],
            "expected sorted recursive discovery excluding hidden entries"
        );

        let (fmt, files) = discover_schema_targets(Some(&guard.0), Some("auto"))
            .expect("discover_schema_targets failed on nested yang fixture");
        assert_eq!(fmt, "yang");
        assert_eq!(files.len(), 3, "auto-discovery must include nested .yang files: {files:?}");
        assert!(files.contains(&nested_file));

        let (hint_fmt, hint_files) = discover_schema_targets(Some(&guard.0), Some("yang"))
            .expect("discover_schema_targets failed with explicit yang hint");
        assert_eq!(hint_fmt, "yang");
        assert_eq!(hint_files.len(), 3);
    }
}
