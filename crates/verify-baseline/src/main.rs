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
//!   to the `Cli` grammar enforced by `clap::Parser`.
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

use clap::Parser;
use std::path::PathBuf;
use std::process;
use verify_baseline::check_secrets::{self, CheckSecretsArgs};
use verify_baseline::commit_msg::{self, CommitMsgArgs};
use verify_baseline::defect_dossier::{self, DefectDossierArgs};
use verify_baseline::output_integrity::{self, OutputIntegrityArgs};
use verify_baseline::runner::{run, BaselineOptions};
use verify_baseline::setup_hooks::{self, SetupHooksArgs};

/// Executes deterministic struct `Cli` within `crates/verify-baseline/src/main.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `Cli` in `crates/verify-baseline/src/main.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `Cli` across member set `[command: Option<Subcommands>, destination: String, no_domain: bool, strict: bool, allow_missing_specs: bool]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `Cli` members (`command: Option<Subcommands>, destination: String, no_domain: bool, strict: bool, allow_missing_specs: bool`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `Cli` representation for struct `Cli`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `Cli`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `Cli`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `Cli` (`Cli`).
/// - State: Maintains consistent state invariants across `Cli` without partial mutation on error.
/// - Action: Executes `Cli` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `Cli` (`VERIFY-BASELINE-MAIN`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-VERIFY-BASELINE-MAIN-CLI] | SysML v2: PartDef(Cli)
#[derive(Parser, Debug)]
#[command(name = "verify-baseline")]
#[command(about = "Verify downstream project baseline conformance", long_about = None)]
#[command(subcommand_precedence_over_arg = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Subcommands>,

    /// Target destination directory or repository root (defaults to current directory)
    #[arg(default_value = ".")]
    destination: String,

    /// Skip domain type compatibility validation and build/test commands
    #[arg(long)]
    no_domain: bool,

    /// Enforce strict baseline conformance (reject pending or missing specifications)
    #[arg(long)]
    strict: bool,

    /// Gracefully allow missing specifications if in downstream customer mode
    #[arg(long)]
    allow_missing_specs: bool,

    /// Target execution platform profile (e.g. flutter, react)
    #[arg(long)]
    target: Option<String>,

    /// Optional path to write output report or defect dossier
    #[arg(long)]
    output: Option<String>,

    /// Run only the specification quality and coverage audit suite
    #[arg(long)]
    spec_only: bool,

    /// Audit only a specific markdown file
    #[arg(long)]
    only: Option<String>,

    /// Filter to run a specific audit gate
    #[arg(long)]
    gate: Option<String>,
}

/// Executes deterministic enum `Subcommands` within `crates/verify-baseline/src/main.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `Subcommands` in `crates/verify-baseline/src/main.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `Subcommands` across member set `[CheckCommitMsg, CheckOutput, CheckDefect, SetupHooks, CheckSecrets]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `Subcommands` members (`CheckCommitMsg, CheckOutput, CheckDefect, SetupHooks, CheckSecrets`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `Subcommands` representation for enum `Subcommands`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `Subcommands`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `Subcommands`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `Subcommands` (`Subcommands`).
/// - State: Maintains consistent state invariants across `Subcommands` without partial mutation on error.
/// - Action: Executes `Subcommands` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `Subcommands` (`VERIFY-BASELINE-MAIN`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-VERIFY-BASELINE-MAIN-SUBCOMMANDS] | SysML v2: PartDef(Subcommands)
#[derive(clap::Subcommand, Debug)]
#[allow(clippy::enum_variant_names)]
enum Subcommands {
    /// Port of verify_commit_messages: check commit message non-closure invariant
    #[command(name = "check-commit-msg", alias = "check-commit")]
    CheckCommitMsg(CommitMsgArgs),

    /// Port of verify_subagent_output: check markdown size, escape tokens, code fences
    #[command(name = "check-output", alias = "verify-subagent-output")]
    CheckOutput(OutputIntegrityArgs),

    /// Port of file_defect validator: check 7-section defect dossier schema
    #[command(name = "check-defect", alias = "verify-defect-dossier")]
    CheckDefect(DefectDossierArgs),

    /// Setup cross-platform git hooks natively
    #[command(name = "setup-hooks")]
    SetupHooks(SetupHooksArgs),

    /// Check post-build assets for leaked credentials and secrets
    #[command(name = "check-secrets")]
    CheckSecrets(CheckSecretsArgs),
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
    let cli = Cli::parse();

    if let Some(cmd) = cli.command {
        match cmd {
            Subcommands::CheckCommitMsg(args) => {
                if let Err(()) = commit_msg::run_check_commit_msg(&args) {
                    process::exit(1);
                }
            }
            Subcommands::CheckOutput(args) => {
                if let Err(code) = output_integrity::run_check_output(&args) {
                    process::exit(code);
                }
            }
            Subcommands::CheckDefect(args) => {
                if let Err(code) = defect_dossier::run_check_defect(&args) {
                    process::exit(code);
                }
            }
            Subcommands::SetupHooks(args) => {
                if let Err(code) = setup_hooks::run_setup_hooks(&args) {
                    process::exit(code);
                }
            }
            Subcommands::CheckSecrets(args) => {
                if let Err(code) = check_secrets::run_check_secrets(&args) {
                    process::exit(code);
                }
            }
        }
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

        if let Err(()) = run(&options) {
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_verify_baseline_main_7_section_rustdoc() {
        let src = include_str!("main.rs");
        let module_header = src.split("\nuse ").next().unwrap_or("");
        assert!(module_header.contains("//! ## 1. Safety Intent & Regulatory Scope"));
        assert!(module_header.contains("//! ## 2. Invariants & Mathematical Properties"));
        assert!(module_header.contains("//! ## 3. Preconditions"));
        assert!(module_header.contains("//! ## 4. Postconditions"));
        assert!(module_header.contains("//! ## 5. Algorithmic Complexity Bounds"));
        assert!(module_header.contains("//! ## 6. Failure Modes & Fault Tolerance"));
        assert!(module_header.contains("//! ## 7. Realization & Traceability"));
        assert!(module_header.contains("//! Realises: [REQ-VERIFY-BASELINE-CLI], [DO-330-TQL-4-VERIFY-CLI]"));
        assert!(src.contains("/// ## 1. Safety Intent & Regulatory Scope"));
        assert!(src.contains("/// ## 7. Realization & Traceability"));
        assert!(src.contains("/// Realises: [REQ-VERIFY-BASELINE-MAIN]"));
    }
}
