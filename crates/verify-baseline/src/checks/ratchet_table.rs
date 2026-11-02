#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Baseline line-count ratchet table locking legacy `.rs` files at exact HEAD ceilings.
//! Non-ratcheted and new modules must satisfy strict modularity bounds without exemption.
//! Realises: [REQ-0601], [REQ-0608]

pub use deap_core::workspace::ratchet_table::{
    active_greater_than_500_ratchet_count, lookup_ratchet_ceiling, LEGACY_RATCHET_MAX_LINES,
    ZERO_NEW_RATCHET_ENFORCED,
};

/// Unit tests verifying ratchet ordering, key prefixing, and on-disk line parity (`[REQ-0601]`, `[REQ-0608]`).
#[cfg(test)]
mod tests {
    use super::{
        active_greater_than_500_ratchet_count, lookup_ratchet_ceiling, LEGACY_RATCHET_MAX_LINES,
        ZERO_NEW_RATCHET_ENFORCED,
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
}
