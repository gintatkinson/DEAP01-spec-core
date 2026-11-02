#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Lossless Rust CST inspection and virtual filesystem integration. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D qualification standards.
//! Exposes lossless concrete syntax tree parsing and virtual filesystem bindings for DEAP.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Module re-exports are deterministic, pure, and free of global mutable state.
//! - Filesystem interactions preserve cross-platform LF line endings and normalized paths.
//!
//! ## 3. Preconditions
//! - Callers provide valid UTF-8 paths and source buffers to exported submodules.
//!
//! ## 4. Postconditions
//! - Re-exports lossless VFS types and helpers without unhandled panic paths.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - $O(1)$ compile-time re-export resolution and zero static initialization overhead.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Submodules fail closed via structured `CoreResult` or `Result` types.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0608], [SC-01], [FLS: fls_cst_vfs_mod]
//! SysML v2: part def RustCstSubsystem

pub mod diagnostics_ndjson;
pub mod doc_edit;
pub mod extract_module;
pub mod items;
pub mod parse;
pub mod vfs;

pub use diagnostics_ndjson::{
    apply_machine_applicable_fixes, emit_ndjson_line, render_ariadne_diagnostic,
    NdjsonDiagnostic, NdjsonSpan,
};
pub use doc_edit::{apply_non_overlapping_edits, classify_file_doc_tier, compact_tier_bc_docstrings, DocTier};
pub use extract_module::{extract_rust_module, rewrite_blueprint_line_refs, ExtractedModulePlan};
pub use items::{list_rust_items, RustCstItem, RustItemKind};
pub use parse::ParsedRustFile;
pub use vfs::{normalize_lf, normalize_rel_path, WorkspaceVfs};
