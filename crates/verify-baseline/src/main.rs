//! CLI entrypoint for verify-baseline utility.

use clap::Parser;
use std::path::PathBuf;
use std::process;
use verify_baseline::check_secrets::{self, CheckSecretsArgs};
use verify_baseline::commit_msg::{self, CommitMsgArgs};
use verify_baseline::defect_dossier::{self, DefectDossierArgs};
use verify_baseline::output_integrity::{self, OutputIntegrityArgs};
use verify_baseline::runner::{run, BaselineOptions};
use verify_baseline::setup_hooks::{self, SetupHooksArgs};

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
