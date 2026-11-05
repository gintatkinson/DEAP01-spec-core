#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! File manifest extraction and crash-consistent atomic disk persistence for SysML v2 digests.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for artifact persistence.
//! ## 2. Invariants & Mathematical Properties
//! Normalizes manifest paths deterministically across host roots and persists files via PID-scoped temporary rename.
//! ## 3. Preconditions
//! Input file paths are readable or fall back safely to empty content; target parent directories are writable.
//! ## 4. Postconditions
//! Produces deterministic `FileDigestEntry` records and atomic disk writes leaving zero residual `.tmp` files.
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(B) over file bytes `B`. Space Complexity: O(B) buffer memory per file.
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Removes temporary sibling file on write/sync/rename failure.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603, REQ-SEMANTIC-DIGEST-COMPUTE-FILE-MANIFEST, REQ-SEMANTIC-DIGEST-WRITE-ATOMIC]

use super::digest::{compute_sha256, count_structural_elements, FileDigestEntry, SchemaDigest};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Normalizes a file path `p` into a workspace-relative forward-slash string for `FileDigestEntry::path`.
/// Realises: [REQ-SEMANTIC-DIGEST-COMPUTE-FILE-MANIFEST]
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

/// Compute a file manifest for a slice of SysML files.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `compute_file_manifest`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `compute_file_manifest`.
/// ## 3. Preconditions
/// - `files: &[PathBuf]` must satisfy domain and type-safety constraints prior to invoking `compute_file_manifest`.
/// ## 4. Postconditions
/// - Returns `Vec<FileDigestEntry>` from `compute_file_manifest` with deterministic post-execution state.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(files)) bounded over input dimension. Space Complexity: O(N(files)) auxiliary memory.
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface / State / Action / Resource: Total computation falling back to empty bytes on unreadable paths.
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-COMPUTE-FILE-MANIFEST] | SysML v2: ActionDef(compute_file_manifest)
pub fn compute_file_manifest(files: &[PathBuf]) -> Vec<FileDigestEntry> {
    let mut manifest = Vec::new();
    let cwd = std::env::current_dir().ok();
    let common_root: Option<PathBuf> = files.first().and_then(|first| {
        let first_parent = first.parent()?;
        first_parent
            .ancestors()
            .find(|anc| files.iter().all(|f| f.starts_with(anc)))
            .map(Path::to_path_buf)
    });

    for p in files {
        let path = resolve_manifest_relative_path(p, cwd.as_ref(), common_root.as_ref());
        let bytes = fs::metadata(p).map(|m| m.len()).unwrap_or(0);
        let content = fs::read(p).unwrap_or_default();
        let sha256 = compute_sha256(&content);
        let content_str = String::from_utf8_lossy(&content);
        let lines = if content.is_empty() {
            0
        } else {
            content_str.lines().count()
        };
        let entities = crate::parse_sysml(&content_str)
            .map(|pkg| count_structural_elements(&pkg))
            .unwrap_or(0);

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

/// Atomically write digest JSON to disk using a temporary sibling file and rename.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `write_digest_atomic`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `write_digest_atomic`.
/// ## 3. Preconditions
/// - `path: &Path` and `digest: &SchemaDigest` satisfy domain and type-safety constraints.
/// ## 4. Postconditions
/// - Returns `Result<(), io::Error>` from `write_digest_atomic` with deterministic post-execution state.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(digest)) bounded over serialized JSON size. Space Complexity: O(N(digest)).
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface / State / Action / Resource: Delegates to `SchemaDigest::write_to_file`.
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-WRITE-DIGEST-ATOMIC] | SysML v2: ActionDef(write_digest_atomic)
pub fn write_digest_atomic(path: &Path, digest: &SchemaDigest) -> Result<(), io::Error> {
    digest.write_to_file(path)
}

/// Atomically write arbitrary text to disk using a temporary sibling file and rename.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `write_atomic`.
/// ## 2. Invariants & Mathematical Properties
/// - Writes to `.{file_name}.tmp.{pid}`, syncs via `sync_all`, and renames atomically without clobbering `.tmp` siblings.
/// ## 3. Preconditions
/// - `path: &Path` and `content: &str` satisfy domain and type-safety constraints.
/// ## 4. Postconditions
/// - Returns `Result<(), io::Error>` and removes temporary sibling file on error.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(content)). Space Complexity: O(1) auxiliary heap memory.
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface / State / Action / Resource: Propagates `io::Error` after unlinking temporary file.
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-WRITE-ATOMIC] | SysML v2: ActionDef(write_atomic)
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
    let tmp_name = format!(".{file_name}.tmp.{pid}");
    let tmp_path = path.with_file_name(tmp_name);
    let write_res = (|| -> Result<(), io::Error> {
        let mut file = fs::File::create(&tmp_path)?;
        io::Write::write_all(&mut file, content.as_bytes())?;
        file.sync_all()?;
        fs::rename(&tmp_path, path)?;
        Ok(())
    })();
    if write_res.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    write_res
}
