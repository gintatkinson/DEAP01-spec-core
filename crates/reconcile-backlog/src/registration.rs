//! Bottom-up issue registration, cross-reference backpatching, and checklist publish-gate validation.
//! Realises: [REQ-RECONCILE-BACKLOG-CORE, Issue Tracker as Canonical SSOT, Zero Fallback Placeholders, Automated Spec Reconciliation]

use crate::checklist::{
    clean_unpopulated_placeholders_counted, contains_unpopulated_placeholders,
    extract_checklist_title_and_issue, replace_stale_issue_in_checklist_line,
    synthesize_and_reconcile_hierarchy, DiscoveredSpec,
};
use crate::error::{write_atomic, ReconcileError};
use crate::frontmatter::{
    clean_markdown_key, collect_code_block_ranges, extract_issue_number, first_h2_byte_offset,
    format_issue_link, is_offset_in_ranges, update_frontmatter_field,
};
use crate::projection::project_body_for_tracker_with_prefix;
use crate::tracker::{
    canonicalize_spec_type, infer_spec_type_from_title, Issue, IssueIndex, IssueTracker,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// [DOC-TIER: A] Resolves `(tier_order, canonical_type, tracker_label)` for bottom-up registration.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-RESOLVE-CANONICAL-SPEC-TIER]
/// Contract: Maps Use Case -> 0, User Story -> 1, Feature -> 2, Epic -> 3 from path, frontmatter, or title.
/// Bounds & Faults: Time O(P + T), Space O(P + T) over path and title lengths.
pub fn resolve_canonical_spec_tier(rel_path: &str, frontmatter_type: Option<&str>, title: &str) -> (usize, &'static str, &'static str) {
    let lower_path = rel_path.replace('\\', "/").to_lowercase();
    if lower_path.contains("use-cases/") || lower_path.contains("use_cases/") {
        return (0, "use_case", "type:use-case");
    }
    if lower_path.contains("user-stories/") || lower_path.contains("user_stories/") {
        return (1, "user_story", "type:user-story");
    }
    if lower_path.contains("features/") {
        return (2, "feature", "type:feature");
    }
    if lower_path.contains("epics/") {
        return (3, "epic", "type:epic");
    }
    let resolved_type = frontmatter_type
        .map(canonicalize_spec_type)
        .filter(|s| !s.is_empty())
        .or_else(|| infer_spec_type_from_title(title))
        .unwrap_or_else(|| "feature".to_string());
    match resolved_type.as_str() {
        "use_case" => (0, "use_case", "type:use-case"),
        "user_story" => (1, "user_story", "type:user-story"),
        "epic" => (3, "epic", "type:epic"),
        _ => (2, "feature", "type:feature"),
    }
}

/// [DOC-TIER: A] Builds a lookup map from spec relative path, filename, and Document ID to allocated issue number.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-BUILD-SPEC-PATH-TO-ISSUE-MAP]
/// Contract: Indexes only specs with `Some(num)` where `num > 0`.
/// Bounds & Faults: Time O(S * K), Space O(S) over discovered specs S.
pub fn build_spec_path_to_issue_map(specs: &[DiscoveredSpec]) -> BTreeMap<String, u64> {
    let mut map = BTreeMap::new();
    for spec in specs {
        if let Some(num) = spec.issue_number.filter(|&n| n > 0) {
            let clean_rel = spec.rel_path.strip_prefix("./").unwrap_or(&spec.rel_path).replace('\\', "/");
            if !clean_rel.is_empty() {
                map.insert(clean_rel.clone(), num);
                map.insert(clean_rel.to_lowercase(), num);
                map.insert(format!("./{}", clean_rel), num);
                if let Some(fname) = Path::new(&clean_rel).file_name().and_then(|n| n.to_str()) {
                    map.insert(fname.to_string(), num);
                    map.insert(fname.to_lowercase(), num);
                }
            }
            if let Some(id_trim) = spec.document_id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                map.insert(id_trim.to_string(), num);
                map.insert(id_trim.to_lowercase(), num);
            }
            let title_trim = spec.title.trim();
            if !title_trim.is_empty() {
                map.insert(title_trim.to_lowercase(), num);
            }
        }
    }
    map
}

/// Backpatches a single frontmatter `Parent Epic` or `Parent Feature` table row if its issue ref is missing or stale.
/// Realises: [REQ-RECONCILE-BACKLOG-LIB-BACKPATCH-SPEC-REFERENCES-IN-CONTENT]
fn try_backpatch_parent_row(
    trimmed: &str,
    path_to_issue: &BTreeMap<String, u64>,
    repo: Option<&str>,
) -> Option<String> {
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return None;
    }
    let inner = trimmed.get(1..trimmed.len().saturating_sub(1)).unwrap_or("");
    let cells: Vec<&str> = inner.split('|').collect();
    let (Some(&first_cell), Some(&second_cell)) = (cells.first(), cells.get(1)) else {
        return None;
    };
    let key = clean_markdown_key(first_cell);
    if !matches!(key.as_str(), "parent epic" | "epic" | "parent feature" | "feature") {
        return None;
    }
    let val_cell = second_cell.trim();
    let val_lower = val_cell.to_lowercase();
    let issue_num = path_to_issue
        .iter()
        .filter(|(k, _)| k.len() >= 4 && val_lower.contains(&k.to_lowercase()))
        .max_by_key(|(k, _)| k.len())
        .map(|(_, &num)| num)?;
    if let Some(old_num) = extract_issue_number(val_cell) {
        return replace_stale_issue_in_checklist_line(trimmed, old_num, issue_num);
    }
    let issue_link = format_issue_link(issue_num, repo);
    let new_val = if val_cell.is_empty() || val_cell.eq_ignore_ascii_case("pending") {
        issue_link
    } else {
        format!("{} ({})", val_cell, issue_link)
    };
    Some(format!("| {} | {} |", first_cell.trim(), new_val))
}

/// Backpatches a single checklist line with an allocated issue link if it matches `path_to_issue`.
/// Realises: [REQ-RECONCILE-BACKLOG-LIB-BACKPATCH-SPEC-REFERENCES-IN-CONTENT]
fn try_backpatch_checklist_line(
    line: &str,
    path_to_issue: &BTreeMap<String, u64>,
    repo: Option<&str>,
) -> Option<String> {
    let (title, existing_issue) = extract_checklist_title_and_issue(line)?;
    let line_lower = line.to_lowercase();
    let title_lower = title.to_lowercase();
    let matched_num = path_to_issue
        .iter()
        .filter(|(k, _)| {
            k.len() >= 4 && (line_lower.contains(&k.to_lowercase()) || title_lower.starts_with(&k.to_lowercase()))
        })
        .max_by_key(|(k, _)| k.len())
        .map(|(_, &num)| num)?;
    if let Some(old_num) = existing_issue {
        return replace_stale_issue_in_checklist_line(line, old_num, matched_num);
    }

    let trimmed_start = line.trim_start();
    let indent = line.get(..line.len().saturating_sub(trimmed_start.len())).unwrap_or("");
    let prefix = trimmed_start.get(..6).unwrap_or("- [ ] ");
    if line.contains("](../") || line.contains("](./") {
        let rest = trimmed_start.get(6..).unwrap_or("").trim_start();
        return Some(format!("{}{}#{} - {}", indent, prefix, matched_num, rest));
    }
    let issue_target = match repo.map(str::trim).filter(|r| !r.is_empty()) {
        Some(r) => format!("https://github.com/{}/issues/{}", r, matched_num),
        None => format!("#{}", matched_num),
    };
    Some(format!("{}{} [{}]({})", indent, prefix, title, issue_target))
}

/// [DOC-TIER: A] Backpatches parent frontmatter references and child checklist items with newly allocated issue IDs.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-BACKPATCH-SPEC-REFERENCES-IN-CONTENT]
/// Contract: Updates unlinked parent rows and checklist items outside `pulldown-cmark` code blocks.
/// Bounds & Faults: Time O(L * M), Space O(N) where L is line count and M is `path_to_issue.len()`.
pub fn backpatch_spec_references_in_content(
    content: &str,
    path_to_issue: &BTreeMap<String, u64>,
    repo: Option<&str>,
) -> (String, usize) {
    if path_to_issue.is_empty() {
        return (content.to_string(), 0);
    }

    let line_ending = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let h2_offset = first_h2_byte_offset(content);
    let code_ranges = collect_code_block_ranges(content);
    let mut in_frontmatter_region = true;
    let mut changes = 0usize;
    let mut out_lines: Vec<String> = Vec::new();
    let mut byte_offset = 0usize;

    for raw_line in content.split('\n') {
        let line_len = raw_line.len();
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let trimmed = line.trim();
        let in_code = is_offset_in_ranges(byte_offset, &code_ranges);

        if !in_code && byte_offset >= h2_offset && trimmed.starts_with("## ") {
            in_frontmatter_region = false;
        }

        if !in_code && in_frontmatter_region {
            if let Some(updated_row) = try_backpatch_parent_row(trimmed, path_to_issue, repo) {
                out_lines.push(updated_row);
                changes += 1;
                byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
                continue;
            }
        } else if !in_code {
            if let Some(updated_item) = try_backpatch_checklist_line(line, path_to_issue, repo) {
                out_lines.push(updated_item);
                changes += 1;
                byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
                continue;
            }
        }

        out_lines.push(line.to_string());
        byte_offset = byte_offset.saturating_add(line_len).saturating_add(1);
    }

    (out_lines.join(line_ending), changes)
}

/// Bundled tracker and repository context for bottom-up registration.
/// Realises: [REQ-RECONCILE-BACKLOG-LIB-REGISTER-UNALLOCATED-SPECS-BOTTOM-UP]
pub(crate) struct RegistrationContext<'a> {
    pub index: &'a mut IssueIndex,
    pub tracker: &'a dyn IssueTracker,
    pub repo: &'a str,
    pub branch: &'a str,
    pub upstream_prefix: Option<&'a str>,
}

/// Registers or links a single specification at `spec_idx` within `discovered`.
/// Realises: [REQ-RECONCILE-BACKLOG-LIB-REGISTER-UNALLOCATED-SPECS-BOTTOM-UP]
fn register_single_spec(
    spec: &mut DiscoveredSpec,
    canonical_type: &str,
    label: &str,
    path_to_issue: &BTreeMap<String, u64>,
    ctx: &mut RegistrationContext<'_>,
) -> Result<bool, ReconcileError> {
    let repo_opt = Some(ctx.repo.trim()).filter(|r| !r.is_empty());
    let local_repo = if ctx.upstream_prefix.is_some() { None } else { repo_opt };
    let raw_content = fs::read_to_string(&spec.path)?;
    let (cleaned, purged) = clean_unpopulated_placeholders_counted(&raw_content);
    let (mut current_content, bp_changes) =
        backpatch_spec_references_in_content(&cleaned, path_to_issue, local_repo);
    let mut modified_on_disk = purged > 0 || bp_changes > 0;

    if spec.issue_number.is_none() {
        if let Some(existing_num) = ctx.index.find_issue_for_spec(&spec.title, Some(canonical_type))
        {
            spec.issue_number = Some(existing_num);
            spec.is_resolved = ctx.index.is_issue_resolved(existing_num).unwrap_or(false);
            let issue_val = format_issue_link(existing_num, local_repo);
            current_content = update_frontmatter_field(&current_content, "GitHub Issue", &issue_val)?;
            modified_on_disk = true;
        } else {
            if contains_unpopulated_placeholders(&current_content) {
                return Err(ReconcileError::PreconditionFailed(format!(
                    "Specification '{}' contains unpopulated placeholders before issue registration",
                    spec.path.display()
                )));
            }
            let projected_body = project_body_for_tracker_with_prefix(
                &current_content, &spec.rel_path, repo_opt, ctx.branch, ctx.upstream_prefix,
            );
            let created_num = ctx.tracker.create_issue(
                &spec.title, &projected_body, &[label.to_string()], repo_opt,
            )?;
            if created_num > 0 {
                spec.issue_number = Some(created_num);
                spec.is_resolved = false;
                let issue_val = format_issue_link(created_num, local_repo);
                current_content = update_frontmatter_field(&current_content, "GitHub Issue", &issue_val)?;
                write_atomic(&spec.path, &current_content)?;
                ctx.index.insert_issue(Issue {
                    number: created_num, title: spec.title.clone(), state: "OPEN".to_string(),
                    labels: vec![label.to_string()], body: projected_body,
                });
                return Ok(true);
            }
        }
    }

    if modified_on_disk {
        write_atomic(&spec.path, &current_content)?;
    }
    Ok(false)
}

/// Executes bottom-up registration across Tiers `0..4` (`Use Case -> User Story -> Feature -> Epic`).
/// Realises: [REQ-RECONCILE-BACKLOG-LIB-REGISTER-UNALLOCATED-SPECS-BOTTOM-UP]
pub(crate) fn register_specs_bottom_up(
    _dir: &Path,
    discovered: &mut [DiscoveredSpec],
    ctx: &mut RegistrationContext<'_>,
) -> Result<usize, ReconcileError> {
    let repo_opt = Some(ctx.repo.trim()).filter(|r| !r.is_empty());
    let local_repo = if ctx.upstream_prefix.is_some() { None } else { repo_opt };
    let mut created_count = 0usize;

    for tier in 0..4 {
        if tier >= 2 {
            synthesize_and_reconcile_hierarchy(discovered, local_repo, ctx.branch, false)?;
        }
        let path_to_issue = build_spec_path_to_issue_map(discovered);

        for spec in discovered.iter_mut() {
            let (spec_tier, canonical_type, label) = resolve_canonical_spec_tier(
                &spec.rel_path, Some(&spec.spec_type), &spec.title,
            );
            if spec_tier == tier
                && register_single_spec(spec, canonical_type, label, &path_to_issue, ctx)?
            {
                created_count += 1;
            }
        }
    }

    synthesize_and_reconcile_hierarchy(discovered, local_repo, ctx.branch, false)?;
    validate_checklist_publish_integrity(discovered)?;
    Ok(created_count)
}

/// [DOC-TIER: A] Validates that all Epic and Feature child checklists contain only resolved/allocated issue links.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-LIB-VALIDATE-CHECKLIST-PUBLISH-INTEGRITY]
/// Contract: Returns `Err(ReconcileError::PublishGateViolation)` if any child checklist item lacks an issue ID.
/// Bounds & Faults: Time O(S * L), Space O(L) over parent specs S and document lines L.
pub fn validate_checklist_publish_integrity(specs: &[DiscoveredSpec]) -> Result<(), ReconcileError> {
    let child_headers = [
        "features", "child features", "feature breakdown", "linked features", "requirements & checklist",
        "use cases", "system use cases", "linked use cases", "associated use cases",
        "user stories", "child user stories", "stories", "linked user stories", "associated user stories",
    ];

    for spec in specs {
        let st = canonicalize_spec_type(&spec.spec_type);
        if (st != "epic" && st != "feature") || !spec.path.exists() {
            continue;
        }
        let content = fs::read_to_string(&spec.path)?;
        let code_ranges = collect_code_block_ranges(&content);
        let mut in_child_section = false;
        let mut byte_offset = 0usize;

        for (line_idx, raw_line) in content.split('\n').enumerate() {
            let trimmed = raw_line.trim();
            let in_code = is_offset_in_ranges(byte_offset, &code_ranges);
            let hash_count = trimmed.bytes().take_while(|&b| b == b'#').count();
            let is_h2_to_h4 = (2..=4).contains(&hash_count) && trimmed.as_bytes().get(hash_count) == Some(&b' ');

            if !in_code && is_h2_to_h4 {
                let h = trimmed.trim_start_matches('#').trim().to_lowercase();
                in_child_section = child_headers.iter().any(|ch| h.contains(ch));
            } else if !in_code && in_child_section && (trimmed.starts_with("- [") || trimmed.starts_with("* [")) {
                match extract_checklist_title_and_issue(trimmed) {
                    Some((_, Some(_))) => {}
                    Some((title, None)) => {
                        return Err(ReconcileError::PublishGateViolation(format!(
                            "{}:{}: Checklist item '{}' in parent spec is missing a linked GitHub issue number",
                            spec.path.display(), line_idx + 1, title
                        )));
                    }
                    None => {
                        return Err(ReconcileError::PublishGateViolation(format!(
                            "{}:{}: Unpopulated placeholder or malformed checklist item '{}' in parent spec",
                            spec.path.display(), line_idx + 1, trimmed
                        )));
                    }
                }
            }
            byte_offset = byte_offset.saturating_add(raw_line.len()).saturating_add(1);
        }
    }

    Ok(())
}
