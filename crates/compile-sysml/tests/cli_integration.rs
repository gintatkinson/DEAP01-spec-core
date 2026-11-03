#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] CLI & Driver Integration Tests for `compile-sysml` (`TD-11`, `TD-15`, `TD-18`, `TD-19`).
//! ## 1. Safety Intent & Regulatory Scope
//! Verifies Track-1 CST refactoring flags (`--list-rust-items`, `--extract-rust-module`, `--compact-tier-bc`,
//! `--message-format`, `--apply-machine-fixes`) and SESE driver gates under DO-178C Level A and DO-330 TQL-4.
//! ## 2. Invariants & Mathematical Properties
//! - Conflicting operational mode flags fail closed with non-zero exit codes and zero partial writes.
//! - `--dry-run` validates and lowers schemas in memory without mutating disk state.
//! ## 3. Preconditions
//! - Writable OS temporary directory for isolated test workspaces.
//! ## 4. Postconditions
//! - Temporary workspaces are removed prior to test completion.
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ over temporary test files; Space Complexity: $O(N)$.
//! ## 6. Failure Modes & Fault Tolerance
//! - Propagates CLI parse and I/O errors explicitly via `Result`.
//! ## 7. Realization & Traceability
//! Safety & Traceability: Realises: [REQ-COMPILER-SYSML-CLI], [REQ-0608], [DO-330-TQL-4-CLI]
//! Contract: Verifies Track-1 CST refactoring flags and SESE driver gates.
//! Bounds & Faults: Self-cleaning temporary directories in `std::env::temp_dir()`; deterministic assertions.

use crate::cli::{Cli, CommandMode, MessageFormat};
use crate::driver::{
    dispatch_pipeline_mode, execute, execute_cli, resolve_schema_targets, run_compilation_gate,
    DriverError, SchemaTarget,
};
use crate::driver_cst_tracks::format_driver_ndjson_line;
use crate::driver_mode_handlers::{
    handle_compile, handle_slice_package, handle_stpa_to_constraints,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static CLI_TEST_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Creates an isolated temporary workspace directory for CLI and driver integration tests.
/// Realises: [REQ-COMPILER-SYSML-CLI]
fn unique_cli_temp_dir(prefix: &str) -> PathBuf {
    let seq = CLI_TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        ".deap_cli_int_test_{}_{}_{}",
        prefix,
        std::process::id(),
        seq
    ));
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Seeds the initial parent Rust module fixture for Track-1 CLI flag testing.
/// Realises: [REQ-COMPILER-SYSML-CLI]
fn write_track1_parent_fixture(src_file: &Path) -> Result<(), String> {
    let initial_rust = "\
#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] Parent test module.
//! Safety & Traceability: Realises: [REQ-TEST-01]
//! Contract: Parent contract.
//! Bounds & Faults: O(1).

/// Verbose private line 1.
/// Verbose line 2.
/// Verbose line 3.
/// Verbose line 4.
/// Realises: [REQ-TEST-01]
fn helper_alpha() -> i32 {
    10
}

/// Summary for beta.
/// Safety & Traceability: Realises: [REQ-TEST-01]
/// Contract: Returns 20.
/// Bounds & Faults: O(1).
pub fn moved_beta() -> i32 {
    20
}
";
    fs::write(src_file, initial_rust).map_err(|e| e.to_string())
}

#[test]
fn test_track1_cli_flags_parse_and_execute_end_to_end() -> Result<(), String> {
    let dir = unique_cli_temp_dir("track1_e2e");
    let src_file = dir.join("parent_mod.rs");
    let dest_file = dir.join("extracted_child.rs");
    write_track1_parent_fixture(&src_file)?;

    let list_cli = Cli::parse_from([
        "compile-sysml",
        "--list-rust-items",
        &src_file.to_string_lossy(),
        "--message-format",
        "json",
    ]);
    assert_eq!(list_cli.message_format, MessageFormat::Json);
    assert_eq!(execute_cli(list_cli), 0);

    let compact_cli = Cli::parse_from([
        "compile-sysml",
        "--compact-tier-bc",
        &src_file.to_string_lossy(),
    ]);
    assert_eq!(execute_cli(compact_cli), 0);
    let after_compact = fs::read_to_string(&src_file).map_err(|e| e.to_string())?;
    assert!(!after_compact.contains("Verbose line 3."));

    let extract_cli = Cli::parse_from([
        "compile-sysml",
        "--extract-rust-module",
        &src_file.to_string_lossy(),
        "--items",
        "moved_beta",
        "--dest",
        &dest_file.to_string_lossy(),
    ]);
    assert_eq!(execute_cli(extract_cli), 0);
    assert!(dest_file.is_file(), "extracted submodule file must exist");
    let child_src = fs::read_to_string(&dest_file).map_err(|e| e.to_string())?;
    let parent_src = fs::read_to_string(&src_file).map_err(|e| e.to_string())?;
    assert!(child_src.contains("pub fn moved_beta()"));
    assert!(parent_src.contains("mod extracted_child;"));

    let fix_cli = Cli::parse_from([
        "compile-sysml",
        "--inject-safety-docstrings",
        &src_file.to_string_lossy(),
        "--apply-machine-fixes",
    ]);
    let fix_mode = fix_cli.clone().into_mode()?;
    assert!(matches!(fix_mode, CommandMode::ApplyMachineFixes { .. }));
    let _ = execute_cli(fix_cli);

    let ndjson = format_driver_ndjson_line("CST-01", "info", "check", "src/lib.rs", true);
    assert!(ndjson.contains("\"code\":\"CST-01\""));
    assert!(ndjson.contains("\"MachineApplicable\""));

    let _ = fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn test_cli_parses_compile_and_sync_flags_and_rejects_conflicts() -> Result<(), String> {
    let cli = Cli::try_parse_from([
        "compile-sysml",
        "--compile",
        "--schema",
        "schema/test.sysml",
        "--dry-run",
    ])
    .map_err(|e| e.to_string())?;
    assert!(cli.compile);
    assert!(cli.dry_run);
    assert_eq!(
        cli.into_mode()?,
        CommandMode::Compile {
            schema: Some(PathBuf::from("schema/test.sysml")),
            out: PathBuf::from(".pipeline/schema.sysml"),
            digest: PathBuf::from(".pipeline/schema-digest.json"),
            dry_run: true,
        }
    );

    let conflict_cli = Cli::try_parse_from(["compile-sysml", "--compile", "--forward-sync"])
        .map_err(|e| e.to_string())?;
    let err = conflict_cli
        .into_mode()
        .err()
        .ok_or("expected conflict error")?;
    assert!(err.contains("Multiple operational mode flags specified"));
    Ok(())
}

#[test]
fn test_resolve_schema_targets_and_subworkspace_compilation_gate(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_ws = unique_cli_temp_dir("subworkspace");
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

    let fallback_resolved = resolve_schema_targets(Some(schema_dir.clone()));
    assert_eq!(
        fallback_resolved,
        Ok(SchemaTarget::SingleFile(pipeline_schema.clone()))
    );

    let resolved = resolve_schema_targets(Some(temp_ws.clone()));
    let code = run_compilation_gate(Some(temp_ws.clone()), &pipeline_schema, &pipeline_digest);
    let model_exists = schema_dir.join("model.sysml").is_file();
    let _ = fs::remove_dir_all(&temp_ws);

    assert_eq!(resolved, Ok(SchemaTarget::SingleFile(pipeline_schema)));
    assert_eq!(code, 0);
    assert!(model_exists);
    Ok(())
}

#[test]
fn test_arch_01_and_02_fail_closed_validation_in_driver_modes(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_ws = unique_cli_temp_dir("arch01_02");
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
        &bad_schema.to_string_lossy(),
        "--docs",
        &docs_dir.to_string_lossy(),
        "--out",
        &out_schema.to_string_lossy(),
        "--digest",
        &out_digest.to_string_lossy(),
        "--force",
    ]);
    assert!(matches!(execute(rev_cli), Err((1, _))));
    assert!(!out_schema.exists());

    let invalid_schema = temp_ws.join("invalid_semantics.sysml");
    fs::write(
        &invalid_schema,
        "package InvalidPkg {\n    part def ValidController {\n        part sub_unit : NonExistentComponentType;\n    }\n}\n",
    )?;
    let slice_cli = Cli::parse_from(vec![
        "compile-sysml",
        "--slice-package",
        "InvalidPkg",
        "--schema",
        &invalid_schema.to_string_lossy(),
    ]);
    let slice_err = execute(slice_cli).err().ok_or("SlicePackage must reject")?;
    let _ = fs::remove_dir_all(&temp_ws);
    assert_eq!(slice_err.0, 1);
    assert!(slice_err.1.contains("Semantic validation error"));
    Ok(())
}

#[test]
fn test_arch_04_and_05_dry_run_and_driver_error_contracts(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_ws = unique_cli_temp_dir("arch04_err");
    let valid_schema = temp_ws.join("valid.sysml");
    fs::write(
        &valid_schema,
        "package ValidPkg {\n    part def ControllerUnit {\n        attribute gain : Real;\n        action def ComputeStep;\n    }\n}\n",
    )?;

    let compiled_out = temp_ws.join("compiled.sysml");
    let compiled_digest = temp_ws.join("compiled-digest.json");
    let compile_cli = Cli::parse_from(vec![
        "compile-sysml",
        "--compile",
        "--schema",
        &valid_schema.to_string_lossy(),
        "--out",
        &compiled_out.to_string_lossy(),
        "--digest",
        &compiled_digest.to_string_lossy(),
        "--dry-run",
    ]);
    assert_eq!(execute_cli(compile_cli), 0);
    assert!(!compiled_out.exists());
    assert!(!compiled_digest.exists());

    let cli_err = dispatch_pipeline_mode(CommandMode::ExportMbd {
        mbd_dir: temp_ws.join("mbd"),
        schema: None,
        dry_run: true,
    })
    .err()
    .ok_or("expected DriverError::Cli")?;
    assert!(matches!(cli_err, DriverError::Cli(_)));

    let missing_err = handle_compile(
        Some(temp_ws.join("missing.sysml")),
        &compiled_out,
        &compiled_digest,
        true,
    )
    .err()
    .ok_or("expected DriverError::Resolution")?;
    assert!(matches!(missing_err, DriverError::Resolution(_)));

    let dir_as_stpa = temp_ws.join("stpa_dir");
    fs::create_dir_all(&dir_as_stpa)?;
    let io_err = handle_stpa_to_constraints(&dir_as_stpa, None, &compiled_out, true)
        .err()
        .ok_or("expected DriverError::Io")?;
    assert!(matches!(io_err, DriverError::Io(_)));

    let codegen_err = handle_slice_package("UnknownPkg", Some(valid_schema))
        .err()
        .ok_or("expected DriverError::Codegen")?;
    let _ = fs::remove_dir_all(&temp_ws);
    assert!(matches!(codegen_err, DriverError::Codegen(1, _)));
    let tuple: (i32, String) = codegen_err.into();
    assert_eq!(tuple.0, 1);
    Ok(())
}
