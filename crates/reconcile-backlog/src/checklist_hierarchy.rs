//! Bottom-up hierarchy synthesis and parent-child checklist injection for Epics, Features, User Stories, and Use Cases.
//!
//! Uses `deap_core::markdown_ast::parse_markdown_ast` and `pulldown-cmark` code block spans
//! to locate H2 sections and inject or update canonical child checklist items deterministically.
//!
//! Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST, Automated Spec Reconciliation]

use crate::checklist::extract_checklist_title_and_issue;
use crate::frontmatter::{collect_code_block_ranges, extract_issue_number, is_offset_in_ranges};
use crate::tracker::{canonicalize_spec_type, normalize_title};
use crate::ReconcileError;
use deap_core::markdown_ast::parse_markdown_ast;
use std::fs;
use std::path::PathBuf;

/// [DOC-TIER: A] Metadata for a discovered specification file participating in hierarchy synthesis.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-DISCOVEREDSPEC]
/// Contract: Holds canonical path, type, title, issue allocation, resolution state, and parent links.
/// Bounds & Faults: Time O(1), Space O(P + T) over path and title strings.
#[derive(Debug, Clone)]
pub struct DiscoveredSpec {
    pub path: PathBuf,
    pub rel_path: String,
    pub spec_type: String,
    pub document_id: Option<String>,
    pub title: String,
    pub issue_number: Option<u64>,
    pub is_resolved: bool,
    pub parent_epic: Option<String>,
    pub parent_feature: Option<String>,
}

/// [DOC-TIER: A] Formats a canonical checklist line linking both the tracker issue and the Git blob specification path.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-FORMAT-CANONICAL-CHECKLIST-ITEM]
/// Contract: Emits `- [x| ] [Title](issue_url) -- [rel_path](blob_url)` when `repo` is provided.
/// Bounds & Faults: Time O(T + P), Space O(T + P) over title and path lengths.
pub fn format_canonical_checklist_item(
    spec: &DiscoveredSpec,
    repo: Option<&str>,
    branch: &str,
) -> String {
    let checkbox = if spec.is_resolved { "[x]" } else { "[ ]" };
    let issue_link = match (spec.issue_number, repo.map(str::trim).filter(|r| !r.is_empty())) {
        (Some(num), Some(r)) => format!("[{}](https://github.com/{}/issues/{})", spec.title, r, num),
        (Some(num), None) => format!("[{}](#{})", spec.title, num),
        (None, _) => spec.title.clone(),
    };

    let spec_link = if !spec.rel_path.is_empty() {
        let clean_rel = spec.rel_path.strip_prefix("./").unwrap_or(&spec.rel_path).replace('\\', "/");
        match repo.map(str::trim).filter(|r| !r.is_empty()) {
            Some(r) => {
                let b = if branch.trim().is_empty() { "main" } else { branch.trim() };
                format!(" -- [{}](https://github.com/{}/blob/{}/{})", clean_rel, r, b, clean_rel)
            }
            None => format!(" -- [{}]({})", clean_rel, clean_rel),
        }
    } else {
        String::new()
    };

    format!("- {} {}{}", checkbox, issue_link, spec_link)
}

/// Checks if `parent_ref` matches a candidate parent `DiscoveredSpec` by issue ID, document ID, path, or title.
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-SPEC-MATCHES-EPIC]
fn parent_ref_matches_spec(parent_ref: &str, parent: &DiscoveredSpec) -> bool {
    let ref_lower = parent_ref.to_lowercase();
    if let (Some(ref_num), Some(p_num)) = (extract_issue_number(parent_ref), parent.issue_number) {
        if ref_num == p_num {
            return true;
        }
    }
    if parent.document_id.as_deref().map(str::trim).is_some_and(|id| !id.is_empty() && ref_lower.contains(&id.to_lowercase())) {
        return true;
    }
    if !parent.rel_path.is_empty() {
        if ref_lower.contains(&parent.rel_path.to_lowercase()) {
            return true;
        }
        if PathBuf::from(&parent.rel_path).file_name().and_then(|n| n.to_str()).is_some_and(|f| ref_lower.contains(&f.to_lowercase())) {
            return true;
        }
    }
    if let (Some(ord_ref), Some(ord_parent)) = (
        crate::tracker::extract_spec_ordinal(parent_ref),
        crate::tracker::extract_spec_ordinal(&parent.title),
    ) {
        if ord_ref == ord_parent {
            return true;
        }
    }
    let (norm_parent, norm_ref) = (normalize_title(&parent.title), normalize_title(parent_ref));
    !norm_parent.is_empty() && !norm_ref.is_empty() && (norm_ref.contains(&norm_parent) || norm_parent.contains(&norm_ref))
}

/// Checks if `child` declares `epic` as its parent Epic.
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-SPEC-MATCHES-EPIC]
pub(crate) fn spec_matches_epic(child: &DiscoveredSpec, epic: &DiscoveredSpec) -> bool {
    child.parent_epic.as_deref().is_some_and(|r| parent_ref_matches_spec(r, epic))
}

/// Checks if `child` declares `feat` as its parent Feature.
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-SPEC-MATCHES-FEATURE]
pub(crate) fn spec_matches_feature(child: &DiscoveredSpec, feat: &DiscoveredSpec) -> bool {
    child.parent_feature.as_deref().is_some_and(|r| parent_ref_matches_spec(r, feat))
}

/// Formatting and section configuration for `reconcile_or_inject_section`.
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-RECONCILE-OR-INJECT-SECTION]
struct HierarchySectionSpec<'a> {
    section_headers: &'a [&'a str],
    inject_header: &'a str,
    repo: Option<&'a str>,
    branch: &'a str,
}

/// Finds the `(start_line_idx, end_line_idx)` of the target H2 section outside code blocks.
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-RECONCILE-OR-INJECT-SECTION]
fn find_target_section_bounds(
    content: &str,
    lines: &[&str],
    section_headers: &[&str],
) -> Option<(usize, usize)> {
    let ast = parse_markdown_ast(content);
    if !ast.sections.iter().any(|s| s.level == 2) {
        return None;
    }
    let code_ranges = collect_code_block_ranges(content);
    let mut section_start: Option<usize> = None;
    let mut section_end: usize = lines.len();
    let mut byte_offset = 0usize;

    for (idx, &line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let in_code = is_offset_in_ranges(byte_offset, &code_ranges);
        if !in_code && trimmed.starts_with("## ") {
            let heading_text = trimmed.trim_start_matches('#').trim().to_lowercase();
            if section_start.is_some() {
                section_end = idx;
                break;
            }
            let matches_header = section_headers
                .iter()
                .any(|h| heading_text.contains(&h.to_lowercase()));
            if matches_header {
                section_start = Some(idx);
            }
        }
        byte_offset = byte_offset.saturating_add(line.len()).saturating_add(1);
    }

    section_start.map(|start| (start, section_end))
}

/// Matches a checklist line `(item_title, item_issue)` to the index of a child in `children`.
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-RECONCILE-OR-INJECT-SECTION]
fn find_matching_child_idx(
    item_title: &str,
    item_issue: Option<u64>,
    children: &[&DiscoveredSpec],
) -> Option<usize> {
    let norm_item = normalize_title(item_title);
    for (c_idx, child) in children.iter().enumerate() {
        if let (Some(i_num), Some(c_num)) = (item_issue, child.issue_number) {
            if i_num == c_num {
                return Some(c_idx);
            }
        }
        if !norm_item.is_empty() && normalize_title(&child.title) == norm_item {
            return Some(c_idx);
        }
        if let Some(ref doc_id) = child.document_id {
            let norm_id = normalize_title(doc_id);
            if !norm_id.is_empty()
                && (norm_item == norm_id || norm_item.starts_with(&format!("{} ", norm_id)))
            {
                return Some(c_idx);
            }
        }
    }
    None
}

/// Reconciles checklist items within an existing H2 section range `(sec_start, sec_end)`.
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-RECONCILE-OR-INJECT-SECTION]
fn reconcile_existing_section_lines(
    lines: &[&str],
    bounds: (usize, usize),
    spec: &HierarchySectionSpec<'_>,
    children: &[&DiscoveredSpec],
) -> (Vec<String>, usize) {
    let (sec_start, sec_end) = bounds;
    let mut matched_children = vec![false; children.len()];
    let mut section_lines: Vec<String> = Vec::new();
    let mut last_checklist_rel_idx: Option<usize> = None;
    let mut changes = 0usize;

    for &line in lines.get(sec_start + 1..sec_end).unwrap_or_default() {
        if let Some((item_title, item_issue)) = extract_checklist_title_and_issue(line) {
            if let Some((c_idx, child)) = find_matching_child_idx(&item_title, item_issue, children)
                .and_then(|i| children.get(i).map(|c| (i, *c)))
            {
                if let Some(m) = matched_children.get_mut(c_idx) {
                    *m = true;
                }
                let canonical = format_canonical_checklist_item(child, spec.repo, spec.branch);
                if line.trim() != canonical {
                    changes += 1;
                }
                last_checklist_rel_idx = Some(section_lines.len());
                section_lines.push(canonical);
                continue;
            }
            last_checklist_rel_idx = Some(section_lines.len());
        }
        section_lines.push(line.to_string());
    }

    let missing: Vec<String> = children
        .iter()
        .zip(&matched_children)
        .filter(|(_, matched)| !**matched)
        .map(|(c, _)| format_canonical_checklist_item(c, spec.repo, spec.branch))
        .collect();

    if !missing.is_empty() {
        changes += missing.len();
        let mut insert_pos = match last_checklist_rel_idx {
            Some(idx) => idx + 1,
            None => {
                let mut pos = section_lines.len();
                while pos > 0 && section_lines.get(pos - 1).is_some_and(|l| l.trim().is_empty()) {
                    pos -= 1;
                }
                pos
            }
        };
        for item in missing {
            section_lines.insert(insert_pos, item);
            insert_pos += 1;
        }
    }

    (section_lines, changes)
}

/// Reconciles an existing H2 section or injects a new one with canonical child checklist items.
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-RECONCILE-OR-INJECT-SECTION]
fn reconcile_or_inject_section(
    content: &str,
    spec: &HierarchySectionSpec<'_>,
    children: &[&DiscoveredSpec],
) -> (String, usize) {
    if children.is_empty() {
        return (content.to_string(), 0);
    }

    let line_ending = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let lines: Vec<&str> = content
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();

    if let Some((sec_start, sec_end)) = find_target_section_bounds(content, &lines, spec.section_headers) {
        let (section_lines, changes) =
            reconcile_existing_section_lines(&lines, (sec_start, sec_end), spec, children);
        if changes == 0 {
            return (content.to_string(), 0);
        }
        let head = lines.get(..=sec_start).unwrap_or_default();
        let tail = lines.get(sec_end..).unwrap_or_default();
        let mut out_lines: Vec<String> = head.iter().map(|s| (*s).to_string()).collect();
        out_lines.extend(section_lines);
        out_lines.extend(tail.iter().map(|s| (*s).to_string()));
        (out_lines.join(line_ending), changes)
    } else {
        let mut out_lines: Vec<String> = lines.iter().map(|s| (*s).to_string()).collect();
        while out_lines.last().is_some_and(|l| l.trim().is_empty()) {
            out_lines.pop();
        }
        out_lines.push(String::new());
        out_lines.push(spec.inject_header.to_string());
        out_lines.push(String::new());
        for child in children {
            out_lines.push(format_canonical_checklist_item(child, spec.repo, spec.branch));
        }
        out_lines.push(String::new());
        (out_lines.join(line_ending), children.len())
    }
}

/// Reconciles and persists child checklist sections for a single parent specification file.
/// Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-SYNTHESIZE-AND-RECONCILE-HIERARCHY]
fn reconcile_parent_spec_sections(
    parent: &DiscoveredSpec,
    sections: &[(&HierarchySectionSpec<'_>, Vec<&DiscoveredSpec>)],
    dry_run: bool,
) -> Result<usize, ReconcileError> {
    if sections.iter().all(|(_, children)| children.is_empty()) {
        return Ok(0);
    }
    let original = fs::read_to_string(&parent.path)?;
    let (cleaned, purged) = crate::checklist::clean_unpopulated_placeholders_counted(&original);
    let mut current = cleaned;
    let mut changes = purged;

    for (sec_spec, children) in sections {
        let (updated, c) = reconcile_or_inject_section(&current, sec_spec, children);
        current = updated;
        changes += c;
    }

    if changes > 0 && current != original {
        if !dry_run {
            crate::write_atomic(&parent.path, &current)?;
        }
        return Ok(changes);
    }
    Ok(0)
}

/// [DOC-TIER: A] Synthesizes and reconciles parent-child checklists across Epics, Features, User Stories, and Use Cases.
/// Safety & Traceability: Realises: [REQ-RECONCILE-BACKLOG-CHECKLIST-SYNTHESIZE-AND-RECONCILE-HIERARCHY]
/// Contract: Updates Epic (`## Features`, `## Use Cases`) and Feature (`## User Stories`) checklists bottom-up.
/// Bounds & Faults: Time O(S^2 + L), Space O(L) where S is spec count and L is document length.
pub fn synthesize_and_reconcile_hierarchy(
    specs: &[DiscoveredSpec],
    repo: Option<&str>,
    branch: &str,
    dry_run: bool,
) -> Result<usize, ReconcileError> {
    let by_type = |t: &str| -> Vec<&DiscoveredSpec> {
        specs.iter().filter(|s| canonicalize_spec_type(&s.spec_type) == t).collect()
    };
    let epics = by_type("epic");
    let features = by_type("feature");
    let stories = by_type("user_story");
    let use_cases = by_type("use_case");

    let feat_sec = HierarchySectionSpec {
        section_headers: &["features", "child features", "feature breakdown", "linked features"],
        inject_header: "## Features",
        repo,
        branch,
    };
    let uc_sec = HierarchySectionSpec {
        section_headers: &["use cases", "system use cases", "linked use cases"],
        inject_header: "## Use Cases",
        repo,
        branch,
    };
    let story_sec = HierarchySectionSpec {
        section_headers: &["user stories", "child user stories", "stories", "linked user stories"],
        inject_header: "## User Stories",
        repo,
        branch,
    };

    let mut total_changes = 0usize;
    for epic in &epics {
        let child_feats: Vec<&DiscoveredSpec> = features.iter().copied().filter(|f| spec_matches_epic(f, epic)).collect();
        let child_ucs: Vec<&DiscoveredSpec> = use_cases.iter().copied().filter(|u| spec_matches_epic(u, epic)).collect();
        total_changes += reconcile_parent_spec_sections(epic, &[(&feat_sec, child_feats), (&uc_sec, child_ucs)], dry_run)?;
    }

    for feat in &features {
        let child_stories: Vec<&DiscoveredSpec> = stories.iter().copied().filter(|s| spec_matches_feature(s, feat)).collect();
        total_changes += reconcile_parent_spec_sections(feat, &[(&story_sec, child_stories)], dry_run)?;
    }

    Ok(total_changes)
}
