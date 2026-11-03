#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! [DOC-TIER: A] Deterministic AST-to-CommonMark Projection Engine (`BP-04` / `#751`).
//! ## 1. Safety Intent & Regulatory Scope
//! Safety & Traceability: Realises: [REQ-0603], [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
//! ## 2. Invariants & Mathematical Properties
//! Contract: Consolidates Agile (`EPIC`, `FEAT`, `UC`, `US`), Logical ICD (`ICD_01`, `ICD_02`),
//! MIL-STD-881E WBS (`WBS_01..03`), Reverse Sync, and 10-pillar STPA/FMECA/RTA/SLDV projections
//! behind `ForwardCompilerDb`, `pulldown_cmark::Event`, `pulldown_cmark_to_cmark::cmark`, and `toml_edit`.
//! ## 3. Preconditions
//! ## 4. Postconditions
//! ## 5. Algorithmic Complexity Bounds
//! ## 6. Failure Modes & Fault Tolerance
//! Bounds & Faults: Operates in O(N log N) time and O(N) space; fails closed via typed `CompilerError`.
//! ## 7. Realization & Traceability

pub mod agile;
pub mod cmark_helpers;
pub mod forward;
pub mod icd;
pub mod reverse;
pub mod stpa;
pub mod wbs;

pub use cmark_helpers::{
    cmark_table_events, emit_lossless_markdown, normalize_line_endings, parse_frontmatter_metadata,
    parse_markdown_tables, parse_task_list_items, render_cmark_table, write_spec_file_atomic,
    CmarkTable, FrontmatterDoc, TaskListItem,
};
pub use forward::{
    forward_sync_sysml_to_specs, ForwardSyncOptions, IcdDocumentPair, SignalDictionaryRow,
    WbsExportBundle, WbsNode,
};
pub use reverse::{
    reverse_sync_specs_to_sysml, reverse_sync_specs_to_sysml_with_dry_run, ReverseSyncOptions,
};
pub use stpa::{
    compile_stpa_to_constraints, compile_stpa_to_constraints_filtered, emit_safety_suite,
    expand_cartesian_stpa, generate_fmeca_matrix, transpile_safety_suite, transpile_stpa, FmecaRow,
    FmecaScoringConfig, UnsafeControlAction,
};

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use deap_core::sysml_cst::PackageDef;
use serde::{Deserialize, Serialize};

/// Typed compilation and projection error variants for `BP-04` (`#751`).
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Classifies empty models, cyclic WBS graphs, orphan requirements, frontmatter syntax, cmark serialization, and atomic I/O faults.
/// Bounds & Faults: Value enum with deterministic `Display` and `std::error::Error` implementations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompilerError {
    EmptyModel { package: String },
    CyclicWbsDependency { wbs_code: String },
    OrphanRequirement { req_id: String },
    MalformedFrontmatter { detail: String },
    CmarkSerialization(String),
    AtomicWriteError { path: PathBuf, detail: String },
}

impl std::fmt::Display for CompilerError {
    /// Formats `CompilerError` into a deterministic diagnostic string.
    /// Safety & Traceability: Realises: [REQ-0604]
    /// Contract: Produces a non-empty human-readable description for every variant.
    /// Bounds & Faults: O(M) in message length; infallible formatting unless formatter errors.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyModel { package } => {
                write!(f, "SysML package '{package}' contains zero projectable AST nodes")
            }
            Self::CyclicWbsDependency { wbs_code } => {
                write!(f, "Cyclic WBS dependency detected at node '{wbs_code}'")
            }
            Self::OrphanRequirement { req_id } => {
                write!(f, "Requirement '{req_id}' has no realization binding")
            }
            Self::MalformedFrontmatter { detail } => {
                write!(f, "Malformed frontmatter metadata: {detail}")
            }
            Self::CmarkSerialization(msg) => write!(f, "CommonMark serialization failed: {msg}"),
            Self::AtomicWriteError { path, detail } => {
                write!(f, "Atomic write failed for '{}': {detail}", path.display())
            }
        }
    }
}

impl std::error::Error for CompilerError {}

/// Projected Agile specification suite partitioned by artifact category (`BP-04` Section 6.1).
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Maps deterministic relative file paths to normalized LF Markdown contents across Epics, Features, Use Cases, and User Stories.
/// Bounds & Faults: O(N) memory proportional to rendered specification count.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SpecSuiteBundle {
    pub epics: BTreeMap<String, String>,
    pub features: BTreeMap<String, String>,
    pub use_cases: BTreeMap<String, String>,
    pub user_stories: BTreeMap<String, String>,
}

/// Demand-driven forward compiler query database interface (`BP-04` Section 6.1).
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Provides pure memoizable queries for schema digest, parsed AST, Agile spec suite, Logical ICD, and MIL-STD-881E WBS.
/// Bounds & Faults: Implementations must be deterministic and free of wall-clock dependencies.
pub trait ForwardCompilerDb {
    /// Returns the hex-encoded SHA-256 digest of the active SysML v2 schema package.
    /// Safety & Traceability: Realises: [REQ-0604]
    /// Contract: Deterministic 64-character lowercase hex digest wrapped in `Arc`.
    /// Bounds & Faults: O(|sysml|) time and O(1) space.
    fn schema_digest_hex(&self) -> Arc<String>;

    /// Returns the validated `PackageDef` AST root or `CompilerError::EmptyModel`.
    /// Safety & Traceability: Realises: [REQ-0604]
    /// Contract: Rejects empty packages having zero parts, packages, actions, use cases, requirements, and constraints.
    /// Bounds & Faults: O(1) clone of `Arc<PackageDef>`.
    fn parsed_sysml_package(&self) -> Result<Arc<PackageDef>, CompilerError>;

    /// Projects the complete Agile specification suite (`EPIC`, `FEAT`, `UC`, `US`).
    /// Safety & Traceability: Realises: [REQ-0604]
    /// Contract: Delegates to `project_agile_spec_suite`.
    /// Bounds & Faults: O(N log N) time and O(N) space.
    fn project_agile_spec_suite(&self) -> Result<Arc<SpecSuiteBundle>, CompilerError>;

    /// Compiles `ICD_01_SYSTEM_INTERFACE_MATRIX.md` and `ICD_02_MASTER_SIGNAL_DICTIONARY.md`.
    /// Safety & Traceability: Realises: [REQ-0604]
    /// Contract: Delegates to `compile_logical_icd_matrix`.
    /// Bounds & Faults: O(P^2 + C) time and space.
    fn compile_logical_icd_matrix(&self) -> Result<Arc<IcdDocumentPair>, CompilerError>;

    /// Synthesizes MIL-STD-881E `WBS_01..03` and enterprise CSV/JSON exports.
    /// Safety & Traceability: Realises: [REQ-0604]
    /// Contract: Delegates to `synthesize_mil_std_881e_wbs`.
    /// Bounds & Faults: O(W log W) time and O(W) space.
    fn synthesize_mil_std_881e_wbs(&self) -> Result<Arc<WbsExportBundle>, CompilerError>;
}

/// In-memory `ForwardCompilerDb` implementation backed by a parsed `PackageDef` and SHA-256 digest.
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Evaluates all `BP-04` forward projection queries deterministically in memory.
/// Bounds & Faults: O(N) memory for stored AST and digest.
#[derive(Debug, Clone)]
pub struct InMemoryForwardCompilerDb {
    pkg: Arc<PackageDef>,
    digest_hex: Arc<String>,
}

impl InMemoryForwardCompilerDb {
    /// Constructs a new `InMemoryForwardCompilerDb` from a parsed `PackageDef`.
    /// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
    /// Contract: Computes the canonical SysML SHA-256 digest via `crate::salsa_db::generate_digest`.
    /// Bounds & Faults: O(N) time and space in AST node count N.
    #[must_use]
    pub fn new(pkg: PackageDef) -> Self {
        let sysml = crate::salsa_db::to_sysml(&pkg);
        let digest = crate::salsa_db::generate_digest(&pkg, &sysml);
        Self {
            pkg: Arc::new(pkg),
            digest_hex: Arc::new(digest.sha256),
        }
    }
}

impl ForwardCompilerDb for InMemoryForwardCompilerDb {
    /// Returns the SHA-256 hex digest of the canonical SysML v2 schema (`Realises: [REQ-0604]`).
    fn schema_digest_hex(&self) -> Arc<String> {
        Arc::clone(&self.digest_hex)
    }

    /// Returns the validated `PackageDef` AST (`Realises: [REQ-0604]`).
    fn parsed_sysml_package(&self) -> Result<Arc<PackageDef>, CompilerError> {
        let p = &self.pkg;
        if p.part_defs.is_empty()
            && p.packages.is_empty()
            && p.action_defs.is_empty()
            && p.use_case_defs.is_empty()
            && p.requirement_defs.is_empty()
            && p.constraint_defs.is_empty()
        {
            return Err(CompilerError::EmptyModel { package: p.name.clone() });
        }
        Ok(Arc::clone(&self.pkg))
    }

    /// Projects the Agile specification suite (`Realises: [REQ-0604]`).
    fn project_agile_spec_suite(&self) -> Result<Arc<SpecSuiteBundle>, CompilerError> {
        project_agile_spec_suite(self)
    }

    /// Compiles the Level 1C logical ICD matrix (`Realises: [REQ-0604]`).
    fn compile_logical_icd_matrix(&self) -> Result<Arc<IcdDocumentPair>, CompilerError> {
        compile_logical_icd_matrix(self)
    }

    /// Synthesizes the MIL-STD-881E WBS export bundle (`Realises: [REQ-0604]`).
    fn synthesize_mil_std_881e_wbs(&self) -> Result<Arc<WbsExportBundle>, CompilerError> {
        synthesize_mil_std_881e_wbs(self)
    }
}

/// Projects the Agile specification suite (`EPIC`, `FEAT`, `UC`, `US`) from a `ForwardCompilerDb`.
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Invokes `forward_sync_sysml_to_specs` in dry-run mode and partitions outputs into `SpecSuiteBundle`.
/// Bounds & Faults: O(N log N) time and O(N) space; fails closed on `CompilerError::EmptyModel`.
pub fn project_agile_spec_suite(
    db: &dyn ForwardCompilerDb,
) -> Result<Arc<SpecSuiteBundle>, CompilerError> {
    let pkg = db.parsed_sysml_package()?;
    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/model.sysml"),
        docs_dir: PathBuf::from("docs"),
        out_dir: None,
        dry_run: true,
        force: true,
    };
    let all = forward_sync_sysml_to_specs(&pkg, &opts).map_err(CompilerError::CmarkSerialization)?;
    let mut bundle = SpecSuiteBundle::default();
    for (rel_path, content) in all {
        if rel_path.starts_with("epics/") {
            bundle.epics.insert(rel_path, content);
        } else if rel_path.starts_with("features/") {
            bundle.features.insert(rel_path, content);
        } else if rel_path.starts_with("use-cases/") {
            bundle.use_cases.insert(rel_path, content);
        } else if rel_path.starts_with("user-stories/") {
            bundle.user_stories.insert(rel_path, content);
        }
    }
    Ok(Arc::new(bundle))
}

/// Compiles the Logical ICD matrix and master signal dictionary from a `ForwardCompilerDb`.
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Delegates to `icd::try_compile_logical_icd_artifacts` after validating non-empty package.
/// Bounds & Faults: O(P^2 + C) time and space.
pub fn compile_logical_icd_matrix(
    db: &dyn ForwardCompilerDb,
) -> Result<Arc<IcdDocumentPair>, CompilerError> {
    let pkg = db.parsed_sysml_package()?;
    let pair =
        icd::try_compile_logical_icd_artifacts(&pkg).map_err(CompilerError::CmarkSerialization)?;
    Ok(Arc::new(pair))
}

/// Synthesizes the MIL-STD-881E WBS bundle from a `ForwardCompilerDb`.
/// Safety & Traceability: Realises: [REQ-0604], [REQ-BP04-CMARK-PROJECTION]
/// Contract: Delegates to `wbs::try_compile_wbs_artifacts` after validating non-empty package.
/// Bounds & Faults: O(W log W) time and O(W) space.
pub fn synthesize_mil_std_881e_wbs(
    db: &dyn ForwardCompilerDb,
) -> Result<Arc<WbsExportBundle>, CompilerError> {
    let pkg = db.parsed_sysml_package()?;
    let bundle =
        wbs::try_compile_wbs_artifacts(&pkg).map_err(CompilerError::CmarkSerialization)?;
    Ok(Arc::new(bundle))
}
