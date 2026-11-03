//! Core library for `reconcile-backlog`: deterministic Agile specification and issue tracker reconciliation.
//!
//! Synchronizes Markdown specifications (`docs/epics/`, `docs/features/`, `docs/user-stories/`,
//! `docs/use-cases/`) with the canonical issue tracker (`GitHubTracker` or `OfflineTracker`).
//!
//! Realises: [REQ-RECONCILE-BACKLOG-CORE, Issue Tracker as Canonical SSOT, Zero Fallback Placeholders, Automated Spec Reconciliation]

pub mod checklist;
mod checklist_hierarchy;
mod checklist_placeholders;
pub mod create_issue;
mod error;
pub mod frontmatter;
mod frontmatter_update;
mod git_env;
mod projection;
mod registration;
pub mod tracker;
mod tracker_github;
mod tracker_normalize;

pub use self::error::{
    collect_spec_files, write_atomic, ReconcileError, ReconcileOptions, ReconcileSummary,
};
pub use self::git_env::{
    clean_repo_name, detect_git_branch, detect_git_repo, detect_upstream_prefix,
    parse_repo_from_remote_url,
};
pub use self::projection::{project_body_for_tracker, project_body_for_tracker_with_prefix};
pub use self::registration::{
    backpatch_spec_references_in_content, build_spec_path_to_issue_map,
    resolve_canonical_spec_tier, validate_checklist_publish_integrity,
};

use crate::checklist::{
    clean_unpopulated_placeholders_counted, reconcile_checklists_in_content,
    synthesize_and_reconcile_hierarchy, DiscoveredSpec,
};
use crate::frontmatter::{
    extract_issue_number, format_issue_link, is_placeholder_issue_id, parse_frontmatter,
    update_frontmatter_field,
};
use crate::registration::{register_specs_bottom_up, RegistrationContext};
use crate::tracker::{infer_spec_type_from_title, IssueIndex, IssueTracker};
use std::fs;
use std::path::Path;

/// [DOC-TIER: A] Registers unallocated specifications bottom-up (`Use Case -> User Story -> Feature -> Epic`).
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-REGISTER-UNALLOCATED-SPECS-BOTTOM-UP]
/// Contract: Allocates tracker issues tier-by-tier, backpatches parent/child links, and enforces publish gate integrity.
/// Bounds & Faults: Time O(S^2 + L), Space O(S + L) over discovered specs S and document length L.
pub fn register_unallocated_specs_bottom_up(
    dir: &Path,
    discovered: &mut [DiscoveredSpec],
    index: &mut IssueIndex,
    tracker: &dyn IssueTracker,
    (repo, branch, upstream_prefix): (&str, &str, Option<&str>),
) -> Result<usize, ReconcileError> {
    let mut ctx = RegistrationContext {
        index,
        tracker,
        repo,
        branch,
        upstream_prefix,
    };
    register_specs_bottom_up(dir, discovered, &mut ctx)
}

/// Discovers specification files and reconciles frontmatter issue links and placeholders (Phase 2).
/// Realises: [REQ-RECONCILE-BACKLOG-LIB-RUN-RECONCILIATION]
fn discover_and_reconcile_frontmatters(
    spec_files: &[std::path::PathBuf],
    options: &ReconcileOptions,
    effective_repo: Option<&str>,
    index: &IssueIndex,
    summary: &mut ReconcileSummary,
) -> Result<Vec<DiscoveredSpec>, ReconcileError> {
    let mut discovered_specs = Vec::with_capacity(spec_files.len());

    for path in spec_files {
        summary.total_specs_scanned += 1;
        let rel_path = path.strip_prefix(&options.workspace_root).unwrap_or(path).to_string_lossy().replace('\\', "/");
        let raw_content = fs::read_to_string(path)?;
        let fm = parse_frontmatter(&raw_content)?;
        let title = fm.title.clone().unwrap_or_else(|| {
            path.file_stem().and_then(|s| s.to_str()).unwrap_or("Untitled").to_string()
        });
        let spec_type = fm.spec_type.clone().or_else(|| infer_spec_type_from_title(&title)).unwrap_or_else(|| {
            resolve_canonical_spec_tier(&rel_path, None, &title).1.to_string()
        });

        let mut issue_num = fm.github_issue.as_deref().and_then(extract_issue_number);
        let mut had_placeholder_issue =
            fm.github_issue.as_deref().is_none_or(is_placeholder_issue_id);
        let mut had_invalid_remote_issue = false;
        if !options.offline {
            if let Some(num) = issue_num {
                if !index.is_issue_valid_for_spec(num, &title, Some(&spec_type)) {
                    issue_num = None;
                    had_placeholder_issue = true;
                    had_invalid_remote_issue = true;
                }
            }
            if issue_num.is_none() {
                issue_num = index.find_issue_for_spec(&title, Some(&spec_type));
            }
        }

        let (mut current_content, purged_count) = clean_unpopulated_placeholders_counted(&raw_content);
        summary.placeholders_purged += purged_count;

        if let Some(num) = issue_num {
            let expected_link = format_issue_link(num, effective_repo);
            let current_raw = fm.github_issue.as_deref().unwrap_or("").trim();
            if had_placeholder_issue || (effective_repo.is_some() && !current_raw.contains("http")) {
                let updated = update_frontmatter_field(&current_content, "GitHub Issue", &expected_link)?;
                if updated != current_content {
                    current_content = updated;
                    summary.frontmatters_updated += 1;
                }
            }
        } else if had_invalid_remote_issue {
            let updated = update_frontmatter_field(&current_content, "GitHub Issue", "TBD")?;
            if updated != current_content {
                current_content = updated;
                summary.frontmatters_updated += 1;
            }
        }

        if current_content != raw_content && !options.dry_run {
            write_atomic(path, &current_content)?;
        }

        discovered_specs.push(DiscoveredSpec {
            path: path.clone(),
            rel_path,
            spec_type,
            document_id: fm.document_id,
            title,
            issue_number: issue_num,
            is_resolved: issue_num.and_then(|n| index.is_issue_resolved(n)).unwrap_or(false),
            parent_epic: fm.parent_epic,
            parent_feature: fm.parent_feature,
        });
    }

    Ok(discovered_specs)
}

/// Executes Phase 5 checklist tick/untick reconciliation, reference backpatching, and remote tracker body sync.
/// Realises: [REQ-RECONCILE-BACKLOG-LIB-RUN-RECONCILIATION]
fn reconcile_checklists_and_sync_remote(
    discovered_specs: &[DiscoveredSpec],
    options: &ReconcileOptions,
    (effective_repo, local_repo, effective_branch, upstream_prefix): (
        Option<&str>,
        Option<&str>,
        &str,
        Option<&str>,
    ),
    (index, tracker): (&IssueIndex, &dyn IssueTracker),
    summary: &mut ReconcileSummary,
) -> Result<(), ReconcileError> {
    let path_to_issue = build_spec_path_to_issue_map(discovered_specs);

    for spec in discovered_specs {
        let raw_content = fs::read_to_string(&spec.path)?;
        let (reconciled, chk_changes) =
            reconcile_checklists_in_content(&raw_content, index, Some(&spec.spec_type))?;
        let (backpatched, bp_changes) =
            backpatch_spec_references_in_content(&reconciled, &path_to_issue, local_repo);

        if chk_changes > 0 || bp_changes > 0 {
            summary.checklists_updated += chk_changes + bp_changes;
            if backpatched != raw_content && !options.dry_run {
                write_atomic(&spec.path, &backpatched)?;
            }
        }

        if !options.offline && options.sync_bodies {
            if let Some(issue_num) = spec.issue_number {
                if issue_num > 0
                    && index.is_issue_valid_for_spec(issue_num, &spec.title, Some(&spec.spec_type))
                {
                    let projected = project_body_for_tracker_with_prefix(
                        &backpatched,
                        &spec.rel_path,
                        effective_repo,
                        effective_branch,
                        upstream_prefix,
                    );
                    let remote_issue = index.get_issue(issue_num);
                    let needs_body_sync =
                        remote_issue.is_none_or(|r| r.body.trim() != projected.trim());
                    let needs_title_sync =
                        remote_issue.is_some_and(|r| r.title.trim() != spec.title.trim());
                    if needs_body_sync && !options.dry_run {
                        tracker.edit_issue_body(issue_num, &projected, effective_repo)?;
                    }
                    if needs_title_sync && !options.dry_run {
                        tracker.edit_issue_title(issue_num, spec.title.trim(), effective_repo)?;
                    }
                    if needs_body_sync || needs_title_sync {
                        summary.remote_issues_synced += 1;
                    }
                }
            }
        }
    }

    Ok(())
}

/// [DOC-TIER: A] Executes the full 5-phase workspace backlog reconciliation workflow.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-RUN-RECONCILIATION]
/// Contract: Synchronizes frontmatter IDs, bottom-up registrations, parent/child checklists, and remote issue bodies.
/// Bounds & Faults: Time O(S^2 + I + L), Space O(S + I + L) over specs S, issues I, and document size L.
pub fn run_reconciliation(
    tracker: &dyn IssueTracker,
    options: &ReconcileOptions,
) -> Result<ReconcileSummary, ReconcileError> {
    let mut summary = ReconcileSummary::default();
    let effective_repo = options.repo.as_deref().and_then(clean_repo_name)
        .or_else(|| detect_git_repo(&options.workspace_root));
    let effective_branch = options.default_branch.clone().filter(|b| !b.trim().is_empty())
        .unwrap_or_else(|| detect_git_branch(&options.workspace_root));
    let upstream_prefix = detect_upstream_prefix(&options.workspace_root);
    let is_upstream = upstream_prefix.is_some()
        || options.workspace_root.to_string_lossy().replace('\\', "/").contains(".pipeline/upstream");
    let local_repo = if is_upstream { None } else { effective_repo.as_deref() };

    let issues = if options.offline {
        Vec::new()
    } else {
        tracker.fetch_all_issues(effective_repo.as_deref())?
    };
    let mut index = IssueIndex::from_issues(issues);
    let spec_files = collect_spec_files(&options.workspace_root)?;
    let mut discovered_specs = discover_and_reconcile_frontmatters(
        &spec_files, options, local_repo, &index, &mut summary,
    )?;

    if !options.offline && !options.dry_run {
        let repo_str = effective_repo.as_deref().unwrap_or("");
        let registered = register_unallocated_specs_bottom_up(
            &options.workspace_root, &mut discovered_specs, &mut index, tracker,
            (repo_str, &effective_branch, upstream_prefix.as_deref()),
        )?;
        summary.issues_registered += registered;
        summary.frontmatters_updated += registered;
    }

    let hier_changes = synthesize_and_reconcile_hierarchy(
        &discovered_specs, local_repo, &effective_branch, options.dry_run,
    )?;
    summary.checklists_updated += hier_changes;

    reconcile_checklists_and_sync_remote(
        &discovered_specs, options,
        (effective_repo.as_deref(), local_repo, &effective_branch, upstream_prefix.as_deref()),
        (&index, tracker), &mut summary,
    )?;
    Ok(summary)
}

#[cfg(test)]
mod checklist_hierarchy_tests;
#[cfg(test)]
mod checklist_tests;
#[cfg(test)]
mod create_issue_tests;
#[cfg(test)]
mod frontmatter_tests;
#[cfg(test)]
mod lib_tests;
#[cfg(test)]
mod registration_tests;
#[cfg(test)]
mod tracker_tests;
