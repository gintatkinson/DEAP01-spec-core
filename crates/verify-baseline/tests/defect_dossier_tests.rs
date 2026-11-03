//! Integration and unit tests for the 12-check native Rust adversarial defect dossier validator (Part A).

use verify_baseline::defect_dossier::{
    derive_defect_title, domain_cleanliness_gate, extract_file_basename, validate_defect_dossier,
    validate_defect_title,
};

fn sample_valid_critical_dossier() -> String {
    r#"## 1. Context and References

<!-- test-target: crates/verify-baseline/tests/defect_dossier_tests.rs -->

- **File**: `cesium_native_bridge/src/bridge.cpp:56-61`
- **Pillar**: Memory Safety
- **Symptom**: Dart FFI caller reads garbage or crashes after calling bridge_get_last_error when another thread concurrently calls bridge_shutdown on the same handle.
- **Test-Target**: `crates/verify-baseline/tests/defect_dossier_tests.rs`

## 2. Root Cause Analysis (5 Whys)

1. **Why does the Dart VM crash?** Because it dereferences a pointer whose backing memory was freed.
2. **Why was the memory freed?** Because bridge_get_last_error returns c_str() and releases the mutex while a concurrent bridge_shutdown erases the BridgeState.
3. **Why return a raw pointer to internal state?** Because the API was designed for zero-copy convenience.
4. **Why is that assumption violated?** Because no ownership protocol or lifetime contract exists across the FFI boundary.
5. **Why was no contract designed?** Because the C FFI pattern chose raw C string returns without ownership semantics.

## 3. Correctness Analysis

Thread T1 calls bridge_get_last_error (line 56), acquires g_statesMutex (line 57), evaluates c_str() on line 60 producing pointer p. The lock_guard destructor at line 61 releases the mutex. Thread T2 enters bridge_shutdown (line 46), acquires the mutex (line 47), erases the map entry (line 48).

## 4. UML Diagrams

```mermaid
sequenceDiagram
    participant T1 as Thread 1 (Get Error)
    participant T2 as Thread 2 (Shutdown)
    T1->>T2: Concurrent execution
    Note over T1: Acquires mutex, extracts c_str pointer p, releases mutex
    Note over T2: Acquires mutex, erases BridgeState, deallocates string backing p
    T1-->>T2: Dereferences p (Use-After-Free)
```

## 5. Affected Callers / Downstream Impact

Dart FFI caller getLastError() -- receives dangling pointer after concurrent shutdown.

## 6. Proposed Correction

```cpp
int32_t bridge_get_last_error(bridge_handle_t handle, char* out, int32_t size) {
  if (!out || size <= 0) return BRIDGE_ERR_MEMORY;
  return BRIDGE_OK;
}
```

## 7. Relationship to Existing Issues

Discovered in audit -- new finding.

## Audit Source

Adversarial Memory Safety Audit
SEVERITY: Critical
FILE_LOCATION: cesium_native_bridge/src/bridge.cpp:56-61
"#
    .to_string()
}

#[test]
fn test_valid_critical_dossier_passes_all_12_checks() -> Result<(), Box<dyn std::error::Error>> {
    let body = sample_valid_critical_dossier();
    let res = validate_defect_dossier(&body);
    assert!(
        res.is_ok(),
        "Expected valid critical dossier to pass all 12 checks, got: {:?}",
        res.err()
    );
    let val = res.map_err(|e| e.join("; "))?;
    assert_eq!(val.severity, "Critical");
    assert_eq!(
        val.file_location,
        "cesium_native_bridge/src/bridge.cpp:56-61"
    );
    assert_eq!(val.suggested_label, "bug");
    assert_eq!(
        val.test_target,
        "crates/verify-baseline/tests/defect_dossier_tests.rs"
    );
    assert!(val
        .symptom
        .contains("Dart FFI caller reads garbage or crashes"));
    Ok(())
}

#[test]
fn test_valid_suggestion_dossier_with_na_section_4_passes() -> Result<(), Box<dyn std::error::Error>>
{
    let body = sample_valid_critical_dossier()
        .replace(
            "```mermaid\nsequenceDiagram\n    participant T1 as Thread 1 (Get Error)\n    participant T2 as Thread 2 (Shutdown)\n    T1->>T2: Concurrent execution\n    Note over T1: Acquires mutex, extracts c_str pointer p, releases mutex\n    Note over T2: Acquires mutex, erases BridgeState, deallocates string backing p\n    T1-->>T2: Dereferences p (Use-After-Free)\n```",
            "N/A -- Suggestion severity.",
        )
        .replace("SEVERITY: Critical", "SEVERITY: Suggestion");
    let res = validate_defect_dossier(&body);
    assert!(
        res.is_ok(),
        "Expected Suggestion dossier with N/A -- section 4 to pass, got: {:?}",
        res.err()
    );
    let val = res.map_err(|e| e.join("; "))?;
    assert_eq!(val.severity, "Suggestion");
    assert_eq!(val.suggested_label, "enhancement");
    Ok(())
}

#[test]
fn test_rejects_leading_h1_header_in_dossier_body() -> Result<(), Box<dyn std::error::Error>> {
    let body = format!(
        "# [AUDIT] [bridge.cpp]: Leading H1 header is forbidden\n\n{}",
        sample_valid_critical_dossier()
    );
    let res = validate_defect_dossier(&body);
    assert!(res.is_err(), "Leading # H1 header must be rejected");
    let errs = res.err().ok_or("expected Err")?.join("\n");
    assert!(
        errs.contains("## 1. Context and References") || errs.contains("H1"),
        "Unexpected error messages: {}",
        errs
    );
    Ok(())
}

#[test]
fn test_rejects_non_exact_section_headers() -> Result<(), Box<dyn std::error::Error>> {
    let body = sample_valid_critical_dossier().replace(
        "## 2. Root Cause Analysis (5 Whys)",
        "## 2. Wrong Header Name",
    );
    let res = validate_defect_dossier(&body);
    assert!(
        res.is_err(),
        "Non-exact section header must fail validation"
    );
    let errs = res.err().ok_or("expected Err")?.join("\n");
    assert!(
        errs.contains("## 2. Root Cause Analysis (5 Whys)"),
        "Expected missing exact header error, got: {}",
        errs
    );
    Ok(())
}

#[test]
fn test_rejects_missing_or_mismatched_test_target_and_section_1_bullets() {
    let missing_comment = sample_valid_critical_dossier().replace(
        "<!-- test-target: crates/verify-baseline/tests/defect_dossier_tests.rs -->\n",
        "",
    );
    let res1 = validate_defect_dossier(&missing_comment);
    assert!(
        res1.is_err(),
        "Missing <!-- test-target: ... --> comment must fail"
    );

    let mismatched = sample_valid_critical_dossier().replace(
        "- **Test-Target**: `crates/verify-baseline/tests/defect_dossier_tests.rs`",
        "- **Test-Target**: `crates/verify-baseline/tests/other_test.rs`",
    );
    let res2 = validate_defect_dossier(&mismatched);
    assert!(
        res2.is_err(),
        "Mismatched HTML comment test-target and - **Test-Target**: bullet must fail"
    );

    let missing_symptom = sample_valid_critical_dossier().replace("- **Symptom**:", "- **Issue**:");
    let res3 = validate_defect_dossier(&missing_symptom);
    assert!(res3.is_err(), "Missing - **Symptom**: bullet must fail");
}

#[test]
fn test_rejects_incomplete_or_malformed_5_whys() {
    let missing_why_5 = sample_valid_critical_dossier().replace(
        "5. **Why was no contract designed?** Because the C FFI pattern chose raw C string returns without ownership semantics.\n",
        "",
    );
    let res1 = validate_defect_dossier(&missing_why_5);
    assert!(res1.is_err(), "Dossier with only 4 Whys must fail");

    let malformed_why_3 = sample_valid_critical_dossier().replace(
        "3. **Why return a raw pointer to internal state?** Because the API was designed for zero-copy convenience.",
        "3. **Why return a raw pointer to internal state?** The API lacked a Because clause.",
    );
    let res2 = validate_defect_dossier(&malformed_why_3);
    assert!(
        res2.is_err(),
        "Why line missing '?** Because ' pattern must fail"
    );
}

#[test]
fn test_rejects_invalid_mermaid_or_missing_diagram_for_critical() {
    let no_mermaid = sample_valid_critical_dossier().replace(
        "```mermaid\nsequenceDiagram\n    participant T1 as Thread 1 (Get Error)\n    participant T2 as Thread 2 (Shutdown)\n    T1->>T2: Concurrent execution\n    Note over T1: Acquires mutex, extracts c_str pointer p, releases mutex\n    Note over T2: Acquires mutex, erases BridgeState, deallocates string backing p\n    T1-->>T2: Dereferences p (Use-After-Free)\n```",
        "N/A -- Critical severity.",
    );
    assert!(
        validate_defect_dossier(&no_mermaid).is_err(),
        "Critical finding without Mermaid diagram must fail"
    );

    let semicolon_in_note = sample_valid_critical_dossier().replace(
        "Note over T1: Acquires mutex, extracts c_str pointer p, releases mutex",
        "Note over T1: Acquires mutex; extracts c_str pointer p; releases mutex",
    );
    let res_semi = validate_defect_dossier(&semicolon_in_note);
    assert!(
        res_semi.is_err(),
        "Semicolon in Mermaid Note must fail Check 7 via deap_core::markdown::check_mermaid_syntax"
    );
}

#[test]
fn test_rejects_untagged_section_6_fence_ascii_art_and_em_dash() {
    let untagged_sec6 = sample_valid_critical_dossier().replace("```cpp\n", "```\n");
    assert!(
        validate_defect_dossier(&untagged_sec6).is_err(),
        "Section 6 code fence without language tag must fail"
    );

    let ascii_art_arrow = sample_valid_critical_dossier().replace(
        "Thread T1 calls bridge_get_last_error",
        "Thread T1 ->> Thread T2 calls bridge_get_last_error",
    );
    assert!(
        validate_defect_dossier(&ascii_art_arrow).is_err(),
        "Unescaped '->>' outside Mermaid blocks must fail"
    );

    let unicode_arrow = sample_valid_critical_dossier().replace(
        "Thread T1 calls bridge_get_last_error",
        "Thread T1 \u{2192} Thread T2 calls bridge_get_last_error",
    );
    assert!(
        validate_defect_dossier(&unicode_arrow).is_err(),
        "Unescaped '→' outside Mermaid blocks must fail"
    );

    let em_dash = sample_valid_critical_dossier().replace(
        "Discovered in audit -- new finding.",
        "Discovered in audit \u{2014} new finding.",
    );
    assert!(
        validate_defect_dossier(&em_dash).is_err(),
        "Unicode em dash (\\u2014) in dossier must fail"
    );
}

#[test]
fn test_validate_and_derive_defect_title() {
    assert_eq!(
        extract_file_basename("cesium_native_bridge/src/bridge.cpp:56-61"),
        "bridge.cpp"
    );
    assert!(validate_defect_title(
        "[AUDIT] [bridge.cpp]: Use-after-free in bridge_get_last_error",
        "cesium_native_bridge/src/bridge.cpp:56-61"
    )
    .is_ok());
    assert!(validate_defect_title(
        "[AUDIT] [wrong_file.cpp]: Use-after-free in bridge_get_last_error",
        "cesium_native_bridge/src/bridge.cpp:56-61"
    )
    .is_err());
    assert!(validate_defect_title(
        "Missing audit prefix",
        "cesium_native_bridge/src/bridge.cpp:56-61"
    )
    .is_err());

    let derived = derive_defect_title(
        "cesium_native_bridge/src/bridge.cpp:56-61",
        "Use-after-free when calling bridge_get_last_error concurrently.",
    );
    assert_eq!(
        derived,
        "[AUDIT] [bridge.cpp]: Use-after-free when calling bridge_get_last_error concurrently."
    );
    assert!(validate_defect_title(&derived, "cesium_native_bridge/src/bridge.cpp:56-61").is_ok());
}

#[test]
fn test_domain_cleanliness_gate() {
    let dirty_sec6 = vec![
        "Update `crates/cuas-coa-engine/src/threat_eval.rs` as follows:".to_string(),
        "```rust".to_string(),
        "// crates/cuas-coa-engine/src/threat_eval.rs".to_string(),
        "pub fn evaluate() {}".to_string(),
        "```".to_string(),
    ];
    let mut errs_dirty = Vec::new();
    domain_cleanliness_gate(&dirty_sec6, true, &mut errs_dirty);
    assert!(
        !errs_dirty.is_empty(),
        "domain_cleanliness_gate must reject downstream crates/cuas-coa-engine/... in upstream workspace"
    );

    let mut errs_downstream = Vec::new();
    domain_cleanliness_gate(&dirty_sec6, false, &mut errs_downstream);
    assert!(
        errs_downstream.is_empty(),
        "domain_cleanliness_gate must allow downstream crate paths when is_upstream is false, got: {:?}",
        errs_downstream
    );

    let clean_sec6 = vec![
        "Update `crates/verify-baseline/src/defect_dossier.rs`:".to_string(),
        "```rust".to_string(),
        "pub fn check_clean() -> bool { true }".to_string(),
        "```".to_string(),
    ];
    let mut errs_clean = Vec::new();
    domain_cleanliness_gate(&clean_sec6, true, &mut errs_clean);
    assert!(
        errs_clean.is_empty(),
        "domain_cleanliness_gate must accept upstream crates/verify-baseline/... path, got: {:?}",
        errs_clean
    );

    let dirty_dossier = sample_valid_critical_dossier().replace(
        "```cpp\nint32_t bridge_get_last_error(bridge_handle_t handle, char* out, int32_t size) {\n  if (!out || size <= 0) return BRIDGE_ERR_MEMORY;\n  return BRIDGE_OK;\n}\n```",
        &dirty_sec6.join("\n"),
    );
    assert!(
        validate_defect_dossier(&dirty_dossier).is_err(),
        "validate_defect_dossier must reject downstream crate reference in Section 6"
    );
}
