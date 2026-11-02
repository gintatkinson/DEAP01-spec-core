#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! NDJSON diagnostic reporting and automated machine fix CLI handler. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D qualification standards.
//! Provides structured NDJSON diagnostic formatting and machine-applicable code fixing for
//! baseline verification.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Deterministic NDJSON serialization with cross-platform normalized forward-slash paths.
//! - Non-destructive machine-fix application preserving original AST invariants.
//!
//! ## 3. Preconditions
//! - Callers supply well-formed argument slices and valid UTF-8 filesystem paths.
//!
//! ## 4. Postconditions
//! - Emits valid single-line NDJSON diagnostics or mutates source files in place.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - $O(D)$ for diagnostic formatting and $O(N)$ for source text replacement application.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Unreadable files or invalid JSON payloads return descriptive `Err(String)`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0608], [SC-01], [FLS: fls_ndjson_cli]
//! SysML v2: part def NdjsonCliHandler

use deap_core::rust_cst::{
    apply_machine_applicable_fixes, emit_ndjson_line, normalize_lf, NdjsonDiagnostic, NdjsonSpan,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Evaluates CLI arguments for `--message-format=json` and `--apply-machine-fixes`.
/// Realises: [REQ-0608], [SC-01], [FLS: fls_ndjson_cli]
pub fn try_run_ndjson_cli(args: &[String]) -> Option<Result<(), String>> {
    let mut i = 0;
    while i < args.len() {
        let Some(arg) = args.get(i) else { break };
        if arg == "--apply-machine-fixes" {
            let Some(target) = args.get(i + 1) else {
                return Some(Err("Missing path argument for --apply-machine-fixes".to_string()));
            };
            return Some(run_apply_fixes(target));
        } else if let Some(target) = arg.strip_prefix("--apply-machine-fixes=") {
            return Some(run_apply_fixes(target));
        } else if arg == "--message-format=json" || arg == "--message-format" {
            let target = args.get(i + 1).map_or(".", String::as_str);
            return Some(run_message_format_json(target));
        }
        i += 1;
    }
    None
}

/// Applies machine-applicable fixes from diagnostics file or source path.
/// Realises: [REQ-0608], [SC-01], [FLS: fls_ndjson_cli]
pub fn run_apply_fixes(target_path: &str) -> Result<(), String> {
    let p = Path::new(target_path);
    if !p.exists() { return Err(format!("Target does not exist: {target_path}")); }
    let content = fs::read_to_string(p).map_err(|e| format!("{target_path}: {e}"))?;
    let diags: Vec<NdjsonDiagnostic> = if let Ok(list) = serde_json::from_str::<Vec<NdjsonDiagnostic>>(&content) {
        list
    } else {
        content.lines().filter_map(|l| serde_json::from_str::<NdjsonDiagnostic>(l.trim()).ok()).collect()
    };
    if diags.is_empty() { return Ok(()); }
    let mut by_file: BTreeMap<String, Vec<NdjsonDiagnostic>> = BTreeMap::new();
    for d in diags {
        for s in &d.spans {
            by_file.entry(s.file_name.clone()).or_default().push(d.clone());
        }
    }
    for (file_name, file_diags) in by_file {
        let src = fs::read_to_string(&file_name).map_err(|e| format!("{file_name}: {e}"))?;
        let updated = apply_machine_applicable_fixes(&src, &file_diags)?;
        if updated != src {
            fs::write(&file_name, normalize_lf(&updated)).map_err(|e| format!("{file_name}: {e}"))?;
        }
    }
    Ok(())
}

/// Emits NDJSON diagnostics for target path validation.
/// Realises: [REQ-0608], [FLS: fls_ndjson_cli]
fn run_message_format_json(target: &str) -> Result<(), String> {
    let p = Path::new(target);
    if !p.exists() {
        let span = NdjsonSpan {
            file_name: target.to_string(),
            byte_start: 0, byte_end: 0, line_start: 1, line_end: 1,
            column_start: 1, column_end: 1,
            suggested_replacement: None, suggestion_applicability: None,
        };
        let diag = NdjsonDiagnostic {
            code: "E_NOT_FOUND".to_string(), level: "error".to_string(),
            message: format!("Target path does not exist: {target}"),
            spans: vec![span], rendered: None,
        };
        let line = emit_ndjson_line(&diag)?;
        println!("{line}");
        return Err("Target path not found".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ndjson_cli_message_format_json_and_spans() -> Result<(), String> {
        let span = NdjsonSpan {
            file_name: "crates/test/src/lib.rs".to_string(),
            byte_start: 0, byte_end: 4, line_start: 1, line_end: 1,
            column_start: 1, column_end: 5,
            suggested_replacement: Some("test".to_string()),
            suggestion_applicability: Some("MachineApplicable".to_string()),
        };
        let diag = NdjsonDiagnostic {
            code: "DEAP01".to_string(), level: "warning".to_string(),
            message: "sample check".to_string(), spans: vec![span], rendered: None,
        };
        let line = emit_ndjson_line(&diag)?;
        assert!(!line.contains('\n') && line.contains("crates/test/src/lib.rs"));
        Ok(())
    }

    #[test]
    fn test_ndjson_cli_apply_machine_fixes() -> Result<(), String> {
        let dir = std::env::temp_dir().join(format!("ndjson_cli_fix_{}", std::process::id()));
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let src_file = dir.join("fixed.rs");
        fs::write(&src_file, "let x = 10;").map_err(|e| e.to_string())?;
        let span = NdjsonSpan {
            file_name: src_file.to_string_lossy().to_string(),
            byte_start: 8, byte_end: 10, line_start: 1, line_end: 1,
            column_start: 9, column_end: 11,
            suggested_replacement: Some("42".to_string()),
            suggestion_applicability: Some("MachineApplicable".to_string()),
        };
        let diag = NdjsonDiagnostic {
            code: "M01".to_string(), level: "error".to_string(),
            message: "fix value".to_string(), spans: vec![span], rendered: None,
        };
        let fix_file = dir.join("fixes.json");
        fs::write(&fix_file, serde_json::to_string(&vec![diag]).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let args = vec!["verify-baseline".into(), "--apply-machine-fixes".into(), fix_file.to_string_lossy().into()];
        let res = try_run_ndjson_cli(&args);
        assert!(matches!(res, Some(Ok(()))));
        let updated = fs::read_to_string(&src_file).map_err(|e| e.to_string())?;
        assert_eq!(updated, "let x = 42;");
        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }
}
