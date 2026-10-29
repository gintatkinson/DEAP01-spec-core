//! Flutter contract error, options, report, and topology node data structures.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A and DO-330 TQL-4 contract codegen data governance.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Closed error algebra and deterministic `BTreeMap` property ordering.
//!
//! ## 3. Preconditions
//! - Caller supplies valid path buffers and normalized JSON entity attributes.
//!
//! ## 4. Postconditions
//! - Exposes typed options, report, error, and topology contracts for synthesis.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(1)$ per type constructor or error conversion.
//! - Space Complexity: $O(P)$ bounded by path buffers and node property entries.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Fail-closed error variants preserve root-cause `io` and `serde_json` diagnostics.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607], [REQ-EMIT-FLUTTER-CONTRACTS]

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

/// Error types emitted during Flutter contract code generation.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 diagnostic traceability controls.
/// Provides exhaustive, strongly-typed error enumeration for file I/O, JSON decoding,
/// and model constraint validation failures.
///
/// ## 2. Invariants & Mathematical Properties
/// - Closed enumeration: every potential failure condition maps to a distinct variant.
/// - Preserves root cause diagnostics through owned error representations.
///
/// ## 3. Preconditions
/// - Instantiated upon encountering unrecoverable I/O, parsing, or structural schema defects.
///
/// ## 4. Postconditions
/// - Supplies structured error messages suitable for CLI reporting and baseline verification.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ variant construction.
/// - Space Complexity: $O(M)$ proportional to internal error message length.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Implements `std::error::Error` for standard error propagation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FLUTTER-CODEGEN-ERROR-ENUM]
#[derive(Debug)]
pub enum FlutterCodegenError {
    /// Filesystem I/O error while reading schemas or writing generated files.
    Io(std::io::Error),
    /// JSON serialization or deserialization failure.
    Json(serde_json::Error),
    /// Semantic or structural validation failure within the input SysML model.
    InvalidModel(String),
}

impl fmt::Display for FlutterCodegenError {
    /// Formats the codegen error for display output.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and DO-330 TQL-4 diagnostic formatting standards.
    /// Emits structured error messages to standard output and diagnostic logs.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Pure formatting function: does not mutate error state or allocate unbounded memory.
    ///
    /// ## 3. Preconditions
    /// - `self` is a valid `FlutterCodegenError`; `f` is an initialized `fmt::Formatter`.
    ///
    /// ## 4. Postconditions
    /// - Writes formatted representation to `f` and returns `fmt::Result`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(M)$ linear in error message length.
    /// - Space Complexity: $O(1)$ stream output.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Formatter errors are propagated via `fmt::Result`.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FLUTTER-CODEGEN-ERROR-FMT]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error during Flutter codegen: {}", err),
            Self::Json(err) => write!(f, "JSON error during Flutter codegen: {}", err),
            Self::InvalidModel(msg) => write!(f, "Invalid model for Flutter codegen: {}", msg),
        }
    }
}

impl std::error::Error for FlutterCodegenError {
    /// Returns the underlying source error cause if available.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A root-cause traceability standards.
    /// Allows error chaining for nested I/O and JSON deserialization failures.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Injective extraction of internal error references.
    ///
    /// ## 3. Preconditions
    /// - `self` is an initialized `FlutterCodegenError`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Some(&err)` for `Io` and `Json` variants, or `None` for `InvalidModel`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$ field accessor.
    /// - Space Complexity: $O(1)$ reference return.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible reference accessor.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FLUTTER-CODEGEN-ERROR-SOURCE]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Json(err) => Some(err),
            Self::InvalidModel(_) => None,
        }
    }
}

impl From<std::io::Error> for FlutterCodegenError {
    /// Converts a standard I/O error into a `FlutterCodegenError`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A fail-closed I/O error containment.
    /// Wraps filesystem read and write errors in `FlutterCodegenError::Io`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Lossless conversion preserving the underlying `std::io::Error`.
    ///
    /// ## 3. Preconditions
    /// - `err` is a standard filesystem I/O error.
    ///
    /// ## 4. Postconditions
    /// - Returns `FlutterCodegenError::Io(err)`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$ variant wrapping.
    /// - Space Complexity: $O(1)$ move semantics.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible error conversion.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FLUTTER-CODEGEN-ERROR-FROM-IO]
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for FlutterCodegenError {
    /// Converts a JSON serialization error into a `FlutterCodegenError`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A JSON parsing defect containment.
    /// Wraps topology and contract serialization errors in `FlutterCodegenError::Json`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves exact syntax error line and column offsets from `serde_json`.
    ///
    /// ## 3. Preconditions
    /// - `err` is a `serde_json::Error` instance.
    ///
    /// ## 4. Postconditions
    /// - Returns `FlutterCodegenError::Json(err)`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: $O(1)$ move constructor.
    /// - Space Complexity: $O(1)$ zero additional allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Infallible error wrapping.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FLUTTER-CODEGEN-ERROR-FROM-JSON]
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

/// Execution options for deterministic Flutter and C-ABI contract generation.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 configuration parameter controls.
/// Encapsulates destination directories, schema file paths, and topology dataset inputs.
///
/// ## 2. Invariants & Mathematical Properties
/// - Value semantics: immutable configuration snapshot for a single compilation run.
///
/// ## 3. Preconditions
/// - Caller provides a valid `out_dir` path.
///
/// ## 4. Postconditions
/// - Establishes input and output coordinates for contract artifact emission.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ struct instantiation.
/// - Space Complexity: $O(P)$ proportional to path buffer sizes.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Invalid path strings are captured when filesystem operations are attempted.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FLUTTER-CONTRACT-OPTIONS]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FlutterContractOptions {
    /// Base output directory (typically `app_flutter/` or workspace root).
    pub out_dir: PathBuf,
    /// Optional explicit path to SysML schema file.
    pub schema_path: Option<PathBuf>,
    /// Optional explicit path to topology dataset JSON.
    pub topology_path: Option<PathBuf>,
}

/// Comprehensive execution report summarizing synthesized Flutter contract artifacts.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and DO-330 TQL-4 verification artifact audit standards.
/// Records exact filesystem targets, entity metrics, and the computed schema hash.
///
/// ## 2. Invariants & Mathematical Properties
/// - Exact provenance: every output path corresponds to a file created during execution.
/// - Cryptographic hash agreement: `schema_hash` matches embedded constant in generated code.
///
/// ## 3. Preconditions
/// - Populated exclusively upon successful completion of contract generation.
///
/// ## 4. Postconditions
/// - Conveys empirical artifact metrics and verification hashes to callers.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ report construction.
/// - Space Complexity: $O(1)$ fixed report structure.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Read-only summary record.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FLUTTER-CONTRACT-REPORT]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FlutterContractReport {
    /// Absolute or relative path to `generated_domain_seed.g.dart`.
    pub seed_path: PathBuf,
    /// Absolute or relative path to `generated_schema_models.g.dart`.
    pub models_path: PathBuf,
    /// Absolute or relative path to `solver_bridge.g.dart`.
    pub solver_bridge_path: PathBuf,
    /// Absolute or relative path to `schema_ui_contract_test.g.dart`.
    pub ui_contract_test_path: PathBuf,
    /// 64-bit cryptographic schema hash derived from SHA-256 digest.
    pub schema_hash: u64,
    /// Total count of concrete entity instances emitted into seeds and tests.
    pub entities_count: usize,
    /// Total count of part definition metamodel types emitted into models.
    pub types_count: usize,
}

/// In-memory representation of an individual topology entity node.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A data normalization and schema independence requirements.
/// Stores normalized fields parsed dynamically from arbitrary topology datasets.
///
/// ## 2. Invariants & Mathematical Properties
/// - Value object holding string identifiers and a sorted property map.
///
/// ## 3. Preconditions
/// - Populated via JSON parsing with non-empty `id`.
///
/// ## 4. Postconditions
/// - Provides structured access to entity attributes for code generation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(1)$ constructor.
/// - Space Complexity: $O(P)$ bounded by number of properties.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Default values supplied for missing optional attributes.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FLUTTER-TOPOLOGY-NODE]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TopologyNode {
    /// Unique entity instance identifier.
    pub id: String,
    /// Associated metamodel part definition type name.
    pub type_name: String,
    /// Descriptive human-readable label.
    pub label: String,
    /// Optional parent entity identifier for hierarchical containment.
    pub parent_id: Option<String>,
    /// Key-value property bag.
    pub properties: BTreeMap<String, serde_json::Value>,
    /// Epoch creation timestamp in milliseconds.
    pub created_at: i64,
    /// Epoch update timestamp in milliseconds.
    pub updated_at: i64,
}
