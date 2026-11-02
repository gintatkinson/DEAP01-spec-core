#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Repository topology and workspace inspection utilities (`[DOCTYPE: TIER-A]`).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C baseline configuration gates.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Upstream Sentinel Invariant: `is_upstream_compiler` holds iff `<repo_root>/.pipeline/upstream` is a directory.
//! - Deterministic Traversal: Directory walks sort entries lexicographically and prune `EXCLUDED_DIRS`.
//!
//! ## 3. Preconditions
//! - `repo_root` arguments reference a valid local filesystem path.
//!
//! ## 4. Postconditions
//! - Returns `Ok(())` when all baseline invariants hold or `Err` with deterministic diagnostics.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Single-path probes: O(1) time/space; tree walks: O(N log N) time and O(V) space.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Strongly typed `Result` diagnostics without panics.
//! - State: Pure read-only queries on baseline checks.
//! - Action: Fails closed on missing required artifacts or forbidden files.
//! - Resource: Prunes `EXCLUDED_DIRS` to bound I/O and memory.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0601], [REQ-WS-MODULE], [DO-330-TQL-4-WORKSPACE] | SysML v2: part def WorkspaceInspector

pub mod cargo_tooling;

use crate::error::CoreError;
use crate::rules::{EXCLUDED_DIRS, MASTER_BLUEPRINTS};
pub use cargo_tooling::{
    evaluate_cargo_platform, load_cargo_lockfile, parse_cargo_toml_lossless,
    resolve_cargo_metadata, set_workspace_dependency_version,
};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Determines whether `repo_root` is the upstream specification core compiler repository.
/// Realises: [REQ-0601], [REQ-WS-UPSTREAM], [FLS: fls_9ucqbbd0s2yo]
#[must_use]
pub fn is_upstream_compiler(repo_root: &Path) -> bool {
    repo_root.join(".pipeline").join("upstream").is_dir()
}

/// Check 10: Verifies that a `.gitignore` file exists in the repository root.
/// Realises: [REQ-0601], [REQ-WS-C10], [FLS: fls_9ucqbbd0s2yo]
pub fn check_gitignore_exists(repo_root: &Path) -> Result<(), CoreError> {
    if repo_root.join(".gitignore").is_file() {
        Ok(())
    } else {
        Err(CoreError::workspace(format!(
            "Check 10 failed: .gitignore missing in repository root '{}'.",
            repo_root.display()
        )))
    }
}

/// Returns `true` if `entry` is not a directory in `EXCLUDED_DIRS`.
/// Realises: [REQ-0601], [FLS: fls_9ucqbbd0s2yo]
fn is_non_excluded_entry(entry: &walkdir::DirEntry) -> bool {
    if entry.file_type().is_dir() {
        let name = entry.file_name().to_string_lossy();
        !EXCLUDED_DIRS.iter().any(|ex| name == ex)
    } else {
        true
    }
}

/// Check 11: Verifies that zero `.DS_Store` OS metadata files exist in the working tree.
/// Realises: [REQ-0601], [REQ-WS-C11], [FLS: fls_9ucqbbd0s2yo]
pub fn check_no_ds_store_files(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut ds_store_files = Vec::new();
    for entry in WalkDir::new(repo_root)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(is_non_excluded_entry)
        .flatten()
    {
        if entry.file_name() == ".DS_Store" {
            let rel = entry
                .path()
                .strip_prefix(repo_root)
                .unwrap_or_else(|_| entry.path())
                .display()
                .to_string();
            ds_store_files.push(rel);
        }
    }
    if ds_store_files.is_empty() {
        Ok(())
    } else {
        Err(vec![format!(
            "Check 11 failed: Found {} .DS_Store file(s): {}",
            ds_store_files.len(),
            ds_store_files.join(", ")
        )])
    }
}

/// Collects leaked Tier 0 crates, forbidden directories, and Tier 1 SDK duplicates in downstream repos.
/// Realises: [REQ-0601], [REQ-WS-C12], [FLS: fls_9ucqbbd0s2yo]
fn collect_downstream_tier_leaks(repo_root: &Path, duplicates: &mut Vec<String>) -> Result<(), Vec<String>> {
    for crate_name in [
        "deap-core",
        "verify-baseline",
        "compile-sysml",
        "ingest-sysml",
        "assemble-conops",
        "reconcile-backlog",
    ] {
        if repo_root.join("crates").join(crate_name).is_dir() {
            duplicates.push(format!("crates/{crate_name}"));
        }
    }
    for rel_dir in ["tests/fixtures/safety", "app_flutter/test/topology/failures"] {
        if repo_root.join(rel_dir).is_dir() {
            duplicates.push(rel_dir.to_string());
        }
    }
    let pubspec_path = repo_root.join("app_flutter").join("pubspec.yaml");
    if pubspec_path.is_file() {
        let pubspec_text = fs::read_to_string(&pubspec_path).map_err(|e| {
            vec![format!("Check 12 failed: Unable to read {}: {e}", pubspec_path.display())]
        })?;
        if !pubspec_text.contains("name: app_flutter") && pubspec_text.contains("app_flutter:") {
            for rel_file in [
                "app_flutter/lib/features/layout/component_factory.dart",
                "app_flutter/lib/features/layout/layout.dart",
                "app_flutter/lib/core/theme/theme_controller.dart",
                "app_flutter/lib/domain/result.dart",
            ] {
                if repo_root.join(rel_file).exists() {
                    duplicates.push(rel_file.to_string());
                }
            }
        }
    }
    Ok(())
}

/// Check 12: Verifies that downstream repositories do not contain duplicate master blueprints or Tier 0 crates.
/// Realises: [REQ-0601], [REQ-WS-C12], [FLS: fls_9ucqbbd0s2yo]
pub fn check_no_duplicate_master_blueprints(repo_root: &Path) -> Result<(), Vec<String>> {
    if is_upstream_compiler(repo_root) {
        return Ok(());
    }
    let mut duplicates = Vec::new();
    collect_downstream_tier_leaks(repo_root, &mut duplicates)?;

    let blueprints_dir = repo_root.join("docs").join("architecture").join("blueprints");
    if blueprints_dir.is_dir() {
        let entries = fs::read_dir(&blueprints_dir).map_err(|e| {
            vec![format!("Check 12 failed: Unable to read {}: {e}", blueprints_dir.display())]
        })?;
        for entry_res in entries {
            let path = entry_res
                .map_err(|e| vec![format!("Check 12 failed: Unable to read {}: {e}", blueprints_dir.display())])?
                .path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                duplicates.push(path.strip_prefix(repo_root).unwrap_or(&path).display().to_string());
            }
        }
    }

    for entry in WalkDir::new(repo_root)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(is_non_excluded_entry)
        .flatten()
    {
        if entry.file_type().is_file() {
            let fname = entry.file_name().to_string_lossy();
            let is_master = MASTER_BLUEPRINTS.iter().any(|bp| bp == fname.as_ref())
                || fname == "DEAP_SYSML_V2_SAFETY_MODEL_SPECIFICATION.sysml";
            if is_master {
                let rel = entry
                    .path()
                    .strip_prefix(repo_root)
                    .unwrap_or_else(|_| entry.path())
                    .display()
                    .to_string();
                if !duplicates.contains(&rel) {
                    duplicates.push(rel);
                }
            }
        }
    }

    if duplicates.is_empty() {
        Ok(())
    } else {
        Err(vec![format!(
            "Check 12 failed: Downstream repository contains duplicate master core blueprint file(s) or Tier 0 compiler crates: {}",
            duplicates.join(", ")
        )])
    }
}

/// Check 14: Verifies presence and non-emptiness of `README.md`, agent instructions, and SSOT rules.
/// Realises: [REQ-0601], [REQ-WS-C14], [FLS: fls_9ucqbbd0s2yo]
pub fn check_downstream_instructions_exist(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let readme_path = repo_root.join("README.md");
    if !readme_path.is_file() {
        errors.push(format!(
            "Check 14 failed: README.md missing in repository root '{}'.",
            repo_root.display()
        ));
    } else {
        match fs::read_to_string(&readme_path) {
            Ok(content) if content.trim().is_empty() => errors.push(format!(
                "Check 14 failed: README.md is empty in repository root '{}'.",
                repo_root.display()
            )),
            Ok(content)
                if !content.contains("# Downstream Cyber-Physical Infrastructure Safety Project")
                    && !content.contains("Operator Prompt Catalog") =>
            {
                errors.push(format!(
                    "Check 14 failed: README.md in '{}' lacks canonical downstream content (missing '# Downstream Cyber-Physical Infrastructure Safety Project' or 'Operator Prompt Catalog').",
                    repo_root.display()
                ));
            }
            Ok(_) => {}
            Err(e) => errors.push(format!("Check 14 failed: Unable to read README.md: {e}")),
        }
    }

    let valid_entrypoint = [
        repo_root.join("AGENTS.md"),
        repo_root.join("CLAUDE.md"),
        repo_root.join(".agents").join("AGENTS.md"),
    ]
    .iter()
    .any(|p| p.is_file() && fs::metadata(p).map(|m| m.len() > 0).unwrap_or(false));
    if !valid_entrypoint {
        errors.push(format!(
            "Check 14 failed: No non-empty agent instruction entrypoint found in '{}' (expected AGENTS.md, CLAUDE.md, or .agents/AGENTS.md).",
            repo_root.display()
        ));
    }

    let sysml_rule_path = repo_root.join("rules").join("sysml-ssot-completeness.md");
    if !sysml_rule_path.is_file() {
        errors.push(format!(
            "Check 14 failed: rules/sysml-ssot-completeness.md missing in repository root '{}'.",
            repo_root.display()
        ));
    } else if fs::metadata(&sysml_rule_path).map(|m| m.len()).unwrap_or(0) == 0 {
        errors.push(format!(
            "Check 14 failed: rules/sysml-ssot-completeness.md is empty in repository root '{}'.",
            repo_root.display()
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check 15: Verifies the presence of required tooling in downstream or upstream workspaces.
/// Realises: [REQ-0601], [REQ-WS-C15], [FLS: fls_9ucqbbd0s2yo]
pub fn check_reconcile_backlog_tooling_exists(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if is_upstream_compiler(repo_root) {
        let release_bin = repo_root.join("target").join("release").join("reconcile-backlog");
        let crate_dir = repo_root.join("crates").join("reconcile-backlog");
        let crate_cargo = crate_dir.join("Cargo.toml");
        let crate_src = crate_dir.join("src").join("main.rs");

        let has_release_bin =
            release_bin.is_file() && fs::metadata(&release_bin).map(|m| m.len() > 0).unwrap_or(false);
        let has_crate = crate_cargo.is_file()
            && parse_cargo_toml_lossless(&crate_cargo).is_ok()
            && crate_src.is_file();

        if !has_release_bin && !has_crate {
            errors.push(format!(
                "Check 15 failed: Reconcile backlog tooling missing in repository root '{}' (expected target/release/reconcile-backlog or crates/reconcile-backlog).",
                repo_root.display()
            ));
        }
    } else {
        let target_release = repo_root.join("target").join("release");
        let mut missing_bins = Vec::new();
        for bin_name in [
            "verify-baseline",
            "compile-sysml",
            "ingest-sysml",
            "assemble-conops",
            "reconcile-backlog",
        ] {
            let bin_path = target_release.join(bin_name);
            let is_valid =
                bin_path.is_file() && fs::metadata(&bin_path).map(|m| m.len() > 0).unwrap_or(false);
            if !is_valid {
                missing_bins.push(format!("target/release/{bin_name}"));
            }
        }
        if !missing_bins.is_empty() {
            errors.push(format!(
                "Check 15 failed: Downstream repository is missing precompiled Tier 0 binary-only tooling (file size > 0 required): {}",
                missing_bins.join(", ")
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Check 16: Enforces the Upstream Template Clean Landing Zone Gate across specification directories.
/// Realises: [REQ-0601], [REQ-WS-C16], [FLS: fls_9ucqbbd0s2yo]
pub fn check_upstream_template_clean_landing_zones(repo_root: &Path) -> Result<(), Vec<String>> {
    if !is_upstream_compiler(repo_root) {
        return Ok(());
    }
    let landing_zones = [
        "docs/conops",
        "docs/safety",
        "docs/interfaces",
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
        for entry in WalkDir::new(&zone_path)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(is_non_excluded_entry)
            .flatten()
        {
            if entry.file_type().is_file() {
                let fname = entry.file_name().to_string_lossy();
                if !allowed_files.contains(&fname.as_ref()) {
                    let rel = entry
                        .path()
                        .strip_prefix(repo_root)
                        .unwrap_or_else(|_| entry.path())
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
        Err(vec![format!(
            "Check 16 failed: Upstream distribution template landing zones contain concrete specification files: {}",
            violations.join(", ")
        )])
    }
}

/// Unit tests for baseline workspace topology checks 10, 11, 12, 14, 15, and 16.
/// Realises: [REQ-0601], [FLS: fls_9ucqbbd0s2yo]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    /// Auto-cleaned temporary directory fixture for isolated filesystem tests.
    /// Realises: [REQ-0601], [FLS: fls_9ucqbbd0s2yo]
    struct TempDir {
        path: std::path::PathBuf,
    }

    impl TempDir {
        /// Creates a unique temporary directory under `std::env::temp_dir()`.
        /// Realises: [REQ-0601], [FLS: fls_9ucqbbd0s2yo]
        fn new(name: &str) -> Result<Self, std::io::Error> {
            let path =
                std::env::temp_dir().join(format!("deap_test_{}_{}", name, uuid::Uuid::new_v4()));
            fs::create_dir_all(&path)?;
            Ok(Self { path })
        }
    }

    impl Drop for TempDir {
        /// Removes the temporary directory upon scope exit.
        /// Realises: [REQ-0601], [FLS: fls_9ucqbbd0s2yo]
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    /// Verifies upstream sentinel, Check 10, Check 11, and Check 12 across isolated fixtures.
    /// Realises: [REQ-0601], [REQ-WS-C10], [REQ-WS-C11], [REQ-WS-C12]
    #[test]
    fn test_checks_10_11_12() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c10_11_12")?;
        assert!(!is_upstream_compiler(&tmp.path));
        assert!(check_gitignore_exists(&tmp.path).is_err());
        File::create(tmp.path.join(".gitignore"))?;
        assert!(check_gitignore_exists(&tmp.path).is_ok());

        File::create(tmp.path.join(".DS_Store"))?;
        assert!(check_no_ds_store_files(&tmp.path).is_err());
        fs::remove_file(tmp.path.join(".DS_Store"))?;
        assert!(check_no_ds_store_files(&tmp.path).is_ok());

        File::create(tmp.path.join("DEAP_MASTER_ARCHITECTURE.md"))?;
        assert!(check_no_duplicate_master_blueprints(&tmp.path).is_err());
        fs::remove_file(tmp.path.join("DEAP_MASTER_ARCHITECTURE.md"))?;
        assert!(check_no_duplicate_master_blueprints(&tmp.path).is_ok());
        Ok(())
    }

    /// Verifies Check 14, Check 15, and Check 16 across downstream and upstream fixtures.
    /// Realises: [REQ-0601], [REQ-WS-C14], [REQ-WS-C15], [REQ-WS-C16]
    #[test]
    fn test_checks_14_15_16() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c14_15_16")?;
        assert!(check_downstream_instructions_exist(&tmp.path).is_err());
        assert!(check_reconcile_backlog_tooling_exists(&tmp.path).is_err());

        writeln!(
            File::create(tmp.path.join("README.md"))?,
            "# Downstream Cyber-Physical Infrastructure Safety Project"
        )?;
        writeln!(File::create(tmp.path.join("AGENTS.md"))?, "# Agents")?;
        fs::create_dir_all(tmp.path.join("rules"))?;
        writeln!(File::create(tmp.path.join("rules/sysml-ssot-completeness.md"))?, "# Rule")?;
        assert!(check_downstream_instructions_exist(&tmp.path).is_ok());

        fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
        fs::create_dir_all(tmp.path.join("crates/reconcile-backlog/src"))?;
        fs::write(
            tmp.path.join("crates/reconcile-backlog/Cargo.toml"),
            "[package]\nname = \"reconcile-backlog\"\nversion = \"0.1.0\"\n",
        )?;
        File::create(tmp.path.join("crates/reconcile-backlog/src/main.rs"))?;
        assert!(check_reconcile_backlog_tooling_exists(&tmp.path).is_ok());

        fs::create_dir_all(tmp.path.join("schema"))?;
        File::create(tmp.path.join("schema/.gitkeep"))?;
        assert!(check_upstream_template_clean_landing_zones(&tmp.path).is_ok());
        File::create(tmp.path.join("schema/domain_model.sysml"))?;
        assert!(check_upstream_template_clean_landing_zones(&tmp.path).is_err());
        Ok(())
    }
}
