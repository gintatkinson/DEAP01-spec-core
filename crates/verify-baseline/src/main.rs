//! CLI entrypoint for the `verify-baseline` deterministic verification utility.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides the primary command-line interface and subcommand dispatcher for
//! deterministic baseline verification, commit-message non-closure checks,
//! subagent output integrity verification, 7-section defect dossier validation,
//! native git hook installation, and post-build secret scanning under DO-178C
//! Level A, DO-330 TQL-4, and ISO 26262 ASIL D tool qualification governance.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Dispatch determinism: Every parsed CLI invocation maps to exactly one
//!   execution branch (`Subcommands` variant or default `BaselineOptions` audit run).
//! - Exit-code monotonicity: Any verification failure (`Err`) terminates the
//!   process with a non-zero exit code (`>= 1`); exit code `0` is emitted if and
//!   only if the selected verification routine returns `Ok(())`.
//!
//! ## 3. Preconditions
//! - Command-line arguments provided by the host operating system must conform
//!   to the `Cli` grammar enforced by `clap::Command`.
//! - Target filesystem paths supplied via arguments must be accessible to the
//!   invoking user process.
//!
//! ## 4. Postconditions
//! - On verification success, all enabled baseline or subcommand checks have
//!   completed without violation and the process exits with status `0`.
//! - On verification failure, diagnostic telemetry is emitted and the process
//!   terminates immediately via `std::process::exit` with a non-zero status code.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - CLI argument parsing and dispatch overhead is $O(A)$ in time and space,
//!   where $A$ is the number of command-line tokens passed to the binary.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Invalid CLI flags or malformed subcommand syntax are intercepted by `clap`
//!   before execution, emitting structured usage diagnostics and exiting non-zero.
//! - Downstream audit failures propagate explicit error codes to `process::exit`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-VERIFY-BASELINE-CLI], [DO-330-TQL-4-VERIFY-CLI]

mod cli_parser;

use cli_parser::{resolve_audit_workspace_root, Cli, Subcommands};
use std::path::PathBuf;
use std::process;
use verify_baseline::audit_code;
use verify_baseline::blueprint_gate;
use verify_baseline::check_secrets;
use verify_baseline::commit_msg;
use verify_baseline::defect_dossier;
use verify_baseline::output_integrity;
use verify_baseline::runner::{run, BaselineOptions};
use verify_baseline::setup_hooks;

/// Dispatches a parsed [`Subcommands`] variant and exits non-zero on failure.
/// Realises: [REQ-VERIFY-BASELINE-MAIN]
fn dispatch_subcommand(destination: &str, cmd: Subcommands) {
    match cmd {
        Subcommands::CommitMsg(args) => {
            if commit_msg::run_check_commit_msg(&args).is_err() {
                process::exit(1);
            }
        }
        Subcommands::OutputIntegrity(args) => {
            if let Err(code) = output_integrity::run_check_output(&args) {
                process::exit(code);
            }
        }
        Subcommands::Defect(args) => {
            if let Err(code) = defect_dossier::run_check_defect(&args) {
                process::exit(code);
            }
        }
        Subcommands::Blueprint(opts) => {
            let workspace_root = PathBuf::from(destination);
            let code = blueprint_gate::run_blueprint_cli(&workspace_root, &opts);
            if code != 0 {
                process::exit(code);
            }
        }
        Subcommands::AuditCode(mut args) => {
            let anchored_path = if args.path.is_absolute() {
                args.path
            } else {
                match std::env::current_dir() {
                    Ok(cwd) => cwd.join(&args.path),
                    Err(_) => args.path,
                }
            };
            let workspace_root = resolve_audit_workspace_root(&anchored_path);
            args.path = anchored_path;
            if let Err(code) = audit_code::run_audit_code(&workspace_root, &args) {
                process::exit(code);
            }
        }
        Subcommands::SetupHooks(args) => {
            if let Err(code) = setup_hooks::run_setup_hooks(&args) {
                process::exit(code);
            }
        }
        Subcommands::Secrets(args) => {
            if let Err(code) = check_secrets::run_check_secrets(&args) {
                process::exit(code);
            }
        }
    }
}

/// Executes the `verify-baseline` CLI entrypoint and dispatches subcommands.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Parses process command-line arguments and routes execution deterministically
/// to either the requested verification subcommand (`check-commit-msg`,
/// `check-output`, `check-defect`, `setup-hooks`, `check-secrets`) or the full
/// baseline verification runner under DO-178C Level A / ISO 26262 ASIL D
/// tool qualification controls.
///
/// ## 2. Invariants & Mathematical Properties
/// - Total branch coverage: Either `Some(cmd)` dispatches to its exact module
///   handler, or `None` constructs a complete `BaselineOptions` record and
///   invokes `verify_baseline::runner::run`.
/// - Non-zero fault exit: Every `Err` result triggers `std::process::exit` with
///   a non-zero exit status.
///
/// ## 3. Preconditions
/// - Invoked as the main OS process entrypoint with valid UTF-8 process arguments.
///
/// ## 4. Postconditions
/// - Returns normally (exit code `0`) if and only if the dispatched verification
///   gate succeeds without errors.
/// - Calls `std::process::exit` with a non-zero code on any verification failure.
///
/// ## 5. Algorithmic Complexity Bounds
/// - $O(A)$ argument parsing and struct construction prior to subcommand execution,
///   where $A$ is the count of CLI argument strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Propagates subcommand-specific exit codes directly to the OS process termination
///   handler without panicking or silently ignoring errors.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-VERIFY-BASELINE-MAIN]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(res) = verify_baseline::ndjson_cli::try_run_ndjson_cli(&args) {
        if let Err(e) = res {
            eprintln!("{e}");
            process::exit(1);
        }
        return;
    }
    let cli = Cli::parse();

    if let Some(cmd) = cli.command {
        dispatch_subcommand(&cli.destination, cmd);
    } else {
        let options = BaselineOptions {
            destination: PathBuf::from(&cli.destination),
            no_domain: cli.no_domain,
            strict: cli.strict,
            allow_missing_specs: cli.allow_missing_specs,
            target: cli.target,
            output: cli.output,
            spec_only: cli.spec_only,
            only: cli.only,
            gate: cli.gate,
        };

        if run(&options).is_err() {
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_verify_baseline_main_7_section_rustdoc() {
        let src = include_str!("main.rs");
        let module_header = src.split("\nmod ").next().unwrap_or("");
        assert!(module_header.contains("//! ## 1. Safety Intent & Regulatory Scope"));
        assert!(module_header.contains("//! ## 2. Invariants & Mathematical Properties"));
        assert!(module_header.contains("//! ## 3. Preconditions"));
        assert!(module_header.contains("//! ## 4. Postconditions"));
        assert!(module_header.contains("//! ## 5. Algorithmic Complexity Bounds"));
        assert!(module_header.contains("//! ## 6. Failure Modes & Fault Tolerance"));
        assert!(module_header.contains("//! ## 7. Realization & Traceability"));
        assert!(module_header
            .contains("//! Realises: [REQ-VERIFY-BASELINE-CLI], [DO-330-TQL-4-VERIFY-CLI]"));
        assert!(src.contains("/// ## 1. Safety Intent & Regulatory Scope"));
        assert!(src.contains("/// ## 7. Realization & Traceability"));
        assert!(src.contains("/// Realises: [REQ-VERIFY-BASELINE-MAIN]"));
        let prod_src = src.split("#[cfg(test)]").next().unwrap_or("");
        assert!(!prod_src.contains("process::exit(0)"));
    }
}
