#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Unified `rustc`-compatible NDJSON diagnostic envelope and UTF-8 span resolution. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C
//! qualification standards. Provides deterministic `rustc` JSON diagnostic envelopes
//! (`$message_type: "diagnostic"`), `O(log N)` 1-indexed UTF-8 line/column span
//! resolution via `ra_ap_line_index::LineIndex`, and single-line NDJSON stream emission.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Every resolved `RustcDiagnosticSpan` satisfies `1 <= line_start <= line_end`,
//!   `1 <= column_start`, `1 <= column_end`, and `byte_start <= byte_end <= source.len()`
//!   aligned on valid UTF-8 code-point boundaries.
//! - Every serialized NDJSON record contains `"$message_type":"diagnostic"` and zero
//!   raw newline (`0x0A`, `0x0D`) or ANSI escape (`0x1B`) bytes.
//!
//! ## 3. Preconditions
//! - Callers provide valid UTF-8 source slices and workspace-relative file names.
//!
//! ## 4. Postconditions
//! - Returns strongly-typed `RustcDiagnosticSpan` records and single-line NDJSON strings
//!   without panicking on out-of-bounds or mid-codepoint byte ranges.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Index construction: $O(N)$ over source bytes; per-span lookup: $O(\log L)$ over line count $L$.
//! - NDJSON serialization: $O(M)$ linear in envelope payload length.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Out-of-bounds or inverted ranges clamp deterministically to valid UTF-8 boundaries.
//! - Serialization failures propagate fail-closed via `Result::Err(serde_json::Error)`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-DIAG-MODULE], [REQ-0608], [DO-330-TQL-4-DIAGNOSTICS]

use std::ops::Range;

use ra_ap_line_index::{LineIndex, TextSize};
use serde::{Deserialize, Serialize};

/// Returns `"diagnostic"` as the canonical `rustc` `$message_type` discriminator.
/// Realises: [REQ-0608]
fn default_diagnostic_message_type() -> String {
    String::from("diagnostic")
}

/// [DOC-TIER: A] Machine-fix applicability classification matching `rustc` JSON diagnostics.
///
/// Safety & Traceability: Realises: [REQ-DIAG-MODULE], [REQ-0608]
/// Contract: Classifies whether a `suggested_replacement` on a `RustcDiagnosticSpan` can be spliced automatically by `--apply-machine-fixes`.
/// Bounds & Faults: O(1) copyable enum with total ordering and deterministic serde serialization.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub enum Applicability {
    /// The suggestion is guaranteed to be valid Rust/SysML syntax and preserve semantics.
    MachineApplicable,
    /// The suggestion contains placeholder tokens such as `/* ... */` that require human completion.
    HasPlaceholders,
    /// The suggestion may compile but could alter program semantics.
    MaybeIncorrect,
    /// Applicability could not be determined automatically.
    #[default]
    Unspecified,
}

/// [DOC-TIER: A] 1-indexed UTF-8 source span matching `rustc` JSON diagnostic span layout.
///
/// Safety & Traceability: Realises: [REQ-DIAG-MODULE], [REQ-0608]
/// Contract: Anchors a diagnostic to `[byte_start..byte_end)` alongside 1-indexed `line_start`, `line_end`, `column_start`, `column_end`, and optional machine-applicable replacement metadata.
/// Bounds & Faults: O(1) structural record construction and lossless JSON round-trip serialization.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RustcDiagnosticSpan {
    /// Workspace-relative file path using forward-slash separators.
    pub file_name: String,
    /// 0-indexed starting UTF-8 byte offset (inclusive).
    pub byte_start: usize,
    /// 0-indexed ending UTF-8 byte offset (exclusive).
    pub byte_end: usize,
    /// 1-indexed starting line number.
    pub line_start: usize,
    /// 1-indexed ending line number.
    pub line_end: usize,
    /// 1-indexed starting Unicode scalar column offset.
    pub column_start: usize,
    /// 1-indexed ending Unicode scalar column offset.
    pub column_end: usize,
    /// Whether this span is the primary focal span of the diagnostic.
    pub is_primary: bool,
    /// Optional replacement text for automated codemod splicing.
    pub suggested_replacement: Option<String>,
    /// Optional applicability tier governing automated fix execution.
    pub suggestion_applicability: Option<Applicability>,
}

/// [DOC-TIER: A] Top-level `rustc`-compatible JSON diagnostic message envelope (`TD-08..TD-11`).
///
/// Safety & Traceability: Realises: [REQ-DIAG-MODULE], [REQ-0608]
/// Contract: Serializes with `"$message_type": "diagnostic"`, `message`, optional `code`, `level`, `spans`, `children`, and optional human-rendered `rendered` text.
/// Bounds & Faults: O(N) serialization complexity proportional to span and child envelope count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcCompilerMessageEnvelope {
    /// Discriminator field serialized as `"$message_type": "diagnostic"`.
    #[serde(rename = "$message_type", default = "default_diagnostic_message_type")]
    pub message_type: String,
    /// Primary human-readable diagnostic message.
    pub message: String,
    /// Optional diagnostic code identifier (e.g., `"DEAP-E0801"`).
    pub code: Option<String>,
    /// Severity level string (`"error"`, `"warning"`, `"note"`, `"help"`).
    pub level: String,
    /// Ordered collection of source spans attached to this diagnostic.
    pub spans: Vec<RustcDiagnosticSpan>,
    /// Sub-diagnostics (notes, help items) attached to this envelope.
    #[serde(default)]
    pub children: Vec<RustcCompilerMessageEnvelope>,
    /// Optional pre-rendered human-readable representation (zero ANSI escape codes in NDJSON).
    pub rendered: Option<String>,
}

impl Default for RustcCompilerMessageEnvelope {
    /// Creates an empty diagnostic envelope with `message_type` set to `"diagnostic"`.
    /// Realises: [REQ-0608]
    fn default() -> Self {
        Self {
            message_type: default_diagnostic_message_type(),
            message: String::new(),
            code: None,
            level: String::from("error"),
            spans: Vec::new(),
            children: Vec::new(),
            rendered: None,
        }
    }
}

/// Clamps an arbitrary byte offset to the nearest preceding or equal UTF-8 char boundary in `source`.
/// Realises: [REQ-0608]
fn clamp_utf8_boundary(source: &str, offset: usize) -> usize {
    let mut idx = offset.min(source.len());
    while idx > 0 && !source.is_char_boundary(idx) {
        idx = idx.saturating_sub(1);
    }
    idx
}

/// Resolves a UTF-8 char-boundary byte offset into 1-indexed `(line, column)` coordinates.
/// Realises: [REQ-0608]
fn resolve_one_indexed_line_col(
    source: &str,
    line_index: &LineIndex,
    byte_offset: usize,
) -> (usize, usize) {
    let clamped = u32::try_from(byte_offset).unwrap_or(u32::MAX);
    let lc = line_index.line_col(TextSize::from(clamped));
    let line_1 = (lc.line as usize).saturating_add(1);
    let line_byte_start = byte_offset.saturating_sub(lc.col as usize);
    let col_1 = source
        .get(line_byte_start..byte_offset)
        .map_or(lc.col as usize, |slice| slice.chars().count())
        .saturating_add(1);
    (line_1, col_1)
}

/// [DOC-TIER: A] Resolves a UTF-8 byte range into a 1-indexed `RustcDiagnosticSpan` via `ra_ap_line_index::LineIndex`.
///
/// Safety & Traceability: Realises: [REQ-DIAG-MODULE], [REQ-0608]
/// Contract: Clamps `byte_range` to valid UTF-8 code-point boundaries within `source` and computes 1-indexed `line_start`, `line_end`, `column_start`, and `column_end` coordinates.
/// Bounds & Faults: O(N) index build and O(log L) line lookup; never panics on inverted, empty, or out-of-bounds byte ranges.
#[must_use]
pub fn resolve_utf8_span(
    source: &str,
    byte_range: Range<usize>,
    file_name: &str,
) -> RustcDiagnosticSpan {
    let byte_start = clamp_utf8_boundary(source, byte_range.start);
    let byte_end = clamp_utf8_boundary(source, byte_range.end).max(byte_start);
    let line_index = LineIndex::new(source);
    let (line_start, column_start) = resolve_one_indexed_line_col(source, &line_index, byte_start);
    let (line_end, column_end) = resolve_one_indexed_line_col(source, &line_index, byte_end);
    RustcDiagnosticSpan {
        file_name: file_name.replace('\\', "/"),
        byte_start,
        byte_end,
        line_start,
        line_end,
        column_start,
        column_end,
        is_primary: true,
        suggested_replacement: None,
        suggestion_applicability: None,
    }
}

/// [DOC-TIER: A] Serializes a `RustcCompilerMessageEnvelope` into a single-line NDJSON string.
///
/// Safety & Traceability: Realises: [REQ-DIAG-MODULE], [REQ-0608]
/// Contract: Guarantees `"$message_type": "diagnostic"` and strips any raw `\r`, `\n`, or `\x1b` characters so every emitted record is a valid single-line NDJSON object.
/// Bounds & Faults: O(M) linear in serialized byte length; propagates `serde_json::Error` fail-closed.
pub fn format_ndjson_line(
    envelope: &RustcCompilerMessageEnvelope,
) -> Result<String, serde_json::Error> {
    let mut normalized = envelope.clone();
    if normalized.message_type.is_empty() {
        normalized.message_type = default_diagnostic_message_type();
    }
    let json = serde_json::to_string(&normalized)?;
    Ok(json
        .replace(['\r', '\n', '\x1b'], "")
        .replace("\\u001b", ""))
}

/// [DOC-TIER: A] Formats a slice of `RustcCompilerMessageEnvelope` records into a newline-delimited NDJSON stream.
///
/// Safety & Traceability: Realises: [REQ-DIAG-MODULE], [REQ-0608]
/// Contract: Serializes each envelope via `format_ndjson_line` and joins all valid records with `\n` terminators.
/// Bounds & Faults: O(K * M) over `K` envelopes of average byte size `M`; returns an empty string when `envelopes` is empty.
#[must_use]
pub fn emit_cli_ndjson_stream(envelopes: &[RustcCompilerMessageEnvelope]) -> String {
    let mut out = String::new();
    for envelope in envelopes {
        if let Ok(line) = format_ndjson_line(envelope) {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&line);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies 1-indexed line and UTF-8 scalar column resolution across single-line and multi-line spans.
    #[test]
    fn resolves_one_indexed_utf8_spans_across_ascii_and_multibyte_lines() {
        let src = "fn alpha() {}\nlet \u{03b1}\u{03b2} = 42;\n";
        let span1 = resolve_utf8_span(src, 3..8, "src\\lib.rs");
        assert_eq!(span1.file_name, "src/lib.rs");
        assert_eq!(span1.byte_start, 3);
        assert_eq!(span1.byte_end, 8);
        assert_eq!(span1.line_start, 1);
        assert_eq!(span1.line_end, 1);
        assert_eq!(span1.column_start, 4);
        assert_eq!(span1.column_end, 9);
        assert!(span1.is_primary);

        let line2_start = 14;
        let eq_offset = line2_start + "let \u{03b1}\u{03b2} ".len();
        let span2 = resolve_utf8_span(src, eq_offset..(eq_offset + 1), "src/lib.rs");
        assert_eq!(span2.line_start, 2);
        assert_eq!(span2.line_end, 2);
        assert_eq!(span2.column_start, 8);
        assert_eq!(span2.column_end, 9);
    }

    /// Verifies clamping of inverted, mid-codepoint, and out-of-bounds byte ranges.
    #[test]
    fn clamps_out_of_bounds_and_mid_codepoint_utf8_spans_safely() {
        let src = "\u{03b1}bc";
        let mid = resolve_utf8_span(src, 1..99, "src/main.rs");
        assert_eq!(mid.byte_start, 0);
        assert_eq!(mid.byte_end, src.len());
        assert_eq!(mid.line_start, 1);
        assert_eq!(mid.column_start, 1);
        assert_eq!(mid.column_end, 4);

        let inverted = resolve_utf8_span(src, std::ops::Range { start: 3, end: 2 }, "src/main.rs");
        assert_eq!(inverted.byte_start, 3);
        assert_eq!(inverted.byte_end, 3);
    }

    /// Verifies single-line NDJSON serialization and multi-envelope CLI stream formatting.
    #[test]
    fn formats_single_line_rustc_ndjson_and_emits_cli_stream() -> Result<(), serde_json::Error> {
        let mut span = resolve_utf8_span("let x = 1;\n", 4..5, "src/lib.rs");
        span.suggested_replacement = Some(String::from("y"));
        span.suggestion_applicability = Some(Applicability::MachineApplicable);

        let env1 = RustcCompilerMessageEnvelope {
            message_type: String::from("diagnostic"),
            message: String::from("unused variable `x`"),
            code: Some(String::from("DEAP-E0801")),
            level: String::from("error"),
            spans: vec![span],
            children: Vec::new(),
            rendered: Some(String::from("\x1b[31merror[DEAP-E0801]\x1b[0m\n")),
        };
        let line = format_ndjson_line(&env1)?;
        assert!(line.contains("\"$message_type\":\"diagnostic\""));
        assert!(!line.contains('\n'));
        assert!(!line.contains('\x1b'));
        assert!(!line.contains("\\u001b"));

        let roundtrip: RustcCompilerMessageEnvelope = serde_json::from_str(&line)?;
        assert_eq!(roundtrip.message_type, "diagnostic");
        assert_eq!(roundtrip.code.as_deref(), Some("DEAP-E0801"));
        assert_eq!(
            roundtrip.spans.first().and_then(|s| s.suggestion_applicability),
            Some(Applicability::MachineApplicable)
        );

        let env2 = RustcCompilerMessageEnvelope {
            message: String::from("second diagnostic"),
            code: Some(String::from("DEAP-E0802")),
            ..RustcCompilerMessageEnvelope::default()
        };
        let stream = emit_cli_ndjson_stream(&[env1, env2]);
        let lines: Vec<&str> = stream.lines().collect();
        assert_eq!(lines.len(), 2);
        Ok(())
    }
}
