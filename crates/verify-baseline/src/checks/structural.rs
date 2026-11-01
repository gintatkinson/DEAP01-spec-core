//! Structural repository topology, compiler AST source purity (Check 34), and governance integrity gates.
//!
//! Safety & Traceability: Realises: [REQ-CHECK-34-STRUCTURAL-GATES], [DO-330-TQL-4-STRUCTURAL-PURITY]
//! Contract: Enforces deterministic workspace topology, script/handoff cleanliness, manifest lint gates, and tiered Check 34 compiler AST purity invariants across audited Rust sources.
//! Bounds & Faults: O(F * L) bounded file and line traversal with non-panicking slice inspection and fail-closed `Result<(), Vec<String>>` error reporting.

use deap_core::rules::{CanonicalArchitectureBlueprint, EXCLUDED_DIRS};
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
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Resolves the enclosing upstream compiler root when `repo_root` points to `<root>/.pipeline/upstream`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C configuration management controls.
/// Enables self-hosted dogfood verification of `<root>/.pipeline/upstream` against shared root-level
/// governance and tooling artifacts without weakening fail-closed checks on standalone downstream workspaces.
///
/// ## 2. Invariants & Mathematical Properties
/// Returns `Some(parent_repo)` if and only if `repo_root` has terminal path components `.pipeline/upstream`
/// and `is_upstream_compiler(&parent_repo)` evaluates to `true`. All standalone downstream workspaces
/// evaluate deterministically to `None`.
///
/// ## 3. Preconditions
/// `repo_root` is a valid filesystem path reference.
///
/// ## 4. Postconditions
/// Returns `Some(PathBuf)` pointing to the verified enclosing upstream compiler root, or `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1) bounded path component inspection. Space Complexity: O(P) where P is path length.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Pure path inspection plus `is_upstream_compiler` directory check; returns `None` on any mismatch.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-UPSTREAM-DOGFOOD-ROOT-RESOLUTION].
pub fn enclosing_upstream_compiler_root(repo_root: &Path) -> Option<PathBuf> {
    if repo_root.file_name().and_then(|n| n.to_str()) != Some("upstream") {
        return None;
    }
    let pipeline_dir = repo_root.parent()?;
    if pipeline_dir.file_name().and_then(|n| n.to_str()) != Some(".pipeline") {
        return None;
    }
    let raw_parent = pipeline_dir.parent()?;
    let parent_repo = if raw_parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        raw_parent
    };
    if is_upstream_compiler(parent_repo) {
        Some(parent_repo.to_path_buf())
    } else {
        None
    }
}

/// Check 10: Verify .gitignore exists in repository root.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C configuration management controls.
/// Ensures `.gitignore` is present at the repository root (or enclosing upstream compiler root for `<root>/.pipeline/upstream`).
///
/// ## 2. Invariants & Mathematical Properties
/// `repo_root.join(".gitignore")` (or `parent_repo.join(".gitignore")` for verified `<root>/.pipeline/upstream`) must resolve to an existing regular file.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` if `.gitignore` exists, or `Err(String)` describing the violation.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1). Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed if `.gitignore` is missing or inaccessible.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-10-GITIGNORE-EXISTS].
pub fn check_gitignore_exists(repo_root: &Path) -> Result<(), String> {
    match core_check_gitignore_exists(repo_root) {
        Ok(()) => Ok(()),
        Err(err) => {
            if let Some(parent_repo) = enclosing_upstream_compiler_root(repo_root) {
                core_check_gitignore_exists(&parent_repo)
            } else {
                Err(err)
            }
        }
    }
}

/// Check 11: Verify zero .DS_Store files exist in the working tree.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C workspace hygiene controls.
/// Ensures zero macOS `.DS_Store` metadata artifacts pollute the repository tree.
///
/// ## 2. Invariants & Mathematical Properties
/// Every non-excluded file entry in `repo_root` has filename != `.DS_Store`.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` when zero `.DS_Store` files exist, or `Err(Vec<String>)` listing violations.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) where N is directory entry count. Space Complexity: O(V) where V is violation count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Delegates to `deap_core::workspace::check_no_ds_store_files` which fails closed on I/O errors.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-11-NO-DS-STORE].
pub fn check_no_ds_store_files(repo_root: &Path) -> Result<(), Vec<String>> {
    core_check_no_ds_store_files(repo_root)
}

/// Check 12: Verify downstream repositories do not contain duplicate master core blueprints.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C single-source-of-truth (SSOT) architecture controls.
/// Prevents downstream repositories from duplicating upstream master core blueprints.
///
/// ## 2. Invariants & Mathematical Properties
/// Downstream repositories must contain zero duplicate master blueprint files.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` on compliance, or `Err(Vec<String>)` listing duplicate blueprint paths.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(B) where B is number of master blueprints. Space Complexity: O(V).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed if forbidden duplicate blueprints exist in downstream workspaces.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-12-NO-DUPLICATE-MASTER-BLUEPRINTS].
pub fn check_no_duplicate_master_blueprints(repo_root: &Path) -> Result<(), Vec<String>> {
    core_check_no_duplicate_master_blueprints(repo_root)
}

/// Check 14: Verify presence of README.md, agent instruction entrypoints, and rules/sysml-ssot-completeness.md.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C operational governance controls.
/// Verifies all mandatory instruction entrypoints and SSOT completeness rules exist.
///
/// ## 2. Invariants & Mathematical Properties
/// Required governance files must exist and be non-empty.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` when all instruction entrypoints exist, or `Err(Vec<String>)` on missing files.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1). Space Complexity: O(V).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed when any required instruction file is absent.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-14-DOWNSTREAM-INSTRUCTIONS].
pub fn check_downstream_instructions_exist(repo_root: &Path) -> Result<(), Vec<String>> {
    match core_check_downstream_instructions_exist(repo_root) {
        Ok(()) => Ok(()),
        Err(errs) => {
            if let Some(parent_repo) = enclosing_upstream_compiler_root(repo_root) {
                core_check_downstream_instructions_exist(&parent_repo)
            } else {
                Err(errs)
            }
        }
    }
}

/// Check 15: Verify reconcile-backlog tooling exists and is executable/buildable.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C traceability tooling controls.
/// Ensures backlog reconciliation tooling is present in the workspace (or enclosing upstream compiler root for `<root>/.pipeline/upstream`).
///
/// ## 2. Invariants & Mathematical Properties
/// Either compiled `reconcile-backlog` binary or its Rust crate manifest must exist.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` if tooling is present, or `Err(Vec<String>)` if missing.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1). Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed if neither binary nor crate manifest is found.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-15-RECONCILE-BACKLOG-TOOLING].
pub fn check_reconcile_backlog_tooling_exists(repo_root: &Path) -> Result<(), Vec<String>> {
    match core_check_reconcile_backlog_tooling_exists(repo_root) {
        Ok(()) => Ok(()),
        Err(errs) => {
            if let Some(parent_repo) = enclosing_upstream_compiler_root(repo_root) {
                core_check_reconcile_backlog_tooling_exists(&parent_repo)
            } else {
                Err(errs)
            }
        }
    }
}

/// Check 16: Upstream Template Clean Landing Zone Gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C M2/M1 separation controls.
/// Ensures upstream template landing zones contain only `.gitkeep` sentinel files.
///
/// ## 2. Invariants & Mathematical Properties
/// In upstream repositories, landing zone directories contain zero non-`.gitkeep` files.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` on clean landing zones, or `Err(Vec<String>)` listing contaminating files.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) where N is files in landing zones. Space Complexity: O(V).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on any non-`.gitkeep` file or directory traversal error.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-16-CLEAN-LANDING-ZONES].
pub fn check_upstream_template_clean_landing_zones(repo_root: &Path) -> Result<(), Vec<String>> {
    core_check_upstream_template_clean_landing_zones(repo_root)
}

/// Check 18: Upstream Template Clean Landing Zone & Blueprint Cleanliness Gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C M2 metamodel purity controls.
/// Enforces landing zone cleanliness, zero `.sysml` files in architecture blueprints,
/// and zero ephemeral handover checkpoint files in upstream repositories.
///
/// ## 2. Invariants & Mathematical Properties
/// 1. `schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, and `docs/use-cases/`
///    contain only `.gitkeep`.
/// 2. `docs/architecture/blueprints/` contains zero `.sysml` files.
/// 3. Zero `MASTER_HANDOVER_CHECKPOINT*` files exist at root or `docs/`.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` on compliance, or `Err(Vec<String>)` with sorted violation messages.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) where N is scanned directory entries. Space Complexity: O(V).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on any discovered contaminating artifact.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-18-UPSTREAM-BLUEPRINT-CLEANLINESS].
pub fn verify_upstream_blueprint_domain_cleanliness(repo_root: &Path) -> Result<(), Vec<String>> {
    if !is_upstream_compiler(repo_root) {
        return Ok(());
    }

    let mut violations = Vec::new();

    // 1. Enforce pure schema landing zone cleanliness (only .gitkeep)
    let landing_zones = [
        repo_root.join("schema"),
        repo_root.join("docs").join("epics"),
        repo_root.join("docs").join("features"),
        repo_root.join("docs").join("user-stories"),
        repo_root.join("docs").join("use-cases"),
    ];

    for zone_dir in &landing_zones {
        if !zone_dir.is_dir() {
            continue;
        }

        for entry_res in WalkDir::new(zone_dir).into_iter().filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !EXCLUDED_DIRS.iter().any(|ex| name == ex)
            } else {
                true
            }
        }) {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    violations.push(format!(
                        "Check 18 I/O error in '{}': {}",
                        zone_dir.display(),
                        err
                    ));
                    continue;
                }
            };
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
                        zone_dir.display(), rel
                    ));
                }
            }
        }
    }

    // 2. Enforce zero concrete modeling files (.sysml or .digest.json) in upstream blueprints
    let blueprints_dir = repo_root
        .join("docs")
        .join("architecture")
        .join("blueprints");
    if blueprints_dir.is_dir() {
        for entry_res in WalkDir::new(&blueprints_dir).into_iter().filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !EXCLUDED_DIRS.iter().any(|ex| name == ex)
            } else {
                true
            }
        }) {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    violations.push(format!(
                        "Check 18 I/O error in '{}': {}",
                        blueprints_dir.display(),
                        err
                    ));
                    continue;
                }
            };
            if entry.file_type().is_file() {
                let fname = entry.file_name().to_string_lossy();
                let fname_lower = fname.to_lowercase();
                if fname_lower.ends_with(".sysml") || fname_lower.ends_with(".digest.json") {
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

    let designs_dir = repo_root.join("docs").join("designs");
    if designs_dir.is_dir() {
        for entry_res in WalkDir::new(&designs_dir).into_iter().filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !EXCLUDED_DIRS.iter().any(|ex| name == ex)
            } else {
                true
            }
        }) {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    violations.push(format!(
                        "Check 18 I/O error in '{}': {}",
                        designs_dir.display(),
                        err
                    ));
                    continue;
                }
            };
            if entry.file_type().is_file() {
                let fname = entry.file_name().to_string_lossy();
                let fname_lower = fname.to_lowercase();
                if fname_lower.ends_with(".sysml") || fname_lower.ends_with(".digest.json") {
                    let stem_opt = if fname_lower.ends_with(".sysml") {
                        fname.get(..fname.len().saturating_sub(".sysml".len()))
                    } else {
                        fname.get(..fname.len().saturating_sub(".digest.json".len()))
                    };
                    let is_valid_triplet = if let Some(stem) = stem_opt {
                        if stem.starts_with("blueprint-") && stem.len() > "blueprint-".len() {
                            let parent_opt = entry.path().parent();
                            let sysml_non_empty = match parent_opt {
                                Some(p) => match fs::metadata(p.join(format!("{stem}.sysml"))) {
                                    Ok(m) => m.len() > 0,
                                    Err(_) => false,
                                },
                                None => false,
                            };
                            let md_non_empty = match parent_opt {
                                Some(p) => match fs::metadata(p.join(format!("{stem}.md"))) {
                                    Ok(m) => m.len() > 0,
                                    Err(_) => false,
                                },
                                None => false,
                            };
                            let digest_non_empty = match parent_opt {
                                Some(p) => {
                                    match fs::metadata(p.join(format!("{stem}.digest.json"))) {
                                        Ok(m) => m.len() > 0,
                                        Err(_) => false,
                                    }
                                }
                                None => false,
                            };
                            sysml_non_empty && md_non_empty && digest_non_empty
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if !is_valid_triplet {
                        let rel = entry
                            .path()
                            .strip_prefix(repo_root)
                            .unwrap_or(entry.path())
                            .display()
                            .to_string();
                        violations
                            .push(format!("Upstream blueprint contains SysML model: {}", rel));
                    }
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

/// Check 32: Zero Shell Scripts Invariant.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C deterministic Rust-only execution controls.
/// Scans the repository tree and fails closed if any `.sh` script exists on disk.
///
/// ## 2. Invariants & Mathematical Properties
/// Zero files in `repo_root` (excluding `target`, `.git`, `build`, `Pods`, `ephemeral`, and `.dart_tool`) may have extension `.sh`.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` if zero `.sh` files exist, or `Err(Vec<String>)` with sorted relative paths.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) where N is repository file count. Space Complexity: O(V).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on any discovered `.sh` file.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-32-ZERO-SHELL-SCRIPTS].
pub fn check_zero_shell_scripts(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();

    for entry_res in WalkDir::new(repo_root).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let name = e.file_name().to_string_lossy();
            name != "target"
                && name != ".git"
                && name != "build"
                && name != "Pods"
                && name != "ephemeral"
                && name != ".dart_tool"
        } else {
            true
        }
    }) {
        let entry = match entry_res {
            Ok(e) => e,
            Err(err) => {
                violations.push(format!("Check 32 I/O error: {}", err));
                continue;
            }
        };
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

/// Check 33: Zero Python Scripts & Operational Python/Pytest Reference Invariant.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C Rust-native toolchain purity controls.
/// Recursively walks `repo_root: &Path` to reject any physical Python source/bytecode file, cache/venv
/// directory, or Python manifest on disk and scans governed text files across `skills/`, `.agents/skills/`,
/// `rules/`, `.pipeline/` (excluding `.pipeline/defects/`), `docs/`, `README.md`, and `AGENTS.md` for
/// operational Python/Pytest tokens.
/// ## 2. Invariants & Mathematical Properties
/// - Zero files in `repo_root` (excluding `target`, `.git`, `build`, `Pods`, `ephemeral`, and `.dart_tool`)
///   may have extension `.py`, `.pyc`, `.pyo`, `.pyd`, `.pyi`, or `.ipynb`, or match Python manifest names
///   (`pyproject.toml`, `requirements.txt`, `pipfile`, `pipfile.lock`, `poetry.lock`, `setup.cfg`).
/// - Zero directories in `repo_root` may match `__pycache__`, `.pytest_cache`, `.mypy_cache`, `.ruff_cache`,
///   `.venv`, or `venv`.
/// - Zero governed `.md`, `.yaml`, `.yml`, `.json`, or `.toml` lines (outside explicit governance
///   prohibition clauses and `.pipeline/defects/` historical dossiers) may contain operational
///   Python/Pytest tokens.
/// ## 3. Preconditions
/// `repo_root: &Path` must be an accessible directory path on the local filesystem.
/// ## 4. Postconditions
/// Returns `Ok(())` if zero Python files/directories/manifests and zero operational Python/Pytest references
/// exist, or `Err(Vec<String>)` with sorted, deduplicated relative paths and `rel_path:line_no` diagnostics.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N + G * L) where N is repository entry count, G is governed text file count,
/// and L is average line count per governed file. Space Complexity: O(V) where V is violation count.
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on any discovered Python artifact, operational Python/Pytest token, or filesystem I/O error.
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-33-ZERO-PYTHON-SCRIPTS].
pub fn check_zero_python_scripts(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    for entry_res in WalkDir::new(repo_root).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let name = e.file_name().to_string_lossy();
            name != "target"
                && name != ".git"
                && name != "build"
                && name != "Pods"
                && name != "ephemeral"
                && name != ".dart_tool"
        } else {
            true
        }
    }) {
        let entry = match entry_res {
            Ok(e) => e,
            Err(err) => {
                violations.push(format!("Check 33 I/O error: {}", err));
                continue;
            }
        };
        let fname = entry.file_name().to_string_lossy();
        let lower_name = fname.to_ascii_lowercase();
        if entry.file_type().is_dir() {
            if matches!(
                lower_name.as_str(),
                "__pycache__" | ".pytest_cache" | ".mypy_cache" | ".ruff_cache" | ".venv" | "venv"
            ) {
                let rel = entry
                    .path()
                    .strip_prefix(repo_root)
                    .unwrap_or(entry.path())
                    .display()
                    .to_string();
                violations.push(rel);
            }
            continue;
        }
        if entry.file_type().is_file() {
            let is_py_ext = lower_name.ends_with(".py")
                || entry
                    .path()
                    .extension()
                    .map(|ext| {
                        let e = ext.to_string_lossy().to_ascii_lowercase();
                        matches!(e.as_str(), "py" | "pyc" | "pyo" | "pyd" | "pyi" | "ipynb")
                    })
                    .unwrap_or(false);
            let is_py_manifest = matches!(
                lower_name.as_str(),
                "pyproject.toml"
                    | "requirements.txt"
                    | "pipfile"
                    | "pipfile.lock"
                    | "poetry.lock"
                    | "setup.cfg"
            );
            if is_py_ext || is_py_manifest {
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
    scan_governed_files_for_python_references(repo_root, &mut violations);
    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        violations.dedup();
        Err(violations)
    }
}

/// Traverses governed directories and root files to detect operational Python/Pytest references.
/// Realises: [REQ-CHECK-33-SCAN-GOVERNED-FILES]
fn scan_governed_files_for_python_references(repo_root: &Path, violations: &mut Vec<String>) {
    let governed_dirs = [
        repo_root.join("skills"),
        repo_root.join(".agents").join("skills"),
        repo_root.join("rules"),
        repo_root.join(".pipeline"),
        repo_root.join("docs"),
    ];

    for dir_path in &governed_dirs {
        if !dir_path.is_dir() {
            continue;
        }
        let is_pipeline_root = dir_path
            .file_name()
            .map(|n| n.to_string_lossy() == ".pipeline")
            .unwrap_or(false);
        for entry_res in WalkDir::new(dir_path).into_iter().filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                if is_pipeline_root && name == "defects" {
                    return false;
                }
                name != "target"
                    && name != ".git"
                    && name != "build"
                    && name != "Pods"
                    && name != "ephemeral"
                    && name != ".dart_tool"
            } else {
                true
            }
        }) {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    violations.push(format!("Check 33 I/O error: {}", err));
                    continue;
                }
            };
            if entry.file_type().is_file() && is_governed_python_scan_extension(entry.path()) {
                scan_file_for_python_references(entry.path(), repo_root, violations);
            }
        }
    }

    let root_files = [repo_root.join("README.md"), repo_root.join("AGENTS.md")];
    for file_path in &root_files {
        if file_path.is_file() {
            scan_file_for_python_references(file_path, repo_root, violations);
        }
    }
}

/// Evaluates whether `path` carries a governed text extension while excluding `*.digest.json` sidecars.
/// Realises: [REQ-CHECK-33-GOVERNED-EXTENSIONS]
fn is_governed_python_scan_extension(path: &Path) -> bool {
    if let Some(fname) = path
        .file_name()
        .map(|n| n.to_string_lossy().to_ascii_lowercase())
    {
        if fname.ends_with(".digest.json") || fname.ends_with("-digest.json") {
            return false;
        }
    }
    let ext = match path.extension() {
        Some(e) => e.to_string_lossy().to_ascii_lowercase(),
        None => return false,
    };
    matches!(ext.as_str(), "md" | "yaml" | "yml" | "json" | "toml")
}

/// Scans a single governed text file line-by-line for operational Python/Pytest tokens.
/// Realises: [REQ-CHECK-33-LINE-SCANNER]
fn scan_file_for_python_references(path: &Path, repo_root: &Path, violations: &mut Vec<String>) {
    let rel = path
        .strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) => {
            violations.push(format!("{}: Unable to read file in Check 33: {}", rel, err));
            return;
        }
    };

    for (idx, raw_line) in content.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let ascii_folded = raw_line.to_ascii_lowercase();
        if is_python_prohibition_clause(&ascii_folded) {
            continue;
        }
        if let Some(token_label) = detect_operational_python_token(&ascii_folded) {
            violations.push(format!(
                "{}:{}: forbidden operational Python/Pytest reference ({}): {}",
                rel,
                line_no,
                token_label,
                raw_line.trim()
            ));
        }
    }
}

/// Determines whether an ASCII-lowercased line states an explicit governance prohibition clause.
/// Realises: [REQ-CHECK-33-PROHIBITION-CLAUSE-EXEMPTION]
fn is_python_prohibition_clause(ascii_folded: &str) -> bool {
    ascii_folded.contains("prohibit")
        || ascii_folded.contains("forbid")
        || ascii_folded.contains("retired")
        || ascii_folded.contains("zero python")
        || ascii_folded.contains("check 33")
        || ascii_folded.contains("never invoke")
        || ascii_folded.contains("never run")
        || ascii_folded.contains("never execute")
        || ascii_folded.contains("do not invoke")
        || ascii_folded.contains("do not run")
        || contains_ascii_word(ascii_folded, "ban")
        || contains_ascii_word(ascii_folded, "bans")
        || contains_ascii_word(ascii_folded, "banned")
}

/// Checks whether `haystack` contains `word` delimited on both sides by non-identifier boundaries.
/// Realises: [REQ-CHECK-33-PROHIBITION-CLAUSE-EXEMPTION]
fn contains_ascii_word(haystack: &str, word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    let bytes = haystack.as_bytes();
    for (idx, _) in haystack.match_indices(word) {
        let before_ok = match idx.checked_sub(1).and_then(|prev| bytes.get(prev)) {
            Some(&b) => !b.is_ascii_alphanumeric() && b != b'_',
            None => true,
        };
        let after_idx = idx.saturating_add(word.len());
        let after_ok = match bytes.get(after_idx) {
            Some(&b) => !b.is_ascii_alphanumeric() && b != b'_',
            None => true,
        };
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// Detects whether an ASCII-lowercased line contains an operational Python or Pytest token.
/// Realises: [REQ-CHECK-33-OPERATIONAL-TOKEN-DETECTOR]
fn detect_operational_python_token(ascii_folded: &str) -> Option<&'static str> {
    if ascii_folded.contains("python3") {
        Some("python3")
    } else if ascii_folded.contains("#!/usr/bin/env python")
        || ascii_folded.contains("#!/usr/bin/python")
    {
        Some("python shebang")
    } else if ascii_folded.contains("pytest") {
        Some("pytest")
    } else if ascii_folded.contains("pip install")
        || ascii_folded.contains("pip3 ")
        || ascii_folded.contains("uv run ")
        || ascii_folded.contains("poetry run ")
    {
        Some("python package runner")
    } else if ascii_folded.contains("models/python") {
        Some("models/python")
    } else if ascii_folded.contains("unittest") {
        Some("unittest")
    } else if ascii_folded.contains("execution_environment: [python") {
        Some("execution_environment: [python")
    } else if ascii_folded.contains("language: python") {
        Some("language: python")
    } else if contains_dot_py_token(ascii_folded) {
        Some(".py")
    } else {
        None
    }
}

/// Checks whether an ASCII-lowercased line contains a boundary-terminated `.py` extension token.
/// Realises: [REQ-CHECK-33-DOT-PY-BOUNDARY-SCAN]
fn contains_dot_py_token(ascii_folded: &str) -> bool {
    let bytes = ascii_folded.as_bytes();
    for (idx, _) in ascii_folded.match_indices(".py") {
        let after_idx = idx.saturating_add(3);
        let is_boundary = match bytes.get(after_idx) {
            Some(&b) => !b.is_ascii_alphanumeric() && b != b'_',
            None => true,
        };
        if is_boundary {
            return true;
        }
    }
    false
}

/// Zero-allocation `(prefix, suffix)` token pair for self-hosting Check 34 source scans.
/// Realises: [REQ-CHECK-34-SPLIT-TOKEN-MATCH]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SplitToken(&'static str, &'static str);

/// Specification pairing an audited module's path-match prefix with its canonical workspace stem.
/// Realises: [REQ-CHECK-34-AUDITED-MODULE-PATH]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AuditedModuleSpec {
    match_prefix: &'static str,
    workspace_stem: &'static str,
}

/// Complete ordered registry of Check 34 audited compiler modules (`<module>.rs` or `<module>/`).
/// Realises: [REQ-CHECK-34-AUDITED-MODULE-PATH]
const AUDITED_MODULES: &[AuditedModuleSpec] = &[
    AuditedModuleSpec {
        match_prefix: "checks/structural",
        workspace_stem: "crates/verify-baseline/src/checks/structural",
    },
    AuditedModuleSpec {
        match_prefix: "parity_gates",
        workspace_stem: "crates/verify-baseline/src/checks/parity_gates",
    },
    AuditedModuleSpec {
        match_prefix: "docstring_gate",
        workspace_stem: "crates/verify-baseline/src/checks/docstring_gate",
    },
    AuditedModuleSpec {
        match_prefix: "defect_dossier",
        workspace_stem: "crates/verify-baseline/src/defect_dossier",
    },
    AuditedModuleSpec {
        match_prefix: "deap-core/src/diagnostics",
        workspace_stem: "crates/deap-core/src/diagnostics",
    },
    AuditedModuleSpec {
        match_prefix: "deap-core/src/rules",
        workspace_stem: "crates/deap-core/src/rules",
    },
    AuditedModuleSpec {
        match_prefix: "deap-core/src/workspace",
        workspace_stem: "crates/deap-core/src/workspace",
    },
    AuditedModuleSpec {
        match_prefix: "deap-core/src/markdown",
        workspace_stem: "crates/deap-core/src/markdown",
    },
    AuditedModuleSpec {
        match_prefix: "deap-core/src/sysml_ast",
        workspace_stem: "crates/deap-core/src/sysml_ast",
    },
    AuditedModuleSpec {
        match_prefix: "deap-core/src/source_ast",
        workspace_stem: "crates/deap-core/src/source_ast",
    },
    AuditedModuleSpec {
        match_prefix: "compile-sysml/src/docstring",
        workspace_stem: "crates/compile-sysml/src/docstring",
    },
    AuditedModuleSpec {
        match_prefix: "ingest-sysml/src/translators/code",
        workspace_stem: "crates/ingest-sysml/src/translators/code",
    },
];

/// Split-token table detecting banned regular-expression crate imports and qualified paths.
/// Realises: [REQ-CHECK-34-ZERO-REGEX-INVARIANT]
const FORBIDDEN_REGEX_TOKENS: &[SplitToken] = &[
    SplitToken("Reg", "ex::"),
    SplitToken("reg", "ex::"),
    SplitToken("use reg", "ex;"),
    SplitToken("extern crate reg", "ex;"),
];

/// Split-token table for static string-slice types gated on `const`, `static`, or `=` bindings.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
const BINDING_STR_SLICE_TOKENS: &[SplitToken] =
    &[SplitToken("&[", "&str]"), SplitToken(": &[", "&str]")];

/// Split-token table for unconditionally forbidden static string-slice array and iterator types.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
const UNCONDITIONAL_STR_SLICE_TOKENS: &[SplitToken] = &[
    SplitToken("&[(", "&str"),
    SplitToken("[", "&str;"),
    SplitToken("const : &[", "&str]"),
    SplitToken("&'static [", "&'static str]"),
    SplitToken("[&'static ", "str;"),
    SplitToken("IntoIter<", "&'static str"),
];

/// Split-token table for fail-open `.filter_map(|...| ....ok())` closures.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
const FAIL_OPEN_FILTER_MAP_TOKENS: &[SplitToken] = &[
    SplitToken(".filter_map(|e| e.ok", "())"),
    SplitToken(".filter_map(|res| res.ok", "())"),
    SplitToken(".filter_map(|x| x.ok", "())"),
];

/// Split-token table for direct indexing immediately following lowercase string conversion.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
const DIRECT_LOWER_SLICE_TOKENS: &[SplitToken] = &[
    SplitToken(".to_lowercase()", "["),
    SplitToken(".to_ascii_lowercase()", "["),
];

/// Split-token table for range indexing on lowercase-named local bindings.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
const LOWER_VAR_INDEX_TOKENS: &[SplitToken] =
    &[SplitToken("lower", "["), SplitToken("_lower", "[")];

/// Forbidden Cargo dependency name split across two slices to avoid self-scan false positives.
/// Realises: [REQ-CARGO-TOML-REGEX-SCAN]
const FORBIDDEN_CARGO_DEP_TOKEN: SplitToken = SplitToken("reg", "ex");

/// Checks whether `haystack` contains `prefix` immediately followed by `suffix` without allocation.
/// Realises: [REQ-CHECK-34-SPLIT-TOKEN-MATCH]
fn has_split_token(haystack: &str, prefix: &str, suffix: &str) -> bool {
    if prefix.is_empty() {
        return haystack.contains(suffix);
    }
    for (idx, _) in haystack.match_indices(prefix) {
        if haystack
            .get(idx.saturating_add(prefix.len())..)
            .is_some_and(|rest| rest.starts_with(suffix))
        {
            return true;
        }
    }
    false
}

/// Checks whether `haystack` matches at least one `(prefix, suffix)` pair in `tokens`.
/// Realises: [REQ-CHECK-34-SPLIT-TOKEN-MATCH]
fn any_split_token(haystack: &str, tokens: &[SplitToken]) -> bool {
    tokens
        .iter()
        .any(|tok| has_split_token(haystack, tok.0, tok.1))
}

/// Checks whether `line` contains `prefix` + `suffix` bounded on both sides by non-identifier bytes.
/// Realises: [REQ-CHECK-34-SPLIT-TOKEN-MATCH]
fn contains_bounded_split_token(line: &str, prefix: &str, suffix: &str) -> bool {
    if prefix.is_empty() {
        return contains_bounded_token(line, suffix);
    }
    let bytes = line.as_bytes();
    let total_len = prefix.len().saturating_add(suffix.len());
    for (idx, _) in line.match_indices(prefix) {
        let has_suffix = line
            .get(idx.saturating_add(prefix.len())..)
            .is_some_and(|rest| rest.starts_with(suffix));
        if !has_suffix {
            continue;
        }
        let before_ok = match idx.checked_sub(1).and_then(|prev| bytes.get(prev)) {
            Some(&b) => !b.is_ascii_alphanumeric() && b != b'_' && b != b'-',
            None => true,
        };
        let after_ok = match bytes.get(idx.saturating_add(total_len)) {
            Some(&b) => !b.is_ascii_alphanumeric() && b != b'_' && b != b'-',
            None => true,
        };
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// Checks whether `path` matches `<prefix>.rs` or a subpath under `<prefix>/` at a path boundary.
/// Realises: [REQ-CHECK-34-AUDITED-MODULE-PATH]
fn matches_module_file_or_dir(path: &str, prefix: &str) -> bool {
    if prefix.is_empty() {
        return false;
    }
    let bytes = path.as_bytes();
    for (idx, _) in path.match_indices(prefix) {
        let before_ok = match idx.checked_sub(1).and_then(|prev| bytes.get(prev)) {
            Some(&b) => b == b'/' || b == b'\\',
            None => true,
        };
        if !before_ok {
            continue;
        }
        let tail = path.get(idx.saturating_add(prefix.len())..).unwrap_or("");
        if tail == ".rs" || tail.starts_with('/') || tail.starts_with('\\') {
            return true;
        }
    }
    false
}

/// Determines whether a Rust source path belongs to a Check 34 audited module file or directory.
/// Realises: [REQ-CHECK-34-AUDITED-MODULE-PATH]
fn is_audited_source_path(rel: &str) -> bool {
    if !rel.ends_with(".rs") {
        return false;
    }
    AUDITED_MODULES
        .iter()
        .any(|spec| matches_module_file_or_dir(rel, spec.match_prefix))
}

/// Determines whether strict fail-open I/O match arm rules apply to `file_rel`.
/// Realises: [REQ-CHECK-34-STRICT-IO-ARMS-SCOPE]
fn enforces_strict_io_arms(file_rel: &str) -> bool {
    !matches_module_file_or_dir(file_rel, "sysml_ast")
        && !matches_module_file_or_dir(file_rel, "workspace")
        && !matches_module_file_or_dir(file_rel, "parity_gates")
}

/// Check 34: Zero Regex Invariant & Self-Hosting Compiler Purity Gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C safety-critical software requirements.
/// Mechanically enforces that zero regular expressions exist across the compiler codebase
/// and enforces complete AST purity and tiered Rustdoc on audited modules and module directories.
///
/// ## 2. Invariants & Mathematical Properties
/// - All AST parsers and scanners must be deterministic finite-state machines without regex.
/// - Audited modules (`<module>.rs` and `<module>/`) must possess zero static string-slice arrays,
///   zero stubs, zero fail-open constructs, zero cross-string UTF-8 byte slices, and tiered Rustdoc.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible local filesystem directory.
///
/// ## 4. Postconditions
/// Returns `Ok(())` on clean verification.
/// Returns `Err(Vec<String>)` containing explicit file, line, and diagnostic messages on violation.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(F * L) where F is file count and L is average line length.
/// Space Complexity: O(V) where V is violation count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on any parse, I/O, or pattern matching error.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-34-ZERO-REGEX-INVARIANT].
pub fn check_zero_regex(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();

    let root_cargo = repo_root.join("Cargo.toml");
    if root_cargo.is_file() {
        scan_cargo_toml_file(&root_cargo, repo_root, &mut violations);
        if let Err(errs) = WorkspaceManifestForbidGate::verify(repo_root) {
            violations.extend(errs);
        }
    }

    let crates_dir = repo_root.join("crates");
    if crates_dir.is_dir() {
        for entry_res in WalkDir::new(&crates_dir).into_iter().filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !matches!(
                    name.as_ref(),
                    "target" | ".git" | "build" | "Pods" | "ephemeral" | ".dart_tool"
                )
            } else {
                true
            }
        }) {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    violations.push(format!("Check 34 I/O error: {}", err));
                    continue;
                }
            };
            if entry.file_type().is_file() {
                scan_crates_entry_for_check_34(entry.path(), repo_root, &mut violations);
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

/// Dispatches a single file inside `crates/` to `Cargo.toml` or `.rs` Check 34 purity auditing.
/// Realises: [REQ-CHECK-34-ZERO-REGEX-INVARIANT]
fn scan_crates_entry_for_check_34(path: &Path, repo_root: &Path, violations: &mut Vec<String>) {
    if path.file_name().is_some_and(|name| name == "Cargo.toml") {
        scan_cargo_toml_file(path, repo_root, violations);
        return;
    }
    if !path.extension().is_some_and(|ext| ext == "rs") {
        return;
    }
    let rel = path
        .strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string();
    match fs::read_to_string(path) {
        Ok(content) => {
            let is_audited = is_audited_source_path(&rel);
            if let Err(errs) = audit_source_purity(&content, &rel, is_audited) {
                if is_audited {
                    violations.extend(errs);
                } else {
                    for err in errs {
                        if err.contains("regex usage detected") {
                            violations.push(err);
                        }
                    }
                }
            }
        }
        Err(err) => {
            violations.push(format!("{}: Unable to read file: {}", rel, err));
        }
    }
}

/// Check 34 Layer 1: Self-hosting compiler AST purity gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Mechanically audits verified modules
/// across the codebase to ensure zero regex, zero static string-slice arrays, zero hollow stubs,
/// zero fail-open error swallowing, zero cross-string byte-slicing hazards, and tiered Tier A / Tier B/C Rustdoc completeness.
///
/// ## 2. Invariants & Mathematical Properties
/// Audited modules (`<module>.rs` or decomposed `<module>/` directories) must satisfy all 6 purity conditions simultaneously.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible local directory.
///
/// ## 4. Postconditions
/// Returns `Ok(())` on complete compliance.
/// Returns `Err(Vec<String>)` with sorted list of purity violations.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(A * L) where A is audited files count and L is lines count.
/// Space Complexity: O(V) where V is violation count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed if any audited file or directory is missing, empty, unreadable, or non-compliant.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-34-COMPILER-PURITY-GATE].
pub fn check_compiler_ast_purity(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();

    for module in AUDITED_MODULES {
        let stem_rel = module.workspace_stem;
        let single_file = repo_root.join(format!("{stem_rel}.rs"));
        let module_dir = repo_root.join(stem_rel);
        let has_file = single_file.is_file();
        let has_dir = module_dir.is_dir();

        if has_file {
            audit_mandatory_file_path(&single_file, repo_root, &mut violations);
        }
        if has_dir {
            audit_decomposed_module_dir(&module_dir, repo_root, stem_rel, &mut violations);
        }
        if !has_file && !has_dir {
            violations.push(format!(
                "Check 34: Missing mandatory audited file '{}.rs'",
                stem_rel
            ));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        Err(violations)
    }
}

/// Audits a single mandatory Rust file on disk against all Check 34 Layer 1 purity rules.
/// Realises: [REQ-CHECK-34-COMPILER-PURITY-GATE]
fn audit_mandatory_file_path(file_path: &Path, repo_root: &Path, violations: &mut Vec<String>) {
    let rel = file_path
        .strip_prefix(repo_root)
        .unwrap_or(file_path)
        .display()
        .to_string();
    match fs::read_to_string(file_path) {
        Ok(content) => {
            if let Err(errs) = audit_source_purity(&content, &rel, true) {
                violations.extend(errs);
            }
        }
        Err(err) => {
            violations.push(format!(
                "Check 34: Unable to read audited file '{}': {}",
                rel, err
            ));
        }
    }
}

/// Audits every `.rs` file inside a decomposed `<module>/` directory, failing closed if empty.
/// Realises: [REQ-CHECK-34-COMPILER-PURITY-GATE]
fn audit_decomposed_module_dir(
    module_dir: &Path,
    repo_root: &Path,
    stem_rel: &str,
    violations: &mut Vec<String>,
) {
    let mut rs_files = Vec::new();
    for entry_res in WalkDir::new(module_dir) {
        match entry_res {
            Ok(entry) => {
                let path = entry.path();
                if entry.file_type().is_file() && path.extension().is_some_and(|ext| ext == "rs") {
                    rs_files.push(path.to_path_buf());
                }
            }
            Err(err) => {
                violations.push(format!(
                    "Check 34: Unable to traverse audited module directory '{}': {}",
                    stem_rel, err
                ));
            }
        }
    }
    rs_files.sort();
    if rs_files.is_empty() {
        violations.push(format!(
            "Check 34: Missing mandatory audited file '{}.rs'",
            stem_rel
        ));
        return;
    }
    for rs_path in &rs_files {
        audit_mandatory_file_path(rs_path, repo_root, violations);
    }
}

/// Masks non-doc comments, string literals, raw strings, and character literals while preserving 1-based line numbers.
/// Realises: [REQ-CHECK-34-MASK-COMMENTS-AND-LITERALS]
fn mask_comments_and_literals_preserving_lines(source: &str) -> Vec<String> {
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i: usize = 0;
    let mut lines: Vec<String> = Vec::new();
    let mut current_line = String::new();
    let mut only_ws_on_line = true;

    while i < len {
        let c = match chars.get(i).copied() {
            Some(ch) => ch,
            None => break,
        };

        if c == '\r' {
            i = i.saturating_add(1);
            continue;
        }

        if c == '\n' {
            lines.push(std::mem::take(&mut current_line));
            only_ws_on_line = true;
            i = i.saturating_add(1);
            continue;
        }

        // Line comment `//` or doc comment `///` / `//!`
        if c == '/' && chars.get(i.saturating_add(1)).copied() == Some('/') {
            let third = chars.get(i.saturating_add(2)).copied();
            if only_ws_on_line && (third == Some('/') || third == Some('!')) {
                while let Some(ch) = chars.get(i).copied() {
                    if ch == '\n' {
                        break;
                    }
                    if ch != '\r' {
                        current_line.push(ch);
                    }
                    i = i.saturating_add(1);
                }
                continue;
            }
            i = i.saturating_add(2);
            while let Some(ch) = chars.get(i).copied() {
                if ch == '\n' {
                    break;
                }
                i = i.saturating_add(1);
            }
            continue;
        }

        // Nested block comment `/* ... */`
        if c == '/' && chars.get(i.saturating_add(1)).copied() == Some('*') {
            current_line.push(' ');
            let mut depth: usize = 1;
            i = i.saturating_add(2);
            while depth > 0 {
                match chars.get(i).copied() {
                    None => break,
                    Some('\n') => {
                        lines.push(std::mem::take(&mut current_line));
                        only_ws_on_line = true;
                        i = i.saturating_add(1);
                    }
                    Some('/') if chars.get(i.saturating_add(1)).copied() == Some('*') => {
                        depth = depth.saturating_add(1);
                        i = i.saturating_add(2);
                    }
                    Some('*') if chars.get(i.saturating_add(1)).copied() == Some('/') => {
                        depth = depth.saturating_sub(1);
                        i = i.saturating_add(2);
                    }
                    Some(_) => {
                        i = i.saturating_add(1);
                    }
                }
            }
            continue;
        }

        // Raw string literal `r"..."`, `r#"..."#`, `br"..."`, `br#"..."#`
        let prev_is_ident = if i > 0 {
            chars
                .get(i.saturating_sub(1))
                .copied()
                .map(|p| p.is_ascii_alphanumeric() || p == '_' || p == '\'')
                .unwrap_or(false)
        } else {
            false
        };
        if !prev_is_ident {
            let raw_r_pos = if c == 'r' {
                Some(i)
            } else if c == 'b' && chars.get(i.saturating_add(1)).copied() == Some('r') {
                Some(i.saturating_add(1))
            } else {
                None
            };
            if let Some(r_pos) = raw_r_pos {
                let mut scan = r_pos.saturating_add(1);
                let mut hashes: usize = 0;
                while chars.get(scan).copied() == Some('#') {
                    hashes = hashes.saturating_add(1);
                    scan = scan.saturating_add(1);
                }
                if chars.get(scan).copied() == Some('"') {
                    only_ws_on_line = false;
                    current_line.push('"');
                    current_line.push('"');
                    i = scan.saturating_add(1);
                    while let Some(ch) = chars.get(i).copied() {
                        if ch == '\n' {
                            lines.push(std::mem::take(&mut current_line));
                            only_ws_on_line = true;
                            i = i.saturating_add(1);
                        } else if ch == '"' {
                            i = i.saturating_add(1);
                            let mut matched: usize = 0;
                            while matched < hashes
                                && chars.get(i.saturating_add(matched)).copied() == Some('#')
                            {
                                matched = matched.saturating_add(1);
                            }
                            if matched == hashes {
                                i = i.saturating_add(hashes);
                                break;
                            }
                        } else {
                            i = i.saturating_add(1);
                        }
                    }
                    continue;
                }
            }
        }

        // Standard string literal `"..."`
        if c == '"' {
            only_ws_on_line = false;
            current_line.push('"');
            current_line.push('"');
            i = i.saturating_add(1);
            while let Some(ch) = chars.get(i).copied() {
                if ch == '\\' {
                    if chars.get(i.saturating_add(1)).copied() == Some('\n') {
                        lines.push(std::mem::take(&mut current_line));
                        only_ws_on_line = true;
                    }
                    i = i.saturating_add(2);
                } else if ch == '"' {
                    i = i.saturating_add(1);
                    break;
                } else {
                    if ch == '\n' {
                        lines.push(std::mem::take(&mut current_line));
                        only_ws_on_line = true;
                    }
                    i = i.saturating_add(1);
                }
            }
            continue;
        }

        // Character literal `'x'` or `'\\.'` (distinguished from lifetimes `'static` / `'a`)
        if c == '\'' {
            if chars.get(i.saturating_add(1)).copied() == Some('\\') {
                only_ws_on_line = false;
                current_line.push_str("' '");
                i = i.saturating_add(2);
                while let Some(ch) = chars.get(i).copied() {
                    i = i.saturating_add(1);
                    if ch == '\'' || ch == '\n' {
                        if ch == '\n' {
                            lines.push(std::mem::take(&mut current_line));
                            only_ws_on_line = true;
                        }
                        break;
                    }
                }
                continue;
            } else if chars.get(i.saturating_add(2)).copied() == Some('\'')
                && chars.get(i.saturating_add(1)).copied() != Some('\n')
            {
                only_ws_on_line = false;
                current_line.push_str("' '");
                i = i.saturating_add(3);
                continue;
            }
            only_ws_on_line = false;
            current_line.push('\'');
            i = i.saturating_add(1);
            continue;
        }

        if !c.is_whitespace() {
            only_ws_on_line = false;
        }
        current_line.push(c);
        i = i.saturating_add(1);
    }

    if (!current_line.is_empty() || !source.ends_with('\n')) && !source.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Brace-depth state tracker for `mod tests` blocks during Check 34 line-by-line source auditing.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
#[derive(Default)]
struct ModTestsScope {
    active: bool,
    brace_depth: usize,
    entered_block: bool,
    has_test: bool,
}

impl ModTestsScope {
    /// Resets and activates `mod tests` scope tracking upon encountering a `mod tests` declaration.
    /// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
    fn enter(&mut self) {
        self.active = true;
        self.brace_depth = 0;
        self.entered_block = false;
        self.has_test = false;
    }
}

/// Outcome of inspecting one masked line inside an active `mod tests` block.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
enum ModTestsStep<'a> {
    Consumed,
    TrailingProduction(&'a str),
}

/// Audits a single Rust source string for compiler purity violations (a)-(f).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, ECSS self-hosting compiler purity requirements, and DEAP Rust
/// Coding Standard v2.0 Section 4. Mechanically audits source lines for banned regex, static
/// string-slice arrays, hollow gate stubs, fail-open error swallowing, cross-string byte-slicing
/// hazards, tiered Tier A / Tier B/C Rustdoc completeness, and `C-PURITY-03` test anti-gaming invariants.
///
/// ## 2. Invariants & Mathematical Properties
/// - Banned constructs are matched via static split-token tables with zero runtime string concatenation.
/// - Non-doc comments and string/char literals are masked before token inspection.
/// - `mod tests` scope tracks brace depth `{` / `}` so closed blocks do not exempt trailing code.
/// - In audited modules, every item and structured module header must satisfy its normative Tier A
///   (7-section Rustdoc or 4-Block Contract) or Tier B/C (summary + `Realises: [...]`) contract.
///
/// ## 3. Preconditions
/// `source` must be valid UTF-8 Rust source code.
///
/// ## 4. Postconditions
/// Returns `Ok(())` if source passes all purity checks.
/// Returns `Err(Vec<String>)` containing all detected violations.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(L) where L is line count.
/// Space Complexity: O(V) where V is violation count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Pure non-panicking string analysis.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY].
pub fn audit_source_purity(
    source: &str,
    file_rel: &str,
    check_rustdoc: bool,
) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    collect_forbidden_clippy_allows(source, file_rel, &mut violations);

    let masked_lines = mask_comments_and_literals_preserving_lines(source);
    if check_rustdoc {
        if let Err(doc_errs) = super::docstring_gate::audit_rust_source_docstrings(source, file_rel)
        {
            violations.extend(doc_errs);
        }
        audit_tier_a_module_docblocks(&masked_lines, file_rel, &mut violations);
    }

    let mut scope = ModTestsScope::default();
    let strict_io = enforces_strict_io_arms(file_rel);

    for (idx, masked_line) in masked_lines.iter().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = masked_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//!") || trimmed.starts_with("///") {
            continue;
        }

        if !scope.active && contains_bounded_split_token(trimmed, "mod", " tests") {
            scope.enter();
        }

        let code = if scope.active {
            match advance_mod_tests_line(&mut scope, trimmed, file_rel, line_no, &mut violations) {
                ModTestsStep::Consumed => continue,
                ModTestsStep::TrailingProduction(trailing) => trailing,
            }
        } else {
            trimmed
        };

        let line_ctx = LinePurityContext {
            file_rel,
            line_no,
            check_rustdoc,
            strict_io,
        };
        audit_production_code_line(code, line_ctx, &mut violations);
    }

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        violations.dedup();
        Err(violations)
    }
}

/// Appends violations for any `#[allow(clippy::...)]` suppressing a `RequiredClippyForbidLint`.
/// Realises: [REQ-CHECK-34-CLIPPY-ALLOW-TOKEN-SCAN]
fn collect_forbidden_clippy_allows(source: &str, file_rel: &str, violations: &mut Vec<String>) {
    for clippy_viol in scan_clippy_allow_tokens(source, file_rel) {
        let is_forbidden = RequiredClippyForbidLint::ALL
            .iter()
            .any(|lint| clippy_viol.attribute.contains(lint.as_str()));
        if is_forbidden {
            violations.push(format!(
                "{}:{}: forbidden clippy allow attribute detected: {}",
                clippy_viol.file, clippy_viol.line, clippy_viol.attribute
            ));
        }
    }
}

/// Audits one masked line inside `mod tests`, updating brace depth and returning any trailing production slice.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
fn advance_mod_tests_line<'a>(
    scope: &mut ModTestsScope,
    code: &'a str,
    file_rel: &str,
    line_no: usize,
    violations: &mut Vec<String>,
) -> ModTestsStep<'a> {
    if code.contains("#[test]") || code.contains("fn test_") {
        scope.has_test = true;
    }
    if any_split_token(code, FORBIDDEN_REGEX_TOKENS) {
        violations.push(format!("{}:{}: regex usage detected", file_rel, line_no));
    }
    if has_split_token(code, "#[ig", "nore") {
        violations.push(format!(
            "{}:{}: C-PURITY-03 ignored test attribute detected",
            file_rel, line_no
        ));
    }
    if has_split_token(code, "assert!", "(true)") {
        violations.push(format!(
            "{}:{}: C-PURITY-03 tautological assertion detected",
            file_rel, line_no
        ));
    }
    if code.contains(".exists()") && code.contains("return;") {
        violations.push(format!(
            "{}:{}: C-PURITY-03 vacuous conditional early-return in test detected",
            file_rel, line_no
        ));
    }
    if has_split_token(code, "temp_dir()", ".join(\"\")") {
        violations.push(format!(
            "{}:{}: C-PURITY-03 unisolated literal temp_dir().join(...) in test detected",
            file_rel, line_no
        ));
    }

    let mut closed_at_byte: Option<usize> = None;
    for (b_idx, ch) in code.char_indices() {
        if ch == '{' {
            scope.brace_depth = scope.brace_depth.saturating_add(1);
            scope.entered_block = true;
        } else if ch == '}' {
            scope.brace_depth = scope.brace_depth.saturating_sub(1);
            if scope.entered_block && scope.brace_depth == 0 {
                closed_at_byte = Some(b_idx.saturating_add(1));
                break;
            }
        }
    }

    if closed_at_byte.is_some() || (!scope.entered_block && code.contains(';')) {
        if !scope.has_test {
            violations.push(format!(
                "{}:{}: C-PURITY-03 empty or decoy mod tests block detected",
                file_rel, line_no
            ));
        }
        scope.active = false;
        if let Some(after_idx) = closed_at_byte {
            let trailing = code.get(after_idx..).unwrap_or("").trim();
            if !trailing.is_empty() {
                return ModTestsStep::TrailingProduction(trailing);
            }
        }
    }
    ModTestsStep::Consumed
}

/// Context carrying per-file Check 34 flags when auditing a single masked production line.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
#[derive(Clone, Copy)]
struct LinePurityContext<'a> {
    file_rel: &'a str,
    line_no: usize,
    check_rustdoc: bool,
    strict_io: bool,
}

/// Audits one masked production code line against Check 34 rules (a)-(e).
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
fn audit_production_code_line(
    code: &str,
    ctx: LinePurityContext<'_>,
    violations: &mut Vec<String>,
) {
    if any_split_token(code, FORBIDDEN_REGEX_TOKENS) {
        violations.push(format!(
            "{}:{}: regex usage detected",
            ctx.file_rel, ctx.line_no
        ));
    }
    if has_forbidden_static_str_slice(code) {
        violations.push(format!(
            "{}:{}: static string-slice array detected",
            ctx.file_rel, ctx.line_no
        ));
    }
    if has_split_token(code, "let", " _ = ") {
        violations.push(format!(
            "{}:{}: hollow gate stub detected (`let _ = `)",
            ctx.file_rel, ctx.line_no
        ));
    }
    if has_fail_open_error_swallowing(code, ctx.strict_io) {
        violations.push(format!(
            "{}:{}: fail-open error swallowing detected",
            ctx.file_rel, ctx.line_no
        ));
    }
    if has_cross_string_utf8_slice_hazard(code) {
        violations.push(format!(
            "{}:{}: cross-string UTF-8 byte-slice hazard detected",
            ctx.file_rel, ctx.line_no
        ));
    }
    if ctx.check_rustdoc && contains_direct_range_index(code) {
        violations.push(format!(
            "{}:{}: direct panicking range-slice indexing detected (use .get(..))",
            ctx.file_rel, ctx.line_no
        ));
    }
}

/// Detects banned static string-slice array or iterator types on one masked production line.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
fn has_forbidden_static_str_slice(code: &str) -> bool {
    if any_split_token(code, UNCONDITIONAL_STR_SLICE_TOKENS) {
        return true;
    }
    let is_binding_or_const =
        code.contains('=') || code.contains("const ") || code.contains("static ");
    is_binding_or_const && any_split_token(code, BINDING_STR_SLICE_TOKENS)
}

/// Detects fail-open error swallowing patterns on one masked production line.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
fn has_fail_open_error_swallowing(code: &str, strict_io: bool) -> bool {
    if any_split_token(code, FAIL_OPEN_FILTER_MAP_TOKENS) {
        return true;
    }
    let has_dot_ok = has_split_token(code, ".ok", "()");
    let has_read_str = has_split_token(code, "read_to", "_string");
    if has_dot_ok && (has_split_token(code, ".filter", "_map(") || has_read_str) {
        return true;
    }
    if !strict_io {
        return false;
    }
    if has_split_token(code, "Err(_)", " => continue") {
        return true;
    }
    let has_read_io = has_read_str || has_split_token(code, "read_", "dir(");
    has_split_token(code, "if let ", "Ok(") && has_read_io
}

/// Detects cross-string UTF-8 byte-slice indexing on lowercase-converted buffers.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
fn has_cross_string_utf8_slice_hazard(code: &str) -> bool {
    if any_split_token(code, DIRECT_LOWER_SLICE_TOKENS) {
        return true;
    }
    any_split_token(code, LOWER_VAR_INDEX_TOKENS) && has_split_token(code, ".", ".")
}

/// Audits top-level `//!` module header docblocks in `masked_lines` against the Tier A contract.
/// Realises: [REQ-CHECK-34-TIER-A-MODULE-DOCBLOCK]
fn audit_tier_a_module_docblocks(
    masked_lines: &[String],
    file_rel: &str,
    violations: &mut Vec<String>,
) {
    let mut module_doc: Vec<String> = Vec::new();
    let mut start_line: usize = 1;

    for (idx, line) in masked_lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//!") {
            if module_doc.is_empty() {
                start_line = idx.saturating_add(1);
            }
            module_doc.push(trimmed.to_string());
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with("#![") {
            continue;
        }
        flush_tier_a_module_docblock(&mut module_doc, file_rel, start_line, violations);
        if trimmed.contains("cfg(test)") || contains_bounded_split_token(trimmed, "mod", " tests") {
            break;
        }
    }
    flush_tier_a_module_docblock(&mut module_doc, file_rel, start_line, violations);
}

/// Validates and clears an accumulated `//!` module docblock against the Tier A contract.
/// Realises: [REQ-CHECK-34-FLUSH-TIER-A-MODULE-DOCBLOCK]
fn flush_tier_a_module_docblock(
    module_doc: &mut Vec<String>,
    file_rel: &str,
    start_line: usize,
    violations: &mut Vec<String>,
) {
    if module_doc.is_empty() {
        return;
    }
    if !is_legacy_single_line_module_title(module_doc)
        && !super::docstring_gate::is_compliant_rust_docblock(module_doc, true)
    {
        violations.push(format!(
            "{}:{}: Tier A module header missing valid 7-section Rustdoc, 4-Block Contract, or non-empty 'Realises: [...]' tag",
            file_rel, start_line
        ));
    }
    module_doc.clear();
}

/// Identifies a legacy single-line non-empty `//!` module title that carries no partial contract or `Realises:` claim.
/// Realises: [REQ-CHECK-34-LEGACY-SINGLE-LINE-MODULE-TITLE]
fn is_legacy_single_line_module_title(module_doc: &[String]) -> bool {
    if module_doc.len() != 1 {
        return false;
    }
    let Some(first) = module_doc.first() else {
        return false;
    };
    let body = first.trim().strip_prefix("//!").unwrap_or("").trim();
    body.chars().any(char::is_alphanumeric)
        && !body.starts_with('#')
        && !body.contains("Realises:")
        && !body.contains("Safety & Traceability")
        && !body.contains("Contract:")
        && !body.contains("Bounds & Faults:")
}

/// Detects direct panicking range-slice indexing (`expr[a..b]`) on one masked code line.
/// Realises: [REQ-CHECK-34-DIRECT-RANGE-INDEX-SCAN]
fn contains_direct_range_index(code: &str) -> bool {
    let bytes = code.as_bytes();
    let mut prev_byte: Option<u8> = None;
    for (open_idx, &byte) in bytes.iter().enumerate() {
        let is_index_operator = byte == b'['
            && prev_byte
                .map(|p| p.is_ascii_alphanumeric() || p == b'_' || p == b')' || p == b']')
                .unwrap_or(false);
        if is_index_operator {
            let mut depth: usize = 0;
            let mut close_idx = bytes.len();
            for (scan_idx, &scan_byte) in bytes.iter().enumerate().skip(open_idx) {
                if scan_byte == b'[' {
                    depth = depth.saturating_add(1);
                } else if scan_byte == b']' {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        close_idx = scan_idx;
                        break;
                    }
                }
            }
            let inner = code
                .get(open_idx.saturating_add(1)..close_idx)
                .unwrap_or(code);
            if has_split_token(inner, ".", ".") {
                return true;
            }
        }
        prev_byte = Some(byte);
    }
    false
}

/// Scans a `Cargo.toml` manifest for any forbidden dependency on the regex crate without allocation.
/// Realises: [REQ-CARGO-TOML-REGEX-SCAN]
fn scan_cargo_toml_file(path: &Path, repo_root: &Path, violations: &mut Vec<String>) {
    let rel = path
        .strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) => {
            violations.push(format!("{}: Unable to read Cargo.toml: {}", rel, err));
            return;
        }
    };

    for (idx, line) in content.lines().enumerate() {
        let code = match line.split_once('#') {
            Some((before, _)) => before,
            None => line,
        };
        let trimmed = code.trim();
        if trimmed.is_empty() {
            continue;
        }
        if cargo_toml_line_declares_forbidden_dep(trimmed) {
            violations.push(format!(
                "{}:{}: {}",
                rel,
                idx.saturating_add(1),
                line.trim()
            ));
        }
    }
}

/// Checks whether a comment-stripped `Cargo.toml` line declares or quotes `FORBIDDEN_CARGO_DEP_TOKEN`.
/// Realises: [REQ-CARGO-TOML-REGEX-SCAN]
fn cargo_toml_line_declares_forbidden_dep(trimmed: &str) -> bool {
    let (prefix, suffix) = (FORBIDDEN_CARGO_DEP_TOKEN.0, FORBIDDEN_CARGO_DEP_TOKEN.1);
    if let Some((dep_key, _)) = trimmed.split_once('=') {
        let key = dep_key.trim().trim_matches('"').trim_matches('\'');
        let Some(after_prefix) = key.strip_prefix(prefix) else {
            return false;
        };
        return after_prefix == suffix
            || after_prefix
                .strip_prefix(suffix)
                .is_some_and(|rest| rest.starts_with('.'));
    }
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let section = trimmed
            .get(1..trimmed.len().saturating_sub(1))
            .unwrap_or("")
            .trim();
        return section.split('.').any(|part| {
            part.trim()
                .trim_matches('"')
                .trim_matches('\'')
                .strip_prefix(prefix)
                == Some(suffix)
        });
    }
    has_split_token(trimmed, "\"reg", "ex\"") || has_split_token(trimmed, "'reg", "ex'")
}

/// Represents a detected `#[allow(clippy::...)]` or `#![allow(clippy::...)]` suppression violation.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34 (`REQ-0601`) static analysis suppression prohibition.
/// Captures the exact file, 1-based line number, and normalized attribute text of any detected
/// `clippy::` allow directive.
///
/// ## 2. Invariants & Mathematical Properties
/// `line` is always `>= 1`, `file` is the non-empty workspace-relative path, and `attribute`
/// is the whitespace-normalized attribute token sequence starting with `#[` or `#![`.
///
/// ## 3. Preconditions
/// Constructed by `scan_clippy_allow_tokens` when an active attribute contains `allow(clippy::`.
///
/// ## 4. Postconditions
/// Provides immutable diagnostic fields for baseline verification reporting.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1) field access. Space Complexity: O(A) where A is attribute length.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Pure value structure; cannot fail or panic.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-34-CLIPPY-SUPPRESSION-VIOLATION].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClippySuppressionViolation {
    /// Workspace-relative file path containing the forbidden attribute.
    pub file: String,
    /// 1-based line number where the attribute token starts.
    pub line: usize,
    /// Whitespace-normalized attribute representation.
    pub attribute: String,
}

/// Canonical Clippy lint keys required at `"forbid"` level in `[workspace.lints.clippy]`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and `REQ-0601` compiler lint enforcement.
/// Enumerates the 9 mandatory `[workspace.lints.clippy]` keys that must be configured at `"forbid"`
/// without relying on banned static string-slice arrays.
///
/// ## 2. Invariants & Mathematical Properties
/// Closed 9-variant enumeration corresponding 1-to-1 with `RequiredClippyForbidLint::ALL`.
///
/// ## 3. Preconditions
/// Used by `WorkspaceManifestForbidGate` and `audit_source_purity` during Check 34 verification.
///
/// ## 4. Postconditions
/// Guarantees exhaustive iteration over all required workspace forbid lint keys.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1) per variant. Space Complexity: O(1) `Copy` enum representation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Type-safe closed enum; invalid lint keys are unrepresentable.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-34-REQUIRED-CLIPPY-FORBID-LINT].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequiredClippyForbidLint {
    /// Forbids banned standard-library types configured in `clippy.toml`.
    DisallowedTypes,
    /// Forbids `.unwrap()` calls across production code.
    UnwrapUsed,
    /// Forbids `.expect(...)` calls across production code.
    ExpectUsed,
    /// Forbids `panic!(...)` macro invocations across production code.
    Panic,
    /// Forbids direct panicking indexing and slicing `expr[idx]` / `expr[a..b]`.
    IndexingSlicing,
    /// Forbids overly complex composite types.
    TypeComplexity,
    /// Forbids `dbg!(...)` macro invocations.
    DbgMacro,
    /// Forbids `todo!(...)` stubs.
    Todo,
    /// Forbids `unimplemented!(...)` stubs.
    Unimplemented,
}

impl RequiredClippyForbidLint {
    /// Complete ordered set of mandatory workspace Clippy forbid lints under `REQ-0601`.
    pub const ALL: &'static [Self] = &[
        Self::DisallowedTypes,
        Self::UnwrapUsed,
        Self::ExpectUsed,
        Self::Panic,
        Self::IndexingSlicing,
        Self::TypeComplexity,
        Self::DbgMacro,
        Self::Todo,
        Self::Unimplemented,
    ];

    /// Returns the exact `[workspace.lints.clippy]` TOML key name for this lint variant.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C static analysis configuration controls.
    /// Maps each strongly typed `RequiredClippyForbidLint` variant to its canonical Cargo
    /// manifest lint key without relying on untyped string-slice tables.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// Total, bijective, compile-time constant mapping from `RequiredClippyForbidLint` to
    /// the 9 required `[workspace.lints.clippy]` keys mandated by `REQ-0601`.
    ///
    /// ## 3. Preconditions
    /// `self` is a valid `RequiredClippyForbidLint` enum variant.
    ///
    /// ## 4. Postconditions
    /// Returns a non-empty `'static` ASCII identifier matching the Clippy lint name.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(1) constant branch dispatch.
    /// Space Complexity: O(1) zero heap allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Infallible total match over a closed enum; cannot panic or fail.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-CHECK-34-CLIPPY-FORBID-LINT-KEY].
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DisallowedTypes => "disallowed_types",
            Self::UnwrapUsed => "unwrap_used",
            Self::ExpectUsed => "expect_used",
            Self::Panic => "panic",
            Self::IndexingSlicing => "indexing_slicing",
            Self::TypeComplexity => "type_complexity",
            Self::DbgMacro => "dbg_macro",
            Self::Todo => "todo",
            Self::Unimplemented => "unimplemented",
        }
    }
}

/// Gate verifying workspace and member `Cargo.toml` lint enforcement invariants (`REQ-0601`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and `REQ-0601` manifest governance.
/// Provides a stateless verification gate that audits root `Cargo.toml` and `crates/*/Cargo.toml`
/// for mandatory `"forbid"` lint settings and workspace lint inheritance.
///
/// ## 2. Invariants & Mathematical Properties
/// Zero-sized unit struct; all verification logic is pure and deterministic via `Self::verify`.
///
/// ## 3. Preconditions
/// Invoked with a valid repository root path during Check 34 execution.
///
/// ## 4. Postconditions
/// Enforces closed-loop parity between `REQ-0601` lint mandates and workspace Cargo manifests.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1) type instantiation. Space Complexity: O(1) zero-sized type.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Stateless gate type; all I/O and manifest violations are surfaced via `Self::verify`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-34-WORKSPACE-MANIFEST-GATE-TYPE].
pub struct WorkspaceManifestForbidGate;

impl WorkspaceManifestForbidGate {
    /// Verifies root `Cargo.toml` forbid lints and member `crates/*/Cargo.toml` lint inheritance.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, and `REQ-0601` compiler enforcement mandates.
    /// Ensures the workspace root `Cargo.toml` sets `unsafe_code = "forbid"` under
    /// `[workspace.lints.rust]` and sets all 9 mandatory Clippy lints to `"forbid"` under
    /// `[workspace.lints.clippy]`, while every member crate under `crates/*/Cargo.toml`
    /// inherits `[lints] workspace = true` and omits `clap` derive macros.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - `[workspace.lints.rust]` must bind `unsafe_code` to `"forbid"`.
    /// - Every variant in `RequiredClippyForbidLint::ALL` must be bound to `"forbid"` in
    ///   `[workspace.lints.clippy]`.
    /// - Every `crates/*/Cargo.toml` manifest must declare `workspace = true` under `[lints]`.
    ///
    /// ## 3. Preconditions
    /// `repo_root` must reference a workspace directory containing `Cargo.toml`.
    ///
    /// ## 4. Postconditions
    /// Returns `Ok(())` iff root and member manifests satisfy all forbid-lint and inheritance
    /// constraints; returns `Err(Vec<String>)` with deterministic diagnostics otherwise.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: O(M * L) where M is manifest count and L is average line count per manifest.
    /// Space Complexity: O(V) where V is the number of detected manifest violations.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// Fails closed if `Cargo.toml` or any `crates/*/Cargo.toml` cannot be read or omits any
    /// required `"forbid"` setting or workspace lint inheritance directive.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-CHECK-34-WORKSPACE-MANIFEST-FORBID-GATE].
    pub fn verify(repo_root: &Path) -> Result<(), Vec<String>> {
        let mut violations = Vec::new();
        let root_cargo = repo_root.join("Cargo.toml");
        let content = match fs::read_to_string(&root_cargo) {
            Ok(c) => c,
            Err(err) => {
                return Err(vec![format!(
                    "Cargo.toml: Unable to read workspace manifest: {}",
                    err
                )]);
            }
        };

        let mut current_section = String::new();
        let mut rust_unsafe_code: Option<String> = None;
        let mut clippy_lints: std::collections::BTreeMap<String, String> =
            std::collections::BTreeMap::new();

        for raw_line in content.lines() {
            let without_comment = match raw_line.split_once('#') {
                Some((before, _)) => before,
                None => raw_line,
            };
            let trimmed = without_comment.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let sec = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
                current_section = sec.to_string();
                continue;
            }
            if let Some((raw_key, raw_val)) = trimmed.split_once('=') {
                let key = raw_key.trim().trim_matches('"').trim_matches('\'');
                let val = raw_val.trim().trim_matches('"').trim_matches('\'');
                if current_section == "workspace.lints.rust" && key == "unsafe_code" {
                    rust_unsafe_code = Some(val.to_string());
                } else if current_section == "workspace.lints.clippy" {
                    clippy_lints.insert(key.to_string(), val.to_string());
                }
            }
        }

        if rust_unsafe_code.as_deref() != Some("forbid") {
            violations.push(
                "Cargo.toml: [workspace.lints.rust] must set unsafe_code = \"forbid\"".to_string(),
            );
        }

        for lint in RequiredClippyForbidLint::ALL {
            let key = lint.as_str();
            match clippy_lints.get(key) {
                Some(level) if level == "forbid" => {}
                Some(level) => {
                    violations.push(format!(
                        "Cargo.toml: [workspace.lints.clippy] lint '{}' must be \"forbid\", found \"{}\"",
                        key, level
                    ));
                }
                None => {
                    violations.push(format!(
                        "Cargo.toml: [workspace.lints.clippy] missing mandatory forbid lint '{}'",
                        key
                    ));
                }
            }
        }

        let crates_dir = repo_root.join("crates");
        if crates_dir.is_dir() {
            let mut member_manifests = Vec::new();
            match fs::read_dir(&crates_dir) {
                Ok(entries) => {
                    for entry_res in entries {
                        match entry_res {
                            Ok(entry) => {
                                let p = entry.path().join("Cargo.toml");
                                if p.is_file() {
                                    member_manifests.push(p);
                                }
                            }
                            Err(err) => {
                                violations.push(format!(
                                    "crates/: Unable to read directory entry: {}",
                                    err
                                ));
                            }
                        }
                    }
                }
                Err(err) => {
                    violations.push(format!("crates/: Unable to read directory: {}", err));
                }
            }
            member_manifests.sort();

            for manifest_path in &member_manifests {
                let rel = manifest_path
                    .strip_prefix(repo_root)
                    .unwrap_or(manifest_path)
                    .display()
                    .to_string();
                let member_content = match fs::read_to_string(manifest_path) {
                    Ok(c) => c,
                    Err(err) => {
                        violations.push(format!("{}: Unable to read Cargo.toml: {}", rel, err));
                        continue;
                    }
                };

                let mut in_lints_section = false;
                let mut has_lints_workspace_true = false;

                for raw_line in member_content.lines() {
                    let without_comment = match raw_line.split_once('#') {
                        Some((before, _)) => before,
                        None => raw_line,
                    };
                    let trimmed = without_comment.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if trimmed.starts_with('[') && trimmed.ends_with(']') {
                        let sec = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
                        in_lints_section = sec == "lints";
                        continue;
                    }
                    if let Some((raw_key, raw_val)) = trimmed.split_once('=') {
                        let key = raw_key.trim();
                        let val = raw_val.trim();
                        if in_lints_section && key == "workspace" && val == "true" {
                            has_lints_workspace_true = true;
                        }
                        if key == "lints.workspace" && val == "true" {
                            has_lints_workspace_true = true;
                        }
                    }
                }

                if !has_lints_workspace_true {
                    violations.push(format!(
                        "{}: missing [lints] workspace = true inheritance",
                        rel
                    ));
                }
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}

/// Scans Rust source tokens for forbidden `#[allow(clippy::...)]` and `#![allow(clippy::...)]` attributes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and Check 34 (`REQ-0601`) static analysis integrity.
/// Detects any inner or outer attribute suppressing a `clippy::` lint across arbitrary whitespace
/// and multiline formatting while ignoring comments, string literals, raw strings, and character literals.
///
/// ## 2. Invariants & Mathematical Properties
/// - Operates as a deterministic single-pass character state machine with zero regular expressions
///   and zero panicking slice indexing.
/// - Normalizes whitespace inside `#[...]` and `#![...]` token trees before checking for the
///   dynamically assembled `allow(clippy::` substring, catching both direct `allow(clippy::...)`
///   and conditional `cfg_attr(..., allow(clippy::...))` suppressions.
///
/// ## 3. Preconditions
/// `source` is valid UTF-8 Rust source text and `file_rel` is the repository-relative path label.
///
/// ## 4. Postconditions
/// Returns a vector of `ClippySuppressionViolation` entries ordered by source occurrence line.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) where N is the number of Unicode scalar values in `source`.
/// Space Complexity: O(N + V) for character buffer and V detected violations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Never panics on malformed or unclosed comments, literals, or brackets; terminates cleanly at EOF.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-34-CLIPPY-ALLOW-TOKEN-SCAN].
pub fn scan_clippy_allow_tokens(source: &str, file_rel: &str) -> Vec<ClippySuppressionViolation> {
    let mut violations = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i: usize = 0;
    let mut line: usize = 1;

    while i < len {
        let c = match chars.get(i).copied() {
            Some(ch) => ch,
            None => break,
        };

        if c == '\n' {
            line = line.saturating_add(1);
            i = i.saturating_add(1);
            continue;
        }

        // Line comment `// ...`
        if c == '/' && chars.get(i.saturating_add(1)).copied() == Some('/') {
            i = i.saturating_add(2);
            while let Some(ch) = chars.get(i).copied() {
                if ch == '\n' {
                    break;
                }
                i = i.saturating_add(1);
            }
            continue;
        }

        // Nested block comment `/* ... */`
        if c == '/' && chars.get(i.saturating_add(1)).copied() == Some('*') {
            let mut depth: usize = 1;
            i = i.saturating_add(2);
            while depth > 0 {
                match chars.get(i).copied() {
                    None => break,
                    Some('\n') => {
                        line = line.saturating_add(1);
                        i = i.saturating_add(1);
                    }
                    Some('/') if chars.get(i.saturating_add(1)).copied() == Some('*') => {
                        depth = depth.saturating_add(1);
                        i = i.saturating_add(2);
                    }
                    Some('*') if chars.get(i.saturating_add(1)).copied() == Some('/') => {
                        depth = depth.saturating_sub(1);
                        i = i.saturating_add(2);
                    }
                    Some(_) => {
                        i = i.saturating_add(1);
                    }
                }
            }
            continue;
        }

        // Raw string literal `r"..."`, `r#"..."#`, `br"..."`, `br#"..."#`
        let prev_is_ident = if i > 0 {
            chars
                .get(i.saturating_sub(1))
                .copied()
                .map(|p| p.is_ascii_alphanumeric() || p == '_')
                .unwrap_or(false)
        } else {
            false
        };
        if !prev_is_ident {
            let raw_r_pos = if c == 'r' {
                Some(i)
            } else if c == 'b' && chars.get(i.saturating_add(1)).copied() == Some('r') {
                Some(i.saturating_add(1))
            } else {
                None
            };
            if let Some(r_pos) = raw_r_pos {
                let mut scan = r_pos.saturating_add(1);
                let mut hashes: usize = 0;
                while chars.get(scan).copied() == Some('#') {
                    hashes = hashes.saturating_add(1);
                    scan = scan.saturating_add(1);
                }
                if chars.get(scan).copied() == Some('"') {
                    i = scan.saturating_add(1);
                    while let Some(ch) = chars.get(i).copied() {
                        if ch == '\n' {
                            line = line.saturating_add(1);
                            i = i.saturating_add(1);
                        } else if ch == '"' {
                            i = i.saturating_add(1);
                            let mut matched: usize = 0;
                            while matched < hashes
                                && chars.get(i.saturating_add(matched)).copied() == Some('#')
                            {
                                matched = matched.saturating_add(1);
                            }
                            if matched == hashes {
                                i = i.saturating_add(hashes);
                                break;
                            }
                        } else {
                            i = i.saturating_add(1);
                        }
                    }
                    continue;
                }
            }
        }

        // Standard string literal `"..."`
        if c == '"' {
            i = i.saturating_add(1);
            while let Some(ch) = chars.get(i).copied() {
                if ch == '\\' {
                    if chars.get(i.saturating_add(1)).copied() == Some('\n') {
                        line = line.saturating_add(1);
                    }
                    i = i.saturating_add(2);
                } else if ch == '"' {
                    i = i.saturating_add(1);
                    break;
                } else {
                    if ch == '\n' {
                        line = line.saturating_add(1);
                    }
                    i = i.saturating_add(1);
                }
            }
            continue;
        }

        // Character literal `'x'` or `'\\.'` (distinguished from lifetimes `'a`)
        if c == '\'' {
            if chars.get(i.saturating_add(1)).copied() == Some('\\') {
                i = i.saturating_add(2);
                while let Some(ch) = chars.get(i).copied() {
                    i = i.saturating_add(1);
                    if ch == '\'' || ch == '\n' {
                        if ch == '\n' {
                            line = line.saturating_add(1);
                        }
                        break;
                    }
                }
                continue;
            } else if chars.get(i.saturating_add(2)).copied() == Some('\'')
                && chars.get(i.saturating_add(1)).copied() != Some('\n')
            {
                i = i.saturating_add(3);
                continue;
            }
            i = i.saturating_add(1);
            continue;
        }

        // Attribute `#![...]` or `#[...]`
        if c == '#' {
            let attr_line = line;
            let mut scan = i.saturating_add(1);
            let mut scan_line = line;
            while let Some(ch) = chars.get(scan).copied() {
                if ch.is_whitespace() {
                    if ch == '\n' {
                        scan_line = scan_line.saturating_add(1);
                    }
                    scan = scan.saturating_add(1);
                } else {
                    break;
                }
            }
            let is_inner = if chars.get(scan).copied() == Some('!') {
                scan = scan.saturating_add(1);
                true
            } else {
                false
            };
            while let Some(ch) = chars.get(scan).copied() {
                if ch.is_whitespace() {
                    if ch == '\n' {
                        scan_line = scan_line.saturating_add(1);
                    }
                    scan = scan.saturating_add(1);
                } else {
                    break;
                }
            }
            if chars.get(scan).copied() == Some('[') {
                scan = scan.saturating_add(1);
                let mut normalized = String::from(if is_inner { "#![" } else { "#[" });
                let mut depth: usize = 1;
                while depth > 0 {
                    match chars.get(scan).copied() {
                        None => break,
                        Some('\n') => {
                            scan_line = scan_line.saturating_add(1);
                            scan = scan.saturating_add(1);
                        }
                        Some('[') => {
                            depth = depth.saturating_add(1);
                            normalized.push('[');
                            scan = scan.saturating_add(1);
                        }
                        Some(']') => {
                            depth = depth.saturating_sub(1);
                            normalized.push(']');
                            scan = scan.saturating_add(1);
                        }
                        Some(ch) if ch.is_whitespace() => {
                            scan = scan.saturating_add(1);
                        }
                        Some(ch) => {
                            normalized.push(ch);
                            scan = scan.saturating_add(1);
                        }
                    }
                }
                if has_split_token(&normalized, "allow(clip", "py::") {
                    violations.push(ClippySuppressionViolation {
                        file: file_rel.to_string(),
                        line: attr_line,
                        attribute: normalized,
                    });
                }
                i = scan;
                line = scan_line;
                continue;
            }
        }

        i = i.saturating_add(1);
    }

    violations
}

/// Check documentation and backlog integrity (refs #318).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C documentation traceability controls.
/// Prevents root `HANDOFF.md` drift and stale backlog attribution in solution designs.
///
/// ## 2. Invariants & Mathematical Properties
/// 1. Root `HANDOFF.md` must not exist.
/// 2. `docs/designs/feat-289-290-293-solution.md` (if present) must cite `#318` and contain zero em dashes or stale work packages.
/// 3. Zero `MASTER_HANDOVER_CHECKPOINT*` files exist.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` on compliance, or `Err(Vec<String>)` with detected integrity violations.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(L) where L is solution file length and directory entries. Space Complexity: O(V).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on any discovered handoff artifact or attribution violation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-HANDOFF-BACKLOG-INTEGRITY].
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
    let solution_doc = repo_root
        .join("docs")
        .join("designs")
        .join("feat-289-290-293-solution.md");
    if solution_doc.is_file() {
        match fs::read_to_string(&solution_doc) {
            Ok(content) => {
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
                    && !content
                        .contains("skills/spec-orchestrator/scripts/bootstrap_tracker_labels.py")
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
            Err(err) => {
                violations.push(format!(
                    "Unable to read {}: {}",
                    solution_doc.display(),
                    err
                ));
            }
        }
    }

    // 3. Ephemeral MASTER_HANDOVER_CHECKPOINT.md must not exist at root or docs (refs #321)
    if let Err(errs) = check_no_master_handover_checkpoint(repo_root) {
        violations.extend(errs);
    }

    // 4. Check zero retired script references in documentation (refs #441)
    if let Err(errs) = check_zero_retired_scripts_in_documentation(repo_root) {
        violations.extend(errs);
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Check documentation and root artifact cleanliness (refs #321).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C artifact hygiene controls.
/// Verifies that ephemeral/spurious handover checkpoints (`MASTER_HANDOVER_CHECKPOINT.md`)
/// are strictly forbidden at the repository root and across `docs/`.
///
/// ## 2. Invariants & Mathematical Properties
/// Zero files starting with `MASTER_HANDOVER_CHECKPOINT` (case-insensitive) exist at root or inside `docs/`.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` when clean, or `Err(Vec<String>)` with sorted, deduplicated violations.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N) where N is entries in root and `docs/`. Space Complexity: O(V).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on any discovered handover checkpoint file.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-NO-MASTER-HANDOVER-CHECKPOINT].
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

    match fs::read_dir(repo_root) {
        Ok(entries) => {
            for entry_res in entries {
                let entry = match entry_res {
                    Ok(e) => e,
                    Err(err) => {
                        violations.push(format!(
                            "Unable to read directory entry in '{}': {}",
                            repo_root.display(),
                            err
                        ));
                        continue;
                    }
                };
                if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    if fname != "MASTER_HANDOVER_CHECKPOINT.md"
                        && fname
                            .to_ascii_uppercase()
                            .starts_with("MASTER_HANDOVER_CHECKPOINT")
                    {
                        violations.push(format!(
                            "Root checkpoint variant detected: '{}' is strictly forbidden (refs #321).",
                            fname
                        ));
                    }
                }
            }
        }
        Err(err) => {
            violations.push(format!(
                "Unable to read repository root directory '{}': {}",
                repo_root.display(),
                err
            ));
        }
    }

    // 2. docs/ directory check: forbidden anywhere inside docs/
    let docs_dir = repo_root.join("docs");
    if docs_dir.is_dir() {
        for entry_res in WalkDir::new(&docs_dir).into_iter().filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !EXCLUDED_DIRS.iter().any(|ex| name == ex)
            } else {
                true
            }
        }) {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    violations.push(format!(
                        "Unable to read directory entry under '{}': {}",
                        docs_dir.display(),
                        err
                    ));
                    continue;
                }
            };
            if entry.file_type().is_file() {
                let fname = entry.file_name().to_string_lossy();
                if fname.to_ascii_uppercase() == "MASTER_HANDOVER_CHECKPOINT.MD"
                    || fname
                        .to_ascii_uppercase()
                        .starts_with("MASTER_HANDOVER_CHECKPOINT")
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

/// Check documentation for references to retired Python and shell scripts (refs #441).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C documentation integrity controls.
/// Enforces Pure-Rust Toolchain Invariant by preventing documentation drift that references
/// deleted Python or shell scripts or non-existent binaries.
///
/// ## 2. Invariants & Mathematical Properties
/// Documentation markdown files under `docs/`, `skills/`, and `README.md` must contain zero
/// references to retired scripts: `create_issue.sh`, `verify_downstream_baseline.py`,
/// `install_pipeline.sh`, `generate-wbs-suite`, `compile_yang.py`, `compile_sysml.py`,
/// `assemble_conops.py`, `parity_auditor`, `setup_git_hooks.py`, `file_defect.py`,
/// `verify_commit_messages.py`, `verify_subagent_output.py`, `reconcile_backlog.py`.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` on compliance, or `Err(Vec<String>)` with sorted detected violations.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N * L) where N is file count and L is line count. Space Complexity: O(V).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on any discovered reference to retired scripts.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-ZERO-RETIRED-SCRIPTS-IN-DOCS].
pub fn check_zero_retired_scripts_in_documentation(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();

    let mut retired_needles: Vec<String> = vec![
        "create_issue.sh".to_string(),
        "verify_downstream_baseline.py".to_string(),
        "install_pipeline.sh".to_string(),
        "generate-wbs-suite".to_string(),
        "compile_yang.py".to_string(),
        "compile_sysml.py".to_string(),
        "assemble_conops.py".to_string(),
        "parity_auditor".to_string(),
        "setup_git_hooks.py".to_string(),
    ];
    retired_needles.push("file_defect.py".to_string());
    retired_needles.push("verify_commit_messages.py".to_string());
    retired_needles.push("verify_subagent_output.py".to_string());
    retired_needles.push("reconcile_backlog.py".to_string());
    retired_needles.push("spec_validator.py".to_string());

    let target_dirs = vec![repo_root.join("docs"), repo_root.join("skills")];

    for dir in &target_dirs {
        if !dir.is_dir() {
            continue;
        }
        for entry_res in WalkDir::new(dir).into_iter().filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !EXCLUDED_DIRS.iter().any(|ex| name == ex)
                    && name != "designs"
                    && name != "audits"
                    && name != "blueprints"
                    && name != "decisions"
            } else {
                true
            }
        }) {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    violations.push(format!(
                        "Unable to read directory entry under '{}': {}",
                        dir.display(),
                        err
                    ));
                    continue;
                }
            };
            if entry.file_type().is_file() {
                let path = entry.path();
                let is_md = path.extension().map(|ext| ext == "md").unwrap_or(false);
                if is_md {
                    let rel = path
                        .strip_prefix(repo_root)
                        .unwrap_or(path)
                        .display()
                        .to_string();
                    if rel.ends_with("_HANDOFF.md") || rel.ends_with("_HANDOVER.md") {
                        continue;
                    }
                    match fs::read_to_string(path) {
                        Ok(content) => {
                            for (idx, line) in content.lines().enumerate() {
                                for needle in &retired_needles {
                                    if line.contains(needle) {
                                        violations.push(format!(
                                            "{}:{}: references retired script '{}'",
                                            rel,
                                            idx.saturating_add(1),
                                            needle
                                        ));
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            violations.push(format!(
                                "{}: unable to read documentation file: {}",
                                rel, err
                            ));
                        }
                    }
                }
            }
        }
    }

    let target_single_files = vec![
        repo_root.join("README.md"),
        repo_root.join(".pipeline/constitution.md"),
        repo_root.join(".pipeline/upstream/pipeline-tooling.md"),
    ];

    for file in &target_single_files {
        if file.is_file() {
            let rel = file
                .strip_prefix(repo_root)
                .unwrap_or(file)
                .display()
                .to_string();
            match fs::read_to_string(file) {
                Ok(content) => {
                    for (idx, line) in content.lines().enumerate() {
                        for needle in &retired_needles {
                            if line.contains(needle) {
                                violations.push(format!(
                                    "{}:{}: references retired script '{}'",
                                    rel,
                                    idx.saturating_add(1),
                                    needle
                                ));
                            }
                        }
                    }
                }
                Err(err) => {
                    violations.push(format!("{}: unable to read file: {}", rel, err));
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

/// Verify FEAT-42 design blueprint compliance (refs #459).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C architectural integrity controls.
/// Enforces single-source-of-truth alignment between `FEAT-42` design blueprint,
/// the workspace AST crate architecture (`deap-core`), the `ingest-sysml` CLI contract,
/// and Check 34 `seen_docblocks` docstring uniqueness invariants.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero occurrences of phantom crate `sysml-core` across `docs/designs/FEAT-42_*.md`.
/// - Section 2.1 specifies `ingest-sysml [OPTIONS] [--source <DIR> | --schema <PATH>]`,
///   preserves optional `--source`, documents `--schema`, and sets `--format` default `auto`.
/// - Section 3.2 mandates Check 34 `seen_docblocks` uniqueness via per-function AST
///   interpolation across Sections 1 through 6.
///
/// ## 3. Preconditions
/// `repo_root` must be an accessible directory path.
///
/// ## 4. Postconditions
/// Returns `Ok(())` on full compliance, or `Err(Vec<String>)` enumerating contract violations.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(L) where L is the byte length of the design document. Space Complexity: O(V).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed if the blueprint is missing, unreadable, or violates any architectural contract.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-34-FEAT42-BLUEPRINT-COMPLIANCE].
pub fn check_feat42_design_blueprint_compliance(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    let feat42_path = repo_root
        .join("docs/designs/FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md");
    if !feat42_path.is_file() {
        violations.push(format!(
            "Missing mandatory design blueprint: {}",
            feat42_path.display()
        ));
        return Err(violations);
    }

    let content = match fs::read_to_string(&feat42_path) {
        Ok(c) => c,
        Err(e) => {
            violations.push(format!(
                "Failed to read design blueprint {}: {e}",
                feat42_path.display()
            ));
            return Err(violations);
        }
    };

    // 1. Assert zero occurrences of phantom crate 'sysml-core'
    if content.contains("sysml-core") {
        violations.push(
            "FEAT-42 contains forbidden reference to non-existent 'sysml-core' crate (must reference 'deap-core')"
                .to_string(),
        );
    }

    // 2. Assert Section 2.1 specifies CLI syntax and backwards compatibility
    if !content.contains("ingest-sysml [OPTIONS] [--source <DIR> | --schema <PATH>]") {
        violations.push(
            "FEAT-42 Section 2.1 must specify CLI syntax: ingest-sysml [OPTIONS] [--source <DIR> | --schema <PATH>]"
                .to_string(),
        );
    }
    let has_optional_source = content.contains("| `--source`")
        && (content.contains("| No (mutually exclusive with `--schema`)")
            || content.contains("| Path (`<DIR>`) | No"));
    if !has_optional_source {
        violations.push(
            "FEAT-42 Section 2.1 must declare --source as optional (No), preserving backward compatibility"
                .to_string(),
        );
    }
    if !content.contains("| `--schema`") {
        violations.push(
            "FEAT-42 Section 2.1 must specify --schema in CLI option definitions table".to_string(),
        );
    }
    let has_format_auto = content
        .lines()
        .any(|l| l.contains("--format") && l.contains("`auto`"));
    if !has_format_auto {
        violations
            .push("FEAT-42 Section 2.1 must specify --format default value as `auto`".to_string());
    }

    // 3. Assert Section 3.2 mandates per-function AST interpolation across Sections 1-6 for Check 34 uniqueness
    if !content.contains("seen_docblocks") {
        violations.push(
            "FEAT-42 Section 3.2 must mandate docstring uniqueness under 'seen_docblocks'"
                .to_string(),
        );
    }
    if !content.contains("Check 34") {
        violations.push("FEAT-42 Section 3.2 must cite Check 34 uniqueness contract".to_string());
    }
    let has_interpolation = (content.contains("interpolate") || content.contains("Interpolate"))
        && content.contains("Section 1")
        && content.contains("Section 2")
        && content.contains("Section 3")
        && content.contains("Section 4")
        && content.contains("Section 5")
        && content.contains("Section 6");
    if !has_interpolation {
        violations.push(
            "FEAT-42 Section 3.2 must mandate per-function AST signature interpolation across Sections 1 through 6"
                .to_string(),
        );
    }

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        violations.dedup();
        Err(violations)
    }
}

/// Check 37: Verify architecture blueprint grounding in workspace crates and on-disk paths.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C architectural
/// traceability and configuration management controls. Enforces that all 13 canonical architecture
/// blueprints under `docs/architecture/blueprints/` exist on disk and reference only actual
/// workspace crates, existing `crates/...` file paths, native Rust tools, and ASCII punctuation.
///
/// ## 2. Invariants & Mathematical Properties
/// - Canonical Completeness: Every variant in `CanonicalArchitectureBlueprint::ALL` resolves to an
///   existing regular file under `docs/architecture/blueprints/`.
/// - Zero Em Dash Invariant: Zero lines in `docs/architecture/blueprints/*.md` contain `\u2014`.
/// - Zero Fictional Crate Names: Zero blueprint lines reference `deap-ast`, `deap-codegen`,
///   `deap-harness`, `deap-cli`, or their underscore variants outside explicit negative rule text.
/// - Zero Retired Tool References: Zero blueprint lines reference `verify_downstream`,
///   `sysmlv2_ingest`, `reconcile_backlog.py`, `parity_auditor`, or `setup_git_hooks.sh`.
/// - On-Disk Path Grounding: Every non-placeholder `crates/...` path token resolves to one of the
///   6 workspace crates (`deap-core`, `verify-baseline`, `compile-sysml`, `ingest-sysml`,
///   `assemble-conops`, `reconcile-backlog`) and exists on disk.
///
/// ## 3. Preconditions
/// - `repo_root: &Path` references an accessible workspace directory on the local filesystem.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` when `docs/architecture/blueprints/` is absent in a downstream workspace or
///   when all 13 blueprints exist and satisfy all grounding invariants.
/// - Returns `Err(Vec<String>)` with sorted, deduplicated `file:line` violations upon any failure.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(B * L) where B is the number of blueprint markdown files and L is line count.
/// - Space Complexity: O(V) where V is the number of detected grounding violations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Missing blueprint files or unreadable markdown files fail closed with explicit errors.
/// - State: Stateless verification pass with deterministic sorted diagnostic output.
/// - Action: Non-panicking slice inspection using `.get(..)` exclusively.
/// - Resource: Bounded memory proportional to single-file content and violation vector size.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-37-BLUEPRINT-GROUNDING]
pub fn check_blueprint_grounding(repo_root: &Path) -> Result<(), Vec<String>> {
    let blueprints_dir = repo_root
        .join("docs")
        .join("architecture")
        .join("blueprints");
    let designs_dir = repo_root.join("docs").join("designs");
    if !blueprints_dir.is_dir() && !designs_dir.is_dir() && !is_upstream_compiler(repo_root) {
        return Ok(());
    }

    let mut violations = Vec::new();
    if is_upstream_compiler(repo_root) && !blueprints_dir.is_dir() {
        violations.push(
            "Missing docs/architecture/blueprints directory in upstream specification compiler repository."
                .to_string(),
        );
        return Err(violations);
    }

    let mut md_files = Vec::new();

    if blueprints_dir.is_dir() {
        if is_upstream_compiler(repo_root) {
            for bp in CanonicalArchitectureBlueprint::ALL {
                let bp_path = blueprints_dir.join(bp.as_str());
                if !bp_path.is_file() {
                    violations.push(format!(
                        "Missing canonical architecture blueprint: docs/architecture/blueprints/{}",
                        bp.as_str()
                    ));
                }
            }
        }

        match fs::read_dir(&blueprints_dir) {
            Ok(entries) => {
                for entry_res in entries {
                    match entry_res {
                        Ok(entry) => {
                            let path = entry.path();
                            if path.is_file()
                                && path.extension().map(|ext| ext == "md").unwrap_or(false)
                            {
                                md_files.push(path);
                            }
                        }
                        Err(err) => {
                            violations.push(format!(
                                "docs/architecture/blueprints: directory entry read error: {}",
                                err
                            ));
                        }
                    }
                }
            }
            Err(err) => {
                violations.push(format!(
                    "docs/architecture/blueprints: unable to read directory: {}",
                    err
                ));
            }
        }
    }

    if designs_dir.is_dir() {
        match fs::read_dir(&designs_dir) {
            Ok(entries) => {
                for entry_res in entries {
                    match entry_res {
                        Ok(entry) => {
                            let path = entry.path();
                            let is_blueprint_md = path.is_file()
                                && path.extension().map(|ext| ext == "md").unwrap_or(false)
                                && path
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .map(|name| name.starts_with("blueprint-"))
                                    .unwrap_or(false);
                            if is_blueprint_md {
                                md_files.push(path);
                            }
                        }
                        Err(err) => {
                            violations
                                .push(format!("docs/designs: directory entry read error: {}", err));
                        }
                    }
                }
            }
            Err(err) => {
                violations.push(format!("docs/designs: unable to read directory: {}", err));
            }
        }
    }

    md_files.sort();
    for md_path in &md_files {
        scan_blueprint_markdown_file(repo_root, md_path, &mut violations);
    }

    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        violations.dedup();
        Err(violations)
    }
}

/// Scans a single architecture blueprint Markdown file for Check 37 grounding violations.
/// Realises: [REQ-CHECK-37-SCAN-BLUEPRINT-FILE]
fn scan_blueprint_markdown_file(repo_root: &Path, md_path: &Path, violations: &mut Vec<String>) {
    let rel = md_path
        .strip_prefix(repo_root)
        .unwrap_or(md_path)
        .display()
        .to_string();

    let content = match fs::read_to_string(md_path) {
        Ok(c) => c,
        Err(err) => {
            violations.push(format!("{}: unable to read blueprint file: {}", rel, err));
            return;
        }
    };

    let is_design_blueprint = rel.starts_with("docs/designs/");
    let allow_proposed_design_paths =
        is_design_blueprint && !is_approved_design_blueprint(&content);

    for (idx, line) in content.lines().enumerate() {
        let line_no = idx.saturating_add(1);

        if !is_design_blueprint && line.contains('\u{2014}') {
            violations.push(format!(
                "{}:{}: forbidden Unicode em dash (\\u2014) in architecture blueprint",
                rel, line_no
            ));
        }

        check_blueprint_line_fictional_crates(&rel, line_no, line, violations);
        check_blueprint_line_retired_tools(&rel, line_no, line, violations);
        check_blueprint_line_crates_paths(
            repo_root,
            &rel,
            line_no,
            line,
            allow_proposed_design_paths,
            violations,
        );
    }
}

/// Determines whether a design blueprint Markdown payload declares an `Approved` lifecycle status.
/// Realises: [REQ-CHECK-37-APPROVED-BLUEPRINT-STATUS]
fn is_approved_design_blueprint(content: &str) -> bool {
    let mut in_fence = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if trimmed.contains("**Status**") && trimmed.contains('|') {
            let mut seen_status_key = false;
            for cell in trimmed.split('|') {
                let clean = cell.trim().trim_matches(|c: char| c == '`' || c == '*');
                if seen_status_key && !clean.is_empty() {
                    if clean.eq_ignore_ascii_case("approved") {
                        return true;
                    }
                    break;
                }
                if clean.eq_ignore_ascii_case("status") {
                    seen_status_key = true;
                }
            }
        } else if let Some((_, rhs)) = trimmed
            .split_once("**Status**:")
            .or_else(|| trimmed.split_once("Status:"))
        {
            let clean = rhs
                .trim()
                .trim_matches(|c: char| c == '|' || c == '`' || c == '*' || c.is_whitespace());
            if clean.eq_ignore_ascii_case("approved") {
                return true;
            }
        }
    }
    false
}

/// Checks whether `line` contains `token` bounded on both sides by non-identifier characters.
/// Realises: [REQ-CHECK-37-BOUNDED-TOKEN-MATCH]
fn contains_bounded_token(line: &str, token: &str) -> bool {
    let bytes = line.as_bytes();
    let token_len = token.len();
    for (idx, _) in line.match_indices(token) {
        let before_ok = if idx == 0 {
            true
        } else {
            match bytes.get(idx.saturating_sub(1)) {
                Some(&b) => !b.is_ascii_alphanumeric() && b != b'_' && b != b'-',
                None => true,
            }
        };
        let after_idx = idx.saturating_add(token_len);
        let after_ok = match bytes.get(after_idx) {
            Some(&b) => !b.is_ascii_alphanumeric() && b != b'_' && b != b'-',
            None => true,
        };
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// Checks whether a blueprint line states an explicit negative prohibition or retirement rule.
/// Realises: [REQ-CHECK-37-NEGATIVE-RULE-CONTEXT]
fn is_negative_blueprint_rule_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("forbidden")
        || lower.contains("prohibited")
        || lower.contains("retired")
        || lower.contains("nonexistent")
        || lower.contains("non-existent")
        || lower.contains("fictional")
}

/// Scans a blueprint line for references to fictional, non-existent DEAP crate names.
/// Realises: [REQ-CHECK-37-DETECT-FICTIONAL-CRATE]
fn check_blueprint_line_fictional_crates(
    rel_file: &str,
    line_no: usize,
    line: &str,
    violations: &mut Vec<String>,
) {
    if is_negative_blueprint_rule_line(line) {
        return;
    }
    if contains_bounded_token(line, "deap-ast") {
        violations.push(format!(
            "{}:{}: references fictional crate 'deap-ast'",
            rel_file, line_no
        ));
    }
    if contains_bounded_token(line, "deap-codegen") {
        violations.push(format!(
            "{}:{}: references fictional crate 'deap-codegen'",
            rel_file, line_no
        ));
    }
    if contains_bounded_token(line, "deap-harness") {
        violations.push(format!(
            "{}:{}: references fictional crate 'deap-harness'",
            rel_file, line_no
        ));
    }
    if contains_bounded_token(line, "deap-cli") {
        violations.push(format!(
            "{}:{}: references fictional crate 'deap-cli'",
            rel_file, line_no
        ));
    }
    if contains_bounded_token(line, "deap_ast") {
        violations.push(format!(
            "{}:{}: references fictional crate 'deap_ast'",
            rel_file, line_no
        ));
    }
    if contains_bounded_token(line, "deap_codegen") {
        violations.push(format!(
            "{}:{}: references fictional crate 'deap_codegen'",
            rel_file, line_no
        ));
    }
    if contains_bounded_token(line, "deap_harness") {
        violations.push(format!(
            "{}:{}: references fictional crate 'deap_harness'",
            rel_file, line_no
        ));
    }
    if contains_bounded_token(line, "deap_cli") {
        violations.push(format!(
            "{}:{}: references fictional crate 'deap_cli'",
            rel_file, line_no
        ));
    }
}

/// Scans a blueprint line for references to retired Python or shell tools outside negative rules.
/// Realises: [REQ-CHECK-37-DETECT-RETIRED-TOOL]
fn check_blueprint_line_retired_tools(
    rel_file: &str,
    line_no: usize,
    line: &str,
    violations: &mut Vec<String>,
) {
    if is_negative_blueprint_rule_line(line) {
        return;
    }
    if line.contains("verify_downstream") {
        violations.push(format!(
            "{}:{}: references retired tool 'verify_downstream'",
            rel_file, line_no
        ));
    }
    if line.contains("sysmlv2_ingest") {
        violations.push(format!(
            "{}:{}: references retired tool 'sysmlv2_ingest'",
            rel_file, line_no
        ));
    }
    if line.contains("reconcile_backlog.py") {
        violations.push(format!(
            "{}:{}: references retired tool 'reconcile_backlog.py'",
            rel_file, line_no
        ));
    }
    if line.contains("parity_auditor") {
        violations.push(format!(
            "{}:{}: references retired tool 'parity_auditor'",
            rel_file, line_no
        ));
    }
    if line.contains("setup_git_hooks.sh") {
        violations.push(format!(
            "{}:{}: references retired tool 'setup_git_hooks.sh'",
            rel_file, line_no
        ));
    }
}

/// Checks whether `crate_name` matches one of the 6 canonical workspace crates in `Cargo.toml`.
/// Realises: [REQ-CHECK-37-VALID-WORKSPACE-CRATE]
fn is_valid_workspace_crate_name(crate_name: &str) -> bool {
    matches!(
        crate_name,
        "deap-core"
            | "verify-baseline"
            | "compile-sysml"
            | "ingest-sysml"
            | "assemble-conops"
            | "reconcile-backlog"
    )
}

/// Extracts and validates concrete `crates/...` path tokens on a blueprint line against disk.
/// Realises: [REQ-CHECK-37-LINE-CRATES-PATHS]
fn check_blueprint_line_crates_paths(
    repo_root: &Path,
    rel_file: &str,
    line_no: usize,
    line: &str,
    allow_proposed_design_paths: bool,
    violations: &mut Vec<String>,
) {
    let bytes = line.as_bytes();
    for (start_idx, _) in line.match_indices("crates/") {
        if start_idx > 0 {
            if let Some(&prev) = bytes.get(start_idx.saturating_sub(1)) {
                if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'-' {
                    continue;
                }
            }
        }

        let tail = match line.get(start_idx..) {
            Some(t) => t,
            None => continue,
        };

        let end_offset = tail
            .find(|c: char| {
                c.is_whitespace()
                    || matches!(
                        c,
                        '`' | '\'' | '"' | '(' | ')' | '[' | ']' | ';' | ',' | '|'
                    )
            })
            .unwrap_or(tail.len());

        let raw_token = match tail.get(..end_offset) {
            Some(tok) => tok,
            None => continue,
        };

        if raw_token.contains('*')
            || raw_token.contains('<')
            || raw_token.contains('>')
            || raw_token.contains('{')
            || raw_token.contains('}')
            || raw_token.contains("...")
            || raw_token.contains('$')
        {
            continue;
        }

        let without_hash = match raw_token.split_once('#') {
            Some((before, _)) => before,
            None => raw_token,
        };
        let without_colon = match without_hash.split_once(':') {
            Some((before, _)) => before,
            None => without_hash,
        };
        let cleaned = without_colon.trim_end_matches(['.', ',', ':', ';', '/']);

        if cleaned.is_empty() || cleaned == "crates" {
            continue;
        }

        let after_crates = match cleaned.strip_prefix("crates/") {
            Some(rest) => rest,
            None => continue,
        };
        let crate_name = after_crates.split('/').next().unwrap_or("");
        if crate_name.is_empty() {
            continue;
        }

        let is_downstream_crate =
            !is_upstream_compiler(repo_root) && repo_root.join("crates").join(crate_name).is_dir();
        if !is_valid_workspace_crate_name(crate_name) && !is_downstream_crate {
            violations.push(format!(
                "{}:{}: ungrounded crate '{}' in blueprint path '{}' (must be one of deap-core, verify-baseline, compile-sysml, ingest-sysml, assemble-conops, reconcile-backlog)",
                rel_file, line_no, crate_name, cleaned
            ));
            continue;
        }

        let is_design_proposed_path =
            allow_proposed_design_paths && repo_root.join("crates").join(crate_name).is_dir();
        if !is_design_proposed_path && !repo_root.join(cleaned).exists() {
            violations.push(format!(
                "{}:{}: non-existent workspace path '{}' referenced in architecture blueprint",
                rel_file, line_no, cleaned
            ));
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
        fn new(name: &str) -> Result<Self, std::io::Error> {
            static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let path = std::env::temp_dir()
                .join(format!("verify_structural_{}_{}_{}", name, nanos, count));
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
    fn test_check_18_downstream_exempt() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c18_downstream")?;
        assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_ok());
        Ok(())
    }

    #[test]
    fn test_check_18_upstream_clean() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c18_clean")?;
        fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
        let schema_dir = tmp.path.join("schema");
        fs::create_dir_all(&schema_dir)?;
        File::create(schema_dir.join(".gitkeep"))?;

        let bp_dir = tmp.path.join("docs/architecture/blueprints");
        fs::create_dir_all(&bp_dir)?;
        File::create(bp_dir.join("CORE_ARCHITECTURE.md"))?;
        // Domain-agnostic markdown blueprint that formerly failed fragile blacklist must now pass
        File::create(bp_dir.join("FRONTEND_SYSTEMS_PATTERN.md"))?;

        assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_ok());
        Ok(())
    }

    #[test]
    fn test_check_18_upstream_violations() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c18_upstream")?;
        fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
        let bp_dir = tmp.path.join("docs/architecture/blueprints");
        fs::create_dir_all(&bp_dir)?;

        let clean_file = bp_dir.join("CORE_ARCHITECTURE.md");
        File::create(&clean_file)?;
        assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_ok());

        let dirty_file = bp_dir.join("FLIGHT_SYSTEMS_MODEL.sysml");
        File::create(&dirty_file)?;
        assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_err());
        Ok(())
    }

    /// Regression test for Check 18 scanning `docs/designs/` for ungrounded `.sysml` files (refs #530).
    #[test]
    fn test_check_18_upstream_designs_sysml_violations() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c18_designs_sysml")?;
        fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
        let designs_dir = tmp.path.join("docs/designs");
        fs::create_dir_all(&designs_dir)?;

        // 1. Complete, non-empty blueprint triplet -> Ok(())
        fs::write(
            designs_dir.join("blueprint-01-valid.md"),
            "# Valid Blueprint\n",
        )?;
        fs::write(
            designs_dir.join("blueprint-01-valid.sysml"),
            "package Valid {}\n",
        )?;
        fs::write(designs_dir.join("blueprint-01-valid.digest.json"), "{}\n")?;
        assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_ok());

        // 2. Ungrounded domain.sysml -> Err containing "Upstream blueprint contains SysML model: docs/designs/domain.sysml"
        let domain_sysml = designs_dir.join("domain.sysml");
        fs::write(&domain_sysml, "package ForbiddenDomain {}\n")?;
        let res = verify_upstream_blueprint_domain_cleanliness(&tmp.path);
        assert!(res.is_err());
        let errs = res.err().ok_or("expected Err")?;
        assert!(
            errs.iter().any(|e| e
                .contains("Upstream blueprint contains SysML model: docs/designs/domain.sysml")),
            "Expected docs/designs/domain.sysml violation, got: {:?}",
            errs
        );
        fs::remove_file(&domain_sysml)?;

        // 3. Incomplete triplet (orphan .sysml without .md and .digest.json) -> Err
        let orphan_sysml = designs_dir.join("blueprint-99-orphan.sysml");
        fs::write(&orphan_sysml, "package Orphan {}\n")?;
        let res_orphan = verify_upstream_blueprint_domain_cleanliness(&tmp.path);
        assert!(res_orphan.is_err());
        let errs_orphan = res_orphan.err().ok_or("expected Err")?;
        assert!(
            errs_orphan.iter().any(|e| {
                e.contains("Upstream blueprint contains SysML model: docs/designs/blueprint-99-orphan.sysml")
            }),
            "Expected docs/designs/blueprint-99-orphan.sysml violation, got: {:?}",
            errs_orphan
        );
        Ok(())
    }

    /// Regression test for Check 18 scanning `docs/architecture/blueprints/` and `docs/designs/` for ungrounded `.digest.json` sidecars (refs #531).
    #[test]
    fn test_check_18_upstream_digest_json_sidecar_violations(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c18_digest_json")?;
        fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
        let bp_dir = tmp.path.join("docs/architecture/blueprints");
        let designs_dir = tmp.path.join("docs/designs");
        fs::create_dir_all(&bp_dir)?;
        fs::create_dir_all(&designs_dir)?;

        // 1. .digest.json inside docs/architecture/blueprints/ -> Err
        let bp_digest = bp_dir.join("blueprint-01.digest.json");
        fs::write(&bp_digest, "{}\n")?;
        let res_bp = verify_upstream_blueprint_domain_cleanliness(&tmp.path);
        assert!(res_bp.is_err());
        let errs_bp = res_bp.err().ok_or("expected Err")?;
        assert!(
            errs_bp.iter().any(|e| {
                e.contains("Upstream blueprint contains SysML model: docs/architecture/blueprints/blueprint-01.digest.json")
            }),
            "Expected docs/architecture/blueprints/blueprint-01.digest.json violation, got: {:?}",
            errs_bp
        );
        fs::remove_file(&bp_digest)?;

        // 2. Ungrounded domain.digest.json in docs/designs/ -> Err
        let domain_digest = designs_dir.join("domain.digest.json");
        fs::write(&domain_digest, "{}\n")?;
        let res_domain = verify_upstream_blueprint_domain_cleanliness(&tmp.path);
        assert!(res_domain.is_err());
        let errs_domain = res_domain.err().ok_or("expected Err")?;
        assert!(
            errs_domain.iter().any(|e| {
                e.contains(
                    "Upstream blueprint contains SysML model: docs/designs/domain.digest.json",
                )
            }),
            "Expected docs/designs/domain.digest.json violation, got: {:?}",
            errs_domain
        );
        fs::remove_file(&domain_digest)?;

        // 3. Orphan blueprint-99-orphan.digest.json without .md and .sysml companions -> Err
        let orphan_digest = designs_dir.join("blueprint-99-orphan.digest.json");
        fs::write(&orphan_digest, "{}\n")?;
        let res_orphan = verify_upstream_blueprint_domain_cleanliness(&tmp.path);
        assert!(res_orphan.is_err());
        let errs_orphan = res_orphan.err().ok_or("expected Err")?;
        assert!(
            errs_orphan.iter().any(|e| {
                e.contains("Upstream blueprint contains SysML model: docs/designs/blueprint-99-orphan.digest.json")
            }),
            "Expected docs/designs/blueprint-99-orphan.digest.json violation, got: {:?}",
            errs_orphan
        );
        Ok(())
    }

    #[test]
    fn test_check_18_landing_zone_non_gitkeep_fails() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c18_lz")?;
        fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
        let schema_dir = tmp.path.join("schema");
        fs::create_dir_all(&schema_dir)?;
        File::create(schema_dir.join("domain_model.sysml"))?;
        assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_err());
        Ok(())
    }

    #[test]
    fn test_check_19_downstream_exempt() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c19_downstream")?;
        assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_ok());
        Ok(())
    }

    #[test]
    fn test_check_19_upstream_clean() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c19_clean")?;
        fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
        let src_dir = tmp.path.join("crates/test-crate/src");
        fs::create_dir_all(&src_dir)?;
        let mut f = File::create(src_dir.join("lib.rs"))?;
        writeln!(f, "pub fn compute_hash(val: &[u8]) -> u64 {{ 42 }}")?;
        assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_ok());
        Ok(())
    }

    #[test]
    fn test_check_19_rust_crates_domain_params_fails() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c19_rust")?;
        fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
        let src_dir = tmp.path.join("crates/test-crate/src");
        fs::create_dir_all(&src_dir)?;
        let mut f = File::create(src_dir.join("lib.rs"))?;
        let kw = join_pair("DOM", "AIN_PARAMS");
        writeln!(f, "pub const {}: &[&str] = &[\"speed\"];", kw)?;
        assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_err());
        Ok(())
    }

    #[test]
    fn test_reproduce_defect_265_unflagged_m1_domain_dict_bypasses_check_19(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c19_m1_bypass")?;
        fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
        let src_dir = tmp.path.join("crates/test-crate/src");
        fs::create_dir_all(&src_dir)?;
        let mut f = File::create(src_dir.join("lib.rs"))?;
        writeln!(
            f,
            "pub const FLIGHT_ENVELOPE_MAP: &[(&str, f64)] = &[(\"max_altitude\", 10000.0)];"
        )?;
        // Under unfixed Check 19, this fails because check_domain_agnostic_ast_cleanliness incorrectly returns Ok(())
        assert!(
            check_domain_agnostic_ast_cleanliness(&tmp.path).is_err(),
            "Check 19 failed to reject unflagged M1 domain dict FLIGHT_ENVELOPE_MAP"
        );
        Ok(())
    }

    #[test]
    fn test_check_19_legacy_python_parity_auditor_dir_fails(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c19_legacy")?;
        fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
        let crates_dir = tmp.path.join("crates");
        fs::create_dir_all(&crates_dir)?;
        let legacy_dir = tmp.path.join("skills/spec-orchestrator/parity_auditor");
        fs::create_dir_all(&legacy_dir)?;

        assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_err());
        Ok(())
    }

    #[test]
    fn test_check_32_zero_shell_scripts_exempts_build_dirs_and_rejects_others(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c32_exempt_dirs")?;
        for dir in [
            "build/macos",
            "macos/Pods",
            "macos/Flutter/ephemeral",
            ".dart_tool/build",
        ] {
            let d = tmp.path.join(dir);
            fs::create_dir_all(&d)?;
            File::create(d.join("generated.sh"))?;
        }
        assert!(
            check_zero_shell_scripts(&tmp.path).is_ok(),
            "Generated .sh files inside build, Pods, ephemeral, and .dart_tool must be exempted"
        );

        let forbidden_sh = tmp.path.join("scripts").join("deploy.sh");
        let parent_dir = forbidden_sh.parent().ok_or("missing parent")?;
        fs::create_dir_all(parent_dir)?;
        File::create(&forbidden_sh)?;
        let res = check_zero_shell_scripts(&tmp.path);
        assert!(res.is_err(), "Non-exempt .sh script must fail Check 32");
        let errs = res.err().ok_or("expected Err")?;
        assert_eq!(errs, vec!["scripts/deploy.sh".to_string()]);
        Ok(())
    }

    #[test]
    fn test_check_33_zero_python_clean() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c33_clean")?;
        let rs_file = tmp.path.join("main.rs");
        File::create(&rs_file)?;
        assert!(check_zero_python_scripts(&tmp.path).is_ok());
        Ok(())
    }

    #[test]
    fn test_check_33_zero_python_exempts_build_dirs_and_rejects_others(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c33_exempt_dirs")?;
        for dir in [
            "build/macos",
            "macos/Pods",
            "macos/Flutter/ephemeral",
            ".dart_tool/build",
        ] {
            let d = tmp.path.join(dir);
            fs::create_dir_all(&d)?;
            File::create(d.join("helper.py"))?;
        }
        assert!(
            check_zero_python_scripts(&tmp.path).is_ok(),
            "Generated .py files inside build, Pods, ephemeral, and .dart_tool must be exempted"
        );

        let forbidden_py = tmp.path.join("scripts").join("legacy.py");
        let parent_dir = forbidden_py.parent().ok_or("missing parent")?;
        fs::create_dir_all(parent_dir)?;
        File::create(&forbidden_py)?;
        let res = check_zero_python_scripts(&tmp.path);
        assert!(res.is_err(), "Non-exempt .py script must fail Check 33");
        let errs = res.err().ok_or("expected Err")?;
        assert_eq!(errs, vec!["scripts/legacy.py".to_string()]);
        Ok(())
    }

    #[test]
    fn test_check_33_zero_python_violation() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c33_violation")?;
        let py_file = tmp.path.join("script.py");
        File::create(&py_file)?;
        let res = check_zero_python_scripts(&tmp.path);
        assert!(res.is_err());
        let errs = res.err().ok_or("expected Err")?;
        assert_eq!(errs.len(), 1);
        assert_eq!(errs.first().map(String::as_str), Some("script.py"));
        Ok(())
    }

    #[test]
    fn test_check_34_zero_regex_clean() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c34_clean")?;
        let crates_dir = tmp.path.join("crates").join("sample").join("src");
        fs::create_dir_all(&crates_dir)?;
        let rs_file = crates_dir.join("lib.rs");
        let mut f = File::create(&rs_file)?;
        writeln!(f, "pub fn add(a: i32, b: i32) -> i32 {{ a + b }}")?;

        let cargo_file = tmp.path.join("crates").join("sample").join("Cargo.toml");
        let mut fc = File::create(&cargo_file)?;
        writeln!(fc, "[package]\nname = \"sample\"\nversion = \"0.1.0\"")?;

        assert!(check_zero_regex(&tmp.path).is_ok());
        Ok(())
    }

    #[test]
    fn test_check_34_zero_regex_violation_rs() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c34_viol_rs")?;
        let crates_dir = tmp.path.join("crates").join("sample").join("src");
        fs::create_dir_all(&crates_dir)?;
        let rs_file = crates_dir.join("lib.rs");
        let mut f = File::create(&rs_file)?;
        // Construct forbidden pattern dynamically to avoid literal in this file
        let pat = join_pair("Reg", "ex::");
        writeln!(f, "let _ = {}new(\"\");", pat)?;

        let res = check_zero_regex(&tmp.path);
        assert!(res.is_err());
        let errs = res.err().ok_or("expected Err")?;
        assert_eq!(errs.len(), 1);
        assert!(errs
            .first()
            .is_some_and(|e| e.contains("crates/sample/src/lib.rs:1:")));
        Ok(())
    }

    #[test]
    fn test_check_34_zero_regex_violation_cargo() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c34_viol_cargo")?;
        let crates_dir = tmp.path.join("crates").join("sample");
        fs::create_dir_all(&crates_dir)?;
        let cargo_file = crates_dir.join("Cargo.toml");
        let mut fc = File::create(&cargo_file)?;
        // Construct forbidden dependency dynamically
        let dep = join_pair("reg", "ex");
        writeln!(fc, "[dependencies]\n{} = \"1.10\"", dep)?;

        let res = check_zero_regex(&tmp.path);
        assert!(res.is_err());
        let errs = res.err().ok_or("expected Err")?;
        assert_eq!(errs.len(), 1);
        assert!(errs
            .first()
            .is_some_and(|e| e.contains("crates/sample/Cargo.toml:2:")));
        Ok(())
    }

    #[test]
    fn test_check_34_repo_root_passes() -> Result<(), Box<dyn std::error::Error>> {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("repo root")?;
        let res = check_zero_regex(repo_root);
        assert!(
            res.is_ok(),
            "Check 34 Zero Regex Invariant failed on repository root: {:?}",
            res.err()
        );
        Ok(())
    }

    #[test]
    fn test_check_34_audited_parity_gates_passes() -> Result<(), Box<dyn std::error::Error>> {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("repo root")?;
        let res = check_compiler_ast_purity(repo_root);
        assert!(
            res.is_ok(),
            "Check 34 Compiler AST Purity failed on audited modules: {:?}",
            res.err()
        );

        let structural_path = repo_root
            .join("crates")
            .join("verify-baseline")
            .join("src")
            .join("checks")
            .join("structural.rs");
        let structural_src = fs::read_to_string(&structural_path)?;
        let res_structural = audit_source_purity(
            &structural_src,
            "crates/verify-baseline/src/checks/structural.rs",
            true,
        );
        assert!(
            res_structural.is_ok(),
            "structural.rs failed audit_source_purity: {:?}",
            res_structural.err()
        );
        Ok(())
    }

    #[test]
    fn test_check_34_purity_a_rejects_regex() -> Result<(), Box<dyn std::error::Error>> {
        let clean = "pub fn add(a: i32, b: i32) -> i32 { a + b }";
        let res_clean = audit_source_purity(clean, "sample/src/lib.rs", false);
        assert!(
            res_clean.is_ok(),
            "Clean source must pass: {:?}",
            res_clean.err()
        );

        let pat = join_pair("Reg", "ex::");
        let mutant = format!("pub fn check() {{ {}new(\"\"); }}", pat);
        let res_mutant = audit_source_purity(&mutant, "sample/src/lib.rs", false);
        assert!(res_mutant.is_err(), "Mutant with regex must fail");
        let errs = res_mutant.err().ok_or("expected Err")?;
        assert!(errs.iter().any(|e| e.contains("regex usage detected")));
        Ok(())
    }

    #[test]
    fn test_check_34_purity_b_rejects_static_string_slices(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let clean = "pub const ALL: &'static [Self] = &[Self::VariantA, Self::VariantB];";
        let res_clean = audit_source_purity(clean, "sample/src/lib.rs", false);
        assert!(
            res_clean.is_ok(),
            "Clean enum array must pass: {:?}",
            res_clean.err()
        );

        let slice_kw = join_pair(": &[", "&str]");
        let mutant = format!("pub const KEYWORDS{kw} = &[\"forbidden\"];", kw = slice_kw);
        let res_mutant = audit_source_purity(&mutant, "sample/src/lib.rs", false);
        assert!(
            res_mutant.is_err(),
            "Mutant with static string slice array must fail"
        );
        let errs = res_mutant.err().ok_or("expected Err")?;
        assert!(errs
            .iter()
            .any(|e| e.contains("static string-slice array detected")));
        Ok(())
    }

    #[test]
    fn test_check_34_purity_c_rejects_hollow_gate_stubs() -> Result<(), Box<dyn std::error::Error>>
    {
        let clean = "pub fn check_gate(repo_root: &Path) -> Result<(), Vec<String>> { if repo_root.is_dir() { Ok(()) } else { Err(vec![]) } }";
        let res_clean = audit_source_purity(clean, "sample/src/checks/gate.rs", false);
        assert!(
            res_clean.is_ok(),
            "Clean genuine gate must pass: {:?}",
            res_clean.err()
        );

        let hollow_pat = join_pair("let", " _ = ");
        let mutant = format!("pub fn check_gate(repo_root: &Path) -> Result<(), Vec<String>> {{ {}repo_root; Ok(()) }}", hollow_pat);
        let res_mutant = audit_source_purity(&mutant, "sample/src/checks/gate.rs", false);
        assert!(
            res_mutant.is_err(),
            "Mutant with hollow gate stub must fail"
        );
        let errs = res_mutant.err().ok_or("expected Err")?;
        assert!(errs.iter().any(|e| e.contains("hollow gate stub detected")));
        Ok(())
    }

    #[test]
    fn test_check_34_purity_d_rejects_fail_open_error_swallowing(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let clean = "for entry_res in WalkDir::new(dir) { let entry = match entry_res { Ok(e) => e, Err(err) => return Err(vec![err.to_string()]) }; }";
        let res_clean = audit_source_purity(clean, "sample/src/checks/gate.rs", false);
        assert!(
            res_clean.is_ok(),
            "Clean fail-closed loop must pass: {:?}",
            res_clean.err()
        );

        let fm_pat = join_pair(".filter_map(|e|", " e.ok())");
        let mutant = format!("for entry in WalkDir::new(dir).into_iter(){} {{ }}", fm_pat);
        let res_mutant = audit_source_purity(&mutant, "sample/src/checks/gate.rs", false);
        assert!(
            res_mutant.is_err(),
            "Mutant with fail-open filter_map must fail"
        );
        let errs = res_mutant.err().ok_or("expected Err")?;
        assert!(errs
            .iter()
            .any(|e| e.contains("fail-open error swallowing detected")));
        Ok(())
    }

    #[test]
    fn test_check_34_purity_e_rejects_cross_string_byte_slicing(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let clean = "let lower = s.to_ascii_lowercase(); let slice = lower.get(..4);";
        let res_clean = audit_source_purity(clean, "sample/src/lib.rs", false);
        assert!(
            res_clean.is_ok(),
            "Clean character boundary slicing must pass: {:?}",
            res_clean.err()
        );

        let bad_slice = join_pair(".to_lowercase()", "[..4]");
        let mutant = format!("let slice = &s{};", bad_slice);
        let res_mutant = audit_source_purity(&mutant, "sample/src/lib.rs", false);
        assert!(
            res_mutant.is_err(),
            "Mutant with cross-string byte indexing must fail"
        );
        let errs = res_mutant.err().ok_or("expected Err")?;
        assert!(errs
            .iter()
            .any(|e| e.contains("cross-string UTF-8 byte-slice hazard detected")));
        Ok(())
    }

    #[test]
    fn test_check_34_purity_f_rejects_missing_seven_section_rustdoc(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let clean_tiered = r#"//! Sample parity module documented with a Tier A 4-Block Contract.
//!
//! Safety & Traceability: Realises: [REQ-PARITY-MOD]
//! Contract: Enforces deterministic baseline parity checks across audited artifacts.
//! Bounds & Faults: O(N) bounded traversal; fails closed with diagnostic vectors.

/// Public gate carrier documented with a Tier A 4-Block Contract.
///
/// Safety & Traceability: Realises: [REQ-PARITY-CARRIER]
/// Contract: Wraps immutable verification threshold `limit`.
/// Bounds & Faults: O(1) stack footprint; infallible value type.
pub struct SampleGate {
    pub limit: usize,
}

impl SampleGate {
    /// Evaluates the configured gate limit via a Tier A 4-Block Contract.
    ///
    /// Safety & Traceability: Realises: [REQ-PARITY-INHERENT-PUB]
    /// Contract: Pure read-only accessor returning `self.limit`.
    /// Bounds & Faults: O(1) time and space; infallible.
    pub fn configured_limit(&self) -> usize {
        self.limit
    }

    /// Computes internal clamped offset as a Tier B/C private method.
    /// Realises: [REQ-PARITY-PRIV-METHOD]
    fn clamped_offset(&self) -> usize {
        self.limit.saturating_add(1)
    }
}

impl Default for SampleGate {
    /// Constructs a zero-limit `SampleGate` via Tier B/C trait implementation.
    /// Realises: [REQ-PARITY-DEFAULT-IMPL]
    fn default() -> Self {
        Self { limit: 0 }
    }
}

/// High-integrity parity check documented with full 7-section Tier A Rustdoc.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A.
///
/// ## 2. Invariants & Mathematical Properties
/// Deterministic.
///
/// ## 3. Preconditions
/// repo_root is valid directory.
///
/// ## 4. Postconditions
/// Returns Ok(()) on verification.
///
/// ## 5. Algorithmic Complexity Bounds
/// O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-PARITY-CHECK]
pub fn check_audit() -> Result<(), Vec<String>> {
    let gate = SampleGate::default();
    if gate.configured_limit() == gate.clamped_offset() {
        return Err(Vec::new());
    }
    Ok(())
}

/// Public helper documented with a Tier A 4-Block Contract.
///
/// Safety & Traceability: Realises: [REQ-PARITY-PUB-4BLOCK]
/// Contract: Increments `count` with saturating arithmetic.
/// Bounds & Faults: O(1) time and space; never overflows.
pub fn increment_bound(count: usize) -> usize {
    crate_step(local_step(count))
}

/// Crate-internal helper documented under Tier B/C.
/// Realises: [REQ-PARITY-CRATE-FN]
pub(crate) fn crate_step(v: usize) -> usize {
    super_step(v)
}

/// Parent-module helper documented under Tier B/C.
/// Realises: [REQ-PARITY-SUPER-FN]
pub(super) fn super_step(v: usize) -> usize {
    v.saturating_add(1)
}

/// Private helper documented under Tier B/C without 7-section boilerplate.
/// Realises: [REQ-PARITY-PRIV-FN]
fn local_step(v: usize) -> usize {
    v.saturating_add(1)
}
"#;
        let res_clean =
            audit_source_purity(clean_tiered, "sample/src/checks/parity_gates.rs", true);
        assert!(
            res_clean.is_ok(),
            "Clean Tier A (7-section & 4-Block) and Tier B/C items must pass Check 34: {:?}",
            res_clean.err()
        );

        let partial_tier_a = r#"/// Partial doc missing section 7 and Realises tag.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A.
/// ## 2. Invariants & Mathematical Properties
/// Deterministic.
/// ## 3. Preconditions
/// repo_root is valid directory.
/// ## 4. Postconditions
/// Returns Ok(()) on verification.
/// ## 5. Algorithmic Complexity Bounds
/// O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed.
pub fn check_audit() -> Result<(), Vec<String>> {
    Ok(())
}
"#;
        let res_partial_a =
            audit_source_purity(partial_tier_a, "sample/src/checks/parity_gates.rs", true);
        assert!(
            res_partial_a.is_err(),
            "Tier A pub fn missing section 7 must fail"
        );
        let errs_partial_a = res_partial_a.err().ok_or("expected Err")?;
        assert!(
            errs_partial_a.iter().any(|e| e.contains(
                "Tier A fn 'check_audit' missing valid 7-section Rustdoc or 4-Block Contract"
            )),
            "{:?}",
            errs_partial_a
        );

        let downgraded_pub = r#"/// Concise two-line docstring is invalid on unrestricted Tier A pub fn.
/// Realises: [REQ-DOWNGRADED-PUB]
pub fn downgraded_api() {}
"#;
        assert!(
            audit_source_purity(downgraded_pub, "sample/src/checks/parity_gates.rs", true).is_err(),
            "Unrestricted pub fn downgraded to Tier B/C summary must fail closed"
        );

        let downgraded_mod = r#"//! Two-line module docstring lacking Tier A 7-section or 4-Block Contract.
//! Realises: [REQ-DOWNGRADED-MOD]

/// Private helper with valid Tier B/C docstring.
/// Realises: [REQ-VALID-HELPER]
fn valid_helper() {}
"#;
        assert!(
            audit_source_purity(downgraded_mod, "sample/src/checks/parity_gates.rs", true).is_err(),
            "Tier A //! module header downgraded to 2-line Tier B/C format must fail closed"
        );

        let undocumented_priv = "fn undocumented_helper() {}\n";
        assert!(
            audit_source_purity(undocumented_priv, "sample/src/checks/parity_gates.rs", true)
                .is_err(),
            "Undocumented private Tier B/C fn must fail closed"
        );

        let missing_realises_priv =
            "/// Private helper missing traceability tag.\nfn untraced_helper() {}\n";
        assert!(
            audit_source_purity(
                missing_realises_priv,
                "sample/src/checks/parity_gates.rs",
                true
            )
            .is_err(),
            "Private Tier B/C fn missing Realises: [...] must fail closed"
        );

        let pub_4block_missing_realises = r#"/// Public helper with 4-Block headers lacking Realises bracketed ID.
/// Safety & Traceability: Manually verified.
/// Contract: Preserves invariants.
/// Bounds & Faults: O(1) time and space.
pub fn untraced_pub_api() {}
"#;
        assert!(
            audit_source_purity(
                pub_4block_missing_realises,
                "sample/src/checks/parity_gates.rs",
                true
            )
            .is_err(),
            "Tier A 4-Block pub fn missing Realises: [...] must fail closed"
        );

        let undocumented_pub_crate = "pub(crate) fn undocumented_crate_helper() {}\n";
        assert!(
            audit_source_purity(
                undocumented_pub_crate,
                "sample/src/checks/parity_gates.rs",
                true
            )
            .is_err(),
            "Undocumented pub(crate) Tier B/C fn must fail closed"
        );

        let undocumented_trait_impl = r#"/// Public carrier with valid Tier A 4-Block Contract.
/// Safety & Traceability: Realises: [REQ-CARRIER-01]
/// Contract: Unit struct.
/// Bounds & Faults: O(1) stack.
pub struct UnitGate;

impl Default for UnitGate {
    fn default() -> Self {
        Self
    }
}
"#;
        assert!(
            audit_source_purity(
                undocumented_trait_impl,
                "sample/src/checks/parity_gates.rs",
                true
            )
            .is_err(),
            "Undocumented impl Trait for Type method must fail closed"
        );

        for marker in ["TODO", "TBD", "FIXME", "XXX"] {
            let placeholder_priv = format!(
                "/// Helper summary with {marker} placeholder.\n/// Realises: [REQ-PLACEHOLDER]\nfn helper_with_marker() {{}}\n"
            );
            assert!(
                audit_source_purity(&placeholder_priv, "sample/src/checks/parity_gates.rs", true)
                    .is_err(),
                "Placeholder marker {marker} in Tier B/C docstring must fail closed"
            );

            let placeholder_mod = format!(
                "//! Module header with {marker} placeholder.\n//! Safety & Traceability: Realises: [REQ-MOD-PLACEHOLDER]\n//! Contract: Deterministic.\n//! Bounds & Faults: O(1).\n"
            );
            assert!(
                audit_source_purity(&placeholder_mod, "sample/src/checks/parity_gates.rs", true)
                    .is_err(),
                "Placeholder marker {marker} in Tier A //! module header must fail closed"
            );
        }

        Ok(())
    }

    #[test]
    fn test_check_34_hardening_b_rejects_static_lifetime_string_slice_const(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let clean = "pub const ALL: &'static [Self] = &[Self::VariantA, Self::VariantB];";
        let res_clean = audit_source_purity(clean, "sample/src/lib.rs", false);
        assert!(
            res_clean.is_ok(),
            "Clean typed enum array must pass: {:?}",
            res_clean.err()
        );

        let ty = join_pair("&'static [", "&'static str]");
        let mutant = format!("pub const BAD: {} = &[\"a\"];", ty);
        let res_mutant = audit_source_purity(&mutant, "sample/src/lib.rs", false);
        assert!(
            res_mutant.is_err(),
            "Mutant with 'static string-slice const must fail"
        );
        let errs = res_mutant.err().ok_or("expected Err")?;
        assert!(
            errs.iter()
                .any(|e| e.contains("static string-slice array detected")),
            "{:?}",
            errs
        );

        let arr_ty = join_pair("[&'static ", "str; 2]");
        let mutant_arr = format!("pub const BAD: {} = [\"a\", \"b\"];", arr_ty);
        let res_arr = audit_source_purity(&mutant_arr, "sample/src/lib.rs", false);
        assert!(
            res_arr.is_err(),
            "Mutant with 'static string array must fail"
        );
        Ok(())
    }

    #[test]
    fn test_check_34_hardening_b_rejects_static_str_array_into_iter(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let clean = "fn kinds() -> std::array::IntoIter<Kind, 3> { [Kind::A, Kind::B, Kind::C].into_iter() }";
        let res_clean = audit_source_purity(clean, "sample/src/lib.rs", false);
        assert!(
            res_clean.is_ok(),
            "Clean typed IntoIter must pass: {:?}",
            res_clean.err()
        );

        let iter_ty = join_pair("IntoIter<", "&'static str, 3>");
        let mutant = format!(
            "fn names() -> std::array::{} {{ [\"a\", \"b\", \"c\"].into_iter() }}",
            iter_ty
        );
        let res_mutant = audit_source_purity(&mutant, "sample/src/lib.rs", false);
        assert!(
            res_mutant.is_err(),
            "Mutant with 'static str array iterator must fail"
        );
        let errs = res_mutant.err().ok_or("expected Err")?;
        assert!(
            errs.iter()
                .any(|e| e.contains("static string-slice array detected")),
            "{:?}",
            errs
        );
        Ok(())
    }

    #[test]
    fn test_check_34_hardening_e_rejects_direct_range_slice_indexing(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let clean = "let sub = s.get(1..3);";
        let res_clean = audit_source_purity(clean, "sample/src/checks/parity_gates.rs", true);
        assert!(
            res_clean.is_ok(),
            "Clean .get(..) slicing must pass: {:?}",
            res_clean.err()
        );

        let mutant = "let sub = &s[1..3];";
        let res_mutant = audit_source_purity(mutant, "sample/src/checks/parity_gates.rs", true);
        assert!(
            res_mutant.is_err(),
            "Mutant with direct panicking range indexing must fail"
        );
        let errs = res_mutant.err().ok_or("expected Err")?;
        assert!(
            errs.iter()
                .any(|e| e.contains("direct panicking range-slice indexing detected")),
            "{:?}",
            errs
        );

        let res_unaudited = audit_source_purity(mutant, "sample/src/lib.rs", false);
        assert!(
            res_unaudited.is_ok(),
            "Rule (e) applies only when check_rustdoc is true: {:?}",
            res_unaudited.err()
        );
        Ok(())
    }

    #[test]
    fn test_check_34_hardening_f_rejects_duplicate_boilerplate_rustdoc(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let body = r#"///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A.
///
/// ## 2. Invariants & Mathematical Properties
/// Deterministic.
///
/// ## 3. Preconditions
/// None.
///
/// ## 4. Postconditions
/// Returns Ok(()).
///
/// ## 5. Algorithmic Complexity Bounds
/// O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed.
///
/// ## 7. Realization & Traceability
"#;
        let mutant = format!(
            "/// First gate summary.\n{body}/// Realises: [REQ-A].\npub fn gate_a() {{}}\n\n/// Second gate summary.\n{body}/// Realises: [REQ-B].\npub fn gate_b() {{}}\n",
            body = body
        );
        let res_mutant = audit_source_purity(&mutant, "sample/src/checks/parity_gates.rs", true);
        assert!(
            res_mutant.is_err(),
            "Mutant with duplicated 7-section Rustdoc bodies must fail"
        );
        let errs = res_mutant.err().ok_or("expected Err")?;
        assert!(
            errs.iter()
                .any(|e| e.contains("duplicate/boilerplate 7-section")),
            "{:?}",
            errs
        );

        let unique_body = body.replace("Deterministic.", "Deterministic and idempotent.");
        let clean = format!(
            "/// First gate summary.\n{body}/// Realises: [REQ-A].\npub fn gate_a() {{}}\n\n/// Second gate summary.\n{unique}/// Realises: [REQ-B].\npub fn gate_b() {{}}\n",
            body = body,
            unique = unique_body
        );
        let res_clean = audit_source_purity(&clean, "sample/src/checks/parity_gates.rs", true);
        assert!(
            res_clean.is_ok(),
            "Function-specific Rustdoc bodies must pass: {:?}",
            res_clean.err()
        );
        Ok(())
    }

    #[test]
    fn test_reproduce_defect_318_stale_backlog_hallucinations_and_attribution_errors(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("repo root")?;
        let res = check_handoff_and_backlog_integrity(repo_root);
        assert!(
            res.is_ok(),
            "Defect #318 integrity check failed on repository root: {:?}",
            res.err()
        );
        Ok(())
    }

    #[test]
    fn test_defect_318_mutation_rejects_stale_root_handoff(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c318_root_handoff")?;
        let handoff = tmp.path.join("HANDOFF.md");
        File::create(&handoff)?;
        let res = check_handoff_and_backlog_integrity(&tmp.path);
        assert!(res.is_err(), "Must reject existence of root HANDOFF.md");
        Ok(())
    }

    #[test]
    fn test_defect_318_mutation_rejects_stale_attribution_error(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c318_stale_attr")?;
        let designs = tmp.path.join("docs").join("designs");
        fs::create_dir_all(&designs)?;
        let sol = designs.join("feat-289-290-293-solution.md");
        let mut f = File::create(&sol)?;
        writeln!(
            f,
            "# Solution (refs #318)\nprompt quarantine across all skills"
        )?;
        let res = check_handoff_and_backlog_integrity(&tmp.path);
        assert!(res.is_err(), "Must reject stale attribution error");
        Ok(())
    }

    #[test]
    fn test_defect_318_mutation_rejects_stale_work_package_queue(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c318_stale_queue")?;
        let designs = tmp.path.join("docs").join("designs");
        fs::create_dir_all(&designs)?;
        let sol = designs.join("feat-289-290-293-solution.md");
        let mut f = File::create(&sol)?;
        writeln!(f, "# Solution (refs #318)\nWork Package B: Issue #293")?;
        let res = check_handoff_and_backlog_integrity(&tmp.path);
        assert!(
            res.is_err(),
            "Must reject listing resolved issues as outstanding work packages"
        );
        Ok(())
    }

    #[test]
    fn test_reproduce_defect_321_root_master_handover_checkpoint_rejected(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c321_root_rejection")?;
        fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
        let schema_dir = tmp.path.join("schema");
        fs::create_dir_all(&schema_dir)?;
        File::create(schema_dir.join(".gitkeep"))?;

        let checkpoint = tmp.path.join("MASTER_HANDOVER_CHECKPOINT.md");
        File::create(&checkpoint)?;

        let res_no_checkpoint = check_no_master_handover_checkpoint(&tmp.path);
        assert!(
            res_no_checkpoint.is_err(),
            "Must reject root MASTER_HANDOVER_CHECKPOINT.md"
        );

        let res_upstream = verify_upstream_blueprint_domain_cleanliness(&tmp.path);
        assert!(
            res_upstream.is_err(),
            "Upstream cleanliness must reject root MASTER_HANDOVER_CHECKPOINT.md"
        );

        let res_handoff = check_handoff_and_backlog_integrity(&tmp.path);
        assert!(
            res_handoff.is_err(),
            "Handoff integrity must reject root MASTER_HANDOVER_CHECKPOINT.md"
        );
        Ok(())
    }

    #[test]
    fn test_defect_321_docs_master_handover_checkpoint_rejected(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c321_docs_rejection")?;
        fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
        let schema_dir = tmp.path.join("schema");
        fs::create_dir_all(&schema_dir)?;
        File::create(schema_dir.join(".gitkeep"))?;

        let docs_dir = tmp.path.join("docs").join("architecture");
        fs::create_dir_all(&docs_dir)?;
        let checkpoint = docs_dir.join("MASTER_HANDOVER_CHECKPOINT.md");
        File::create(&checkpoint)?;

        let res_no_checkpoint = check_no_master_handover_checkpoint(&tmp.path);
        assert!(
            res_no_checkpoint.is_err(),
            "Must reject docs/architecture/MASTER_HANDOVER_CHECKPOINT.md"
        );

        let res_upstream = verify_upstream_blueprint_domain_cleanliness(&tmp.path);
        assert!(
            res_upstream.is_err(),
            "Upstream cleanliness must reject docs MASTER_HANDOVER_CHECKPOINT.md"
        );

        let res_handoff = check_handoff_and_backlog_integrity(&tmp.path);
        assert!(
            res_handoff.is_err(),
            "Handoff integrity must reject docs MASTER_HANDOVER_CHECKPOINT.md"
        );
        Ok(())
    }

    #[test]
    fn test_defect_321_root_checkpoint_variant_rejected() -> Result<(), Box<dyn std::error::Error>>
    {
        let tmp = TempDir::new("c321_variant_rejection")?;
        let variant = tmp.path.join("MASTER_HANDOVER_CHECKPOINT_V2.md");
        File::create(&variant)?;

        let res = check_no_master_handover_checkpoint(&tmp.path);
        assert!(
            res.is_err(),
            "Must reject root checkpoint variant MASTER_HANDOVER_CHECKPOINT_V2.md"
        );
        Ok(())
    }

    #[test]
    fn test_defect_321_current_repo_passes() -> Result<(), Box<dyn std::error::Error>> {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("repo root")?;

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
        Ok(())
    }

    #[test]
    fn test_issue_441_documentation_contains_no_retired_script_references(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("repo root")?;
        let res = check_zero_retired_scripts_in_documentation(repo_root);
        assert!(
            res.is_ok(),
            "Documentation must not reference retired scripts (refs #441): {:?}",
            res.err()
        );
        Ok(())
    }

    #[test]
    fn test_defect_441_mutation_rejects_retired_script_in_markdown(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c441_script_rejection")?;
        let docs = tmp.path.join("docs").join("operations");
        fs::create_dir_all(&docs)?;
        let guide = docs.join("install-guide.md");
        let mut f = File::create(&guide)?;
        let needle = "create_issue.sh";
        writeln!(f, "# Guide\nRun {needle} now")?;

        let res = check_zero_retired_scripts_in_documentation(&tmp.path);
        assert!(res.is_err(), "Must reject retired script reference in docs");
        let errs = res.err().ok_or("expected Err")?;
        assert!(errs.iter().any(|e| e.contains(needle)));
        Ok(())
    }

    #[test]
    fn test_defect_450_retired_python_in_pipeline_docs_fails(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c450_pipeline_retired")?;
        let pipeline_dir = tmp.path.join(".pipeline");
        fs::create_dir_all(&pipeline_dir)?;
        let const_file = pipeline_dir.join("constitution.md");
        let mut f = File::create(&const_file)?;
        writeln!(
            f,
            "# Constitution\nRuns validators/spec_validator.py for quality gate"
        )?;

        let res = check_zero_retired_scripts_in_documentation(&tmp.path);
        assert!(
            res.is_err(),
            "Must reject retired Python validator in .pipeline/constitution.md"
        );
        let errs = res.err().ok_or("expected Err")?;
        assert!(errs.iter().any(|e| e.contains("spec_validator.py")));
        Ok(())
    }

    #[test]
    fn test_defect_459_feat42_design_blueprint_compliance() -> Result<(), Box<dyn std::error::Error>>
    {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("repo root")?;

        let res = check_feat42_design_blueprint_compliance(repo_root);
        assert!(
            res.is_ok(),
            "FEAT-42 design blueprint must comply with deap-core references, CLI backwards compatibility, and Check 34 uniqueness (refs #459): {:?}",
            res.err()
        );
        Ok(())
    }

    #[test]
    fn test_defect_459_mutation_rejects_sysml_core_in_design(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c459_sysml_core_mutation")?;
        let designs = tmp.path.join("docs").join("designs");
        fs::create_dir_all(&designs)?;
        let feat = designs.join("FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md");
        let mut f = File::create(&feat)?;
        writeln!(f, "# FEAT-42\nparticipant Core as sysml-core Parser")?;

        let res = check_feat42_design_blueprint_compliance(&tmp.path);
        assert!(
            res.is_err(),
            "Must reject sysml-core in FEAT-42 design blueprint"
        );
        let errs = res.err().ok_or("expected Err")?;
        assert!(errs.iter().any(|e| e.contains("sysml-core")));
        Ok(())
    }

    #[test]
    fn test_check_37_blueprint_grounding_passes_on_live_repo(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("repo root")?;

        let res = check_blueprint_grounding(repo_root);
        assert!(
            res.is_ok(),
            "Check 37 blueprint grounding failed on live repository: {:?}",
            res.err()
        );
        Ok(())
    }

    #[test]
    fn test_check_37_rejects_fictional_crates_and_nonexistent_paths(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c37_mutations")?;
        fs::create_dir_all(tmp.path.join(".pipeline").join("upstream"))?;
        let bp_dir = tmp
            .path
            .join("docs")
            .join("architecture")
            .join("blueprints");
        fs::create_dir_all(&bp_dir)?;
        fs::create_dir_all(tmp.path.join("crates").join("deap-core").join("src"))?;
        File::create(
            tmp.path
                .join("crates")
                .join("deap-core")
                .join("src")
                .join("lib.rs"),
        )?;

        for bp in CanonicalArchitectureBlueprint::ALL {
            let mut f = File::create(bp_dir.join(bp.as_str()))?;
            writeln!(f, "# Blueprint\nGrounded in `crates/deap-core/src/lib.rs`.")?;
        }

        let clean_res = check_blueprint_grounding(&tmp.path);
        assert!(
            clean_res.is_ok(),
            "Clean grounded blueprints must pass Check 37: {:?}",
            clean_res.err()
        );

        let target_bp = bp_dir
            .join(CanonicalArchitectureBlueprint::DeapSysmlV2IngestionEngineBlueprint.as_str());
        let mut f = File::create(&target_bp)?;
        writeln!(
            f,
            "# Corrupted Blueprint\nUses `deap-ast` and `deap-harness`.\nRuns `verify_downstream`.\nInspects `crates/verify-baseline/src/checks/uml.rs`.\nContains em dash \u{2014} here."
        )?;

        let err_res = check_blueprint_grounding(&tmp.path);
        assert!(
            err_res.is_err(),
            "Check 37 must fail closed on fictional crates, retired tools, nonexistent paths, and em dashes"
        );
        let errs = err_res.err().ok_or("expected Err")?;
        assert!(
            errs.iter().any(|e| e.contains("deap-ast")),
            "Must report fictional crate deap-ast: {:?}",
            errs
        );
        assert!(
            errs.iter().any(|e| e.contains("deap-harness")),
            "Must report fictional crate deap-harness: {:?}",
            errs
        );
        assert!(
            errs.iter().any(|e| e.contains("verify_downstream")),
            "Must report retired tool verify_downstream: {:?}",
            errs
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("crates/verify-baseline/src/checks/uml.rs")),
            "Must report nonexistent path crates/verify-baseline/src/checks/uml.rs: {:?}",
            errs
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("\\u2014") || e.contains("em dash")),
            "Must report em dash violation: {:?}",
            errs
        );
        Ok(())
    }

    /// Verifies that Check 33 excludes machine-generated digest JSON sidecars from governed text scanning (refs #532).
    #[test]
    fn test_check_33_excludes_digest_json_sidecars() -> Result<(), Box<dyn std::error::Error>> {
        assert!(!is_governed_python_scan_extension(Path::new(
            "docs/designs/blueprint-01.digest.json"
        )));
        assert!(!is_governed_python_scan_extension(Path::new(
            "docs/designs/foo-digest.json"
        )));
        assert!(!is_governed_python_scan_extension(Path::new(
            ".pipeline/schema-digest.json"
        )));
        assert!(is_governed_python_scan_extension(Path::new(
            "docs/config.json"
        )));

        let tmp = TempDir::new("c33_digest_json_sidecar")?;
        let designs_dir = tmp.path.join("docs").join("designs");
        fs::create_dir_all(&designs_dir)?;
        let digest_file = designs_dir.join("blueprint-01.digest.json");
        let mut f = File::create(&digest_file)?;
        let token = join_pair("run_", "tests.py");
        writeln!(
            f,
            "{{\"summary\": \"executes pytest and {} in downstream runner\"}}",
            token
        )?;

        let res = check_zero_python_scripts(&tmp.path);
        assert!(
            res.is_ok(),
            "Check 33 must exclude machine-generated .digest.json sidecars (refs #532), got: {:?}",
            res.err()
        );
        Ok(())
    }

    /// Verifies that Check 37 scans downstream `docs/designs/blueprint-*.md` files for grounding violations (refs #533).
    #[test]
    fn test_check_37_scans_docs_designs_blueprints() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c37_docs_designs_blueprints")?;
        let designs_dir = tmp.path.join("docs").join("designs");
        fs::create_dir_all(&designs_dir)?;

        let bp_file = designs_dir.join("blueprint-01-core.md");
        {
            let mut f = File::create(&bp_file)?;
            writeln!(
                f,
                "# Design Blueprint 01\nGrounded in `crates/deap-parser/src/lib.rs`."
            )?;
        }

        let err_res = check_blueprint_grounding(&tmp.path);
        assert!(
            err_res.is_err(),
            "Check 37 must scan docs/designs/blueprint-*.md even when docs/architecture/blueprints is absent (refs #533)"
        );
        let errs = err_res.err().ok_or("expected Err")?;
        assert!(
            errs.iter()
                .any(|e| e.contains("docs/designs/blueprint-01-core.md")
                    && e.contains("deap-parser")),
            "Expected grounding error on docs/designs/blueprint-01-core.md for deap-parser, got: {:?}",
            errs
        );

        {
            let mut f = File::create(&bp_file)?;
            writeln!(
                f,
                "# Design Blueprint 01\nPure domain design specification without ungrounded crate paths."
            )?;
        }

        let clean_res = check_blueprint_grounding(&tmp.path);
        assert!(
            clean_res.is_ok(),
            "Valid docs/designs/blueprint-01-core.md must pass Check 37: {:?}",
            clean_res.err()
        );
        Ok(())
    }

    /// Verifies that Check 37 does not require upstream `CanonicalArchitectureBlueprint` files in downstream workspaces (refs #743).
    #[test]
    fn test_check_37_exempts_downstream_from_canonical_architecture_blueprints(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c37_downstream_blueprints_dir")?;
        let bp_dir = tmp
            .path
            .join("docs")
            .join("architecture")
            .join("blueprints");
        fs::create_dir_all(&bp_dir)?;
        File::create(bp_dir.join(".gitkeep"))?;

        let designs_dir = tmp.path.join("docs").join("designs");
        fs::create_dir_all(&designs_dir)?;
        let design_bp = designs_dir.join("blueprint-01-core.md");
        {
            let mut f = File::create(&design_bp)?;
            writeln!(
                f,
                "# Design Blueprint 01\nPure downstream domain design specification."
            )?;
        }

        let clean_res = check_blueprint_grounding(&tmp.path);
        assert!(
            clean_res.is_ok(),
            "Downstream workspace with docs/architecture/blueprints/.gitkeep must not require upstream canonical blueprints (refs #743): {:?}",
            clean_res.err()
        );

        {
            let mut f = File::create(&design_bp)?;
            writeln!(
                f,
                "# Design Blueprint 01\nReferences fictional `crates/deap-parser/src/lib.rs`."
            )?;
        }

        let err_res = check_blueprint_grounding(&tmp.path);
        assert!(
            err_res.is_err(),
            "Check 37 must still fail closed on ungrounded crate paths in downstream design blueprints"
        );
        Ok(())
    }

    /// Verifies `REQ-0601` Clippy suppression token scanning and workspace manifest forbid gates (refs #525).
    #[test]
    fn test_issue_525_scan_clippy_allow_tokens_and_manifest_forbid_gate(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let kw = join_pair("allow(clip", "py::");

        // 1. Comments, strings, raw strings, and character literals must NOT trigger false positives
        let clean_src = format!(
            "// #[{kw}unwrap_used)]\n\
             /* outer /* #[{kw}panic)] */ still comment */\n\
             pub fn demo() {{\n\
                 let s = \"#[{kw}expect_used)]\";\n\
                 let r = r#\"#![{kw}indexing_slicing)]\"#;\n\
                 let ch = '#';\n\
                 let br = '[';\n\
                 let _pair = (s, r, ch, br);\n\
             }}\n"
        );
        let clean_viols = scan_clippy_allow_tokens(&clean_src, "crates/sample/src/lib.rs");
        assert!(
            clean_viols.is_empty(),
            "Comments and string/raw-string/char literals must not trigger ClippySuppressionViolation: {:?}",
            clean_viols
        );

        // 2. Outer, inner, multiline/whitespace-split, and cfg_attr-wrapped attributes MUST be caught
        let dirty_src = format!(
            "#![{kw}indexing_slicing)]\n\
             #[{kw}unwrap_used)]\n\
             #[\n  allow(\n    clippy :: type_complexity\n  )\n]\n\
             #[cfg_attr(test, {kw}panic))]\n\
             pub fn bad() {{}}\n"
        );
        let dirty_viols = scan_clippy_allow_tokens(&dirty_src, "crates/sample/src/lib.rs");
        assert_eq!(
            dirty_viols.len(),
            4,
            "Expected 4 violations, got: {:?}",
            dirty_viols
        );
        let v0 = dirty_viols.first().ok_or("missing v0")?;
        assert_eq!(v0.line, 1);
        assert_eq!(v0.attribute, format!("#![{kw}indexing_slicing)]"));
        let v1 = dirty_viols.get(1).ok_or("missing v1")?;
        assert_eq!(v1.line, 2);
        assert_eq!(v1.attribute, format!("#[{kw}unwrap_used)]"));
        let v2 = dirty_viols.get(2).ok_or("missing v2")?;
        assert_eq!(v2.line, 3);
        assert_eq!(v2.attribute, format!("#[{kw}type_complexity)]"));
        let v3 = dirty_viols.get(3).ok_or("missing v3")?;
        assert_eq!(v3.line, 8);
        assert_eq!(v3.attribute, format!("#[cfg_attr(test,{kw}panic))]"));

        let purity_res = audit_source_purity(&dirty_src, "crates/deap-core/src/rules.rs", false);
        assert!(
            purity_res.is_err(),
            "audit_source_purity must reject forbidden clippy allow attributes"
        );
        let purity_errs = purity_res.err().ok_or("expected Err")?;
        assert!(
            purity_errs
                .iter()
                .any(|e| e.contains("forbidden clippy allow attribute detected")),
            "Expected forbidden clippy allow attribute diagnostic, got: {:?}",
            purity_errs
        );

        // 3. WorkspaceManifestForbidGate negative probe: "deny" instead of "forbid" and missing lints
        let tmp_bad = TempDir::new("c525_manifest_bad")?;
        {
            let mut f = File::create(tmp_bad.path.join("Cargo.toml"))?;
            writeln!(
                f,
                "[workspace]\nmembers = [\"crates/sample\"]\n\n\
                 [workspace.lints.rust]\nunsafe_code = \"deny\"\n\n\
                 [workspace.lints.clippy]\nunwrap_used = \"deny\"\n"
            )?;
        }
        let sample_dir = tmp_bad.path.join("crates").join("sample");
        fs::create_dir_all(&sample_dir)?;
        {
            let mut f = File::create(sample_dir.join("Cargo.toml"))?;
            writeln!(
                f,
                "[package]\nname = \"sample\"\nversion = \"0.1.0\"\n\
                 [dependencies]\nclap = {{ version = \"4.4\", features = [\"derive\"] }}\n"
            )?;
        }
        let bad_res = WorkspaceManifestForbidGate::verify(&tmp_bad.path);
        assert!(
            bad_res.is_err(),
            "WorkspaceManifestForbidGate must reject non-forbid lints, missing lints, missing [lints] workspace = true, and clap derive"
        );
        let bad_errs = bad_res.err().ok_or("expected Err")?;
        assert!(bad_errs.iter().any(|e| e.contains("unsafe_code")));
        assert!(bad_errs.iter().any(|e| e.contains("unwrap_used")));
        assert!(bad_errs.iter().any(|e| e.contains("type_complexity")));
        assert!(bad_errs
            .iter()
            .any(|e| e.contains("missing [lints] workspace = true")));

        // 4. Live repository root must pass WorkspaceManifestForbidGate and check_zero_regex
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("repo root")?;
        let live_gate = WorkspaceManifestForbidGate::verify(repo_root);
        assert!(
            live_gate.is_ok(),
            "Live repository manifests failed WorkspaceManifestForbidGate (refs #525): {:?}",
            live_gate.err()
        );
        let live_c34 = check_zero_regex(repo_root);
        assert!(
            live_c34.is_ok(),
            "Live repository failed check_zero_regex (refs #525): {:?}",
            live_c34.err()
        );
        Ok(())
    }

    /// Regression test for Issue #526 Check 34 self-hosting AST purity, comment/literal masking, and fail-closed gates.
    #[test]
    fn test_issue_526_self_hosting_purity_and_fail_closed_gates(
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Chained range-slice indexing `arr[0][1..2]` must be rejected
        let chained_mutant = "let sub = &arr[0][1..2];";
        let res_chained =
            audit_source_purity(chained_mutant, "sample/src/checks/parity_gates.rs", true);
        assert!(
            res_chained.is_err(),
            "Chained range-slice indexing arr[0][1..2] must be rejected"
        );
        assert!(res_chained
            .err()
            .ok_or("expected Err")?
            .iter()
            .any(|e| e.contains("direct panicking range-slice indexing detected")));

        // 2. String literal `"mod tests"` or closed `mod tests { ... }` before production code must not bypass Check 34
        let hollow = join_pair("let", " _ = ");
        let mod_str = join_pair("mod", " tests");
        let literal_bypass_mutant =
            format!("pub fn prod(x: i32) {{\n    let s = \"{mod_str}\";\n    {hollow}x;\n}}\n");
        let res_lit_bypass =
            audit_source_purity(&literal_bypass_mutant, "sample/src/lib.rs", false);
        assert!(
            res_lit_bypass.is_err(),
            "String literal containing 'mod tests' must not disable Check 34 on subsequent lines"
        );

        let closed_mod_bypass_mutant = format!(
            "mod tests {{\n    #[test]\n    fn valid_test() {{ assert_eq!(1, 1); }}\n}}\npub fn sneaky(x: i32) {{\n    {hollow}x;\n}}\n"
        );
        let res_closed_mod =
            audit_source_purity(&closed_mod_bypass_mutant, "sample/src/lib.rs", false);
        assert!(
            res_closed_mod.is_err(),
            "Closed mod tests block must not disable Check 34 on subsequent production code"
        );

        // 3. C-PURITY-03 test anti-gaming violations inside `mod tests`
        let ign = join_pair("#[ig", "nore]");
        let res_ign = audit_source_purity(
            &format!(
                "mod tests {{\n    #[test]\n    {ign}\n    fn t() {{ assert_eq!(1, 1); }}\n}}\n"
            ),
            "sample/src/checks/parity_gates.rs",
            true,
        );
        assert!(
            res_ign.is_err(),
            "C-PURITY-03 must reject #[ignore] in mod tests"
        );

        let taut = join_pair("assert!", "(true)");
        let res_taut = audit_source_purity(
            &format!("mod tests {{\n    #[test]\n    fn t() {{ {taut}; }}\n}}\n"),
            "sample/src/checks/parity_gates.rs",
            true,
        );
        assert!(
            res_taut.is_err(),
            "C-PURITY-03 must reject tautological assert!(true)"
        );

        let vacuous = join_pair("if p.ex", "ists() { return; }");
        let res_vacuous = audit_source_purity(
            &format!("mod tests {{\n    #[test]\n    fn t() {{ {vacuous} }}\n}}\n"),
            "sample/src/checks/parity_gates.rs",
            true,
        );
        assert!(
            res_vacuous.is_err(),
            "C-PURITY-03 must reject vacuous early-return if p.exists() {{ return; }}"
        );

        let unisolated = join_pair("temp_dir()", ".join(\"fixed_shared_dir\")");
        let res_unisolated = audit_source_purity(
            &format!("mod tests {{\n    #[test]\n    fn t() {{ let d = std::env::{unisolated}; assert!(d.is_Absolute() || true); }}\n}}\n"),
            "sample/src/checks/parity_gates.rs",
            true,
        );
        assert!(
            res_unisolated.is_err(),
            "C-PURITY-03 must reject unisolated literal temp_dir().join(\"...\")"
        );

        let decoy_mod = join_pair("mod", " tests {}");
        let res_decoy = audit_source_purity(&decoy_mod, "sample/src/checks/parity_gates.rs", true);
        assert!(
            res_decoy.is_err(),
            "C-PURITY-03 must reject empty decoy mod tests {{}}"
        );

        // 4. Rule (d) must reject `Err(_) => continue` and `if let Ok(...) = fs::read_to_string(...)`
        let err_cont = join_pair("Err(_)", " => continue");
        let res_err_cont = audit_source_purity(
            &format!("pub fn scan() {{ match res {{ Ok(v) => v, {err_cont} }} }}\n"),
            "sample/src/lib.rs",
            false,
        );
        assert!(
            res_err_cont.is_err(),
            "Rule (d) must reject fail-open Err(_) => continue"
        );

        let if_ok_read = join_pair("if let ", "Ok(c) = fs::read_to_string(p)");
        let res_if_ok = audit_source_purity(
            &format!("pub fn scan() {{ {if_ok_read} {{}} }}\n"),
            "sample/src/lib.rs",
            false,
        );
        assert!(
            res_if_ok.is_err(),
            "Rule (d) must reject fail-open if let Ok(...) = fs::read_to_string(...)"
        );

        // 5. check_compiler_ast_purity must fail closed when audited files are missing
        let tmp = TempDir::new("c526_missing_audited")?;
        let res_missing = check_compiler_ast_purity(&tmp.path);
        assert!(
            res_missing.is_err(),
            "check_compiler_ast_purity must fail closed when audited files are missing"
        );
        Ok(())
    }

    /// Verifies `enclosing_upstream_compiler_root` resolves `<root>` for `<root>/.pipeline/upstream` while standalone dirs fail closed.
    #[test]
    fn test_enclosing_upstream_compiler_root_resolves_dogfood_and_fails_closed_standalone(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root_tmp = TempDir::new("upstream_dogfood_root")?;
        let dogfood_dir = root_tmp.path.join(".pipeline").join("upstream");
        fs::create_dir_all(&dogfood_dir)?;

        fs::write(root_tmp.path.join(".gitignore"), "target/\n")?;
        fs::write(
            root_tmp.path.join("README.md"),
            "# Upstream Compiler\n## Operator Prompt Catalog\n",
        )?;
        fs::write(root_tmp.path.join("AGENTS.md"), "# Agent Governance\n")?;
        let rules_dir = root_tmp.path.join("rules");
        fs::create_dir_all(&rules_dir)?;
        fs::write(
            rules_dir.join("sysml-ssot-completeness.md"),
            "# SysML SSOT Completeness\n",
        )?;
        let reconcile_crate = root_tmp.path.join("crates").join("reconcile-backlog");
        fs::create_dir_all(reconcile_crate.join("src"))?;
        fs::write(
            reconcile_crate.join("Cargo.toml"),
            "[package]\nname = \"reconcile-backlog\"\nversion = \"0.1.0\"\n",
        )?;
        fs::write(
            reconcile_crate.join("src").join("main.rs"),
            "fn main() {}\n",
        )?;

        assert_eq!(
            enclosing_upstream_compiler_root(&dogfood_dir),
            Some(root_tmp.path.clone()),
            "enclosing_upstream_compiler_root must resolve <root> for <root>/.pipeline/upstream"
        );
        assert_eq!(
            enclosing_upstream_compiler_root(&root_tmp.path),
            None,
            "enclosing_upstream_compiler_root must return None for <root> itself"
        );

        assert!(
            check_gitignore_exists(&dogfood_dir).is_ok(),
            "Check 10 must pass for <root>/.pipeline/upstream when <root>/.gitignore exists"
        );
        assert!(
            check_downstream_instructions_exist(&dogfood_dir).is_ok(),
            "Check 14 must pass for <root>/.pipeline/upstream when <root> instructions exist: {:?}",
            check_downstream_instructions_exist(&dogfood_dir)
        );
        assert!(
            check_reconcile_backlog_tooling_exists(&dogfood_dir).is_ok(),
            "Check 15 must pass for <root>/.pipeline/upstream when <root>/crates/reconcile-backlog exists: {:?}",
            check_reconcile_backlog_tooling_exists(&dogfood_dir)
        );

        let standalone_tmp = TempDir::new("standalone_downstream_fail_closed")?;
        assert_eq!(enclosing_upstream_compiler_root(&standalone_tmp.path), None);
        assert!(
            check_gitignore_exists(&standalone_tmp.path).is_err(),
            "Standalone downstream directory without .gitignore must fail Check 10 closed"
        );
        assert!(
            check_downstream_instructions_exist(&standalone_tmp.path).is_err(),
            "Standalone downstream directory without instructions must fail Check 14 closed"
        );
        assert!(
            check_reconcile_backlog_tooling_exists(&standalone_tmp.path).is_err(),
            "Standalone downstream directory without reconcile-backlog must fail Check 15 closed"
        );
        Ok(())
    }

    /// Verifies Check 33 rejects Python cache dirs, manifests, bytecode files, package runners, and banner/never evasion lines.
    #[test]
    fn test_check_33_hardened_artifacts_and_substring_loophole_closed(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("c33_hardened")?;
        let pycache = tmp.path.join("__pycache__");
        fs::create_dir_all(&pycache)?;
        let req_txt = tmp.path.join("requirements.txt");
        File::create(&req_txt)?;
        let docs_dir = tmp.path.join("docs");
        fs::create_dir_all(&docs_dir)?;
        let doc_file = docs_dir.join("guide.md");
        let py_runner = join_pair("py", "test");
        let pip_cmd = join_pair("pip", " install");
        fs::write(
            &doc_file,
            format!(
                "Run {py_runner} to verify the banner never flickers.\nUse {pip_cmd} for deps.\n"
            ),
        )?;
        let res = check_zero_python_scripts(&tmp.path);
        assert!(
            res.is_err(),
            "Check 33 must reject __pycache__, requirements.txt, and banner/never evasion lines"
        );
        let errs = res.err().ok_or("expected Err")?;
        assert!(errs.iter().any(|e| e.contains("__pycache__")));
        assert!(errs.iter().any(|e| e.contains("requirements.txt")));
        assert!(errs.iter().any(|e| e.contains("guide.md:1:")));
        assert!(errs.iter().any(|e| e.contains("guide.md:2:")));
        Ok(())
    }

    /// Joins two string slices for negative-probe test inputs without array-join or macro-join tokens.
    fn join_pair(a: &str, b: &str) -> String {
        let mut s = String::with_capacity(a.len().saturating_add(b.len()));
        s.push_str(a);
        s.push_str(b);
        s
    }

    /// Verifies zero-allocation split-token matching, decomposed `<module>/sub.rs` directory matching across all 12 audited modules, and zero self-scan evasion needles in `structural.rs`.
    #[test]
    fn test_audited_module_dirs_and_strict_io_arms() -> Result<(), Box<dyn std::error::Error>> {
        // 1. Verify zero-allocation split-token helpers on overlapping, UTF-8, and boundary cases
        let overlap_probe = format!("let _ = Reg{}new();", join_pair("Reg", "ex::"));
        assert!(has_split_token(&overlap_probe, "Reg", "ex::"));
        assert!(has_split_token("prefix αβγδ suffix", "αβ", "γδ"));
        assert!(!has_split_token("let _ = RegOnly;", "Reg", "ex::"));
        assert!(!has_split_token("short", "longer_prefix", "x"));

        let cargo_probe = format!("{} = \"1.10\"", join_pair("reg", "ex"));
        let cargo_ident = format!("my_{}_helper = \"1.10\"", join_pair("reg", "ex"));
        assert!(contains_bounded_split_token(&cargo_probe, "reg", "ex"));
        assert!(!contains_bounded_split_token(&cargo_ident, "reg", "ex"));

        // 2. Verify all 12 audited modules match both `<stem>.rs` and `<stem>/sub.rs`, while siblings do not
        assert_eq!(AUDITED_MODULES.len(), 12);
        for spec in AUDITED_MODULES {
            let file_path = format!("{}.rs", spec.workspace_stem);
            let dir_sub_path = format!("{}/sub.rs", spec.workspace_stem);
            let sibling_path = format!("{}_extra.rs", spec.workspace_stem);
            assert!(
                is_audited_source_path(&file_path),
                "Expected {file_path} to match is_audited_source_path"
            );
            assert!(
                is_audited_source_path(&dir_sub_path),
                "Expected {dir_sub_path} to match is_audited_source_path"
            );
            assert!(
                !is_audited_source_path(&sibling_path),
                "Expected non-audited sibling {sibling_path} not to match"
            );
            assert_eq!(
                enforces_strict_io_arms(&file_path),
                enforces_strict_io_arms(&dir_sub_path),
                "Strict I/O arm policy must match between {file_path} and {dir_sub_path}"
            );
        }
        assert!(!is_audited_source_path("crates/deap-core/src/lib.rs"));

        // 3. End-to-end check_zero_regex & check_compiler_ast_purity on decomposed `<module>/sub.rs` directories
        let tmp = TempDir::new("c34_decomposed_module_dirs")?;
        let markdown_sub_dir = tmp
            .path
            .join("crates")
            .join("deap-core")
            .join("src")
            .join("markdown");
        fs::create_dir_all(&markdown_sub_dir)?;
        fs::write(
            markdown_sub_dir.join("sub.rs"),
            "pub fn undocumented_in_decomposed_markdown_dir() {}\n",
        )?;

        let res_decomposed_audit = check_zero_regex(&tmp.path);
        assert!(
            res_decomposed_audit.is_err(),
            "Decomposed crates/deap-core/src/markdown/sub.rs must be matched as audited in check_zero_regex"
        );

        let tmp_purity = TempDir::new("c34_decomposed_compiler_purity")?;
        let valid_mod_src = clean_decomposed_tier_a_source();
        for spec in AUDITED_MODULES {
            let mod_dir = tmp_purity.path.join(spec.workspace_stem);
            fs::create_dir_all(&mod_dir)?;
            fs::write(mod_dir.join("mod.rs"), &valid_mod_src)?;
            fs::write(mod_dir.join("sub.rs"), &valid_mod_src)?;
        }
        let res_all_dirs_clean = check_compiler_ast_purity(&tmp_purity.path);
        assert!(
            res_all_dirs_clean.is_ok(),
            "Decomposed <module>/ directories with clean Rust submodules must pass check_compiler_ast_purity: {:?}",
            res_all_dirs_clean.err()
        );

        // Impure sub.rs inside decomposed directory must fail closed
        fs::write(
            tmp_purity
                .path
                .join("crates/ingest-sysml/src/translators/code/sub.rs"),
            "pub fn undocumented_sub() {}\n",
        )?;
        assert!(
            check_compiler_ast_purity(&tmp_purity.path).is_err(),
            "Impure sub.rs inside decomposed module directory must fail check_compiler_ast_purity"
        );

        // 4. Verify strict I/O arms apply identically to `<module>.rs` and `<module>/sub.rs`
        let err_arm = join_pair("Err(_)", " => continue");
        let io_loop_probe = format!(
            "pub fn parse_loop() {{ loop {{ match step() {{ Ok(()) => break, {err_arm} }} }} }}\n"
        );

        for exempt_path in [
            "crates/deap-core/src/sysml_ast.rs",
            "crates/deap-core/src/sysml_ast/sub.rs",
            "crates/deap-core/src/workspace.rs",
            "crates/deap-core/src/workspace/sub.rs",
            "crates/verify-baseline/src/checks/parity_gates.rs",
            "crates/verify-baseline/src/checks/parity_gates/sub.rs",
        ] {
            let res = audit_source_purity(&io_loop_probe, exempt_path, false);
            assert!(
                res.is_ok(),
                "Strict I/O arm exemption must apply uniformly to {exempt_path}: {:?}",
                res.err()
            );
        }

        for strict_path in [
            "crates/verify-baseline/src/checks/structural.rs",
            "crates/verify-baseline/src/checks/structural/sub.rs",
            "crates/deap-core/src/markdown.rs",
            "crates/deap-core/src/markdown/sub.rs",
            "crates/deap-core/src/source_ast.rs",
            "crates/deap-core/src/source_ast/sub.rs",
        ] {
            let res = audit_source_purity(&io_loop_probe, strict_path, false);
            assert!(
                res.is_err(),
                "Strict I/O arms must fail closed on {strict_path}"
            );
        }

        // 5. Self-scan invariant: structural.rs contains zero slice-join or macro-join evasion tokens
        let structural_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("checks")
            .join("structural.rs");
        let structural_text = fs::read_to_string(&structural_path)?;
        let forbidden_method = join_pair(".con", "cat(");
        let forbidden_macro = join_pair("con", "cat!");
        assert!(
            !structural_text.contains(&forbidden_method),
            "structural.rs must not contain slice-join evasion calls"
        );
        assert!(
            !structural_text.contains(&forbidden_macro),
            "structural.rs must not contain macro-join evasion calls"
        );
        Ok(())
    }

    /// Returns a minimal valid Tier A module source snippet for decomposed-directory purity tests.
    fn clean_decomposed_tier_a_source() -> String {
        String::from(
            "//! Decomposed submodule fixture.\n\
             //!\n\
             //! Safety & Traceability: Realises: [REQ-DECOMPOSED-FIXTURE]\n\
             //! Contract: Pure deterministic test fixture.\n\
             //! Bounds & Faults: O(1) time and space; infallible.\n\n\
             /// Evaluates deterministic constant.\n\
             ///\n\
             /// Safety & Traceability: Realises: [REQ-DECOMPOSED-FN]\n\
             /// Contract: Returns `1`.\n\
             /// Bounds & Faults: O(1) time and space; infallible.\n\
             pub const fn unit_value() -> usize {\n\
                 1\n\
             }\n",
        )
    }
}
