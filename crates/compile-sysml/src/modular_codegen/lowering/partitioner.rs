//! Hierarchical Level-1 package and Level-2 part-facet partitioner for SysML v2 modular code generation.
//! ## 1. Safety Intent & Regulatory Scope
//! Partitions parsed SysML v2 `PackageDef` hierarchies (`partitioner.rs`) into single-responsibility Rust compilation units under DO-178C and ISO 26262 modularity rules.
//! ## 2. Invariants & Mathematical Properties
//! Accepts borrowed `PackageDef` AST trees and validated `ModularityBounds` thresholds without filesystem I/O or ambient mutable state.
//! ## 3. Preconditions
//! Verifies downward `@Layer` stratification and DAG acyclicity before emitting deterministic package barrels, part facets, and sharded behavioral units.
//! ## 4. Postconditions
//! Propagates `E0702` cycle and `E0703` upward-layer diagnostics via `Result<ModuleGraph, Vec<ModularityDiagnostic>>` with zero panics or unchecked traps.
//! ## 5. Algorithmic Complexity Bounds
//! Traverses package hierarchies iteratively with explicit `Vec` workstacks and ordered `BTreeMap`/`BTreeSet` indices while bounding every function to `v(G) <= 7`.
//! ## 6. Failure Modes & Fault Tolerance
//! Verified by unit and mutation tests exercising multi-package DAG lowering, part facet chunking, action/state sharding, and cycle/layer rejection gates.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]
//! Anchors the lowering-tier AST-to-`ModuleGraph` partitioning pipeline for `compile-sysml` modular Rust code generation.

use super::dag_verifier::verify_dag_and_layers;
use super::outliner::to_snake_slug;
use super::partition_facets::{
    emit_action_units, emit_state_units, empty_unit, join_rel, partition_part_facets,
    PackageFacetCtx,
};
use crate::modular_codegen::domain::{
    CompilationUnit, DependencyEdge, LayerRole, ModularityBounds, ModularityDiagnostic,
    ModuleGraph, UnitKind,
};
use deap_core::sysml_cst::{PackageDef, PartDef};
use std::collections::{BTreeMap, BTreeSet};

type ClassifierRegistry = BTreeMap<String, (String, LayerRole)>;

/// Iteratively indexes all `PartDef` and `PortDef` classifiers across a `PackageDef` tree.
///
/// Realises: [REQ-0607]
fn collect_classifier_registry(root: &PackageDef) -> (ClassifierRegistry, Vec<String>) {
    let (mut reg, mut ids, mut stack) = (
        ClassifierRegistry::new(),
        Vec::new(),
        vec![(root, Vec::<String>::new())],
    );
    while let Some((pkg, path)) = stack.pop() {
        let (prefix, pkg_layer) = (path.join("::"), LayerRole::resolve_part_layer(None, &path));
        for part in pkg.part_defs.iter().filter(|p| p.is_def) {
            let (id, layer) = (
                format!("{prefix}::{}", part.name),
                LayerRole::resolve_part_layer(part.layer.as_deref(), &path),
            );
            reg.insert(part.name.clone(), (id.clone(), layer));
            ids.push(id);
        }
        for port in &pkg.port_defs {
            let id = format!("{prefix}::{}", port.name);
            reg.insert(port.name.clone(), (id.clone(), pkg_layer));
            ids.push(id);
        }
        for sub in pkg.packages.iter().rev() {
            stack.push((
                sub,
                path.iter()
                    .cloned()
                    .chain(std::iter::once(to_snake_slug(&sub.name)))
                    .collect(),
            ));
        }
    }
    (reg, ids)
}

/// Appends directed `DependencyEdge` records for every registered classifier referenced by a `PartDef`.
///
/// Realises: [REQ-0607]
fn record_part_edges(
    part: &PartDef,
    from_id: &str,
    from_layer: LayerRole,
    reg: &ClassifierRegistry,
    edges: &mut Vec<DependencyEdge>,
) {
    let targets = part
        .type_name
        .iter()
        .map(String::as_str)
        .chain(
            part.parts
                .iter()
                .map(|u| u.type_name.as_deref().unwrap_or(u.name.as_str())),
        )
        .chain(part.ports.iter().map(|p| p.type_name.as_str()));
    for raw in targets {
        let clean = raw.rsplit("::").next().unwrap_or(raw).trim();
        if let Some((to_id, to_layer)) = reg.get(clean).filter(|_| !clean.is_empty()) {
            edges.push(DependencyEdge {
                from_unit: from_id.to_string(),
                to_unit: to_id.clone(),
                from_layer,
                to_layer: *to_layer,
                reason: clean.to_string(),
            });
        }
    }
}

/// Iteratively harvests all inter-classifier `DependencyEdge` relations across a `PackageDef` hierarchy.
///
/// Realises: [REQ-0607]
fn collect_dependency_edges(root: &PackageDef, reg: &ClassifierRegistry) -> Vec<DependencyEdge> {
    let (mut edges, mut stack) = (Vec::new(), vec![root]);
    while let Some(pkg) = stack.pop() {
        for part in &pkg.part_defs {
            if let Some((from_id, from_layer)) = reg.get(&part.name).filter(|_| part.is_def) {
                record_part_edges(part, from_id, *from_layer, reg, &mut edges);
            }
        }
        stack.extend(pkg.packages.iter().rev());
    }
    edges
}

/// Lowers all immediate members of one `PackageDef` into child `CompilationUnit` items and a barrel.
///
/// Realises: [REQ-0607]
fn partition_single_package(
    pkg: &PackageDef,
    pkg_path: &[String],
    bounds: &ModularityBounds,
    units: &mut Vec<CompilationUnit>,
) {
    let (pkg_dir, pkg_prefix, layer) = (
        pkg_path.join("/"),
        pkg_path.join("::"),
        LayerRole::resolve_part_layer(None, pkg_path),
    );
    let ctx = PackageFacetCtx {
        pkg_dir: &pkg_dir,
        pkg_path,
        layer,
        bounds,
    };
    let mut raw_slugs: Vec<String> = pkg
        .packages
        .iter()
        .map(|s| to_snake_slug(&s.name))
        .collect();
    for p in pkg.part_defs.iter().filter(|p| p.is_def) {
        raw_slugs.push(partition_part_facets(p, &ctx, units));
    }
    for port in &pkg.port_defs {
        let slug = to_snake_slug(&port.name);
        units.push(CompilationUnit {
            port_defs: vec![port.clone()],
            ..empty_unit(
                format!("{pkg_prefix}::{}", port.name),
                join_rel(&pkg_dir, &format!("{slug}.rs")),
                pkg_path.to_vec(),
                layer,
                UnitKind::StandalonePort,
            )
        });
        raw_slugs.push(slug);
    }
    for calc in &pkg.calc_defs {
        let slug = to_snake_slug(&calc.name);
        units.push(CompilationUnit {
            calc_defs: vec![calc.clone()],
            ..empty_unit(
                format!("{pkg_prefix}::{}", calc.name),
                join_rel(&pkg_dir, &format!("{slug}.rs")),
                pkg_path.to_vec(),
                layer,
                UnitKind::StandaloneCalc,
            )
        });
        raw_slugs.push(slug);
    }
    for act in pkg.action_defs.iter().filter(|a| a.is_def) {
        raw_slugs.push(emit_action_units(act, &ctx, units));
    }
    for st in &pkg.state_defs {
        raw_slugs.push(emit_state_units(st, &ctx, units));
    }
    let mut seen = BTreeSet::new();
    units.push(CompilationUnit {
        child_modules: raw_slugs
            .into_iter()
            .filter(|s| seen.insert(s.clone()))
            .collect(),
        ..empty_unit(
            format!("{pkg_prefix}::mod"),
            join_rel(&pkg_dir, "mod.rs"),
            pkg_path.to_vec(),
            layer,
            UnitKind::BarrelFacade,
        )
    });
}

/// Lowers a SysML v2 root `PackageDef` hierarchy into a cycle-verified, layer-stratified `ModuleGraph`.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub fn lower_package_to_module_graph(pkg: &PackageDef, bounds: &ModularityBounds) -> Result<ModuleGraph, Vec<ModularityDiagnostic>>` gates modular partitioning behind `E0702` DAG and `E0703` `@Layer` verification.
/// ## 2. Invariants & Mathematical Properties
/// `pub fn lower_package_to_module_graph(pkg: &PackageDef, bounds: &ModularityBounds) -> Result<ModuleGraph, Vec<ModularityDiagnostic>>` borrows a parsed root `PackageDef` and `ModularityBounds` configuration.
/// ## 3. Preconditions
/// `pub fn lower_package_to_module_graph(pkg: &PackageDef, bounds: &ModularityBounds) -> Result<ModuleGraph, Vec<ModularityDiagnostic>>` returns `Ok(ModuleGraph)` with preorder partitioned `units`, `edges`, and `topological_order`.
/// ## 4. Postconditions
/// `pub fn lower_package_to_module_graph(pkg: &PackageDef, bounds: &ModularityBounds) -> Result<ModuleGraph, Vec<ModularityDiagnostic>>` short-circuits with `Err(diagnostics)` whenever upward layer imports or cycles exist.
/// ## 5. Algorithmic Complexity Bounds
/// `pub fn lower_package_to_module_graph(pkg: &PackageDef, bounds: &ModularityBounds) -> Result<ModuleGraph, Vec<ModularityDiagnostic>>` executes iteratively via explicit `Vec` workstacks with zero recursion at `v(G) = 4`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by testing `pub fn lower_package_to_module_graph(pkg: &PackageDef, bounds: &ModularityBounds) -> Result<ModuleGraph, Vec<ModularityDiagnostic>>` on stratified multi-package schemas and negative `E0702`/`E0703` probes.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
pub fn lower_package_to_module_graph(
    pkg: &PackageDef,
    bounds: &ModularityBounds,
) -> Result<ModuleGraph, Vec<ModularityDiagnostic>> {
    let (reg, node_ids) = collect_classifier_registry(pkg);
    let edges = collect_dependency_edges(pkg, &reg);
    let topological_order = verify_dag_and_layers(&node_ids, &edges)?;
    let (mut units, mut stack) = (Vec::new(), vec![(pkg, Vec::<String>::new())]);
    while let Some((cur_pkg, cur_path)) = stack.pop() {
        partition_single_package(cur_pkg, &cur_path, bounds, &mut units);
        for sub in cur_pkg.packages.iter().rev() {
            stack.push((
                sub,
                cur_path
                    .iter()
                    .cloned()
                    .chain(std::iter::once(to_snake_slug(&sub.name)))
                    .collect(),
            ));
        }
    }
    Ok(ModuleGraph {
        root_package_name: pkg.name.clone(),
        units,
        edges,
        topological_order,
    })
}
