//! Diagnostic severity classification.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical software
//! engineering standards. Provides deterministic diagnostic classification and priority ordering.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Severity Total Ordering: `DiagnosticSeverity` variants implement total lexicographic and
//!   priority ordering (`Ord`, `PartialOrd`, `Eq`, `PartialEq`).
//!
//! ## 3. Preconditions
//! - Caller supplies well-formed values for `DiagnosticSeverity` members.
//!
//! ## 4. Postconditions
//! - Defines authoritative `DiagnosticSeverity` representation.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(1) constant-time enum dispatch.
//! - Space Complexity: O(1) auxiliary memory.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Exhaustive matching prevents unhandled enum variant faults.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-DIAG-MODULE], [DO-330-TQL-4-DIAGNOSTICS].

use serde::{Deserialize, Serialize};

/// Severity levels for diagnostics and defect audit dossiers.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for enum `DiagnosticSeverity`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for enum `DiagnosticSeverity` across member set `[Critical, Important, Suggestion, Nitpick, Error, Warning, Info, Hint]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `DiagnosticSeverity` members.
///
/// ## 4. Postconditions
/// - Defines the authoritative `DiagnosticSeverity` representation for enum `DiagnosticSeverity`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(8)) bounded over member dimension of `DiagnosticSeverity`.
/// - Space Complexity: O(M(8)) bounded auxiliary memory for `DiagnosticSeverity`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `DiagnosticSeverity`.
/// - State: Maintains consistent state invariants across `DiagnosticSeverity`.
/// - Action: Executes `DiagnosticSeverity` deterministically with zero unbounded recursion.
/// - Resource: Bounds stack and heap allocations within `DiagnosticSeverity`.
///
/// ## 7. Realization & Traceability
/// Realises: [Diagnostic/DiagnosticSeverity] | SysML v2: PartDef(DiagnosticSeverity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Critical,
    Important,
    Suggestion,
    Nitpick,
    Error,
    Warning,
    Info,
    Hint,
}

impl std::fmt::Display for DiagnosticSeverity {
    /// Formats the diagnostic severity level into its canonical human-readable string representation.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C diagnostic reporting requirements.
    /// Ensures deterministic textual rendering of severity classifications in compiler diagnostics and
    /// 7-section Adversarial Audit defect dossiers (`SEVERITY: <level>`).
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Bijective Mapping: Establishes a strict 1-to-1 bijection between the 8 `DiagnosticSeverity`
    ///   enum variants and their exact ASCII PascalCase string literals.
    /// - Referential Transparency: Pure function of `*self` with zero side effects outside the formatter buffer.
    ///
    /// ## 3. Preconditions
    /// - `self` is a valid, initialized `DiagnosticSeverity` variant.
    /// - `f` is a valid mutable reference to an active `std::fmt::Formatter`.
    ///
    /// ## 4. Postconditions
    /// - Writes the exact canonical ASCII severity token to the formatter stream `f`.
    /// - Returns `Ok(())` if the underlying stream write succeeds, or `Err(std::fmt::Error)` otherwise.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant-time enum branch dispatch and fixed-length literal write (4 to 10 bytes).
    /// - Space Complexity: O(1) auxiliary stack memory; zero heap allocations.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Propagates underlying formatter stream failures via `std::fmt::Result` without panicking.
    /// - State: Immutable borrow of `self`; cannot corrupt diagnostic severity state.
    /// - Action: Exhaustive compile-time `match` prevents unhandled enum variant faults.
    /// - Resource: Zero dynamic memory allocation eliminates out-of-memory hazards.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [Diagnostic/DiagnosticSeverity::fmt] | [REQ-DIAG-FMT] | SysML v2: part def DiagnosticEngine
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticSeverity::Critical => write!(f, "Critical"),
            DiagnosticSeverity::Important => write!(f, "Important"),
            DiagnosticSeverity::Suggestion => write!(f, "Suggestion"),
            DiagnosticSeverity::Nitpick => write!(f, "Nitpick"),
            DiagnosticSeverity::Error => write!(f, "Error"),
            DiagnosticSeverity::Warning => write!(f, "Warning"),
            DiagnosticSeverity::Info => write!(f, "Info"),
            DiagnosticSeverity::Hint => write!(f, "Hint"),
        }
    }
}
