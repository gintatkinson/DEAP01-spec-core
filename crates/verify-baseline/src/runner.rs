//! Baseline verification runner orchestrating checks 10 through 36 and platform gates.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C baseline
//! verification orchestration standards. Executes Checks 10 through 36 sequentially in fail-closed
//! order and emits structured 5-Whys defect dossiers upon any verification failure.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Sequential Gate Monotonicity: Returns `Ok(())` if and only if every enabled check (`10..=36`)
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
use crate::spec_audit::{run_spec_audit, SpecAuditOptions};
use chrono::Utc;
use deap_core::diagnostics::{DefectDossier, DiagnosticSeverity, WhyEntry};
use deap_core::rules::EXCLUDED_DIRS;
use deap_core::workspace::is_upstream_compiler;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Configuration options for the baseline verification runner (`BaselineOptions`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 tool configuration management requirements.
/// Encapsulates all CLI and programmatic flags governing `verify_baseline::runner::run`.
///
/// ## 2. Invariants & Mathematical Properties
/// - `destination: PathBuf` specifies the workspace or platform root directory to audit.
/// - Boolean flags (`no_domain`, `strict`, `allow_missing_specs`, `spec_only`) deterministically
///   partition strict vs. permissive specification audit modes without disabling structural gates `10..=36`.
///
/// ## 3. Preconditions
/// - Constructed by CLI argument parsing (`main.rs`) or integration test harnesses with a valid `destination`.
///
/// ## 4. Postconditions
/// - Provides immutable configuration parameters to `run(&BaselineOptions)`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) field access.
/// - Space Complexity: O(P) where P is the byte length of configured path and filter strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Strongly typed fields eliminate untyped string flag misinterpretation.
/// - State: Value semantics (`Clone`, `Debug`) with no interior mutability.
/// - Action: Passed by shared reference `&BaselineOptions` into `run`.
/// - Resource: Bounded stack and small heap string footprint.
///
/// ## 7. Realization & Traceability
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

/// Detects whether `repo_root` contains concrete specification markdown files under `docs/`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A specification presence detection (`has_concrete_specifications`).
/// Determines whether `docs/epics`, `docs/features`, `docs/user-stories`, or `docs/use-cases`
/// contain concrete `.md` specifications requiring full specification quality auditing.
///
/// ## 2. Invariants & Mathematical Properties
/// - Ignores `.gitkeep`, `README.md`, dotfiles, and directories listed in `EXCLUDED_DIRS`.
/// - Returns `true` if and only if at least one concrete `.md` file exists in the four specification zones.
///
/// ## 3. Preconditions
/// - `repo_root: &Path` is a valid workspace directory path.
///
/// ## 4. Postconditions
/// - Returns `true` when concrete specification markdown files exist, or `false` when landing zones are empty.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(E) where E is the number of directory entries under the 4 specification zones.
/// - Space Complexity: O(D) where D is the maximum directory tree depth.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-existent specification zone directories are skipped cleanly without panicking.
/// - State: Read-only filesystem inspection with zero state mutation.
/// - Action: Short-circuits and returns `true` on the first discovered concrete `.md` specification file.
/// - Resource: Bounded `WalkDir` iterator state.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RUNNER-HAS-CONCRETE-SPECS]
pub fn has_concrete_specifications(repo_root: &Path) -> bool {
    let spec_zones = [
        repo_root.join("docs").join("epics"),
        repo_root.join("docs").join("features"),
        repo_root.join("docs").join("user-stories"),
        repo_root.join("docs").join("use-cases"),
    ];

    for szone in &spec_zones {
        if szone.is_dir() {
            for entry_res in WalkDir::new(szone).into_iter().filter_entry(|e| {
                if e.file_type().is_dir() {
                    let name = e.file_name().to_string_lossy();
                    !EXCLUDED_DIRS.iter().any(|ex| name == ex)
                } else {
                    true
                }
            }) {
                let entry = match entry_res {
                    Ok(val) => val,
                    Err(_) => continue,
                };
                if entry.file_type().is_file() {
                    let fname = entry.file_name().to_string_lossy();
                    if fname.ends_with(".md")
                        && fname != ".gitkeep"
                        && fname != "README.md"
                        && !fname.starts_with('.')
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Detects whether `repo_root` has clean schema or specification landing zones (`has_clean_landing_zones`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A M2/M1 template landing-zone detection rules.
/// Identifies whether `schema/` or the specification landing zones are still in their unpopulated
/// template state (containing only `.gitkeep` or `README.md`).
///
/// ## 2. Invariants & Mathematical Properties
/// - `schema_clean` is `true` iff `schema/` contains zero files other than `.gitkeep`, `README.md`, or dotfiles.
/// - `specs_clean` is `!has_concrete_specifications(repo_root)`.
/// - Returns `schema_clean || specs_clean`.
///
/// ## 3. Preconditions
/// - `repo_root: &Path` references a valid workspace directory.
///
/// ## 4. Postconditions
/// - Returns `true` when either the schema landing zone or specification landing zones are clean.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(S + E) where S is entry count in `schema/` and E is entry count in spec zones.
/// - Space Complexity: O(D) bounded by directory traversal depth.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Safely handles absent `schema/` or `docs/` directories.
/// - State: Pure read-only filesystem query.
/// - Action: Breaks early as soon as a concrete file is found in `schema/`.
/// - Resource: O(1) auxiliary variables plus `WalkDir` stack.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RUNNER-CLEAN-LANDING-ZONES]
pub fn has_clean_landing_zones(repo_root: &Path) -> bool {
    // 1. Schema landing zone check
    let schema_dir = repo_root.join("schema");
    let mut schema_clean = true;
    if schema_dir.is_dir() {
        for entry_res in WalkDir::new(&schema_dir).into_iter().filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !EXCLUDED_DIRS.iter().any(|ex| name == ex)
            } else {
                true
            }
        }) {
            let entry = match entry_res {
                Ok(val) => val,
                Err(_) => continue,
            };
            if entry.file_type().is_file() {
                let fname = entry.file_name().to_string_lossy();
                if fname != ".gitkeep" && fname != "README.md" && !fname.starts_with('.') {
                    schema_clean = false;
                    break;
                }
            }
        }
    }

    // 2. Specification landing zones check (epics, features, user-stories, use-cases)
    let specs_clean = !has_concrete_specifications(repo_root);
    schema_clean || specs_clean
}

/// Persists a structured 5-Whys `DefectDossier` to `.pipeline/defects/` on baseline check failure.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C non-conformance reporting (`write_defect_dossier`).
/// Synthesizes and writes both JSON (`defect_<ts>.json`) and Markdown (`defect_<ts>.md`) failure
/// dossiers into `.pipeline/defects/` whenever a baseline verification check fails.
///
/// ## 2. Invariants & Mathematical Properties
/// - Populates all 5 causal `WhyEntry` records, `DiagnosticSeverity::Critical`, `check_name`, and `errors`.
/// - Emits paired JSON and Markdown representations under `repo_root/.pipeline/defects/`.
///
/// ## 3. Preconditions
/// - `repo_root: &Path` is the target workspace directory.
/// - `check_name: &str` identifies the failing baseline check (`Check 10`..`Check 36`).
/// - `errors: &[String]` contains one or more diagnostic violation messages.
///
/// ## 4. Postconditions
/// - Creates `.pipeline/defects/` if absent and writes the serialized `DefectDossier` artifacts.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M) where M is the total byte length of `errors`.
/// - Space Complexity: O(M) for the formatted `DefectDossier` JSON and Markdown buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Logs an explicit warning to `stderr` if directory creation or file persistence fails.
/// - State: Modifies only `.pipeline/defects/` on disk.
/// - Action: Non-panicking error handling across serialization and file writes.
/// - Resource: Bounded by diagnostic message slice size.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RUNNER-WRITE-DEFECT-DOSSIER]
pub fn write_defect_dossier(repo_root: &Path, check_name: &str, errors: &[String]) {
    let defects_dir = repo_root.join(".pipeline").join("defects");
    if let Err(err) = fs::create_dir_all(&defects_dir) {
        eprintln!(
            "Warning: Failed to create defects directory '{}': {}",
            defects_dir.display(),
            err
        );
    }

    let ts = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let err_summary = errors.join("\n- ");
    let dossier = DefectDossier {
        title: format!("Baseline check failure: {}", check_name),
        severity: DiagnosticSeverity::Critical,
        file_location: repo_root.display().to_string(),
        pillar: "Pillar 1: System Baseline Integrity".to_string(),
        symptom: format!("Violations found during {}:\n- {}", check_name, err_summary),
        whys: [
            WhyEntry::new(
                "did the baseline verification check fail",
                format!("{} failed with {} violation(s)", check_name, errors.len()),
            ),
            WhyEntry::new(
                "were these violations present",
                "required specification files or syntax constraints were violated",
            ),
            WhyEntry::new(
                "did this occur in the repository",
                "specifications or workspace files drifted from baseline governance standards",
            ),
            WhyEntry::new(
                "was this not caught earlier",
                "verification was not executed before commit",
            ),
            WhyEntry::new(
                "is this critical",
                "all downstream projects must strictly satisfy baseline gates before delivery",
            ),
        ],
        impact: "Halts pipeline verification and downstream implementation until resolved.".to_string(),
        architectural_models: "DEAP Platform Specification and Baseline Architecture.".to_string(),
        remediation: format!("Resolve the following violations:\n- {}", err_summary),
        verification: "Re-run verify-baseline until all checks pass cleanly.".to_string(),
        regression_prevention: "Run verify-baseline before every commit and PR merge.".to_string(),
    };

    let json_path = defects_dir.join(format!("defect_{}.json", ts));
    let md_path = defects_dir.join(format!("defect_{}.md", ts));

    if let Ok(json_str) = dossier.to_json() {
        if let Err(err) = fs::write(&json_path, json_str) {
            eprintln!(
                "Warning: Failed to write JSON defect dossier '{}': {}",
                json_path.display(),
                err
            );
        }
    }
    if let Err(err) = fs::write(&md_path, dossier.to_markdown()) {
        eprintln!(
            "Warning: Failed to write Markdown defect dossier '{}': {}",
            md_path.display(),
            err
        );
    }
}

/// Executes all baseline verification checks (Checks 10 through 36), platform gates, and spec audits.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D baseline verification rules.
/// Orchestrates Checks 10 through 36 (including Check 36 per-item 7-section safety docstring gate),
/// Flutter/React platform baseline file checks, and `run_spec_audit` according to `options`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strict Gate Ordering: Executes Checks 10..=36 in deterministic ascending order; halts immediately
///   with `Err(())` and writes a `DefectDossier` if any check returns `Err`.
/// - Environment Override: Honors `options.strict` or `DEAP_STRICT_BASELINE=1|true|yes`.
///
/// ## 3. Preconditions
/// - `options: &BaselineOptions` specifies an existing `destination` path and valid audit flags.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` if and only if all enabled checks (`10..=36`), platform checks, and spec audits succeed.
/// - Returns `Err(())` and persists a defect dossier on any check violation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(F * L) across all repository files F and average line count L.
/// - Space Complexity: O(F + V) for file manifests and diagnostic buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Canonicalizes `options.destination` with graceful fallback to the raw path.
/// - State: Stateless execution writing only diagnostic telemetry and `.pipeline/defects/` dossiers on error.
/// - Action: Every check failure returns `Err(())` immediately (fail-closed).
/// - Resource: Sequential gate execution bounds peak memory usage.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RUNNER-EXECUTE-ALL-CHECKS]
#[allow(clippy::result_unit_err)]
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

    let effective_allow_missing = if !is_strict && !options.allow_missing_specs && has_clean_landing_zones(&repo_root) {
        true
    } else {
        options.allow_missing_specs && !is_strict
    };

    if options.spec_only {
        let spec_options = SpecAuditOptions {
            repo_root: repo_root.clone(),
            only_file: options.only.as_ref().map(PathBuf::from),
            gate_filter: options.gate.clone(),
            allow_missing_specs: options.allow_missing_specs,
            strict: is_strict,
        };

        match run_spec_audit(&spec_options) {
            Ok(summary) => {
                if !summary.passed {
                    let finding_msgs: Vec<String> =
                        summary.findings.iter().map(|f| f.to_string()).collect();
                    if is_strict || options.only.is_some() {
                        write_defect_dossier(
                            &repo_root,
                            "Specification Quality & Coverage Audit",
                            &finding_msgs,
                        );
                        eprintln!(
                            "ERROR: Specification audit failed with {} finding(s):",
                            summary.total_findings
                        );
                        for msg in &finding_msgs {
                            eprintln!("  - {}", msg);
                        }
                        return Err(());
                    } else {
                        println!(
                            "Specification Quality & Coverage Audit completed with {} finding(s) (permissive mode).",
                            summary.total_findings
                        );
                        return Ok(());
                    }
                } else {
                    println!(
                        "Success: Specification Quality & Coverage Audit passed ({} files audited in {} ms).",
                        summary.total_files_audited, summary.duration_ms
                    );
                    return Ok(());
                }
            }
            Err(e) => {
                eprintln!("ERROR: Specification audit execution failed: {}", e);
                write_defect_dossier(
                    &repo_root,
                    "Specification Audit Execution",
                    &[e.to_string()],
                );
                return Err(());
            }
        }
    }

    // Check 10: .gitignore exists
    if let Err(err) = check_gitignore_exists(&repo_root) {
        eprintln!("ERROR: {}", err);
        write_defect_dossier(&repo_root, "Check 10 (.gitignore exists)", &[err]);
        return Err(());
    }
    println!("Success: Check 10 verified (.gitignore exists in repository root).");

    // Check 11: Zero .DS_Store files
    if let Err(errs) = check_no_ds_store_files(&repo_root) {
        for e in &errs {
            eprintln!("ERROR: {}", e);
        }
        write_defect_dossier(&repo_root, "Check 11 (Zero .DS_Store files)", &errs);
        return Err(());
    }
    println!("Success: Check 11 verified (Zero .DS_Store files in working tree).");

    // Check 12: Zero duplicate master core blueprints
    if let Err(errs) = check_no_duplicate_master_blueprints(&repo_root) {
        for e in &errs {
            eprintln!("ERROR: {}", e);
        }
        write_defect_dossier(&repo_root, "Check 12 (Zero duplicate master core blueprints)", &errs);
        return Err(());
    }
    println!("Success: Check 12 verified (Zero duplicate master core blueprints in downstream repository).");

    // Check 13: KaTeX / LaTeX syntax validation
    if let Err(errs) = check_latex_katex_syntax(&repo_root) {
        eprintln!("ERROR: Check 13 failed (KaTeX / LaTeX mathematical syntax violations found):");
        for e in &errs {
            eprintln!("  - {}", e);
        }
        write_defect_dossier(&repo_root, "Check 13 (KaTeX / LaTeX syntax)", &errs);
        return Err(());
    }
    println!("Success: Check 13 verified (KaTeX / LaTeX mathematical syntax conforms to markdown rendering rules across all markdown files).");

    // Check 13B: Mermaid syntax validation
    if let Err(errs) = check_mermaid_syntax(&repo_root) {
        eprintln!("ERROR: Check 13B failed (Mermaid syntax violations found):");
        for e in &errs {
            eprintln!("  - {}", e);
        }
        write_defect_dossier(&repo_root, "Check 13B (Mermaid syntax)", &errs);
        return Err(());
    }
    println!("Success: Check 13B verified (Mermaid syntax and diagram ergonomics verified across all markdown files).");

    // Check 14: Downstream instructions exist
    if let Err(errs) = check_downstream_instructions_exist(&repo_root) {
        for e in &errs {
            eprintln!("ERROR: {}", e);
        }
        write_defect_dossier(&repo_root, "Check 14 (Downstream instructions)", &errs);
        return Err(());
    }
    println!("Success: Check 14 verified (Downstream repository contains valid instructions: README.md, AGENTS.md, and rules/sysml-ssot-completeness.md).");

    // Check 15: Reconcile backlog tooling exists
    if let Err(errs) = check_reconcile_backlog_tooling_exists(&repo_root) {
        for e in &errs {
            eprintln!("ERROR: {}", e);
        }
        write_defect_dossier(&repo_root, "Check 15 (Reconcile backlog tooling)", &errs);
        return Err(());
    }
    println!("Success: Check 15 verified (Reconcile backlog tooling exists: target/release/reconcile-backlog or crates/reconcile-backlog).");

    // Check 16: Upstream template clean landing zone gate
    if let Err(errs) = check_upstream_template_clean_landing_zones(&repo_root) {
        for e in &errs {
            eprintln!("ERROR: {}", e);
        }
        write_defect_dossier(&repo_root, "Check 16 (Upstream template clean landing zones)", &errs);
        return Err(());
    }
    println!("Success: Check 16 verified (Upstream template clean landing zone gate passed).");

    // Check 17: Safety integrity quality gate
    match check_safety_integrity(&repo_root, effective_allow_missing, is_strict) {
        Ok(SafetyCheckOutcome::UpstreamClean) => {
            println!("Success: Check 17 verified (Upstream distribution template safety landing zone is clean).");
        }
        Ok(SafetyCheckOutcome::DownstreamPendingOrClean) => {
            println!("Success: Check 17 verified (Downstream repository detected -- safety specifications pending or clean).");
        }
        Ok(SafetyCheckOutcome::DownstreamVerified) => {
            println!("Success: Check 17 verified (Safety Integrity Quality Gate: 8 pillars, regulatory objectives mapping, FMECA matrix with AST closure, 4 UCA categories, safety architecture, and toolchain hooks).");
        }
        Err(errs) => {
            eprintln!("ERROR: Check 17 failed (Safety Integrity Quality Gate and Regulatory Objectives Completeness violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(&repo_root, "Check 17 (Safety integrity & 8 pillars)", &errs);
            return Err(());
        }
    }

    // Check 18: Upstream blueprint domain cleanliness
    match verify_upstream_blueprint_domain_cleanliness(&repo_root) {
        Ok(()) => {
            if is_upstream_compiler(&repo_root) {
                println!("Success: Check 18 verified (Upstream architecture blueprints are clean with zero domain concept papers or sysml models).");
            } else {
                println!("Success: Check 18 verified (Downstream repository detected -- skipping upstream blueprint domain cleanliness gate).");
            }
        }
        Err(errs) => {
            for e in &errs {
                eprintln!("ERROR: {}", e);
            }
            write_defect_dossier(&repo_root, "Check 18 (Upstream blueprint domain cleanliness)", &errs);
            return Err(());
        }
    }

    // Check 19: Domain-agnostic AST cleanliness
    match check_domain_agnostic_ast_cleanliness(&repo_root) {
        Ok(()) => {
            if is_upstream_compiler(&repo_root) {
                println!("Success: Check 19 verified (Domain-Agnostic AST Cleanliness & Closed-Grammar Metamodel Gate passed -- pure dynamic schema AST architecture verified).");
            } else {
                println!("Success: Check 19 verified (Downstream repository detected -- skipping domain-agnostic AST cleanliness gate).");
            }
        }
        Err(errs) => {
            eprintln!("ERROR: Check 19 failed (Domain-Agnostic AST Cleanliness & Closed-Grammar Metamodel Gate violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(&repo_root, "Check 19 (Domain-agnostic AST cleanliness)", &errs);
            return Err(());
        }
    }

    // Check 20: WBS & Enterprise Deliverables Suite Validation
    match check_wbs_suite_integrity(&repo_root, effective_allow_missing, is_strict) {
        Ok(WbsCheckOutcome::PendingOrNotPresent) => {
            println!("Success: Check 20 verified (WBS & Enterprise Deliverables Suite pending or not present).");
        }
        Ok(WbsCheckOutcome::Verified) => {
            println!("Success: Check 20 verified (WBS & Enterprise Deliverables Suite validated: Markdown structure, CSV RFC 4180 with 12 headers, JSON AST, and zero em dashes).");
        }
        Err(errs) => {
            eprintln!("ERROR: Check 20 failed (WBS & Enterprise Deliverables Suite violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(&repo_root, "Check 20 (WBS & Enterprise deliverables suite)", &errs);
            return Err(());
        }
    }

    // Check 21: Semantic diagram AST parity
    match check_semantic_diagram_ast_parity(&repo_root) {
        Ok(ParityOutcome::PendingOrClean) => {
            println!("Success: Check 21 verified (SysML model pending or landing zone clean).");
        }
        Ok(_) => {
            println!("Success: Check 21 verified (Semantic Diagram-to-AST Topology Parity Gate passed -- zero undeclared nodes, inverted flows, or ungrounded actuators).");
        }
        Err(errs) => {
            eprintln!("ERROR: Check 21 failed (Semantic Diagram-to-AST Topology Parity Gate violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(&repo_root, "Check 21 (Semantic diagram AST parity)", &errs);
            return Err(());
        }
    }

    // Check 22: Semantic prose invariants
    match check_semantic_prose_invariants(&repo_root) {
        Ok(ParityOutcome::PendingOrClean) => {
            println!("Success: Check 22 verified (SysML model pending or landing zone clean).");
        }
        Ok(_) => {
            println!("Success: Check 22 verified (Physical Invariant Semantic Prose Gate passed -- zero ungrounded operational assertions).");
        }
        Err(errs) => {
            eprintln!("ERROR: Check 22 failed (Physical Invariant Semantic Prose Gate violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(&repo_root, "Check 22 (Semantic prose invariants)", &errs);
            return Err(());
        }
    }

    // Check 23: Factual grounding
    match check_factual_grounding(&repo_root, effective_allow_missing, is_strict) {
        Ok(ParityOutcome::PendingOrClean) => {
            println!("Success: Check 23 verified (SysML model pending or landing zone clean).");
        }
        Ok(_) => {
            println!("Success: Check 23 verified (Factual Grounding & Numeric Provenance Gate passed -- zero ungrounded assertions).");
        }
        Err(errs) => {
            eprintln!("ERROR: Check 23 failed (Factual Grounding & Numeric Provenance Gate violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(&repo_root, "Check 23 (Factual grounding)", &errs);
            return Err(());
        }
    }

    // Check 24: Level 1C ICD completeness
    match check_icd_completeness(&repo_root) {
        Ok(ParityOutcome::PendingOrClean) => {
            println!("Success: Level 1C ICD Completeness verified (SysML model pending or landing zone clean).");
        }
        Ok(ParityOutcome::InterfacesDirMissing) => {
            println!("Success: Level 1C ICD Completeness verified (Downstream repository detected -- docs/interfaces/ directory not present).");
        }
        Ok(ParityOutcome::IcdSpecsPending) => {
            println!("Success: Level 1C ICD Completeness verified (Downstream repository detected -- Level 1C ICD specifications pending).");
        }
        Ok(_) => {
            println!("Success: Level 1C ICD Completeness verified (zero dangling ports, 100% port contract parity).");
        }
        Err(errs) => {
            eprintln!("ERROR: ICD Completeness failed (Interface & Signal Dictionary violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(&repo_root, "Check 24 (ICD completeness)", &errs);
            return Err(());
        }
    }

    // Check 25: Operational allocation
    match check_operational_allocation(&repo_root) {
        Ok(_) => {
            println!("Success: Check 25 verified (Operational-to-Resource Allocation passed -- zero orphan activities or phantom allocation tags).");
        }
        Err(errs) => {
            eprintln!("ERROR: Check 25 failed (Operational-to-Resource Allocation violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(&repo_root, "Check 25 (Operational allocation)", &errs);
            return Err(());
        }
    }

    // Check 26: Standards measurement
    match check_standards_measurement(&repo_root, effective_allow_missing, is_strict) {
        Ok(_) => {
            println!("Success: Check 26 verified (Standards & SI 7D Parameter Metrology passed -- all parameter dimensions, units, and SDO baselines valid).");
        }
        Err(errs) => {
            eprintln!("ERROR: Check 26 failed (Standards & SI 7D Parameter Metrology violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(&repo_root, "Check 26 (Standards measurement)", &errs);
            return Err(());
        }
    }

    // Check 27: Cross-document diagram parity
    match check_cross_document_diagram_parity(&repo_root) {
        Ok(_) => {
            println!("Success: Check 27 verified (Cross-Document Diagram Parity Gate passed -- zero disparity in subgraphs, nodes, ports, or connections).");
        }
        Err(errs) => {
            eprintln!("ERROR: Check 27 failed (Cross-Document Diagram Parity Gate violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(&repo_root, "Check 27 (Cross-document diagram parity)", &errs);
            return Err(());
        }
    }

    // Check 28: ConOps and Mission Intent completeness
    match check_conops_and_mission_intent_completeness(&repo_root, effective_allow_missing, is_strict) {
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
            eprintln!("ERROR: Check 28 failed (ConOps & Mission Intent Completeness violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(&repo_root, "Check 28 (ConOps & Mission Intent)", &errs);
            return Err(());
        }
    }

    // Check 29: Research inventory
    match check_research_inventory(&repo_root, effective_allow_missing, is_strict) {
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
            write_defect_dossier(&repo_root, "Check 29 (Research inventory)", &errs);
            return Err(());
        }
    }

    // Check 30: Executive deliverable traceability
    match check_executive_deliverable_traceability(&repo_root) {
        Ok(_) => {
            println!("Success: Check 30 verified (Executive Deliverable Traceability Gate passed -- all tables and diagrams anchored to SSOT).");
        }
        Err(errs) => {
            eprintln!("ERROR: Check 30 failed (Executive Deliverable Traceability Gate violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(&repo_root, "Check 30 (Executive deliverable traceability)", &errs);
            return Err(());
        }
    }

    // Checks 30A-30C and 31: Coverage digest (30A), obligation witness (30B), architecture viewpoint diagrams (30C), dual-schema parity (31)
    match check_coverage_digest(&repo_root) {
        Ok(ParityOutcome::PendingOrClean) => {
            println!("Success: Check 30A verified (Downstream repository detected -- coverage digest pending or landing zone clean).");
        }
        Ok(_) => {
            println!("Success: Check 30A verified (Coverage-Digest Population Gate passed -- zero phantom realizations).");
        }
        Err(errs) => {
            eprintln!("ERROR: Check 30A failed (Coverage-Digest Population Gate violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(&repo_root, "Check 30A (Coverage digest)", &errs);
            return Err(());
        }
    }

    match check_obligation_witness(&repo_root) {
        Ok(ParityOutcome::PendingOrClean) => {
            println!("Success: Check 30B verified (Downstream repository detected -- obligation witness pending or landing zone clean).");
        }
        Ok(_) => {
            println!("Success: Check 30B verified (Obligation-Witness Registry Gate passed -- zero phantom witnesses).");
        }
        Err(errs) => {
            eprintln!("ERROR: Check 30B failed (Obligation-Witness Registry Gate violations found):");
            for e in &errs {
                eprintln!("  - {}", e);
            }
            write_defect_dossier(&repo_root, "Check 30B (Obligation witness)", &errs);
            return Err(());
        }
    }

    match check_architecture_viewpoint_diagrams(&repo_root, effective_allow_missing, is_strict) {
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
            write_defect_dossier(&repo_root, "Check 30C (Architecture viewpoint diagrams)", &errs);
            return Err(());
        }
    }

    match check_dual_schema_ssot_parity(&repo_root) {
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
            write_defect_dossier(&repo_root, "Check 31 (Dual-schema SSOT parity)", &errs);
            return Err(());
        }
    }

    // Check 32: Zero Shell Scripts Invariant
    if let Err(errs) = check_zero_shell_scripts(&repo_root) {
        eprintln!("ERROR: Check 32 failed (Zero Shell Scripts Invariant violated -- .sh files found on disk):");
        for e in &errs {
            eprintln!("  - {}", e);
        }
        write_defect_dossier(&repo_root, "Check 32 (Zero Shell Scripts Invariant)", &errs);
        return Err(());
    }
    println!("Success: Check 32 verified (Zero Shell Scripts Invariant passed -- zero .sh files exist on disk).");

    // Check 33: Zero Python Scripts Invariant
    if let Err(errs) = check_zero_python_scripts(&repo_root) {
        eprintln!("ERROR: Check 33 failed (Zero Python Scripts Invariant violated -- .py files found on disk):");
        for e in &errs {
            eprintln!("  - {}", e);
        }
        write_defect_dossier(&repo_root, "Check 33 (Zero Python Scripts Invariant)", &errs);
        return Err(());
    }
    println!("Success: Check 33 verified (Zero Python Scripts Invariant passed -- zero .py files exist on disk).");

    // Check 34: Zero Regex Invariant
    if let Err(errs) = check_zero_regex(&repo_root) {
        eprintln!("ERROR: Check 34 failed (Zero Regex Invariant violated -- regex usage found across crates):");
        for e in &errs {
            eprintln!("  - {}", e);
        }
        write_defect_dossier(&repo_root, "Check 34 (Zero Regex Invariant)", &errs);
        return Err(());
    }
    println!("Success: Check 34 verified (Zero Regex Invariant passed -- zero regex occurrences exist across crates).");

    // Check 35: Handoff & Backlog Integrity
    if let Err(errs) = check_handoff_and_backlog_integrity(&repo_root) {
        eprintln!("ERROR: Check 35 failed (Handoff & Backlog Integrity violated):");
        for e in &errs {
            eprintln!("  - {}", e);
        }
        write_defect_dossier(&repo_root, "Check 35 (Handoff & Backlog Integrity)", &errs);
        return Err(());
    }
    println!("Success: Check 35 verified (Handoff & Backlog Integrity passed).");

    // Check 36: Per-Item 7-Section Safety Docstring Completeness Gate
    if let Err(errs) = check_docstring_completeness(&repo_root) {
        eprintln!(
            "ERROR: Check 36 failed (Per-Item 7-Section Safety Docstring Completeness Gate violated):"
        );
        for e in &errs {
            eprintln!("  - {}", e);
        }
        write_defect_dossier(
            &repo_root,
            "Check 36 (Per-Item 7-Section Safety Docstring Completeness Gate)",
            &errs,
        );
        return Err(());
    }
    println!("Success: Check 36 verified (Per-Item 7-Section Safety Docstring Completeness Gate passed -- 100% per-item 7-section docstrings verified across Rust and Dart sources).");


    // Platform Checks
    let dest = &options.destination;
    let is_flutter = dest.join("pubspec.yaml").is_file();
    let is_react = dest.join("tsconfig.json").is_file() || (dest.join("package.json").is_file() && !is_flutter);

    if is_flutter {
        println!("Verifying conformance for platform 'flutter' at '{}'...", dest.display());
        let baseline_files = [
            "pubspec.yaml",
            "analysis_options.yaml",
            "lib/main.dart",
            "lib/domain/validation.dart",
        ];
        let mut missing_files = Vec::new();
        for f in &baseline_files {
            if !dest.join(f).exists() {
                missing_files.push(f.to_string());
            }
        }
        let repo_resolvers = [
            dest.join("lib").join("domain").join("repository_resolver.dart"),
            dest.join("lib").join("core").join("di").join("repository_resolver.dart"),
        ];
        if !repo_resolvers.iter().any(|p| p.exists()) && !options.no_domain {
            missing_files.push("lib/domain/repository_resolver.dart (or lib/core/di/repository_resolver.dart)".to_string());
        }

        if !missing_files.is_empty() {
            eprintln!("ERROR: Flutter baseline file(s) missing: {}", missing_files.join(", "));
            write_defect_dossier(
                &repo_root,
                "Flutter Baseline Files",
                &[format!("Missing Flutter baseline files: {}", missing_files.join(", "))],
            );
            return Err(());
        }
        println!("Success: All Flutter baseline files exist.");

        if options.no_domain {
            println!("Skipping domain type compatibility validation (--no-domain specified).");
            println!("Skipping build and test suite execution (--no-domain specified, domain implementation pending).");
        }
    } else if is_react {
        println!("Verifying conformance for platform 'react' at '{}'...", dest.display());
        let baseline_files = [
            "package.json",
            "tsconfig.json",
            "src/App.tsx",
            "src/domain/validation.ts",
        ];
        let mut missing_files = Vec::new();
        for f in &baseline_files {
            if !dest.join(f).exists() {
                missing_files.push(f.to_string());
            }
        }
        let repo_resolver = dest.join("src").join("domain").join("RepositoryResolver.ts");
        if !repo_resolver.exists() && !options.no_domain {
            missing_files.push("src/domain/RepositoryResolver.ts".to_string());
        }

        if !missing_files.is_empty() {
            eprintln!("ERROR: React baseline file(s) missing: {}", missing_files.join(", "));
            write_defect_dossier(
                &repo_root,
                "React Baseline Files",
                &[format!("Missing React baseline files: {}", missing_files.join(", "))],
            );
            return Err(());
        }
        println!("Success: All React baseline files exist.");

        if options.no_domain {
            println!("Skipping domain type compatibility validation (--no-domain specified).");
            println!("Skipping build command execution (--no-domain specified, domain implementation pending).");
        }
    } else {
        println!("Downstream repository root verified (non-framework platform or root orchestration workspace).");
    }

    if has_concrete_specifications(&repo_root) {
        println!("Concrete specifications detected -- executing Specification Quality & Coverage Audit...");
        let spec_options = SpecAuditOptions {
            repo_root: repo_root.clone(),
            only_file: options.only.as_ref().map(PathBuf::from),
            gate_filter: options.gate.clone(),
            allow_missing_specs: true,
            strict: is_strict,
        };

        match run_spec_audit(&spec_options) {
            Ok(summary) => {
                if !summary.passed {
                    let finding_msgs: Vec<String> =
                        summary.findings.iter().map(|f| f.to_string()).collect();
                    if is_strict {
                        write_defect_dossier(
                            &repo_root,
                            "Specification Quality & Coverage Audit",
                            &finding_msgs,
                        );
                        eprintln!(
                            "ERROR: Specification audit failed in strict mode with {} finding(s):",
                            summary.total_findings
                        );
                        for msg in &finding_msgs {
                            eprintln!("  - {}", msg);
                        }
                        return Err(());
                    } else {
                        println!(
                            "Specification Quality & Coverage Audit completed with {} finding(s) (permissive mode).",
                            summary.total_findings
                        );
                    }
                } else {
                    println!(
                        "Success: Specification Quality & Coverage Audit passed ({} files audited in {} ms).",
                        summary.total_files_audited, summary.duration_ms
                    );
                }
            }
            Err(e) => {
                eprintln!("ERROR: Specification audit execution failed: {}", e);
                write_defect_dossier(
                    &repo_root,
                    "Specification Audit Execution",
                    &[e.to_string()],
                );
                return Err(());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_repo_has_clean_landing_zones() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("repo root");

        let is_clean = has_clean_landing_zones(repo_root);
        let spec_zones = [
            repo_root.join("docs").join("epics"),
            repo_root.join("docs").join("features"),
        ];
        let has_specs = spec_zones.iter().any(|d| {
            d.is_dir()
                && WalkDir::new(d)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .any(|e| e.file_type().is_file() && e.file_name().to_string_lossy().ends_with(".md"))
        });

        if has_specs {
            assert!(!is_clean || is_upstream_compiler(repo_root));
        } else {
            assert!(is_clean);
        }
    }

    #[test]
    fn test_runner_check_34_zero_regex() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("repo root");
        let res = check_zero_regex(repo_root);
        assert!(res.is_ok(), "Check 34 Zero Regex Invariant failed on repo: {:?}", res.err());
    }

    #[test]
    fn test_issue_440_runner_check_identifiers_unique() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("repo root");
        let runner_src = fs::read_to_string(repo_root.join("crates/verify-baseline/src/runner.rs"))
            .expect("read runner.rs");

        let mut check_identifiers = Vec::new();
        for line in runner_src.lines() {
            if let Some(pos) = line.find("write_defect_dossier(") {
                let rest = &line[pos..];
                if let Some(first_quote) = rest.find('"') {
                    let after_first = &rest[first_quote + 1..];
                    if let Some(second_quote) = after_first.find('"') {
                        let full_check_name = &after_first[..second_quote];
                        if let Some(paren) = full_check_name.find(" (") {
                            let check_id = &full_check_name[..paren];
                            check_identifiers.push(check_id.to_string());
                        }
                    }
                }
            }
        }

        let post_27: Vec<&String> = check_identifiers
            .iter()
            .skip_while(|id| *id != "Check 28")
            .take_while(|id| *id != "Check 35")
            .collect();

        let mut seen = std::collections::HashSet::new();
        let mut duplicates = Vec::new();
        for id in &post_27 {
            if !seen.insert(*id) {
                duplicates.push((*id).clone());
            }
        }

        assert!(
            duplicates.is_empty(),
            "Duplicate check identifiers found in runner.rs write_defect_dossier calls: {:?}",
            duplicates
        );

        assert!(
            !runner_src.contains("\"Check 28 (Coverage digest)\""),
            "runner.rs must not label Coverage Digest as Check 28"
        );
        assert!(
            !runner_src.contains("\"Check 29 (Obligation witness)\""),
            "runner.rs must not label Obligation Witness as Check 29"
        );
        assert!(
            !runner_src.contains("\"Check 30 (Architecture viewpoint diagrams)\""),
            "runner.rs must not label Architecture Viewpoint as Check 30"
        );
        assert!(
            runner_src.contains("\"Check 30A (Coverage digest)\""),
            "runner.rs must label Coverage Digest as Check 30A"
        );
        assert!(
            runner_src.contains("\"Check 30B (Obligation witness)\""),
            "runner.rs must label Obligation Witness as Check 30B"
        );
        assert!(
            runner_src.contains("\"Check 30C (Architecture viewpoint diagrams)\""),
            "runner.rs must label Architecture Viewpoint as Check 30C"
        );
    }
}

