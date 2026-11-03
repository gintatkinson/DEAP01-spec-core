//! Unit tests for Check 36 docstring gate and delimiter depth trackers.
//! Realises: [REQ-CHECK-36-DOCSTRING-GATE], [DO-330-TQL-4-DOCSTRING-GATE]

use crate::checks::doc_gate_validate::update_line_delimiter_depths;
use crate::checks::docstring_gate::{audit_dart_source_docstrings, audit_rust_source_docstrings};
use deap_core::rust_cst::source_bridge::{
    scan_line_with_unified_lex_state, CstLexState, SourceLanguage,
};

#[test]
fn test_update_line_delimiter_depths_dart_interpolation_raw_and_utf8_char() {
    let mut braces = 1usize;
    let mut parens = 0usize;
    update_line_delimiter_depths(r#"final s = "val: ${map['}']}"; }"#, &mut braces, &mut parens);
    assert_eq!(braces, 0, "closing brace inside ${{...}} string literal must not decrement outer brace depth");

    let mut braces_raw = 1usize;
    update_line_delimiter_depths(r#"final r = r"raw { string"; }"#, &mut braces_raw, &mut parens);
    assert_eq!(braces_raw, 0, "brace inside Dart raw string r\"...\" must be ignored");

    let mut braces_utf8 = 1usize;
    update_line_delimiter_depths("let ch = 'é'; let b = '{'; }", &mut braces_utf8, &mut parens);
    assert_eq!(braces_utf8, 0, "multi-byte UTF-8 char literal must not treat trailing quote as string start");

    let mut braces_esc = 1usize;
    update_line_delimiter_depths(r"let ch = '\''; let b = '{'; }", &mut braces_esc, &mut parens);
    assert_eq!(braces_esc, 0, "escaped single quote char literal must be consumed cleanly");
}

#[test]
fn test_audit_rust_source_docstrings_preserves_double_slash_in_string_literal() {
    let valid_fn_doc = "/// Summary.\n/// Realises: [REQ-1]\n";
    let src = format!(
        "{doc}pub fn check_url() {{\n    let u = \"http://example.com\";\n}}\nfn undocumented_after_url() {{}}\n",
        doc = valid_fn_doc
    );
    let res = audit_rust_source_docstrings(&src, "crates/demo/src/lib.rs");
    assert!(
        res.is_err(),
        "undocumented function after string literal with '//' must be caught when brace depth closes"
    );
}

#[test]
fn test_audit_rust_source_docstrings_ignores_multihash_raw_and_multiline_strings(
) -> Result<(), String> {
    let valid_fn_doc = "/// Summary.\n/// Realises: [REQ-1]\n";
    let src = format!(
        "{doc}pub fn raw_str_holder() {{\n    let a = r##\"raw with \"# and {{ inside\"##;\n    let b = \"multiline {{\npub fn fake_inside_string() {{}}\n\";\n}}\nfn undocumented_after_raw() {{}}\n",
        doc = valid_fn_doc
    );
    let res = audit_rust_source_docstrings(&src, "crates/demo/src/lib.rs");
    let Err(errs) = res else {
        return Err("undocumented function after multi-hash raw string must be caught".into());
    };
    assert!(
        errs.iter().any(|e| e.contains("undocumented_after_raw")),
        "expected undocumented_after_raw violation, got: {:?}",
        errs
    );
    assert!(
        !errs.iter().any(|e| e.contains("fake_inside_string")),
        "fake declaration inside multiline string must be ignored, got: {:?}",
        errs
    );
    Ok(())
}

#[test]
fn test_audit_dart_source_docstrings_ignores_triple_quoted_strings() -> Result<(), String> {
    let valid_dart_doc = "/// Summary.\n/// ## 1. Safety Intent & Regulatory Scope\n/// A.\n/// ## 2. Invariants & Mathematical Properties\n/// B.\n/// ## 3. Preconditions\n/// C.\n/// ## 4. Postconditions\n/// D.\n/// ## 5. Algorithmic Complexity Bounds\n/// E.\n/// ## 6. Failure Modes & Fault Tolerance\n/// F.\n/// ## 7. Realization & Traceability\n/// Realises: [REQ-1]\n";
    let src = format!(
        "{doc}void holder() {{\n  final s = '''\nclass FakeInTripleQuote {{}}\n{{\n''';\n}}\nclass UndocumentedAfterTriple {{}}\n",
        doc = valid_dart_doc
    );
    let res = audit_dart_source_docstrings(&src, "app_flutter/lib/demo.dart");
    let Err(errs) = res else {
        return Err("undocumented class after Dart triple-quoted string must be caught".into());
    };
    assert!(
        errs.iter().any(|e| e.contains("UndocumentedAfterTriple")),
        "expected UndocumentedAfterTriple violation, got: {:?}",
        errs
    );
    assert!(
        !errs.iter().any(|e| e.contains("FakeInTripleQuote")),
        "FakeInTripleQuote inside triple-quoted string must not be flagged, got: {:?}",
        errs
    );
    Ok(())
}

#[test]
fn test_consume_rust_char_literal_and_nested_block_comments() {
    let mut state = CstLexState {
        language: SourceLanguage::Rust,
        ..Default::default()
    };
    let stripped = scan_line_with_unified_lex_state(
        "let a = '\\u{1F600}'; /* outer /* inner */ still outer */ let b = 1; // trailing",
        &mut state,
    );
    assert_eq!(
        stripped.split_whitespace().collect::<Vec<_>>().join(" "),
        "let a = '\\u{1F600}'; let b = 1;"
    );
    assert_eq!(state.block_comment_depth, 0);
}

#[test]
fn test_audit_rust_tiered_docstrings_accepts_tier_a_and_tier_bc() {
    let src = "\
//! Module summary for Tier A header.
//! - **Safety & Traceability**: DO-178C Level A. Realises: [REQ-MOD-1]
//! - **Contract**: Pre: valid input. Post: deterministic result. Invariant: pure.
//! - **Bounds & Faults**: O(1) time/space; returns Err on invalid state.

/// Concise Tier B/C summary for internal helper.
/// Realises: [REQ-HELPER-1]
fn internal_helper() -> usize {
    42
}

/// Public API function using Tier A 4-Block Contract.
/// - **Safety & Traceability**: ASIL-D gate. Realises: [REQ-PUB-1]
/// - **Contract**: Pre: x > 0. Post: returns x + 1. Invariant: no overflow.
/// - **Bounds & Faults**: O(1) time, O(1) space; saturates on usize::MAX.
pub fn public_entry(x: usize) -> usize {
    x.saturating_add(internal_helper())
}
";
    let res = audit_rust_source_docstrings(src, "crates/demo/src/new_module.rs");
    assert!(
        res.is_ok(),
        "valid Tier A module + Tier B/C private fn + Tier A pub fn must pass: {:?}",
        res
    );

    let bloated_bc = "\
/// Line 1 summary.
/// Line 2 extra.
/// Line 3 extra.
/// Line 4 extra.
/// Line 5 exceeds 4-line cap.
/// Realises: [REQ-HELPER-2]
fn over_cap_helper() {}
";
    let err_res = audit_rust_source_docstrings(bloated_bc, "crates/demo/src/new_module.rs");
    assert!(
        err_res.is_err(),
        "unratcheted Tier B/C function exceeding 4 doc lines must be rejected"
    );
}
