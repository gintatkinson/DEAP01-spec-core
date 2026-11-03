//! Check 34 audited module registry, split-token tables, and zero-regex scanner.
//!
//! Safety & Traceability: Realises: [REQ-CHECK-34-ZERO-REGEX-INVARIANT]
//! Contract: Enforces zero regular expressions across `crates/` and provides split-token helpers for Check 34.
//! Bounds & Faults: Single-pass directory and line walks; fails closed with sorted diagnostic vectors.

use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use super::clippy_ndjson_gate::{scan_cargo_toml_file, WorkspaceManifestForbidGate};
use super::struct_check_audit::audit_source_purity;
use super::struct_check_grounding::contains_bounded_token;

/// Zero-allocation `(prefix, suffix)` token pair for self-hosting Check 34 source scans.
/// Realises: [REQ-CHECK-34-SPLIT-TOKEN-MATCH]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SplitToken(pub(crate) &'static str, pub(crate) &'static str);

/// Specification pairing an audited module's path-match prefix with its canonical workspace stem.
/// Realises: [REQ-CHECK-34-AUDITED-MODULE-PATH]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AuditedModuleSpec {
    pub(crate) match_prefix: &'static str,
    pub(crate) workspace_stem: &'static str,
}

/// Complete ordered registry of Check 34 audited compiler modules (`<module>.rs` or `<module>/`).
/// Realises: [REQ-CHECK-34-AUDITED-MODULE-PATH]
pub(crate) const AUDITED_MODULES: &[AuditedModuleSpec] = &[
    AuditedModuleSpec {
        match_prefix: "checks/structural",
        workspace_stem: "crates/verify-baseline/src/checks/structural",
    },
    AuditedModuleSpec {
        match_prefix: "parity_gates",
        workspace_stem: "crates/verify-baseline/src/checks/parity_gates",
    },
    AuditedModuleSpec {
        match_prefix: "docstring_gate",
        workspace_stem: "crates/verify-baseline/src/checks/docstring_gate",
    },
    AuditedModuleSpec {
        match_prefix: "defect_dossier",
        workspace_stem: "crates/verify-baseline/src/defect_dossier",
    },
    AuditedModuleSpec {
        match_prefix: "deap-core/src/diagnostics",
        workspace_stem: "crates/deap-core/src/diagnostics",
    },
    AuditedModuleSpec {
        match_prefix: "deap-core/src/rules",
        workspace_stem: "crates/deap-core/src/rules",
    },
    AuditedModuleSpec {
        match_prefix: "deap-core/src/workspace",
        workspace_stem: "crates/deap-core/src/workspace",
    },
    AuditedModuleSpec {
        match_prefix: "deap-core/src/markdown",
        workspace_stem: "crates/deap-core/src/markdown",
    },
    AuditedModuleSpec {
        match_prefix: "deap-core/src/sysml_ast",
        workspace_stem: "crates/deap-core/src/sysml_ast",
    },
    AuditedModuleSpec {
        match_prefix: "deap-core/src/source_ast",
        workspace_stem: "crates/deap-core/src/source_ast",
    },
    AuditedModuleSpec {
        match_prefix: "compile-sysml/src/docstring",
        workspace_stem: "crates/compile-sysml/src/docstring",
    },
    AuditedModuleSpec {
        match_prefix: "ingest-sysml/src/translators/code",
        workspace_stem: "crates/ingest-sysml/src/translators/code",
    },
];

/// Split-token table detecting banned regular-expression crate imports and qualified paths.
/// Realises: [REQ-CHECK-34-ZERO-REGEX-INVARIANT]
pub(crate) const FORBIDDEN_REGEX_TOKENS: &[SplitToken] = &[
    SplitToken("Reg", "ex::"),
    SplitToken("reg", "ex::"),
    SplitToken("use reg", "ex;"),
    SplitToken("extern crate reg", "ex;"),
];

/// Split-token table for static string-slice types gated on `const`, `static`, or `=` bindings.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
pub(crate) const BINDING_STR_SLICE_TOKENS: &[SplitToken] =
    &[SplitToken("&[", "&str]"), SplitToken(": &[", "&str]")];

/// Split-token table for unconditionally forbidden static string-slice array and iterator types.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
pub(crate) const UNCONDITIONAL_STR_SLICE_TOKENS: &[SplitToken] = &[
    SplitToken("&[(", "&str"),
    SplitToken("[", "&str;"),
    SplitToken("const : &[", "&str]"),
    SplitToken("&'static [", "&'static str]"),
    SplitToken("[&'static ", "str;"),
    SplitToken("IntoIter<", "&'static str"),
];

/// Split-token table for fail-open `.filter_map(|...| ....ok())` closures.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
pub(crate) const FAIL_OPEN_FILTER_MAP_TOKENS: &[SplitToken] = &[
    SplitToken(".filter_map(|e| e.ok", "())"),
    SplitToken(".filter_map(|res| res.ok", "())"),
    SplitToken(".filter_map(|x| x.ok", "())"),
];

/// Split-token table for direct indexing immediately following lowercase string conversion.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
pub(crate) const DIRECT_LOWER_SLICE_TOKENS: &[SplitToken] = &[
    SplitToken(".to_lowercase()", "["),
    SplitToken(".to_ascii_lowercase()", "["),
];

/// Split-token table for range indexing on lowercase-named local bindings.
/// Realises: [REQ-CHECK-34-AUDIT-SOURCE-PURITY]
pub(crate) const LOWER_VAR_INDEX_TOKENS: &[SplitToken] =
    &[SplitToken("lower", "["), SplitToken("_lower", "[")];

/// Forbidden Cargo dependency name split across two slices to avoid self-scan false positives.
/// Realises: [REQ-CARGO-TOML-REGEX-SCAN]
pub(crate) const FORBIDDEN_CARGO_DEP_TOKEN: SplitToken = SplitToken("reg", "ex");

/// Checks whether `haystack` contains `prefix` immediately followed by `suffix` without allocation.
/// Realises: [REQ-CHECK-34-SPLIT-TOKEN-MATCH]
pub(crate) fn has_split_token(haystack: &str, prefix: &str, suffix: &str) -> bool {
    if prefix.is_empty() {
        return haystack.contains(suffix);
    }
    for (idx, _) in haystack.match_indices(prefix) {
        if haystack
            .get(idx.saturating_add(prefix.len())..)
            .is_some_and(|rest| rest.starts_with(suffix))
        {
            return true;
        }
    }
    false
}

/// Checks whether `haystack` matches at least one `(prefix, suffix)` pair in `tokens`.
/// Realises: [REQ-CHECK-34-SPLIT-TOKEN-MATCH]
pub(crate) fn any_split_token(haystack: &str, tokens: &[SplitToken]) -> bool {
    tokens
        .iter()
        .any(|tok| has_split_token(haystack, tok.0, tok.1))
}

/// Checks whether `line` contains `prefix` + `suffix` bounded on both sides by non-identifier bytes.
/// Realises: [REQ-CHECK-34-SPLIT-TOKEN-MATCH]
pub(crate) fn contains_bounded_split_token(line: &str, prefix: &str, suffix: &str) -> bool {
    if prefix.is_empty() {
        return contains_bounded_token(line, suffix);
    }
    let bytes = line.as_bytes();
    let total_len = prefix.len().saturating_add(suffix.len());
    for (idx, _) in line.match_indices(prefix) {
        let has_suffix = line
            .get(idx.saturating_add(prefix.len())..)
            .is_some_and(|rest| rest.starts_with(suffix));
        if !has_suffix {
            continue;
        }
        let before_ok = match idx.checked_sub(1).and_then(|prev| bytes.get(prev)) {
            Some(&b) => !b.is_ascii_alphanumeric() && b != b'_' && b != b'-',
            None => true,
        };
        let after_ok = match bytes.get(idx.saturating_add(total_len)) {
            Some(&b) => !b.is_ascii_alphanumeric() && b != b'_' && b != b'-',
            None => true,
        };
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// Checks whether `path` matches `<prefix>.rs` or a subpath under `<prefix>/` at a path boundary.
/// Realises: [REQ-CHECK-34-AUDITED-MODULE-PATH]
pub(crate) fn matches_module_file_or_dir(path: &str, prefix: &str) -> bool {
    if prefix.is_empty() {
        return false;
    }
    let bytes = path.as_bytes();
    for (idx, _) in path.match_indices(prefix) {
        let before_ok = match idx.checked_sub(1).and_then(|prev| bytes.get(prev)) {
            Some(&b) => b == b'/' || b == b'\\',
            None => true,
        };
        if !before_ok {
            continue;
        }
        let tail = path.get(idx.saturating_add(prefix.len())..).unwrap_or("");
        if tail == ".rs" || tail.starts_with('/') || tail.starts_with('\\') {
            return true;
        }
    }
    false
}

/// Determines whether a Rust source path belongs to a Check 34 audited module file or directory.
/// Realises: [REQ-CHECK-34-AUDITED-MODULE-PATH]
pub(crate) fn is_audited_source_path(rel: &str) -> bool {
    if !rel.ends_with(".rs") {
        return false;
    }
    AUDITED_MODULES
        .iter()
        .any(|spec| matches_module_file_or_dir(rel, spec.match_prefix))
}

/// Determines whether strict fail-open I/O match arm rules apply to `file_rel`.
/// Realises: [REQ-CHECK-34-STRICT-IO-ARMS-SCOPE]
pub(crate) fn enforces_strict_io_arms(file_rel: &str) -> bool {
    !matches_module_file_or_dir(file_rel, "sysml_ast")
        && !matches_module_file_or_dir(file_rel, "parity_gates")
}

/// Check 34: Zero Regex Invariant & Self-Hosting Compiler Purity Gate.
/// Realises: [REQ-CHECK-34-ZERO-REGEX-INVARIANT].
pub fn check_zero_regex(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    let root_cargo = repo_root.join("Cargo.toml");
    if root_cargo.is_file() {
        scan_cargo_toml_file(&root_cargo, repo_root, &mut violations);
        if let Err(errs) = WorkspaceManifestForbidGate::verify(repo_root) {
            violations.extend(errs);
        }
    }

    let crates_dir = repo_root.join("crates");
    if crates_dir.is_dir() {
        for entry_res in WalkDir::new(&crates_dir).into_iter().filter_entry(|e| {
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
                    violations.push(format!("Check 34 I/O error: {}", err));
                    continue;
                }
            };
            if entry.file_type().is_file() {
                scan_crates_entry_for_check_34(entry.path(), repo_root, &mut violations);
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

/// Dispatches a single file inside `crates/` to `Cargo.toml` or `.rs` Check 34 purity auditing.
/// Realises: [REQ-CHECK-34-ZERO-REGEX-INVARIANT]
fn scan_crates_entry_for_check_34(path: &Path, repo_root: &Path, violations: &mut Vec<String>) {
    if path.file_name().is_some_and(|name| name == "Cargo.toml") {
        scan_cargo_toml_file(path, repo_root, violations);
        return;
    }
    if !path.extension().is_some_and(|ext| ext == "rs") {
        return;
    }
    let rel = path
        .strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string();
    match fs::read_to_string(path) {
        Ok(content) => {
            let is_audited = is_audited_source_path(&rel);
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
