#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Baseline line-count ratchet table locking legacy `.rs` files at exact HEAD ceilings.
//! Non-ratcheted and new modules must satisfy strict modularity bounds without exemption.
//! Realises: [REQ-0601], [REQ-0608]

pub use deap_core::workspace::ratchet_table::{
    active_greater_than_500_ratchet_count, lookup_ratchet_ceiling, LEGACY_RATCHET_MAX_LINES,
    ZERO_NEW_RATCHET_ENFORCED,
};

/// Hard upper bound on `LEGACY_RATCHET_MAX_LINES.len()` so the ratchet table can never grow (`[REQ-0601]`, `[REQ-0608]`).
pub const MAX_LEGACY_RATCHET_ENTRIES: usize = 206;

/// Compile-time invariant enforcing that `LEGACY_RATCHET_MAX_LINES` never increases in length.
const _: () = assert!(LEGACY_RATCHET_MAX_LINES.len() <= MAX_LEGACY_RATCHET_ENTRIES);

/// Forbidden sharded paths that must never exist on disk (`[REQ-0601]`, `[REQ-0608]`).
pub const FORBIDDEN_SHARD_PATHS: &[&str] = &[
    "crates/deap-core/src/source_ast",
    "crates/deap-core/src/sysml_ast/brace_and_masking.rs",
];

/// Forbidden dummy CST discard substrings in production/test Rust code (`[REQ-0601]`, `[REQ-0608]`).
pub const FORBIDDEN_DUMMY_CST_PATTERNS: &[&str] = &[
    "drop(parse_rust_source",
    "drop(parse_dart_cst",
    "drop(parse_sysml_cst",
    "drop(crate::dart_cst::parse_dart_",
    "let _ = parse_rust_source",
    "let _ = parse_dart_cst",
    "let _ = parse_sysml_cst",
    "let _ = deap_core::rust_cst::parse::parse_rust_source",
    "let _ = deap_core::dart_cst::parse_dart_cst",
    "let _ = deap_core::sysml_cst::parse_sysml_cst",
];

/// Mechanically verifies anti-sharding paths, ratchet ceiling non-growth, and zero dummy CST discards (`[REQ-0601]`, `[REQ-0608]`).
///
/// # Errors
/// Returns `Err(String)` if any forbidden shard path exists, the ratchet table grows, or a dummy CST discard is found.
pub fn verify_anti_cheat_and_anti_sharding(workspace_root: &std::path::Path) -> Result<(), String> {
    if LEGACY_RATCHET_MAX_LINES.len() > MAX_LEGACY_RATCHET_ENTRIES {
        return Err(format!(
            "LEGACY_RATCHET_MAX_LINES grew to {} (max {})",
            LEGACY_RATCHET_MAX_LINES.len(),
            MAX_LEGACY_RATCHET_ENTRIES
        ));
    }
    for &rel in FORBIDDEN_SHARD_PATHS {
        let full = rel.split('/').fold(workspace_root.to_path_buf(), |acc, s| acc.join(s));
        if full.exists() {
            return Err(format!("forbidden sharded path exists on disk: {rel}"));
        }
    }
    let crates_dir = workspace_root.join("crates");
    if !crates_dir.exists() {
        return Ok(());
    }
    for entry in walkdir::WalkDir::new(&crates_dir).into_iter().flatten() {
        if !entry.file_type().is_file() || entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(workspace_root)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        if rel == "crates/verify-baseline/src/checks/ratchet_table.rs" {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).map_err(|e| format!("{rel}: {e}"))?;
        for &pat in FORBIDDEN_DUMMY_CST_PATTERNS {
            if content.contains(pat) {
                return Err(format!("forbidden dummy CST discard '{pat}' in {rel}"));
            }
        }
    }
    Ok(())
}

/// Unit tests verifying ratchet ordering, key prefixing, and on-disk line parity (`[REQ-0601]`, `[REQ-0608]`).
#[cfg(test)]
mod tests {
    use super::{
        active_greater_than_500_ratchet_count, lookup_ratchet_ceiling,
        verify_anti_cheat_and_anti_sharding, LEGACY_RATCHET_MAX_LINES,
        MAX_LEGACY_RATCHET_ENTRIES, ZERO_NEW_RATCHET_ENFORCED,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use walkdir::WalkDir;

    /// Resolves the workspace root directory from `CARGO_MANIFEST_DIR` (`[REQ-0601]`).
    fn workspace_root() -> Result<PathBuf, String> {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .ok_or_else(|| String::from("unable to resolve workspace root"))
    }

    /// Verifies strict lexicographical sort, `./crates/` prefix, and positive ceilings (`[REQ-0601]`, `[REQ-0608]`).
    #[test]
    fn ratchet_table_is_strictly_sorted_and_prefixed() -> Result<(), String> {
        if LEGACY_RATCHET_MAX_LINES.is_empty() {
            return Err(String::from("LEGACY_RATCHET_MAX_LINES must not be empty"));
        }
        for &(key, ceiling) in LEGACY_RATCHET_MAX_LINES {
            if !key.starts_with("./crates/") || !key.ends_with(".rs") || key.contains('\\') || ceiling == 0 {
                return Err(format!("invalid ratchet entry: ({key}, {ceiling})"));
            }
        }
        for pair in LEGACY_RATCHET_MAX_LINES.windows(2) {
            if let [(prev, _), (next, _)] = pair {
                if prev >= next {
                    return Err(format!("ratchet keys not strictly sorted: {prev} >= {next}"));
                }
            }
        }
        Ok(())
    }

    /// Verifies every ratchet entry matches exact on-disk lines and covers all `> 500`-line files (`[REQ-0608]`).
    #[test]
    fn ratchet_table_matches_exact_on_disk_line_counts() -> Result<(), String> {
        let root = workspace_root()?;
        for &(rel_key, expected) in LEGACY_RATCHET_MAX_LINES {
            let stripped = rel_key.strip_prefix("./").unwrap_or(rel_key);
            let disk_path = stripped.split('/').fold(root.clone(), |acc, seg| acc.join(seg));
            let actual = fs::read_to_string(&disk_path).map_err(|e| format!("{rel_key}: {e}"))?.lines().count();
            if actual != expected || lookup_ratchet_ceiling(rel_key) != Some(expected) {
                return Err(format!("mismatch for {rel_key}: disk={actual}, table={expected}"));
            }
        }
        for entry in WalkDir::new(root.join("crates")).into_iter().flatten() {
            if !entry.file_type().is_file() || entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let rel = entry.path().strip_prefix(&root).map_err(|e| e.to_string())?.to_string_lossy().replace('\\', "/");
            let key = format!("./{rel}");
            let lines = fs::read_to_string(entry.path()).map_err(|e| e.to_string())?.lines().count();
            if lines > 500 && lookup_ratchet_ceiling(&key) != Some(lines) {
                return Err(format!(">500-line file missing from ratchet table: {key} ({lines})"));
            }
        }
        Ok(())
    }

    /// Verifies cross-platform path separator normalization and non-ratcheted misses (`[REQ-0601]`).
    #[test]
    fn lookup_ratchet_ceiling_handles_normalization_and_misses() -> Result<(), String> {
        if lookup_ratchet_ceiling(".\\crates\\compile-sysml\\tests\\reproducer_tests.rs") != Some(3804)
            || lookup_ratchet_ceiling("crates/compile-sysml/tests/reproducer_tests.rs") != Some(3804)
        {
            return Err(String::from("path normalization lookup failed"));
        }
        if lookup_ratchet_ceiling("./crates/verify-baseline/src/checks/ratchet_table.rs").is_some()
            || lookup_ratchet_ceiling("./crates/nonexistent.rs").is_some()
            || lookup_ratchet_ceiling("").is_some()
        {
            return Err(String::from("unexpected ceiling hit for non-ratcheted path"));
        }
        let expected_gt_500 = LEGACY_RATCHET_MAX_LINES.iter().filter(|&&(_, c)| c > 500).count();
        if !ZERO_NEW_RATCHET_ENFORCED || active_greater_than_500_ratchet_count() != expected_gt_500 {
            return Err(String::from("active_greater_than_500_ratchet_count invariant failed"));
        }
        Ok(())
    }

    /// Verifies anti-sharding paths, ratchet ceiling non-growth, and zero dummy CST discards (`[REQ-0601]`, `[REQ-0608]`).
    #[test]
    fn anti_cheat_forbids_sharded_paths_and_dummy_cst_discards() -> Result<(), String> {
        let root = workspace_root()?;
        if LEGACY_RATCHET_MAX_LINES.len() > MAX_LEGACY_RATCHET_ENTRIES {
            return Err(format!("ratchet table grew beyond {MAX_LEGACY_RATCHET_ENTRIES}"));
        }
        verify_anti_cheat_and_anti_sharding(&root)?;
        let tmp = std::env::temp_dir().join(format!("deap_anti_cheat_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let shard_dir = tmp.join("crates/deap-core/src/source_ast");
        fs::create_dir_all(&shard_dir).map_err(|e| e.to_string())?;
        let shard_res = verify_anti_cheat_and_anti_sharding(&tmp);
        let _ = fs::remove_dir_all(&shard_dir);
        if shard_res.is_ok() {
            let _ = fs::remove_dir_all(&tmp);
            return Err(String::from("expected failure when source_ast directory exists"));
        }
        let dummy_file = tmp.join("crates/demo/src/lib.rs");
        fs::create_dir_all(dummy_file.parent().ok_or("no parent")?).map_err(|e| e.to_string())?;
        fs::write(&dummy_file, format!("fn f() {{ {}(\"x\"); }}\n", "drop(parse_rust_source"))
            .map_err(|e| e.to_string())?;
        let dummy_res = verify_anti_cheat_and_anti_sharding(&tmp);
        let _ = fs::remove_dir_all(&tmp);
        if dummy_res.is_ok() {
            return Err(String::from("expected failure on dummy CST discard"));
        }
        Ok(())
    }
}
