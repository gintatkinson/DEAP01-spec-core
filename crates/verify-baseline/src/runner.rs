//! Baseline verification runner orchestrating checks 10 through 42 and platform gates.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C baseline
//! verification orchestration standards. Executes Checks 10 through 42 sequentially in fail-closed
//! order and emits structured 5-Whys defect dossiers upon any verification failure.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Sequential Gate Monotonicity: Returns `Ok(())` if and only if every enabled check (`10..=42`)
//!   and platform/specification audit passes without error.
//! - Dossier Persistence on Failure: Every failing gate persists a timestamped JSON and Markdown
//!   `DefectDossier` under `.pipeline/defects/` prior to returning `Err(())`.
//!
//! ## 3. Preconditions
//! - Caller supplies an initialized `BaselineOptions` structure with an accessible `destination` directory.
//!
//! ## 4. Postconditions
//! - Returns `Ok(())` when all baseline checks pass, or `Err(())` immediately upon the first failing gate.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(F * L) bounded by repository directory traversal and per-file check complexity.
//! - Space Complexity: O(F + V) for discovered file manifests and diagnostic vectors.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Unreadable paths or check failures emit explicit `ERROR:` lines to `stderr` and return `Err(())`.
//! - State: Stateless runner execution parameterized solely by `BaselineOptions`.
//! - Action: Fails closed on the first violated gate without skipping mandatory checks.
//! - Resource: Bounded memory across sequential gate invocations.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-VERIFY-BASELINE-RUNNER], [DO-330-TQL-4-RUNNER]

use crate::checks::*;
use crate::runner_gates::{
    run_checks_10_to_16, run_checks_17_to_22, run_checks_23_to_27, run_checks_36_to_42,
};
use crate::runner_helpers::{
    run_platform_baseline_checks, run_post_baseline_spec_audit, run_spec_only_mode,
};
pub use crate::runner_helpers::{
    has_clean_landing_zones, has_concrete_specifications, write_defect_dossier,
};
use std::path::{Path, PathBuf};

/// Configuration options for the baseline verification runner (`BaselineOptions`).
/// Realises: [REQ-VERIFY-BASELINE-OPTIONS]
#[derive(Debug, Clone)]
pub struct BaselineOptions {
    pub destination: PathBuf,
    pub no_domain: bool,
    pub strict: bool,
    pub allow_missing_specs: bool,
    pub target: Option<String>,
    pub output: Option<String>,
    pub spec_only: bool,
    pub only: Option<String>,
    pub gate: Option<String>,
}

/// Executes all baseline verification checks (Checks 10 through 42), platform gates, and spec audits.
/// Realises: [REQ-RUNNER-EXECUTE-ALL-CHECKS]
pub fn run(options: &BaselineOptions) -> Result<(), ()> {
    let repo_root = options
        .destination
        .canonicalize()
        .unwrap_or_else(|_| options.destination.clone());

    let is_strict = options.strict
        || std::env::var("DEAP_STRICT_BASELINE")
            .map(|v| {
                let lower = v.to_lowercase();
                lower == "1" || lower == "true" || lower == "yes"
            })
            .unwrap_or(false);

    let effective_allow_missing =
        if !is_strict && !options.allow_missing_specs && has_clean_landing_zones(&repo_root) {
            true
        } else {
            options.allow_missing_specs && !is_strict
        };

    if options.spec_only {
        return run_spec_only_mode(&repo_root, options, is_strict);
    }

    run_checks_10_to_16(&repo_root)?;
    run_checks_17_to_22(&repo_root, effective_allow_missing, is_strict)?;
    run_checks_23_to_27(
        &repo_root,
        effective_allow_missing,
        is_strict,
        options.only.as_deref(),
    )?;
    run_checks_28_to_30(&repo_root, effective_allow_missing, is_strict)?;
    run_checks_30a_to_30c(&repo_root, effective_allow_missing, is_strict)?;
    run_checks_31_to_35(&repo_root)?;
    run_checks_36_to_42(&repo_root)?;
    run_platform_baseline_checks(&repo_root, options)?;
    run_post_baseline_spec_audit(&repo_root, options, is_strict)?;

    Ok(())
}

/// Executes Checks 28 through 30 sequentially.
/// Realises: [REQ-RUNNER-EXECUTE-ALL-CHECKS]
fn run_checks_28_to_30(
    repo_root: &Path,
    effective_allow_missing: bool,
    is_strict: bool,
) -> Result<(), ()> {
    match check_conops_and_mission_intent_completeness(
        repo_root,
        effective_allow_missing,
        is_strict,
    ) {
        Ok(ParityOutcome::ConopsDirMissing) => {
            println!("Success: Check 28 verified (Downstream repository detected -- docs/conops/ directory not present).");
        }
        Ok(ParityOutcome::PendingOrClean) => {
            println!("Success: Check 28 verified (Downstream repository detected -- ConOps & Mission Intent pending or clean).");
        }
        Ok(_) => {
            println!("Success: Check 28 verified (ConOps & Mission Intent Completeness passed -- all mandatory sections, tables, and METL rosters valid).");
        }
        Err(errs) => {
            eprintln!(
                "ERROR: Check 28 failed (ConOps & Mission Intent Completeness violations found):"
            );
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(repo_root, "Check 28 (ConOps & Mission Intent)", &errs);
            return Err(());
        }
    }

    match check_research_inventory(repo_root, effective_allow_missing, is_strict) {
        Ok(ParityOutcome::PendingOrClean) => {
            println!("Success: Check 29 verified (Downstream repository detected -- research inventory pending or landing zone clean).");
        }
        Ok(_) => {
            println!("Success: Check 29 verified (Cited Research Inventory & Declared-Total Population Register passed).");
        }
        Err(errs) => {
            eprintln!("ERROR: Check 29 failed (Cited Research Inventory violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(repo_root, "Check 29 (Research inventory)", &errs);
            return Err(());
        }
    }

    match check_executive_deliverable_traceability(repo_root) {
        Ok(_) => {
            println!("Success: Check 30 verified (Executive Deliverable Traceability Gate passed -- all tables and diagrams anchored to SSOT).");
        }
        Err(errs) => {
            eprintln!("ERROR: Check 30 failed (Executive Deliverable Traceability Gate violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(repo_root, "Check 30 (Executive deliverable traceability)", &errs);
            return Err(());
        }
    }
    Ok(())
}

/// Executes Checks 30A through 30C sequentially.
/// Realises: [REQ-RUNNER-EXECUTE-ALL-CHECKS]
fn run_checks_30a_to_30c(
    repo_root: &Path,
    effective_allow_missing: bool,
    is_strict: bool,
) -> Result<(), ()> {

    match check_coverage_digest(repo_root) {
        Ok(ParityOutcome::PendingOrClean) => {
            println!("Success: Check 30A verified (Downstream repository detected -- coverage digest pending or landing zone clean).");
        }
        Ok(_) => {
            println!("Success: Check 30A verified (Coverage-Digest Population Gate passed -- zero phantom realizations).");
        }
        Err(errs) => {
            eprintln!(
                "ERROR: Check 30A failed (Coverage-Digest Population Gate violations found):"
            );
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(repo_root, "Check 30A (Coverage digest)", &errs);
            return Err(());
        }
    }

    match check_obligation_witness(repo_root) {
        Ok(ParityOutcome::PendingOrClean) => {
            println!("Success: Check 30B verified (Downstream repository detected -- obligation witness pending or landing zone clean).");
        }
        Ok(_) => {
            println!("Success: Check 30B verified (Obligation-Witness Registry Gate passed -- zero phantom witnesses).");
        }
        Err(errs) => {
            eprintln!(
                "ERROR: Check 30B failed (Obligation-Witness Registry Gate violations found):"
            );
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(repo_root, "Check 30B (Obligation witness)", &errs);
            return Err(());
        }
    }

    match check_architecture_viewpoint_diagrams(repo_root, effective_allow_missing, is_strict) {
        Ok(ParityOutcome::PendingOrClean) => {
            println!("Success: Check 30C verified (Downstream repository detected -- architecture diagrams pending or landing zone clean).");
        }
        Ok(_) => {
            println!("Success: Check 30C verified (Architecture Viewpoint & Diagram Completeness Gate passed -- all 11 canonical diagrams verified).");
        }
        Err(errs) => {
            eprintln!("ERROR: Check 30C failed (Architecture Viewpoint & Diagram Completeness violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(repo_root, "Check 30C (Architecture viewpoint diagrams)", &errs);
            return Err(());
        }
    }
    Ok(())
}

/// Executes Checks 31 through 35 sequentially.
/// Realises: [REQ-RUNNER-EXECUTE-ALL-CHECKS]
fn run_checks_31_to_35(repo_root: &Path) -> Result<(), ()> {
    match check_dual_schema_ssot_parity(repo_root) {
        Ok(ParityOutcome::DualSchemaSingleOrClean) => {
            println!("Success: Check 31 verified (Dual-schema SSOT parity gate passed -- single schema or landing zone clean).");
        }
        Ok(ParityOutcome::DualSchemaIdentical) => {
            println!("Success: Check 31 verified (Dual-Schema SSOT Parity Gate passed -- schema/*.sysml and .pipeline/schema.sysml AST definitions are identical).");
        }
        Ok(_) => {
            println!("Success: Check 31 verified (Dual-Schema SSOT Parity Gate passed).");
        }
        Err(errs) => {
            eprintln!("ERROR: Check 31 failed (Dual-Schema SSOT Parity violations found -- schema drift detected):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(repo_root, "Check 31 (Dual-schema SSOT parity)", &errs);
            return Err(());
        }
    }

    if let Err(errs) = check_zero_shell_scripts(repo_root) {
        eprintln!("ERROR: Check 32 failed (Zero Shell Scripts Invariant violated -- .sh files found on disk):");
        for e in &errs {
            eprintln!("  - {}", e);
        }
        write_defect_dossier(repo_root, "Check 32 (Zero Shell Scripts Invariant)", &errs);
        return Err(());
    }
    println!("Success: Check 32 verified (Zero Shell Scripts Invariant passed -- zero .sh files exist on disk).");

    if let Err(errs) = check_zero_python_scripts(repo_root) {
        eprintln!("ERROR: Check 33 failed (Zero Python Scripts Invariant violated -- .py files found on disk):");
        for e in &errs {
            eprintln!("  - {}", e);
        }
        write_defect_dossier(repo_root, "Check 33 (Zero Python Scripts Invariant)", &errs);
        return Err(());
    }
    println!("Success: Check 33 verified (Zero Python Scripts Invariant passed -- zero .py files exist on disk).");

    if let Err(errs) = check_zero_regex(repo_root) {
        eprintln!("ERROR: Check 34 failed (Zero Regex Invariant violated -- regex usage found across crates):");
        for e in &errs {
            eprintln!("  - {}", e);
        }
        write_defect_dossier(repo_root, "Check 34 (Zero Regex Invariant)", &errs);
        return Err(());
    }
    println!("Success: Check 34 verified (Zero Regex Invariant passed -- zero regex occurrences exist across crates).");

    if let Err(errs) = check_handoff_and_backlog_integrity(repo_root) {
        eprintln!("ERROR: Check 35 failed (Handoff & Backlog Integrity violated):");
        for e in &errs {
            eprintln!("  - {}", e);
        }
        write_defect_dossier(repo_root, "Check 35 (Handoff & Backlog Integrity)", &errs);
        return Err(());
    }
    println!("Success: Check 35 verified (Handoff & Backlog Integrity passed).");
    Ok(())
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
