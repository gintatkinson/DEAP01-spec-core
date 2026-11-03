//! Helper utilities, dossier writer, and platform/spec-audit phase runners for `runner.rs`.

use crate::runner::BaselineOptions;
use crate::spec_audit::{run_spec_audit, SpecAuditOptions};
use chrono::Utc;
use deap_core::diagnostics::{DefectDossier, DiagnosticSeverity, WhyEntry};
use deap_core::rules::EXCLUDED_DIRS;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Detects whether `repo_root` contains concrete specification markdown files under `docs/`.
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
/// Realises: [REQ-RUNNER-CLEAN-LANDING-ZONES]
pub fn has_clean_landing_zones(repo_root: &Path) -> bool {
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

    let specs_clean = !has_concrete_specifications(repo_root);
    schema_clean || specs_clean
}

/// Persists a structured 5-Whys `DefectDossier` to `.pipeline/defects/` on baseline check failure.
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
        impact: "Halts pipeline verification and downstream implementation until resolved."
            .to_string(),
        architectural_models: "DEAP Platform Specification and Baseline Architecture.".to_string(),
        remediation: format!("Resolve the following violations:\n- {}", err_summary),
        verification: "Re-run verify-baseline until all checks pass cleanly.".to_string(),
        regression_prevention: "Run verify-baseline before every commit and PR merge.".to_string(),
    };

    let json_path = defects_dir.join(format!("defect_{}.json", ts));
    let md_path = defects_dir.join(format!("defect_{}.md", ts));

    if let Ok(json_str) = dossier.to_json() {
        if let Err(err) = crate::write_atomic_file(&json_path, json_str) {
            eprintln!("Warning: Failed to write JSON defect dossier '{}': {}", json_path.display(), err);
        }
    }
    if let Err(err) = crate::write_atomic_file(&md_path, dossier.to_markdown()) {
        eprintln!("Warning: Failed to write Markdown defect dossier '{}': {}", md_path.display(), err);
    }
}

/// Executes `--spec-only` audit mode and returns early.
/// Realises: [REQ-RUNNER-EXECUTE-ALL-CHECKS]
pub(crate) fn run_spec_only_mode(
    repo_root: &Path,
    options: &BaselineOptions,
    is_strict: bool,
) -> Result<(), ()> {
    let spec_options = SpecAuditOptions {
        repo_root: repo_root.to_path_buf(),
        only_file: options.only.as_ref().map(PathBuf::from),
        gate_filter: options.gate.clone(),
        allow_missing_specs: options.allow_missing_specs,
        strict: is_strict,
    };

    match run_spec_audit(&spec_options) {
        Ok(summary) => {
            if !summary.passed {
                let finding_msgs: Vec<String> = summary.findings.iter().map(|f| f.to_string()).collect();
                if is_strict || options.only.is_some() {
                    write_defect_dossier(repo_root, "Specification Quality & Coverage Audit", &finding_msgs);
                    eprintln!("ERROR: Specification audit failed with {} finding(s):", summary.total_findings);
                    for msg in &finding_msgs {
                        eprintln!("  - {}", msg);
                    }
                    return Err(());
                }
                println!(
                    "Specification Quality & Coverage Audit completed with {} finding(s) (permissive mode).",
                    summary.total_findings
                );
                return Ok(());
            }
            println!(
                "Success: Specification Quality & Coverage Audit passed ({} files audited in {} ms).",
                summary.total_files_audited, summary.duration_ms
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("ERROR: Specification audit execution failed: {}", e);
            write_defect_dossier(repo_root, "Specification Audit Execution", &[e.to_string()]);
            Err(())
        }
    }
}

/// Verifies Flutter or React platform baseline files under `options.destination`.
/// Realises: [REQ-RUNNER-EXECUTE-ALL-CHECKS]
pub(crate) fn run_platform_baseline_checks(
    repo_root: &Path,
    options: &BaselineOptions,
) -> Result<(), ()> {
    let dest = &options.destination;
    let is_flutter = dest.join("pubspec.yaml").is_file();
    let is_react = dest.join("tsconfig.json").is_file()
        || (dest.join("package.json").is_file() && !is_flutter);

    if is_flutter {
        verify_flutter_baseline(repo_root, dest, options.no_domain)
    } else if is_react {
        verify_react_baseline(repo_root, dest, options.no_domain)
    } else {
        println!("Downstream repository root verified (non-framework platform or root orchestration workspace).");
        Ok(())
    }
}

/// Verifies Flutter baseline files in `dest`.
/// Realises: [REQ-RUNNER-EXECUTE-ALL-CHECKS]
fn verify_flutter_baseline(repo_root: &Path, dest: &Path, no_domain: bool) -> Result<(), ()> {
    println!(
        "Verifying conformance for platform 'flutter' at '{}'...",
        dest.display()
    );
    let pubspec_text = std::fs::read_to_string(dest.join("pubspec.yaml")).unwrap_or_default();
    let consumes_tier1_sdk =
        !pubspec_text.contains("name: app_flutter") && pubspec_text.contains("app_flutter:");
    let baseline_files = ["pubspec.yaml", "analysis_options.yaml", "lib/main.dart"];
    let mut missing_files = Vec::new();
    for f in &baseline_files {
        if !dest.join(f).exists() {
            missing_files.push(f.to_string());
        }
    }
    if !consumes_tier1_sdk && !dest.join("lib/domain/validation.dart").exists() {
        missing_files.push("lib/domain/validation.dart".to_string());
    }
    let repo_resolvers = [
        dest.join("lib").join("domain").join("repository_resolver.dart"),
        dest.join("lib").join("core").join("di").join("repository_resolver.dart"),
    ];
    if !consumes_tier1_sdk && !repo_resolvers.iter().any(|p| p.exists()) && !no_domain {
        missing_files.push(
            "lib/domain/repository_resolver.dart (or lib/core/di/repository_resolver.dart)"
                .to_string(),
        );
    }

    if !missing_files.is_empty() {
        eprintln!(
            "ERROR: Flutter baseline file(s) missing: {}",
            missing_files.join(", ")
        );
        write_defect_dossier(
            repo_root,
            "Flutter Baseline Files",
            &[format!("Missing Flutter baseline files: {}", missing_files.join(", "))],
        );
        return Err(());
    }
    println!("Success: All Flutter baseline files exist.");
    if no_domain {
        println!("Skipping domain type compatibility validation (--no-domain specified).");
        println!("Skipping build and test suite execution (--no-domain specified, domain implementation pending).");
    }
    Ok(())
}

/// Verifies React baseline files in `dest`.
/// Realises: [REQ-RUNNER-EXECUTE-ALL-CHECKS]
fn verify_react_baseline(repo_root: &Path, dest: &Path, no_domain: bool) -> Result<(), ()> {
    println!(
        "Verifying conformance for platform 'react' at '{}'...",
        dest.display()
    );
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
    if !repo_resolver.exists() && !no_domain {
        missing_files.push("src/domain/RepositoryResolver.ts".to_string());
    }

    if !missing_files.is_empty() {
        eprintln!(
            "ERROR: React baseline file(s) missing: {}",
            missing_files.join(", ")
        );
        write_defect_dossier(
            repo_root,
            "React Baseline Files",
            &[format!("Missing React baseline files: {}", missing_files.join(", "))],
        );
        return Err(());
    }
    println!("Success: All React baseline files exist.");
    if no_domain {
        println!("Skipping domain type compatibility validation (--no-domain specified).");
        println!("Skipping build command execution (--no-domain specified, domain implementation pending).");
    }
    Ok(())
}

/// Executes the post-baseline specification quality audit when concrete specifications exist.
/// Realises: [REQ-RUNNER-EXECUTE-ALL-CHECKS]
pub(crate) fn run_post_baseline_spec_audit(
    repo_root: &Path,
    options: &BaselineOptions,
    is_strict: bool,
) -> Result<(), ()> {
    if !has_concrete_specifications(repo_root) {
        return Ok(());
    }
    println!(
        "Concrete specifications detected -- executing Specification Quality & Coverage Audit..."
    );
    let spec_options = SpecAuditOptions {
        repo_root: repo_root.to_path_buf(),
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
                        repo_root,
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
                }
                println!(
                    "Specification Quality & Coverage Audit completed with {} finding(s) (permissive mode).",
                    summary.total_findings
                );
            } else {
                println!(
                    "Success: Specification Quality & Coverage Audit passed ({} files audited in {} ms).",
                    summary.total_files_audited, summary.duration_ms
                );
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("ERROR: Specification audit execution failed: {}", e);
            write_defect_dossier(
                repo_root,
                "Specification Audit Execution",
                &[e.to_string()],
            );
            Err(())
        }
    }
}
