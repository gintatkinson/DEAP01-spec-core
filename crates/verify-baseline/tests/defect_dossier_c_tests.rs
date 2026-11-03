//! Integration and unit tests for the 12-check native Rust adversarial defect dossier validator (Part C).

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use verify_baseline::defect_dossier::{
    parse_github_repo_slug, resolve_target_repo, run_check_defect, validate_defect_dossier,
    DefectDossierArgs,
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
            "deap_defect_dossier_c_{}_{}_{}",
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

    let err_no_git = resolve_target_repo(None, repo_dir);
    assert!(
        err_no_git.is_err(),
        "Expected resolve_target_repo(None) in non-git dir to fail"
    );

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
    let cpp_block = "```cpp\nint32_t bridge_get_last_error(bridge_handle_t handle, char* out, int32_t size) {\n  if (!out || size <= 0) return BRIDGE_ERR_MEMORY;\n  return BRIDGE_OK;\n}\n```";

    let snippet_1 = "```rust\npub fn parse_text(input: &str) -> bool {\n    let re = regex::Regex::new(r\"^[a-z]+$\").unwrap();\n    re.is_match(input)\n}\n```";
    let regex_path = base.replace(cpp_block, snippet_1);
    let res1 = validate_defect_dossier(&regex_path);
    assert!(
        res1.is_err(),
        "Must reject regex colons in Section 6 rust block"
    );
    let errs1 = res1.err().ok_or("expected Err")?.join("\n");
    assert!(
        errs1.contains("Check 34") || errs1.contains("regex"),
        "Expected Check 34 / regex violation, got: {}",
        errs1
    );

    let snippet_2 = "```rust\npub fn parse_text(input: &str) -> bool {\n    let re = Regex::new(\"^abc\").unwrap();\n    true\n}\n```";
    let regex_new = base.replace(cpp_block, snippet_2);
    assert!(
        validate_defect_dossier(&regex_new).is_err(),
        "Must reject constructor in Section 6 rust block"
    );

    let snippet_3 =
        "```rust\npub fn parse_set() {\n    let set = RegexSet::new(&[\"a\", \"b\"]).unwrap();\n}\n```";
    let regex_set = base.replace(cpp_block, snippet_3);
    assert!(
        validate_defect_dossier(&regex_set).is_err(),
        "Must reject RegexSet in Section 6 rust block"
    );

    let snippet_4 = "```rust\nuse regex::Regex;\npub fn helper() {}\n```";
    let use_regex = base.replace(cpp_block, snippet_4);
    assert!(
        validate_defect_dossier(&use_regex).is_err(),
        "Must reject use regex in Section 6 rust block"
    );

    let valid_rust = base.replace(
        cpp_block,
        "```rust\npub fn parse_text_deterministic(input: &str) -> bool {\n    input.starts_with(\"abc\")\n}\n```",
    );
    assert!(
        validate_defect_dossier(&valid_rust).is_ok(),
        "Valid zero-regex Rust block must pass"
    );
    Ok(())
}

#[test]
fn test_rejects_downstream_crate_paths_in_upstream_workspace(
) -> Result<(), Box<dyn std::error::Error>> {
    let base = sample_valid_critical_dossier();

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

    for crate_name in [
        "deap-core",
        "compile-sysml",
        "ingest-sysml",
        "verify-baseline",
        "assemble-conops",
        "reconcile-backlog",
    ] {
        let valid_upstream = base.replace(
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
