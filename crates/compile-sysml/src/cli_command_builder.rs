#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] Declarative `clap::Command` builder and `ArgMatches` extractor for `compile-sysml`.
//! Safety & Traceability: Realises: [REQ-COMPILER-SYSML-CLI-COMMAND], [REQ-0608], [DO-330-TQL-4-CLI]
//! Contract: Constructs the explicit `clap::Command` grammar and extracts validated `Cli` records without `clap_derive` suppressions.
//! Bounds & Faults: O(A) time and space in CLI argument token count `A`; zero `.unwrap()` or `.expect()` calls.

use crate::cli::{Cli, MessageFormat};
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::PathBuf;

/// Constructs the explicit `clap::Command` grammar for `compile-sysml` including Track-1 CST flags.
/// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-CLI-COMMAND], [REQ-0608]
/// Contract: Registers all pipeline, codegen, docstring, and Track-1 CST refactoring arguments.
/// Bounds & Faults: O(1) static schema construction; infallible.
#[must_use]
pub(crate) fn build_cli_command() -> Command {
    let base = Command::new("compile-sysml")
        .about("SysML v2 / KerML compiler and specification synchronization engine");
    let with_core = register_core_pipeline_args(base);
    let with_codegen = register_codegen_args(with_core);
    let with_output = register_output_path_args(with_codegen);
    register_track1_cst_args(with_output)
}

/// Registers primary SysML v2 input, compilation, STPA, and synchronization flags on `cmd`.
/// Realises: [REQ-COMPILER-SYSML-CLI-COMMAND]
fn register_core_pipeline_args(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("file")
            .value_name("FILE")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Path to input .sysml file"),
    )
    .arg(
        Arg::new("schema")
            .long("schema")
            .value_name("SCHEMA_FILE")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Path to schema file or directory (default searches schema/*.sysml)"),
    )
    .arg(
        Arg::new("compile")
            .long("compile")
            .action(ArgAction::SetTrue)
            .help("Run the Pipeline 0 compilation gate: parse, validate semantics, serialize, and emit digest"),
    )
    .arg(
        Arg::new("stpa_transpile")
            .long("stpa-transpile")
            .action(ArgAction::SetTrue)
            .help("Transpile SysML AST into the 10-pillar STPA safety artifact suite"),
    )
    .arg(
        Arg::new("stpa")
            .long("stpa")
            .value_name("STPA_FILE")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Path to STPA markdown file to compile into SysML constraint notation"),
    )
    .arg(
        Arg::new("reverse_sync")
            .long("reverse-sync")
            .action(ArgAction::SetTrue)
            .help("Execute reverse synchronization from markdown specs into SysML SSOT"),
    )
    .arg(
        Arg::new("forward_sync")
            .long("forward-sync")
            .action(ArgAction::SetTrue)
            .help("Execute forward synchronization from SysML SSOT into markdown specs"),
    )
}

/// Registers docstring synthesis, source extraction, MBD export, package slicing, and codegen flags on `cmd`.
/// Realises: [REQ-COMPILER-SYSML-CLI-COMMAND]
fn register_codegen_args(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("inject_safety_docstrings")
            .long("inject-safety-docstrings")
            .value_name("DIR")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Synthesize and inject 7-section safety docstrings into source files under DIR"),
    )
    .arg(
        Arg::new("extract_source")
            .long("extract-source")
            .value_name("DIR")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Extract structural source AST elements from DIR into SysML v2 notation"),
    )
    .arg(
        Arg::new("export_mbd")
            .long("export-mbd")
            .value_name("DIR")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Export grounded MBD verification artifacts (.slreqx, .sfx, .m, layer_contracts.json) to DIR"),
    )
    .arg(
        Arg::new("slice_package")
            .long("slice-package")
            .value_name("PACKAGE")
            .action(ArgAction::Set)
            .help("Slice a target package subsystem AST within a deterministic token budget"),
    )
    .arg(
        Arg::new("emit_flutter_contracts")
            .long("emit-flutter-contracts")
            .value_name("FLUTTER_DIR")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Emit deterministic Flutter and C-ABI contracts (seeds, models, solver bridge, and UI tests)"),
    )
    .arg(
        Arg::new("emit_modular_rust")
            .long("emit-modular-rust")
            .value_name("RUST_DIR")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Emit deterministic 4-level modular multi-file Rust code to RUST_DIR"),
    )
}

/// Registers documentation, output schema, digest, execution mode, and scoring configuration flags on `cmd`.
/// Realises: [REQ-COMPILER-SYSML-CLI-COMMAND]
fn register_output_path_args(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("docs")
            .long("docs")
            .default_value("docs/")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Path to documentation directory"),
    )
    .arg(
        Arg::new("out")
            .long("out")
            .default_value(".pipeline/schema.sysml")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Path to output schema model file"),
    )
    .arg(
        Arg::new("out_dir")
            .long("out-dir")
            .value_name("OUT_DIR")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Path to explicit output directory (for --stpa-transpile or --forward-sync)"),
    )
    .arg(
        Arg::new("digest")
            .long("digest")
            .default_value(".pipeline/schema-digest.json")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Path to output schema digest JSON"),
    )
    .arg(
        Arg::new("force")
            .long("force")
            .action(ArgAction::SetTrue)
            .help("Force overwrite of existing files / bypass landing zone guard"),
    )
    .arg(
        Arg::new("dry_run")
            .long("dry-run")
            .action(ArgAction::SetTrue)
            .help("Simulate synchronization without writing changes to disk"),
    )
    .arg(
        Arg::new("scoring_config")
            .long("scoring-config")
            .value_name("SCORING_CONFIG")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Path to optional FMECA scoring configuration JSON"),
    )
}

/// Registers Track-1 CST inspection, module extraction, Tier-B/C compaction, NDJSON, and machine-fix flags on `cmd`.
/// Realises: [REQ-0608], [SC-01], [FLS: fls_compile_cst_cli]
fn register_track1_cst_args(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("list_rust_items")
            .long("list-rust-items")
            .value_name("FILE")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("List top-level Rust CST items and line spans from FILE"),
    )
    .arg(
        Arg::new("extract_rust_module")
            .long("extract-rust-module")
            .value_name("SRC")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Extract named top-level Rust CST items from SRC into DEST"),
    )
    .arg(
        Arg::new("items")
            .long("items")
            .value_name("CSV")
            .action(ArgAction::Set)
            .help("Comma-separated item identifiers for --extract-rust-module"),
    )
    .arg(
        Arg::new("dest")
            .long("dest")
            .value_name("DEST")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Destination submodule file path for --extract-rust-module"),
    )
    .arg(
        Arg::new("compact_tier_bc")
            .long("compact-tier-bc")
            .value_name("PATH")
            .action(ArgAction::Set)
            .value_parser(clap::value_parser!(PathBuf))
            .help("Compact verbose Tier-B/C Rust docstrings under PATH to <= 4 lines"),
    )
    .arg(
        Arg::new("message_format")
            .long("message-format")
            .value_name("FORMAT")
            .default_value("human")
            .action(ArgAction::Set)
            .help("Diagnostic output format: human or json (NDJSON)"),
    )
    .arg(
        Arg::new("apply_machine_fixes")
            .long("apply-machine-fixes")
            .action(ArgAction::SetTrue)
            .help("Apply MachineApplicable TextEdit fixes to target Rust source files"),
    )
}

/// Extracts a strongly-typed `Cli` struct from validated `clap::ArgMatches`.
/// Safety & Traceability: Realises: [REQ-COMPILER-SYSML-CLI-MATCHES], [REQ-0608]
/// Contract: Maps every matched CLI option and flag deterministically to its corresponding `Cli` field.
/// Bounds & Faults: O(1) over fixed fields; uses `.unwrap_or_else` fallbacks without panicking.
#[must_use]
pub(crate) fn cli_from_matches(matches: &ArgMatches) -> Cli {
    let message_format = matches
        .get_one::<String>("message_format")
        .map(|s| MessageFormat::from_flag_lossless(s))
        .unwrap_or(MessageFormat::Human);

    Cli {
        file: matches.get_one::<PathBuf>("file").cloned(),
        schema: matches.get_one::<PathBuf>("schema").cloned(),
        compile: matches.get_flag("compile"),
        stpa_transpile: matches.get_flag("stpa_transpile"),
        stpa: matches.get_one::<PathBuf>("stpa").cloned(),
        reverse_sync: matches.get_flag("reverse_sync"),
        forward_sync: matches.get_flag("forward_sync"),
        inject_safety_docstrings: matches
            .get_one::<PathBuf>("inject_safety_docstrings")
            .cloned(),
        extract_source: matches.get_one::<PathBuf>("extract_source").cloned(),
        export_mbd: matches.get_one::<PathBuf>("export_mbd").cloned(),
        slice_package: matches.get_one::<String>("slice_package").cloned(),
        emit_flutter_contracts: matches
            .get_one::<PathBuf>("emit_flutter_contracts")
            .cloned(),
        emit_modular_rust: matches.get_one::<PathBuf>("emit_modular_rust").cloned(),
        list_rust_items: matches.get_one::<PathBuf>("list_rust_items").cloned(),
        extract_rust_module: matches.get_one::<PathBuf>("extract_rust_module").cloned(),
        items: matches.get_one::<String>("items").cloned(),
        dest: matches.get_one::<PathBuf>("dest").cloned(),
        compact_tier_bc: matches.get_one::<PathBuf>("compact_tier_bc").cloned(),
        message_format,
        apply_machine_fixes: matches.get_flag("apply_machine_fixes"),
        docs: matches
            .get_one::<PathBuf>("docs")
            .cloned()
            .unwrap_or_else(|| PathBuf::from("docs/")),
        out: matches
            .get_one::<PathBuf>("out")
            .cloned()
            .unwrap_or_else(|| PathBuf::from(".pipeline/schema.sysml")),
        out_dir: matches.get_one::<PathBuf>("out_dir").cloned(),
        digest: matches
            .get_one::<PathBuf>("digest")
            .cloned()
            .unwrap_or_else(|| PathBuf::from(".pipeline/schema-digest.json")),
        force: matches.get_flag("force"),
        dry_run: matches.get_flag("dry_run"),
        scoring_config: matches.get_one::<PathBuf>("scoring_config").cloned(),
    }
}
