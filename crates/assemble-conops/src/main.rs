//! CLI binary for `assemble-conops` -- Deterministic ConOps & Mission Intent Assembly Engine.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides the command-line entry point for deterministic assembly and verification of
//! Concept of Operations (`CONOPS.md`) and Tactical Mission Intent (`MISSION_INTENT.md`)
//! specification artifacts under DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D,
//! ISO/IEC/IEEE 29148:2018, and ISO/IEC/IEEE 15288:2023.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Deterministic Workspace Resolution: Candidate input directory discovery evaluates a fixed,
//!   ordered sequence of workspace-relative candidate paths (`docs/conops/units`, `docs/conops`,
//!   `units`, and workspace root) and selects the first structural match.
//! - Exit Code Soundness: The process returns `ExitCode::SUCCESS` if and only if
//!   `assemble_conops` returns `Ok(true)`, and returns `ExitCode::FAILURE` on `Ok(false)` or `Err`.
//!
//! ## 3. Preconditions
//! - Invoked with valid CLI arguments parsed by `clap::Parser`.
//! - Target workspace or explicit `--input-dir` / `--output-dir` paths reside on an accessible
//!   local filesystem.
//!
//! ## 4. Postconditions
//! - When `--verify` is false and assembly succeeds, assembled `CONOPS.md` and `MISSION_INTENT.md`
//!   files are persisted in the resolved output directory.
//! - Process exits with `ExitCode::SUCCESS` (`0`) on clean assembly/verification or
//!   `ExitCode::FAILURE` (`1`) on validation failure or I/O error.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(D)$ for candidate input directory probing where $D$ is the number of
//!   immediate directory entries in each candidate root, followed by downstream assembly complexity.
//! - Space Complexity: $O(P)$ auxiliary heap allocation where $P$ is the byte length of resolved
//!   `PathBuf` arguments.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Missing or unreadable candidate directories during auto-discovery safely evaluate to `false`
//!   and fall back to `<workspace>/docs/conops/units`.
//! - Any downstream assembly error is emitted to `stderr` via diagnostic formatting and mapped
//!   deterministically to `ExitCode::FAILURE` without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-ASSEMBLE-CONOPS-CLI], [DO-330-TQL-4-CONOPS-CLI]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;

use assemble_conops::assemble_conops;

#[derive(Parser, Debug)]
#[command(
    name = "assemble-conops",
    author = "Gint Atkinson",
    version = "0.1.0",
    about = "Deterministic ConOps & Mission Intent Assembly Engine (ISO/IEC/IEEE 29148 / ISO/IEC/IEEE 15288 / OMG UAF)."
)]
struct Args {
    /// Target workspace or project directory (optional positional argument).
    #[arg(value_name = "WORKSPACE")]
    workspace: Option<PathBuf>,

    /// Target workspace or project directory.
    #[arg(long = "workspace")]
    workspace_flag: Option<PathBuf>,

    /// Input directory containing 'conops/' and 'mission_intent/' unit markdown directories (default: docs/conops/units).
    #[arg(long = "input-dir")]
    input_dir: Option<PathBuf>,

    /// Output directory where assembled CONOPS.md and MISSION_INTENT.md are written (default: docs/conops).
    #[arg(long = "output-dir")]
    output_dir: Option<PathBuf>,

    /// Verify unit integrity and link resolution without writing output files.
    #[arg(long = "verify", action = clap::ArgAction::SetTrue)]
    verify: bool,

    /// Path to JSON parameter dictionary or schema file (auto-detects .pipeline/schema-digest.json if not specified).
    #[arg(long = "params")]
    params: Option<PathBuf>,

    /// Target operational domain. Auto-detected if not specified.
    #[arg(long = "domain")]
    domain: Option<String>,
}

/// Inspects whether a candidate filesystem path is a valid ConOps unit root directory.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Prevents silent misrouting of ConOps assembly inputs by verifying that a candidate path
/// either directly contains `conops/` or `mission_intent/` subdirectories or houses a domain-scoped
/// child directory containing `conops/` under DO-330 TQL-4 tool qualification controls.
///
/// ## 2. Invariants & Mathematical Properties
/// - Pure Structural Predicate: Returns `true` if and only if `cand` is an existing directory
///   satisfying `is_dir(cand/conops) || is_dir(cand/mission_intent) || exists d in read_dir(cand): is_dir(d/conops)`.
/// - Side-Effect Free: Performs read-only filesystem metadata inspection without mutating state.
///
/// ## 3. Preconditions
/// - `cand` references a valid, null-free OS path slice (`&Path`).
///
/// ## 4. Postconditions
/// - Returns `true` when `cand` is a structurally valid ConOps input root; returns `false`
///   otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(E)$ where $E$ is the number of immediate directory entries in `cand`.
/// - Space Complexity: $O(1)$ auxiliary memory beyond a single `PathBuf` per directory entry.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-existent paths, permission denials on `fs::read_dir`, or corrupted directory entries
///   are gracefully filtered and yield `false` without panicking.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ASSEMBLE-CONOPS-CLI-DISCOVERY], [DO-330-TQL-4-CONOPS-CLI]
fn check_candidate_input_dir(cand: &Path) -> bool {
    if !cand.is_dir() {
        return false;
    }
    if cand.join("conops").is_dir() || cand.join("mission_intent").is_dir() {
        return true;
    }
    if let Ok(entries) = fs::read_dir(cand) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() && path.join("conops").is_dir() {
                return true;
            }
        }
    }
    false
}

/// Executes the `assemble-conops` command-line interface and maps outcomes to process exit codes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Serves as the deterministic CLI execution harness for `assemble_conops`, ensuring that
/// workspace path resolution, input/output directory binding, and verification gates return
/// unambiguous POSIX exit statuses suitable for CI/CD and DO-330 TQL-4 baseline gates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Explicit Flag Precedence: `--workspace` takes precedence over positional `WORKSPACE`, and
///   explicit `--input-dir` / `--output-dir` flags bypass heuristic candidate probing.
/// - Total Exit Mapping: Every execution path terminates in either `ExitCode::SUCCESS` or
///   `ExitCode::FAILURE` with zero unhandled exceptions.
///
/// ## 3. Preconditions
/// - Process environment supplies valid command-line arguments compatible with `Args`.
///
/// ## 4. Postconditions
/// - Returns `ExitCode::SUCCESS` if and only if `assemble_conops` completes with `Ok(true)`.
/// - Returns `ExitCode::FAILURE` if `assemble_conops` returns `Ok(false)` or `Err(err)`, printing
///   diagnostic details to `stderr` on error.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(D + N \log N)$ where $D$ is candidate directory probing cost and
///   $N \log N$ is the downstream unit sorting and assembly cost.
/// - Space Complexity: $O(M)$ where $M$ is the aggregate byte size of loaded ConOps and Mission
///   Intent units.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - If canonicalization of the workspace path fails, falls back deterministically to the
///   un-canonicalized path.
/// - If current working directory retrieval fails, defaults `ws_ref` to `.`.
/// - All library-level errors are caught, logged to `stderr`, and converted to `ExitCode::FAILURE`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ASSEMBLE-CONOPS-CLI], [DO-330-TQL-4-CONOPS-CLI]
fn main() -> ExitCode {
    let args = Args::parse();

    let target_ws = args.workspace_flag.or(args.workspace);
    let explicit_io = args.input_dir.is_some() || args.output_dir.is_some();

    let workspace: Option<PathBuf> = if let Some(ws) = target_ws {
        Some(ws.canonicalize().unwrap_or(ws))
    } else if !explicit_io {
        std::env::current_dir().ok()
    } else {
        None
    };

    let ws_ref = workspace
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));

    let input_dir = if let Some(in_dir) = args.input_dir {
        in_dir
    } else {
        let mut resolved = None;
        for cand in &[
            ws_ref.join("docs").join("conops").join("units"),
            ws_ref.join("docs").join("conops"),
            ws_ref.join("units"),
            ws_ref.clone(),
        ] {
            if check_candidate_input_dir(cand) {
                resolved = Some(cand.clone());
                break;
            }
        }
        resolved.unwrap_or_else(|| ws_ref.join("docs").join("conops").join("units"))
    };

    let output_dir = if let Some(out_dir) = args.output_dir {
        out_dir
    } else {
        ws_ref.join("docs").join("conops")
    };

    match assemble_conops(
        &input_dir,
        &output_dir,
        args.verify,
        args.params.as_deref(),
        args.domain.as_deref(),
        workspace.as_deref(),
    ) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(err) => {
            eprintln!("[!] Assembly execution failed: {err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_assemble_conops_main_7_section_rustdoc() {
        let src = include_str!("main.rs");
        assert!(src.contains("//! ## 1. Safety Intent & Regulatory Scope"));
        assert!(src.contains("//! ## 2. Invariants & Mathematical Properties"));
        assert!(src.contains("//! ## 3. Preconditions"));
        assert!(src.contains("//! ## 4. Postconditions"));
        assert!(src.contains("//! ## 5. Algorithmic Complexity Bounds"));
        assert!(src.contains("//! ## 6. Failure Modes & Fault Tolerance"));
        assert!(src.contains("//! ## 7. Realization & Traceability"));
        assert!(src.contains("//! Realises: [REQ-ASSEMBLE-CONOPS-CLI], [DO-330-TQL-4-CONOPS-CLI]"));
        assert!(src.contains("/// ## 1. Safety Intent & Regulatory Scope"));
        assert!(src.contains("/// Realises: ["));
    }
}
