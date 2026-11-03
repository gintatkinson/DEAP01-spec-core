//! Attribute-domain conformance (`#746`) and SysML v2 / OMG UAF M2 metamodel validators (`#745`) (`[REQ-0402]`, `[FLS: fls_conops_cmark]`).

use std::collections::BTreeMap;

use super::cmark_engine::SubsystemPart;

/// [DOC-TIER: A] Subsystem domain classification for attribute-domain conformance (`#746`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0402] [FLS: fls_conops_cmark]
/// Contract: Partitions subsystem parts into `Physical`, `Cyber`, `Logical`, or `Human` operational domains.
/// Bounds & Faults: O(1) copy enum; governs required and forbidden physical/cyber/human envelope attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    Physical,
    Cyber,
    Logical,
    Human,
}

/// [DOC-TIER: A] Classifies a domain token string into [`Domain`] without substring collision (`#746`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0402] [FLS: fls_conops_cmark]
/// Contract: Splits `token` on non-alphanumeric delimiters and maps exact domain tokens to [`Domain`].
/// Bounds & Faults: O(N) time over `token.len()`; defaults to `Domain::Logical` when no physical/cyber/human token matches.
pub fn classify_domain_token(token: &str) -> Domain {
    let words: Vec<String> = token
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    if words
        .iter()
        .any(|w| matches!(w.as_str(), "human" | "operator" | "personnel" | "crew"))
    {
        Domain::Human
    } else if words
        .iter()
        .any(|w| matches!(w.as_str(), "cyber" | "software" | "network" | "service" | "api"))
    {
        Domain::Cyber
    } else if words
        .iter()
        .any(|w| matches!(w.as_str(), "physical" | "hardware" | "mechanical" | "avionics" | "lru"))
    {
        Domain::Physical
    } else {
        Domain::Logical
    }
}

/// [DOC-TIER: A] Returns the required envelope attribute names for a [`Domain`] (`#746`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0402] [FLS: fls_conops_cmark]
/// Contract: Maps `Domain` to its mandatory domain-specific envelope attribute keys.
/// Bounds & Faults: O(1) static slice lookup; infallible.
pub fn required_attributes(domain: Domain) -> &'static [&'static str] {
    match domain {
        Domain::Physical => &["mass_kg", "power_w", "thermal_c", "ip_rating"],
        Domain::Cyber => &["latency_ms", "throughput_mbps"],
        Domain::Logical => &["invariants"],
        Domain::Human => &["workload_index", "authority_tier"],
    }
}

/// [DOC-TIER: A] Validates that a [`SubsystemPart`] does not carry cross-domain physical envelopes (`#746`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0402] [FLS: fls_conops_cmark]
/// Contract: Rejects `mass_kg`, `power_w`, and `ip_rating` on non-`Physical` domain parts.
/// Bounds & Faults: O(A) over part attributes; returns `Err(String)` when physical envelopes leak onto non-physical parts.
pub fn validate_part_envelopes(part: &SubsystemPart, domain: Domain) -> Result<(), String> {
    if domain != Domain::Physical {
        if part.mass_kg.is_some() || part.attributes.contains_key("mass_kg") {
            return Err(format!(
                "Part '{}' in {:?} domain must not declare physical mass_kg envelope",
                part.name, domain
            ));
        }
        if part.power_w.is_some() || part.attributes.contains_key("power_w") {
            return Err(format!(
                "Part '{}' in {:?} domain must not declare physical power_w envelope",
                part.name, domain
            ));
        }
        if part.attributes.contains_key("ip_rating") {
            return Err(format!(
                "Part '{}' in {:?} domain must not declare physical ip_rating envelope",
                part.name, domain
            ));
        }
    }
    Ok(())
}

/// [DOC-TIER: A] SysML v2 M2 metamodel element classification (`#745`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0402] [FLS: fls_conops_cmark]
/// Contract: Enumerates structural, behavioral, requirement, and interface M2 metaclasses.
/// Bounds & Faults: O(1) copy enum; governs allowed envelope kinds per metaclass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M2Type {
    Part,
    Action,
    State,
    Requirement,
    UseCase,
    Port,
    Connection,
}

/// [DOC-TIER: A] UAF performer classification for process-to-performer allocation (`#745`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0402] [FLS: fls_conops_cmark]
/// Contract: Distinguishes `Technical`, `Human`, `Organizational`, and `Environmental` performers.
/// Bounds & Faults: O(1) copy enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformerKind {
    Technical,
    Human,
    Organizational,
    Environmental,
}

/// [DOC-TIER: A] OMG UAF v2.0 viewpoint family taxonomy (`#745`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0402] [FLS: fls_conops_cmark]
/// Contract: Enumerates canonical UAF grid viewpoint families.
/// Bounds & Faults: O(1) copy enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewpointFamily {
    Strategic,
    Operational,
    Services,
    Personnel,
    Resources,
    Security,
    Projects,
    Standards,
}

/// [DOC-TIER: A] Quantitative operating envelope kind for M2 metamodel nodes (`#745`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0402] [FLS: fls_conops_cmark]
/// Contract: Classifies physical, cyber, and human performance envelope dimensions.
/// Bounds & Faults: O(1) copy enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Envelope {
    Mass,
    Power,
    Thermal,
    Ingress,
    Latency,
    Throughput,
    Workload,
    None,
}

/// [DOC-TIER: A] Metamodel AST node with M2 metaclass and bound envelopes (`#745`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0402] [FLS: fls_conops_cmark]
/// Contract: Pairs node `id` and `m2_type` with declared `envelopes`.
/// Bounds & Faults: O(E) over envelope list length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetamodelNode {
    pub id: String,
    pub m2_type: M2Type,
    pub envelopes: Vec<Envelope>,
}

/// [DOC-TIER: A] ISO/IEC/IEEE 15288 process node for process-performer separation checks (`#745`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0402] [FLS: fls_conops_cmark]
/// Contract: Represents an abstract or concrete process node and optional `parent_id`.
/// Bounds & Faults: O(1) record storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub is_abstract_process: bool,
}

/// [DOC-TIER: A] Allocation binding from a `ProcessNode` to a performer (`#745`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0402] [FLS: fls_conops_cmark]
/// Contract: Links `source_process` to `target_performer`.
/// Bounds & Faults: O(1) record storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allocation {
    pub source_process: String,
    pub target_performer: String,
}

/// [DOC-TIER: A] Registry of performer nodes keyed by identifier (`#745`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0402] [FLS: fls_conops_cmark]
/// Contract: Stores `BTreeMap<String, PerformerKind>` for deterministic lookup.
/// Bounds & Faults: O(log P) lookup over registered performers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Performers {
    pub nodes: BTreeMap<String, PerformerKind>,
}

/// [DOC-TIER: A] Classifies a performer identifier token on exact word boundaries (`#745`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0402] [FLS: fls_conops_cmark]
/// Contract: Splits `token` on non-alphanumeric boundaries so `"therapist"` never matches `"api"`.
/// Bounds & Faults: O(N) over `token.len()`; defaults to `PerformerKind::Technical`.
pub fn classify_performer_token(token: &str) -> PerformerKind {
    let words: Vec<String> = token
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    if words.iter().any(|w| {
        matches!(
            w.as_str(),
            "human" | "operator" | "therapist" | "clinician" | "pilot" | "surgeon"
        )
    }) {
        PerformerKind::Human
    } else if words
        .iter()
        .any(|w| matches!(w.as_str(), "org" | "organization" | "authority" | "agency"))
    {
        PerformerKind::Organizational
    } else if words
        .iter()
        .any(|w| matches!(w.as_str(), "environment" | "weather" | "atmosphere" | "terrain"))
    {
        PerformerKind::Environmental
    } else {
        PerformerKind::Technical
    }
}

/// [DOC-TIER: A] Returns the allowed [`Envelope`] kinds for a given [`M2Type`] (`#745`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0402] [FLS: fls_conops_cmark]
/// Contract: Restricts `Mass`/`Power`/`Thermal`/`Ingress` to `M2Type::Part` and timing/throughput to behavioral/interface types.
/// Bounds & Faults: O(1) static slice return.
pub fn allowed_envelopes(m2_type: M2Type) -> &'static [Envelope] {
    match m2_type {
        M2Type::Part => &[
            Envelope::Mass,
            Envelope::Power,
            Envelope::Thermal,
            Envelope::Ingress,
            Envelope::Workload,
            Envelope::None,
        ],
        M2Type::Action | M2Type::UseCase => &[Envelope::Latency, Envelope::Workload, Envelope::None],
        M2Type::Port | M2Type::Connection => {
            &[Envelope::Latency, Envelope::Throughput, Envelope::None]
        }
        M2Type::State | M2Type::Requirement => &[Envelope::None],
    }
}

/// [DOC-TIER: A] Validates that `MetamodelNode` envelopes conform to `allowed_envelopes(node.m2_type)` (`#745`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0402] [FLS: fls_conops_cmark]
/// Contract: Rejects any envelope on `node` not permitted for `node.m2_type`.
/// Bounds & Faults: O(E) over `node.envelopes`; returns descriptive `Err(String)` on violation.
pub fn validate_node_envelopes(node: &MetamodelNode) -> Result<(), String> {
    let allowed = allowed_envelopes(node.m2_type);
    for env in &node.envelopes {
        if !allowed.contains(env) {
            return Err(format!(
                "Metamodel node '{}' ({:?}) cannot carry {:?} envelope",
                node.id, node.m2_type, env
            ));
        }
    }
    Ok(())
}

/// [DOC-TIER: A] Validates process-to-performer separation and allocation integrity (`#745`).
/// Safety & Traceability: Conforms to DO-178C Level A / DO-330 TQL-4; Realises: [REQ-0402] [FLS: fls_conops_cmark]
/// Contract: Ensures processes are not registered as performers and every process has a valid performer allocation.
/// Bounds & Faults: O(P * A) over processes and allocations; returns `Err(String)` on unallocated or conflated nodes.
pub fn validate_process_layer(
    processes: &[ProcessNode],
    allocations: &[Allocation],
    performers: &Performers,
) -> Result<(), String> {
    for proc_node in processes {
        if performers.nodes.contains_key(&proc_node.id) {
            return Err(format!(
                "Process '{}' is conflated with a Performer node",
                proc_node.id
            ));
        }
        if let Some(parent) = &proc_node.parent_id {
            if performers.nodes.contains_key(parent) {
                return Err(format!(
                    "Process '{}' cannot be nested under Performer '{parent}'",
                    proc_node.id
                ));
            }
        }
        let has_alloc = allocations.iter().any(|a| {
            a.source_process == proc_node.id && performers.nodes.contains_key(&a.target_performer)
        });
        if !has_alloc {
            return Err(format!(
                "Process '{}' has no valid Performer allocation",
                proc_node.id
            ));
        }
    }
    Ok(())
}
