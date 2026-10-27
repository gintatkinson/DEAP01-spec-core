//! CLI entrypoint for compile-sysml compiler and specification synchronization engine.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides the Single-Entry/Single-Exit (SESE) OS process adapter for the DEAP SysML v2 / KerML
//! compiler and synchronization engine under DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D.
//! ## 2. Invariants & Mathematical Properties
//! - Cyclomatic complexity $v(G) = 1$: delegates 100% of mode resolution and pipeline orchestration
//!   to `compile_sysml::driver::execute_cli` and exits via a single `std::process::exit(exit_code)`.
//! ## 3. Preconditions
//! - Invoked with OS command-line arguments parseable by `compile_sysml::cli::Cli::parse()`.
//! ## 4. Postconditions
//! - Propagates the deterministic exit code returned by `execute_cli` to the host operating system.
//! ## 5. Algorithmic Complexity Bounds
//! - Time complexity: $O(N \log N)$ bounded by `execute_cli`. Space complexity: $O(N)$.
//! ## 6. Failure Modes & Fault Tolerance
//! - All operational errors are logged to `stderr` by `execute_cli` and returned as a non-zero exit code.
//! ## 7. Realization & Traceability
//! Realises: [REQ-COMPILER-SYSML-CLI], [DO-330-TQL-4-CLI]

use clap::Parser;
use compile_sysml::cli::Cli;
use compile_sysml::driver::execute_cli;
use std::process;

/// Primary SESE OS entrypoint for the `compile-sysml` executable ($v(G) = 1$).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Binds OS command-line arguments via `Cli::parse()`, invokes `execute_cli(cli)`, and exits with
/// the returned status code under DO-178C Level A and DO-330 TQL-4 Humble Object design rules.
/// ## 2. Invariants & Mathematical Properties
/// - Zero conditional branches ($v(G) = 1$); single `process::exit(exit_code)` call.
/// ## 3. Preconditions
/// - Host environment supplies valid argument strings.
/// ## 4. Postconditions
/// - Terminates the process with `exit_code`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(N \log N)$. Space complexity: $O(N)$.
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-zero exit codes from `execute_cli` propagate directly to the OS process status.
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-MAIN], [DO-330-TQL-4-CLI]
fn main() {
    let exit_code = compile_sysml::sync::forward::run_cli_with_emit_flags(|| execute_cli(Cli::parse()));
    process::exit(exit_code);
}
