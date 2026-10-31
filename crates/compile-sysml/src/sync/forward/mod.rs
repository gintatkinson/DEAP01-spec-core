//! SysML v2 to Markdown Forward Synchronization Engine.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-FORWARD-SYNC-ENGINE]
//! Contract: Synthesizes deterministic Agile Epics, Features, User Stories, Use Cases, ICD, WBS, and STPA specifications from SysML v2 `PackageDef` AST models while preserving authored prose and tracker metadata via checked slice accessors.
//! Bounds & Faults: Enforces bounds-checked UTF-8 string slicing (`.get`, `.split_once`, `.strip_prefix`), saturating index arithmetic, and atomic specification persistence without runtime panics on malformed inputs.
//!
//! Generates canonical Agile markdown specifications dynamically from the SysML v2 SSOT AST:
//! - Epics (docs/epics/EPIC-*.md) from PackageDef & CapabilityDef
//! - Features (docs/features/FEAT-*.md) from structural PartDef
//! - User Stories (docs/user-stories/US-*.md) from InteractionDef & ActionDef
//! - Use Cases (docs/use-cases/UC-*.md) from UseCaseDef
//! - Safety Matrix (docs/safety/STPA_MATRIX.md) from ConstraintDef & STPA Generator

pub mod epic_diagrams;
pub mod epic_emitter;
pub mod epic_renderer;
pub mod feature_diagrams;
pub mod feature_emitter;
pub mod feature_layers;
pub mod feature_renderer;
pub mod icd_wbs_emitter;
pub mod merge_and_cleanup;
pub mod model_indexer;
pub mod naming_and_dates;
pub mod story_allocator;
pub mod story_context;
pub mod story_emitter;
pub mod story_layers;
pub mod story_renderer;
pub mod use_case_clauses;
pub mod use_case_emitter;
pub mod use_case_renderer;
pub mod wbs_emitter;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_diagrams_and_stories;
#[cfg(test)]
mod tests_non_destructive_and_actions;
#[cfg(test)]
mod tests_provenance_and_utf8;
#[cfg(test)]
mod tests_roundtrip_and_icd_wbs;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use deap_core::sysml_ast::{
    ActionDef, AttributeDef, ConnectionDef, ConstraintDef, InteractionDef, PackageDef, PartDef,
    RequirementDef, StateDef, TransitionDef, UseCaseDef,
};
pub(crate) use epic_diagrams::{
    extract_transition_endpoints, generate_epic_class_diagram, generate_epic_statechart,
};
pub(crate) use epic_emitter::emit_epic_specs;
pub use epic_renderer::*;
pub(crate) use epic_renderer::{
    render_epic_architecture, render_epic_context, render_epic_operational_considerations,
    render_epic_security_and_governance,
};
pub(crate) use feature_diagrams::generate_class_diagram;
pub(crate) use feature_emitter::emit_feature_specs;
pub(crate) use feature_layers::*;
pub use feature_renderer::*;
pub(crate) use feature_renderer::{
    collect_feature_constraint_obligations, generate_feature_bdd, generate_json_schema,
};
pub use icd_wbs_emitter::*;
pub use merge_and_cleanup::*;
pub(crate) use merge_and_cleanup::{
    clean_colliding_prefix_specifications, clean_stale_specifications, extract_id_prefix,
    merge_existing_spec_content, read_existing_spec_content,
};
pub(crate) use model_indexer::*;
pub(crate) use naming_and_dates::*;
pub(crate) use story_allocator::*;
pub(crate) use story_context::*;
pub(crate) use story_emitter::emit_user_story_specs;
pub(crate) use story_layers::*;
pub use story_renderer::*;
pub(crate) use story_renderer::{generate_user_story_bdd, generate_user_story_statement};
pub(crate) use use_case_clauses::*;
pub(crate) use use_case_emitter::emit_use_case_specs;
pub(crate) use use_case_renderer::*;
use walkdir::WalkDir;
pub use wbs_emitter::*;

use crate::semantic::digest::write_atomic;
use crate::stpa::uca::expand_cartesian_stpa;

/// Configuration paths and execution mode flags controlling SysML v2 forward synchronization.
///
/// Safety & Traceability: Realises: [REQ-SYNC-FORWARD-FORWARDSYNCOPTIONS]
/// Contract: Specifies the input `schema_path`, target `docs_dir`, optional `out_dir` override, `dry_run` preview flag, and `force` overwrite/stale-pruning flag.
/// Bounds & Faults: Pure value struct holding owned `PathBuf` paths and boolean execution selectors with zero panics or side effects.
#[derive(Debug, Clone)]
pub struct ForwardSyncOptions {
    pub schema_path: PathBuf,
    pub docs_dir: PathBuf,
    pub out_dir: Option<PathBuf>,
    pub dry_run: bool,
    pub force: bool,
}

/// Synthesizes `safety/STPA_MATRIX.md` from `pkg` using Cartesian UCA expansion and FMECA token projection.
/// Realises: [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
fn emit_stpa_matrix_spec(pkg: &PackageDef, generated_files: &mut BTreeMap<String, String>) {
    let ucas = expand_cartesian_stpa(pkg);
    let fmeca_rows = crate::stpa::fmeca::generate_fmeca_matrix(pkg, None);
    let tokens = crate::stpa::safety_suite::collect_parameter_tokens(pkg);
    let all_constraints = crate::stpa::safety_suite::collect_all_constraints(pkg);
    let stpa_content = crate::stpa::safety_suite::render_unified_stpa_matrix(
        pkg,
        &ucas,
        &all_constraints,
        &fmeca_rows,
        &tokens,
    );
    generated_files.insert("safety/STPA_MATRIX.md".to_string(), stpa_content);
}

/// Merges existing authored sections when `!opts.force` and persists or previews `generated_files` according to `opts.dry_run`.
/// Realises: [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
fn merge_and_persist_specs(
    target_out_dir: &Path,
    opts: &ForwardSyncOptions,
    generated_files: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    if !opts.force {
        let keys: Vec<String> = generated_files.keys().cloned().collect();
        for rel_path in keys {
            if let Some(existing_content) = read_existing_spec_content(target_out_dir, &rel_path) {
                if let Some(gen_content) = generated_files.get(&rel_path).cloned() {
                    let merged =
                        merge_existing_spec_content(&rel_path, &gen_content, &existing_content);
                    generated_files.insert(rel_path, merged);
                }
            }
        }
    }

    if !opts.dry_run {
        if opts.force {
            clean_stale_specifications(target_out_dir, generated_files)?;
        } else {
            clean_colliding_prefix_specifications(target_out_dir, generated_files)?;
        }

        for (rel_path, text) in &*generated_files {
            let full_dest = target_out_dir.join(rel_path);
            if let Some(parent) = full_dest.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create directory '{}': {}", parent.display(), e)
                })?;
            }
            write_atomic(&full_dest, text).map_err(|e| e.to_string())?;
        }
        println!(
            "[SysML v2 Forward-Sync] Successfully synchronized {} specifications to '{}'",
            generated_files.len(),
            target_out_dir.display()
        );
    } else {
        println!(
            "[SysML v2 Forward-Sync] [DRY RUN] Would generate {} specifications in '{}'",
            generated_files.len(),
            target_out_dir.display()
        );
    }
    Ok(())
}

/// Executes forward synchronization from the SysML v2 `PackageDef` model to Markdown specifications.
///
/// Safety & Traceability: Realises: [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
/// Contract: Indexes `pkg` packages, parts, actions, interactions, and use cases, emits deterministic `EPIC-*.md`, `FEAT-*.md`, `US-*.md`, `UC-*.md`, and `safety/STPA_MATRIX.md` artifacts, preserves authored prose when `!opts.force`, and atomically writes files when `!opts.dry_run`.
/// Bounds & Faults: Linear `O(N)` traversal over AST elements with deterministic `BTreeMap` ordering, zero wall-clock calls, and atomic filesystem persistence returning `Err(String)` on I/O failure.
pub fn forward_sync_sysml_to_specs(
    pkg: &PackageDef,
    opts: &ForwardSyncOptions,
) -> Result<BTreeMap<String, String>, String> {
    let index = build_forward_sync_index(pkg, opts);
    let mut generated_files = BTreeMap::new();

    emit_epic_specs(pkg, &index, &mut generated_files);
    emit_feature_specs(pkg, &index, &mut generated_files);
    emit_user_story_specs(pkg, &index, &mut generated_files);
    emit_use_case_specs(&index, &mut generated_files);
    emit_stpa_matrix_spec(pkg, &mut generated_files);

    merge_and_persist_specs(&index.target_out_dir, opts, &mut generated_files)?;

    Ok(generated_files)
}
