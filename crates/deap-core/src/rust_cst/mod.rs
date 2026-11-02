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
pub mod doc_tier;
pub mod envelope;
pub mod extract_module;
pub mod gate;
pub mod items;
pub mod kani_proofs;
pub mod metrics;
pub mod parse;
pub mod source_bridge;
pub mod source_bridge_helpers;
pub mod source_bridge_rust;
pub mod thresholds;
pub mod verified_cst;
pub mod vfs;

#[cfg(test)]
mod cst_tests;

pub use diagnostics_ndjson::{
    apply_machine_applicable_fixes, emit_ndjson_line, render_ariadne_diagnostic,
    NdjsonDiagnostic, NdjsonSpan,
};
pub use doc_edit::{apply_non_overlapping_edits, classify_file_doc_tier, compact_tier_bc_docstrings};
pub use doc_tier::{
    classify_cfg_test_excluded_doc_tier, ClassifiedDocTierItem, DocTier, DocTierInventory,
};
pub use envelope::{
    verify_analyzer_envelope_and_seal_receipt, CstComplexityEnvelope, LockManifestSnapshot,
    LockedPackageRecord, OfflineLockGovernanceError, PackageSourceKind,
    SupplyChainVerificationReceipt,
};
pub use extract_module::{extract_rust_module, rewrite_blueprint_line_refs, ExtractedModulePlan};
pub use gate::{
    evaluate_metrics, evaluate_metrics_and_format_ndjson, run_offline_locked_workspace_gate,
};
pub use items::{list_rust_items, RustCstItem, RustItemKind};
pub use kani_proofs::{
    evaluate_stage4_kani, guard_inbound_frb_f64, probe_formal_driver,
    synthesize_kani_proof_module, FrbGuardError, FrbPortGuardSpec, KaniProofHarnessSpec,
    KaniStageReceipt,
};
pub use metrics::{
    analyze_cst_v2_metrics, extract_callable_v2_metrics, DeapV2AstMetricsReport, FnAstMetrics,
};
pub use parse::ParsedRustFile;
pub use thresholds::{validate_metrics_thresholds, CstMetricsError, CstMetricsThresholds};
pub use verified_cst::{parse_verified_lossless_cst, VerifiedLosslessCst};
pub use vfs::{normalize_lf, normalize_rel_path, WorkspaceVfs};
