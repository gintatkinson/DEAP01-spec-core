//! # SysML v2 / KerML Lossless Rowan CST Printer & Disjoint Span Splicer
//! [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides lossless Rowan CST printing (`print_lossless_cst`), canonical HIR formatting
//! (`format_canonical_hir_summary`), and disjoint reverse-byte-order CST span splicing
//! (`validate_disjoint_splice_ranges`, `apply_formatting_preserving_splices`) under DO-178C Level A
//! (`BP-03`, `BP-10`).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Lossless Identity: `print_lossless_cst(&parse_sysml_cst(src)) == src`.
//! - Disjoint Reverse-Sorted Splicing: Patches are applied from highest byte offset to lowest
//!   after verifying pairwise disjointness (`a.end <= b.start`) and UTF-8 char boundaries (`DEAP-E1005`).
//!
//! ## 3. Preconditions
//! - `source` is a valid UTF-8 string slice; `patches` reference byte ranges within `0..=source.len()`.
//!
//! ## 4. Postconditions
//! - Unpatched byte spans retain 100% byte-for-byte trivia and comment fidelity.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(P log P + N) for `P` patches over `N` source bytes.
//! - Space Complexity: O(N + P) output buffer.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Rejects overlapping, inverted, out-of-bounds, or mid-UTF-8 ranges with `[DEAP-E1005]`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]

#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]

use std::ops::Range;

use super::hir_lower::HirPackageSummary;
use super::parser::{parse_sysml_cst, ParsedSysmlCst};

/// Atomic byte-range replacement patch against a lossless Rowan CST source buffer.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Encapsulates a single formatting-preserving CST node splice (`BP-10`).
/// ## 2. Invariants & Mathematical Properties
/// `byte_range.start <= byte_range.end`.
/// ## 3. Preconditions
/// Byte offsets align to UTF-8 character boundaries in the target source.
/// ## 4. Postconditions
/// Used by `apply_formatting_preserving_splices` to update target CST spans in place.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(1) field access; Space Complexity: O(R) over `replacement.len()`.
/// ## 6. Failure Modes & Fault Tolerance
/// Validated by `validate_disjoint_splice_ranges` prior to application.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_8tsynkj2cufj]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CstSplicePatch {
    pub byte_range: Range<usize>,
    pub replacement: String,
}

/// Renders a `ParsedSysmlCst` back to its exact lossless source text.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Guarantees bijective round-trip source reconstruction from the Rowan green tree (`BP-03`).
/// ## 2. Invariants & Mathematical Properties
/// `print_lossless_cst(&parse_sysml_cst(s)) == s` for all `s`.
/// ## 3. Preconditions
/// `parsed` is a valid `ParsedSysmlCst`.
/// ## 4. Postconditions
/// Returns the full UTF-8 source text including all whitespace and comment trivia.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(N); Space Complexity: O(N).
/// ## 6. Failure Modes & Fault Tolerance
/// Total function with zero panics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn print_lossless_cst(parsed: &ParsedSysmlCst) -> String {
    parsed.syntax().text().to_string()
}

/// Formats a `HirPackageSummary` into canonical trivia-normalized SysML v2 text.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Produces a canonical textual projection of lowered HIR state (`BP-03`, `BP-10`).
/// ## 2. Invariants & Mathematical Properties
/// Independent of source trivia, comments, and byte spans.
/// ## 3. Preconditions
/// `hir` is a valid `HirPackageSummary`.
/// ## 4. Postconditions
/// Returns a deterministic multi-line canonical summary string.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(M) over HIR members; Space Complexity: O(M).
/// ## 6. Failure Modes & Fault Tolerance
/// Total pure function with zero panics.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn format_canonical_hir_summary(hir: &HirPackageSummary) -> String {
    let mut lines = Vec::new();
    lines.push(format!("package {} {{", hir.package_name));
    for imp in &hir.imports {
        lines.push(format!("    import {imp};"));
    }
    for m in &hir.members {
        if m.name == hir.package_name && m.kind_name == "PackageDecl" {
            continue;
        }
        let kw = if m.is_definition { "def" } else { "usage" };
        let ty = m
            .type_ref
            .as_ref()
            .map(|t| format!(" : {t}"))
            .unwrap_or_default();
        lines.push(format!("    /* {} */ {} {}{};", m.kind_name, kw, m.name, ty));
    }
    lines.push(String::from("}"));
    lines.join("\n")
}

/// Validates that all `CstSplicePatch` ranges are within `0..=source_len`, non-inverted, and
/// pairwise disjoint, returning the patches sorted in reverse byte order (`start` descending).
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Prevents overlapping or out-of-bounds CST splices under DO-178C Level A (`DEAP-E1005`).
/// ## 2. Invariants & Mathematical Properties
/// For every adjacent pair `(later, earlier)` in the returned vector, `earlier.byte_range.end <= later.byte_range.start`.
/// ## 3. Preconditions
/// `source_len` is the byte length of the target buffer.
/// ## 4. Postconditions
/// Returns `Ok(Vec<CstSplicePatch>)` sorted by descending `byte_range.start`, or `Err(String)` with `DEAP-E1005`.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(P log P); Space Complexity: O(P).
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed with `[DEAP-E1005]` on inverted, out-of-bounds, or overlapping ranges.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_9ucqbbd0s2yo]
pub fn validate_disjoint_splice_ranges(
    patches: &[CstSplicePatch],
    source_len: usize,
) -> Result<Vec<CstSplicePatch>, String> {
    let mut sorted = patches.to_vec();
    sorted.sort_by(|a, b| {
        a.byte_range
            .start
            .cmp(&b.byte_range.start)
            .then_with(|| a.byte_range.end.cmp(&b.byte_range.end))
    });
    let mut prev_end: Option<usize> = None;
    for patch in &sorted {
        if patch.byte_range.start > patch.byte_range.end {
            return Err(format!(
                "[DEAP-E1005] Inverted CST splice range {}..{}",
                patch.byte_range.start, patch.byte_range.end
            ));
        }
        if patch.byte_range.end > source_len {
            return Err(format!(
                "[DEAP-E1005] CST splice range {}..{} exceeds source byte length {source_len}",
                patch.byte_range.start, patch.byte_range.end
            ));
        }
        if let Some(pe) = prev_end {
            if patch.byte_range.start < pe {
                return Err(format!(
                    "[DEAP-E1005] Overlapping CST splice range {}..{} collides with prior end {pe}",
                    patch.byte_range.start, patch.byte_range.end
                ));
            }
        }
        prev_end = Some(patch.byte_range.end);
    }
    sorted.reverse();
    Ok(sorted)
}

/// Applies disjoint `CstSplicePatch` edits in reverse byte order while preserving 100% of
/// unpatched formatting, whitespace, and comments (`BP-10`).
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope
/// Executes non-destructive Rowan CST span splicing under DO-178C Level A (`DEAP-E1005`).
/// ## 2. Invariants & Mathematical Properties
/// Unmodified spans outside `patches` remain byte-for-byte identical.
/// ## 3. Preconditions
/// `source` is valid UTF-8; `patches` satisfy `validate_disjoint_splice_ranges`.
/// ## 4. Postconditions
/// Returns `Ok(String)` verified via `parse_sysml_cst` round-trip, or `Err(String)` on invalid spans.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: O(P log P + N); Space Complexity: O(N).
/// ## 6. Failure Modes & Fault Tolerance
/// Rejects non-char-boundary offsets and overlapping spans with `[DEAP-E1005]`.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
pub fn apply_formatting_preserving_splices(
    source: &str,
    patches: &[CstSplicePatch],
) -> Result<String, String> {
    let before_cst = parse_sysml_cst(source);
    if print_lossless_cst(&before_cst) != source {
        return Err(String::from(
            "[DEAP-E1005] Initial CST failed lossless round-trip invariant",
        ));
    }
    let reverse_sorted = validate_disjoint_splice_ranges(patches, source.len())?;
    let mut output = source.to_string();
    for patch in &reverse_sorted {
        let (start, end) = (patch.byte_range.start, patch.byte_range.end);
        let (Some(prefix), Some(suffix)) = (output.get(..start), output.get(end..)) else {
            return Err(format!(
                "[DEAP-E1005] CST splice range {start}..{end} splits a UTF-8 codepoint"
            ));
        };
        let mut next = String::with_capacity(
            prefix
                .len()
                .saturating_add(patch.replacement.len())
                .saturating_add(suffix.len()),
        );
        next.push_str(prefix);
        next.push_str(&patch.replacement);
        next.push_str(suffix);
        output = next;
    }
    let after_cst = parse_sysml_cst(&output);
    if print_lossless_cst(&after_cst) != output {
        return Err(String::from(
            "[DEAP-E1005] Spliced CST failed lossless round-trip invariant",
        ));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splicer_preserves_surrounding_comments_and_rejects_overlap() -> Result<(), String> {
        let src = "// header comment\npackage P {\n    part def OldName;\n}\n";
        let start = src.find("OldName").ok_or("missing OldName")?;
        let end = start.saturating_add("OldName".len());
        let spliced = apply_formatting_preserving_splices(
            src,
            &[CstSplicePatch {
                byte_range: start..end,
                replacement: "NewName".to_string(),
            }],
        )?;
        assert_eq!(spliced, "// header comment\npackage P {\n    part def NewName;\n}\n");
        let overlap = apply_formatting_preserving_splices(
            src,
            &[
                CstSplicePatch {
                    byte_range: 0..5,
                    replacement: "a".to_string(),
                },
                CstSplicePatch {
                    byte_range: 3..8,
                    replacement: "b".to_string(),
                },
            ],
        );
        assert!(overlap.is_err());
        Ok(())
    }
}
