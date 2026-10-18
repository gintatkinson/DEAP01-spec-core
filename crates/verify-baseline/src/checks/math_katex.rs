//! Check 13: KaTeX and LaTeX mathematical syntax verification across markdown files.

use deap_core::markdown::check_latex_katex_syntax as core_check_latex_katex_syntax;
use deap_core::rules::EXCLUDED_DIRS;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Check 13: KaTeX / LaTeX mathematical syntax validation across all .md files in the repository.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `check_latex_katex_syntax` in `crates/verify-baseline/src/checks/math_katex.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `check_latex_katex_syntax` (declaration #1) within `CHECKS-MATH-KATEX`.
///
/// ## 3. Preconditions
/// - `repo_root: &Path` must satisfy domain and type-safety constraints prior to invoking `check_latex_katex_syntax`.
///
/// ## 4. Postconditions
/// - Returns `Result<(), Vec<String>>` from `check_latex_katex_syntax` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(repo_root)) bounded over input/member dimension of `check_latex_katex_syntax`.
/// - Space Complexity: O(N(repo_root)) bounded auxiliary memory for `check_latex_katex_syntax`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `check_latex_katex_syntax` (`Result<(), Vec<String>>`).
/// - State: Maintains consistent state invariants across `check_latex_katex_syntax` without partial mutation on error.
/// - Action: Executes `check_latex_katex_syntax` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `check_latex_katex_syntax` (`CHECKS-MATH-KATEX`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECKS-MATH-KATEX-CHECK-LATEX-KATEX-SYNTAX] | SysML v2: ActionDef(check_latex_katex_syntax)
pub fn check_latex_katex_syntax(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut all_errors = Vec::new();

    for entry in WalkDir::new(repo_root)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !EXCLUDED_DIRS.iter().any(|ex| name == ex)
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
    {
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
                        let errors = core_check_latex_katex_syntax(&content, &rel);
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
            let path = std::env::temp_dir().join(format!("verify_katex_{}_{}_{}", name, nanos, count));
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
    fn test_clean_katex_passes() {
        let tmp = TempDir::new("clean");
        let md = tmp.path.join("doc.md");
        let mut f = File::create(md).unwrap();
        writeln!(
            f,
            "# Title\n\nInline math: $`x = 1`$\n\n$$\n\\begin{{aligned}}\na &= b \\\\\nc &= d\n\\end{{aligned}}\n$$\n"
        )
        .unwrap();

        assert!(check_latex_katex_syntax(&tmp.path).is_ok());
    }

    #[test]
    fn test_unbalanced_display_math_fails() {
        let tmp = TempDir::new("unbalanced");
        let md = tmp.path.join("doc.md");
        let mut f = File::create(md).unwrap();
        writeln!(f, "# Title\n\n$$\nx = 1\n").unwrap();

        let res = check_latex_katex_syntax(&tmp.path);
        assert!(res.is_err());
        assert!(res.unwrap_err()[0].contains("Unbalanced $$"));
    }

    #[test]
    fn test_bare_alignment_outside_aligned_fails() {
        let tmp = TempDir::new("bare_amp");
        let md = tmp.path.join("doc.md");
        let mut f = File::create(md).unwrap();
        writeln!(f, "$$\nx & y = 1\n$$").unwrap();

        let res = check_latex_katex_syntax(&tmp.path);
        assert!(res.is_err());
        assert!(res.unwrap_err()[0].contains("Bare alignment operator '&'"));
    }
}
