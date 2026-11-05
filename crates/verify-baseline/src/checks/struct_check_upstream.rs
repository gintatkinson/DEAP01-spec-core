//! Upstream blueprint domain cleanliness (Check 18) and zero shell script (Check 32) gates.
//!
//! Safety & Traceability: Realises: [REQ-CHECK-18-UPSTREAM-BLUEPRINT-CLEANLINESS], [REQ-CHECK-32-ZERO-SHELL-SCRIPTS]
//! Contract: Enforces upstream landing zone and blueprint cleanliness (Check 18) and zero `.sh` scripts (Check 32).
//! Bounds & Faults: Single-pass directory walks; fails closed with sorted diagnostic vectors.

use deap_core::rules::EXCLUDED_DIRS;
use deap_core::workspace::is_upstream_compiler;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use super::struct_check_handoff::check_no_master_handover_checkpoint;

/// Check 18: Upstream Blueprint Domain-Agnostic Cleanliness Invariant.
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
    scan_upstream_landing_zones(repo_root, &mut violations);
    scan_upstream_blueprints_dir(repo_root, &mut violations);
    scan_upstream_designs_dir(repo_root, &mut violations);

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

/// Scans upstream landing zones to ensure only `.gitkeep` files exist.
/// Realises: [REQ-CHECK-18-UPSTREAM-BLUEPRINT-CLEANLINESS]
fn scan_upstream_landing_zones(repo_root: &Path, violations: &mut Vec<String>) {
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
                !EXCLUDED_DIRS.iter().any(|ex| name == *ex)
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
            if entry.file_type().is_file() && entry.file_name().to_string_lossy() != ".gitkeep" {
                let rel = entry
                    .path()
                    .strip_prefix(repo_root)
                    .unwrap_or(entry.path())
                    .display()
                    .to_string();
                violations.push(format!(
                    "Upstream template landing zone '{}' contains forbidden non-.gitkeep file: {}",
                    zone_dir.display(),
                    rel
                ));
            }
        }
    }
}

/// Scans `docs/architecture/blueprints` for forbidden `.sysml` or `.digest.json` files.
/// Realises: [REQ-CHECK-18-UPSTREAM-BLUEPRINT-CLEANLINESS]
fn scan_upstream_blueprints_dir(repo_root: &Path, violations: &mut Vec<String>) {
    let blueprints_dir = repo_root
        .join("docs")
        .join("architecture")
        .join("blueprints");
    if !blueprints_dir.is_dir() {
        return;
    }
    for entry_res in WalkDir::new(&blueprints_dir).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let name = e.file_name().to_string_lossy();
            !EXCLUDED_DIRS.iter().any(|ex| name == *ex)
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
            let fname_lower = entry.file_name().to_string_lossy().to_lowercase();
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

/// Scans `docs/designs` for ungrounded `.sysml` or `.digest.json` files lacking a complete triplet.
/// Realises: [REQ-CHECK-18-UPSTREAM-BLUEPRINT-CLEANLINESS]
fn scan_upstream_designs_dir(repo_root: &Path, violations: &mut Vec<String>) {
    let designs_dir = repo_root.join("docs").join("designs");
    if !designs_dir.is_dir() {
        return;
    }
    for entry_res in WalkDir::new(&designs_dir).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let name = e.file_name().to_string_lossy();
            !EXCLUDED_DIRS.iter().any(|ex| name == *ex)
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
            if (fname_lower.ends_with(".sysml") || fname_lower.ends_with(".digest.json"))
                && !is_valid_blueprint_triplet(entry.path(), &fname, &fname_lower)
            {
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

/// Checks whether a `.sysml` or `.digest.json` file in `docs/designs/` belongs to a valid non-empty triplet.
/// Realises: [REQ-CHECK-18-UPSTREAM-BLUEPRINT-CLEANLINESS]
fn is_valid_blueprint_triplet(entry_path: &Path, fname: &str, fname_lower: &str) -> bool {
    let stem_opt = if fname_lower.ends_with(".sysml") {
        fname.get(..fname.len().saturating_sub(".sysml".len()))
    } else {
        fname.get(..fname.len().saturating_sub(".digest.json".len()))
    };
    let Some(stem) = stem_opt else {
        return false;
    };
    if !stem.starts_with("blueprint-") || stem.len() <= "blueprint-".len() {
        return false;
    }
    let Some(parent) = entry_path.parent() else {
        return false;
    };
    let non_empty = |ext: &str| -> bool {
        fs::metadata(parent.join(format!("{stem}.{ext}"))).is_ok_and(|m| m.len() > 0)
    };
    non_empty("sysml") && non_empty("md") && non_empty("digest.json")
}

/// Check 32: Zero Shell Scripts Invariant.
/// Realises: [REQ-CHECK-32-ZERO-SHELL-SCRIPTS].
pub fn check_zero_shell_scripts(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    for entry_res in WalkDir::new(repo_root).into_iter().filter_entry(|e| {
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
                violations.push(format!("Check 32 I/O error: {}", err));
                continue;
            }
        };
        if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "sh") {
            let rel = entry
                .path()
                .strip_prefix(repo_root)
                .unwrap_or(entry.path())
                .display()
                .to_string();
            violations.push(rel);
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        violations.sort();
        Err(violations)
    }
}
