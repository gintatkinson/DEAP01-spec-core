//! Cycle-freedom DAG topological verifier and architectural layer coupling gate for modular code generation.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Module `modular_codegen::lowering::dag_verifier` enforces directed acyclic dependency structure (`E0702`)
//! and downward-only architectural stratification (`E0703`) under DO-178C and ISO 26262 partitioning rules.
//! ## 2. Invariants & Mathematical Properties
//! Callers supply borrowed compilation-unit identifier slices (`&[String]`) and directed `DependencyEdge` slices.
//! ## 3. Preconditions
//! Produces either a deterministic lexicographically tie-broken topological emission order or structured `ModularityDiagnostic` lists.
//! ## 4. Postconditions
//! All graph and layer checks are total and non-aborting, surfacing violations strictly via `UpwardLayerImportE0703` and `DirectedCycleE0702`.
//! ## 5. Algorithmic Complexity Bounds
//! Uses iterative Kahn traversal over ordered `BTreeMap` and `BTreeSet` containers in `O((|V| + |E|) log |V|)` time with zero recursion.
//! ## 6. Failure Modes & Fault Tolerance
//! Verified via unit and mutation probes covering downward and same-tier edges, upward `@Layer` rejections, tie-breaking, and multi-node cycles.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]
//! Module anchor: `crates/compile-sysml/src/modular_codegen/lowering/dag_verifier.rs`.

use std::collections::{BTreeMap, BTreeSet};

use crate::modular_codegen::domain::{DependencyEdge, ModularityDiagnostic, ModularityErrorCode};

/// Audits all inter-unit directed dependency edges for forbidden upward architectural `@Layer` coupling (`E0703`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// `pub fn verify_layer_edges` prevents lower-tier `Domain` or `Application` partitions from importing higher-tier abstractions.
/// ## 2. Invariants & Mathematical Properties
/// Accepts an immutable `edges: &[DependencyEdge]` slice carrying pre-resolved `from_layer`, `to_layer`, and traceability `reason` fields.
/// ## 3. Preconditions
/// Ignores intra-unit self-references (`from_unit == to_unit`) and emits one `UpwardLayerImportE0703` diagnostic per upward inter-unit edge.
/// ## 4. Postconditions
/// Never aborts on malformed or cyclic edge sets; returns an empty `Vec` when every distinct-unit edge satisfies `permits_dependency_on`.
/// ## 5. Algorithmic Complexity Bounds
/// Scans `edges` sequentially in `O(|E|)` time and `O(|E_invalid|)` diagnostic space with cyclomatic complexity `v(G) = 4`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified against the `3x3` `LayerRole` coupling matrix, intra-unit self-edges, and exact `"E0703"` tier-annotated message formatting.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Enforces unidirectional layer stratification across partitioned `SysML` v2 compilation units prior to Rust source emission.
#[must_use]
pub fn verify_layer_edges(edges: &[DependencyEdge]) -> Vec<ModularityDiagnostic> {
    let mut diagnostics = Vec::new();
    for edge in edges {
        if edge.from_unit != edge.to_unit && !edge.from_layer.permits_dependency_on(edge.to_layer) {
            diagnostics.push(ModularityDiagnostic::new(
                ModularityErrorCode::UpwardLayerImportE0703,
                format!("{} -> {}", edge.from_unit, edge.to_unit),
                format!(
                    "illegal upward @Layer dependency from {} (tier {}) to {} (tier {}): {}",
                    edge.from_layer.as_str(),
                    edge.from_layer.tier(),
                    edge.to_layer.as_str(),
                    edge.to_layer.tier(),
                    edge.reason
                ),
            ));
        }
    }
    diagnostics
}

/// Constructs deduplicated ordered adjacency sets and vertex in-degree counters for registered graph nodes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// `fn build_adjacency_and_indegree` isolates deterministic graph indexing so duplicate edges or unknown endpoints cannot skew cycle detection.
/// ## 2. Invariants & Mathematical Properties
/// Borrows `node_ids: &[String]` enumerating declared compilation units and `edges: &[DependencyEdge]` listing candidate directed dependencies.
/// ## 3. Preconditions
/// Initializes every declared node in `in_degree` to `0`, records in-bounds unique `from_unit -> to_unit` adjacencies, and counts unique incoming edges.
/// ## 4. Postconditions
/// Safely ignores edges referencing unregistered `from_unit` or `to_unit` identifiers and bounds counter updates via `saturating_add(1)`.
/// ## 5. Algorithmic Complexity Bounds
/// Runs in `O((|V| + |E|) log |V|)` time over `BTreeMap` and `BTreeSet` structures with zero recursion and cyclomatic complexity `v(G) = 5`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by testing duplicate parallel edges, out-of-graph dangling endpoints, and self-loop `in_degree` registration.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Supplies normalized adjacency and in-degree tables to `run_kahn_topological_sort`.
fn build_adjacency_and_indegree(
    node_ids: &[String],
    edges: &[DependencyEdge],
) -> (BTreeMap<String, BTreeSet<String>>, BTreeMap<String, usize>) {
    let mut in_degree: BTreeMap<String, usize> =
        node_ids.iter().map(|id| (id.clone(), 0)).collect();
    let mut adj: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for edge in edges {
        if in_degree.contains_key(&edge.from_unit) && in_degree.contains_key(&edge.to_unit) {
            let inserted = adj
                .entry(edge.from_unit.clone())
                .or_default()
                .insert(edge.to_unit.clone());
            if let Some(deg) = in_degree.get_mut(&edge.to_unit).filter(|_| inserted) {
                *deg = deg.saturating_add(1);
            }
        }
    }
    (adj, in_degree)
}

/// Decrements outgoing neighbor in-degrees for a popped vertex and enqueues newly unblocked zero-in-degree vertices.
///
/// ## 1. Safety Intent & Regulatory Scope
/// `fn release_ready_neighbors` bounds per-function control-flow nesting inside iterative Kahn topological traversal.
/// ## 2. Invariants & Mathematical Properties
/// Accepts popped vertex identifier `u`, immutable `adj` map, mutable `in_degree` map, and mutable `ready` ordered workset.
/// ## 3. Preconditions
/// Subtracts `1` from `in_degree[neighbor]` for every outgoing neighbor of `u` and inserts each neighbor reaching `0` into `ready`.
/// ## 4. Postconditions
/// Leaf vertices absent from `adj` return immediately, and `saturating_sub(1)` prevents unsigned arithmetic underflow.
/// ## 5. Algorithmic Complexity Bounds
/// Traverses `adj[u]` in lexicographical `BTreeSet` order in `O(deg_out(u) log |V|)` time with cyclomatic complexity `v(G) = 5`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified across leaf nodes, diamond reconvergent DAGs, and partially reducible graphs feeding downstream cycles.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Executes the single-vertex edge-relaxation step for `run_kahn_topological_sort`.
fn release_ready_neighbors(
    u: &str,
    adj: &BTreeMap<String, BTreeSet<String>>,
    in_degree: &mut BTreeMap<String, usize>,
    ready: &mut BTreeSet<String>,
) {
    let Some(neighbors) = adj.get(u) else {
        return;
    };
    for neighbor in neighbors {
        if let Some(deg) = in_degree.get_mut(neighbor) {
            *deg = deg.saturating_sub(1);
            if *deg == 0 {
                ready.insert(neighbor.clone());
            }
        }
    }
}

/// Computes a deterministic topological ordering of `node_ids` via iterative Kahn's algorithm or rejects cyclic graphs (`E0702`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// `pub fn run_kahn_topological_sort` guarantees cycle-free module dependencies and reproducible emission sequencing without call-stack recursion.
/// ## 2. Invariants & Mathematical Properties
/// Accepts `node_ids: &[String]` declaring graph vertices and `edges: &[DependencyEdge]` specifying directed `from_unit -> to_unit` constraints.
/// ## 3. Preconditions
/// Pops ready vertices via `BTreeSet::pop_first` for lexicographical tie-breaking, returning `Ok(order)` iff every registered vertex reaches `0` in-degree.
/// ## 4. Postconditions
/// When `order.len() < in_degree.len()`, collects sorted remaining vertices (`deg > 0`) and returns `Err` with `DirectedCycleE0702`.
/// ## 5. Algorithmic Complexity Bounds
/// Executes iteratively in `O((|V| + |E|) log |V|)` time and `O(|V| + |E|)` heap memory with cyclomatic complexity `v(G) = 3`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified on empty graphs, disconnected components with shuffled `node_ids`, diamond DAGs, self-loops, and multi-node cycles.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Provides the authoritative `E0702` cycle detector and canonical topological sorter for `ModuleGraph` construction.
pub fn run_kahn_topological_sort(
    node_ids: &[String],
    edges: &[DependencyEdge],
) -> Result<Vec<String>, Vec<ModularityDiagnostic>> {
    let (adj, mut in_degree) = build_adjacency_and_indegree(node_ids, edges);
    let mut ready: BTreeSet<String> = in_degree
        .iter()
        .filter(|(_, deg)| **deg == 0)
        .map(|(id, _)| id.clone())
        .collect();
    let mut order = Vec::with_capacity(in_degree.len());
    while let Some(u) = ready.pop_first() {
        release_ready_neighbors(&u, &adj, &mut in_degree, &mut ready);
        order.push(u);
    }
    if order.len() < in_degree.len() {
        let participants: Vec<String> = in_degree
            .into_iter()
            .filter(|(_, deg)| *deg > 0)
            .map(|(id, _)| id)
            .collect();
        return Err(vec![ModularityDiagnostic::new(
            ModularityErrorCode::DirectedCycleE0702,
            participants.join(" -> "),
            format!(
                "directed dependency cycle detected across [{}]",
                participants.join(", ")
            ),
        )]);
    }
    Ok(order)
}

/// Validates both downward-only `@Layer` coupling (`E0703`) and DAG cycle freedom (`E0702`) in a single combined pass.
///
/// ## 1. Safety Intent & Regulatory Scope
/// `pub fn verify_dag_and_layers` ensures architectural layer violations and directed cycles are reported together without masking either defect class.
/// ## 2. Invariants & Mathematical Properties
/// Accepts `node_ids: &[String]` and `edges: &[DependencyEdge]` representing a partitioned `SysML` v2 compilation-unit dependency graph.
/// ## 3. Preconditions
/// Returns `Ok(order)` exclusively when `verify_layer_edges` produces zero diagnostics and `run_kahn_topological_sort` succeeds.
/// ## 4. Postconditions
/// Accumulates all `E0703` upward-layer diagnostics ahead of any `E0702` cycle diagnostics and returns `Err(diags)` whenever `diags` is non-empty.
/// ## 5. Algorithmic Complexity Bounds
/// Composes `O(|E|)` layer scanning with `O((|V| + |E|) log |V|)` Kahn sorting in bounded iterative space with cyclomatic complexity `v(G) = 4`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified across all four quadrant outcomes: clean stratified DAG, layer-only violation, cycle-only violation, and simultaneous layer-plus-cycle faults.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Serves as the primary lowering-tier DAG and stratification verification entry point for modular code generation.
pub fn verify_dag_and_layers(
    node_ids: &[String],
    edges: &[DependencyEdge],
) -> Result<Vec<String>, Vec<ModularityDiagnostic>> {
    let mut diags = verify_layer_edges(edges);
    match run_kahn_topological_sort(node_ids, edges) {
        Ok(order) if diags.is_empty() => Ok(order),
        Ok(_) => Err(diags),
        Err(cycle_diags) => {
            diags.extend(cycle_diags);
            Err(diags)
        }
    }
}
