//! # Heterogeneous Schema Format Detectors & Target Discovery Engine
//!
//! Deterministic schema format detection and filesystem target discovery for Level 0 OEM
//! specifications (Markdown, SysML v2, OMG IDL, AUTOSAR ARXML, Protobuf, OpenAPI, YANG).
//!
//! Realises: [REQ-0001]

use std::fs;
use std::path::{Path, PathBuf};

/// Raw uncompiled document extensions that require formal AST extraction prior to lowering. Realises: [REQ-0001/RAW_EXTENSIONS]
pub const RAW_EXTENSIONS: &[&str] = &[".pdf", ".txt", ".doc", ".docx"];

/// Detects the formal specification format from file extension and content. Realises: [REQ-0001/detect_format]
pub fn detect_format(schema_path: &str, content: &str) -> Result<String, String> {
    let path = Path::new(schema_path);
    let lower_name = path.file_name().and_then(|s| s.to_str()).unwrap_or(schema_path).to_ascii_lowercase();
    if lower_name == "digest.json"
        || lower_name.ends_with(".digest.json")
        || lower_name.ends_with("-digest.json")
        || lower_name.ends_with("_digest.json")
    {
        return Err(format!("Unsupported schema format for '{schema_path}': digest sidecar files are not ingestible schemas."));
    }

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| format!(".{}", s.to_ascii_lowercase()))
        .unwrap_or_default();

    const KEYWORDS: &[&str] = &["component", "part", "bom", "port", "signal", "parameter", "interface", "property", "attribute"];
    let has_table_header = content.lines().any(|line| {
        line.contains('|') && line.split('|').any(|cell| KEYWORDS.contains(&cell.trim().to_ascii_lowercase().as_str()))
    });

    let has_table_delim = content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .windows(2)
        .any(|w| {
            let (Some(w0), Some(w1)) = (w.first(), w.get(1)) else { return false };
            w0.contains('|') && w1.starts_with('|') && w1.trim_matches('|').contains('-')
                && w1.trim_matches('|').chars().all(|c| matches!(c, '-' | ':' | '|') || c.is_whitespace())
        });

    if ext == ".md" || has_table_header || has_table_delim {
        return Ok("markdown".to_string());
    }
    if RAW_EXTENSIONS.contains(&ext.as_str()) {
        return Ok("raw".to_string());
    }
    match ext.as_str() {
        ".sysml" => return Ok("sysml".to_string()),
        ".yang" => return Ok("yang".to_string()),
        ".idl" => return Ok("idl".to_string()),
        ".arxml" | ".xml" => return Ok("autosar".to_string()),
        ".proto" => return Ok("protobuf".to_string()),
        ".rs" | ".dart" => return Ok("code".to_string()),
        _ => {}
    }

    let has_openapi_markers = ["\"openapi\"", "openapi:", "\"swagger\"", "swagger:", "\"paths\":", "paths:"].iter().any(|m| content.contains(m));
    if matches!(ext.as_str(), ".json" | ".yaml" | ".yml") && (content.trim().is_empty() || has_openapi_markers) {
        return Ok("openapi".to_string());
    }
    for (markers, fmt) in [
        (&["part def ", "package ", "capability def ", "requirement def "][..], "sysml"),
        (&["yang-version", "namespace ", "prefix ", "typedef ", "grouping "][..], "yang"),
        (&["module ", "interface ", "struct "][..], "idl"),
        (&["<AUTOSAR", "<AR-PACKAGE"][..], "autosar"),
        (&["syntax =", "message "][..], "protobuf"),
    ] {
        if (fmt == "yang" && path.ends_with(".yang")) || markers.iter().any(|m| content.contains(m)) {
            return Ok(fmt.to_string());
        }
    }
    if has_openapi_markers {
        return Ok("openapi".to_string());
    }
    Err(format!("Unsupported schema format for '{schema_path}'. Supported formats: .sysml, .yang, .idl, .arxml/.xml, .proto, .json/.yaml/.yml, markdown (.md)."))
}

/// Identifies whether a given path is an output or intermediate SysML file rather than an input schema. Realises: [REQ-INGEST-SYSML-DETECTORS-IS-OUTPUT-SYSML]
fn is_output_sysml(p: &Path) -> bool {
    p.file_name().and_then(|s| s.to_str()).is_some_and(|name| {
        name == "model.sysml"
            || name == "schema.sysml"
            || (name.starts_with("blueprint-") && name.ends_with(".sysml"))
            || p.with_extension("md").is_file()
    })
}

/// Discovers markdown files under `search_dir/extracted` or `search_dir`, ignoring `README.md`. Realises: [REQ-INGEST-SYSML-DETECTORS-DISCOVER-MARKDOWN-FILES]
fn discover_markdown_files(search_dir: &Path) -> Vec<PathBuf> {
    let extracted_dir = if search_dir.ends_with("extracted") { search_dir.to_path_buf() } else { search_dir.join("extracted") };
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
    let blueprint_md: Vec<PathBuf> = root_md
        .iter()
        .filter(|p| p.file_name().and_then(|s| s.to_str()).is_some_and(|name| name.starts_with("blueprint-")))
        .cloned()
        .collect();
    if !blueprint_md.is_empty() { blueprint_md } else { root_md }
}

/// Discovers specification files or target directories adhering to deterministic search precedence. Realises: [REQ-0001/discover_schema_targets]
pub fn discover_schema_targets(path_opt: Option<&Path>, format_hint: Option<&str>) -> Result<(String, Vec<PathBuf>), String> {
    let normalized_hint = format_hint.map(|h| h.trim().to_ascii_lowercase());
    let is_explicit = normalized_hint.as_deref().is_some_and(|h| !h.is_empty() && h != "auto");

    if let Some(p) = path_opt {
        if p.is_file() {
            let fmt = if is_explicit {
                match normalized_hint.as_deref().unwrap_or_default() {
                    "markdown" | "md" => "markdown".to_string(),
                    "autosar" | "arxml" => "autosar".to_string(),
                    "protobuf" | "proto" => "protobuf".to_string(),
                    other => other.to_string(),
                }
            } else {
                detect_format(p.to_str().unwrap_or(""), &fs::read_to_string(p).unwrap_or_default())?
            };
            return Ok((fmt, vec![p.to_path_buf()]));
        }
        return discover_schema_targets_in_dir(p, normalized_hint.as_deref(), is_explicit);
    }

    let mut last_result = ("unknown".to_string(), Vec::new());
    for candidate in [Path::new("schema"), Path::new("docs/designs")] {
        let (fmt, files) = discover_schema_targets_in_dir(candidate, normalized_hint.as_deref(), is_explicit)?;
        if !files.is_empty() {
            return Ok((fmt, files));
        }
        last_result = (fmt, files);
    }
    Ok(last_result)
}

/// Scans a single target directory for specification files adhering to explicit format hints or auto-discovery precedence. Realises: [REQ-0001/discover_schema_targets]
fn discover_schema_targets_in_dir(search_dir: &Path, normalized_hint: Option<&str>, is_explicit: bool) -> Result<(String, Vec<PathBuf>), String> {
    if !search_dir.exists() {
        return Ok(("unknown".to_string(), Vec::new()));
    }
    if is_explicit {
        let canonical_fmt = match normalized_hint.unwrap_or_default() {
            "markdown" | "md" => return Ok(("markdown".to_string(), discover_markdown_files(search_dir))),
            "autosar" | "arxml" => "autosar",
            "protobuf" | "proto" => "protobuf",
            other => other,
        };
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
        let mut matching_files: Vec<PathBuf> = exts.iter().flat_map(|ext| find_files_with_ext(search_dir, ext)).filter(|p| p.file_name().and_then(|s| s.to_str()) != Some("README.md")).collect();
        if canonical_fmt == "sysml" {
            let (non_output, output): (Vec<_>, Vec<_>) = matching_files.into_iter().partition(|p| !is_output_sysml(p));
            matching_files = if !non_output.is_empty() { non_output } else { output };
        }
        matching_files.sort();
        return Ok((canonical_fmt.to_string(), matching_files));
    }

    let (mut non_output_sysml, mut output_sysml): (Vec<_>, Vec<_>) = find_files_with_ext(search_dir, "sysml").into_iter().partition(|p| !is_output_sysml(p));
    if !non_output_sysml.is_empty() {
        non_output_sysml.sort();
        return Ok(("sysml".to_string(), non_output_sysml));
    }
    let md_files = discover_markdown_files(search_dir);
    if !md_files.is_empty() {
        return Ok(("markdown".to_string(), md_files));
    }
    for (ext, fmt) in [("yang", "yang"), ("idl", "idl"), ("arxml", "autosar"), ("xml", "autosar"), ("proto", "protobuf"), ("yaml", "openapi"), ("json", "openapi")] {
        let mut other_files = find_files_with_ext(search_dir, ext);
        if !other_files.is_empty() {
            other_files.sort();
            return Ok((fmt.to_string(), other_files));
        }
    }
    if !output_sysml.is_empty() {
        output_sysml.sort();
        return Ok(("sysml".to_string(), output_sysml));
    }
    Ok(("unknown".to_string(), Vec::new()))
}

/// Recursively locates files matching a target extension under a directory. Realises: [REQ-0001/find_files_with_ext_recursive]
fn find_files_with_ext(dir: &Path, ext: &str) -> Vec<PathBuf> {
    const EXCLUDED_DIRS: &[&str] = &[
        "target", "node_modules", "build", "Pods", "vendor", "dist", "ephemeral", "generated",
        "epics", "features", "user-stories", "use-cases", "safety", "conops", "interfaces", "management",
    ];
    let mut out = Vec::new();
    let iter = walkdir::WalkDir::new(dir)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            e.depth() == 0 || {
                let n = e.file_name().to_string_lossy();
                !n.starts_with(['.', '#']) && (!e.file_type().is_dir() || !EXCLUDED_DIRS.contains(&n.as_ref()))
            }
        });
    for entry_res in iter {
        let Ok(entry) = entry_res else { continue };
        let p = entry.path();
        let low = entry.file_name().to_string_lossy().to_ascii_lowercase();
        let is_digest = low == "digest.json" || [".digest.json", "-digest.json", "_digest.json"].iter().any(|s| low.ends_with(s));
        if p.is_file() && !is_digest && p.extension().is_some_and(|e| e.to_string_lossy().eq_ignore_ascii_case(ext)) {
            out.push(p.to_path_buf());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDirGuard(PathBuf);
    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn get_schema_dir() -> PathBuf {
        for p in [Path::new("schema"), Path::new("../../schema")] {
            if p.exists() {
                return p.to_path_buf();
            }
        }
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schema")
    }

    #[test]
    fn test_detect_format_by_extension_and_content() {
        for (f, expected) in [
            ("schema.md", "markdown"), ("model.sysml", "sysml"), ("interface.yang", "yang"),
            ("interface.idl", "idl"), ("system.arxml", "autosar"), ("messages.proto", "protobuf"),
            ("api.json", "openapi"), ("api.yaml", "openapi"), ("doc.pdf", "raw"),
        ] {
            assert_eq!(detect_format(f, "").as_deref(), Ok(expected));
        }
        for (content, expected) in [
            ("package Package_0 { part def Classifier_Alpha; }", "sysml"),
            ("module ietf-geo-location { yang-version 1.1; namespace \"urn:ietf\"; prefix geo; }", "yang"),
            ("module Module_0 { struct Struct_1 { long id; }; };", "idl"),
            ("<AUTOSAR><AR-PACKAGE></AR-PACKAGE></AUTOSAR>", "autosar"),
            ("syntax = \"proto3\"; message Msg_0 { int32 val = 1; }", "protobuf"),
            ("{\"openapi\": \"3.0.0\", \"paths\": {}}", "openapi"),
            ("| Component | Part Number |\n| --- | --- |\n| Classifier_Alpha | 1 |", "markdown"),
        ] {
            assert_eq!(detect_format("unknown_file", content).as_deref(), Ok(expected));
        }
    }

    #[test]
    fn test_discover_schema_targets_existing_workspace() -> Result<(), String> {
        let schema_dir = get_schema_dir();
        let path = schema_dir.join("REQ-0001.md");
        if path.exists() {
            let (fmt, files) = discover_schema_targets(Some(&path), None)?;
            assert_eq!(fmt, "markdown");
            assert_eq!(files, vec![path]);
            let (m_fmt, m_files) = discover_schema_targets(Some(&schema_dir), Some("markdown"))?;
            assert_eq!(m_fmt, "markdown");
            assert!(!m_files.contains(&schema_dir.join("model.sysml")));
            let (a_fmt, a_files) = discover_schema_targets(Some(&schema_dir), Some("auto"))?;
            assert_eq!(a_fmt, "markdown");
            assert!(!a_files.contains(&schema_dir.join("model.sysml")));
        }
        if schema_dir.join("model.sysml").exists() {
            let (fmt, files) = discover_schema_targets(Some(&schema_dir), Some("sysml"))?;
            assert_eq!(fmt, "sysml");
            assert!(!files.is_empty());
        }
        Ok(())
    }

    #[test]
    fn test_yang_autodiscovery_and_recursive_discovery() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = std::env::temp_dir().join(format!("test_yang_probe_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        let guard = TempDirGuard(temp_dir);
        fs::create_dir_all(&guard.0)?;
        let yang_source = "module ietf_interfaces { yang-version 1.1; namespace \"urn:ietf:params:xml:ns:yang:ietf-interfaces\"; prefix \"if\"; typedef interface-ref { type string; } container interfaces { list interface { key \"name\"; leaf name { type string; } leaf enabled { type boolean; default \"true\"; } } } }";
        let yang_file = guard.0.join("ietf_interfaces.yang");
        fs::write(&yang_file, yang_source)?;
        assert_eq!(detect_format("schema_without_ext", yang_source)?, "yang");
        let (disc_fmt, files) = discover_schema_targets(Some(&guard.0), Some("auto"))?;
        assert_eq!((disc_fmt.as_str(), files.as_slice()), ("yang", std::slice::from_ref(&yang_file)));
        let pkg = crate::translate_yang(yang_source, "ietf_interfaces")?;
        assert_eq!(pkg.name, "IetfInterfaces");
        assert!(pkg.attribute_defs.iter().any(|a| a.name == "interfaceRef") && pkg.part_defs.iter().any(|p| p.name == "Interfaces"));
        Ok(())
    }

    #[test]
    fn test_find_files_with_ext_exclusions_and_sidecars() -> Result<(), Box<dyn std::error::Error>> {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos();
        let guard = TempDirGuard(std::env::temp_dir().join(format!(".deap_detectors_test_{}_{nanos}", std::process::id())));
        let designs_dir = guard.0.join("docs").join("designs");
        let features_dir = guard.0.join("docs").join("features");
        let target_dir = guard.0.join("target").join("doc");
        fs::create_dir_all(&designs_dir)?;
        fs::create_dir_all(&features_dir)?;
        fs::create_dir_all(&target_dir)?;

        let bp1_md = designs_dir.join("blueprint-01-lint.md");
        let bp2_md = designs_dir.join("blueprint-02-purity.md");
        fs::write(&bp1_md, "# Blueprint 01\n")?;
        fs::write(&bp2_md, "# Blueprint 02\n")?;
        fs::write(designs_dir.join("blueprint-01-lint.sysml"), "package StaleSidecar {}\n")?;
        fs::write(designs_dir.join("feat-41-solution.md"), "# Legacy Solution\n")?;
        fs::write(features_dir.join("FEAT-001.md"), "# Generated Feature\n")?;
        fs::write(target_dir.join("cached.md"), "# Cached Doc\n")?;

        let (auto_fmt, auto_files) = discover_schema_targets(Some(&designs_dir), Some("auto"))?;
        assert_eq!((auto_fmt.as_str(), auto_files.as_slice()), ("markdown", &[bp1_md.clone(), bp2_md.clone()][..]));
        assert!(is_output_sysml(Path::new("docs/designs/blueprint-01-lint.sysml")));
        assert!(!is_output_sysml(Path::new("schema/domain.sysml")));

        assert!(detect_format("docs/designs/blueprint-01.digest.json", r#"{"sha256":"abc"}"#).is_err());
        assert!(detect_format("schema-digest.json", r#"{"sha256":"abc"}"#).is_err());
        assert!(detect_format("arbitrary.json", r#"{"sha256":"abc","not_openapi":true}"#).is_err());

        let json_dir = guard.0.join("json_test");
        fs::create_dir_all(&json_dir)?;
        fs::write(json_dir.join("blueprint-01.digest.json"), r#"{"sha256":"def"}"#)?;
        let openapi_json = json_dir.join("openapi.json");
        fs::write(&openapi_json, r#"{"openapi":"3.0.0","paths":{}}"#)?;
        assert_eq!(find_files_with_ext(&json_dir, "json"), vec![openapi_json]);
        Ok(())
    }

    #[test]
    fn test_discover_schema_targets_falls_back_to_docs_designs_when_schema_empty() -> Result<(), Box<dyn std::error::Error>> {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos();
        let guard = TempDirGuard(std::env::temp_dir().join(format!(".deap_fallback_designs_{}_{nanos}", std::process::id())));
        let (schema_dir, designs_dir) = (guard.0.join("schema"), guard.0.join("docs").join("designs"));
        fs::create_dir_all(&schema_dir)?;
        fs::create_dir_all(&designs_dir)?;
        fs::write(schema_dir.join(".gitkeep"), "")?;
        fs::write(designs_dir.join("blueprint-01-core.md"), "# Blueprint 01 Core\n")?;

        let orig_cwd = std::env::current_dir()?;
        std::env::set_current_dir(&guard.0)?;
        let res_fallback = discover_schema_targets(None, Some("auto"));
        fs::write(schema_dir.join("REQ-0001.md"), "# Requirement 0001\n## 1. Normative Statement\n")?;
        let res_priority = discover_schema_targets(None, Some("auto"));
        std::env::set_current_dir(&orig_cwd)?;

        assert_eq!(res_fallback, Ok(("markdown".to_string(), vec![PathBuf::from("docs/designs/blueprint-01-core.md")])));
        assert_eq!(res_priority, Ok(("markdown".to_string(), vec![PathBuf::from("schema/REQ-0001.md")])));
        Ok(())
    }
}
