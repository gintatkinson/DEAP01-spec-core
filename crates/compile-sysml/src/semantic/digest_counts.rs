#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Iterative AST element counting and schema node inventory collection for `SchemaDigest`.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for AST metrics.
//! ## 2. Invariants & Mathematical Properties
//! Traverses package and part hierarchies iteratively using explicit work-stacks and saturating arithmetic.
//! ## 3. Preconditions
//! Caller supplies a valid borrowed `PackageDef` or `PartDef` AST node.
//! ## 4. Postconditions
//! Produces deterministic `BTreeMap<String, usize>` node counts and sorted unique `Vec<String>` schema node names.
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: O(V log V) over AST node count `V`. Space Complexity: O(V) work-stack and set storage.
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface / State / Action / Resource: Zero call-stack recursion, zero overflow via `saturating_add`, zero panics.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0603, REQ-SEMANTIC-DIGEST-COUNT-STRUCTURAL-ELEMENTS, REQ-SEMANTIC-DIGEST-COMPUTE-NODE-COUNTS]

use deap_core::sysml_ast::{PackageDef, PartDef};
use std::collections::{BTreeMap, BTreeSet};

/// Iteratively counts all direct and nested structural AST elements within a `PartDef` subtree.
/// Realises: [REQ-SEMANTIC-DIGEST-COUNT-PART-STRUCTURAL-ELEMENTS]
fn count_part_structural_elements(root_part: &PartDef) -> usize {
    let mut total: usize = 0;
    let mut stack: Vec<&PartDef> = vec![root_part];
    while let Some(part) = stack.pop() {
        let direct = part
            .parts
            .len()
            .saturating_add(part.attributes.len())
            .saturating_add(part.ports.len())
            .saturating_add(part.actions.len())
            .saturating_add(part.operations.len())
            .saturating_add(part.capabilities.len())
            .saturating_add(part.interactions.len())
            .saturating_add(part.constraints.len())
            .saturating_add(part.calcs.len())
            .saturating_add(part.test_cases.len())
            .saturating_add(part.requirements.len())
            .saturating_add(part.connections.len())
            .saturating_add(part.states.len())
            .saturating_add(part.use_cases.len())
            .saturating_add(part.item_defs.len())
            .saturating_add(part.hazards.len())
            .saturating_add(part.risks.len());
        total = total.saturating_add(direct);
        for child in part.parts.iter().rev() {
            stack.push(child);
        }
    }
    total
}

/// Counts all structural AST definitions across a `PackageDef` hierarchy using iterative work-stacks.
///
/// Safety & Traceability: Realises: [REQ-SEMANTIC-DIGEST-COUNT-STRUCTURAL-ELEMENTS]
/// Contract: Traverses `pkg`, all nested `packages`, and all nested `PartDef` trees in deterministic pre-order.
/// Bounds & Faults: O(V) time and O(V) heap work-stack space over AST node count V; never panics or overflows.
pub fn count_structural_elements(pkg: &PackageDef) -> usize {
    let mut total: usize = 0;
    let mut stack: Vec<&PackageDef> = vec![pkg];
    while let Some(curr) = stack.pop() {
        let direct = curr
            .part_defs
            .len()
            .saturating_add(curr.attribute_defs.len())
            .saturating_add(curr.port_defs.len())
            .saturating_add(curr.action_defs.len())
            .saturating_add(curr.operation_defs.len())
            .saturating_add(curr.capability_defs.len())
            .saturating_add(curr.interaction_defs.len())
            .saturating_add(curr.constraint_defs.len())
            .saturating_add(curr.calc_defs.len())
            .saturating_add(curr.test_case_defs.len())
            .saturating_add(curr.requirement_defs.len())
            .saturating_add(curr.connection_defs.len())
            .saturating_add(curr.state_defs.len())
            .saturating_add(curr.use_case_defs.len())
            .saturating_add(curr.item_defs.len())
            .saturating_add(curr.hazard_defs.len())
            .saturating_add(curr.risk_defs.len());
        total = total.saturating_add(direct);
        for part in &curr.part_defs {
            total = total.saturating_add(count_part_structural_elements(part));
        }
        for sub_pkg in curr.packages.iter().rev() {
            stack.push(sub_pkg);
        }
    }
    total
}

/// Computes per-kind AST definition counts across `pkg`.
/// Realises: [REQ-SEMANTIC-DIGEST-COMPUTE-NODE-COUNTS] | SysML v2: ActionDef(compute_node_counts)
pub(crate) fn compute_node_counts(pkg: &PackageDef) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for key in [
        "packages",
        "part_defs",
        "attribute_defs",
        "port_defs",
        "action_defs",
        "capability_defs",
        "operation_defs",
        "interaction_defs",
        "constraint_defs",
        "calc_defs",
        "test_case_defs",
        "requirement_defs",
        "state_defs",
        "use_case_defs",
        "item_defs",
        "hazard_defs",
        "risk_defs",
        "connection_defs",
        "containers",
        "lists",
        "leaves",
        "typedefs",
        "identities",
        "groupings",
    ] {
        counts.insert(key.to_string(), usize::from(key == "packages"));
    }

    accumulate_package_counts(pkg, &mut counts);

    let containers = *counts.get("part_defs").unwrap_or(&0);
    let lists = *counts.get("action_defs").unwrap_or(&0);
    let leaves = *counts.get("attribute_defs").unwrap_or(&0);

    counts.insert("containers".to_string(), containers);
    counts.insert("lists".to_string(), lists);
    counts.insert("leaves".to_string(), leaves);

    counts
}

/// Increments `key` in `counts` by `n` using saturating addition so counters never overflow.
/// Realises: [REQ-SEMANTIC-DIGEST-BUMP-COUNT]
fn bump_count(counts: &mut BTreeMap<String, usize>, key: &str, n: usize) {
    let entry = counts.entry(key.to_string()).or_insert(0);
    *entry = entry.saturating_add(n);
}

/// Iteratively accumulates per-kind AST definition counts across `pkg` and all nested subpackages.
/// Realises: [REQ-SEMANTIC-DIGEST-ACCUMULATE-PACKAGE-COUNTS]
fn accumulate_package_counts(pkg: &PackageDef, counts: &mut BTreeMap<String, usize>) {
    let mut stack: Vec<&PackageDef> = vec![pkg];
    while let Some(curr) = stack.pop() {
        bump_count(counts, "attribute_defs", curr.attribute_defs.len());
        bump_count(counts, "port_defs", curr.port_defs.len());
        bump_count(counts, "action_defs", curr.action_defs.len());
        bump_count(counts, "capability_defs", curr.capability_defs.len());
        bump_count(counts, "operation_defs", curr.operation_defs.len());
        bump_count(counts, "interaction_defs", curr.interaction_defs.len());
        bump_count(counts, "constraint_defs", curr.constraint_defs.len());
        bump_count(counts, "calc_defs", curr.calc_defs.len());
        bump_count(counts, "test_case_defs", curr.test_case_defs.len());
        bump_count(counts, "requirement_defs", curr.requirement_defs.len());
        bump_count(counts, "state_defs", curr.state_defs.len());
        bump_count(counts, "use_case_defs", curr.use_case_defs.len());
        bump_count(counts, "item_defs", curr.item_defs.len());
        bump_count(counts, "hazard_defs", curr.hazard_defs.len());
        bump_count(counts, "risk_defs", curr.risk_defs.len());
        bump_count(counts, "connection_defs", curr.connection_defs.len());
        bump_count(counts, "part_defs", curr.part_defs.len());

        for part in &curr.part_defs {
            accumulate_part_counts(part, counts);
        }

        for subpkg in curr.packages.iter().rev() {
            bump_count(counts, "packages", 1);
            stack.push(subpkg);
        }
    }
}

/// Iteratively accumulates per-kind AST definition counts across a `PartDef` composition tree.
/// Realises: [REQ-SEMANTIC-DIGEST-ACCUMULATE-PART-COUNTS]
fn accumulate_part_counts(root_part: &PartDef, counts: &mut BTreeMap<String, usize>) {
    let mut stack: Vec<&PartDef> = vec![root_part];
    while let Some(part) = stack.pop() {
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

        for sub in part.parts.iter().rev() {
            stack.push(sub);
        }
    }
}

/// Inserts direct non-recursive definition names from `curr` into `nodes`.
/// Realises: [REQ-SEMANTIC-DIGEST-COLLECT-SCHEMA-NODES]
fn insert_package_leaf_names(curr: &PackageDef, nodes: &mut BTreeSet<String>) {
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
}

/// Iteratively collects sorted unique identifier names across `pkg`, nested subpackages, and parts.
/// Realises: [REQ-SEMANTIC-DIGEST-COLLECT-SCHEMA-NODES]
pub(crate) fn collect_schema_nodes(pkg: &PackageDef) -> Vec<String> {
    let mut nodes = BTreeSet::new();
    let mut stack: Vec<&PackageDef> = vec![pkg];
    while let Some(curr) = stack.pop() {
        insert_package_leaf_names(curr, &mut nodes);
        for part in &curr.part_defs {
            collect_part_nodes(part, &mut nodes);
        }
        for sub in curr.packages.iter().rev() {
            stack.push(sub);
        }
    }
    nodes.into_iter().collect()
}

/// Iteratively collects identifier names across a `PartDef` composition subtree into `nodes`.
/// Realises: [REQ-SEMANTIC-DIGEST-COLLECT-PART-NODES]
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
        for sub in part.parts.iter().rev() {
            stack.push(sub);
        }
    }
}
