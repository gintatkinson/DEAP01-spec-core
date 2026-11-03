//! Integration and unit tests for the 12-check native Rust adversarial defect dossier validator (Part B).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use verify_baseline::checks::{
    audit_rust_source_docstrings, audit_source_purity, is_compliant_rust_docblock,
};
use verify_baseline::defect_dossier::{
    build_gh_command, run_check_defect, validate_defect_dossier, verify_closed_loop_payload,
    DefectDossierArgs,
};

const MAX_MODULE_WALK_DEPTH: usize = 16;

/// Resolves a target `.rs` file or its decomposed `<stem>/` module directory into sorted `.rs` paths.
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
    let resolved_files =
        resolve_module_rs_files(&repo_root.join("crates/verify-baseline/src/defect_dossier.rs"));
    assert!(
        !resolved_files.is_empty(),
        "Expected crates/verify-baseline/src/defect_dossier.rs (or defect_dossier/ directory) to resolve to at least one .rs file"
    );

    let mut total_item_count = 0usize;
    for path in &resolved_files {
        let rel_path = path
            .strip_prefix(repo_root)
            .unwrap_or(path)
            .display()
            .to_string();
        let src = fs::read_to_string(path)?;
        let res = audit_source_purity(&src, &rel_path, true);
        assert!(
            res.is_ok(),
            "{} failed Check 34 source purity: {:?}",
            rel_path,
            res.err()
        );

        let module_doc: Vec<String> = src
            .lines()
            .map(str::trim)
            .skip_while(|line| line.is_empty())
            .take_while(|line| line.starts_with("//!"))
            .map(String::from)
            .collect();
        assert!(
            is_compliant_rust_docblock(&module_doc, true),
            "{} module header must satisfy Tier A Rustdoc contract",
            rel_path
        );

        let item_count =
            audit_rust_source_docstrings(&src, &rel_path).map_err(|errs| errs.join("\n"))?;
        total_item_count = total_item_count.saturating_add(item_count);
    }

    assert!(
        total_item_count >= 18,
        "Expected at least 18 documented structs/enums/functions across defect_dossier module files, found {}",
        total_item_count
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
        !content_a.contains("reproducer_test.py")
            && !content_a.contains("test_bridge_reproducer.py"),
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
