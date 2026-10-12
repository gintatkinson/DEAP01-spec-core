//! Repository topology and workspace inspection utilities.

use crate::rules::{EXCLUDED_DIRS, MASTER_BLUEPRINTS};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Check if repository is the upstream specification core compiler repository.
pub fn is_upstream_compiler(repo_root: &Path) -> bool {
    repo_root.join(".pipeline").join("upstream").is_dir()
}

/// Check 10: Verify .gitignore exists in the repository root.
pub fn check_gitignore_exists(repo_root: &Path) -> Result<(), String> {
    let gitignore = repo_root.join(".gitignore");
    if gitignore.is_file() {
        Ok(())
    } else {
        Err(format!(
            "Check 10 failed: .gitignore missing in repository root '{}'.",
            repo_root.display()
        ))
    }
}

/// Check 11: Verify zero .DS_Store files exist in the working tree.
pub fn check_no_ds_store_files(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut ds_store_files = Vec::new();

    for e in WalkDir::new(repo_root)
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
        if e.file_name() == ".DS_Store" {
            let rel = e
                .path()
                .strip_prefix(repo_root)
                .unwrap_or(e.path())
                .display()
                .to_string();
            ds_store_files.push(rel);
        }
    }

    if ds_store_files.is_empty() {
        Ok(())
    } else {
        let msg = format!(
            "Check 11 failed: Found {} .DS_Store file(s): {}",
            ds_store_files.len(),
            ds_store_files.join(", ")
        );
        Err(vec![msg])
    }
}

/// Check 12: Verify downstream repositories do not contain duplicate master core blueprints.
pub fn check_no_duplicate_master_blueprints(repo_root: &Path) -> Result<(), Vec<String>> {
    if is_upstream_compiler(repo_root) {
        return Ok(());
    }

    let mut duplicates = Vec::new();
    let blueprints: Vec<&str> = MASTER_BLUEPRINTS
        .iter()
        .copied()
        .chain(std::iter::once("DEAP_SYSML_V2_SAFETY_MODEL_SPECIFICATION.sysml"))
        .collect();

    for e in WalkDir::new(repo_root)
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
        if e.file_type().is_file() {
            let fname = e.file_name().to_string_lossy();
            if blueprints.iter().any(|&bp| bp == fname) {
                let rel = e
                    .path()
                    .strip_prefix(repo_root)
                    .unwrap_or(e.path())
                    .display()
                    .to_string();
                duplicates.push(rel);
            }
        }
    }

    if duplicates.is_empty() {
        Ok(())
    } else {
        let msg = format!(
            "Check 12 failed: Downstream repository contains duplicate master core blueprint file(s): {}",
            duplicates.join(", ")
        );
        Err(vec![msg])
    }
}

/// Check 14: Verify presence of README.md, agent instruction entrypoints, and rules/sysml-ssot-completeness.md.
pub fn check_downstream_instructions_exist(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // 1. README.md
    let readme_path = repo_root.join("README.md");
    if !readme_path.is_file() {
        errors.push(format!(
            "Check 14 failed: README.md missing in repository root '{}'.",
            repo_root.display()
        ));
    } else {
        match fs::read_to_string(&readme_path) {
            Ok(content) => {
                if content.trim().is_empty() {
                    errors.push(format!(
                        "Check 14 failed: README.md is empty in repository root '{}'.",
                        repo_root.display()
                    ));
                } else if !content.contains("# Downstream Cyber-Physical Infrastructure Safety Project")
                    && !content.contains("Operator Prompt Catalog")
                {
                    errors.push(format!(
                        "Check 14 failed: README.md in '{}' lacks canonical downstream content (missing '# Downstream Cyber-Physical Infrastructure Safety Project' or 'Operator Prompt Catalog').",
                        repo_root.display()
                    ));
                }
            }
            Err(e) => {
                errors.push(format!(
                    "Check 14 failed: Unable to read README.md: {}",
                    e
                ));
            }
        }
    }

    // 2. Agent instruction entrypoint
    let agent_entrypoints = [
        repo_root.join("AGENTS.md"),
        repo_root.join("CLAUDE.md"),
        repo_root.join(".agents").join("AGENTS.md"),
    ];
    let valid_entrypoint = agent_entrypoints.iter().any(|p| {
        p.is_file() && fs::metadata(p).map(|m| m.len() > 0).unwrap_or(false)
    });
    if !valid_entrypoint {
        errors.push(format!(
            "Check 14 failed: No non-empty agent instruction entrypoint found in '{}' (expected AGENTS.md, CLAUDE.md, or .agents/AGENTS.md).",
            repo_root.display()
        ));
    }

    // 3. rules/sysml-ssot-completeness.md
    let sysml_rule_path = repo_root.join("rules").join("sysml-ssot-completeness.md");
    if !sysml_rule_path.is_file() {
        errors.push(format!(
            "Check 14 failed: rules/sysml-ssot-completeness.md missing in repository root '{}'.",
            repo_root.display()
        ));
    } else {
        let size = fs::metadata(&sysml_rule_path).map(|m| m.len()).unwrap_or(0);
        if size == 0 {
            errors.push(format!(
                "Check 14 failed: rules/sysml-ssot-completeness.md is empty in repository root '{}'.",
                repo_root.display()
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check 15: Verify reconcile-backlog tooling exists and is executable/buildable.
///
/// Supports:
/// 1. Native compiled binary: `target/release/reconcile-backlog`
/// 2. Crate source directory: `crates/reconcile-backlog` (with Cargo.toml and src/main.rs)
pub fn check_reconcile_backlog_tooling_exists(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    let release_bin = repo_root.join("target").join("release").join("reconcile-backlog");
    let crate_dir = repo_root.join("crates").join("reconcile-backlog");
    let crate_cargo = crate_dir.join("Cargo.toml");
    let crate_src = crate_dir.join("src").join("main.rs");

    let has_release_bin = release_bin.is_file() && fs::metadata(&release_bin).map(|m| m.len() > 0).unwrap_or(false);
    let has_crate = crate_cargo.is_file() && crate_src.is_file();

    if !has_release_bin && !has_crate {
        errors.push(format!(
            "Check 15 failed: Reconcile backlog tooling missing in repository root '{}' (expected target/release/reconcile-backlog or crates/reconcile-backlog).",
            repo_root.display()
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check 16: Upstream Template Clean Landing Zone Gate.
pub fn check_upstream_template_clean_landing_zones(repo_root: &Path) -> Result<(), Vec<String>> {
    if !is_upstream_compiler(repo_root) {
        return Ok(());
    }

    let landing_zones = [
        "docs/conops",
        "docs/safety",
        "docs/epics",
        "docs/features",
        "docs/user-stories",
        "docs/use-cases",
        "docs/management",
        "schema",
    ];
    let allowed_files = [".gitkeep", "README.md"];
    let mut violations = Vec::new();

    for zone in &landing_zones {
        let zone_path = repo_root.join(zone);
        if !zone_path.is_dir() {
            continue;
        }

        for e in WalkDir::new(&zone_path)
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
            if e.file_type().is_file() {
                let fname = e.file_name().to_string_lossy();
                if !allowed_files.contains(&fname.as_ref()) {
                    let rel = e
                        .path()
                        .strip_prefix(repo_root)
                        .unwrap_or(e.path())
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
        let msg = format!(
            "Check 16 failed: Upstream distribution template landing zones contain concrete specification files: {}",
            violations.join(", ")
        );
        Err(vec![msg])
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
            let path = std::env::temp_dir().join(format!("deap_test_{}_{}", name, uuid::Uuid::new_v4()));
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
    fn test_is_upstream_compiler() {
        let tmp = TempDir::new("is_upstream");
        assert!(!is_upstream_compiler(&tmp.path));
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        assert!(is_upstream_compiler(&tmp.path));
    }

    #[test]
    fn test_check_gitignore() {
        let tmp = TempDir::new("gitignore");
        assert!(check_gitignore_exists(&tmp.path).is_err());
        File::create(tmp.path.join(".gitignore")).unwrap();
        assert!(check_gitignore_exists(&tmp.path).is_ok());
    }

    #[test]
    fn test_check_no_ds_store() {
        let tmp = TempDir::new("ds_store");
        assert!(check_no_ds_store_files(&tmp.path).is_ok());
        File::create(tmp.path.join(".DS_Store")).unwrap();
        let res = check_no_ds_store_files(&tmp.path);
        assert!(res.is_err());
        assert!(res.unwrap_err()[0].contains(".DS_Store"));
    }

    #[test]
    fn test_check_duplicate_blueprints() {
        let tmp = TempDir::new("blueprints");
        assert!(check_no_duplicate_master_blueprints(&tmp.path).is_ok());
        File::create(tmp.path.join("DEAP_MASTER_ARCHITECTURE.md")).unwrap();
        let res = check_no_duplicate_master_blueprints(&tmp.path);
        assert!(res.is_err());

        // Upstream repo exempt
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        assert!(check_no_duplicate_master_blueprints(&tmp.path).is_ok());
    }

    #[test]
    fn test_check_downstream_instructions() {
        let tmp = TempDir::new("instructions");
        assert!(check_downstream_instructions_exist(&tmp.path).is_err());

        // Create README with required phrase
        let mut readme = File::create(tmp.path.join("README.md")).unwrap();
        writeln!(readme, "# Downstream Cyber-Physical Infrastructure Safety Project\n\nDetails.").unwrap();

        // Create AGENTS.md
        let mut agents = File::create(tmp.path.join("AGENTS.md")).unwrap();
        writeln!(agents, "# Agents").unwrap();

        // Create rules/sysml-ssot-completeness.md
        fs::create_dir_all(tmp.path.join("rules")).unwrap();
        let mut sysml = File::create(tmp.path.join("rules/sysml-ssot-completeness.md")).unwrap();
        writeln!(sysml, "# SysML SSOT Completeness").unwrap();

        assert!(check_downstream_instructions_exist(&tmp.path).is_ok());
    }

    #[test]
    fn test_check_reconcile_backlog_tooling() {
        let tmp = TempDir::new("reconcile");
        assert!(check_reconcile_backlog_tooling_exists(&tmp.path).is_err());

        fs::create_dir_all(tmp.path.join("target/release")).unwrap();
        let bin_path = tmp.path.join("target/release/reconcile-backlog");
        let mut f = File::create(&bin_path).unwrap();
        writeln!(f, "#!/bin/sh\necho reconcile").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&bin_path, fs::Permissions::from_mode(0o755)).unwrap();
        }

        assert!(check_reconcile_backlog_tooling_exists(&tmp.path).is_ok());
    }

    #[test]
    fn test_check_reconcile_backlog_tooling_rust_native() {
        let tmp = TempDir::new("reconcile_rust");
        // Empty workspace must fail
        assert!(check_reconcile_backlog_tooling_exists(&tmp.path).is_err());

        // Case A: crates/reconcile-backlog
        let crate_dir = tmp.path.join("crates/reconcile-backlog/src");
        fs::create_dir_all(&crate_dir).unwrap();
        File::create(tmp.path.join("crates/reconcile-backlog/Cargo.toml")).unwrap();
        File::create(tmp.path.join("crates/reconcile-backlog/src/main.rs")).unwrap();
        assert!(check_reconcile_backlog_tooling_exists(&tmp.path).is_ok());

        // Case B: target/release/reconcile-backlog
        let tmp_bin = TempDir::new("reconcile_bin");
        fs::create_dir_all(tmp_bin.path.join("target/release")).unwrap();
        let bin_path = tmp_bin.path.join("target/release/reconcile-backlog");
        let mut f = File::create(&bin_path).unwrap();
        writeln!(f, "binary content").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&bin_path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        assert!(check_reconcile_backlog_tooling_exists(&tmp_bin.path).is_ok());
    }

    #[test]
    fn test_upstream_clean_landing_zones() {
        let tmp = TempDir::new("clean_zones");
        // Downstream should be exempt
        assert!(check_upstream_template_clean_landing_zones(&tmp.path).is_ok());

        // Upstream with clean landing zone
        fs::create_dir_all(tmp.path.join(".pipeline/upstream")).unwrap();
        fs::create_dir_all(tmp.path.join("schema")).unwrap();
        File::create(tmp.path.join("schema/.gitkeep")).unwrap();
        assert!(check_upstream_template_clean_landing_zones(&tmp.path).is_ok());

        // Adding concrete spec file causes violation
        File::create(tmp.path.join("schema/domain_model.sysml")).unwrap();
        let res = check_upstream_template_clean_landing_zones(&tmp.path);
        assert!(res.is_err());
    }

    #[test]
    fn test_current_repo_baseline_checks() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("repo root");

        assert_eq!(is_upstream_compiler(repo_root), false);
        assert!(check_gitignore_exists(repo_root).is_ok());
        assert!(check_no_ds_store_files(repo_root).is_ok());
        assert!(check_no_duplicate_master_blueprints(repo_root).is_ok());
        assert!(check_downstream_instructions_exist(repo_root).is_ok());
        assert!(check_reconcile_backlog_tooling_exists(repo_root).is_ok());
        assert!(check_upstream_template_clean_landing_zones(repo_root).is_ok());
    }
}
