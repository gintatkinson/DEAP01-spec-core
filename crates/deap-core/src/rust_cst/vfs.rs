#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Cross-platform virtual filesystem and CRLF/LF normalization. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A and ISO 26262 ASIL D deterministic compilation rules.
//! Governs workspace Rust source discovery, path canonicalization, and line normalization.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Path normalization: all path separators map to `/`, without redundant `./` prefixes.
//! - Deterministic traversal: `.rs` files are indexed in strict lexicographic order.
//! - Line ending normalization: CRLF byte pairs `\r\n` map to LF `\n`.
//!
//! ## 3. Preconditions
//! - Root directory must be accessible and contain valid UTF-8 filesystem paths.
//!
//! ## 4. Postconditions
//! - Virtual filesystem mirrors workspace `.rs` content with normalized LF bytes.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Discovery: $O(N \log N)$ where $N$ is discovered file count due to sorting.
//! - Normalization: $O(M)$ linear over total workspace source byte size $M$.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - I/O or non-UTF-8 errors return structured `CoreError::Io` without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0608], [SC-01], [FLS: fls_cst_vfs]
//! SysML v2: part def WorkspaceVfs

use crate::error::{CoreError, CoreResult};
use camino::{Utf8Path, Utf8PathBuf};
use ignore::WalkBuilder;
use ra_ap_paths::AbsPathBuf;
use ra_ap_vfs::{FileId, Vfs, VfsPath};
use std::collections::BTreeMap;
use std::fs;

/// Normalizes Windows CRLF (`\r\n`) line endings to standard Unix LF (`\n`).
/// Realises: [REQ-0608], [FLS: fls_cst_vfs]
#[must_use]
pub fn normalize_lf(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// Normalizes relative or platform-specific path separators to `/` and strips `./`.
/// Realises: [REQ-0608], [FLS: fls_cst_vfs]
#[must_use]
pub fn normalize_rel_path(path: &str) -> String {
    let slash = path.replace('\\', "/");
    let trimmed = slash.trim();
    let mut cur = trimmed;
    while let Some(rest) = cur.strip_prefix("./") {
        cur = rest;
    }
    cur.to_string()
}

/// In-memory virtual filesystem capturing workspace `.rs` files.
/// Realises: [REQ-0608], [SC-01]
#[derive(Default)]
pub struct WorkspaceVfs {
    root: Utf8PathBuf,
    vfs: Vfs,
    files: BTreeMap<Utf8PathBuf, (FileId, String)>,
    id_to_path: BTreeMap<FileId, Utf8PathBuf>,
}

impl WorkspaceVfs {
    /// Loads and indexes all `.rs` files under `root` deterministically.
    /// Realises: [REQ-0608], [SC-01]
    pub fn load(root: &Utf8Path) -> CoreResult<Self> {
        let abs_root = if root.is_absolute() {
            root.to_path_buf()
        } else {
            let cwd = std::env::current_dir().map_err(|e| CoreError::io(e.to_string()))?;
            let cwd_utf8 = Utf8PathBuf::from_path_buf(cwd)
                .map_err(|p| CoreError::io(format!("non-utf8 cwd: {}", p.display())))?;
            cwd_utf8.join(root)
        };

        let mut discovered = Vec::new();
        for entry in WalkBuilder::new(root.as_std_path()).standard_filters(true).build() {
            let entry = entry.map_err(|e| CoreError::io(e.to_string()))?;
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("rs") {
                let utf8_p = Utf8PathBuf::from_path_buf(p.to_path_buf())
                    .map_err(|e| CoreError::io(format!("non-utf8 path: {}", e.display())))?;
                discovered.push(utf8_p);
            }
        }
        discovered.sort();

        let mut vfs = Vfs::default();
        let mut files = BTreeMap::new();
        let mut id_to_path = BTreeMap::new();

        for path in discovered {
            let abs_path = if path.is_absolute() {
                path.clone()
            } else {
                abs_root.join(&path)
            };
            let abs_buf = AbsPathBuf::assert(abs_path);
            let raw = fs::read_to_string(path.as_std_path())
                .map_err(|e| CoreError::io(format!("failed to read {path}: {e}")))?;
            let normalized = normalize_lf(&raw);
            let vfs_path = VfsPath::from(abs_buf);
            vfs.set_file_contents(vfs_path.clone(), Some(normalized.as_bytes().to_vec()));
            let (file_id, _) = vfs
                .file_id(&vfs_path)
                .ok_or_else(|| CoreError::workspace(format!("vfs failed to intern {path}")))?;
            files.insert(path.clone(), (file_id, normalized));
            id_to_path.insert(file_id, path);
        }

        Ok(Self {
            root: abs_root,
            vfs,
            files,
            id_to_path,
        })
    }

    /// Returns the root path.
    /// Realises: [REQ-0608], [FLS: fls_cst_vfs]
    #[must_use]
    pub fn root(&self) -> &Utf8Path {
        &self.root
    }

    /// Returns a reference to the underlying `ra_ap_vfs::Vfs`.
    /// Realises: [REQ-0608], [FLS: fls_cst_vfs]
    #[must_use]
    pub fn vfs(&self) -> &Vfs {
        &self.vfs
    }

    /// Returns a mutable reference to the underlying `ra_ap_vfs::Vfs`.
    /// Realises: [REQ-0608], [FLS: fls_cst_vfs]
    pub fn vfs_mut(&mut self) -> &mut Vfs {
        &mut self.vfs
    }

    /// Returns the number of loaded `.rs` files.
    /// Realises: [REQ-0608], [FLS: fls_cst_vfs]
    #[must_use]
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Returns normalized text for a given `FileId`.
    /// Realises: [REQ-0608], [FLS: fls_cst_vfs]
    #[must_use]
    pub fn file_text(&self, file_id: FileId) -> Option<&str> {
        self.id_to_path
            .get(&file_id)
            .and_then(|p| self.files.get(p))
            .map(|(_, t)| t.as_str())
    }

    /// Returns the path for a given `FileId`.
    /// Realises: [REQ-0608], [FLS: fls_cst_vfs]
    #[must_use]
    pub fn file_path(&self, file_id: FileId) -> Option<&Utf8Path> {
        self.id_to_path.get(&file_id).map(|p| p.as_path())
    }

    /// Looks up `FileId` by its workspace path.
    /// Realises: [REQ-0608], [FLS: fls_cst_vfs]
    #[must_use]
    pub fn file_id_for_path(&self, path: &Utf8Path) -> Option<FileId> {
        self.files.get(path).map(|(id, _)| *id)
    }

    /// Iterates over all indexed files with `FileId` and path.
    /// Realises: [REQ-0608], [FLS: fls_cst_vfs]
    pub fn iter_files(&self) -> impl Iterator<Item = (FileId, &Utf8Path)> {
        self.id_to_path.iter().map(|(id, path)| (*id, path.as_path()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static VFS_SEQ: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn test_normalize_lf() {
        assert_eq!(normalize_lf("a\r\nb\r\nc"), "a\nb\nc");
        assert_eq!(normalize_lf("x\ny\n"), "x\ny\n");
        assert_eq!(normalize_lf(""), "");
    }

    #[test]
    fn test_normalize_rel_path() {
        assert_eq!(normalize_rel_path("crates\\foo\\src\\lib.rs"), "crates/foo/src/lib.rs");
        assert_eq!(normalize_rel_path("./crates/foo/bar.rs"), "crates/foo/bar.rs");
        assert_eq!(normalize_rel_path(".\\foo\\bar.rs"), "foo/bar.rs");
    }

    #[test]
    fn test_workspace_vfs_load() -> Result<(), String> {
        let seq = VFS_SEQ.fetch_add(1, Ordering::Relaxed);
        let temp_dir = std::env::temp_dir().join(format!("deap_test_vfs_1_{}_{seq}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(temp_dir.join("src")).map_err(|e| e.to_string())?;
        fs::write(temp_dir.join("src/lib.rs"), "pub fn hello() {}\r\n").map_err(|e| e.to_string())?;
        fs::write(temp_dir.join("src/main.rs"), "fn main() {}\r\n").map_err(|e| e.to_string())?;
        fs::write(temp_dir.join("README.md"), "# ignored").map_err(|e| e.to_string())?;

        let root = Utf8PathBuf::from_path_buf(temp_dir.clone()).map_err(|_| "utf8 temp failed".to_string())?;
        let vfs = WorkspaceVfs::load(&root)?;
        assert_eq!(vfs.file_count(), 2);
        for (id, path) in vfs.iter_files() {
            let text = vfs.file_text(id).ok_or_else(|| "missing file text".to_string())?;
            assert!(!text.contains("\r\n"), "must be LF normalized: {path}");
        }
        let _ = fs::remove_dir_all(&temp_dir);
        Ok(())
    }
}
