//! Check 33 zero Python scripts and operational Python/Pytest reference gate.
//!
//! Safety & Traceability: Realises: [REQ-CHECK-33-ZERO-PYTHON-SCRIPTS]
//! Contract: Enforces zero Python artifacts on disk and zero operational Python/Pytest references in governed text files.
//! Bounds & Faults: Single-pass directory and line walks; fails closed with sorted diagnostic vectors.

use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Check 33: Zero Python Scripts & Operational Python/Pytest Reference Invariant.
/// Realises: [REQ-CHECK-33-ZERO-PYTHON-SCRIPTS].
pub fn check_zero_python_scripts(repo_root: &Path) -> Result<(), Vec<String>> {
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
                violations.push(format!("Check 33 I/O error: {}", err));
                continue;
            }
        };
        check_entry_for_python_artifact(&entry, repo_root, &mut violations);
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

/// Inspects a single directory entry for forbidden Python directories, bytecode files, or manifests.
/// Realises: [REQ-CHECK-33-ZERO-PYTHON-SCRIPTS]
fn check_entry_for_python_artifact(
    entry: &walkdir::DirEntry,
    repo_root: &Path,
    violations: &mut Vec<String>,
) {
    let lower_name = entry.file_name().to_string_lossy().to_ascii_lowercase();
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
        return;
    }
    if entry.file_type().is_file() {
        let is_py_ext = lower_name.ends_with(".py")
            || entry.path().extension().is_some_and(|ext| {
                matches!(
                    ext.to_string_lossy().to_ascii_lowercase().as_str(),
                    "py" | "pyc" | "pyo" | "pyd" | "pyi" | "ipynb"
                )
            });
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
            .is_some_and(|n| n.to_string_lossy() == ".pipeline");
        for entry_res in WalkDir::new(dir_path).into_iter().filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                if is_pipeline_root && name == "defects" {
                    return false;
                }
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
                    violations.push(format!("Check 33 I/O error: {}", err));
                    continue;
                }
            };
            if entry.file_type().is_file() && is_governed_python_scan_extension(entry.path()) {
                scan_file_for_python_references(entry.path(), repo_root, violations);
            }
        }
    }
    for file_path in [repo_root.join("README.md"), repo_root.join("AGENTS.md")] {
        if file_path.is_file() {
            scan_file_for_python_references(&file_path, repo_root, violations);
        }
    }
}

/// Evaluates whether `path` carries a governed text extension while excluding `*.digest.json` sidecars.
/// Realises: [REQ-CHECK-33-GOVERNED-EXTENSIONS]
pub(crate) fn is_governed_python_scan_extension(path: &Path) -> bool {
    if let Some(fname) = path
        .file_name()
        .map(|n| n.to_string_lossy().to_ascii_lowercase())
    {
        if fname.ends_with(".digest.json") || fname.ends_with("-digest.json") {
            return false;
        }
    }
    path.extension().is_some_and(|e| {
        matches!(
            e.to_string_lossy().to_ascii_lowercase().as_str(),
            "md" | "yaml" | "yml" | "json" | "toml"
        )
    })
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
        let ascii_folded = raw_line.to_ascii_lowercase();
        if !is_python_prohibition_clause(&ascii_folded) {
            if let Some(token_label) = detect_operational_python_token(&ascii_folded) {
                violations.push(format!(
                    "{}:{}: forbidden operational Python/Pytest reference ({}): {}",
                    rel,
                    idx.saturating_add(1),
                    token_label,
                    raw_line.trim()
                ));
            }
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
        let after_ok = match bytes.get(idx.saturating_add(word.len())) {
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
        let is_boundary = match bytes.get(idx.saturating_add(3)) {
            Some(&b) => !b.is_ascii_alphanumeric() && b != b'_',
            None => true,
        };
        if is_boundary {
            return true;
        }
    }
    false
}
