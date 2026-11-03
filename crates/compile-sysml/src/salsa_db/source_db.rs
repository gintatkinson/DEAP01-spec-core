//! # Salsa Source File Input & Atomic Manifest Persistence (`source_db`)
//! [DOC-TIER: A]
//! ## 1. Safety Intent & Regulatory Scope -- Provides `#[salsa::input]` source file tracking, durability tiering (`LOW` vs `HIGH`), workspace `.sysml` discovery, and atomic disk persistence (`BP-10`).
//! ## 2. Invariants & Mathematical Properties -- Normalizes CRLF line endings deterministically and persists files via PID-scoped temporary rename.
//! ## 3. Preconditions -- Input paths are valid UTF-8 / OS paths; target parent directories are writable for atomic writes.
//! ## 4. Postconditions -- Emits deterministic `SalsaFileInput` handles, sorted `.sysml` file sets, and `FileDigestEntry` manifests.
//! ## 5. Algorithmic Complexity Bounds -- Time: O(B) over file byte length `B`; Space: O(B).
//! ## 6. Failure Modes & Fault Tolerance -- Unlinks temporary sibling files on atomic write failure and returns typed `io::Error`.
//! ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0603], [REQ-0608] | [FLS: fls_66m4rnbssgig]

#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub use deap_core::sysml_cst::normalize_line_endings;

use super::safety_digest::{compute_sha256, count_structural_elements, FileDigestEntry, SchemaDigest};

/// Stable numeric identifier for a tracked source file in `salsa_db`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Identifies tracked source buffers deterministically (`BP-10`).
/// ## 2. Invariants & Mathematical Properties -- Total ordering over `u32` indices.
/// ## 3. Preconditions -- Assigned monotonically by `SysmlIncrementalDb`.
/// ## 4. Postconditions -- Uniquely identifies a file within the database instance.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(1); Space: O(1).
/// ## 6. Failure Modes & Fault Tolerance -- Pure value struct with zero panics.
/// ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(pub u32);

/// Salsa durability tier distinguishing mutable user workspace files (`LOW`) from standard library packages (`HIGH`).
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Partitions `salsa::Durability::LOW` user models from `salsa::Durability::HIGH` standard libraries (`BP-10`).
/// ## 2. Invariants & Mathematical Properties -- Bijective two-tier partition.
/// ## 3. Preconditions -- Determined from package/file classification.
/// ## 4. Postconditions -- Controls incremental revision invalidation scope.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(1); Space: O(1).
/// ## 6. Failure Modes & Fault Tolerance -- Pure enum with zero runtime faults.
/// ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0608] | [FLS: fls_8tsynkj2cufj]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DurabilityTier {
    LowUserWorkspace,
    HighStandardLibrary,
}

/// Salsa input representing a tracked source file in `compile-sysml::salsa_db`.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Leaf input in the `salsa` dependency graph (`BP-10`).
/// ## 2. Invariants & Mathematical Properties -- Stores normalized UTF-8 source text and `DurabilityTier`.
/// ## 3. Preconditions -- Constructed via `SalsaFileInput::new(&db, path, text, durability_tier)`.
/// ## 4. Postconditions -- Invalidates dependent `#[salsa::tracked]` queries only when `text` mutates.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(1) field access; Space: O(N).
/// ## 6. Failure Modes & Fault Tolerance -- Managed by `salsa::Storage`.
/// ## 7. Realization & Traceability -- Realises: [REQ-0201], [REQ-0608] | [FLS: fls_66m4rnbssgig]
#[salsa::input]
pub struct SalsaFileInput {
    #[returns(ref)]
    pub path: String,
    #[returns(ref)]
    pub text: String,
    pub durability_tier: DurabilityTier,
}

/// Returns `true` when directory basename `name` is an excluded build/cache directory.
/// Realises: [REQ-PARSER-GRAMMAR-DISCOVER-SYSML-FILES] | [FLS: fls_66m4rnbssgig]
fn is_excluded_sysml_dir(name: &str) -> bool {
    matches!(
        name,
        "target" | "node_modules" | "build" | "Pods" | "vendor" | "dist" | "ephemeral" | "generated"
    )
}

/// Discovers all `.sysml` files under `dir` iteratively using an explicit `Vec<PathBuf>` work-stack.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Deterministic workspace `.sysml` discovery with zero recursion (`BP-10`).
/// ## 2. Invariants & Mathematical Properties -- Returns lexicographically sorted `Vec<PathBuf>` pruning hidden and build-cache directories.
/// ## 3. Preconditions -- `dir` is a filesystem path reference.
/// ## 4. Postconditions -- Returns sorted `.sysml` paths or empty `Vec` if `dir` is inaccessible.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(F log F) over discovered files `F`; Space: O(F).
/// ## 6. Failure Modes & Fault Tolerance -- Skips unreadable entries without panicking.
/// ## 7. Realization & Traceability -- Realises: [REQ-PARSER-GRAMMAR-DISCOVER-SYSML-FILES] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn discover_sysml_files(dir: &Path) -> Vec<PathBuf> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut dir_stack: Vec<PathBuf> = vec![dir.to_path_buf()];
    let mut files = Vec::new();
    while let Some(current_dir) = dir_stack.pop() {
        let entries = match fs::read_dir(&current_dir) {
            Ok(rd) => rd,
            Err(_) => continue,
        };
        let mut child_dirs = Vec::new();
        for entry_res in entries {
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };
            let raw_name = entry.file_name();
            let file_name = raw_name.to_string_lossy();
            if file_name.starts_with('.') || file_name.starts_with('#') {
                continue;
            }
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };
            let path = entry.path();
            if file_type.is_dir() {
                if !is_excluded_sysml_dir(file_name.as_ref()) {
                    child_dirs.push(path);
                }
            } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "sysml") {
                files.push(path);
            }
        }
        child_dirs.sort();
        dir_stack.extend(child_dirs);
    }
    files.sort();
    files
}

/// Normalizes a file path `p` into a workspace-relative forward-slash string for `FileDigestEntry::path`.
/// Realises: [REQ-SEMANTIC-DIGEST-COMPUTE-FILE-MANIFEST] | [FLS: fls_66m4rnbssgig]
fn resolve_manifest_relative_path(
    p: &Path,
    cwd: Option<&PathBuf>,
    common_root: Option<&PathBuf>,
) -> String {
    let rel_p = p.strip_prefix(".").unwrap_or(p);
    let rel_p = if !rel_p.is_absolute() {
        rel_p
    } else if let Some(stripped) = cwd.and_then(|cwd_path| rel_p.strip_prefix(cwd_path).ok()) {
        stripped
    } else if let Some(schema_ws_root) = rel_p
        .ancestors()
        .find(|anc| {
            anc.file_name()
                .is_some_and(|n| n == "schema" || n == "docs" || n == ".pipeline")
        })
        .and_then(Path::parent)
    {
        rel_p.strip_prefix(schema_ws_root).unwrap_or(rel_p)
    } else if let Some(root) = common_root {
        rel_p
            .strip_prefix(root)
            .ok()
            .filter(|s| !s.as_os_str().is_empty())
            .unwrap_or(rel_p)
    } else {
        rel_p.file_name().map(Path::new).unwrap_or(rel_p)
    };
    rel_p.to_string_lossy().replace('\\', "/")
}

/// Computes a deterministic `FileDigestEntry` manifest for a slice of SysML files.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Produces SHA-256 and structural entity counts per file under DO-178C Level A (`BP-10`).
/// ## 2. Invariants & Mathematical Properties -- Normalizes paths across host roots and computes SHA-256 digests deterministically.
/// ## 3. Preconditions -- `files` is a slice of `.sysml` `PathBuf` items.
/// ## 4. Postconditions -- Returns `Vec<FileDigestEntry>` in input slice order.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(B) over total bytes `B`; Space: O(F).
/// ## 6. Failure Modes & Fault Tolerance -- Falls back safely to zero counts on unreadable paths.
/// ## 7. Realization & Traceability -- Realises: [REQ-SEMANTIC-DIGEST-COMPUTE-FILE-MANIFEST] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn compute_file_manifest(files: &[PathBuf]) -> Vec<FileDigestEntry> {
    let mut manifest = Vec::new();
    let cwd = match std::env::current_dir() {
        Ok(d) => Some(d),
        Err(_) => None,
    };
    let common_root: Option<PathBuf> = files.first().and_then(|first| {
        let first_parent = first.parent()?;
        first_parent
            .ancestors()
            .find(|anc| files.iter().all(|f| f.starts_with(anc)))
            .map(Path::to_path_buf)
    });
    for p in files {
        let path = resolve_manifest_relative_path(p, cwd.as_ref(), common_root.as_ref());
        let bytes = match fs::metadata(p) {
            Ok(m) => m.len(),
            Err(_) => 0,
        };
        let content = match fs::read(p) {
            Ok(c) => c,
            Err(_) => Vec::new(),
        };
        let sha256 = compute_sha256(&content);
        let content_str = String::from_utf8_lossy(&content);
        let lines = if content.is_empty() { 0 } else { content_str.lines().count() };
        let entities = match crate::parse_sysml(&content_str) {
            Ok(pkg) => count_structural_elements(&pkg),
            Err(_) => 0,
        };
        manifest.push(FileDigestEntry {
            path,
            sha256,
            bytes,
            lines,
            entities,
        });
    }
    manifest
}

/// Atomically writes `SchemaDigest` JSON to disk using a temporary sibling file and rename.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Crash-consistent digest persistence (`BP-10`).
/// ## 2. Invariants & Mathematical Properties -- Delegates to `SchemaDigest::write_to_file`.
/// ## 3. Preconditions -- `path` parent directory is writable.
/// ## 4. Postconditions -- Persists pretty-printed JSON digest atomically.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(N); Space: O(N).
/// ## 6. Failure Modes & Fault Tolerance -- Propagates `io::Error` on failure.
/// ## 7. Realization & Traceability -- Realises: [REQ-SEMANTIC-DIGEST-WRITE-DIGEST-ATOMIC] | [FLS: fls_66m4rnbssgig]
pub fn write_digest_atomic(path: &Path, digest: &SchemaDigest) -> Result<(), io::Error> {
    digest.write_to_file(path)
}

/// Atomically writes arbitrary text to disk using a PID-scoped temporary sibling file and rename.
/// [DOC-TIER: A]
/// ## 1. Safety Intent & Regulatory Scope -- Crash-consistent artifact persistence under DO-330 TQL-4 (`BP-10`).
/// ## 2. Invariants & Mathematical Properties -- Writes to `.{file_name}.tmp.{pid}`, syncs via `sync_all`, and renames atomically.
/// ## 3. Preconditions -- `path` parent directory is writable; `content` is valid UTF-8.
/// ## 4. Postconditions -- Leaves zero residual `.tmp` sibling files on success or failure.
/// ## 5. Algorithmic Complexity Bounds -- Time: O(N); Space: O(1).
/// ## 6. Failure Modes & Fault Tolerance -- Removes temporary file and propagates `io::Error` on failure.
/// ## 7. Realization & Traceability -- Realises: [REQ-SEMANTIC-DIGEST-WRITE-ATOMIC] | [FLS: fls_66m4rnbssgig]
pub fn write_atomic(path: &Path, content: &str) -> Result<(), io::Error> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("artifact");
    let pid = std::process::id();
    let tmp_path = path.with_file_name(format!(".{file_name}.tmp.{pid}"));
    let write_res = (|| -> Result<(), io::Error> {
        let mut file = fs::File::create(&tmp_path)?;
        io::Write::write_all(&mut file, content.as_bytes())?;
        file.sync_all()?;
        fs::rename(&tmp_path, path)?;
        Ok(())
    })();
    if let Err(primary_err) = write_res {
        if tmp_path.exists() {
            if let Err(_cleanup_err) = fs::remove_file(&tmp_path) {
                return Err(primary_err);
            }
        }
        return Err(primary_err);
    }
    Ok(())
}
