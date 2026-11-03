//! Diagnostic reporting and defect dossier synthesis.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical software
//! engineering standards. Provides deterministic diagnostic classification, source location
//! anchoring, 5-Whys root cause structures, and 7-section Adversarial Audit defect dossier
//! Markdown and JSON synthesis.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Severity Total Ordering: `DiagnosticSeverity` variants implement total lexicographic and
//!   priority ordering (`Ord`, `PartialOrd`, `Eq`, `PartialEq`).
//! - Zero Em Dash Invariant: Synthesized Markdown dossiers emit strictly ASCII punctuation (`--`).
//! - Deterministic Serialization: JSON round-trip (`to_json` / `from_json`) preserves 1:1 structural equality.
//!
//! ## 3. Preconditions
//! - Input strings and JSON slices must be valid UTF-8 sequences.
//!
//! ## 4. Postconditions
//! - Constructors return initialized, strongly typed diagnostic or defect dossier structures.
//! - Serialization and parsing operations return explicit `Result` types without panicking.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) bounded linearly by the byte length of diagnostic messages or dossier fields.
//! - Space Complexity: O(N) bounded by the allocated string representations.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - JSON serialization and deserialization errors propagate fail-closed via `Result::Err(serde_json::Error)`.
//! - Display formatting propagates formatter errors via `std::fmt::Result`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-DIAG-MODULE], [DO-330-TQL-4-DIAGNOSTICS].

pub mod dossier;
pub mod item;
pub mod ndjson;
pub mod severity;

pub use dossier::{DefectDossier, WhyEntry};
pub use item::{
    render_diagnostic_multispan_with_ariadne, render_diagnostic_with_ariadne, Diagnostic,
    DiagnosticItem, StructuredDiagnostic,
};
pub use ndjson::{
    emit_cli_ndjson_stream, format_ndjson_line, resolve_utf8_span, Applicability,
    DiagnosticEmitError, RustcCompilerMessageEnvelope, RustcDiagnosticSpan,
};
pub use severity::DiagnosticSeverity;

#[cfg(test)]
mod tests;
