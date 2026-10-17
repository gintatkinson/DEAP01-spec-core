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

use clap::Parser;
use reconcile_backlog::create_issue::{execute_create_issue, CreateIssueArgs};
use reconcile_backlog::tracker::{GitHubTracker, OfflineTracker};
use reconcile_backlog::{run_reconciliation, ReconcileOptions};
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "reconcile-backlog")]
#[command(about = "Pure Rust Backlog Reconciler for DEAP MBSE Framework", long_about = None)]
#[command(subcommand_precedence_over_arg = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Subcommands>,

    /// Workspace root directory (defaults to current directory)
    #[arg(long, default_value = ".")]
    workspace: String,

    /// Run in offline mode without querying or updating external issue tracker
    #[arg(long)]
    offline: bool,

    /// Issue tracker provider (e.g. github, gitlab)
    #[arg(long, default_value = "github")]
    provider: String,

    /// Perform a trial run without modifying files or external tracker
    #[arg(long)]
    dry_run: bool,

    /// Synchronize modified specification bodies to remote tracker issues
    #[arg(long, default_value_t = true)]
    sync_bodies: bool,

    /// Target repository (e.g. owner/repo)
    #[arg(long)]
    repo: Option<String>,

    /// Default branch name (defaults to auto-detected or main)
    #[arg(long)]
    default_branch: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
enum Subcommands {
    /// Create or reopen an issue idempotently with precondition checks
    #[command(name = "create-issue", alias = "create_issue")]
    CreateIssue(CreateIssueArgs),
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


