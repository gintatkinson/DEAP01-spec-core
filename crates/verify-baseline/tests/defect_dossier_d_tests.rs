//! Integration and unit tests for the 12-check native Rust adversarial defect dossier validator (Part D).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use verify_baseline::defect_dossier::{
    run_check_defect, validate_defect_dossier, validate_section_6_code_and_target_grounding,
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
            "deap_defect_dossier_d_{}_{}_{}",
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

/// Verifies dry-run non-persistence contract for `run_check_defect` (refs #474).
fn assert_dry_run_does_not_persist() -> Result<(), Box<dyn std::error::Error>> {
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
        body_file: Some(custom_dossier),
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
fn test_issue_474_governance_and_dry_run_contract_alignment(
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;

    let agents_md = fs::read_to_string(repo_root.join("AGENTS.md"))?;
    let dispatch_rules =
        fs::read_to_string(repo_root.join("rules/subagent-dispatch-standards.md"))?;
    let dossier_files =
        resolve_module_rs_files(&repo_root.join("crates/verify-baseline/src/defect_dossier.rs"));
    assert!(
        !dossier_files.is_empty(),
        "Expected defect_dossier.rs (or defect_dossier/ directory) to resolve to at least one .rs file"
    );
    let mut dossier_rs = String::new();
    for file_path in &dossier_files {
        dossier_rs.push_str(&fs::read_to_string(file_path)?);
        dossier_rs.push('\n');
    }

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

    assert_dry_run_does_not_persist()
}

#[test]
fn test_section_6_rust_zero_regex_rejection() {
    let base = sample_valid_critical_dossier();

    let diff_lines = vec![
        "```diff".to_string(),
        "+use regex::Regex;".to_string(),
        "+let re = Regex::new(\"^a+\").unwrap();".to_string(),
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

    for bad_line in [
        "use  regex::Regex;".to_string(),
        "extern crate regex;".to_string(),
        "let _ = regex :: Regex::new(\"a\");".to_string(),
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
fn test_resolve_module_rs_files_single_file_and_decomposed_directory(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = ScopedTempWorkspace::new("resolve_mod_rs")?;
    let src_dir = tmp.root.join("src");
    fs::create_dir_all(&src_dir)?;
    let single_file = src_dir.join("foo.rs");
    fs::write(&single_file, "// single file\n")?;
    assert_eq!(
        resolve_module_rs_files(&single_file),
        vec![single_file.clone()]
    );

    fs::remove_file(&single_file)?;
    let mod_dir = src_dir.join("foo");
    fs::create_dir_all(&mod_dir)?;
    fs::write(mod_dir.join("sub.rs"), "// submodule\n")?;
    fs::write(mod_dir.join("mod.rs"), "// mod root\n")?;
    fs::write(mod_dir.join("README.md"), "ignored\n")?;
    assert_eq!(
        resolve_module_rs_files(&single_file),
        vec![mod_dir.join("mod.rs"), mod_dir.join("sub.rs")]
    );
    Ok(())
}
