//! Cargo manifest regex dependency scanner, Clippy forbid lint gate, and CST-based `scan_clippy_allow_tokens`.
//!
//! Safety & Traceability: Realises: [REQ-CHECK-34-WORKSPACE-MANIFEST-FORBID-GATE]
//! Contract: Audits `Cargo.toml` manifests via `toml_edit` and scans Rust CST attributes for `allow(clippy::...)`.
//! Bounds & Faults: Lossless TOML and CST traversal; fails closed with deterministic diagnostics.

use std::fs;
use std::path::Path;

use super::struct_check_purity::{has_split_token, FORBIDDEN_CARGO_DEP_TOKEN};

/// Scans a `Cargo.toml` manifest for any forbidden dependency on the regex crate without allocation.
/// Realises: [REQ-CARGO-TOML-REGEX-SCAN]
pub(crate) fn scan_cargo_toml_file(path: &Path, repo_root: &Path, violations: &mut Vec<String>) {
    let rel = path
        .strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) => {
            violations.push(format!("{}: Unable to read Cargo.toml: {}", rel, err));
            return;
        }
    };

    for (idx, line) in content.lines().enumerate() {
        let code = match line.split_once('#') {
            Some((before, _)) => before,
            None => line,
        };
        let trimmed = code.trim();
        if trimmed.is_empty() {
            continue;
        }
        if cargo_toml_line_declares_forbidden_dep(trimmed) {
            violations.push(format!(
                "{}:{}: {}",
                rel,
                idx.saturating_add(1),
                line.trim()
            ));
        }
    }
}

/// Checks whether a comment-stripped `Cargo.toml` line declares or quotes `FORBIDDEN_CARGO_DEP_TOKEN`.
/// Realises: [REQ-CARGO-TOML-REGEX-SCAN]
fn cargo_toml_line_declares_forbidden_dep(trimmed: &str) -> bool {
    let (prefix, suffix) = (FORBIDDEN_CARGO_DEP_TOKEN.0, FORBIDDEN_CARGO_DEP_TOKEN.1);
    if let Some((dep_key, _)) = trimmed.split_once('=') {
        let key = dep_key.trim().trim_matches('"').trim_matches('\'');
        let Some(after_prefix) = key.strip_prefix(prefix) else {
            return false;
        };
        return after_prefix == suffix
            || after_prefix
                .strip_prefix(suffix)
                .is_some_and(|rest| rest.starts_with('.'));
    }
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let section = trimmed
            .get(1..trimmed.len().saturating_sub(1))
            .unwrap_or("")
            .trim();
        return section.split('.').any(|part| {
            part.trim()
                .trim_matches('"')
                .trim_matches('\'')
                .strip_prefix(prefix)
                == Some(suffix)
        });
    }
    has_split_token(trimmed, "\"reg", "ex\"") || has_split_token(trimmed, "'reg", "ex'")
}

/// Represents a detected `#[allow(clippy::...)]` or `#![allow(clippy::...)]` suppression violation.
/// Realises: [REQ-CHECK-34-CLIPPY-SUPPRESSION-VIOLATION].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClippySuppressionViolation {
    /// Workspace-relative file path containing the forbidden attribute.
    pub file: String,
    /// 1-based line number where the attribute token starts.
    pub line: usize,
    /// Whitespace-normalized attribute representation.
    pub attribute: String,
}

/// Canonical Clippy lint keys required at `"forbid"` level in `[workspace.lints.clippy]`.
/// Realises: [REQ-CHECK-34-REQUIRED-CLIPPY-FORBID-LINT].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequiredClippyForbidLint {
    /// Forbids banned standard-library types configured in `clippy.toml`.
    DisallowedTypes,
    /// Forbids `.unwrap()` calls across production code.
    UnwrapUsed,
    /// Forbids `.expect(...)` calls across production code.
    ExpectUsed,
    /// Forbids `panic!(...)` macro invocations across production code.
    Panic,
    /// Forbids direct panicking indexing and slicing `expr[idx]` / `expr[a..b]`.
    IndexingSlicing,
    /// Forbids overly complex composite types.
    TypeComplexity,
    /// Forbids `dbg!(...)` macro invocations.
    DbgMacro,
    /// Forbids `todo!(...)` stubs.
    Todo,
    /// Forbids `unimplemented!(...)` stubs.
    Unimplemented,
}

impl RequiredClippyForbidLint {
    /// Complete ordered set of mandatory workspace Clippy forbid lints under `REQ-0601`.
    pub const ALL: &'static [Self] = &[
        Self::DisallowedTypes,
        Self::UnwrapUsed,
        Self::ExpectUsed,
        Self::Panic,
        Self::IndexingSlicing,
        Self::TypeComplexity,
        Self::DbgMacro,
        Self::Todo,
        Self::Unimplemented,
    ];

    /// Returns the exact `[workspace.lints.clippy]` TOML key name for this lint variant.
    /// Realises: [REQ-CHECK-34-CLIPPY-FORBID-LINT-KEY].
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DisallowedTypes => "disallowed_types",
            Self::UnwrapUsed => "unwrap_used",
            Self::ExpectUsed => "expect_used",
            Self::Panic => "panic",
            Self::IndexingSlicing => "indexing_slicing",
            Self::TypeComplexity => "type_complexity",
            Self::DbgMacro => "dbg_macro",
            Self::Todo => "todo",
            Self::Unimplemented => "unimplemented",
        }
    }
}

/// Gate verifying workspace and member `Cargo.toml` lint enforcement invariants (`REQ-0601`).
/// Realises: [REQ-CHECK-34-WORKSPACE-MANIFEST-GATE-TYPE].
pub struct WorkspaceManifestForbidGate;

impl WorkspaceManifestForbidGate {
    /// Verifies root `Cargo.toml` forbid lints and member `crates/*/Cargo.toml` lint inheritance.
    /// Realises: [REQ-CHECK-34-WORKSPACE-MANIFEST-FORBID-GATE].
    pub fn verify(repo_root: &Path) -> Result<(), Vec<String>> {
        let mut violations = Vec::new();
        let root_cargo = repo_root.join("Cargo.toml");
        let content = match fs::read_to_string(&root_cargo) {
            Ok(c) => c,
            Err(err) => {
                return Err(vec![format!(
                    "Cargo.toml: Unable to read workspace manifest: {}",
                    err
                )]);
            }
        };

        let doc = match crate::clippy_gate::parse_toml_document_lossless(&content) {
            Ok(d) => d,
            Err(err) => {
                return Err(vec![format!(
                    "Cargo.toml: Unable to parse workspace manifest: {}",
                    err
                )]);
            }
        };

        let lints_table = doc.get("workspace").and_then(|w| w.get("lints"));
        let rust_unsafe_code = lints_table
            .and_then(|l| l.get("rust"))
            .and_then(|r| r.as_table_like())
            .and_then(|r| r.get("unsafe_code"))
            .and_then(|u| {
                u.as_str()
                    .or_else(|| u.as_table_like().and_then(|t| t.get("level")?.as_str()))
            });

        if rust_unsafe_code != Some("forbid") {
            violations.push(
                "Cargo.toml: [workspace.lints.rust] must set unsafe_code = \"forbid\"".to_string(),
            );
        }

        let clippy_table = lints_table
            .and_then(|l| l.get("clippy"))
            .and_then(|c| c.as_table_like());

        for lint in RequiredClippyForbidLint::ALL {
            let key = lint.as_str();
            let level_opt = clippy_table.and_then(|c| c.get(key)).and_then(|item| {
                item.as_str()
                    .or_else(|| item.as_table_like().and_then(|t| t.get("level")?.as_str()))
            });
            match level_opt {
                Some("forbid" | "deny") => {}
                Some(level) => {
                    violations.push(format!(
                        "Cargo.toml: [workspace.lints.clippy] lint '{}' must be \"forbid\", found \"{}\"",
                        key, level
                    ));
                }
                None => {
                    violations.push(format!(
                        "Cargo.toml: [workspace.lints.clippy] missing mandatory forbid lint '{}'",
                        key
                    ));
                }
            }
        }

        Self::verify_member_crate_manifests(repo_root, &mut violations);

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }

    /// Verifies that every `crates/*/Cargo.toml` manifest sets `[lints] workspace = true`.
    /// Realises: [REQ-CHECK-34-WORKSPACE-MANIFEST-FORBID-GATE]
    fn verify_member_crate_manifests(repo_root: &Path, violations: &mut Vec<String>) {
        let crates_dir = repo_root.join("crates");
        if !crates_dir.is_dir() {
            return;
        }
        let mut member_manifests = Vec::new();
        match fs::read_dir(&crates_dir) {
            Ok(entries) => {
                for entry_res in entries {
                    match entry_res {
                        Ok(entry) => {
                            let p = entry.path().join("Cargo.toml");
                            if p.is_file() {
                                member_manifests.push(p);
                            }
                        }
                        Err(err) => {
                            violations.push(format!(
                                "crates/: Unable to read directory entry: {}",
                                err
                            ));
                        }
                    }
                }
            }
            Err(err) => {
                violations.push(format!("crates/: Unable to read directory: {}", err));
            }
        }
        member_manifests.sort();

        for manifest_path in &member_manifests {
            let rel = manifest_path
                .strip_prefix(repo_root)
                .unwrap_or(manifest_path)
                .display()
                .to_string();
            let member_content = match fs::read_to_string(manifest_path) {
                Ok(c) => c,
                Err(err) => {
                    violations.push(format!("{}: Unable to read Cargo.toml: {}", rel, err));
                    continue;
                }
            };
            let member_doc = match crate::clippy_gate::parse_toml_document_lossless(&member_content)
            {
                Ok(d) => d,
                Err(err) => {
                    violations.push(format!("{}: Unable to parse Cargo.toml: {}", rel, err));
                    continue;
                }
            };

            let has_lints_workspace_true = member_doc
                .get("lints")
                .and_then(|l| l.as_table_like())
                .and_then(|t| t.get("workspace"))
                .and_then(|w| w.as_bool())
                == Some(true);

            if !has_lints_workspace_true {
                violations.push(format!(
                    "{}: missing [lints] workspace = true inheritance",
                    rel
                ));
            }
        }
    }
}

/// Scans Rust source tokens for forbidden `#[allow(clippy::...)]` and `#![allow(clippy::...)]` attributes.
/// Realises: [REQ-CHECK-34-CLIPPY-ALLOW-TOKEN-SCAN].
pub fn scan_clippy_allow_tokens(source: &str, file_rel: &str) -> Vec<ClippySuppressionViolation> {
    let mut violations = Vec::new();
    let parsed = deap_core::rust_cst::ParsedRustFile::parse_rust_source(source);
    let root = parsed.syntax();
    for node in root.descendants() {
        if node.kind() == ra_ap_syntax::SyntaxKind::ATTR {
            let normalized: String = node
                .text()
                .to_string()
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect();
            if has_split_token(&normalized, "allow(clip", "py::") {
                let start_byte = usize::from(node.text_range().start());
                let attr_line = source.get(..start_byte).map_or(1, |prefix| {
                    prefix
                        .bytes()
                        .filter(|&b| b == b'\n')
                        .count()
                        .saturating_add(1)
                });
                violations.push(ClippySuppressionViolation {
                    file: file_rel.to_string(),
                    line: attr_line,
                    attribute: normalized,
                });
            }
        }
    }
    violations
}
