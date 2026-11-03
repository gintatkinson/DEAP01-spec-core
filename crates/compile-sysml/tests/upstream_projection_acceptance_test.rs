//! End-to-End Upstream Blueprint-to-Specification and GitHub Projection Acceptance Tests.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides an end-to-end integration acceptance gate verifying that compiling the canonical
//! upstream SysML v2 metamodel (`.pipeline/upstream/schema/model.sysml`) produces a complete,
//! defect-free Agile specification hierarchy across Epics, Features, User Stories, and Use Cases.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Exact cardinality invariant: $|Epics| = 15 \land |Features| = 58 \land |UserStories| = 79 \land |UseCases| = 65$,
//!   yielding $|Specs_{total}| = 217$.
//! - Link closure invariant: $\forall d \in Specs_{total}, \forall l \in Links_{rel}(d): \text{exists}(\text{dir}(d) \oplus l) = \text{true}$.
//! - Local purity invariant: $\forall d \in Specs_{total}: \text{https://github.com/} \notin d$.
//! - Anti-stutter invariant: $\forall d \in Specs_{total}: \text{UC\_AC\_} \notin \text{name}(d) \land \text{US\_AC\_} \notin \text{name}(d)$.
//! - Package isolation invariant: $\forall u \in \{US\text{-}06 \dots US\text{-}44\}: \text{FEAT-01-} \notin u$.
//! - Behavioral state-machine invariant: $\forall e \in Epics: \text{stateDiagram-v2} \in e \land \text{-->} \in e$.
//!
//! ## 3. Preconditions
//! - The repository root contains `.pipeline/upstream/schema/model.sysml`.
//! - The temporary filesystem directory is writable for isolated test execution.
//!
//! ## 4. Postconditions
//! - All 208 specification files are synthesized and verified inside an isolated RAII directory.
//! - The temporary directory is deterministically removed on test completion via `Drop`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N + L)$ where $N = 208$ generated specification files and $L$ is the total byte length.
//! - Space Complexity: $O(L)$ bounded by the in-memory AST and generated specification strings.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Missing upstream schema, broken relative markdown links, hardcoded GitHub URLs, anti-stutter violations,
//!   cross-package `FEAT-01` pollution, or empty epic state diagrams return a descriptive `Err`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-UPSTREAM-PROJECTION-ACCEPTANCE]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// RAII temporary workspace guard that cleans up its root directory when dropped.
/// Realises: [REQ-UPSTREAM-PROJECTION-ACCEPTANCE]
struct TempUpstreamWorkspace {
    root: PathBuf,
}

impl TempUpstreamWorkspace {
    /// Creates a new isolated temporary workspace containing `schema/` and `docs/` subdirectories.
    /// Realises: [REQ-UPSTREAM-PROJECTION-ACCEPTANCE]
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!(
            "deap_upstream_projection_acceptance_{}_{}",
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(root.join("schema"))?;
        fs::create_dir_all(root.join("docs"))?;
        Ok(Self { root })
    }
}

impl Drop for TempUpstreamWorkspace {
    /// Removes the temporary workspace directory tree when `TempUpstreamWorkspace` goes out of scope.
    /// Realises: [REQ-UPSTREAM-PROJECTION-ACCEPTANCE]
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Extracts all markdown link targets from `[label](target)` expressions outside fenced code blocks.
/// Realises: [REQ-UPSTREAM-PROJECTION-ACCEPTANCE]
fn extract_markdown_link_targets(content: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut in_fence = false;

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }

        let mut rest = line;
        while let Some(open_idx) = rest.find("](") {
            let after_open_idx = open_idx.saturating_add(2);
            let Some(after_open) = rest.get(after_open_idx..) else {
                break;
            };
            let Some(close_idx) = after_open.find(')') else {
                break;
            };
            if let Some(raw_target) = after_open.get(..close_idx) {
                let target_trimmed = raw_target.trim();
                if !target_trimmed.is_empty() {
                    targets.push(target_trimmed.to_string());
                }
            }
            let next_idx = close_idx.saturating_add(1);
            let Some(next_rest) = after_open.get(next_idx..) else {
                break;
            };
            rest = next_rest;
        }
    }

    targets
}

/// Collects and lexicographically sorts all `.md` file paths directly inside `dir`.
/// Realises: [REQ-UPSTREAM-PROJECTION-ACCEPTANCE]
fn collect_sorted_md_files(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    for entry_res in fs::read_dir(dir)? {
        let entry = entry_res?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

/// Verifies that no Mermaid diagram block inside `content` contains inline backticks or HTML entities.
/// Realises: [REQ-UPSTREAM-PROJECTION-ACCEPTANCE]
fn verify_mermaid_hygiene(file_name: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut in_mermaid = false;
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if !in_mermaid && trimmed.starts_with("```mermaid") {
            in_mermaid = true;
            continue;
        }
        if in_mermaid && trimmed.starts_with("```") {
            in_mermaid = false;
            continue;
        }
        if in_mermaid {
            if trimmed.contains('`') {
                return Err(format!(
                    "Inline backtick found inside Mermaid block at {}:{} -> {}",
                    file_name,
                    line_idx + 1,
                    trimmed
                )
                .into());
            }
            if trimmed.contains("&lt;") || trimmed.contains("&gt;") {
                return Err(format!(
                    "HTML entity found inside Mermaid block at {}:{} -> {}",
                    file_name,
                    line_idx + 1,
                    trimmed
                )
                .into());
            }
        }
    }
    Ok(())
}

/// Verifies all link closure, local purity, anti-stutter, package isolation, and diagram invariants for `md_path`.
/// Realises: [REQ-UPSTREAM-PROJECTION-ACCEPTANCE]
fn verify_spec_file_invariants(md_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = md_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("Invalid UTF-8 filename at {}", md_path.display()))?;
    let md_file_dir = md_path
        .parent()
        .ok_or_else(|| format!("Missing parent directory for {}", md_path.display()))?;
    let content = fs::read_to_string(md_path)?;

    for target in extract_markdown_link_targets(&content) {
        if target.starts_with("http://") || target.starts_with("https://") || target.starts_with('#') {
            continue;
        }
        if !md_file_dir.join(&target).exists() {
            return Err(format!("Broken relative link target '{}' in file {}", target, md_path.display()).into());
        }
    }
    if content.contains("https://github.com/") {
        return Err(format!("Hardcoded GitHub link found in local specification file {}", md_path.display()).into());
    }
    if file_name.contains("UC_AC_") || file_name.contains("US_AC_") {
        return Err(format!("Anti-stutter filename violation: {}", file_name).into());
    }
    for line in content.lines() {
        if line.starts_with('#') && (line.contains("UC AC ") || line.contains("US AC ")) {
            return Err(format!("Anti-stutter title violation '{}' in {}", line, md_path.display()).into());
        }
    }
    if file_name.starts_with("US-") {
        let is_bp01_story = ["US-01-", "US-02-", "US-03-", "US-04-", "US-05-"]
            .iter()
            .any(|p| file_name.starts_with(p));
        if !is_bp01_story && content.contains("FEAT-01-") {
            return Err(format!("Cross-package FEAT-01 pollution detected in {}", file_name).into());
        }
    }
    if file_name.starts_with("EPIC-")
        && (!content.contains("stateDiagram-v2") || !content.contains("-->"))
    {
        return Err(format!("Epic stateDiagram-v2 missing or lacks transitions in {}", file_name).into());
    }
    verify_mermaid_hygiene(file_name, &content)?;
    if file_name.starts_with("UC-")
        && !content.contains("UC_Main -. \"<<include>>\" .-> UC_Step01")
    {
        return Err(format!("Use Case diagram missing clean <<include>> edge in {}", file_name).into());
    }
    Ok(())
}

#[test]
fn test_upstream_blueprint_to_spec_and_github_projection_zero_defects(
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let upstream_schema = repo_root.join(".pipeline/upstream/schema/model.sysml");
    if !upstream_schema.exists() {
        return Err(format!("Missing canonical upstream schema at {}", upstream_schema.display()).into());
    }

    let temp_ws = TempUpstreamWorkspace::new()?;
    let target_schema = temp_ws.root.join("schema/model.sysml");
    fs::copy(&upstream_schema, &target_schema)?;

    let schema_text = fs::read_to_string(&target_schema)?;
    let pkg = compile_sysml::parse_sysml(&schema_text)?;

    let opts = compile_sysml::projections::forward::ForwardSyncOptions {
        schema_path: target_schema,
        docs_dir: temp_ws.root.join("docs"),
        out_dir: Some(temp_ws.root.join("docs")),
        dry_run: false,
        force: true,
    };
    let generated = compile_sysml::projections::forward::forward_sync_sysml_to_specs(&pkg, &opts)?;
    if generated.len() < 208 {
        return Err(format!("Expected at least 208 forward-synced files, got {}", generated.len()).into());
    }

    let epic_files = collect_sorted_md_files(&temp_ws.root.join("docs/epics"))?;
    let feature_files = collect_sorted_md_files(&temp_ws.root.join("docs/features"))?;
    let story_files = collect_sorted_md_files(&temp_ws.root.join("docs/user-stories"))?;
    let use_case_files = collect_sorted_md_files(&temp_ws.root.join("docs/use-cases"))?;

    for (label, count, expected) in [
        ("Epics", epic_files.len(), 15usize),
        ("Features", feature_files.len(), 58),
        ("User Stories", story_files.len(), 79),
        ("Use Cases", use_case_files.len(), 65),
    ] {
        if count != expected {
            return Err(format!(
                "Expected {expected} {label}, found {count} (totals: epics={}, features={}, stories={}, ucs={})",
                epic_files.len(), feature_files.len(), story_files.len(), use_case_files.len()
            ).into());
        }
    }

    let mut all_files = Vec::with_capacity(218);
    all_files.extend(epic_files);
    all_files.extend(feature_files);
    all_files.extend(story_files);
    all_files.extend(use_case_files);
    if all_files.len() != 217 {
        return Err(format!("Expected total of 217 specification files, found {}", all_files.len()).into());
    }

    for md_path in &all_files {
        verify_spec_file_invariants(md_path)?;
    }

    Ok(())
}
