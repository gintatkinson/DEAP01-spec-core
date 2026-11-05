#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! Stale Boilerplate Predicates, Fallback Template Detectors, and Markdown Section Body Helpers.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical
//! software engineering standards. Prevents compiler-contaminated boilerplate from surviving
//! non-forced forward synchronization while preserving human-authored specification sections.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Fence-Aware Section Parsing: `extract_markdown_section_body` and `replace_markdown_section_body`
//!   ignore `#` headings inside fenced ` ``` ` code blocks.
//! - Zero-Regex Substring Matching: All boilerplate and fallback detectors operate via pure substring
//!   predicates over `&str` without regular expressions or panics.
//!
//! ## 3. Preconditions
//! - Input strings are valid UTF-8 Markdown documents or section bodies.
//!
//! ## 4. Postconditions
//! - Accurately classifies stale compiler boilerplate and empty-AST fallback templates across Epic,
//!   Feature, User Story, and Use Case specifications.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear in document or section length.
//! - Space Complexity: $O(N)$ for extracted or replaced section buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Missing headings return `None` on extraction and leave `doc` unchanged on replacement.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0604] [REQ-SYNC-FORWARD-MERGE-EXISTING-SPEC]

/// Detects legacy hardcoded 5-step compiler-contaminated Use Case boilerplate phrases for eviction during non-forced forward sync.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-STALE-UC-BOILERPLATE]
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
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-UNDOC-UC-FALLBACK-DETECT]
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
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-STALE-US-BOILERPLATE]
pub(super) fn is_stale_user_story_boilerplate(s: &str) -> bool {
    s.contains("processes the AST within defined latency envelope")
        || s.contains("When semantic validator executes")
        || s.contains("graceful recovery occurs without data loss")
}

/// Detects legacy hardcoded compiler-boilerplate Epic section text for eviction during non-forced forward sync.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-STALE-EPIC-BOILERPLATE]
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
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-AUTO-EPIC-FALLBACK]
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
            "declares no package-scoped `requirement` or `constraint` governance invariants in schema.",
        ),
        _ => false,
    }
}

/// Detects legacy hardcoded CSS/UI/compiler Feature boilerplate text for eviction during non-forced forward sync.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-STALE-FEAT-BOILERPLATE]
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
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-AUTO-FEAT-FALLBACK]
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

/// Extracts the trimmed body of a Markdown section under `heading` while ignoring headings inside fenced code blocks.
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-EXTRACT-MARKDOWN-SECTION]
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
/// Realises: [REQ-0604] [REQ-SYNC-FORWARD-REPLACE-MARKDOWN-SECTION]
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
