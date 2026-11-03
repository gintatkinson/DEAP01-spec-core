#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Repository topology, manifest governance, and Clippy total-function gate. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C qualification.
//! Unifies cargo tooling, workspace topology checks, and stable Clippy gates.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Re-exports pure deterministic workspace queries and manifest validators.
//! - Enforces strict containment and inheritance of workspace configuration settings.
//!
//! ## 3. Preconditions
//! - Callers provide valid workspace paths to verification entry points.
//!
//! ## 4. Postconditions
//! - Exposes cohesive workspace inspection surface failing closed on governance violations.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - O(1) compile-time resolution and constant dispatch overhead.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Pure functions return typed CoreError or ClippyGateError diagnostics.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0601], [REQ-0609], [REQ-WS-MODULE]
//! SysML v2: part def WorkspaceInspector

pub mod cargo_tooling;
pub mod checks;
pub mod clippy_gate;
pub mod dylint_gate;
pub mod ferrocene_gate;
pub mod lock_governor;
pub mod ratchet_table;

#[cfg(test)]
#[path = "workspace_tests.rs"]
mod tests;

pub use cargo_tooling::{
    evaluate_cargo_platform, load_cargo_lockfile, parse_cargo_toml_lossless,
    resolve_cargo_metadata, set_workspace_dependency_version,
};
pub use checks::{
    check_downstream_instructions_exist, check_gitignore_exists, check_no_ds_store_files,
    check_no_duplicate_master_blueprints, check_reconcile_backlog_tooling_exists,
    check_upstream_template_clean_landing_zones, is_upstream_compiler,
};
pub use clippy_gate::{
    filter_clippy_ndjson_stream, run_stable_clippy_gate, validate_manifest_and_clippy_bundle,
    ClippyGateError, ClippyNdjsonFinding, ManifestBundleDigest, StableClippyGateConfig,
    StageGateReceipt, MANDATORY_DENIED_CLIPPY_LINTS,
};
pub use dylint_gate::{
    audit_source_in_process, probe_executable_in_path, run_stage2_dylint_gate,
    verify_workspace_exclude, DylintFinding, DylintGateError, Stage2DylintConfig,
    Stage2DylintReceipt,
};
pub use ferrocene_gate::{
    probe_ferrocene_driver, run_stage5_ferrocene_gate, verify_certified_core_and_coverage,
    verify_dual_toolchain_parity, verify_fls_traceability,
    verify_three_ring_coupling_and_monitor, CoverageSummary, DualToolchainSpec,
    FerroceneGateError, SafetyMonitorEnvelope, Stage5FerroceneReceipt,
};
pub use lock_governor::{
    parse_locked_manifest_snapshot, verify_pinned_cst_dependency_graph, LockManifestSnapshot,
    LockedPackageRecord, OfflineLockGovernanceError, PackageSourceKind,
};
