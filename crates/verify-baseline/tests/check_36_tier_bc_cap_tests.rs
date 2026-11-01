#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Integration tests for Check 36 anti-padding `<= 4`-line Tier-B/C cap and explicit `[DOC-TIER: A]` opt-in.
//! Realises: [REQ-0605]

use verify_baseline::checks::{audit_rust_source_docstrings, is_compliant_rust_docblock};

/// Verifies ordinary `pub` items and `<= 4`-line `[REQ-*]`/`[FLS: fls_*]` Tier-B/C docblocks pass without `[DOC-TIER: A]` (`[REQ-0605]`).
#[test]
fn test_check_36_anti_padding_cap_and_explicit_doc_tier_a() {
    let concise_green = concat!(
        "/// Concise ordinary public struct without forced multi-section headers (`[REQ-0605]`).\n",
        "pub struct ConciseConfig {\n",
        "    pub limit: usize,\n",
        "}\n\n",
        "/// Concise 4-line Tier-B/C public function at exact cap.\n",
        "/// Validates bounded threshold input.\n",
        "/// Preserves deterministic saturating arithmetic.\n",
        "/// Traceability anchor (`[FLS: fls_saturating_addition]`).\n",
        "pub fn four_line_public_fn(v: usize) -> usize {\n",
        "    v.saturating_add(1)\n",
        "}\n\n",
        "/// Single-line private helper (`[REQ-0605]`).\n",
        "fn one_line_private_fn(v: usize) -> usize {\n",
        "    v\n",
        "}\n",
    );
    assert_eq!(
        audit_rust_source_docstrings(concise_green, "crates/demo/src/concise.rs"),
        Ok(3),
        "Ordinary pub struct, 4-line FLS pub fn, and 1-line REQ fn must pass Check 36 without [DOC-TIER: A]"
    );

    let padded_red = concat!(
        "/// Line 1 of padded Tier-B/C docstring (note: backticked `[DOC-TIER: A]` is prose).\n",
        "/// Line 2 of padded Tier-B/C docstring.\n",
        "/// Line 3 of padded Tier-B/C docstring.\n",
        "/// Line 4 of padded Tier-B/C docstring.\n",
        "/// Line 5 of padded Tier-B/C docstring (`[REQ-0605]`).\n",
        "pub fn five_line_padded_fn(v: usize) -> usize {\n",
        "    v\n",
        "}\n",
    );
    let red_errs = audit_rust_source_docstrings(padded_red, "crates/demo/src/padded.rs")
        .err()
        .unwrap_or_default()
        .join("\n");
    assert!(
        red_errs.contains("five_line_padded_fn") && red_errs.contains("exceeds <= 4 line cap"),
        "Expected > 4-line Tier-B/C docblock in non-ratcheted module to fail Check 36, got:\n{red_errs}"
    );

    assert_eq!(
        audit_rust_source_docstrings(
            padded_red,
            "crates/verify-baseline/src/checks/docstring_gate.rs"
        ),
        Ok(1),
        "Legacy ratcheted path must remain exempt from > 4-line Tier-B/C cap until Stream 4.3 compaction"
    );
}

/// Verifies `is_compliant_rust_docblock` enforces `<= 4`-line Tier-B/C cap unless `[DOC-TIER: A]` is present (`[REQ-0605]`).
#[test]
fn test_is_compliant_rust_docblock_enforces_tier_bc_cap_and_tier_a_opt_in() {
    let four_line_bc: Vec<String> = [
        "/// Summary line.",
        "/// Detail line 2.",
        "/// Detail line 3.",
        "/// Anchor (`[REQ-0605]`).",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert!(is_compliant_rust_docblock(&four_line_bc, false));

    let five_line_bc: Vec<String> = [
        "/// Summary line.",
        "/// Detail line 2.",
        "/// Detail line 3.",
        "/// Detail line 4.",
        "/// Anchor (`[REQ-0605]`).",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert!(!is_compliant_rust_docblock(&five_line_bc, false));

    let explicit_tier_a: Vec<String> = [
        "/// [DOC-TIER: A] Public API contract.",
        "///",
        "/// Safety & Traceability: Realises: [REQ-0605]",
        "/// Contract: Deterministic bounded check.",
        "/// Bounds & Faults: O(1) time and space; infallible.",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    assert!(is_compliant_rust_docblock(&explicit_tier_a, false));
}
