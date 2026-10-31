//! Specification Merge, Stale Boilerplate Eviction, and Prefix Collision Cleanup.
//!
//! Safety & Traceability: Realises: [REQ-SYNC-FORWARD-MERGE-EXISTING-SPEC]
//! Contract: Preserves human-authored Markdown specification sections and completed checklist states across non-forced forward synchronization while evicting legacy compiler boilerplate and stale slug-colliding files.
//! Bounds & Faults: Enforces bounds-checked UTF-8 string slicing (`.get`, `.split_once`, `.strip_prefix`), deterministic sorted directory traversal, and zero regexes or runtime panics.

use super::*;

/// Detects legacy hardcoded 5-step compiler-contaminated Use Case boilerplate phrases for eviction during non-forced forward sync.
/// Pure substring predicate over `&str` with zero regexes, heap allocations, or panics.
///
/// Realises: [REQ-SYNC-FORWARD-STALE-UC-BOILERPLATE]
pub(super) fn is_stale_use_case_boilerplate(s: &str) -> bool {
    s.contains("The compiler execution environment is initialized in nominal operational state")
        || s.contains("Authoritative input schema files exist with read access")
        || s.contains("Operational workflow command dispatched for ")
        || s.contains("submits input payload for ")
        || s.contains("validates input schema structure and precondition guards")
        || s.contains("validates structural schema and boundary preconditions")
        || s.contains("executes core domain transformation and state transition logic")
        || s.contains("executes core operational transformation")
        || s.contains("Downstream resource timeout or communication fault")
        || s.contains(
            "All semantic models and generated artifacts satisfy formal invariant requirements",
        )
        || s.contains("Output artifacts are deterministically emitted with bitwise reproducibility")
}

/// Checks whether a generated `UC-*.md` section body matches the default undocumented `UseCaseDef` fallback signature for `heading`.
/// Allows human-authored non-boilerplate `UC-*.md` prose to survive `!opts.force` when the AST carries no explicit override.
///
/// Realises: [REQ-SYNC-FORWARD-UNDOC-UC-FALLBACK-DETECT]
pub(super) fn is_undocumented_use_case_fallback_section(heading: &str, body: &str) -> bool {
    match heading {
        "## 2. Preconditions" => {
            body.contains("is initialized in its nominal operational state for `")
                && body.contains("holds authorized operational access to `")
        }
        "## 3. Trigger" => body.contains("` initiates `") && body.contains("` request on `"),
        "## 4. Main Success Scenario" => {
            body.contains("verifies declared preconditions and structural boundary guards for `")
                && body.contains("behavioral transformation while preserving subsystem invariants.")
        }
        "## 5. Alternate and Exception Flows" => {
            body.contains("- Precondition guard failure in `")
                && body.contains("- Invariant or constraint violation during `")
        }
        "## 6. Postconditions" => {
            body.contains("with all declared subsystem state invariants satisfied.")
                && body.contains("` outcome is recorded for `")
        }
        _ => false,
    }
}

/// Detects legacy hardcoded compiler-boilerplate User Story text for eviction during forward sync.
/// Pure substring predicate over `&str` returning `false` on authored or AST-derived text.
///
/// Realises: [REQ-SYNC-FORWARD-STALE-US-BOILERPLATE]
pub(super) fn is_stale_user_story_boilerplate(s: &str) -> bool {
    s.contains("processes the AST within defined latency envelope")
        || s.contains("When semantic validator executes")
        || s.contains("graceful recovery occurs without data loss")
}

/// Detects legacy hardcoded compiler-boilerplate Epic section text for eviction during non-forced forward sync.
/// Pure substring predicate over `EPIC-*.md` section bodies across Sections 1, 3, 4, and 5.
///
/// Realises: [REQ-SYNC-FORWARD-STALE-EPIC-BOILERPLATE]
pub(super) fn is_stale_epic_boilerplate(s: &str) -> bool {
    s.contains("Deterministic compilation passes, error containment")
        || s.contains("provable polynomial complexity execution")
        || s.contains("zero hardcoded domain semantics")
        || s.contains("Safety-critical invariant satisfaction, formal trace matrix closure")
        || s.contains(
            "Subsystem structural composition, port allocations, and directional data connectors realized by ",
        )
        || s.contains(
            "establishing architectural layout, mathematical invariants, and verification criteria.",
        )
}

/// Detects whether an `EPIC-*.md` section body matches the compiler's empty-AST fallback template for `heading`.
/// Enables automatic non-forced upgrade when AST members are later populated without clobbering authored prose.
///
/// Realises: [REQ-SYNC-FORWARD-AUTO-EPIC-FALLBACK]
pub(super) fn is_auto_epic_fallback_section(heading: &str, body: &str) -> bool {
    match heading {
        "## 1. Context" => body.contains(
            "defines the top-level structural namespace scope declared in the SysML v2 model.",
        ),
        "## 3. Architecture" => {
            body.contains("serves as a structural namespace container with no direct")
        }
        "## 4. Operational Considerations" => {
            body.contains("declares structural definitions with no direct")
        }
        "## 5. Security & Governance" => body.contains(
            "declares no package-scoped `RequirementDef` or `ConstraintDef` governance invariants in schema.",
        ),
        _ => false,
    }
}

/// Detects legacy hardcoded CSS/UI/compiler Feature boilerplate text for eviction during non-forced forward sync.
/// Pure substring predicate over `FEAT-*.md` section bodies and checklist items.
///
/// Realises: [REQ-SYNC-FORWARD-STALE-FEAT-BOILERPLATE]
pub(super) fn is_stale_feature_boilerplate(s: &str) -> bool {
    s.contains("@container")
        || s.contains("viewport containment")
        || s.contains("platform design tokens")
        || s.contains("Formal constraints and invariant guarantees enforced by ")
        || s.contains("AST references")
        || s.contains("Deterministic state machine solver and safety monitor logic for ")
        || s.contains("Presentation interface binding and diagnostic event routing for ")
        || s.contains("operational_invariants")
}

/// Detects whether a `FEAT-*.md` section body matches the compiler's empty-AST fallback template for `heading`.
/// Enables automatic non-forced upgrade once AST attributes, ports, constraints, or operations are added.
///
/// Realises: [REQ-SYNC-FORWARD-AUTO-FEAT-FALLBACK]
pub(super) fn is_auto_feature_fallback_section(heading: &str, body: &str) -> bool {
    match heading {
        "### 2. Validation & Constraints" => {
            body.contains("declares no additional constraints specified in schema.")
        }
        "### 3. Visual Layout & Arrangement" => body.contains(
            "no nested subparts declared in schema paired with no external boundary ports declared in schema.",
        ),
        "### 4. Interactive Flow & States" => {
            body.trim()
                .starts_with("Operational lifecycle events and discrete state transitions for ")
                && !body.contains("\n- ")
        }
        "### Layer 1: Domain State & Signal Model" => {
            body.contains("with no additional value attributes declared in schema.")
        }
        "### Layer 2: Logic & Safety State Management" => body.contains(
            "governs stateless structural transition evaluation enforcing no additional invariant guards declared in schema.",
        ),
        "### Layer 3: Presentation & Actuator Interface Binding" => {
            body.contains("with no external ports, operations, or actions declared in schema.")
        }
        _ => false,
    }
}

/// Extracts the leading 2-dash specification ID prefix (such as `EPIC-02-` or `UC-01-`) from `file_name`.
/// Uses bounds-checked `.split_once('-')` and `.get(..)` slicing, returning `None` if fewer than two hyphens exist.
///
/// Realises: [REQ-SYNC-FORWARD-EXTRACT-ID-PREFIX]
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
/// Returns `None` without panicking when neither exact nor prefix-matching `.md` files exist in the target subdirectory.
///
/// Realises: [REQ-SYNC-FORWARD-READ-EXISTING-SPEC]
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
/// Maps `AC-*` items to their AC identifier and linked `FEAT-*`/`UC-*`/`US-*` items to their 2-dash prefix.
///
/// Realises: [REQ-SYNC-FORWARD-EXTRACT-CHECKLIST-KEY]
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

/// Extracts the trimmed body of a Markdown section under `heading` while ignoring headings inside fenced code blocks.
/// Returns `None` when `heading` is absent or its extracted body is empty.
///
/// Realises: [REQ-SYNC-FORWARD-EXTRACT-MARKDOWN-SECTION]
pub(super) fn extract_markdown_section_body(doc: &str, heading: &str) -> Option<String> {
    let target_level = heading.chars().take_while(|c| *c == '#').count();
    let mut in_fence = false;
    let mut collecting = false;
    let mut body_lines = Vec::new();

    for line in doc.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
        }
        if !in_fence {
            if !collecting {
                if trimmed == heading {
                    collecting = true;
                    continue;
                }
            } else if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|c| *c == '#').count();
                if level > 0
                    && level <= target_level
                    && trimmed.get(level..level.saturating_add(1)) == Some(" ")
                {
                    break;
                }
            }
        }
        if collecting {
            body_lines.push(line);
        }
    }

    let joined = body_lines.join("\n");
    let trimmed_body = joined.trim();
    if collecting && !trimmed_body.is_empty() {
        Some(trimmed_body.to_string())
    } else {
        None
    }
}

/// Replaces the body of a Markdown section identified by `heading` with `new_body` while respecting code fences.
/// Preserves surrounding sections and trailing newline invariants, returning `doc` unchanged if `heading` is absent.
///
/// Realises: [REQ-SYNC-FORWARD-REPLACE-MARKDOWN-SECTION]
pub(super) fn replace_markdown_section_body(doc: &str, heading: &str, new_body: &str) -> String {
    let target_level = heading.chars().take_while(|c| *c == '#').count();
    let mut in_fence = false;
    let mut skipping = false;
    let mut out_lines: Vec<String> = Vec::new();

    for line in doc.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
        }
        if !in_fence {
            if !skipping {
                if trimmed == heading {
                    out_lines.push(line.to_string());
                    out_lines.push(new_body.trim().to_string());
                    out_lines.push(String::new());
                    skipping = true;
                    continue;
                }
            } else if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|c| *c == '#').count();
                if level > 0
                    && level <= target_level
                    && trimmed.get(level..level.saturating_add(1)) == Some(" ")
                {
                    skipping = false;
                }
            }
        }
        if !skipping {
            out_lines.push(line.to_string());
        }
    }

    let mut result = out_lines.join("\n");
    if doc.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// Merges authored prose sections and `- [x]` checkbox completion states from `existing` into `generated` when `!opts.force`.
/// Evicts retired compiler boilerplate and auto-fallback templates so AST updates take primacy without clobbering authored prose.
///
/// Realises: [REQ-SYNC-FORWARD-MERGE-EXISTING-SPEC]
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
    let mut checked_keys: BTreeSet<String> = BTreeSet::new();
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

    let mut merged = if checked_keys.is_empty() {
        generated.to_string()
    } else {
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
    };

    if is_epic {
        let epic_headings = [
            "## 1. Context",
            "## 3. Architecture",
            "## 4. Operational Considerations",
            "## 5. Security & Governance",
        ];
        for heading in &epic_headings {
            if let Some(existing_body) = extract_markdown_section_body(existing, heading) {
                if is_stale_epic_boilerplate(&existing_body)
                    || is_auto_epic_fallback_section(heading, &existing_body)
                {
                    continue;
                }
                merged = replace_markdown_section_body(&merged, heading, &existing_body);
            }
        }
    } else if is_feature {
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
                if is_stale_feature_boilerplate(&existing_body)
                    || is_auto_feature_fallback_section(heading, &existing_body)
                {
                    continue;
                }
                merged = replace_markdown_section_body(&merged, heading, &existing_body);
            }
        }
    } else if is_user_story && !is_stale_user_story_boilerplate(existing) {
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
                if is_stale_use_case_boilerplate(&existing_body) {
                    continue;
                }
                let gen_is_undocumented_fallback = extract_markdown_section_body(&merged, heading)
                    .is_some_and(|gen_body| {
                        is_undocumented_use_case_fallback_section(heading, &gen_body)
                    });
                if gen_is_undocumented_fallback {
                    merged = replace_markdown_section_body(&merged, heading, &existing_body);
                }
            }
        }
    }

    merged
}

/// Removes obsolete `.md` slug variants in `epics/`, `features/`, `user-stories/`, and `use-cases/` that collide with an active `(subdir, ID-prefix)`.
/// Leaves all active specifications and non-colliding files untouched using deterministic sorted directory traversal.
///
/// Realises: [REQ-SYNC-FORWARD-CLEAN-COLLIDING-PREFIX]
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
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let file_path = entry.path();
            if file_path.extension().map(|e| e == "md").unwrap_or(false) {
                if let Ok(rel) = file_path.strip_prefix(target_out_dir) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    if !active_keys.contains(&rel_str) {
                        if let Some(file_name) = file_path.file_name().and_then(|f| f.to_str()) {
                            if let Some(prefix) = extract_id_prefix(file_name) {
                                if active_prefixes
                                    .contains(&((*subdir).to_string(), prefix.to_string()))
                                {
                                    let _ = fs::remove_file(file_path);
                                }
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
/// Invoked only during forced (`opts.force`) non-dry-run forward synchronization.
///
/// Realises: [REQ-SYNC-FORWARD-CLEAN-STALE-SPECIFICATIONS]
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
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `advance_html_comment_state` in `crates/compile-sysml/src/sync/forward/merge_and_cleanup.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Uses bounds-checked `.split_once(..)` slicing and preserves `*in_html_comment == true` when an inline `-->` is followed by an unclosed `<!--` on the same line.
///
/// ## 3. Preconditions
/// - `line_trimmed: &str` is a trimmed line slice and `in_html_comment: &mut bool` tracks the active comment state.
///
/// ## 4. Postconditions
/// - Updates `*in_html_comment` in place and returns `true` when the line should be skipped as a comment line.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(line_trimmed)).
/// - Space Complexity: O(1) zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Bounds-checked `.split_once(..)` prevents slice panics across multi-byte UTF-8 boundaries.
/// - State: Deterministic boolean transition.
/// - Action: Zero recursion or blocking.
/// - Resource: Zero heap allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0604] | SysML v2: ActionDef(advance_html_comment_state)
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
