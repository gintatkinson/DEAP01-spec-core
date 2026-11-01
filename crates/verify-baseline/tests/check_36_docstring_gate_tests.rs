//! Integration tests for Check 36: Per-Item 7-Section Safety Docstring Completeness Gate
//! across Rust (`crates/**/src/**/*.rs`) and Dart (`app_flutter/lib/**/*.dart`) sources.

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use verify_baseline::checks::{
    audit_dart_source_docstrings, audit_rust_source_docstrings, audit_source_purity,
    check_docstring_completeness,
};
use walkdir::WalkDir;

const MAX_MODULE_WALK_DEPTH: usize = 16;

/// Resolves a target `.rs` file or its decomposed `<stem>/` module directory into sorted `.rs` paths.
/// Realises: [REQ-CHECK-36-MODULE-RESOLVER]
fn resolve_module_rs_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }
    let dir_path = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.with_extension("")
    };
    if !dir_path.is_dir() {
        return Vec::new();
    }
    let mut collected = BTreeSet::new();
    let mut stack = vec![(dir_path, 0usize)];
    while let Some((current_dir, depth)) = stack.pop() {
        if depth > MAX_MODULE_WALK_DEPTH {
            continue;
        }
        let Ok(entries) = fs::read_dir(&current_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                let next_depth = depth.saturating_add(1);
                if next_depth <= MAX_MODULE_WALK_DEPTH {
                    stack.push((entry_path, next_depth));
                }
            } else if file_type.is_file()
                && entry_path.extension().and_then(|ext| ext.to_str()) == Some("rs")
            {
                collected.insert(entry_path);
            }
        }
    }
    collected.into_iter().collect()
}

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
    // 1. Reproduce exact Tier 2 defect from DEAP01-cuas-coa-engine:
    // app_flutter/lib/app/app.dart:10-14 where `const MyApp({super.key});` had
    // `/// Member documentation.` and `class MyApp` lacked the 7-section docstring.
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

    // 2. Test Rust source with `/// Member documentation.` and missing 7-section docstrings
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

    // 3. Verify end-to-end workspace check_docstring_completeness fails closed on temp workspace
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

#[test]
fn test_check_36_rejects_undocumented_rust_items_even_when_lib_rs_has_module_doc(
) -> Result<(), Box<dyn Error>> {
    let ws = ScopedTempWorkspace::new("no_module_exemption")?;
    let src_dir = ws.root.join("crates").join("sample-crate").join("src");
    fs::create_dir_all(&src_dir)?;

    // lib.rs has a complete 7-section `//!` module header, followed immediately by an
    // undocumented struct and function, plus a submodule `submod.rs` with undocumented items.
    let lib_rs_with_module_doc = r#"//! Sample crate module documentation.
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
"#;

    let submod_rs = r#"//! Submodule header with 7 sections that must never exempt items.
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
"#;

    fs::write(src_dir.join("lib.rs"), lib_rs_with_module_doc)?;
    fs::write(src_dir.join("submod.rs"), submod_rs)?;

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

#[test]
fn test_live_workspace_crates_have_100_percent_per_item_7_section_docstrings(
) -> Result<(), Box<dyn Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("failed to resolve repo_root from CARGO_MANIFEST_DIR")?;

    // 1. Verify docstring_gate, source_ast, docstring, and code modules (single .rs file or decomposed directory)
    // satisfy Check 34 source purity and Check 36 Rustdoc completeness.
    for rel_mod in [
        "crates/verify-baseline/src/checks/docstring_gate.rs",
        "crates/deap-core/src/source_ast.rs",
        "crates/compile-sysml/src/docstring.rs",
        "crates/ingest-sysml/src/translators/code.rs",
    ] {
        let resolved_files = resolve_module_rs_files(&repo_root.join(rel_mod));
        assert!(
            !resolved_files.is_empty(),
            "Expected {} (or decomposed module directory) to resolve to at least one .rs file",
            rel_mod
        );
        for file_path in resolved_files {
            let rel_file = file_path
                .strip_prefix(repo_root)
                .unwrap_or(&file_path)
                .display()
                .to_string();
            let mod_src = fs::read_to_string(&file_path)?;
            let purity_res = audit_source_purity(&mod_src, &rel_file, true);
            assert!(
                purity_res.is_ok(),
                "{} failed Check 34 source purity: {:?}",
                rel_file,
                purity_res.err()
            );
            let doc_res = audit_rust_source_docstrings(&mod_src, &rel_file);
            assert!(
                doc_res.is_ok(),
                "{} failed Check 36 per-item docstring gate: {:?}",
                rel_file,
                doc_res.err()
            );
        }
    }

    let runner_files =
        resolve_module_rs_files(&repo_root.join("crates/verify-baseline/src/runner.rs"));
    assert!(
        !runner_files.is_empty(),
        "Expected crates/verify-baseline/src/runner.rs (or runner/ directory) to resolve to at least one .rs file"
    );
    for runner_file in runner_files {
        let rel_runner = runner_file
            .strip_prefix(repo_root)
            .unwrap_or(&runner_file)
            .display()
            .to_string();
        let runner_src = fs::read_to_string(&runner_file)?;
        let runner_doc_res = audit_rust_source_docstrings(&runner_src, &rel_runner);
        assert!(
            runner_doc_res.is_ok(),
            "{} failed Check 36 per-item docstring gate: {:?}",
            rel_runner,
            runner_doc_res.err()
        );
    }

    // 2. Verify 100% of non-test .rs files across all 6 workspace crates pass Check 36
    let crates_dir = repo_root.join("crates");
    let mut total_files = 0usize;
    let mut total_items = 0usize;
    let mut all_violations = Vec::new();

    for entry_res in WalkDir::new(&crates_dir).into_iter().filter_entry(|e| {
        if e.file_type().is_dir() {
            let name = e.file_name().to_string_lossy();
            name != "target"
                && name != ".git"
                && name != "tests"
                && name != "benches"
                && name != "examples"
        } else {
            true
        }
    }) {
        let entry = entry_res?;
        if entry.file_type().is_file()
            && entry
                .path()
                .extension()
                .map(|ext| ext == "rs")
                .unwrap_or(false)
        {
            total_files += 1;
            let rel = entry
                .path()
                .strip_prefix(repo_root)
                .unwrap_or(entry.path())
                .display()
                .to_string();
            let content = fs::read_to_string(entry.path())?;
            match audit_rust_source_docstrings(&content, &rel) {
                Ok(count) => {
                    total_items += count;
                }
                Err(errs) => {
                    all_violations.extend(errs);
                }
            }
        }
    }

    assert!(
        total_files >= 40,
        "Expected at least 40 non-test .rs files across crates/, scanned {}",
        total_files
    );
    assert!(
        all_violations.is_empty(),
        "Live workspace crates have {} Check 36 per-item 7-section docstring violation(s) (first 25 shown):\n{}",
        all_violations.len(),
        all_violations
            .iter()
            .take(25)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        total_items >= 200,
        "Expected at least 200 documented Rust items across workspace crates, verified {}",
        total_items
    );

    let full_gate = check_docstring_completeness(repo_root);
    assert!(
        full_gate.is_ok(),
        "check_docstring_completeness failed on live workspace: {:?}",
        full_gate.err()
    );
    Ok(())
}

#[test]
fn test_check_36_catches_multiline_dart_classes_string_slashes_interpolated_quotes_and_dart3_items()
{
    let valid_dart = r#"
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
"#;

    let res = audit_dart_source_docstrings(valid_dart, "lib/services.dart");
    assert_eq!(res, Ok(8), "Should classify exactly 8 items");

    // Negative probe: missing docstring on getter
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

#[test]
fn test_check_36_nested_block_comments_and_escaped_char_literals() {
    let rust_source = "\
/* outer block comment
   /* inner nested block comment */
   /// Member documentation.
   pub fn fake_inside_nested_comment() {}
*/
/// Documented function using hex and unicode character escapes alongside lifetimes.
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
";

    assert_eq!(
        audit_rust_source_docstrings(rust_source, "crates/demo/src/lib.rs"),
        Ok(1),
        "Nested Rust block comment must remain active after inner `*/` until outer `*/`"
    );

    let dart_source = "\
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
";

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

/// Verifies that `audit_source_purity` fails closed on outer and inner clippy lint suppressions.
/// Realises: [REQ-CHECK-36-SOURCE-PURITY-NEG-PROBE]
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

/// Verifies Dart nested quotes inside `${...}` interpolations containing `//` or `/*` and multiline `@Annotation(...)` preservation.
/// Realises: [REQ-CHECK-36-DART-INTERP-QUOTES-AND-MULTILINE-ANNOTATION]
#[test]
fn test_check_36_dart_interpolated_nested_quotes_with_comment_tokens_and_multiline_annotations() {
    let dart_with_undocumented_after_http = r#"
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
"#;

    let neg_http_res = audit_dart_source_docstrings(
        dart_with_undocumented_after_http,
        "app_flutter/lib/telemetry.dart",
    );
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

    let dart_with_undocumented_after_block_open = r#"
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
"#;

    let neg_block_res = audit_dart_source_docstrings(
        dart_with_undocumented_after_block_open,
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

    let dart_with_multiline_annotation = r#"
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
  /// Formats endpoint URI and glob pattern from configuration map.
  ///
  /// ## 1. Safety Intent & Regulatory Scope
  /// Evaluates nested quotes in string interpolations safely.
  ///
  /// ## 2. Invariants & Mathematical Properties
  /// - Tokens `//` and `/*` inside nested interpolation quotes remain string data.
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
    final glob = "${map["/*"]}";
    return glob;
  }

  /// Legacy reset endpoint preserved across multiline Dart annotation.
  ///
  /// ## 1. Safety Intent & Regulatory Scope
  /// Verifies docstring retention across multiline annotations under DO-178C Level A.
  ///
  /// ## 2. Invariants & Mathematical Properties
  /// - `annotation_paren_depth` preserves `current_doc` until closing `)`.
  ///
  /// ## 3. Preconditions
  /// - Formatter instance is initialized.
  ///
  /// ## 4. Postconditions
  /// - Resets formatter state cleanly.
  ///
  /// ## 5. Algorithmic Complexity Bounds
  /// - Time Complexity: O(1).
  /// - Space Complexity: O(1).
  ///
  /// ## 6. Failure Modes & Fault Tolerance
  /// - No-op reset cannot fail.
  ///
  /// ## 7. Realization & Traceability
  /// Realises: [REQ-DART-INTERP-METHOD-03]
  @Deprecated(
    'Use formatEndpoint instead',
  )
  void legacyReset() {}
}
"#;

    assert_eq!(
        audit_dart_source_docstrings(
            dart_with_multiline_annotation,
            "app_flutter/lib/telemetry.dart"
        ),
        Ok(3),
        "Expected class + 2 documented methods (including multiline @Deprecated) to return Ok(3)"
    );
}

/// Verifies scoped `#[cfg(test)]` / `mod tests` handling and `pub(self)`, `unsafe`, and `extern` Rust item classification.
/// Realises: [REQ-CHECK-36-RUST-SCOPED-CFG-TEST-AND-QUALIFIERS]
#[test]
fn test_check_36_rust_scoped_cfg_test_and_extended_qualifiers() {
    // 1. Production items after `#[cfg(test)] mod tests;` and inline `mod tests { ... }` must still be audited.
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

    // 2. `#[cfg(test)] impl` methods must be exempted while subsequent production items are audited.
    let rust_cfg_test_impl = r#"/// Documented carrier struct for cfg(test) impl block verification.
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
"#;
    assert_eq!(
        audit_rust_source_docstrings(rust_cfg_test_impl, "crates/demo/src/lib.rs"),
        Ok(1),
        "#[cfg(test)] impl methods must be exempted while Carrier struct is counted"
    );

    // 3. Undocumented `pub(self) fn`, `pub unsafe fn`, `pub unsafe trait`, and `pub extern "C" fn` must fail closed.
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

/// Verifies that inline block comments and adjacent string/raw-string literals preserve token separation in Rust and Dart (refs #517).
/// Realises: [REQ-CHECK-36-TOKEN-SEPARATION-517]
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

/// Verifies `resolve_module_rs_files` resolves both a single `foo.rs` file and a decomposed `foo/` module directory within bounded depth.
/// Realises: [REQ-CHECK-36-MODULE-RESOLVER]
#[test]
fn test_resolve_module_rs_files_single_file_and_decomposed_directory() -> Result<(), Box<dyn Error>>
{
    let ws = ScopedTempWorkspace::new("resolve_module_rs")?;
    let src_dir = ws.root.join("src");
    fs::create_dir_all(&src_dir)?;

    // 1. Single foo.rs file resolves directly to vec![foo.rs]
    let single_foo = src_dir.join("foo.rs");
    fs::write(&single_foo, "// single module file\n")?;
    assert_eq!(
        resolve_module_rs_files(&single_foo),
        vec![single_foo.clone()],
        "Single foo.rs file must resolve to vec![foo.rs]"
    );

    // 2. Decomposed foo/{mod.rs, sub.rs, nested/leaf.rs} resolves recursively in deterministic BTreeSet order
    fs::remove_file(&single_foo)?;
    let foo_dir = src_dir.join("foo");
    let nested_dir = foo_dir.join("nested");
    fs::create_dir_all(&nested_dir)?;
    let mod_rs = foo_dir.join("mod.rs");
    let sub_rs = foo_dir.join("sub.rs");
    let leaf_rs = nested_dir.join("leaf.rs");
    fs::write(&sub_rs, "// sub\n")?;
    fs::write(&mod_rs, "// mod\n")?;
    fs::write(&leaf_rs, "// leaf\n")?;
    fs::write(foo_dir.join("notes.md"), "ignored non-rust file\n")?;

    assert_eq!(
        resolve_module_rs_files(&single_foo),
        vec![mod_rs, leaf_rs, sub_rs],
        "Decomposed foo/ directory must resolve all .rs submodules in deterministic sorted order"
    );

    // 3. Depth boundary: depth 16 is included, depth 17 is excluded
    let bar_rs = src_dir.join("bar.rs");
    let mut cur_dir = src_dir.join("bar");
    for level in 1..=16usize {
        cur_dir = cur_dir.join(format!("d{level:02}"));
    }
    fs::create_dir_all(&cur_dir)?;
    let depth_16_file = cur_dir.join("at_max_depth.rs");
    fs::write(&depth_16_file, "// depth 16\n")?;

    let depth_17_dir = cur_dir.join("d17");
    fs::create_dir_all(&depth_17_dir)?;
    let depth_17_file = depth_17_dir.join("beyond_max_depth.rs");
    fs::write(&depth_17_file, "// depth 17\n")?;

    let bar_resolved = resolve_module_rs_files(&bar_rs);
    assert_eq!(
        bar_resolved,
        vec![depth_16_file],
        "Resolver must include depth <= 16 and exclude depth 17"
    );

    // 4. Non-existent module path returns empty Vec
    assert!(
        resolve_module_rs_files(&src_dir.join("missing.rs")).is_empty(),
        "Missing module path must return empty Vec"
    );
    Ok(())
}
