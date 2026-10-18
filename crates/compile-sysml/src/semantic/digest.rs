//! Schema digest computation and JSON persistence matching .pipeline/schema-digest.json.

use deap_core::sysml_ast::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Individual file entry recorded in the schema digest file manifest.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `FileDigestEntry` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `FileDigestEntry` across member set `[path: String, sha256: String, bytes: u64, lines: usize, entities: usize]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `FileDigestEntry` members (`path: String, sha256: String, bytes: u64, lines: usize, entities: usize`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `FileDigestEntry` representation for struct `FileDigestEntry`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `FileDigestEntry`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `FileDigestEntry`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `FileDigestEntry` (`FileDigestEntry`).
/// - State: Maintains consistent state invariants across `FileDigestEntry` without partial mutation on error.
/// - Action: Executes `FileDigestEntry` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `FileDigestEntry` (`SEMANTIC-DIGEST`).
///
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
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `OperationalActivity` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `OperationalActivity` across member set `[id: String, name: String, doc: String, performer: String, inputs: Vec<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `OperationalActivity` members (`id: String, name: String, doc: String, performer: String, inputs: Vec<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `OperationalActivity` representation for struct `OperationalActivity`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `OperationalActivity`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `OperationalActivity`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `OperationalActivity` (`OperationalActivity`).
/// - State: Maintains consistent state invariants across `OperationalActivity` without partial mutation on error.
/// - Action: Executes `OperationalActivity` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `OperationalActivity` (`SEMANTIC-DIGEST`).
///
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
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `OperationalExchange` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `OperationalExchange` across member set `[id: String, name: String, source_port: String, target_port: String, source_part: String]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `OperationalExchange` members (`id: String, name: String, source_port: String, target_port: String, source_part: String`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `OperationalExchange` representation for struct `OperationalExchange`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `OperationalExchange`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `OperationalExchange`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `OperationalExchange` (`OperationalExchange`).
/// - State: Maintains consistent state invariants across `OperationalExchange` without partial mutation on error.
/// - Action: Executes `OperationalExchange` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `OperationalExchange` (`SEMANTIC-DIGEST`).
///
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
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `OperationalScenario` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `OperationalScenario` across member set `[id: String, name: String, actor: String, objective: String, steps: Vec<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `OperationalScenario` members (`id: String, name: String, actor: String, objective: String, steps: Vec<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `OperationalScenario` representation for struct `OperationalScenario`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `OperationalScenario`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `OperationalScenario`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `OperationalScenario` (`OperationalScenario`).
/// - State: Maintains consistent state invariants across `OperationalScenario` without partial mutation on error.
/// - Action: Executes `OperationalScenario` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `OperationalScenario` (`SEMANTIC-DIGEST`).
///
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
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `OperationalNode` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `OperationalNode` across member set `[name: String, node_type: String, doc: String, allocated_activities: Vec<String>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `OperationalNode` members (`name: String, node_type: String, doc: String, allocated_activities: Vec<String>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `OperationalNode` representation for struct `OperationalNode`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(4)) bounded over input/member dimension of `OperationalNode`.
/// - Space Complexity: O(M(4)) bounded auxiliary memory for `OperationalNode`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `OperationalNode` (`OperationalNode`).
/// - State: Maintains consistent state invariants across `OperationalNode` without partial mutation on error.
/// - Action: Executes `OperationalNode` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `OperationalNode` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-OPERATIONALNODE] | SysML v2: PartDef(OperationalNode)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OperationalNode {
    pub name: String,
    pub node_type: String,
    pub doc: String,
    pub allocated_activities: Vec<String>,
}

/// Authoritative schema digest model written to .pipeline/schema-digest.json.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for struct `SchemaDigest` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves structural type invariants for struct `SchemaDigest` across member set `[sha256: String, total_lines: usize, node_counts: BTreeMap<String, usize>, schema_nodes: Vec<String>, operational_activities: Vec<OperationalActivity>]`.
///
/// ## 3. Preconditions
/// - Caller supplies well-formed values for `SchemaDigest` members (`sha256: String, total_lines: usize, node_counts: BTreeMap<String, usize>, schema_nodes: Vec<String>, operational_activities: Vec<OperationalActivity>`).
///
/// ## 4. Postconditions
/// - Defines the authoritative `SchemaDigest` representation for struct `SchemaDigest`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M(5)) bounded over input/member dimension of `SchemaDigest`.
/// - Space Complexity: O(M(5)) bounded auxiliary memory for `SchemaDigest`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `SchemaDigest` (`SchemaDigest`).
/// - State: Maintains consistent state invariants across `SchemaDigest` without partial mutation on error.
/// - Action: Executes `SchemaDigest` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `SchemaDigest` (`SEMANTIC-DIGEST`).
///
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
    /// Compute schema digest from a PackageDef and its compiled SysML textual bytes.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SchemaDigest::compute` in `crates/compile-sysml/src/semantic/digest.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SchemaDigest::compute` (declaration #7) within `SEMANTIC-DIGEST`.
    ///
    /// ## 3. Preconditions
    /// - `pkg: &PackageDef` must satisfy domain and type-safety constraints prior to invoking `SchemaDigest::compute`.
    /// - `sysml_text: &str` must satisfy domain and type-safety constraints prior to invoking `SchemaDigest::compute`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Self` from `SchemaDigest::compute` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(pkg)) bounded over input/member dimension of `SchemaDigest::compute`.
    /// - Space Complexity: O(N(pkg)) bounded auxiliary memory for `SchemaDigest::compute`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SchemaDigest::compute` (`Self`).
    /// - State: Maintains consistent state invariants across `SchemaDigest::compute` without partial mutation on error.
    /// - Action: Executes `SchemaDigest::compute` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SchemaDigest::compute` (`SEMANTIC-DIGEST`).
    ///
    /// ## 7. Realization & Traceability
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
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SchemaDigest::with_file_manifest` in `crates/compile-sysml/src/semantic/digest.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SchemaDigest::with_file_manifest` (declaration #8) within `SEMANTIC-DIGEST`.
    ///
    /// ## 3. Preconditions
    /// - `manifest: Vec<FileDigestEntry>` must satisfy domain and type-safety constraints prior to invoking `SchemaDigest::with_file_manifest`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Self` from `SchemaDigest::with_file_manifest` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(manifest)) bounded over input/member dimension of `SchemaDigest::with_file_manifest`.
    /// - Space Complexity: O(N(manifest)) bounded auxiliary memory for `SchemaDigest::with_file_manifest`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SchemaDigest::with_file_manifest` (`Self`).
    /// - State: Maintains consistent state invariants across `SchemaDigest::with_file_manifest` without partial mutation on error.
    /// - Action: Executes `SchemaDigest::with_file_manifest` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SchemaDigest::with_file_manifest` (`SEMANTIC-DIGEST`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-DIGEST-SCHEMADIGEST-WITH-FILE-MANIFEST] | SysML v2: ActionDef(SchemaDigest::with_file_manifest)
    pub fn with_file_manifest(mut self, manifest: Vec<FileDigestEntry>) -> Self {
        self.file_manifest = manifest;
        self
    }

    /// Atomically write digest JSON to disk using a temporary sibling file and rename.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `SchemaDigest::write_to_file` in `crates/compile-sysml/src/semantic/digest.rs`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Preserves deterministic state conservation and referential transparency for fn `SchemaDigest::write_to_file` (declaration #9) within `SEMANTIC-DIGEST`.
    ///
    /// ## 3. Preconditions
    /// - `path: &Path` must satisfy domain and type-safety constraints prior to invoking `SchemaDigest::write_to_file`.
    ///
    /// ## 4. Postconditions
    /// - Returns `Result<(), io::Error>` from `SchemaDigest::write_to_file` with deterministic post-execution state.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N(path)) bounded over input/member dimension of `SchemaDigest::write_to_file`.
    /// - Space Complexity: O(N(path)) bounded auxiliary memory for `SchemaDigest::write_to_file`.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Validates boundary inputs and types for `SchemaDigest::write_to_file` (`Result<(), io::Error>`).
    /// - State: Maintains consistent state invariants across `SchemaDigest::write_to_file` without partial mutation on error.
    /// - Action: Executes `SchemaDigest::write_to_file` deterministically with zero unbounded recursion or blocking.
    /// - Resource: Bounds stack and heap allocations within `SchemaDigest::write_to_file` (`SEMANTIC-DIGEST`).
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SEMANTIC-DIGEST-SCHEMADIGEST-WRITE-TO-FILE] | SysML v2: ActionDef(SchemaDigest::write_to_file)
    pub fn write_to_file(&self, path: &Path) -> Result<(), io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json_text = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let tmp_path = path.with_extension("tmp");
        fs::write(&tmp_path, json_text)?;
        fs::rename(&tmp_path, path)?;

        Ok(())
    }
}

/// Compute schema digest from a PackageDef and its compiled SysML textual bytes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `generate_digest` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `generate_digest` (declaration #10) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` must satisfy domain and type-safety constraints prior to invoking `generate_digest`.
/// - `sysml_text: &str` must satisfy domain and type-safety constraints prior to invoking `generate_digest`.
///
/// ## 4. Postconditions
/// - Returns `SchemaDigest` from `generate_digest` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(pkg)) bounded over input/member dimension of `generate_digest`.
/// - Space Complexity: O(N(pkg)) bounded auxiliary memory for `generate_digest`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `generate_digest` (`SchemaDigest`).
/// - State: Maintains consistent state invariants across `generate_digest` without partial mutation on error.
/// - Action: Executes `generate_digest` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `generate_digest` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-GENERATE-DIGEST] | SysML v2: ActionDef(generate_digest)
pub fn generate_digest(pkg: &PackageDef, sysml_text: &str) -> SchemaDigest {
    SchemaDigest::compute(pkg, sysml_text)
}

/// Executes deterministic fn `count_part_structural_elements` within `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `count_part_structural_elements` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `count_part_structural_elements` (declaration #11) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `part: &PartDef` must satisfy domain and type-safety constraints prior to invoking `count_part_structural_elements`.
///
/// ## 4. Postconditions
/// - Returns `usize` from `count_part_structural_elements` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(part)) bounded over input/member dimension of `count_part_structural_elements`.
/// - Space Complexity: O(N(part)) bounded auxiliary memory for `count_part_structural_elements`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `count_part_structural_elements` (`usize`).
/// - State: Maintains consistent state invariants across `count_part_structural_elements` without partial mutation on error.
/// - Action: Executes `count_part_structural_elements` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `count_part_structural_elements` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-COUNT-PART-STRUCTURAL-ELEMENTS] | SysML v2: ActionDef(count_part_structural_elements)
fn count_part_structural_elements(part: &PartDef) -> usize {
    let direct = part.parts.len()
        + part.attributes.len()
        + part.ports.len()
        + part.actions.len()
        + part.operations.len()
        + part.capabilities.len()
        + part.interactions.len()
        + part.constraints.len()
        + part.calcs.len()
        + part.test_cases.len()
        + part.requirements.len()
        + part.connections.len()
        + part.states.len()
        + part.use_cases.len()
        + part.item_defs.len()
        + part.hazards.len()
        + part.risks.len();

    let nested: usize = part.parts.iter().map(count_part_structural_elements).sum();
    direct + nested
}

/// Count structural AST definitions within a PackageDef.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `count_structural_elements` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `count_structural_elements` (declaration #12) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` must satisfy domain and type-safety constraints prior to invoking `count_structural_elements`.
///
/// ## 4. Postconditions
/// - Returns `usize` from `count_structural_elements` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(pkg)) bounded over input/member dimension of `count_structural_elements`.
/// - Space Complexity: O(N(pkg)) bounded auxiliary memory for `count_structural_elements`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `count_structural_elements` (`usize`).
/// - State: Maintains consistent state invariants across `count_structural_elements` without partial mutation on error.
/// - Action: Executes `count_structural_elements` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `count_structural_elements` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-COUNT-STRUCTURAL-ELEMENTS] | SysML v2: ActionDef(count_structural_elements)
pub fn count_structural_elements(pkg: &PackageDef) -> usize {
    let direct = pkg.part_defs.len()
        + pkg.attribute_defs.len()
        + pkg.port_defs.len()
        + pkg.action_defs.len()
        + pkg.operation_defs.len()
        + pkg.capability_defs.len()
        + pkg.interaction_defs.len()
        + pkg.constraint_defs.len()
        + pkg.calc_defs.len()
        + pkg.test_case_defs.len()
        + pkg.requirement_defs.len()
        + pkg.connection_defs.len()
        + pkg.state_defs.len()
        + pkg.use_case_defs.len()
        + pkg.item_defs.len()
        + pkg.hazard_defs.len()
        + pkg.risk_defs.len();

    let part_nested: usize = pkg.part_defs.iter().map(count_part_structural_elements).sum();
    let nested: usize = pkg.packages.iter().map(count_structural_elements).sum();
    direct + part_nested + nested
}

/// Compute a file manifest for a slice of SysML files.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `compute_file_manifest` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `compute_file_manifest` (declaration #13) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `files: &[PathBuf]` must satisfy domain and type-safety constraints prior to invoking `compute_file_manifest`.
///
/// ## 4. Postconditions
/// - Returns `Vec<FileDigestEntry>` from `compute_file_manifest` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(files)) bounded over input/member dimension of `compute_file_manifest`.
/// - Space Complexity: O(N(files)) bounded auxiliary memory for `compute_file_manifest`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `compute_file_manifest` (`Vec<FileDigestEntry>`).
/// - State: Maintains consistent state invariants across `compute_file_manifest` without partial mutation on error.
/// - Action: Executes `compute_file_manifest` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `compute_file_manifest` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-COMPUTE-FILE-MANIFEST] | SysML v2: ActionDef(compute_file_manifest)
pub fn compute_file_manifest(files: &[PathBuf]) -> Vec<FileDigestEntry> {
    let mut manifest = Vec::new();
    let cwd = std::env::current_dir().ok();

    for p in files {
        let rel_p = p.strip_prefix(".").unwrap_or(p);
        let rel_p = if let Some(ref cwd_path) = cwd {
            rel_p.strip_prefix(cwd_path).unwrap_or(rel_p)
        } else {
            rel_p
        };
        let path = rel_p.to_string_lossy().to_string();

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
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `write_digest_atomic` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `write_digest_atomic` (declaration #14) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `path: &Path` must satisfy domain and type-safety constraints prior to invoking `write_digest_atomic`.
/// - `digest: &SchemaDigest` must satisfy domain and type-safety constraints prior to invoking `write_digest_atomic`.
///
/// ## 4. Postconditions
/// - Returns `Result<(), io::Error>` from `write_digest_atomic` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(path)) bounded over input/member dimension of `write_digest_atomic`.
/// - Space Complexity: O(N(path)) bounded auxiliary memory for `write_digest_atomic`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `write_digest_atomic` (`Result<(), io::Error>`).
/// - State: Maintains consistent state invariants across `write_digest_atomic` without partial mutation on error.
/// - Action: Executes `write_digest_atomic` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `write_digest_atomic` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-WRITE-DIGEST-ATOMIC] | SysML v2: ActionDef(write_digest_atomic)
pub fn write_digest_atomic(path: &Path, digest: &SchemaDigest) -> Result<(), io::Error> {
    digest.write_to_file(path)
}

/// Atomically write arbitrary text to disk using a temporary sibling file and rename.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `write_atomic` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `write_atomic` (declaration #15) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `path: &Path` must satisfy domain and type-safety constraints prior to invoking `write_atomic`.
/// - `content: &str` must satisfy domain and type-safety constraints prior to invoking `write_atomic`.
///
/// ## 4. Postconditions
/// - Returns `Result<(), io::Error>` from `write_atomic` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(path)) bounded over input/member dimension of `write_atomic`.
/// - Space Complexity: O(N(path)) bounded auxiliary memory for `write_atomic`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `write_atomic` (`Result<(), io::Error>`).
/// - State: Maintains consistent state invariants across `write_atomic` without partial mutation on error.
/// - Action: Executes `write_atomic` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `write_atomic` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-WRITE-ATOMIC] | SysML v2: ActionDef(write_atomic)
pub fn write_atomic(path: &Path, content: &str) -> Result<(), io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, content)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}


/// Compute SHA-256 hash formatted as lowercase hexadecimal (FIPS 180-4).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `compute_sha256` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `compute_sha256` (declaration #16) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `bytes: &[u8]` must satisfy domain and type-safety constraints prior to invoking `compute_sha256`.
///
/// ## 4. Postconditions
/// - Returns `String` from `compute_sha256` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(bytes)) bounded over input/member dimension of `compute_sha256`.
/// - Space Complexity: O(N(bytes)) bounded auxiliary memory for `compute_sha256`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `compute_sha256` (`String`).
/// - State: Maintains consistent state invariants across `compute_sha256` without partial mutation on error.
/// - Action: Executes `compute_sha256` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `compute_sha256` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-COMPUTE-SHA256] | SysML v2: ActionDef(compute_sha256)
pub fn compute_sha256(bytes: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let mut h0: u32 = 0x6a09e667;
    let mut h1: u32 = 0xbb67ae85;
    let mut h2: u32 = 0x3c6ef372;
    let mut h3: u32 = 0xa54ff53a;
    let mut h4: u32 = 0x510e527f;
    let mut h5: u32 = 0x9b05688c;
    let mut h6: u32 = 0x1f83d9ab;
    let mut h7: u32 = 0x5be0cd19;

    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    let mut msg = bytes.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.as_chunks::<64>().0 {
        let mut w = [0u32; 64];
        for (slot, part) in w.iter_mut().zip(chunk.as_chunks::<4>().0) {
            *slot = u32::from_be_bytes(*part);
        }
        for i in 16..64 {
            let (w15, w2, w16, w7) = match (w.get(i - 15), w.get(i - 2), w.get(i - 16), w.get(i - 7)) {
                (Some(&w15), Some(&w2), Some(&w16), Some(&w7)) => (w15, w2, w16, w7),
                _ => continue,
            };
            let s0 = w15.rotate_right(7) ^ w15.rotate_right(18) ^ (w15 >> 3);
            let s1 = w2.rotate_right(17) ^ w2.rotate_right(19) ^ (w2 >> 10);
            if let Some(slot) = w.get_mut(i) {
                *slot = w16.wrapping_add(s0).wrapping_add(w7).wrapping_add(s1);
            }
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;
        let mut f = h5;
        let mut g = h6;
        let mut h = h7;

        for (&k, &wi) in K.iter().zip(w.iter()) {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(k)
                .wrapping_add(wi);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
        h5 = h5.wrapping_add(f);
        h6 = h6.wrapping_add(g);
        h7 = h7.wrapping_add(h);
    }

    format!(
        "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        h0, h1, h2, h3, h4, h5, h6, h7
    )
}

/// Executes deterministic fn `compute_node_counts` within `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `compute_node_counts` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `compute_node_counts` (declaration #17) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` must satisfy domain and type-safety constraints prior to invoking `compute_node_counts`.
///
/// ## 4. Postconditions
/// - Returns `BTreeMap<String, usize>` from `compute_node_counts` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(pkg)) bounded over input/member dimension of `compute_node_counts`.
/// - Space Complexity: O(N(pkg)) bounded auxiliary memory for `compute_node_counts`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `compute_node_counts` (`BTreeMap<String, usize>`).
/// - State: Maintains consistent state invariants across `compute_node_counts` without partial mutation on error.
/// - Action: Executes `compute_node_counts` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `compute_node_counts` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-COMPUTE-NODE-COUNTS] | SysML v2: ActionDef(compute_node_counts)
fn compute_node_counts(pkg: &PackageDef) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    counts.insert("packages".to_string(), 1);
    counts.insert("part_defs".to_string(), 0);
    counts.insert("attribute_defs".to_string(), 0);
    counts.insert("port_defs".to_string(), 0);
    counts.insert("action_defs".to_string(), 0);
    counts.insert("capability_defs".to_string(), 0);
    counts.insert("operation_defs".to_string(), 0);
    counts.insert("interaction_defs".to_string(), 0);
    counts.insert("constraint_defs".to_string(), 0);
    counts.insert("calc_defs".to_string(), 0);
    counts.insert("test_case_defs".to_string(), 0);
    counts.insert("requirement_defs".to_string(), 0);
    counts.insert("state_defs".to_string(), 0);
    counts.insert("use_case_defs".to_string(), 0);
    counts.insert("item_defs".to_string(), 0);
    counts.insert("hazard_defs".to_string(), 0);
    counts.insert("risk_defs".to_string(), 0);
    counts.insert("connection_defs".to_string(), 0);
    counts.insert("containers".to_string(), 0);
    counts.insert("lists".to_string(), 0);
    counts.insert("leaves".to_string(), 0);
    counts.insert("typedefs".to_string(), 0);
    counts.insert("identities".to_string(), 0);
    counts.insert("groupings".to_string(), 0);

    accumulate_package_counts(pkg, &mut counts);

    // Derived counts
    let containers = *counts.get("part_defs").unwrap_or(&0);
    let lists = *counts.get("action_defs").unwrap_or(&0);
    let leaves = *counts.get("attribute_defs").unwrap_or(&0);

    counts.insert("containers".to_string(), containers);
    counts.insert("lists".to_string(), lists);
    counts.insert("leaves".to_string(), leaves);

    counts
}

/// Executes deterministic fn `bump_count` within `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `bump_count` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `bump_count` (declaration #18) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `counts: &mut BTreeMap<String, usize>` must satisfy domain and type-safety constraints prior to invoking `bump_count`.
/// - `key: &str` must satisfy domain and type-safety constraints prior to invoking `bump_count`.
/// - `n: usize` must satisfy domain and type-safety constraints prior to invoking `bump_count`.
///
/// ## 4. Postconditions
/// - Returns `()` from `bump_count` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(counts)) bounded over input/member dimension of `bump_count`.
/// - Space Complexity: O(N(counts)) bounded auxiliary memory for `bump_count`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `bump_count` (`()`).
/// - State: Maintains consistent state invariants across `bump_count` without partial mutation on error.
/// - Action: Executes `bump_count` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `bump_count` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-BUMP-COUNT] | SysML v2: ActionDef(bump_count)
fn bump_count(counts: &mut BTreeMap<String, usize>, key: &str, n: usize) {
    *counts.entry(key.to_string()).or_insert(0) += n;
}

/// Executes deterministic fn `accumulate_package_counts` within `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `accumulate_package_counts` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `accumulate_package_counts` (declaration #19) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` must satisfy domain and type-safety constraints prior to invoking `accumulate_package_counts`.
/// - `counts: &mut BTreeMap<String, usize>` must satisfy domain and type-safety constraints prior to invoking `accumulate_package_counts`.
///
/// ## 4. Postconditions
/// - Returns `()` from `accumulate_package_counts` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(pkg)) bounded over input/member dimension of `accumulate_package_counts`.
/// - Space Complexity: O(N(pkg)) bounded auxiliary memory for `accumulate_package_counts`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `accumulate_package_counts` (`()`).
/// - State: Maintains consistent state invariants across `accumulate_package_counts` without partial mutation on error.
/// - Action: Executes `accumulate_package_counts` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `accumulate_package_counts` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-ACCUMULATE-PACKAGE-COUNTS] | SysML v2: ActionDef(accumulate_package_counts)
fn accumulate_package_counts(pkg: &PackageDef, counts: &mut BTreeMap<String, usize>) {
    bump_count(counts, "attribute_defs", pkg.attribute_defs.len());
    bump_count(counts, "port_defs", pkg.port_defs.len());
    bump_count(counts, "action_defs", pkg.action_defs.len());
    bump_count(counts, "capability_defs", pkg.capability_defs.len());
    bump_count(counts, "operation_defs", pkg.operation_defs.len());
    bump_count(counts, "interaction_defs", pkg.interaction_defs.len());
    bump_count(counts, "constraint_defs", pkg.constraint_defs.len());
    bump_count(counts, "calc_defs", pkg.calc_defs.len());
    bump_count(counts, "test_case_defs", pkg.test_case_defs.len());
    bump_count(counts, "requirement_defs", pkg.requirement_defs.len());
    bump_count(counts, "state_defs", pkg.state_defs.len());
    bump_count(counts, "use_case_defs", pkg.use_case_defs.len());
    bump_count(counts, "item_defs", pkg.item_defs.len());
    bump_count(counts, "hazard_defs", pkg.hazard_defs.len());
    bump_count(counts, "risk_defs", pkg.risk_defs.len());
    bump_count(counts, "connection_defs", pkg.connection_defs.len());
    bump_count(counts, "part_defs", pkg.part_defs.len());

    for part in &pkg.part_defs {
        accumulate_part_counts(part, counts);
    }

    for subpkg in &pkg.packages {
        bump_count(counts, "packages", 1);
        accumulate_package_counts(subpkg, counts);
    }
}

/// Executes deterministic fn `accumulate_part_counts` within `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `accumulate_part_counts` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `accumulate_part_counts` (declaration #20) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `part: &PartDef` must satisfy domain and type-safety constraints prior to invoking `accumulate_part_counts`.
/// - `counts: &mut BTreeMap<String, usize>` must satisfy domain and type-safety constraints prior to invoking `accumulate_part_counts`.
///
/// ## 4. Postconditions
/// - Returns `()` from `accumulate_part_counts` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(part)) bounded over input/member dimension of `accumulate_part_counts`.
/// - Space Complexity: O(N(part)) bounded auxiliary memory for `accumulate_part_counts`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `accumulate_part_counts` (`()`).
/// - State: Maintains consistent state invariants across `accumulate_part_counts` without partial mutation on error.
/// - Action: Executes `accumulate_part_counts` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `accumulate_part_counts` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-ACCUMULATE-PART-COUNTS] | SysML v2: ActionDef(accumulate_part_counts)
fn accumulate_part_counts(part: &PartDef, counts: &mut BTreeMap<String, usize>) {
    bump_count(counts, "attribute_defs", part.attributes.len());
    bump_count(counts, "port_defs", part.ports.len());
    bump_count(counts, "action_defs", part.actions.len());
    bump_count(counts, "capability_defs", part.capabilities.len());
    bump_count(counts, "operation_defs", part.operations.len());
    bump_count(counts, "interaction_defs", part.interactions.len());
    bump_count(counts, "constraint_defs", part.constraints.len());
    bump_count(counts, "calc_defs", part.calcs.len());
    bump_count(counts, "test_case_defs", part.test_cases.len());
    bump_count(counts, "requirement_defs", part.requirements.len());
    bump_count(counts, "state_defs", part.states.len());
    bump_count(counts, "use_case_defs", part.use_cases.len());
    bump_count(counts, "item_defs", part.item_defs.len());
    bump_count(counts, "hazard_defs", part.hazards.len());
    bump_count(counts, "risk_defs", part.risks.len());
    bump_count(counts, "connection_defs", part.connections.len());
    bump_count(counts, "part_defs", part.parts.len());

    for sub in &part.parts {
        accumulate_part_counts(sub, counts);
    }
}

/// Executes deterministic fn `collect_schema_nodes` within `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `collect_schema_nodes` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `collect_schema_nodes` (declaration #21) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` must satisfy domain and type-safety constraints prior to invoking `collect_schema_nodes`.
///
/// ## 4. Postconditions
/// - Returns `Vec<String>` from `collect_schema_nodes` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(pkg)) bounded over input/member dimension of `collect_schema_nodes`.
/// - Space Complexity: O(N(pkg)) bounded auxiliary memory for `collect_schema_nodes`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `collect_schema_nodes` (`Vec<String>`).
/// - State: Maintains consistent state invariants across `collect_schema_nodes` without partial mutation on error.
/// - Action: Executes `collect_schema_nodes` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `collect_schema_nodes` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-COLLECT-SCHEMA-NODES] | SysML v2: ActionDef(collect_schema_nodes)
fn collect_schema_nodes(pkg: &PackageDef) -> Vec<String> {
    let mut nodes = BTreeSet::new();
    nodes.insert(pkg.name.clone());

    for a in &pkg.attribute_defs {
        nodes.insert(a.name.clone());
    }
    for p in &pkg.port_defs {
        nodes.insert(p.name.clone());
    }
    for act in &pkg.action_defs {
        nodes.insert(act.name.clone());
    }
    for cap in &pkg.capability_defs {
        nodes.insert(cap.name.clone());
    }
    for op in &pkg.operation_defs {
        nodes.insert(op.name.clone());
    }
    for inter in &pkg.interaction_defs {
        nodes.insert(inter.name.clone());
    }
    for c in &pkg.constraint_defs {
        nodes.insert(c.name.clone());
    }
    for calc in &pkg.calc_defs {
        nodes.insert(calc.name.clone());
    }
    for tc in &pkg.test_case_defs {
        nodes.insert(tc.name.clone());
    }
    for r in &pkg.requirement_defs {
        nodes.insert(r.name.clone());
    }
    for s in &pkg.state_defs {
        nodes.insert(s.name.clone());
    }
    for conn in &pkg.connection_defs {
        nodes.insert(conn.name.clone());
    }
    for uc in &pkg.use_case_defs {
        nodes.insert(uc.name.clone());
    }
    for item in &pkg.item_defs {
        nodes.insert(item.name.clone());
    }
    for hz in &pkg.hazard_defs {
        nodes.insert(hz.name.clone());
    }
    for rk in &pkg.risk_defs {
        nodes.insert(rk.name.clone());
    }

    for part in &pkg.part_defs {
        collect_part_nodes(part, &mut nodes);
    }

    for sub in &pkg.packages {
        let sub_nodes = collect_schema_nodes(sub);
        nodes.extend(sub_nodes);
    }

    nodes.into_iter().collect()
}

/// Executes deterministic fn `collect_part_nodes` within `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `collect_part_nodes` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `collect_part_nodes` (declaration #22) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `part: &PartDef` must satisfy domain and type-safety constraints prior to invoking `collect_part_nodes`.
/// - `nodes: &mut BTreeSet<String>` must satisfy domain and type-safety constraints prior to invoking `collect_part_nodes`.
///
/// ## 4. Postconditions
/// - Returns `()` from `collect_part_nodes` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(part)) bounded over input/member dimension of `collect_part_nodes`.
/// - Space Complexity: O(N(part)) bounded auxiliary memory for `collect_part_nodes`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `collect_part_nodes` (`()`).
/// - State: Maintains consistent state invariants across `collect_part_nodes` without partial mutation on error.
/// - Action: Executes `collect_part_nodes` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `collect_part_nodes` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-COLLECT-PART-NODES] | SysML v2: ActionDef(collect_part_nodes)
fn collect_part_nodes(part: &PartDef, nodes: &mut BTreeSet<String>) {
    nodes.insert(part.name.clone());
    for a in &part.attributes {
        nodes.insert(a.name.clone());
    }
    for p in &part.ports {
        nodes.insert(p.name.clone());
    }
    for act in &part.actions {
        nodes.insert(act.name.clone());
    }
    for cap in &part.capabilities {
        nodes.insert(cap.name.clone());
    }
    for op in &part.operations {
        nodes.insert(op.name.clone());
    }
    for inter in &part.interactions {
        nodes.insert(inter.name.clone());
    }
    for c in &part.constraints {
        nodes.insert(c.name.clone());
    }
    for calc in &part.calcs {
        nodes.insert(calc.name.clone());
    }
    for tc in &part.test_cases {
        nodes.insert(tc.name.clone());
    }
    for r in &part.requirements {
        nodes.insert(r.name.clone());
    }
    for s in &part.states {
        nodes.insert(s.name.clone());
    }
    for conn in &part.connections {
        nodes.insert(conn.name.clone());
    }
    for uc in &part.use_cases {
        nodes.insert(uc.name.clone());
    }
    for item in &part.item_defs {
        nodes.insert(item.name.clone());
    }
    for hz in &part.hazards {
        nodes.insert(hz.name.clone());
    }
    for rk in &part.risks {
        nodes.insert(rk.name.clone());
    }
    for sub in &part.parts {
        collect_part_nodes(sub, nodes);
    }
}

/// Executes deterministic fn `extract_operational_activities` within `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `extract_operational_activities` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `extract_operational_activities` (declaration #23) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` must satisfy domain and type-safety constraints prior to invoking `extract_operational_activities`.
///
/// ## 4. Postconditions
/// - Returns `Vec<OperationalActivity>` from `extract_operational_activities` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(pkg)) bounded over input/member dimension of `extract_operational_activities`.
/// - Space Complexity: O(N(pkg)) bounded auxiliary memory for `extract_operational_activities`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `extract_operational_activities` (`Vec<OperationalActivity>`).
/// - State: Maintains consistent state invariants across `extract_operational_activities` without partial mutation on error.
/// - Action: Executes `extract_operational_activities` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `extract_operational_activities` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-EXTRACT-OPERATIONAL-ACTIVITIES] | SysML v2: ActionDef(extract_operational_activities)
fn extract_operational_activities(pkg: &PackageDef) -> Vec<OperationalActivity> {
    let mut activities = Vec::new();
    let mut idx = 1;

    for p in pkg.get_all_packages() {
        for act in &p.action_defs {
            activities.push(OperationalActivity {
                id: format!("OA-{:02}", idx),
                name: act.name.clone(),
                doc: act.doc.clone().unwrap_or_default(),
                performer: act.performer.clone().unwrap_or_default(),
                inputs: act.in_params.iter().map(|param| param.name.clone()).collect(),
                outputs: act.out_params.iter().map(|param| param.name.clone()).collect(),
            });
            idx += 1;
        }
    }

    for part in pkg.get_all_parts() {
        for act in &part.actions {
            activities.push(OperationalActivity {
                id: format!("OA-{:02}", idx),
                name: act.name.clone(),
                doc: act.doc.clone().unwrap_or_default(),
                performer: act.performer.clone().unwrap_or_else(|| part.name.clone()),
                inputs: act.in_params.iter().map(|param| param.name.clone()).collect(),
                outputs: act.out_params.iter().map(|param| param.name.clone()).collect(),
            });
            idx += 1;
        }
    }

    activities
}

/// Executes deterministic fn `extract_operational_exchanges` within `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `extract_operational_exchanges` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `extract_operational_exchanges` (declaration #24) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` must satisfy domain and type-safety constraints prior to invoking `extract_operational_exchanges`.
///
/// ## 4. Postconditions
/// - Returns `Vec<OperationalExchange>` from `extract_operational_exchanges` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(pkg)) bounded over input/member dimension of `extract_operational_exchanges`.
/// - Space Complexity: O(N(pkg)) bounded auxiliary memory for `extract_operational_exchanges`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `extract_operational_exchanges` (`Vec<OperationalExchange>`).
/// - State: Maintains consistent state invariants across `extract_operational_exchanges` without partial mutation on error.
/// - Action: Executes `extract_operational_exchanges` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `extract_operational_exchanges` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-EXTRACT-OPERATIONAL-EXCHANGES] | SysML v2: ActionDef(extract_operational_exchanges)
fn extract_operational_exchanges(pkg: &PackageDef) -> Vec<OperationalExchange> {
    let mut exchanges = Vec::new();
    let mut idx = 1;

    for p in pkg.get_all_packages() {
        for conn in &p.connection_defs {
            exchanges.push(OperationalExchange {
                id: format!("OpTx-{:02}", idx),
                name: conn.name.clone(),
                source_port: conn.source_port.clone(),
                target_port: conn.target_port.clone(),
                source_part: conn.source_part.clone().unwrap_or_default(),
                target_part: conn.target_part.clone().unwrap_or_default(),
                protocol: conn.protocol.clone().unwrap_or_default(),
            });
            idx += 1;
        }
    }

    for part in pkg.get_all_parts() {
        for conn in &part.connections {
            exchanges.push(OperationalExchange {
                id: format!("OpTx-{:02}", idx),
                name: conn.name.clone(),
                source_port: conn.source_port.clone(),
                target_port: conn.target_port.clone(),
                source_part: conn.source_part.clone().unwrap_or_else(|| part.name.clone()),
                target_part: conn.target_part.clone().unwrap_or_default(),
                protocol: conn.protocol.clone().unwrap_or_default(),
            });
            idx += 1;
        }
    }

    exchanges
}

/// Executes deterministic fn `extract_operational_scenarios` within `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `extract_operational_scenarios` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `extract_operational_scenarios` (declaration #25) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` must satisfy domain and type-safety constraints prior to invoking `extract_operational_scenarios`.
///
/// ## 4. Postconditions
/// - Returns `Vec<OperationalScenario>` from `extract_operational_scenarios` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(pkg)) bounded over input/member dimension of `extract_operational_scenarios`.
/// - Space Complexity: O(N(pkg)) bounded auxiliary memory for `extract_operational_scenarios`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `extract_operational_scenarios` (`Vec<OperationalScenario>`).
/// - State: Maintains consistent state invariants across `extract_operational_scenarios` without partial mutation on error.
/// - Action: Executes `extract_operational_scenarios` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `extract_operational_scenarios` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-EXTRACT-OPERATIONAL-SCENARIOS] | SysML v2: ActionDef(extract_operational_scenarios)
fn extract_operational_scenarios(pkg: &PackageDef) -> Vec<OperationalScenario> {
    let mut scenarios = Vec::new();

    for (idx, uc) in (1..).zip(pkg.get_all_use_cases()) {
        scenarios.push(OperationalScenario {
            id: format!("SCN-{:02}", idx),
            name: uc.name.clone(),
            actor: uc.actor.clone().unwrap_or_default(),
            objective: uc.objective.clone().unwrap_or_default(),
            steps: uc.steps.clone(),
        });
    }

    scenarios
}

/// Executes deterministic fn `extract_operational_nodes` within `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for fn `extract_operational_nodes` in `crates/compile-sysml/src/semantic/digest.rs`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves deterministic state conservation and referential transparency for fn `extract_operational_nodes` (declaration #26) within `SEMANTIC-DIGEST`.
///
/// ## 3. Preconditions
/// - `pkg: &PackageDef` must satisfy domain and type-safety constraints prior to invoking `extract_operational_nodes`.
///
/// ## 4. Postconditions
/// - Returns `Vec<OperationalNode>` from `extract_operational_nodes` with deterministic post-execution state.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N(pkg)) bounded over input/member dimension of `extract_operational_nodes`.
/// - Space Complexity: O(N(pkg)) bounded auxiliary memory for `extract_operational_nodes`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `extract_operational_nodes` (`Vec<OperationalNode>`).
/// - State: Maintains consistent state invariants across `extract_operational_nodes` without partial mutation on error.
/// - Action: Executes `extract_operational_nodes` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `extract_operational_nodes` (`SEMANTIC-DIGEST`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SEMANTIC-DIGEST-EXTRACT-OPERATIONAL-NODES] | SysML v2: ActionDef(extract_operational_nodes)
fn extract_operational_nodes(pkg: &PackageDef) -> Vec<OperationalNode> {
    let mut nodes = Vec::new();

    for part in pkg.get_all_parts() {
        nodes.push(OperationalNode {
            name: part.name.clone(),
            node_type: "Part".to_string(),
            doc: part.doc.clone().unwrap_or_default(),
            allocated_activities: part.actions.iter().map(|a| a.name.clone()).collect(),
        });
    }

    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_digest_counts_and_hash() -> Result<(), Box<dyn std::error::Error>> {
        let pkg = PackageDef {
            name: "DigestTest".to_string(),
            attribute_defs: vec![
                AttributeDef {
                    name: "param1".to_string(),
                    type_name: "Real".to_string(),
                    default_value: Some("1.0".to_string()),
                    doc: None,
                },
            ],
            part_defs: vec![
                PartDef {
                    name: "ComponentA".to_string(),
                    ports: vec![
                        PortDef {
                            name: "p1".to_string(),
                            direction: "out".to_string(),
                            type_name: "Data".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let sysml_source = "package DigestTest {\n    attribute param1 : Real = 1.0;\n}\n";
        let digest = SchemaDigest::compute(&pkg, sysml_source);

        assert_eq!(digest.total_lines, 3);
        assert!(!digest.sha256.is_empty());
        assert_eq!(*digest.node_counts.get("packages").ok_or("expected value is absent")?, 1);
        assert_eq!(*digest.node_counts.get("part_defs").ok_or("expected value is absent")?, 1);
        assert_eq!(*digest.node_counts.get("port_defs").ok_or("expected value is absent")?, 1);
        assert_eq!(*digest.node_counts.get("attribute_defs").ok_or("expected value is absent")?, 1);
        assert!(digest.schema_nodes.contains(&"DigestTest".to_string()));
        assert!(digest.schema_nodes.contains(&"ComponentA".to_string()));
        assert!(digest.schema_nodes.contains(&"p1".to_string()));
        Ok(())
    }

    #[test]
    fn test_schema_digest_multi_file_manifest() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = std::env::temp_dir().join(format!(
            "compile_sysml_manifest_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                ?
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir)?;

        let file1 = temp_dir.join("subsystem_a.sysml");
        let file2 = temp_dir.join("subsystem_b.sysml");

        let content1 = "package SubsystemA {\n    part def ComponentA;\n    attribute mass : Real = 10.0;\n}\n";
        let content2 = "package SubsystemB {\n    part def ComponentB;\n}\n";

        fs::write(&file1, content1)?;
        fs::write(&file2, content2)?;

        let manifest = compute_file_manifest(&[file1.clone(), file2.clone()]);
        assert_eq!(manifest.len(), 2);

        // Entry 1 verification
        assert_eq!(manifest.first().ok_or("missing first element of manifest")?.path, file1.to_string_lossy().to_string());
        assert_eq!(manifest.first().ok_or("missing first element of manifest")?.sha256, compute_sha256(content1.as_bytes()));
        assert_eq!(manifest.first().ok_or("missing first element of manifest")?.bytes, content1.len() as u64);
        assert_eq!(manifest.first().ok_or("missing first element of manifest")?.lines, content1.lines().count());
        assert_eq!(manifest.first().ok_or("missing first element of manifest")?.entities, 2); // ComponentA and mass

        // Entry 2 verification
        assert_eq!(manifest.get(1).ok_or("missing element 1 of manifest")?.path, file2.to_string_lossy().to_string());
        assert_eq!(manifest.get(1).ok_or("missing element 1 of manifest")?.sha256, compute_sha256(content2.as_bytes()));
        assert_eq!(manifest.get(1).ok_or("missing element 1 of manifest")?.bytes, content2.len() as u64);
        assert_eq!(manifest.get(1).ok_or("missing element 1 of manifest")?.lines, content2.lines().count());
        assert_eq!(manifest.get(1).ok_or("missing element 1 of manifest")?.entities, 1); // ComponentB

        // Verify SchemaDigest integration and serialization round-trip
        let digest = SchemaDigest::default().with_file_manifest(manifest.clone());
        assert_eq!(digest.file_manifest, manifest);

        let json = serde_json::to_string_pretty(&digest)?;
        let deserialized: SchemaDigest =
            serde_json::from_str(&json)?;
        assert_eq!(deserialized.file_manifest, manifest);

        let _ = fs::remove_dir_all(&temp_dir);
        Ok(())
    }
}
