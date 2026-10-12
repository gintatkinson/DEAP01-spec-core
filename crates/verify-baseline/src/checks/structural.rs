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
/// Helper checking whether a file name matches forbidden upstream blueprint domain concepts.
fn is_forbidden_blueprint_name(fname: &str) -> bool {
    let lower = fname.to_lowercase();
    if lower.ends_with(".sysml") {
        return true;
    }
    const FORBIDDEN_PATTERNS: &[&str] = &[
        "flightsystems",
        "flight-systems",
        "flight_systems",
        "uasinfrastructure",
        "uas-infrastructure",
        "uas_infrastructure",
        "frontendsystems",
        "frontend-systems",
        "frontend_systems",
        "safetymodel",
        "safety-model",
        "safety_model",
        "conceptpaper",
        "concept-paper",
        "concept_paper",
    ];
    FORBIDDEN_PATTERNS.iter().any(|&p| lower.contains(p))
}

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
            if is_forbidden_blueprint_name(&fname) {
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

/// Helper checking if a line is a static hardcoded parameter assignment in python tools.
fn is_static_param_assignment(line: &str) -> bool {
    const FORBIDDEN_KEYWORDS: &[&str] = &[
        "GROUND_TRUTH",
        "EXPECTED_SPECS",
        "DOMAIN_SPECS",
        "DOMAIN_PARAMS",
        "SYSML_PARAMS",
        "MANDATED_SPECS",
        "PHYSICAL_LIMITS",
        "HARDWARE_REGISTRY",
    ];
    for &kw in FORBIDDEN_KEYWORDS {
        if let Some(rest) = line.strip_prefix(kw) {
            let after_ws = rest.trim_start();
            if after_ws.starts_with('=') {
                return true;
            }
        }
    }
    false
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
                            if is_static_param_assignment(trimmed) {
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

/// Check 33: Zero Python Scripts Invariant
///
/// Recursively walks the repository filesystem from root using WalkDir,
/// skipping `target` and `.git` directories.
/// Fails closed if ANY file with extension `.py` (case-insensitive) is detected.
pub fn check_zero_python_scripts(repo_root: &Path) -> Result<(), Vec<String>> {
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
            let fname = entry.file_name().to_string_lossy();
            let is_py = fname.to_ascii_lowercase().ends_with(".py")
                || entry
                    .path()
                    .extension()
                    .map(|ext| ext.to_string_lossy().eq_ignore_ascii_case("py"))
                    .unwrap_or(false);
            if is_py {
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
        violations.sort();
        Err(violations)
    }
}

/// Check 34: Zero Regex Invariant
///
/// Scans all `.rs` and `Cargo.toml` files in `crates/` and root `Cargo.toml`.
/// Fails closed if any line contains forbidden pattern tokens
/// or Cargo.toml dependency on the regex crate.
///
/// Note: To prevent self-triggering false positives on scanner source code,
/// forbidden tokens are dynamically synthesized without raw literal patterns.
pub fn check_zero_regex(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();

    // Dynamically assemble forbidden pattern needles to ensure scanner source
    // does not match itself as a false positive.
    let pat_regex_colons = ["Reg", "ex::"].concat();
    let pat_inline_regex = ["reg", "ex::"].concat();
    let pat_use_bare = ["use", " ", "reg", "ex;"].concat();
    let pat_extern = ["extern", " crate ", "reg", "ex;"].concat();
    let crate_needle = ["reg", "ex"].concat();

    // 1. Check root Cargo.toml if it exists
    let root_cargo = repo_root.join("Cargo.toml");
    if root_cargo.is_file() {
        scan_cargo_toml_file(&root_cargo, repo_root, &crate_needle, &mut violations);
    }

    // 2. Scan crates/ directory for all .rs and Cargo.toml files
    let crates_dir = repo_root.join("crates");
    if crates_dir.is_dir() {
        for entry in WalkDir::new(&crates_dir)
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
                let path = entry.path();
                let fname = entry.file_name().to_string_lossy();
                if fname == "Cargo.toml" {
                    scan_cargo_toml_file(path, repo_root, &crate_needle, &mut violations);
                } else if path.extension().map(|ext| ext == "rs").unwrap_or(false) {
                    scan_rs_file(
                        path,
                        repo_root,
                        &pat_regex_colons,
                        &pat_inline_regex,
                        &pat_use_bare,
                        &pat_extern,
                        &mut violations,
                    );
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

fn scan_cargo_toml_file(
    path: &Path,
    repo_root: &Path,
    crate_needle: &str,
    violations: &mut Vec<String>,
) {
    if let Ok(content) = fs::read_to_string(path) {
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(path)
            .display()
            .to_string();

        for (idx, line) in content.lines().enumerate() {
            let code = if let Some((before, _)) = line.split_once('#') {
                before
            } else {
                line
            };
            let trimmed = code.trim();
            if trimmed.is_empty() {
                continue;
            }

            let mut is_dep = false;
            if let Some((dep_key, _)) = trimmed.split_once('=') {
                let key = dep_key.trim().trim_matches('"').trim_matches('\'');
                if key == crate_needle || key.starts_with(&format!("{crate_needle}.")) {
                    is_dep = true;
                }
            } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section = trimmed[1..trimmed.len() - 1].trim();
                if section
                    .split('.')
                    .any(|part| part.trim().trim_matches('"').trim_matches('\'') == crate_needle)
                {
                    is_dep = true;
                }
            } else if trimmed.contains(&format!("\"{crate_needle}\""))
                || trimmed.contains(&format!("'{crate_needle}'"))
            {
                is_dep = true;
            }

            if is_dep {
                violations.push(format!("{}:{}: {}", rel, idx + 1, line.trim()));
            }
        }
    }
}

fn scan_rs_file(
    path: &Path,
    repo_root: &Path,
    pat_regex_colons: &str,
    pat_inline_regex: &str,
    pat_use_bare: &str,
    pat_extern: &str,
    violations: &mut Vec<String>,
) {
    if let Ok(content) = fs::read_to_string(path) {
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(path)
            .display()
            .to_string();

        for (idx, line) in content.lines().enumerate() {
            let code = if let Some((before, _)) = line.split_once("//") {
                before
            } else {
                line
            };
            let trimmed = code.trim();
            if trimmed.is_empty() || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                continue;
            }

            if trimmed.contains(pat_regex_colons)
                || trimmed.contains(pat_inline_regex)
                || trimmed.contains(pat_use_bare)
                || trimmed.contains(pat_extern)
            {
                violations.push(format!("{}:{}: {}", rel, idx + 1, line.trim()));
            }
        }
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

    #[test]
    fn test_check_33_zero_python_clean() {
        let tmp = TempDir::new("c33_clean");
        let rs_file = tmp.path.join("main.rs");
        File::create(&rs_file).unwrap();
        assert!(check_zero_python_scripts(&tmp.path).is_ok());
    }

    #[test]
    fn test_check_33_zero_python_violation() {
        let tmp = TempDir::new("c33_violation");
        let py_file = tmp.path.join("script.py");
        File::create(&py_file).unwrap();
        let res = check_zero_python_scripts(&tmp.path);
        assert!(res.is_err());
        let errs = res.unwrap_err();
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0], "script.py");
    }

    #[test]
    fn test_check_34_zero_regex_clean() {
        let tmp = TempDir::new("c34_clean");
        let crates_dir = tmp.path.join("crates").join("sample").join("src");
        fs::create_dir_all(&crates_dir).unwrap();
        let rs_file = crates_dir.join("lib.rs");
        let mut f = File::create(&rs_file).unwrap();
        writeln!(f, "pub fn add(a: i32, b: i32) -> i32 {{ a + b }}").unwrap();

        let cargo_file = tmp.path.join("crates").join("sample").join("Cargo.toml");
        let mut fc = File::create(&cargo_file).unwrap();
        writeln!(fc, "[package]\nname = \"sample\"\nversion = \"0.1.0\"").unwrap();

        assert!(check_zero_regex(&tmp.path).is_ok());
    }

    #[test]
    fn test_check_34_zero_regex_violation_rs() {
        let tmp = TempDir::new("c34_viol_rs");
        let crates_dir = tmp.path.join("crates").join("sample").join("src");
        fs::create_dir_all(&crates_dir).unwrap();
        let rs_file = crates_dir.join("lib.rs");
        let mut f = File::create(&rs_file).unwrap();
        // Construct forbidden pattern dynamically to avoid literal in this file
        let pat = ["Reg", "ex::"].concat();
        writeln!(f, "let _ = {}new(\"\");", pat).unwrap();

        let res = check_zero_regex(&tmp.path);
        assert!(res.is_err());
        let errs = res.unwrap_err();
        assert_eq!(errs.len(), 1);
        assert!(errs[0].contains("crates/sample/src/lib.rs:1:"));
    }

    #[test]
    fn test_check_34_zero_regex_violation_cargo() {
        let tmp = TempDir::new("c34_viol_cargo");
        let crates_dir = tmp.path.join("crates").join("sample");
        fs::create_dir_all(&crates_dir).unwrap();
        let cargo_file = crates_dir.join("Cargo.toml");
        let mut fc = File::create(&cargo_file).unwrap();
        // Construct forbidden dependency dynamically
        let dep = ["reg", "ex"].concat();
        writeln!(fc, "[dependencies]\n{} = \"1.10\"", dep).unwrap();

        let res = check_zero_regex(&tmp.path);
        assert!(res.is_err());
        let errs = res.unwrap_err();
        assert_eq!(errs.len(), 1);
        assert!(errs[0].contains("crates/sample/Cargo.toml:2:"));
    }
}
