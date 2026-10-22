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
//! Realises: [REQ-WS-MODULE], [DO-330-TQL-4-WORKSPACE] | SysML v2: part def WorkspaceInspector

use crate::rules::{EXCLUDED_DIRS, MASTER_BLUEPRINTS};
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
/// - Interface: Returns a strongly typed `Result<(), String>` diagnostic.
/// - State: Read-only filesystem metadata inspection; zero side effects.
/// - Action: Fails closed with `Err` if `.gitignore` is absent, a broken symlink, or a directory; never panics.
/// - Resource: Minimal stack and single short-lived `PathBuf` allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [Workspace/check_gitignore_exists] | [REQ-WS-C10] | SysML v2: part def WorkspaceInspector
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
        let Ok(e) = e else { continue; };
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
            if !pubspec_text.contains("name: app_flutter") && pubspec_text.contains("app_flutter:") {
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

    let blueprints_dir = repo_root.join("docs").join("architecture").join("blueprints");
    if blueprints_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&blueprints_dir) {
            for entry in entries.flatten() {
                if entry.path().is_file()
                    && entry.path().extension().map(|ext| ext == "md").unwrap_or(false)
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
        .chain(std::iter::once("DEAP_SYSML_V2_SAFETY_MODEL_SPECIFICATION.sysml"))
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
        let Ok(e) = e else { continue; };
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
                && fs::metadata(&bin_path).map(|m| m.len() > 0).unwrap_or(false);
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
            let Ok(e) = e else { continue; };
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
        fn new(name: &str) -> Result<Self, std::io::Error> {
            let path = std::env::temp_dir().join(format!("deap_test_{}_{}", name, uuid::Uuid::new_v4()));
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
        assert!(errs.first().ok_or("expected error message")?.contains(".DS_Store"));
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
        File::create(tmp.path.join("app_flutter/lib/features/layout/component_factory.dart"))?;
        assert!(check_no_duplicate_master_blueprints(&tmp.path).is_err());
        fs::remove_file(tmp.path.join("app_flutter/lib/features/layout/component_factory.dart"))?;
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
        writeln!(readme, "# Downstream Cyber-Physical Infrastructure Safety Project\n\nDetails.")?;

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
    fn test_check_reconcile_backlog_tooling_rust_native() -> Result<(), Box<dyn std::error::Error>> {
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

        assert_eq!(is_upstream_compiler(repo_root), repo_root.join(".pipeline/upstream").is_dir());
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
        let mut fn_count = 0usize;

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

            if code.starts_with("pub fn ")
                || code.starts_with("fn ")
                || code.starts_with("pub const fn ")
                || code.starts_with("pub(crate) fn ")
            {
                fn_count += 1;
                let doc_block = current_doc.join("\n");
                for section in [s1, s2, s3, s4, s5, s6, s7] {
                    assert!(
                        doc_block.contains(section),
                        "Line {}: function missing section {}",
                        line_no,
                        section
                    );
                }
                current_doc.clear();
            } else if !code.starts_with("#[") {
                current_doc.clear();
            }
        }
        assert_eq!(fn_count, 7, "Expected exactly 7 documented functions in workspace.rs");
    }

    #[test]
    fn test_issue_450_active_rules_bundle_bounds() -> Result<(), Box<dyn std::error::Error>> {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("repo root")?;
        if !is_upstream_compiler(repo_root) {
            return Ok(());
        }
        let bundle_path = repo_root.join(".pipeline/ACTIVE_RULES_BUNDLE.md");
        assert!(bundle_path.is_file(), "Active rules bundle must exist at {:?}", bundle_path);
        let metadata = fs::metadata(&bundle_path)?;
        let byte_len = metadata.len();
        let content = fs::read_to_string(&bundle_path)?;
        let line_count = content.lines().count();

        assert!(
            byte_len < 40_000,
            "ACTIVE_RULES_BUNDLE.md must be under 40,000 bytes for single-chunk view_file read (got {} bytes)",
            byte_len
        );
        assert!(
            line_count < 750,
            "ACTIVE_RULES_BUNDLE.md must be under 750 lines for single-chunk view_file read (got {} lines)",
            line_count
        );
        Ok(())
    }

    #[test]
    fn test_issue_450_agents_md_scoped_initialization_invariants() -> Result<(), Box<dyn std::error::Error>> {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("repo root")?;
        if !is_upstream_compiler(repo_root) {
            return Ok(());
        }
        let agents_md = fs::read_to_string(repo_root.join("AGENTS.md"))?;

        assert!(
            !agents_md.contains("Read the target platform execution profile (e.g., `.pipeline/profiles/flutter.md`"),
            "AGENTS.md must not unconditionally command flutter.md profile read on Turn 1"
        );
        assert!(
            agents_md.contains("Conditional Platform Profile Load")
                || agents_md.contains("downstream feature implementation phase")
                || agents_md.contains("ONLY when implementing downstream application code"),
            "AGENTS.md must condition platform profile loading to downstream feature implementation"
        );

        assert!(
            !agents_md.contains("Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` (and active skills under `skills/`)"),
            "AGENTS.md must not unconditionally preload feature-driven-implementation/SKILL.md"
        );
        assert!(
            agents_md.contains("Load Active Task Skill On-Demand") || agents_md.contains("relevant to the active task"),
            "AGENTS.md must load active task skill on demand"
        );

        assert!(
            agents_md.contains("---GOVERNANCE-END---"),
            "AGENTS.md must explicitly reference ---GOVERNANCE-END--- bypass for curated subagents"
        );
        assert!(
            agents_md.contains("NEVER run `verify-baseline`, `cargo test`, or `flutter test` on session startup/initialization"),
            "AGENTS.md Step 5 must explicitly forbid running verify-baseline, cargo test, or flutter test on session startup/initialization"
        );

        let rule_20 = fs::read_to_string(repo_root.join("rules/verification-required.md"))?;
        assert!(
            rule_20.contains("TRIGGER SCOPE -- PRE-COMMIT / POST-EDIT GATE ONLY (NEVER ON INIT OR CONVERSATIONAL TURNS):")
                || rule_20.contains("never on startup or read-only turns"),
            "rules/verification-required.md must declare pre-commit/post-edit gate only"
        );
        assert!(
            !rule_20.contains("**ALWAYS enforce:**"),
            "rules/verification-required.md must not use unconditional **ALWAYS enforce:** language"
        );

        let bundle = fs::read_to_string(repo_root.join(".pipeline/ACTIVE_RULES_BUNDLE.md"))?;
        assert!(
            bundle.contains("TRIGGER SCOPE -- PRE-COMMIT / POST-EDIT GATE ONLY (NEVER ON INIT OR CONVERSATIONAL TURNS):")
                || bundle.contains("never on startup or read-only turns"),
            ".pipeline/ACTIVE_RULES_BUNDLE.md must declare pre-commit/post-edit gate only in Rule 20"
        );

        Ok(())
    }

    #[test]
    fn test_issue_450_zero_retired_python_references_in_pipeline_governance() -> Result<(), Box<dyn std::error::Error>> {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("repo root")?;
        if !is_upstream_compiler(repo_root) {
            return Ok(());
        }
        let constitution = fs::read_to_string(repo_root.join(".pipeline/constitution.md"))?;
        let tooling = fs::read_to_string(repo_root.join(".pipeline/upstream/pipeline-tooling.md"))?;

        let retired_patterns = [
            "spec_validator.py",
            "verify_downstream_baseline.py",
            "link_validator.py",
            "uml.py",
            "mermaid_syntax_validator.py",
            "behavioral.py",
            "codebase.py",
            "test_skill_path_references.py",
            "test_constitution_integrity.py",
            "docs.py",
            "spec_title_uniqueness_validator.py",
            "source_reference_validator.py",
            "logical_ui_validator.py",
            "pytest",
            "parity_auditor",
        ];

        for needle in retired_patterns {
            assert!(
                !constitution.contains(needle),
                ".pipeline/constitution.md must not reference retired script '{}'",
                needle
            );
            assert!(
                !tooling.contains(needle),
                ".pipeline/upstream/pipeline-tooling.md must not reference retired script '{}'",
                needle
            );
        }

        Ok(())
    }
}
