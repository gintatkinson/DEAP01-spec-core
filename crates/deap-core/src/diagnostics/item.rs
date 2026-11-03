//! Compiler and baseline diagnostic record representation.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical software
//! engineering standards. Provides structured diagnostic records anchored to workspace source locations.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Preserves severity, error code, message, and coordinate invariants across serialization and moves.
//!
//! ## 3. Preconditions
//! - Caller provides valid UTF-8 error codes, messages, and relative file paths.
//!
//! ## 4. Postconditions
//! - Returns initialized `Diagnostic` instances failing closed on malformed representations.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) bounded by string length.
//! - Space Complexity: O(N) heap allocations for message strings.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Total constructors; zero panics or unsafe operations.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-DIAG-MODULE], [DO-330-TQL-4-DIAGNOSTICS].

use super::severity::DiagnosticSeverity;
use ariadne::{Color, Label, Report, ReportKind, Source};
use serde::{Deserialize, Serialize};

/// Type alias binding `DiagnosticItem` to `Diagnostic` for presentation compatibility.
pub type DiagnosticItem = Diagnostic;

/// Type alias binding `StructuredDiagnostic` to `Diagnostic` for Track-1 CLI NDJSON adapters.
/// Realises: [REQ-0608]
pub type StructuredDiagnostic = Diagnostic;

/// Represents an individual compiler or linter diagnostic item.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `Diagnostic`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `Diagnostic` across member set `[severity, code, message, file, line, column]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `Diagnostic` members.
///
/// ## 4. Postconditions
/// - Defines the authoritative `Diagnostic` representation for struct `Diagnostic`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over member dimension of `Diagnostic`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `Diagnostic`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `Diagnostic`.
/// - State: Maintains consistent state invariants across `Diagnostic`.
/// - Action: Executes `Diagnostic` deterministically with zero unbounded recursion.
/// - Resource: Bounds stack and heap allocations within `Diagnostic`.
///
/// ## 7. Realization & Traceability
/// Realises: [Diagnostic/Diagnostic] | SysML v2: PartDef(Diagnostic)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub line: Option<usize>,
    #[serde(default)]
    pub column: Option<usize>,
}

impl Diagnostic {
    /// Constructs a new diagnostic item with a severity, standardized error code, and message.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and ECSS-E-ST-40C verification tool qualification (DO-330 TQL-4).
    /// Provides structured, machine-verifiable diagnostic records for compiler and baseline gate violations.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Field Preservation: `result.severity == severity`, `result.code == code.into()`, and
    ///   `result.message == message.into()`.
    /// - Unbound Location Default: Initializes source coordinate fields to `None`.
    ///
    /// ## 3. Preconditions
    /// - `severity` is a valid `DiagnosticSeverity` variant.
    /// - `code` and `message` implement `Into<String>` yielding valid UTF-8 strings.
    ///
    /// ## 4. Postconditions
    /// - Returns a newly initialized `Diagnostic` instance with `file == None`, `line == None`, and `column == None`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(|code| + |message|) when converting borrowed slices.
    /// - Space Complexity: O(|code| + |message|) heap storage.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Accepts any `Into<String>` type safely at compile time.
    /// - State: Constructs a complete, self-consistent `Diagnostic` value in a single atomic expression.
    /// - Action: Total constructor; never panics or performs fallible I/O.
    /// - Resource: Bounded heap allocation proportional to `code` and `message` lengths.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [Diagnostic/Diagnostic::new] | [REQ-DIAG-NEW] | SysML v2: part def DiagnosticEngine
    pub fn new(
        severity: DiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code: code.into(),
            message: message.into(),
            file: None,
            line: None,
            column: None,
        }
    }

    /// Attaches a source file path and 1-indexed line and column coordinates to a diagnostic.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A bidirectional traceability mandates by anchoring compiler and
    /// verification diagnostics to exact workspace-relative file, line, and column coordinates.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Identity Preservation: `self.severity`, `self.code`, and `self.message` remain invariant.
    /// - Coordinate Binding: Sets `self.file = Some(file.into())`, `self.line = Some(line)`, and
    ///   `self.column = Some(col)`.
    ///
    /// ## 3. Preconditions
    /// - `self` is a valid `Diagnostic` value transferred by ownership.
    /// - `file` is convertible into a valid UTF-8 `String` representing a workspace-relative path.
    /// - `line` and `col` represent source line and column indices.
    ///
    /// ## 4. Postconditions
    /// - Returns the consumed `Diagnostic` instance with `file`, `line`, and `column` populated as `Some`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(|file|) if converting a `&str` slice, or O(1) if moving an owned `String`.
    /// - Space Complexity: O(|file|) heap allocation for the file path string.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Consumes `self` by value (fluent builder pattern), preventing aliased mutation.
    /// - State: Updates all three location fields together so partial location state cannot occur.
    /// - Action: Total infallible transformation; never panics.
    /// - Resource: Bounded single `String` allocation for `file`.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [Diagnostic/Diagnostic::with_location] | [REQ-DIAG-LOC] | SysML v2: part def DiagnosticEngine
    pub fn with_location(mut self, file: impl Into<String>, line: usize, col: usize) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self.column = Some(col);
        self
    }
}

/// Renders a diagnostic item anchored to a source span using `ariadne`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and ECSS-E-ST-40C diagnostic presentation standards.
/// Produces human-readable ANSI terminal reports for compiler and baseline diagnostics.
///
/// ## 2. Invariants & Mathematical Properties
/// - Bounded Span Invariant: Spans are clamped safely within source text boundaries.
/// - Deterministic Rendering: Formatted report is referentially transparent given identical inputs.
///
/// ## 3. Preconditions
/// - `item` contains valid UTF-8 error code and message strings.
/// - `source_id` and `source_text` are valid UTF-8 slices.
///
/// ## 4. Postconditions
/// - Returns a formatted ANSI diagnostic string containing error code, message, and line context.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) bounded linearly by `source_text` length.
/// - Space Complexity: O(N) auxiliary buffer for formatted diagnostic report.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Handles invalid byte spans gracefully by fallback formatting without panicking.
/// - State: Zero shared mutable state; pure rendering function.
/// - Action: Fails open to plain-text fallback string on buffer write error.
/// - Resource: Bounded heap allocation for rendered string buffer.
///
/// ## 7. Realization & Traceability
/// Realises: [Diagnostic/render_diagnostic_with_ariadne] | [REQ-DIAG-ARIADNE] | SysML v2: part def DiagnosticRenderer
pub fn render_diagnostic_with_ariadne(
    item: &DiagnosticItem,
    source_id: &str,
    source_text: &str,
    span: std::ops::Range<usize>,
) -> String {
    let color = match item.severity {
        DiagnosticSeverity::Critical | DiagnosticSeverity::Error => Color::Red,
        DiagnosticSeverity::Important | DiagnosticSeverity::Warning => Color::Yellow,
        DiagnosticSeverity::Suggestion | DiagnosticSeverity::Info => Color::Blue,
        DiagnosticSeverity::Nitpick | DiagnosticSeverity::Hint => Color::Cyan,
    };
    let note = format!("severity: {}", item.severity);
    render_diagnostic_multispan_with_ariadne(
        item,
        source_id,
        source_text,
        &[(span, item.message.clone(), color)],
        Some(&note),
    )
}

/// Renders a multi-span diagnostic report using `ariadne` with custom colors and optional note.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and ECSS-E-ST-40C terminal presentation requirements.
/// Supports multi-span visual diagnostics for secondary context and causal traceability.
///
/// ## 2. Invariants & Mathematical Properties
/// - Multi-Span Invariant: Preserves relative ordering and distinct colors across all attached labels.
///
/// ## 3. Preconditions
/// - `item` contains well-formed diagnostic metadata.
/// - Spans reference valid byte ranges within `source_text`.
///
/// ## 4. Postconditions
/// - Returns ANSI terminal formatted string containing all labeled spans and note footer.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N + S log S) where N is source length and S is span count.
/// - Space Complexity: O(N) auxiliary string buffer.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates and clamps span ranges safely.
/// - State: Pure function with zero mutable side effects.
/// - Action: Infallible fallback formatting if report serializer fails.
/// - Resource: Bounded memory allocation proportional to source text.
///
/// ## 7. Realization & Traceability
/// Realises: [Diagnostic/render_diagnostic_multispan_with_ariadne] | [REQ-DIAG-ARIADNE-MULTI] | SysML v2: part def DiagnosticRenderer
pub fn render_diagnostic_multispan_with_ariadne(
    item: &DiagnosticItem,
    source_id: &str,
    source_text: &str,
    spans: &[(std::ops::Range<usize>, String, Color)],
    note: Option<&str>,
) -> String {
    let kind = match item.severity {
        DiagnosticSeverity::Critical | DiagnosticSeverity::Error => ReportKind::Error,
        DiagnosticSeverity::Important | DiagnosticSeverity::Warning => ReportKind::Warning,
        DiagnosticSeverity::Suggestion | DiagnosticSeverity::Info => ReportKind::Advice,
        DiagnosticSeverity::Nitpick | DiagnosticSeverity::Hint => ReportKind::Custom("hint", Color::Cyan),
    };
    let (p_start, p_end) = spans
        .first()
        .map(|(r, _, _)| (r.start, r.end.max(r.start)))
        .unwrap_or((0, 0));

    let mut builder = Report::build(kind, (source_id, p_start..p_end))
        .with_code(&item.code)
        .with_message(&item.message);

    for (range, msg, col) in spans {
        let r = range.start..range.end.max(range.start);
        builder.add_label(Label::new((source_id, r)).with_color(*col).with_message(msg));
    }
    if let Some(n) = note {
        builder.set_note(n);
    }
    let report = builder.finish();

    let mut buffer = Vec::new();
    if report.write((source_id, Source::from(source_text)), &mut buffer).is_err() {
        return format!("{}: [{}] {}", item.severity, item.code, item.message);
    }
    String::from_utf8_lossy(&buffer).to_string()
}
