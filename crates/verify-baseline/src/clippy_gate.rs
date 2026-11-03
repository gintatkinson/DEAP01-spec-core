//! Workspace `Cargo.toml` lint-table integrity and `cargo clippy --message-format=json` gate (`TD-23`).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Enforces deterministic Stage 1 static analysis and workspace lint-table integrity under
//! DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D tool qualification standards. Prevents
//! weakening of `[workspace.lints.rust]` or `[workspace.lints.clippy]` in `Cargo.toml` and parses
//! `cargo clippy --message-format=json` NDJSON compiler envelopes via `serde_json`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Lossless TOML AST Inspection: Parses `Cargo.toml` text via `toml_edit::DocumentMut` rather
//!   than line-based string matching, ensuring comments and formatting cannot spoof lint entries.
//! - Mandatory Deny Invariant: Every lint in [`MANDATORY_DENIED_CLIPPY_LINTS`] must resolve to
//!   `"deny"` or `"forbid"` inside `[workspace.lints.clippy]`, and `unsafe_code` must resolve to
//!   `"deny"` or `"forbid"` inside `[workspace.lints.rust]`.
//!
//! ## 3. Preconditions
//! - Callers provide valid UTF-8 `Cargo.toml` text or a valid workspace root path containing a
//!   `Cargo.toml` manifest.
//!
//! ## 4. Postconditions
//! - `check_cargo_toml_lint_table_integrity` returns `Ok(())` if and only if all mandatory lints
//!   are enforced at `"deny"` or `"forbid"` level and zero lint entries are weakened to `"allow"`
//!   or `"warn"`.
//! - `check_workspace_clippy_clean` returns all parsed [`RustcCompilerMessageEnvelope`] records
//!   when `cargo clippy --message-format=json` succeeds with zero warning/error diagnostics.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(|T| + |N|)$ where $|T|$ is the `Cargo.toml` byte length and $|N|$ is the
//!   NDJSON stream byte length.
//! - Space Complexity: $O(|T| + |M|)$ where $|M|$ is the number of parsed compiler message envelopes.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Malformed TOML syntax, missing `[workspace.lints.*]` tables, weakened lint levels, non-zero
//!   `cargo clippy` exit codes, or warning/error diagnostics fail closed with descriptive errors.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-VERIFY-CLIPPY-GATE], [TD-23], [DO-330-TQL-4-CLIPPY-GATE]

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use toml_edit::DocumentMut;

pub use deap_core::workspace::{
    run_stable_clippy_gate, validate_manifest_and_clippy_bundle, ClippyGateError,
    ClippyNdjsonFinding, ManifestBundleDigest, StableClippyGateConfig, StageGateReceipt,
    MANDATORY_DENIED_CLIPPY_LINTS,
};

/// Diagnostic code entry inside a rustc/clippy JSON compiler message.
/// Realises: [REQ-VERIFY-CLIPPY-GATE], [TD-23]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcDiagnosticCode {
    pub code: String,
}

/// Primary or secondary source span inside a rustc/clippy JSON compiler message.
/// Realises: [REQ-VERIFY-CLIPPY-GATE], [TD-23]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcDiagnosticSpan {
    pub file_name: String,
    pub line_start: usize,
    pub column_start: usize,
    #[serde(default)]
    pub is_primary: bool,
}

/// Inner diagnostic payload inside a `compiler-message` NDJSON envelope.
/// Realises: [REQ-VERIFY-CLIPPY-GATE], [TD-23]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcCompilerMessage {
    pub message: String,
    pub level: String,
    #[serde(default)]
    pub code: Option<RustcDiagnosticCode>,
    #[serde(default)]
    pub spans: Vec<RustcDiagnosticSpan>,
}

/// Top-level Cargo/rustc `--message-format=json` NDJSON envelope.
/// Realises: [REQ-VERIFY-CLIPPY-GATE], [TD-23]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcCompilerMessageEnvelope {
    pub reason: String,
    #[serde(default)]
    pub message: Option<RustcCompilerMessage>,
    #[serde(default)]
    pub success: Option<bool>,
}

/// Parses a newline-delimited stream of `cargo clippy --message-format=json` envelopes via `serde_json`.
/// Realises: [REQ-VERIFY-CLIPPY-GATE], [TD-23]
pub fn parse_rustc_ndjson_stream(
    ndjson_stdout: &str,
) -> Result<Vec<RustcCompilerMessageEnvelope>, String> {
    let mut out = Vec::new();
    for (idx, line) in ndjson_stdout.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let env: RustcCompilerMessageEnvelope = serde_json::from_str(trimmed).map_err(|err| {
            format!(
                "invalid rustc NDJSON on line {}: {err}",
                idx.saturating_add(1)
            )
        })?;
        out.push(env);
    }
    Ok(out)
}

/// Parses `cargo_toml_text` into a lossless `toml_edit::DocumentMut` AST without line-splitting.
/// Realises: [REQ-VERIFY-CLIPPY-GATE], [TD-01], [TD-23]
pub(crate) fn parse_toml_document_lossless(cargo_toml_text: &str) -> Result<DocumentMut, String> {
    cargo_toml_text
        .parse::<DocumentMut>()
        .map_err(|err| format!("failed to parse Cargo.toml via toml_edit::DocumentMut: {err}"))
}

/// Extracts the effective lint level (`"forbid"`, `"deny"`, `"warn"`, `"allow"`) from a `toml_edit::Item`.
/// Realises: [REQ-VERIFY-CLIPPY-GATE], [TD-23]
fn extract_toml_lint_level(item: &toml_edit::Item) -> Option<&str> {
    if let Some(level) = item.as_str() {
        return Some(level);
    }
    if let Some(inline_tbl) = item.as_inline_table() {
        return inline_tbl.get("level").and_then(|val| val.as_str());
    }
    if let Some(tbl) = item.as_table_like() {
        return tbl.get("level").and_then(|entry| entry.as_str());
    }
    None
}

/// Validates `[workspace.lints.rust]` and `[workspace.lints.clippy]` in `cargo_toml_text` using
/// `toml_edit::DocumentMut` (`TD-01`, `TD-23`).
/// Realises: [REQ-VERIFY-CLIPPY-GATE], [TD-01], [TD-23]
pub fn check_cargo_toml_lint_table_integrity(cargo_toml_text: &str) -> Result<(), Vec<String>> {
    let doc = parse_toml_document_lossless(cargo_toml_text).map_err(|err| vec![err])?;
    let mut violations = Vec::new();

    let Some(workspace_item) = doc.get("workspace") else {
        return Err(vec![
            "missing `[workspace]` table in Cargo.toml".to_string()
        ]);
    };
    let Some(lints_item) = workspace_item.get("lints") else {
        return Err(vec![
            "missing `[workspace.lints]` table in Cargo.toml".to_string()
        ]);
    };

    if let Some(rust_lints) = lints_item.get("rust").and_then(|item| item.as_table_like()) {
        match rust_lints.get("unsafe_code").and_then(extract_toml_lint_level) {
            Some("forbid" | "deny") => {}
            Some(other) => violations.push(format!(
                "[workspace.lints.rust].unsafe_code must be \"forbid\" or \"deny\", found \"{other}\""
            )),
            None => violations.push(
                "missing mandatory `unsafe_code = \"forbid\"` in `[workspace.lints.rust]`"
                    .to_string(),
            ),
        }
        for (lint_name, lint_item) in rust_lints.iter() {
            if let Some(level @ ("allow" | "warn")) = extract_toml_lint_level(lint_item) {
                violations.push(format!(
                    "[workspace.lints.rust].{lint_name} is weakened to \"{level}\""
                ));
            }
        }
    } else {
        violations.push("missing `[workspace.lints.rust]` table in Cargo.toml".to_string());
    }

    if let Some(clippy_lints) = lints_item
        .get("clippy")
        .and_then(|item| item.as_table_like())
    {
        for (lint_name, lint_item) in clippy_lints.iter() {
            if let Some(level @ ("allow" | "warn")) = extract_toml_lint_level(lint_item) {
                violations.push(format!(
                    "[workspace.lints.clippy].{lint_name} is weakened to \"{level}\""
                ));
            }
        }
        for mandatory_lint in MANDATORY_DENIED_CLIPPY_LINTS {
            match clippy_lints
                .get(mandatory_lint)
                .and_then(extract_toml_lint_level)
            {
                Some("deny" | "forbid") => {}
                Some(other) => violations.push(format!(
                    "[workspace.lints.clippy].{mandatory_lint} must be \"deny\" or \"forbid\", found \"{other}\""
                )),
                None => violations.push(format!(
                    "missing mandatory denied lint `{mandatory_lint}` in `[workspace.lints.clippy]`"
                )),
            }
        }
    } else {
        violations.push("missing `[workspace.lints.clippy]` table in Cargo.toml".to_string());
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Filters `cargo clippy --message-format=json` NDJSON envelopes into [`ClippyNdjsonFinding`] items (`TD-23`).
/// Realises: [REQ-VERIFY-CLIPPY-GATE], [TD-23]
pub fn filter_clippy_ndjson_stream(
    ndjson_stdout: &str,
) -> Result<Vec<ClippyNdjsonFinding>, String> {
    let envelopes = parse_rustc_ndjson_stream(ndjson_stdout)?;
    let mut findings = Vec::new();
    for env in envelopes {
        if env.reason == "compiler-message" {
            if let Some(msg) = env.message {
                if matches!(msg.level.as_str(), "error" | "warning") {
                    let lint_code = msg
                        .code
                        .map_or_else(|| "rustc".to_string(), |c| c.code);
                    let primary = msg.spans.iter().find(|s| s.is_primary);
                    findings.push(ClippyNdjsonFinding {
                        file_name: primary.map_or_else(String::new, |s| s.file_name.clone()),
                        line: primary.map_or(0, |s| s.line_start),
                        column: primary.map_or(0, |s| s.column_start),
                        lint_code,
                        message: msg.message,
                        suggested_replacement: None,
                    });
                }
            }
        }
    }
    Ok(findings)
}

/// Parses `cargo clippy --message-format=json` NDJSON output via `serde_json` and verifies zero
/// warning or error diagnostics (`TD-23`).
/// Realises: [REQ-VERIFY-CLIPPY-GATE], [TD-23]
pub fn verify_clippy_ndjson_clean(
    ndjson_stdout: &str,
) -> Result<Vec<RustcCompilerMessageEnvelope>, String> {
    let envelopes = parse_rustc_ndjson_stream(ndjson_stdout)?;
    let mut actionable_diagnostics = Vec::new();
    for env in &envelopes {
        if env.reason == "compiler-message" {
            if let Some(msg) = &env.message {
                if matches!(msg.level.as_str(), "error" | "warning") {
                    let code = msg
                        .code
                        .as_ref()
                        .map_or_else(|| "rustc".to_string(), |c| c.code.clone());
                    let loc = msg.spans.iter().find(|s| s.is_primary).map_or_else(
                        || "<unknown>".to_string(),
                        |s| format!("{}:{}:{}", s.file_name, s.line_start, s.column_start),
                    );
                    actionable_diagnostics.push(format!(
                        "[{}] {} at {}: {}",
                        msg.level, code, loc, msg.message
                    ));
                }
            }
        }
    }
    if actionable_diagnostics.is_empty() {
        Ok(envelopes)
    } else {
        Err(format!(
            "cargo clippy reported {} warning/error diagnostic(s):\n{}",
            actionable_diagnostics.len(),
            actionable_diagnostics.join("\n")
        ))
    }
}

/// Executes `cargo clippy --message-format=json` against `workspace_root`, parses all NDJSON
/// compiler envelopes via `serde_json`, and fails closed on any warning, error, or weakened
/// `Cargo.toml` lint table (`TD-23`).
/// Realises: [REQ-VERIFY-CLIPPY-GATE], [TD-23]
pub fn check_workspace_clippy_clean(
    workspace_root: &Path,
) -> Result<Vec<RustcCompilerMessageEnvelope>, String> {
    let manifest_path = workspace_root.join("Cargo.toml");
    let cargo_toml_text = std::fs::read_to_string(&manifest_path).map_err(|err| {
        format!(
            "failed to read workspace manifest `{}`: {err}",
            manifest_path.display()
        )
    })?;
    check_cargo_toml_lint_table_integrity(&cargo_toml_text).map_err(|violations| {
        format!(
            "workspace Cargo.toml lint table integrity failed:\n{}",
            violations.join("\n")
        )
    })?;

    let output = Command::new("cargo")
        .arg("clippy")
        .arg("--workspace")
        .arg("--all-targets")
        .arg("--offline")
        .arg("--message-format=json")
        .arg("--")
        .arg("-D")
        .arg("warnings")
        .current_dir(workspace_root)
        .output()
        .map_err(|err| format!("failed to spawn `cargo clippy --message-format=json`: {err}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let envelopes = verify_clippy_ndjson_clean(&stdout)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "`cargo clippy --message-format=json` exited with status {}: {}",
            output.status,
            stderr.trim()
        ));
    }
    Ok(envelopes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_workspace_cargo_toml() -> String {
        let mut lines = vec![
            "[workspace]".to_string(),
            "members = [\"crates/deap-core\"]".to_string(),
            "[workspace.lints.rust]".to_string(),
            "unsafe_code = \"forbid\"".to_string(),
            "[workspace.lints.clippy]".to_string(),
        ];
        for lint in MANDATORY_DENIED_CLIPPY_LINTS {
            lines.push(format!("{lint} = \"deny\""));
        }
        lines.join("\n")
    }

    #[test]
    fn rejects_weakened_workspace_clippy_table_and_filters_ndjson_spans() -> Result<(), String> {
        let valid_toml = valid_workspace_cargo_toml();
        assert!(check_cargo_toml_lint_table_integrity(&valid_toml).is_ok());
        let weakened = valid_toml.replace("unwrap_used = \"deny\"", "unwrap_used = \"allow\"");
        let Err(err) = check_cargo_toml_lint_table_integrity(&weakened) else {
            return Err("expected weakened unwrap_used to fail".to_string());
        };
        assert!(err.iter().any(|msg| msg.contains("unwrap_used")));
        let bad_unsafe = valid_toml.replace("unsafe_code = \"forbid\"", "unsafe_code = \"warn\"");
        let Err(unsafe_err) = check_cargo_toml_lint_table_integrity(&bad_unsafe) else {
            return Err("expected weakened unsafe_code to fail".to_string());
        };
        assert!(unsafe_err.iter().any(|msg| msg.contains("unsafe_code")));
        let ndjson = concat!(
            "{\"reason\":\"compiler-message\",\"message\":{\"message\":\"used unwrap()\",\"level\":\"error\",",
            "\"code\":{\"code\":\"clippy::unwrap_used\"},\"spans\":[{\"file_name\":\"crates/verify-baseline/src/lib.rs\",",
            "\"line_start\":42,\"column_start\":9,\"is_primary\":true}]}}\n",
            "{\"reason\":\"build-finished\",\"success\":false}\n"
        );
        let findings = filter_clippy_ndjson_stream(ndjson)?;
        let first = findings.first().ok_or("missing finding")?;
        assert_eq!(findings.len(), 1);
        assert_eq!(first.lint_code, "clippy::unwrap_used");
        assert_eq!(first.line, 42);
        assert!(verify_clippy_ndjson_clean(ndjson).is_err());
        let clean_ndjson = "{\"reason\":\"build-finished\",\"success\":true}\n";
        assert_eq!(verify_clippy_ndjson_clean(clean_ndjson)?.len(), 1);
        Ok(())
    }
}
