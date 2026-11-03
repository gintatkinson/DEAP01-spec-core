#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # Forward Sync Model Indexer and Package Traversal Ordering
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-MODEL-INDEX]
//! Builds deterministic package traversal orders, `PartContext` structural indices, `StoryMeta`
//! behavioral allocations, and `UseCaseMeta` traceability tables from a root `PackageDef`.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Input `PackageDef` and `ForwardSyncOptions` are valid.
//! - **Postconditions**: Produces a complete `ForwardSyncIndex` lookup table with 1-based numbering.
//! - **Invariants**: All 1-based indices use saturating arithmetic (`saturating_add(1)`).
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Missing subject engines or unallocated packages fall back deterministically to index `1`.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Single-threaded `O(P + F + S + U)` indexing over bounded AST slices.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Consumes `story_allocator` and `use_case_renderer`; consumed by all forward-sync emitters.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Verified by forward-sync ordering and traceability tests in `sync/forward/tests*.rs`.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-MODEL-INDEX]

use super::*;

/// Resolved package, engine feature number, and paired User Story index for a projected `UseCaseDef`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-USECASEMETA]
pub(crate) struct UseCaseMeta<'a> {
    pub(crate) uc: &'a UseCaseDef,
    pub(crate) pkg: &'a PackageDef,
    pub(crate) pkg_name: String,
    pub(crate) subsys_dir: String,
    pub(crate) engine_name: String,
    pub(crate) engine_feat_num: usize,
    pub(crate) u_num: usize,
    pub(crate) story_s_num: Option<usize>,
}

/// Pre-computed AST package, part, requirement, user-story, and use-case lookup tables shared across forward-sync emitters.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-MODEL-INDEX]
pub(crate) struct ForwardSyncIndex<'a> {
    pub(crate) target_out_dir: PathBuf,
    pub(crate) today_iso: &'static str,
    pub(crate) has_modular_schema: bool,
    pub(crate) has_conops_uc: bool,
    pub(crate) has_conops_us: bool,
    pub(crate) all_packages: Vec<&'a PackageDef>,
    pub(crate) ordered_packages: Vec<&'a PackageDef>,
    pub(crate) part_packages: Vec<&'a PackageDef>,
    pub(crate) all_requirements: Vec<RequirementDef>,
    pub(crate) part_contexts: Vec<PartContext<'a>>,
    pub(crate) story_metas: Vec<StoryMeta>,
    pub(crate) story_ast_contexts: Vec<StoryAstContext<'a>>,
    pub(crate) story_display_titles: BTreeMap<usize, String>,
    pub(crate) uc_metas: Vec<UseCaseMeta<'a>>,
}

/// Selects the canonical root `PackageDef` from `all_packages` for `pkg`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-MODEL-INDEX]
fn select_root_package<'a>(
    pkg: &'a PackageDef,
    all_packages: &[&'a PackageDef],
) -> Option<&'a PackageDef> {
    let is_empty_synthetic_root = pkg.name == "SysML_Model"
        && !pkg.packages.is_empty()
        && pkg.part_defs.is_empty()
        && pkg.action_defs.is_empty()
        && pkg.use_case_defs.is_empty()
        && pkg.requirement_defs.is_empty();

    if is_empty_synthetic_root {
        all_packages
            .iter()
            .copied()
            .find(|p| p.name != "SysML_Model" && p.name != "ConOps" && !p.imports.is_empty())
            .or_else(|| all_packages.iter().copied().find(|p| p.name != "SysML_Model" && p.name != "ConOps"))
            .or_else(|| all_packages.iter().copied().find(|p| p.name != "SysML_Model"))
    } else {
        all_packages
            .iter()
            .copied()
            .find(|p| p.name == pkg.name)
            .or_else(|| all_packages.iter().copied().find(|p| p.name == "SysML_Model"))
            .or_else(|| all_packages.first().copied())
    }
}

/// Resolves the canonical Epic package ordering: root package first, `ConOps` second, imported packages in AST order, then remaining packages.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-MODEL-INDEX]
fn collect_ordered_packages<'a>(
    pkg: &'a PackageDef,
    all_packages: &[&'a PackageDef],
) -> (
    Option<&'a PackageDef>,
    Option<&'a PackageDef>,
    Vec<&'a PackageDef>,
) {
    let root_pkg = select_root_package(pkg, all_packages);
    let conops_pkg = all_packages.iter().copied().find(|p| p.name == "ConOps");
    let mut ordered_packages: Vec<&'a PackageDef> = Vec::new();

    if let Some(rp) = root_pkg {
        ordered_packages.push(rp);
    }
    if let Some(cp) = conops_pkg {
        if !ordered_packages.contains(&cp) {
            ordered_packages.push(cp);
        }
    }
    if let Some(rp) = root_pkg {
        for imp in &rp.imports {
            let imp_pkg = imp
                .path
                .trim_end_matches("::**")
                .trim_end_matches("::*")
                .split("::")
                .next()
                .unwrap_or("");
            if !imp_pkg.is_empty() {
                if let Some(sp) = all_packages.iter().copied().find(|p| p.name == imp_pkg) {
                    if !ordered_packages.contains(&sp) {
                        ordered_packages.push(sp);
                    }
                }
            }
        }
    }
    for p in all_packages {
        if p.name != "SysML_Model" && !ordered_packages.contains(p) {
            ordered_packages.push(*p);
        }
    }

    (root_pkg, conops_pkg, ordered_packages)
}

/// Orders packages for structural `PartDef`, User Story, and Use Case projection (`ConOps` first, root package second, then subsystem packages).
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-MODEL-INDEX]
fn collect_part_packages<'a>(
    root_pkg: Option<&'a PackageDef>,
    conops_pkg: Option<&'a PackageDef>,
    ordered_packages: &[&'a PackageDef],
) -> Vec<&'a PackageDef> {
    let mut part_packages: Vec<&'a PackageDef> = Vec::new();
    if let Some(cp) = conops_pkg {
        part_packages.push(cp);
    }
    if let Some(rp) = root_pkg {
        if !part_packages.contains(&rp) {
            part_packages.push(rp);
        }
    }
    for p in ordered_packages {
        if p.name != "SysML_Model" && !part_packages.contains(p) {
            part_packages.push(*p);
        }
    }
    part_packages
}

/// Resolves the canonical `(subsys_display, subsys_dir)` tuple for package `pkg_name` relative to `root_pkg_name`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARTCONTEXT]
pub(crate) fn resolve_package_subsystem_labels(
    root_pkg_name: &str,
    pkg_name: &str,
) -> (String, String) {
    let subsys_dir = if pkg_name == root_pkg_name || pkg_name == "SysML_Model" {
        "architecture".to_string()
    } else if pkg_name == "ConOps" {
        "conops".to_string()
    } else {
        to_snake_case(pkg_name)
    };
    let subsys_display = if pkg_name == root_pkg_name || pkg_name == "SysML_Model" {
        "System Architecture".to_string()
    } else {
        to_display_name(pkg_name)
    };
    (subsys_display, subsys_dir)
}

/// Collects ordered `PartContext` entries across `part_packages` to assign 1-based Feature numbers (`FEAT-XX-*`).
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PARTCONTEXT]
fn collect_part_contexts<'a>(
    root_pkg_name: &str,
    part_packages: &[&'a PackageDef],
) -> Vec<PartContext<'a>> {
    let mut part_contexts: Vec<PartContext<'a>> = Vec::new();
    for p in part_packages {
        let (subsys_display, subsys_dir) = resolve_package_subsystem_labels(root_pkg_name, &p.name);
        for part in &p.part_defs {
            part_contexts.push(PartContext {
                part,
                pkg_name: p.name.clone(),
                subsys_display: subsys_display.clone(),
                subsys_dir: subsys_dir.clone(),
            });
        }
    }
    part_contexts
}

/// Resolves the 1-based `FEAT-XX` index of `part_name` within `pkg_name` (falling back to global `part_contexts` match).
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-MODEL-INDEX]
pub(crate) fn lookup_part_feature_number(
    part_contexts: &[PartContext<'_>],
    pkg_name: &str,
    part_name: &str,
) -> Option<usize> {
    part_contexts
        .iter()
        .position(|ctx| ctx.pkg_name == pkg_name && ctx.part.name == part_name)
        .or_else(|| {
            part_contexts
                .iter()
                .position(|ctx| ctx.part.name == part_name)
        })
        .map(|pos| pos.saturating_add(1))
}

/// Allocates 1-based `UseCaseMeta` descriptors across `part_packages`, pairing each `UseCaseDef` with its subject engine Feature and package User Story.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-USECASEMETA]
fn collect_use_case_allocations<'a>(
    root_pkg_name: &str,
    part_packages: &[&'a PackageDef],
    part_contexts: &[PartContext<'a>],
    story_metas: &[StoryMeta],
) -> Vec<UseCaseMeta<'a>> {
    let mut uc_metas: Vec<UseCaseMeta<'a>> = Vec::new();
    for p in part_packages {
        let (_, subsys_dir) = resolve_package_subsystem_labels(root_pkg_name, &p.name);
        let default_engine_name = select_default_package_engine(p);
        let default_engine_feat_num =
            lookup_part_feature_number(part_contexts, &p.name, &default_engine_name).unwrap_or(1);
        let pkg_stories: Vec<usize> = collect_package_story_numbers(story_metas, &p.name);

        for (u_idx, uc) in p.use_case_defs.iter().enumerate() {
            let story_s_num = pkg_stories
                .get(u_idx)
                .copied()
                .or_else(|| pkg_stories.first().copied());
            let parsed_doc = parse_use_case_doc(uc.doc.as_deref());
            let subject_candidate = uc.subject.as_ref().or(parsed_doc.subject.as_ref());
            let engine_name = if let Some(subj) = subject_candidate {
                let declared_in_pkg = p.part_defs.iter().any(|part| &part.name == subj)
                    || part_contexts
                        .iter()
                        .any(|ctx| &ctx.part.name == subj && ctx.pkg_name == p.name);
                if declared_in_pkg {
                    subj.clone()
                } else {
                    default_engine_name.clone()
                }
            } else {
                default_engine_name.clone()
            };

            let engine_feat_num = lookup_part_feature_number(part_contexts, &p.name, &engine_name)
                .unwrap_or(default_engine_feat_num);
            let u_num = uc_metas.len().saturating_add(1);
            uc_metas.push(UseCaseMeta {
                uc,
                pkg: p,
                pkg_name: p.name.clone(),
                subsys_dir: subsys_dir.clone(),
                engine_name,
                engine_feat_num,
                u_num,
                story_s_num,
            });
        }
    }
    uc_metas
}

/// Selects the primary `*Engine` `PartDef` in `p` (falling back to the first `PartDef` or sanitized package name).
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-MODEL-INDEX]
pub(crate) fn select_default_package_engine(p: &PackageDef) -> String {
    p.part_defs
        .iter()
        .find(|part| part.name.ends_with("Engine"))
        .or_else(|| p.part_defs.first())
        .map(|part| part.name.clone())
        .unwrap_or_else(|| sanitize_id(&p.name))
}

/// Constructs the complete `ForwardSyncIndex` lookup table for `pkg` and `opts`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-MODEL-INDEX]
pub(crate) fn build_forward_sync_index<'a>(
    pkg: &'a PackageDef,
    opts: &ForwardSyncOptions,
) -> ForwardSyncIndex<'a> {
    let target_out_dir = opts
        .out_dir
        .as_ref()
        .cloned()
        .unwrap_or_else(|| opts.docs_dir.clone());
    let today_iso = "1970-01-01";

    let all_packages = pkg.get_all_packages();
    let mut all_requirements = Vec::new();
    for p in &all_packages {
        all_requirements.extend(p.requirement_defs.clone());
    }

    let (root_pkg, conops_pkg, ordered_packages) = collect_ordered_packages(pkg, &all_packages);
    let part_packages = collect_part_packages(root_pkg, conops_pkg, &ordered_packages);
    let part_contexts = collect_part_contexts(&pkg.name, &part_packages);
    let (story_metas, story_ast_contexts) =
        collect_story_allocations(&pkg.name, &part_packages, &part_contexts);
    let story_display_titles = build_story_display_titles(&story_metas);
    let uc_metas =
        collect_use_case_allocations(&pkg.name, &part_packages, &part_contexts, &story_metas);

    let has_conops_uc = uc_metas.iter().any(|m| m.pkg_name == "ConOps");
    let has_conops_us = story_metas.iter().any(|m| m.pkg_name == "ConOps");
    let schema_base = target_out_dir
        .parent()
        .unwrap_or(&target_out_dir)
        .join("schema");
    let has_modular_schema =
        schema_base.join("subsystems").exists() && schema_base.join("conops").exists();

    ForwardSyncIndex {
        target_out_dir,
        today_iso,
        has_modular_schema,
        has_conops_uc,
        has_conops_us,
        all_packages,
        ordered_packages,
        part_packages,
        all_requirements,
        part_contexts,
        story_metas,
        story_ast_contexts,
        story_display_titles,
        uc_metas,
    }
}
