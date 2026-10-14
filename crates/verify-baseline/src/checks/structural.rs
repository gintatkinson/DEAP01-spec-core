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

/// Check 18: Upstream Template Clean Landing Zone & Blueprint Cleanliness Gate.
///
/// In upstream distribution template repositories (`DEAP-*`), enforces:
/// 1. Pure schema landing zone cleanliness: `schema/`, `docs/epics/`, `docs/features/`,
///    `docs/user-stories/`, and `docs/use-cases/` must contain ONLY `.gitkeep` files.
/// 2. Zero concrete modeling files (`.sysml`) in upstream architecture blueprints (`docs/architecture/blueprints`).
/// Downstream repositories are exempt.
pub fn verify_upstream_blueprint_domain_cleanliness(repo_root: &Path) -> Result<(), Vec<String>> {
    if !is_upstream_compiler(repo_root) {
        return Ok(());
    }

    let mut violations = Vec::new();

    // 1. Enforce pure schema landing zone cleanliness (only .gitkeep)
    const LANDING_ZONES: &[&str] = &[
        "schema",
        "docs/epics",
        "docs/features",
        "docs/user-stories",
        "docs/use-cases",
    ];

    for zone in LANDING_ZONES {
        let zone_dir = repo_root.join(zone);
        if !zone_dir.is_dir() {
            continue;
        }

        for entry in WalkDir::new(&zone_dir)
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
                if fname != ".gitkeep" {
                    let rel = entry
                        .path()
                        .strip_prefix(repo_root)
                        .unwrap_or(entry.path())
                        .display()
                        .to_string();
                    violations.push(format!(
                        "Upstream template landing zone '{}' contains forbidden non-.gitkeep file: {}",
                        zone, rel
                    ));
                }
            }
        }
    }

    // 2. Enforce zero concrete modeling files (.sysml) in upstream blueprints
    let blueprints_dir = repo_root.join("docs").join("architecture").join("blueprints");
    if blueprints_dir.is_dir() {
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
                if fname.to_lowercase().ends_with(".sysml") {
                    let rel = entry
                        .path()
                        .strip_prefix(repo_root)
                        .unwrap_or(entry.path())
                        .display()
                        .to_string();
                    violations.push(format!("Upstream blueprint contains SysML model: {}", rel));
                }
            }
        }
    }

    // 3. Enforce zero MASTER_HANDOVER_CHECKPOINT.md at repository root or docs (refs #321)
    if let Err(errs) = check_no_master_handover_checkpoint(repo_root) {
        violations.extend(errs);
    }

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        let msg = format!(
            "Check 18 failed: Upstream cleanliness violations found: {}",
            violations.join("; ")
        );
        Err(vec![msg])
    }
}

pub use super::domain_cleanliness_gate::check_domain_agnostic_ast_cleanliness;

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

/// Check documentation and backlog integrity (refs #318):
/// 1. Root `HANDOFF.md` must not exist in repository root (purged in commit 69c749c to eradicate documentation drift and stale backlog hallucinations).
/// 2. If `docs/designs/feat-289-290-293-solution.md` is present, verify:
///    - It cites Issue #318.
///    - It correctly attributes commit 402d56c to refs #315 (not prompt quarantine).
///    - It does not list issues #289, #290, #293 as unstarted/remaining work packages.
///    - It does not cite invalid path `scripts/bootstrap_tracker_labels.py` without package scoping.
///    - It contains zero Unicode em dashes.
pub fn check_handoff_and_backlog_integrity(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();

    // 1. Root HANDOFF.md must not exist (prevent documentation drift and stale backlog hallucinations)
    let root_handoff = repo_root.join("HANDOFF.md");
    if root_handoff.exists() {
        violations.push(
            "Root HANDOFF.md detected: root handoff artifacts are forbidden per commit 69c749c to prevent documentation drift and stale backlog hallucinations."
                .to_string(),
        );
    }

    // 2. Validate docs/designs/feat-289-290-293-solution.md if present
    let solution_doc = repo_root.join("docs").join("designs").join("feat-289-290-293-solution.md");
    if solution_doc.is_file() {
        if let Ok(content) = fs::read_to_string(&solution_doc) {
            if !content.contains("318") {
                violations.push(
                    "feat-289-290-293-solution.md must cite Issue #318 resolving stale backlog hallucinations."
                        .to_string(),
                );
            }
            if content.contains("prompt quarantine across all skills") {
                violations.push(
                    "feat-289-290-293-solution.md contains stale attribution error: 402d56c delivered commit message non-closure rule (refs #315), not prompt quarantine."
                        .to_string(),
                );
            }
            if content.contains("Work Package B: Issue #293")
                || content.contains("Work Package C: Issue #289")
                || content.contains("Work Package D: Issue #290")
            {
                violations.push(
                    "feat-289-290-293-solution.md lists resolved issues #289, #290, or #293 as outstanding work packages."
                        .to_string(),
                );
            }
            if content.contains("scripts/bootstrap_tracker_labels.py")
                && !content.contains("skills/spec-orchestrator/scripts/bootstrap_tracker_labels.py")
            {
                violations.push(
                    "feat-289-290-293-solution.md contains invalid path scripts/bootstrap_tracker_labels.py."
                        .to_string(),
                );
            }
            if content.contains('\u{2014}') {
                violations.push(
                    "feat-289-290-293-solution.md contains forbidden Unicode em dash (\\u2014)."
                        .to_string(),
                );
            }
        }
    }

    // 3. Ephemeral MASTER_HANDOVER_CHECKPOINT.md must not exist at root or docs (refs #321)
    if let Err(errs) = check_no_master_handover_checkpoint(repo_root) {
        violations.extend(errs);
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Check documentation and root artifact cleanliness (refs #321):
/// Verifies that ephemeral/spurious handover checkpoints (`MASTER_HANDOVER_CHECKPOINT.md`)
/// are strictly forbidden at the repository root and across `docs/`.
pub fn check_no_master_handover_checkpoint(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();

    // 1. Root check: MASTER_HANDOVER_CHECKPOINT.md or variants
    let root_checkpoint = repo_root.join("MASTER_HANDOVER_CHECKPOINT.md");
    if root_checkpoint.exists() {
        violations.push(
            "Root MASTER_HANDOVER_CHECKPOINT.md detected: ephemeral/spurious handover checkpoint files containing hardcoded domain specifications are strictly forbidden at repository root (refs #321)."
                .to_string(),
        );
    }

    if let Ok(entries) = fs::read_dir(repo_root) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname != "MASTER_HANDOVER_CHECKPOINT.md"
                    && fname.to_ascii_uppercase().starts_with("MASTER_HANDOVER_CHECKPOINT")
                {
                    violations.push(format!(
                        "Root checkpoint variant detected: '{}' is strictly forbidden (refs #321).",
                        fname
                    ));
                }
            }
        }
    }

    // 2. docs/ directory check: forbidden anywhere inside docs/
    let docs_dir = repo_root.join("docs");
    if docs_dir.is_dir() {
        for entry in WalkDir::new(&docs_dir)
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
                if fname.to_ascii_uppercase() == "MASTER_HANDOVER_CHECKPOINT.MD"
                    || fname.to_ascii_uppercase().starts_with("MASTER_HANDOVER_CHECKPOINT")
                {
                    let rel = entry
                        .path()
                        .strip_prefix(repo_root)
                        .unwrap_or(entry.path())
                        .display()
                        .to_string();
                    violations.push(format!(
                        "Forbidden handover checkpoint file detected at '{}': ephemeral/spurious handover checkpoints containing domain specifications are strictly forbidden (refs #321).",
                        rel
                    ));
                }
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        violations.dedup();
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
    fn test_check_18_upstream_clean() {
        let tmp = TempDir::new("c18_clean");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        let schema_dir = tmp.path.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        File::create(schema_dir.join(".gitkeep")).unwrap();

        let bp_dir = tmp.path.join("docs/architecture/blueprints");
        fs::create_dir_all(&bp_dir).unwrap();
        File::create(bp_dir.join("CORE_ARCHITECTURE.md")).unwrap();
        // Domain-agnostic markdown blueprint that formerly failed fragile blacklist must now pass
        File::create(bp_dir.join("FRONTEND_SYSTEMS_PATTERN.md")).unwrap();

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
    fn test_check_18_landing_zone_non_gitkeep_fails() {
        let tmp = TempDir::new("c18_lz");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        let schema_dir = tmp.path.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        File::create(schema_dir.join("domain_model.sysml")).unwrap();
        assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_err());
    }

    #[test]
    fn test_check_19_downstream_exempt() {
        let tmp = TempDir::new("c19_downstream");
        assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_ok());
    }

    #[test]
    fn test_check_19_upstream_clean() {
        let tmp = TempDir::new("c19_clean");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        let src_dir = tmp.path.join("crates/test-crate/src");
        fs::create_dir_all(&src_dir).unwrap();
        let mut f = File::create(src_dir.join("lib.rs")).unwrap();
        writeln!(f, "pub fn compute_hash(val: &[u8]) -> u64 {{ 42 }}").unwrap();
        assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_ok());
    }

    #[test]
    fn test_check_19_rust_crates_domain_params_fails() {
        let tmp = TempDir::new("c19_rust");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        let src_dir = tmp.path.join("crates/test-crate/src");
        fs::create_dir_all(&src_dir).unwrap();
        let mut f = File::create(src_dir.join("lib.rs")).unwrap();
        let kw = ["DOM", "AIN_PARAMS"].concat();
        writeln!(f, "pub const {}: &[&str] = &[\"speed\"];", kw).unwrap();
        assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_err());
    }

    #[test]
    fn test_reproduce_defect_265_unflagged_m1_domain_dict_bypasses_check_19() {
        let tmp = TempDir::new("c19_m1_bypass");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        let src_dir = tmp.path.join("crates/test-crate/src");
        fs::create_dir_all(&src_dir).unwrap();
        let mut f = File::create(src_dir.join("lib.rs")).unwrap();
        writeln!(f, "pub const FLIGHT_ENVELOPE_MAP: &[(&str, f64)] = &[(\"max_altitude\", 10000.0)];").unwrap();
        // Under unfixed Check 19, this fails because check_domain_agnostic_ast_cleanliness incorrectly returns Ok(())
        assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_err(), "Check 19 failed to reject unflagged M1 domain dict FLIGHT_ENVELOPE_MAP");
    }

    #[test]
    fn test_check_19_legacy_python_parity_auditor_dir_fails() {
        let tmp = TempDir::new("c19_legacy");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        let crates_dir = tmp.path.join("crates");
        fs::create_dir_all(&crates_dir).unwrap();
        let legacy_dir = tmp
            .path
            .join("skills/spec-orchestrator/parity_auditor");
        fs::create_dir_all(&legacy_dir).unwrap();

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

    #[test]
    fn test_check_34_repo_root_passes() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("repo root");
        let res = check_zero_regex(repo_root);
        assert!(res.is_ok(), "Check 34 Zero Regex Invariant failed on repository root: {:?}", res.err());
    }

    #[test]
    fn test_reproduce_defect_318_stale_backlog_hallucinations_and_attribution_errors() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("repo root");
        let res = check_handoff_and_backlog_integrity(repo_root);
        assert!(
            res.is_ok(),
            "Defect #318 integrity check failed on repository root: {:?}",
            res.err()
        );
    }

    #[test]
    fn test_defect_318_mutation_rejects_stale_root_handoff() {
        let tmp = TempDir::new("c318_root_handoff");
        let handoff = tmp.path.join("HANDOFF.md");
        File::create(&handoff).unwrap();
        let res = check_handoff_and_backlog_integrity(&tmp.path);
        assert!(res.is_err(), "Must reject existence of root HANDOFF.md");
    }

    #[test]
    fn test_defect_318_mutation_rejects_stale_attribution_error() {
        let tmp = TempDir::new("c318_stale_attr");
        let designs = tmp.path.join("docs").join("designs");
        fs::create_dir_all(&designs).unwrap();
        let sol = designs.join("feat-289-290-293-solution.md");
        let mut f = File::create(&sol).unwrap();
        writeln!(
            f,
            "# Solution (refs #318)\nprompt quarantine across all skills"
        )
        .unwrap();
        let res = check_handoff_and_backlog_integrity(&tmp.path);
        assert!(res.is_err(), "Must reject stale attribution error");
    }

    #[test]
    fn test_defect_318_mutation_rejects_stale_work_package_queue() {
        let tmp = TempDir::new("c318_stale_queue");
        let designs = tmp.path.join("docs").join("designs");
        fs::create_dir_all(&designs).unwrap();
        let sol = designs.join("feat-289-290-293-solution.md");
        let mut f = File::create(&sol).unwrap();
        writeln!(f, "# Solution (refs #318)\nWork Package B: Issue #293").unwrap();
        let res = check_handoff_and_backlog_integrity(&tmp.path);
        assert!(
            res.is_err(),
            "Must reject listing resolved issues as outstanding work packages"
        );
    }

    #[test]
    fn test_reproduce_defect_321_root_master_handover_checkpoint_rejected() {
        let tmp = TempDir::new("c321_root_rejection");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        let schema_dir = tmp.path.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        File::create(schema_dir.join(".gitkeep")).unwrap();

        let checkpoint = tmp.path.join("MASTER_HANDOVER_CHECKPOINT.md");
        File::create(&checkpoint).unwrap();

        let res_no_checkpoint = check_no_master_handover_checkpoint(&tmp.path);
        assert!(res_no_checkpoint.is_err(), "Must reject root MASTER_HANDOVER_CHECKPOINT.md");

        let res_upstream = verify_upstream_blueprint_domain_cleanliness(&tmp.path);
        assert!(res_upstream.is_err(), "Upstream cleanliness must reject root MASTER_HANDOVER_CHECKPOINT.md");

        let res_handoff = check_handoff_and_backlog_integrity(&tmp.path);
        assert!(res_handoff.is_err(), "Handoff integrity must reject root MASTER_HANDOVER_CHECKPOINT.md");
    }

    #[test]
    fn test_defect_321_docs_master_handover_checkpoint_rejected() {
        let tmp = TempDir::new("c321_docs_rejection");
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        let schema_dir = tmp.path.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        File::create(schema_dir.join(".gitkeep")).unwrap();

        let docs_dir = tmp.path.join("docs").join("architecture");
        fs::create_dir_all(&docs_dir).unwrap();
        let checkpoint = docs_dir.join("MASTER_HANDOVER_CHECKPOINT.md");
        File::create(&checkpoint).unwrap();

        let res_no_checkpoint = check_no_master_handover_checkpoint(&tmp.path);
        assert!(res_no_checkpoint.is_err(), "Must reject docs/architecture/MASTER_HANDOVER_CHECKPOINT.md");

        let res_upstream = verify_upstream_blueprint_domain_cleanliness(&tmp.path);
        assert!(res_upstream.is_err(), "Upstream cleanliness must reject docs MASTER_HANDOVER_CHECKPOINT.md");

        let res_handoff = check_handoff_and_backlog_integrity(&tmp.path);
        assert!(res_handoff.is_err(), "Handoff integrity must reject docs MASTER_HANDOVER_CHECKPOINT.md");
    }

    #[test]
    fn test_defect_321_root_checkpoint_variant_rejected() {
        let tmp = TempDir::new("c321_variant_rejection");
        let variant = tmp.path.join("MASTER_HANDOVER_CHECKPOINT_V2.md");
        File::create(&variant).unwrap();

        let res = check_no_master_handover_checkpoint(&tmp.path);
        assert!(res.is_err(), "Must reject root checkpoint variant MASTER_HANDOVER_CHECKPOINT_V2.md");
    }

    #[test]
    fn test_defect_321_current_repo_passes() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("repo root");

        let res_no_checkpoint = check_no_master_handover_checkpoint(repo_root);
        assert!(
            res_no_checkpoint.is_ok(),
            "Current repo must pass check_no_master_handover_checkpoint: {:?}",
            res_no_checkpoint.err()
        );

        let res_upstream = verify_upstream_blueprint_domain_cleanliness(repo_root);
        assert!(
            res_upstream.is_ok(),
            "Current repo must pass verify_upstream_blueprint_domain_cleanliness: {:?}",
            res_upstream.err()
        );

        let res_handoff = check_handoff_and_backlog_integrity(repo_root);
        assert!(
            res_handoff.is_ok(),
            "Current repo must pass check_handoff_and_backlog_integrity: {:?}",
            res_handoff.err()
        );
    }
}

