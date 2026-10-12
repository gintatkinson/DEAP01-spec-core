//! Structural repository topology and integrity checks.

use deap_core::rules::EXCLUDED_DIRS;
use deap_core::workspace::{
    check_downstream_instructions_exist as core_check_downstream_instructions_exist,
    check_gitignore_exists as core_check_gitignore_exists,
    check_no_ds_store_files as core_check_no_ds_store_files,
    check_no_duplicate_master_blueprints as core_check_no_duplicate_master_blueprints,
    check_reconcile_backlog_tooling_exists as core_check_reconcile_backlog_tooling_exists,
    check_upstream_template_clean_landing_zones as core_check_upstream_template_clean_landing_zones,
    is_upstream_compiler,
};
use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Check 10: Verify .gitignore exists in repository root.
pub fn check_gitignore_exists(repo_root: &Path) -> Result<(), String> {
    core_check_gitignore_exists(repo_root)
}

/// Check 11: Verify zero .DS_Store files exist in the working tree.
pub fn check_no_ds_store_files(repo_root: &Path) -> Result<(), Vec<String>> {
    core_check_no_ds_store_files(repo_root)
}

/// Check 12: Verify downstream repositories do not contain duplicate master core blueprints.
pub fn check_no_duplicate_master_blueprints(repo_root: &Path) -> Result<(), Vec<String>> {
    core_check_no_duplicate_master_blueprints(repo_root)
}

/// Check 14: Verify presence of README.md, agent instruction entrypoints, and rules/sysml-ssot-completeness.md.
pub fn check_downstream_instructions_exist(repo_root: &Path) -> Result<(), Vec<String>> {
    core_check_downstream_instructions_exist(repo_root)
}

/// Check 15: Verify reconcile-backlog tooling exists and is executable/buildable.
pub fn check_reconcile_backlog_tooling_exists(repo_root: &Path) -> Result<(), Vec<String>> {
    core_check_reconcile_backlog_tooling_exists(repo_root)
}

/// Check 16: Upstream Template Clean Landing Zone Gate.
pub fn check_upstream_template_clean_landing_zones(repo_root: &Path) -> Result<(), Vec<String>> {
    core_check_upstream_template_clean_landing_zones(repo_root)
}

/// Check 18: Upstream Blueprint Domain Cleanliness Gate.
///
/// Verifies that upstream architecture blueprints contain zero concrete domain platform
/// concept papers or domain SysML models (e.g. *FLIGHT_SYSTEMS*, *UAS_INFRASTRUCTURE*,
/// *FRONTEND_SYSTEMS*, *SAFETY_MODEL*.sysml). Downstream repositories are exempt.
pub fn verify_upstream_blueprint_domain_cleanliness(repo_root: &Path) -> Result<(), Vec<String>> {
    if !is_upstream_compiler(repo_root) {
        return Ok(());
    }

    let blueprints_dir = repo_root.join("docs").join("architecture").join("blueprints");
    if !blueprints_dir.is_dir() {
        return Ok(());
    }

    let forbidden_patterns = [
        Regex::new(r"(?i)flight[-_]?systems").unwrap(),
        Regex::new(r"(?i)uas[-_]?infrastructure").unwrap(),
        Regex::new(r"(?i)frontend[-_]?systems").unwrap(),
        Regex::new(r"(?i)safety[-_]?model").unwrap(),
        Regex::new(r"(?i)\.sysml$").unwrap(),
        Regex::new(r"(?i)concept[-_]?paper").unwrap(),
    ];

    let mut violations = Vec::new();
    for entry in WalkDir::new(&blueprints_dir)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !EXCLUDED_DIRS.iter().any(|&ex| name == ex)
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let fname = entry.file_name().to_string_lossy();
            if forbidden_patterns.iter().any(|re| re.is_match(&fname)) {
                let rel = entry
                    .path()
                    .strip_prefix(repo_root)
                    .unwrap_or(entry.path())
                    .display()
                    .to_string();
                violations.push(rel);
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        let msg = format!(
            "Check 18 failed: Upstream blueprints contain concrete domain platform concept papers or sysml models: {}",
            violations.join(", ")
        );
        Err(vec![msg])
    }
}

/// Check 19: Domain-Agnostic AST Cleanliness & Closed-Grammar Metamodel Gate.
///
/// Verifies that upstream tools, scripts, and validator modules contain zero static/hardcoded
/// parameter dictionaries (e.g. GROUND_TRUTH = {...}, EXPECTED_SPECS = {...}).
/// Downstream repositories are exempt.
pub fn check_domain_agnostic_ast_cleanliness(repo_root: &Path) -> Result<(), Vec<String>> {
    if !is_upstream_compiler(repo_root) {
        return Ok(());
    }

    let scan_dirs = [
        repo_root
            .join("skills")
            .join("spec-orchestrator")
            .join("parity_auditor")
            .join("src")
            .join("parity_auditor")
            .join("validators"),
        repo_root
            .join("skills")
            .join("spec-orchestrator")
            .join("parity_auditor")
            .join("src")
            .join("parity_auditor")
            .join("core"),
    ];

    let static_param_re = Regex::new(
        r#"^(?:GROUND_TRUTH|EXPECTED_SPECS|DOMAIN_SPECS|DOMAIN_PARAMS|SYSML_PARAMS|MANDATED_SPECS|PHYSICAL_LIMITS|HARDWARE_REGISTRY)\s*="#,
    )
    .unwrap();

    let mut violations = Vec::new();
    for dir in &scan_dirs {
        if !dir.is_dir() {
            continue;
        }

        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    let name = e.file_name().to_string_lossy();
                    !EXCLUDED_DIRS.iter().any(|&ex| name == ex)
                } else {
                    true
                }
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                if path.extension().map(|ext| ext == "py").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(path) {
                        for (idx, line) in content.lines().enumerate() {
                            let trimmed = line.trim();
                            if static_param_re.is_match(trimmed) {
                                let rel = path
                                    .strip_prefix(repo_root)
                                    .unwrap_or(path)
                                    .display()
                                    .to_string();
                                violations.push(format!(
                                    "Check 19 violation: Static hardcoded parameter dictionary declared in {}:{}.",
                                    rel,
                                    idx + 1
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Check 32: Zero Shell Scripts Invariant
///
/// Scans the entire repository tree (excluding `target` and `.git` directories)
/// and fails closed if ANY `.sh` file exists on disk.
pub fn check_zero_shell_scripts(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();

    for entry in WalkDir::new(repo_root)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                name != "target" && name != ".git"
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "sh" {
                    let rel = entry
                        .path()
                        .strip_prefix(repo_root)
                        .unwrap_or(entry.path())
                        .display()
                        .to_string();
                    violations.push(rel);
                }
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        Err(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    struct TempDir {
        path: std::path::PathBuf,
    }

    impl TempDir {
        fn new(name: &str) -> Self {
            static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("verify_structural_{}_{}_{}", name, nanos, count));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn test_check_18_downstream_exempt() {
        let tmp = TempDir::new("c18_downstream");
        assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_ok());
    }

    #[test]
    fn test_check_18_upstream_violations() {
        let tmp = TempDir::new("c18_upstream");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        let bp_dir = tmp.path.join("docs/architecture/blueprints");
        fs::create_dir_all(&bp_dir).unwrap();

        let clean_file = bp_dir.join("CORE_ARCHITECTURE.md");
        File::create(&clean_file).unwrap();
        assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_ok());

        let dirty_file = bp_dir.join("FLIGHT_SYSTEMS_MODEL.sysml");
        File::create(&dirty_file).unwrap();
        assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_err());
    }

    #[test]
    fn test_check_19_downstream_exempt() {
        let tmp = TempDir::new("c19_downstream");
        assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_ok());
    }

    #[test]
    fn test_check_19_upstream_violations() {
        let tmp = TempDir::new("c19_upstream");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        let val_dir = tmp
            .path
            .join("skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators");
        fs::create_dir_all(&val_dir).unwrap();

        let py_file = val_dir.join("test_validator.py");
        let mut f = File::create(&py_file).unwrap();
        writeln!(f, "GROUND_TRUTH = {{'speed': 100}}").unwrap();

        assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_err());
    }
}
