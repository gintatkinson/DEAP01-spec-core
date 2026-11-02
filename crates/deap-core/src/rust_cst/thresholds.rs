#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Structural complexity thresholds and typed CST error taxonomy. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C qualification.
//! Provides structural complexity thresholds and non-exhaustive fail-closed diagnostics.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Bounded thresholds: max_fn_exec_sloc in 1..=50, max_fn_param_count <= 5, max_mut_bool_flags == 0.
//! - Canonical BP-08 UUIDv5 identifier matches "6ba7b810-9dad-11d1-80b4-00c04fd430c8".
//!
//! ## 3. Preconditions
//! - Callers construct CstMetricsThresholds via default() or explicit fields before validation.
//!
//! ## 4. Postconditions
//! - Returns Ok(()) when thresholds conform or Err(CstMetricsError) fail-closed.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - O(1) constant scalar checks and static string comparisons.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Typed CstMetricsError taxonomy for DEAP-E0801..DEAP-E0804 without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [LosslessRustCstAnalyzer/CstMetricsThresholds], [REQ-0608]
//! SysML v2: part def LosslessRustCstAnalyzer

use super::envelope::{BP08_DETERMINISTIC_UUID, MAX_AST_DEPTH};

/// Canonical BP-08 C_CST_01 upper ceiling on executable SLOC per Rust callable.
pub const MAX_THRESHOLD_FN_EXEC_SLOC: usize = 50;
/// Canonical BP-08 C_CST_02 upper ceiling on formal parameters excluding self receivers.
pub const MAX_THRESHOLD_FN_PARAM_COUNT: usize = 5;
/// Canonical BP-08 C_CST_03 required zero ceiling on mutable boolean local flags.
pub const REQUIRED_THRESHOLD_MUT_BOOL_FLAGS: usize = 0;
/// Canonical BP-08 required compound boolean predicate clause arity ceiling.
pub const REQUIRED_THRESHOLD_PREDICATE_ARITY: usize = 3;

/// Immutable structural complexity and lossless CST configuration thresholds.
/// Realises: [LosslessRustCstAnalyzer/CstMetricsThresholds], [REQ-0608]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct CstMetricsThresholds {
    /// Upper bound on non-trivia executable lines per Rust callable under C_CST_01 (1..=50).
    pub max_fn_exec_sloc: usize,
    /// Upper bound on formal parameters excluding self receivers under C_CST_02 (0..=5).
    pub max_fn_param_count: usize,
    /// Permitted mutable bool local flag count per Rust callable under C_CST_03 (0).
    pub max_mut_bool_flags: usize,
    /// Maximum boolean operator clauses permitted in an unhoisted compound predicate (3).
    pub max_predicate_arity: usize,
    /// Hard iterative Red/Green CST traversal depth ceiling (MAX_AST_DEPTH = 64).
    pub max_ast_depth: usize,
    /// Requires bit-for-bit root rowan::GreenNode round-trip equality (true).
    pub lossless_roundtrip_required: bool,
    /// Canonical deterministic UUIDv5 namespace identifier.
    pub deterministic_uuid: &'static str,
}

impl Default for CstMetricsThresholds {
    fn default() -> Self {
        Self {
            max_fn_exec_sloc: MAX_THRESHOLD_FN_EXEC_SLOC,
            max_fn_param_count: MAX_THRESHOLD_FN_PARAM_COUNT,
            max_mut_bool_flags: REQUIRED_THRESHOLD_MUT_BOOL_FLAGS,
            max_predicate_arity: REQUIRED_THRESHOLD_PREDICATE_ARITY,
            max_ast_depth: MAX_AST_DEPTH,
            lossless_roundtrip_required: true,
            deterministic_uuid: BP08_DETERMINISTIC_UUID,
        }
    }
}

/// Typed non-exhaustive domain error taxonomy for LosslessRustCstAnalyzer (DEAP-E0801..DEAP-E0804).
/// Realises: [LosslessRustCstAnalyzer/CstMetricsError], [REQ-0608]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub enum CstMetricsError {
    /// Out-of-order lifecycle dispatch or unverified parse state precondition (DEAP-E0801).
    UnverifiedParsePrecondition { state: &'static str },
    /// Root rowan::GreenNode round-trip mismatch, span collision, or non-char boundary (DEAP-E0802).
    LosslessRoundtripOrSpanMismatch { byte_start: usize, byte_end: usize },
    /// Invalid UTF-8 byte sequence or lossless ra_ap_syntax Rust syntax error (DEAP-E0803).
    MalformedUtf8OrRustSyntax {
        byte_start: usize,
        byte_end: usize,
        detail: String,
    },
    /// C_CST_01..C_CST_03 threshold drift or callable v2_metrics constraint violation (DEAP-E0803).
    ThresholdEnvelopeOrMetricsViolation {
        code: &'static str,
        symbol: String,
        byte_start: usize,
        byte_end: usize,
    },
    /// Iterative CST traversal ceiling breach or checked arithmetic overflow (DEAP-E0804).
    AstDepthOrArithmeticExhaustion {
        observed_depth: usize,
        max_depth: usize,
        byte_offset: usize,
    },
}

impl CstMetricsError {
    /// Returns the canonical DEAP-E0801..DEAP-E0804 diagnostic code.
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::UnverifiedParsePrecondition { .. } => "DEAP-E0801",
            Self::LosslessRoundtripOrSpanMismatch { .. } => "DEAP-E0802",
            Self::MalformedUtf8OrRustSyntax { .. }
            | Self::ThresholdEnvelopeOrMetricsViolation { .. } => "DEAP-E0803",
            Self::AstDepthOrArithmeticExhaustion { .. } => "DEAP-E0804",
        }
    }

    /// Returns the half-open UTF-8 byte span (byte_start, byte_end).
    pub const fn byte_span(&self) -> (usize, usize) {
        match self {
            Self::UnverifiedParsePrecondition { .. } => (0, 0),
            Self::LosslessRoundtripOrSpanMismatch { byte_start, byte_end }
            | Self::MalformedUtf8OrRustSyntax { byte_start, byte_end, .. }
            | Self::ThresholdEnvelopeOrMetricsViolation { byte_start, byte_end, .. } => {
                (*byte_start, *byte_end)
            }
            Self::AstDepthOrArithmeticExhaustion { byte_offset, .. } => {
                (*byte_offset, *byte_offset)
            }
        }
    }
}

fn clean_diag_line(raw: &str) -> String {
    raw.chars()
        .map(|ch| if ch == '\n' || ch == '\r' || ch == '\x1b' { ' ' } else { ch })
        .collect()
}

impl std::fmt::Display for CstMetricsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let diag = self.diagnostic_code();
        match self {
            Self::UnverifiedParsePrecondition { state } => {
                write!(f, "[{diag}] unverified CST parse precondition in state '{}'", clean_diag_line(state))
            }
            Self::LosslessRoundtripOrSpanMismatch { byte_start, byte_end } => {
                write!(f, "[{diag}] lossless CST roundtrip or UTF-8 span mismatch at bytes {byte_start}..{byte_end}")
            }
            Self::MalformedUtf8OrRustSyntax { byte_start, byte_end, detail } => {
                write!(f, "[{diag}] malformed UTF-8 or Rust syntax at bytes {byte_start}..{byte_end}: {}", clean_diag_line(detail))
            }
            Self::ThresholdEnvelopeOrMetricsViolation { code, symbol, byte_start, byte_end } => {
                write!(f, "[{diag}] CST constraint {} violated by '{}' at bytes {byte_start}..{byte_end}", clean_diag_line(code), clean_diag_line(symbol))
            }
            Self::AstDepthOrArithmeticExhaustion { observed_depth, max_depth, byte_offset } => {
                write!(f, "[{diag}] CST depth or arithmetic exhaustion (observed {observed_depth}, max {max_depth}) at byte {byte_offset}")
            }
        }
    }
}

impl std::error::Error for CstMetricsError {}

fn threshold_violation(code: &'static str, symbol: &'static str) -> CstMetricsError {
    CstMetricsError::ThresholdEnvelopeOrMetricsViolation {
        code,
        symbol: symbol.to_string(),
        byte_start: 0,
        byte_end: 0,
    }
}

/// Validates CstMetricsThresholds against C_CST_01..C_CST_03 and MAX_AST_DEPTH invariants.
/// Realises: [LosslessRustCstAnalyzer/validate_metrics_thresholds], [REQ-0608]
pub fn validate_metrics_thresholds(
    thresholds: &CstMetricsThresholds,
) -> Result<(), CstMetricsError> {
    if thresholds.max_fn_exec_sloc == 0 || thresholds.max_fn_exec_sloc > MAX_THRESHOLD_FN_EXEC_SLOC {
        return Err(threshold_violation("C_CST_01", "max_fn_exec_sloc"));
    }
    if thresholds.max_fn_param_count > MAX_THRESHOLD_FN_PARAM_COUNT {
        return Err(threshold_violation("C_CST_02", "max_fn_param_count"));
    }
    if thresholds.max_mut_bool_flags != REQUIRED_THRESHOLD_MUT_BOOL_FLAGS {
        return Err(threshold_violation("C_CST_03", "max_mut_bool_flags"));
    }
    if thresholds.max_predicate_arity != REQUIRED_THRESHOLD_PREDICATE_ARITY {
        return Err(threshold_violation("C_CST_PREDICATE_ARITY", "max_predicate_arity"));
    }
    if thresholds.max_ast_depth != MAX_AST_DEPTH {
        return Err(threshold_violation("C_CST_MAX_AST_DEPTH", "max_ast_depth"));
    }
    if !thresholds.lossless_roundtrip_required {
        return Err(threshold_violation("C_CST_LOSSLESS_ROUNDTRIP", "lossless_roundtrip_required"));
    }
    if thresholds.deterministic_uuid != BP08_DETERMINISTIC_UUID {
        return Err(threshold_violation("C_CST_DETERMINISTIC_UUID", "deterministic_uuid"));
    }
    Ok(())
}
