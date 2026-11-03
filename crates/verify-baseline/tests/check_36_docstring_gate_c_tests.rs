//! Integration tests for Check 36: Dart multiline classes, string slashes, and nested block comments (Part C).

use verify_baseline::checks::{audit_dart_source_docstrings, audit_rust_source_docstrings};

fn valid_dart_services_fixture_part_a() -> &'static str {
    r#"
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A PrimaryService classification.
///
/// ## 2. Invariants & Mathematical Properties
/// - Endpoint URL points to secure API gateway.
///
/// ## 3. Preconditions
/// - System network stack is initialized.
///
/// ## 4. Postconditions
/// - Instantiates primary service handler.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) constant time initialization.
/// - Space Complexity: O(1) instance allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Network failures fall back to secondary service.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SERV-01-PRIMARY-SERVICE]
class PrimaryService {
  final String url = "https://example.com/api//v1";

  /// ## 1. Safety Intent & Regulatory Scope
  /// PrimaryService constructor safety intent.
  ///
  /// ## 2. Invariants & Mathematical Properties
  /// - Default instance state.
  ///
  /// ## 3. Preconditions
  /// - None.
  ///
  /// ## 4. Postconditions
  /// - Returns new PrimaryService.
  ///
  /// ## 5. Algorithmic Complexity Bounds
  /// - Time: O(1).
  /// - Space: O(1).
  ///
  /// ## 6. Failure Modes & Fault Tolerance
  /// - Infallible constructor.
  ///
  /// ## 7. Realization & Traceability
  /// Realises: [REQ-SERV-01-PRIMARY-SERVICE-CTOR]
  PrimaryService();

  /// ## 1. Safety Intent & Regulatory Scope
  /// Named factory constructor for JSON initialization.
  ///
  /// ## 2. Invariants & Mathematical Properties
  /// - Validates JSON configuration map.
  ///
  /// ## 3. Preconditions
  /// - json map is not null.
  ///
  /// ## 4. Postconditions
  /// - Returns initialized PrimaryService.
  ///
  /// ## 5. Algorithmic Complexity Bounds
  /// - Time: O(1).
  /// - Space: O(1).
  ///
  /// ## 6. Failure Modes & Fault Tolerance
  /// - Invalid fields throw FormatException.
  ///
  /// ## 7. Realization & Traceability
  /// Realises: [REQ-SERV-01-PRIMARY-SERVICE-FROM-CONFIG]
  factory PrimaryService.fromConfig(Map<String, dynamic> json) => PrimaryService();
"#
}

fn valid_dart_services_fixture_part_b() -> &'static str {
    r#"
  /// ## 1. Safety Intent & Regulatory Scope
  /// Getter for service display name.
  ///
  /// ## 2. Invariants & Mathematical Properties
  /// - Returns static identifier.
  ///
  /// ## 3. Preconditions
  /// - None.
  ///
  /// ## 4. Postconditions
  /// - Returns non-empty string.
  ///
  /// ## 5. Algorithmic Complexity Bounds
  /// - Time: O(1).
  /// - Space: O(1).
  ///
  /// ## 6. Failure Modes & Fault Tolerance
  /// - Infallible getter.
  ///
  /// ## 7. Realization & Traceability
  /// Realises: [REQ-SERV-01-PRIMARY-SERVICE-NAME]
  String get name => "Primary";

  /// ## 1. Safety Intent & Regulatory Scope
  /// Executes service operations.
  ///
  /// ## 2. Invariants & Mathematical Properties
  /// - Logs message with interpolated braces.
  ///
  /// ## 3. Preconditions
  /// - Service is running.
  ///
  /// ## 4. Postconditions
  /// - Completes async operation.
  ///
  /// ## 5. Algorithmic Complexity Bounds
  /// - Time: O(1).
  /// - Space: O(1).
  ///
  /// ## 6. Failure Modes & Fault Tolerance
  /// - Caught and recorded in audit log.
  ///
  /// ## 7. Realization & Traceability
  /// Realises: [REQ-SERV-01-PRIMARY-SERVICE-EXEC]
  @override
  void execute() {
    final log = "Status: ${name.toUpperCase()}//ok";
  }
}
"#
}

fn valid_dart_services_fixture_part_c() -> &'static str {
    r#"
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A SecondaryService classification.
///
/// ## 2. Invariants & Mathematical Properties
/// - Secondary failover controller.
///
/// ## 3. Preconditions
/// - System online.
///
/// ## 4. Postconditions
/// - Creates secondary controller instance.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time: O(1).
/// - Space: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SERV-02-SECONDARY-SERVICE]
class SecondaryService {
  /// ## 1. Safety Intent & Regulatory Scope
  /// SecondaryService constructor.
  ///
  /// ## 2. Invariants & Mathematical Properties
  /// - Initialized state.
  ///
  /// ## 3. Preconditions
  /// - None.
  ///
  /// ## 4. Postconditions
  /// - New instance.
  ///
  /// ## 5. Algorithmic Complexity Bounds
  /// - Time: O(1).
  /// - Space: O(1).
  ///
  /// ## 6. Failure Modes & Fault Tolerance
  /// - Infallible.
  ///
  /// ## 7. Realization & Traceability
  /// Realises: [REQ-SERV-02-SECONDARY-SERVICE-CTOR]
  SecondaryService();

  /// ## 1. Safety Intent & Regulatory Scope
  /// Reset method for secondary service.
  ///
  /// ## 2. Invariants & Mathematical Properties
  /// - Resets all internal buffers.
  ///
  /// ## 3. Preconditions
  /// - Secondary service online.
  ///
  /// ## 4. Postconditions
  /// - Clean buffers.
  ///
  /// ## 5. Algorithmic Complexity Bounds
  /// - Time: O(1).
  /// - Space: O(1).
  ///
  /// ## 6. Failure Modes & Fault Tolerance
  /// - Infallible.
  ///
  /// ## 7. Realization & Traceability
  /// Realises: [REQ-SERV-02-SECONDARY-SERVICE-RESET]
  @visibleForTesting
  @override
  void reset() {}
}
"#
}

#[test]
fn test_check_36_catches_multiline_dart_classes_string_slashes_interpolated_quotes_and_dart3_items()
{
    let valid_dart = format!(
        "{}{}{}",
        valid_dart_services_fixture_part_a(),
        valid_dart_services_fixture_part_b(),
        valid_dart_services_fixture_part_c()
    );

    let res = audit_dart_source_docstrings(&valid_dart, "lib/services.dart");
    assert_eq!(res, Ok(8), "Should classify exactly 8 items");

    let missing_getter_doc = valid_dart.replace(
        "  /// ## 1. Safety Intent & Regulatory Scope\n  /// Getter for service display name.",
        "  // Naive comment\n  // not docstring",
    );
    let neg_res = audit_dart_source_docstrings(&missing_getter_doc, "lib/services.dart");
    assert!(
        neg_res.is_err(),
        "Missing docstring on getter must fail closed"
    );
}

fn nested_comment_rust_fixture() -> &'static str {
    "\
/* outer block comment
   /* inner nested block comment */
   /// Member documentation.
   pub fn fake_inside_nested_comment() {}
*/
/// [DOC-TIER: A] Documented function using hex and unicode character escapes alongside lifetimes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Verifies nested block comments and escaped char literals under DO-178C Level A.
///
/// ## 2. Invariants & Mathematical Properties
/// - Character escapes `\\x7F` and `\\u{10FFFF}` never perturb string or comment state.
///
/// ## 3. Preconditions
/// - Input slice is valid UTF-8.
///
/// ## 4. Postconditions
/// - Returns first matching char.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1).
/// - Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function with zero panics.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-NESTED-BLOCK-COMMENT]
pub fn inspect_char_escapes<'a>(input: &'a str) -> (char, char, &'a str) {
    let del = '\\x7F';
    let max_scalar = '\\u{10FFFF}';
    let quote_ch = '\\\"';
    let _ = quote_ch;
    (del, max_scalar, input)
}
"
}

fn nested_comment_dart_fixture() -> &'static str {
    "\
/* outer dart block comment
   /* inner nested dart block comment */
   /// Member documentation.
   class FakeDartClassInNestedComment {}
*/
/// Documented Dart function after nested block comment.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Verifies nested Dart block comments under DO-178C Level A.
///
/// ## 2. Invariants & Mathematical Properties
/// - Nested `/* ... */` preserves comment depth.
///
/// ## 3. Preconditions
/// - None.
///
/// ## 4. Postconditions
/// - Completes deterministically.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1).
/// - Space Complexity: O(1).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Infallible execution.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-CHECK-36-DART-NESTED-BLOCK-COMMENT]
void realDartFn() {}
"
}

#[test]
fn test_check_36_nested_block_comments_and_escaped_char_literals() {
    let rust_source = nested_comment_rust_fixture();
    assert_eq!(
        audit_rust_source_docstrings(rust_source, "crates/demo/src/lib.rs"),
        Ok(1),
        "Nested Rust block comment must remain active after inner `*/` until outer `*/`"
    );

    let dart_source = nested_comment_dart_fixture();
    assert_eq!(
        audit_dart_source_docstrings(dart_source, "app_flutter/lib/nested.dart"),
        Ok(1),
        "Nested Dart block comment must remain active after inner `*/` until outer `*/`"
    );

    let mut rust_neg = String::from(rust_source);
    rust_neg.push_str("\npub fn undocumented_after_nested_comment() {}\n");
    let rust_neg_res = audit_rust_source_docstrings(&rust_neg, "crates/demo/src/lib.rs");
    assert!(
        rust_neg_res.is_err(),
        "Undocumented Rust function after nested block comment must fail closed"
    );
    assert!(
        rust_neg_res
            .err()
            .unwrap_or_default()
            .iter()
            .any(|e| e.contains("undocumented_after_nested_comment")),
        "Expected diagnostic for undocumented_after_nested_comment"
    );

    let mut dart_neg = String::from(dart_source);
    dart_neg.push_str("\nvoid undocumentedAfterNestedDartComment() {}\n");
    let dart_neg_res = audit_dart_source_docstrings(&dart_neg, "app_flutter/lib/nested.dart");
    assert!(
        dart_neg_res.is_err(),
        "Undocumented Dart function after nested block comment must fail closed"
    );
    assert!(
        dart_neg_res
            .err()
            .unwrap_or_default()
            .iter()
            .any(|e| e.contains("undocumentedAfterNestedDartComment")),
        "Expected diagnostic for undocumentedAfterNestedDartComment"
    );
}
