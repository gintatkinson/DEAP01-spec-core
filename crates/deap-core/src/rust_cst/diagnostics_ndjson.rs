#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Lossless NDJSON diagnostics and human-readable reporting. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A and ISO 26262 ASIL D qualification standards.
//! Provides rustc-compatible NDJSON diagnostic interchange and ariadne report rendering.
//!
//! ## 2. Invariants & Mathematical Properties
//! - File paths in NDJSON output are normalized with `/` separators and no redundant prefixes.
//! - Single-line NDJSON guarantees: each diagnostic emits exactly one JSON record without raw newlines.
//! - Machine-applicable fixes apply non-overlapping replacements deterministically.
//!
//! ## 3. Preconditions
//! - Source text is valid UTF-8 and spans reference byte offsets within source bounds.
//!
//! ## 4. Postconditions
//! - Emits structured JSON diagnostics or applies machine fixes without unhandled panics.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Serialization: $O(N)$ linear in diagnostic field length.
//! - Fix application: $O(M \log M)$ where $M$ is the number of applicable fix spans.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Malformed spans, overlapping edits, or serialization errors return structured `Err(String)`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0608], [SC-01], [FLS: fls_cst_diagnostics_ndjson]
//! SysML v2: part def RustCstDiagnosticsNdjson

use ariadne::{Label, Report, ReportKind, Source};

/// Source span anchored to a file and byte/line coordinates.
/// Realises: [REQ-0608], [SC-01]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NdjsonSpan {
    pub file_name: String,
    pub byte_start: usize,
    pub byte_end: usize,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub suggested_replacement: Option<String>,
    pub suggestion_applicability: Option<String>,
}

/// Structured rustc-compatible NDJSON diagnostic item.
/// Realises: [REQ-0608], [SC-01]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NdjsonDiagnostic {
    pub code: String,
    pub level: String,
    pub message: String,
    pub spans: Vec<NdjsonSpan>,
    pub rendered: Option<String>,
}

/// Serializes a diagnostic as a single-line JSON string with normalized paths.
/// Realises: [REQ-0608], [SC-01], [FLS: fls_cst_diagnostics_ndjson]
pub fn emit_ndjson_line(diag: &NdjsonDiagnostic) -> Result<String, String> {
    let mut normalized = diag.clone();
    for span in &mut normalized.spans {
        span.file_name = crate::rust_cst::vfs::normalize_rel_path(&span.file_name);
    }
    let serialized = serde_json::to_string(&normalized).map_err(|e| e.to_string())?;
    Ok(serialized.replace(['\r', '\n'], ""))
}

/// Renders a deterministic human-readable ariadne diagnostic report.
/// Realises: [REQ-0608], [SC-01], [FLS: fls_cst_diagnostics_ndjson]
pub fn render_ariadne_diagnostic(diag: &NdjsonDiagnostic, source: &str) -> String {
    let kind = match diag.level.to_ascii_lowercase().as_str() {
        "warning" => ReportKind::Warning,
        "advice" | "note" | "help" => ReportKind::Advice,
        _ => ReportKind::Error,
    };
    let (file_id, p_start, p_end) = match diag.spans.first() {
        Some(s) => (s.file_name.as_str(), s.byte_start, s.byte_end.max(s.byte_start)),
        None => ("source", 0, 0),
    };

    let mut builder = Report::build(kind, (file_id, p_start..p_end))
        .with_code(&diag.code)
        .with_message(&diag.message);

    for span in &diag.spans {
        let range = span.byte_start..span.byte_end.max(span.byte_start);
        let msg = match &span.suggested_replacement {
            Some(repl) => format!("suggestion: {repl}"),
            None => diag.message.clone(),
        };
        builder.add_label(Label::new((span.file_name.as_str(), range)).with_message(msg));
    }

    let report = builder.finish();
    let mut buffer = Vec::new();
    if report.write((file_id, Source::from(source)), &mut buffer).is_err() {
        return format!("{}: [{}] {}", diag.level, diag.code, diag.message);
    }
    String::from_utf8_lossy(&buffer).to_string()
}

/// Applies all machine-applicable fixes from diagnostic spans to the source text.
/// Realises: [REQ-0608], [SC-01], [FLS: fls_cst_diagnostics_ndjson]
pub fn apply_machine_applicable_fixes(
    source: &str,
    diags: &[NdjsonDiagnostic],
) -> Result<String, String> {
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    for diag in diags {
        for span in &diag.spans {
            if span.suggestion_applicability.as_deref() == Some("MachineApplicable") {
                if let Some(replacement) = &span.suggested_replacement {
                    edits.push((span.byte_start, span.byte_end, replacement.clone()));
                }
            }
        }
    }
    crate::rust_cst::doc_edit::apply_non_overlapping_edits(source, &edits)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_span(file: &str, (s, e): (usize, usize), repl: Option<&str>, app: Option<&str>) -> NdjsonSpan {
        NdjsonSpan {
            file_name: file.to_string(), byte_start: s, byte_end: e,
            line_start: 1, line_end: 1, column_start: s + 1, column_end: e + 1,
            suggested_replacement: repl.map(ToString::to_string),
            suggestion_applicability: app.map(ToString::to_string),
        }
    }

    #[test]
    fn test_emit_ndjson_line_single_line_and_forward_slash_paths() -> Result<(), String> {
        let span = mk_span(".\\crates\\foo\\src\\lib.rs", (0, 10), Some("x\ny"), Some("MachineApplicable"));
        let diag = NdjsonDiagnostic {
            code: "DEAP01".to_string(), level: "error".to_string(),
            message: "msg\nline2".to_string(), spans: vec![span], rendered: None,
        };

        let line = emit_ndjson_line(&diag)?;
        if line.contains('\n') || line.contains('\r') {
            return Err("emitted NDJSON line must not contain unescaped newlines".to_string());
        }
        if !line.contains("crates/foo/src/lib.rs") || line.contains(".\\") || line.contains("./") {
            return Err("emitted line must have forward-slash normalized path".to_string());
        }

        let parsed: NdjsonDiagnostic =
            serde_json::from_str(&line).map_err(|e| format!("parse error: {e}"))?;
        let first = parsed.spans.first().ok_or_else(|| "missing parsed span".to_string())?;
        if first.file_name != "crates/foo/src/lib.rs" {
            return Err(format!("unexpected file_name: {}", first.file_name));
        }
        Ok(())
    }

    #[test]
    fn test_render_ariadne_diagnostic_contains_code_and_message() -> Result<(), String> {
        let source = "fn compute() -> i32 { \"wrong type\" }";
        let span = mk_span("test.rs", (22, 34), Some("42"), Some("MachineApplicable"));
        let diag = NdjsonDiagnostic {
            code: "E0308".to_string(), level: "error".to_string(),
            message: "mismatched types in function body".to_string(),
            spans: vec![span], rendered: None,
        };

        let rendered = render_ariadne_diagnostic(&diag, source);
        if !rendered.contains("E0308") || !rendered.contains("mismatched types in function body") {
            return Err("rendered output missing diagnostic code or message".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_apply_machine_applicable_fixes() -> Result<(), String> {
        let source = "let alpha = 10; let beta = 20;";
        let s1 = mk_span("test.rs", (12, 14), Some("42"), Some("MachineApplicable"));
        let s2 = mk_span("test.rs", (27, 29), Some("99"), Some("MachineApplicable"));
        let s3 = mk_span("test.rs", (4, 9), Some("gamma"), Some("MaybeIncorrect"));

        let diags = vec![
            NdjsonDiagnostic {
                code: "W01".into(), level: "warning".into(),
                message: "update alpha".into(), spans: vec![s1], rendered: None,
            },
            NdjsonDiagnostic {
                code: "W02".into(), level: "warning".into(),
                message: "update beta".into(), spans: vec![s2, s3], rendered: None,
            },
        ];

        let fixed = apply_machine_applicable_fixes(source, &diags)?;
        if fixed != "let alpha = 42; let beta = 99;" {
            return Err(format!("unexpected fixed source: {fixed}"));
        }
        Ok(())
    }
}
