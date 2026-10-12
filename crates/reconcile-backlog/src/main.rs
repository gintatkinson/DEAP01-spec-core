//! CLI entrypoint for reconcile-backlog utility.
//!
//! Realises: [Issue Tracker as Canonical SSOT, Zero Fallback Placeholders, Automated Spec Reconciliation]

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
}

#[derive(clap::Subcommand, Debug)]
enum Subcommands {
    /// Create or reopen an issue idempotently with precondition checks
    #[command(name = "create-issue", alias = "create_issue")]
    CreateIssue(CreateIssueArgs),
}

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

