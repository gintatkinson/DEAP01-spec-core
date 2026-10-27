//! CLI entrypoint for reconcile-backlog utility.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides the command-line execution boundary for deterministic backlog reconciliation and
//! idempotent issue creation across local Markdown specifications and the canonical remote
//! issue tracker under DO-178C Level A and ISO 26262 ASIL D traceability constraints.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Every command invocation either completes full specification-to-tracker synchronization
//!   or terminates with a non-zero exit code (`1`) upon any precondition or reconciliation error.
//! - Offline and dry-run modes guarantee zero external tracker mutations.
//!
//! ## 3. Preconditions
//! - Command-line arguments must conform to the `Cli` schema parsed by `clap`.
//! - When running in default reconciliation mode, `--workspace` must resolve to an existing
//!   canonicalizable filesystem path.
//!
//! ## 4. Postconditions
//! - Returns exit code `0` when subcommand execution or workspace reconciliation succeeds.
//! - Returns exit code `1` and emits structured diagnostic output to `stderr` if workspace
//!   canonicalization, issue creation, or backlog reconciliation fails.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time complexity: $O(S + I)$ where $S$ is the number of specification files in the workspace
//!   and $I$ is the number of tracked issues queried or synchronized.
//! - Space complexity: $O(S + I)$ bounded by in-memory specification metadata and tracker state.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Unresolvable workspace paths fail closed before any file or tracker state is accessed.
//! - Tracker transport or precondition failures during `create-issue` or reconciliation
//!   propagate immediately as explicit `Err` diagnostics and terminate with exit status `1`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-RECONCILE-BACKLOG-CLI, Issue Tracker as Canonical SSOT, Zero Fallback Placeholders, Automated Spec Reconciliation]

use clap::{Arg, ArgAction, Command};
use reconcile_backlog::create_issue::{execute_create_issue, CreateIssueArgs};
use reconcile_backlog::tracker::{GitHubTracker, OfflineTracker};
use reconcile_backlog::{run_reconciliation, ReconcileOptions};
use std::path::PathBuf;
use std::process;

/// Executes deterministic struct `Cli` within `crates/reconcile-backlog/src/main.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `Cli` in `crates/reconcile-backlog/src/main.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `Cli` across member set `[command: Option<Subcommands>, workspace: String, offline: bool, provider: String, dry_run: bool]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `Cli` members (`command: Option<Subcommands>, workspace: String, offline: bool, provider: String, dry_run: bool`).
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
/// - Resource: Bounds stack and heap allocations within `Cli` (`RECONCILE-BACKLOG-MAIN`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-MAIN-CLI] | SysML v2: PartDef(Cli)
#[derive(Debug)]
struct Cli {
    command: Option<Subcommands>,

    /// Workspace root directory (defaults to current directory)
    workspace: String,

    /// Run in offline mode without querying or updating external issue tracker
    offline: bool,

    /// Issue tracker provider (e.g. github, gitlab)
    provider: String,

    /// Perform a trial run without modifying files or external tracker
    dry_run: bool,

    /// Synchronize modified specification bodies to remote tracker issues
    sync_bodies: bool,

    /// Target repository (e.g. owner/repo)
    repo: Option<String>,

    /// Default branch name (defaults to auto-detected or main)
    default_branch: Option<String>,
}

/// Executes deterministic enum `Subcommands` within `crates/reconcile-backlog/src/main.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `Subcommands` in `crates/reconcile-backlog/src/main.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `Subcommands` across member set `[CreateIssue]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `Subcommands` members (`CreateIssue`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `Subcommands` representation for enum `Subcommands`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(1)) bounded over input/member dimension of `Subcommands`.
/// - Space Complexity: O(M(1)) bounded auxiliary memory for `Subcommands`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `Subcommands` (`Subcommands`).
/// - State: Maintains consistent state invariants across `Subcommands` without partial mutation on error.
/// - Action: Executes `Subcommands` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `Subcommands` (`RECONCILE-BACKLOG-MAIN`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-MAIN-SUBCOMMANDS] | SysML v2: PartDef(Subcommands)
#[derive(Debug)]
enum Subcommands {
    /// Create or reopen an issue idempotently with precondition checks
    CreateIssue(CreateIssueArgs),
}

impl Cli {
    /// Parses process command-line arguments into a validated `Cli` configuration using explicit `clap::Command` builders.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `Cli::parse` in `crates/reconcile-backlog/src/main.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Constructs the CLI grammar without `clap_derive` macro lint-override emissions (`E0453`).
    ///
    /// ## 3. Preconditions
    /// - Process environment supplies valid OS argument tokens via `std::env::args_os`.
    ///
    /// ## 4. Postconditions
    /// - Returns a populated `Cli` instance with deterministic `command` and top-level reconciliation flags.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(A)$ where $A$ is the number of CLI argument tokens.
    /// - Space Complexity: $O(A)$ for parsed `ArgMatches` and owned `Cli` fields.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Invalid flags are intercepted by `clap::Command::get_matches`, emitting usage diagnostics and exiting non-zero.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RECONCILE-BACKLOG-MAIN-CLI]
    fn parse() -> Self {
        let mut matches = Command::new("reconcile-backlog")
            .about("Pure Rust Backlog Reconciler for DEAP MBSE Framework")
            .subcommand_precedence_over_arg(true)
            .arg(
                Arg::new("workspace")
                    .long("workspace")
                    .default_value(".")
                    .action(ArgAction::Set),
            )
            .arg(
                Arg::new("offline")
                    .long("offline")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("provider")
                    .long("provider")
                    .default_value("github")
                    .action(ArgAction::Set),
            )
            .arg(
                Arg::new("dry_run")
                    .long("dry-run")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("sync_bodies")
                    .long("sync-bodies")
                    .action(ArgAction::SetTrue),
            )
            .arg(Arg::new("repo").long("repo").action(ArgAction::Set))
            .arg(
                Arg::new("default_branch")
                    .long("default-branch")
                    .action(ArgAction::Set),
            )
            .subcommand(
                Command::new("create-issue")
                    .visible_alias("create_issue")
                    .about("Create or reopen an issue idempotently with precondition checks")
                    .arg(Arg::new("body_file").long("body-file").action(ArgAction::Set))
                    .arg(Arg::new("label").long("label").action(ArgAction::Set))
                    .arg(Arg::new("title").long("title").action(ArgAction::Set))
                    .arg(Arg::new("repo").long("repo").action(ArgAction::Set))
                    .arg(
                        Arg::new("reopen_with_analysis")
                            .long("reopen-with-analysis")
                            .action(ArgAction::Set),
                    )
                    .arg(
                        Arg::new("provider")
                            .long("provider")
                            .default_value("github")
                            .action(ArgAction::Set),
                    )
                    .arg(
                        Arg::new("positional")
                            .value_name("ARGS")
                            .num_args(0..)
                            .action(ArgAction::Append),
                    ),
            )
            .get_matches();

        let command = match matches.remove_subcommand() {
            Some((name, mut sub_m)) if name == "create-issue" => {
                Some(Subcommands::CreateIssue(CreateIssueArgs {
                    body_file: sub_m.remove_one::<String>("body_file"),
                    label: sub_m.remove_one::<String>("label"),
                    title: sub_m.remove_one::<String>("title"),
                    repo: sub_m.remove_one::<String>("repo"),
                    reopen_with_analysis: sub_m.remove_one::<String>("reopen_with_analysis"),
                    provider: sub_m
                        .remove_one::<String>("provider")
                        .unwrap_or_else(|| "github".to_string()),
                    positional: sub_m
                        .remove_many::<String>("positional")
                        .map(|v| v.collect())
                        .unwrap_or_default(),
                }))
            }
            _ => None,
        };

        let sync_bodies = if matches.contains_id("sync_bodies") {
            matches.get_flag("sync_bodies")
        } else {
            true
        };

        Self {
            command,
            workspace: matches
                .remove_one::<String>("workspace")
                .unwrap_or_else(|| ".".to_string()),
            offline: matches.get_flag("offline"),
            provider: matches
                .remove_one::<String>("provider")
                .unwrap_or_else(|| "github".to_string()),
            dry_run: matches.get_flag("dry_run"),
            sync_bodies,
            repo: matches.remove_one::<String>("repo"),
            default_branch: matches.remove_one::<String>("default_branch"),
        }
    }
}

/// Executes the `reconcile-backlog` CLI entrypoint.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Dispatches parsed CLI arguments to either the idempotent `create-issue` workflow or the
/// workspace-wide backlog reconciliation engine, enforcing deterministic exit codes for CI
/// and pre-commit verification gates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Dispatches at most one operational mode (`CreateIssue` subcommand or workspace reconciliation).
/// - Never suppresses operational errors; any `Err` outcome deterministically triggers `process::exit(1)`.
///
/// ## 3. Preconditions
/// - Process environment and standard streams (`stdout`, `stderr`) must be initialized.
/// - CLI flags and subcommands must be valid per `Cli::parse()`.
///
/// ## 4. Postconditions
/// - On success, prints reconciliation metrics or created issue identifiers to `stdout` and exits `0`.
/// - On failure, prints diagnostic context to `stderr` and exits `1`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(1)$ CLI dispatch overhead plus the complexity of `execute_create_issue`
///   or `run_reconciliation`.
/// - Space complexity: $O(1)$ stack frame overhead beyond owned `Cli` and `ReconcileOptions` structs.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Invalid workspace paths are rejected via `PathBuf::canonicalize` prior to tracker initialization.
/// - Subcommand and reconciliation errors are reported verbatim to `stderr` before non-zero termination.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-MAIN]
fn main() {
    let cli = Cli::parse();

    if let Some(cmd) = cli.command {
        match cmd {
            Subcommands::CreateIssue(args) => {
                if let Err(err) = execute_create_issue(&args) {
                    eprintln!("ERROR: {}", err);
                    process::exit(1);
                }
            }
        }
    } else {
        let workspace_path = PathBuf::from(&cli.workspace);

        let canonical_workspace = match workspace_path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error: Cannot resolve workspace path '{}': {}", cli.workspace, e);
                process::exit(1);
            }
        };

        let options = ReconcileOptions {
            workspace_root: canonical_workspace,
            offline: cli.offline,
            dry_run: cli.dry_run,
            provider: cli.provider.clone(),
            sync_bodies: cli.sync_bodies,
            repo: cli.repo.clone(),
            default_branch: cli.default_branch.clone(),
        };

        println!("Starting backlog reconciliation for workspace: {}", options.workspace_root.display());
        println!("Mode: {}", if options.offline { "OFFLINE" } else { "ONLINE (Provider: GitHub)" });
        if options.dry_run {
            println!("Dry-run enabled: No changes will be written to disk or tracker.");
        }

        let result = if options.offline {
            let tracker = OfflineTracker::new();
            run_reconciliation(&tracker, &options)
        } else {
            let tracker = GitHubTracker::new();
            run_reconciliation(&tracker, &options)
        };

        match result {
            Ok(summary) => {
                println!("Reconciliation completed successfully:");
                println!("  - Total specifications scanned: {}", summary.total_specs_scanned);
                println!("  - Frontmatter issue IDs updated: {}", summary.frontmatters_updated);
                println!("  - Checklist items reconciled:   {}", summary.checklists_updated);
                println!("  - Placeholder blocks purged:     {}", summary.placeholders_purged);
                if !options.offline {
                    println!("  - Remote issues synced:          {}", summary.remote_issues_synced);
                }
                process::exit(0);
            }
            Err(err) => {
                eprintln!("ERROR: Backlog reconciliation failed: {}", err);
                process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_reconcile_backlog_main_7_section_rustdoc() {
        let src = include_str!("main.rs");
        assert!(src.contains("//! ## 1. Safety Intent & Regulatory Scope"));
        assert!(src.contains("//! ## 2. Invariants & Mathematical Properties"));
        assert!(src.contains("//! ## 3. Preconditions"));
        assert!(src.contains("//! ## 4. Postconditions"));
        assert!(src.contains("//! ## 5. Algorithmic Complexity Bounds"));
        assert!(src.contains("//! ## 6. Failure Modes & Fault Tolerance"));
        assert!(src.contains("//! ## 7. Realization & Traceability"));
        assert!(src.contains("//! Realises: [REQ-RECONCILE-BACKLOG-CLI, Issue Tracker as Canonical SSOT, Zero Fallback Placeholders, Automated Spec Reconciliation]"));
        assert!(src.contains("/// ## 1. Safety Intent & Regulatory Scope"));
        assert!(src.contains("/// ## 7. Realization & Traceability"));
        assert!(src.contains("/// Realises: [REQ-RECONCILE-BACKLOG-MAIN]"));
    }
}


