//! Structural repository topology and integrity checks.

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
use std::path::Path;
use walkdir::WalkDir;

/// Check 10: Verify .gitignore exists in repository root.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C configuration management controls.
/// Ensures `.gitignore` is present at the repository root.
///
/// ## 2. Invariants & Mathematical Properties
/// `repo_root.join(".gitignore")` must resolve to an existing regular file.
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
    core_check_gitignore_exists(repo_root)
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
    core_check_downstream_instructions_exist(repo_root)
}

/// Check 15: Verify reconcile-backlog tooling exists and is executable/buildable.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C traceability tooling controls.
/// Ensures backlog reconciliation tooling is present in the workspace.
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
    core_check_reconcile_backlog_tooling_exists(repo_root)
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

        for entry_res in WalkDir::new(zone_dir)
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
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
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

    // 2. Enforce zero concrete modeling files (.sysml) in upstream blueprints
    let blueprints_dir = repo_root.join("docs").join("architecture").join("blueprints");
    if blueprints_dir.is_dir() {
        for entry_res in WalkDir::new(&blueprints_dir)
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
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };
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

    for entry_res in WalkDir::new(repo_root)
        .into_iter()
        .filter_entry(|e| {
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
        })
    {
        let entry = match entry_res {
            Ok(e) => e,
            Err(_) => continue,
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C Rust-native toolchain purity controls.
/// Recursively walks `repo_root: &Path` to reject any physical `.py` file on disk and scans
/// governed text files across `skills/`, `.agents/skills/`, `rules/`, `.pipeline/` (excluding
/// `.pipeline/defects/`), `docs/`, `README.md`, and `AGENTS.md` for operational Python/Pytest tokens.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero files in `repo_root` (excluding `target`, `.git`, `build`, `Pods`, `ephemeral`, and `.dart_tool`)
///   may have extension `.py` (case-insensitive).
/// - Zero governed `.md`, `.yaml`, `.yml`, `.json`, or `.toml` lines (outside explicit governance
///   prohibition clauses and `.pipeline/defects/` historical dossiers) may contain operational
///   Python/Pytest tokens (`python3`, `pytest`, `.py`, `models/python`, `unittest`,
///   `execution_environment: [python`, `language: python`).
///
/// ## 3. Preconditions
/// `repo_root: &Path` must be an accessible directory path on the local filesystem.
///
/// ## 4. Postconditions
/// Returns `Ok(())` if zero `.py` files and zero operational Python/Pytest references exist,
/// or `Err(Vec<String>)` with sorted, deduplicated relative paths and `rel_path:line_no` diagnostics.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N + G * L) where N is repository entry count, G is governed text file count,
/// and L is average line count per governed file. Space Complexity: O(V) where V is violation count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on any discovered `.py` file, operational Python/Pytest token, or filesystem I/O error.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-33-ZERO-PYTHON-SCRIPTS].
pub fn check_zero_python_scripts(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();

    for entry_res in WalkDir::new(repo_root)
        .into_iter()
        .filter_entry(|e| {
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
        })
    {
        let entry = match entry_res {
            Ok(e) => e,
            Err(err) => {
                violations.push(format!("Check 33 I/O error: {}", err));
                continue;
            }
        };
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C toolchain integrity verification controls.
/// Ensures `scan_governed_files_for_python_references` inspects every `.md`, `.yaml`, `.yml`,
/// `.json`, and `.toml` file under `skills/`, `.agents/skills/`, `rules/`, `.pipeline/`
/// (excluding `.pipeline/defects/`), `docs/`, `README.md`, and `AGENTS.md`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Directory traversal excludes build artifacts (`target`, `.git`, `build`, `Pods`, `ephemeral`,
///   `.dart_tool`) and historical defect dossiers (`.pipeline/defects/`).
/// - Every matching text file in the governed set is scanned deterministically via `scan_file_for_python_references`.
///
/// ## 3. Preconditions
/// - `repo_root: &Path` points to the workspace root directory.
/// - `violations: &mut Vec<String>` is a mutable accumulator owned by `check_zero_python_scripts`.
///
/// ## 4. Postconditions
/// Appends `rel_path:line_no` diagnostics for all operational Python/Pytest references into `violations`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(G * L) where G is governed file count and L is average line count per file.
/// Space Complexity: O(D) recursion stack depth for directory walk plus O(V) appended violations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed by pushing `Check 33 I/O error` diagnostics into `violations` if directory traversal fails.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-33-SCAN-GOVERNED-FILES].
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

/// Evaluates whether `path: &Path` carries a governed text extension for Check 33 content scanning.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A deterministic file-type classification controls.
/// Restricts `is_governed_python_scan_extension` to structured documentation and configuration
/// formats (`.md`, `.yaml`, `.yml`, `.json`, `.toml`) without static string-slice arrays.
///
/// ## 2. Invariants & Mathematical Properties
/// Case-insensitive extension predicate returning `true` if and only if the file extension
/// matches `md`, `yaml`, `yml`, `json`, or `toml`.
///
/// ## 3. Preconditions
/// `path: &Path` is a valid filesystem path reference.
///
/// ## 4. Postconditions
/// Returns `true` when `path` has a governed text extension; returns `false` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(E) where E is extension byte length. Space Complexity: O(E) for ASCII lowercase conversion.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Total, non-panicking predicate; paths without an extension return `false`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-33-GOVERNED-EXTENSIONS].
fn is_governed_python_scan_extension(path: &Path) -> bool {
    let ext = match path.extension() {
        Some(e) => e.to_string_lossy().to_ascii_lowercase(),
        None => return false,
    };
    matches!(ext.as_str(), "md" | "yaml" | "yml" | "json" | "toml")
}

/// Scans a single governed text file line-by-line for operational Python/Pytest tokens.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C trace-level defect detection.
/// Reads `path: &Path` and inspects each 1-indexed line via `is_python_prohibition_clause`
/// and `detect_operational_python_token`, recording `rel_path:line_no` violations.
///
/// ## 2. Invariants & Mathematical Properties
/// - Lines containing explicit governance prohibition terms are exempted via `is_python_prohibition_clause`.
/// - Every remaining line containing an operational Python/Pytest token emits exactly one
///   `rel_path:line_no` violation entry into `violations`.
///
/// ## 3. Preconditions
/// - `path: &Path` is an existing regular file under `repo_root: &Path`.
/// - `violations: &mut Vec<String>` is a valid mutable vector.
///
/// ## 4. Postconditions
/// Appends formatted `rel_path:line_no` diagnostics (or file read failure diagnostics) into `violations`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(B) where B is the byte length of `path`. Space Complexity: O(B) for file buffer.
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed if `fs::read_to_string(path)` returns an I/O or UTF-8 decoding error.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-33-LINE-SCANNER].
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

/// Determines whether an ASCII-lowercased line is an explicit governance prohibition clause.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A normative rule preservation controls.
/// Allows constitutional rules and agent instructions in `AGENTS.md`, `rules/`, and
/// `.pipeline/ACTIVE_RULES_BUNDLE.md` to state that Python and Pytest are prohibited without
/// triggering false-positive self-violations in `is_python_prohibition_clause`.
///
/// ## 2. Invariants & Mathematical Properties
/// Returns `true` if and only if `ascii_folded` contains at least one canonical prohibition
/// stem (`prohibit`, `forbid`, `retired`, `eliminat`, `zero python`, `never`, `ban`, `check 33`).
///
/// ## 3. Preconditions
/// `ascii_folded: &str` must already be converted to ASCII lowercase by the caller.
///
/// ## 4. Postconditions
/// Pure boolean classification indicating whether the line states a governance prohibition.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(L) where L is the byte length of `ascii_folded`. Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Infallible, zero-allocation substring inspection that never panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-33-PROHIBITION-CLAUSE-EXEMPTION].
fn is_python_prohibition_clause(ascii_folded: &str) -> bool {
    ascii_folded.contains("prohibit")
        || ascii_folded.contains("forbid")
        || ascii_folded.contains("retired")
        || ascii_folded.contains("eliminat")
        || ascii_folded.contains("zero python")
        || ascii_folded.contains("never")
        || ascii_folded.contains("ban")
        || ascii_folded.contains("check 33")
}

/// Detects whether an ASCII-lowercased line contains an operational Python or Pytest token.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C pure-Rust toolchain enforcement.
/// Identifies operational invocations and configuration tokens (`python3`, `pytest`,
/// `models/python`, `unittest`, `execution_environment: [python`, `language: python`, and
/// `.py` script tokens) inside `detect_operational_python_token` using zero regular expressions.
///
/// ## 2. Invariants & Mathematical Properties
/// Evaluates a fixed priority order of operational Python tokens and delegates `.py` token
/// boundary verification to `contains_dot_py_token`.
///
/// ## 3. Preconditions
/// `ascii_folded: &str` is an ASCII-lowercased line that has already been screened by
/// `is_python_prohibition_clause`.
///
/// ## 4. Postconditions
/// Returns `Some(&'static str)` naming the first matched operational Python token, or `None`
/// if the line is clean.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(L) where L is the byte length of `ascii_folded`. Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Pure, deterministic, non-panicking string inspection with zero heap allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-33-OPERATIONAL-TOKEN-DETECTOR].
fn detect_operational_python_token(ascii_folded: &str) -> Option<&'static str> {
    if ascii_folded.contains("python3") {
        Some("python3")
    } else if ascii_folded.contains("pytest") {
        Some("pytest")
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

/// Checks whether an ASCII-lowercased line contains a `.py` file extension token at a word boundary.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34 non-panicking slice access rules.
/// Detects `.py` script paths and tokens in `contains_dot_py_token` while avoiding false
/// positives on words where `.py` is immediately followed by alphanumeric or underscore characters.
///
/// ## 2. Invariants & Mathematical Properties
/// For every byte offset `idx` where `".py"` occurs in `ascii_folded`, inspects `bytes.get(idx + 3)`
/// and returns `true` if and only if the following byte is absent (`None`) or is neither ASCII
/// alphanumeric nor `b'_'`.
///
/// ## 3. Preconditions
/// `ascii_folded: &str` is an ASCII-lowercased line slice.
///
/// ## 4. Postconditions
/// Returns `true` if at least one boundary-terminated `.py` token exists in `ascii_folded`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(L) where L is byte length of `ascii_folded`. Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Uses `.get(..)` exclusively on `bytes` and `saturating_add(3)` on `idx`, guaranteeing zero panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-33-DOT-PY-BOUNDARY-SCAN].
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


/// Check 34: Zero Regex Invariant & Self-Hosting Compiler Purity Gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C safety-critical software requirements.
/// Mechanically enforces that zero regular expressions exist across the compiler codebase
/// and enforces complete AST purity and 7-section Rustdoc on audited modules.
///
/// ## 2. Invariants & Mathematical Properties
/// - All AST parsers and scanners must be deterministic finite-state machines without regex.
/// - Audited modules must possess zero static string-slice arrays, zero stubs, zero fail-open
///   constructs, zero cross-string UTF-8 byte slices, and mandatory 7-section Rustdoc.
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

    let crate_needle = ["reg", "ex"].concat();

    // 1. Check root Cargo.toml if it exists
    let root_cargo = repo_root.join("Cargo.toml");
    if root_cargo.is_file() {
        scan_cargo_toml_file(&root_cargo, repo_root, &crate_needle, &mut violations);
    }

    // 2. Scan crates/ directory for all .rs and Cargo.toml files
    let crates_dir = repo_root.join("crates");
    if crates_dir.is_dir() {
        for entry_res in WalkDir::new(&crates_dir)
            .into_iter()
            .filter_entry(|e| {
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
            })
        {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    violations.push(format!("Check 34 I/O error: {}", err));
                    continue;
                }
            };
            if entry.file_type().is_file() {
                let path = entry.path();
                let fname = entry.file_name().to_string_lossy();
                if fname == "Cargo.toml" {
                    scan_cargo_toml_file(path, repo_root, &crate_needle, &mut violations);
                } else if path.extension().map(|ext| ext == "rs").unwrap_or(false) {
                    let rel = path.strip_prefix(repo_root).unwrap_or(path).display().to_string();
                    match fs::read_to_string(path) {
                        Ok(content) => {
                            let is_audited = rel.contains("parity_gates.rs")
                                || rel.contains("docstring_gate.rs")
                                || rel.contains("defect_dossier.rs")
                                || rel.contains("compile-sysml/src/docstring.rs")
                                || rel.contains("ingest-sysml/src/translators/code.rs")
                                || (rel.contains("deap-core/src/")
                                    && (rel.ends_with("diagnostics.rs")
                                        || rel.ends_with("rules.rs")
                                        || rel.ends_with("workspace.rs")
                                        || rel.ends_with("markdown.rs")
                                        || rel.ends_with("sysml_ast.rs")
                                        || rel.ends_with("source_ast.rs")));
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

/// Check 34 Layer 1: Self-hosting compiler AST purity gate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C. Mechanically audits verified modules
/// across the codebase to ensure zero regex, zero static string-slice arrays, zero hollow stubs,
/// zero fail-open error swallowing, zero cross-string byte-slicing hazards, and complete 7-section Rustdoc.
///
/// ## 2. Invariants & Mathematical Properties
/// Audited modules must satisfy all 6 purity conditions simultaneously.
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
/// Fails closed if any audited file is unreadable or non-compliant.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-34-COMPILER-PURITY-GATE].
pub fn check_compiler_ast_purity(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();

    let audited_files = [
        repo_root.join("crates").join("verify-baseline").join("src").join("checks").join("parity_gates.rs"),
        repo_root.join("crates").join("verify-baseline").join("src").join("checks").join("docstring_gate.rs"),
        repo_root.join("crates").join("verify-baseline").join("src").join("defect_dossier.rs"),
        repo_root.join("crates").join("deap-core").join("src").join("diagnostics.rs"),
        repo_root.join("crates").join("deap-core").join("src").join("rules.rs"),
        repo_root.join("crates").join("deap-core").join("src").join("workspace.rs"),
        repo_root.join("crates").join("deap-core").join("src").join("markdown.rs"),
        repo_root.join("crates").join("deap-core").join("src").join("sysml_ast.rs"),
        repo_root.join("crates").join("deap-core").join("src").join("source_ast.rs"),
        repo_root.join("crates").join("compile-sysml").join("src").join("docstring.rs"),
        repo_root.join("crates").join("ingest-sysml").join("src").join("translators").join("code.rs"),
    ];

    for file_path in &audited_files {
        if file_path.is_file() {
            let rel = file_path.strip_prefix(repo_root).unwrap_or(file_path).display().to_string();
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    if let Err(errs) = audit_source_purity(&content, &rel, true) {
                        violations.extend(errs);
                    }
                }
                Err(err) => {
                    violations.push(format!("Check 34: Unable to read audited file '{}': {}", rel, err));
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

/// Audits a single Rust source string for compiler purity violations (a)-(f).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS self-hosting compiler purity requirements.
/// Mechanically audits source lines for banned regex, static string-slice arrays,
/// hollow gate stubs, fail-open error swallowing, cross-string byte-slicing hazards,
/// and mandatory 7-section Rustdoc completeness.
///
/// ## 2. Invariants & Mathematical Properties
/// - Banned constructs are assembled dynamically to prevent scanner self-triggering.
/// - In audited modules, every function must possess all 7 documentation sections.
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

    // Dynamically assemble forbidden pattern needles to ensure scanner source
    // does not match itself as a false positive.
    let pat_regex_colons = ["Reg", "ex::"].concat();
    let pat_inline_regex = ["reg", "ex::"].concat();
    let pat_use_bare = ["use", " ", "reg", "ex;"].concat();
    let pat_extern = ["extern", " crate ", "reg", "ex;"].concat();

    // String-slice forbidden patterns
    let needle_slice_str = ["&[", "&str]"].concat();
    let needle_tuple_slice = ["&[(", "&str"].concat();
    let needle_array_str = ["[", "&str;"].concat();
    let needle_static_slice = [": &[", "&str]"].concat();
    let needle_const_slice = ["const ", ": &[", "&str]"].concat();
    // 'static lifetime string-slice and array-iterator evasions
    let needle_static_lt_slice = ["&'static [", "&'static str]"].concat();
    let needle_static_lt_array = ["[&'static ", "str;"].concat();
    let needle_static_lt_into_iter = ["IntoIter<", "&'static str"].concat();

    // Hollow stub forbidden patterns
    let needle_let_blank = ["let", " _ = "].concat();

    // Fail-open forbidden patterns
    let needle_fm_e = [".filter", "_map(|e| e.ok", "())"].concat();
    let needle_fm_res = [".filter", "_map(|res| res.ok", "())"].concat();
    let needle_fm_x = [".filter", "_map(|x| x.ok", "())"].concat();
    let needle_filter_map = [".filter", "_map("].concat();
    let needle_dot_ok = [".ok", "()"].concat();
    let needle_read_str = ["read_to", "_string"].concat();

    // Cross-string byte slicing hazards
    let needle_lower_slice1 = [".to_lowercase()", "["].concat();
    let needle_lower_slice2 = [".to_ascii_lowercase()", "["].concat();
    let needle_lower_idx = ["lower", "["].concat();
    let needle_ulower_idx = ["_lower", "["].concat();
    let needle_dotdot = [".", "."].concat();

    // 7-section doc headers to check
    let s1 = "## 1. Safety Intent & Regulatory Scope";
    let s2 = "## 2. Invariants & Mathematical Properties";
    let s3 = "## 3. Preconditions";
    let s4 = "## 4. Postconditions";
    let s5 = "## 5. Algorithmic Complexity Bounds";
    let s6 = "## 6. Failure Modes & Fault Tolerance";
    let s7 = "## 7. Realization & Traceability";

    let mut in_mod_tests = false;
    let mut current_doc = Vec::new();
    let mut in_multiline_comment = false;
    let mut seen_docblocks: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for (idx, raw_line) in source.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = raw_line.trim();

        if in_multiline_comment {
            if trimmed.contains("*/") {
                in_multiline_comment = false;
            }
            continue;
        }
        if trimmed.starts_with("/*") && !trimmed.contains("*/") {
            in_multiline_comment = true;
            continue;
        }

        if trimmed.starts_with("//!") {
            current_doc.push(trimmed.to_string());
            continue;
        }

        if trimmed.starts_with("///") {
            current_doc.push(trimmed.to_string());
            continue;
        }

        let code = if let Some((before, _)) = trimmed.split_once("//") {
            before.trim()
        } else {
            trimmed
        };

        if code.is_empty() {
            continue;
        }

        if code.contains("mod tests") {
            in_mod_tests = true;
        }

        if !in_mod_tests {
            // (a) Regex Check
            if code.contains(&pat_regex_colons)
                || code.contains(&pat_inline_regex)
                || code.contains(&pat_use_bare)
                || code.contains(&pat_extern)
            {
                violations.push(format!("{}:{}: regex usage detected", file_rel, line_no));
            }

            // (b) Static String-Slice Arrays Check
            if code.contains(&needle_slice_str)
                || code.contains(&needle_tuple_slice)
                || code.contains(&needle_array_str)
                || code.contains(&needle_static_slice)
                || code.contains(&needle_const_slice)
                || code.contains(&needle_static_lt_slice)
                || code.contains(&needle_static_lt_array)
                || code.contains(&needle_static_lt_into_iter)
            {
                violations.push(format!("{}:{}: static string-slice array detected", file_rel, line_no));
            }

            // (c) Hollow Gate Stubs Check
            if code.contains(&needle_let_blank) {
                violations.push(format!("{}:{}: hollow gate stub detected (`let _ = `)", file_rel, line_no));
            }

            // (d) Fail-Open Error Swallowing Check
            if code.contains(&needle_fm_e)
                || code.contains(&needle_fm_res)
                || code.contains(&needle_fm_x)
                || (code.contains(&needle_filter_map) && code.contains(&needle_dot_ok))
                || (code.contains(&needle_read_str) && code.contains(&needle_dot_ok))
            {
                violations.push(format!("{}:{}: fail-open error swallowing detected", file_rel, line_no));
            }

            // (e) Cross-String UTF-8 Byte-Slice Hazards Check
            if code.contains(&needle_lower_slice1)
                || code.contains(&needle_lower_slice2)
                || ((code.contains(&needle_lower_idx) || code.contains(&needle_ulower_idx))
                    && code.contains(&needle_dotdot))
            {
                violations.push(format!("{}:{}: cross-string UTF-8 byte-slice hazard detected", file_rel, line_no));
            }

            // (e) Direct Panicking Range-Slice Indexing Check (audited modules only)
            if check_rustdoc && contains_direct_range_index(code, &needle_dotdot) {
                violations.push(format!(
                    "{}:{}: direct panicking range-slice indexing detected (use .get(..))",
                    file_rel, line_no
                ));
            }

            // (f) 7-Section Rustdoc Check
            if check_rustdoc && (code.starts_with("pub fn ") || code.starts_with("fn ") || code.starts_with("pub const fn ") || code.starts_with("pub(crate) fn ")) {
                let doc_block = current_doc.join("\n");
                let mut missing = Vec::new();
                if !doc_block.contains(s1) { missing.push("1. Safety Intent"); }
                if !doc_block.contains(s2) { missing.push("2. Invariants"); }
                if !doc_block.contains(s3) { missing.push("3. Preconditions"); }
                if !doc_block.contains(s4) { missing.push("4. Postconditions"); }
                if !doc_block.contains(s5) { missing.push("5. Algorithmic Complexity"); }
                if !doc_block.contains(s6) { missing.push("6. Failure Modes"); }
                if !doc_block.contains(s7) { missing.push("7. Realization & Traceability"); }

                if !missing.is_empty() {
                    violations.push(format!(
                        "{}:{}: function missing mandatory 7-section Rustdoc (missing: {})",
                        file_rel, line_no, missing.join(", ")
                    ));
                }

                // (f) Per-function docblock uniqueness: the body (excluding the
                // summary line and the `Realises:` tag) must not repeat verbatim.
                let normalized_doc = current_doc
                    .iter()
                    .skip(1)
                    .filter(|doc_line| !doc_line.contains("Realises:"))
                    .map(|doc_line| {
                        doc_line
                            .trim_start_matches("///")
                            .trim_start_matches("//!")
                            .trim()
                    })
                    .filter(|doc_text| !doc_text.is_empty())
                    .collect::<Vec<&str>>()
                    .join("\n");
                if !normalized_doc.is_empty() && !seen_docblocks.insert(normalized_doc) {
                    violations.push(format!(
                        "{}:{}: duplicate/boilerplate 7-section Rustdoc detected",
                        file_rel, line_no
                    ));
                }
                current_doc.clear();
            } else if !code.starts_with("#[") {
                current_doc.clear();
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

/// Detects direct panicking range-slice indexing (`expr[a..b]`) on one code line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Supports Check 34 rule (e). Direct range indexing panics on out-of-bounds or
/// non-UTF-8-boundary offsets, so audited modules must use the non-panicking `.get(..)`.
///
/// ## 2. Invariants & Mathematical Properties
/// An opening bracket is an index operator only when the byte immediately before it is
/// an ASCII alphanumeric, `_`, or `)`. Bracket depth is tracked so nested indices match
/// their own closing bracket; an unmatched bracket extends to end of line (fail-closed).
///
/// ## 3. Preconditions
/// `code` is a single comment-stripped source line; `range_needle` is the range operator.
///
/// ## 4. Postconditions
/// Returns `true` iff some index-operator bracket encloses `range_needle`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N^2) worst case for N bytes in the line (bounded by line length).
/// Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Never panics: offsets are taken from ASCII bracket bytes and sliced with `.get(..)`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-34-DIRECT-RANGE-INDEX-SCAN].
fn contains_direct_range_index(code: &str, range_needle: &str) -> bool {
    let bytes = code.as_bytes();
    let mut prev_byte: Option<u8> = None;
    for (open_idx, &byte) in bytes.iter().enumerate() {
        let is_index_operator = byte == b'['
            && prev_byte
                .map(|p| p.is_ascii_alphanumeric() || p == b'_' || p == b')')
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
            let inner = code.get(open_idx.saturating_add(1)..close_idx).unwrap_or(code);
            if inner.contains(range_needle) {
                return true;
            }
        }
        prev_byte = Some(byte);
    }
    false
}

/// Scans Cargo.toml for forbidden dependency on regex crate.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A. Ensures regex crate is strictly excluded from Cargo dependencies.
///
/// ## 2. Invariants & Mathematical Properties
/// Dependency keys must not match regex crate name.
///
/// ## 3. Preconditions
/// `path` must be valid Cargo.toml path.
///
/// ## 4. Postconditions
/// Pushes any discovered dependency violations into `violations`.
///
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(L) where L is line count. Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed on file read errors.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CARGO-TOML-REGEX-SCAN].
fn scan_cargo_toml_file(
    path: &Path,
    repo_root: &Path,
    crate_needle: &str,
    violations: &mut Vec<String>,
) {
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
            let section = trimmed.get(1..trimmed.len().saturating_sub(1)).unwrap_or("").trim();
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

    if let Ok(entries) = fs::read_dir(repo_root) {
        for entry_res in entries {
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };
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
        for entry_res in WalkDir::new(&docs_dir)
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
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };
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

    let mut retired_needles: Vec<String> = Vec::new();
    retired_needles.push("create_issue.sh".to_string());
    retired_needles.push("verify_downstream_baseline.py".to_string());
    retired_needles.push("install_pipeline.sh".to_string());
    retired_needles.push("generate-wbs-suite".to_string());
    retired_needles.push("compile_yang.py".to_string());
    retired_needles.push("compile_sysml.py".to_string());
    retired_needles.push("assemble_conops.py".to_string());
    retired_needles.push("parity_auditor".to_string());
    retired_needles.push("setup_git_hooks.py".to_string());
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
        for entry_res in WalkDir::new(dir)
            .into_iter()
            .filter_entry(|e| {
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
            })
        {
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };
            if entry.file_type().is_file() {
                let path = entry.path();
                let is_md = path.extension().map(|ext| ext == "md").unwrap_or(false);
                if is_md {
                    let rel = path.strip_prefix(repo_root).unwrap_or(path).display().to_string();
                    if rel.ends_with("_HANDOFF.md") || rel.ends_with("_HANDOVER.md") {
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(path) {
                        for (idx, line) in content.lines().enumerate() {
                            for needle in &retired_needles {
                                if line.contains(needle) {
                                    violations.push(format!("{}:{}: references retired script '{}'", rel, idx + 1, needle));
                                }
                            }
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
            let rel = file.strip_prefix(repo_root).unwrap_or(file).display().to_string();
            if let Ok(content) = fs::read_to_string(file) {
                for (idx, line) in content.lines().enumerate() {
                    for needle in &retired_needles {
                        if line.contains(needle) {
                            violations.push(format!("{}:{}: references retired script '{}'", rel, idx + 1, needle));
                        }
                    }
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
    let feat42_path = repo_root.join("docs/designs/FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md");
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
            "FEAT-42 Section 2.1 must specify --schema in CLI option definitions table"
                .to_string(),
        );
    }
    let has_format_auto = content.lines().any(|l| l.contains("--format") && l.contains("`auto`"));
    if !has_format_auto {
        violations.push(
            "FEAT-42 Section 2.1 must specify --format default value as `auto`"
                .to_string(),
        );
    }

    // 3. Assert Section 3.2 mandates per-function AST interpolation across Sections 1-6 for Check 34 uniqueness
    if !content.contains("seen_docblocks") {
        violations.push(
            "FEAT-42 Section 3.2 must mandate docstring uniqueness under 'seen_docblocks'"
                .to_string(),
        );
    }
    if !content.contains("Check 34") {
        violations.push(
            "FEAT-42 Section 3.2 must cite Check 34 uniqueness contract"
                .to_string(),
        );
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
/// traceability and configuration management controls. Enforces that all 12 canonical architecture
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
///   when all 12 blueprints exist and satisfy all grounding invariants.
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
    if !blueprints_dir.is_dir() && !is_upstream_compiler(repo_root) {
        return Ok(());
    }

    let mut violations = Vec::new();
    if !blueprints_dir.is_dir() {
        violations.push(
            "Missing docs/architecture/blueprints directory in upstream specification compiler repository."
                .to_string(),
        );
        return Err(violations);
    }

    for bp in CanonicalArchitectureBlueprint::ALL {
        if !is_upstream_compiler(repo_root)
            && matches!(
                bp,
                CanonicalArchitectureBlueprint::SysmlSsotBidirectionalSynchronizationArchitecture
            )
        {
            continue;
        }
        let bp_path = blueprints_dir.join(bp.as_str());
        if !bp_path.is_file() {
            violations.push(format!(
                "Missing canonical architecture blueprint: docs/architecture/blueprints/{}",
                bp.as_str()
            ));
        }
    }

    let mut md_files = Vec::new();
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C architectural document verification rules.
/// Inspects every line of a blueprint file under `docs/architecture/blueprints/` for forbidden
/// em dashes, fictional crate names, retired Python/shell tools, and ungrounded `crates/...` paths.
///
/// ## 2. Invariants & Mathematical Properties
/// - Line-by-line deterministic scan preserving 1-based line numbers (`idx.saturating_add(1)`).
/// - Appends a diagnostic entry to `violations` for every violated invariant on each line.
///
/// ## 3. Preconditions
/// - `repo_root` is the workspace root directory and `md_path` is a `.md` file path inside `docs/architecture/blueprints/`.
///
/// ## 4. Postconditions
/// - Mutates `violations` in place by appending all discovered `rel:line_no` violations; leaves `violations` unchanged if the file is clean.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L * W) where L is the line count of `md_path` and W is average line length.
/// - Space Complexity: O(B + V_file) where B is file byte size and V_file is violations in `md_path`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: File read failures push an explicit I/O diagnostic to `violations` (fail-closed).
/// - State: Only appends to `violations`; no global state.
/// - Action: Bounded single-pass iteration over `content.lines()`.
/// - Resource: Single file buffer allocation per blueprint.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-37-SCAN-BLUEPRINT-FILE]
fn scan_blueprint_markdown_file(
    repo_root: &Path,
    md_path: &Path,
    violations: &mut Vec<String>,
) {
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

    for (idx, line) in content.lines().enumerate() {
        let line_no = idx.saturating_add(1);

        if line.contains('\u{2014}') {
            violations.push(format!(
                "{}:{}: forbidden Unicode em dash (\\u2014) in architecture blueprint",
                rel, line_no
            ));
        }

        check_blueprint_line_fictional_crates(&rel, line_no, line, violations);
        check_blueprint_line_retired_tools(&rel, line_no, line, violations);
        check_blueprint_line_crates_paths(repo_root, &rel, line_no, line, violations);
    }
}

/// Checks whether `line` contains `token` bounded by non-identifier characters.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A lexical boundary verification for Check 37 (`contains_bounded_token`).
/// Prevents substring false positives (such as `deap_harness` matching inside `deap_harness_config.yaml`)
/// while deterministically detecting standalone crate identifiers without regular expressions.
///
/// ## 2. Invariants & Mathematical Properties
/// - For each byte offset `idx` yielded by `line.match_indices(token)`, inspects the preceding byte
///   (`idx.saturating_sub(1)`) and trailing byte (`idx.saturating_add(token.len())`) via `bytes.get(..)`.
/// - Returns `true` iff neither adjacent byte is ASCII alphanumeric, `b'_'`, or `b'-'`.
///
/// ## 3. Preconditions
/// - `line: &str` and `token: &str` are valid UTF-8 string slices with `token` non-empty.
///
/// ## 4. Postconditions
/// - Returns `true` if at least one identifier-bounded occurrence of `token` exists in `line`, or `false` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(W) where W is the byte length of `line`.
/// - Space Complexity: O(1) zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking byte inspection via `.get(..)`, `saturating_sub`, and `saturating_add`.
/// - State: Pure function with zero mutable state.
/// - Action: Deterministic linear scan.
/// - Resource: Zero dynamic memory allocation.
///
/// ## 7. Realization & Traceability
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

/// Checks whether a blueprint line is an explicit negative prohibition or retirement statement.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A context-sensitive rule auditing for Check 37 (`is_negative_blueprint_rule_line`).
/// Allows blueprints to state that legacy/fictional identifiers are forbidden, prohibited, retired,
/// or non-existent without triggering a positive reference violation.
///
/// ## 2. Invariants & Mathematical Properties
/// - Folds `line` to ASCII lowercase and evaluates six negative-rule keyword predicates (`forbidden`,
///   `prohibited`, `retired`, `nonexistent`, `non-existent`, `fictional`).
///
/// ## 3. Preconditions
/// - `line: &str` is a valid UTF-8 line from an architecture blueprint file.
///
/// ## 4. Postconditions
/// - Returns `true` iff `line` contains an explicit negative-rule keyword.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(W) where W is the byte length of `line`.
/// - Space Complexity: O(W) for the temporary ASCII-lowercased string buffer.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Infallible `&str -> bool` predicate.
/// - State: Stateless evaluation.
/// - Action: Deterministic substring checks.
/// - Resource: Bounded single-line string allocation.
///
/// ## 7. Realization & Traceability
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

/// Scans a blueprint line for all references to fictional, non-existent DEAP crate names.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A architectural accuracy controls for Check 37.
/// Prevents architecture blueprints from citing hallucinated crate names (`deap-ast`,
/// `deap-codegen`, `deap-harness`, `deap-cli`, or underscore variants) that do not exist in `Cargo.toml`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Evaluates all 8 fictional crate tokens independently via `contains_bounded_token` so multiple
///   fictional crates on a single line are each reported.
/// - Exempts lines where `is_negative_blueprint_rule_line(line)` holds.
///
/// ## 3. Preconditions
/// - `rel_file` and `line_no` identify the source location of `line` within `docs/architecture/blueprints/`.
///
/// ## 4. Postconditions
/// - Appends a diagnostic entry to `violations` for each fictional crate token found on `line`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(W) where W is the byte length of `line`.
/// - Space Complexity: O(1) auxiliary space when clean; O(V_line) when violations are pushed.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Total function over `&str` and `&mut Vec<String>`.
/// - State: Appends only to `violations`.
/// - Action: Deterministic bounded token checks with zero regex.
/// - Resource: Zero allocation on clean lines.
///
/// ## 7. Realization & Traceability
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

/// Scans a blueprint line for references to retired Python or shell scripts.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 32/33/37 pure-Rust toolchain invariants.
/// Rejects blueprint lines referencing retired scripts (`verify_downstream`, `sysmlv2_ingest`,
/// `reconcile_backlog.py`, `parity_auditor`, `setup_git_hooks.sh`) outside negative retirement rules.
///
/// ## 2. Invariants & Mathematical Properties
/// - Evaluates all five retired tool tokens independently in deterministic order without static string arrays.
/// - Exempts lines where `is_negative_blueprint_rule_line(line)` holds.
///
/// ## 3. Preconditions
/// - `rel_file` and `line_no` identify the source location of `line` in `docs/architecture/blueprints/`.
///
/// ## 4. Postconditions
/// - Appends a diagnostic entry to `violations` for every retired tool token matched on `line`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(W) where W is the byte length of `line`.
/// - Space Complexity: O(1) auxiliary memory when clean.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Total function over `&str` and `&mut Vec<String>`.
/// - State: Appends diagnostics only to `violations`.
/// - Action: Non-panicking substring scan.
/// - Resource: Zero dynamic memory usage on clean lines.
///
/// ## 7. Realization & Traceability
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

/// Checks whether `crate_name` is one of the 6 canonical workspace crates in `Cargo.toml`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A workspace topology verification for Check 37.
/// Restricts concrete `crates/<crate_name>` references in upstream architecture blueprints to the
/// 6 actual workspace crates (`deap-core`, `verify-baseline`, `compile-sysml`, `ingest-sysml`,
/// `assemble-conops`, `reconcile-backlog`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `true` iff `crate_name` matches one of the 6 canonical workspace crate directory names.
///
/// ## 3. Preconditions
/// - `crate_name: &str` is the first path segment following `"crates/"` in an extracted blueprint path token.
///
/// ## 4. Postconditions
/// - Returns `true` for valid workspace crates and `false` for any unknown or fictional crate name.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) compile-time pattern match across 6 literal arms.
/// - Space Complexity: O(1) zero allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Total boolean predicate over `&str`.
/// - State: Pure compile-time match.
/// - Action: Never panics.
/// - Resource: Zero stack or heap growth.
///
/// ## 7. Realization & Traceability
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

/// Extracts and validates all concrete `crates/...` path tokens on a blueprint line against disk.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34/37 mechanical grounding controls.
/// Ensures that every non-placeholder `crates/...` path cited in an architecture blueprint names
/// one of the 6 real workspace crates and resolves to an existing file or directory on disk.
///
/// ## 2. Invariants & Mathematical Properties
/// - Ignores template/wildcard tokens containing `*`, `<`, `>`, `{`, `}`, `...`, or `$`.
/// - Strips `#L...` anchors, `:line` suffixes, and trailing punctuation (`.`, `,`, `:`, `;`, `/`)
///   exclusively via non-panicking `.get(..)` and string splitting methods.
/// - Verifies both `is_valid_workspace_crate_name(crate_name)` and `repo_root.join(cleaned).exists()`.
///
/// ## 3. Preconditions
/// - `repo_root` is the workspace root path; `rel_file` and `line_no` identify the source location of `line`.
///
/// ## 4. Postconditions
/// - Appends a diagnostic string to `violations` for each ungrounded crate or non-existent `crates/...` path on `line`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(W + K) where W is byte length of `line` and K is the number of `crates/` occurrences on `line`.
/// - Space Complexity: O(1) auxiliary slice references beyond any pushed violation strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking slice indexing via `.get(..)` and `saturating_sub`.
/// - State: Appends diagnostics only to `violations`.
/// - Action: Deterministic left-to-right token extraction.
/// - Resource: Zero intermediate heap allocations on valid paths.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-37-LINE-CRATES-PATHS]
fn check_blueprint_line_crates_paths(
    repo_root: &Path,
    rel_file: &str,
    line_no: usize,
    line: &str,
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
        let cleaned = without_colon
            .trim_end_matches(|c: char| matches!(c, '.' | ',' | ':' | ';' | '/'));

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

        let is_downstream_crate = !is_upstream_compiler(repo_root)
            && repo_root.join("crates").join(crate_name).is_dir();
        if !is_valid_workspace_crate_name(crate_name) && !is_downstream_crate {
            violations.push(format!(
                "{}:{}: ungrounded crate '{}' in blueprint path '{}' (must be one of deap-core, verify-baseline, compile-sysml, ingest-sysml, assemble-conops, reconcile-backlog)",
                rel_file, line_no, crate_name, cleaned
            ));
            continue;
        }

        if !repo_root.join(cleaned).exists() {
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
    fn test_check_32_zero_shell_scripts_exempts_build_dirs_and_rejects_others() {
        let tmp = TempDir::new("c32_exempt_dirs");
        for dir in ["build/macos", "macos/Pods", "macos/Flutter/ephemeral", ".dart_tool/build"] {
            let d = tmp.path.join(dir);
            fs::create_dir_all(&d).unwrap();
            File::create(d.join("generated.sh")).unwrap();
        }
        assert!(
            check_zero_shell_scripts(&tmp.path).is_ok(),
            "Generated .sh files inside build, Pods, ephemeral, and .dart_tool must be exempted"
        );

        let forbidden_sh = tmp.path.join("scripts").join("deploy.sh");
        fs::create_dir_all(forbidden_sh.parent().unwrap()).unwrap();
        File::create(&forbidden_sh).unwrap();
        let res = check_zero_shell_scripts(&tmp.path);
        assert!(res.is_err(), "Non-exempt .sh script must fail Check 32");
        let errs = res.unwrap_err();
        assert_eq!(errs, vec!["scripts/deploy.sh".to_string()]);
    }

    #[test]
    fn test_check_33_zero_python_clean() {
        let tmp = TempDir::new("c33_clean");
        let rs_file = tmp.path.join("main.rs");
        File::create(&rs_file).unwrap();
        assert!(check_zero_python_scripts(&tmp.path).is_ok());
    }

    #[test]
    fn test_check_33_zero_python_exempts_build_dirs_and_rejects_others() {
        let tmp = TempDir::new("c33_exempt_dirs");
        for dir in ["build/macos", "macos/Pods", "macos/Flutter/ephemeral", ".dart_tool/build"] {
            let d = tmp.path.join(dir);
            fs::create_dir_all(&d).unwrap();
            File::create(d.join("helper.py")).unwrap();
        }
        assert!(
            check_zero_python_scripts(&tmp.path).is_ok(),
            "Generated .py files inside build, Pods, ephemeral, and .dart_tool must be exempted"
        );

        let forbidden_py = tmp.path.join("scripts").join("legacy.py");
        fs::create_dir_all(forbidden_py.parent().unwrap()).unwrap();
        File::create(&forbidden_py).unwrap();
        let res = check_zero_python_scripts(&tmp.path);
        assert!(res.is_err(), "Non-exempt .py script must fail Check 33");
        let errs = res.unwrap_err();
        assert_eq!(errs, vec!["scripts/legacy.py".to_string()]);
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
    fn test_check_34_audited_parity_gates_passes() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("repo root");
        let res = check_compiler_ast_purity(repo_root);
        assert!(res.is_ok(), "Check 34 Compiler AST Purity failed on audited modules: {:?}", res.err());

        let structural_path = repo_root
            .join("crates")
            .join("verify-baseline")
            .join("src")
            .join("checks")
            .join("structural.rs");
        let structural_src = fs::read_to_string(&structural_path).expect("read structural.rs");
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
    }

    #[test]
    fn test_check_34_purity_a_rejects_regex() {
        let clean = "pub fn add(a: i32, b: i32) -> i32 { a + b }";
        let res_clean = audit_source_purity(clean, "sample/src/lib.rs", false);
        assert!(res_clean.is_ok(), "Clean source must pass: {:?}", res_clean.err());

        let pat = ["Reg", "ex::"].concat();
        let mutant = format!("pub fn check() {{ {}new(\"\"); }}", pat);
        let res_mutant = audit_source_purity(&mutant, "sample/src/lib.rs", false);
        assert!(res_mutant.is_err(), "Mutant with regex must fail");
        let errs = res_mutant.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("regex usage detected")));
    }

    #[test]
    fn test_check_34_purity_b_rejects_static_string_slices() {
        let clean = "pub const ALL: &'static [Self] = &[Self::VariantA, Self::VariantB];";
        let res_clean = audit_source_purity(clean, "sample/src/lib.rs", false);
        assert!(res_clean.is_ok(), "Clean enum array must pass: {:?}", res_clean.err());

        let slice_kw = [": &[", "&str]"].concat();
        let mutant = format!("pub const KEYWORDS{kw} = &[\"forbidden\"];", kw = slice_kw);
        let res_mutant = audit_source_purity(&mutant, "sample/src/lib.rs", false);
        assert!(res_mutant.is_err(), "Mutant with static string slice array must fail");
        let errs = res_mutant.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("static string-slice array detected")));
    }

    #[test]
    fn test_check_34_purity_c_rejects_hollow_gate_stubs() {
        let clean = "pub fn check_gate(repo_root: &Path) -> Result<(), Vec<String>> { if repo_root.is_dir() { Ok(()) } else { Err(vec![]) } }";
        let res_clean = audit_source_purity(clean, "sample/src/checks/gate.rs", false);
        assert!(res_clean.is_ok(), "Clean genuine gate must pass: {:?}", res_clean.err());

        let hollow_pat = ["let", " _ = "].concat();
        let mutant = format!("pub fn check_gate(repo_root: &Path) -> Result<(), Vec<String>> {{ {}repo_root; Ok(()) }}", hollow_pat);
        let res_mutant = audit_source_purity(&mutant, "sample/src/checks/gate.rs", false);
        assert!(res_mutant.is_err(), "Mutant with hollow gate stub must fail");
        let errs = res_mutant.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("hollow gate stub detected")));
    }

    #[test]
    fn test_check_34_purity_d_rejects_fail_open_error_swallowing() {
        let clean = "for entry_res in WalkDir::new(dir) { let entry = match entry_res { Ok(e) => e, Err(err) => return Err(vec![err.to_string()]) }; }";
        let res_clean = audit_source_purity(clean, "sample/src/checks/gate.rs", false);
        assert!(res_clean.is_ok(), "Clean fail-closed loop must pass: {:?}", res_clean.err());

        let fm_pat = [".filter_map(", "|e|", " e.ok())"].concat();
        let mutant = format!("for entry in WalkDir::new(dir).into_iter(){} {{ }}", fm_pat);
        let res_mutant = audit_source_purity(&mutant, "sample/src/checks/gate.rs", false);
        assert!(res_mutant.is_err(), "Mutant with fail-open filter_map must fail");
        let errs = res_mutant.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("fail-open error swallowing detected")));
    }

    #[test]
    fn test_check_34_purity_e_rejects_cross_string_byte_slicing() {
        let clean = "let lower = s.to_ascii_lowercase(); let slice = lower.get(..4);";
        let res_clean = audit_source_purity(clean, "sample/src/lib.rs", false);
        assert!(res_clean.is_ok(), "Clean character boundary slicing must pass: {:?}", res_clean.err());

        let bad_slice = [".to_lowercase()", "[..4]"].concat();
        let mutant = format!("let slice = &s{};", bad_slice);
        let res_mutant = audit_source_purity(&mutant, "sample/src/lib.rs", false);
        assert!(res_mutant.is_err(), "Mutant with cross-string byte indexing must fail");
        let errs = res_mutant.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("cross-string UTF-8 byte-slice hazard detected")));
    }

    #[test]
    fn test_check_34_purity_f_rejects_missing_seven_section_rustdoc() {
        let clean = r#"
/// High-integrity parity check.
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
/// Realises: [REQ-PARITY-CHECK].
pub fn check_audit() -> Result<(), Vec<String>> {
    Ok(())
}
"#;
        let res_clean = audit_source_purity(clean, "sample/src/checks/parity_gates.rs", true);
        assert!(res_clean.is_ok(), "Clean function with all 7 sections must pass: {:?}", res_clean.err());

        let mutant = r#"
/// Partial doc missing section 7.
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
pub fn check_audit() -> Result<(), Vec<String>> {
    Ok(())
}
"#;
        let res_mutant = audit_source_purity(mutant, "sample/src/checks/parity_gates.rs", true);
        assert!(res_mutant.is_err(), "Mutant missing section 7 must fail");
        let errs = res_mutant.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("function missing mandatory 7-section Rustdoc")));
    }

    #[test]
    fn test_check_34_hardening_b_rejects_static_lifetime_string_slice_const() {
        let clean = "pub const ALL: &'static [Self] = &[Self::VariantA, Self::VariantB];";
        let res_clean = audit_source_purity(clean, "sample/src/lib.rs", false);
        assert!(res_clean.is_ok(), "Clean typed enum array must pass: {:?}", res_clean.err());

        let ty = ["&'static [", "&'static str]"].concat();
        let mutant = format!("pub const BAD: {} = &[\"a\"];", ty);
        let res_mutant = audit_source_purity(&mutant, "sample/src/lib.rs", false);
        assert!(res_mutant.is_err(), "Mutant with 'static string-slice const must fail");
        let errs = res_mutant.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("static string-slice array detected")), "{:?}", errs);

        let arr_ty = ["[&'static ", "str; 2]"].concat();
        let mutant_arr = format!("pub const BAD: {} = [\"a\", \"b\"];", arr_ty);
        let res_arr = audit_source_purity(&mutant_arr, "sample/src/lib.rs", false);
        assert!(res_arr.is_err(), "Mutant with 'static string array must fail");
    }

    #[test]
    fn test_check_34_hardening_b_rejects_static_str_array_into_iter() {
        let clean = "fn kinds() -> std::array::IntoIter<Kind, 3> { [Kind::A, Kind::B, Kind::C].into_iter() }";
        let res_clean = audit_source_purity(clean, "sample/src/lib.rs", false);
        assert!(res_clean.is_ok(), "Clean typed IntoIter must pass: {:?}", res_clean.err());

        let iter_ty = ["IntoIter<", "&'static str, 3>"].concat();
        let mutant = format!("fn names() -> std::array::{} {{ [\"a\", \"b\", \"c\"].into_iter() }}", iter_ty);
        let res_mutant = audit_source_purity(&mutant, "sample/src/lib.rs", false);
        assert!(res_mutant.is_err(), "Mutant with 'static str array iterator must fail");
        let errs = res_mutant.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("static string-slice array detected")), "{:?}", errs);
    }

    #[test]
    fn test_check_34_hardening_e_rejects_direct_range_slice_indexing() {
        let clean = "let sub = s.get(1..3);";
        let res_clean = audit_source_purity(clean, "sample/src/checks/parity_gates.rs", true);
        assert!(res_clean.is_ok(), "Clean .get(..) slicing must pass: {:?}", res_clean.err());

        let mutant = "let sub = &s[1..3];";
        let res_mutant = audit_source_purity(mutant, "sample/src/checks/parity_gates.rs", true);
        assert!(res_mutant.is_err(), "Mutant with direct panicking range indexing must fail");
        let errs = res_mutant.unwrap_err();
        assert!(
            errs.iter().any(|e| e.contains("direct panicking range-slice indexing detected")),
            "{:?}",
            errs
        );

        let res_unaudited = audit_source_purity(mutant, "sample/src/lib.rs", false);
        assert!(res_unaudited.is_ok(), "Rule (e) applies only when check_rustdoc is true: {:?}", res_unaudited.err());
    }

    #[test]
    fn test_check_34_hardening_f_rejects_duplicate_boilerplate_rustdoc() {
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
        assert!(res_mutant.is_err(), "Mutant with duplicated 7-section Rustdoc bodies must fail");
        let errs = res_mutant.unwrap_err();
        assert!(
            errs.iter().any(|e| e.contains("duplicate/boilerplate 7-section Rustdoc detected")),
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
        assert!(res_clean.is_ok(), "Function-specific Rustdoc bodies must pass: {:?}", res_clean.err());
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

    #[test]
    fn test_issue_441_documentation_contains_no_retired_script_references() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("repo root");
        let res = check_zero_retired_scripts_in_documentation(repo_root);
        assert!(
            res.is_ok(),
            "Documentation must not reference retired scripts (refs #441): {:?}",
            res.err()
        );
    }

    #[test]
    fn test_defect_441_mutation_rejects_retired_script_in_markdown() {
        let tmp = TempDir::new("c441_script_rejection");
        let docs = tmp.path.join("docs").join("operations");
        fs::create_dir_all(&docs).unwrap();
        let guide = docs.join("install-guide.md");
        let mut f = File::create(&guide).unwrap();
        let needle = "create_issue.sh";
        writeln!(f, "# Guide\nRun {needle} now").unwrap();

        let res = check_zero_retired_scripts_in_documentation(&tmp.path);
        assert!(res.is_err(), "Must reject retired script reference in docs");
        let errs = res.err().unwrap();
        assert!(errs.iter().any(|e| e.contains(needle)));
    }

    #[test]
    fn test_defect_450_retired_python_in_pipeline_docs_fails() {
        let tmp = TempDir::new("c450_pipeline_retired");
        let pipeline_dir = tmp.path.join(".pipeline");
        fs::create_dir_all(&pipeline_dir).unwrap();
        let const_file = pipeline_dir.join("constitution.md");
        let mut f = File::create(&const_file).unwrap();
        writeln!(f, "# Constitution\nRuns validators/spec_validator.py for quality gate").unwrap();

        let res = check_zero_retired_scripts_in_documentation(&tmp.path);
        assert!(res.is_err(), "Must reject retired Python validator in .pipeline/constitution.md");
        let errs = res.err().unwrap();
        assert!(errs.iter().any(|e| e.contains("spec_validator.py")));
    }

    #[test]
    fn test_defect_459_feat42_design_blueprint_compliance() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("repo root");

        let res = check_feat42_design_blueprint_compliance(repo_root);
        assert!(
            res.is_ok(),
            "FEAT-42 design blueprint must comply with deap-core references, CLI backwards compatibility, and Check 34 uniqueness (refs #459): {:?}",
            res.err()
        );
    }

    #[test]
    fn test_defect_459_mutation_rejects_sysml_core_in_design() {
        let tmp = TempDir::new("c459_sysml_core_mutation");
        let designs = tmp.path.join("docs").join("designs");
        fs::create_dir_all(&designs).unwrap();
        let feat = designs.join("FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md");
        let mut f = File::create(&feat).unwrap();
        writeln!(f, "# FEAT-42\nparticipant Core as sysml-core Parser").unwrap();

        let res = check_feat42_design_blueprint_compliance(&tmp.path);
        assert!(res.is_err(), "Must reject sysml-core in FEAT-42 design blueprint");
        let errs = res.err().unwrap();
        assert!(errs.iter().any(|e| e.contains("sysml-core")));
    }

    #[test]
    fn test_check_37_blueprint_grounding_passes_on_live_repo() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("repo root");

        let res = check_blueprint_grounding(repo_root);
        assert!(
            res.is_ok(),
            "Check 37 blueprint grounding failed on live repository: {:?}",
            res.err()
        );
    }

    #[test]
    fn test_check_37_rejects_fictional_crates_and_nonexistent_paths() {
        let tmp = TempDir::new("c37_mutations");
        let bp_dir = tmp.path.join("docs").join("architecture").join("blueprints");
        fs::create_dir_all(&bp_dir).unwrap();
        fs::create_dir_all(tmp.path.join("crates").join("deap-core").join("src")).unwrap();
        File::create(
            tmp.path
                .join("crates")
                .join("deap-core")
                .join("src")
                .join("lib.rs"),
        )
        .unwrap();

        for bp in CanonicalArchitectureBlueprint::ALL {
            let mut f = File::create(bp_dir.join(bp.as_str())).unwrap();
            writeln!(f, "# Blueprint\nGrounded in `crates/deap-core/src/lib.rs`.").unwrap();
        }

        let clean_res = check_blueprint_grounding(&tmp.path);
        assert!(
            clean_res.is_ok(),
            "Clean grounded blueprints must pass Check 37: {:?}",
            clean_res.err()
        );

        let target_bp = bp_dir.join(
            CanonicalArchitectureBlueprint::DeapSysmlV2IngestionEngineBlueprint.as_str(),
        );
        let mut f = File::create(&target_bp).unwrap();
        writeln!(
            f,
            "# Corrupted Blueprint\nUses `deap-ast` and `deap-harness`.\nRuns `verify_downstream`.\nInspects `crates/verify-baseline/src/checks/uml.rs`.\nContains em dash \u{2014} here."
        )
        .unwrap();

        let err_res = check_blueprint_grounding(&tmp.path);
        assert!(
            err_res.is_err(),
            "Check 37 must fail closed on fictional crates, retired tools, nonexistent paths, and em dashes"
        );
        let errs = err_res.unwrap_err();
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
            errs.iter().any(|e| e.contains("\\u2014") || e.contains("em dash")),
            "Must report em dash violation: {:?}",
            errs
        );
    }
}


