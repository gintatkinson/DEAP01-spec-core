//! # SysML v2 Universal Ingestion Engine CLI Entrypoint
//!
//! Discovers, classifies, translates, and serializes heterogeneous Level 0 OEM technical
//! specifications into canonical SysML v2 textual models (`.pipeline/schema.sysml`) and
//! cryptographic schema digests (`.pipeline/schema-digest.json`).
//!
//! Realises: [REQ-SYSML-INGEST-CLI], [DO-330-TQL-4-INGEST-CLI]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use compile_sysml::{
    generate_digest, parse_sysml, parse_sysml_tree, to_sysml, write_atomic, write_digest_atomic,
};
use deap_core::source_ast::{
    resolve_source_package_name, sanitize_sysml_identifier, SourceLanguage,
};
use deap_core::PackageDef;
use ingest_sysml::cst::{
    CodeTranslator, IDLTranslator, MarkdownTranslator, OpenAPITranslator, YangTranslator,
};
use ingest_sysml::detectors::discover_schema_targets;
use ingest_sysml::{
    cache_ingested_sysml_model, collect_declared_part_names, filter_ast_to_realized_or_allowed,
    filter_ast_to_target_scope, verify_scope_and_negative_invariants, IngestExitCode,
};

/// Ground 0.0 SysML v2 Universal Ingestion Engine Command-Line Arguments. Realises: [REQ-SYSML-INGEST-CLI/CliArgs], [DO-330-TQL-4-INGEST-CLI]
#[derive(Debug)]
pub struct CliArgs {
    pub source: Option<PathBuf>,
    pub schema: Option<PathBuf>,
    pub package: Option<String>,
    pub format: String,
    pub language: String,
    pub out: PathBuf,
    pub digest: PathBuf,
    pub allowed_parts: Option<String>,
    pub negative_invariants: Option<String>,
    pub cache_ast: bool,
}

impl CliArgs {
    /// Builds the `clap::Command` specification for `ingest-sysml`. Realises: [REQ-SYSML-INGEST-CLI/CliArgs]
    fn build_command() -> clap::Command {
        use clap::{Arg, ArgAction, Command};
        Command::new("ingest-sysml")
            .version("0.1.0")
            .about("SysML v2 Universal Ingestion Engine")
            .arg(Arg::new("source").long("source").value_parser(clap::value_parser!(PathBuf)))
            .arg(Arg::new("schema").short('s').long("schema").value_parser(clap::value_parser!(PathBuf)))
            .arg(Arg::new("package").long("package"))
            .arg(Arg::new("format").short('f').long("format").default_value("auto"))
            .arg(Arg::new("language").long("language").default_value("auto"))
            .arg(
                Arg::new("out")
                    .short('o')
                    .long("out")
                    .default_value(".pipeline/schema.sysml")
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("digest")
                    .short('d')
                    .long("digest")
                    .default_value(".pipeline/schema-digest.json")
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(Arg::new("allowed_parts").long("allowed-parts"))
            .arg(Arg::new("negative_invariants").long("negative-invariants"))
            .arg(Arg::new("cache_ast").long("cache-ast").action(ArgAction::SetTrue))
    }

    /// Extracts a [`CliArgs`] configuration struct from parsed `clap::ArgMatches`. Realises: [REQ-SYSML-INGEST-CLI/CliArgs]
    fn from_matches(mut matches: clap::ArgMatches) -> Self {
        Self {
            source: matches.remove_one::<PathBuf>("source"),
            schema: matches.remove_one::<PathBuf>("schema"),
            package: matches.remove_one::<String>("package"),
            format: matches.remove_one::<String>("format").unwrap_or_else(|| "auto".to_string()),
            language: matches.remove_one::<String>("language").unwrap_or_else(|| "auto".to_string()),
            out: matches.remove_one::<PathBuf>("out").unwrap_or_else(|| PathBuf::from(".pipeline/schema.sysml")),
            digest: matches.remove_one::<PathBuf>("digest").unwrap_or_else(|| PathBuf::from(".pipeline/schema-digest.json")),
            allowed_parts: matches.remove_one::<String>("allowed_parts"),
            negative_invariants: matches.remove_one::<String>("negative_invariants"),
            cache_ast: matches.get_flag("cache_ast"),
        }
    }

    /// Parses process command-line arguments into [`CliArgs`]. Realises: [REQ-SYSML-INGEST-CLI/CliArgs]
    pub fn parse() -> Self {
        Self::from_matches(Self::build_command().get_matches())
    }

    /// Parses an explicit iterator of argument strings into [`CliArgs`]. Realises: [REQ-SYSML-INGEST-CLI/CliArgs]
    pub fn parse_from<I, T>(itr: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Self::from_matches(Self::build_command().get_matches_from(itr))
    }
}

/// Derives a deterministic SysML v2 package identifier for an ingested source directory or explicit override. Realises: [REQ-SYSML-INGEST-CLI/resolve_ingested_package_name], [REQ-ISSUE-472-PACKAGE-QUALIFICATION]
pub fn resolve_ingested_package_name(source_path: &Path, explicit_package: Option<&str>) -> String {
    resolve_source_package_name(source_path, explicit_package)
}

/// Binary entrypoint for the `ingest-sysml` command-line executable. Realises: [REQ-SYSML-INGEST-CLI/main], [DO-330-TQL-4-INGEST-CLI]
fn main() {
    let args = CliArgs::parse();
    if let Err((exit_code, e)) = run_ingestion_with_code(args) {
        eprintln!("[SysML v2 Ingestion] ERROR: {e}");
        process::exit(exit_code.code());
    }
}

/// Executes the end-to-end ingestion pipeline with validated input arguments. Realises: [REQ-SYSML-INGEST-CLI/run_ingestion], [DO-330-TQL-4-INGEST-CLI]
pub fn run_ingestion(args: CliArgs) -> Result<(), String> {
    run_ingestion_with_code(args).map_err(|(_, msg)| msg)
}

/// Loads a reference schema package from a directory or file for realization scoping. Realises: [REQ-ISSUE-472-SCHEMA-SCOPED-CODE-INGESTION]
fn load_reference_schema_parts(schema_path: &Path) -> Result<BTreeSet<String>, (IngestExitCode, String)> {
    let ref_pkg = if schema_path.is_dir() {
        let (_, sysml_files) = discover_schema_targets(Some(schema_path), Some("sysml"))
            .map_err(|e| (IngestExitCode::ParseOrIoError, e))?;
        parse_sysml_tree(&sysml_files).map_err(|e| {
            (IngestExitCode::ParseOrIoError, format!("Failed to parse reference schema directory '{}': {e}", schema_path.display()))
        })?
    } else {
        let schema_content = fs::read_to_string(schema_path).map_err(|e| {
            (IngestExitCode::ParseOrIoError, format!("Failed to read reference schema '{}': {e}", schema_path.display()))
        })?;
        parse_sysml(&schema_content).map_err(|e| {
            (IngestExitCode::ParseOrIoError, format!("Failed to parse reference schema '{}': {e}", schema_path.display()))
        })?
    };
    let mut schema_parts = BTreeSet::new();
    collect_declared_part_names(&ref_pkg, &mut schema_parts);
    Ok(schema_parts)
}

/// Translates discovered schema files according to the resolved format. Realises: [REQ-SYSML-INGEST-CLI/run_ingestion_with_code]
fn translate_schema_targets(args: &CliArgs) -> Result<PackageDef, (IngestExitCode, String)> {
    let (disc_fmt, disc_files) = discover_schema_targets(args.schema.as_deref(), Some(&args.format))
        .map_err(|e| (IngestExitCode::ParseOrIoError, e))?;
    if disc_files.is_empty() {
        let target_desc = args
            .schema
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "schema/ or schema/extracted/".to_string());
        return Err((IngestExitCode::ParseOrIoError, format!("No supported schema files found in {target_desc}")));
    }

    let fmt = if args.format == "auto" { disc_fmt } else { args.format.to_ascii_lowercase() };
    let is_dir = args.schema.as_ref().is_some_and(|p| p.is_dir());

    if fmt == "markdown" && (disc_files.len() > 1 || is_dir) {
        let all_blueprints = !disc_files.is_empty()
            && disc_files.iter().all(|p| p.file_name().and_then(|s| s.to_str()).is_some_and(|n| n.starts_with("blueprint-")));
        let def_name = if all_blueprints {
            "SysML_Model"
        } else if is_dir {
            args.schema.as_ref().and_then(|p| p.file_name()).and_then(|s| s.to_str()).unwrap_or("OEM_System_Model")
        } else {
            "OEM_System_Model"
        };
        let file_strs: Vec<String> = disc_files.iter().map(|p| p.to_string_lossy().to_string()).collect();
        let pkg = MarkdownTranslator::new().translate_files(&file_strs, def_name).map_err(|e| (IngestExitCode::ParseOrIoError, e))?;
        println!("[SysML v2 Ingestion] Successfully ingested {} markdown files -> {}", disc_files.len(), args.out.display());
        Ok(pkg)
    } else if fmt == "yang" && (disc_files.len() > 1 || is_dir) {
        let def_name = if is_dir {
            args.schema.as_ref().and_then(|p| p.file_name()).and_then(|s| s.to_str()).unwrap_or("YangModel")
        } else {
            "YangModel"
        };
        let file_strs: Vec<String> = disc_files.iter().map(|p| p.to_string_lossy().to_string()).collect();
        let pkg = YangTranslator::new().translate_files(&file_strs, def_name).map_err(|e| (IngestExitCode::ParseOrIoError, e))?;
        println!("[SysML v2 Ingestion] Successfully ingested {} yang files -> {}", disc_files.len(), args.out.display());
        Ok(pkg)
    } else {
        let target_file = disc_files.first().ok_or_else(|| {
            (IngestExitCode::ParseOrIoError, "No schema file available for single-file ingestion".to_string())
        })?;
        let content = fs::read_to_string(target_file).map_err(|e| {
            (IngestExitCode::ParseOrIoError, format!("Failed to read file '{}': {e}", target_file.display()))
        })?;
        let basename = target_file.file_stem().and_then(|s| s.to_str()).unwrap_or("Model");
        let lang = SourceLanguage::from_flag(&args.language).map_err(|e| (IngestExitCode::ParseOrIoError, e.into_string()))?;
        let pkg = match fmt.as_str() {
            "sysml" => parse_sysml(&content).map_err(|e| (IngestExitCode::ParseOrIoError, format!("SysML v2 parse error in '{}': {e}", target_file.display())))?,
            "code" => CodeTranslator::with_language(lang).translate(&content, basename).map_err(|e| (IngestExitCode::ParseOrIoError, e))?,
            "idl" => IDLTranslator::new().translate(&content, basename).map_err(|e| (IngestExitCode::ParseOrIoError, e))?,
            "openapi" => OpenAPITranslator::new().translate(&content, basename).map_err(|e| (IngestExitCode::ParseOrIoError, e))?,
            "markdown" => MarkdownTranslator::new().translate(&content, basename).map_err(|e| (IngestExitCode::ParseOrIoError, e))?,
            "yang" => YangTranslator::new().translate(&content, basename).map_err(|e| (IngestExitCode::ParseOrIoError, e))?,
            "raw" => return Err((IngestExitCode::ParseOrIoError, "AST translation is required for raw document formats. SSOT ingestion failed.".to_string())),
            other => return Err((IngestExitCode::ParseOrIoError, format!("Unsupported format '{other}' for file '{}'", target_file.display()))),
        };
        println!("[SysML v2 Ingestion] Successfully ingested '{}' -> {}", target_file.display(), args.out.display());
        Ok(pkg)
    }
}

/// Executes the end-to-end ingestion pipeline and categorizes failures by [`IngestExitCode`]. Realises: [REQ-SYSML-INGEST-CLI/run_ingestion_with_code], [DO-330-TQL-4-INGEST-CLI]
pub fn run_ingestion_with_code(args: CliArgs) -> Result<(), (IngestExitCode, String)> {
    let mut pkg = if let Some(ref source_path) = args.source {
        let fmt_lower = args.format.trim().to_ascii_lowercase();
        if fmt_lower != "auto" && fmt_lower != "code" {
            return Err((IngestExitCode::ParseOrIoError, format!("Invalid format '{}' when '--source' is specified. Expected 'auto' or 'code'.", args.format)));
        }
        let lang = SourceLanguage::from_flag(&args.language).map_err(|e| (IngestExitCode::ParseOrIoError, e.into_string()))?;
        let pkg_name = resolve_ingested_package_name(source_path, args.package.as_deref());
        let mut extracted = CodeTranslator::with_language(lang).translate_directory(source_path, &pkg_name).map_err(|e| (IngestExitCode::ParseOrIoError, e))?;
        if let (Some(schema_path), None) = (&args.schema, &args.allowed_parts) {
            let schema_parts = load_reference_schema_parts(schema_path)?;
            filter_ast_to_realized_or_allowed(&mut extracted, &schema_parts);
        }
        println!("[SysML v2 Ingestion] Successfully ingested source AST from '{}' -> {}", source_path.display(), args.out.display());
        extracted
    } else if args.format.eq_ignore_ascii_case("code") {
        let target_path = args.schema.clone().unwrap_or_else(|| PathBuf::from("src"));
        let lang = SourceLanguage::from_flag(&args.language).map_err(|e| (IngestExitCode::ParseOrIoError, e.into_string()))?;
        let pkg_name = resolve_ingested_package_name(&target_path, args.package.as_deref());
        let extracted = CodeTranslator::with_language(lang).translate_directory(&target_path, &pkg_name).map_err(|e| (IngestExitCode::ParseOrIoError, e))?;
        println!("[SysML v2 Ingestion] Successfully ingested source AST from '{}' -> {}", target_path.display(), args.out.display());
        extracted
    } else {
        translate_schema_targets(&args)?
    };

    if let Some(explicit_pkg) = args.package.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        pkg.name = sanitize_sysml_identifier(explicit_pkg, &pkg.name);
    }

    let allowed_set: Option<BTreeSet<String>> = args.allowed_parts.as_ref().map(|s| s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect());
    let neg_vec: Option<Vec<String>> = args.negative_invariants.as_ref().map(|s| s.split(',').map(|n| n.trim().to_string()).filter(|n| !n.is_empty()).collect());

    filter_ast_to_target_scope(&mut pkg, allowed_set.as_ref(), neg_vec.as_deref());
    verify_scope_and_negative_invariants(&pkg, allowed_set.as_ref(), neg_vec.as_deref())
        .map_err(|e| (IngestExitCode::ScopeOrCryptoError, e))?;

    persist_and_verify_ingested_model(&args, &pkg)
}

/// Writes the SysML model, downstream fallback model, and cryptographic digest, then verifies SHA-256 parity. Realises: [REQ-SYSML-INGEST-CLI/run_ingestion_with_code]
fn persist_and_verify_ingested_model(args: &CliArgs, pkg: &PackageDef) -> Result<(), (IngestExitCode, String)> {
    let sysml_text = to_sysml(pkg);
    write_atomic(&args.out, &sysml_text).map_err(|e| {
        (IngestExitCode::ParseOrIoError, format!("Failed to write output to '{}': {e}", args.out.display()))
    })?;

    if args.out.ends_with(".pipeline/schema.sysml") {
        let ws_root = args.out.parent().and_then(|p| p.parent()).filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
        let upstream_dir = ws_root.join(".pipeline").join("upstream");
        let schema_dir = ws_root.join("schema");
        if !upstream_dir.exists() && schema_dir.is_dir() {
            let mut has_sysml = false;
            if let Ok(entries) = fs::read_dir(&schema_dir) {
                for entry_res in entries {
                    let Ok(e) = entry_res else { continue };
                    if e.path().is_file() && e.path().extension().is_some_and(|ext| ext == "sysml") {
                        has_sysml = true;
                        break;
                    }
                }
            }
            if !has_sysml {
                let model_out = schema_dir.join("model.sysml");
                write_atomic(&model_out, &sysml_text).map_err(|e| {
                    (IngestExitCode::ParseOrIoError, format!("Failed to write downstream schema model to '{}': {e}", model_out.display()))
                })?;
            }
        }
    }

    let digest = generate_digest(pkg, &sysml_text);
    write_digest_atomic(&args.digest, &digest).map_err(|e| {
        (IngestExitCode::ParseOrIoError, format!("Failed to write digest to '{}': {e}", args.digest.display()))
    })?;

    let persisted_sysml = fs::read_to_string(&args.out).map_err(|e| {
        (IngestExitCode::ScopeOrCryptoError, format!("Post-write cryptographic verification failed reading '{}': {e}", args.out.display()))
    })?;
    let persisted_digest_json = fs::read_to_string(&args.digest).map_err(|e| {
        (IngestExitCode::ScopeOrCryptoError, format!("Post-write cryptographic verification failed reading '{}': {e}", args.digest.display()))
    })?;
    let computed_sha = ingest_sysml::digest::compute_sha256(persisted_sysml.as_bytes());
    if computed_sha != digest.sha256 || !persisted_digest_json.contains(&digest.sha256) {
        return Err((
            IngestExitCode::ScopeOrCryptoError,
            format!("Post-write cryptographic SHA-256 mismatch for '{}' (expected {}, computed {computed_sha})", args.out.display(), digest.sha256),
        ));
    }

    println!("[SysML v2 Ingestion] Schema digest generated at {}", args.digest.display());

    if args.cache_ast {
        let (_cached_model, cache_hit) = cache_ingested_sysml_model(Path::new("."), &sysml_text)
            .map_err(|e| (IngestExitCode::ParseOrIoError, e))?;
        println!("[SysML v2 Ingestion] Incremental AST cache updated (cache_hit={cache_hit})");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_cache_ast_flag() {
        let parsed = CliArgs::parse_from(vec!["ingest-sysml", "--cache-ast"]);
        assert!(parsed.cache_ast);
    }

    #[test]
    fn test_downstream_populates_schema_dir() -> Result<(), Box<dyn std::error::Error>> {
        let temp_ws = std::env::temp_dir().join(format!(".deap_test_downstream_ingest_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_ws);
        let (schema_dir, pipeline_dir) = (temp_ws.join("schema"), temp_ws.join(".pipeline"));
        fs::create_dir_all(&schema_dir)?;
        fs::create_dir_all(&pipeline_dir)?;
        let input_md = schema_dir.join("spec.md");
        fs::write(&input_md, "# SubsystemSpec\n\n### FlightActuator\n\n| Attribute | Type |\n| --- | --- |\n| torque | Real |\n")?;

        let make_args = || CliArgs {
            source: None,
            schema: Some(input_md.clone()),
            package: Some("DownstreamPkg".to_string()),
            format: "markdown".to_string(),
            language: "auto".to_string(),
            out: pipeline_dir.join("schema.sysml"),
            digest: pipeline_dir.join("schema.digest.json"),
            allowed_parts: None,
            negative_invariants: None,
            cache_ast: false,
        };
        let res = run_ingestion_with_code(make_args());
        assert!(res.is_ok(), "downstream ingestion failed: {:?}", res.err());
        assert!(schema_dir.join("model.sysml").is_file());

        fs::remove_file(schema_dir.join("model.sysml"))?;
        fs::create_dir_all(pipeline_dir.join("upstream"))?;
        let res_upstream = run_ingestion_with_code(make_args());
        let _ = fs::remove_dir_all(&temp_ws);
        assert!(res_upstream.is_ok());
        assert!(!schema_dir.join("model.sysml").exists());
        Ok(())
    }
}
