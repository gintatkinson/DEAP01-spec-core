//! Integration tests for the #468 defect test-target reconciliation contract
//! in crates/reconcile-backlog (refs #468).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety
//! objectives for verifying defect resolution gating and bidirectional test-target parity.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Bug issues transitioning to resolved status MUST have identical, non-empty test-targets
//!   in both HTML comment and bullet format, grounded under `crates/`, whose `cargo test` passes.
//! - Non-bug issues (feature, epic, user-story, use-case, chore) are exempted.
//!
//! ## 3. Preconditions
//! - Local filesystem access to the DEAP01-spec-core repository root or temporary test fixtures.
//!
//! ## 4. Postconditions
//! - All 6 mandatory test scenarios and edge cases execute deterministically.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Bounded by linear string scanning O(N) and external `cargo test` execution time.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Fails closed on any parse mismatch, missing path, or test process exit failure.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-RECONCILE-BACKLOG-TEST-TARGET-CONTRACT-TESTS, Issue #468]

use reconcile_backlog::tracker::{OfflineTracker, TrackerIssue, TrackerLabel};
use reconcile_backlog::{
    extract_defect_test_target, is_bug_issue, run_reconciliation,
    validate_issue_resolution_transition, verify_defect_test_target,
    verify_reconciled_defect_issues, ReconcileOptions,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Scoped temporary Cargo workspace fixture for negative-probe testing of failing test targets.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `ScopedTempTestWorkspace`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Creates isolated temporary Cargo workspace with an intentional assertion failure.
/// - Implements RAII `Drop` to guarantee immediate removal upon test exit.
///
/// ## 3. Preconditions
/// - Host OS provides write access to `std::env::temp_dir()`.
///
/// ## 4. Postconditions
/// - Guaranteed filesystem cleanup via RAII destructor.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) directory and file generation.
/// - Space Complexity: O(1) temporary disk footprint (< 10 KB).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Drops cleanly even if panics occur during test assertions.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-TEMP-FIXTURE]
struct ScopedTempTestWorkspace {
    root: PathBuf,
}

impl ScopedTempTestWorkspace {
    /// Creates a minimal Cargo workspace containing a failing test probe.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `ScopedTempTestWorkspace::new_with_failing_test`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Emits valid Cargo.toml and src/lib.rs with an intentionally failing unit test assertion.
    ///
    /// ## 3. Preconditions
    /// - None.
    ///
    /// ## 4. Postconditions
    /// - Returns an initialized `ScopedTempTestWorkspace` instance.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1).
    /// - Space Complexity: O(1).
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Panics with clean diagnostic message if filesystem writes fail.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RECONCILE-BACKLOG-TEMP-FIXTURE-INIT]
    fn new_with_failing_test() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!(
            "deap_test_target_fixture_{}_{}",
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(root.join("crates/fixture-pkg/src")).expect("create fixture dir");

        // Root workspace Cargo.toml
        let root_cargo = r#"[workspace]
members = ["crates/fixture-pkg"]
resolver = "2"
"#;
        fs::write(root.join("Cargo.toml"), root_cargo).expect("write root cargo.toml");

        // Crate Cargo.toml
        let crate_cargo = r#"[package]
name = "fixture-pkg"
version = "0.1.0"
edition = "2021"
"#;
        fs::write(root.join("crates/fixture-pkg/Cargo.toml"), crate_cargo).expect("write crate cargo.toml");

        // Failing test in lib.rs
        let lib_src = r#"
#[test]
fn test_intentional_failure_for_red_probe() {
    assert_eq!(2 + 2, 5, "Intentional failure for red-probe");
}
"#;
        fs::write(root.join("crates/fixture-pkg/src/lib.rs"), lib_src).expect("write crate src/lib.rs");

        Self { root }
    }
}

impl Drop for ScopedTempTestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// =========================================================================
// Mandatory Scenarios 1 to 6
// =========================================================================

/// Scenario 1: Issue labeled `bug` transitioning to `status:fixed-resolved` with NO `test-target` -> rejected/blocked.
#[test]
fn test_defect_resolution_blocked_when_bug_has_no_test_target() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let issue = TrackerIssue {
        number: 101,
        title: "Fix crash on invalid input".to_string(),
        state: "OPEN".to_string(),
        labels: vec![
            TrackerLabel::String("bug".to_string()),
        ],
        body: Some("## Description\nThis is a bug without test target annotations.".to_string()),
        url: None,
    };

    assert!(is_bug_issue(&issue), "Expected issue labeled 'bug' to be classified as bug");
    let result = validate_issue_resolution_transition(repo_root, &issue, Some("status:fixed-resolved"));
    assert!(result.is_err(), "Expected bug without test-target to be blocked from fixed-resolved");
    let err = result.unwrap_err();
    assert!(err.contains("test-target"), "Error message should mention test-target: {}", err);
}

/// Scenario 2: Issue with mismatched `<!-- test-target: path_a -->` and `- **Test-Target**: path_b` -> rejected/blocked.
#[test]
fn test_defect_resolution_blocked_when_test_targets_mismatch() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let body = r#"## 1. Context and References

<!-- test-target: crates/reconcile-backlog/src/lib.rs -->

- **File**: `crates/reconcile-backlog/src/lib.rs:1-10`
- **Pillar**: Semantic Traceability
- **Symptom**: missing test target parser
- **Test-Target**: `crates/verify-baseline/src/lib.rs`
"#;
    let issue = TrackerIssue {
        number: 102,
        title: "Mismatched test target paths".to_string(),
        state: "OPEN".to_string(),
        labels: vec![
            TrackerLabel::String("bug".to_string()),
        ],
        body: Some(body.to_string()),
        url: None,
    };

    let result = validate_issue_resolution_transition(repo_root, &issue, Some("status:fixed-resolved"));
    assert!(result.is_err(), "Expected mismatched test-target to be blocked from fixed-resolved");
    let err = result.unwrap_err();
    assert!(err.contains("mismatch"), "Error message should mention mismatch: {}", err);
}

/// Scenario 3: Issue with non-existent `test-target` file path -> rejected/blocked.
#[test]
fn test_defect_resolution_blocked_when_test_target_file_nonexistent() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let body = r#"## 1. Context and References

<!-- test-target: crates/reconcile-backlog/tests/nonexistent_ghost_test.rs -->

- **File**: `crates/reconcile-backlog/src/lib.rs:1-10`
- **Pillar**: Semantic Traceability
- **Symptom**: ghost test target
- **Test-Target**: `crates/reconcile-backlog/tests/nonexistent_ghost_test.rs`
"#;
    let issue = TrackerIssue {
        number: 103,
        title: "Non-existent test target".to_string(),
        state: "OPEN".to_string(),
        labels: vec![
            TrackerLabel::String("bug".to_string()),
        ],
        body: Some(body.to_string()),
        url: None,
    };

    let result = validate_issue_resolution_transition(repo_root, &issue, Some("status:fixed-resolved"));
    assert!(result.is_err(), "Expected non-existent test-target to be blocked from fixed-resolved");
    let err = result.unwrap_err();
    assert!(
        err.contains("does not exist") || err.contains("missing"),
        "Error should explain missing file: {}",
        err
    );
}

/// Scenario 4: Issue whose `test-target` fails `cargo test` -> rejected/blocked.
#[test]
fn test_defect_resolution_blocked_when_test_target_cargo_test_fails() {
    let fixture = ScopedTempTestWorkspace::new_with_failing_test();
    let body = r#"## 1. Context and References

<!-- test-target: crates/fixture-pkg/src/lib.rs -->

- **File**: `crates/fixture-pkg/src/lib.rs:1-5`
- **Pillar**: Test Integrity
- **Symptom**: failing unit test
- **Test-Target**: `crates/fixture-pkg/src/lib.rs`
"#;
    let issue = TrackerIssue {
        number: 104,
        title: "Failing test target".to_string(),
        state: "OPEN".to_string(),
        labels: vec![
            TrackerLabel::String("bug".to_string()),
        ],
        body: Some(body.to_string()),
        url: None,
    };

    let result = validate_issue_resolution_transition(&fixture.root, &issue, Some("status:fixed-resolved"));
    assert!(result.is_err(), "Expected failing cargo test to be blocked from fixed-resolved");
    let err = result.unwrap_err();
    assert!(
        err.contains("cargo test") && (err.contains("failed") || err.contains("exit")),
        "Error should mention cargo test failure: {}",
        err
    );
}

/// Scenario 5: Issue with valid matching `test-target` whose `cargo test` passes -> allowed.
#[test]
fn test_defect_resolution_allowed_when_test_target_passes_cargo_test() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let body = r#"## 1. Context and References

<!-- test-target: crates/reconcile-backlog/src/lib.rs -->

- **File**: `crates/reconcile-backlog/src/lib.rs:1-10`
- **Pillar**: Semantic Traceability
- **Symptom**: verified resolution
- **Test-Target**: `crates/reconcile-backlog/src/lib.rs`
"#;
    let issue = TrackerIssue {
        number: 105,
        title: "Valid passing test target".to_string(),
        state: "OPEN".to_string(),
        labels: vec![
            TrackerLabel::String("bug".to_string()),
        ],
        body: Some(body.to_string()),
        url: None,
    };

    let result = validate_issue_resolution_transition(repo_root, &issue, Some("status:fixed-resolved"));
    assert!(result.is_ok(), "Expected valid test target passing cargo test to be allowed, got: {:?}", result.err());
}

/// Scenario 6: Non-bug issues (feature/epic/user-story/use-case/chore) without `test-target` -> allowed.
#[test]
fn test_non_bug_issues_allowed_to_resolve_without_test_target() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let non_bug_types = [
        ("feature", "FEAT-01: Ingestion Pipeline"),
        ("epic", "EPIC-01: Core Architecture"),
        ("user-story", "US-01: System Bootstrap"),
        ("use-case", "UC-01: Operational Workflow"),
        ("chore", "CHORE: Tooling Maintenance"),
    ];

    for (label, title) in non_bug_types {
        let issue = TrackerIssue {
            number: 200,
            title: title.to_string(),
            state: "OPEN".to_string(),
            labels: vec![
                TrackerLabel::String(label.to_string()),
            ],
            body: Some("## Description\nNon-bug specification item without test-target annotation.".to_string()),
            url: None,
        };

        assert!(!is_bug_issue(&issue), "Expected issue labeled '{}' to not be classified as bug", label);
        let result = validate_issue_resolution_transition(repo_root, &issue, Some("status:fixed-resolved"));
        assert!(
            result.is_ok(),
            "Expected non-bug issue labeled '{}' to be allowed to transition to fixed-resolved without test-target, got: {:?}",
            label,
            result.err()
        );
    }
}

// =========================================================================
// Boundary & Integration Tests
// =========================================================================

/// Tests linear string extraction variations and placeholder rejection.
#[test]
fn test_extract_defect_test_target_variations() {
    // 1. With backticks on both
    let body1 = "<!-- test-target: `crates/deap-core/src/lib.rs` -->\n- **Test-Target**: `crates/deap-core/src/lib.rs`";
    assert_eq!(
        extract_defect_test_target(body1).unwrap(),
        "crates/deap-core/src/lib.rs"
    );

    // 2. Without backticks
    let body2 = "<!-- test-target: crates/deap-core/src/lib.rs -->\n- **Test-Target**: crates/deap-core/src/lib.rs";
    assert_eq!(
        extract_defect_test_target(body2).unwrap(),
        "crates/deap-core/src/lib.rs"
    );

    // 3. Asterisk bullet
    let body3 = "<!-- test-target: crates/deap-core/src/lib.rs -->\n* **Test-Target**: `crates/deap-core/src/lib.rs`";
    assert_eq!(
        extract_defect_test_target(body3).unwrap(),
        "crates/deap-core/src/lib.rs"
    );

    // 4. Missing comment
    let body4 = "- **Test-Target**: `crates/deap-core/src/lib.rs`";
    assert!(extract_defect_test_target(body4).is_err());

    // 5. Missing bullet
    let body5 = "<!-- test-target: crates/deap-core/src/lib.rs -->";
    assert!(extract_defect_test_target(body5).is_err());

    // 6. Placeholder value
    let body6 = "<!-- test-target: [path] -->\n- **Test-Target**: `[path]`";
    assert!(extract_defect_test_target(body6).is_err());

    // 7. Whitespace in path
    let body7 = "<!-- test-target: crates/foo bar/lib.rs -->\n- **Test-Target**: `crates/foo bar/lib.rs`";
    assert!(extract_defect_test_target(body7).is_err());
}

/// Tests that verify_defect_test_target accurately resolves integration test targets vs lib targets.
#[test]
fn test_verify_defect_test_target_resolves_integration_test_and_lib() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    // Lib target under crates/reconcile-backlog/src/lib.rs
    let lib_res = verify_defect_test_target(repo_root, "crates/reconcile-backlog/src/lib.rs");
    assert!(lib_res.is_ok(), "Expected crates/reconcile-backlog/src/lib.rs to pass verify_defect_test_target");

    // Integration test target under crates/verify-baseline/tests/check_33_zero_python_tests.rs
    let it_res = verify_defect_test_target(repo_root, "crates/verify-baseline/tests/check_33_zero_python_tests.rs");
    assert!(it_res.is_ok(), "Expected check_33_zero_python_tests.rs to pass verify_defect_test_target");

    // Path outside crates/
    let out_res = verify_defect_test_target(repo_root, "docs/epics/EPIC-01.md");
    assert!(out_res.is_err(), "Expected path outside crates/ to be rejected");
}

/// Tests that run_reconciliation blocks marking bug checklist items [x] if the bug violates #468.
#[test]
fn test_reconciliation_workflow_rejects_unverified_defect_in_checklist() -> Result<(), Box<dyn std::error::Error>> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!("reconcile_defect_gate_{}", nanos));
    let epics_dir = temp_dir.join("docs/epics");
    fs::create_dir_all(&epics_dir)?;

    let epic_content = r#"---
issue_id: 10
title: "Epic 01: Core"
type: epic
---
# Epic 01: Core

## Defects
- [ ] #500 - Defect 500: Bogus Bug Without Test Target
"#;
    let epic_path = epics_dir.join("EPIC-01-Core.md");
    fs::write(&epic_path, epic_content)?;

    // Bug issue #500 has status:fixed-resolved label, but NO test-target in body
    let issues = vec![
        TrackerIssue {
            number: 10,
            title: "Epic 01: Core".to_string(),
            state: "OPEN".to_string(),
            labels: vec![TrackerLabel::String("epic".to_string())],
            body: None,
            url: None,
        },
        TrackerIssue {
            number: 500,
            title: "Defect 500: Bogus Bug Without Test Target".to_string(),
            state: "OPEN".to_string(),
            labels: vec![
                TrackerLabel::String("bug".to_string()),
                TrackerLabel::String("status:fixed-resolved".to_string()),
            ],
            body: Some("## Description\nNo test target provided.".to_string()),
            url: None,
        },
    ];

    let tracker = OfflineTracker::with_issues(issues);
    let options = ReconcileOptions {
        workspace_root: temp_dir.clone(),
        offline: true,
        dry_run: false,
        provider: "offline".to_string(),
        sync_bodies: false,
        repo: None,
        default_branch: None,
    };

    let _ = run_reconciliation(&tracker, &options)?;

    // Because issue #500 violated the #468 test-target contract, the checklist item must NOT be checked [x]
    let updated = fs::read_to_string(&epic_path)?;
    assert!(
        updated.contains("- [ ] #500 - Defect 500: Bogus Bug Without Test Target"),
        "Defect violating #468 contract should NOT be marked [x], found in spec:\n{}",
        updated
    );

    let _ = fs::remove_dir_all(&temp_dir);
    Ok(())
}

// =========================================================================
// WP2 Iteration 2 Regression Tests (Path Traversal, File Types & Gating Scope)
// =========================================================================

/// Tests that verify_defect_test_target rejects directory traversal ('..') escape attempts.
/// Probes `crates/reconcile-backlog/../../Cargo.toml` and other traversal variants.
#[test]
fn test_verify_defect_test_target_rejects_path_traversal() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    let traversal_variants = [
        "crates/reconcile-backlog/../../Cargo.toml",
        "crates/reconcile-backlog/../../.gitignore",
        "crates/../Cargo.toml",
        "crates/../../etc/passwd",
        "crates/reconcile-backlog/src/../../Cargo.toml",
        "crates/reconcile-backlog/../compile-sysml/Cargo.toml",
        "crates/reconcile-backlog/tests/../src/lib.rs",
    ];

    for path in traversal_variants {
        let result = verify_defect_test_target(repo_root, path);
        assert!(
            result.is_err(),
            "Expected traversal path '{}' to be rejected by verify_defect_test_target",
            path
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("traversal") || err.contains("..") || err.contains("crates/"),
            "Error message for '{}' should explain path traversal or boundary rejection, got: {}",
            path,
            err
        );
    }
}

/// Tests that verify_defect_test_target rejects non-Rust files (must be a .rs file).
/// Probes `crates/reconcile-backlog/Cargo.toml` and `README.md`.
#[test]
fn test_verify_defect_test_target_rejects_non_rust_files() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    // 1. Probing crates/reconcile-backlog/Cargo.toml (valid file under crates/, but .toml not .rs)
    let toml_target = "crates/reconcile-backlog/Cargo.toml";
    assert!(
        repo_root.join(toml_target).is_file(),
        "Prerequisite: crates/reconcile-backlog/Cargo.toml must exist on disk"
    );
    let toml_res = verify_defect_test_target(repo_root, toml_target);
    assert!(
        toml_res.is_err(),
        "Expected non-Rust file '{}' to be rejected by verify_defect_test_target",
        toml_target
    );
    let toml_err = toml_res.unwrap_err();
    assert!(
        toml_err.contains(".rs") || toml_err.contains("Rust source file"),
        "Error message for '{}' should explain .rs requirement, got: {}",
        toml_target,
        toml_err
    );

    // 2. Probing README.md (top-level documentation file, not under crates/ and not .rs)
    let readme_target = "README.md";
    assert!(
        repo_root.join(readme_target).is_file(),
        "Prerequisite: README.md must exist on disk at repo root"
    );
    let readme_res = verify_defect_test_target(repo_root, readme_target);
    assert!(
        readme_res.is_err(),
        "Expected non-Rust, non-crates file '{}' to be rejected by verify_defect_test_target",
        readme_target
    );
    let readme_err = readme_res.unwrap_err();
    assert!(
        readme_err.contains("crates/") || readme_err.contains(".rs"),
        "Error message for '{}' should reject non-crates / non-Rust file, got: {}",
        readme_target,
        readme_err
    );

    // 3. Probing a hypothetical markdown file path under crates/
    let md_target = "crates/reconcile-backlog/README.md";
    let md_res = verify_defect_test_target(repo_root, md_target);
    assert!(
        md_res.is_err(),
        "Expected non-Rust target '{}' to be rejected",
        md_target
    );
}

/// Tests that verify_reconciled_defect_issues fails closed when an active (open) defect issue
/// claims resolved status (e.g. `status:fixed-resolved`) but lacks a valid test-target.
#[test]
fn test_verify_reconciled_defect_issues_fails_closed_on_open_targetless_defect() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    // Case 1: Open bug issue claiming resolved status but lacking test-target annotations
    let open_targetless_issue = TrackerIssue {
        number: 701,
        title: "Active defect claiming resolution without test target".to_string(),
        state: "OPEN".to_string(),
        labels: vec![
            TrackerLabel::String("bug".to_string()),
            TrackerLabel::String("status:fixed-resolved".to_string()),
        ],
        body: Some("## 1. Context and References\nNo test target annotations provided.".to_string()),
        url: None,
    };

    let result = verify_reconciled_defect_issues(repo_root, &[open_targetless_issue]);
    assert!(
        result.is_err(),
        "Expected open defect with status:fixed-resolved lacking test-target to fail closed with Err, got: {:?}",
        result
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("#701") && err.contains("test-target"),
        "Error message should mention issue #701 and test-target failure: {}",
        err
    );

    // Case 2: Open bug issue with body = None claiming resolved status
    let open_none_body_issue = TrackerIssue {
        number: 702,
        title: "Active defect with None body claiming resolved status".to_string(),
        state: "open".to_string(),
        labels: vec![
            TrackerLabel::String("defect".to_string()),
            TrackerLabel::String("status:fixed-resolved".to_string()),
        ],
        body: None,
        url: None,
    };

    let result_none_body = verify_reconciled_defect_issues(repo_root, &[open_none_body_issue]);
    assert!(
        result_none_body.is_err(),
        "Expected open defect with None body claiming resolved status to fail closed, got: {:?}",
        result_none_body
    );

    // Case 3: Open bug issue with mismatched test target annotations
    let open_mismatched_issue = TrackerIssue {
        number: 703,
        title: "Active defect with mismatched test target".to_string(),
        state: "OPEN".to_string(),
        labels: vec![
            TrackerLabel::String("bug".to_string()),
            TrackerLabel::String("fixed-resolved".to_string()),
        ],
        body: Some(
            "<!-- test-target: crates/reconcile-backlog/src/lib.rs -->\n- **Test-Target**: `crates/verify-baseline/src/lib.rs`"
                .to_string(),
        ),
        url: None,
    };

    let result_mismatched = verify_reconciled_defect_issues(repo_root, &[open_mismatched_issue]);
    assert!(
        result_mismatched.is_err(),
        "Expected open defect with mismatched test target to fail closed, got: {:?}",
        result_mismatched
    );
}

/// Tests that verify_reconciled_defect_issues ignores historical closed defects (state == "closed")
/// that lack local test targets, ensuring historical issues do not cause reconciliation failure.
#[test]
fn test_verify_reconciled_defect_issues_ignores_historical_closed_defects() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    // 1. Historical closed defect without any test target annotation
    let historical_closed_no_target = TrackerIssue {
        number: 350,
        title: "Historical closed defect from early milestone".to_string(),
        state: "closed".to_string(),
        labels: vec![
            TrackerLabel::String("bug".to_string()),
            TrackerLabel::String("status:fixed-resolved".to_string()),
        ],
        body: Some("## Description\nLegacy closed defect from pre-468 era.".to_string()),
        url: None,
    };

    // 2. Historical closed defect pointing to legacy non-existent path (e.g. Python tests or refactored files)
    let historical_closed_legacy_target = TrackerIssue {
        number: 450,
        title: "Historical closed defect pointing to retired test path".to_string(),
        state: "CLOSED".to_string(),
        labels: vec![
            TrackerLabel::String("bug".to_string()),
            TrackerLabel::String("status:fixed-resolved".to_string()),
        ],
        body: Some(
            "<!-- test-target: tests/test_baseline.py -->\n- **Test-Target**: `tests/test_baseline.py`".to_string(),
        ),
        url: None,
    };

    // Verifying only historical closed defects must succeed with Ok(0)
    let res_historical = verify_reconciled_defect_issues(
        repo_root,
        &[historical_closed_no_target.clone(), historical_closed_legacy_target.clone()],
    );
    assert!(
        res_historical.is_ok(),
        "Expected historical closed defects without local test targets to be ignored, got: {:?}",
        res_historical.err()
    );
    assert_eq!(res_historical.unwrap(), 0);

    // 3. Batch with historical closed defect AND active resolved bug with passing test target
    let active_valid_defect = TrackerIssue {
        number: 468,
        title: "Active defect with valid passing test target".to_string(),
        state: "OPEN".to_string(),
        labels: vec![
            TrackerLabel::String("bug".to_string()),
            TrackerLabel::String("status:fixed-resolved".to_string()),
        ],
        body: Some(
            r#"## 1. Context and References

<!-- test-target: crates/reconcile-backlog/src/lib.rs -->

- **File**: `crates/reconcile-backlog/src/lib.rs:1-10`
- **Pillar**: Semantic Traceability
- **Symptom**: passing verification
- **Test-Target**: `crates/reconcile-backlog/src/lib.rs`
"#
            .to_string(),
        ),
        url: None,
    };

    let res_batch = verify_reconciled_defect_issues(
        repo_root,
        &[historical_closed_no_target, historical_closed_legacy_target, active_valid_defect],
    );
    assert!(
        res_batch.is_ok(),
        "Expected batch with historical closed defects and valid active defect to succeed, got: {:?}",
        res_batch.err()
    );
    assert_eq!(
        res_batch.unwrap(),
        1,
        "Expected exactly 1 active defect to be verified while historical closed defects are ignored"
    );
}

/// Integration test verifying that historical closed defect issues (state == "CLOSED") pointing to
/// retired test paths resolve cleanly to `- [x]` in specification checklists (refs #468).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
/// for backlog reconciliation checklist integrity. Ensures that closed historical defects are not
/// improperly blocked from checklist resolution or reverted to unchecked `- [ ]` state when specs are reconciled.
///
/// ## 2. Invariants & Mathematical Properties
/// - Historical closed defects (`iss.state == "CLOSED"` or `"closed"`) with `iss.is_resolved() == true`
///   MUST resolve cleanly to `- [x]` in specification checklists without being blocked by modern
///   `<!-- test-target: crates/... -->` grounding rules.
/// - Active open defects (`iss.state == "OPEN"`) claiming resolved status MUST remain gated by `validate_issue_resolution_transition`.
/// - Reconciling an unchecked historical closed defect (`- [ ] #450`) to checked (`- [x] #450`) increments `summary.checklists_updated` by 1.
///
/// ## 3. Preconditions
/// - Host OS provides write access to `std::env::temp_dir()`.
/// - An Epic specification exists on disk under `docs/epics/` containing `- [ ] #450 - Defect 450: Historical closed defect pointing to retired test path`.
/// - Offline issue tracker provides a `TrackerIssue` for `#450` with `state: "CLOSED"`, `labels: ["bug", "status:fixed-resolved"]`, and a legacy test target pointing to a retired test path (`tests/test_baseline.py`).
///
/// ## 4. Postconditions
/// - `run_reconciliation` executes successfully (`Ok(summary)`).
/// - The Epic file on disk contains `- [x] #450 - Defect 450: Historical closed defect pointing to retired test path`.
/// - `summary.checklists_updated` equals 1.
/// - The temporary test directory in `std::env::temp_dir()` is removed deterministically via RAII drop.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(lines) + M(issues)) bounded by linear checklist line scanning.
/// - Space Complexity: O(1) auxiliary memory beyond short file and diagnostic buffers (< 10 KB).
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Fails closed: Panics if the checklist item remains `- [ ]` or if `run_reconciliation` returns an error.
/// - Resource safety: Employs a RAII drop guard to ensure temporary directory cleanup even upon test assertion panic.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-HISTORICAL-CLOSED-DEFECT-CHECKLIST, Issue #468]
#[test]
fn test_reconciliation_workflow_historical_closed_defect_resolves_in_checklist(
) -> Result<(), Box<dyn std::error::Error>> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!("reconcile_historical_closed_gate_{}", nanos));
    let epics_dir = temp_dir.join("docs/epics");
    fs::create_dir_all(&epics_dir)?;

    // RAII cleanup guard to guarantee removal of temp_dir on drop (including test failure/panic)
    struct TempDirCleanupGuard(PathBuf);
    impl Drop for TempDirCleanupGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _guard = TempDirCleanupGuard(temp_dir.clone());

    let epic_content = r#"---
issue_id: 10
title: "Epic 01: Core"
type: epic
---
# Epic 01: Core

## Defects
- [ ] #450 - Defect 450: Historical closed defect pointing to retired test path
"#;
    let epic_path = epics_dir.join("EPIC-01-Core.md");
    fs::write(&epic_path, epic_content)?;

    // Bug issue #450 is CLOSED on tracker, but its body points to a legacy retired test path
    let issues = vec![
        TrackerIssue {
            number: 10,
            title: "Epic 01: Core".to_string(),
            state: "OPEN".to_string(),
            labels: vec![TrackerLabel::String("epic".to_string())],
            body: None,
            url: None,
        },
        TrackerIssue {
            number: 450,
            title: "Defect 450: Historical closed defect pointing to retired test path".to_string(),
            state: "CLOSED".to_string(),
            labels: vec![
                TrackerLabel::String("bug".to_string()),
                TrackerLabel::String("status:fixed-resolved".to_string()),
            ],
            body: Some(
                "<!-- test-target: tests/test_baseline.py -->\n- **Test-Target**: `tests/test_baseline.py`"
                    .to_string(),
            ),
            url: None,
        },
    ];

    let tracker = OfflineTracker::with_issues(issues);
    let options = ReconcileOptions {
        workspace_root: temp_dir.clone(),
        offline: true,
        dry_run: false,
        provider: "offline".to_string(),
        sync_bodies: false,
        repo: None,
        default_branch: None,
    };

    let summary = run_reconciliation(&tracker, &options)?;

    // Historical closed defect must resolve cleanly to [x] in the checklist
    let updated = fs::read_to_string(&epic_path)?;
    assert!(
        updated.contains("- [x] #450 - Defect 450: Historical closed defect pointing to retired test path"),
        "Historical closed defect should resolve cleanly to [x] in checklist, but got:\n{}",
        updated
    );
    assert_eq!(
        summary.checklists_updated, 1,
        "Expected exactly 1 checklist item to be resolved and updated"
    );

    Ok(())
}

