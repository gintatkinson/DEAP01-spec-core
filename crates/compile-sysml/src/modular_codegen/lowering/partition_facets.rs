//! Level-2 part-facet and sharded behavioral compilation-unit builders for modular Rust lowering.
//!
//! Safety & Traceability:
//! Realises: [REQ-0607]
//! Partitions individual SysML v2 `PartDef`, `ActionDef`, and `StateDef` classifiers into bounded
//! compilation units under DO-178C and ISO 26262 modularity rules.
//!
//! Contract:
//! Accepts borrowed package path context and AST definitions, appending deterministic facet and
//! shard `CompilationUnit` entries without filesystem I/O or ambient mutable state.
//!
//! Bounds & Faults:
//! Runs in linear-log time over definition members using saturating shard counters (`v(G) <= 7`)
//! with zero panics, unchecked slice indexing, or narrowing casts.

use super::expr_anf::hoist_constraint_anf_with_params;
use super::outliner::{outline_action_def, outline_state_def, to_pascal_ident, to_snake_slug};
use crate::modular_codegen::domain::{
    CompilationUnit, LayerRole, ModularityBounds, OutlinedActionPlan, OutlinedStateMachinePlan,
    UnitKind,
};
use deap_core::sysml_cst::{ActionDef, PartDef, StateDef};
use std::collections::BTreeSet;

/// Borrowed package lowering context shared across Level-2 facet and behavioral unit builders.
///
/// Realises: [REQ-0607]
pub(super) struct PackageFacetCtx<'a> {
    pub(super) pkg_dir: &'a str,
    pub(super) pkg_path: &'a [String],
    pub(super) layer: LayerRole,
    pub(super) bounds: &'a ModularityBounds,
}

/// Constructs a blank `CompilationUnit` initialized with empty AST payload vectors.
///
/// Realises: [REQ-0607]
pub(super) fn empty_unit(
    unit_id: String,
    relative_path: String,
    package_path: Vec<String>,
    layer: LayerRole,
    kind: UnitKind,
) -> CompilationUnit {
    CompilationUnit {
        unit_id,
        relative_path,
        package_path,
        layer,
        kind,
        child_modules: Vec::new(),
        reexport_children: false,
        part_def: None,
        port_defs: Vec::new(),
        calc_defs: Vec::new(),
        hoisted_predicates: Vec::new(),
        action_plans: Vec::new(),
        state_plans: Vec::new(),
    }
}

/// Joins an optional relative parent directory prefix and a child module path segment using `/`.
///
/// Realises: [REQ-0607]
pub(super) fn join_rel(dir: &str, leaf: &str) -> String {
    if dir.is_empty() {
        leaf.to_string()
    } else {
        format!("{dir}/{leaf}")
    }
}

/// Extracts normalized `snake_case` attribute field names declared on a `PartDef`.
///
/// Realises: [REQ-0607]
fn extract_part_field_names(part: &PartDef) -> BTreeSet<String> {
    part.attributes
        .iter()
        .map(|a| to_snake_slug(&a.name))
        .collect()
}

/// Computes the single or 1-based sharded facet module stem (`<prefix>` or `<prefix>_NN`).
///
/// Realises: [REQ-0607]
fn shard_facet_name(prefix: &str, total: usize, limit: usize, idx: usize) -> String {
    if total <= limit {
        prefix.to_string()
    } else {
        format!("{prefix}_{:02}", idx.saturating_add(1))
    }
}

/// Lowers `PartDef` constraints into ANF predicates and appends sharded `invariants` facet units.
///
/// Realises: [REQ-0607]
fn emit_part_invariants_facets(
    part: &PartDef,
    ctx: &PackageFacetCtx<'_>,
    (base_id, part_dir): (&str, &str),
    units: &mut Vec<CompilationUnit>,
) -> Vec<String> {
    let layer = LayerRole::resolve_part_layer(part.layer.as_deref(), ctx.pkg_path);
    let (fields, ctx_ty) = (extract_part_field_names(part), to_pascal_ident(&part.name));
    let preds: Vec<_> = part
        .constraints
        .iter()
        .flat_map(|c| {
            hoist_constraint_anf_with_params(
                (&to_snake_slug(&c.name), &ctx_ty),
                &fields,
                &c.parameters,
                &c.expression,
                ctx.bounds.max_cyclomatic_complexity,
            )
        })
        .collect();
    let mut names = Vec::new();
    for (i, chunk) in preds.chunks(6).enumerate() {
        let m = shard_facet_name("invariants", preds.len(), 6, i);
        units.push(CompilationUnit {
            part_def: Some(part.clone()),
            hoisted_predicates: chunk.to_vec(),
            ..empty_unit(
                format!("{base_id}::{m}"),
                join_rel(part_dir, &format!("{m}.rs")),
                ctx.pkg_path.to_vec(),
                layer,
                UnitKind::PartInvariantsFacet,
            )
        });
        names.push(m);
    }
    names
}

/// Decomposes a `PartDef` into Level-2 model, sharded invariant/port/calc facets, and a barrel.
///
/// Realises: [REQ-0607]
pub(super) fn partition_part_facets(
    part: &PartDef,
    ctx: &PackageFacetCtx<'_>,
    units: &mut Vec<CompilationUnit>,
) -> String {
    let layer = LayerRole::resolve_part_layer(part.layer.as_deref(), ctx.pkg_path);
    let (part_slug, base_id) = (
        to_snake_slug(&part.name),
        format!("{}::{}", ctx.pkg_path.join("::"), part.name),
    );
    let (part_dir, pkg_vec) = (join_rel(ctx.pkg_dir, &part_slug), ctx.pkg_path.to_vec());
    let mut children = vec!["model".to_string()];
    units.push(CompilationUnit {
        part_def: Some(part.clone()),
        ..empty_unit(
            format!("{base_id}::model"),
            join_rel(&part_dir, "model.rs"),
            pkg_vec.clone(),
            layer,
            UnitKind::PartModelFacet,
        )
    });
    children.extend(emit_part_invariants_facets(part, ctx, (&base_id, &part_dir), units));
    for (i, chunk) in part.ports.chunks(4).enumerate() {
        let m = shard_facet_name("ports", part.ports.len(), 4, i);
        units.push(CompilationUnit {
            port_defs: chunk.to_vec(),
            ..empty_unit(
                format!("{base_id}::{m}"),
                join_rel(&part_dir, &format!("{m}.rs")),
                pkg_vec.clone(),
                layer,
                UnitKind::PartPortsFacet,
            )
        });
        children.push(m);
    }
    for (i, chunk) in part.calcs.chunks(5).enumerate() {
        let m = shard_facet_name("calcs", part.calcs.len(), 5, i);
        units.push(CompilationUnit {
            calc_defs: chunk.to_vec(),
            ..empty_unit(
                format!("{base_id}::{m}"),
                join_rel(&part_dir, &format!("{m}.rs")),
                pkg_vec.clone(),
                layer,
                UnitKind::PartCalcsFacet,
            )
        });
        children.push(m);
    }
    units.push(CompilationUnit {
        child_modules: children,
        reexport_children: true,
        ..empty_unit(base_id, join_rel(&part_dir, "mod.rs"), pkg_vec, layer, UnitKind::BarrelFacade)
    });
    part_slug
}

/// Emits a single `StandaloneAction` module or a sharded `stages_XX` directory for an `ActionDef`.
///
/// Realises: [REQ-0607]
pub(super) fn emit_action_units(
    action: &ActionDef,
    ctx: &PackageFacetCtx<'_>,
    units: &mut Vec<CompilationUnit>,
) -> String {
    let (plan, slug, base_id) = (
        outline_action_def(action, ctx.bounds),
        to_snake_slug(&action.name),
        format!("{}::{}", ctx.pkg_path.join("::"), action.name),
    );
    if plan.stages.len() <= 5 {
        units.push(CompilationUnit {
            action_plans: vec![plan],
            ..empty_unit(
                base_id,
                join_rel(ctx.pkg_dir, &format!("{slug}.rs")),
                ctx.pkg_path.to_vec(),
                ctx.layer,
                UnitKind::StandaloneAction,
            )
        });
        return slug;
    }
    let (act_dir, mut shard_names) = (join_rel(ctx.pkg_dir, &slug), Vec::new());
    for (i, chunk) in plan.stages.chunks(5).enumerate() {
        let shard = format!("stages_{:02}", i.saturating_add(1));
        units.push(CompilationUnit {
            action_plans: vec![OutlinedActionPlan {
                stages: chunk.to_vec(),
                ..plan.clone()
            }],
            ..empty_unit(
                format!("{base_id}::{shard}"),
                join_rel(&act_dir, &format!("{shard}.rs")),
                ctx.pkg_path.to_vec(),
                ctx.layer,
                UnitKind::PartActionsFacet,
            )
        });
        shard_names.push(shard);
    }
    units.push(CompilationUnit {
        child_modules: shard_names,
        reexport_children: true,
        action_plans: vec![plan],
        ..empty_unit(
            base_id,
            join_rel(&act_dir, "mod.rs"),
            ctx.pkg_path.to_vec(),
            ctx.layer,
            UnitKind::StandaloneAction,
        )
    });
    slug
}

/// Emits a single `StandaloneState` module or a sharded `states.rs` + `handlers_XX.rs` directory.
///
/// Realises: [REQ-0607]
pub(super) fn emit_state_units(
    state_def: &StateDef,
    ctx: &PackageFacetCtx<'_>,
    units: &mut Vec<CompilationUnit>,
) -> String {
    let (plan, slug, base_id) = (
        outline_state_def(state_def, ctx.bounds),
        to_snake_slug(&state_def.name),
        format!("{}::{}", ctx.pkg_path.join("::"), state_def.name),
    );
    if plan.handlers.len() <= 4 {
        units.push(CompilationUnit {
            state_plans: vec![plan],
            ..empty_unit(
                base_id,
                join_rel(ctx.pkg_dir, &format!("{slug}.rs")),
                ctx.pkg_path.to_vec(),
                ctx.layer,
                UnitKind::StandaloneState,
            )
        });
        return slug;
    }
    let (st_dir, mut children) = (join_rel(ctx.pkg_dir, &slug), vec!["states".to_string()]);
    units.push(CompilationUnit {
        state_plans: vec![OutlinedStateMachinePlan {
            handlers: Vec::new(),
            ..plan.clone()
        }],
        ..empty_unit(
            format!("{base_id}::states"),
            join_rel(&st_dir, "states.rs"),
            ctx.pkg_path.to_vec(),
            ctx.layer,
            UnitKind::PartStatesFacet,
        )
    });
    for (i, chunk) in plan.handlers.chunks(4).enumerate() {
        let shard = format!("handlers_{:02}", i.saturating_add(1));
        units.push(CompilationUnit {
            state_plans: vec![OutlinedStateMachinePlan {
                handlers: chunk.to_vec(),
                ..plan.clone()
            }],
            ..empty_unit(
                format!("{base_id}::{shard}"),
                join_rel(&st_dir, &format!("{shard}.rs")),
                ctx.pkg_path.to_vec(),
                ctx.layer,
                UnitKind::PartStatesFacet,
            )
        });
        children.push(shard);
    }
    units.push(CompilationUnit {
        child_modules: children,
        reexport_children: true,
        state_plans: vec![plan],
        ..empty_unit(
            base_id,
            join_rel(&st_dir, "mod.rs"),
            ctx.pkg_path.to_vec(),
            ctx.layer,
            UnitKind::StandaloneState,
        )
    });
    slug
}
