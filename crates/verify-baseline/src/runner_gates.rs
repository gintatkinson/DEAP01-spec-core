//! Gate execution stages for Checks 10..27 and Checks 36..42.

use crate::checks::*;
use crate::runner_helpers::write_defect_dossier;
use deap_core::workspace::is_upstream_compiler;
use std::path::Path;

/// Logs error messages and writes a defect dossier before returning `Err(())`.
/// Realises: [REQ-RUNNER-EXECUTE-ALL-CHECKS]
fn fail_check(repo_root: &Path, header: Option<&str>, check_name: &str, errs: &[String]) -> Result<(), ()> {
    if let Some(h) = header {
        eprintln!("{}", h);
        for e in errs {
            eprintln!("  - {}", e);
        }
    } else {
        for e in errs {
            eprintln!("ERROR: {}", e);
        }
    }
    write_defect_dossier(repo_root, check_name, errs);
    Err(())
}

/// Executes Checks 10 through 16 sequentially.
/// Realises: [REQ-RUNNER-EXECUTE-ALL-CHECKS]
pub(crate) fn run_checks_10_to_16(repo_root: &Path) -> Result<(), ()> {
    if let Err(err) = check_gitignore_exists(repo_root) {
        return fail_check(repo_root, None, "Check 10 (.gitignore exists)", &[err]);
    }
    println!("Success: Check 10 verified (.gitignore exists in repository root).");

    if let Err(errs) = check_no_ds_store_files(repo_root) {
        return fail_check(repo_root, None, "Check 11 (Zero .DS_Store files)", &errs);
    }
    println!("Success: Check 11 verified (Zero .DS_Store files in working tree).");

    if let Err(errs) = check_no_duplicate_master_blueprints(repo_root) {
        return fail_check(repo_root, None, "Check 12 (Zero duplicate master core blueprints)", &errs);
    }
    println!("Success: Check 12 verified (Zero duplicate master core blueprints in downstream repository).");

    if let Err(errs) = check_latex_katex_syntax(repo_root) {
        return fail_check(
            repo_root,
            Some("ERROR: Check 13 failed (KaTeX / LaTeX mathematical syntax violations found):"),
            "Check 13 (KaTeX / LaTeX syntax)",
            &errs,
        );
    }
    println!("Success: Check 13 verified (KaTeX / LaTeX mathematical syntax conforms to markdown rendering rules across all markdown files).");

    if let Err(errs) = check_mermaid_syntax(repo_root) {
        return fail_check(
            repo_root,
            Some("ERROR: Check 13B failed (Mermaid syntax violations found):"),
            "Check 13B (Mermaid syntax)",
            &errs,
        );
    }
    println!("Success: Check 13B verified (Mermaid syntax and diagram ergonomics verified across all markdown files).");

    if let Err(errs) = check_downstream_instructions_exist(repo_root) {
        return fail_check(repo_root, None, "Check 14 (Downstream instructions)", &errs);
    }
    println!("Success: Check 14 verified (Downstream repository contains valid instructions: README.md, AGENTS.md, and rules/sysml-ssot-completeness.md).");

    if let Err(errs) = check_reconcile_backlog_tooling_exists(repo_root) {
        return fail_check(repo_root, None, "Check 15 (Reconcile backlog tooling)", &errs);
    }
    println!("Success: Check 15 verified (Reconcile backlog tooling exists: target/release/reconcile-backlog or crates/reconcile-backlog).");

    if let Err(errs) = check_upstream_template_clean_landing_zones(repo_root) {
        return fail_check(repo_root, None, "Check 16 (Upstream template clean landing zones)", &errs);
    }
    println!("Success: Check 16 verified (Upstream template clean landing zone gate passed).");
    Ok(())
}

/// Executes Checks 17 through 22 sequentially.
/// Realises: [REQ-RUNNER-EXECUTE-ALL-CHECKS]
pub(crate) fn run_checks_17_to_22(
    repo_root: &Path,
    effective_allow_missing: bool,
    is_strict: bool,
) -> Result<(), ()> {
    match check_safety_integrity(repo_root, effective_allow_missing, is_strict) {
        Ok(SafetyCheckOutcome::UpstreamClean) => println!(
            "Success: Check 17 verified (Upstream distribution template safety landing zone is clean)."
        ),
        Ok(SafetyCheckOutcome::DownstreamPendingOrClean) => println!(
            "Success: Check 17 verified (Downstream repository detected -- safety specifications pending or clean)."
        ),
        Ok(SafetyCheckOutcome::DownstreamVerified) => println!(
            "Success: Check 17 verified (Safety Integrity Quality Gate: 8 pillars, regulatory objectives mapping, FMECA matrix with AST closure, 4 UCA categories, safety architecture, and toolchain hooks)."
        ),
        Err(errs) => {
            return fail_check(
                repo_root,
                Some("ERROR: Check 17 failed (Safety Integrity Quality Gate and Regulatory Objectives Completeness violations found):"),
                "Check 17 (Safety integrity & 8 pillars)",
                &errs,
            );
        }
    }

    match verify_upstream_blueprint_domain_cleanliness(repo_root) {
        Ok(()) => {
            if is_upstream_compiler(repo_root) {
                println!("Success: Check 18 verified (Upstream architecture blueprints are clean with zero domain concept papers or sysml models).");
            } else {
                println!("Success: Check 18 verified (Downstream repository detected -- skipping upstream blueprint domain cleanliness gate).");
            }
        }
        Err(errs) => {
            return fail_check(
                repo_root,
                None,
                "Check 18 (Upstream blueprint domain cleanliness)",
                &errs,
            );
        }
    }

    match check_domain_agnostic_ast_cleanliness(repo_root) {
        Ok(()) => {
            if is_upstream_compiler(repo_root) {
                println!("Success: Check 19 verified (Domain-Agnostic AST Cleanliness & Closed-Grammar Metamodel Gate passed -- pure dynamic schema AST architecture verified).");
            } else {
                println!("Success: Check 19 verified (Downstream repository detected -- skipping domain-agnostic AST cleanliness gate).");
            }
        }
        Err(errs) => {
            return fail_check(
                repo_root,
                Some("ERROR: Check 19 failed (Domain-Agnostic AST Cleanliness & Closed-Grammar Metamodel Gate violations found):"),
                "Check 19 (Domain-agnostic AST cleanliness)",
                &errs,
            );
        }
    }

    run_checks_20_to_22(repo_root, effective_allow_missing, is_strict)
}

/// Executes Checks 20 through 22 sequentially.
/// Realises: [REQ-RUNNER-EXECUTE-ALL-CHECKS]
fn run_checks_20_to_22(
    repo_root: &Path,
    effective_allow_missing: bool,
    is_strict: bool,
) -> Result<(), ()> {
    match check_wbs_suite_integrity(repo_root, effective_allow_missing, is_strict) {
        Ok(WbsCheckOutcome::PendingOrNotPresent) => {
            println!("Success: Check 20 verified (WBS & Enterprise Deliverables Suite pending or not present).");
        }
        Ok(WbsCheckOutcome::Verified) => {
            println!("Success: Check 20 verified (WBS & Enterprise Deliverables Suite validated: Markdown structure, CSV RFC 4180 with 12 headers, JSON AST, and zero em dashes).");
        }
        Err(errs) => {
            return fail_check(
                repo_root,
                Some("ERROR: Check 20 failed (WBS & Enterprise Deliverables Suite violations found):"),
                "Check 20 (WBS & Enterprise deliverables suite)",
                &errs,
            );
        }
    }

    match check_semantic_diagram_ast_parity(repo_root) {
        Ok(ParityOutcome::PendingOrClean) => {
            println!("Success: Check 21 verified (SysML model pending or landing zone clean).");
        }
        Ok(_) => {
            println!("Success: Check 21 verified (Semantic Diagram-to-AST Topology Parity Gate passed -- zero undeclared nodes, inverted flows, or ungrounded actuators).");
        }
        Err(errs) => {
            return fail_check(
                repo_root,
                Some("ERROR: Check 21 failed (Semantic Diagram-to-AST Topology Parity Gate violations found):"),
                "Check 21 (Semantic diagram AST parity)",
                &errs,
            );
        }
    }

    match check_semantic_prose_invariants(repo_root) {
        Ok(ParityOutcome::PendingOrClean) => {
            println!("Success: Check 22 verified (SysML model pending or landing zone clean).");
        }
        Ok(_) => {
            println!("Success: Check 22 verified (Physical Invariant Semantic Prose Gate passed -- zero ungrounded operational assertions).");
        }
        Err(errs) => {
            return fail_check(
                repo_root,
                Some("ERROR: Check 22 failed (Physical Invariant Semantic Prose Gate violations found):"),
                "Check 22 (Semantic prose invariants)",
                &errs,
            );
        }
    }
    Ok(())
}

/// Executes Checks 23 through 27 sequentially.
/// Realises: [REQ-RUNNER-EXECUTE-ALL-CHECKS]
pub(crate) fn run_checks_23_to_27(
    repo_root: &Path,
    effective_allow_missing: bool,
    is_strict: bool,
) -> Result<(), ()> {
    match check_factual_grounding(repo_root, effective_allow_missing, is_strict) {
        Ok(ParityOutcome::PendingOrClean) => println!("Success: Check 23 verified (SysML model pending or landing zone clean)."),
        Ok(_) => println!("Success: Check 23 verified (Factual Grounding & Numeric Provenance Gate passed -- zero ungrounded assertions)."),
        Err(errs) => {
            return fail_check(
                repo_root,
                Some("ERROR: Check 23 failed (Factual Grounding & Numeric Provenance Gate violations found):"),
                "Check 23 (Factual grounding)",
                &errs,
            );
        }
    }

    match check_icd_completeness(repo_root) {
        Ok(ParityOutcome::PendingOrClean) => println!("Success: Level 1C ICD Completeness verified (SysML model pending or landing zone clean)."),
        Ok(ParityOutcome::InterfacesDirMissing) => println!("Success: Level 1C ICD Completeness verified (Downstream repository detected -- docs/interfaces/ directory not present)."),
        Ok(ParityOutcome::IcdSpecsPending) => println!("Success: Level 1C ICD Completeness verified (Downstream repository detected -- Level 1C ICD specifications pending)."),
        Ok(_) => println!("Success: Level 1C ICD Completeness verified (zero dangling ports, 100% port contract parity)."),
        Err(errs) => {
            return fail_check(
                repo_root,
                Some("ERROR: ICD Completeness failed (Interface & Signal Dictionary violations found):"),
                "Check 24 (ICD completeness)",
                &errs,
            );
        }
    }

    match check_operational_allocation(repo_root) {
        Ok(_) => println!("Success: Check 25 verified (Operational-to-Resource Allocation passed -- zero orphan activities or phantom allocation tags)."),
        Err(errs) => {
            return fail_check(
                repo_root,
                Some("ERROR: Check 25 failed (Operational-to-Resource Allocation violations found):"),
                "Check 25 (Operational allocation)",
                &errs,
            );
        }
    }

    match check_standards_measurement(repo_root, effective_allow_missing, is_strict) {
        Ok(_) => {
            println!("Success: Check 26 verified (Standards & SI 7D Parameter Metrology passed -- all parameter dimensions, units, and SDO baselines valid).");
        }
        Err(errs) => {
            return fail_check(
                repo_root,
                Some("ERROR: Check 26 failed (Standards & SI 7D Parameter Metrology violations found):"),
                "Check 26 (Standards measurement)",
                &errs,
            );
        }
    }

    match check_cross_document_diagram_parity(repo_root) {
        Ok(_) => {
            println!("Success: Check 27 verified (Cross-Document Diagram Parity Gate passed -- zero disparity in subgraphs, nodes, ports, or connections).");
        }
        Err(errs) => {
            return fail_check(
                repo_root,
                Some("ERROR: Check 27 failed (Cross-Document Diagram Parity Gate violations found):"),
                "Check 27 (Cross-document diagram parity)",
                &errs,
            );
        }
    }
    Ok(())
}

/// Executes Checks 36 through 42 sequentially.
/// Realises: [REQ-RUNNER-EXECUTE-ALL-CHECKS]
pub(crate) fn run_checks_36_to_42(repo_root: &Path) -> Result<(), ()> {
    if let Err(errs) = check_docstring_completeness(repo_root) {
        return fail_check(
            repo_root,
            Some("ERROR: Check 36 failed (Per-Item 7-Section Safety Docstring Completeness Gate violated):"),
            "Check 36 (Per-Item 7-Section Safety Docstring Completeness Gate)",
            &errs,
        );
    }
    println!("Success: Check 36 verified (Per-Item 7-Section Safety Docstring Completeness Gate passed -- 100% per-item 7-section docstrings verified across Rust and Dart sources).");

    if let Err(errs) = check_blueprint_grounding(repo_root) {
        return fail_check(
            repo_root,
            Some("ERROR: Check 37 failed (Architecture Blueprint Grounding Gate violated):"),
            "Check 37 (Architecture Blueprint Grounding Gate)",
            &errs,
        );
    }
    println!("Success: Check 37 verified (Architecture Blueprint Grounding Gate passed -- all 13 blueprints grounded in workspace crates and paths).");

    if let Err(errs) = check_38_presentation_anti_hardcoding_ast_gate(repo_root) {
        return fail_check(
            repo_root,
            Some("ERROR: Check 38 failed (Presentation Anti-Hardcoding AST Gate violated):"),
            "Check 38 (Presentation Anti-Hardcoding AST Gate)",
            &errs,
        );
    }
    println!("Success: Check 38 verified (Presentation Anti-Hardcoding AST Gate passed).");

    if let Err(errs) = check_39_zero_mocking_test_ast_gate(repo_root) {
        return fail_check(
            repo_root,
            Some("ERROR: Check 39 failed (Zero-Mocking Test AST & SQLite Disposal Gate violated):"),
            "Check 39 (Zero-Mocking Test AST & SQLite Disposal Gate)",
            &errs,
        );
    }
    println!("Success: Check 39 verified (Zero-Mocking Test AST & SQLite Disposal Gate passed).");

    if let Err(errs) = check_40_interactive_gesture_and_hud_non_overlap_gate(repo_root) {
        return fail_check(
            repo_root,
            Some("ERROR: Check 40 failed (Interactive Gesture, 1080p-6K Viewport & Pairwise 2D HUD Non-Overlap Gate violated):"),
            "Check 40 (Interactive Gesture, 1080p-6K Viewport & Pairwise 2D HUD Non-Overlap Gate)",
            &errs,
        );
    }
    println!("Success: Check 40 verified (Interactive Gesture, 1080p-6K Viewport & Pairwise 2D HUD Non-Overlap Gate passed).");

    if let Err(errs) = check_41_solver_bridge_and_sqlite_seed_parity_gate(repo_root) {
        return fail_check(
            repo_root,
            Some("ERROR: Check 41 failed (Track B C-ABI Solver Bridge SSOT & 5-Table SQLite Seed Parity Gate violated):"),
            "Check 41 (Track B C-ABI Solver Bridge SSOT & 5-Table SQLite Seed Parity Gate)",
            &errs,
        );
    }
    println!("Success: Check 41 verified (Track B C-ABI Solver Bridge SSOT & 5-Table SQLite Seed Parity Gate passed).");

    if let Err(errs) = check_dod_gates(repo_root) {
        return fail_check(
            repo_root,
            Some("ERROR: Check 42 failed (DoD Modularity & Ratchet AST Gate violated):"),
            "Check 42 (DoD Modularity & Ratchet AST Gate)",
            &errs,
        );
    }
    println!("Success: Check 42 verified (DoD Modularity & Ratchet AST Gate passed).");
    Ok(())
}
