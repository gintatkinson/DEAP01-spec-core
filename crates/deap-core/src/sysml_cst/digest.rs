//! # SysML v2 / KerML Bijective CST Digest & Ariadne Diagnostic Renderer
//! [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides CRLF-normalized SHA-256 digest computation (`normalize_line_endings`,
//! `sha256_hex_digest`), bijective Rowan CST round-trip receipts (`compute_bijective_digest`),
//! and `ariadne` byte-span diagnostic rendering (`render_ariadne_diagnostic`) under DO-178C Level A
//! (`BP-03`, `BP-10`).
//!
//! ## 2. Invariants & Mathematical Properties
//! - CRLF Normalization: `normalize_line_endings` maps `\r\n` and `\r` to `\n` so digests are
//!   identical across POSIX and Windows checkouts.
//! - Bijective Identity: `receipt.raw_sha256 == receipt.canonical_cst_sha256` holds for all UTF-8 inputs.
//!
//! ## 3. Preconditions
//! - `source` is a valid UTF-8 string slice; `span` is clamped to `0..=source.chars().count()`.
//!
//! ## 4. Postconditions
//! - Produces deterministic SHA-256 digests and ANSI-free `ariadne` multi-line diagnostic reports.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N) over input byte length `N`.
//! - Space Complexity: O(N) normalized buffer storage.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Clamps out-of-range diagnostic spans safely to valid character boundaries with zero panics.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]

#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]

use std::ops::Range;

use ariadne::{Config, Label, Report, ReportKind, Source};
use serde::{Deserialize, Serialize};

use crate::digest::compute_sha256;
use super::hir_lower::lower_cst_to_hir;
use super::parser::parse_sysml_cst;
use super::printer::print_lossless_cst;

/// Receipt proving bijective Rowan CST round-trip identity and trivia-blind HIR hashing (`BP-03`, `BP-10`).
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Certifies that CST parsing and printing preserve 100% of source bytes while HIR hashing
/// isolates semantic structure under DO-178C Level A.
/// ## 2. Invariants & Mathematical Properties
/// `is_bijective == (raw_sha256 == canonical_cst_sha256)`.
/// ## 3. Preconditions
/// Computed by `compute_bijective_digest`.
/// ## 4. Postconditions
/// Exposes 64-hex SHA-256 digests for raw normalized source, printed CST, and lowered HIR.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1) field access; Space Complexity: O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// Pure value struct with zero panics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BijectiveDigestReceipt {
    pub raw_sha256: String,
    pub canonical_cst_sha256: String,
    pub hir_structural_sha256: String,
    pub node_count: usize,
    pub is_bijective: bool,
}

/// Rendered `ariadne` span diagnostic report for `DEAP-E1000..DEAP-E1005`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Provides deterministic source-span error reporting with line/column underlines (`BP-10`).
/// ## 2. Invariants & Mathematical Properties
/// `rendered` is a deterministic, ANSI-free UTF-8 diagnostic block containing `code` and `message`.
/// ## 3. Preconditions
/// Constructed via `render_ariadne_diagnostic`.
/// ## 4. Postconditions
/// Preserves `code`, `message`, `file_id`, `byte_range`, and `rendered`.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1) field access; Space Complexity: O(L) over rendered text length.
/// ## 6. Failure Modes & Fault Tolerance
/// Pure value struct with zero panics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AriadneSpanReport {
    pub code: String,
    pub message: String,
    pub file_id: String,
    pub byte_range: Range<usize>,
    pub rendered: String,
}

/// Normalizes `\r\n` and standalone `\r` line endings into canonical `\n` line feeds.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Eliminates OS line-ending drift before cryptographic hashing (`BP-03`, `BP-10`).
/// ## 2. Invariants & Mathematical Properties
/// Output contains zero `\r` bytes and is idempotent: `normalize_line_endings(&normalize_line_endings(s)) == normalize_line_endings(s)`.
/// ## 3. Preconditions
/// `input` is a valid UTF-8 string slice.
/// ## 4. Postconditions
/// Returns a `String` with all `\r\n` and `\r` sequences replaced by `\n`.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N); Space Complexity: O(N).
/// ## 6. Failure Modes & Fault Tolerance
/// Total function with zero panics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn normalize_line_endings(input: &str) -> String {
    if !input.contains('\r') {
        return input.to_string();
    }
    input.replace("\r\n", "\n").replace('\r', "\n")
}

/// Computes the 64-character lowercase hexadecimal SHA-256 digest of `bytes`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Cryptographic integrity primitive for SysML CST/HIR artifacts (`BP-03`, `BP-10`).
/// ## 2. Invariants & Mathematical Properties
/// Deterministic 256-bit FIPS 180-4 digest formatted as 64 lowercase hex ASCII characters.
/// ## 3. Preconditions
/// `bytes` is any byte slice.
/// ## 4. Postconditions
/// Returns a 64-byte hex `String`.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N); Space Complexity: O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// Total function with zero panics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn sha256_hex_digest(bytes: &[u8]) -> String {
    compute_sha256(bytes)
}

/// Computes a `BijectiveDigestReceipt` over `source` after CRLF normalization.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Verifies bijective Rowan CST round-trip identity and computes the trivia-blind HIR hash (`BP-03`).
/// ## 2. Invariants & Mathematical Properties
/// `raw_sha256 == canonical_cst_sha256` and `hir_structural_sha256` is invariant under whitespace/comment edits.
/// ## 3. Preconditions
/// `source` is a valid UTF-8 string slice.
/// ## 4. Postconditions
/// Returns a populated `BijectiveDigestReceipt`.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N); Space Complexity: O(N).
/// ## 6. Failure Modes & Fault Tolerance
/// Total function with zero panics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn compute_bijective_digest(source: &str) -> BijectiveDigestReceipt {
    let normalized = normalize_line_endings(source);
    let raw_sha256 = sha256_hex_digest(normalized.as_bytes());
    let parsed = parse_sysml_cst(&normalized);
    let printed = print_lossless_cst(&parsed);
    let canonical_cst_sha256 = sha256_hex_digest(printed.as_bytes());
    let (hir, _) = lower_cst_to_hir(&parsed);
    let node_count = parsed.syntax().descendants().count();
    let is_bijective = raw_sha256 == canonical_cst_sha256;
    BijectiveDigestReceipt {
        raw_sha256,
        canonical_cst_sha256,
        hir_structural_sha256: hir.structural_hash,
        node_count,
        is_bijective,
    }
}

/// Renders an ANSI-free `ariadne` span diagnostic report for `file_id` and byte `span`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Formats human- and machine-auditable span diagnostics for `DEAP-E1000..DEAP-E1005` (`BP-10`).
/// ## 2. Invariants & Mathematical Properties
/// Maps byte offsets to `ariadne` character offsets safely and disables ANSI color codes.
/// ## 3. Preconditions
/// `file_id`, `source`, `code`, and `message` are valid UTF-8 slices.
/// ## 4. Postconditions
/// Returns `AriadneSpanReport` with non-empty `rendered` text.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N); Space Complexity: O(N).
/// ## 6. Failure Modes & Fault Tolerance
/// Clamps out-of-bounds spans to `0..=char_count` and falls back to a formatted header if buffer write fails.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn render_ariadne_diagnostic(
    file_id: &str,
    source: &str,
    code: &str,
    message: &str,
    span: Range<usize>,
) -> AriadneSpanReport {
    let char_len = source.chars().count();
    let start_char = source
        .get(..span.start)
        .map_or_else(|| span.start.min(char_len), |s| s.chars().count());
    let end_char = source
        .get(..span.end)
        .map_or_else(|| span.end.min(char_len), |s| s.chars().count())
        .max(start_char);
    let mut buf = Vec::new();
    let report = Report::build(ReportKind::Error, (file_id, start_char..end_char))
        .with_config(Config::default().with_color(false))
        .with_code(code)
        .with_message(message)
        .with_label(
            Label::new((file_id, start_char..end_char)).with_message(format!("{code}: {message}")),
        )
        .finish();
    let rendered = match report.write((file_id, Source::from(source)), &mut buf) {
        Ok(()) => String::from_utf8(buf)
            .unwrap_or_else(|_| format!("[{code}] Error: {message} ({file_id})")),
        Err(_) => format!("[{code}] Error: {message} ({file_id})"),
    };
    AriadneSpanReport {
        code: code.to_string(),
        message: message.to_string(),
        file_id: file_id.to_string(),
        byte_range: span,
        rendered,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bijective_digest_crlf_and_trivia_invariance() {
        let lf = "package P {\n    part def A;\n}\n";
        let crlf = "package P {\r\n    part def A;\r\n}\r\n";
        let commented = "// extra comment\npackage P {\n    /* doc */\n    part def A;\n}\n";
        let r_lf = compute_bijective_digest(lf);
        let r_crlf = compute_bijective_digest(crlf);
        let r_comm = compute_bijective_digest(commented);
        assert!(r_lf.is_bijective && r_crlf.is_bijective && r_comm.is_bijective);
        assert_eq!(r_lf.raw_sha256, r_crlf.raw_sha256);
        assert_eq!(r_lf.hir_structural_sha256, r_comm.hir_structural_sha256);
        let diag = render_ariadne_diagnostic("test.sysml", lf, "DEAP-E1001", "Syntax error", 0..7);
        assert!(diag.rendered.contains("DEAP-E1001"));
    }
}
