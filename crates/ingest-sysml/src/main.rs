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
#![allow(
    clippy::collapsible_if,
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unnecessary_map_or,
    clippy::unwrap_used
)]

use clap::Parser;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use compile_sysml::{
    generate_digest, parse_sysml, parse_sysml_tree, to_sysml, write_atomic, write_digest_atomic,
};
use deap_core::source_ast::{resolve_source_package_name, sanitize_sysml_identifier, SourceLanguage};
use deap_core::sysml_ast::PackageDef;
use ingest_sysml::detectors::discover_schema_targets;
use ingest_sysml::translators::{
    code::CodeTranslator, idl::IDLTranslator, markdown::MarkdownTranslator,
    openapi::OpenAPITranslator, yang::YangTranslator,
};
use ingest_sysml::{
    cache_ingested_sysml_model, collect_declared_part_names, filter_ast_to_realized_or_allowed,
    filter_ast_to_target_scope, verify_scope_and_negative_invariants, IngestExitCode,
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
    /// Path to root of source code tree for AST extraction (can be combined with --schema for realization scoping)
    #[arg(long)]
    pub source: Option<PathBuf>,

    /// Path to input schema file or directory containing schemas (default: schema/)
    #[arg(short, long)]
    pub schema: Option<PathBuf>,

    /// Optional explicit SysML v2 root package name override for the ingested model
    #[arg(long)]
    pub package: Option<String>,

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

    /// Populate or validate the SHA-256 incremental AST cache (.pipeline/cache/sysml_ast_cache.json)
    #[arg(long)]
    pub cache_ast: bool,
}

/// Derives a deterministic SysML v2 package identifier for an ingested source directory or explicit override.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and Issue #472 package namespace hygiene
/// (`resolve_ingested_package_name`). Ensures that pointing `--source` at `<component>/src`
/// or `<component>/lib` produces `<component>_<src|lib>_Realization` instead of polluting
/// the root namespace with raw `package src` or `package lib`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Honors `explicit_package` whenever a non-empty trimmed package identifier is supplied.
/// - Delegates parent-qualified `"src"` / `"lib"` realization naming to [`resolve_source_package_name`].
///
/// ## 3. Preconditions
/// - `source_path: &Path` is the target source directory or file path.
/// - `explicit_package: Option<&str>` is the optional `--package` CLI flag value.
///
/// ## 4. Postconditions
/// - Returns a non-empty, sanitized SysML v2 identifier suitable for `PackageDef::name`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: $O(P)$ where $P$ is the length of the path leaf/parent or explicit package string.
/// - **Space Complexity**: $O(P)$ heap allocation for the returned `String`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Falls back deterministically to `"SourceCodeModel"` when `source_path` has no usable basename.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-CLI/resolve_ingested_package_name], [REQ-ISSUE-472-PACKAGE-QUALIFICATION]
pub fn resolve_ingested_package_name(
    source_path: &Path,
    explicit_package: Option<&str>,
) -> String {
    resolve_source_package_name(source_path, explicit_package)
}

/// Binary entrypoint for the `ingest-sysml` command-line executable.
///
/// # Summary
/// Parses process arguments via [`CliArgs`] and invokes [`run_ingestion_with_code`], terminating
/// with exit status `0` (`Clean`), `1` (`ParseOrIoError`), or `2` (`ScopeOrCryptoError`).
///
/// # Preconditions
/// - Invoked as an OS process with standard argument vector and `stderr`/`stdout` streams attached.
///
/// # Postconditions
/// - Terminates with exit status `0` iff [`run_ingestion_with_code`] returns `Ok(())`; terminates
///   with `exit_code.code()` (`1` or `2`) on `Err((exit_code, e))`.
///
/// # Invariants
/// - Total termination contract: never panics or leaves partial staging files on failure.
///
/// # Failure Modes
/// - Prints structured diagnostic `[SysML v2 Ingestion] ERROR: <msg>` to `stderr` before calling `process::exit`.
///
/// # Side Effects
/// - Reads input schemas/sources, writes `.pipeline/schema.sysml` and `.pipeline/schema-digest.json`, and exits the process.
///
/// # Determinism
/// - Deterministic exit status mapping via [`IngestExitCode::code`].
///
/// ## 1. Safety Intent & Regulatory Scope
/// Parses process arguments via [`CliArgs`] and invokes [`run_ingestion_with_code`], enforcing the
/// DO-178C Level A / DO-330 TQL-4 fail-safe `0 / 1 / 2` process exit contract on any ingestion failure.
///
/// ## 2. Invariants & Mathematical Properties
/// - Total termination: every execution path either completes [`run_ingestion_with_code`] with exit code `0`
///   or prints a structured diagnostic to `stderr` and exits with code `1` or `2`.
///
/// ## 3. Preconditions
/// - Invoked as an OS process with standard argument vector and `stderr`/`stdout` streams attached.
///
/// ## 4. Postconditions
/// - Terminates with exit status `0` iff [`run_ingestion_with_code`] returns `Ok(())`; terminates with `exit_code.code()` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: Dominated by [`run_ingestion_with_code`], $O(N \log N)$ in total input specification size $N$.
/// - **Space Complexity**: Dominated by [`run_ingestion_with_code`], $O(N)$ for AST and serialization buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Any `Err((exit_code, e))` from [`run_ingestion_with_code`] is formatted to `stderr` with the
///   `[SysML v2 Ingestion] ERROR:` prefix before immediate `process::exit(exit_code.code())`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-CLI/main], [DO-330-TQL-4-INGEST-CLI] | SysML v2: part def SourceCodeAstIngestion
fn main() {
    let args = CliArgs::parse();

    if let Err((exit_code, e)) = run_ingestion_with_code(args) {
        eprintln!("[SysML v2 Ingestion] ERROR: {}", e);
        process::exit(exit_code.code());
    }
}

/// Executes the end-to-end ingestion pipeline with validated input arguments.
///
/// # Summary
/// Delegates to [`run_ingestion_with_code`] and projects any `(IngestExitCode, String)` error
/// into a plain `Err(String)` diagnostic for backward-compatible library callers.
///
/// # Preconditions
/// - `args` contains well-formed [`CliArgs`] parameters with readable schema/source paths and writable output directories.
///
/// # Postconditions
/// - Returns `Ok(())` on clean ingestion, scope verification, and SHA-256 digest verification, or `Err(String)` on failure.
///
/// # Invariants
/// - Preserves transactional atomicity of `args.out` and `args.digest`.
///
/// # Failure Modes
/// - Propagates any `ParseOrIoError` or `ScopeOrCryptoError` message from [`run_ingestion_with_code`].
///
/// # Side Effects
/// - Atomically writes `args.out` and `args.digest` upon successful validation.
///
/// # Determinism
/// - Identical inputs and flags produce byte-identical outputs and digests.
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
/// - Propagates discovery, file read, translator parse, scope/invariant, and atomic write errors via `Result::Err`
///   without panicking or leaking temporary staging files.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-CLI/run_ingestion], [DO-330-TQL-4-INGEST-CLI] | SysML v2: part def SourceCodeAstIngestion
pub fn run_ingestion(args: CliArgs) -> Result<(), String> {
    run_ingestion_with_code(args).map_err(|(_, msg)| msg)
}

/// Executes the end-to-end ingestion pipeline and categorizes failures by [`IngestExitCode`].
///
/// # Summary
/// Performs source/schema discovery, AST translation, architectural scope & negative-invariant
/// verification, atomic SysML/digest persistence, and post-write SHA-256 read-back verification,
/// returning typed `(IngestExitCode, String)` diagnostics on failure.
///
/// # Preconditions
/// - `args` specifies valid CLI flags for `--source` or `--schema`, `--out`, and `--digest`.
///
/// # Postconditions
/// - Returns `Ok(())` when translation, scope verification, atomic writes, and post-write SHA-256
///   verification all succeed.
/// - Returns `Err((IngestExitCode::ParseOrIoError, msg))` on discovery, format, I/O, or syntax errors (`1`).
/// - Returns `Err((IngestExitCode::ScopeOrCryptoError, msg))` on scope starvation, negative-invariant
///   violations, or post-write SHA-256 mismatch (`2`).
///
/// # Invariants
/// - Zero `.unwrap()`, `.expect()`, or direct slice indexing (`[i]`).
/// - Scope and negative-invariant checks execute before any output file is written to disk.
///
/// # Failure Modes
/// - Maps lexical/syntactic/filesystem errors to `IngestExitCode::ParseOrIoError` (`1`).
/// - Maps `verify_scope_and_negative_invariants` and post-write SHA-256 verification errors to
///   `IngestExitCode::ScopeOrCryptoError` (`2`).
///
/// # Side Effects
/// - Reads input source/schema files and atomically writes `args.out`, `args.digest`, and optional AST cache.
///
/// # Determinism
/// - Deterministic AST lowering, serialization, SHA-256 hashing, and exit code classification.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Implements Layer 3 CLI & Atomic Persistence Interface Binding for `IngestSysml::SourceCodeAstIngestion`
/// under DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D.
///
/// ## 2. Invariants & Mathematical Properties
/// - **Pre-Write Scope Gate**: [`verify_scope_and_negative_invariants`] runs before [`write_atomic`],
///   guaranteeing that scope or negative-invariant violations never overwrite `args.out` or `args.digest`.
/// - **Closed-Loop Cryptographic Verification**: Re-reads `args.out` and `args.digest` from disk after
///   atomic rename and verifies $\text{SHA256}(\text{persisted\_sysml}) = \text{digest.sha256}$.
///
/// ## 3. Preconditions
/// - `args: CliArgs` is a populated command-line configuration structure.
///
/// ## 4. Postconditions
/// - Persists verified `.sysml` and `.json` digest artifacts on `Ok(())`, or returns a categorized
///   `(IngestExitCode, String)` error tuple.
///
/// ## 5. Algorithmic Complexity Bounds
/// - **Time Complexity**: $O(N \log N + P + I)$ in total input byte size $N$ and filter counts $P, I$.
/// - **Space Complexity**: $O(N + P + I)$ for AST and serialization buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Halts closed on any lexical, syntactic, I/O, scope, or cryptographic verification error.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-CLI/run_ingestion_with_code], [DO-330-TQL-4-INGEST-CLI] | SysML v2: part def SourceCodeAstIngestion
pub fn run_ingestion_with_code(args: CliArgs) -> Result<(), (IngestExitCode, String)> {
    let mut pkg: PackageDef;

    if let Some(ref source_path) = args.source {
        let fmt_lower = args.format.trim().to_ascii_lowercase();
        if fmt_lower != "auto" && fmt_lower != "code" {
            return Err((
                IngestExitCode::ParseOrIoError,
                format!(
                    "Invalid format '{}' when '--source' is specified. Expected 'auto' or 'code'.",
                    args.format
                ),
            ));
        }
        let lang = SourceLanguage::from_flag(&args.language)
            .map_err(|e| (IngestExitCode::ParseOrIoError, e))?;
        let pkg_name = resolve_ingested_package_name(source_path, args.package.as_deref());
        pkg = CodeTranslator::with_language(lang)
            .translate_directory(source_path, &pkg_name)
            .map_err(|e| (IngestExitCode::ParseOrIoError, e))?;

        // When --schema is supplied alongside --source (and --allowed-parts is omitted),
        // scope code PartDef extraction to architectural part defs declared in the reference schema
        // or source items carrying explicit Realises: requirement bindings.
        if let Some(ref schema_path) = args.schema {
            if args.allowed_parts.is_none() {
                let ref_pkg = if schema_path.is_dir() {
                    let (_, sysml_files) =
                        discover_schema_targets(Some(schema_path), Some("sysml"))
                            .map_err(|e| (IngestExitCode::ParseOrIoError, e))?;
                    parse_sysml_tree(&sysml_files).map_err(|e| {
                        (
                            IngestExitCode::ParseOrIoError,
                            format!(
                                "Failed to parse reference schema directory '{}': {}",
                                schema_path.display(),
                                e
                            ),
                        )
                    })?
                } else {
                    let schema_content = fs::read_to_string(schema_path).map_err(|e| {
                        (
                            IngestExitCode::ParseOrIoError,
                            format!(
                                "Failed to read reference schema '{}': {}",
                                schema_path.display(),
                                e
                            ),
                        )
                    })?;
                    parse_sysml(&schema_content).map_err(|e| {
                        (
                            IngestExitCode::ParseOrIoError,
                            format!(
                                "Failed to parse reference schema '{}': {}",
                                schema_path.display(),
                                e
                            ),
                        )
                    })?
                };
                let mut schema_parts = HashSet::new();
                collect_declared_part_names(&ref_pkg, &mut schema_parts);
                filter_ast_to_realized_or_allowed(&mut pkg, &schema_parts);
            }
        }

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
        let lang = SourceLanguage::from_flag(&args.language)
            .map_err(|e| (IngestExitCode::ParseOrIoError, e))?;
        let pkg_name = resolve_ingested_package_name(&target_path, args.package.as_deref());
        pkg = CodeTranslator::with_language(lang)
            .translate_directory(&target_path, &pkg_name)
            .map_err(|e| (IngestExitCode::ParseOrIoError, e))?;
        println!(
            "[SysML v2 Ingestion] Successfully ingested source AST from '{}' -> {}",
            target_path.display(),
            args.out.display()
        );
    } else {
        // Step 1: Discover schema targets and detect input format
        let (disc_fmt, disc_files) =
            discover_schema_targets(args.schema.as_deref(), Some(&args.format))
                .map_err(|e| (IngestExitCode::ParseOrIoError, e))?;

        if disc_files.is_empty() {
            let target_desc = args
                .schema
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "schema/ or schema/extracted/".to_string());
            return Err((
                IngestExitCode::ParseOrIoError,
                format!("No supported schema files found in {}", target_desc),
            ));
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
            pkg = translator
                .translate_files(&file_strs, def_name)
                .map_err(|e| (IngestExitCode::ParseOrIoError, e))?;

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
            pkg = translator
                .translate_files(&file_strs, def_name)
                .map_err(|e| (IngestExitCode::ParseOrIoError, e))?;

            println!(
                "[SysML v2 Ingestion] Successfully ingested {} yang files -> {}",
                disc_files.len(),
                args.out.display()
            );
        } else {
            let target_file = disc_files.first().ok_or_else(|| {
                (
                    IngestExitCode::ParseOrIoError,
                    "No schema file available for single-file ingestion".to_string(),
                )
            })?;
            let content = fs::read_to_string(target_file).map_err(|e| {
                (
                    IngestExitCode::ParseOrIoError,
                    format!("Failed to read file '{}': {}", target_file.display(), e),
                )
            })?;
            let basename = target_file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Model");

            let lang = SourceLanguage::from_flag(&args.language)
                .map_err(|e| (IngestExitCode::ParseOrIoError, e))?;
            pkg = match fmt.as_str() {
                "sysml" => parse_sysml(&content).map_err(|e| {
                    (
                        IngestExitCode::ParseOrIoError,
                        format!("SysML v2 parse error in '{}': {}", target_file.display(), e),
                    )
                })?,
                "code" => CodeTranslator::with_language(lang)
                    .translate(&content, basename)
                    .map_err(|e| (IngestExitCode::ParseOrIoError, e))?,
                "idl" => IDLTranslator::new()
                    .translate(&content, basename)
                    .map_err(|e| (IngestExitCode::ParseOrIoError, e))?,
                "openapi" => OpenAPITranslator::new()
                    .translate(&content, basename)
                    .map_err(|e| (IngestExitCode::ParseOrIoError, e))?,
                "markdown" => MarkdownTranslator::new()
                    .translate(&content, basename)
                    .map_err(|e| (IngestExitCode::ParseOrIoError, e))?,
                "yang" => YangTranslator::new()
                    .translate(&content, basename)
                    .map_err(|e| (IngestExitCode::ParseOrIoError, e))?,
                "raw" => {
                    return Err((
                        IngestExitCode::ParseOrIoError,
                        "AST translation is required for raw document formats. SSOT ingestion failed."
                            .to_string(),
                    ));
                }
                other => {
                    return Err((
                        IngestExitCode::ParseOrIoError,
                        format!(
                            "Unsupported format '{}' for file '{}'",
                            other,
                            target_file.display()
                        ),
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

    if let Some(explicit_pkg) = args.package.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        pkg.name = sanitize_sysml_identifier(explicit_pkg, &pkg.name);
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
    verify_scope_and_negative_invariants(&pkg, allowed_set.as_ref(), neg_vec.as_deref())
        .map_err(|e| (IngestExitCode::ScopeOrCryptoError, e))?;

    // Step 5: Serialize AST to canonical SysML v2 textual representation
    let sysml_text = to_sysml(&pkg);

    // Step 6: Atomically write output SysML v2 textual model
    write_atomic(&args.out, &sysml_text).map_err(|e| {
        (
            IngestExitCode::ParseOrIoError,
            format!("Failed to write output to '{}': {}", args.out.display(), e),
        )
    })?;

    // Step 7: Generate and atomically write cryptographic schema digest
    let digest = generate_digest(&pkg, &sysml_text);
    write_digest_atomic(&args.digest, &digest).map_err(|e| {
        (
            IngestExitCode::ParseOrIoError,
            format!(
                "Failed to write digest to '{}': {}",
                args.digest.display(),
                e
            ),
        )
    })?;

    // Step 7b: Closed-loop post-write cryptographic SHA-256 verification
    let persisted_sysml = fs::read_to_string(&args.out).map_err(|e| {
        (
            IngestExitCode::ScopeOrCryptoError,
            format!(
                "Post-write cryptographic verification failed reading '{}': {}",
                args.out.display(),
                e
            ),
        )
    })?;
    let persisted_digest_json = fs::read_to_string(&args.digest).map_err(|e| {
        (
            IngestExitCode::ScopeOrCryptoError,
            format!(
                "Post-write cryptographic verification failed reading '{}': {}",
                args.digest.display(),
                e
            ),
        )
    })?;
    let computed_sha = ingest_sysml::digest::compute_sha256(persisted_sysml.as_bytes());
    if computed_sha != digest.sha256 || !persisted_digest_json.contains(&digest.sha256) {
        return Err((
            IngestExitCode::ScopeOrCryptoError,
            format!(
                "Post-write cryptographic SHA-256 mismatch for '{}' (expected {}, computed {})",
                args.out.display(),
                digest.sha256,
                computed_sha
            ),
        ));
    }

    println!(
        "[SysML v2 Ingestion] Schema digest generated at {}",
        args.digest.display()
    );

    if args.cache_ast {
        let (_cached_model, cache_hit) = cache_ingested_sysml_model(Path::new("."), &sysml_text)
            .map_err(|e| (IngestExitCode::ParseOrIoError, e))?;
        println!(
            "[SysML v2 Ingestion] Incremental AST cache updated (cache_hit={})",
            cache_hit
        );
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
}
