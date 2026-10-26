//! End-to-End Upstream Blueprint-to-Specification and GitHub Projection Acceptance Tests.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides an end-to-end integration acceptance gate verifying that compiling the canonical
//! upstream SysML v2 metamodel (`.pipeline/upstream/schema/model.sysml`) produces a complete,
//! defect-free Agile specification hierarchy across Epics, Features, User Stories, and Use Cases.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Exact cardinality invariant: $|Epics| = 6 \land |Features| = 30 \land |UserStories| = 62 \land |UseCases| = 31$,
//!   yielding $|Specs_{total}| = 129$.
//! - Link closure invariant: $\forall d \in Specs_{total}, \forall l \in Links_{rel}(d): \text{exists}(\text{dir}(d) \oplus l) = \text{true}$.
//! - Local purity invariant: $\forall d \in Specs_{total}: \text{https://github.com/} \notin d$.
//! - Anti-stutter invariant: $\forall d \in Specs_{total}: \text{UC\_AC\_} \notin \text{name}(d) \land \text{US\_AC\_} \notin \text{name}(d)$.
//! - Package isolation invariant: $\forall u \in \{US\text{-}06 \dots US\text{-}62\}: \text{FEAT-01-} \notin u$.
//! - Behavioral state-machine invariant: $\forall e \in Epics: \text{stateDiagram-v2} \in e \land \text{-->} \in e$.
//!
//! ## 3. Preconditions
//! - The repository root contains `.pipeline/upstream/schema/model.sysml`.
//! - The temporary filesystem directory is writable for isolated test execution.
//!
//! ## 4. Postconditions
//! - All 129 specification files are synthesized and verified inside an isolated RAII directory.
//! - The temporary directory is deterministically removed on test completion via `Drop`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N + L)$ where $N = 129$ generated specification files and $L$ is the total byte length of all generated markdown files.
//! - Space Complexity: $O(L)$ bounded by the in-memory AST and generated specification strings.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Missing upstream schema, broken relative markdown links, hardcoded GitHub URLs, anti-stutter violations,
//!   cross-package `FEAT-01` pollution, or empty epic state diagrams return a descriptive `Err`.
//!
//! ## 7. Realization & Traceability
//! - Verifies `compile_sysml::sync::forward::forward_sync_sysml_to_specs` against `.pipeline/upstream/schema/model.sysml`.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// RAII temporary workspace guard that cleans up its root directory when dropped.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Ensures isolated filesystem fixtures created during acceptance testing never pollute
/// the repository working tree or persist across test runs.
///
/// ## 2. Invariants & Mathematical Properties
/// - `self.root` points to a unique directory under `std::env::temp_dir()`.
/// - Upon scope exit, `Drop::drop` removes `self.root` and all nested contents.
///
/// ## 3. Preconditions
/// - The host OS temporary directory must be writable.
///
/// ## 4. Postconditions
/// - `self.root/schema` and `self.root/docs` exist upon construction and are removed upon drop.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ creation and $O(F)$ cleanup for $F$ created fixture files.
/// - Space Complexity: $O(1)$ in-memory path storage.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `Err` if directory creation fails; ignores cleanup I/O errors during `Drop`.
///
/// ## 7. Realization & Traceability
/// - Enforces the Forbidden Test Workspace Creation rule via deterministic RAII cleanup.
struct TempUpstreamWorkspace {
    root: PathBuf,
}

impl TempUpstreamWorkspace {
    /// Creates a new isolated temporary workspace containing `schema/` and `docs/` subdirectories.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Allocates a collision-free temporary directory for forward-sync specification compilation tests.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Unique directory name derived from process ID and monotonic system timestamp nanos.
    ///
    /// ## 3. Preconditions
    /// - `std::env::temp_dir()` is accessible and writable.
    ///
    /// ## 4. Postconditions
    /// - Returns an initialized `TempUpstreamWorkspace` with `schema/` and `docs/` directories created on disk.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$ filesystem directory allocation.
    /// - Space Complexity: $O(1)$ heap allocation for `PathBuf`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Propagates any `std::io::Error` encountered while creating the temporary directory structure.
    ///
    /// ## 7. Realization & Traceability
    /// - Supports `test_upstream_blueprint_to_spec_and_github_projection_zero_defects`.
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let pid = std::process::id();
        let root = std::env::temp_dir().join(format!(
            "deap_upstream_projection_acceptance_{}_{}",
            pid, nanos
        ));
        fs::create_dir_all(root.join("schema"))?;
        fs::create_dir_all(root.join("docs"))?;
        Ok(Self { root })
    }
}

impl Drop for TempUpstreamWorkspace {
    /// Removes the temporary workspace directory tree when `TempUpstreamWorkspace` goes out of scope.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Prevents test fixture leakage on both passing and failing execution paths.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Idempotent best-effort recursive removal of `self.root`.
    ///
    /// ## 3. Preconditions
    /// - `self.root` was initialized by `TempUpstreamWorkspace::new`.
    ///
    /// ## 4. Postconditions
    /// - `self.root` no longer exists on the filesystem.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(F)$ where $F$ is the number of files inside `self.root`.
    /// - Space Complexity: $O(1)$ auxiliary memory.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Suppresses `remove_dir_all` errors so unwinding never double-panics.
    ///
    /// ## 7. Realization & Traceability
    /// - Implements `std::ops::Drop` for `TempUpstreamWorkspace`.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Extracts all markdown link targets from `[label](target)` expressions outside fenced code blocks.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Provides memory-safe, zero-indexing extraction of markdown hyperlink targets so that every
/// generated relative link can be verified to exist on disk.
///
/// ## 2. Invariants & Mathematical Properties
/// - Ignores lines enclosed within fenced code blocks (` ``` `).
/// - Uses only bounds-checked `.find(..)` and `.get(..)` slicing with zero direct index operations.
///
/// ## 3. Preconditions
/// - `content` is a valid UTF-8 string slice representing a markdown document.
///
/// ## 4. Postconditions
/// - Returns a `Vec<String>` containing every non-empty trimmed link target in document order.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(B)$ where $B$ is the byte length of `content`.
/// - Space Complexity: $O(K)$ where $K$ is the number of extracted markdown link targets.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Malformed or unclosed `[label](` patterns are safely skipped without error or panic.
///
/// ## 7. Realization & Traceability
/// - Used by `test_upstream_blueprint_to_spec_and_github_projection_zero_defects` to enforce Check 1 (Zero Broken Relative Links).
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
///
/// ## 1. Safety Intent & Regulatory Scope
/// Deterministically enumerates generated markdown files within a specification category directory.
///
/// ## 2. Invariants & Mathematical Properties
/// - Filters strictly to regular files whose extension equals `"md"`.
/// - Output vector is sorted in ascending lexicographical order.
///
/// ## 3. Preconditions
/// - `dir` exists on disk and is a readable directory.
///
/// ## 4. Postconditions
/// - Returns the sorted list of `.md` `PathBuf` entries inside `dir`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(M \log M)$ for $M$ directory entries.
/// - Space Complexity: $O(M)$ to store the resulting `PathBuf` collection.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `Err` if `fs::read_dir` or directory entry inspection fails.
///
/// ## 7. Realization & Traceability
/// - Helper for `test_upstream_blueprint_to_spec_and_github_projection_zero_defects`.
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

/// Verifies end-to-end compilation of `.pipeline/upstream/schema/model.sysml` into 129 defect-free specifications.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Acts as the primary regression and acceptance gate for upstream SysML-to-specification projection,
/// ensuring zero broken links, zero hardcoded GitHub URLs in local specs, zero anti-stutter defects,
/// zero cross-package `FEAT-01` feature link pollution, and non-empty Epic state diagrams.
///
/// ## 2. Invariants & Mathematical Properties
/// - Synthesizes 6 Epics, 30 Features, 62 User Stories, and 31 Use Cases (129 total markdown files).
/// - Every relative link target resolves to an existing file on disk.
/// - No generated markdown file contains `"https://github.com/"`.
/// - No filename contains `"UC_AC_"` or `"US_AC_"`, and no heading title contains `"UC AC "` or `"US AC "`.
/// - User Stories `US-06` through `US-62` do not reference `"FEAT-01-"`.
/// - Every Epic file contains `"stateDiagram-v2"` and `"-->"`.
///
/// ## 3. Preconditions
/// - `.pipeline/upstream/schema/model.sysml` is present relative to the workspace root.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` only if all cardinality and content invariants hold across all 129 files.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N + L)$ over $N = 129$ files and $L$ total markdown bytes.
/// - Space Complexity: $O(L)$ bounded by the temporary workspace and in-memory verification buffers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns a descriptive `Err` identifying the exact file and invariant violation if any check fails.
///
/// ## 7. Realization & Traceability
/// - Validates `compile_sysml::sync::forward::forward_sync_sysml_to_specs`.
#[test]
fn test_upstream_blueprint_to_spec_and_github_projection_zero_defects(
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let upstream_schema = repo_root.join(".pipeline/upstream/schema/model.sysml");
    if !upstream_schema.exists() {
        return Err(format!(
            "Missing canonical upstream schema at {}",
            upstream_schema.display()
        )
        .into());
    }

    let temp_ws = TempUpstreamWorkspace::new()?;
    let target_schema = temp_ws.root.join("schema/model.sysml");
    fs::copy(&upstream_schema, &target_schema)?;

    let schema_text = fs::read_to_string(&target_schema)?;
    let pkg = compile_sysml::parse_sysml(&schema_text)?;

    let opts = compile_sysml::sync::forward::ForwardSyncOptions {
        schema_path: target_schema,
        docs_dir: temp_ws.root.join("docs"),
        out_dir: Some(temp_ws.root.join("docs")),
        dry_run: false,
        force: true,
    };
    let generated = compile_sysml::sync::forward::forward_sync_sysml_to_specs(&pkg, &opts)?;
    if generated.len() < 129 {
        return Err(format!(
            "Expected at least 129 forward-synced files, got {}",
            generated.len()
        )
        .into());
    }

    let epics_dir = temp_ws.root.join("docs/epics");
    let features_dir = temp_ws.root.join("docs/features");
    let stories_dir = temp_ws.root.join("docs/user-stories");
    let use_cases_dir = temp_ws.root.join("docs/use-cases");

    let epic_files = collect_sorted_md_files(&epics_dir)?;
    let feature_files = collect_sorted_md_files(&features_dir)?;
    let story_files = collect_sorted_md_files(&stories_dir)?;
    let use_case_files = collect_sorted_md_files(&use_cases_dir)?;

    if epic_files.len() != 6 {
        return Err(format!("Expected 6 Epics, found {}", epic_files.len()).into());
    }
    if feature_files.len() != 30 {
        return Err(format!("Expected 30 Features, found {}", feature_files.len()).into());
    }
    if story_files.len() != 62 {
        return Err(format!("Expected 62 User Stories, found {}", story_files.len()).into());
    }
    if use_case_files.len() != 31 {
        return Err(format!("Expected 31 Use Cases, found {}", use_case_files.len()).into());
    }

    let mut all_files = Vec::with_capacity(129);
    all_files.extend(epic_files.iter().cloned());
    all_files.extend(feature_files.iter().cloned());
    all_files.extend(story_files.iter().cloned());
    all_files.extend(use_case_files.iter().cloned());

    if all_files.len() != 129 {
        return Err(format!(
            "Expected total of 129 specification files, found {}",
            all_files.len()
        )
        .into());
    }

    for md_path in &all_files {
        let file_name = md_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Invalid UTF-8 filename at {}", md_path.display()))?;
        let md_file_dir = md_path
            .parent()
            .ok_or_else(|| format!("Missing parent directory for {}", md_path.display()))?;
        let content = fs::read_to_string(md_path)?;

        // 1. Zero Broken Relative Links
        let targets = extract_markdown_link_targets(&content);
        for target in targets {
            if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with('#')
            {
                continue;
            }
            if !md_file_dir.join(&target).exists() {
                return Err(format!(
                    "Broken relative link target '{}' in file {}",
                    target,
                    md_path.display()
                )
                .into());
            }
        }

        // 2. Zero Hardcoded GitHub Blob Links in Local Markdown
        if content.contains("https://github.com/") {
            return Err(format!(
                "Hardcoded GitHub link found in local specification file {}",
                md_path.display()
            )
            .into());
        }

        // 3. Zero Anti-Stutter Filenames or Titles
        if file_name.contains("UC_AC_") || file_name.contains("US_AC_") {
            return Err(format!("Anti-stutter filename violation: {}", file_name).into());
        }
        for line in content.lines() {
            if line.starts_with('#') && (line.contains("UC AC ") || line.contains("US AC ")) {
                return Err(format!(
                    "Anti-stutter title violation '{}' in {}",
                    line,
                    md_path.display()
                )
                .into());
            }
        }

        // 4. Zero Cross-Package FEAT-01 Pollution on BP_02..06 User Stories (US-06..US-62)
        if file_name.starts_with("US-") {
            let is_bp01_story = file_name.starts_with("US-01-")
                || file_name.starts_with("US-02-")
                || file_name.starts_with("US-03-")
                || file_name.starts_with("US-04-")
                || file_name.starts_with("US-05-");
            if !is_bp01_story && content.contains("FEAT-01-") {
                return Err(format!(
                    "Cross-package FEAT-01 pollution detected in {}",
                    file_name
                )
                .into());
            }
        }

        // 5. All Epic stateDiagram-v2 Blocks Have Non-Empty Transitions
        if file_name.starts_with("EPIC-")
            && (!content.contains("stateDiagram-v2") || !content.contains("-->"))
        {
            return Err(format!(
                "Epic stateDiagram-v2 missing or lacks transitions in {}",
                file_name
            )
            .into());
        }
    }

    Ok(())
}
