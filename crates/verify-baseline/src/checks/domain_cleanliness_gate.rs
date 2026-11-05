//! Domain-Agnostic AST Cleanliness and Closed-Grammar Metamodel Allowlist Gate (Check 19).
//! Realises: [REQ-CHECKS-DOMAIN-CLEANLINESS-GATE-CHECK-DOMAIN-AGNOSTIC-AST-CLEANLINESS]

pub use crate::checks::domain_cleanliness_rules::{
    is_allowed_m2_type, ALLOWED_M2_METAMODEL_TYPES, METAMODEL_TYPE_KEYS,
};
use crate::checks::domain_cleanliness_rules::{
    check_line_for_metamodel_typing_violation, extract_rust_const_or_static_ident,
    is_m1_domain_dict_name, is_static_param_dict_name,
};
use deap_core::rules::EXCLUDED_DIRS;
use deap_core::workspace::is_upstream_compiler;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Check 19: Domain-Agnostic AST Cleanliness & Closed-Grammar Metamodel Gate.
/// Realises: [REQ-CHECKS-DOMAIN-CLEANLINESS-GATE-CHECK-DOMAIN-AGNOSTIC-AST-CLEANLINESS]
pub fn check_domain_agnostic_ast_cleanliness(repo_root: &Path) -> Result<(), Vec<String>> {
    if !is_upstream_compiler(repo_root) {
        return Ok(());
    }

    let mut violations = Vec::new();
    let legacy_paths = [
        repo_root
            .join("skills")
            .join("spec-orchestrator")
            .join("parity_auditor"),
        repo_root.join("parity_auditor"),
    ];
    for p in &legacy_paths {
        if p.exists() {
            let rel = p.strip_prefix(repo_root).unwrap_or(p).display().to_string();
            violations.push(format!(
                "Check 19 violation: Legacy Python parity_auditor directory detected on disk: {rel}"
            ));
        }
    }

    let crates_dir = repo_root.join("crates");
    if !crates_dir.is_dir() {
        violations.push(
            "Check 19 violation: Native Rust workspace crates/ directory missing in upstream compiler repository."
                .to_string(),
        );
    } else {
        scan_crates_for_domain_cleanliness(repo_root, &crates_dir, &mut violations);
    }

    scan_templates_for_domain_cleanliness(repo_root, &mut violations);

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        Err(violations)
    }
}

/// Scans `.rs` files under `crates/` for M1 domain dictionaries and metamodel typing violations.
/// Realises: [REQ-CHECKS-DOMAIN-CLEANLINESS-GATE-CHECK-DOMAIN-AGNOSTIC-AST-CLEANLINESS]
fn scan_crates_for_domain_cleanliness(
    repo_root: &Path,
    crates_dir: &Path,
    violations: &mut Vec<String>,
) {
    for entry_res in WalkDir::new(crates_dir).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let name = e.file_name().to_string_lossy();
            name != "target" && name != ".git" && !EXCLUDED_DIRS.iter().any(|ex| name == ex)
        } else {
            true
        }
    }) {
        let entry = match entry_res {
            Ok(val) => val,
            Err(err) => {
                violations.push(format!("Check 19 directory walk I/O error: {err}"));
                continue;
            }
        };
        if entry.file_type().is_file()
            && entry.path().extension().is_some_and(|ext| ext == "rs")
        {
            scan_single_rust_file_cleanliness(repo_root, entry.path(), violations);
        }
    }
}

/// Audits a single Rust source file for Check 19 M1 domain dictionaries and metamodel violations.
/// Realises: [REQ-CHECKS-DOMAIN-CLEANLINESS-GATE-CHECK-DOMAIN-AGNOSTIC-AST-CLEANLINESS]
fn scan_single_rust_file_cleanliness(
    repo_root: &Path,
    path: &Path,
    violations: &mut Vec<String>,
) {
    let rel = path
        .strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string();
    match fs::read_to_string(path) {
        Ok(content) => {
            for (idx, line) in content.lines().enumerate() {
                if let Some(ident) = extract_rust_const_or_static_ident(line) {
                    if is_m1_domain_dict_name(ident) {
                        violations.push(format!(
                            "Check 19 violation (domain-metamodel-typing-violation): Unvalidated M1 domain instance dictionary/constant \"{ident}\" declared in {rel}:{}. Upstream compiler ASTs must adhere strictly to the closed M2 metamodel allowlist.",
                            idx + 1
                        ));
                    } else if is_static_param_dict_name(ident) {
                        violations.push(format!(
                            "Check 19 violation: Static hardcoded parameter dictionary declared in {rel}:{}.",
                            idx + 1
                        ));
                    }
                }
                if let Some(bad_type) = check_line_for_metamodel_typing_violation(line) {
                    violations.push(format!(
                        "Check 19 violation (domain-metamodel-typing-violation): Unvalidated M1 domain instance entity/type '{bad_type}' declared in {rel}:{}. Upstream compiler ASTs and dictionaries must adhere strictly to the closed M2 metamodel allowlist.",
                        idx + 1
                    ));
                }
            }
        }
        Err(err) => violations.push(format!("Check 19 violation: Failed to read '{rel}': {err}")),
    }
}

/// Scans upstream templates and rules directories for unmapped domain concepts and em dashes.
/// Realises: [REQ-CHECKS-DOMAIN-CLEANLINESS-GATE-CHECK-DOMAIN-AGNOSTIC-AST-CLEANLINESS]
fn scan_templates_for_domain_cleanliness(repo_root: &Path, violations: &mut Vec<String>) {
    let template_dirs = [
        repo_root
            .join("skills")
            .join("spec-conops-engineering")
            .join("resources")
            .join("units"),
        repo_root.join(".pipeline").join("templates"),
        repo_root.join("rules"),
    ];

    for t_dir in &template_dirs {
        if !t_dir.is_dir() {
            continue;
        }
        for entry_res in WalkDir::new(t_dir).into_iter().filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !EXCLUDED_DIRS.iter().any(|ex| name == ex)
            } else {
                true
            }
        }) {
            let entry = match entry_res {
                Ok(val) => val,
                Err(err) => {
                    violations.push(format!("Check 19 template directory walk I/O error: {err}"));
                    continue;
                }
            };
            if entry.file_type().is_file() {
                scan_single_template_file_cleanliness(repo_root, entry.path(), violations);
            }
        }
    }
}

/// Audits a single template or rule file for Unicode em dashes and M1 domain concepts.
/// Realises: [REQ-CHECKS-DOMAIN-CLEANLINESS-GATE-CHECK-DOMAIN-AGNOSTIC-AST-CLEANLINESS]
fn scan_single_template_file_cleanliness(
    repo_root: &Path,
    path: &Path,
    violations: &mut Vec<String>,
) {
    let rel = path
        .strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string();
    match fs::read_to_string(path) {
        Ok(content) => {
            for (idx, line) in content.lines().enumerate() {
                if line.contains('\u{2014}') {
                    violations.push(format!(
                        "Check 19 violation: Upstream template '{rel}:{}' contains forbidden Unicode em dash. Use ASCII -- exclusively.",
                        idx + 1
                    ));
                }
                for concept in &[
                    "FLIGHT_ENVELOPE_MAP",
                    "SENSOR_CALIBRATION_DATA",
                    "PAYLOAD_CONFIG",
                    "ruddervator",
                    "empennage",
                ] {
                    if line.to_lowercase().contains(&concept.to_lowercase()) {
                        violations.push(format!(
                            "Check 19 violation (domain-metamodel-typing-violation): Upstream rule or template '{rel}:{}' contains forbidden domain concept '{concept}'. Upstream templates and rules must derive strictly from schema AST nodes.",
                            idx + 1
                        ));
                    }
                }
                if let Some(bad_type) = check_line_for_metamodel_typing_violation(line) {
                    violations.push(format!(
                        "Check 19 violation (domain-metamodel-typing-violation): Upstream template '{rel}:{}' declares unvalidated M1 domain instance entity/type '{bad_type}'. Upstream templates must adhere strictly to the closed M2 metamodel allowlist.",
                        idx + 1
                    ));
                }
            }
        }
        Err(err) => violations.push(format!("Check 19 violation: Failed to read '{rel}': {err}")),
    }
}

#[cfg(test)]
#[path = "domain_cleanliness_tests.rs"]
mod tests;
