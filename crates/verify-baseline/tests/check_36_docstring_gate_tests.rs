//! Integration tests for Check 36: Per-Item 7-Section Safety Docstring Completeness Gate (Part A).

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use verify_baseline::checks::{
    audit_dart_source_docstrings, audit_rust_source_docstrings, audit_source_purity,
    check_docstring_completeness,
};

struct ScopedTempWorkspace {
    root: PathBuf,
}

impl ScopedTempWorkspace {
    fn new(prefix: &str) -> Result<Self, Box<dyn Error>> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!(
            "deap_check36_{}_{}_{}",
            prefix,
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }
}

impl Drop for ScopedTempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn test_check_36_rejects_placeholder_and_missing_docstrings() -> Result<(), Box<dyn Error>> {
    let defective_dart = r#"import 'package:flutter/material.dart';

class MyApp extends StatelessWidget {
  /// Member documentation.
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return const MaterialApp(
      home: Scaffold(),
    );
  }
}
"#;

    let dart_res = audit_dart_source_docstrings(defective_dart, "app_flutter/lib/app/app.dart");
    assert!(
        dart_res.is_err(),
        "Check 36 must reject Dart file with '/// Member documentation.' and missing 7-section docstrings"
    );
    let dart_errs = dart_res.err().unwrap_or_default().join("\n");
    assert!(
        dart_errs.contains("Member documentation."),
        "Expected placeholder docstring diagnostic for '/// Member documentation.', got:\n{}",
        dart_errs
    );
    assert!(
        dart_errs.contains("MyApp"),
        "Expected missing 7-section docstring diagnostic on class/constructor MyApp, got:\n{}",
        dart_errs
    );

    let defective_rust = r#"/// Member documentation.
pub struct SensorPacket {
    pub seq: u64,
}

pub fn decode_packet(raw: &[u8]) -> Option<SensorPacket> {
    if raw.is_empty() {
        return None;
    }
    Some(SensorPacket { seq: 1 })
}
"#;

    let rust_res = audit_rust_source_docstrings(defective_rust, "crates/demo/src/lib.rs");
    assert!(
        rust_res.is_err(),
        "Check 36 must reject Rust file with '/// Member documentation.' and missing 7-section docstrings"
    );
    let rust_errs = rust_res.err().unwrap_or_default().join("\n");
    assert!(
        rust_errs.contains("Member documentation."),
        "Expected placeholder docstring diagnostic in Rust source, got:\n{}",
        rust_errs
    );
    assert!(
        rust_errs.contains("SensorPacket") && rust_errs.contains("decode_packet"),
        "Expected diagnostics for both SensorPacket and decode_packet, got:\n{}",
        rust_errs
    );

    let ws = ScopedTempWorkspace::new("placeholder_and_missing")?;
    let dart_dir = ws.root.join("app_flutter").join("lib").join("app");
    fs::create_dir_all(&dart_dir)?;
    fs::write(dart_dir.join("app.dart"), defective_dart)?;

    let ws_res = check_docstring_completeness(&ws.root);
    assert!(
        ws_res.is_err(),
        "check_docstring_completeness must fail closed on workspace with defective app.dart"
    );
    Ok(())
}

fn lib_rs_with_module_doc_fixture() -> &'static str {
    r#"//! Sample crate module documentation.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Module-level safety intent under DO-178C Level A.
//!
//! ## 2. Invariants & Mathematical Properties
//! Module-level invariants hold across exported types.
//!
//! ## 3. Preconditions
//! Caller supplies valid UTF-8 inputs.
//!
//! ## 4. Postconditions
//! Module operations return deterministic results.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(N)
//! - Space Complexity: O(N)
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Rejects malformed inputs.
//! - State: Preserves immutable state.
//! - Action: Deterministic execution.
//! - Resource: Bounded memory.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SAMPLE-MOD]

pub mod submod;

pub struct UndocumentedInLib {
    pub value: u32,
}

pub fn undocumented_lib_fn(x: u32) -> u32 {
    x.saturating_add(1)
}
"#
}

fn submod_rs_with_module_doc_fixture() -> &'static str {
    r#"//! Submodule header with 7 sections that must never exempt items.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Submodule safety intent.
//!
//! ## 2. Invariants & Mathematical Properties
//! Submodule invariants.
//!
//! ## 3. Preconditions
//! Submodule preconditions.
//!
//! ## 4. Postconditions
//! Submodule postconditions.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(1)
//! - Space Complexity: O(1)
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: None.
//! - State: Pure.
//! - Action: Infallible.
//! - Resource: O(1).
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SUBMOD]

pub enum UndocumentedEnum {
    Alpha,
    Beta,
}

pub trait UndocumentedTrait {
    fn compute(&self) -> u32;
}

struct PrivateUndocumentedStruct {
    count: usize,
}

impl PrivateUndocumentedStruct {
    fn increment(&mut self) {
        self.count = self.count.saturating_add(1);
    }
}
"#
}

#[test]
fn test_check_36_rejects_undocumented_rust_items_even_when_lib_rs_has_module_doc(
) -> Result<(), Box<dyn Error>> {
    let ws = ScopedTempWorkspace::new("no_module_exemption")?;
    let src_dir = ws.root.join("crates").join("sample-crate").join("src");
    fs::create_dir_all(&src_dir)?;

    fs::write(src_dir.join("lib.rs"), lib_rs_with_module_doc_fixture())?;
    fs::write(
        src_dir.join("submod.rs"),
        submod_rs_with_module_doc_fixture(),
    )?;

    let res = check_docstring_completeness(&ws.root);
    assert!(
        res.is_err(),
        "Check 36 must NEVER exempt structs, enums, traits, or functions when lib.rs or submodule has //! module doc"
    );
    let errs = res.err().unwrap_or_default().join("\n");
    for expected_symbol in [
        "UndocumentedInLib",
        "undocumented_lib_fn",
        "UndocumentedEnum",
        "UndocumentedTrait",
        "PrivateUndocumentedStruct",
        "increment",
    ] {
        assert!(
            errs.contains(expected_symbol),
            "Expected Check 36 to flag undocumented symbol '{}' despite //! module doc, got:\n{}",
            expected_symbol,
            errs
        );
    }
    Ok(())
}

/// Verifies that `audit_source_purity` fails closed on outer and inner clippy lint suppressions.
#[test]
fn test_check_36_source_purity_negative_probes() {
    let outer_allow = "#[allow(clippy::too_many_arguments)]\npub fn impure_fn() {}\n";
    assert!(
        audit_source_purity(outer_allow, "crates/demo/src/lib.rs", true).is_err(),
        "audit_source_purity must reject outer clippy allow attribute in production source"
    );

    let inner_allow = "#![allow(clippy::unwrap_used)]\npub fn impure_mod_fn() {}\n";
    assert!(
        audit_source_purity(inner_allow, "crates/demo/src/lib.rs", true).is_err(),
        "audit_source_purity must reject inner clippy allow attribute in production source"
    );

    let test_allow = "\
#[cfg(test)]
mod tests {
    #[allow(clippy::unwrap_used)]
    #[test]
    fn bad_test() {}
}
";
    assert!(
        audit_source_purity(test_allow, "crates/demo/src/lib.rs", true).is_err(),
        "audit_source_purity must reject clippy allow attribute inside #[cfg(test)] when check_test_targets is true"
    );
}

#[test]
fn test_check_36_inline_comment_and_literal_token_separation_issue_517() {
    let rust_fixture = r##"/* outer /* inner */ pub fn fake_undocumented() {} */
pub/*comment*/fn undocumented_comment_fn() {}
pub/*comment*/struct UndocumentedCommentStruct {}
pub extern"C"fn undocumented_extern_fn() {}
pub extern r#"C"#fn undocumented_raw_extern_fn() {}
"##;

    let rust_res = audit_rust_source_docstrings(rust_fixture, "crates/demo/src/lib.rs");
    assert!(
        rust_res.is_err(),
        "Check 36 must reject undocumented Rust items separated by inline comments or string literals, got: {:?}",
        rust_res
    );
    let rust_errs = rust_res.err().unwrap_or_default().join("\n");
    assert!(
        !rust_errs.contains("fake_undocumented"),
        "Nested block comment item must not be flagged, got:\n{}",
        rust_errs
    );
    for expected in [
        "undocumented_comment_fn",
        "UndocumentedCommentStruct",
        "undocumented_extern_fn",
        "undocumented_raw_extern_fn",
    ] {
        assert!(
            rust_errs.contains(expected),
            "Expected Check 36 to flag '{}', got:\n{}",
            expected,
            rust_errs
        );
    }

    let dart_fixture = "\
/* outer /* inner */ void fakeUndocumentedDartFn() {} */
void/*comment*/undocumentedDartFn() {}
";
    let dart_res = audit_dart_source_docstrings(dart_fixture, "app_flutter/lib/inline.dart");
    assert!(
        dart_res.is_err(),
        "Check 36 must reject undocumented Dart function separated by inline block comment, got: {:?}",
        dart_res
    );
    let dart_errs = dart_res.err().unwrap_or_default().join("\n");
    assert!(
        !dart_errs.contains("fakeUndocumentedDartFn"),
        "Nested Dart block comment item must not be flagged, got:\n{}",
        dart_errs
    );
    assert!(
        dart_errs.contains("undocumentedDartFn"),
        "Expected Check 36 to flag 'undocumentedDartFn', got:\n{}",
        dart_errs
    );
}
