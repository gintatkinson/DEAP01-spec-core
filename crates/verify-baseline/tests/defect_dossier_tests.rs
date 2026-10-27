//! Integration and unit tests for the 12-check native Rust adversarial defect dossier validator,
//! dual-location `.pipeline/defects/` persistence, GitHub SSOT publishing/editing, Closed-Loop
//! Payload Verification Gate, and minimized `adversarial-code-auditor` skill parity (refs #465, #466, #467, #468).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use verify_baseline::checks::audit_source_purity;
use verify_baseline::defect_dossier::{
    build_gh_command, derive_defect_title, domain_cleanliness_gate, extract_file_basename,
    parse_github_repo_slug, resolve_target_repo, run_check_defect, validate_defect_dossier,
    validate_defect_title, validate_section_6_code_and_target_grounding,
    verify_closed_loop_payload, DefectDossierArgs,
};

struct ScopedTempWorkspace {
    root: PathBuf,
}

impl ScopedTempWorkspace {
    fn new(prefix: &str) -> Result<Self, std::io::Error> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!(
            "deap_defect_dossier_{}_{}_{}",
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
    assert_eq!(val.file_location, "cesium_native_bridge/src/bridge.cpp:56-61");
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
fn test_valid_suggestion_dossier_with_na_section_4_passes() -> Result<(), Box<dyn std::error::Error>> {
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
    assert!(
        validate_defect_title(&derived, "cesium_native_bridge/src/bridge.cpp:56-61").is_ok()
    );
}

#[test]
fn test_rejects_ephemeral_tmp_path_when_publish_or_issue_is_set(
) -> Result<(), Box<dyn std::error::Error>> {
    let args = DefectDossierArgs {
        body_file: Some(PathBuf::from("/tmp/gh_body_ephemeral_12345.md")),
        title: Some("[AUDIT] [bridge.cpp]: Use-after-free".to_string()),
        repo: Some("gintatkinson/DEAP01-spec-core".to_string()),
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

    let default_pub_args = DefectDossierArgs {
        body_file: Some(PathBuf::from("/tmp/gh_body_ephemeral_12345.md")),
        title: Some("[AUDIT] [bridge.cpp]: Use-after-free".to_string()),
        repo: Some("gintatkinson/DEAP01-spec-core".to_string()),
        label: None,
        provider: "github".to_string(),
        dry_run: false,
        publish: false,
        issue: None,
        target: None,
    };
    assert!(
        run_check_defect(&default_pub_args).is_err(),
        "Ephemeral /tmp/ path must be rejected by default when dry_run is false even if publish is false"
    );

    let ephemeral_dry_run_file = PathBuf::from("/tmp/gh_body_ephemeral_dry_run_480.md");
    fs::write(&ephemeral_dry_run_file, sample_valid_critical_dossier())?;
    let dry_run_args = DefectDossierArgs {
        body_file: Some(ephemeral_dry_run_file.clone()),
        title: Some("[AUDIT] [bridge.cpp]: Use-after-free".to_string()),
        repo: Some("gintatkinson/DEAP01-spec-core".to_string()),
        label: None,
        provider: "github".to_string(),
        dry_run: true,
        publish: false,
        issue: None,
        target: None,
    };
    let res_dry_run = run_check_defect(&dry_run_args);
    let _ = fs::remove_file(&ephemeral_dry_run_file);
    assert!(
        res_dry_run.is_err(),
        "Ephemeral /tmp/ path outside .pipeline/defects/ must be rejected even when --dry-run is set"
    );
    Ok(())
}

#[test]
fn test_build_gh_command_injects_ssl_cert_file_for_macos_sandbox() {
    let cmd = build_gh_command();
    assert_eq!(cmd.get_program(), "gh");
    if std::env::var_os("SSL_CERT_FILE").is_none() && Path::new("/etc/ssl/cert.pem").is_file() {
        let has_ssl_cert_file = cmd.get_envs().any(|(k, v)| {
            k == "SSL_CERT_FILE" && v == Some(std::ffi::OsStr::new("/etc/ssl/cert.pem"))
        });
        assert!(
            has_ssl_cert_file,
            "build_gh_command() must inject SSL_CERT_FILE=/etc/ssl/cert.pem when unset and /etc/ssl/cert.pem exists"
        );
    }
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
fn test_defect_dossier_rs_check_34_purity_and_7_section_rustdoc(
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    let path = repo_root.join("crates/verify-baseline/src/defect_dossier.rs");
    let src = fs::read_to_string(&path)?;
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
    Ok(())
}

#[test]
fn test_persisted_dogfood_audit_dossiers_466_467_468_pass_12_checks(
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    for name in [
        "DEFECT_DOCSTRING_REGRESSION.md",
        "DEFECT_AUDITOR_466.md",
        "DEFECT_AUDITOR_467.md",
        "DEFECT_AUDITOR_468.md",
        "DEFECT_ZERO_PYTHON_REFERENCES.md",
        "DEFECT_RULE_20_INIT_LOOP_AND_WP_CONTAMINATION.md",
    ] {
        let p = repo_root.join(".pipeline/defects").join(name);
        let content = fs::read_to_string(&p)?;
        let res = validate_defect_dossier(&content);
        assert!(
            res.is_ok(),
            "Dogfood dossier {} failed 12-check validation: {:?}",
            name,
            res.err()
        );
    }
    Ok(())
}

#[test]
fn test_adversarial_code_auditor_skill_minimized_and_identical(
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    let skill_a = repo_root.join("skills/adversarial-code-auditor/SKILL.md");
    let skill_b = repo_root.join(".agents/skills/adversarial-code-auditor/SKILL.md");

    let content_a = fs::read_to_string(&skill_a)?;
    let content_b = fs::read_to_string(&skill_b)?;

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
    assert!(
        !content_a.contains("BypassSandbox: true")
            && content_a.contains("standard sandbox")
            && content_a.contains("[CLOSED-LOOP VERIFIED]")
            && content_a.contains("--dry-run"),
        "SKILL.md must forbid BypassSandbox: true, require standard sandbox, default publishing unless --dry-run, and hard failure unless [CLOSED-LOOP VERIFIED] is emitted"
    );
    assert!(
        !content_a.contains('\u{2014}'),
        "SKILL.md must contain zero Unicode em dashes"
    );
    Ok(())
}

#[test]
fn test_parse_github_repo_slug_ssh_https_and_invalid() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        parse_github_repo_slug("git@github.com:gintatkinson/DEAP01-cuas-coa-engine.git")?,
        "gintatkinson/DEAP01-cuas-coa-engine"
    );
    assert_eq!(
        parse_github_repo_slug("git@github.com:gintatkinson/DEAP01-spec-core")?,
        "gintatkinson/DEAP01-spec-core"
    );
    assert_eq!(
        parse_github_repo_slug("ssh://git@github.com/gintatkinson/DEAP01-cuas-coa-engine.git")?,
        "gintatkinson/DEAP01-cuas-coa-engine"
    );
    assert_eq!(
        parse_github_repo_slug("https://github.com/gintatkinson/DEAP01-cuas-coa-engine.git")?,
        "gintatkinson/DEAP01-cuas-coa-engine"
    );
    assert_eq!(
        parse_github_repo_slug("https://github.com/gintatkinson/DEAP01-cuas-coa-engine")?,
        "gintatkinson/DEAP01-cuas-coa-engine"
    );
    assert_eq!(
        parse_github_repo_slug("http://github.com/gintatkinson/DEAP01-cuas-coa-engine.git")?,
        "gintatkinson/DEAP01-cuas-coa-engine"
    );
    assert_eq!(
        parse_github_repo_slug("gintatkinson/DEAP01-cuas-coa-engine")?,
        "gintatkinson/DEAP01-cuas-coa-engine"
    );

    assert!(parse_github_repo_slug("").is_err());
    assert!(parse_github_repo_slug("https://github.com/only-owner").is_err());
    assert!(parse_github_repo_slug("not-a-valid-slug").is_err());
    assert!(parse_github_repo_slug("owner/repo/extra").is_err());
    Ok(())
}

#[test]
fn test_resolve_target_repo_auto_detects_origin_and_fails_without_remote(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempWorkspace::new("resolve_repo")?;
    let repo_dir = &tmp.root;

    // Without git init / origin remote, omitting --repo must return a hard Err
    let err_no_git = resolve_target_repo(None, repo_dir);
    assert!(
        err_no_git.is_err(),
        "Expected resolve_target_repo(None) in non-git dir to fail"
    );

    // Initialize git repo without origin remote -> must still fail
    let status_init = Command::new("git")
        .args(["init", "-q"])
        .current_dir(repo_dir)
        .status()?;
    assert!(status_init.success());

    let err_no_origin = resolve_target_repo(None, repo_dir);
    assert!(
        err_no_origin.is_err(),
        "Expected resolve_target_repo(None) without origin remote to fail"
    );

    // Configure downstream origin remote -> must auto-resolve to downstream repo, NOT DEAP01-spec-core
    let status_remote = Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            "git@github.com:gintatkinson/DEAP01-cuas-coa-engine.git",
        ])
        .current_dir(repo_dir)
        .status()?;
    assert!(status_remote.success());

    let resolved = resolve_target_repo(None, repo_dir)?;
    assert_eq!(resolved, "gintatkinson/DEAP01-cuas-coa-engine");

    // Explicit --repo overrides git remote
    let explicit = resolve_target_repo(Some("gintatkinson/DEAP01-spec-core"), repo_dir)?;
    assert_eq!(explicit, "gintatkinson/DEAP01-spec-core");
    Ok(())
}

#[test]
fn test_run_check_defect_publishes_by_default_without_publish_flag_and_fails_closed(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempWorkspace::new("default_publish")?;
    let defects_dir = tmp.root.join(".pipeline").join("defects");
    fs::create_dir_all(&defects_dir)?;
    let dossier_path = defects_dir.join("DEFECT_TEST_DEFAULT_PUBLISH.md");
    fs::write(&dossier_path, sample_valid_critical_dossier())?;

    // Case 1: --repo is None and directory has no git origin remote -> must fail closed with Err(1)
    let args_no_remote = DefectDossierArgs {
        body_file: Some(dossier_path.clone()),
        title: None,
        repo: None,
        label: None,
        provider: "github".to_string(),
        dry_run: false,
        publish: false,
        issue: None,
        target: None,
    };
    assert_eq!(
        run_check_defect(&args_no_remote),
        Err(1),
        "run_check_defect without --dry-run and without --repo in a non-git directory must fail with Err(1)"
    );

    // Case 2: --dry-run is false and --publish is false with an unreachable/nonexistent repo ->
    // must attempt publication by default (even though --publish was not passed) and fail closed with Err(1)
    let args_default_pub_fail = DefectDossierArgs {
        body_file: Some(dossier_path),
        title: None,
        repo: Some("invalid-nonexistent-owner-99999/invalid-nonexistent-repo-99999".to_string()),
        label: None,
        provider: "github".to_string(),
        dry_run: false,
        publish: false,
        issue: None,
        target: None,
    };
    assert_eq!(
        run_check_defect(&args_default_pub_fail),
        Err(1),
        "run_check_defect with publish: false and dry_run: false must attempt GitHub publication by default and return Err(1) on failure"
    );
    Ok(())
}

#[test]
fn test_rejects_regex_in_section_6_rust_blocks() -> Result<(), Box<dyn std::error::Error>> {
    let base = sample_valid_critical_dossier();
    let pat_regex_colons = ["reg", "ex::"].concat();
    let pat_regex_new = ["Reg", "ex::new"].concat();
    let pat_regex_set = ["Reg", "exSet"].concat();
    let pat_use_regex = ["use", " ", "reg", "ex"].concat();
    let ident_re = ["Reg", "ex"].concat();

    // 1. Rejection of regex colons in ```rust block
    let snippet_1 = format!(
        "```rust\npub fn parse_text(input: &str) -> bool {{\n    let re = {}{}::new(r\"^[a-z]+$\").unwrap();\n    re.is_match(input)\n}}\n```",
        pat_regex_colons, ident_re
    );
    let regex_path = base.replace(
        "```cpp\nint32_t bridge_get_last_error(bridge_handle_t handle, char* out, int32_t size) {\n  if (!out || size <= 0) return BRIDGE_ERR_MEMORY;\n  return BRIDGE_OK;\n}\n```",
        &snippet_1,
    );
    let res1 = validate_defect_dossier(&regex_path);
    assert!(res1.is_err(), "Must reject regex colons in Section 6 rust block");
    let errs1 = res1.err().ok_or("expected Err")?.join("\n");
    assert!(
        errs1.contains("Check 34") || errs1.contains("regex"),
        "Expected Check 34 / regex violation, got: {}",
        errs1
    );

    // 2. Rejection of constructor
    let snippet_2 = format!(
        "```rust\npub fn parse_text(input: &str) -> bool {{\n    let re = {}(\"^abc\").unwrap();\n    true\n}}\n```",
        pat_regex_new
    );
    let regex_new = base.replace(
        "```cpp\nint32_t bridge_get_last_error(bridge_handle_t handle, char* out, int32_t size) {\n  if (!out || size <= 0) return BRIDGE_ERR_MEMORY;\n  return BRIDGE_OK;\n}\n```",
        &snippet_2,
    );
    let res2 = validate_defect_dossier(&regex_new);
    assert!(res2.is_err(), "Must reject constructor in Section 6 rust block");

    // 3. Rejection of RegexSet
    let snippet_3 = format!(
        "```rust\npub fn parse_set() {{\n    let set = {}::new(&[\"a\", \"b\"]).unwrap();\n}}\n```",
        pat_regex_set
    );
    let regex_set = base.replace(
        "```cpp\nint32_t bridge_get_last_error(bridge_handle_t handle, char* out, int32_t size) {\n  if (!out || size <= 0) return BRIDGE_ERR_MEMORY;\n  return BRIDGE_OK;\n}\n```",
        &snippet_3,
    );
    let res3 = validate_defect_dossier(&regex_set);
    assert!(res3.is_err(), "Must reject RegexSet in Section 6 rust block");

    // 4. Rejection of use regex
    let snippet_4 = format!(
        "```rust\n{}::{};\npub fn helper() {{}}\n```",
        pat_use_regex, ident_re
    );
    let use_regex = base.replace(
        "```cpp\nint32_t bridge_get_last_error(bridge_handle_t handle, char* out, int32_t size) {\n  if (!out || size <= 0) return BRIDGE_ERR_MEMORY;\n  return BRIDGE_OK;\n}\n```",
        &snippet_4,
    );
    let res4 = validate_defect_dossier(&use_regex);
    assert!(res4.is_err(), "Must reject use regex in Section 6 rust block");

    // 5. Valid Rust code without regex passes
    let valid_rust = base.replace(
        "```cpp\nint32_t bridge_get_last_error(bridge_handle_t handle, char* out, int32_t size) {\n  if (!out || size <= 0) return BRIDGE_ERR_MEMORY;\n  return BRIDGE_OK;\n}\n```",
        "```rust\npub fn parse_text_deterministic(input: &str) -> bool {\n    input.starts_with(\"abc\")\n}\n```",
    );
    let res5 = validate_defect_dossier(&valid_rust);
    assert!(res5.is_ok(), "Valid zero-regex Rust block must pass");
    Ok(())
}

#[test]
fn test_rejects_downstream_crate_paths_in_upstream_workspace(
) -> Result<(), Box<dyn std::error::Error>> {
    let base = sample_valid_critical_dossier();

    // 1. Downstream crate cuas-coa-engine must be rejected in UPSTREAM_SPEC_CORE_COMPILER
    let downstream = base
        .replace(
            "<!-- test-target: crates/verify-baseline/tests/defect_dossier_tests.rs -->",
            "<!-- test-target: crates/cuas-coa-engine/src/lib.rs -->",
        )
        .replace(
            "- **Test-Target**: `crates/verify-baseline/tests/defect_dossier_tests.rs`",
            "- **Test-Target**: `crates/cuas-coa-engine/src/lib.rs`",
        );
    let res_downstream = validate_defect_dossier(&downstream);
    assert!(
        res_downstream.is_err(),
        "Must reject downstream crates/cuas-coa-engine/ path in UPSTREAM_SPEC_CORE_COMPILER"
    );
    let errs = res_downstream.err().ok_or("expected Err")?.join("\n");
    assert!(
        errs.contains("upstream") || errs.contains("grounding"),
        "Expected upstream crate grounding error, got: {}",
        errs
    );

    // 2. All 6 canonical upstream crates must pass
    for crate_name in [
        "deap-core",
        "compile-sysml",
        "ingest-sysml",
        "verify-baseline",
        "assemble-conops",
        "reconcile-backlog",
    ] {
        let valid_upstream = base
            .replace(
                "crates/verify-baseline/tests/defect_dossier_tests.rs",
                &format!("crates/{}/tests/test_foo.rs", crate_name),
            );
        let res = validate_defect_dossier(&valid_upstream);
        assert!(
            res.is_ok(),
            "Expected valid upstream crate '{}' to pass, got: {:?}",
            crate_name,
            res.err()
        );
    }
    Ok(())
}

#[test]
fn test_issue_474_governance_and_dry_run_contract_alignment(
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;

    let agents_md = fs::read_to_string(repo_root.join("AGENTS.md"))?;
    let dispatch_rules =
        fs::read_to_string(repo_root.join("rules/subagent-dispatch-standards.md"))?;
    let dossier_rs =
        fs::read_to_string(repo_root.join("crates/verify-baseline/src/defect_dossier.rs"))?;

    for (name, content) in [
        ("AGENTS.md", &agents_md),
        ("rules/subagent-dispatch-standards.md", &dispatch_rules),
    ] {
        assert!(
            !content.contains("validate it offline via `./target/release/verify-baseline check-defect` before submitting"),
            "{} must not describe check-defect as an offline-only step before separate submission (refs #474)",
            name
        );
        assert!(
            !content.contains("./target/release/file-defect"),
            "{} must not reference retired ./target/release/file-defect binary (refs #474)",
            name
        );
        assert!(
            content.contains("./target/release/verify-baseline check-defect .pipeline/defects/"),
            "{} must mandate atomic validation, persistence, and publishing via verify-baseline check-defect (refs #474)",
            name
        );
    }

    assert!(
        !dossier_rs.contains("if !args.dry_run || args.publish || args.issue.is_some()"),
        "defect_dossier.rs must gate persistence strictly on `if !args.dry_run` (refs #474)"
    );
    assert!(
        !dossier_rs.contains("When `--publish` or `--issue` is active without `--dry-run`"),
        "defect_dossier.rs module Rustdoc must state publishing is active whenever `--dry-run` is not passed (refs #474)"
    );

    let tmp = ScopedTempWorkspace::new("dry_run_no_persist_474")?;
    let custom_dossier = tmp
        .root
        .join("DEFECT_DRY_RUN_NO_PERSIST_UNIQUE_474_TEST.md");
    fs::write(&custom_dossier, sample_valid_critical_dossier())?;
    let leaked_dest = PathBuf::from(".pipeline")
        .join("defects")
        .join("DEFECT_DRY_RUN_NO_PERSIST_UNIQUE_474_TEST.md");
    let _ = fs::remove_file(&leaked_dest);

    let dry_run_with_publish = DefectDossierArgs {
        body_file: Some(custom_dossier.clone()),
        title: Some("[AUDIT] [bridge.cpp]: Dry run must not persist".to_string()),
        repo: Some("gintatkinson/DEAP01-spec-core".to_string()),
        label: None,
        provider: "github".to_string(),
        dry_run: true,
        publish: true,
        issue: Some(474),
        target: None,
    };
    let res = run_check_defect(&dry_run_with_publish);
    let leaked_exists = leaked_dest.exists();
    let _ = fs::remove_file(&leaked_dest);
    assert_eq!(res, Ok(()), "Expected dry_run: true to succeed");
    assert!(
        !leaked_exists,
        "run_check_defect with dry_run: true must NOT persist into .pipeline/defects/ even when publish: true or issue: Some(...) is set"
    );
    Ok(())
}

#[test]
fn test_section_6_rust_zero_regex_rejection() {
    let base = sample_valid_critical_dossier();
    let kw_re = ["reg", "ex"].concat();
    let kw_cap_re = ["Reg", "ex"].concat();

    // 1. Diff fence with added regex import/constructor must be rejected
    let diff_lines = vec![
        "```diff".to_string(),
        format!("+use {}::{};", kw_re, kw_cap_re),
        format!("+let re = {}::new(\"^a+\").unwrap();", kw_cap_re),
        "```".to_string(),
    ];
    let mut errs_diff = Vec::new();
    validate_section_6_code_and_target_grounding(
        &diff_lines,
        "crates/verify-baseline/tests/defect_dossier_tests.rs",
        true,
        &mut errs_diff,
    );
    assert!(
        !errs_diff.is_empty(),
        "Expected Section 6 diff block containing regex usage to be rejected"
    );

    // 2. Whitespace-separated `use  regex`, `extern crate regex`, and `regex ::` must be rejected
    for bad_line in [
        format!("use  {}::{};", kw_re, kw_cap_re),
        format!("extern crate {};", kw_re),
        format!("let _ = {} :: {}::new(\"a\");", kw_re, kw_cap_re),
    ] {
        let sec6 = vec!["```rust".to_string(), bad_line.clone(), "```".to_string()];
        let mut errs = Vec::new();
        validate_section_6_code_and_target_grounding(
            &sec6,
            "crates/verify-baseline/tests/defect_dossier_tests.rs",
            true,
            &mut errs,
        );
        assert!(
            !errs.is_empty(),
            "Expected Section 6 rust block with '{}' to be rejected",
            bad_line
        );
    }

    // 3. Full dossier validation with ```diff regex block must fail
    let diff_dossier = base.replace(
        "```cpp\nint32_t bridge_get_last_error(bridge_handle_t handle, char* out, int32_t size) {\n  if (!out || size <= 0) return BRIDGE_ERR_MEMORY;\n  return BRIDGE_OK;\n}\n```",
        &diff_lines.join("\n"),
    );
    assert!(
        validate_defect_dossier(&diff_dossier).is_err(),
        "Full dossier with regex in Section 6 diff fence must be rejected"
    );
}

#[test]
fn test_upstream_test_target_crate_grounding() {
    let valid_sec6 = vec![
        "```rust".to_string(),
        "pub fn clean_helper(s: &str) -> bool { s.starts_with(\"ok\") }".to_string(),
        "```".to_string(),
    ];

    // 1. Bare crate root without file subpath (`crates/verify-baseline` or `crates/verify-baseline/`) must be rejected
    for invalid_target in [
        "",
        "crates/verify-baseline",
        "crates/verify-baseline/",
        "crates/cuas-coa-engine/src/lib.rs",
        "tests/standalone_test.rs",
    ] {
        let mut errs = Vec::new();
        validate_section_6_code_and_target_grounding(&valid_sec6, invalid_target, true, &mut errs);
        assert!(
            !errs.is_empty(),
            "Expected invalid upstream test_target '{}' to be rejected",
            invalid_target
        );
    }

    // 2. Valid upstream crate with concrete file subpath passes
    let mut errs_ok = Vec::new();
    validate_section_6_code_and_target_grounding(
        &valid_sec6,
        "crates/verify-baseline/tests/defect_dossier_tests.rs",
        true,
        &mut errs_ok,
    );
    assert!(
        errs_ok.is_empty(),
        "Expected valid upstream test_target to pass, got: {:?}",
        errs_ok
    );

    // 3. Full dossier with bare `crates/verify-baseline/` in Section 1 must fail
    let bare_crate_dossier = sample_valid_critical_dossier().replace(
        "crates/verify-baseline/tests/defect_dossier_tests.rs",
        "crates/verify-baseline/",
    );
    assert!(
        validate_defect_dossier(&bare_crate_dossier).is_err(),
        "Dossier with bare crates/verify-baseline/ test-target must be rejected"
    );
}

#[test]
fn test_domain_cleanliness_gate() {
    // 1. Downstream crate path in Section 6 must be rejected when is_upstream is true
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

    // 2. Downstream crate path in Section 6 is permitted when is_upstream is false
    let mut errs_downstream = Vec::new();
    domain_cleanliness_gate(&dirty_sec6, false, &mut errs_downstream);
    assert!(
        errs_downstream.is_empty(),
        "domain_cleanliness_gate must allow downstream crate paths when is_upstream is false, got: {:?}",
        errs_downstream
    );

    // 3. Clean upstream crate path in Section 6 passes when is_upstream is true
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

    // 4. Full dossier with downstream crate reference in Section 6 must fail validate_defect_dossier
    let dirty_dossier = sample_valid_critical_dossier().replace(
        "```cpp\nint32_t bridge_get_last_error(bridge_handle_t handle, char* out, int32_t size) {\n  if (!out || size <= 0) return BRIDGE_ERR_MEMORY;\n  return BRIDGE_OK;\n}\n```",
        &dirty_sec6.join("\n"),
    );
    assert!(
        validate_defect_dossier(&dirty_dossier).is_err(),
        "validate_defect_dossier must reject downstream crate reference in Section 6"
    );
}


