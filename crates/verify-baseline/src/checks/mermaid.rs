//! Check 13B: Mermaid diagram syntax validation across markdown files.

use deap_core::markdown::check_mermaid_syntax as core_check_mermaid_syntax;
use deap_core::rules::EXCLUDED_DIRS;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Check 13B: Mermaid diagram syntax validation across all .md files in the repository.
///
/// Uses `deap_core::markdown::check_mermaid_syntax` to validate:
/// - Unclosed ```mermaid code fences.
/// - Mandatory diagram type header on first non-comment line.
/// - Unquoted comparison operators `<` and `>` in labels.
/// - Layout ergonomics and diagram boundaries.
pub fn check_mermaid_syntax(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut all_errors = Vec::new();

    for entry in WalkDir::new(repo_root)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !EXCLUDED_DIRS.iter().any(|&ex| name == ex)
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
}
