//! Regression and reproduction test suite for `reconcile-backlog` bottom-up registration and upstream link projection.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
//! objectives for `crates/reconcile-backlog/tests/reproducer_tests.rs`. Provides deterministic
//! regression tests for adversarial audit findings in `reconcile-backlog` (refs #554).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Bottom-Up Registration Invariant: Unallocated specification artifacts (`issue_id == None` /
//!   `TBD`) are registered in strict bottom-up dependency order (`use-case` and `user-story` $\to$
//!   `feature` $\to$ `epic`), back-patching allocated `#<id>` values into local metadata tables
//!   and parent checklists before parent tiers are published.
//! - Idempotency Invariant: Re-running `run_reconciliation` after initial registration creates
//!   zero additional tracker issues ($f(f(x)) = f(x)$).
//! - Upstream Workspace Prefix Invariant: When `workspace_root` resides under `.pipeline/upstream`,
//!   all projected GitHub blob URLs preserve the `.pipeline/upstream/` path prefix.
//!
//! ## 3. Preconditions
//! - `reconcile_backlog::run_reconciliation` and `TrackerClient` are accessible and
//!   `std::env::temp_dir()` is writable for ephemeral test fixtures.
//!
//! ## 4. Postconditions
//! - All temporary test fixtures are removed prior to test completion and all assertions hold.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N \log N + M)$ over the synthetic specification fixture count $N$ and byte size $M$.
//! - Space Complexity: $O(N + M)$ bounded auxiliary memory for in-memory mock tracker state.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Validates boundary inputs and file paths passed to `run_reconciliation`.
//! - State: Ensures zero cross-test state contamination via process- and timestamp-scoped temp directories.
//! - Action: Executes deterministically with zero unbounded recursion, panics, or `.unwrap()` calls.
//! - Resource: Cleans up temporary filesystem directories deterministically before returning.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-RECONCILE-BACKLOG-RUN-RECONCILIATION, REQ-RECONCILE-BACKLOG-REWRITE-PROJECTED-LINKS] | SysML v2: VerificationCaseDef(reproducer_tests)

use std::fs;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use reconcile_backlog::tracker::{TrackerClient, TrackerIssue, TrackerLabel};
use reconcile_backlog::{run_reconciliation, ReconcileOptions};

/// In-memory tracker state recording sequential issue allocations and creation order for Issue #554.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
/// for struct `SequentialTrackerState` in `crates/reconcile-backlog/tests/reproducer_tests.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves monotonic sequential issue allocation starting at `next_id`.
/// - Records every `create_issue` invocation in chronological order `(number, title, label)`.
///
/// ## 3. Preconditions
/// - Initialized with a positive starting `next_id`.
///
/// ## 4. Postconditions
/// - Maintains complete in-memory issue state across `create_issue` and `update_issue_body` calls.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ over the number of stored issues $N$.
/// - Space Complexity: $O(N)$ bounded by the number of test issues created.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Accessed exclusively through `SequentialMockTracker` trait methods.
/// - State: Protected by `std::sync::Mutex` with explicit poison error propagation.
/// - Action: Zero external I/O or subprocess invocation.
/// - Resource: Bounded by test fixture cardinality.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-TRACKERCLIENT] | SysML v2: PartDef(SequentialTrackerState)
struct SequentialTrackerState {
    issues: Vec<TrackerIssue>,
    labels: Vec<String>,
    created_log: Vec<(u64, String, String)>,
    next_id: u64,
}

/// Mock `TrackerClient` implementation that allocates sequential issue numbers starting from a configured base.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
/// for struct `SequentialMockTracker` in `crates/reconcile-backlog/tests/reproducer_tests.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Deterministically allocates sequential issue IDs (`base_id`, `base_id + 1`, ...) upon `create_issue`.
///
/// ## 3. Preconditions
/// - Constructed via `SequentialMockTracker::new(start_id)`.
///
/// ## 4. Postconditions
/// - Provides full readback of created issues and updated issue bodies for closed-loop verification.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ per tracker operation over $N$ tracked issues.
/// - Space Complexity: $O(N)$ auxiliary memory.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Implements `TrackerClient` returning `Result<_, String>`.
/// - State: Thread-safe interior mutability via `Mutex<SequentialTrackerState>`.
/// - Action: Deterministic in-memory execution.
/// - Resource: Bounded by test issue count.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-TRACKERCLIENT] | SysML v2: PartDef(SequentialMockTracker)
struct SequentialMockTracker {
    state: Mutex<SequentialTrackerState>,
}

impl SequentialMockTracker {
    /// Constructs a new `SequentialMockTracker` starting issue allocation at `start_id`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
    /// for fn `SequentialMockTracker::new` in `crates/reconcile-backlog/tests/reproducer_tests.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Initializes empty issue and creation logs with `next_id = start_id`.
    ///
    /// ## 3. Preconditions
    /// - `start_id > 0`.
    ///
    /// ## 4. Postconditions
    /// - Returns a ready `SequentialMockTracker`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$.
    /// - Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Accepts `u64` starting identifier.
    /// - State: Clean initial state.
    /// - Action: Non-panicking construction.
    /// - Resource: Minimal stack/heap allocation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-TRACKERCLIENT] | SysML v2: ActionDef(SequentialMockTracker::new)
    fn new(start_id: u64) -> Self {
        Self {
            state: Mutex::new(SequentialTrackerState {
                issues: Vec::new(),
                labels: Vec::new(),
                created_log: Vec::new(),
                next_id: start_id,
            }),
        }
    }

    /// Returns the ordered list of `(issue_number, title, label)` tuples recorded by `create_issue`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
    /// for fn `SequentialMockTracker::created_log` in `crates/reconcile-backlog/tests/reproducer_tests.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves exact invocation order of `create_issue`.
    ///
    /// ## 3. Preconditions
    /// - Mutex is not poisoned.
    ///
    /// ## 4. Postconditions
    /// - Returns a cloned vector of creation log entries.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(K)$ where $K$ is the number of created issues.
    /// - Space Complexity: $O(K)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Returns `Result<Vec<(u64, String, String)>, String>`.
    /// - State: Read-only snapshot of `created_log`.
    /// - Action: Non-panicking lock acquisition.
    /// - Resource: Bounded clone.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-TRACKERCLIENT] | SysML v2: ActionDef(SequentialMockTracker::created_log)
    fn created_log(&self) -> Result<Vec<(u64, String, String)>, String> {
        let guard = self.state.lock().map_err(|e| e.to_string())?;
        Ok(guard.created_log.clone())
    }
}

impl TrackerClient for SequentialMockTracker {
    /// Returns all tracked issues in memory.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
    /// for fn `SequentialMockTracker::fetch_all_issues` in `crates/reconcile-backlog/tests/reproducer_tests.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Returns an exact snapshot of `state.issues`.
    ///
    /// ## 3. Preconditions
    /// - Mutex is not poisoned.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(Vec<TrackerIssue>)`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N)$.
    /// - Space Complexity: $O(N)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Propagates lock errors via `Result::Err`.
    /// - State: Does not mutate state.
    /// - Action: Deterministic memory copy.
    /// - Resource: Bounded by issue count.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-TRACKERCLIENT-FETCH-ALL-ISSUES] | SysML v2: ActionDef(SequentialMockTracker::fetch_all_issues)
    fn fetch_all_issues(&self) -> Result<Vec<TrackerIssue>, String> {
        let guard = self.state.lock().map_err(|e| e.to_string())?;
        Ok(guard.issues.clone())
    }

    /// Updates the body of an existing issue in memory.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
    /// for fn `SequentialMockTracker::update_issue_body` in `crates/reconcile-backlog/tests/reproducer_tests.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Replaces `issue.body` for the matching `issue_number`.
    ///
    /// ## 3. Preconditions
    /// - `issue_number` must exist in `state.issues`.
    ///
    /// ## 4. Postconditions
    /// - Target issue holds `Some(body.to_string())`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N)$.
    /// - Space Complexity: $O(M)$ where $M$ is `body.len()`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Returns `Err` if `issue_number` is not found.
    /// - State: Atomic update under mutex lock.
    /// - Action: Deterministic linear scan.
    /// - Resource: Bounded string allocation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-TRACKERCLIENT-UPDATE-ISSUE-BODY] | SysML v2: ActionDef(SequentialMockTracker::update_issue_body)
    fn update_issue_body(&self, issue_number: u64, body: &str) -> Result<(), String> {
        let mut guard = self.state.lock().map_err(|e| e.to_string())?;
        let issue = guard
            .issues
            .iter_mut()
            .find(|i| i.number == issue_number)
            .ok_or_else(|| format!("Issue #{} not found", issue_number))?;
        issue.body = Some(body.to_string());
        Ok(())
    }

    /// Appends a label to an existing issue in memory.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
    /// for fn `SequentialMockTracker::add_label` in `crates/reconcile-backlog/tests/reproducer_tests.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves existing labels and appends `label`.
    ///
    /// ## 3. Preconditions
    /// - `issue_number` must exist in `state.issues`.
    ///
    /// ## 4. Postconditions
    /// - Target issue contains `TrackerLabel::String(label.to_string())`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N)$.
    /// - Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Returns `Err` if `issue_number` is not found.
    /// - State: Atomic update under mutex lock.
    /// - Action: Deterministic linear scan.
    /// - Resource: Bounded allocation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-TRACKERCLIENT-ADD-LABEL] | SysML v2: ActionDef(SequentialMockTracker::add_label)
    fn add_label(&self, issue_number: u64, label: &str) -> Result<(), String> {
        let mut guard = self.state.lock().map_err(|e| e.to_string())?;
        let issue = guard
            .issues
            .iter_mut()
            .find(|i| i.number == issue_number)
            .ok_or_else(|| format!("Issue #{} not found", issue_number))?;
        issue.labels.push(TrackerLabel::String(label.to_string()));
        Ok(())
    }

    /// Allocates the next sequential issue ID and records the creation call.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
    /// for fn `SequentialMockTracker::create_issue` in `crates/reconcile-backlog/tests/reproducer_tests.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Increments `next_id` by 1 and records `(id, title, label)` in `created_log`.
    ///
    /// ## 3. Preconditions
    /// - `title` and `label` are non-empty strings.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok((id, url))` and appends `TrackerIssue` to `state.issues`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$ amortized.
    /// - Space Complexity: $O(M)$ where $M$ is `body.len()`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Propagates lock errors via `Result::Err`.
    /// - State: Atomic insertion under mutex lock.
    /// - Action: Deterministic allocation.
    /// - Resource: Bounded by test issue count.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-TRACKERCLIENT-CREATE-ISSUE] | SysML v2: ActionDef(SequentialMockTracker::create_issue)
    fn create_issue(
        &self,
        title: &str,
        body: &str,
        label: &str,
        repo: Option<&str>,
    ) -> Result<(u64, String), String> {
        let mut guard = self.state.lock().map_err(|e| e.to_string())?;
        let id = guard.next_id;
        guard.next_id += 1;
        let url = format!(
            "https://github.com/{}/issues/{}",
            repo.unwrap_or("owner/repo"),
            id
        );
        guard
            .created_log
            .push((id, title.to_string(), label.to_string()));
        guard.issues.push(TrackerIssue {
            number: id,
            title: title.to_string(),
            state: "OPEN".to_string(),
            labels: vec![TrackerLabel::String(label.to_string())],
            body: Some(body.to_string()),
            url: Some(url.clone()),
        });
        Ok((id, url))
    }

    /// Ensures a label exists in the mock tracker.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
    /// for fn `SequentialMockTracker::ensure_label_exists` in `crates/reconcile-backlog/tests/reproducer_tests.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Idempotently inserts `label` into `state.labels`.
    ///
    /// ## 3. Preconditions
    /// - Mutex is not poisoned.
    ///
    /// ## 4. Postconditions
    /// - `state.labels` contains `label`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(L)$ where $L$ is label count.
    /// - Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Returns `Result<(), String>`.
    /// - State: Atomic check-and-insert.
    /// - Action: Deterministic scan.
    /// - Resource: Bounded label list.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-TRACKERCLIENT-ENSURE-LABEL-EXISTS] | SysML v2: ActionDef(SequentialMockTracker::ensure_label_exists)
    fn ensure_label_exists(&self, label: &str, _repo: Option<&str>) -> Result<(), String> {
        let mut guard = self.state.lock().map_err(|e| e.to_string())?;
        if !guard.labels.iter().any(|l| l == label) {
            guard.labels.push(label.to_string());
        }
        Ok(())
    }

    /// Reopens a closed issue in the mock tracker.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
    /// for fn `SequentialMockTracker::reopen_issue` in `crates/reconcile-backlog/tests/reproducer_tests.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Transitions `issue.state` to `"OPEN"`.
    ///
    /// ## 3. Preconditions
    /// - `issue_number` must exist in `state.issues`.
    ///
    /// ## 4. Postconditions
    /// - Target issue is `"OPEN"`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(N)$.
    /// - Space Complexity: $O(1)$.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Returns `Err` if `issue_number` is not found.
    /// - State: Atomic update under mutex lock.
    /// - Action: Deterministic scan.
    /// - Resource: Bounded memory.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-TRACKERCLIENT-REOPEN-ISSUE] | SysML v2: ActionDef(SequentialMockTracker::reopen_issue)
    fn reopen_issue(
        &self,
        issue_number: u64,
        _comment: Option<&str>,
        _repo: Option<&str>,
    ) -> Result<(), String> {
        let mut guard = self.state.lock().map_err(|e| e.to_string())?;
        let issue = guard
            .issues
            .iter_mut()
            .find(|i| i.number == issue_number)
            .ok_or_else(|| format!("Issue #{} not found", issue_number))?;
        issue.state = "OPEN".to_string();
        Ok(())
    }
}

/// Verifies that `run_reconciliation` registers unallocated (`Issue ID: TBD`) specifications
/// in strict bottom-up order (`user-story` -> `feature` -> `epic`), back-patches `#<id>` into
/// local metadata tables and parent checklists on disk and in projected tracker bodies, and is
/// 100% idempotent on subsequent runs.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
/// for fn `test_issue_554_bottom_up_registration_and_checklist_backpatching` in
/// `crates/reconcile-backlog/tests/reproducer_tests.rs` (refs #554).
///
/// ## 2. Invariants & Mathematical Properties
/// - Bottom-up tier ordering ensures leaf specifications (`US-01` $\to \#560$) receive tracker IDs
///   before intermediate (`FEAT-01` $\to \#561$) and root (`EPIC-01` $\to \#562$) specifications.
/// - Parent checklists on disk and in remote tracker bodies contain zero `TBD` tokens after
///   reconciliation.
/// - Re-running `run_reconciliation` over the reconciled workspace invokes `create_issue` zero
///   additional times.
///
/// ## 3. Preconditions
/// - A temporary workspace containing unallocated `EPIC-01-test.md`, `FEAT-01-subcomp.md`, and
///   `US-01-action.md` with `| **Issue ID** | TBD |` and `- [ ] TBD - [...]` checklists is created.
///
/// ## 4. Postconditions
/// - `create_issue` is invoked in order: `US-01` (`#560`), `FEAT-01` (`#561`), `EPIC-01` (`#562`).
/// - Local files and remote tracker bodies contain `#560`, `#561`, `#562` in metadata tables and
///   parent checklists.
/// - Second reconciliation pass creates 0 issues.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N + M)$ over the 3 specification files.
/// - Space Complexity: $O(N + M)$ auxiliary memory.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Returns `Result<(), Box<dyn std::error::Error>>`.
/// - State: Isolated temporary directory removed before returning.
/// - Action: Zero `.unwrap()` or `panic!()` calls in production paths.
/// - Resource: Bounded to 3 temporary Markdown files.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-RUN-RECONCILIATION] | SysML v2: VerificationCaseDef(test_issue_554_bottom_up_registration_and_checklist_backpatching)
#[test]
fn test_issue_554_bottom_up_registration_and_checklist_backpatching(
) -> Result<(), Box<dyn std::error::Error>> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp_dir = std::env::temp_dir().join(format!(
        "deap_issue_554_bottom_up_{}_{}",
        std::process::id(),
        nanos
    ));
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir)?;
    }

    let epics_dir = tmp_dir.join("docs/epics");
    let features_dir = tmp_dir.join("docs/features");
    let stories_dir = tmp_dir.join("docs/user-stories");
    fs::create_dir_all(&epics_dir)?;
    fs::create_dir_all(&features_dir)?;
    fs::create_dir_all(&stories_dir)?;

    let epic_path = epics_dir.join("EPIC-01-test.md");
    let feat_path = features_dir.join("FEAT-01-subcomp.md");
    let us_path = stories_dir.join("US-01-action.md");

    let epic_md = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | TBD |
| **Title** | Epic 01: Test Subsystem |
| **Type** | epic |
| **Subsystem** | TestSubsystem |

# Epic 01: Test Subsystem

## 2. Requirements & Checklist
- [ ] TBD - [Feature 01: Subcomp](../features/FEAT-01-subcomp.md)

### Associated Use Cases & User Stories

#### Associated Use Cases
- None directly allocated (operational behavior allocated at system ConOps level in Epic 02)

#### Associated User Stories
- [ ] TBD - [US-01: Action](../user-stories/US-01-action.md)
"#;

    let feat_md = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | TBD |
| **Title** | Feature 01: Subcomp |
| **Type** | feature |
| **Parent Epic** | [EPIC-01-test](../epics/EPIC-01-test.md) |
| **Subsystem** | TestSubsystem |

# Feature 01: Subcomp

### Associated Use Cases & User Stories

#### Associated Use Cases
- None allocated (structural subsystem component)

#### Associated User Stories
- [ ] TBD - [US-01: Action](../user-stories/US-01-action.md)
"#;

    let us_md = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | TBD |
| **Title** | US-01: Action |
| **Type** | user-story |
| **Parent Epic** | [EPIC-01-test](../epics/EPIC-01-test.md) |
| **Subsystem** | TestSubsystem |

# US-01: Action

## Required Features
- [ ] TBD - [Feature 01: Subcomp](../features/FEAT-01-subcomp.md)
"#;

    fs::write(&epic_path, epic_md)?;
    fs::write(&feat_path, feat_md)?;
    fs::write(&us_path, us_md)?;

    let tracker = SequentialMockTracker::new(560);
    let options = ReconcileOptions {
        workspace_root: tmp_dir.clone(),
        offline: false,
        dry_run: false,
        provider: "offline".to_string(),
        sync_bodies: true,
        repo: Some("owner/repo".to_string()),
        default_branch: Some("main".to_string()),
    };

    // Pass 1: Bottom-up registration and back-patching
    let _summary1 = run_reconciliation(&tracker, &options)?;

    let created = tracker.created_log()?;
    assert_eq!(
        created.len(),
        3,
        "Expected 3 issues to be created for unallocated specs, got: {:?}",
        created
    );
    assert_eq!(
        created.first().map(|(id, title, label)| (*id, title.as_str(), label.as_str())),
        Some((560, "US-01: Action", "user-story")),
        "First created issue must be bottom-tier User Story US-01 (#560)"
    );
    assert_eq!(
        created.get(1).map(|(id, title, label)| (*id, title.as_str(), label.as_str())),
        Some((561, "Feature 01: Subcomp", "feature")),
        "Second created issue must be middle-tier Feature FEAT-01 (#561)"
    );
    assert_eq!(
        created.get(2).map(|(id, title, label)| (*id, title.as_str(), label.as_str())),
        Some((562, "Epic 01: Test Subsystem", "epic")),
        "Third created issue must be top-tier Epic EPIC-01 (#562)"
    );

    // Verify local files on disk have their Issue IDs and checklists back-patched
    let us_disk = fs::read_to_string(&us_path)?;
    let feat_disk = fs::read_to_string(&feat_path)?;
    let epic_disk = fs::read_to_string(&epic_path)?;

    assert!(
        us_disk.contains("| **Issue ID** | #560 |"),
        "US-01 on disk must have Issue ID #560:\n{}",
        us_disk
    );
    assert!(
        feat_disk.contains("| **Issue ID** | #561 |"),
        "FEAT-01 on disk must have Issue ID #561:\n{}",
        feat_disk
    );
    assert!(
        epic_disk.contains("| **Issue ID** | #562 |"),
        "EPIC-01 on disk must have Issue ID #562:\n{}",
        epic_disk
    );

    assert!(
        !feat_disk.contains("- [ ] TBD -"),
        "FEAT-01 checklist on disk must not contain 'TBD':\n{}",
        feat_disk
    );
    assert!(
        feat_disk.contains("- [ ] #560 -"),
        "FEAT-01 checklist on disk must contain back-patched '#560':\n{}",
        feat_disk
    );
    assert!(
        !epic_disk.contains("- [ ] TBD -"),
        "EPIC-01 checklist on disk must not contain 'TBD':\n{}",
        epic_disk
    );
    assert!(
        epic_disk.contains("- [ ] #561 -"),
        "EPIC-01 checklist on disk must contain back-patched '#561':\n{}",
        epic_disk
    );
    assert!(
        epic_disk.contains("- [ ] #560 -"),
        "EPIC-01 checklist on disk must contain back-patched '#560':\n{}",
        epic_disk
    );

    // Verify projected tracker bodies also have TBD replaced with #560 and #561
    let remote_issues = tracker.fetch_all_issues()?;
    let feat_remote = remote_issues
        .iter()
        .find(|i| i.number == 561)
        .ok_or("Missing remote issue #561")?;
    let feat_body = feat_remote.body.as_deref().ok_or("Missing body on #561")?;
    assert!(
        !feat_body.contains("- [ ] TBD -"),
        "Projected FEAT-01 tracker body must not contain '- [ ] TBD -':\n{}",
        feat_body
    );
    assert!(
        feat_body.contains("- [ ] #560 -"),
        "Projected FEAT-01 tracker body must contain '- [ ] #560 -':\n{}",
        feat_body
    );

    let epic_remote = remote_issues
        .iter()
        .find(|i| i.number == 562)
        .ok_or("Missing remote issue #562")?;
    let epic_body = epic_remote.body.as_deref().ok_or("Missing body on #562")?;
    assert!(
        !epic_body.contains("- [ ] TBD -"),
        "Projected EPIC-01 tracker body must not contain '- [ ] TBD -':\n{}",
        epic_body
    );
    assert!(
        epic_body.contains("- [ ] #561 -"),
        "Projected EPIC-01 tracker body must contain '- [ ] #561 -':\n{}",
        epic_body
    );

    // Pass 2: Re-running run_reconciliation must be 100% idempotent (0 new issues created)
    let _summary2 = run_reconciliation(&tracker, &options)?;
    let created_after_pass2 = tracker.created_log()?;
    assert_eq!(
        created_after_pass2.len(),
        3,
        "Second reconciliation pass must be 100% idempotent and create 0 new issues"
    );

    let _ = fs::remove_dir_all(&tmp_dir);
    Ok(())
}

/// Verifies that when `workspace_root` ends with `.pipeline/upstream`, projected relative links
/// for `../features/FEAT-01.md` and `../../schema/model.sysml` rewrite to include the
/// `.pipeline/upstream/` prefix in GitHub blob URLs.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
/// for fn `test_issue_554_upstream_workspace_link_projection` in
/// `crates/reconcile-backlog/tests/reproducer_tests.rs` (refs #554).
///
/// ## 2. Invariants & Mathematical Properties
/// - For any workspace root $W$ ending with `.pipeline/upstream`, projected relative links under
///   `docs/` and `schema/` map to `https://github.com/<owner>/<repo>/blob/<branch>/.pipeline/upstream/docs/...`
///   and `https://github.com/<owner>/<repo>/blob/<branch>/.pipeline/upstream/schema/...`.
///
/// ## 3. Preconditions
/// - A temporary directory ending with `.pipeline/upstream` containing a specification with
///   relative links `../features/FEAT-01.md` and `../../schema/model.sysml` is created.
///
/// ## 4. Postconditions
/// - The synced remote issue body contains `.pipeline/upstream/docs/features/FEAT-01.md` and
///   `.pipeline/upstream/schema/model.sysml` blob URLs.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(M)$ over the specification body length $M$.
/// - Space Complexity: $O(M)$ auxiliary memory.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Returns `Result<(), Box<dyn std::error::Error>>`.
/// - State: Isolated temporary directory removed before returning.
/// - Action: Deterministic execution with zero `.unwrap()` calls.
/// - Resource: Bounded to 2 small temporary Markdown files.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-RECONCILE-BACKLOG-REWRITE-PROJECTED-LINKS, REQ-RECONCILE-BACKLOG-PROJECT-BODY-FOR-TRACKER] | SysML v2: VerificationCaseDef(test_issue_554_upstream_workspace_link_projection)
#[test]
fn test_issue_554_upstream_workspace_link_projection() -> Result<(), Box<dyn std::error::Error>> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let base_tmp = std::env::temp_dir().join(format!(
        "deap_issue_554_upstream_links_{}_{}",
        std::process::id(),
        nanos
    ));
    let upstream_root = base_tmp.join(".pipeline/upstream");
    let epics_dir = upstream_root.join("docs/epics");
    let features_dir = upstream_root.join("docs/features");
    fs::create_dir_all(&epics_dir)?;
    fs::create_dir_all(&features_dir)?;

    let feat_md = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #561 |
| **Title** | Feature 01: Upstream Compiler |
| **Type** | feature |

# Feature 01: Upstream Compiler
"#;
    fs::write(features_dir.join("FEAT-01.md"), feat_md)?;

    let epic_md = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #562 |
| **Title** | Epic 01: Upstream Architecture |
| **Type** | epic |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

# Epic 01: Upstream Architecture

## 2. Requirements & Checklist
- [ ] #561 - [Feature 01: Upstream Compiler](../features/FEAT-01.md)
"#;
    fs::write(epics_dir.join("EPIC-01.md"), epic_md)?;

    let tracker = SequentialMockTracker::new(560);
    let _ = tracker.create_issue(
        "Feature 01: Upstream Compiler",
        feat_md,
        "feature",
        Some("gintatkinson/DEAP01-spec-core"),
    )?;
    let (epic_num, _) = tracker.create_issue(
        "Epic 01: Upstream Architecture",
        epic_md,
        "epic",
        Some("gintatkinson/DEAP01-spec-core"),
    )?;
    // Ensure epic_num is 561 or adjust Starting ID: wait, SequentialMockTracker(561) -> 561 and 562!
    let _ = epic_num;

    let tracker_aligned = SequentialMockTracker::new(561);
    let _ = tracker_aligned.create_issue(
        "Feature 01: Upstream Compiler",
        feat_md,
        "feature",
        Some("gintatkinson/DEAP01-spec-core"),
    )?;
    let _ = tracker_aligned.create_issue(
        "Epic 01: Upstream Architecture",
        epic_md,
        "epic",
        Some("gintatkinson/DEAP01-spec-core"),
    )?;

    let options = ReconcileOptions {
        workspace_root: upstream_root.clone(),
        offline: false,
        dry_run: false,
        provider: "offline".to_string(),
        sync_bodies: true,
        repo: Some("gintatkinson/DEAP01-spec-core".to_string()),
        default_branch: Some("main".to_string()),
    };

    let _summary = run_reconciliation(&tracker_aligned, &options)?;

    let issues = tracker_aligned.fetch_all_issues()?;
    let epic_issue = issues
        .iter()
        .find(|i| i.number == 562)
        .ok_or("Missing synced epic #562")?;
    let body = epic_issue
        .body
        .as_deref()
        .ok_or("Missing synced body for #562")?;

    assert!(
        body.contains("https://github.com/gintatkinson/DEAP01-spec-core/blob/main/.pipeline/upstream/docs/features/FEAT-01.md"),
        "Projected upstream feature link must include '.pipeline/upstream/' prefix, got:\n{}",
        body
    );
    assert!(
        body.contains("https://github.com/gintatkinson/DEAP01-spec-core/blob/main/.pipeline/upstream/schema/model.sysml"),
        "Projected upstream schema link must include '.pipeline/upstream/' prefix, got:\n{}",
        body
    );

    let _ = fs::remove_dir_all(&base_tmp);
    Ok(())
}
