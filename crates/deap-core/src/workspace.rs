//! Repository topology and workspace inspection utilities.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C configuration management and
//! software baseline verification standards. Implements deterministic filesystem topology gates
//! (Checks 10, 11, 12, 14, 15, and 16) that enforce upstream vs. downstream repository boundaries,
//! clean landing zones, OS metadata hygiene, and mandatory governance artifacts.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Upstream Sentinel Invariant: A workspace is classified as `UPSTREAM_SPEC_CORE_COMPILER` iff
//!   `<repo_root>/.pipeline/upstream` exists as a directory.
//! - Deterministic Traversal: All multi-file workspace scans sort directory entries lexicographically
//!   by filename (`sort_by_file_name()`) and prune directories matching `EXCLUDED_DIRS`.
//! - Zero Mutation Invariant: All workspace verification functions are strictly read-only queries over
//!   filesystem metadata and file contents.
//!
//! ## 3. Preconditions
//! - `repo_root` arguments must reference a valid local filesystem path.
//!
//! ## 4. Postconditions
//! - Verification gates return `Ok(())` iff all checked baseline invariants hold, or `Err` containing
//!   deterministic, human-auditable diagnostic strings enumerating every violation.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Single-Path Probes (`is_upstream_compiler`, `check_gitignore_exists`, `check_reconcile_backlog_tooling_exists`):
//!   O(1) time and O(1) space.
//! - Directory Tree Traversals (`check_no_ds_store_files`, `check_no_duplicate_master_blueprints`,
//!   `check_upstream_template_clean_landing_zones`): O(N log N) time and O(V) space where N is visited
//!   non-excluded entries and V is violation count.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - All checks fail closed on missing required artifacts or forbidden file presence without panicking.
//! - Unreadable directory entries encountered during tree walks are safely skipped without aborting traversal.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-WS-MODULE], [REQ-0608], [DO-330-TQL-4-WORKSPACE] | SysML v2: part def WorkspaceInspector

#![forbid(unsafe_code)]

use crate::error::CoreError;
use crate::rules::{EXCLUDED_DIRS, MASTER_BLUEPRINTS};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Determines whether the repository at `repo_root` is the upstream specification core compiler.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C configuration management controls. Enforces
/// the strict architectural role boundary between `UPSTREAM_SPEC_CORE_COMPILER` and
/// `DOWNSTREAM_CUSTOMER_PROJECT` workspaces.
///
/// ## 2. Invariants & Mathematical Properties
/// - Sentinel Equivalence: Evaluates to `true` iff the path `<repo_root>/.pipeline/upstream` exists
///   on disk and its filesystem metadata classifies it as a directory (`is_dir() == true`).
/// - Pure Read-Only Probe: Performs zero mutations to the workspace filesystem.
///
/// ## 3. Preconditions
/// - `repo_root` is a valid filesystem path reference to a candidate workspace root.
///
/// ## 4. Postconditions
/// - Returns `true` if `<repo_root>/.pipeline/upstream` is an existing directory, `false` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) bounded to a single path join and OS `stat` metadata query.
/// - Space Complexity: O(|repo_root|) temporary `PathBuf` allocation for the joined sentinel path.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Accepts any `&Path` reference without lifetime or encoding hazards.
/// - State: Stateless query; cannot corrupt repository or process state.
/// - Action: Returns `false` cleanly if the sentinel path does not exist, is a regular file, or lacks permissions; never panics.
/// - Resource: Bounded single `PathBuf` allocation freed immediately upon return.
///
/// ## 7. Realization & Traceability
/// Realises: [Workspace/is_upstream_compiler] | [REQ-WS-UPSTREAM] | SysML v2: part def WorkspaceInspector
pub fn is_upstream_compiler(repo_root: &Path) -> bool {
    repo_root.join(".pipeline").join("upstream").is_dir()
}

/// Check 10: Verifies that a `.gitignore` file exists in the repository root.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C baseline configuration hygiene mandates. Prevents
/// accidental commitment of untracked build artifacts, temporary files, or workstation metadata.
///
/// ## 2. Invariants & Mathematical Properties
/// - Regular File Invariant: Succeeds (`Ok(())`) iff `<repo_root>/.gitignore` exists on disk and is
///   a regular file (`is_file() == true`).
///
/// ## 3. Preconditions
/// - `repo_root` is a valid filesystem path reference to a workspace root directory.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` when `<repo_root>/.gitignore` is a regular file.
/// - Returns `Err(String)` containing `"Check 10 failed: .gitignore missing in repository root ..."` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) single filesystem metadata lookup.
/// - Space Complexity: O(|repo_root|) for the joined path and formatted error string on failure.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Returns a strongly typed `Result<(), CoreError>` diagnostic.
/// - State: Read-only filesystem metadata inspection; zero side effects.
/// - Action: Fails closed with `Err` if `.gitignore` is absent, a broken symlink, or a directory; never panics.
/// - Resource: Minimal stack and single short-lived `PathBuf` allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [Workspace/check_gitignore_exists] | [REQ-WS-C10] | SysML v2: part def WorkspaceInspector
pub fn check_gitignore_exists(repo_root: &Path) -> Result<(), CoreError> {
    let gitignore = repo_root.join(".gitignore");
    if gitignore.is_file() {
        Ok(())
    } else {
        Err(CoreError::workspace(format!(
            "Check 10 failed: .gitignore missing in repository root '{}'.",
            repo_root.display()
        )))
    }
}

/// Check 11: Verifies that zero `.DS_Store` OS metadata files exist in the working tree.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C clean baseline mandates. Prevents workstation-specific
/// macOS Finder metadata files (`.DS_Store`) from polluting deterministic engineering repositories.
///
/// ## 2. Invariants & Mathematical Properties
/// - Exhaustive Non-Excluded Walk: Traverses `<repo_root>` in deterministic lexicographical filename
///   order while pruning any directory whose name matches `EXCLUDED_DIRS.iter()`.
/// - Zero Tolerance: Succeeds (`Ok(())`) iff the set of visited entries with `file_name() == ".DS_Store"`
///   is empty.
///
/// ## 3. Preconditions
/// - `repo_root` is a valid filesystem path reference to a workspace root directory.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` if zero `.DS_Store` files are detected outside excluded directories.
/// - Returns `Err(Vec<String>)` containing a single formatted `"Check 11 failed: Found N .DS_Store file(s): ..."`
///   diagnostic listing all detected workspace-relative paths in sorted order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N log N) where N is the number of non-excluded directory entries under `repo_root`.
/// - Space Complexity: O(D + V) where D is maximum directory depth and V is the count of `.DS_Store` violations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Returns `Result<(), Vec<String>>` compatible with baseline check aggregators.
/// - State: Pure read-only directory walk; does not delete or modify workspace files.
/// - Action: Skips unreadable directory entries safely (`let Ok(e) = e else { continue; }`) and fails closed
///   if one or more `.DS_Store` files are found; never panics.
/// - Resource: Prunes `EXCLUDED_DIRS` (such as `target`, `.git`, `node_modules`) to bound traversal I/O.
///
/// ## 7. Realization & Traceability
/// Realises: [Workspace/check_no_ds_store_files] | [REQ-WS-C11] | SysML v2: part def WorkspaceInspector
pub fn check_no_ds_store_files(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut ds_store_files = Vec::new();

    for e in WalkDir::new(repo_root)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !EXCLUDED_DIRS.iter().any(|ex| name == ex)
            } else {
                true
            }
        })
    {
        let Ok(e) = e else {
            continue;
        };
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

/// Check 12: Verifies that downstream repositories do not contain duplicate upstream master core blueprints
/// or Tier 0 compiler crate source directories.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ECSS-E-ST-40C Single Source of Truth (SSOT) governance, and
/// DEAP-BLUEPRINT-POLYREPO-013 tier isolation controls. Ensures master architecture blueprints
/// and Tier 0 compiler crates reside exclusively in the upstream specification compiler repository
/// (`gintatkinson/DEAP01-spec-core`) and are never duplicated in downstream customer workspaces.
///
/// ## 2. Invariants & Mathematical Properties
/// - Upstream Exemption: If `is_upstream_compiler(repo_root) == true`, returns `Ok(())` immediately.
/// - Downstream Compiler Source Isolation: In downstream workspaces (`!is_upstream_compiler(repo_root)`),
///   none of the 6 Tier 0 compiler crate directories (`crates/deap-core`, `crates/verify-baseline`,
///   `crates/compile-sysml`, `crates/ingest-sysml`, `crates/assemble-conops`, `crates/reconcile-backlog`)
///   may exist on disk.
/// - Downstream Blueprint Directory Isolation: `docs/architecture/blueprints/` must not contain `.md` files.
/// - Downstream Fixture & Tier 1 SDK Isolation: `tests/fixtures/safety` and `app_flutter/test/topology/failures`
///   must not exist on disk, and Tier 2 Flutter applications consuming `package:app_flutter` must not duplicate
///   canonical Tier 1 SDK core source files locally.
/// - Downstream Blueprint Disjointness: Non-excluded regular files matching `MASTER_BLUEPRINTS.iter()` or
///   `"DEAP_SYSML_V2_SAFETY_MODEL_SPECIFICATION.sysml"` must be empty.
///
/// ## 3. Preconditions
/// - `repo_root` is a valid filesystem path reference to a workspace root directory.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` if `repo_root` is an upstream compiler workspace or satisfies all downstream isolation invariants.
/// - Returns `Err(Vec<String>)` with a `"Check 12 failed: ..."` diagnostic enumerating all duplicate
///   blueprint paths and leaked Tier 0 compiler crate directories.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) for upstream workspaces; O(N log N) for downstream workspaces where N is the
///   number of non-excluded filesystem entries.
/// - Space Complexity: O(D + V) where D is directory tree depth and V is the count of detected violations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Returns structured `Result<(), Vec<String>>` without panicking.
/// - State: Read-only filesystem inspection; never mutates workspace files.
/// - Action: Fails closed whenever any forbidden master blueprint or Tier 0 crate is present downstream.
/// - Resource: Prunes `EXCLUDED_DIRS` during traversal to prevent unbounded directory scans.
///
/// ## 7. Realization & Traceability
/// Realises: [Workspace/check_no_duplicate_master_blueprints] | [REQ-WS-C12] | SysML v2: part def WorkspaceInspector
pub fn check_no_duplicate_master_blueprints(repo_root: &Path) -> Result<(), Vec<String>> {
    if is_upstream_compiler(repo_root) {
        return Ok(());
    }

    let mut duplicates = Vec::new();

    let tier0_crates = [
        "deap-core",
        "verify-baseline",
        "compile-sysml",
        "ingest-sysml",
        "assemble-conops",
        "reconcile-backlog",
    ];
    let crates_dir = repo_root.join("crates");
    for crate_name in &tier0_crates {
        let crate_path = crates_dir.join(crate_name);
        if crate_path.is_dir() {
            duplicates.push(format!("crates/{}", crate_name));
        }
    }

    let forbidden_downstream_dirs = [
        "tests/fixtures/safety",
        "app_flutter/test/topology/failures",
    ];
    for rel_dir in &forbidden_downstream_dirs {
        if repo_root.join(rel_dir).is_dir() {
            duplicates.push(rel_dir.to_string());
        }
    }

    let pubspec_path = repo_root.join("app_flutter").join("pubspec.yaml");
    if pubspec_path.is_file() {
        if let Ok(pubspec_text) = fs::read_to_string(&pubspec_path) {
            if !pubspec_text.contains("name: app_flutter") && pubspec_text.contains("app_flutter:")
            {
                let tier1_sdk_files = [
                    "app_flutter/lib/features/layout/component_factory.dart",
                    "app_flutter/lib/features/layout/layout.dart",
                    "app_flutter/lib/core/theme/theme_controller.dart",
                    "app_flutter/lib/domain/result.dart",
                ];
                for rel_file in &tier1_sdk_files {
                    if repo_root.join(rel_file).exists() {
                        duplicates.push(rel_file.to_string());
                    }
                }
            }
        }
    }

    let blueprints_dir = repo_root
        .join("docs")
        .join("architecture")
        .join("blueprints");
    if blueprints_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&blueprints_dir) {
            for entry in entries.flatten() {
                if entry.path().is_file()
                    && entry
                        .path()
                        .extension()
                        .map(|ext| ext == "md")
                        .unwrap_or(false)
                {
                    let rel = entry
                        .path()
                        .strip_prefix(repo_root)
                        .unwrap_or(&entry.path())
                        .display()
                        .to_string();
                    duplicates.push(rel);
                }
            }
        }
    }

    let blueprints: Vec<&str> = MASTER_BLUEPRINTS
        .iter()
        .chain(std::iter::once(
            "DEAP_SYSML_V2_SAFETY_MODEL_SPECIFICATION.sysml",
        ))
        .collect();

    for e in WalkDir::new(repo_root)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !EXCLUDED_DIRS.iter().any(|ex| name == ex)
            } else {
                true
            }
        })
    {
        let Ok(e) = e else {
            continue;
        };
        if e.file_type().is_file() {
            let fname = e.file_name().to_string_lossy();
            if blueprints.iter().any(|&bp| bp == fname) {
                let rel = e
                    .path()
                    .strip_prefix(repo_root)
                    .unwrap_or(e.path())
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
        let msg = format!(
            "Check 12 failed: Downstream repository contains duplicate master core blueprint file(s) or Tier 0 compiler crates: {}",
            duplicates.join(", ")
        );
        Err(vec![msg])
    }
}

/// Check 14: Verifies presence and non-emptiness of `README.md`, agent instruction entrypoints,
/// and `rules/sysml-ssot-completeness.md`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C operational documentation completeness gates.
/// Guarantees that every workspace provides mandatory operator onboarding documentation, autonomous
/// agent governance entrypoints, and SysML v2 Single Source of Truth completeness rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Conjunctive Completeness Invariant: Returns `Ok(())` iff all three conditions hold simultaneously:
///   1. `<repo_root>/README.md` is a readable, non-whitespace file containing either
///      `"# Downstream Cyber-Physical Infrastructure Safety Project"` or `"Operator Prompt Catalog"`.
///   2. At least one of `AGENTS.md`, `CLAUDE.md`, or `.agents/AGENTS.md` exists under `repo_root`
///      as a regular file with byte length `> 0`.
///   3. `<repo_root>/rules/sysml-ssot-completeness.md` exists as a regular file with byte length `> 0`.
///
/// ## 3. Preconditions
/// - `repo_root` is a valid filesystem path reference to a workspace root directory.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` when all three instruction artifacts exist and satisfy content invariants.
/// - Returns `Err(Vec<String>)` enumerating every failed sub-condition with `"Check 14 failed: ..."` diagnostics.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(|README.md|) linear in the byte length of `README.md` plus O(1) metadata probes.
/// - Space Complexity: O(|README.md|) heap buffer for `README.md` content inspection.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Returns `Result<(), Vec<String>>` aggregating all detected Check 14 errors in a single pass.
/// - State: Read-only file content and metadata verification.
/// - Action: Captures I/O read errors on `README.md` and metadata errors on rule files fail-closed into
///   `errors` without panicking.
/// - Resource: Bounded memory allocation equal to the size of `README.md`.
///
/// ## 7. Realization & Traceability
/// Realises: [Workspace/check_downstream_instructions_exist] | [REQ-WS-C14] | SysML v2: part def WorkspaceInspector
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
                } else if !content
                    .contains("# Downstream Cyber-Physical Infrastructure Safety Project")
                    && !content.contains("Operator Prompt Catalog")
                {
                    errors.push(format!(
                        "Check 14 failed: README.md in '{}' lacks canonical downstream content (missing '# Downstream Cyber-Physical Infrastructure Safety Project' or 'Operator Prompt Catalog').",
                        repo_root.display()
                    ));
                }
            }
            Err(e) => {
                errors.push(format!("Check 14 failed: Unable to read README.md: {}", e));
            }
        }
    }

    // 2. Agent instruction entrypoint
    let agent_entrypoints = [
        repo_root.join("AGENTS.md"),
        repo_root.join("CLAUDE.md"),
        repo_root.join(".agents").join("AGENTS.md"),
    ];
    let valid_entrypoint = agent_entrypoints
        .iter()
        .any(|p| p.is_file() && fs::metadata(p).map(|m| m.len() > 0).unwrap_or(false));
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

/// Check 15: Verifies the presence of required tooling in downstream or upstream workspaces.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ECSS-E-ST-40C binary-only distribution mandates, and
/// DEAP-BLUEPRINT-POLYREPO-013 tier isolation controls. Guarantees all 5 precompiled release binaries
/// exist in downstream customer repositories, or that backlog reconciliation tooling is present in
/// upstream compiler workspaces.
///
/// ## 2. Invariants & Mathematical Properties
/// - Upstream Compiler Invariant: If `is_upstream_compiler(repo_root)`, succeeds (`Ok(())`) iff:
///   - `(<repo_root>/target/release/reconcile-backlog` is a regular file with byte length `> 0`) OR
///   - (`<repo_root>/crates/reconcile-backlog/Cargo.toml` and `<repo_root>/crates/reconcile-backlog/src/main.rs`
///     are both existing regular files).
/// - Downstream Binary-Only Invariant: If `!is_upstream_compiler(repo_root)`, succeeds (`Ok(())`) iff:
///   - All 5 precompiled binaries exist under `<repo_root>/target/release/` with file length `> 0`:
///     `verify-baseline`, `compile-sysml`, `ingest-sysml`, `assemble-conops`, `reconcile-backlog`.
///
/// ## 3. Preconditions
/// - `repo_root` is a valid filesystem path reference to a workspace root directory.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` when upstream tooling exists or all 5 downstream release binaries exist and are non-empty.
/// - Returns `Err(Vec<String>)` with a `"Check 15 failed: ..."` diagnostic enumerating missing tooling.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) bounded to at most 10 filesystem metadata probes.
/// - Space Complexity: O(|repo_root|) for joined path buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Returns `Result<(), Vec<String>>` without panicking.
/// - State: Pure read-only filesystem probe; zero workspace side effects.
/// - Action: Treats zero-byte binary files or unreadable metadata as absent and fails closed.
/// - Resource: Constant O(1) file descriptor and memory footprint.
///
/// ## 7. Realization & Traceability
/// Realises: [Workspace/check_reconcile_backlog_tooling_exists] | [REQ-WS-C15] | SysML v2: part def WorkspaceInspector
pub fn check_reconcile_backlog_tooling_exists(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if is_upstream_compiler(repo_root) {
        let release_bin = repo_root
            .join("target")
            .join("release")
            .join("reconcile-backlog");
        let crate_dir = repo_root.join("crates").join("reconcile-backlog");
        let crate_cargo = crate_dir.join("Cargo.toml");
        let crate_src = crate_dir.join("src").join("main.rs");

        let has_release_bin = release_bin.is_file()
            && fs::metadata(&release_bin)
                .map(|m| m.len() > 0)
                .unwrap_or(false);
        let has_crate = crate_cargo.is_file() && crate_src.is_file();

        if !has_release_bin && !has_crate {
            errors.push(format!(
                "Check 15 failed: Reconcile backlog tooling missing in repository root '{}' (expected target/release/reconcile-backlog or crates/reconcile-backlog).",
                repo_root.display()
            ));
        }
    } else {
        let required_binaries = [
            "verify-baseline",
            "compile-sysml",
            "ingest-sysml",
            "assemble-conops",
            "reconcile-backlog",
        ];

        let target_release = repo_root.join("target").join("release");
        let mut missing_bins = Vec::new();

        for bin_name in &required_binaries {
            let bin_path = target_release.join(bin_name);
            let is_valid = bin_path.is_file()
                && fs::metadata(&bin_path)
                    .map(|m| m.len() > 0)
                    .unwrap_or(false);
            if !is_valid {
                missing_bins.push(format!("target/release/{}", bin_name));
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C upstream distribution cleanliness mandates.
/// Prevents concrete downstream customer schemas, domain models, or generated specifications from
/// being committed into the upstream specification core compiler repository.
///
/// ## 2. Invariants & Mathematical Properties
/// - Downstream Exemption: If `!is_upstream_compiler(repo_root)`, returns `Ok(())` immediately.
/// - Clean Landing Zone Invariant: In upstream compiler workspaces (`is_upstream_compiler(repo_root)`),
///   every regular file inside `docs/conops`, `docs/safety`, `docs/interfaces`, `docs/epics`,
///   `docs/features`, `docs/user-stories`, `docs/use-cases`, `docs/management`, and `schema` (excluding
///   `EXCLUDED_DIRS`) must have a filename in `{".gitkeep", "README.md"}`.
///
/// ## 3. Preconditions
/// - `repo_root` is a valid filesystem path reference to a workspace root directory.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` if `repo_root` is a downstream workspace or all 9 upstream landing zones contain
///   only `.gitkeep` or `README.md` files.
/// - Returns `Err(Vec<String>)` with a `"Check 16 failed: ..."` diagnostic listing all violating
///   workspace-relative file paths in deterministic order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) for downstream workspaces; O(N log N) for upstream workspaces where N is the
///   total number of directory entries across the 9 landing zones.
/// - Space Complexity: O(D + V) where D is maximum subdirectory depth and V is the count of violating files.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Returns `Result<(), Vec<String>>` for deterministic gate reporting.
/// - State: Read-only directory traversal; never deletes or modifies user files.
/// - Action: Gracefully skips non-existent landing zone directories and unreadable entries while failing
///   closed on any forbidden concrete file; never panics.
/// - Resource: Bounded traversal restricted to the 9 designated landing zone subtrees.
///
/// ## 7. Realization & Traceability
/// Realises: [Workspace/check_upstream_template_clean_landing_zones] | [REQ-WS-C16] | SysML v2: part def WorkspaceInspector
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

        for e in WalkDir::new(&zone_path)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    let name = e.file_name().to_string_lossy();
                    !EXCLUDED_DIRS.iter().any(|ex| name == ex)
                } else {
                    true
                }
            })
        {
            let Ok(e) = e else {
                continue;
            };
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

/// Hard upper bound on AST, TOML delimiter, and dependency-graph traversal depth under BP-08.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C bounded
/// iteration mandates (REQ-0608, SC-BP08-03, FM-BP08-003, FM-BP08-007). Prevents stack exhaustion
/// and unbounded loop execution when parsing manifests, lockfiles, and Rust syntax trees.
///
/// ## 2. Invariants & Mathematical Properties
/// - Constant Ceiling: Fixed compile-time constant `MAX_AST_DEPTH == 64`.
///
/// ## 3. Preconditions
/// - None; compile-time constant accessible across all verification routines.
///
/// ## 4. Postconditions
/// - Evaluates deterministically to `64usize`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) compile-time constant evaluation.
/// - Space Complexity: O(1) zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Typed `usize` constant preventing negative threshold configuration.
/// - State: Immutable constant; zero mutable global state.
/// - Action: Guards iterative depth accumulators fail-closed before overflow.
/// - Resource: Zero runtime memory overhead.
///
/// ## 7. Realization & Traceability
/// Realises: [AirGappedCargoLockGovernor/MAX_AST_DEPTH] | [REQ-0608] | SysML v2: part def AirGappedCargoLockGovernor
pub const MAX_AST_DEPTH: usize = 64;

/// Maximum permitted byte length for ingested `Cargo.toml` and vendor configuration payloads.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 resource bounding controls (REQ-0608, SC-BP08-03).
/// Bounds memory ingestion on `manifest_lock_in` to 1 MiB (`1_048_576` bytes).
///
/// ## 2. Invariants & Mathematical Properties
/// - Constant Ceiling: Fixed compile-time constant `MAX_MANIFEST_BYTES == 1_048_576`.
///
/// ## 3. Preconditions
/// - None; compile-time constant.
///
/// ## 4. Postconditions
/// - Evaluates deterministically to `1_048_576usize`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) compile-time constant evaluation.
/// - Space Complexity: O(1) zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Typed `usize` byte ceiling compatible with slice `.get(...)` bounds probes.
/// - State: Immutable constant; cannot drift at runtime.
/// - Action: Triggers `OfflineLockGovernanceError::ManifestSizeCeilingExceeded` when exceeded.
/// - Resource: Caps manifest buffer ingestion to 1 MiB.
///
/// ## 7. Realization & Traceability
/// Realises: [AirGappedCargoLockGovernor/MAX_MANIFEST_BYTES] | [REQ-0608] | SysML v2: part def AirGappedCargoLockGovernor
pub const MAX_MANIFEST_BYTES: usize = 1_048_576;

/// Default maximum permitted byte length for ingested workspace `Cargo.lock` payloads.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 resource bounding controls (REQ-0608, SC-BP08-03).
/// Bounds lockfile ingestion on `manifest_lock_in` to 8 MiB (`8_388_608` bytes).
///
/// ## 2. Invariants & Mathematical Properties
/// - Constant Ceiling: Fixed compile-time constant `MAX_CARGO_LOCK_BYTES == 8_388_608`.
///
/// ## 3. Preconditions
/// - None; compile-time constant.
///
/// ## 4. Postconditions
/// - Evaluates deterministically to `8_388_608usize`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) compile-time constant evaluation.
/// - Space Complexity: O(1) zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Typed `usize` byte ceiling passed to `parse_locked_manifest_snapshot`.
/// - State: Immutable constant; zero runtime mutation.
/// - Action: Triggers `OfflineLockGovernanceError::ManifestSizeCeilingExceeded` when exceeded.
/// - Resource: Caps lockfile buffer ingestion to 8 MiB.
///
/// ## 7. Realization & Traceability
/// Realises: [AirGappedCargoLockGovernor/MAX_CARGO_LOCK_BYTES] | [REQ-0608] | SysML v2: part def AirGappedCargoLockGovernor
pub const MAX_CARGO_LOCK_BYTES: usize = 8_388_608;

/// Provenance classification for a locked Cargo package entry in an air-gapped build snapshot.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 supply-chain provenance controls (REQ-0608,
/// SC-BP08-01). Distinguishes external registry-pinned archives, first-party workspace members,
/// and air-gapped `cargo vendor` directory bundles without stringly-typed flags.
///
/// ## 2. Invariants & Mathematical Properties
/// - Exhaustive Partition: Every valid `LockedPackageRecord` belongs to exactly one of
///   `RegistryLocked`, `WorkspacePath`, or `VendoredBundle`.
/// - Checksum Coupling: `RegistryLocked` and `VendoredBundle` packages always carry a validated
///   64-hex-digit SHA-256 checksum.
///
/// ## 3. Preconditions
/// - Constructed exclusively after `Cargo.toml`, `Cargo.lock`, and optional vendor configuration
///   pass syntax and policy verification.
///
/// ## 4. Postconditions
/// - Preserves total lexical and structural ordering (`Ord`, `Eq`) across deterministic snapshots.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) comparison and copy operations.
/// - Space Complexity: O(1) single-byte discriminant representation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Closed enum preventing unpinned git or unknown source schemes from representation.
/// - State: Copyable immutable value type.
/// - Action: Rejects unsupported schemes upstream via `UnpinnedGitOrWildcardSpecifier`.
/// - Resource: Constant 1-byte stack footprint.
///
/// ## 7. Realization & Traceability
/// Realises: [AirGappedCargoLockGovernor/PackageSourceKind] | [REQ-0608] | SysML v2: part def AirGappedCargoLockGovernor
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum PackageSourceKind {
    /// Registry-sourced crate pinned with a 64-hex-digit SHA-256 archive checksum.
    /// Realises: [REQ-0608]
    RegistryLocked,
    /// First-party workspace member crate resolved via local workspace path.
    /// Realises: [REQ-0608]
    WorkspacePath,
    /// Air-gapped cargo vendor directory bundle crate entry.
    /// Realises: [REQ-0608]
    VendoredBundle,
}

/// Deterministic record of a single `[[package]]` entry parsed from an immutable `Cargo.lock` snapshot.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 deterministic dependency verification (REQ-0608,
/// SC-BP08-01). Captures exact SemVer version, validated 64-hex-digit SHA-256 archive digest,
/// provenance kind, and `BTreeSet`-ordered dependency edges for air-gapped graph inspection.
///
/// ## 2. Invariants & Mathematical Properties
/// - Exact SemVer Invariant: `version` satisfies three-component `major.minor.patch` SemVer syntax
///   with zero wildcard characters.
/// - Digest Integrity Invariant: When `source_kind` is `RegistryLocked` or `VendoredBundle`,
///   `checksum` is `Some(h)` with `h.len() == 64` lowercase hexadecimal ASCII digits.
/// - Ordered Edge Set: `dependencies` stores canonical `<crate>@<version>` edges in `BTreeSet` order.
///
/// ## 3. Preconditions
/// - Populated by `parse_locked_manifest_snapshot` from validated UTF-8 `Cargo.lock` statements.
///
/// ## 4. Postconditions
/// - Exposes immutable package metadata ready for bounded `MAX_AST_DEPTH` graph traversal.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(D log D) construction over D direct dependencies of the package.
/// - Space Complexity: O(|name| + |version| + D) heap storage.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Strongly typed fields with `Option<String>` checksum and `BTreeSet<String>` edges.
/// - State: Immutable value record once inserted into `LockManifestSnapshot`.
/// - Action: Construction fails closed if checksum hex length or SemVer format is malformed.
/// - Resource: Bounded by validated `max_lock_bytes` input size ceiling.
///
/// ## 7. Realization & Traceability
/// Realises: [AirGappedCargoLockGovernor/LockedPackageRecord] | [REQ-0608] | SysML v2: part def AirGappedCargoLockGovernor
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LockedPackageRecord {
    /// Canonical crate package identifier from the Cargo.lock package entry.
    /// Realises: [REQ-0608]
    pub name: String,
    /// Exact SemVer version string pinned in Cargo.lock.
    /// Realises: [REQ-0608]
    pub version: String,
    /// Validated 64-hex-digit SHA-256 archive digest for registry or vendored crates.
    /// Realises: [REQ-0608]
    pub checksum: Option<String>,
    /// Supply-chain provenance classification for this locked package record.
    /// Realises: [REQ-0608]
    pub source_kind: PackageSourceKind,
    /// Lexically ordered directed dependency edges resolved in Cargo.lock.
    /// Realises: [REQ-0608]
    pub dependencies: BTreeSet<String>,
}

/// Immutable cryptographic and structural snapshot of workspace `Cargo.toml` and `Cargo.lock` state.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D air-gapped build reproducibility
/// mandates (REQ-0608, SC-BP08-01, SC-BP08-02). Binds 32-byte SHA-256 digests of `Cargo.toml` and
/// `Cargo.lock` to a `BTreeMap`-ordered package graph before CST analysis may transition from `Idle`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Lexical Determinism: `packages` is a `BTreeMap<String, LockedPackageRecord>` keyed by
///   `"<crate>@<version>"`, and `workspace_members` is a `BTreeSet<String>`, guaranteeing identical
///   iteration order across platforms without hash randomization.
/// - Policy Enforcement Invariant: Valid snapshots always satisfy `offline_locked_enforced == true`
///   and `vendor_bundle_supported == true`.
/// - TOCTOU Parity Anchor: `cargo_toml_sha256` and `cargo_lock_sha256` capture exact `[u8; 32]`
///   SHA-256 digests of the ingested byte buffers.
///
/// ## 3. Preconditions
/// - Constructed via `parse_locked_manifest_snapshot` over size-bounded UTF-8 manifest and lock bytes.
///
/// ## 4. Postconditions
/// - Contains at least one validated `LockedPackageRecord`, zero forbidden semantic solver crates,
///   and complete manifest-to-lock package presence parity.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(log P) package lookup by `<crate>@<version>` where P is package count.
/// - Space Complexity: O(P + E + M) where P is package count, E is dependency edges, and M is members.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Exposes deterministic `BTreeMap` and `BTreeSet` collections with zero `HashMap` usage.
/// - State: Value snapshot preventing TOCTOU drift when compared against post-execution digests.
/// - Action: Never constructed partially; any policy, syntax, or checksum error aborts fail-closed.
/// - Resource: Bounded strictly by `MAX_MANIFEST_BYTES` and `max_lock_bytes`.
///
/// ## 7. Realization & Traceability
/// Realises: [AirGappedCargoLockGovernor/LockManifestSnapshot] | [REQ-0608] | SysML v2: part def AirGappedCargoLockGovernor
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LockManifestSnapshot {
    /// Immutable 32-byte SHA-256 digest of the workspace Cargo.toml payload.
    /// Realises: [REQ-0608]
    pub cargo_toml_sha256: [u8; 32],
    /// Immutable 32-byte SHA-256 digest of the workspace Cargo.lock payload.
    /// Realises: [REQ-0608]
    pub cargo_lock_sha256: [u8; 32],
    /// Deterministic package inventory keyed lexically by `<crate>@<version>`.
    /// Realises: [REQ-0608]
    pub packages: BTreeMap<String, LockedPackageRecord>,
    /// Lexically ordered set of declared workspace member crate names.
    /// Realises: [REQ-0608]
    pub workspace_members: BTreeSet<String>,
    /// Indicates that `--offline --locked` supply-chain enforcement is active.
    /// Realises: [REQ-0608]
    pub offline_locked_enforced: bool,
    /// Indicates that air-gapped `cargo vendor` bundle resolution is supported.
    /// Realises: [REQ-0608]
    pub vendor_bundle_supported: bool,
}

/// Typed fail-closed error taxonomy for air-gapped `--offline --locked` Cargo lock governance.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 strongly-typed fault classification mandates
/// (REQ-0608, SC-BP08-01..SC-BP08-05). Eliminates stringly-typed `Result<T, String>` error payloads
/// across `AirGappedCargoLockGovernor` and `LosslessRustCstAnalyzer` precondition checks.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero Stringly-Typed Payload Invariant: Every variant is a discrete unit discriminant deriving
///   `Debug`, `Clone`, `Copy`, `PartialEq`, and `Eq`.
/// - Total Diagnostic Mapping: Every variant maps deterministically to `"DEAP-E0804"` (supply-chain
///   lock governance rejection) or `"DEAP-E0803"` (CST complexity envelope violation).
///
/// ## 3. Preconditions
/// - Returned whenever any supply-chain, lockfile, graph depth, checksum, or CST envelope guard fails.
///
/// ## 4. Postconditions
/// - Provides deterministic diagnostic codes and single-line ASCII messages without embedded newlines
///   or ANSI escape sequences.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) discriminant dispatch, comparison, and formatting.
/// - Space Complexity: O(1) single-byte enum representation with zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Implements `std::error::Error` and `fmt::Display` with static ASCII strings.
/// - State: Pure immutable value discriminant.
/// - Action: Forces fail-closed transition to `EmittingJsonDiagnostics` without state mutation.
/// - Resource: Zero dynamic memory allocation on error construction.
///
/// ## 7. Realization & Traceability
/// Realises: [AirGappedCargoLockGovernor/OfflineLockGovernanceError] | [REQ-0608] | SysML v2: part def AirGappedCargoLockGovernor
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum OfflineLockGovernanceError {
    /// Offline or vendored lock governance policy flag was explicitly disabled.
    /// Realises: [REQ-0608]
    LockPolicyDisabled,
    /// Manifest, vendor config, or Cargo.lock payload exceeded the configured byte ceiling.
    /// Realises: [REQ-0608]
    ManifestSizeCeilingExceeded,
    /// Invalid UTF-8, unbalanced delimiters, or malformed Cargo.toml/Cargo.lock syntax.
    /// Realises: [REQ-0608]
    MalformedTomlOrLockSyntax,
    /// Floating git source or wildcard SemVer dependency specifier detected.
    /// Realises: [REQ-0608]
    UnpinnedGitOrWildcardSpecifier,
    /// Forbidden heavyweight semantic or compiler-internal dependency detected.
    /// Realises: [REQ-0608]
    ForbiddenSemanticDependencyDetected,
    /// Required pinned CST or Rust codegen dependency missing from the locked graph.
    /// Realises: [REQ-0608]
    MissingRequiredCstDependency,
    /// Registry or vendored package lacked a valid 64-hex-digit SHA-256 checksum.
    /// Realises: [REQ-0608]
    InvalidSha256ChecksumHex,
    /// Dependency graph or TOML nesting exceeded MAX_AST_DEPTH (64) or formed a cycle.
    /// Realises: [REQ-0608]
    PackageGraphDepthOrCycleExceeded,
    /// Workspace Cargo.toml and Cargo.lock diverged or failed TOCTOU digest parity.
    /// Realises: [REQ-0608]
    ManifestToLockDriftDetected,
    /// LosslessRustCstAnalyzer complexity envelope breached C_CST_01..C_CST_06 bounds.
    /// Realises: [REQ-0608]
    CstConstraintEnvelopeViolation,
    /// Air-gapped `cargo check --workspace --offline --locked` execution returned non-zero.
    /// Realises: [REQ-0608]
    OfflineCargoCheckFailed,
}

impl OfflineLockGovernanceError {
    /// Returns the canonical DEAP compiler diagnostic code (`DEAP-E0804` or `DEAP-E0803`) for this fault.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and DO-330 TQL-4 machine-readable diagnostic traceability
    /// (REQ-0608, SC-BP08-04). Binds every governance rejection to its normative BP-08 error code.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Total Mapping: `CstConstraintEnvelopeViolation` maps to `"DEAP-E0803"`; all ten supply-chain
    ///   and lockfile governance variants map to `"DEAP-E0804"`.
    ///
    /// ## 3. Preconditions
    /// - `self` is any valid `OfflineLockGovernanceError` variant.
    ///
    /// ## 4. Postconditions
    /// - Returns a non-empty `'static` ASCII diagnostic code literal containing zero whitespace.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant-time pattern match.
    /// - Space Complexity: O(1) static string reference.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: `const fn` callable in compile-time and runtime contexts.
    /// - State: Pure projection over `self`.
    /// - Action: Exhaustive `match` guarantees total coverage without panicking.
    /// - Resource: Zero heap allocation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [AirGappedCargoLockGovernor/diagnostic_code] | [REQ-0608] | SysML v2: part def AirGappedCargoLockGovernor
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::CstConstraintEnvelopeViolation => "DEAP-E0803",
            Self::LockPolicyDisabled
            | Self::ManifestSizeCeilingExceeded
            | Self::MalformedTomlOrLockSyntax
            | Self::UnpinnedGitOrWildcardSpecifier
            | Self::ForbiddenSemanticDependencyDetected
            | Self::MissingRequiredCstDependency
            | Self::InvalidSha256ChecksumHex
            | Self::PackageGraphDepthOrCycleExceeded
            | Self::ManifestToLockDriftDetected
            | Self::OfflineCargoCheckFailed => "DEAP-E0804",
        }
    }

    /// Returns a single-line, zero-ANSI ASCII diagnostic description for this governance error.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and DO-330 TQL-4 deterministic NDJSON diagnostic serialization
    /// standards (REQ-0608, SC-BP08-04). Guarantees zero embedded `0x0A` newline or `0x1B` ANSI bytes.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Single-Line ASCII Invariant: Returned `&'static str` contains printable ASCII bytes only,
    ///   with `0` newline (`\n`) bytes and `0` escape (`\x1b`) bytes.
    ///
    /// ## 3. Preconditions
    /// - `self` is any valid `OfflineLockGovernanceError` variant.
    ///
    /// ## 4. Postconditions
    /// - Returns a human- and machine-auditable `'static` summary message corresponding to `self`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant-time branch lookup.
    /// - Space Complexity: O(1) static read-only data reference.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: `const fn` returning `'static str`.
    /// - State: Stateless read-only lookup.
    /// - Action: Exhaustive `match` over all 11 variants; never panics.
    /// - Resource: Zero heap allocation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [AirGappedCargoLockGovernor/summary_message] | [REQ-0608] | SysML v2: part def AirGappedCargoLockGovernor
    pub const fn summary_message(&self) -> &'static str {
        match self {
            Self::LockPolicyDisabled => "air-gapped offline or vendor lock policy is disabled",
            Self::ManifestSizeCeilingExceeded => {
                "Cargo manifest or lockfile exceeded configured byte ceiling"
            }
            Self::MalformedTomlOrLockSyntax => {
                "malformed UTF-8 or TOML syntax in Cargo.toml, Cargo.lock, or vendor config"
            }
            Self::UnpinnedGitOrWildcardSpecifier => {
                "unpinned git dependency or wildcard SemVer specifier rejected"
            }
            Self::ForbiddenSemanticDependencyDetected => {
                "forbidden heavyweight semantic or compiler-internal dependency detected"
            }
            Self::MissingRequiredCstDependency => {
                "required pinned CST or Rust codegen dependency missing from Cargo.lock"
            }
            Self::InvalidSha256ChecksumHex => {
                "registry or vendored crate missing valid 64-hex-digit SHA-256 checksum"
            }
            Self::PackageGraphDepthOrCycleExceeded => {
                "dependency graph or syntax nesting exceeded MAX_AST_DEPTH (64) or cycled"
            }
            Self::ManifestToLockDriftDetected => {
                "Cargo.toml to Cargo.lock package or SHA-256 snapshot drift detected"
            }
            Self::CstConstraintEnvelopeViolation => {
                "LosslessRustCstAnalyzer complexity envelope violated C_CST_01..C_CST_06 bounds"
            }
            Self::OfflineCargoCheckFailed => {
                "air-gapped cargo check --workspace --offline --locked verification failed"
            }
        }
    }
}

impl fmt::Display for OfflineLockGovernanceError {
    /// Formats the error as `<DEAP-CODE>: <single-line summary>` without embedded newlines or ANSI escapes.
    /// Realises: [REQ-0608]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.diagnostic_code(), self.summary_message())
    }
}

impl std::error::Error for OfflineLockGovernanceError {}

/// Quote lexer states used while scanning TOML statements without mutable boolean flags.
/// Realises: [REQ-0608]
#[derive(Clone, Copy, PartialEq, Eq)]
enum TomlQuoteMode {
    /// Scanner is outside any string literal.
    /// Realises: [REQ-0608]
    Unquoted,
    /// Scanner is inside a double-quoted basic TOML string.
    /// Realises: [REQ-0608]
    Double,
    /// Scanner just consumed a backslash inside a double-quoted TOML string.
    /// Realises: [REQ-0608]
    DoubleEscaped,
    /// Scanner is inside a single-quoted literal TOML string.
    /// Realises: [REQ-0608]
    Single,
}

/// Active section categories tracked while scanning `Cargo.toml` statements.
/// Realises: [REQ-0608]
#[derive(Clone, Copy, PartialEq, Eq)]
enum ManifestSection {
    /// Non-dependency metadata, lints, or profile section.
    /// Realises: [REQ-0608]
    Other,
    /// Top-level `[workspace]` table declaring workspace members.
    /// Realises: [REQ-0608]
    Workspace,
    /// Top-level `[package]` table declaring a crate name.
    /// Realises: [REQ-0608]
    Package,
    /// Dependency table (`[dependencies]`, `[workspace.dependencies]`, etc.) mapping crate keys.
    /// Realises: [REQ-0608]
    DependencyList,
    /// Expanded single-dependency table such as `[dependencies.foo]`.
    /// Realises: [REQ-0608]
    NamedDependency,
}

/// Parsed workspace members and declared dependency package names extracted from `Cargo.toml`.
/// Realises: [REQ-0608]
struct ParsedManifestMeta {
    /// Normalized workspace member crate names declared in `Cargo.toml`.
    /// Realises: [REQ-0608]
    workspace_members: BTreeSet<String>,
    /// Declared dependency package names that must resolve in `Cargo.lock`.
    /// Realises: [REQ-0608]
    declared_deps: BTreeSet<String>,
}

/// Intermediate `[[package]]` accumulator populated while scanning `Cargo.lock` statements.
/// Realises: [REQ-0608]
#[derive(Default)]
struct RawLockPackage {
    /// Parsed `name` attribute of the `[[package]]` block.
    /// Realises: [REQ-0608]
    name: Option<String>,
    /// Parsed `version` attribute of the `[[package]]` block.
    /// Realises: [REQ-0608]
    version: Option<String>,
    /// Optional `source` URI attribute of the `[[package]]` block.
    /// Realises: [REQ-0608]
    source: Option<String>,
    /// Optional `checksum` hex string attribute of the `[[package]]` block.
    /// Realises: [REQ-0608]
    checksum: Option<String>,
    /// Raw dependency specifier strings listed under `dependencies = [...]`.
    /// Realises: [REQ-0608]
    raw_dependencies: Vec<String>,
}

/// Parses and validates an immutable `Cargo.toml` and `Cargo.lock` snapshot under air-gapped governance.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C air-gapped tool
/// qualification mandates (REQ-0608, FLS `fls_8tsynkj2cufj`, SC-BP08-01, SC-BP08-03). Establishes
/// deterministic SHA-256 manifest/lockfile digests and `BTreeMap`-ordered package records prior to
/// `LosslessRustCstAnalyzer` CST parsing.
///
/// ## 2. Invariants & Mathematical Properties
/// - Byte-Ceiling Guard: Rejects `max_lock_bytes == 0`, `cargo_lock_bytes.len() > max_lock_bytes`,
///   `cargo_toml_bytes.len() > MAX_MANIFEST_BYTES`, or `vendor_cfg_bytes` exceeding `MAX_MANIFEST_BYTES`
///   using `.get(...)` bounds probes and `checked_add` / `saturating_add` totals.
/// - Supply-Chain Purity: Rejects `offline = false`, disabled policy flags, wildcard `*` versions,
///   `git` dependency sources, truncated/non-hex SHA-256 checksums, forbidden semantic crates
///   (`ra_ap_hir`, `ra_ap_ide`, `rustc_private`, `dylint`), and manifest-to-lock package drift.
/// - Complexity Compliance: Executes in `<= 46` executable SLOC with `4` formal parameters,
///   `0` mutable boolean flags, and compound predicate arity `<= 3`.
///
/// ## 3. Preconditions
/// - Caller supplies raw `Cargo.toml` bytes, `Cargo.lock` bytes, optional `.cargo/config.toml`
///   vendor configuration bytes, and a positive `max_lock_bytes` size ceiling.
///
/// ## 4. Postconditions
/// - Returns `Ok(LockManifestSnapshot)` with 32-byte SHA-256 digests, lexically keyed
///   `BTreeMap<String, LockedPackageRecord>`, `workspace_members`, and active offline/vendor flags.
/// - Returns `Err(OfflineLockGovernanceError)` fail-closed on any size, syntax, or policy violation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O((|M| + |L|) log P) where |M| and |L| are manifest/lock byte sizes and P is
///   the number of `[[package]]` entries.
/// - Space Complexity: O(|M| + |L|) bounded strictly by `MAX_MANIFEST_BYTES` and `max_lock_bytes`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Accepts borrowed byte slices; returns typed `OfflineLockGovernanceError` with zero panics.
/// - State: Pure total function performing zero filesystem or global state mutations.
/// - Action: Short-circuits immediately on oversized inputs, malformed TOML, or unpinned dependencies.
/// - Resource: Enforces `MAX_AST_DEPTH = 64` delimiter nesting cap and checked byte totals.
///
/// ## 7. Realization & Traceability
/// Realises: [AirGappedCargoLockGovernor/parse_locked_manifest_snapshot] | [REQ-0608] | SysML v2: part def AirGappedCargoLockGovernor
pub fn parse_locked_manifest_snapshot(
    cargo_toml_bytes: &[u8],
    cargo_lock_bytes: &[u8],
    vendor_cfg_bytes: Option<&[u8]>,
    max_lock_bytes: usize,
) -> Result<LockManifestSnapshot, OfflineLockGovernanceError> {
    if max_lock_bytes == 0 || cargo_lock_bytes.get(max_lock_bytes).is_some() {
        return Err(OfflineLockGovernanceError::ManifestSizeCeilingExceeded);
    }
    if cargo_toml_bytes.get(MAX_MANIFEST_BYTES).is_some() {
        return Err(OfflineLockGovernanceError::ManifestSizeCeilingExceeded);
    }
    let v_len = vendor_cfg_bytes.map_or(0usize, <[u8]>::len);
    let total = cargo_toml_bytes
        .len()
        .checked_add(cargo_lock_bytes.len())
        .and_then(|n| n.checked_add(v_len))
        .ok_or(OfflineLockGovernanceError::ManifestSizeCeilingExceeded)?;
    if total > max_lock_bytes.saturating_add(MAX_MANIFEST_BYTES.saturating_mul(2)) {
        return Err(OfflineLockGovernanceError::ManifestSizeCeilingExceeded);
    }
    let toml_slice = cargo_toml_bytes
        .get(..cargo_toml_bytes.len())
        .ok_or(OfflineLockGovernanceError::ManifestSizeCeilingExceeded)?;
    let lock_slice = cargo_lock_bytes
        .get(..max_lock_bytes.min(cargo_lock_bytes.len()))
        .ok_or(OfflineLockGovernanceError::ManifestSizeCeilingExceeded)?;
    let vendored_mode = validate_vendor_config(vendor_cfg_bytes)?;
    let (toml_str, lock_str) = decode_non_empty_utf8_pair(toml_slice, lock_slice)?;
    let meta = parse_cargo_toml_manifest(toml_str)?;
    let packages = parse_cargo_lock_packages(lock_str, &meta.workspace_members, vendored_mode)?;
    verify_manifest_to_lock_presence(&meta, &packages)?;
    Ok(LockManifestSnapshot {
        cargo_toml_sha256: crate::digest::compute_sha256_bytes(toml_slice),
        cargo_lock_sha256: crate::digest::compute_sha256_bytes(lock_slice),
        packages,
        workspace_members: meta.workspace_members,
        offline_locked_enforced: true,
        vendor_bundle_supported: true,
    })
}

/// Validates that both manifest and lockfile byte slices are valid non-empty UTF-8 strings.
/// Realises: [REQ-0608]
fn decode_non_empty_utf8_pair<'a>(
    toml_slice: &'a [u8],
    lock_slice: &'a [u8],
) -> Result<(&'a str, &'a str), OfflineLockGovernanceError> {
    let toml_str = std::str::from_utf8(toml_slice)
        .map_err(|_| OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?;
    let lock_str = std::str::from_utf8(lock_slice)
        .map_err(|_| OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?;
    if toml_str.trim().is_empty() || lock_str.trim().is_empty() {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    Ok((toml_str, lock_str))
}

/// Converts a single ASCII hexadecimal byte into its 4-bit numeric value.
/// Realises: [REQ-0608]
fn decode_hex_nibble(byte: u8) -> Result<u8, OfflineLockGovernanceError> {
    match byte {
        b'0'..=b'9' => Ok(byte.saturating_sub(b'0')),
        b'a'..=b'f' => Ok(byte.saturating_sub(b'a').saturating_add(10)),
        b'A'..=b'F' => Ok(byte.saturating_sub(b'A').saturating_add(10)),
        _ => Err(OfflineLockGovernanceError::InvalidSha256ChecksumHex),
    }
}

/// Decodes a 64-character hexadecimal SHA-256 string into an immutable `[u8; 32]` array.
/// Realises: [REQ-0608]
fn decode_sha256_hex(hex_str: &str) -> Result<[u8; 32], OfflineLockGovernanceError> {
    if hex_str.len() != 64 {
        return Err(OfflineLockGovernanceError::InvalidSha256ChecksumHex);
    }
    let raw = hex_str.as_bytes();
    let mut digest = [0u8; 32];
    for idx in 0..32usize {
        let hi_pos = idx.saturating_mul(2);
        let lo_pos = hi_pos.saturating_add(1);
        let hi_byte = *raw
            .get(hi_pos)
            .ok_or(OfflineLockGovernanceError::InvalidSha256ChecksumHex)?;
        let lo_byte = *raw
            .get(lo_pos)
            .ok_or(OfflineLockGovernanceError::InvalidSha256ChecksumHex)?;
        let combined = (decode_hex_nibble(hi_byte)? << 4) | decode_hex_nibble(lo_byte)?;
        let slot = digest
            .get_mut(idx)
            .ok_or(OfflineLockGovernanceError::InvalidSha256ChecksumHex)?;
        *slot = combined;
    }
    Ok(digest)
}

/// Validates that an optional package checksum is present and consists of 64 ASCII hex digits.
/// Realises: [REQ-0608]
fn validate_sha256_checksum(checksum: Option<&str>) -> Result<String, OfflineLockGovernanceError> {
    let text = checksum.ok_or(OfflineLockGovernanceError::InvalidSha256ChecksumHex)?;
    let _verified = decode_sha256_hex(text)?;
    Ok(text.to_ascii_lowercase())
}

/// Normalizes Cargo crate names by mapping hyphens to underscores for canonical equivalence checks.
/// Realises: [REQ-0608]
fn normalize_crate_key(name: &str) -> String {
    name.trim()
        .chars()
        .map(|ch| if ch == '-' { '_' } else { ch })
        .collect()
}

/// Checks whether a normalized crate identifier belongs to the forbidden semantic or compiler-internal set.
/// Realises: [REQ-0608]
fn is_forbidden_semantic_crate(name: &str) -> bool {
    let norm = normalize_crate_key(name);
    matches!(
        norm.as_str(),
        "ra_ap_hir"
            | "ra_ap_hir_def"
            | "ra_ap_hir_ty"
            | "ra_ap_hir_expand"
            | "ra_ap_ide"
            | "ra_ap_ide_db"
            | "ra_ap_ide_assists"
            | "ra_ap_ide_diagnostics"
            | "rustc_private"
            | "dylint"
            | "dylint_linting"
            | "dylint_internal"
    )
}

/// Advances the TOML string quote lexer mode for a single byte.
/// Realises: [REQ-0608]
fn advance_quote_mode(mode: TomlQuoteMode, byte: u8) -> TomlQuoteMode {
    match (mode, byte) {
        (TomlQuoteMode::Unquoted, b'"') => TomlQuoteMode::Double,
        (TomlQuoteMode::Unquoted, b'\'') => TomlQuoteMode::Single,
        (TomlQuoteMode::Double, b'\\') => TomlQuoteMode::DoubleEscaped,
        (TomlQuoteMode::Double, b'"') => TomlQuoteMode::Unquoted,
        (TomlQuoteMode::DoubleEscaped, _) => TomlQuoteMode::Double,
        (TomlQuoteMode::Single, b'\'') => TomlQuoteMode::Unquoted,
        (current, _) => current,
    }
}

/// Strips trailing TOML comments on a single line while enforcing balanced quotes and `MAX_AST_DEPTH`.
/// Realises: [REQ-0608]
fn strip_toml_line_and_track_depth(
    line: &str,
    start_depth: usize,
) -> Result<(&str, usize), OfflineLockGovernanceError> {
    let mut mode = TomlQuoteMode::Unquoted;
    let mut depth = start_depth;
    let mut end_pos = line.len();
    for (idx, &byte) in line.as_bytes().iter().enumerate() {
        if mode == TomlQuoteMode::Unquoted {
            if byte == b'#' {
                end_pos = idx;
                break;
            }
            if matches!(byte, b'[' | b'{') {
                depth = depth
                    .checked_add(1)
                    .ok_or(OfflineLockGovernanceError::PackageGraphDepthOrCycleExceeded)?;
                if depth > MAX_AST_DEPTH {
                    return Err(OfflineLockGovernanceError::PackageGraphDepthOrCycleExceeded);
                }
            } else if matches!(byte, b']' | b'}') {
                if depth == 0 {
                    return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
                }
                depth = depth.saturating_sub(1);
            }
        }
        mode = advance_quote_mode(mode, byte);
    }
    if mode != TomlQuoteMode::Unquoted {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    let slice = line
        .get(..end_pos)
        .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?;
    Ok((slice.trim(), depth))
}

/// Accumulates stripped TOML lines into complete logical statements with verified delimiter balance.
/// Realises: [REQ-0608]
fn collect_toml_statements(source: &str) -> Result<Vec<String>, OfflineLockGovernanceError> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    for raw_line in source.lines() {
        let (cleaned, next_depth) = strip_toml_line_and_track_depth(raw_line, depth)?;
        if current.is_empty() && cleaned.starts_with('[') && next_depth != 0 {
            return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
        }
        depth = next_depth;
        if cleaned.is_empty() && current.is_empty() {
            continue;
        }
        if !cleaned.is_empty() {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(cleaned);
        }
        if depth == 0 && !current.is_empty() {
            statements.push(std::mem::take(&mut current));
        }
    }
    if depth != 0 || !current.is_empty() {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    Ok(statements)
}

/// Checks whether a TOML key token contains only legal bare, dotted, or quoted identifier bytes.
/// Realises: [REQ-0608]
fn is_valid_toml_key_token(key: &str) -> bool {
    !key.is_empty()
        && key.as_bytes().iter().all(|&b| {
            matches!(
                b,
                b'a'..=b'z'
                    | b'A'..=b'Z'
                    | b'0'..=b'9'
                    | b'_'
                    | b'-'
                    | b'.'
                    | b'"'
                    | b'\''
            )
        })
}

/// Checks whether a TOML right-hand value token is a syntactically well-formed scalar, array, or inline table.
/// Realises: [REQ-0608]
fn is_valid_toml_value_token(val: &str) -> bool {
    if matches!(val, "true" | "false") {
        return true;
    }
    let is_double = val.len() >= 2 && val.starts_with('"') && val.ends_with('"');
    let is_single = val.len() >= 2 && val.starts_with('\'') && val.ends_with('\'');
    if is_double || is_single {
        return true;
    }
    let is_array = val.starts_with('[') && val.ends_with(']');
    let is_table = val.starts_with('{') && val.ends_with('}');
    if is_array || is_table {
        return true;
    }
    !val.is_empty()
        && val
            .as_bytes()
            .iter()
            .all(|&b| matches!(b, b'0'..=b'9' | b'_'))
}

/// Splits a logical TOML assignment statement at its top-level `=` separator into validated `(key, value)` slices.
/// Realises: [REQ-0608]
fn split_toml_key_value(stmt: &str) -> Result<(&str, &str), OfflineLockGovernanceError> {
    let mut mode = TomlQuoteMode::Unquoted;
    let mut depth = 0usize;
    let mut eq_idx = None;
    for (idx, &byte) in stmt.as_bytes().iter().enumerate() {
        if mode == TomlQuoteMode::Unquoted {
            if matches!(byte, b'[' | b'{') {
                depth = depth.saturating_add(1);
            } else if matches!(byte, b']' | b'}') {
                depth = depth.saturating_sub(1);
            } else if byte == b'=' && depth == 0 {
                eq_idx = Some(idx);
                break;
            }
        }
        mode = advance_quote_mode(mode, byte);
    }
    let pos = eq_idx.ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?;
    let key = stmt
        .get(..pos)
        .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?
        .trim();
    let val = stmt
        .get(pos.saturating_add(1)..)
        .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?
        .trim();
    if !is_valid_toml_key_token(key) || !is_valid_toml_value_token(val) {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    Ok((key, val))
}

/// Extracts the unquoted inner content of a single- or double-quoted TOML string literal.
/// Realises: [REQ-0608]
fn parse_toml_string_literal(raw: &str) -> Result<String, OfflineLockGovernanceError> {
    let trimmed = raw.trim();
    let is_double = trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"');
    let is_single = trimmed.len() >= 2 && trimmed.starts_with('\'') && trimmed.ends_with('\'');
    if !is_double && !is_single {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    let inner = trimmed
        .get(1..trimmed.len().saturating_sub(1))
        .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?;
    Ok(inner.to_string())
}

/// Splits the interior of a TOML array or inline table on top-level commas outside quotes and nested delimiters.
/// Realises: [REQ-0608]
fn split_top_level_csv(body: &str) -> Result<Vec<&str>, OfflineLockGovernanceError> {
    let mut items = Vec::new();
    let mut mode = TomlQuoteMode::Unquoted;
    let mut depth = 0usize;
    let mut start = 0usize;
    for (idx, &byte) in body.as_bytes().iter().enumerate() {
        if mode == TomlQuoteMode::Unquoted {
            if matches!(byte, b'[' | b'{') {
                depth = depth.saturating_add(1);
            } else if matches!(byte, b']' | b'}') {
                depth = depth.saturating_sub(1);
            } else if byte == b',' && depth == 0 {
                let seg = body
                    .get(start..idx)
                    .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?
                    .trim();
                if !seg.is_empty() {
                    items.push(seg);
                }
                start = idx.saturating_add(1);
            }
        }
        mode = advance_quote_mode(mode, byte);
    }
    let tail = body
        .get(start..)
        .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?
        .trim();
    if !tail.is_empty() {
        items.push(tail);
    }
    Ok(items)
}

/// Parses a TOML string array literal (`["a", "b"]`) into an ordered vector of strings.
/// Realises: [REQ-0608]
fn parse_toml_string_array(raw: &str) -> Result<Vec<String>, OfflineLockGovernanceError> {
    let trimmed = raw.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    let inner = trimmed
        .get(1..trimmed.len().saturating_sub(1))
        .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?;
    let mut out = Vec::new();
    for entry in split_top_level_csv(inner)? {
        out.push(parse_toml_string_literal(entry)?);
    }
    Ok(out)
}

/// Parses a TOML inline table (`{ version = "1.0", workspace = true }`) into a `BTreeMap`.
/// Realises: [REQ-0608]
fn parse_inline_table_pairs(
    raw: &str,
) -> Result<BTreeMap<String, String>, OfflineLockGovernanceError> {
    let trimmed = raw.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    let inner = trimmed
        .get(1..trimmed.len().saturating_sub(1))
        .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?;
    let mut pairs = BTreeMap::new();
    for entry in split_top_level_csv(inner)? {
        let (k, v) = split_toml_key_value(entry)?;
        pairs.insert(k.to_string(), v.to_string());
    }
    Ok(pairs)
}

/// Validates a Cargo.toml dependency version requirement, rejecting wildcards and empty strings.
/// Realises: [REQ-0608]
fn validate_manifest_version_req(ver: &str) -> Result<(), OfflineLockGovernanceError> {
    let trimmed = ver.trim();
    if trimmed.is_empty() || trimmed.contains('*') {
        return Err(OfflineLockGovernanceError::UnpinnedGitOrWildcardSpecifier);
    }
    if !trimmed.bytes().any(|b| b.is_ascii_digit()) {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    Ok(())
}

/// Validates that a `Cargo.lock` package version is an exact three-component SemVer string without wildcards.
/// Realises: [REQ-0608]
fn validate_lock_exact_semver(ver: &str) -> Result<(), OfflineLockGovernanceError> {
    if ver.contains('*') {
        return Err(OfflineLockGovernanceError::UnpinnedGitOrWildcardSpecifier);
    }
    let valid_chars = ver
        .as_bytes()
        .iter()
        .all(|&b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'.' | b'-' | b'+'));
    if ver.is_empty() || !valid_chars {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    let core = ver
        .split(['-', '+'])
        .next()
        .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?;
    let mut parts = core.split('.');
    for _ in 0..3usize {
        let seg = parts
            .next()
            .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?;
        if seg.is_empty() || !seg.bytes().all(|b| b.is_ascii_digit()) {
            return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
        }
    }
    if parts.next().is_some() {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    Ok(())
}

/// Validates optional `.cargo/config.toml` vendor configuration bytes and reports whether vendoring is active.
/// Realises: [REQ-0608]
fn validate_vendor_config(
    vendor_cfg_bytes: Option<&[u8]>,
) -> Result<bool, OfflineLockGovernanceError> {
    let Some(raw) = vendor_cfg_bytes else {
        return Ok(false);
    };
    if raw.get(MAX_MANIFEST_BYTES).is_some() {
        return Err(OfflineLockGovernanceError::ManifestSizeCeilingExceeded);
    }
    let cfg_str = std::str::from_utf8(raw)
        .map_err(|_| OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?;
    let mut replace_target: Option<String> = None;
    let mut vendor_dir: Option<String> = None;
    for stmt in collect_toml_statements(cfg_str)? {
        if stmt.starts_with('[') {
            let _hdr = extract_toml_header(&stmt)?;
            continue;
        }
        let (key, val) = split_toml_key_value(&stmt)?;
        check_policy_flag_not_disabled(key, val)?;
        if key == "replace-with" {
            let target = parse_toml_string_literal(val)?;
            if target.trim().is_empty() {
                return Err(OfflineLockGovernanceError::LockPolicyDisabled);
            }
            replace_target = Some(target);
        } else if matches!(key, "directory" | "local-registry") {
            let dir = parse_toml_string_literal(val)?;
            if dir.trim().is_empty() {
                return Err(OfflineLockGovernanceError::LockPolicyDisabled);
            }
            vendor_dir = Some(dir);
        }
    }
    if replace_target.is_some() && vendor_dir.is_none() {
        return Err(OfflineLockGovernanceError::LockPolicyDisabled);
    }
    Ok(replace_target.is_some())
}

/// Extracts and validates the non-empty section path from a `[section]` or `[[array_section]]` statement.
/// Realises: [REQ-0608]
fn extract_toml_header(stmt: &str) -> Result<(&str, bool), OfflineLockGovernanceError> {
    if stmt.starts_with("[[") && stmt.ends_with("]]") && stmt.len() >= 5 {
        let inner = stmt
            .get(2..stmt.len().saturating_sub(2))
            .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?
            .trim();
        if inner.is_empty() || inner.starts_with('[') || inner.ends_with(']') {
            return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
        }
        return Ok((inner, true));
    }
    if stmt.starts_with('[') && stmt.ends_with(']') && stmt.len() >= 3 {
        let inner = stmt
            .get(1..stmt.len().saturating_sub(1))
            .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?
            .trim();
        if inner.is_empty() || inner.starts_with('[') || inner.ends_with(']') {
            return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
        }
        return Ok((inner, false));
    }
    Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)
}

/// Rejects TOML assignments that disable offline/vendor lock governance or enable `rustc_private`.
/// Realises: [REQ-0608]
fn check_policy_flag_not_disabled(key: &str, val: &str) -> Result<(), OfflineLockGovernanceError> {
    let is_policy_key = matches!(
        key,
        "offline" | "offline_locked_enforced" | "vendor_bundle_supported"
    );
    if is_policy_key && val == "false" {
        return Err(OfflineLockGovernanceError::LockPolicyDisabled);
    }
    if key == "rustc_private" && val == "true" {
        return Err(OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected);
    }
    Ok(())
}

/// Classifies a `Cargo.toml` table header into its manifest section kind and optional named dependency target.
/// Realises: [REQ-0608]
fn classify_manifest_section(
    header: &str,
) -> Result<(ManifestSection, Option<String>), OfflineLockGovernanceError> {
    if header == "workspace" {
        return Ok((ManifestSection::Workspace, None));
    }
    if header == "package" {
        return Ok((ManifestSection::Package, None));
    }
    let direct_dep_list = matches!(
        header,
        "dependencies" | "workspace.dependencies" | "dev-dependencies" | "build-dependencies"
    );
    let target_dep_list = header.ends_with(".dependencies")
        || header.ends_with(".dev-dependencies")
        || header.ends_with(".build-dependencies");
    if direct_dep_list || target_dep_list {
        return Ok((ManifestSection::DependencyList, None));
    }
    for prefix in [
        "workspace.dependencies.",
        "dependencies.",
        "dev-dependencies.",
        "build-dependencies.",
    ] {
        if let Some(rest) = header.strip_prefix(prefix) {
            let dep_name = rest.trim_matches(['"', '\'']).trim();
            if dep_name.is_empty() {
                return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
            }
            if is_forbidden_semantic_crate(dep_name) {
                return Err(OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected);
            }
            return Ok((ManifestSection::NamedDependency, Some(dep_name.to_string())));
        }
    }
    Ok((ManifestSection::Other, None))
}

/// Validates a `Cargo.toml` dependency specification entry and returns its resolved package name.
/// Realises: [REQ-0608]
fn validate_dependency_entry(
    dep_key: &str,
    raw_val: &str,
) -> Result<String, OfflineLockGovernanceError> {
    let clean_key = dep_key.trim_matches(['"', '\'']).trim();
    if is_forbidden_semantic_crate(clean_key) {
        return Err(OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected);
    }
    if raw_val.starts_with('"') || raw_val.starts_with('\'') {
        let ver = parse_toml_string_literal(raw_val)?;
        validate_manifest_version_req(&ver)?;
        return Ok(clean_key.to_string());
    }
    if raw_val.starts_with('{') {
        let table = parse_inline_table_pairs(raw_val)?;
        if table.contains_key("git") {
            return Err(OfflineLockGovernanceError::UnpinnedGitOrWildcardSpecifier);
        }
        let pkg_name = match table.get("package") {
            Some(lit) => parse_toml_string_literal(lit)?,
            None => clean_key.to_string(),
        };
        if is_forbidden_semantic_crate(&pkg_name) {
            return Err(OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected);
        }
        if let Some(ver_lit) = table.get("version") {
            validate_manifest_version_req(&parse_toml_string_literal(ver_lit)?)?;
        } else {
            let ws_true = table.get("workspace").map(String::as_str) == Some("true");
            if !ws_true && !table.contains_key("path") {
                return Err(OfflineLockGovernanceError::UnpinnedGitOrWildcardSpecifier);
            }
        }
        return Ok(pkg_name);
    }
    Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)
}

/// Parses `[workspace] members = [...]` paths and inserts their leaf crate names into `members`.
/// Realises: [REQ-0608]
fn insert_workspace_members(
    val: &str,
    members: &mut BTreeSet<String>,
) -> Result<(), OfflineLockGovernanceError> {
    for member_path in parse_toml_string_array(val)? {
        let leaf = member_path
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("")
            .trim();
        if leaf.is_empty() || leaf.contains('*') {
            return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
        }
        members.insert(leaf.to_string());
    }
    Ok(())
}

/// Applies a single `key = value` statement inside the active `Cargo.toml` section to `ParsedManifestMeta`.
/// Realises: [REQ-0608]
fn apply_manifest_kv(
    section: ManifestSection,
    key: &str,
    val: &str,
    meta: &mut ParsedManifestMeta,
) -> Result<(), OfflineLockGovernanceError> {
    check_policy_flag_not_disabled(key, val)?;
    match section {
        ManifestSection::Workspace if key == "members" => {
            insert_workspace_members(val, &mut meta.workspace_members)?;
        }
        ManifestSection::Package if key == "name" => {
            let pkg_name = parse_toml_string_literal(val)?;
            if pkg_name.trim().is_empty() {
                return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
            }
            meta.workspace_members.insert(pkg_name);
        }
        ManifestSection::DependencyList => {
            meta.declared_deps
                .insert(validate_dependency_entry(key, val)?);
        }
        ManifestSection::NamedDependency => {
            if key == "git" {
                return Err(OfflineLockGovernanceError::UnpinnedGitOrWildcardSpecifier);
            }
            if key == "version" {
                validate_manifest_version_req(&parse_toml_string_literal(val)?)?;
            } else if key == "package" {
                let pkg = parse_toml_string_literal(val)?;
                if is_forbidden_semantic_crate(&pkg) {
                    return Err(OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected);
                }
                meta.declared_deps.insert(pkg);
            }
        }
        _ => {}
    }
    Ok(())
}

/// Parses `Cargo.toml` statements to extract workspace members, declared dependencies, and policy invariants.
/// Realises: [REQ-0608]
fn parse_cargo_toml_manifest(
    toml_str: &str,
) -> Result<ParsedManifestMeta, OfflineLockGovernanceError> {
    let mut section = ManifestSection::Other;
    let mut meta = ParsedManifestMeta {
        workspace_members: BTreeSet::new(),
        declared_deps: BTreeSet::new(),
    };
    for stmt in collect_toml_statements(toml_str)? {
        if stmt.starts_with('[') {
            let (header, is_array_table) = extract_toml_header(&stmt)?;
            if is_array_table {
                section = ManifestSection::Other;
            } else {
                let (next_sec, named_dep) = classify_manifest_section(header)?;
                if let Some(dep_pkg) = named_dep {
                    meta.declared_deps.insert(dep_pkg);
                }
                section = next_sec;
            }
        } else {
            let (key, val) = split_toml_key_value(&stmt)?;
            apply_manifest_kv(section, key, val, &mut meta)?;
        }
    }
    Ok(meta)
}

/// Applies a single `key = value` statement inside a `Cargo.lock` `[[package]]` block.
/// Realises: [REQ-0608]
fn apply_lock_package_kv(
    pkg: &mut RawLockPackage,
    key: &str,
    val: &str,
) -> Result<(), OfflineLockGovernanceError> {
    match key {
        "name" => pkg.name = Some(parse_toml_string_literal(val)?),
        "version" => pkg.version = Some(parse_toml_string_literal(val)?),
        "source" => pkg.source = Some(parse_toml_string_literal(val)?),
        "checksum" => pkg.checksum = Some(parse_toml_string_literal(val)?),
        "dependencies" => pkg.raw_dependencies = parse_toml_string_array(val)?,
        _ => {}
    }
    Ok(())
}

/// Collects raw `[[package]]` entries from `Cargo.lock` statements while validating top-level syntax.
/// Realises: [REQ-0608]
fn collect_raw_lock_packages(
    lock_str: &str,
) -> Result<Vec<RawLockPackage>, OfflineLockGovernanceError> {
    let mut packages = Vec::new();
    let mut active_pkg: Option<RawLockPackage> = None;
    for stmt in collect_toml_statements(lock_str)? {
        if stmt.starts_with('[') {
            if let Some(done) = active_pkg.take() {
                packages.push(done);
            }
            let (header, is_array_table) = extract_toml_header(&stmt)?;
            if is_array_table && header == "package" {
                active_pkg = Some(RawLockPackage::default());
            }
        } else {
            let (key, val) = split_toml_key_value(&stmt)?;
            check_policy_flag_not_disabled(key, val)?;
            if let Some(pkg) = active_pkg.as_mut() {
                apply_lock_package_kv(pkg, key, val)?;
            }
        }
    }
    if let Some(done) = active_pkg.take() {
        packages.push(done);
    }
    if packages.is_empty() {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    Ok(packages)
}

/// Selects `VendoredBundle` when cargo vendor replacement is active, or `RegistryLocked` otherwise.
/// Realises: [REQ-0608]
const fn external_package_source_kind(vendored_mode: bool) -> PackageSourceKind {
    if vendored_mode {
        PackageSourceKind::VendoredBundle
    } else {
        PackageSourceKind::RegistryLocked
    }
}

/// Determines provenance kind and validates the 64-hex SHA-256 checksum for a raw locked package.
/// Realises: [REQ-0608]
fn classify_package_source_and_checksum(
    name: &str,
    source: Option<&str>,
    checksum: Option<&str>,
    workspace_members: &BTreeSet<String>,
    vendored_mode: bool,
) -> Result<(PackageSourceKind, Option<String>), OfflineLockGovernanceError> {
    let ext_kind = external_package_source_kind(vendored_mode);
    match source {
        Some(src) if src.starts_with("git+") => {
            Err(OfflineLockGovernanceError::UnpinnedGitOrWildcardSpecifier)
        }
        Some(src) if src.starts_with("vendored+") => Ok((
            PackageSourceKind::VendoredBundle,
            Some(validate_sha256_checksum(checksum)?),
        )),
        Some(src) if src.starts_with("registry+") || src.starts_with("sparse+") => {
            Ok((ext_kind, Some(validate_sha256_checksum(checksum)?)))
        }
        Some(_) => Err(OfflineLockGovernanceError::UnpinnedGitOrWildcardSpecifier),
        None => {
            let norm = normalize_crate_key(name);
            if workspace_members
                .iter()
                .any(|m| normalize_crate_key(m) == norm)
            {
                let checked = match checksum {
                    Some(cs) => Some(validate_sha256_checksum(Some(cs))?),
                    None => None,
                };
                Ok((PackageSourceKind::WorkspacePath, checked))
            } else {
                Ok((ext_kind, Some(validate_sha256_checksum(checksum)?)))
            }
        }
    }
}

/// Resolves a raw `Cargo.lock` dependency string (`"syn 3.0.6"` or `"quote"`) into a canonical `<crate>@<version>` edge.
/// Realises: [REQ-0608]
fn resolve_lock_dependency_edge(
    raw_dep: &str,
    unique_versions: &BTreeMap<String, Option<String>>,
) -> Result<String, OfflineLockGovernanceError> {
    let mut tokens = raw_dep.split_whitespace();
    let dep_name = tokens
        .next()
        .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?;
    if is_forbidden_semantic_crate(dep_name) {
        return Err(OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected);
    }
    if let Some(dep_ver) = tokens.next() {
        return Ok(format!("{dep_name}@{dep_ver}"));
    }
    if dep_name.contains('@') {
        return Ok(dep_name.to_string());
    }
    if let Some(Some(ver)) = unique_versions.get(dep_name) {
        return Ok(format!("{dep_name}@{ver}"));
    }
    Ok(dep_name.to_string())
}

/// Builds a map of unique package versions across `Cargo.lock` while validating package names and SemVer strings.
/// Realises: [REQ-0608]
fn index_unique_package_versions(
    raw_list: &[RawLockPackage],
) -> Result<BTreeMap<String, Option<String>>, OfflineLockGovernanceError> {
    let mut unique_versions: BTreeMap<String, Option<String>> = BTreeMap::new();
    for raw in raw_list {
        let name = raw
            .name
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?;
        let version = raw
            .version
            .as_deref()
            .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?;
        if is_forbidden_semantic_crate(name) {
            return Err(OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected);
        }
        validate_lock_exact_semver(version)?;
        unique_versions
            .entry(name.to_string())
            .and_modify(|slot| *slot = None)
            .or_insert_with(|| Some(version.to_string()));
    }
    Ok(unique_versions)
}

/// Parses and validates all `Cargo.lock` `[[package]]` blocks into a lexically ordered `BTreeMap`.
/// Realises: [REQ-0608]
fn parse_cargo_lock_packages(
    lock_str: &str,
    workspace_members: &BTreeSet<String>,
    vendored_mode: bool,
) -> Result<BTreeMap<String, LockedPackageRecord>, OfflineLockGovernanceError> {
    let raw_list = collect_raw_lock_packages(lock_str)?;
    let unique_versions = index_unique_package_versions(&raw_list)?;
    let mut records = BTreeMap::new();
    for raw in raw_list {
        let name = raw
            .name
            .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?;
        let version = raw
            .version
            .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?;
        let (source_kind, checksum) = classify_package_source_and_checksum(
            &name,
            raw.source.as_deref(),
            raw.checksum.as_deref(),
            workspace_members,
            vendored_mode,
        )?;
        let mut dependencies = BTreeSet::new();
        for dep in &raw.raw_dependencies {
            dependencies.insert(resolve_lock_dependency_edge(dep, &unique_versions)?);
        }
        let key = format!("{name}@{version}");
        if records.contains_key(&key) {
            return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
        }
        records.insert(
            key,
            LockedPackageRecord {
                name,
                version,
                checksum,
                source_kind,
                dependencies,
            },
        );
    }
    Ok(records)
}

/// Verifies that every workspace member and declared dependency in `Cargo.toml` resolves in `Cargo.lock`.
/// Realises: [REQ-0608]
fn verify_manifest_to_lock_presence(
    meta: &ParsedManifestMeta,
    packages: &BTreeMap<String, LockedPackageRecord>,
) -> Result<(), OfflineLockGovernanceError> {
    let locked_names: BTreeSet<String> = packages
        .values()
        .map(|rec| normalize_crate_key(&rec.name))
        .collect();
    for member in &meta.workspace_members {
        if !locked_names.contains(&normalize_crate_key(member)) {
            return Err(OfflineLockGovernanceError::ManifestToLockDriftDetected);
        }
    }
    for dep in &meta.declared_deps {
        if !locked_names.contains(&normalize_crate_key(dep)) {
            return Err(OfflineLockGovernanceError::ManifestToLockDriftDetected);
        }
    }
    Ok(())
}

/// Checks whether a normalized crate or `<crate>@<version>` specifier matches a locked package.
/// Realises: [REQ-0608]
fn matches_crate_spec(spec: &str, pkg_name: &str, pkg_version: &str) -> bool {
    let trimmed = spec.trim();
    if let Some((spec_name, spec_ver)) = trimmed.split_once('@') {
        normalize_crate_key(spec_name) == normalize_crate_key(pkg_name)
            && spec_ver.trim() == pkg_version
    } else {
        normalize_crate_key(trimmed) == normalize_crate_key(pkg_name)
    }
}

/// Checks whether a package or dependency identifier is blocked by built-in or caller-supplied rules.
/// Realises: [REQ-0608]
fn is_blocked_graph_crate(name: &str, version: &str, forbidden_crates: &BTreeSet<String>) -> bool {
    is_forbidden_semantic_crate(name)
        || forbidden_crates
            .iter()
            .any(|spec| matches_crate_spec(spec, name, version))
}

/// Validates a single locked package record for forbidden crates, exact SemVer, and 64-hex SHA-256 digest.
/// Realises: [REQ-0608]
fn validate_locked_package_node(
    key: &str,
    rec: &LockedPackageRecord,
    workspace_members: &BTreeSet<String>,
    forbidden_crates: &BTreeSet<String>,
) -> Result<(), OfflineLockGovernanceError> {
    if is_blocked_graph_crate(&rec.name, &rec.version, forbidden_crates) {
        return Err(OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected);
    }
    if rec.name.contains('*') || rec.name.starts_with("git+") {
        return Err(OfflineLockGovernanceError::UnpinnedGitOrWildcardSpecifier);
    }
    validate_lock_exact_semver(&rec.version)?;
    let expected_key = format!("{}@{}", rec.name.trim(), rec.version);
    if rec.name.trim().is_empty() || key != expected_key {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    match rec.source_kind {
        PackageSourceKind::RegistryLocked | PackageSourceKind::VendoredBundle => {
            let _checked = validate_sha256_checksum(rec.checksum.as_deref())?;
        }
        PackageSourceKind::WorkspacePath => {
            if let Some(cs) = rec.checksum.as_deref() {
                let _checked = validate_sha256_checksum(Some(cs))?;
            }
            let norm_name = normalize_crate_key(&rec.name);
            let is_member = workspace_members
                .iter()
                .any(|m| normalize_crate_key(m) == norm_name);
            if !is_member {
                return Err(if rec.checksum.is_none() {
                    OfflineLockGovernanceError::InvalidSha256ChecksumHex
                } else {
                    OfflineLockGovernanceError::ManifestToLockDriftDetected
                });
            }
        }
    }
    Ok(())
}

/// Verifies workspace member lock parity and exact SHA-256 pinning of all `required_crates`.
/// Realises: [REQ-0608]
fn verify_workspace_and_required_crates(
    snapshot: &LockManifestSnapshot,
    required_crates: &BTreeSet<String>,
    forbidden_crates: &BTreeSet<String>,
) -> Result<(), OfflineLockGovernanceError> {
    for member in &snapshot.workspace_members {
        if is_blocked_graph_crate(member, "", forbidden_crates) {
            return Err(OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected);
        }
        if member.contains('*') || member.starts_with("git+") {
            return Err(OfflineLockGovernanceError::UnpinnedGitOrWildcardSpecifier);
        }
        let norm_member = normalize_crate_key(member);
        if !snapshot
            .packages
            .values()
            .any(|rec| normalize_crate_key(&rec.name) == norm_member)
        {
            return Err(OfflineLockGovernanceError::ManifestToLockDriftDetected);
        }
    }
    for req in required_crates {
        if req.trim().is_empty() {
            return Err(OfflineLockGovernanceError::MissingRequiredCstDependency);
        }
        let matched: Vec<&LockedPackageRecord> = snapshot
            .packages
            .values()
            .filter(|rec| matches_crate_spec(req, &rec.name, &rec.version))
            .collect();
        if matched.is_empty() {
            return Err(OfflineLockGovernanceError::MissingRequiredCstDependency);
        }
        for rec in matched {
            let _checked = validate_sha256_checksum(rec.checksum.as_deref())?;
        }
    }
    Ok(())
}

/// Resolves a `<crate>` and exact SemVer `<version>` pair to its canonical key in `packages`.
/// Realises: [REQ-0608]
fn lookup_versioned_package_key(
    dep_name: &str,
    ver: &str,
    packages: &BTreeMap<String, LockedPackageRecord>,
    by_norm_name: &BTreeMap<String, Vec<String>>,
) -> Result<String, OfflineLockGovernanceError> {
    validate_lock_exact_semver(ver)?;
    let candidate = format!("{dep_name}@{ver}");
    if packages.get(&candidate).is_some() {
        return Ok(candidate);
    }
    if let Some(keys) = by_norm_name.get(&normalize_crate_key(dep_name)) {
        for key in keys {
            if packages.get(key).is_some_and(|rec| rec.version == ver) {
                return Ok(key.clone());
            }
        }
    }
    Err(OfflineLockGovernanceError::ManifestToLockDriftDetected)
}

/// Resolves and validates a single directed dependency edge into a canonical `<crate>@<version>` key.
/// Realises: [REQ-0608]
fn resolve_verified_dependency_target(
    raw_edge: &str,
    packages: &BTreeMap<String, LockedPackageRecord>,
    by_norm_name: &BTreeMap<String, Vec<String>>,
    forbidden_crates: &BTreeSet<String>,
) -> Result<String, OfflineLockGovernanceError> {
    let edge = raw_edge.trim();
    if edge.is_empty() {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    if edge.contains('*') || edge.contains("git+") {
        return Err(OfflineLockGovernanceError::UnpinnedGitOrWildcardSpecifier);
    }
    let (dep_name, dep_ver_opt) = if let Some((n, v)) = edge.split_once('@') {
        (n.trim(), Some(v.trim()))
    } else {
        let mut parts = edge.split_whitespace();
        let n = parts
            .next()
            .ok_or(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)?;
        (n, parts.next())
    };
    let ver_probe = dep_ver_opt.unwrap_or_default();
    if is_blocked_graph_crate(dep_name, ver_probe, forbidden_crates) {
        return Err(OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected);
    }
    if let Some(ver) = dep_ver_opt {
        return lookup_versioned_package_key(dep_name, ver, packages, by_norm_name);
    }
    let keys = by_norm_name
        .get(&normalize_crate_key(dep_name))
        .ok_or(OfflineLockGovernanceError::ManifestToLockDriftDetected)?;
    if keys.len() != 1 {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    keys.first()
        .cloned()
        .ok_or(OfflineLockGovernanceError::ManifestToLockDriftDetected)
}

/// Computes initial incoming edge counts across all locked package nodes in `adj`.
/// Realises: [REQ-0608]
fn build_in_degree_table(
    adj: &BTreeMap<String, Vec<String>>,
) -> Result<BTreeMap<String, usize>, OfflineLockGovernanceError> {
    let mut in_degree: BTreeMap<String, usize> = adj.keys().map(|k| (k.clone(), 0usize)).collect();
    for targets in adj.values() {
        for dst in targets {
            let deg = in_degree
                .get_mut(dst)
                .ok_or(OfflineLockGovernanceError::ManifestToLockDriftDetected)?;
            *deg = deg
                .checked_add(1)
                .ok_or(OfflineLockGovernanceError::PackageGraphDepthOrCycleExceeded)?;
        }
    }
    Ok(in_degree)
}

/// Relaxes a directed dependency edge and reports whether the target node reached zero in-degree.
/// Realises: [REQ-0608]
fn relax_dependency_neighbor(
    next: &str,
    next_depth: usize,
    max_depth: usize,
    in_degree: &mut BTreeMap<String, usize>,
    max_depths: &mut BTreeMap<String, usize>,
) -> Result<bool, OfflineLockGovernanceError> {
    if next_depth > max_depth {
        return Err(OfflineLockGovernanceError::PackageGraphDepthOrCycleExceeded);
    }
    let slot_depth = max_depths
        .get_mut(next)
        .ok_or(OfflineLockGovernanceError::ManifestToLockDriftDetected)?;
    if next_depth > *slot_depth {
        *slot_depth = next_depth;
    }
    let deg = in_degree
        .get_mut(next)
        .ok_or(OfflineLockGovernanceError::ManifestToLockDriftDetected)?;
    *deg = deg.saturating_sub(1);
    Ok(*deg == 0)
}

/// Iteratively traverses the package dependency graph to enforce acyclicity and `max_depth` bounds.
/// Realises: [REQ-0608]
fn traverse_bounded_dag_depths(
    adj: &BTreeMap<String, Vec<String>>,
    max_depth: usize,
) -> Result<BTreeSet<String>, OfflineLockGovernanceError> {
    let mut in_degree = build_in_degree_table(adj)?;
    let mut max_depths: BTreeMap<String, usize> = adj.keys().map(|k| (k.clone(), 0usize)).collect();
    let mut queue: Vec<String> = in_degree
        .iter()
        .filter_map(|(k, &d)| if d == 0 { Some(k.clone()) } else { None })
        .collect();
    let mut head = 0usize;
    let mut verified = BTreeSet::new();
    while let Some(curr) = queue.get(head).cloned() {
        head = head.saturating_add(1);
        let curr_depth = *max_depths
            .get(&curr)
            .ok_or(OfflineLockGovernanceError::ManifestToLockDriftDetected)?;
        verified.insert(curr.clone());
        if let Some(neighbors) = adj.get(&curr) {
            let next_depth = curr_depth.saturating_add(1);
            for next in neighbors {
                if relax_dependency_neighbor(
                    next,
                    next_depth,
                    max_depth,
                    &mut in_degree,
                    &mut max_depths,
                )? {
                    queue.push(next.clone());
                }
            }
        }
    }
    if verified.len() != adj.len() {
        return Err(OfflineLockGovernanceError::PackageGraphDepthOrCycleExceeded);
    }
    Ok(verified)
}

/// Verifies the pinned CST supply-chain dependency DAG, 64-hex SHA-256 checksums, and forbidden crate isolation.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C air-gapped supply-chain
/// qualification mandates (REQ-0608, FLS `fls_qcb1n9c0e5hz`, SC-BP08-01, SC-BP08-02, SC-BP08-03). Guarantees
/// deterministic SHA-256 package pinning of lightweight CST crates (`ra_ap_syntax`, `rowan`, `syn`, `quote`,
/// `prettyplease`) while blocking heavyweight semantic solvers (`ra_ap_hir`, `ra_ap_ide`, root `rustc_private`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Supply-Chain Policy Invariant: Requires `snapshot.offline_locked_enforced == true` and
///   `snapshot.vendor_bundle_supported == true`.
/// - Pinned Graph Invariant: Every `RegistryLocked` and `VendoredBundle` entry (and every crate in
///   `required_crates`) carries a validated 64-hex-digit SHA-256 checksum, and `P n P_forbid == empty`.
/// - Bounded Iterative DAG Traversal: Enforces `max_depth <= MAX_AST_DEPTH` (`64`), zero directed cycles,
///   and maximum directed dependency path depth `<= max_depth` using iterative topological relaxation.
/// - Complexity Compliance: Executes in `<= 48` executable SLOC with `4` formal parameters,
///   `0` mutable boolean flags, and compound predicate arity `<= 3`.
///
/// ## 3. Preconditions
/// - `snapshot` references a candidate `LockManifestSnapshot`.
/// - `required_crates` and `forbidden_crates` specify normalized or `<crate>@<version>` identifiers.
/// - `max_depth` specifies a traversal depth ceiling `<= MAX_AST_DEPTH` (`64`).
///
/// ## 4. Postconditions
/// - Returns `Ok(BTreeSet<String>)` containing all verified `<crate>@<version>` package identifiers in
///   deterministic lexical order when all policy, checksum, pinning, acyclicity, and depth checks pass.
/// - Returns `Err(OfflineLockGovernanceError)` fail-closed (`DEAP-E0804`) on any violation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O((V + E) log V) where V is the number of locked packages and E is the number of
///   directed dependency edges.
/// - Space Complexity: O(V + E) deterministic `BTreeMap` and `BTreeSet` working state with zero recursion.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Accepts borrowed references and returns typed `OfflineLockGovernanceError` with zero panics.
/// - State: Pure read-only verification; performs zero mutations to `snapshot` or the filesystem.
/// - Action: Fails closed on disabled lock policy, missing SHA-256 checksums, forbidden semantic crates,
///   missing required CST crates, manifest-to-lock drift, cycles, or depth `> max_depth`.
/// - Resource: Strictly iterative traversal with `.get(...)` lookups and `checked_add` / `saturating_*` counters.
///
/// ## 7. Realization & Traceability
/// Realises: [AirGappedCargoLockGovernor/verify_pinned_cst_dependency_graph] | [REQ-0608] | SysML v2: part def AirGappedCargoLockGovernor
pub fn verify_pinned_cst_dependency_graph(
    snapshot: &LockManifestSnapshot,
    required_crates: &BTreeSet<String>,
    forbidden_crates: &BTreeSet<String>,
    max_depth: usize,
) -> Result<BTreeSet<String>, OfflineLockGovernanceError> {
    if !snapshot.offline_locked_enforced || !snapshot.vendor_bundle_supported {
        return Err(OfflineLockGovernanceError::LockPolicyDisabled);
    }
    if max_depth > MAX_AST_DEPTH {
        return Err(OfflineLockGovernanceError::PackageGraphDepthOrCycleExceeded);
    }
    if snapshot.packages.is_empty() {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    let mut by_norm_name: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (key, rec) in &snapshot.packages {
        validate_locked_package_node(key, rec, &snapshot.workspace_members, forbidden_crates)?;
        by_norm_name
            .entry(normalize_crate_key(&rec.name))
            .or_default()
            .push(key.clone());
    }
    verify_workspace_and_required_crates(snapshot, required_crates, forbidden_crates)?;
    let mut adj: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (key, rec) in &snapshot.packages {
        let mut edges = Vec::with_capacity(rec.dependencies.len());
        for raw_edge in &rec.dependencies {
            let target = resolve_verified_dependency_target(
                raw_edge,
                &snapshot.packages,
                &by_norm_name,
                forbidden_crates,
            )?;
            edges.push(target);
        }
        adj.insert(key.clone(), edges);
    }
    traverse_bounded_dag_depths(&adj, max_depth)
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
        fn new(name: &str) -> Result<Self, std::io::Error> {
            let path =
                std::env::temp_dir().join(format!("deap_test_{}_{}", name, uuid::Uuid::new_v4()));
            fs::create_dir_all(&path)?;
            Ok(Self { path })
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn test_is_upstream_compiler() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("is_upstream")?;
        assert!(!is_upstream_compiler(&tmp.path));
        fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
        assert!(is_upstream_compiler(&tmp.path));
        Ok(())
    }

    #[test]
    fn test_check_gitignore() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("gitignore")?;
        assert!(check_gitignore_exists(&tmp.path).is_err());
        File::create(tmp.path.join(".gitignore"))?;
        assert!(check_gitignore_exists(&tmp.path).is_ok());
        Ok(())
    }

    #[test]
    fn test_check_no_ds_store() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("ds_store")?;
        assert!(check_no_ds_store_files(&tmp.path).is_ok());
        File::create(tmp.path.join(".DS_Store"))?;
        let res = check_no_ds_store_files(&tmp.path);
        let errs = res.err().ok_or("expected Err")?;
        assert!(errs
            .first()
            .ok_or("expected error message")?
            .contains(".DS_Store"));
        Ok(())
    }

    #[test]
    fn test_check_duplicate_blueprints() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("blueprints")?;
        assert!(check_no_duplicate_master_blueprints(&tmp.path).is_ok());

        // Negative Case A: duplicate master blueprint
        File::create(tmp.path.join("DEAP_MASTER_ARCHITECTURE.md"))?;
        assert!(check_no_duplicate_master_blueprints(&tmp.path).is_err());
        fs::remove_file(tmp.path.join("DEAP_MASTER_ARCHITECTURE.md"))?;

        // Negative Case B: Tier 0 compiler crate directory leaked downstream
        fs::create_dir_all(tmp.path.join("crates/deap-core"))?;
        assert!(check_no_duplicate_master_blueprints(&tmp.path).is_err());
        fs::remove_dir_all(tmp.path.join("crates"))?;

        // Negative Case C: docs/architecture/blueprints contains markdown
        fs::create_dir_all(tmp.path.join("docs/architecture/blueprints"))?;
        File::create(tmp.path.join("docs/architecture/blueprints/LEAKED.md"))?;
        assert!(check_no_duplicate_master_blueprints(&tmp.path).is_err());
        fs::remove_dir_all(tmp.path.join("docs"))?;

        // Negative Case D: upstream safety fixtures directory leaked downstream
        fs::create_dir_all(tmp.path.join("tests/fixtures/safety"))?;
        assert!(check_no_duplicate_master_blueprints(&tmp.path).is_err());
        fs::remove_dir_all(tmp.path.join("tests"))?;

        // Negative Case E: upstream topology failure fixtures directory leaked downstream
        fs::create_dir_all(tmp.path.join("app_flutter/test/topology/failures"))?;
        assert!(check_no_duplicate_master_blueprints(&tmp.path).is_err());
        fs::remove_dir_all(tmp.path.join("app_flutter"))?;

        // Negative Case F: Tier 2 Flutter app consuming package:app_flutter duplicates Tier 1 SDK core files
        fs::create_dir_all(tmp.path.join("app_flutter/lib/features/layout"))?;
        fs::write(
            tmp.path.join("app_flutter/pubspec.yaml"),
            "name: cuas_coa_app\ndependencies:\n  app_flutter:\n    path: ../../DEAP-profile-flutter-app/app_flutter\n",
        )?;
        File::create(
            tmp.path
                .join("app_flutter/lib/features/layout/component_factory.dart"),
        )?;
        assert!(check_no_duplicate_master_blueprints(&tmp.path).is_err());
        fs::remove_file(
            tmp.path
                .join("app_flutter/lib/features/layout/component_factory.dart"),
        )?;
        assert!(check_no_duplicate_master_blueprints(&tmp.path).is_ok());
        fs::remove_dir_all(tmp.path.join("app_flutter"))?;

        // Upstream repo exempt
        fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
        assert!(check_no_duplicate_master_blueprints(&tmp.path).is_ok());
        Ok(())
    }

    #[test]
    fn test_check_downstream_instructions() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("instructions")?;
        assert!(check_downstream_instructions_exist(&tmp.path).is_err());

        // Create README with required phrase
        let mut readme = File::create(tmp.path.join("README.md"))?;
        writeln!(
            readme,
            "# Downstream Cyber-Physical Infrastructure Safety Project\n\nDetails."
        )?;

        // Create AGENTS.md
        let mut agents = File::create(tmp.path.join("AGENTS.md"))?;
        writeln!(agents, "# Agents")?;

        // Create rules/sysml-ssot-completeness.md
        fs::create_dir_all(tmp.path.join("rules"))?;
        let mut sysml = File::create(tmp.path.join("rules/sysml-ssot-completeness.md"))?;
        writeln!(sysml, "# SysML SSOT Completeness")?;

        assert!(check_downstream_instructions_exist(&tmp.path).is_ok());
        Ok(())
    }

    #[test]
    fn test_check_reconcile_backlog_tooling() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("reconcile")?;
        assert!(check_reconcile_backlog_tooling_exists(&tmp.path).is_err());

        // Downstream requires all 5 binaries
        fs::create_dir_all(tmp.path.join("target/release"))?;
        for bin in [
            "verify-baseline",
            "compile-sysml",
            "ingest-sysml",
            "assemble-conops",
            "reconcile-backlog",
        ] {
            let bin_path = tmp.path.join("target/release").join(bin);
            let mut f = File::create(&bin_path)?;
            writeln!(f, "binary")?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&bin_path, fs::Permissions::from_mode(0o755))?;
            }
        }

        assert!(check_reconcile_backlog_tooling_exists(&tmp.path).is_ok());
        Ok(())
    }

    #[test]
    fn test_check_reconcile_backlog_tooling_rust_native() -> Result<(), Box<dyn std::error::Error>>
    {
        let tmp = TempDir::new("reconcile_rust")?;
        // Empty workspace must fail
        assert!(check_reconcile_backlog_tooling_exists(&tmp.path).is_err());

        // Upstream mode (.pipeline/upstream present)
        fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;

        // Case A: crates/reconcile-backlog
        let crate_dir = tmp.path.join("crates/reconcile-backlog/src");
        fs::create_dir_all(&crate_dir)?;
        File::create(tmp.path.join("crates/reconcile-backlog/Cargo.toml"))?;
        File::create(tmp.path.join("crates/reconcile-backlog/src/main.rs"))?;
        assert!(check_reconcile_backlog_tooling_exists(&tmp.path).is_ok());

        // Case B: target/release/reconcile-backlog
        let tmp_bin = TempDir::new("reconcile_bin")?;
        fs::create_dir_all(tmp_bin.path.join(".pipeline/upstream"))?;
        fs::create_dir_all(tmp_bin.path.join("target/release"))?;
        let bin_path = tmp_bin.path.join("target/release/reconcile-backlog");
        let mut f = File::create(&bin_path)?;
        writeln!(f, "binary content")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&bin_path, fs::Permissions::from_mode(0o755))?;
        }
        assert!(check_reconcile_backlog_tooling_exists(&tmp_bin.path).is_ok());
        Ok(())
    }

    #[test]
    fn test_upstream_clean_landing_zones() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("clean_zones")?;
        // Downstream should be exempt
        assert!(check_upstream_template_clean_landing_zones(&tmp.path).is_ok());

        // Upstream with clean landing zone
        fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
        fs::create_dir_all(tmp.path.join("schema"))?;
        File::create(tmp.path.join("schema/.gitkeep"))?;
        assert!(check_upstream_template_clean_landing_zones(&tmp.path).is_ok());

        // Adding concrete spec file causes violation
        File::create(tmp.path.join("schema/domain_model.sysml"))?;
        let res = check_upstream_template_clean_landing_zones(&tmp.path);
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    fn test_current_repo_baseline_checks() -> Result<(), Box<dyn std::error::Error>> {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("repo root")?;

        assert_eq!(
            is_upstream_compiler(repo_root),
            repo_root.join(".pipeline/upstream").is_dir()
        );
        assert!(check_gitignore_exists(repo_root).is_ok());
        assert!(check_no_ds_store_files(repo_root).is_ok());
        assert!(check_no_duplicate_master_blueprints(repo_root).is_ok());
        assert!(check_downstream_instructions_exist(repo_root).is_ok());
        assert!(check_reconcile_backlog_tooling_exists(repo_root).is_ok());
        assert!(check_upstream_template_clean_landing_zones(repo_root).is_ok());
        Ok(())
    }

    #[test]
    fn test_workspace_source_purity_and_7_section_rustdoc() {
        let source = include_str!("workspace.rs");
        let s1 = "## 1. Safety Intent & Regulatory Scope";
        let s2 = "## 2. Invariants & Mathematical Properties";
        let s3 = "## 3. Preconditions";
        let s4 = "## 4. Postconditions";
        let s5 = "## 5. Algorithmic Complexity Bounds";
        let s6 = "## 6. Failure Modes & Fault Tolerance";
        let s7 = "## 7. Realization & Traceability";

        let module_docs: String = source
            .lines()
            .take_while(|l| l.trim().starts_with("//!"))
            .collect::<Vec<_>>()
            .join("\n");
        for section in [s1, s2, s3, s4, s5, s6, s7] {
            assert!(
                module_docs.contains(section),
                "Module-level inner Rustdoc missing mandatory section: {}",
                section
            );
        }

        let prod = source.split("#[cfg(test)]").next().unwrap_or(source);
        let mut current_doc = Vec::new();
        let mut pub_fn_count = 0usize;
        let mut tier_bc_fn_count = 0usize;

        let pat_regex_colons = ["Reg", "ex::"].concat();
        let pat_inline_regex = ["reg", "ex::"].concat();
        let needle_slice_str = ["&[", "&str]"].concat();
        let needle_let_blank = ["let", " _ = "].concat();

        for (idx, raw_line) in prod.lines().enumerate() {
            let line_no = idx + 1;
            let trimmed = raw_line.trim();
            if trimmed.starts_with("//!") || trimmed.starts_with("///") {
                current_doc.push(trimmed.to_string());
                continue;
            }
            let code = trimmed.split("//").next().unwrap_or(trimmed).trim();
            if code.is_empty() {
                continue;
            }
            assert!(
                !code.contains(&pat_regex_colons) && !code.contains(&pat_inline_regex),
                "Line {}: regex usage detected",
                line_no
            );
            assert!(
                !code.contains(&needle_slice_str),
                "Line {}: static string-slice array detected",
                line_no
            );
            assert!(
                !code.contains(&needle_let_blank),
                "Line {}: hollow gate stub detected",
                line_no
            );

            if code.starts_with("pub fn ") || code.starts_with("pub const fn ") {
                pub_fn_count += 1;
                let doc_block = current_doc.join("\n");
                for section in [s1, s2, s3, s4, s5, s6, s7] {
                    assert!(
                        doc_block.contains(section),
                        "Line {}: public function missing section {}",
                        line_no,
                        section
                    );
                }
                current_doc.clear();
            } else if code.starts_with("fn ")
                || code.starts_with("const fn ")
                || code.starts_with("pub(crate) fn ")
            {
                tier_bc_fn_count += 1;
                let doc_block = current_doc.join("\n");
                assert_eq!(
                    current_doc.len(),
                    2,
                    "Line {}: RustTierBC helper must carry exact 2-line docstring",
                    line_no
                );
                assert!(
                    doc_block.contains("Realises: ["),
                    "Line {}: RustTierBC helper missing Realises header",
                    line_no
                );
                current_doc.clear();
            } else if !code.starts_with("#[") {
                current_doc.clear();
            }
        }
        assert!(
            pub_fn_count >= 10,
            "Expected at least 10 RustTierA public functions in workspace.rs, found {}",
            pub_fn_count
        );
        assert!(
            tier_bc_fn_count >= 1,
            "Expected documented RustTierBC helper functions in workspace.rs"
        );
    }

    #[test]
    fn rejects_unpinned_or_oversized_lock_manifest() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(MAX_AST_DEPTH, 64);
        assert_eq!(
            OfflineLockGovernanceError::CstConstraintEnvelopeViolation.diagnostic_code(),
            "DEAP-E0803"
        );

        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("repo root")?;
        let live_toml = fs::read(repo_root.join("Cargo.toml"))?;
        let live_lock = fs::read(repo_root.join("Cargo.lock"))?;
        let live_snap =
            parse_locked_manifest_snapshot(&live_toml, &live_lock, None, MAX_CARGO_LOCK_BYTES)
                .map_err(|e| format!("live workspace snapshot failed: {e}"))?;

        assert!(live_snap.offline_locked_enforced);
        assert!(live_snap.vendor_bundle_supported);
        assert_ne!(live_snap.cargo_toml_sha256, [0u8; 32]);
        assert_ne!(live_snap.cargo_lock_sha256, [0u8; 32]);

        for member in [
            "deap-core",
            "compile-sysml",
            "verify-baseline",
            "ingest-sysml",
            "assemble-conops",
            "reconcile-backlog",
        ] {
            assert!(
                live_snap.workspace_members.contains(member),
                "missing workspace member {member}"
            );
            let member_key = format!("{member}@0.1.0");
            let rec = live_snap
                .packages
                .get(&member_key)
                .ok_or("missing workspace package record")?;
            assert_eq!(rec.source_kind, PackageSourceKind::WorkspacePath);
            assert!(rec.checksum.is_none());
        }

        for pinned_key in [
            "ra_ap_syntax@0.0.360",
            "rowan@0.17.0",
            "syn@3.0.6",
            "quote@1.0.47",
            "prettyplease@0.3.0",
        ] {
            let rec = live_snap
                .packages
                .get(pinned_key)
                .ok_or("missing pinned CST crate in Cargo.lock")?;
            assert_eq!(rec.source_kind, PackageSourceKind::RegistryLocked);
            let cs = rec.checksum.as_deref().ok_or("missing checksum")?;
            assert_eq!(cs.len(), 64);
            assert!(cs.bytes().all(|b| b.is_ascii_hexdigit()));
        }

        let core_rec = live_snap
            .packages
            .get("deap-core@0.1.0")
            .ok_or("missing deap-core@0.1.0")?;
        assert!(core_rec.dependencies.contains("ra_ap_syntax@0.0.360"));
        assert!(core_rec.dependencies.contains("syn@3.0.6"));

        let tmp = TempDir::new("us51_lock_manifest")?;
        let valid_toml = b"[workspace]\nmembers = [\"crates/demo\"]\n\n[workspace.dependencies]\nrowan = \"0.17.0\"\n";
        let valid_lock = b"version = 4\n\n[[package]]\nname = \"demo\"\nversion = \"0.1.0\"\ndependencies = [\n \"rowan\",\n]\n\n[[package]]\nname = \"rowan\"\nversion = \"0.17.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"14b574c58582fa59fa43a2feb6608b8744184659f08a2e0117e4b8224d95ed61\"\n";
        let vendor_cfg = b"[net]\noffline = true\n\n[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n";
        fs::write(tmp.path.join("Cargo.toml"), valid_toml)?;
        fs::write(tmp.path.join("Cargo.lock"), valid_lock)?;
        let disk_toml = fs::read(tmp.path.join("Cargo.toml"))?;
        let disk_lock = fs::read(tmp.path.join("Cargo.lock"))?;

        // Exact max_lock_bytes boundary with cargo vendor config
        let vendored = parse_locked_manifest_snapshot(
            &disk_toml,
            &disk_lock,
            Some(vendor_cfg),
            disk_lock.len(),
        )
        .map_err(|e| format!("vendored boundary failed: {e}"))?;
        let rowan_rec = vendored
            .packages
            .get("rowan@0.17.0")
            .ok_or("missing rowan@0.17.0")?;
        assert_eq!(rowan_rec.source_kind, PackageSourceKind::VendoredBundle);

        // Oversized lockfile & zero ceiling
        let err_oversized = parse_locked_manifest_snapshot(
            &disk_toml,
            &disk_lock,
            None,
            disk_lock.len().saturating_sub(1),
        );
        assert_eq!(
            err_oversized,
            Err(OfflineLockGovernanceError::ManifestSizeCeilingExceeded)
        );
        assert_eq!(
            OfflineLockGovernanceError::ManifestSizeCeilingExceeded.diagnostic_code(),
            "DEAP-E0804"
        );
        assert_eq!(
            parse_locked_manifest_snapshot(&disk_toml, &disk_lock, None, 0),
            Err(OfflineLockGovernanceError::ManifestSizeCeilingExceeded)
        );

        // Wildcard and unpinned git specifiers
        let wildcard_toml =
            b"[workspace]\nmembers = [\"crates/demo\"]\n[workspace.dependencies]\nrowan = \"*\"\n";
        assert_eq!(
            parse_locked_manifest_snapshot(wildcard_toml, &disk_lock, None, MAX_CARGO_LOCK_BYTES),
            Err(OfflineLockGovernanceError::UnpinnedGitOrWildcardSpecifier)
        );
        let git_toml = b"[workspace]\nmembers = [\"crates/demo\"]\n[workspace.dependencies]\nrowan = { git = \"https://example.invalid/rowan.git\" }\n";
        assert_eq!(
            parse_locked_manifest_snapshot(git_toml, &disk_lock, None, MAX_CARGO_LOCK_BYTES),
            Err(OfflineLockGovernanceError::UnpinnedGitOrWildcardSpecifier)
        );
        let git_lock = b"version = 4\n[[package]]\nname = \"demo\"\nversion = \"0.1.0\"\n[[package]]\nname = \"rowan\"\nversion = \"0.17.0\"\nsource = \"git+https://example.invalid/rowan.git#123456\"\n";
        assert_eq!(
            parse_locked_manifest_snapshot(&disk_toml, git_lock, None, MAX_CARGO_LOCK_BYTES),
            Err(OfflineLockGovernanceError::UnpinnedGitOrWildcardSpecifier)
        );

        // Truncated or non-hex SHA-256 checksum
        let bad_hex_lock = b"version = 4\n[[package]]\nname = \"demo\"\nversion = \"0.1.0\"\n[[package]]\nname = \"rowan\"\nversion = \"0.17.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"14b574c5\"\n";
        assert_eq!(
            parse_locked_manifest_snapshot(&disk_toml, bad_hex_lock, None, MAX_CARGO_LOCK_BYTES),
            Err(OfflineLockGovernanceError::InvalidSha256ChecksumHex)
        );

        // Forbidden semantic/compiler-internal crate
        let hir_lock = b"version = 4\n[[package]]\nname = \"demo\"\nversion = \"0.1.0\"\n[[package]]\nname = \"rowan\"\nversion = \"0.17.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"14b574c58582fa59fa43a2feb6608b8744184659f08a2e0117e4b8224d95ed61\"\n[[package]]\nname = \"ra_ap_hir\"\nversion = \"0.0.360\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"14b574c58582fa59fa43a2feb6608b8744184659f08a2e0117e4b8224d95ed61\"\n";
        assert_eq!(
            parse_locked_manifest_snapshot(&disk_toml, hir_lock, None, MAX_CARGO_LOCK_BYTES),
            Err(OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected)
        );

        // Disabled offline lock policy
        let disabled_cfg = b"[net]\noffline = false\n";
        assert_eq!(
            parse_locked_manifest_snapshot(
                &disk_toml,
                &disk_lock,
                Some(disabled_cfg),
                MAX_CARGO_LOCK_BYTES
            ),
            Err(OfflineLockGovernanceError::LockPolicyDisabled)
        );

        // Malformed TOML, excessive bracket depth (> 64), and manifest-to-lock drift
        assert_eq!(
            parse_locked_manifest_snapshot(b"[workspace\n", &disk_lock, None, MAX_CARGO_LOCK_BYTES),
            Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)
        );
        let deep_toml = format!("x = {}\"a\"{}\n", "[".repeat(65), "]".repeat(65));
        assert_eq!(
            parse_locked_manifest_snapshot(
                deep_toml.as_bytes(),
                &disk_lock,
                None,
                MAX_CARGO_LOCK_BYTES
            ),
            Err(OfflineLockGovernanceError::PackageGraphDepthOrCycleExceeded)
        );
        let drift_toml = b"[workspace]\nmembers = [\"crates/demo\"]\n[workspace.dependencies]\nrowan = \"0.17.0\"\nmissing_dep = \"1.0.0\"\n";
        assert_eq!(
            parse_locked_manifest_snapshot(drift_toml, &disk_lock, None, MAX_CARGO_LOCK_BYTES),
            Err(OfflineLockGovernanceError::ManifestToLockDriftDetected)
        );

        Ok(())
    }

    fn make_pinned_cst_fixture_snapshot() -> LockManifestSnapshot {
        let valid_sha =
            "14b574c58582fa59fa43a2feb6608b8744184659f08a2e0117e4b8224d95ed61".to_string();
        let mut packages = BTreeMap::new();
        let cst_crates = [
            ("ra_ap_syntax", "0.0.360"),
            ("rowan", "0.17.0"),
            ("syn", "3.0.6"),
            ("quote", "1.0.47"),
            ("prettyplease", "0.3.0"),
        ];
        let mut root_deps = BTreeSet::new();
        for (name, ver) in cst_crates {
            let key = format!("{name}@{ver}");
            root_deps.insert(key.clone());
            packages.insert(
                key,
                LockedPackageRecord {
                    name: name.to_string(),
                    version: ver.to_string(),
                    checksum: Some(valid_sha.clone()),
                    source_kind: PackageSourceKind::VendoredBundle,
                    dependencies: BTreeSet::new(),
                },
            );
        }
        packages.insert(
            "demo@0.1.0".to_string(),
            LockedPackageRecord {
                name: "demo".to_string(),
                version: "0.1.0".to_string(),
                checksum: None,
                source_kind: PackageSourceKind::WorkspacePath,
                dependencies: root_deps,
            },
        );
        LockManifestSnapshot {
            cargo_toml_sha256: [0x11u8; 32],
            cargo_lock_sha256: [0x22u8; 32],
            packages,
            workspace_members: BTreeSet::from(["demo".to_string()]),
            offline_locked_enforced: true,
            vendor_bundle_supported: true,
        }
    }

    #[test]
    fn rejects_forbidden_ra_ap_hir_and_missing_sha256() -> Result<(), Box<dyn std::error::Error>> {
        let valid_sha =
            "14b574c58582fa59fa43a2feb6608b8744184659f08a2e0117e4b8224d95ed61".to_string();
        let required_cst = BTreeSet::from([
            "ra_ap_syntax".to_string(),
            "rowan".to_string(),
            "syn".to_string(),
            "quote".to_string(),
            "prettyplease".to_string(),
        ]);
        let forbidden_cst = BTreeSet::from([
            "ra_ap_hir".to_string(),
            "ra_ap_ide".to_string(),
            "rustc_private".to_string(),
        ]);

        // 1. Nominal live workspace verification across all 6 crates and 5 pinned CST crates
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("repo root")?;
        let live_toml = fs::read(repo_root.join("Cargo.toml"))?;
        let live_lock = fs::read(repo_root.join("Cargo.lock"))?;
        let live_snap =
            parse_locked_manifest_snapshot(&live_toml, &live_lock, None, MAX_CARGO_LOCK_BYTES)
                .map_err(|e| format!("live snapshot parse failed: {e}"))?;
        let live_verified = verify_pinned_cst_dependency_graph(
            &live_snap,
            &required_cst,
            &forbidden_cst,
            MAX_AST_DEPTH,
        )
        .map_err(|e| format!("live CST graph verification failed: {e}"))?;
        for expected in [
            "deap-core@0.1.0",
            "compile-sysml@0.1.0",
            "verify-baseline@0.1.0",
            "ingest-sysml@0.1.0",
            "assemble-conops@0.1.0",
            "reconcile-backlog@0.1.0",
            "ra_ap_syntax@0.0.360",
            "rowan@0.17.0",
            "syn@3.0.6",
            "quote@1.0.47",
            "prettyplease@0.3.0",
        ] {
            assert!(
                live_verified.contains(expected),
                "expected {expected} in verified CST graph"
            );
        }

        // Hyphenated required crate alias ("ra-ap-syntax") also resolves cleanly
        let hyphen_req = BTreeSet::from(["ra-ap-syntax".to_string(), "rowan".to_string()]);
        let base_snap = make_pinned_cst_fixture_snapshot();
        assert!(verify_pinned_cst_dependency_graph(
            &base_snap,
            &hyphen_req,
            &forbidden_cst,
            MAX_AST_DEPTH
        )
        .is_ok());

        // 2. Boundary MAX_AST_DEPTH == 64 vendored DAG chain (with diamond convergence)
        let mut depth64_snap = make_pinned_cst_fixture_snapshot();
        for idx in 0..MAX_AST_DEPTH {
            let cur_name = if idx == 0 {
                "demo".to_string()
            } else {
                format!("chain_{idx}")
            };
            let cur_key = format!("{cur_name}@0.1.0");
            let next_name = format!("chain_{}", idx + 1);
            let next_key = format!("{next_name}@0.1.0");
            depth64_snap.packages.insert(
                next_key.clone(),
                LockedPackageRecord {
                    name: next_name,
                    version: "0.1.0".to_string(),
                    checksum: Some(valid_sha.clone()),
                    source_kind: PackageSourceKind::VendoredBundle,
                    dependencies: BTreeSet::new(),
                },
            );
            if let Some(cur_rec) = depth64_snap.packages.get_mut(&cur_key) {
                cur_rec.dependencies.insert(next_key);
                if idx == 0 {
                    cur_rec.dependencies.insert("chain_64@0.1.0".to_string());
                }
            }
        }
        let verified_64 = verify_pinned_cst_dependency_graph(
            &depth64_snap,
            &required_cst,
            &forbidden_cst,
            MAX_AST_DEPTH,
        )
        .map_err(|e| format!("depth 64 boundary should pass: {e}"))?;
        assert!(verified_64.contains("chain_64@0.1.0"));

        // Depth 65 exceeds MAX_AST_DEPTH = 64 -> fail-closed PackageGraphDepthOrCycleExceeded
        let mut depth65_snap = depth64_snap.clone();
        depth65_snap.packages.insert(
            "chain_65@0.1.0".to_string(),
            LockedPackageRecord {
                name: "chain_65".to_string(),
                version: "0.1.0".to_string(),
                checksum: Some(valid_sha.clone()),
                source_kind: PackageSourceKind::VendoredBundle,
                dependencies: BTreeSet::new(),
            },
        );
        if let Some(rec64) = depth65_snap.packages.get_mut("chain_64@0.1.0") {
            rec64.dependencies.insert("chain_65@0.1.0".to_string());
        }
        assert_eq!(
            verify_pinned_cst_dependency_graph(
                &depth65_snap,
                &required_cst,
                &forbidden_cst,
                MAX_AST_DEPTH
            ),
            Err(OfflineLockGovernanceError::PackageGraphDepthOrCycleExceeded)
        );
        assert_eq!(
            verify_pinned_cst_dependency_graph(
                &base_snap,
                &required_cst,
                &forbidden_cst,
                MAX_AST_DEPTH + 1
            ),
            Err(OfflineLockGovernanceError::PackageGraphDepthOrCycleExceeded)
        );

        // Cyclic dependency graph -> fail-closed PackageGraphDepthOrCycleExceeded
        let mut cycle_snap = make_pinned_cst_fixture_snapshot();
        cycle_snap.packages.insert(
            "cyc_a@0.1.0".to_string(),
            LockedPackageRecord {
                name: "cyc_a".to_string(),
                version: "0.1.0".to_string(),
                checksum: Some(valid_sha.clone()),
                source_kind: PackageSourceKind::RegistryLocked,
                dependencies: BTreeSet::from(["cyc_b@0.1.0".to_string()]),
            },
        );
        cycle_snap.packages.insert(
            "cyc_b@0.1.0".to_string(),
            LockedPackageRecord {
                name: "cyc_b".to_string(),
                version: "0.1.0".to_string(),
                checksum: Some(valid_sha.clone()),
                source_kind: PackageSourceKind::RegistryLocked,
                dependencies: BTreeSet::from(["cyc_a@0.1.0".to_string()]),
            },
        );
        assert_eq!(
            verify_pinned_cst_dependency_graph(
                &cycle_snap,
                &required_cst,
                &forbidden_cst,
                MAX_AST_DEPTH
            ),
            Err(OfflineLockGovernanceError::PackageGraphDepthOrCycleExceeded)
        );

        // 3. Forbidden ra_ap_hir, ra_ap_ide, and root rustc_private -> DEAP-E0804
        for forbidden_name in ["ra_ap_hir", "ra_ap_ide", "rustc_private"] {
            let mut bad_snap = make_pinned_cst_fixture_snapshot();
            let bad_key = format!("{forbidden_name}@0.0.360");
            bad_snap.packages.insert(
                bad_key,
                LockedPackageRecord {
                    name: forbidden_name.to_string(),
                    version: "0.0.360".to_string(),
                    checksum: Some(valid_sha.clone()),
                    source_kind: PackageSourceKind::RegistryLocked,
                    dependencies: BTreeSet::new(),
                },
            );
            let res = verify_pinned_cst_dependency_graph(
                &bad_snap,
                &required_cst,
                &BTreeSet::new(),
                MAX_AST_DEPTH,
            );
            assert_eq!(
                res,
                Err(OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected)
            );
            assert_eq!(
                OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected.diagnostic_code(),
                "DEAP-E0804"
            );
        }

        let mut root_rustc_priv = make_pinned_cst_fixture_snapshot();
        root_rustc_priv
            .workspace_members
            .insert("rustc_private".to_string());
        assert_eq!(
            verify_pinned_cst_dependency_graph(
                &root_rustc_priv,
                &required_cst,
                &forbidden_cst,
                MAX_AST_DEPTH
            ),
            Err(OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected)
        );

        let custom_forbidden = BTreeSet::from(["custom_banned_crate".to_string()]);
        let mut custom_bad = make_pinned_cst_fixture_snapshot();
        custom_bad.packages.insert(
            "custom_banned_crate@1.0.0".to_string(),
            LockedPackageRecord {
                name: "custom_banned_crate".to_string(),
                version: "1.0.0".to_string(),
                checksum: Some(valid_sha),
                source_kind: PackageSourceKind::RegistryLocked,
                dependencies: BTreeSet::new(),
            },
        );
        assert_eq!(
            verify_pinned_cst_dependency_graph(
                &custom_bad,
                &required_cst,
                &custom_forbidden,
                MAX_AST_DEPTH
            ),
            Err(OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected)
        );

        // 4. Missing or truncated/non-hex SHA-256 checksum on registry/vendored crate
        let mut missing_sha_snap = make_pinned_cst_fixture_snapshot();
        if let Some(rowan) = missing_sha_snap.packages.get_mut("rowan@0.17.0") {
            rowan.checksum = None;
        }
        assert_eq!(
            verify_pinned_cst_dependency_graph(
                &missing_sha_snap,
                &required_cst,
                &forbidden_cst,
                MAX_AST_DEPTH
            ),
            Err(OfflineLockGovernanceError::InvalidSha256ChecksumHex)
        );

        let mut short_sha_snap = make_pinned_cst_fixture_snapshot();
        if let Some(syn_rec) = short_sha_snap.packages.get_mut("syn@3.0.6") {
            syn_rec.checksum = Some("deadbeef".to_string());
        }
        assert_eq!(
            verify_pinned_cst_dependency_graph(
                &short_sha_snap,
                &required_cst,
                &forbidden_cst,
                MAX_AST_DEPTH
            ),
            Err(OfflineLockGovernanceError::InvalidSha256ChecksumHex)
        );

        // 5. Missing required CST dependency, disabled lock policy, wildcard, and drift
        let mut missing_req_snap = make_pinned_cst_fixture_snapshot();
        missing_req_snap.packages.remove("prettyplease@0.3.0");
        if let Some(demo) = missing_req_snap.packages.get_mut("demo@0.1.0") {
            demo.dependencies.remove("prettyplease@0.3.0");
        }
        assert_eq!(
            verify_pinned_cst_dependency_graph(
                &missing_req_snap,
                &required_cst,
                &forbidden_cst,
                MAX_AST_DEPTH
            ),
            Err(OfflineLockGovernanceError::MissingRequiredCstDependency)
        );

        let mut policy_off_snap = make_pinned_cst_fixture_snapshot();
        policy_off_snap.offline_locked_enforced = false;
        assert_eq!(
            verify_pinned_cst_dependency_graph(
                &policy_off_snap,
                &required_cst,
                &forbidden_cst,
                MAX_AST_DEPTH
            ),
            Err(OfflineLockGovernanceError::LockPolicyDisabled)
        );

        let mut git_edge_snap = make_pinned_cst_fixture_snapshot();
        if let Some(demo) = git_edge_snap.packages.get_mut("demo@0.1.0") {
            demo.dependencies
                .insert("git+https://example.invalid/foo.git".to_string());
        }
        assert_eq!(
            verify_pinned_cst_dependency_graph(
                &git_edge_snap,
                &required_cst,
                &forbidden_cst,
                MAX_AST_DEPTH
            ),
            Err(OfflineLockGovernanceError::UnpinnedGitOrWildcardSpecifier)
        );

        let mut drift_snap = make_pinned_cst_fixture_snapshot();
        drift_snap
            .workspace_members
            .insert("unlisted_member".to_string());
        assert_eq!(
            verify_pinned_cst_dependency_graph(
                &drift_snap,
                &required_cst,
                &forbidden_cst,
                MAX_AST_DEPTH
            ),
            Err(OfflineLockGovernanceError::ManifestToLockDriftDetected)
        );

        Ok(())
    }
}
