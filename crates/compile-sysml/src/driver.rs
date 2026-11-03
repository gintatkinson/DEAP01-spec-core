#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] Layer 2 & 4 Application Orchestration & Filesystem Target Resolution for `compile-sysml`.
//! ## 1. Safety Intent & Regulatory Scope
//! Safety & Traceability: Realises: [REQ-COMPILER-SYSML-DRIVER], [DO-330-TQL-4-CLI]
//! ## 2. Invariants & Mathematical Properties
//! Contract: Implements Single-Entry/Single-Exit (SESE) pipeline mode orchestration, unified fail-closed SysML v2 schema loading, and Track-1 CST codemod routing.
//! ## 3. Preconditions
//! ## 4. Postconditions
//! ## 5. Algorithmic Complexity Bounds
//! ## 6. Failure Modes & Fault Tolerance
//! Bounds & Faults: Operates in O(N log N) time and O(N) space; zero `std::process::exit` calls in library driver functions.
//! ## 7. Realization & Traceability

use crate::cli::{Cli, CommandMode, MessageFormat};
pub use crate::driver_cst_tracks::{
    emit_codemod_ndjson_diagnostic, emit_rust_item_inventory, extract_rust_module_atomic,
    format_driver_ndjson_line,
};
use crate::driver_cst_tracks::{
    handle_apply_machine_fixes, handle_compact_tier_bc, handle_extract_rust_module,
    handle_list_rust_items,
};
use crate::driver_mode_handlers::{
    handle_compile, handle_emit_flutter_contracts, handle_emit_modular_rust, handle_export_mbd,
    handle_extract_source, handle_forward_sync, handle_inject_docstrings, handle_reverse_sync,
    handle_slice_package, handle_stpa_to_constraints, handle_stpa_transpile,
};
use crate::driver_schema_loader::{
    load_and_validate_schema_typed, resolve_schema_targets_typed, run_compilation_gate_with_dry_run,
};
use crate::projections::ReverseSyncOptions;
use std::path::{Path, PathBuf};

/// Remediation message emitted when no `.sysml` schema file is found in `schema/` or `.pipeline/`.
/// ## 1. Safety Intent & Regulatory Scope
/// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-REMEDIATION-MSG], [DO-330-TQL-4-CLI]
/// ## 2. Invariants & Mathematical Properties
/// Contract: Supplies deterministic operator remediation instructions when default schema discovery finds zero `.sysml` files.
/// ## 3. Preconditions
/// ## 4. Postconditions
/// ## 5. Algorithmic Complexity Bounds
/// ## 6. Failure Modes & Fault Tolerance
/// Bounds & Faults: O(1) compile-time static UTF-8 string slice; cannot fail.
/// ## 7. Realization & Traceability
pub const SCHEMA_REMEDIATION_MESSAGE: &str = "\
Error: No .sysml schema file found in schema/ or .pipeline/schema.sysml.
If starting from unstructured OEM prose manuals, PDF documentation, or BOM markdown tables:
  1. Place your OEM documentation or extract tables into schema/ or schema/extracted/.
  2. Execute Step 0.0 Level 0 OEM Ground Truth Ingestion:
     ./target/release/ingest-sysml --schema <path_to_markdown> --format markdown --out .pipeline/schema.sysml
  3. Re-run ./target/release/compile-sysml --compile to satisfy the compilation gate.";

/// Typed operational error returned by internal `compile-sysml` pipeline and codegen handlers.
/// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-DRIVER], [DO-330-TQL-4-CLI]
/// Contract: Partitions CLI flag, target resolution, schema validation, filesystem I/O, and codegen failures into closed variants with deterministic process exit codes.
/// Bounds & Faults: Total value type; `Cli`, `Resolution`, `Validation`, and `Io` map to exit code `1`, while `Codegen(code, _)` preserves its subsystem exit code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriverError {
    /// Invalid or conflicting command-line flags or unexpected dispatch mode.
    Cli(String),
    /// Missing schema, scoring configuration, or input target path during resolution.
    Resolution(String),
    /// Empty schema, lexical/grammatical parse failure, zero elements, or semantic validation failure.
    Validation(String),
    /// Filesystem read or atomic write failure.
    Io(String),
    /// Downstream codegen, synchronization, STPA, or docstring synthesizer failure carrying an explicit exit code.
    Codegen(i32, String),
}

impl DriverError {
    /// Returns the human-readable diagnostic message carried by this error variant.
    /// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-DRIVER]
    /// Contract: Borrows the inner UTF-8 diagnostic string slice without cloning or reformatting.
    /// Bounds & Faults: O(1) time and O(1) space; total projection that never panics.
    pub fn message(&self) -> &str {
        match self {
            Self::Cli(msg)
            | Self::Resolution(msg)
            | Self::Validation(msg)
            | Self::Io(msg)
            | Self::Codegen(_, msg) => msg.as_str(),
        }
    }

    /// Returns the deterministic OS process exit code associated with this error variant.
    /// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-DRIVER], [DO-330-TQL-4-CLI]
    /// Contract: Maps `Cli`, `Resolution`, `Validation`, and `Io` to `1` and `Codegen(code, _)` to `code`.
    /// Bounds & Faults: O(1) compile-time `const fn` evaluation with zero heap allocation and zero panics.
    pub const fn exit_code(&self) -> i32 {
        match self {
            Self::Cli(_) | Self::Resolution(_) | Self::Validation(_) | Self::Io(_) => 1,
            Self::Codegen(code, _) => *code,
        }
    }
}

impl std::fmt::Display for DriverError {
    /// Formats `DriverError` by writing its underlying diagnostic message slice.
    /// Realises: [REQ-COMPILER-SYSML-DRIVER]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for DriverError {}

impl std::ops::Deref for DriverError {
    type Target = str;

    /// Dereferences `DriverError` to its inner diagnostic `str` slice for direct string inspection.
    /// Realises: [REQ-COMPILER-SYSML-DRIVER]
    fn deref(&self) -> &Self::Target {
        self.message()
    }
}

impl From<DriverError> for (i32, String) {
    /// Converts `DriverError` into an `(exit_code, diagnostic_message)` tuple at external CLI boundaries.
    /// Realises: [REQ-COMPILER-SYSML-DRIVER]
    fn from(err: DriverError) -> Self {
        let code = err.exit_code();
        let msg = match err {
            DriverError::Cli(m)
            | DriverError::Resolution(m)
            | DriverError::Validation(m)
            | DriverError::Io(m)
            | DriverError::Codegen(_, m) => m,
        };
        (code, msg)
    }
}

/// Target SysML schema input resolved from CLI flags or default paths.
/// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-SCHEMA-TARGET], [DO-330-TQL-4-CLI]
/// Contract: Distinguishes between a single `.sysml` schema file and a multi-file schema directory tree.
/// Bounds & Faults: O(1) variant inspection and O(F) space for discovered directory paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaTarget {
    /// A single concrete `.sysml` file path.
    SingleFile(PathBuf),
    /// A directory containing multiple discovered `.sysml` files.
    Directory(PathBuf, Vec<PathBuf>),
}

/// Resolves input schema path into either a single file or a directory containing `.sysml` files.
/// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-RESOLVE-TARGETS], [DO-330-TQL-4-CLI]
/// Contract: Normalizes singleton directories to `SchemaTarget::SingleFile` and falls back to `.pipeline/schema.sysml` when `schema/` is empty.
/// Bounds & Faults: O(F log F) time and O(F) space; returns descriptive `Err(String)` when no schema exists.
pub fn resolve_schema_targets(explicit_path: Option<PathBuf>) -> Result<SchemaTarget, String> {
    resolve_schema_targets_typed(explicit_path).map_err(|e| <(i32, String)>::from(e).1)
}

/// Resolves a single concrete schema file path for single-file callers.
/// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-RESOLVE-FILE], [DO-330-TQL-4-CLI]
/// Contract: Prioritizes `model.sysml` in multi-file directories or returns the lexicographically first `.sysml` path.
/// Bounds & Faults: O(F log F) time and O(F) space; propagates resolution errors as `Err(String)`.
pub fn resolve_schema_file(explicit_path: Option<PathBuf>) -> Result<PathBuf, String> {
    match resolve_schema_targets(explicit_path)? {
        SchemaTarget::SingleFile(p) => Ok(p),
        SchemaTarget::Directory(_, files) => {
            if let Some(model) = files
                .iter()
                .find(|p| p.file_name().is_some_and(|n| n == "model.sysml"))
            {
                Ok(model.clone())
            } else {
                files
                    .first()
                    .cloned()
                    .ok_or_else(|| SCHEMA_REMEDIATION_MESSAGE.to_string())
            }
        }
    }
}

/// Resolves, parses, checks structural element cardinality, and optionally validates semantics of a SysML schema.
/// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-LOAD-AND-VALIDATE-SCHEMA], [DO-330-TQL-4-GATE]
/// Contract: Serves as the unified fail-closed schema loading and semantic validation gate across all operational modes.
/// Bounds & Faults: O(N log N) time and O(N) space; fails closed on zero structural elements or `SemanticValidator` errors.
pub fn load_and_validate_schema(
    explicit_path: Option<PathBuf>,
    require_validation: bool,
) -> Result<(SchemaTarget, deap_core::sysml_cst::PackageDef, String), String> {
    load_and_validate_schema_typed(explicit_path, require_validation)
        .map_err(|e| <(i32, String)>::from(e).1)
}

/// Resolves, parses, and semantically validates a single-file or multi-file SysML schema target for CLI modes.
/// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-LOAD-PACKAGE-CLI], [DO-330-TQL-4-GATE]
/// Contract: Delegates directly to `load_and_validate_schema(explicit_path, true)`.
/// Bounds & Faults: O(N log N) time and O(N) space; returns `Err(String)` on any parse or semantic error.
pub fn load_schema_package_for_cli(
    explicit_path: Option<PathBuf>,
) -> Result<(SchemaTarget, deap_core::sysml_cst::PackageDef, String), String> {
    load_and_validate_schema(explicit_path, true)
}

/// Executes the Pipeline 0 compilation gate.
/// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-GATE], [DO-330-TQL-4-GATE]
/// Contract: Validates the input schema, persists canonical SysML text to `output_path`, and writes the SHA-256 `SchemaDigest` to `digest_path`.
/// Bounds & Faults: O(N log N) time and O(N) space; returns `0` on success or `1` after logging errors to `stderr`.
pub fn run_compilation_gate(
    explicit_schema: Option<PathBuf>,
    output_path: &Path,
    digest_path: &Path,
) -> i32 {
    match run_compilation_gate_with_dry_run(explicit_schema, output_path, digest_path, false) {
        Ok(code) => code,
        Err(err) => {
            let (code, msg) = <(i32, String)>::from(err);
            eprintln!("{msg}");
            code
        }
    }
}

/// Dispatches STPA, synchronization, compilation, and usage `CommandMode` variants.
/// Realises: [REQ-COMPILER-SYSML-DISPATCH-PIPELINE]
pub(crate) fn dispatch_pipeline_mode(mode: CommandMode) -> Result<i32, DriverError> {
    match mode {
        CommandMode::StpaTranspile { schema, out_dir, scoring_config, dry_run } => {
            handle_stpa_transpile(schema, &out_dir, scoring_config.as_deref(), dry_run)
        }
        CommandMode::StpaToConstraints { stpa_file, schema, out, dry_run } => {
            handle_stpa_to_constraints(&stpa_file, schema, &out, dry_run)
        }
        CommandMode::ForwardSync { schema, docs, out_dir, dry_run, force } => {
            handle_forward_sync(schema, docs, out_dir, dry_run, force)
        }
        CommandMode::ReverseSync { schema, docs, out, digest, force, dry_run } => {
            handle_reverse_sync(
                ReverseSyncOptions {
                    docs_dir: docs,
                    schema_path: schema,
                    output_path: out,
                    digest_path: digest,
                    allow_overwrite: force,
                },
                dry_run,
            )
        }
        CommandMode::Compile { schema, out, digest, dry_run } => {
            handle_compile(schema, &out, &digest, dry_run)
        }
        CommandMode::ShowUsage => {
            println!("compile-sysml: Specify --compile, --stpa-transpile, --stpa, --forward-sync, --reverse-sync, --inject-safety-docstrings, or --extract-source. Run with --help for usage.");
            Ok(0)
        }
        _ => Err(DriverError::Cli("Internal error: unexpected command mode".to_string())),
    }
}

/// Dispatches Track-1 CST codemod `CommandMode` variants, forwarding remaining modes to `dispatch_pipeline_mode`.
/// Realises: [REQ-0608], [REQ-COMPILER-SYSML-EXECUTE]
pub(crate) fn dispatch_cst_mode(
    mode: CommandMode,
    message_format: MessageFormat,
) -> Result<i32, DriverError> {
    match mode {
        CommandMode::ListRustItems { target_file } => {
            handle_list_rust_items(&target_file, message_format)
        }
        CommandMode::ExtractRustModule { source_file, dest_file, items_csv, dry_run } => {
            handle_extract_rust_module(&source_file, &dest_file, &items_csv, dry_run)
        }
        CommandMode::CompactTierBc { target_path, dry_run } => {
            handle_compact_tier_bc(&target_path, dry_run)
        }
        CommandMode::ApplyMachineFixes { target_path, schema, dry_run, force } => {
            handle_apply_machine_fixes(&target_path, schema, dry_run, force)
        }
        other => dispatch_pipeline_mode(other),
    }
}

/// Dispatches slicing and modular code-generation `CommandMode` variants, delegating remaining modes to `dispatch_cst_mode`.
/// Realises: [REQ-0607], [REQ-COMPILER-SYSML-EXECUTE]
pub(crate) fn dispatch_codegen_mode(
    mode: CommandMode,
    message_format: MessageFormat,
) -> Result<i32, DriverError> {
    match mode {
        CommandMode::SlicePackage { target_pkg, schema } => handle_slice_package(&target_pkg, schema),
        CommandMode::EmitFlutterContracts { target_dir, schema, dry_run } => {
            handle_emit_flutter_contracts(&target_dir, schema, dry_run)
        }
        CommandMode::EmitModularRust { target_dir, schema, dry_run } => {
            handle_emit_modular_rust(&target_dir, schema, dry_run)
        }
        other => dispatch_cst_mode(other, message_format),
    }
}

/// Dispatches a parsed `Cli` configuration to its typed SESE handler returning `Result<i32, DriverError>`.
/// Realises: [REQ-COMPILER-SYSML-EXECUTE]
pub(crate) fn dispatch_cli(cli: Cli) -> Result<i32, DriverError> {
    let message_format = cli.message_format;
    let mode = cli.into_mode().map_err(DriverError::Cli)?;
    match mode {
        CommandMode::InjectDocstrings { target_dir, schema, dry_run, force } => {
            handle_inject_docstrings(&target_dir, schema, dry_run, force)
        }
        CommandMode::ExtractSource { extract_dir, out, digest, dry_run } => {
            handle_extract_source(&extract_dir, &out, &digest, dry_run)
        }
        CommandMode::ExportMbd { mbd_dir, schema, dry_run } => {
            handle_export_mbd(&mbd_dir, schema, dry_run)
        }
        other => dispatch_codegen_mode(other, message_format),
    }
}

/// Executes a parsed `Cli` configuration in-process without calling `std::process::exit`.
/// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-EXECUTE], [DO-330-TQL-4-CLI]
/// Contract: Dispatches `cli` to its resolved `CommandMode` handler and returns `Ok(exit_code)` or `Err((exit_code, message))`.
/// Bounds & Faults: O(N log N) time and O(N) space; never calls `std::process::exit`.
pub fn execute(cli: Cli) -> Result<i32, (i32, String)> {
    dispatch_cli(cli).map_err(<(i32, String)>::from)
}

/// Executes a parsed `Cli` configuration, emitting human or NDJSON diagnostics and returning the exit code.
/// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-EXECUTE-CLI], [DO-330-TQL-4-CLI]
/// Contract: Bridges `execute(cli)` to `src/main.rs` and emits `NdjsonDiagnostic` lines when `MessageFormat::Json` is active.
/// Bounds & Faults: O(N log N) time and O(N) space; zero `std::process::exit` calls.
pub fn execute_cli(cli: Cli) -> i32 {
    let format = cli.message_format;
    match execute(cli) {
        Ok(code) => {
            if format == MessageFormat::Json {
                let line = format_driver_ndjson_line("CLI-OK", "info", "ok", ".", false);
                println!("{line}");
            }
            code
        }
        Err((code, msg)) => {
            if format == MessageFormat::Json {
                let line = format_driver_ndjson_line("CLI-ERR", "error", &msg, ".", false);
                println!("{line}");
            }
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
            code
        }
    }
}

#[cfg(test)]
#[path = "driver_tests.rs"]
mod tests;
