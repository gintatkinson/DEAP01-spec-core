#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Specification Merge, Stale Boilerplate Eviction, and Prefix Collision Cleanup.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical
//! software engineering standards. Preserves human-authored Markdown specification prose and
//! completed `- [x]` checklist states across non-forced forward synchronization.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Bounds-Checked UTF-8 Slicing: Uses `.get(..)`, `.split_once(..)`, and `.strip_prefix(..)` with
//!   zero direct byte indexing or regular expressions.
//! - Deterministic Cleanup: Traverses specification directories in sorted filename order.
//!
//! ## 3. Preconditions
//! - `target_out_dir` is the active specification output directory.
//!
//! ## 4. Postconditions
//! - Preserves authored prose and completed checkboxes while evicting stale compiler boilerplate and
//!   obsolete slug-colliding files.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N \log N)$ across specification files and directory entries.
//! - Space Complexity: $O(N)$ for merged specification buffers and active prefix sets.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Missing target files or directories return `None` or `Ok(())` without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0604] [REQ-SYNC-FORWARD-MERGE-EXISTING-SPEC]

use super::*;

/// Extracts the leading 2-dash specification ID prefix (such as `EPIC-02-` or `UC-01-`) from `file_name`.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-EXTRACT-ID-PREFIX]
pub(crate) fn extract_id_prefix(file_name: &str) -> Option<&str> {
    let (first_seg, rest) = file_name.split_once('-')?;
    let (second_seg, _) = rest.split_once('-')?;
    let prefix_len = first_seg
        .len()
        .saturating_add(1)
        .saturating_add(second_seg.len())
        .saturating_add(1);
    file_name.get(..prefix_len)
}

/// Reads existing specification content from disk by exact relative path or deterministic sorted 2-dash ID prefix fallback.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-READ-EXISTING-SPEC]
pub(crate) fn read_existing_spec_content(target_out_dir: &Path, rel_path: &str) -> Option<String> {
    let full = target_out_dir.join(rel_path);
    if let Ok(content) = fs::read_to_string(&full) {
        return Some(content);
    }
    let parent = Path::new(rel_path).parent()?;
    let file_name = Path::new(rel_path).file_name().and_then(|f| f.to_str())?;
    let prefix = extract_id_prefix(file_name)?;
    let dir_path = target_out_dir.join(parent);
    let entries = fs::read_dir(&dir_path).ok()?;
    let mut matching_paths: Vec<PathBuf> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(prefix) && name_str.ends_with(".md") {
                Some(e.path())
            } else {
                None
            }
        })
        .collect();
    matching_paths.sort();
    for path in matching_paths {
        if let Ok(content) = fs::read_to_string(&path) {
            return Some(content);
        }
    }
    None
}

/// Extracts a canonical matching key for a Markdown checklist item following `- [ ]` or `- [x]`.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-EXTRACT-CHECKLIST-KEY]
pub(super) fn extract_checklist_item_key(after_checkbox: &str) -> String {
    let trimmed = after_checkbox.trim();
    if trimmed.starts_with("AC-") {
        if let Some((ac_id, _)) = trimmed.split_once(':') {
            return ac_id.trim().to_string();
        }
    }
    for marker in ["../features/", "../use-cases/", "../user-stories/"] {
        if let Some((_, rest)) = trimmed.split_once(marker) {
            if let Some(prefix) = extract_id_prefix(rest) {
                return prefix.to_string();
            }
        }
    }
    trimmed.to_string()
}

/// Preserves `- [x]` checkbox completion states from `existing` across `generated`.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-MERGE-EXISTING-SPEC]
fn merge_checked_checklist_items(
    generated: &str,
    existing: &str,
    is_user_story: bool,
    is_feature: bool,
) -> String {
    let mut checked_keys: BTreeSet<String> = BTreeSet::new();
    for item in crate::projections::cmark_helpers::parse_task_list_items(existing) {
        if item.checked && !(is_user_story && is_stale_user_story_boilerplate(&item.text)) && !(is_feature && item.text.contains("operational_invariants")) {
            checked_keys.insert(extract_checklist_item_key(&item.text));
        }
    }
    for line in existing.lines() {
        let trimmed = line.trim_start();
        if let Some(after) = trimmed
            .strip_prefix("- [x] ")
            .or_else(|| trimmed.strip_prefix("- [X] "))
        {
            if is_user_story && is_stale_user_story_boilerplate(after) {
                continue;
            }
            if is_feature && after.contains("operational_invariants") {
                continue;
            }
            checked_keys.insert(extract_checklist_item_key(after));
        }
    }
    if checked_keys.is_empty() {
        return generated.to_string();
    }

    let mut lines_out = Vec::new();
    for line in generated.lines() {
        let trimmed = line.trim_start();
        if let Some(after) = trimmed.strip_prefix("- [ ] ") {
            let key = extract_checklist_item_key(after);
            if checked_keys.contains(&key) {
                let indent_len = line.len().saturating_sub(trimmed.len());
                let indent = line.get(..indent_len).unwrap_or("");
                lines_out.push(format!("{}- [x] {}", indent, after));
                continue;
            }
        }
        lines_out.push(line.to_string());
    }
    let mut s = lines_out.join("\n");
    if generated.ends_with('\n') && !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// Merges authored Epic or Feature prose sections from `existing` into `merged`, skipping stale boilerplate and auto-fallbacks.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-MERGE-EXISTING-SPEC]
fn merge_epic_or_feature_sections(mut merged: String, existing: &str, is_epic: bool) -> String {
    if is_epic {
        let epic_headings = [
            "## 1. Context",
            "## 3. Architecture",
            "## 4. Operational Considerations",
            "## 5. Security & Governance",
        ];
        for heading in &epic_headings {
            if let Some(existing_body) = extract_markdown_section_body(existing, heading) {
                if !is_stale_epic_boilerplate(&existing_body)
                    && !is_auto_epic_fallback_section(heading, &existing_body)
                {
                    merged = replace_markdown_section_body(&merged, heading, &existing_body);
                }
            }
        }
    } else {
        let feat_headings = [
            "### 2. Validation & Constraints",
            "### 3. Visual Layout & Arrangement",
            "### 4. Interactive Flow & States",
            "### Layer 1: Domain State & Signal Model",
            "### Layer 2: Logic & Safety State Management",
            "### Layer 3: Presentation & Actuator Interface Binding",
        ];
        for heading in &feat_headings {
            if let Some(existing_body) = extract_markdown_section_body(existing, heading) {
                if !is_stale_feature_boilerplate(&existing_body)
                    && !is_auto_feature_fallback_section(heading, &existing_body)
                {
                    merged = replace_markdown_section_body(&merged, heading, &existing_body);
                }
            }
        }
    }
    merged
}

/// Merges authored prose sections and `- [x]` checkbox completion states from `existing` into `generated` when `!opts.force`.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-MERGE-EXISTING-SPEC]
pub(crate) fn merge_existing_spec_content(
    rel_path: &str,
    generated: &str,
    existing: &str,
) -> String {
    let is_epic = rel_path.starts_with("epics/") || rel_path.contains("/epics/");
    let is_feature = rel_path.starts_with("features/") || rel_path.contains("/features/");
    let is_user_story =
        rel_path.starts_with("user-stories/") || rel_path.contains("/user-stories/");
    let is_use_case = rel_path.starts_with("use-cases/") || rel_path.contains("/use-cases/");

    let mut merged = merge_checked_checklist_items(generated, existing, is_user_story, is_feature);

    if is_epic || is_feature {
        return merge_epic_or_feature_sections(merged, existing, is_epic);
    }
    if is_user_story && !is_stale_user_story_boilerplate(existing) {
        if let Some(existing_story) = extract_markdown_section_body(existing, "## User Story") {
            let has_actor_opening =
                existing_story.contains("As a") || existing_story.contains("As an");
            if !is_stale_user_story_boilerplate(&existing_story)
                && has_actor_opening
                && existing_story.contains("I want")
                && existing_story.contains("so that")
            {
                merged = replace_markdown_section_body(&merged, "## User Story", &existing_story);
            }
        }
    } else if is_use_case && !is_stale_use_case_boilerplate(existing) {
        let uc_headings = [
            "## 2. Preconditions",
            "## 3. Trigger",
            "## 4. Main Success Scenario",
            "## 5. Alternate and Exception Flows",
            "## 6. Postconditions",
        ];
        for heading in &uc_headings {
            if let Some(existing_body) = extract_markdown_section_body(existing, heading) {
                if !is_stale_use_case_boilerplate(&existing_body)
                    && extract_markdown_section_body(&merged, heading).is_some_and(|gen_body| {
                        is_undocumented_use_case_fallback_section(heading, &gen_body)
                    })
                {
                    merged = replace_markdown_section_body(&merged, heading, &existing_body);
                }
            }
        }
    }

    merged
}

/// Removes obsolete `.md` slug variants in `epics/`, `features/`, `user-stories/`, and `use-cases/` that collide with an active `(subdir, ID-prefix)`.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-CLEAN-COLLIDING-PREFIX]
pub(crate) fn clean_colliding_prefix_specifications(
    target_out_dir: &Path,
    active_specs: &BTreeMap<String, String>,
) -> Result<(), String> {
    let subdirs = ["epics", "features", "user-stories", "use-cases"];
    let active_keys: BTreeSet<String> = active_specs.keys().cloned().collect();
    let mut active_prefixes: BTreeSet<(String, String)> = BTreeSet::new();

    for key in &active_keys {
        if let Some((subdir, file_name)) = key.split_once('/') {
            if let Some(prefix) = extract_id_prefix(file_name) {
                active_prefixes.insert((subdir.to_string(), prefix.to_string()));
            }
        }
    }

    for subdir in &subdirs {
        let dir_path = target_out_dir.join(subdir);
        if !dir_path.is_dir() {
            continue;
        }
        for entry in WalkDir::new(&dir_path)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            if file_path.extension().map(|e| e == "md").unwrap_or(false) {
                if let Ok(rel) = file_path.strip_prefix(target_out_dir) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    if !active_keys.contains(&rel_str) {
                        if let Some(prefix) = file_path
                            .file_name()
                            .and_then(|f| f.to_str())
                            .and_then(extract_id_prefix)
                        {
                            if active_prefixes.contains(&((*subdir).to_string(), prefix.to_string()))
                            {
                                let _ = fs::remove_file(file_path);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Removes stale generated `.md` specification files across `epics/`, `features/`, `user-stories/`, and `use-cases/` not present in `active_specs`.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-CLEAN-STALE-SPECIFICATIONS]
pub(crate) fn clean_stale_specifications(
    target_out_dir: &Path,
    active_specs: &BTreeMap<String, String>,
) -> Result<(), String> {
    let subdirs = ["epics", "features", "user-stories", "use-cases"];
    let active_keys: BTreeSet<String> = active_specs.keys().cloned().collect();

    for subdir in &subdirs {
        let dir_path = target_out_dir.join(subdir);
        if !dir_path.is_dir() {
            continue;
        }
        for entry in WalkDir::new(&dir_path)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            if file_path.extension().map(|e| e == "md").unwrap_or(false) {
                if let Ok(rel) = file_path.strip_prefix(target_out_dir) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    if !active_keys.contains(&rel_str) {
                        let _ = fs::remove_file(file_path);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Advances the multi-line HTML comment state machine across a single trimmed line, handling inline `--> ... <!--` re-opens on the same line (`Diagnostic: E0400`).
/// Realises: [REQ-0604]
pub fn advance_html_comment_state(line_trimmed: &str, in_html_comment: &mut bool) -> bool {
    if *in_html_comment {
        if let Some((_, after_close)) = line_trimmed.split_once("-->") {
            if let Some((_, after_reopen)) = after_close.split_once("<!--") {
                *in_html_comment = !after_reopen.contains("-->");
            } else {
                *in_html_comment = false;
            }
        }
        return true;
    }
    if let Some((_, after_open)) = line_trimmed.split_once("<!--") {
        if !after_open.contains("-->") {
            *in_html_comment = true;
        }
        if line_trimmed.starts_with("<!--") {
            return true;
        }
    }
    false
}
