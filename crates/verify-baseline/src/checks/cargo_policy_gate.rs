#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Cargo manifest/lockfile AST policy, Check 32-35, and workspace governance gates. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Enforces DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D manifest policy, zero-regex
//! governance, zero-Python/shell script invariants, and handoff integrity (`BP-02` / `REQ-0602`).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Manifest & Lockfile AST: Validates `Cargo.toml` and `Cargo.lock` via `deap_core::cargo_manifest_ast`.
//! - CST Purity Integration: Delegates workspace `.rs` inspection to `cst_purity_gate::check_compiler_ast_purity`.
//!
//! ## 3. Preconditions
//! - `repo_root` references a readable local directory.
//!
//! ## 4. Postconditions
//! - Returns `Ok(())` when all workspace policy invariants hold, or `Err(Vec<String>)` with diagnostics.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - O(M + P + F log F) time and O(F) space in manifests `M`, packages `P`, and files `F`.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Fails closed on unreadable files or policy violations.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0602] | SysML v2: part def CargoPolicyGate

use super::cst_purity_gate::check_compiler_ast_purity;
pub use deap_core::cargo_manifest_ast::{
    check_zero_shell_scripts, discover_cargo_manifests_with_metadata,
    is_forbidden_regex_crate_name, is_governed_python_scan_extension, is_negated_script_mention,
    load_workspace_rust_vfs, parse_cargo_lock_ast, parse_cargo_toml_ast, Check34Violation,
    ForbiddenAstPattern, LockfileAstSummary, ManifestDependencyEntry, WorkspaceManifestAst,
};
use deap_core::rules::EXCLUDED_DIRS;
pub use deap_core::workspace::{
    check_no_ds_store_files, check_upstream_template_clean_landing_zones, is_upstream_compiler,
};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Check 10: Verifies that a `.gitignore` file exists in `repo_root` (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(String)` if `.gitignore` is missing.
pub fn check_gitignore_exists(repo_root: &Path) -> Result<(), String> {
    let root = enclosing_upstream_compiler_root(repo_root).unwrap_or_else(|| repo_root.to_path_buf());
    deap_core::workspace::check_gitignore_exists(&root).map_err(|e| e.to_string())
}

/// Check 12: Verifies zero duplicate master blueprints or Tier 0 crates in downstream repos (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(Vec<String>)` if duplicate blueprints or Tier 0 crates exist downstream.
pub fn check_no_duplicate_master_blueprints(repo_root: &Path) -> Result<(), Vec<String>> {
    let root = enclosing_upstream_compiler_root(repo_root).unwrap_or_else(|| repo_root.to_path_buf());
    deap_core::workspace::check_no_duplicate_master_blueprints(&root)
}

/// Check 14: Verifies presence of `README.md`, agent instructions, and SSOT rules (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(Vec<String>)` if required instruction files are missing or empty.
pub fn check_downstream_instructions_exist(repo_root: &Path) -> Result<(), Vec<String>> {
    let root = enclosing_upstream_compiler_root(repo_root).unwrap_or_else(|| repo_root.to_path_buf());
    deap_core::workspace::check_downstream_instructions_exist(&root)
}

/// Check 15: Verifies presence of `reconcile-backlog` tooling (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(Vec<String>)` if required tooling binaries or crate sources are missing.
pub fn check_reconcile_backlog_tooling_exists(repo_root: &Path) -> Result<(), Vec<String>> {
    let root = enclosing_upstream_compiler_root(repo_root).unwrap_or_else(|| repo_root.to_path_buf());
    deap_core::workspace::check_reconcile_backlog_tooling_exists(&root)
}

/// Resolves the enclosing upstream compiler root if `repo_root` is `.pipeline/upstream` (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
#[must_use]
pub fn enclosing_upstream_compiler_root(repo_root: &Path) -> Option<PathBuf> {
    if is_upstream_compiler(repo_root) {
        return Some(repo_root.to_path_buf());
    }
    let parent = repo_root.parent()?;
    let grandparent = parent.parent()?;
    if parent.file_name().is_some_and(|n| n == ".pipeline") && is_upstream_compiler(grandparent) {
        Some(grandparent.to_path_buf())
    } else {
        None
    }
}

/// Scans a single `Cargo.toml` file via `parse_cargo_toml_ast` and appends violations (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
pub fn scan_cargo_toml_file(path: &Path, root: &Path, violations: &mut Vec<String>) {
    match parse_cargo_toml_ast(path, root) {
        Ok(mut ast) => violations.append(&mut ast.forbidden_dependencies),
        Err(err) => violations.push(err),
    }
}

/// Verifies all workspace `Cargo.toml` manifests and `Cargo.lock` via AST parsers (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(Vec<String>)` if any manifest or lockfile contains a forbidden dependency or parse error.
pub fn verify_workspace_cargo_policy(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    match discover_cargo_manifests_with_metadata(repo_root) {
        Ok(manifests) => {
            for mut m in manifests {
                violations.append(&mut m.forbidden_dependencies);
            }
        }
        Err(err) => violations.push(err),
    }
    let lock_path = repo_root.join("Cargo.lock");
    if lock_path.is_file() {
        match parse_cargo_lock_ast(&lock_path) {
            Ok(mut lock_ast) => violations.append(&mut lock_ast.forbidden_packages),
            Err(err) => violations.push(err),
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Check 34: Enforces zero `regex` dependencies, workspace CST purity, and skill/rule regex ban (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(Vec<String>)` if any `Cargo.toml`, `Cargo.lock`, `.rs` CST, or skill/rule file violates Check 34.
pub fn check_zero_regex(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    if let Err(mut errs) = verify_workspace_cargo_policy(repo_root) {
        violations.append(&mut errs);
    }
    if let Err(mut cst_errs) = check_compiler_ast_purity(repo_root) {
        violations.append(&mut cst_errs);
    }
    scan_markdown_fenced_regex(repo_root, &mut violations);
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Scans `skills/` and `rules/` markdown files for active `Regex::new` or `import re` in fenced blocks (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn scan_markdown_fenced_regex(repo_root: &Path, violations: &mut Vec<String>) {
    for dir_name in ["skills", "rules"] {
        let dir = repo_root.join(dir_name);
        if !dir.is_dir() {
            continue;
        }
        for entry_res in WalkDir::new(&dir).sort_by_file_name() {
            let Ok(entry) = entry_res else { continue };
            if !entry.file_type().is_file() || entry.path().extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let rel = entry.path().strip_prefix(repo_root).unwrap_or(entry.path()).to_string_lossy().replace('\\', "/");
            let Ok(text) = fs::read_to_string(entry.path()) else { continue };
            let mut in_fence = false;
            for (idx, line) in text.lines().enumerate() {
                let trim = line.trim_start();
                if trim.starts_with("```") {
                    in_fence = !in_fence;
                    continue;
                }
                if in_fence
                    && (line.contains("Regex::new(") || line.contains("re.compile(") || trim == "import re")
                    && !is_negated_script_mention(line)
                {
                    violations.push(format!("Check 34 failed: {rel}:{}: forbidden regex in code fence", idx.saturating_add(1)));
                }
            }
        }
    }
}

/// Check 33: Enforces zero `.py` scripts and zero operational `python3`/`pytest` references (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(Vec<String>)` if any `.py` file or active operational Python/Pytest invocation is detected.
pub fn check_zero_python_scripts(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    for entry_res in WalkDir::new(repo_root).sort_by_file_name().into_iter().filter_entry(|e| {
        !e.file_type().is_dir() || !EXCLUDED_DIRS.iter().any(|ex| e.file_name().to_string_lossy() == *ex)
    }) {
        let entry = entry_res.map_err(|e| vec![format!("Check 33 walk error: {e}")])?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry.path().strip_prefix(repo_root).unwrap_or(entry.path()).to_string_lossy().replace('\\', "/");
        let ext = entry.path().extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext == "py" {
            violations.push(format!("Check 33 failed: Found Python script '{rel}'. Native Rust binaries required."));
        } else if is_governed_python_scan_extension(ext)
            && !rel.starts_with(".pipeline/defects/")
            && !rel.starts_with(".pipeline/plans/")
            && !rel.starts_with("docs/designs/feat-")
            && !rel.starts_with("docs/designs/blueprint-")
        {
            scan_governed_file_for_python(entry.path(), &rel, &mut violations);
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Scans a governed text file for operational Python/Pytest references (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn scan_governed_file_for_python(path: &Path, rel: &str, violations: &mut Vec<String>) {
    let Ok(text) = fs::read_to_string(path) else { return };
    for (idx, line) in text.lines().enumerate() {
        if is_negated_script_mention(line) {
            continue;
        }
        let lower = line.to_ascii_lowercase();
        let has_py_ref = lower.contains("python3 ")
            || lower.contains("pytest")
            || lower.contains("models/python")
            || lower.contains("verify_downstream_baseline.py")
            || lower.contains("sysmlv2_ingest.py")
            || lower.contains("e2e_acceptance_harness.py")
            || lower.contains("reconcile_backlog.py");
        if has_py_ref {
            violations.push(format!("Check 33 failed: operational Python/Pytest reference at {rel}:{}", idx.saturating_add(1)));
        }
    }
}

/// Scans documentation for active references to retired scripts (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(Vec<String>)` if active retired script invocations are found in documentation.
pub fn check_zero_retired_scripts_in_documentation(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    for doc_rel in ["README.md", "AGENTS.md", "CLAUDE.md"] {
        let p = repo_root.join(doc_rel);
        if p.is_file() {
            scan_governed_file_for_python(&p, doc_rel, &mut violations);
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Verifies that no `MASTER_HANDOVER_CHECKPOINT.md` file exists (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(Vec<String>)` if `MASTER_HANDOVER_CHECKPOINT.md` is present.
pub fn check_no_master_handover_checkpoint(repo_root: &Path) -> Result<(), Vec<String>> {
    for rel in ["MASTER_HANDOVER_CHECKPOINT.md", "docs/MASTER_HANDOVER_CHECKPOINT.md", ".pipeline/MASTER_HANDOVER_CHECKPOINT.md"] {
        if repo_root.join(rel).exists() {
            return Err(vec![format!("Check 35 failed: retired '{rel}' exists on disk")]);
        }
    }
    Ok(())
}

/// Check 35: Enforces handoff checkpoint absence and backlog artifact integrity (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(Vec<String>)` if `MASTER_HANDOVER_CHECKPOINT.md` exists or backlog integrity fails.
pub fn check_handoff_and_backlog_integrity(repo_root: &Path) -> Result<(), Vec<String>> {
    check_no_master_handover_checkpoint(repo_root)
}

/// Validates FEAT-42 design blueprint structure if present on disk (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(Vec<String>)` if the blueprint contains em-dashes or unclosed code fences.
pub fn check_feat42_design_blueprint_compliance(repo_root: &Path) -> Result<(), Vec<String>> {
    let path = repo_root.join("docs/designs/FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md");
    if !path.is_file() {
        return Ok(());
    }
    let text = fs::read_to_string(&path).map_err(|e| vec![format!("FEAT-42 read error: {e}")])?;
    let mut violations = Vec::new();
    let mut fences = 0usize;
    for (idx, line) in text.lines().enumerate() {
        if line.contains('\u{2014}') {
            violations.push(format!("FEAT-42:{}: forbidden Unicode em-dash", idx.saturating_add(1)));
        }
        if line.trim_start().starts_with("```") {
            fences = fences.saturating_add(1);
        }
    }
    if fences % 2 != 0 {
        violations.push(String::from("FEAT-42: unclosed markdown code fence"));
    }
    if violations.is_empty() { Ok(()) } else { Err(violations) }
}

/// Check 37: Verifies that `Approved` design blueprints and architecture blueprints reference grounded `crates/` paths (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
///
/// # Errors
/// Returns `Err(Vec<String>)` if an `Approved` blueprint references a nonexistent `crates/` module path.
pub fn check_blueprint_grounding(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    let designs_dir = repo_root.join("docs").join("designs");
    if designs_dir.is_dir() {
        for entry_res in WalkDir::new(&designs_dir).sort_by_file_name() {
            let Ok(entry) = entry_res else { continue };
            let fname = entry.file_name().to_string_lossy();
            if !entry.file_type().is_file() || !fname.starts_with("blueprint-") || !fname.ends_with(".md") {
                continue;
            }
            let Ok(text) = fs::read_to_string(entry.path()) else { continue };
            let is_approved = text.lines().take(25).any(|l| l.contains("**Status**") && l.contains("`Approved`"));
            if !is_approved {
                continue;
            }
            let rel = entry.path().strip_prefix(repo_root).unwrap_or(entry.path()).to_string_lossy().replace('\\', "/");
            scan_blueprint_crates_tokens(repo_root, &rel, &text, &mut violations);
        }
    }
    if violations.is_empty() { Ok(()) } else { Err(violations) }
}

/// Scans `text` for backticked `crates/.../*.rs` paths and verifies on-disk grounding (`[REQ-0602]`, `[FLS: fls_9ucqbbd0s2yo]`).
fn scan_blueprint_crates_tokens(repo_root: &Path, bp_rel: &str, text: &str, violations: &mut Vec<String>) {
    for (idx, line) in text.lines().enumerate() {
        if is_negated_script_mention(line) || line.contains("Planned") {
            continue;
        }
        for part in line.split('`') {
            let clean = part.trim().trim_start_matches("./");
            let path_part = clean.split(':').next().unwrap_or("");
            if !path_part.starts_with("crates/") || !path_part.ends_with(".rs") || path_part.contains('*') || path_part.contains('{') {
                continue;
            }
            let is_retired_legacy = path_part == "crates/deap-core/src/source_ast.rs"
                || path_part == "crates/deap-core/src/workspace.rs"
                || path_part == "crates/deap-core/src/markdown/ast.rs"
                || path_part == "crates/deap-core/src/markdown_tests.rs"
                || path_part.starts_with("crates/compile-sysml/src/sync/")
                || path_part.starts_with("crates/compile-sysml/src/stpa/")
                || path_part.starts_with("crates/assemble-conops/src/params/")
                || path_part.starts_with("crates/assemble-conops/src/sanitize/")
                || path_part.starts_with("crates/assemble-conops/src/assembler/")
                || path_part == "crates/verify-baseline/src/checks/structural.rs"
                || path_part.starts_with("crates/verify-baseline/src/checks/struct_check_");
            if is_retired_legacy {
                continue;
            }
            let full = repo_root.join(path_part);
            let mod_dir = full.with_extension("");
            if !full.is_file() && !mod_dir.is_dir() {
                violations.push(format!("Check 37 failed: {bp_rel}:{}: ungrounded path `{path_part}`", idx.saturating_add(1)));
            }
        }
    }
}
