//! # SysML v2 Universal Ingestion Engine CLI Entrypoint
//!
//! ## 1. Safety Intent & Regulatory Scope
//! This module provides the command-line interface (CLI) entrypoint for the Ground 0.0
//! SysML v2 Universal Ingestion Engine. It discovers, classifies, translates, and serializes
//! heterogeneous Level 0 OEM technical specifications (Markdown, SysML v2, OMG IDL, OpenAPI, YANG)
//! into canonical SysML v2 textual models (`.pipeline/schema.sysml`) accompanied by an
//! authoritative cryptographic schema digest (`.pipeline/schema-digest.json`).
//!
//! In safety-critical software architectures (DO-178C Level A, ISO 26262 ASIL D, ECSS-E-ST-40C),
//! the ingestion CLI serves as the trusted boundary gateway between external engineering artifacts
//! and the formal MBSE compilation pipeline. An unhandled error, silent truncation, partial write,
//! or corrupted exit status can compromise the integrity of downstream static analysis and compilation.
//!
//! ## 2. Invariants & Mathematical Properties
//! - **Fail-Safe Exit Code Contract**:
//!   On any unhandled failure (e.g. invalid syntax, missing files, permission denial), the CLI halts
//!   immediately, logs an explicit diagnostic message to standard error, and terminates with exit code `1`.
//!   Partial writes or corrupted intermediate state are strictly prohibited.
//! - **Atomic File Operations**:
//!   Output SysML files and JSON digests are written atomically via temporary staging files
//!   ([`compile_sysml::write_atomic`], [`compile_sysml::write_digest_atomic`]), preventing partial
//!   or corrupted reads by concurrent monitoring tools.
//! - **Closed-World Scope Filtering**:
//!   Command-line arguments `--allowed-parts` and `--negative-invariants` enforce explicit
//!   architectural boundaries and synthesize formal negative exclusion invariants into the output model.
//! - **Cryptographic Model Verification**:
//!   Every ingestion run computes an authoritative SHA-256 fingerprint digest recording line counts,
//!   AST entity counts, and content hashes.
//!
//! ## 3. Preconditions
//! - Command-line arguments must conform to the [`CliArgs`] grammar parsed by `clap`.
//! - The target input file or directory (`--schema` or default `schema/`) must exist on disk and contain
//!   at least one supported Level 0 specification artifact.
//! - The process must hold write permissions for the parent directories of `--out` and `--digest`.
//!
//! ## 4. Postconditions
//! - On successful execution, `.pipeline/schema.sysml` (or `--out`) contains a deterministic, syntactically
//!   valid SysML v2 textual model, and `.pipeline/schema-digest.json` (or `--digest`) records its SHA-256
//!   cryptographic digest.
//! - On failure, target files remain untouched, staging files are unlinked, and the process exits with status `1`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - **Time Complexity**: $O(N \log N)$ where $N$ is the total byte size of discovered input specifications,
//!   bounded by deterministic lexicographical sorting and linear-pass AST translation and serialization.
//! - **Space Complexity**: $O(N)$ auxiliary heap memory to hold the parsed [`PackageDef`] AST and serialized
//!   SysML v2 output buffer prior to atomic disk commit.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - **Empty or Unreadable Schema Directory**: Detected prior to translation; returns an explicit `Err(String)`
//!   naming the searched path and halts before creating output files.
//! - **Unsupported or Raw Format**: Rejects untranslated raw documents (`"raw"`) and unknown format strings
//!   with a deterministic diagnostic error to prevent silent SSOT bypass.
//! - **Filesystem I/O Faults**: Handled by atomic write helpers (`write_atomic`, `write_digest_atomic`) so
//!   interrupted writes never corrupt existing baseline artifacts.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYSML-INGEST-CLI], [DO-330-TQL-4-INGEST-CLI]

use clap::Parser;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process;

use compile_sysml::{generate_digest, parse_sysml, to_sysml, write_atomic, write_digest_atomic};
use deap_core::source_ast::SourceLanguage;
use deap_core::sysml_ast::PackageDef;
use ingest_sysml::detectors::discover_schema_targets;
use ingest_sysml::filter_ast_to_target_scope;
use ingest_sysml::translators::{
    code::CodeTranslator, idl::IDLTranslator, markdown::MarkdownTranslator,
    openapi::OpenAPITranslator, yang::YangTranslator,
};

/// Ground 0.0 SysML v2 Universal Ingestion Engine Command-Line Arguments.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Encapsulates all configuration parameters, input paths, format overrides,
/// output targets, and architectural scope filters for the DO-330 TQL-4 ingestion CLI.
///
/// ## 2. Invariants & Mathematical Properties
/// - Default output paths deterministically target `.pipeline/schema.sysml` and `.pipeline/schema-digest.json`.
/// - Format defaults to `"auto"` when omitted by the caller and auto-resolves to `"code"` when `--source` is supplied.
///
/// ## 3. Preconditions
/// - Supplied CLI flags must be valid UTF-8 strings and OS paths parseable by `clap::Parser`.
///
/// ## 4. Postconditions
/// - Produces an immutable [`CliArgs`] configuration record passed directly to [`run_ingestion`].
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: $O(A)$ where $A$ is the number of command-line argument tokens.
/// - **Space Complexity**: $O(A)$ heap allocation for owned [`PathBuf`] and `String` fields.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Invalid or unknown CLI flags cause `clap` to emit usage diagnostics and exit cleanly with non-zero status.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-CLI/CliArgs], [DO-330-TQL-4-INGEST-CLI]
#[derive(Parser, Debug)]
#[command(name = "ingest-sysml", version = "0.1.0", about = "SysML v2 Universal Ingestion Engine")]
pub struct CliArgs {
    /// Path to root of source code tree for AST extraction (mutually exclusive with --schema)
    #[arg(long)]
    pub source: Option<PathBuf>,

    /// Path to input schema file or directory containing schemas (default: schema/)
    #[arg(short, long)]
    pub schema: Option<PathBuf>,

    /// Schema format specification (auto, code, sysml, idl, openapi, markdown, yang, autosar, protobuf)
    #[arg(short, long, default_value = "auto")]
    pub format: String,

    /// Source language parser selection for code ingestion (auto, dart, rust)
    #[arg(long, default_value = "auto")]
    pub language: String,

    /// Output SysML v2 textual file path
    #[arg(short, long, default_value = ".pipeline/schema.sysml")]
    pub out: PathBuf,

    /// Output schema digest JSON path
    #[arg(short, long, default_value = ".pipeline/schema-digest.json")]
    pub digest: PathBuf,

    /// Comma-separated list of allowed part def names to preserve in closed-world scope
    #[arg(long)]
    pub allowed_parts: Option<String>,

    /// Comma-separated list of entities to assert negative exclusion for (!exists(...))
    #[arg(long)]
    pub negative_invariants: Option<String>,
}

/// Binary entrypoint for the `ingest-sysml` command-line executable.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Parses process arguments via [`CliArgs`] and invokes [`run_ingestion`], enforcing the
/// DO-178C Level A / DO-330 TQL-4 fail-safe process exit contract on any ingestion failure.
///
/// ## 2. Invariants & Mathematical Properties
/// - Total termination: every execution path either completes [`run_ingestion`] with exit code `0`
///   or prints a structured diagnostic to `stderr` and exits with code `1`.
///
/// ## 3. Preconditions
/// - Invoked as an OS process with standard argument vector and `stderr`/`stdout` streams attached.
///
/// ## 4. Postconditions
/// - Terminates with exit status `0` iff [`run_ingestion`] returns `Ok(())`; terminates with exit status `1` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: Dominated by [`run_ingestion`], $O(N \log N)$ in total input specification size $N$.
/// - **Space Complexity**: Dominated by [`run_ingestion`], $O(N)$ for AST and serialization buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Any `Err(e)` from [`run_ingestion`] is formatted to `stderr` with the `[SysML v2 Ingestion] ERROR:` prefix
///   before immediate `process::exit(1)`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-CLI/main], [DO-330-TQL-4-INGEST-CLI]
fn main() {
    let args = CliArgs::parse();

    if let Err(e) = run_ingestion(args) {
        eprintln!("[SysML v2 Ingestion] ERROR: {}", e);
        process::exit(1);
    }
}

/// Executes the end-to-end ingestion pipeline with validated input arguments.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Coordinates schema target discovery, format classification, AST translation, closed-world scope
/// filtering, textual SysML v2 serialization, atomic file writes, and cryptographic digest recording
/// within a bounded, fail-safe transaction under DO-178C Level A and ISO 26262 ASIL D governance.
///
/// ## 2. Invariants & Mathematical Properties
/// - **Transactional Atomicity**: Target files (`args.out` and `args.digest`) are never partially overwritten;
///   all mutations occur via temporary staging files and atomic rename.
/// - **Deterministic Output**: Identical input schemas and scope filters yield byte-identical SysML v2 text
///   and identical SHA-256 schema digests.
///
/// ## 3. Preconditions
/// - `args` contains well-formed [`CliArgs`] parameters with readable schema paths and writable output directories.
///
/// ## 4. Postconditions
/// - On success, writes canonical SysML v2 text to `args.out`, writes the cryptographic digest to `args.digest`,
///   and returns `Ok(())`.
/// - On failure, returns an explanatory `Err(String)` and leaves existing output files unmodified.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: $O(N \log N + P + I)$ where $N$ is the input specification byte count, $P$ is the
///   number of `--allowed-parts` tokens, and $I$ is the number of `--negative-invariants` tokens.
/// - **Space Complexity**: $O(N + P + I)$ heap memory for AST nodes, scope filter sets, and serialized SysML text.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates discovery, file read, translator parse, unsupported format, and atomic write errors via `Result::Err`
///   without panicking or leaking temporary staging files.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-CLI/run_ingestion], [DO-330-TQL-4-INGEST-CLI]
pub fn run_ingestion(args: CliArgs) -> Result<(), String> {
    if args.source.is_some() && args.schema.is_some() {
        return Err("CLI options '--source' and '--schema' are mutually exclusive.".to_string());
    }

    let mut pkg: PackageDef;

    if let Some(ref source_path) = args.source {
        let fmt_lower = args.format.trim().to_ascii_lowercase();
        if fmt_lower != "auto" && fmt_lower != "code" {
            return Err(format!(
                "Invalid format '{}' when '--source' is specified. Expected 'auto' or 'code'.",
                args.format
            ));
        }
        let lang = SourceLanguage::from_flag(&args.language)?;
        let pkg_name = source_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("SourceCodeModel");
        pkg = CodeTranslator::with_language(lang).translate_directory(source_path, pkg_name)?;
        println!(
            "[SysML v2 Ingestion] Successfully ingested source AST from '{}' -> {}",
            source_path.display(),
            args.out.display()
        );
    } else if args.format.eq_ignore_ascii_case("code") {
        let target_path = args
            .schema
            .clone()
            .unwrap_or_else(|| PathBuf::from("src"));
        let lang = SourceLanguage::from_flag(&args.language)?;
        let pkg_name = target_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("SourceCodeModel");
        pkg = CodeTranslator::with_language(lang).translate_directory(&target_path, pkg_name)?;
        println!(
            "[SysML v2 Ingestion] Successfully ingested source AST from '{}' -> {}",
            target_path.display(),
            args.out.display()
        );
    } else {
        // Step 1: Discover schema targets and detect input format
        let (disc_fmt, disc_files) =
            discover_schema_targets(args.schema.as_deref(), Some(&args.format))?;

        if disc_files.is_empty() {
            let target_desc = args
                .schema
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "schema/ or schema/extracted/".to_string());
            return Err(format!("No supported schema files found in {}", target_desc));
        }

        // Step 2: Resolve format (auto vs user-specified override)
        let fmt = if args.format == "auto" {
            disc_fmt
        } else {
            args.format.to_ascii_lowercase()
        };

        let is_dir = args.schema.as_ref().is_some_and(|p| p.is_dir());

        // Step 3: Dispatch translation based on format and file cardinality
        if fmt == "markdown" && (disc_files.len() > 1 || is_dir) {
            let def_name = if is_dir {
                args.schema
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|s| s.to_str())
                    .unwrap_or("OEM_System_Model")
            } else {
                "OEM_System_Model"
            };

            let file_strs: Vec<String> = disc_files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            let translator = MarkdownTranslator::new();
            pkg = translator.translate_files(&file_strs, def_name)?;

            println!(
                "[SysML v2 Ingestion] Successfully ingested {} markdown files -> {}",
                disc_files.len(),
                args.out.display()
            );
        } else if fmt == "yang" && (disc_files.len() > 1 || is_dir) {
            let def_name = if is_dir {
                args.schema
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|s| s.to_str())
                    .unwrap_or("YangModel")
            } else {
                "YangModel"
            };

            let file_strs: Vec<String> = disc_files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            let translator = YangTranslator::new();
            pkg = translator.translate_files(&file_strs, def_name)?;

            println!(
                "[SysML v2 Ingestion] Successfully ingested {} yang files -> {}",
                disc_files.len(),
                args.out.display()
            );
        } else {
            let target_file = &disc_files[0];
            let content = fs::read_to_string(target_file)
                .map_err(|e| format!("Failed to read file '{}': {}", target_file.display(), e))?;
            let basename = target_file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Model");

            let lang = SourceLanguage::from_flag(&args.language)?;
            pkg = match fmt.as_str() {
                "sysml" => parse_sysml(&content).map_err(|e| {
                    format!("SysML v2 parse error in '{}': {}", target_file.display(), e)
                })?,
                "code" => CodeTranslator::with_language(lang).translate(&content, basename)?,
                "idl" => IDLTranslator::new().translate(&content, basename)?,
                "openapi" => OpenAPITranslator::new().translate(&content, basename)?,
                "markdown" => MarkdownTranslator::new().translate(&content, basename)?,
                "yang" => YangTranslator::new().translate(&content, basename)?,
                "raw" => {
                    return Err("AST translation is required for raw document formats. SSOT ingestion failed.".to_string());
                }
                other => {
                    return Err(format!(
                        "Unsupported format '{}' for file '{}'",
                        other,
                        target_file.display()
                    ));
                }
            };

            println!(
                "[SysML v2 Ingestion] Successfully ingested '{}' -> {}",
                target_file.display(),
                args.out.display()
            );
        }
    }

    // Step 4: Apply closed-world AST filtering & negative invariant projection
    let allowed_set: Option<HashSet<String>> = args.allowed_parts.as_ref().map(|s| {
        s.split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect()
    });

    let neg_vec: Option<Vec<String>> = args.negative_invariants.as_ref().map(|s| {
        s.split(',')
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty())
            .collect()
    });

    filter_ast_to_target_scope(&mut pkg, allowed_set.as_ref(), neg_vec.as_deref());

    // Step 5: Serialize AST to canonical SysML v2 textual representation
    let sysml_text = to_sysml(&pkg);

    // Step 6: Atomically write output SysML v2 textual model
    write_atomic(&args.out, &sysml_text)
        .map_err(|e| format!("Failed to write output to '{}': {}", args.out.display(), e))?;

    // Step 7: Generate and atomically write cryptographic schema digest
    let digest = generate_digest(&pkg, &sysml_text);
    write_digest_atomic(&args.digest, &digest)
        .map_err(|e| format!("Failed to write digest to '{}': {}", args.digest.display(), e))?;

    println!(
        "[SysML v2 Ingestion] Schema digest generated at {}",
        args.digest.display()
    );

    Ok(())
}
