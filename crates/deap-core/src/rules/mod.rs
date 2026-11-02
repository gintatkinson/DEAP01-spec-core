//! Manifest rules and codebase configuration parsing.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C safety-critical software
//! engineering standards. Provides deterministic workspace rule models, master blueprint
//! catalogs, and codebase configuration deserialization.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Re-exports typed catalogs and configuration structures with deterministic key ordering.
//!
//! ## 3. Preconditions
//! - Callers provide valid UTF-8 filesystem paths and JSON strings.
//!
//! ## 4. Postconditions
//! - Exposes cohesive configuration parsing surface failing closed on corrupt schemas.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(1) compile-time catalog queries; O(N log K) JSON parsing.
//! - Space Complexity: O(1) catalogs; O(N) configuration buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Manifest errors propagate fail-closed via explicit `Result` returns without panicking.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-RULES-MODULE], [DO-330-TQL-4-RULES].

pub mod canonical_blueprints;
pub mod codebase;
pub mod excluded_dirs;
pub mod master_blueprints;

pub use canonical_blueprints::{
    CanonicalArchitectureBlueprint, CanonicalArchitectureBlueprintsCatalog,
    CANONICAL_ARCHITECTURE_BLUEPRINTS,
};
pub use codebase::{CodebaseRules, MetaConfig, TrackerRules, ValidationRules};
pub use excluded_dirs::{ExcludedDir, ExcludedDirsCatalog, EXCLUDED_DIRS};
pub use master_blueprints::{MasterBlueprint, MasterBlueprintsCatalog, MASTER_BLUEPRINTS};

#[cfg(test)]
mod tests;
