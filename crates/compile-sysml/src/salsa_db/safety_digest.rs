#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! # Schema Digest Computation & Operational UAF Projections (`safety_digest`)
//! [DOC-TIER: A]
//! ## 1. Safety Intent & Regulatory Scope -- Computes FIPS 180-4 SHA-256 schema digests, iterative AST node counts, and UAF operational projections matching `.pipeline/schema-digest.json` (`BP-10`).
//! ## 2. Invariants & Mathematical Properties -- Traverses package and part hierarchies iteratively using explicit work-stacks and saturating arithmetic.
//! ## 3. Preconditions -- Caller supplies a valid `PackageDef` AST and canonical SysML v2 textual notation slice.
//! ## 4. Postconditions -- Produces a serializable `SchemaDigest` model with deterministic `BTreeMap` node counts and operational descriptors.
//! ## 5. Algorithmic Complexity Bounds -- Time: O(V log V + B) over AST nodes `V` and source bytes `B`; Space: O(V).
//! ## 6. Failure Modes & Fault Tolerance -- Zero call-stack recursion, zero overflow via `saturating_add`, delegates disk persistence to `write_atomic`.
//! ## 7. Realization & Traceability -- Realises: [REQ-0603], [REQ-SEMANTIC-DIGEST-SCHEMADIGEST] | [FLS: fls_66m4rnbssgig]

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::Path;
use deap_core::sysml_cst::{PackageDef, PartDef};
use serde::{Deserialize, Serialize};
use super::source_db::write_atomic;

/// Individual file entry recorded in the schema digest file manifest. Realises: [REQ-SEMANTIC-DIGEST-FILEDIGESTENTRY] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FileDigestEntry {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    pub lines: usize,
    pub entities: usize,
}

/// Operational activity descriptor in schema digest. Realises: [REQ-SEMANTIC-DIGEST-OPERATIONALACTIVITY] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OperationalActivity {
    pub id: String,
    pub name: String,
    pub doc: String,
    pub performer: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

/// Operational exchange descriptor in schema digest. Realises: [REQ-SEMANTIC-DIGEST-OPERATIONALEXCHANGE] | [FLS: fls_66m4rnbssgig]
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

/// Operational scenario descriptor in schema digest. Realises: [REQ-SEMANTIC-DIGEST-OPERATIONALSCENARIO] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OperationalScenario {
    pub id: String,
    pub name: String,
    pub actor: String,
    pub objective: String,
    pub steps: Vec<String>,
}

/// Operational node descriptor in schema digest. Realises: [REQ-SEMANTIC-DIGEST-OPERATIONALNODE] | [FLS: fls_66m4rnbssgig]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct OperationalNode {
    pub name: String,
    pub node_type: String,
    pub doc: String,
    pub allocated_activities: Vec<String>,
}

/// Authoritative schema digest model written to `.pipeline/schema-digest.json`. Realises: [REQ-SEMANTIC-DIGEST-SCHEMADIGEST] | [FLS: fls_66m4rnbssgig]
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
    /// Computes schema digest from a `PackageDef` and its compiled SysML textual bytes. Realises: [REQ-SEMANTIC-DIGEST-SCHEMADIGEST-COMPUTE] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn compute(pkg: &PackageDef, sysml_text: &str) -> Self {
        Self {
            sha256: compute_sha256(sysml_text.as_bytes()),
            total_lines: sysml_text.lines().count(),
            node_counts: compute_node_counts(pkg),
            schema_nodes: collect_schema_nodes(pkg),
            operational_activities: extract_operational_activities(pkg),
            operational_exchanges: extract_operational_exchanges(pkg),
            operational_scenarios: extract_operational_scenarios(pkg),
            operational_nodes: extract_operational_nodes(pkg),
            file_manifest: Vec::new(),
        }
    }

    /// Builder method to attach a file manifest to the schema digest. Realises: [REQ-SEMANTIC-DIGEST-SCHEMADIGEST-WITH-FILE-MANIFEST] | [FLS: fls_66m4rnbssgig]
    #[must_use]
    pub fn with_file_manifest(mut self, manifest: Vec<FileDigestEntry>) -> Self {
        self.file_manifest = manifest;
        self
    }

    /// Atomically writes digest JSON to disk using `write_atomic`. Realises: [REQ-SEMANTIC-DIGEST-SCHEMADIGEST-WRITE-TO-FILE] | [FLS: fls_66m4rnbssgig]
    pub fn write_to_file(&self, path: &Path) -> Result<(), io::Error> {
        let json_text = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        write_atomic(path, &json_text)
    }
}

/// Computes schema digest from a `PackageDef` and its compiled SysML textual bytes. Realises: [REQ-SEMANTIC-DIGEST-GENERATE-DIGEST] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn generate_digest(pkg: &PackageDef, sysml_text: &str) -> SchemaDigest {
    SchemaDigest::compute(pkg, sysml_text)
}

/// Computes SHA-256 hash formatted as lowercase hexadecimal (FIPS 180-4). Realises: [REQ-SEMANTIC-DIGEST-COMPUTE-SHA256] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn compute_sha256(bytes: &[u8]) -> String {
    deap_core::digest::compute_sha256(bytes)
}

/// Iteratively counts all direct and nested structural AST elements within a `PartDef` subtree. Realises: [REQ-SEMANTIC-DIGEST-COUNT-PART-STRUCTURAL-ELEMENTS] | [FLS: fls_66m4rnbssgig]
fn count_part_structural_elements(root_part: &PartDef) -> usize {
    let (mut total, mut stack) = (0usize, vec![root_part]);
    while let Some(part) = stack.pop() {
        let direct = part.parts.len()
            .saturating_add(part.attributes.len()).saturating_add(part.ports.len())
            .saturating_add(part.actions.len()).saturating_add(part.operations.len())
            .saturating_add(part.capabilities.len()).saturating_add(part.interactions.len())
            .saturating_add(part.constraints.len()).saturating_add(part.calcs.len())
            .saturating_add(part.test_cases.len()).saturating_add(part.requirements.len())
            .saturating_add(part.connections.len()).saturating_add(part.states.len())
            .saturating_add(part.use_cases.len()).saturating_add(part.item_defs.len())
            .saturating_add(part.hazards.len()).saturating_add(part.risks.len());
        total = total.saturating_add(direct);
        for child in part.parts.iter().rev() { stack.push(child); }
    }
    total
}

/// Counts all structural AST definitions across a `PackageDef` hierarchy using iterative work-stacks. Realises: [REQ-SEMANTIC-DIGEST-COUNT-STRUCTURAL-ELEMENTS] | [FLS: fls_66m4rnbssgig]
#[must_use]
pub fn count_structural_elements(pkg: &PackageDef) -> usize {
    let (mut total, mut stack) = (0usize, vec![pkg]);
    while let Some(curr) = stack.pop() {
        let direct = curr.part_defs.len()
            .saturating_add(curr.attribute_defs.len()).saturating_add(curr.port_defs.len())
            .saturating_add(curr.action_defs.len()).saturating_add(curr.operation_defs.len())
            .saturating_add(curr.capability_defs.len()).saturating_add(curr.interaction_defs.len())
            .saturating_add(curr.constraint_defs.len()).saturating_add(curr.calc_defs.len())
            .saturating_add(curr.test_case_defs.len()).saturating_add(curr.requirement_defs.len())
            .saturating_add(curr.connection_defs.len()).saturating_add(curr.state_defs.len())
            .saturating_add(curr.use_case_defs.len()).saturating_add(curr.item_defs.len())
            .saturating_add(curr.hazard_defs.len()).saturating_add(curr.risk_defs.len());
        total = total.saturating_add(direct);
        for part in &curr.part_defs { total = total.saturating_add(count_part_structural_elements(part)); }
        for sub_pkg in curr.packages.iter().rev() { stack.push(sub_pkg); }
    }
    total
}

/// Increments `key` in `counts` by `n` using saturating addition. Realises: [REQ-SEMANTIC-DIGEST-BUMP-COUNT] | [FLS: fls_66m4rnbssgig]
fn bump_count(counts: &mut BTreeMap<String, usize>, key: &str, n: usize) {
    let entry = counts.entry(key.to_string()).or_insert(0);
    *entry = entry.saturating_add(n);
}

/// Computes per-kind AST definition counts across `pkg`. Realises: [REQ-SEMANTIC-DIGEST-COMPUTE-NODE-COUNTS] | [FLS: fls_66m4rnbssgig]
pub(crate) fn compute_node_counts(pkg: &PackageDef) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for key in [
        "packages", "part_defs", "attribute_defs", "port_defs", "action_defs", "capability_defs",
        "operation_defs", "interaction_defs", "constraint_defs", "calc_defs", "test_case_defs",
        "requirement_defs", "state_defs", "use_case_defs", "item_defs", "hazard_defs", "risk_defs",
        "connection_defs", "containers", "lists", "leaves", "typedefs", "identities", "groupings",
    ] {
        counts.insert(key.to_string(), usize::from(key == "packages"));
    }
    let mut stack: Vec<&PackageDef> = vec![pkg];
    while let Some(curr) = stack.pop() {
        for (k, n) in [
            ("attribute_defs", curr.attribute_defs.len()), ("port_defs", curr.port_defs.len()),
            ("action_defs", curr.action_defs.len()), ("capability_defs", curr.capability_defs.len()),
            ("operation_defs", curr.operation_defs.len()), ("interaction_defs", curr.interaction_defs.len()),
            ("constraint_defs", curr.constraint_defs.len()), ("calc_defs", curr.calc_defs.len()),
            ("test_case_defs", curr.test_case_defs.len()), ("requirement_defs", curr.requirement_defs.len()),
            ("state_defs", curr.state_defs.len()), ("use_case_defs", curr.use_case_defs.len()),
            ("item_defs", curr.item_defs.len()), ("hazard_defs", curr.hazard_defs.len()),
            ("risk_defs", curr.risk_defs.len()), ("connection_defs", curr.connection_defs.len()),
            ("part_defs", curr.part_defs.len()),
        ] {
            bump_count(&mut counts, k, n);
        }
        for part in &curr.part_defs { accumulate_part_counts(part, &mut counts); }
        for subpkg in curr.packages.iter().rev() {
            bump_count(&mut counts, "packages", 1);
            stack.push(subpkg);
        }
    }
    counts.insert("containers".to_string(), *counts.get("part_defs").unwrap_or(&0));
    counts.insert("lists".to_string(), *counts.get("action_defs").unwrap_or(&0));
    counts.insert("leaves".to_string(), *counts.get("attribute_defs").unwrap_or(&0));
    counts
}

/// Iteratively accumulates per-kind AST definition counts across a `PartDef` composition tree. Realises: [REQ-SEMANTIC-DIGEST-ACCUMULATE-PART-COUNTS] | [FLS: fls_66m4rnbssgig]
fn accumulate_part_counts(root_part: &PartDef, counts: &mut BTreeMap<String, usize>) {
    let mut stack: Vec<&PartDef> = vec![root_part];
    while let Some(part) = stack.pop() {
        for (k, n) in [
            ("attribute_defs", part.attributes.len()), ("port_defs", part.ports.len()),
            ("action_defs", part.actions.len()), ("capability_defs", part.capabilities.len()),
            ("operation_defs", part.operations.len()), ("interaction_defs", part.interactions.len()),
            ("constraint_defs", part.constraints.len()), ("calc_defs", part.calcs.len()),
            ("test_case_defs", part.test_cases.len()), ("requirement_defs", part.requirements.len()),
            ("state_defs", part.states.len()), ("use_case_defs", part.use_cases.len()),
            ("item_defs", part.item_defs.len()), ("hazard_defs", part.hazards.len()),
            ("risk_defs", part.risks.len()), ("connection_defs", part.connections.len()),
            ("part_defs", part.parts.len()),
        ] {
            bump_count(counts, k, n);
        }
        for sub in part.parts.iter().rev() { stack.push(sub); }
    }
}

/// Iteratively collects sorted unique identifier names across `pkg`, nested subpackages, and parts. Realises: [REQ-SEMANTIC-DIGEST-COLLECT-SCHEMA-NODES] | [FLS: fls_66m4rnbssgig]
pub(crate) fn collect_schema_nodes(pkg: &PackageDef) -> Vec<String> {
    let (mut nodes, mut stack) = (BTreeSet::new(), vec![pkg]);
    while let Some(curr) = stack.pop() {
        nodes.insert(curr.name.clone());
        nodes.extend(curr.attribute_defs.iter().map(|x| x.name.clone()));
        nodes.extend(curr.port_defs.iter().map(|x| x.name.clone()));
        nodes.extend(curr.action_defs.iter().map(|x| x.name.clone()));
        nodes.extend(curr.capability_defs.iter().map(|x| x.name.clone()));
        nodes.extend(curr.operation_defs.iter().map(|x| x.name.clone()));
        nodes.extend(curr.interaction_defs.iter().map(|x| x.name.clone()));
        nodes.extend(curr.constraint_defs.iter().map(|x| x.name.clone()));
        nodes.extend(curr.calc_defs.iter().map(|x| x.name.clone()));
        nodes.extend(curr.test_case_defs.iter().map(|x| x.name.clone()));
        nodes.extend(curr.requirement_defs.iter().map(|x| x.name.clone()));
        nodes.extend(curr.state_defs.iter().map(|x| x.name.clone()));
        nodes.extend(curr.connection_defs.iter().map(|x| x.name.clone()));
        nodes.extend(curr.use_case_defs.iter().map(|x| x.name.clone()));
        nodes.extend(curr.item_defs.iter().map(|x| x.name.clone()));
        nodes.extend(curr.hazard_defs.iter().map(|x| x.name.clone()));
        nodes.extend(curr.risk_defs.iter().map(|x| x.name.clone()));
        for part in &curr.part_defs { collect_part_nodes(part, &mut nodes); }
        for sub in curr.packages.iter().rev() { stack.push(sub); }
    }
    nodes.into_iter().collect()
}

/// Iteratively collects identifier names across a `PartDef` composition subtree into `nodes`. Realises: [REQ-SEMANTIC-DIGEST-COLLECT-PART-NODES] | [FLS: fls_66m4rnbssgig]
fn collect_part_nodes(root_part: &PartDef, nodes: &mut BTreeSet<String>) {
    let mut stack: Vec<&PartDef> = vec![root_part];
    while let Some(part) = stack.pop() {
        nodes.insert(part.name.clone());
        nodes.extend(part.attributes.iter().map(|x| x.name.clone()));
        nodes.extend(part.ports.iter().map(|x| x.name.clone()));
        nodes.extend(part.actions.iter().map(|x| x.name.clone()));
        nodes.extend(part.capabilities.iter().map(|x| x.name.clone()));
        nodes.extend(part.operations.iter().map(|x| x.name.clone()));
        nodes.extend(part.interactions.iter().map(|x| x.name.clone()));
        nodes.extend(part.constraints.iter().map(|x| x.name.clone()));
        nodes.extend(part.calcs.iter().map(|x| x.name.clone()));
        nodes.extend(part.test_cases.iter().map(|x| x.name.clone()));
        nodes.extend(part.requirements.iter().map(|x| x.name.clone()));
        nodes.extend(part.states.iter().map(|x| x.name.clone()));
        nodes.extend(part.connections.iter().map(|x| x.name.clone()));
        nodes.extend(part.use_cases.iter().map(|x| x.name.clone()));
        nodes.extend(part.item_defs.iter().map(|x| x.name.clone()));
        nodes.extend(part.hazards.iter().map(|x| x.name.clone()));
        nodes.extend(part.risks.iter().map(|x| x.name.clone()));
        for sub in part.parts.iter().rev() { stack.push(sub); }
    }
}

/// Extracts `OperationalActivity` records from `pkg`. Realises: [REQ-SEMANTIC-DIGEST-EXTRACT-OPERATIONAL-ACTIVITIES] | [FLS: fls_66m4rnbssgig]
fn extract_operational_activities(pkg: &PackageDef) -> Vec<OperationalActivity> {
    let (mut activities, mut idx) = (Vec::new(), 1usize);
    for p in pkg.get_all_packages() {
        for act in &p.action_defs {
            activities.push(OperationalActivity {
                id: format!("OA-{idx:02}"), name: act.name.clone(), doc: act.doc.clone().unwrap_or_default(),
                performer: act.performer.clone().unwrap_or_default(),
                inputs: act.in_params.iter().map(|param| param.name.clone()).collect(),
                outputs: act.out_params.iter().map(|param| param.name.clone()).collect(),
            });
            idx = idx.saturating_add(1);
        }
    }
    for part in pkg.get_all_parts() {
        for act in &part.actions {
            activities.push(OperationalActivity {
                id: format!("OA-{idx:02}"), name: act.name.clone(), doc: act.doc.clone().unwrap_or_default(),
                performer: act.performer.clone().unwrap_or_else(|| part.name.clone()),
                inputs: act.in_params.iter().map(|param| param.name.clone()).collect(),
                outputs: act.out_params.iter().map(|param| param.name.clone()).collect(),
            });
            idx = idx.saturating_add(1);
        }
    }
    activities
}

/// Extracts `OperationalExchange` records from `pkg`. Realises: [REQ-SEMANTIC-DIGEST-EXTRACT-OPERATIONAL-EXCHANGES] | [FLS: fls_66m4rnbssgig]
fn extract_operational_exchanges(pkg: &PackageDef) -> Vec<OperationalExchange> {
    let (mut exchanges, mut idx) = (Vec::new(), 1usize);
    for p in pkg.get_all_packages() {
        for conn in &p.connection_defs {
            exchanges.push(OperationalExchange {
                id: format!("OpTx-{idx:02}"), name: conn.name.clone(),
                source_port: conn.source_port.clone(), target_port: conn.target_port.clone(),
                source_part: conn.source_part.clone().unwrap_or_default(),
                target_part: conn.target_part.clone().unwrap_or_default(),
                protocol: conn.protocol.clone().unwrap_or_default(),
            });
            idx = idx.saturating_add(1);
        }
    }
    for part in pkg.get_all_parts() {
        for conn in &part.connections {
            exchanges.push(OperationalExchange {
                id: format!("OpTx-{idx:02}"), name: conn.name.clone(),
                source_port: conn.source_port.clone(), target_port: conn.target_port.clone(),
                source_part: conn.source_part.clone().unwrap_or_else(|| part.name.clone()),
                target_part: conn.target_part.clone().unwrap_or_default(),
                protocol: conn.protocol.clone().unwrap_or_default(),
            });
            idx = idx.saturating_add(1);
        }
    }
    exchanges
}

/// Extracts `OperationalScenario` records from `pkg`. Realises: [REQ-SEMANTIC-DIGEST-EXTRACT-OPERATIONAL-SCENARIOS] | [FLS: fls_66m4rnbssgig]
fn extract_operational_scenarios(pkg: &PackageDef) -> Vec<OperationalScenario> {
    (1..).zip(pkg.get_all_use_cases()).map(|(idx, uc)| OperationalScenario {
        id: format!("SCN-{idx:02}"), name: uc.name.clone(),
        actor: uc.actor.clone().unwrap_or_default(),
        objective: uc.objective.clone().unwrap_or_default(),
        steps: uc.steps.clone(),
    }).collect()
}

/// Extracts `OperationalNode` records from `pkg`. Realises: [REQ-SEMANTIC-DIGEST-EXTRACT-OPERATIONAL-NODES] | [FLS: fls_66m4rnbssgig]
fn extract_operational_nodes(pkg: &PackageDef) -> Vec<OperationalNode> {
    pkg.get_all_parts().into_iter().map(|part| OperationalNode {
        name: part.name.clone(), node_type: "Part".to_string(),
        doc: part.doc.clone().unwrap_or_default(),
        allocated_activities: part.actions.iter().map(|a| a.name.clone()).collect(),
    }).collect()
}
