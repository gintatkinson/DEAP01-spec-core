#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] SysML v2 Schema Target Resolution, Semantic Validation & Compilation Gate Loader.
//! Safety & Traceability: Realises: [REQ-COMPILER-SYSML-DRIVER], [DO-330-TQL-4-GATE]
//! Contract: Resolves single-file and multi-file SysML v2 targets, validates structural element counts and semantics, and persists canonical schemas and SHA-256 digests.
//! Bounds & Faults: Operates in O(N log N) time and O(N) space; fails closed via typed `DriverError` on missing, empty, or invalid schemas.

use crate::driver::{DriverError, SchemaTarget, SCHEMA_REMEDIATION_MESSAGE};
use crate::salsa_db::{
    compute_file_manifest, count_structural_elements, write_atomic, SchemaDigest,
    SemanticValidator, SysmlSerializable,
};
use crate::{discover_sysml_files, parse_sysml, parse_sysml_tree};
use std::fs;
use std::path::{Path, PathBuf};

/// Resolves `<ws>/.pipeline/schema.sysml` when `path` is a `<ws>/schema` directory path.
/// Realises: [REQ-COMPILER-SYSML-RESOLVE-DIR-TARGET]
pub(crate) fn resolve_schema_dir_pipeline_fallback(path: &Path) -> Option<SchemaTarget> {
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
/// Realises: [REQ-COMPILER-SYSML-RESOLVE-DIR-TARGET]
pub(crate) fn resolve_directory_schema_target(dir: PathBuf) -> Result<SchemaTarget, DriverError> {
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
        DriverError::Resolution(format!(
            "Error: No .sysml files found in directory: {}",
            dir.display()
        ))
    })
}

/// Resolves input schema path into a `SchemaTarget` returning typed `DriverError::Resolution` on missing paths.
/// Realises: [REQ-COMPILER-SYSML-RESOLVE-TARGETS]
pub(crate) fn resolve_schema_targets_typed(
    explicit_path: Option<PathBuf>,
) -> Result<SchemaTarget, DriverError> {
    if let Some(path) = explicit_path {
        if path.is_file() {
            return Ok(SchemaTarget::SingleFile(path));
        }
        if path.is_dir() {
            return resolve_directory_schema_target(path);
        }
        return resolve_schema_dir_pipeline_fallback(&path).ok_or_else(|| {
            DriverError::Resolution(format!(
                "Error: Schema file does not exist: {}",
                path.display()
            ))
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

    Err(DriverError::Resolution(
        SCHEMA_REMEDIATION_MESSAGE.to_string(),
    ))
}

/// Parses a resolved `SchemaTarget` into a `(PackageDef, display_name)` pair with fail-closed I/O and emptiness checks.
/// Realises: [REQ-COMPILER-SYSML-PARSE-RESOLVED-TARGET]
pub(crate) fn parse_resolved_schema_target(
    target: &SchemaTarget,
) -> Result<(deap_core::sysml_cst::PackageDef, String), DriverError> {
    match target {
        SchemaTarget::SingleFile(schema_file) => {
            let content = fs::read_to_string(schema_file).map_err(|e| {
                DriverError::Io(format!(
                    "Error reading schema file '{}': {}",
                    schema_file.display(),
                    e
                ))
            })?;
            if content.trim().is_empty() {
                return Err(DriverError::Validation(format!(
                    "Error: Schema file '{}' is empty.",
                    schema_file.display()
                )));
            }
            let display_name = schema_file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("SysML_Model")
                .to_string();
            let parsed = parse_sysml(&content).map_err(|err| {
                DriverError::Validation(format!(
                    "Error parsing schema file '{}': {}",
                    schema_file.display(),
                    err
                ))
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
                DriverError::Validation(format!(
                    "Error parsing schema directory '{}': {}",
                    dir.display(),
                    err
                ))
            })?;
            Ok((parsed, dir_name))
        }
    }
}

/// Resolves, parses, checks structural element cardinality, and optionally validates semantics returning `DriverError`.
/// Realises: [REQ-COMPILER-SYSML-LOAD-AND-VALIDATE-SCHEMA]
pub(crate) fn load_and_validate_schema_typed(
    explicit_path: Option<PathBuf>,
    require_validation: bool,
) -> Result<(SchemaTarget, deap_core::sysml_cst::PackageDef, String), DriverError> {
    let target = resolve_schema_targets_typed(explicit_path)?;
    let (pkg, display_name) = parse_resolved_schema_target(&target)?;

    if count_structural_elements(&pkg) == 0 {
        return Err(DriverError::Validation(format!(
            "Error: Schema '{display_name}' contains 0 structural elements."
        )));
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
            return Err(DriverError::Validation(msgs.join("\n")));
        }
    }

    Ok((target, pkg, display_name))
}

/// Resolves, parses, and semantically validates a SysML schema target for internal CLI handlers returning `DriverError`.
/// Realises: [REQ-COMPILER-SYSML-LOAD-PACKAGE-CLI]
pub(crate) fn load_schema_package_typed(
    explicit_path: Option<PathBuf>,
) -> Result<(SchemaTarget, deap_core::sysml_cst::PackageDef, String), DriverError> {
    load_and_validate_schema_typed(explicit_path, true)
}

/// Synchronizes `<ws>/schema/model.sysml` when compiling into `<ws>/.pipeline/schema.sysml` in downstream workspaces.
/// Realises: [REQ-COMPILER-SYSML-SYNC-DOWNSTREAM-MODEL]
pub(crate) fn sync_downstream_model_sysml(
    output_path: &Path,
    sysml_text: &str,
) -> Result<(), DriverError> {
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
                DriverError::Io(format!(
                    "Error writing synchronized schema model to '{}': {}",
                    model_out.display(),
                    e
                ))
            })?;
        }
    }
    Ok(())
}

/// Checks whether `path` is a blueprint SysML v2 triplet file (`blueprint-*.sysml`).
/// Realises: [REQ-COMPILER-SYSML-BLUEPRINT-TRIPLET-DIR]
pub(crate) fn is_blueprint_sysml_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| name.starts_with("blueprint-") && name.ends_with(".sysml"))
}

/// Checks whether a resolved schema directory represents a `docs/designs` blueprint triplet directory.
/// Realises: [REQ-COMPILER-SYSML-BLUEPRINT-TRIPLET-DIR]
pub(crate) fn is_blueprint_triplet_dir(dir: &Path, files: &[PathBuf]) -> bool {
    !files.is_empty()
        && (dir.ends_with("docs/designs") || files.iter().all(|f| is_blueprint_sysml_file(f)))
}

/// Checks whether `output_path` and `digest_path` match the default CLI compilation targets.
/// Realises: [REQ-COMPILER-SYSML-BLUEPRINT-TRIPLET-DIR]
pub(crate) fn is_default_compilation_paths(output_path: &Path, digest_path: &Path) -> bool {
    output_path == Path::new(".pipeline/schema.sysml")
        && digest_path == Path::new(".pipeline/schema-digest.json")
}

/// Compiles each blueprint `.sysml` file in `files` in place and writes its `.digest.json` sidecar.
/// Realises: [REQ-COMPILER-SYSML-BLUEPRINT-TRIPLET-DIR]
pub(crate) fn compile_blueprint_triplet_dir(
    files: &[PathBuf],
    dry_run: bool,
) -> Result<i32, DriverError> {
    for schema_file in files {
        let digest_sidecar = schema_file.with_extension("digest.json");
        compile_single_or_merged_schema(
            Some(schema_file.clone()),
            schema_file,
            &digest_sidecar,
            dry_run,
        )?;
    }
    Ok(0)
}

/// Compiles a single schema file or merged schema directory into `output_path` and `digest_path`.
/// Realises: [REQ-COMPILER-SYSML-GATE-DRY-RUN]
pub(crate) fn compile_single_or_merged_schema(
    explicit_schema: Option<PathBuf>,
    output_path: &Path,
    digest_path: &Path,
    dry_run: bool,
) -> Result<i32, DriverError> {
    let (target, pkg, display_name) = load_and_validate_schema_typed(explicit_schema, true)?;
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
        DriverError::Io(format!(
            "Error writing compiled schema to '{}': {}",
            output_path.display(),
            e
        ))
    })?;

    sync_downstream_model_sysml(output_path, &sysml_text)?;

    digest.write_to_file(digest_path).map_err(|e| {
        DriverError::Io(format!(
            "Error writing schema digest to '{}': {}",
            digest_path.display(),
            e
        ))
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

/// Internal SESE implementation of the Pipeline 0 compilation gate supporting `dry_run`.
/// Realises: [REQ-COMPILER-SYSML-GATE-DRY-RUN]
pub(crate) fn run_compilation_gate_with_dry_run(
    explicit_schema: Option<PathBuf>,
    output_path: &Path,
    digest_path: &Path,
    dry_run: bool,
) -> Result<i32, DriverError> {
    if is_default_compilation_paths(output_path, digest_path) {
        if let Ok(SchemaTarget::Directory(ref dir, ref files)) =
            resolve_schema_targets_typed(explicit_schema.clone())
        {
            if is_blueprint_triplet_dir(dir, files) {
                return compile_blueprint_triplet_dir(files, dry_run);
            }
        }
    }
    compile_single_or_merged_schema(explicit_schema, output_path, digest_path, dry_run)
}
