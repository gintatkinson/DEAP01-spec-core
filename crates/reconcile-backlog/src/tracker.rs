//! Issue tracker trait, `serde_json` issue models, and deterministic `IssueIndex` lookup engine.
//!
//! Realises: [REQ-RECONCILE-BACKLOG-TRACKER, Issue Tracker as Canonical SSOT, Automated Spec Reconciliation]

pub use crate::tracker_github::GitHubTracker;
pub use crate::tracker_normalize::{
    canonicalize_spec_type, extract_spec_ordinal, infer_spec_type_from_title,
    is_audit_issue_title, is_resolved_label, normalize_label, normalize_title, ordinal_compatible,
};
use crate::tracker_normalize::titles_match_fuzzy;

use crate::ReconcileError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// [DOC-TIER: A] Canonical representation of an issue in the issue tracker (`serde_json` model).
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-ISSUE]
/// Contract: Captures issue number, title, state, labels, and body for SSOT reconciliation.
/// Bounds & Faults: Time O(1), Space O(L + B) over labels and body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub state: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub body: String,
}

impl Issue {
    /// [DOC-TIER: A] Returns `true` if the issue is `CLOSED` or carries a resolved status label.
    /// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-IS-RESOLVED]
    /// Contract: Evaluates `state == "CLOSED"` (case-insensitive) or `labels.any(is_resolved_label)`.
    /// Bounds & Faults: Time O(L), Space O(1) over label count L.
    pub fn is_resolved(&self) -> bool {
        if self.state.eq_ignore_ascii_case("CLOSED") {
            return true;
        }
        self.labels.iter().any(|l| is_resolved_label(l))
    }
}

/// [DOC-TIER: A] Abstraction over issue tracker backends (`GitHubTracker`, `OfflineTracker`, mocks).
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-ISSUETRACKER]
/// Contract: Provides deterministic issue fetching, body synchronization, creation, and reopening.
/// Bounds & Faults: Time O(I), Space O(I) over tracker issue set; returns `ReconcileError` on transport failure.
pub trait IssueTracker {
    /// Fetches all issues (both OPEN and CLOSED) from the tracker.
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-FETCH-ALL-ISSUES]
    fn fetch_all_issues(&self, repo: Option<&str>) -> Result<Vec<Issue>, ReconcileError>;

    /// Updates the Markdown body of an existing issue.
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-EDIT-ISSUE-BODY]
    fn edit_issue_body(
        &self,
        number: u64,
        new_body: &str,
        repo: Option<&str>,
    ) -> Result<(), ReconcileError>;

    /// Updates the title of an existing issue.
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-EDIT-ISSUE-BODY]
    fn edit_issue_title(
        &self,
        _number: u64,
        _new_title: &str,
        _repo: Option<&str>,
    ) -> Result<(), ReconcileError> {
        Ok(())
    }

    /// Creates a new issue and returns its allocated issue number.
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-CREATE-ISSUE]
    fn create_issue(
        &self,
        title: &str,
        body: &str,
        labels: &[String],
        repo: Option<&str>,
    ) -> Result<u64, ReconcileError>;

    /// Reopens a closed issue with an accompanying root-cause analysis comment.
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-REOPEN-ISSUE]
    fn reopen_issue(
        &self,
        number: u64,
        comment: &str,
        repo: Option<&str>,
    ) -> Result<(), ReconcileError>;
}

/// [DOC-TIER: A] Offline no-op tracker used when `--offline` is enabled or during isolated tests.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-OFFLINETRACKER]
/// Contract: Performs zero external network or CLI calls; returns preloaded in-memory issues.
/// Bounds & Faults: Time O(I), Space O(I) over preloaded issues.
#[derive(Debug, Default, Clone)]
pub struct OfflineTracker {
    issues: Vec<Issue>,
}

impl OfflineTracker {
    /// [DOC-TIER: A] Creates an empty `OfflineTracker`.
    /// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-NEW]
    /// Contract: Initializes `OfflineTracker` with an empty issue vector.
    /// Bounds & Faults: Time O(1), Space O(1).
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    /// [DOC-TIER: A] Creates an `OfflineTracker` pre-populated with a fixed list of issues.
    /// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-WITH-ISSUES]
    /// Contract: Stores `issues` for deterministic offline queries.
    /// Bounds & Faults: Time O(1), Space O(I).
    pub fn with_issues(issues: Vec<Issue>) -> Self {
        Self { issues }
    }
}

impl IssueTracker for OfflineTracker {
    /// Returns the preloaded in-memory issues without external I/O.
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-FETCH-ALL-ISSUES]
    fn fetch_all_issues(&self, _repo: Option<&str>) -> Result<Vec<Issue>, ReconcileError> {
        Ok(self.issues.clone())
    }

    /// No-op body edit in offline mode.
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-EDIT-ISSUE-BODY]
    fn edit_issue_body(
        &self,
        _number: u64,
        _new_body: &str,
        _repo: Option<&str>,
    ) -> Result<(), ReconcileError> {
        Ok(())
    }

    /// No-op issue creation in offline mode (returns `0`).
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-CREATE-ISSUE]
    fn create_issue(
        &self,
        _title: &str,
        _body: &str,
        _labels: &[String],
        _repo: Option<&str>,
    ) -> Result<u64, ReconcileError> {
        Ok(0)
    }

    /// No-op issue reopen in offline mode.
    /// Realises: [REQ-RECONCILE-BACKLOG-TRACKER-REOPEN-ISSUE]
    fn reopen_issue(
        &self,
        _number: u64,
        _comment: &str,
        _repo: Option<&str>,
    ) -> Result<(), ReconcileError> {
        Ok(())
    }
}

/// [DOC-TIER: A] Indexed lookup table for matching specification documents to tracker issues.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-ISSUEINDEX]
/// Contract: Indexes issues by number, `(spec_type, normalized_title)`, and `normalized_title`.
/// Bounds & Faults: Time O(I log I), Space O(I) over indexed issues.
#[derive(Debug, Clone, Default)]
pub struct IssueIndex {
    by_number: BTreeMap<u64, Issue>,
    by_type_and_title: BTreeMap<(String, String), u64>,
    by_title_only: BTreeMap<String, Vec<u64>>,
}

impl IssueIndex {
    /// [DOC-TIER: A] Builds an `IssueIndex` from a vector of `Issue` structs.
    /// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-FROM-ISSUES]
    /// Contract: Inserts each issue via `insert_issue` in deterministic order.
    /// Bounds & Faults: Time O(I log I), Space O(I).
    pub fn from_issues(issues: Vec<Issue>) -> Self {
        let mut index = Self::default();
        for issue in issues {
            index.insert_issue(issue);
        }
        index
    }

    /// [DOC-TIER: A] Inserts or updates a single `Issue` in the index.
    /// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-INSERT-ISSUE]
    /// Contract: Populates `by_number`, `by_title_only`, and `by_type_and_title` (from labels and title prefix).
    /// Bounds & Faults: Time O(L log I), Space O(L) over issue labels.
    pub fn insert_issue(&mut self, issue: Issue) {
        let num = issue.number;
        let norm_title = normalize_title(&issue.title);

        if !norm_title.is_empty() && !is_audit_issue_title(&issue.title) {
            let list = self.by_title_only.entry(norm_title.clone()).or_default();
            if !list.contains(&num) {
                list.push(num);
            }

            let mut matched_types = Vec::new();
            for label in &issue.labels {
                let norm_label = normalize_label(label);
                if norm_label.starts_with("type:") || norm_label.starts_with("kind:") {
                    let st = canonicalize_spec_type(&norm_label);
                    if !st.is_empty() {
                        matched_types.push(st);
                    }
                }
            }
            if let Some(inferred) = infer_spec_type_from_title(&issue.title) {
                if !matched_types.contains(&inferred) {
                    matched_types.push(inferred);
                }
            }
            for st in matched_types {
                self.by_type_and_title
                    .entry((st, norm_title.clone()))
                    .or_insert(num);
            }
        }

        self.by_number.insert(num, issue);
    }

    /// [DOC-TIER: A] Retrieves an `Issue` by its numeric ID.
    /// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-GET-ISSUE]
    /// Contract: Returns `Some(&Issue)` when `number` exists in `by_number`.
    /// Bounds & Faults: Time O(log I), Space O(1).
    pub fn get_issue(&self, number: u64) -> Option<&Issue> {
        self.by_number.get(&number)
    }

    /// [DOC-TIER: A] Checks if issue `number` is resolved (`CLOSED` or resolved label).
    /// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-IS-ISSUE-RESOLVED]
    /// Contract: Returns `Some(bool)` if issue exists, or `None` if unknown.
    /// Bounds & Faults: Time O(log I + L), Space O(1).
    pub fn is_issue_resolved(&self, number: u64) -> Option<bool> {
        self.by_number.get(&number).map(|i| i.is_resolved())
    }

    /// [DOC-TIER: A] Validates whether `number` exists in `IssueIndex` and is semantically compatible with `(spec_title, spec_type)`.
    /// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-FIND-ISSUE-FOR-SPEC]
    /// Contract: Rejects missing issues, `[AUDIT]` issues, mismatched spec types, and mismatched spec ordinals.
    /// Bounds & Faults: Time O(log I + T), Space O(T) where I is issue count and T is title length.
    pub fn is_issue_valid_for_spec(
        &self,
        number: u64,
        spec_title: &str,
        spec_type: Option<&str>,
    ) -> bool {
        let Some(remote_issue) = self.get_issue(number) else {
            return false;
        };
        if is_audit_issue_title(&remote_issue.title) {
            return false;
        }
        let canonical_spec_type = spec_type
            .map(canonicalize_spec_type)
            .filter(|s| !s.is_empty())
            .or_else(|| infer_spec_type_from_title(spec_title));
        if let (Some(ref exp_st), Some(rem_st)) = (
            &canonical_spec_type,
            infer_spec_type_from_title(&remote_issue.title),
        ) {
            if exp_st != &rem_st {
                return false;
            }
        }
        if !ordinal_compatible(spec_title, &remote_issue.title) {
            return false;
        }
        if let (Some(ord_spec), Some(ord_remote)) = (
            extract_spec_ordinal(spec_title),
            extract_spec_ordinal(&remote_issue.title),
        ) {
            if ord_spec == ord_remote {
                return true;
            }
        }
        let norm_spec = normalize_title(spec_title);
        let norm_remote = normalize_title(&remote_issue.title);
        !norm_spec.is_empty()
            && !norm_remote.is_empty()
            && (norm_spec == norm_remote
                || norm_remote.contains(&norm_spec)
                || norm_spec.contains(&norm_remote))
    }

    /// [DOC-TIER: A] Finds a matching issue number for a specification by title and optional spec type.
    /// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-TRACKER-FIND-ISSUE-FOR-SPEC]
    /// Contract: Performs 4-tier matching (type+exact, exact, type+substring with ordinal guard, substring with ordinal guard).
    /// Bounds & Faults: Time O(I * T), Space O(T) where I is issue count and T is title length.
    pub fn find_issue_for_spec(&self, spec_title: &str, spec_type: Option<&str>) -> Option<u64> {
        let norm_title = normalize_title(spec_title);
        if norm_title.is_empty() {
            return None;
        }

        let canonical_type = spec_type
            .map(canonicalize_spec_type)
            .filter(|s| !s.is_empty())
            .or_else(|| infer_spec_type_from_title(spec_title));

        if let Some(ref st) = canonical_type {
            if let Some(&num) = self.by_type_and_title.get(&(st.clone(), norm_title.clone())) {
                return Some(num);
            }
        }

        if let Some(nums) = self.by_title_only.get(&norm_title) {
            if let (1, Some(&first_num)) = (nums.len(), nums.first()) {
                if let Some(ref st) = canonical_type {
                    if let Some(issue) = self.by_number.get(&first_num) {
                        if let Some(issue_st) = infer_spec_type_from_title(&issue.title) {
                            if &issue_st != st {
                                return None;
                            }
                        }
                    }
                }
                return Some(first_num);
            }
        }

        if let Some(ref st) = canonical_type {
            for ((cand_st, cand_title), &num) in &self.by_type_and_title {
                if cand_st == st && titles_match_fuzzy(spec_title, &norm_title, cand_title) {
                    return Some(num);
                }
            }
        }

        for (cand_title, nums) in &self.by_title_only {
            if titles_match_fuzzy(spec_title, &norm_title, cand_title) {
                if let Some(&first_num) = nums.first() {
                    if let Some(ref st) = canonical_type {
                        if let Some(issue) = self.by_number.get(&first_num) {
                            if let Some(issue_st) = infer_spec_type_from_title(&issue.title) {
                                if &issue_st != st {
                                    continue;
                                }
                            }
                        }
                    }
                    return Some(first_num);
                }
            }
        }

        None
    }
}
