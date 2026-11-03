#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # SysML v2 to Markdown Forward Synchronization Engine
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYSML-FORWARD-SYNC-ENGINE]
//! Synthesizes deterministic Agile Epics, Features, User Stories, Use Cases, ICD, WBS, and STPA
//! specifications from SysML v2 `PackageDef` AST models while preserving authored prose and
//! tracker metadata under ISO/IEC/IEEE 15288:2023 and ISO/IEC/IEEE 29148:2018.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Input `PackageDef` and `ForwardSyncOptions` are valid.
//! - **Postconditions**: Emits deterministic CommonMark specifications into `BTreeMap<String, String>`.
//! - **Invariants**: Zero wall-clock timestamps are invoked; outputs are bitwise-identical across runs.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Filesystem I/O errors during non-dry-run persistence return `Err(String)` without partial writes.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Linear `O(N)` traversal over AST elements with deterministic `BTreeMap` ordering.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Consumes `deap_core::sysml_cst`, `crate::stpa`, and `crate::salsa_db::write_atomic`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Verified by unit and round-trip tests in `sync/forward/tests*.rs`.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYSML-FORWARD-SYNC-ENGINE]

pub mod epic_diagrams;
pub mod epic_emitter;
pub mod epic_renderer;
pub mod escaping_and_dates;
pub mod feature_diagrams;
pub mod feature_emitter;
pub mod feature_layers;
pub mod feature_renderer;
pub mod feature_schema;
pub mod icd_wbs_emitter;
pub mod merge_and_cleanup;
pub mod merge_boilerplate;
pub mod model_indexer;
pub mod naming_and_dates;
pub mod story_allocator;
pub mod story_context;
pub mod story_emitter;
pub mod story_layers;
pub mod story_renderer;
pub mod story_resolver;
pub mod use_case_clauses;
pub mod use_case_diagrams;
pub mod use_case_emitter;
pub mod use_case_renderer;
pub mod wbs_emitter;
pub mod wbs_formatters;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_arch_and_root_specs;
#[cfg(test)]
mod tests_conops_non_destructive;
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

use deap_core::sysml_cst::{
    ActionDef, AttributeDef, ConnectionDef, ConstraintDef, InteractionDef, PackageDef, PartDef,
    RequirementDef, StateDef, TransitionDef, UseCaseDef,
};
pub(crate) use epic_diagrams::{
    extract_transition_endpoints, generate_epic_class_diagram, generate_epic_statechart,
};
pub(crate) use epic_emitter::emit_epic_specs;
pub(crate) use epic_renderer::{
    render_epic_architecture, render_epic_context, render_epic_operational_considerations,
    render_epic_security_and_governance,
};
pub(crate) use escaping_and_dates::*;
pub(crate) use feature_diagrams::{append_part_class_block, generate_class_diagram};
pub(crate) use feature_emitter::emit_feature_specs;
pub(crate) use feature_layers::*;
pub use feature_renderer::*;
pub(crate) use feature_renderer::generate_feature_bdd;
pub(crate) use feature_schema::*;
pub use icd_wbs_emitter::*;
pub use merge_and_cleanup::*;
pub(crate) use merge_and_cleanup::{
    clean_colliding_prefix_specifications, clean_stale_specifications, extract_id_prefix,
    merge_existing_spec_content, read_existing_spec_content,
};
use merge_boilerplate::*;
pub(crate) use model_indexer::*;
pub(crate) use naming_and_dates::*;
pub use story_allocator::{
    collect_package_story_numbers, count_total_epics_inclusive, resolve_parent_epic_link,
};
pub(crate) use story_allocator::*;
pub(crate) use story_context::*;
pub(crate) use story_emitter::emit_user_story_specs;
pub(crate) use story_layers::*;
pub(crate) use story_renderer::{generate_user_story_bdd, generate_user_story_statement};
pub use story_resolver::StoryMeta;
pub(crate) use story_resolver::*;
pub(crate) use use_case_clauses::*;
pub(crate) use use_case_diagrams::*;
pub(crate) use use_case_emitter::emit_use_case_specs;
pub(crate) use use_case_renderer::*;
use walkdir::WalkDir;
pub use wbs_emitter::*;
use wbs_formatters::*;

use crate::salsa_db::write_atomic;
use crate::stpa::uca::expand_cartesian_stpa;

/// Configuration paths and execution mode flags controlling SysML v2 forward synchronization.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORWARDSYNCOPTIONS]
#[derive(Debug, Clone)]
pub struct ForwardSyncOptions {
    pub schema_path: PathBuf,
    pub docs_dir: PathBuf,
    pub out_dir: Option<PathBuf>,
    pub dry_run: bool,
    pub force: bool,
}

/// Synthesizes `safety/STPA_MATRIX.md` from `pkg` using Cartesian UCA expansion and FMECA token projection.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
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
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
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

/// Resolves a verified tracker issue ID for `rel_path`, returning `None` (`TBD`) when `rel_path` belongs to a rejected top-down sequential fake `#NNN` block.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RESOLVE-ISSUE-ID]
pub(crate) fn resolve_verified_issue_id(
    index: &ForwardSyncIndex<'_>,
    rel_path: &str,
) -> Option<usize> {
    if index.rejected_fake_issue_paths.contains(rel_path) {
        None
    } else {
        resolve_existing_issue_id(&index.target_out_dir, rel_path)
    }
}

/// Collects specification relative paths belonging to top-down sequential fake `#NNN` blocks where a parent Epic's bare `#NNN` is smaller than its child Feature, Use Case, and User Story bare `#NNN` values.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
fn populate_rejected_fake_issue_paths(pkg: &PackageDef, index: &mut ForwardSyncIndex<'_>) {
    for (idx, p) in index.ordered_packages.iter().enumerate() {
        let epic_num = idx.saturating_add(1);
        let is_root = p.name == "SysML_Model" || p.name == pkg.name;
        let is_conops = p.name == "ConOps";
        let epic_rel_path = if is_conops && !is_root {
            "epics/EPIC-02-ConOps.md".to_string()
        } else {
            format!("epics/EPIC-{:02}-{}.md", epic_num, p.name)
        };

        let mut child_paths: Vec<String> = Vec::new();
        for (f_idx, part_ctx) in index.part_contexts.iter().enumerate() {
            let is_match = part_ctx.pkg_name == p.name
                || (is_conops && part_ctx.pkg_name == "ConOps")
                || (is_root
                    && (part_ctx.pkg_name == pkg.name || part_ctx.pkg_name == "SysML_Model"));
            if is_match {
                child_paths.push(format!(
                    "features/FEAT-{:02}-{}.md",
                    f_idx.saturating_add(1),
                    part_ctx.part.name
                ));
            }
        }
        for uc_m in index
            .uc_metas
            .iter()
            .filter(|m| m.pkg_name == p.name || (is_conops && !index.has_conops_uc))
        {
            let clean_name = strip_use_case_prefix(&uc_m.uc.name);
            child_paths.push(format!(
                "use-cases/UC-{:02}-{}.md",
                uc_m.u_num,
                sanitize_id(clean_name)
            ));
        }
        for s_m in index
            .story_metas
            .iter()
            .filter(|m| m.pkg_name == p.name || (is_conops && !index.has_conops_us))
        {
            child_paths.push(format!(
                "user-stories/US-{:02}-{}.md",
                s_m.s_num,
                sanitize_id(&s_m.action_name)
            ));
        }

        let epic_id =
            resolve_existing_issue_id_with_provenance(&index.target_out_dir, &epic_rel_path);
        let child_ids: Vec<Option<ParsedIssueId>> = child_paths
            .iter()
            .map(|cp| resolve_existing_issue_id_with_provenance(&index.target_out_dir, cp))
            .collect();
        if filter_top_down_fake_issue_ids(epic_id, &child_ids) {
            index.rejected_fake_issue_paths.insert(epic_rel_path);
            for (cp, cid) in child_paths.into_iter().zip(child_ids) {
                if cid.is_some_and(|c| !c.has_tracker_url) {
                    index.rejected_fake_issue_paths.insert(cp);
                }
            }
        }
    }
}

/// Executes forward synchronization from the SysML v2 `PackageDef` model to Markdown specifications.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
pub fn forward_sync_sysml_to_specs(
    pkg: &PackageDef,
    opts: &ForwardSyncOptions,
) -> Result<BTreeMap<String, String>, String> {
    let mut index = build_forward_sync_index(pkg, opts);
    populate_rejected_fake_issue_paths(pkg, &mut index);
    let mut generated_files = BTreeMap::new();

    emit_epic_specs(pkg, &index, &mut generated_files);
    emit_feature_specs(pkg, &index, &mut generated_files);
    emit_user_story_specs(pkg, &index, &mut generated_files);
    emit_use_case_specs(&index, &mut generated_files);
    emit_stpa_matrix_spec(pkg, &mut generated_files);

    merge_and_persist_specs(&index.target_out_dir, opts, &mut generated_files)?;

    Ok(generated_files)
}
