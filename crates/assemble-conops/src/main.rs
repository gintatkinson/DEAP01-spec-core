//! CLI binary for `assemble-conops` -- Deterministic ConOps & Mission Intent Assembly Engine. [DOC-TIER: A]
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
//! - Invoked with valid CLI arguments parsed by `clap::Command`.
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
//! Realises: [REQ-ASSEMBLE-CONOPS-CLI], [DO-330-TQL-4-CONOPS-CLI] [FLS: fls_assemble_conops_cli]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Arg, ArgAction, Command};

use assemble_conops::assemble_conops;

/// [DOC-TIER: A] Executes deterministic struct `Args` within `crates/assemble-conops/src/main.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `Args` in `crates/assemble-conops/src/main.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `Args` across member set `[workspace: Option<PathBuf>, workspace_flag: Option<PathBuf>, input_dir: Option<PathBuf>, output_dir: Option<PathBuf>, verify: bool]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `Args` members (`workspace: Option<PathBuf>, workspace_flag: Option<PathBuf>, input_dir: Option<PathBuf>, output_dir: Option<PathBuf>, verify: bool`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `Args` representation for struct `Args`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `Args`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `Args`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `Args` (`Args`).
/// - State: Maintains consistent state invariants across `Args` without partial mutation on error.
/// - Action: Executes `Args` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `Args` (`ASSEMBLE-CONOPS-MAIN`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-ASSEMBLE-CONOPS-MAIN-ARGS] [FLS: fls_assemble_conops_cli] | SysML v2: PartDef(Args)
#[derive(Debug)]
struct Args {
    /// Target workspace or project directory (`[REQ-ASSEMBLE-CONOPS-MAIN-ARGS]`, `[FLS: fls_assemble_conops_cli]`).
    workspace: Option<PathBuf>,

    /// Target workspace or project directory flag (`[REQ-ASSEMBLE-CONOPS-MAIN-ARGS]`, `[FLS: fls_assemble_conops_cli]`).
    workspace_flag: Option<PathBuf>,

    /// Input directory containing 'conops/' and 'mission_intent/' unit markdown directories (`[REQ-ASSEMBLE-CONOPS-MAIN-ARGS]`, `[FLS: fls_assemble_conops_cli]`).
    input_dir: Option<PathBuf>,

    /// Output directory where assembled CONOPS.md and MISSION_INTENT.md are written (`[REQ-ASSEMBLE-CONOPS-MAIN-ARGS]`, `[FLS: fls_assemble_conops_cli]`).
    output_dir: Option<PathBuf>,

    /// Verify unit integrity and link resolution without writing output files (`[REQ-ASSEMBLE-CONOPS-MAIN-ARGS]`, `[FLS: fls_assemble_conops_cli]`).
    verify: bool,

    /// Path to JSON parameter dictionary or schema file (`[REQ-ASSEMBLE-CONOPS-MAIN-ARGS]`, `[FLS: fls_assemble_conops_cli]`).
    params: Option<PathBuf>,

    /// Target operational domain (`[REQ-ASSEMBLE-CONOPS-MAIN-ARGS]`, `[FLS: fls_assemble_conops_cli]`).
    domain: Option<String>,
}

impl Args {
    /// [DOC-TIER: A] Parses process command-line arguments into [`Args`] using explicit `clap::Command` builders.
    /// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-MAIN-ARGS] [FLS: fls_assemble_conops_cli]
    /// Contract: Constructs the CLI grammar via `clap::Command` and returns a populated [`Args`] instance.
    /// Bounds & Faults: O(A) time and space over CLI argument tokens; invalid flags exit non-zero via clap.
    fn parse() -> Self {
        let mut matches = Command::new("assemble-conops")
            .author("Gint Atkinson")
            .version("0.1.0")
            .about("Deterministic ConOps & Mission Intent Assembly Engine (ISO/IEC/IEEE 29148 / ISO/IEC/IEEE 15288 / OMG UAF).")
            .arg(
                Arg::new("workspace")
                    .value_name("WORKSPACE")
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("workspace_flag")
                    .long("workspace")
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("input_dir")
                    .long("input-dir")
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("output_dir")
                    .long("output-dir")
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("verify")
                    .long("verify")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("params")
                    .long("params")
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(Arg::new("domain").long("domain"))
            .get_matches();

        Self {
            workspace: matches.remove_one::<PathBuf>("workspace"),
            workspace_flag: matches.remove_one::<PathBuf>("workspace_flag"),
            input_dir: matches.remove_one::<PathBuf>("input_dir"),
            output_dir: matches.remove_one::<PathBuf>("output_dir"),
            verify: matches.get_flag("verify"),
            params: matches.remove_one::<PathBuf>("params"),
            domain: matches.remove_one::<String>("domain"),
        }
    }
}

/// [DOC-TIER: A] Inspects whether a candidate filesystem path is a valid ConOps unit root directory.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-CLI-DISCOVERY] [FLS: fls_assemble_conops_cli]
/// Contract: Returns `true` if `cand` contains `conops/`, `mission_intent/`, or a child directory containing `conops/`.
/// Bounds & Faults: O(E) time over immediate directory entries; unreadable entries are skipped without panicking.
fn check_candidate_input_dir(cand: &Path) -> bool {
    if !cand.is_dir() {
        return false;
    }
    if cand.join("conops").is_dir() || cand.join("mission_intent").is_dir() {
        return true;
    }
    if let Ok(entries) = fs::read_dir(cand) {
        for entry_res in entries {
            let Ok(entry) = entry_res else { continue };
            let path = entry.path();
            if path.is_dir() && path.join("conops").is_dir() {
                return true;
            }
        }
    }
    false
}

/// [DOC-TIER: A] Executes the `assemble-conops` command-line interface and maps outcomes to process exit codes.
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-ASSEMBLE-CONOPS-CLI] [FLS: fls_assemble_conops_cli]
/// Contract: Resolves workspace/input/output directories, invokes `assemble_conops`, and returns `ExitCode::SUCCESS` or `ExitCode::FAILURE`.
/// Bounds & Faults: O(D + N log N) time; logs errors to `stderr` and exits non-zero on failure.
fn main() -> ExitCode {
    let args = Args::parse();

    let target_ws = args.workspace_flag.or(args.workspace);
    let explicit_io = args.input_dir.is_some() || args.output_dir.is_some();

    let workspace: Option<PathBuf> = if let Some(ws) = target_ws {
        Some(ws.canonicalize().unwrap_or(ws))
    } else if !explicit_io {
        match std::env::current_dir() {
            Ok(dir) => Some(dir),
            Err(_) => None,
        }
    } else {
        None
    };

    let ws_ref = workspace.clone().unwrap_or_else(|| PathBuf::from("."));

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
        (args.domain.as_deref(), workspace.as_deref()),
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

    #[test]
    fn test_assemble_conops_main_no_unconditional_safety_lint_allows() {
        let src = include_str!("main.rs");
        let prod_src = src.split("#[cfg(test)]").next().unwrap_or(src);
        let has_unconditional_inner_allow = prod_src
            .lines()
            .any(|line| line.trim_start().starts_with("#![allow("));
        assert!(!has_unconditional_inner_allow);
    }
}
