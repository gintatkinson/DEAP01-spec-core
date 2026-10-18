//! Integration and unit tests for the 12-check native Rust adversarial defect dossier validator,
//! dual-location `.pipeline/defects/` persistence, GitHub SSOT publishing/editing, Closed-Loop
//! Payload Verification Gate, and minimized `adversarial-code-auditor` skill parity (refs #465, #466, #467, #468).

use std::fs;
use std::path::{Path, PathBuf};
use verify_baseline::checks::audit_source_purity;
use verify_baseline::defect_dossier::{
    derive_defect_title, extract_file_basename, run_check_defect, validate_defect_dossier,
    validate_defect_title, verify_closed_loop_payload, DefectDossierArgs,
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
fn test_valid_critical_dossier_passes_all_12_checks() {
    let body = sample_valid_critical_dossier();
    let res = validate_defect_dossier(&body);
    assert!(
        res.is_ok(),
        "Expected valid critical dossier to pass all 12 checks, got: {:?}",
        res.err()
    );
    let val = res.unwrap();
    assert_eq!(val.severity, "Critical");
    assert_eq!(val.file_location, "cesium_native_bridge/src/bridge.cpp:56-61");
    assert_eq!(val.suggested_label, "bug");
    assert_eq!(
        val.test_target,
        "crates/verify-baseline/tests/defect_dossier_tests.rs"
    );
    assert!(val
        .symptom
        .contains("Dart FFI caller reads garbage or crashes"));
}

#[test]
fn test_valid_suggestion_dossier_with_na_section_4_passes() {
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
    let val = res.unwrap();
    assert_eq!(val.severity, "Suggestion");
    assert_eq!(val.suggested_label, "enhancement");
}

#[test]
fn test_rejects_leading_h1_header_in_dossier_body() {
    let body = format!(
        "# [AUDIT] [bridge.cpp]: Leading H1 header is forbidden\n\n{}",
        sample_valid_critical_dossier()
    );
    let res = validate_defect_dossier(&body);
    assert!(res.is_err(), "Leading # H1 header must be rejected");
    let errs = res.unwrap_err().join("\n");
    assert!(
        errs.contains("## 1. Context and References") || errs.contains("H1"),
        "Unexpected error messages: {}",
        errs
    );
}

#[test]
fn test_rejects_non_exact_section_headers() {
    let body = sample_valid_critical_dossier().replace(
        "## 2. Root Cause Analysis (5 Whys)",
        "## 2. Wrong Header Name",
    );
    let res = validate_defect_dossier(&body);
    assert!(
        res.is_err(),
        "Non-exact section header must fail validation"
    );
    let errs = res.unwrap_err().join("\n");
    assert!(
        errs.contains("## 2. Root Cause Analysis (5 Whys)"),
        "Expected missing exact header error, got: {}",
        errs
    );
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
    assert!(
        validate_defect_title(&derived, "cesium_native_bridge/src/bridge.cpp:56-61").is_ok()
    );
}

#[test]
fn test_rejects_ephemeral_tmp_path_when_publish_or_issue_is_set() {
    let args = DefectDossierArgs {
        body_file: Some(PathBuf::from("/tmp/gh_body_ephemeral_12345.md")),
        title: Some("[AUDIT] [bridge.cpp]: Use-after-free".to_string()),
        repo: "gintatkinson/DEAP01-spec-core".to_string(),
        label: None,
        provider: "github".to_string(),
        dry_run: true,
        publish: true,
        issue: None,
        target: None,
    };
    let res = run_check_defect(&args);
    assert!(
        res.is_err(),
        "Ephemeral /tmp/ path must be rejected when --publish is set"
    );
}

#[test]
fn test_closed_loop_payload_verification_gate() {
    let local_body = sample_valid_critical_dossier();
    let expected_title = "[AUDIT] [bridge.cpp]: Use-after-free in bridge_get_last_error";

    let matching_json = serde_json::json!({
        "title": expected_title,
        "body": local_body.replace('\n', "\r\n"),
        "url": "https://github.com/gintatkinson/DEAP01-spec-core/issues/999"
    })
    .to_string();

    assert!(
        verify_closed_loop_payload(&matching_json, expected_title, &local_body).is_ok(),
        "Matching remote payload with CRLF line endings must pass closed-loop verification"
    );

    let mismatched_title_json = serde_json::json!({
        "title": "[AUDIT] [bridge.cpp]: Truncated title",
        "body": local_body,
        "url": "https://github.com/gintatkinson/DEAP01-spec-core/issues/999"
    })
    .to_string();
    assert!(
        verify_closed_loop_payload(&mismatched_title_json, expected_title, &local_body).is_err(),
        "Mismatched remote title must fail closed-loop verification"
    );

    let mismatched_body_json = serde_json::json!({
        "title": expected_title,
        "body": "## 1. Context and References\nTruncated body",
        "url": "https://github.com/gintatkinson/DEAP01-spec-core/issues/999"
    })
    .to_string();
    assert!(
        verify_closed_loop_payload(&mismatched_body_json, expected_title, &local_body).is_err(),
        "Mismatched remote body must fail closed-loop verification"
    );
}

#[test]
fn test_defect_dossier_rs_check_34_purity_and_7_section_rustdoc() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root");
    let path = repo_root.join("crates/verify-baseline/src/defect_dossier.rs");
    let src = fs::read_to_string(&path).expect("read defect_dossier.rs");
    let res = audit_source_purity(
        &src,
        "crates/verify-baseline/src/defect_dossier.rs",
        true,
    );
    assert!(
        res.is_ok(),
        "defect_dossier.rs failed Check 34 source purity: {:?}",
        res.err()
    );

    // Also verify every struct, enum, trait, and fn in defect_dossier.rs has all 7 sections and Realises: [...]
    let s1 = "## 1. Safety Intent & Regulatory Scope";
    let s2 = "## 2. Invariants & Mathematical Properties";
    let s3 = "## 3. Preconditions";
    let s4 = "## 4. Postconditions";
    let s5 = "## 5. Algorithmic Complexity Bounds";
    let s6 = "## 6. Failure Modes & Fault Tolerance";
    let s7 = "## 7. Realization & Traceability";

    let mut current_doc = Vec::new();
    let mut item_count = 0usize;
    for (idx, raw_line) in src.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.starts_with("//!") || trimmed.starts_with("///") {
            current_doc.push(trimmed.to_string());
            continue;
        }
        let code = trimmed.split("//").next().unwrap_or(trimmed).trim();
        if code.is_empty() || code.starts_with("#[") {
            continue;
        }
        let is_item = code.starts_with("pub struct ")
            || code.starts_with("struct ")
            || code.starts_with("pub enum ")
            || code.starts_with("enum ")
            || code.starts_with("pub trait ")
            || code.starts_with("trait ")
            || code.starts_with("pub fn ")
            || code.starts_with("fn ")
            || code.starts_with("pub const fn ")
            || code.starts_with("const fn ")
            || code.starts_with("pub(crate) fn ");
        if is_item {
            item_count += 1;
            let doc_block = current_doc.join("\n");
            for sec in [s1, s2, s3, s4, s5, s6, s7] {
                assert!(
                    doc_block.contains(sec),
                    "defect_dossier.rs:{}: item '{}' missing section '{}'",
                    line_no,
                    code,
                    sec
                );
            }
            assert!(
                doc_block.contains("Realises: ["),
                "defect_dossier.rs:{}: item '{}' missing non-empty Realises: [...] tag",
                line_no,
                code
            );
            current_doc.clear();
        } else {
            current_doc.clear();
        }
    }
    assert!(
        item_count >= 18,
        "Expected at least 18 documented structs/enums/functions in defect_dossier.rs, found {}",
        item_count
    );
}

#[test]
fn test_persisted_dogfood_audit_dossiers_466_467_468_pass_12_checks() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root");
    for name in [
        "DEFECT_DOCSTRING_REGRESSION.md",
        "DEFECT_AUDITOR_466.md",
        "DEFECT_AUDITOR_467.md",
        "DEFECT_AUDITOR_468.md",
        "DEFECT_ZERO_PYTHON_REFERENCES.md",
    ] {
        let p = repo_root.join(".pipeline/defects").join(name);
        let content = fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {}", name, e));
        let res = validate_defect_dossier(&content);
        assert!(
            res.is_ok(),
            "Dogfood dossier {} failed 12-check validation: {:?}",
            name,
            res.err()
        );
    }
}

#[test]
fn test_adversarial_code_auditor_skill_minimized_and_identical() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root");
    let skill_a = repo_root.join("skills/adversarial-code-auditor/SKILL.md");
    let skill_b = repo_root.join(".agents/skills/adversarial-code-auditor/SKILL.md");

    let content_a = fs::read_to_string(&skill_a).expect("read skills/adversarial-code-auditor/SKILL.md");
    let content_b =
        fs::read_to_string(&skill_b).expect("read .agents/skills/adversarial-code-auditor/SKILL.md");

    assert_eq!(
        content_a, content_b,
        "skills/adversarial-code-auditor/SKILL.md and .agents/skills/adversarial-code-auditor/SKILL.md must be 100% identical"
    );

    assert!(
        content_a.contains("five correctness risk pillars"),
        "SKILL.md frontmatter must describe five correctness risk pillars (refs #468)"
    );
    assert!(
        !content_a.contains("reproducer_test.py") && !content_a.contains("test_bridge_reproducer.py"),
        "SKILL.md must not prescribe legacy Python .py test-target paths (refs #466)"
    );
    assert!(
        !content_a.contains("/tmp/gh_body_") && !content_a.contains("urllib.request"),
        "SKILL.md must not contain /tmp/gh_body_ shell snippets or inline Python REST scripts"
    );
    assert!(
        content_a.contains("./target/release/verify-baseline check-defect .pipeline/defects/"),
        "SKILL.md must delegate verification, persistence, and publishing to verify-baseline check-defect"
    );
}
