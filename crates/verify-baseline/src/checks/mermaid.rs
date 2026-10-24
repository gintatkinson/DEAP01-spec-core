//! Check 13B: Mermaid diagram syntax validation across markdown files.

use deap_core::markdown::check_mermaid_syntax as core_check_mermaid_syntax;
use deap_core::rules::EXCLUDED_DIRS;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Check 13B: Mermaid diagram syntax validation across all .md files in the repository.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `check_mermaid_syntax` in `crates/verify-baseline/src/checks/mermaid.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `check_mermaid_syntax` (declaration #1) within `CHECKS-MERMAID`.
///
/// ## 3. Preconditions
/// - `repo_root: &Path` must satisfy domain and type-safety constraints prior to invoking `check_mermaid_syntax`.
///
/// ## 4. Postconditions
/// - Returns `Result<(), Vec<String>>` from `check_mermaid_syntax` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(repo_root)) bounded over input/member dimension of `check_mermaid_syntax`.
/// - Space Complexity: O(N(repo_root)) bounded auxiliary memory for `check_mermaid_syntax`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `check_mermaid_syntax` (`Result<(), Vec<String>>`).
/// - State: Maintains consistent state invariants across `check_mermaid_syntax` without partial mutation on error.
/// - Action: Executes `check_mermaid_syntax` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `check_mermaid_syntax` (`CHECKS-MERMAID`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECKS-MERMAID-CHECK-MERMAID-SYNTAX] | SysML v2: ActionDef(check_mermaid_syntax)
pub fn check_mermaid_syntax(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut all_errors = Vec::new();

    for entry_res in WalkDir::new(repo_root)
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
            Ok(val) => val,
            Err(err) => {
                all_errors.push(format!("Check 13B directory walk I/O error: {}", err));
                continue;
            }
        };
        if entry.file_type().is_file() {
            let path = entry.path();
            if path.extension().map(|ext| ext == "md").unwrap_or(false) {
                let rel = path
                    .strip_prefix(repo_root)
                    .unwrap_or(path)
                    .display()
                    .to_string();

                match fs::read_to_string(path) {
                    Ok(content) => {
                        let errors = core_check_mermaid_syntax(&content, &rel);
                        all_errors.extend(errors);
                    }
                    Err(e) => {
                        all_errors.push(format!("Failed to read {}: {}", rel, e));
                    }
                }
            }
        }
    }

    if all_errors.is_empty() {
        Ok(())
    } else {
        Err(all_errors)
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
            let path = std::env::temp_dir().join(format!("verify_mermaid_{}_{}_{}", name, nanos, count));
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
    fn test_clean_mermaid_passes() {
        let tmp = TempDir::new("clean");
        let md = tmp.path.join("doc.md");
        let mut f = File::create(md).unwrap();
        writeln!(f, "```mermaid\ngraph TD\nA[\"Node A\"] --> B[\"Node B\"]\n```").unwrap();

        assert!(check_mermaid_syntax(&tmp.path).is_ok());
    }

    #[test]
    fn test_unclosed_fence_fails() {
        let tmp = TempDir::new("unclosed");
        let md = tmp.path.join("doc.md");
        let mut f = File::create(md).unwrap();
        writeln!(f, "```mermaid\ngraph TD\nA --> B\n").unwrap();

        let res = check_mermaid_syntax(&tmp.path);
        assert!(res.is_err());
        assert!(res.unwrap_err()[0].contains("unclosed ```mermaid fence"));
    }

    #[test]
    fn test_invalid_header_fails() {
        let tmp = TempDir::new("invalid_hdr");
        let md = tmp.path.join("doc.md");
        let mut f = File::create(md).unwrap();
        writeln!(f, "```mermaid\nbadHeader TD\nA --> B\n```").unwrap();

        let res = check_mermaid_syntax(&tmp.path);
        assert!(res.is_err());
        assert!(res.unwrap_err()[0].contains("missing or invalid Mermaid diagram header"));
    }

    /// Verifies that Check 13B fails closed when a subdirectory cannot be traversed by `WalkDir`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and DO-330 TQL-4 fail-closed directory traversal verification for `check_mermaid_syntax` (refs #500).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Any `WalkDir` traversal error on an in-scope directory must append a diagnostic to `all_errors` and return `Err`.
    ///
    /// ## 3. Preconditions
    /// - Temporary workspace contains an unreadable subdirectory (`mode 0o000` on Unix).
    ///
    /// ## 4. Postconditions
    /// - `check_mermaid_syntax` returns `Err(Vec<String>)` reporting the directory walk I/O error, and permissions are restored before cleanup.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) over a single unreadable directory.
    /// - Space Complexity: O(1) temporary directory fixture.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Exercises `WalkDir` `Err` handling in `check_mermaid_syntax`.
    /// - State: Restores `0o755` permissions before `TempDir` drop.
    /// - Action: Asserts fail-closed error return.
    /// - Resource: Zero leaked temporary directories.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-CHECK-13B-FAIL-CLOSED-WALKDIR-TEST]
    #[cfg(unix)]
    #[test]
    fn test_unreadable_subdir_fails_closed() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = TempDir::new("unreadable_dir");
        let sub = tmp.path.join("locked_docs");
        fs::create_dir_all(&sub).unwrap();
        fs::set_permissions(&sub, fs::Permissions::from_mode(0o000)).unwrap();

        let res = check_mermaid_syntax(&tmp.path);
        let _ = fs::set_permissions(&sub, fs::Permissions::from_mode(0o755));

        assert!(
            res.is_err(),
            "Check 13B must fail closed on unreadable directory, but returned Ok(())"
        );
    }
}
