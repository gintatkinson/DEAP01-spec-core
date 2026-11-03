//! Command-line argument grammar and parser helpers for `verify-baseline` (`[REQ-VERIFY-BASELINE-CLI]`).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides deterministic `clap::Command` builders and `ArgMatches` extraction helpers for the
//! `verify-baseline` binary without `clap_derive` proc-macro lint-suppression injections under
//! DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D tool qualification standards.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Preserves 100% flag, alias, positional argument, and subcommand parity across `check-commit-msg`,
//!   `check-output`, `check-defect`, `check-blueprint`, `audit-code`, `setup-hooks`, and `check-secrets`.
//!
//! ## 3. Preconditions
//! - Process environment supplies valid OS argument tokens via `std::env::args_os`.
//!
//! ## 4. Postconditions
//! - Returns a populated [`Cli`] instance with deterministic `command` and top-level baseline flags.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(A)$ where $A$ is the number of CLI argument tokens.
//! - Space Complexity: $O(A)$ for parsed `ArgMatches` and owned [`Cli`] fields.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Invalid flags or missing required values are intercepted by `clap::Command::get_matches`,
//!   emitting usage diagnostics and exiting with a non-zero status code.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-VERIFY-BASELINE-CLI], [REQ-0601-VERIFY-BASELINE-CLI-PARSE]

use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::{Path, PathBuf};
use verify_baseline::audit_code::{AuditCodeArgs, AuditPillar};
use verify_baseline::blueprint_gate::BlueprintCliOptions;
use verify_baseline::check_secrets::CheckSecretsArgs;
use verify_baseline::commit_msg::CommitMsgArgs;
use verify_baseline::defect_dossier::DefectDossierArgs;
use verify_baseline::output_integrity::OutputIntegrityArgs;
use verify_baseline::setup_hooks::SetupHooksArgs;

/// Parsed CLI configuration for `verify-baseline`.
/// Realises: [REQ-VERIFY-BASELINE-MAIN-CLI]
#[derive(Debug)]
pub(crate) struct Cli {
    pub(crate) command: Option<Subcommands>,
    pub(crate) destination: String,
    pub(crate) no_domain: bool,
    pub(crate) strict: bool,
    pub(crate) allow_missing_specs: bool,
    pub(crate) target: Option<String>,
    pub(crate) output: Option<String>,
    pub(crate) spec_only: bool,
    pub(crate) only: Option<String>,
    pub(crate) gate: Option<String>,
}

/// Supported subcommands for `verify-baseline`.
/// Realises: [REQ-VERIFY-BASELINE-MAIN-SUBCOMMANDS]
#[derive(Debug)]
pub(crate) enum Subcommands {
    CommitMsg(CommitMsgArgs),
    OutputIntegrity(OutputIntegrityArgs),
    Defect(DefectDossierArgs),
    Blueprint(BlueprintCliOptions),
    AuditCode(AuditCodeArgs),
    SetupHooks(SetupHooksArgs),
    Secrets(CheckSecretsArgs),
}

/// Constructs top-level `verify-baseline` arguments and attaches all subcommands.
/// Realises: [REQ-0601-VERIFY-BASELINE-CLI-PARSE]
fn build_root_command() -> Command {
    Command::new("verify-baseline")
        .about("Verify downstream project baseline conformance")
        .subcommand_precedence_over_arg(true)
        .arg(
            Arg::new("destination")
                .default_value(".")
                .action(ArgAction::Set)
                .help("Target destination directory or repository root (defaults to current directory)"),
        )
        .arg(
            Arg::new("no_domain")
                .long("no-domain")
                .action(ArgAction::SetTrue)
                .help("Skip domain type compatibility validation and build/test commands"),
        )
        .arg(
            Arg::new("strict")
                .long("strict")
                .action(ArgAction::SetTrue)
                .help("Enforce strict baseline conformance (reject pending or missing specifications)"),
        )
        .arg(
            Arg::new("allow_missing_specs")
                .long("allow-missing-specs")
                .action(ArgAction::SetTrue)
                .help("Gracefully allow missing specifications if in downstream customer mode"),
        )
        .arg(Arg::new("target").long("target").action(ArgAction::Set).help("Target execution platform profile (e.g. flutter, react)"))
        .arg(Arg::new("output").long("output").action(ArgAction::Set).help("Optional path to write output report or defect dossier"))
        .arg(Arg::new("spec_only").long("spec-only").action(ArgAction::SetTrue).help("Run only the specification quality and coverage audit suite"))
        .arg(Arg::new("only").long("only").action(ArgAction::Set).help("Audit only a specific markdown file"))
        .arg(Arg::new("gate").long("gate").action(ArgAction::Set).help("Filter to run a specific audit gate"))
        .subcommand(build_commit_and_output_subcommands().0)
        .subcommand(build_commit_and_output_subcommands().1)
        .subcommand(build_defect_and_blueprint_subcommands().0)
        .subcommand(build_defect_and_blueprint_subcommands().1)
        .subcommand(build_audit_and_hook_subcommands().0)
        .subcommand(build_audit_and_hook_subcommands().1)
        .subcommand(build_audit_and_hook_subcommands().2)
}

/// Builds the `check-commit-msg` and `check-output` subcommand definitions.
/// Realises: [REQ-0601-VERIFY-BASELINE-CLI-PARSE]
fn build_commit_and_output_subcommands() -> (Command, Command) {
    let commit_cmd = Command::new("check-commit-msg")
        .visible_alias("check-commit")
        .about("Port of verify_commit_messages: check commit message non-closure invariant")
        .arg(Arg::new("msg_file").long("msg-file").action(ArgAction::Set).value_parser(clap::value_parser!(PathBuf)).help("Path to file containing commit message"))
        .arg(Arg::new("range").long("range").action(ArgAction::Set).help("Git revision range to inspect (e.g. origin/main..HEAD)"))
        .arg(Arg::new("head").long("head").action(ArgAction::SetTrue).help("Check the latest HEAD commit"))
        .arg(Arg::new("check_text").long("check-text").action(ArgAction::Set).help("Check raw text directly"))
        .arg(Arg::new("target").value_name("TARGET").action(ArgAction::Set).help("Positional file or text to check"));

    let output_cmd = Command::new("check-output")
        .visible_alias("verify-subagent-output")
        .visible_alias("check-output-integrity")
        .about("Port of verify_subagent_output: check markdown size, escape tokens, code fences")
        .arg(Arg::new("dir").long("dir").action(ArgAction::Set).value_parser(clap::value_parser!(PathBuf)).help("Target directory to scan for markdown files"))
        .arg(Arg::new("files").long("files").num_args(1..).action(ArgAction::Append).value_parser(clap::value_parser!(PathBuf)).help("Explicit list of files to verify"))
        .arg(Arg::new("targets").value_name("TARGETS").num_args(0..).action(ArgAction::Append).value_parser(clap::value_parser!(PathBuf)).help("Positional targets (directory or file paths)"));

    (commit_cmd, output_cmd)
}

/// Builds the `check-defect` and `check-blueprint` subcommand definitions.
/// Realises: [REQ-0601-VERIFY-BASELINE-CLI-PARSE]
fn build_defect_and_blueprint_subcommands() -> (Command, Command) {
    let defect_cmd = Command::new("check-defect")
        .visible_alias("verify-defect-dossier")
        .about("Port of file_defect validator: check 7-section defect dossier schema")
        .arg(Arg::new("body_file").long("body-file").action(ArgAction::Set).value_parser(clap::value_parser!(PathBuf)).help("Path to markdown defect dossier body file"))
        .arg(Arg::new("title").long("title").action(ArgAction::Set).help("Title of the defect issue ([AUDIT] [file.ext]: <description>)"))
        .arg(Arg::new("repo").long("repo").action(ArgAction::Set).help("Repository identifier (<owner>/<repo>); auto-resolved from git remote get-url origin when omitted"))
        .arg(Arg::new("label").long("label").action(ArgAction::Set).help("Override label to apply"))
        .arg(Arg::new("provider").long("provider").default_value("github").action(ArgAction::Set).help("Issue tracker provider (github or gitlab)"))
        .arg(Arg::new("dry_run").long("dry-run").visible_alias("validate-only").action(ArgAction::SetTrue).help("Validate defect dossier schema without submitting to remote tracker"))
        .arg(Arg::new("publish").long("publish").action(ArgAction::SetTrue).help("Publish a new issue to the remote GitHub SSOT tracker and run closed-loop verification"))
        .arg(Arg::new("issue").long("issue").action(ArgAction::Set).value_parser(clap::value_parser!(u64)).help("Existing GitHub issue number to overwrite atomically (gh issue edit) and verify closed-loop"))
        .arg(Arg::new("target").value_name("BODY_FILE").action(ArgAction::Set).value_parser(clap::value_parser!(PathBuf)).help("Positional body file path"));

    let blueprint_cmd = Command::new("check-blueprint")
        .about("Verify contract-bounded engineering design blueprint (Checks 1-12) and compile in-repo SysML v2")
        .arg(Arg::new("path").value_name("PATH").required(true).action(ArgAction::Set).value_parser(clap::value_parser!(PathBuf)).help("Path to engineering design blueprint (docs/designs/blueprint-*.md)"))
        .arg(Arg::new("repo").long("repo").value_name("OWNER/REPO").action(ArgAction::Set).help("Target GitHub repository identifier (<owner>/<repo>)"))
        .arg(Arg::new("issue").long("issue").value_name("ID").action(ArgAction::Set).value_parser(clap::value_parser!(u64)).help("Existing GitHub issue number to update atomically and verify closed-loop"))
        .arg(Arg::new("title").long("title").value_name("TITLE").action(ArgAction::Set).help("Override issue title for GitHub SSOT publication"))
        .arg(Arg::new("publish").long("publish").action(ArgAction::SetTrue).help("Publish or update issue on remote GitHub SSOT tracker and run closed-loop verification"))
        .arg(Arg::new("dry_run").long("dry-run").action(ArgAction::SetTrue).help("Run Checks 1-11 (including in-repo SysML v2 compilation) without remote GitHub publishing"));

    (defect_cmd, blueprint_cmd)
}

/// Builds the `audit-code`, `setup-hooks`, and `check-secrets` subcommand definitions.
/// Realises: [REQ-0601-VERIFY-BASELINE-CLI-PARSE]
fn build_audit_and_hook_subcommands() -> (Command, Command, Command) {
    let audit_cmd = Command::new("audit-code")
        .about("Deterministic 5-pillar static AST code auditor and 7-section defect dossier synthesizer")
        .arg(Arg::new("path").long("path").value_name("PATH").required(true).action(ArgAction::Set).value_parser(clap::value_parser!(PathBuf)).help("Target file or directory path to audit (.rs and .dart sources)"))
        .arg(Arg::new("pillar").long("pillar").value_name("PILLAR").default_value("all").action(ArgAction::Set).value_parser(["memory-safety", "resource-lifecycle", "concurrency", "test-integrity", "semantic-traceability", "all"]).help("Restrict audit to a single correctness risk pillar or all"))
        .arg(Arg::new("issue").long("issue").value_name("ID").action(ArgAction::Set).value_parser(clap::value_parser!(u64)).help("Existing GitHub issue number to overwrite atomically and verify closed-loop"))
        .arg(Arg::new("repo").long("repo").value_name("OWNER/REPO").action(ArgAction::Set).help("Target GitHub repository identifier (<owner>/<repo>)"))
        .arg(Arg::new("publish").long("publish").action(ArgAction::SetTrue).help("Publish deduplicated dossiers to remote GitHub SSOT tracker with closed-loop verification"))
        .arg(Arg::new("dry_run").long("dry-run").action(ArgAction::SetTrue).help("Validate and persist local defect dossiers without remote GitHub tracker calls"));

    let hooks_cmd = Command::new("setup-hooks")
        .about("Setup cross-platform git hooks natively")
        .arg(Arg::new("destination").default_value(".").action(ArgAction::Set).help("Target repository root or destination directory (defaults to current directory)"));

    let secrets_cmd = Command::new("check-secrets")
        .about("Check post-build assets for leaked credentials and secrets")
        .arg(Arg::new("destination").default_value(".").action(ArgAction::Set).help("Target repository root or destination directory (defaults to current directory)"))
        .arg(Arg::new("dirs").long("dirs").value_delimiter(' ').action(ArgAction::Append).help("Optional explicit directories to scan (defaults to dist, build/web, build/macos)"));

    (audit_cmd, hooks_cmd, secrets_cmd)
}

/// Extracts a [`Subcommands`] variant from parsed `clap::ArgMatches`.
/// Realises: [REQ-0601-VERIFY-BASELINE-CLI-PARSE]
fn extract_subcommand(matches: &ArgMatches) -> Option<Subcommands> {
    match matches.subcommand() {
        Some(("check-commit-msg", sub)) => Some(Subcommands::CommitMsg(CommitMsgArgs {
            msg_file: sub.get_one::<PathBuf>("msg_file").cloned(),
            range: sub.get_one::<String>("range").cloned(),
            head: sub.get_flag("head"),
            check_text: sub.get_one::<String>("check_text").cloned(),
            target: sub.get_one::<String>("target").cloned(),
        })),
        Some(("check-output", sub)) => Some(Subcommands::OutputIntegrity(OutputIntegrityArgs {
            dir: sub.get_one::<PathBuf>("dir").cloned(),
            files: sub.get_many::<PathBuf>("files").map(|v| v.cloned().collect()),
            targets: sub.get_many::<PathBuf>("targets").map(|v| v.cloned().collect()).unwrap_or_default(),
        })),
        Some(("check-defect", sub)) => Some(Subcommands::Defect(DefectDossierArgs {
            body_file: sub.get_one::<PathBuf>("body_file").cloned(),
            title: sub.get_one::<String>("title").cloned(),
            repo: sub.get_one::<String>("repo").cloned(),
            label: sub.get_one::<String>("label").cloned(),
            provider: sub.get_one::<String>("provider").cloned().unwrap_or_else(|| "github".to_string()),
            dry_run: sub.get_flag("dry_run"),
            publish: sub.get_flag("publish"),
            issue: sub.get_one::<u64>("issue").copied(),
            target: sub.get_one::<PathBuf>("target").cloned(),
        })),
        Some(("check-blueprint", sub)) => Some(Subcommands::Blueprint(BlueprintCliOptions {
            path: sub.get_one::<PathBuf>("path").cloned().unwrap_or_default(),
            repo: sub.get_one::<String>("repo").cloned(),
            issue: sub.get_one::<u64>("issue").copied(),
            title: sub.get_one::<String>("title").cloned(),
            publish: sub.get_flag("publish"),
            dry_run: sub.get_flag("dry_run"),
        })),
        Some(("audit-code", sub)) => {
            let pillar_raw = sub.get_one::<String>("pillar").map_or("all", String::as_str);
            let pillar = AuditPillar::from_cli_token(pillar_raw).unwrap_or(None);
            Some(Subcommands::AuditCode(AuditCodeArgs {
                path: sub.get_one::<PathBuf>("path").cloned().unwrap_or_default(),
                pillar,
                issue: sub.get_one::<u64>("issue").copied(),
                repo: sub.get_one::<String>("repo").cloned(),
                publish: sub.get_flag("publish"),
                dry_run: sub.get_flag("dry_run"),
            }))
        }
        Some(("setup-hooks", sub)) => Some(Subcommands::SetupHooks(SetupHooksArgs {
            destination: sub.get_one::<String>("destination").cloned().unwrap_or_else(|| ".".to_string()),
        })),
        Some(("check-secrets", sub)) => Some(Subcommands::Secrets(CheckSecretsArgs {
            destination: sub.get_one::<String>("destination").cloned().unwrap_or_else(|| ".".to_string()),
            dirs: sub.get_many::<String>("dirs").map(|v| v.cloned().collect()).unwrap_or_default(),
        })),
        _ => None,
    }
}

impl Cli {
    /// Parses process command-line arguments into a validated [`Cli`] configuration.
    /// Realises: [REQ-0601-VERIFY-BASELINE-CLI-PARSE]
    pub(crate) fn parse() -> Self {
        let matches = build_root_command().get_matches();
        let command = extract_subcommand(&matches);
        Self {
            command,
            destination: matches
                .get_one::<String>("destination")
                .cloned()
                .unwrap_or_else(|| ".".to_string()),
            no_domain: matches.get_flag("no_domain"),
            strict: matches.get_flag("strict"),
            allow_missing_specs: matches.get_flag("allow_missing_specs"),
            target: matches.get_one::<String>("target").cloned(),
            output: matches.get_one::<String>("output").cloned(),
            spec_only: matches.get_flag("spec_only"),
            only: matches.get_one::<String>("only").cloned(),
            gate: matches.get_one::<String>("gate").cloned(),
        }
    }
}

/// Resolves the enclosing workspace root directory for `verify-baseline audit-code --path <PATH>`.
/// Realises: [REQ-0606-CLI-RESOLVE-WORKSPACE-ROOT]
pub(crate) fn resolve_audit_workspace_root(target_path: &Path) -> PathBuf {
    for anc in target_path.ancestors() {
        if anc.as_os_str().is_empty() {
            break;
        }
        if anc.join(".pipeline").is_dir() || anc.join(".git").exists() {
            return anc.to_path_buf();
        }
    }
    let mut outermost_cargo: Option<&Path> = None;
    for anc in target_path.ancestors() {
        if anc.as_os_str().is_empty() {
            break;
        }
        if anc.join("Cargo.toml").is_file() {
            outermost_cargo = Some(anc);
        }
    }
    if let Some(cargo_root) = outermost_cargo {
        return cargo_root.to_path_buf();
    }
    let fallback = if target_path.is_file() {
        target_path.parent().unwrap_or(target_path)
    } else {
        target_path
    };
    if fallback.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        fallback.to_path_buf()
    }
}
