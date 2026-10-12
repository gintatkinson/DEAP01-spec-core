//! CLI entrypoint for compile-sysml compiler and specification synchronization engine.

use clap::Parser;
use compile_sysml::semantic::serializer::{to_sysml, SysmlSerializable};
use compile_sysml::semantic::validator::SemanticValidator;
use compile_sysml::stpa::compile_stpa_to_constraints;
use compile_sysml::sync::{
    forward_sync_sysml_to_specs, reverse_sync_specs_to_sysml, ForwardSyncOptions,
    ReverseSyncOptions,
};
use compile_sysml::{
    compute_file_manifest, count_structural_elements, discover_sysml_files, parse_sysml,
    parse_sysml_tree, SchemaDigest,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

const SCHEMA_REMEDIATION_MESSAGE: &str = "\
Error: No .sysml schema file found in schema/.
If starting from unstructured OEM prose manuals, PDF documentation, or BOM markdown tables:
  1. Place your OEM documentation or extract tables into schema/ or schema/extracted/.
  2. Execute Step 0.0 Level 0 OEM Ground Truth Ingestion:
     ./target/release/ingest-sysml --schema <path_to_markdown> --format markdown --out schema/model.sysml
  3. Re-run ./target/release/compile-sysml --compile to satisfy the compilation gate.";

#[derive(Parser, Debug)]
#[command(name = "compile-sysml")]
#[command(about = "SysML v2 / KerML compiler and specification synchronization engine")]
struct Cli {
    /// Path to input .sysml file
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Path to schema file or directory (default searches schema/*.sysml)
    #[arg(long, value_name = "SCHEMA_FILE")]
    schema: Option<PathBuf>,

    /// Run the Pipeline 0 compilation gate: parse, validate semantics, serialize, and emit digest
    #[arg(long)]
    compile: bool,

    /// Transpile SysML AST into the 10-pillar STPA safety artifact suite
    #[arg(long)]
    stpa_transpile: bool,

    /// Path to STPA markdown file to compile into SysML constraint notation
    #[arg(long, value_name = "STPA_FILE")]
    stpa: Option<PathBuf>,

    /// Execute reverse synchronization from markdown specs into SysML SSOT
    #[arg(long)]
    reverse_sync: bool,

    /// Execute forward synchronization from SysML SSOT into markdown specs
    #[arg(long)]
    forward_sync: bool,

    /// Path to documentation directory
    #[arg(long, default_value = "docs/")]
    docs: PathBuf,

    /// Path to output schema model file
    #[arg(long, default_value = ".pipeline/schema.sysml")]
    out: PathBuf,

    /// Path to explicit output directory (for --stpa-transpile or --forward-sync)
    #[arg(long, value_name = "OUT_DIR")]
    out_dir: Option<PathBuf>,

    /// Path to output schema digest JSON
    #[arg(long, default_value = ".pipeline/schema-digest.json")]
    digest: PathBuf,

    /// Force overwrite of existing files / bypass landing zone guard
    #[arg(long)]
    force: bool,

    /// Simulate synchronization without writing changes to disk
    #[arg(long)]
    dry_run: bool,

    /// Path to optional FMECA scoring configuration JSON
    #[arg(long, value_name = "SCORING_CONFIG")]
    scoring_config: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    // 1. STPA Transpilation: transpile schema AST into 10-pillar safety suite
    if cli.stpa_transpile {
        let target = match resolve_schema_targets(cli.schema.or(cli.file)) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        };

        let out_dir = cli.out_dir.unwrap_or_else(|| PathBuf::from("docs/safety"));
        let scoring_config = if let Some(cfg_path) = cli.scoring_config.as_deref() {
            if !cfg_path.exists() {
                eprintln!("Error: FMECA scoring config does not exist: {}", cfg_path.display());
                process::exit(1);
            }
            let cfg_text = match fs::read_to_string(cfg_path) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Failed to read scoring config: {}", e);
                    process::exit(1);
                }
            };
            match serde_json::from_str(&cfg_text) {
                Ok(cfg) => Some(cfg),
                Err(e) => {
                    eprintln!("Failed to parse scoring config JSON: {}", e);
                    process::exit(1);
                }
            }
        } else {
            None
        };

        let pkg = match target {
            SchemaTarget::SingleFile(schema_path) => {
                let content = match fs::read_to_string(&schema_path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error reading schema '{}': {}", schema_path.display(), e);
                        process::exit(1);
                    }
                };
                match parse_sysml(&content) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Error parsing schema '{}': {}", schema_path.display(), e);
                        process::exit(1);
                    }
                }
            }
            SchemaTarget::Directory(dir, files) => {
                match parse_sysml_tree(&files) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Error parsing schema directory '{}': {}", dir.display(), e);
                        process::exit(1);
                    }
                }
            }
        };

        if let Err(e) = compile_sysml::stpa::emit_safety_suite(&pkg, &out_dir, scoring_config.as_ref()) {
            eprintln!("Error during STPA transpilation: {}", e);
            process::exit(1);
        }
        println!("[STPA Transpile] Emitted safety artifact suite to '{}'", out_dir.display());
        process::exit(0);
    }

    // 2. STPA to Constraints compilation
    if let Some(ref stpa_file) = cli.stpa {
        if !stpa_file.exists() {
            eprintln!("Error: STPA file does not exist: {}", stpa_file.display());
            process::exit(1);
        }
        let content = match fs::read_to_string(stpa_file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading STPA file '{}': {}", stpa_file.display(), e);
                process::exit(1);
            }
        };

        let pkg = compile_stpa_to_constraints(&content, "System_SafetyConstraints");
        let sysml_code = to_sysml(&pkg);

        if cli.out != PathBuf::from(".pipeline/schema.sysml") {
            if let Some(parent) = cli.out.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Err(e) = fs::write(&cli.out, &sysml_code) {
                eprintln!("Error writing SysML to '{}': {}", cli.out.display(), e);
                process::exit(1);
            }
            println!(
                "Successfully compiled STPA constraints to '{}'",
                cli.out.display()
            );
        } else {
            println!("{}", sysml_code);
        }
        process::exit(0);
    }

    // 3. Forward Synchronization
    if cli.forward_sync {
        let target = match resolve_schema_targets(cli.schema.or(cli.file)) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        };

        let (pkg, schema_path) = match target {
            SchemaTarget::SingleFile(p) => {
                let content = match fs::read_to_string(&p) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error reading schema '{}': {}", p.display(), e);
                        process::exit(1);
                    }
                };

                let parsed = match parse_sysml(&content) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Error parsing schema '{}': {}", p.display(), e);
                        process::exit(1);
                    }
                };
                (parsed, p)
            }
            SchemaTarget::Directory(dir, files) => {
                let parsed = match parse_sysml_tree(&files) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Error parsing schema directory '{}': {}", dir.display(), e);
                        process::exit(1);
                    }
                };
                (parsed, dir)
            }
        };

        let opts = ForwardSyncOptions {
            schema_path,
            docs_dir: cli.docs,
            out_dir: cli.out_dir,
            dry_run: cli.dry_run,
            force: cli.force,
        };

        if let Err(e) = forward_sync_sysml_to_specs(&pkg, &opts) {
            eprintln!("Error during forward sync: {}", e);
            process::exit(1);
        }
        process::exit(0);
    }

    // 4. Reverse Synchronization
    if cli.reverse_sync {
        let base_pkg = match resolve_schema_targets(cli.schema.clone().or_else(|| cli.file.clone())) {
            Ok(SchemaTarget::SingleFile(path)) => {
                let content = fs::read_to_string(&path).unwrap_or_default();
                parse_sysml(&content).ok()
            }
            Ok(SchemaTarget::Directory(_, files)) => {
                parse_sysml_tree(&files).ok()
            }
            Err(_) => None,
        };

        let opts = ReverseSyncOptions {
            docs_dir: cli.docs,
            schema_path: cli.schema.or(cli.file),
            output_path: cli.out,
            digest_path: cli.digest,
            allow_overwrite: cli.force,
        };

        if let Err(e) = reverse_sync_specs_to_sysml(base_pkg.as_ref(), &opts) {
            eprintln!("Error during reverse sync: {}", e);
            process::exit(1);
        }
        process::exit(0);
    }

    // 5. Standard Pipeline 0 compilation gate
    if cli.compile || (cli.file.is_some() && !cli.reverse_sync && !cli.forward_sync) {
        let code = run_compilation_gate(cli.file.or(cli.schema), &cli.out, &cli.digest);
        process::exit(code);
    }

    println!("compile-sysml: Specify --compile, --stpa-transpile, --stpa, --forward-sync, or --reverse-sync. Run with --help for usage.");
}

/// Target SysML schema input resolved from CLI flags or default paths.
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaTarget {
    /// A single concrete .sysml file path.
    SingleFile(PathBuf),
    /// A directory containing multiple discovered .sysml files.
    Directory(PathBuf, Vec<PathBuf>),
}

/// Execute the Pipeline 0 compilation gate.
pub fn run_compilation_gate(
    explicit_schema: Option<PathBuf>,
    output_path: &Path,
    digest_path: &Path,
) -> i32 {
    let target = match resolve_schema_targets(explicit_schema) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{}", msg);
            return 1;
        }
    };

    let (pkg, display_name) = match &target {
        SchemaTarget::SingleFile(schema_file) => {
            let content = match fs::read_to_string(schema_file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error reading schema file '{}': {}", schema_file.display(), e);
                    return 1;
                }
            };

            if content.trim().is_empty() {
                eprintln!("Error: Schema file '{}' is empty.", schema_file.display());
                return 1;
            }

            let default_name = schema_file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("SysML_Model");

            let parsed = match parse_sysml(&content) {
                Ok(p) => p,
                Err(err) => {
                    eprintln!("Error parsing schema file '{}': {}", schema_file.display(), err);
                    return 1;
                }
            };

            (parsed, default_name.to_string())
        }
        SchemaTarget::Directory(dir, files) => {
            let dir_name = dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("SysML_Model")
                .to_string();

            let parsed = match parse_sysml_tree(files) {
                Ok(p) => p,
                Err(err) => {
                    eprintln!("Error parsing schema directory '{}': {}", dir.display(), err);
                    return 1;
                }
            };

            (parsed, dir_name)
        }
    };

    // Check structural elements
    let total_elements = count_structural_elements(&pkg);

    if total_elements == 0 {
        eprintln!(
            "Error: Schema '{}' contains 0 structural elements.",
            display_name
        );
        return 1;
    }

    // Semantic validation
    let mut validator = SemanticValidator::new(&pkg);
    validator.validate();
    if validator.has_errors() {
        for err in validator.errors() {
            eprintln!("Error: Semantic validation error: {}", err.message);
        }
        return 1;
    }

    // Serialization
    let sysml_text = pkg.to_sysml(0);
    if let Some(parent) = output_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Error creating output directory '{}': {}", parent.display(), e);
            return 1;
        }
    }
    if let Err(e) = fs::write(output_path, &sysml_text) {
        eprintln!("Error writing compiled schema to '{}': {}", output_path.display(), e);
        return 1;
    }

    // Digest generation
    let mut digest = SchemaDigest::compute(&pkg, &sysml_text);
    if let SchemaTarget::Directory(_, ref files) = target {
        let manifest = compute_file_manifest(files);
        digest = digest.with_file_manifest(manifest);
    }
    if let Err(e) = digest.write_to_file(digest_path) {
        eprintln!("Error writing schema digest to '{}': {}", digest_path.display(), e);
        return 1;
    }

    println!(
        "Successfully compiled SysML schema '{}' to '{}' (SHA-256: {}, {} elements)",
        display_name,
        output_path.display(),
        digest.sha256,
        total_elements
    );

    0
}

/// Resolve input schema path into either a single file or a directory containing .sysml files.
pub fn resolve_schema_targets(explicit_path: Option<PathBuf>) -> Result<SchemaTarget, String> {
    if let Some(path) = explicit_path {
        if path.is_file() {
            return Ok(SchemaTarget::SingleFile(path));
        } else if path.is_dir() {
            let files = discover_sysml_files(&path);
            if files.is_empty() {
                return Err(format!("Error: No .sysml files found in directory: {}", path.display()));
            }
            if files.len() == 1 {
                return Ok(SchemaTarget::SingleFile(files[0].clone()));
            }
            return Ok(SchemaTarget::Directory(path, files));
        }
        return Err(format!("Error: Schema file does not exist: {}", path.display()));
    }

    // Look in current directory schema/
    let schema_dir = Path::new("schema");
    if schema_dir.is_dir() {
        let files = discover_sysml_files(schema_dir);
        if !files.is_empty() {
            if files.len() == 1 {
                return Ok(SchemaTarget::SingleFile(files[0].clone()));
            }
            return Ok(SchemaTarget::Directory(schema_dir.to_path_buf(), files));
        }
    }

    Err(SCHEMA_REMEDIATION_MESSAGE.to_string())
}

/// Resolve a single concrete schema file path for legacy/single-file callers.
pub fn resolve_schema_file(explicit_path: Option<PathBuf>) -> Result<PathBuf, String> {
    match resolve_schema_targets(explicit_path)? {
        SchemaTarget::SingleFile(p) => Ok(p),
        SchemaTarget::Directory(_, files) => {
            if let Some(model) = files.iter().find(|p| p.file_name().map_or(false, |n| n == "model.sysml")) {
                Ok(model.clone())
            } else {
                Ok(files[0].clone())
            }
        }
    }
}
