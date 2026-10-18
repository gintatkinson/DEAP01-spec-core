//! CLI entrypoint for compile-sysml compiler and specification synchronization engine.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides the primary command-line interface for the DEAP SysML v2 / KerML compilation gate,
//! STPA 10-pillar safety artifact transpilation, STPA-to-constraint compilation, and bidirectional
//! specification synchronization under DO-178C Level A / DO-330 TQL-4 and ISO 26262 ASIL D.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Dispatch determinism: identical CLI arguments and workspace input files produce bitwise-identical
//!   serialized SysML models, SHA-256 digests, and synchronization outputs.
//! - Non-zero exit enforcement: any lexical, syntactic, semantic, or I/O violation terminates execution
//!   with a deterministic non-zero process exit code (`1`).
//!
//! ## 3. Preconditions
//! - Invoked with valid CLI flags parsed via `clap::Parser`.
//! - Target schema files or directories must exist on disk and contain valid UTF-8 SysML v2 / KerML text.
//!
//! ## 4. Postconditions
//! - Executes the requested pipeline stage (`--stpa-transpile`, `--stpa`, `--forward-sync`,
//!   `--reverse-sync`, or `--compile`) and persists verified artifacts to the target paths.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time complexity: $O(N \log N)$ where $N$ is the total byte length and AST node count of the
//!   resolved SysML schema tree and markdown specification corpus.
//! - Space complexity: $O(N)$ for holding the parsed `SysmlPackage` AST and serialized output buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Missing schema files, empty schema models, parser syntax errors, semantic validation errors,
//!   or filesystem write failures emit structured diagnostics to `stderr` and halt execution immediately.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-COMPILER-SYSML-CLI], [DO-330-TQL-4-CLI]

use clap::Parser;
use compile_sysml::docstring::{
    run_docstring_injection, run_extract_source, DocstringExitCode, DocstringInjectionOptions,
};
use compile_sysml::semantic::serializer::{to_sysml, SysmlSerializable};
use compile_sysml::semantic::validator::SemanticValidator;
use compile_sysml::stpa::compile_stpa_to_constraints_filtered;
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

/// Command-line argument definition for the `compile-sysml` compiler and synchronization engine.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Defines the deterministic CLI contract for `compile-sysml` under DO-178C Level A and DO-330 TQL-4,
/// covering Pipeline 0 compilation, STPA transpilation, specification synchronization, source AST
/// extraction, and 7-section safety docstring synthesis.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strongly typed argument binding via `clap::Parser`.
/// - Defaults `docs` to `"docs/"`, `out` to `".pipeline/schema.sysml"`, and `digest` to
///   `".pipeline/schema-digest.json"`.
///
/// ## 3. Preconditions
/// - Constructed from process arguments via `Cli::parse()`.
///
/// ## 4. Postconditions
/// - Populates validated `PathBuf` and boolean mode flags for downstream dispatch in `main`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(A) where A is the number of CLI argument tokens.
/// - Space Complexity: O(A) for owned path buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Invalid or unrecognized CLI flags are rejected by `clap` prior to entering pipeline execution.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-CLI-ARGS], [DO-330-TQL-4-CLI]
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

    /// Synthesize and inject 7-section safety docstrings into source files under DIR
    #[arg(long, value_name = "DIR")]
    inject_safety_docstrings: Option<PathBuf>,

    /// Extract structural source AST elements from DIR into SysML v2 notation
    #[arg(long, value_name = "DIR")]
    extract_source: Option<PathBuf>,

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

/// Primary CLI entrypoint for the `compile-sysml` executable.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Routes command-line execution across STPA safety suite transpilation, STPA constraint compilation,
/// forward/reverse specification synchronization, 7-section safety docstring injection, and the
/// Pipeline 0 SysML compilation gate under DO-178C Level A and DO-330 TQL-4 tool operational requirements.
///
/// ## 2. Invariants & Mathematical Properties
/// - Exclusive mode dispatch: exactly one operational branch executes per CLI invocation in priority
///   order (`--inject-safety-docstrings`, `--extract-source`, `--stpa-transpile`, `--stpa`,
///   `--forward-sync`, `--reverse-sync`, `--compile`).
/// - Fail-closed termination: any operational error exits immediately with a non-zero status code.
///
/// ## 3. Preconditions
/// - Process environment supplies valid OS argument strings parseable by `Cli::parse()`.
/// - Referenced input paths (`--schema`, `FILE`, `--stpa`, `--scoring-config`) must be readable when specified.
///
/// ## 4. Postconditions
/// - Target compilation, safety, docstring, or synchronization artifacts are written to disk, or usage
///   guidance is printed to `stdout` when no operational mode flag is supplied.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(N \log N)$ bounded by downstream AST parsing, validation, and synchronization passes.
/// - Space complexity: $O(N)$ proportional to the in-memory AST and file buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Missing configuration files, malformed JSON scoring configs, unreadable schema files, or downstream
///   transpilation/sync errors log diagnostic messages to `stderr` and invoke `process::exit`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-MAIN], [DO-330-TQL-4-CLI]
fn main() {
    let cli = Cli::parse();

    // 0a. Safety Docstring Synthesis & Injection (FEAT-42)
    if let Some(ref target_dir) = cli.inject_safety_docstrings {
        let schema_model = match resolve_schema_targets(cli.schema.clone().or_else(|| cli.file.clone())) {
            Ok(SchemaTarget::SingleFile(p)) => fs::read_to_string(&p)
                .ok()
                .and_then(|c| parse_sysml(&c).ok()),
            Ok(SchemaTarget::Directory(_, files)) => parse_sysml_tree(&files).ok(),
            Err(_) => None,
        };
        let opts = DocstringInjectionOptions {
            target_path: target_dir.clone(),
            dry_run: cli.dry_run,
            force: cli.force,
            schema_model,
        };
        match run_docstring_injection(&opts) {
            Ok(report) => {
                println!(
                    "[Docstring Synthesizer] Scanned {} file(s), modified {} file(s), enriched {} item(s) in '{}'",
                    report.files_scanned,
                    report.files_modified,
                    report.items_enriched,
                    target_dir.display()
                );
                process::exit(DocstringExitCode::Clean.code());
            }
            Err((code, msg)) => {
                eprintln!("Error during docstring injection: {}", msg);
                process::exit(code.code());
            }
        }
    }

    // 0b. Source Code AST Extraction to SysML (FEAT-42)
    if let Some(ref extract_dir) = cli.extract_source {
        match run_extract_source(extract_dir, &cli.out, &cli.digest, cli.dry_run) {
            Ok(pkg) => {
                println!(
                    "[Extract Source] Extracted {} structural item(s) from '{}' to '{}'",
                    count_structural_elements(&pkg),
                    extract_dir.display(),
                    cli.out.display()
                );
                process::exit(DocstringExitCode::Clean.code());
            }
            Err((code, msg)) => {
                eprintln!("Error during source extraction: {}", msg);
                process::exit(code.code());
            }
        }
    }

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

        let schema_target = cli.schema.or(cli.file);
        let base_model = schema_target.and_then(|sp| {
            fs::read_to_string(&sp).ok().and_then(|txt| parse_sysml(&txt).ok())
        });
        let pkg = compile_stpa_to_constraints_filtered(&content, "System_SafetyConstraints", None, base_model.as_ref());
        let sysml_code = to_sysml(&pkg);

        if cli.out.as_path() != Path::new(".pipeline/schema.sysml") {
            if let Some(parent) = cli.out.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("Error creating output directory '{}': {}", parent.display(), e);
                    process::exit(1);
                }
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

    println!("compile-sysml: Specify --compile, --stpa-transpile, --stpa, --forward-sync, --reverse-sync, --inject-safety-docstrings, or --extract-source. Run with --help for usage.");
}

/// Target SysML schema input resolved from CLI flags or default paths.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Distinguishes between a single `.sysml` schema file and a multi-file schema directory tree
/// under DO-178C Level A and DO-330 TQL-4 deterministic model loading requirements.
///
/// ## 2. Invariants & Mathematical Properties
/// - `Directory` always holds a non-empty, lexicographically sorted `Vec<PathBuf>`.
/// - Single-file directories are canonicalized into `SingleFile`.
///
/// ## 3. Preconditions
/// - Constructed exclusively by `resolve_schema_targets`.
///
/// ## 4. Postconditions
/// - Enables callers to dispatch either `parse_sysml` or `parse_sysml_tree` deterministically.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) variant inspection.
/// - Space Complexity: O(F) where F is the number of files in `Directory`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total sum type with no invalid runtime states.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-SCHEMA-TARGET], [DO-330-TQL-4-CLI]
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaTarget {
    /// A single concrete .sysml file path.
    SingleFile(PathBuf),
    /// A directory containing multiple discovered .sysml files.
    Directory(PathBuf, Vec<PathBuf>),
}

/// Execute the Pipeline 0 compilation gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Enforces the mandatory Pipeline 0 SysML v2 / KerML compilation gate -- parsing input schemas,
/// verifying non-empty structural element counts, running semantic validation, emitting canonical
/// serialized SysML text, and writing the cryptographic SHA-256 `SchemaDigest` under DO-178C Level A
/// and DO-330 TQL-4 verification baselines.
///
/// ## 2. Invariants & Mathematical Properties
/// - Non-vacuous model guarantee: returns exit code `1` whenever `count_structural_elements(&pkg) == 0`.
/// - Semantic soundness: halts prior to serialization if `SemanticValidator` records any error.
/// - Cryptographic digest binding: emitted `SchemaDigest` SHA-256 matches the exact serialized output
///   written to `output_path` and records per-file manifests for directory targets.
///
/// ## 3. Preconditions
/// - `explicit_schema` is either `Some(path)` pointing to an existing `.sysml` file/directory or `None`
///   when `schema/` contains discoverable `.sysml` files.
/// - Parent directories of `output_path` and `digest_path` are writable by the current process.
///
/// ## 4. Postconditions
/// - On return value `0`, `output_path` contains the canonical serialized SysML model and `digest_path`
///   contains the matching JSON `SchemaDigest`.
/// - On return value `1`, diagnostic errors are emitted to `stderr` and downstream gates are blocked.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(N \log N)$ where $N$ is the total input character count and AST node count.
/// - Space complexity: $O(N)$ for the parsed AST, validator symbol tables, and serialized SysML buffer.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Missing or empty schema files, syntax errors, zero structural elements, semantic validation errors,
///   or filesystem write failures return `1` cleanly without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-GATE], [DO-330-TQL-4-GATE]
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Deterministically resolves the target SysML v2 schema source from an explicit CLI path or the
/// default `schema/` workspace directory under DO-178C Level A and ISO 26262 ASIL D configuration
/// management rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Canonical singleton normalization: a directory containing exactly one `.sysml` file resolves
///   to `SchemaTarget::SingleFile(only)` to preserve single-file diagnostic semantics.
/// - Deterministic file ordering: multi-file directories rely on `discover_sysml_files` lexicographical
///   sorting so compilation order is platform-independent.
///
/// ## 3. Preconditions
/// - `explicit_path` is an optional filesystem path provided by the caller or CLI parser.
///
/// ## 4. Postconditions
/// - Returns `Ok(SchemaTarget::SingleFile)` or `Ok(SchemaTarget::Directory)` with non-empty file list,
///   or `Err(String)` containing actionable remediation instructions.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(F \log F)$ where $F$ is the number of filesystem entries under the target directory.
/// - Space complexity: $O(F)$ to store discovered `PathBuf` entries.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-existent explicit paths, directories devoid of `.sysml` files, or missing default `schema/`
///   files return a descriptive `Err(String)` without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-RESOLVE-TARGETS], [DO-330-TQL-4-CLI]
pub fn resolve_schema_targets(explicit_path: Option<PathBuf>) -> Result<SchemaTarget, String> {
    if let Some(path) = explicit_path {
        if path.is_file() {
            return Ok(SchemaTarget::SingleFile(path));
        } else if path.is_dir() {
            let files = discover_sysml_files(&path);
            if files.is_empty() {
                return Err(format!("Error: No .sysml files found in directory: {}", path.display()));
            }
            if let [only] = files.as_slice() {
                return Ok(SchemaTarget::SingleFile(only.clone()));
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
            if let [only] = files.as_slice() {
                return Ok(SchemaTarget::SingleFile(only.clone()));
            }
            return Ok(SchemaTarget::Directory(schema_dir.to_path_buf(), files));
        }
    }

    Err(SCHEMA_REMEDIATION_MESSAGE.to_string())
}

/// Resolve a single concrete schema file path for legacy/single-file callers.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Selects a deterministic primary `.sysml` file path when a single file path is required from
/// a resolved `SchemaTarget` under DO-178C Level A / DO-330 TQL-4 tool qualification rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Root model preference: when resolving a multi-file `SchemaTarget::Directory`, prioritizes
///   `model.sysml` if present; otherwise selects the lexicographically first `.sysml` path.
///
/// ## 3. Preconditions
/// - `explicit_path` is either `Some(path)` or `None` (defaulting to `schema/` discovery).
///
/// ## 4. Postconditions
/// - Returns `Ok(PathBuf)` pointing to a concrete `.sysml` file on disk, or `Err(String)` if no
///   valid schema file can be resolved.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(F \log F)$ where $F$ is the number of discovered `.sysml` files.
/// - Space complexity: $O(F)$ for the intermediate `SchemaTarget` file vector.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates resolution errors from `resolve_schema_targets` or returns `SCHEMA_REMEDIATION_MESSAGE`
///   if the directory file list is empty.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-RESOLVE-FILE], [DO-330-TQL-4-CLI]
pub fn resolve_schema_file(explicit_path: Option<PathBuf>) -> Result<PathBuf, String> {
    match resolve_schema_targets(explicit_path)? {
        SchemaTarget::SingleFile(p) => Ok(p),
        SchemaTarget::Directory(_, files) => {
            if let Some(model) = files.iter().find(|p| p.file_name().is_some_and(|n| n == "model.sysml")) {
                Ok(model.clone())
            } else {
                files.first().cloned().ok_or_else(|| SCHEMA_REMEDIATION_MESSAGE.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_compile_sysml_main_7_section_rustdoc() {
        let src = include_str!("main.rs");
        assert!(src.contains("//! ## 1. Safety Intent & Regulatory Scope"));
        assert!(src.contains("//! ## 2. Invariants & Mathematical Properties"));
        assert!(src.contains("//! ## 3. Preconditions"));
        assert!(src.contains("//! ## 4. Postconditions"));
        assert!(src.contains("//! ## 5. Algorithmic Complexity Bounds"));
        assert!(src.contains("//! ## 6. Failure Modes & Fault Tolerance"));
        assert!(src.contains("//! ## 7. Realization & Traceability"));
        assert!(src.contains("//! Realises: [REQ-COMPILER-SYSML-CLI], [DO-330-TQL-4-CLI]"));
        assert!(src.contains("/// ## 1. Safety Intent & Regulatory Scope"));
        assert!(src.contains("/// Realises: [REQ-COMPILER-SYSML-MAIN]"));
        assert!(src.contains("/// Realises: [REQ-COMPILER-SYSML-GATE]"));
        assert!(src.contains("/// Realises: [REQ-COMPILER-SYSML-RESOLVE-TARGETS]"));
        assert!(src.contains("/// Realises: [REQ-COMPILER-SYSML-RESOLVE-FILE]"));
    }
}
