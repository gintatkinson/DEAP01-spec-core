#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Schema digest computation and JSON persistence matching `.pipeline/schema-digest.json`.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for SysML v2 schema digests.
//! ## 2. Invariants & Mathematical Properties
//! Computes FIPS 180-4 SHA-256 digests, AST node counts, and UAF operational projections deterministically.
//! ## 3. Preconditions
//! Caller supplies a valid `PackageDef` AST and canonical SysML v2 textual notation slice.
//! ## 4. Postconditions
//! Produces a serializable `SchemaDigest` model with deterministic `BTreeMap` node counts and operational descriptors.
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(V log V + B) over AST nodes `V` and source bytes `B`. Space Complexity: O(V) digest storage.
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Delegates SHA-256 to `deap_core::digest` and disk persistence to `write_atomic`.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603, REQ-SEMANTIC-DIGEST-SCHEMADIGEST, REQ-SEMANTIC-DIGEST-SCHEMADIGEST-COMPUTE]

pub use super::digest_counts::count_structural_elements;
pub(crate) use super::digest_counts::{collect_schema_nodes, compute_node_counts};
pub use super::digest_io::{compute_file_manifest, write_atomic, write_digest_atomic};
pub(crate) use super::digest_operational::{
    extract_operational_activities, extract_operational_exchanges, extract_operational_nodes,
    extract_operational_scenarios,
};

use deap_core::sysml_ast::PackageDef;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io;
use std::path::Path;

/// Individual file entry recorded in the schema digest file manifest.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `FileDigestEntry`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for `path`, `sha256`, `bytes`, `lines`, and `entities`.
/// ## 3. Preconditions
/// - Caller supplies normalized workspace-relative path and non-negative metrics.
/// ## 4. Postconditions
/// - Defines the authoritative `FileDigestEntry` record in `SchemaDigest::file_manifest`.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1). Space Complexity: O(L) over path string length.
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface / State / Action / Resource: Total value struct with `Serialize`/`Deserialize`.
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-FILEDIGESTENTRY] | SysML v2: PartDef(FileDigestEntry)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FileDigestEntry {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    pub lines: usize,
    pub entities: usize,
}

/// Operational activity descriptor in schema digest.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `OperationalActivity`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants across `id`, `name`, `doc`, `performer`, `inputs`, and `outputs`.
/// ## 3. Preconditions
/// - Derived from `ActionDef` AST nodes.
/// ## 4. Postconditions
/// - Defines the authoritative `OperationalActivity` representation.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1). Space Complexity: O(P) over parameter count `P`.
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface / State / Action / Resource: Total serializable descriptor.
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-OPERATIONALACTIVITY] | SysML v2: PartDef(OperationalActivity)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OperationalActivity {
    pub id: String,
    pub name: String,
    pub doc: String,
    pub performer: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

/// Operational exchange descriptor in schema digest.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `OperationalExchange`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants across `id`, `name`, `source_port`, `target_port`, `source_part`, `target_part`, and `protocol`.
/// ## 3. Preconditions
/// - Derived from `ConnectionDef` AST nodes.
/// ## 4. Postconditions
/// - Defines the authoritative `OperationalExchange` representation.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1). Space Complexity: O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface / State / Action / Resource: Total serializable descriptor.
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-OPERATIONALEXCHANGE] | SysML v2: PartDef(OperationalExchange)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OperationalExchange {
    pub id: String,
    pub name: String,
    pub source_port: String,
    pub target_port: String,
    pub source_part: String,
    pub target_part: String,
    pub protocol: String,
}

/// Operational scenario descriptor in schema digest.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `OperationalScenario`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants across `id`, `name`, `actor`, `objective`, and `steps`.
/// ## 3. Preconditions
/// - Derived from `UseCaseDef` AST nodes.
/// ## 4. Postconditions
/// - Defines the authoritative `OperationalScenario` representation.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1). Space Complexity: O(S) over step count `S`.
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface / State / Action / Resource: Total serializable descriptor.
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-OPERATIONALSCENARIO] | SysML v2: PartDef(OperationalScenario)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OperationalScenario {
    pub id: String,
    pub name: String,
    pub actor: String,
    pub objective: String,
    pub steps: Vec<String>,
}

/// Operational node descriptor in schema digest.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `OperationalNode`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants across `name`, `node_type`, `doc`, and `allocated_activities`.
/// ## 3. Preconditions
/// - Derived from `PartDef` AST nodes.
/// ## 4. Postconditions
/// - Defines the authoritative `OperationalNode` representation.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1). Space Complexity: O(A) over allocated activity count `A`.
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface / State / Action / Resource: Total serializable descriptor.
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-OPERATIONALNODE] | SysML v2: PartDef(OperationalNode)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OperationalNode {
    pub name: String,
    pub node_type: String,
    pub doc: String,
    pub allocated_activities: Vec<String>,
}

/// Authoritative schema digest model written to `.pipeline/schema-digest.json`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `SchemaDigest`.
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic `BTreeMap` ordering for `node_counts` and sorted unique `schema_nodes`.
/// ## 3. Preconditions
/// - Computed via `SchemaDigest::compute` or deserialized from `.pipeline/schema-digest.json`.
/// ## 4. Postconditions
/// - Defines the authoritative `SchemaDigest` representation.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) serialization. Space Complexity: O(N) over AST inventory.
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface / State / Action / Resource: Total serializable struct with `#[serde(default)]` backward compatibility.
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-SCHEMADIGEST] | SysML v2: PartDef(SchemaDigest)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SchemaDigest {
    pub sha256: String,
    pub total_lines: usize,
    pub node_counts: BTreeMap<String, usize>,
    pub schema_nodes: Vec<String>,
    #[serde(default)]
    pub operational_activities: Vec<OperationalActivity>,
    #[serde(default)]
    pub operational_exchanges: Vec<OperationalExchange>,
    #[serde(default)]
    pub operational_scenarios: Vec<OperationalScenario>,
    #[serde(default)]
    pub operational_nodes: Vec<OperationalNode>,
    #[serde(default)]
    pub file_manifest: Vec<FileDigestEntry>,
}

impl SchemaDigest {
    /// Compute schema digest from a `PackageDef` and its compiled SysML textual bytes.
    /// Realises: [REQ-SEMANTIC-DIGEST-SCHEMADIGEST-COMPUTE] | SysML v2: ActionDef(SchemaDigest::compute)
    pub fn compute(pkg: &PackageDef, sysml_text: &str) -> Self {
        let sha256 = compute_sha256(sysml_text.as_bytes());
        let total_lines = sysml_text.lines().count();
        let node_counts = compute_node_counts(pkg);
        let schema_nodes = collect_schema_nodes(pkg);

        let operational_activities = extract_operational_activities(pkg);
        let operational_exchanges = extract_operational_exchanges(pkg);
        let operational_scenarios = extract_operational_scenarios(pkg);
        let operational_nodes = extract_operational_nodes(pkg);

        Self {
            sha256,
            total_lines,
            node_counts,
            schema_nodes,
            operational_activities,
            operational_exchanges,
            operational_scenarios,
            operational_nodes,
            file_manifest: Vec::new(),
        }
    }

    /// Builder method to attach a file manifest to the schema digest.
    /// Realises: [REQ-SEMANTIC-DIGEST-SCHEMADIGEST-WITH-FILE-MANIFEST] | SysML v2: ActionDef(SchemaDigest::with_file_manifest)
    pub fn with_file_manifest(mut self, manifest: Vec<FileDigestEntry>) -> Self {
        self.file_manifest = manifest;
        self
    }

    /// Atomically write digest JSON to disk using a temporary sibling file and rename.
    /// Realises: [REQ-SEMANTIC-DIGEST-SCHEMADIGEST-WRITE-TO-FILE] | SysML v2: ActionDef(SchemaDigest::write_to_file)
    pub fn write_to_file(&self, path: &Path) -> Result<(), io::Error> {
        let json_text = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        write_atomic(path, &json_text)
    }
}

/// Compute schema digest from a `PackageDef` and its compiled SysML textual bytes.
/// Realises: [REQ-SEMANTIC-DIGEST-GENERATE-DIGEST] | SysML v2: ActionDef(generate_digest)
pub fn generate_digest(pkg: &PackageDef, sysml_text: &str) -> SchemaDigest {
    SchemaDigest::compute(pkg, sysml_text)
}

/// Compute SHA-256 hash formatted as lowercase hexadecimal (FIPS 180-4).
/// Realises: [REQ-SEMANTIC-DIGEST-COMPUTE-SHA256] | SysML v2: ActionDef(compute_sha256)
pub fn compute_sha256(bytes: &[u8]) -> String {
    deap_core::digest::compute_sha256(bytes)
}
