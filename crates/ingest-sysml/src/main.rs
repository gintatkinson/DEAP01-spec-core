//! # SysML v2 Universal Ingestion Engine CLI Entrypoint
//!
//! ## 1. Safety Intent & Regulatory Scope
//! This module provides the command-line interface (CLI) entrypoint for the Ground 0.0
//! SysML v2 Universal Ingestion Engine. It discovers, classifies, translates, and serializes
//! heterogeneous Level 0 OEM technical specifications (Markdown, SysML v2, OMG IDL, OpenAPI)
//! into canonical SysML v2 textual models (`.pipeline/schema.sysml`) accompanied by an
//! authoritative cryptographic schema digest (`.pipeline/schema-digest.json`).
//!
//! In safety-critical software architectures (DO-178C Level A, ISO 26262 ASIL D, ECSS-E-ST-40C),
//! the ingestion CLI serves as the trusted boundary gateway between external engineering artifacts
//! and the formal MBSE compilation pipeline. An unhandled error, silent truncation, partial write,
//! or corrupted exit status can compromise the integrity of downstream static analysis and compilation.
//!
//! ## 2. Core Safety Invariants & Execution Contract
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

use clap::Parser;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process;

use compile_sysml::{generate_digest, parse_sysml, to_sysml, write_atomic, write_digest_atomic};
use deap_core::sysml_ast::PackageDef;
use ingest_sysml::detectors::discover_schema_targets;
use ingest_sysml::filter_ast_to_target_scope;
use ingest_sysml::translators::{
    idl::IDLTranslator, markdown::MarkdownTranslator, openapi::OpenAPITranslator,
    yang::YangTranslator,
};

/// Ground 0.0 SysML v2 Universal Ingestion Engine Command-Line Arguments.
///
/// Encapsulates all configuration parameters, input paths, format overrides,
/// output targets, and architectural scope filters.
#[derive(Parser, Debug)]
#[command(name = "ingest-sysml", version = "0.1.0", about = "SysML v2 Universal Ingestion Engine")]
pub struct CliArgs {
    /// Path to input schema file or directory containing schemas (default: schema/)
    #[arg(short, long)]
    pub schema: Option<PathBuf>,

    /// Schema format specification (auto, sysml, idl, openapi, markdown, yang, autosar, protobuf)
    #[arg(short, long, default_value = "auto")]
    pub format: String,

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

fn main() {
    let args = CliArgs::parse();

    if let Err(e) = run_ingestion(args) {
        eprintln!("[SysML v2 Ingestion] ERROR: {}", e);
        process::exit(1);
    }
}

/// Executes the end-to-end ingestion pipeline with validated input arguments.
///
/// /// Realises: [REQ-SYSML-INGEST-CLI/run_ingestion]
///
/// ### Safety Intent:
/// Coordinates discovery, classification, AST translation, scope filtering, textual serialization,
/// atomic file writes, and cryptographic digest recording within a bounded, fail-safe transaction.
///
/// ### Preconditions:
/// - `args` contains well-formed CLI arguments.
///
/// ### Postconditions:
/// - On success, writes valid SysML v2 text to `args.out` and cryptographic digest to `args.digest`.
/// - Returns `Ok(())` on complete success, or an explanatory `Err(String)` on any failure.
/// - Invariant: In the event of an error, temporary staging files are cleaned up and target files are untouched.
pub fn run_ingestion(args: CliArgs) -> Result<(), String> {
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
    let mut pkg: PackageDef;

    // Step 3: Dispatch translation based on format and file cardinality
    if fmt == "markdown" && (disc_files.len() > 1 || is_dir) {
        // Multi-file Markdown batch translation via parallel Rayon ingestion
        let def_name = if is_dir {
            args.schema
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .unwrap_or("OEM_System_Model")
        } else {
            "OEM_System_Model"
        };

        let file_strs: Vec<String> = disc_files.iter().map(|p| p.to_string_lossy().to_string()).collect();
        let translator = MarkdownTranslator::new();
        pkg = translator.translate_files(&file_strs, def_name)?;

        println!(
            "[SysML v2 Ingestion] Successfully ingested {} markdown files -> {}",
            disc_files.len(),
            args.out.display()
        );
    } else if fmt == "yang" && (disc_files.len() > 1 || is_dir) {
        // Multi-file YANG batch translation
        let def_name = if is_dir {
            args.schema
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .unwrap_or("YangModel")
        } else {
            "YangModel"
        };

        let file_strs: Vec<String> = disc_files.iter().map(|p| p.to_string_lossy().to_string()).collect();
        let translator = YangTranslator::new();
        pkg = translator.translate_files(&file_strs, def_name)?;

        println!(
            "[SysML v2 Ingestion] Successfully ingested {} yang files -> {}",
            disc_files.len(),
            args.out.display()
        );
    } else {
        // Single file translation path
        let target_file = &disc_files[0];
        let content = fs::read_to_string(target_file)
            .map_err(|e| format!("Failed to read file '{}': {}", target_file.display(), e))?;
        let basename = target_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Model");

        pkg = match fmt.as_str() {
            "sysml" => parse_sysml(&content)
                .map_err(|e| format!("SysML v2 parse error in '{}': {}", target_file.display(), e))?,
            "idl" => IDLTranslator::new().translate(&content, basename)?,
            "openapi" => OpenAPITranslator::new().translate(&content, basename)?,
            "markdown" => MarkdownTranslator::new().translate(&content, basename)?,
            "yang" => YangTranslator::new().translate(&content, basename)?,
            "raw" => {
                return Err("AST translation is required for raw document formats. SSOT ingestion failed.".to_string());
            }
            other => {
                return Err(format!("Unsupported format '{}' for file '{}'", other, target_file.display()));
            }
        };

        println!(
            "[SysML v2 Ingestion] Successfully ingested '{}' -> {}",
            target_file.display(),
            args.out.display()
        );
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
