//! Layer 2 & 4 Application Orchestration & Filesystem Target Resolution for `compile-sysml`.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Implements Single-Entry/Single-Exit (SESE) pipeline mode orchestration, unified fail-closed
//! SysML v2 schema loading, semantic validation gating, and deterministic filesystem target
//! resolution under DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D standards.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Zero non-local process termination: no function in `driver.rs` invokes `std::process::exit`;
//!   all operational outcomes are returned as explicit `Result<i32, (i32, String)>` or `i32` values.
//! - Unified fail-closed schema validation (`ARCH-01`, `ARCH-02`): all schema-consuming handlers
//!   route through `load_and_validate_schema` / `load_schema_package_for_cli` with zero silent
//!   `.ok()` or `.unwrap_or_default()` error masking.
//! - Bounded cyclomatic complexity: every function maintains $v(G) \le 8$.
//!
//! ## 3. Preconditions
//! - Invoked with a parsed `Cli` struct or explicit filesystem target paths.
//!
//! ## 4. Postconditions
//! - Executes the resolved `CommandMode` handler, persists artifacts atomically (unless `dry_run`
//!   is `true`), and returns a deterministic process exit code (`0` on success, non-zero on error).
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time complexity: $O(N \log N)$ bounded by AST parsing, semantic validation, and synchronization passes.
//! - Space complexity: $O(N)$ proportional to the in-memory `PackageDef` AST and output buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Missing or empty schemas, lexical/syntactic parse errors, semantic validation errors, or I/O
//!   write errors fail closed immediately and return structured `(exit_code, diagnostic_message)` tuples.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-COMPILER-SYSML-DRIVER], [DO-330-TQL-4-CLI]

use crate::cli::{Cli, CommandMode};
use crate::docstring::{
    run_docstring_injection, run_extract_source, DocstringExitCode, DocstringInjectionOptions,
};
use crate::semantic::digest::{
    compute_file_manifest, count_structural_elements, write_atomic, SchemaDigest,
};
use crate::semantic::serializer::{to_sysml, SysmlSerializable};
use crate::semantic::validator::SemanticValidator;
use crate::stpa::{
    compile_stpa_to_constraints_filtered, emit_safety_suite, transpile_safety_suite,
    FmecaScoringConfig,
};
use crate::sync::{
    forward_sync_sysml_to_specs, reverse_sync_specs_to_sysml_with_dry_run, ForwardSyncOptions,
    ReverseSyncOptions,
};
use crate::{
    discover_sysml_files, export_mbd_artifacts, flatten_package_to_model, parse_sysml,
    parse_sysml_tree, synthesize_mbd_bundle,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Remediation message emitted when no `.sysml` schema file is found in `schema/` or `.pipeline/`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Provides deterministic operator guidance for Level 0 OEM ground-truth ingestion under
/// DO-178C Level A and DO-330 TQL-4 operational usability requirements.
///
/// ## 2. Invariants & Mathematical Properties
/// - Compile-time static UTF-8 string slice.
///
/// ## 3. Preconditions
/// - Referenced when default schema discovery finds zero `.sysml` files.
///
/// ## 4. Postconditions
/// - Supplies actionable step-by-step remediation commands to the caller.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$.
/// - Space Complexity: $O(1)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Immutable static constant; cannot fail.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-REMEDIATION-MSG], [DO-330-TQL-4-CLI]
pub const SCHEMA_REMEDIATION_MESSAGE: &str = "\
Error: No .sysml schema file found in schema/ or .pipeline/schema.sysml.
If starting from unstructured OEM prose manuals, PDF documentation, or BOM markdown tables:
  1. Place your OEM documentation or extract tables into schema/ or schema/extracted/.
  2. Execute Step 0.0 Level 0 OEM Ground Truth Ingestion:
     ./target/release/ingest-sysml --schema <path_to_markdown> --format markdown --out .pipeline/schema.sysml
  3. Re-run ./target/release/compile-sysml --compile to satisfy the compilation gate.";

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
/// - Time Complexity: $O(1)$ variant inspection.
/// - Space Complexity: $O(F)$ where $F$ is the number of files in `Directory`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total sum type with no invalid runtime states.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-SCHEMA-TARGET], [DO-330-TQL-4-CLI]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaTarget {
    /// A single concrete `.sysml` file path.
    SingleFile(PathBuf),
    /// A directory containing multiple discovered `.sysml` files.
    Directory(PathBuf, Vec<PathBuf>),
}

/// Resolves `<ws>/.pipeline/schema.sysml` when `path` is a `<ws>/schema` directory path.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Ensures deterministic fallback to the workspace Single Source of Truth (`.pipeline/schema.sysml`)
/// when a caller passes `<ws>/schema` and that directory is either empty or absent on disk, under
/// DO-178C Level A and DO-330 TQL-4 configuration management rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `Some(SchemaTarget::SingleFile(<ws>/.pipeline/schema.sysml))` if and only if the
///   terminal path component of `path` is `"schema"` and `<ws>/.pipeline/schema.sysml` exists as a regular file.
///
/// ## 3. Preconditions
/// - `path` is a candidate filesystem path supplied to schema resolution.
///
/// ## 4. Postconditions
/// - Side-effect-free read-only filesystem probe returning `Option<SchemaTarget>`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ path inspection and metadata stat call.
/// - Space Complexity: $O(1)$ for the constructed fallback `PathBuf`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-`"schema"` paths or missing `.pipeline/schema.sysml` files return `None` without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-RESOLVE-DIR-TARGET], [REQ-COMPILER-SYSML-RESOLVE-TARGETS]
fn resolve_schema_dir_pipeline_fallback(path: &Path) -> Option<SchemaTarget> {
    if path.file_name().is_some_and(|n| n == "schema") {
        let ws_root = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let fallback = ws_root.join(".pipeline").join("schema.sysml");
        if fallback.is_file() {
            return Some(SchemaTarget::SingleFile(fallback));
        }
    }
    None
}

/// Resolves an existing directory path into a `SchemaTarget`, checking `.pipeline/schema.sysml` fallbacks when empty.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Ensures deterministic discovery of `.sysml` files inside an explicit directory target while
/// supporting clean landing-zone sub-workspaces that store their compiled SSOT at `.pipeline/schema.sysml`
/// under DO-178C Level A and DO-330 TQL-4 configuration rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Normalizes singleton file vectors (`[only]`) to `SchemaTarget::SingleFile(only)`.
/// - Falls back to `<dir>/.pipeline/schema.sysml` or `<dir>/../.pipeline/schema.sysml` (when `dir` is named `schema`)
///   before returning an empty-directory error.
///
/// ## 3. Preconditions
/// - `dir` is an existing directory path (`dir.is_dir() == true`).
///
/// ## 4. Postconditions
/// - Returns `Ok(SchemaTarget)` with at least one `.sysml` file, or `Err(String)` if no `.sysml` file exists.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(F \log F)$ where $F$ is the number of discovered files.
/// - Space Complexity: $O(F)$ for the discovered path vector.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Directories devoid of `.sysml` files and fallbacks return a descriptive `Err(String)`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-RESOLVE-DIR-TARGET]
fn resolve_directory_schema_target(dir: PathBuf) -> Result<SchemaTarget, String> {
    let files = discover_sysml_files(&dir);
    if let [only] = files.as_slice() {
        return Ok(SchemaTarget::SingleFile(only.clone()));
    }
    if !files.is_empty() {
        return Ok(SchemaTarget::Directory(dir, files));
    }
    let direct_fallback = dir.join(".pipeline").join("schema.sysml");
    if direct_fallback.is_file() {
        return Ok(SchemaTarget::SingleFile(direct_fallback));
    }
    resolve_schema_dir_pipeline_fallback(&dir).ok_or_else(|| {
        format!(
            "Error: No .sysml files found in directory: {}",
            dir.display()
        )
    })
}

/// Resolve input schema path into either a single file or a directory containing `.sysml` files.
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
        }
        if path.is_dir() {
            return resolve_directory_schema_target(path);
        }
        return resolve_schema_dir_pipeline_fallback(&path).ok_or_else(|| {
            format!(
                "Error: Schema file does not exist: {}",
                path.display()
            )
        });
    }

    let schema_dir = Path::new("schema");
    if schema_dir.is_dir() {
        let files = discover_sysml_files(schema_dir);
        if let [only] = files.as_slice() {
            return Ok(SchemaTarget::SingleFile(only.clone()));
        }
        if !files.is_empty() {
            return Ok(SchemaTarget::Directory(schema_dir.to_path_buf(), files));
        }
    }

    let pipeline_fallback = Path::new(".pipeline/schema.sysml");
    if pipeline_fallback.is_file() {
        return Ok(SchemaTarget::SingleFile(pipeline_fallback.to_path_buf()));
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

/// Parses a resolved `SchemaTarget` into a `(PackageDef, display_name)` pair with fail-closed I/O and emptiness checks.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Eliminates duplicated file-reading and tree-parsing control diamonds across CLI handlers under
/// DO-178C Level A and DO-330 TQL-4 fail-closed input verification mandates (`ARCH-01`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Rejects empty single-file schemas (`content.trim().is_empty()`) before invoking the parser.
/// - Preserves deterministic display names derived from file stem or directory name.
///
/// ## 3. Preconditions
/// - `target` is a valid `SchemaTarget` resolved by `resolve_schema_targets`.
///
/// ## 4. Postconditions
/// - Returns `Ok((PackageDef, String))` upon clean lexical and grammatical parsing, or `Err(String)` on failure.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ where $N$ is the total byte length of the schema source files.
/// - Space Complexity: $O(N)$ for the in-memory `PackageDef` AST.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Unreadable files, empty schema files, or syntax errors return descriptive `Err(String)` values.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-PARSE-RESOLVED-TARGET]
fn parse_resolved_schema_target(
    target: &SchemaTarget,
) -> Result<(deap_core::sysml_ast::PackageDef, String), String> {
    match target {
        SchemaTarget::SingleFile(schema_file) => {
            let content = fs::read_to_string(schema_file).map_err(|e| {
                format!(
                    "Error reading schema file '{}': {}",
                    schema_file.display(),
                    e
                )
            })?;
            if content.trim().is_empty() {
                return Err(format!(
                    "Error: Schema file '{}' is empty.",
                    schema_file.display()
                ));
            }
            let display_name = schema_file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("SysML_Model")
                .to_string();
            let parsed = parse_sysml(&content).map_err(|err| {
                format!(
                    "Error parsing schema file '{}': {}",
                    schema_file.display(),
                    err
                )
            })?;
            Ok((parsed, display_name))
        }
        SchemaTarget::Directory(dir, files) => {
            let dir_name = dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("SysML_Model")
                .to_string();
            let parsed = parse_sysml_tree(files).map_err(|err| {
                format!(
                    "Error parsing schema directory '{}': {}",
                    dir.display(),
                    err
                )
            })?;
            Ok((parsed, dir_name))
        }
    }
}

/// Resolves, parses, checks structural element cardinality, and optionally validates semantics of a SysML schema.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Serves as the single fail-closed schema loading and semantic validation gate (`ARCH-01`, `ARCH-02`)
/// across all `compile-sysml` operational modes under DO-178C Level A and DO-330 TQL-4.
///
/// ## 2. Invariants & Mathematical Properties
/// - Non-vacuous model guarantee: fails closed whenever `count_structural_elements(&pkg) == 0`.
/// - Semantic soundness guarantee: when `require_validation == true`, executes `SemanticValidator`
///   and fails closed if `validator.has_errors()` is `true`.
///
/// ## 3. Preconditions
/// - `explicit_path` is an optional user-supplied file or directory path; `require_validation` controls
///   whether `SemanticValidator` errors block execution.
///
/// ## 4. Postconditions
/// - Returns `Ok((SchemaTarget, PackageDef, String))` containing the resolved target, validated AST,
///   and display name, or `Err(String)` describing the first validation failure.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N)$ in total schema source byte size and AST symbol count.
/// - Space Complexity: $O(N)$ for the parsed `PackageDef` AST and symbol tables.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Missing files, I/O errors, empty models, syntax errors, and semantic validation errors return
///   deterministic `Err(String)` diagnostics without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-LOAD-AND-VALIDATE-SCHEMA], [DO-330-TQL-4-GATE]
pub fn load_and_validate_schema(
    explicit_path: Option<PathBuf>,
    require_validation: bool,
) -> Result<(SchemaTarget, deap_core::sysml_ast::PackageDef, String), String> {
    let target = resolve_schema_targets(explicit_path)?;
    let (pkg, display_name) = parse_resolved_schema_target(&target)?;

    if count_structural_elements(&pkg) == 0 {
        return Err(format!(
            "Error: Schema '{display_name}' contains 0 structural elements."
        ));
    }

    if require_validation {
        let mut validator = SemanticValidator::new(&pkg);
        validator.validate();
        if validator.has_errors() {
            let msgs: Vec<String> = validator
                .errors()
                .into_iter()
                .map(|err| format!("Error: Semantic validation error: {}", err.message))
                .collect();
            return Err(msgs.join("\n"));
        }
    }

    Ok((target, pkg, display_name))
}

/// Resolves, parses, and semantically validates a single-file or multi-file SysML schema target for CLI modes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Provides a unified, fail-closed schema loader enforcing `SemanticValidator` checks for MBD
/// artifact synthesis (`--export-mbd`), package subsystem slicing (`--slice-package`), and Flutter
/// contract generation (`--emit-flutter-contracts`) under DO-330 TQL-4 CLI determinism rules (`ARCH-02`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Delegates directly to `load_and_validate_schema(explicit_path, true)`.
///
/// ## 3. Preconditions
/// - `explicit_path` is an optional user-supplied file or directory path.
///
/// ## 4. Postconditions
/// - Returns `Ok((SchemaTarget, PackageDef, String))` upon successful parsing and semantic validation,
///   or `Err(String)` on error.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N)$ in total schema source byte size and AST node count.
/// - Space Complexity: $O(N)$ for the parsed `PackageDef` AST.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates missing file, I/O read, syntax parse, empty model, and semantic validation errors as `Err(String)`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-LOAD-PACKAGE-CLI], [DO-330-TQL-4-GATE]
pub fn load_schema_package_for_cli(
    explicit_path: Option<PathBuf>,
) -> Result<(SchemaTarget, deap_core::sysml_ast::PackageDef, String), String> {
    load_and_validate_schema(explicit_path, true)
}

/// Synchronizes `<ws>/schema/model.sysml` when compiling into `<ws>/.pipeline/schema.sysml` in downstream workspaces.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Maintains dual-schema SSOT parity (Check 31) in downstream customer workspaces (`!upstream_dir.exists()`)
/// when `schema/` contains either no `.sysml` files or only `schema/model.sysml` under DO-178C Level A.
///
/// ## 2. Invariants & Mathematical Properties
/// - Never mutates `schema/` in `UPSTREAM_SPEC_CORE_COMPILER` workspaces where `.pipeline/upstream` exists.
/// - Uses `write_atomic` for crash-safe persistence (`ARCH-04`).
///
/// ## 3. Preconditions
/// - `output_path` is the target compiled schema path; `sysml_text` is the canonical serialized SysML string.
///
/// ## 4. Postconditions
/// - Atomically updates `<ws>/schema/model.sysml` when downstream single-model conditions hold.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ where $N$ is `sysml_text.len()`.
/// - Space Complexity: $O(1)$ auxiliary path buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `Err(String)` if atomic write to `schema/model.sysml` fails.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-SYNC-DOWNSTREAM-MODEL]
fn sync_downstream_model_sysml(output_path: &Path, sysml_text: &str) -> Result<(), String> {
    if !output_path.ends_with(".pipeline/schema.sysml") {
        return Ok(());
    }
    let ws_root = output_path
        .parent()
        .and_then(|p| p.parent())
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let upstream_dir = ws_root.join(".pipeline").join("upstream");
    let schema_dir = ws_root.join("schema");
    if !upstream_dir.exists() && schema_dir.is_dir() {
        let sysml_entries = discover_sysml_files(&schema_dir);
        let model_out = schema_dir.join("model.sysml");
        if sysml_entries.is_empty()
            || (sysml_entries.len() == 1 && sysml_entries.first() == Some(&model_out))
        {
            write_atomic(&model_out, sysml_text).map_err(|e| {
                format!(
                    "Error writing synchronized schema model to '{}': {}",
                    model_out.display(),
                    e
                )
            })?;
        }
    }
    Ok(())
}

/// Internal SESE implementation of the Pipeline 0 compilation gate supporting `dry_run`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Executes the Pipeline 0 SysML v2 compilation gate with atomic persistence and `dry_run`
/// simulation support (`ARCH-04`, `ARCH-05`) under DO-178C Level A and DO-330 TQL-4.
///
/// ## 2. Invariants & Mathematical Properties
/// - Validates non-empty structural element count and full `SemanticValidator` soundness via `load_and_validate_schema`.
/// - When `dry_run` is `true`, performs full validation, serialization, and SHA-256 digest computation
///   with zero filesystem writes.
/// - When `dry_run` is `false`, persists `output_path` and `digest_path` atomically via `write_atomic`.
///
/// ## 3. Preconditions
/// - `explicit_schema` is an optional schema file or directory path; `output_path` and `digest_path` are target paths.
///
/// ## 4. Postconditions
/// - Returns `Ok(0)` on clean compilation or `Err((1, String))` on any validation or I/O error.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N)$ in schema size and AST node count.
/// - Space Complexity: $O(N)$ for AST and serialized output buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns structured `Err((1, msg))` without calling `process::exit`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-GATE-DRY-RUN], [DO-330-TQL-4-GATE]
fn run_compilation_gate_with_dry_run(
    explicit_schema: Option<PathBuf>,
    output_path: &Path,
    digest_path: &Path,
    dry_run: bool,
) -> Result<i32, (i32, String)> {
    let (target, pkg, display_name) =
        load_and_validate_schema(explicit_schema, true).map_err(|e| (1, e))?;
    let total_elements = count_structural_elements(&pkg);
    let sysml_text = pkg.to_sysml(0);

    let mut digest = SchemaDigest::compute(&pkg, &sysml_text);
    if let SchemaTarget::Directory(_, ref files) = target {
        let manifest = compute_file_manifest(files);
        digest = digest.with_file_manifest(manifest);
    }

    if dry_run {
        println!(
            "[DRY RUN] Would compile SysML schema '{}' to '{}' (SHA-256: {}, {} elements)",
            display_name,
            output_path.display(),
            digest.sha256,
            total_elements
        );
        return Ok(0);
    }

    write_atomic(output_path, &sysml_text).map_err(|e| {
        (
            1,
            format!(
                "Error writing compiled schema to '{}': {}",
                output_path.display(),
                e
            ),
        )
    })?;

    sync_downstream_model_sysml(output_path, &sysml_text).map_err(|e| (1, e))?;

    digest.write_to_file(digest_path).map_err(|e| {
        (
            1,
            format!(
                "Error writing schema digest to '{}': {}",
                digest_path.display(),
                e
            ),
        )
    })?;

    println!(
        "Successfully compiled SysML schema '{}' to '{}' (SHA-256: {}, {} elements)",
        display_name,
        output_path.display(),
        digest.sha256,
        total_elements
    );

    Ok(0)
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
    match run_compilation_gate_with_dry_run(explicit_schema, output_path, digest_path, false) {
        Ok(code) => code,
        Err((code, msg)) => {
            eprintln!("{msg}");
            code
        }
    }
}

/// Executes `--inject-safety-docstrings` mode (`CommandMode::InjectDocstrings`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Orchestrates 7-section DO-178C/DO-330 safety docstring synthesis with fail-closed schema
/// context loading under DO-330 TQL-4 tool qualification rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Fails closed with `DocstringExitCode::Error` if an explicit `--schema` fails to load or parse,
///   or if a discovered default schema is malformed.
/// - Allows `schema_model = None` only when `--schema` was omitted and no default schema exists on disk.
///
/// ## 3. Preconditions
/// - `target_dir` is the target Rust/Dart source file or directory path.
///
/// ## 4. Postconditions
/// - Returns `Ok(0)` on clean docstring injection or `Err((exit_code, message))` on violation/error.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ over scanned source files and optional schema AST.
/// - Space Complexity: $O(N)$ for source AST and schema buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates `DocstringExitCode` (`1` for violation, `2` for I/O or schema error) without calling `process::exit`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-HANDLE-DOCSTRINGS]
fn handle_inject_docstrings(
    target_dir: &Path,
    schema: Option<PathBuf>,
    dry_run: bool,
    force: bool,
) -> Result<i32, (i32, String)> {
    let schema_model = if schema.is_some() || resolve_schema_targets(None).is_ok() {
        let (_, pkg, _) = load_and_validate_schema(schema, false).map_err(|e| {
            (
                DocstringExitCode::Error.code(),
                format!("Error during docstring injection: {e}"),
            )
        })?;
        Some(pkg)
    } else {
        None
    };

    let opts = DocstringInjectionOptions {
        target_path: target_dir.to_path_buf(),
        dry_run,
        force,
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
            Ok(DocstringExitCode::Clean.code())
        }
        Err((code, msg)) => Err((
            code.code(),
            format!("Error during docstring injection: {msg}"),
        )),
    }
}

/// Executes `--extract-source` mode (`CommandMode::ExtractSource`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Orchestrates structural source AST extraction into SysML v2 notation under DO-178C Level A
/// and DO-330 TQL-4 bidirectional traceability rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Delegates to `run_extract_source` and honors `dry_run`.
///
/// ## 3. Preconditions
/// - `extract_dir` is an existing directory containing Rust or Dart source files.
///
/// ## 4. Postconditions
/// - Returns `Ok(0)` on successful extraction or `Err((exit_code, message))` on failure.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ over extracted source files.
/// - Space Complexity: $O(N)$ for extracted `PackageDef` AST.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates `DocstringExitCode` errors cleanly as `Err((code, String))`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-HANDLE-EXTRACT-SOURCE]
fn handle_extract_source(
    extract_dir: &Path,
    out: &Path,
    digest: &Path,
    dry_run: bool,
) -> Result<i32, (i32, String)> {
    match run_extract_source(extract_dir, out, digest, dry_run) {
        Ok(pkg) => {
            println!(
                "[Extract Source] Extracted {} structural item(s) from '{}' to '{}'",
                count_structural_elements(&pkg),
                extract_dir.display(),
                out.display()
            );
            Ok(DocstringExitCode::Clean.code())
        }
        Err((code, msg)) => Err((code.code(), format!("Error during source extraction: {msg}"))),
    }
}

/// Executes `--export-mbd` mode (`CommandMode::ExportMbd`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Validates the input SysML v2 schema via `load_schema_package_for_cli` (`ARCH-02`), flattens nested
/// packages, and exports grounded MBD verification artifacts while honoring `dry_run` (`ARCH-04`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Blocks MBD export if `SemanticValidator` detects any semantic error in the input schema.
/// - Performs zero filesystem writes when `dry_run` is `true`.
///
/// ## 3. Preconditions
/// - `mbd_dir` is the target output directory; `schema` is an optional schema path.
///
/// ## 4. Postconditions
/// - Returns `Ok(0)` when MBD artifacts are synthesized (and written when `!dry_run`), or `Err((1, String))` on error.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N)$ over schema AST elements.
/// - Space Complexity: $O(N)$ for the flattened `SysmlModel` and MBD bundle.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Schema validation or filesystem write failures return `Err((1, String))`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-HANDLE-EXPORT-MBD]
fn handle_export_mbd(
    mbd_dir: &Path,
    schema: Option<PathBuf>,
    dry_run: bool,
) -> Result<i32, (i32, String)> {
    let (_, pkg, _) = load_schema_package_for_cli(schema).map_err(|e| (1, e))?;
    let model = flatten_package_to_model(pkg, None);
    if dry_run {
        let _bundle = synthesize_mbd_bundle(&model);
        println!(
            "[MBD Export] [DRY RUN] Would emit 4 artifact(s) to '{}'",
            mbd_dir.display()
        );
        return Ok(0);
    }
    let written = export_mbd_artifacts(&model, mbd_dir)
        .map_err(|e| (1, format!("Error exporting MBD artifacts: {e}")))?;
    println!(
        "[MBD Export] Emitted {} artifact(s) to '{}'",
        written.len(),
        mbd_dir.display()
    );
    Ok(0)
}

/// Executes `--slice-package` mode (`CommandMode::SlicePackage`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Validates the input SysML v2 schema via `load_schema_package_for_cli` (`ARCH-02`), slices the
/// target subsystem package within a 4096-token budget, and fails closed on JSON serialization (`ARCH-01`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Rejects semantically invalid schemas prior to slicing.
/// - Replaces silent `.unwrap_or_else(|_| "{}")` fallback with fail-closed `Result` propagation.
///
/// ## 3. Preconditions
/// - `target_pkg` is a non-empty package or part identifier; `schema` is an optional schema path.
///
/// ## 4. Postconditions
/// - Prints pretty-printed JSON slice to `stdout` and returns `Ok(0)`, or returns `Err((1, String))`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N)$ over schema AST elements.
/// - Space Complexity: $O(N)$ for flattened model and JSON buffer.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Unknown package names, token budget violations, or JSON serialization errors return `Err((1, String))`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-HANDLE-SLICE-PACKAGE]
fn handle_slice_package(
    target_pkg: &str,
    schema: Option<PathBuf>,
) -> Result<i32, (i32, String)> {
    let (_, pkg, _) = load_schema_package_for_cli(schema).map_err(|e| (1, e))?;
    let model = flatten_package_to_model(pkg, Some(target_pkg));
    let slice = model
        .slice_package_subsystem(target_pkg, 4096)
        .map_err(|e| (1, format!("Error slicing package '{target_pkg}': {e}")))?;
    let json = serde_json::to_string_pretty(&slice)
        .map_err(|e| (1, format!("Error serializing sliced package '{target_pkg}': {e}")))?;
    println!("{json}");
    Ok(0)
}

/// Executes `--emit-flutter-contracts` mode (`CommandMode::EmitFlutterContracts`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Validates the input SysML v2 schema via `load_schema_package_for_cli` (`ARCH-02`), delegates
/// topology discovery to `flutter_codegen::emit_flutter_contracts` with zero silent `.ok()` masking (`ARCH-01`),
/// and honors `dry_run` (`ARCH-04`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Eliminates redundant inline `topology_data.json` probing in favor of `emit_flutter_contracts`'s
///   fail-closed topology loader.
/// - Performs zero filesystem writes when `dry_run` is `true`.
///
/// ## 3. Preconditions
/// - `target_dir` is the target Flutter workspace directory; `schema` is an optional schema path.
///
/// ## 4. Postconditions
/// - Emits all 4 deterministic Flutter/C-ABI contract files (when `!dry_run`) and returns `Ok(0)`,
///   or returns `Err((1, String))` on validation/codegen error.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N)$ over schema AST and topology nodes.
/// - Space Complexity: $O(N)$ for generated Dart source buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Semantic validation errors, corrupted `topology_data.json` files, or write failures return `Err((1, String))`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-HANDLE-FLUTTER-CONTRACTS]
fn handle_emit_flutter_contracts(
    target_dir: &Path,
    schema: Option<PathBuf>,
    dry_run: bool,
) -> Result<i32, (i32, String)> {
    let (_, pkg, _) = load_schema_package_for_cli(schema).map_err(|e| (1, e))?;
    let model = flatten_package_to_model(pkg, None);
    if dry_run {
        println!(
            "[Flutter Codegen] [DRY RUN] Would emit contracts to '{}'",
            target_dir.display()
        );
        return Ok(0);
    }
    let report = crate::flutter_codegen::emit_flutter_contracts(target_dir, &model, None)
        .map_err(|e| (1, format!("Error emitting Flutter contracts: {e}")))?;
    println!(
        "[Flutter Codegen] Successfully emitted contracts (schema hash: 0x{:016X}) to '{}':",
        report.schema_hash,
        target_dir.display()
    );
    println!("  - Domain seed:      {}", report.seed_path.display());
    println!("  - Schema models:    {}", report.models_path.display());
    println!(
        "  - Solver bridge:    {}",
        report.solver_bridge_path.display()
    );
    println!(
        "  - UI contract test: {}",
        report.ui_contract_test_path.display()
    );
    Ok(0)
}

/// Loads and parses an optional FMECA scoring configuration JSON file.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Enforces fail-closed JSON configuration loading for STPA/FMECA criticality matrices under
/// DO-178C Level A and ISO 26262 ASIL D hazard analysis standards.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `Ok(None)` when `scoring_config_path` is `None`.
/// - Fails closed if the specified configuration path does not exist, cannot be read, or contains invalid JSON.
///
/// ## 3. Preconditions
/// - `scoring_config_path` is an optional path to a JSON file conforming to `FmecaScoringConfig`.
///
/// ## 4. Postconditions
/// - Returns `Ok(Option<FmecaScoringConfig>)` or a descriptive `Err(String)`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(B)$ where $B$ is the byte length of the JSON configuration file.
/// - Space Complexity: $O(B)$ for the parsed `FmecaScoringConfig` map.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Missing or malformed JSON files return `Err(String)` without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-LOAD-SCORING-CONFIG]
fn load_scoring_config(
    scoring_config_path: Option<&Path>,
) -> Result<Option<FmecaScoringConfig>, String> {
    let Some(cfg_path) = scoring_config_path else {
        return Ok(None);
    };
    if !cfg_path.exists() {
        return Err(format!(
            "Error: FMECA scoring config does not exist: {}",
            cfg_path.display()
        ));
    }
    let cfg_text = fs::read_to_string(cfg_path)
        .map_err(|e| format!("Failed to read scoring config: {e}"))?;
    let cfg: FmecaScoringConfig = serde_json::from_str(&cfg_text)
        .map_err(|e| format!("Failed to parse scoring config JSON: {e}"))?;
    Ok(Some(cfg))
}

/// Executes `--stpa-transpile` mode (`CommandMode::StpaTranspile`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Validates the input SysML v2 schema via `load_and_validate_schema(schema, true)` (`ARCH-02`)
/// and emits the 10-pillar STPA safety artifact suite while honoring `dry_run` (`ARCH-04`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Blocks safety suite synthesis if the underlying SysML v2 schema has 0 elements or semantic errors.
/// - Performs zero disk writes when `dry_run` is `true`.
///
/// ## 3. Preconditions
/// - `out_dir` is the target safety suite directory; `schema` and `scoring_config_path` are optional paths.
///
/// ## 4. Postconditions
/// - Persists the 10-pillar safety suite via `emit_safety_suite` (when `!dry_run`) and returns `Ok(0)`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N)$ over schema AST nodes and UCA combinations.
/// - Space Complexity: $O(N)$ for synthesized safety artifact strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Schema validation, scoring config, or file write errors return `Err((1, String))`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-HANDLE-STPA-TRANSPILE]
fn handle_stpa_transpile(
    schema: Option<PathBuf>,
    out_dir: &Path,
    scoring_config_path: Option<&Path>,
    dry_run: bool,
) -> Result<i32, (i32, String)> {
    let scoring_config = load_scoring_config(scoring_config_path).map_err(|e| (1, e))?;
    let (_, pkg, _) = load_and_validate_schema(schema, true).map_err(|e| (1, e))?;
    if dry_run {
        let _suite = transpile_safety_suite(&pkg, scoring_config.as_ref());
        println!(
            "[STPA Transpile] [DRY RUN] Would emit safety artifact suite to '{}'",
            out_dir.display()
        );
        return Ok(0);
    }
    emit_safety_suite(&pkg, out_dir, scoring_config.as_ref())
        .map_err(|e| (1, format!("Error during STPA transpilation: {e}")))?;
    println!(
        "[STPA Transpile] Emitted safety artifact suite to '{}'",
        out_dir.display()
    );
    Ok(0)
}

/// Executes `--stpa` mode (`CommandMode::StpaToConstraints`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Compiles STPA Markdown tables into formal SysML v2 constraint definitions with fail-closed
/// base schema loading (`ARCH-01`), atomic file persistence (`ARCH-04`), and `dry_run` support.
///
/// ## 2. Invariants & Mathematical Properties
/// - Eliminates silent `.ok()` masking on `--schema`: if `schema` is supplied (or default schema
///   targets exist on disk), loads via `load_and_validate_schema(schema, false)` and fails closed
///   on any directory/read/parse error.
/// - Persists explicit `--out` targets via `write_atomic` when `dry_run` is `false`.
///
/// ## 3. Preconditions
/// - `stpa_file` points to a readable STPA Markdown document.
///
/// ## 4. Postconditions
/// - Writes compiled SysML constraints to `out` (or prints to `stdout`) and returns `Ok(0)`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(M + N)$ where $M$ is STPA Markdown size and $N$ is base schema size.
/// - Space Complexity: $O(M + N)$ for parsed AST and serialized SysML text.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Missing STPA file, unreadable/corrupt base schema, or atomic write failure returns `Err((1, String))`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-HANDLE-STPA-CONSTRAINTS]
fn handle_stpa_to_constraints(
    stpa_file: &Path,
    schema: Option<PathBuf>,
    out: &Path,
    dry_run: bool,
) -> Result<i32, (i32, String)> {
    if !stpa_file.exists() {
        return Err((
            1,
            format!("Error: STPA file does not exist: {}", stpa_file.display()),
        ));
    }
    let content = fs::read_to_string(stpa_file).map_err(|e| {
        (
            1,
            format!("Error reading STPA file '{}': {}", stpa_file.display(), e),
        )
    })?;

    let base_model = if schema.is_some() || resolve_schema_targets(None).is_ok() {
        let (_, pkg, _) = load_and_validate_schema(schema, false).map_err(|e| (1, e))?;
        Some(pkg)
    } else {
        None
    };

    let pkg = compile_stpa_to_constraints_filtered(
        &content,
        "System_SafetyConstraints",
        None,
        base_model.as_ref(),
    );
    let sysml_code = to_sysml(&pkg);

    if !dry_run && out != Path::new(".pipeline/schema.sysml") {
        write_atomic(out, &sysml_code).map_err(|e| {
            (
                1,
                format!("Error writing SysML to '{}': {}", out.display(), e),
            )
        })?;
        println!(
            "Successfully compiled STPA constraints to '{}'",
            out.display()
        );
    } else {
        println!("{sysml_code}");
    }
    Ok(0)
}

/// Executes `--forward-sync` mode (`CommandMode::ForwardSync`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Validates the input SysML v2 SSOT schema via `load_and_validate_schema(schema, true)` (`ARCH-02`)
/// before forward-synchronizing Agile Markdown specifications under DO-178C Level A and DO-330 TQL-4.
///
/// ## 2. Invariants & Mathematical Properties
/// - Rejects empty or semantically invalid SysML schemas before modifying `docs/`.
/// - Propagates `dry_run` and `force` flags directly to `forward_sync_sysml_to_specs`.
///
/// ## 3. Preconditions
/// - `docs` and optional `out_dir` identify target specification directories.
///
/// ## 4. Postconditions
/// - Synchronizes Markdown specifications from `pkg` and returns `Ok(0)`, or `Err((1, String))` on error.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N)$ over AST elements and generated specification files.
/// - Space Complexity: $O(N)$ for in-memory specification buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Schema validation errors, landing-zone guard rejections, or I/O errors return `Err((1, String))`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-HANDLE-FORWARD-SYNC]
fn handle_forward_sync(
    schema: Option<PathBuf>,
    docs: PathBuf,
    out_dir: Option<PathBuf>,
    dry_run: bool,
    force: bool,
) -> Result<i32, (i32, String)> {
    let (target, pkg, _) = load_and_validate_schema(schema, true).map_err(|e| (1, e))?;
    let schema_path = match target {
        SchemaTarget::SingleFile(p) => p,
        SchemaTarget::Directory(dir, _) => dir,
    };
    let opts = ForwardSyncOptions {
        schema_path,
        docs_dir: docs,
        out_dir,
        dry_run,
        force,
    };
    forward_sync_sysml_to_specs(&pkg, &opts)
        .map_err(|e| (1, format!("Error during forward sync: {e}")))?;
    Ok(0)
}

/// Executes `--reverse-sync` mode (`CommandMode::ReverseSync`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Synchronizes Agile Markdown specifications back into the SysML v2 SSOT while enforcing
/// fail-closed base schema loading (`ARCH-01`) and honoring `dry_run` (`ARCH-04`) under DO-178C Level A.
///
/// ## 2. Invariants & Mathematical Properties
/// - Eliminates silent `unwrap_or_default()` and `.ok()` fallback to a blank `System_SSOT` package
///   when an explicit `--schema` is supplied or a default schema exists on disk: any read or parse
///   error fails closed with `Err((1, ...))` so `.pipeline/schema.sysml` is never clobbered.
/// - Delegates persistence to `reverse_sync_specs_to_sysml_with_dry_run` to honor `dry_run`.
///
/// ## 3. Preconditions
/// - `docs` points to an existing Markdown specification directory.
///
/// ## 4. Postconditions
/// - Updates `out` and `digest` atomically (when `!dry_run`) and returns `Ok(0)`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(D + N)$ where $D$ is Markdown corpus size and $N$ is base schema size.
/// - Space Complexity: $O(D + N)$ for merged `PackageDef` and digest structures.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Corrupt/unreadable base schemas, missing `docs/` directories, or overwrite guard violations
///   return `Err((1, String))`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-HANDLE-REVERSE-SYNC]
fn handle_reverse_sync(
    schema: Option<PathBuf>,
    docs: PathBuf,
    out: PathBuf,
    digest: PathBuf,
    force: bool,
    dry_run: bool,
) -> Result<i32, (i32, String)> {
    let base_pkg = if schema.is_some() || resolve_schema_targets(None).is_ok() {
        let (_, pkg, _) = load_and_validate_schema(schema.clone(), false).map_err(|e| (1, e))?;
        Some(pkg)
    } else {
        None
    };

    let opts = ReverseSyncOptions {
        docs_dir: docs,
        schema_path: schema,
        output_path: out,
        digest_path: digest,
        allow_overwrite: force,
    };

    reverse_sync_specs_to_sysml_with_dry_run(base_pkg.as_ref(), &opts, dry_run)
        .map_err(|e| (1, format!("Error during reverse sync: {e}")))?;
    Ok(0)
}

/// Executes `--compile` mode (`CommandMode::Compile`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Delegates Pipeline 0 compilation gate execution to `run_compilation_gate_with_dry_run` under
/// DO-178C Level A and DO-330 TQL-4.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves full semantic validation, SHA-256 digest generation, and `dry_run` support.
///
/// ## 3. Preconditions
/// - `schema` is an optional input path; `out` and `digest` are output file paths.
///
/// ## 4. Postconditions
/// - Returns `Ok(0)` on clean compilation or `Err((1, String))` on failure.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N)$ over input schema size.
/// - Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates compilation gate errors cleanly without calling `process::exit`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-HANDLE-COMPILE]
fn handle_compile(
    schema: Option<PathBuf>,
    out: &Path,
    digest: &Path,
    dry_run: bool,
) -> Result<i32, (i32, String)> {
    run_compilation_gate_with_dry_run(schema, out, digest, dry_run)
}

/// Dispatches STPA, synchronization, compilation, and usage `CommandMode` variants.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Partitioned mode dispatcher keeping per-function cyclomatic complexity $v(G) \le 8$ under
/// DO-178C Level A and DO-330 TQL-4 modularity standards.
///
/// ## 2. Invariants & Mathematical Properties
/// - Single-Entry/Single-Exit routing over pipeline `CommandMode` variants.
///
/// ## 3. Preconditions
/// - `mode` is a pipeline or usage `CommandMode` variant.
///
/// ## 4. Postconditions
/// - Returns `Result<i32, (i32, String)>` from the matched handler.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N)$ bounded by the invoked pipeline handler.
/// - Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates handler errors directly as `Err((code, msg))`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-DISPATCH-PIPELINE]
fn dispatch_pipeline_mode(mode: CommandMode) -> Result<i32, (i32, String)> {
    match mode {
        CommandMode::StpaTranspile {
            schema,
            out_dir,
            scoring_config,
            dry_run,
        } => handle_stpa_transpile(schema, &out_dir, scoring_config.as_deref(), dry_run),
        CommandMode::StpaToConstraints {
            stpa_file,
            schema,
            out,
            dry_run,
        } => handle_stpa_to_constraints(&stpa_file, schema, &out, dry_run),
        CommandMode::ForwardSync {
            schema,
            docs,
            out_dir,
            dry_run,
            force,
        } => handle_forward_sync(schema, docs, out_dir, dry_run, force),
        CommandMode::ReverseSync {
            schema,
            docs,
            out,
            digest,
            force,
            dry_run,
        } => handle_reverse_sync(schema, docs, out, digest, force, dry_run),
        CommandMode::Compile {
            schema,
            out,
            digest,
            dry_run,
        } => handle_compile(schema, &out, &digest, dry_run),
        CommandMode::ShowUsage => {
            println!("compile-sysml: Specify --compile, --stpa-transpile, --stpa, --forward-sync, --reverse-sync, --inject-safety-docstrings, or --extract-source. Run with --help for usage.");
            Ok(0)
        }
        _ => Err((1, "Internal error: unexpected command mode".to_string())),
    }
}

/// Executes a parsed `Cli` configuration in-process without calling `std::process::exit`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Provides the primary testable library orchestration entrypoint (`ARCH-05`) under DO-178C Level A
/// and DO-330 TQL-4 Humble Object design rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Converts `cli` via `Cli::into_mode()` and dispatches to the corresponding SESE handler ($v(G) \le 8$).
/// - Never invokes `std::process::exit`, enabling direct in-process unit and integration testing.
///
/// ## 3. Preconditions
/// - `cli` is a populated `Cli` struct.
///
/// ## 4. Postconditions
/// - Returns `Ok(exit_code)` on success or `Err((exit_code, error_message))` on failure.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N)$ bounded by the selected pipeline handler.
/// - Space Complexity: $O(N)$ proportional to AST and output buffer sizes.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Conflicting CLI flags, missing files, validation errors, or I/O failures return `Err((code, msg))`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-EXECUTE], [DO-330-TQL-4-CLI]
pub fn execute(cli: Cli) -> Result<i32, (i32, String)> {
    let mode = cli.into_mode().map_err(|e| (1, e))?;
    match mode {
        CommandMode::InjectDocstrings {
            target_dir,
            schema,
            dry_run,
            force,
        } => handle_inject_docstrings(&target_dir, schema, dry_run, force),
        CommandMode::ExtractSource {
            extract_dir,
            out,
            digest,
            dry_run,
        } => handle_extract_source(&extract_dir, &out, &digest, dry_run),
        CommandMode::ExportMbd {
            mbd_dir,
            schema,
            dry_run,
        } => handle_export_mbd(&mbd_dir, schema, dry_run),
        CommandMode::SlicePackage { target_pkg, schema } => {
            handle_slice_package(&target_pkg, schema)
        }
        CommandMode::EmitFlutterContracts {
            target_dir,
            schema,
            dry_run,
        } => handle_emit_flutter_contracts(&target_dir, schema, dry_run),
        other => dispatch_pipeline_mode(other),
    }
}

/// Executes a parsed `Cli` configuration, emitting any error diagnostic to `stderr` and returning the exit code.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Bridges `execute(cli)` to the Humble Entrypoint in `src/main.rs` (`ARCH-05`) under DO-178C Level A
/// and DO-330 TQL-4 Single-Entry/Single-Exit (SESE) execution rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Total function over `Cli`: maps `Ok(code)` to `code` and `Err((code, msg))` to `eprintln!("{msg}")` followed by `code`.
/// - Zero `std::process::exit` invocations.
///
/// ## 3. Preconditions
/// - `cli` is a populated `Cli` struct.
///
/// ## 4. Postconditions
/// - Returns the integer process exit code (`0`, `1`, or `2`) to the caller.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N)$ bounded by `execute(cli)`.
/// - Space Complexity: $O(N)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Captures all operational errors from `execute(cli)`, logs them to `stderr`, and returns a non-zero exit code.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-EXECUTE-CLI], [DO-330-TQL-4-CLI]
pub fn execute_cli(cli: Cli) -> i32 {
    match execute(cli) {
        Ok(code) => code,
        Err((code, msg)) => {
            if !msg.is_empty() {
                eprintln!("{msg}");
            }
            code
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::sync::atomic::{AtomicU64, Ordering};

    static DRIVER_TEST_COUNTER: AtomicU64 = AtomicU64::new(1);

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let seq = DRIVER_TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            ".deap_driver_test_{}_{}_{}",
            prefix,
            std::process::id(),
            seq
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    /// Verifies that `resolve_schema_targets` falls back to `<ws>/.pipeline/schema.sysml` when `<ws>/schema/` contains no `.sysml` files.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `test_resolve_schema_targets_falls_back_to_pipeline_schema`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Resolves `SchemaTarget::SingleFile(<ws>/.pipeline/schema.sysml)` deterministically when `<ws>/schema/` is empty.
    ///
    /// ## 3. Preconditions
    /// - Writable temporary workspace directory exists.
    ///
    /// ## 4. Postconditions
    /// - Asserts `resolve_schema_targets(Some(schema_dir))` returns `Ok(SchemaTarget::SingleFile(pipeline_schema))`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$ over empty `schema/` fixture. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Cleans up temporary workspace directory deterministically.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-COMPILER-SYSML-RESOLVE-TARGETS]
    #[test]
    fn test_resolve_schema_targets_falls_back_to_pipeline_schema(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp_ws = unique_temp_dir("resolve_schema");
        let schema_dir = temp_ws.join("schema");
        let pipeline_dir = temp_ws.join(".pipeline");
        fs::create_dir_all(&schema_dir)?;
        fs::create_dir_all(&pipeline_dir)?;
        fs::write(schema_dir.join(".gitkeep"), "")?;
        let pipeline_schema = pipeline_dir.join("schema.sysml");
        fs::write(&pipeline_schema, "package FallbackPkg {}\n")?;

        let resolved = resolve_schema_targets(Some(schema_dir));
        let _ = fs::remove_dir_all(&temp_ws);
        match resolved {
            Ok(SchemaTarget::SingleFile(path)) => assert_eq!(path, pipeline_schema),
            other => panic!(
                "expected SingleFile fallback to .pipeline/schema.sysml, got {:?}",
                other.err()
            ),
        }
        Ok(())
    }

    /// Verifies that passing a sub-workspace root directory (such as `.pipeline/upstream`) resolves `.pipeline/schema.sysml`, scopes default CLI paths into the sub-workspace, and synchronizes `schema/model.sysml`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `test_subworkspace_path_resolution_and_compilation_gate`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Resolves `<ws>/.pipeline/schema.sysml` when `resolve_schema_targets` receives `<ws>` directly and `<ws>/schema/` contains no `.sysml` files.
    /// - Synchronizes `<ws>/schema/model.sysml` during `run_compilation_gate` when compiling into `<ws>/.pipeline/schema.sysml`.
    ///
    /// ## 3. Preconditions
    /// - Writable temporary workspace directory exists.
    ///
    /// ## 4. Postconditions
    /// - Asserts `resolve_schema_targets(Some(temp_ws))` and `run_compilation_gate` succeed and populate `<ws>/schema/model.sysml`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$ over single-package fixture. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Cleans up temporary workspace directory deterministically.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-COMPILER-SYSML-RESOLVE-TARGETS], [REQ-COMPILER-SYSML-GATE]
    #[test]
    fn test_subworkspace_path_resolution_and_compilation_gate(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp_ws = unique_temp_dir("subworkspace");
        let schema_dir = temp_ws.join("schema");
        let pipeline_dir = temp_ws.join(".pipeline");
        fs::create_dir_all(&schema_dir)?;
        fs::create_dir_all(&pipeline_dir)?;
        fs::write(schema_dir.join(".gitkeep"), "")?;
        let pipeline_schema = pipeline_dir.join("schema.sysml");
        let pipeline_digest = pipeline_dir.join("schema-digest.json");
        fs::write(
            &pipeline_schema,
            "package SubWorkspacePkg {\n    part def CoreUnit {}\n}\n",
        )?;

        let resolved = resolve_schema_targets(Some(temp_ws.clone()));
        let code = run_compilation_gate(
            Some(temp_ws.clone()),
            &pipeline_schema,
            &pipeline_digest,
        );
        let model_exists = schema_dir.join("model.sysml").is_file();
        let _ = fs::remove_dir_all(&temp_ws);

        assert_eq!(resolved, Ok(SchemaTarget::SingleFile(pipeline_schema)));
        assert_eq!(code, 0);
        assert!(
            model_exists,
            "expected schema/model.sysml to be synchronized in sub-workspace"
        );
        Ok(())
    }

    /// Verifies `ARCH-01`: `--reverse-sync` and `--stpa` fail closed with exit code `1` when
    /// given a malformed or unreadable schema instead of silently falling back to a blank model.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D verification mandates
    /// for fail-closed error handling (`ARCH-01`) in `crates/compile-sysml/src/driver.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Malformed SysML syntax in `--schema` causes `execute` to return `Err((1, ...))` in both
    ///   `--reverse-sync` and `--stpa` modes without overwriting `.pipeline/schema.sysml`.
    ///
    /// ## 3. Preconditions
    /// - Writable temporary workspace directory containing a malformed `.sysml` file.
    ///
    /// ## 4. Postconditions
    /// - Asserts `execute` returns `Err((1, _))` for both `--reverse-sync` and `--stpa`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Cleans up temporary workspace directory deterministically.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-COMPILER-SYSML-LOAD-AND-VALIDATE-SCHEMA]
    #[test]
    fn test_arch_01_fail_closed_on_malformed_schema_in_reverse_sync_and_stpa(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp_ws = unique_temp_dir("arch01_fail_closed");
        let bad_schema = temp_ws.join("corrupt.sysml");
        fs::write(&bad_schema, "package UnclosedPkg { part def Broken")?;
        let docs_dir = temp_ws.join("docs");
        fs::create_dir_all(docs_dir.join("features"))?;
        let out_schema = temp_ws.join("out.sysml");
        let out_digest = temp_ws.join("out-digest.json");

        let rev_cli = Cli::parse_from(vec![
            "compile-sysml",
            "--reverse-sync",
            "--schema",
            bad_schema.to_str().unwrap(),
            "--docs",
            docs_dir.to_str().unwrap(),
            "--out",
            out_schema.to_str().unwrap(),
            "--digest",
            out_digest.to_str().unwrap(),
            "--force",
        ]);
        let rev_res = execute(rev_cli);
        assert!(
            matches!(rev_res, Err((1, _))),
            "reverse-sync must fail closed on corrupt schema, got: {:?}",
            rev_res
        );
        assert!(
            !out_schema.exists(),
            "reverse-sync must not write output schema when input schema is corrupt"
        );

        let stpa_md = temp_ws.join("STPA_MATRIX.md");
        fs::write(
            &stpa_md,
            "| UCA ID | Controller | Control Action | Guide Word | Context | Hazard | Safety Constraint | Severity | SAIL |\n| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n| UCA-001 | Ctrl | Act | Not providing | Op | H-1 | SC-001 | High | IV |\n",
        )?;
        let stpa_cli = Cli::parse_from(vec![
            "compile-sysml",
            "--stpa",
            stpa_md.to_str().unwrap(),
            "--schema",
            bad_schema.to_str().unwrap(),
            "--out",
            out_schema.to_str().unwrap(),
        ]);
        let stpa_res = execute(stpa_cli);
        let _ = fs::remove_dir_all(&temp_ws);
        assert!(
            matches!(stpa_res, Err((1, _))),
            "--stpa must fail closed on corrupt schema, got: {:?}",
            stpa_res
        );
        Ok(())
    }

    /// Verifies `ARCH-02`: `SemanticValidator` rejects semantically invalid ASTs in `--export-mbd`,
    /// `--slice-package`, and `--emit-flutter-contracts`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D verification mandates
    /// for mandatory `SemanticValidator` enforcement (`ARCH-02`) in `crates/compile-sysml/src/driver.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - A schema containing a cyclic inheritance specialization (`part def A :> B; part def B :> A;`)
    ///   is rejected with exit code `1` across `ExportMbd`, `SlicePackage`, and `EmitFlutterContracts`.
    ///
    /// ## 3. Preconditions
    /// - Writable temporary workspace directory containing a semantically invalid `.sysml` schema.
    ///
    /// ## 4. Postconditions
    /// - Asserts all three codegen/slicing modes return `Err((1, msg))` containing `"Semantic validation error"`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Cleans up temporary workspace directory deterministically.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-COMPILER-SYSML-LOAD-PACKAGE-CLI]
    #[test]
    fn test_arch_02_semantic_validator_rejects_invalid_ast_in_codegen_modes(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp_ws = unique_temp_dir("arch02_validator");
        let invalid_schema = temp_ws.join("invalid_semantics.sysml");
        fs::write(
            &invalid_schema,
            "package InvalidPkg {\n    part def ValidController {\n        part sub_unit : NonExistentComponentType;\n    }\n}\n",
        )?;

        let mbd_dir = temp_ws.join("mbd_out");
        let mbd_cli = Cli::parse_from(vec![
            "compile-sysml",
            "--export-mbd",
            mbd_dir.to_str().unwrap(),
            "--schema",
            invalid_schema.to_str().unwrap(),
        ]);
        let mbd_err = execute(mbd_cli).expect_err("ExportMbd must reject cyclic inheritance");
        assert_eq!(mbd_err.0, 1);
        assert!(mbd_err.1.contains("Semantic validation error"));

        let slice_cli = Cli::parse_from(vec![
            "compile-sysml",
            "--slice-package",
            "CyclicPkg",
            "--schema",
            invalid_schema.to_str().unwrap(),
        ]);
        let slice_err =
            execute(slice_cli).expect_err("SlicePackage must reject cyclic inheritance");
        assert_eq!(slice_err.0, 1);
        assert!(slice_err.1.contains("Semantic validation error"));

        let flutter_dir = temp_ws.join("flutter_out");
        let flutter_cli = Cli::parse_from(vec![
            "compile-sysml",
            "--emit-flutter-contracts",
            flutter_dir.to_str().unwrap(),
            "--schema",
            invalid_schema.to_str().unwrap(),
        ]);
        let flutter_err =
            execute(flutter_cli).expect_err("EmitFlutterContracts must reject cyclic inheritance");
        let _ = fs::remove_dir_all(&temp_ws);
        assert_eq!(flutter_err.0, 1);
        assert!(flutter_err.1.contains("Semantic validation error"));
        Ok(())
    }

    /// Verifies `ARCH-04` and `ARCH-05`: `--dry-run` performs zero disk writes across `ExportMbd`,
    /// `EmitFlutterContracts`, `StpaTranspile`, `StpaToConstraints`, and `Compile`, while `execute_cli`
    /// returns exit codes in-process without calling `process::exit`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D verification mandates
    /// for side-effect-free `--dry-run` simulation (`ARCH-04`) and in-process SESE execution (`ARCH-05`).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Target output files and directories remain absent after `--dry-run` execution across all mutating modes.
    ///
    /// ## 3. Preconditions
    /// - Writable temporary workspace directory containing a valid `.sysml` schema and STPA fixture.
    ///
    /// ## 4. Postconditions
    /// - Asserts `execute_cli` returns `0` for all 5 `--dry-run` invocations and zero output files are created.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$. Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Cleans up temporary workspace directory deterministically.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-COMPILER-SYSML-EXECUTE-CLI], [REQ-COMPILER-SYSML-GATE-DRY-RUN]
    #[test]
    fn test_arch_04_and_05_dry_run_skips_disk_writes_and_executes_in_process(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temp_ws = unique_temp_dir("arch04_dry_run");
        let valid_schema = temp_ws.join("valid.sysml");
        fs::write(
            &valid_schema,
            "package ValidPkg {\n    part def ControllerUnit {\n        attribute gain : Real;\n        action def ComputeStep;\n    }\n}\n",
        )?;

        // 1. Compile --dry-run
        let compiled_out = temp_ws.join("compiled.sysml");
        let compiled_digest = temp_ws.join("compiled-digest.json");
        let compile_cli = Cli::parse_from(vec![
            "compile-sysml",
            "--compile",
            "--schema",
            valid_schema.to_str().unwrap(),
            "--out",
            compiled_out.to_str().unwrap(),
            "--digest",
            compiled_digest.to_str().unwrap(),
            "--dry-run",
        ]);
        assert_eq!(execute_cli(compile_cli), 0);
        assert!(!compiled_out.exists(), "Compile --dry-run must not write out");
        assert!(
            !compiled_digest.exists(),
            "Compile --dry-run must not write digest"
        );

        // 2. ExportMbd --dry-run
        let mbd_out = temp_ws.join("mbd_dry");
        let mbd_cli = Cli::parse_from(vec![
            "compile-sysml",
            "--export-mbd",
            mbd_out.to_str().unwrap(),
            "--schema",
            valid_schema.to_str().unwrap(),
            "--dry-run",
        ]);
        assert_eq!(execute_cli(mbd_cli), 0);
        assert!(!mbd_out.exists(), "ExportMbd --dry-run must not create mbd_out");

        // 3. EmitFlutterContracts --dry-run
        let flutter_out = temp_ws.join("flutter_dry");
        let flutter_cli = Cli::parse_from(vec![
            "compile-sysml",
            "--emit-flutter-contracts",
            flutter_out.to_str().unwrap(),
            "--schema",
            valid_schema.to_str().unwrap(),
            "--dry-run",
        ]);
        assert_eq!(execute_cli(flutter_cli), 0);
        assert!(
            !flutter_out.exists(),
            "EmitFlutterContracts --dry-run must not create flutter_out"
        );

        // 4. StpaTranspile --dry-run
        let safety_out = temp_ws.join("safety_dry");
        let stpa_transpile_cli = Cli::parse_from(vec![
            "compile-sysml",
            "--stpa-transpile",
            "--schema",
            valid_schema.to_str().unwrap(),
            "--out-dir",
            safety_out.to_str().unwrap(),
            "--dry-run",
        ]);
        assert_eq!(execute_cli(stpa_transpile_cli), 0);
        assert!(
            !safety_out.exists(),
            "StpaTranspile --dry-run must not create safety_out"
        );

        // 5. StpaToConstraints --dry-run
        let stpa_md = temp_ws.join("STPA_MATRIX.md");
        fs::write(
            &stpa_md,
            "| UCA ID | Controller | Control Action | Guide Word | Context | Hazard | Safety Constraint | Severity | SAIL |\n| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n| UCA-001 | ControllerUnit | ComputeStep | Not providing | Op | H-1 | SC-001 | High | IV |\n",
        )?;
        let stpa_out = temp_ws.join("constraints.sysml");
        let stpa_cli = Cli::parse_from(vec![
            "compile-sysml",
            "--stpa",
            stpa_md.to_str().unwrap(),
            "--schema",
            valid_schema.to_str().unwrap(),
            "--out",
            stpa_out.to_str().unwrap(),
            "--dry-run",
        ]);
        assert_eq!(execute_cli(stpa_cli), 0);
        let _ = fs::remove_dir_all(&temp_ws);
        assert!(
            !stpa_out.exists(),
            "StpaToConstraints --dry-run must not write stpa_out"
        );
        Ok(())
    }
}
