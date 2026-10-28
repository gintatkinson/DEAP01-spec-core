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

use clap::{Arg, ArgAction, Command};
use std::path::{Path, PathBuf};
use std::process;
use verify_baseline::audit_code::{self, AuditCodeArgs, AuditPillar};
use verify_baseline::blueprint_gate::{self, BlueprintCliOptions};
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
#[derive(Debug)]
struct Cli {
    command: Option<Subcommands>,

    /// Target destination directory or repository root (defaults to current directory)
    destination: String,

    /// Skip domain type compatibility validation and build/test commands
    no_domain: bool,

    /// Enforce strict baseline conformance (reject pending or missing specifications)
    strict: bool,

    /// Gracefully allow missing specifications if in downstream customer mode
    allow_missing_specs: bool,

    /// Target execution platform profile (e.g. flutter, react)
    target: Option<String>,

    /// Optional path to write output report or defect dossier
    output: Option<String>,

    /// Run only the specification quality and coverage audit suite
    spec_only: bool,

    /// Audit only a specific markdown file
    only: Option<String>,

    /// Filter to run a specific audit gate
    gate: Option<String>,
}

impl Cli {
    /// Parses process command-line arguments into a validated `Cli` configuration using explicit `clap::Command` builders.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D command-line argument parsing
    /// governance. Constructs the `verify-baseline` CLI grammar without `clap_derive` proc-macro
    /// lint-suppression injections so crate-level `forbid` lint policies remain uncompromised.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves 100% flag, alias, positional argument, and subcommand parity across `check-commit-msg`,
    ///   `check-output`, `check-defect`, `setup-hooks`, and `check-secrets`.
    /// - Enforces `subcommand_precedence_over_arg(true)` so subcommand tokens take priority over `destination`.
    ///
    /// ## 3. Preconditions
    /// - Process environment supplies valid OS argument tokens via `std::env::args_os`.
    ///
    /// ## 4. Postconditions
    /// - Returns a populated `Cli` instance with deterministic `command` and top-level baseline flags.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(A)$ where $A$ is the number of CLI argument tokens.
    /// - Space Complexity: $O(A)$ for parsed `ArgMatches` and owned `Cli` fields.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Invalid flags or missing required values are intercepted by `clap::Command::get_matches`,
    ///   emitting usage diagnostics and exiting with a non-zero status code.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0601-VERIFY-BASELINE-CLI-PARSE]
    fn parse() -> Self {
        let matches = Command::new("verify-baseline")
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
            .arg(
                Arg::new("target")
                    .long("target")
                    .action(ArgAction::Set)
                    .help("Target execution platform profile (e.g. flutter, react)"),
            )
            .arg(
                Arg::new("output")
                    .long("output")
                    .action(ArgAction::Set)
                    .help("Optional path to write output report or defect dossier"),
            )
            .arg(
                Arg::new("spec_only")
                    .long("spec-only")
                    .action(ArgAction::SetTrue)
                    .help("Run only the specification quality and coverage audit suite"),
            )
            .arg(
                Arg::new("only")
                    .long("only")
                    .action(ArgAction::Set)
                    .help("Audit only a specific markdown file"),
            )
            .arg(
                Arg::new("gate")
                    .long("gate")
                    .action(ArgAction::Set)
                    .help("Filter to run a specific audit gate"),
            )
            .subcommand(
                Command::new("check-commit-msg")
                    .visible_alias("check-commit")
                    .about("Port of verify_commit_messages: check commit message non-closure invariant")
                    .arg(
                        Arg::new("msg_file")
                            .long("msg-file")
                            .action(ArgAction::Set)
                            .value_parser(clap::value_parser!(PathBuf))
                            .help("Path to file containing commit message"),
                    )
                    .arg(
                        Arg::new("range")
                            .long("range")
                            .action(ArgAction::Set)
                            .help("Git revision range to inspect (e.g. origin/main..HEAD)"),
                    )
                    .arg(
                        Arg::new("head")
                            .long("head")
                            .action(ArgAction::SetTrue)
                            .help("Check the latest HEAD commit"),
                    )
                    .arg(
                        Arg::new("check_text")
                            .long("check-text")
                            .action(ArgAction::Set)
                            .help("Check raw text directly"),
                    )
                    .arg(
                        Arg::new("target")
                            .value_name("TARGET")
                            .action(ArgAction::Set)
                            .help("Positional file or text to check"),
                    ),
            )
            .subcommand(
                Command::new("check-output")
                    .visible_alias("verify-subagent-output")
                    .visible_alias("check-output-integrity")
                    .about("Port of verify_subagent_output: check markdown size, escape tokens, code fences")
                    .arg(
                        Arg::new("dir")
                            .long("dir")
                            .action(ArgAction::Set)
                            .value_parser(clap::value_parser!(PathBuf))
                            .help("Target directory to scan for markdown files"),
                    )
                    .arg(
                        Arg::new("files")
                            .long("files")
                            .num_args(1..)
                            .action(ArgAction::Append)
                            .value_parser(clap::value_parser!(PathBuf))
                            .help("Explicit list of files to verify"),
                    )
                    .arg(
                        Arg::new("targets")
                            .value_name("TARGETS")
                            .num_args(0..)
                            .action(ArgAction::Append)
                            .value_parser(clap::value_parser!(PathBuf))
                            .help("Positional targets (directory or file paths)"),
                    ),
            )
            .subcommand(
                Command::new("check-defect")
                    .visible_alias("verify-defect-dossier")
                    .about("Port of file_defect validator: check 7-section defect dossier schema")
                    .arg(
                        Arg::new("body_file")
                            .long("body-file")
                            .action(ArgAction::Set)
                            .value_parser(clap::value_parser!(PathBuf))
                            .help("Path to markdown defect dossier body file"),
                    )
                    .arg(
                        Arg::new("title")
                            .long("title")
                            .action(ArgAction::Set)
                            .help("Title of the defect issue ([AUDIT] [file.ext]: <description>)"),
                    )
                    .arg(
                        Arg::new("repo")
                            .long("repo")
                            .action(ArgAction::Set)
                            .help("Repository identifier (<owner>/<repo>); auto-resolved from git remote get-url origin when omitted"),
                    )
                    .arg(
                        Arg::new("label")
                            .long("label")
                            .action(ArgAction::Set)
                            .help("Override label to apply"),
                    )
                    .arg(
                        Arg::new("provider")
                            .long("provider")
                            .default_value("github")
                            .action(ArgAction::Set)
                            .help("Issue tracker provider (github or gitlab)"),
                    )
                    .arg(
                        Arg::new("dry_run")
                            .long("dry-run")
                            .visible_alias("validate-only")
                            .action(ArgAction::SetTrue)
                            .help("Validate defect dossier schema without submitting to remote tracker"),
                    )
                    .arg(
                        Arg::new("publish")
                            .long("publish")
                            .action(ArgAction::SetTrue)
                            .help("Publish a new issue to the remote GitHub SSOT tracker and run closed-loop verification"),
                    )
                    .arg(
                        Arg::new("issue")
                            .long("issue")
                            .action(ArgAction::Set)
                            .value_parser(clap::value_parser!(u64))
                            .help("Existing GitHub issue number to overwrite atomically (gh issue edit) and verify closed-loop"),
                    )
                    .arg(
                        Arg::new("target")
                            .value_name("BODY_FILE")
                            .action(ArgAction::Set)
                            .value_parser(clap::value_parser!(PathBuf))
                            .help("Positional body file path"),
                    ),
            )
            .subcommand(
                Command::new("check-blueprint")
                    .about("Verify contract-bounded engineering design blueprint (Checks 1-12) and compile in-repo SysML v2")
                    .arg(
                        Arg::new("path")
                            .value_name("PATH")
                            .required(true)
                            .action(ArgAction::Set)
                            .value_parser(clap::value_parser!(PathBuf))
                            .help("Path to engineering design blueprint (docs/designs/blueprint-*.md)"),
                    )
                    .arg(
                        Arg::new("repo")
                            .long("repo")
                            .value_name("OWNER/REPO")
                            .action(ArgAction::Set)
                            .help("Target GitHub repository identifier (<owner>/<repo>)"),
                    )
                    .arg(
                        Arg::new("issue")
                            .long("issue")
                            .value_name("ID")
                            .action(ArgAction::Set)
                            .value_parser(clap::value_parser!(u64))
                            .help("Existing GitHub issue number to update atomically and verify closed-loop"),
                    )
                    .arg(
                        Arg::new("title")
                            .long("title")
                            .value_name("TITLE")
                            .action(ArgAction::Set)
                            .help("Override issue title for GitHub SSOT publication"),
                    )
                    .arg(
                        Arg::new("publish")
                            .long("publish")
                            .action(ArgAction::SetTrue)
                            .help("Publish or update issue on remote GitHub SSOT tracker and run closed-loop verification"),
                    )
                    .arg(
                        Arg::new("dry_run")
                            .long("dry-run")
                            .action(ArgAction::SetTrue)
                            .help("Run Checks 1-11 (including in-repo SysML v2 compilation) without remote GitHub publishing"),
                    ),
            )
            .subcommand(
                Command::new("audit-code")
                    .about("Deterministic 5-pillar static AST code auditor and 7-section defect dossier synthesizer")
                    .arg(
                        Arg::new("path")
                            .long("path")
                            .value_name("PATH")
                            .required(true)
                            .action(ArgAction::Set)
                            .value_parser(clap::value_parser!(PathBuf))
                            .help("Target file or directory path to audit (.rs and .dart sources)"),
                    )
                    .arg(
                        Arg::new("pillar")
                            .long("pillar")
                            .value_name("PILLAR")
                            .default_value("all")
                            .action(ArgAction::Set)
                            .value_parser([
                                "memory-safety",
                                "resource-lifecycle",
                                "concurrency",
                                "test-integrity",
                                "semantic-traceability",
                                "all",
                            ])
                            .help("Restrict audit to a single correctness risk pillar or all"),
                    )
                    .arg(
                        Arg::new("issue")
                            .long("issue")
                            .value_name("ID")
                            .action(ArgAction::Set)
                            .value_parser(clap::value_parser!(u64))
                            .help("Existing GitHub issue number to overwrite atomically and verify closed-loop"),
                    )
                    .arg(
                        Arg::new("repo")
                            .long("repo")
                            .value_name("OWNER/REPO")
                            .action(ArgAction::Set)
                            .help("Target GitHub repository identifier (<owner>/<repo>)"),
                    )
                    .arg(
                        Arg::new("publish")
                            .long("publish")
                            .action(ArgAction::SetTrue)
                            .help("Publish deduplicated dossiers to remote GitHub SSOT tracker with closed-loop verification"),
                    )
                    .arg(
                        Arg::new("dry_run")
                            .long("dry-run")
                            .action(ArgAction::SetTrue)
                            .help("Validate and persist local defect dossiers without remote GitHub tracker calls"),
                    ),
            )
            .subcommand(
                Command::new("setup-hooks")
                    .about("Setup cross-platform git hooks natively")
                    .arg(
                        Arg::new("destination")
                            .default_value(".")
                            .action(ArgAction::Set)
                            .help("Target repository root or destination directory (defaults to current directory)"),
                    ),
            )
            .subcommand(
                Command::new("check-secrets")
                    .about("Check post-build assets for leaked credentials and secrets")
                    .arg(
                        Arg::new("destination")
                            .default_value(".")
                            .action(ArgAction::Set)
                            .help("Target repository root or destination directory (defaults to current directory)"),
                    )
                    .arg(
                        Arg::new("dirs")
                            .long("dirs")
                            .value_delimiter(' ')
                            .action(ArgAction::Append)
                            .help("Optional explicit directories to scan (defaults to dist, build/web, build/macos)"),
                    ),
            )
            .get_matches();

        let command = match matches.subcommand() {
            Some(("check-commit-msg", sub)) => Some(Subcommands::CommitMsg(CommitMsgArgs {
                msg_file: sub.get_one::<PathBuf>("msg_file").cloned(),
                range: sub.get_one::<String>("range").cloned(),
                head: sub.get_flag("head"),
                check_text: sub.get_one::<String>("check_text").cloned(),
                target: sub.get_one::<String>("target").cloned(),
            })),
            Some(("check-output", sub)) => {
                let files = sub
                    .get_many::<PathBuf>("files")
                    .map(|vals| vals.cloned().collect());
                let targets = sub
                    .get_many::<PathBuf>("targets")
                    .map(|vals| vals.cloned().collect())
                    .unwrap_or_default();
                Some(Subcommands::OutputIntegrity(OutputIntegrityArgs {
                    dir: sub.get_one::<PathBuf>("dir").cloned(),
                    files,
                    targets,
                }))
            }
            Some(("check-defect", sub)) => Some(Subcommands::Defect(DefectDossierArgs {
                body_file: sub.get_one::<PathBuf>("body_file").cloned(),
                title: sub.get_one::<String>("title").cloned(),
                repo: sub.get_one::<String>("repo").cloned(),
                label: sub.get_one::<String>("label").cloned(),
                provider: sub
                    .get_one::<String>("provider")
                    .cloned()
                    .unwrap_or_else(|| "github".to_string()),
                dry_run: sub.get_flag("dry_run"),
                publish: sub.get_flag("publish"),
                issue: sub.get_one::<u64>("issue").copied(),
                target: sub.get_one::<PathBuf>("target").cloned(),
            })),
            Some(("check-blueprint", sub)) => Some(Subcommands::Blueprint(BlueprintCliOptions {
                path: sub
                    .get_one::<PathBuf>("path")
                    .cloned()
                    .unwrap_or_default(),
                repo: sub.get_one::<String>("repo").cloned(),
                issue: sub.get_one::<u64>("issue").copied(),
                title: sub.get_one::<String>("title").cloned(),
                publish: sub.get_flag("publish"),
                dry_run: sub.get_flag("dry_run"),
            })),
            Some(("audit-code", sub)) => {
                let pillar_raw = sub
                    .get_one::<String>("pillar")
                    .map(String::as_str)
                    .unwrap_or("all");
                let pillar = AuditPillar::from_cli_token(pillar_raw).unwrap_or(None);
                Some(Subcommands::AuditCode(AuditCodeArgs {
                    path: sub
                        .get_one::<PathBuf>("path")
                        .cloned()
                        .unwrap_or_default(),
                    pillar,
                    issue: sub.get_one::<u64>("issue").copied(),
                    repo: sub.get_one::<String>("repo").cloned(),
                    publish: sub.get_flag("publish"),
                    dry_run: sub.get_flag("dry_run"),
                }))
            }
            Some(("setup-hooks", sub)) => Some(Subcommands::SetupHooks(SetupHooksArgs {
                destination: sub
                    .get_one::<String>("destination")
                    .cloned()
                    .unwrap_or_else(|| ".".to_string()),
            })),
            Some(("check-secrets", sub)) => {
                let dirs = sub
                    .get_many::<String>("dirs")
                    .map(|vals| vals.cloned().collect())
                    .unwrap_or_default();
                Some(Subcommands::Secrets(CheckSecretsArgs {
                    destination: sub
                        .get_one::<String>("destination")
                        .cloned()
                        .unwrap_or_else(|| ".".to_string()),
                    dirs,
                }))
            }
            _ => None,
        };

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

/// Executes deterministic enum `Subcommands` within `crates/verify-baseline/src/main.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `Subcommands` in `crates/verify-baseline/src/main.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `Subcommands` across member set `[CommitMsg, OutputIntegrity, Defect, SetupHooks, Secrets]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `Subcommands` members (`CommitMsg, OutputIntegrity, Defect, SetupHooks, Secrets`).
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
#[derive(Debug)]
enum Subcommands {
    /// Port of verify_commit_messages: check commit message non-closure invariant
    CommitMsg(CommitMsgArgs),

    /// Port of verify_subagent_output: check markdown size, escape tokens, code fences
    OutputIntegrity(OutputIntegrityArgs),

    /// Port of file_defect validator: check 7-section defect dossier schema
    Defect(DefectDossierArgs),

    /// Verify contract-bounded engineering design blueprint (Checks 1-12)
    Blueprint(BlueprintCliOptions),

    /// Deterministic 5-pillar static AST code auditor and defect dossier synthesizer
    AuditCode(AuditCodeArgs),

    /// Setup cross-platform git hooks natively
    SetupHooks(SetupHooksArgs),

    /// Check post-build assets for leaked credentials and secrets
    Secrets(CheckSecretsArgs),
}

/// Resolves the enclosing workspace root directory for `verify-baseline audit-code --path <PATH>`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Ensures `.pipeline/defects/` persistence and relative `FILE_LOCATION` paths anchor deterministically
/// to the enclosing workspace root under DO-330 TQL-4 path resolution rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Prefers the nearest ancestor containing `.pipeline` or `.git`, then outermost `Cargo.toml`, else `target_path`.
///
/// ## 3. Preconditions
/// - `target_path` is a CLI-supplied filesystem path.
///
/// ## 4. Postconditions
/// - Returns a deterministic `PathBuf` representing the workspace root directory.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(D)$ over directory ancestor depth $D$.
/// - Space Complexity: $O(|target\_path|)$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Gracefully falls back to `target_path` or `"."` without panicking when `target_path` does not exist.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0606-CLI-RESOLVE-WORKSPACE-ROOT]
fn resolve_audit_workspace_root(target_path: &Path) -> PathBuf {
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
                let workspace_root = PathBuf::from(&cli.destination);
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
        let prod_src = src.split("#[cfg(test)]").next().unwrap_or("");
        assert!(!prod_src.contains("process::exit(0)"));
    }
}
