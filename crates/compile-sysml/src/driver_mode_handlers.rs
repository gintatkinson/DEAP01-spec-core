#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] Pipeline, Synchronization, STPA & Codegen Mode Handlers for `compile-sysml`.
//! Safety & Traceability: Realises: [REQ-COMPILER-SYSML-DRIVER], [DO-330-TQL-4-CLI]
//! Contract: Executes individual `CommandMode` pipeline, synchronization, STPA, MBD, and codegen actions with fail-closed schema validation and `dry_run` semantics.
//! Bounds & Faults: Operates in O(N log N) time and O(N) space; returns typed `Result<i32, DriverError>` with zero `std::process::exit` calls.

use crate::docstring::{
    run_docstring_injection, run_extract_source, DocstringExitCode, DocstringInjectionOptions,
};
use crate::driver::{DriverError, SchemaTarget};
use crate::driver_schema_loader::{
    load_and_validate_schema_typed, load_schema_package_typed, resolve_schema_targets_typed,
    run_compilation_gate_with_dry_run,
};
use crate::semantic::digest::{count_structural_elements, write_atomic};
use crate::semantic::serializer::to_sysml;
use crate::stpa::{
    compile_stpa_to_constraints_filtered, emit_safety_suite, transpile_safety_suite,
    FmecaScoringConfig,
};
use crate::sync::{
    forward_sync_sysml_to_specs, reverse_sync_specs_to_sysml_with_dry_run, ForwardSyncOptions,
    ReverseSyncOptions,
};
use crate::{export_mbd_artifacts, flatten_package_to_model, synthesize_mbd_bundle};
use std::fs;
use std::path::{Path, PathBuf};

/// Executes `--inject-safety-docstrings` mode (`CommandMode::InjectDocstrings`).
/// Realises: [REQ-COMPILER-SYSML-HANDLE-DOCSTRINGS]
pub(crate) fn handle_inject_docstrings(
    target_dir: &Path,
    schema: Option<PathBuf>,
    dry_run: bool,
    force: bool,
) -> Result<i32, DriverError> {
    let schema_model = if schema.is_some() || resolve_schema_targets_typed(None).is_ok() {
        let (_, pkg, _) = load_and_validate_schema_typed(schema, false).map_err(|e| {
            DriverError::Codegen(
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
                report.files_scanned, report.files_modified, report.items_enriched, target_dir.display()
            );
            Ok(DocstringExitCode::Clean.code())
        }
        Err((code, msg)) => Err(DriverError::Codegen(
            code.code(),
            format!("Error during docstring injection: {msg}"),
        )),
    }
}

/// Executes `--extract-source` mode (`CommandMode::ExtractSource`).
/// Realises: [REQ-COMPILER-SYSML-HANDLE-EXTRACT-SOURCE]
pub(crate) fn handle_extract_source(
    extract_dir: &Path,
    out: &Path,
    digest: &Path,
    dry_run: bool,
) -> Result<i32, DriverError> {
    match run_extract_source(extract_dir, out, digest, dry_run) {
        Ok(pkg) => {
            println!(
                "[Extract Source] Extracted {} structural item(s) from '{}' to '{}'",
                count_structural_elements(&pkg), extract_dir.display(), out.display()
            );
            Ok(DocstringExitCode::Clean.code())
        }
        Err((code, msg)) => Err(DriverError::Codegen(
            code.code(),
            format!("Error during source extraction: {msg}"),
        )),
    }
}

/// Executes `--export-mbd` mode (`CommandMode::ExportMbd`).
/// Realises: [REQ-COMPILER-SYSML-HANDLE-EXPORT-MBD]
pub(crate) fn handle_export_mbd(
    mbd_dir: &Path,
    schema: Option<PathBuf>,
    dry_run: bool,
) -> Result<i32, DriverError> {
    let (_, pkg, _) = load_schema_package_typed(schema)?;
    let model = flatten_package_to_model(pkg, None);
    if dry_run {
        let _bundle = synthesize_mbd_bundle(&model);
        println!("[MBD Export] [DRY RUN] Would emit 4 artifact(s) to '{}'", mbd_dir.display());
        return Ok(0);
    }
    let written = export_mbd_artifacts(&model, mbd_dir)
        .map_err(|e| DriverError::Codegen(1, format!("Error exporting MBD artifacts: {e}")))?;
    println!("[MBD Export] Emitted {} artifact(s) to '{}'", written.len(), mbd_dir.display());
    Ok(0)
}

/// Executes `--slice-package` mode (`CommandMode::SlicePackage`).
/// Realises: [REQ-COMPILER-SYSML-HANDLE-SLICE-PACKAGE]
pub(crate) fn handle_slice_package(
    target_pkg: &str,
    schema: Option<PathBuf>,
) -> Result<i32, DriverError> {
    let (_, pkg, _) = load_schema_package_typed(schema)?;
    let model = flatten_package_to_model(pkg, Some(target_pkg));
    let slice = model
        .slice_package_subsystem(target_pkg, 4096)
        .map_err(|e| DriverError::Codegen(1, format!("Error slicing package '{target_pkg}': {e}")))?;
    let json = serde_json::to_string_pretty(&slice).map_err(|e| {
        DriverError::Codegen(1, format!("Error serializing sliced package '{target_pkg}': {e}"))
    })?;
    println!("{json}");
    Ok(0)
}

/// Executes `--emit-flutter-contracts` mode (`CommandMode::EmitFlutterContracts`).
/// Realises: [REQ-COMPILER-SYSML-HANDLE-FLUTTER-CONTRACTS], [REQ-0607]
pub(crate) fn handle_emit_flutter_contracts(
    target_dir: &Path,
    schema: Option<PathBuf>,
    dry_run: bool,
) -> Result<i32, DriverError> {
    let (_, pkg, _) = load_schema_package_typed(schema)?;
    let model = flatten_package_to_model(pkg.clone(), None);
    let report = crate::flutter_codegen::emit_modular_flutter_contracts(
        target_dir, &pkg, &model, None, dry_run,
    )
    .map_err(|e| DriverError::Codegen(1, format!("Error emitting Flutter contracts: {e}")))?;
    if dry_run {
        println!(
            "[Flutter Codegen] [DRY RUN] Would emit contracts (schema hash: 0x{:016X}) to '{}'",
            report.schema_hash, target_dir.display()
        );
        return Ok(0);
    }
    println!(
        "[Flutter Codegen] Successfully emitted contracts (schema hash: 0x{:016X}) to '{}':",
        report.schema_hash, target_dir.display()
    );
    println!("  - Domain seed:      {}", report.seed_path.display());
    println!("  - Schema models:    {}", report.models_path.display());
    println!("  - Solver bridge:    {}", report.solver_bridge_path.display());
    println!("  - UI contract test: {}", report.ui_contract_test_path.display());
    Ok(0)
}

/// Executes `--emit-modular-rust` mode (`CommandMode::EmitModularRust`).
/// Realises: [REQ-0607]
pub(crate) fn handle_emit_modular_rust(
    target_dir: &Path,
    schema: Option<PathBuf>,
    dry_run: bool,
) -> Result<i32, DriverError> {
    let (_, pkg, _) = load_schema_package_typed(schema)?;
    let opts = crate::modular_codegen::emitters::rust_emitter::ModularRustOptions {
        target_dir: target_dir.to_path_buf(),
        dry_run,
        bounds: crate::modular_codegen::domain::ModularityBounds::canonical(),
    };
    let manifest =
        crate::modular_codegen::emitters::rust_emitter::emit_modular_rust_tree(&pkg, &opts)
            .map_err(|diags| {
                let joined = diags
                    .into_iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                DriverError::Codegen(1, joined)
            })?;
    if dry_run {
        println!(
            "[Modular Rust Codegen] [DRY RUN] Verified {} unit(s) for '{}' (tree SHA-256: {})",
            manifest.files.len(),
            target_dir.display(),
            manifest.tree_sha256
        );
    } else {
        println!(
            "[Modular Rust Codegen] Emitted {} unit(s) to '{}' (tree SHA-256: {})",
            manifest.files.len(),
            target_dir.display(),
            manifest.tree_sha256
        );
    }
    Ok(0)
}

/// Loads and parses an optional FMECA scoring configuration JSON file.
/// Realises: [REQ-COMPILER-SYSML-LOAD-SCORING-CONFIG]
pub(crate) fn load_scoring_config(
    scoring_config_path: Option<&Path>,
) -> Result<Option<FmecaScoringConfig>, DriverError> {
    let Some(cfg_path) = scoring_config_path else {
        return Ok(None);
    };
    if !cfg_path.exists() {
        return Err(DriverError::Resolution(format!(
            "Error: FMECA scoring config does not exist: {}",
            cfg_path.display()
        )));
    }
    let cfg_text = fs::read_to_string(cfg_path)
        .map_err(|e| DriverError::Io(format!("Failed to read scoring config: {e}")))?;
    let cfg: FmecaScoringConfig = serde_json::from_str(&cfg_text).map_err(|e| {
        DriverError::Validation(format!("Failed to parse scoring config JSON: {e}"))
    })?;
    Ok(Some(cfg))
}

/// Executes `--stpa-transpile` mode (`CommandMode::StpaTranspile`).
/// Realises: [REQ-COMPILER-SYSML-HANDLE-STPA-TRANSPILE]
pub(crate) fn handle_stpa_transpile(
    schema: Option<PathBuf>,
    out_dir: &Path,
    scoring_config_path: Option<&Path>,
    dry_run: bool,
) -> Result<i32, DriverError> {
    let scoring_config = load_scoring_config(scoring_config_path)?;
    let (_, pkg, _) = load_and_validate_schema_typed(schema, true)?;
    if dry_run {
        let registry_entries = crate::stpa::fmeca::load_failure_mode_registry_for_out_dir(out_dir);
        let _suite = if registry_entries.is_empty() {
            transpile_safety_suite(&pkg, scoring_config.as_ref())
        } else {
            crate::stpa::safety_suite::transpile_safety_suite_with_registry(
                &pkg,
                scoring_config.as_ref(),
                &registry_entries,
            )
        };
        println!(
            "[STPA Transpile] [DRY RUN] Would emit safety artifact suite to '{}'",
            out_dir.display()
        );
        return Ok(0);
    }
    emit_safety_suite(&pkg, out_dir, scoring_config.as_ref())
        .map_err(|e| DriverError::Codegen(1, format!("Error during STPA transpilation: {e}")))?;
    println!(
        "[STPA Transpile] Emitted safety artifact suite to '{}'",
        out_dir.display()
    );
    Ok(0)
}

/// Executes `--stpa` mode (`CommandMode::StpaToConstraints`).
/// Realises: [REQ-COMPILER-SYSML-HANDLE-STPA-CONSTRAINTS]
pub(crate) fn handle_stpa_to_constraints(
    stpa_file: &Path,
    schema: Option<PathBuf>,
    out: &Path,
    dry_run: bool,
) -> Result<i32, DriverError> {
    if !stpa_file.exists() {
        return Err(DriverError::Resolution(format!(
            "Error: STPA file does not exist: {}",
            stpa_file.display()
        )));
    }
    let content = fs::read_to_string(stpa_file).map_err(|e| {
        DriverError::Io(format!(
            "Error reading STPA file '{}': {}",
            stpa_file.display(),
            e
        ))
    })?;

    let base_model = if schema.is_some() || resolve_schema_targets_typed(None).is_ok() {
        let (_, pkg, _) = load_and_validate_schema_typed(schema, false)?;
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
            DriverError::Io(format!("Error writing SysML to '{}': {}", out.display(), e))
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
/// Realises: [REQ-COMPILER-SYSML-HANDLE-FORWARD-SYNC]
pub(crate) fn handle_forward_sync(
    schema: Option<PathBuf>,
    docs: PathBuf,
    out_dir: Option<PathBuf>,
    dry_run: bool,
    force: bool,
) -> Result<i32, DriverError> {
    let (target, pkg, _) = load_and_validate_schema_typed(schema, true)?;
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
        .map_err(|e| DriverError::Codegen(1, format!("Error during forward sync: {e}")))?;
    Ok(0)
}

/// Executes `--reverse-sync` mode (`CommandMode::ReverseSync`).
/// Realises: [REQ-COMPILER-SYSML-HANDLE-REVERSE-SYNC]
pub(crate) fn handle_reverse_sync(
    opts: ReverseSyncOptions,
    dry_run: bool,
) -> Result<i32, DriverError> {
    let schema = opts.schema_path.clone();
    let base_pkg = if schema.is_some() || resolve_schema_targets_typed(None).is_ok() {
        let (_, pkg, _) = load_and_validate_schema_typed(schema, false)?;
        Some(pkg)
    } else {
        None
    };

    reverse_sync_specs_to_sysml_with_dry_run(base_pkg.as_ref(), &opts, dry_run)
        .map_err(|e| DriverError::Codegen(1, format!("Error during reverse sync: {e}")))?;
    Ok(0)
}

/// Executes `--compile` mode (`CommandMode::Compile`).
/// Realises: [REQ-COMPILER-SYSML-HANDLE-COMPILE]
pub(crate) fn handle_compile(
    schema: Option<PathBuf>,
    out: &Path,
    digest: &Path,
    dry_run: bool,
) -> Result<i32, DriverError> {
    run_compilation_gate_with_dry_run(schema, out, digest, dry_run)
}
