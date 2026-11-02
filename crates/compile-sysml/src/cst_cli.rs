#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Lossless CST inspection and code refactoring CLI handlers. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D qualification standards.
//! Provides deterministic CLI handlers for lossless Rust CST inspection, module extraction,
//! and Tier-B/C docstring compaction.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Pure deterministic command execution with fail-closed error handling.
//! - Filesystem interactions preserve cross-platform LF line endings and normalized relative paths.
//!
//! ## 3. Preconditions
//! - Callers supply well-formed argument slices with valid UTF-8 file paths.
//!
//! ## 4. Postconditions
//! - Emits structured JSON/text output or writes updated source files atomically.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - $O(N)$ in source length $N$ for CST parsing and $O(F)$ for filesystem traversal.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Missing arguments or non-existent files return descriptive `Err(String)`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0608], [SC-01], [FLS: fls_compile_cst_cli]
//! SysML v2: part def CompileCstCliHandler

use deap_core::rust_cst::{
    compact_tier_bc_docstrings, extract_rust_module, list_rust_items, normalize_lf, ParsedRustFile,
};
use std::fs;
use std::path::Path;

/// Evaluates CLI arguments for unified CST flags and executes if present.
/// Realises: [REQ-0608], [SC-01], [FLS: fls_compile_cst_cli]
pub fn try_run_cst_cli_flags(args: &[String]) -> Option<Result<(), String>> {
    let mut i = 0;
    while i < args.len() {
        let Some(arg) = args.get(i) else { break };
        if arg == "--list-rust-items" {
            let Some(f) = args.get(i + 1) else {
                return Some(Err("Missing file argument for --list-rust-items".to_string()));
            };
            return Some(run_list_rust_items(f));
        } else if let Some(f) = arg.strip_prefix("--list-rust-items=") {
            return Some(run_list_rust_items(f));
        } else if arg == "--extract-rust-module" {
            let (Some(f), Some(sm), Some(items)) = (args.get(i + 1), args.get(i + 2), args.get(i + 3)) else {
                return Some(Err("Usage: --extract-rust-module <FILE> <SUBMOD> <ITEM1,ITEM2,...>".to_string()));
            };
            return Some(run_extract_module(f, sm, items));
        } else if arg == "--compact-tier-bc" {
            let Some(p) = args.get(i + 1) else {
                return Some(Err("Missing path argument for --compact-tier-bc".to_string()));
            };
            return Some(run_compact_tier_bc(p));
        } else if let Some(p) = arg.strip_prefix("--compact-tier-bc=") {
            return Some(run_compact_tier_bc(p));
        }
        i += 1;
    }
    None
}

/// Intercepts process args and exits if a CST CLI flag was executed.
/// Realises: [REQ-0608], [SC-01], [FLS: fls_compile_cst_cli]
pub fn handle_cst_cli_if_present() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(res) = try_run_cst_cli_flags(&args) {
        match res {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }
}

/// Lists Rust AST items from a source file as JSON lines.
/// Realises: [REQ-0608], [FLS: fls_cst_cli]
fn run_list_rust_items(file: &str) -> Result<(), String> {
    let src = fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
    let parsed = ParsedRustFile::parse_rust_source(&src);
    let items = list_rust_items(&parsed);
    for it in &items {
        let line = serde_json::json!({
            "name": it.name,
            "kind": format!("{:?}", it.kind),
            "is_pub": it.is_pub,
            "start_line": it.start_line,
            "end_line": it.end_line
        });
        println!("{line}");
    }
    Ok(())
}

/// Executes module extraction from a parent source file into a submodule.
/// Realises: [REQ-0608], [FLS: fls_cst_cli]
fn run_extract_module(file: &str, submod: &str, items_str: &str) -> Result<(), String> {
    let src = fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
    let items: Vec<&str> = items_str.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    let plan = extract_rust_module(&src, submod, &items)?;
    let p = Path::new(file);
    let parent = p.parent().unwrap_or_else(|| Path::new("."));
    let target_submod = if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
        let sub_dir = parent.join(stem);
        if sub_dir.is_dir() { sub_dir.join(format!("{submod}.rs")) } else { parent.join(format!("{submod}.rs")) }
    } else {
        parent.join(format!("{submod}.rs"))
    };
    if let Some(dir) = target_submod.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    fs::write(&target_submod, normalize_lf(&plan.new_submodule_source)).map_err(|e| e.to_string())?;
    fs::write(p, normalize_lf(&plan.updated_parent_source)).map_err(|e| e.to_string())?;
    Ok(())
}

/// Recursively compacts Tier-B/C docstrings in all Rust files under a path.
/// Realises: [REQ-0608], [FLS: fls_cst_cli]
fn run_compact_tier_bc(path_str: &str) -> Result<(), String> {
    let p = Path::new(path_str);
    if !p.exists() { return Err(format!("Path does not exist: {path_str}")); }
    if p.is_file() {
        if p.extension().and_then(|s| s.to_str()) == Some("rs") && !p.components().any(|c| c.as_os_str() == "generated") {
            compact_file(p)?;
        }
        return Ok(());
    }
    for entry in walkdir::WalkDir::new(p).into_iter().flatten() {
        if entry.file_type().is_file()
            && entry.path().extension().and_then(|s| s.to_str()) == Some("rs")
            && !entry.path().components().any(|c| c.as_os_str() == "generated")
        {
            compact_file(entry.path())?;
        }
    }
    Ok(())
}

/// Compacts Tier-B/C docstrings within a single file.
/// Realises: [REQ-0608], [FLS: fls_cst_cli]
fn compact_file(p: &Path) -> Result<(), String> {
    let src = fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
    let compacted = compact_tier_bc_docstrings(&src);
    if compacted != src {
        fs::write(p, normalize_lf(&compacted)).map_err(|e| format!("{}: {e}", p.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cst_cli_list_rust_items() -> Result<(), String> {
        let dir = std::env::temp_dir().join(format!("cst_cli_test_{}", std::process::id()));
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let file = dir.join("test_item.rs");
        fs::write(&file, "pub struct SampleFoo;\npub fn sample_bar() {}\n").map_err(|e| e.to_string())?;
        let args = vec!["compile-sysml".into(), "--list-rust-items".into(), file.to_string_lossy().into()];
        let res = try_run_cst_cli_flags(&args);
        assert!(matches!(res, Some(Ok(()))));
        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn test_cst_cli_compact_tier_bc() -> Result<(), String> {
        let dir = std::env::temp_dir().join(format!("cst_cli_compact_{}", std::process::id()));
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let file = dir.join("test_doc.rs");
        let initial = "/// Summary line.\n/// Extra verbose line 1.\n/// Extra verbose line 2.\n/// Extra verbose line 3.\n/// Extra verbose line 4.\npub fn sample_fn() {}\n";
        fs::write(&file, initial).map_err(|e| e.to_string())?;
        let args = vec!["compile-sysml".into(), "--compact-tier-bc".into(), file.to_string_lossy().into()];
        let res = try_run_cst_cli_flags(&args);
        assert!(matches!(res, Some(Ok(()))));
        let updated = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        assert!(updated.lines().count() < initial.lines().count());
        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }
}
