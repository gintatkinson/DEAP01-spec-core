//! Integration tests for Check 36: Dart interpolated nested quotes, multiline annotations, and Rust scoped cfg(test) / qualifiers (Part D).

use verify_baseline::checks::{audit_dart_source_docstrings, audit_rust_source_docstrings};

fn dart_http_interp_fixture() -> &'static str {
    r#"
/// Documented Dart telemetry formatter class.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Formats telemetry endpoints under DO-178C Level A.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves nested string interpolations containing comment tokens.
///
/// ## 3. Preconditions
/// - Input map contains valid string keys.
///
/// ## 4. Postconditions
/// - Instantiates formatter class.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1).
/// - Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible class declaration.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DART-INTERP-CLASS-01]
class TelemetryFormatter {
  /// Formats endpoint URI from configuration map.
  ///
  /// ## 1. Safety Intent & Regulatory Scope
  /// Evaluates nested quotes in string interpolations safely.
  ///
  /// ## 2. Invariants & Mathematical Properties
  /// - Token `//` inside nested interpolation quotes remains string data.
  ///
  /// ## 3. Preconditions
  /// - `map` is non-null.
  ///
  /// ## 4. Postconditions
  /// - Returns formatted endpoint string.
  ///
  /// ## 5. Algorithmic Complexity Bounds
  /// - Time Complexity: O(1).
  /// - Space Complexity: O(1).
  ///
  /// ## 6. Failure Modes & Fault Tolerance
  /// - Falls back to empty string on missing key.
  ///
  /// ## 7. Realization & Traceability
  /// Realises: [REQ-DART-INTERP-METHOD-01]
  String formatEndpoint(Map<String, String> map) {
    if (map.isNotEmpty) { final uri = "${map["http://"]}"; return uri; }
    return "";
  }

  void undocumentedAfterHttpInterp() {}
}
"#
}

fn dart_block_open_interp_fixture() -> &'static str {
    r#"
/// Documented Dart telemetry formatter class.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Formats telemetry endpoints under DO-178C Level A.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves nested string interpolations containing block-comment tokens.
///
/// ## 3. Preconditions
/// - Input map contains valid string keys.
///
/// ## 4. Postconditions
/// - Instantiates formatter class.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1).
/// - Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible class declaration.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-DART-INTERP-CLASS-02]
class GlobFormatter {
  /// Formats glob pattern from configuration map.
  ///
  /// ## 1. Safety Intent & Regulatory Scope
  /// Evaluates nested quotes containing `/*` in string interpolations safely.
  ///
  /// ## 2. Invariants & Mathematical Properties
  /// - Token `/*` inside nested interpolation quotes never increments `block_comment_depth`.
  ///
  /// ## 3. Preconditions
  /// - `map` is non-null.
  ///
  /// ## 4. Postconditions
  /// - Returns formatted glob string.
  ///
  /// ## 5. Algorithmic Complexity Bounds
  /// - Time Complexity: O(1).
  /// - Space Complexity: O(1).
  ///
  /// ## 6. Failure Modes & Fault Tolerance
  /// - Falls back to empty string on missing key.
  ///
  /// ## 7. Realization & Traceability
  /// Realises: [REQ-DART-INTERP-METHOD-02]
  String formatGlob(Map<String, String> map) {
    final glob = "${map["/*"]}";
    return glob;
  }

  void undocumentedAfterBlockOpenInterp() {}
}
"#
}

fn dart_multiline_annotation_fixture() -> &'static str {
    r#"
/// Documented Dart telemetry formatter class.
/// ## 1. Safety Intent & Regulatory Scope
/// Formats telemetry endpoints under DO-178C Level A.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves nested string interpolations containing comment tokens.
/// ## 3. Preconditions
/// - Input map contains valid string keys.
/// ## 4. Postconditions
/// - Instantiates formatter class.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1).
/// - Space Complexity: O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible class declaration.
/// ## 7. Realization & Traceability
/// Realises: [REQ-DART-INTERP-CLASS-01]
class TelemetryFormatter {
  /// Formats endpoint URI and glob pattern from configuration map.
  /// ## 1. Safety Intent & Regulatory Scope
  /// Evaluates nested quotes in string interpolations safely.
  /// ## 2. Invariants & Mathematical Properties
  /// - Tokens `//` and `/*` inside nested interpolation quotes remain string data.
  /// ## 3. Preconditions
  /// - `map` is non-null.
  /// ## 4. Postconditions
  /// - Returns formatted endpoint string.
  /// ## 5. Algorithmic Complexity Bounds
  /// - Time Complexity: O(1).
  /// - Space Complexity: O(1).
  /// ## 6. Failure Modes & Fault Tolerance
  /// - Falls back to empty string on missing key.
  /// ## 7. Realization & Traceability
  /// Realises: [REQ-DART-INTERP-METHOD-01]
  String formatEndpoint(Map<String, String> map) {
    if (map.isNotEmpty) { final uri = "${map["http://"]}"; return uri; }
    final glob = "${map["/*"]}";
    return glob;
  }

  /// Legacy reset endpoint preserved across multiline Dart annotation.
  /// ## 1. Safety Intent & Regulatory Scope
  /// Verifies docstring retention across multiline annotations under DO-178C Level A.
  /// ## 2. Invariants & Mathematical Properties
  /// - `annotation_paren_depth` preserves `current_doc` until closing `)`.
  /// ## 3. Preconditions
  /// - Formatter instance is initialized.
  /// ## 4. Postconditions
  /// - Resets formatter state cleanly.
  /// ## 5. Algorithmic Complexity Bounds
  /// - Time Complexity: O(1).
  /// - Space Complexity: O(1).
  /// ## 6. Failure Modes & Fault Tolerance
  /// - No-op reset cannot fail.
  /// ## 7. Realization & Traceability
  /// Realises: [REQ-DART-INTERP-METHOD-03]
  @Deprecated(
    'Use formatEndpoint instead',
  )
  void legacyReset() {}
}
"#
}

#[test]
fn test_check_36_dart_interpolated_nested_quotes_with_comment_tokens_and_multiline_annotations() {
    let neg_http_res =
        audit_dart_source_docstrings(dart_http_interp_fixture(), "app_flutter/lib/telemetry.dart");
    assert!(
        neg_http_res.is_err(),
        "Undocumented method after interpolated 'http://' quote must be flagged, got: {:?}",
        neg_http_res
    );
    let neg_http_errs = neg_http_res.err().unwrap_or_default().join("\n");
    assert!(
        neg_http_errs.contains("undocumentedAfterHttpInterp"),
        "Expected violation for undocumentedAfterHttpInterp, got:\n{}",
        neg_http_errs
    );

    let neg_block_res = audit_dart_source_docstrings(
        dart_block_open_interp_fixture(),
        "app_flutter/lib/glob.dart",
    );
    assert!(
        neg_block_res.is_err(),
        "Undocumented method after interpolated '/*' quote must be flagged, got: {:?}",
        neg_block_res
    );
    let neg_block_errs = neg_block_res.err().unwrap_or_default().join("\n");
    assert!(
        neg_block_errs.contains("undocumentedAfterBlockOpenInterp"),
        "Expected violation for undocumentedAfterBlockOpenInterp, got:\n{}",
        neg_block_errs
    );

    assert_eq!(
        audit_dart_source_docstrings(
            dart_multiline_annotation_fixture(),
            "app_flutter/lib/telemetry.dart"
        ),
        Ok(3),
        "Expected class + 2 documented methods (including multiline @Deprecated) to return Ok(3)"
    );
}

fn rust_cfg_test_impl_fixture() -> &'static str {
    r#"/// [DOC-TIER: A] Documented carrier struct for cfg(test) impl block verification.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Verifies scoped #[cfg(test)] impl block exemption under DO-178C Level A.
///
/// ## 2. Invariants & Mathematical Properties
/// - Production struct requires 7-section docstring while #[cfg(test)] impl methods are test-scoped.
///
/// ## 3. Preconditions
/// - None.
///
/// ## 4. Postconditions
/// - Instantiates unit carrier struct.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1).
/// - Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Zero-sized struct cannot fail.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CFG-TEST-IMPL-01]
pub struct Carrier;

#[cfg(test)]
impl Carrier {
    pub fn test_only_helper() -> bool {
        true
    }
}
"#
}

#[test]
fn test_check_36_rust_scoped_cfg_test_and_extended_qualifiers() {
    let rust_after_mod_tests = r#"#[cfg(test)]
mod tests;

#[cfg(test)]
mod inline_tests {
    fn helper_in_test_mod() {}
}

pub fn undocumented_after_test_mod() {}
"#;
    let after_mod_res =
        audit_rust_source_docstrings(rust_after_mod_tests, "crates/demo/src/lib.rs");
    assert!(
        after_mod_res.is_err(),
        "Production function after #[cfg(test)] mod tests must not be exempted, got: {:?}",
        after_mod_res
    );
    let after_mod_errs = after_mod_res.err().unwrap_or_default().join("\n");
    assert!(
        after_mod_errs.contains("undocumented_after_test_mod"),
        "Expected undocumented_after_test_mod violation, got:\n{}",
        after_mod_errs
    );

    assert_eq!(
        audit_rust_source_docstrings(rust_cfg_test_impl_fixture(), "crates/demo/src/lib.rs"),
        Ok(1),
        "#[cfg(test)] impl methods must be exempted while Carrier struct is counted"
    );

    let rust_extended_qualifiers = r#"
pub(self) fn hidden_self_fn() {}
pub unsafe fn raw_ptr_fn() {}
pub unsafe trait RawSafetyTrait {
    fn check(&self);
}
pub extern "C" fn ffi_entrypoint() {}
"#;
    let qual_res = audit_rust_source_docstrings(rust_extended_qualifiers, "crates/demo/src/lib.rs");
    assert!(
        qual_res.is_err(),
        "Undocumented pub(self), unsafe, and extern items must fail closed, got: {:?}",
        qual_res
    );
    let qual_errs = qual_res.err().unwrap_or_default().join("\n");
    for expected in [
        "hidden_self_fn",
        "raw_ptr_fn",
        "RawSafetyTrait",
        "ffi_entrypoint",
    ] {
        assert!(
            qual_errs.contains(expected),
            "Expected Check 36 to flag undocumented qualified item '{}', got:\n{}",
            expected,
            qual_errs
        );
    }
}
