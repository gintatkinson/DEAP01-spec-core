#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # User Story Allocation, Lifeline Binding, and Epic Linkage Helpers
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
//! Collects deduplicated package `ActionDef`, `PartDef::actions`, and standalone `InteractionDef`
//! items across ordered packages, binds caller/target lifelines to declared `PartDef` features,
//! disambiguates duplicate humanized User Story titles, and resolves parent Epic links.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: Input `PackageDef` and `PartContext` slices are valid.
//! - **Postconditions**: Produces 1:1 paired `(StoryMeta, StoryAstContext)` allocations.
//! - **Invariants**: `BTreeSet` deduplication and saturating index arithmetic prevent duplicate stories or overflow.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Unresolved caller/target lifelines fall back to the first/second `PartDef` or default package engine.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Pure single-threaded `O(P * B)` traversal over package behaviors.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Consumed by `model_indexer`, `epic_renderer`, `feature_emitter`, `story_emitter`, and `use_case_emitter`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Verified by User Story allocation and Epic count tests in `sync/forward/tests*.rs`.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-ALLOCATOR]

use super::*;

/// Discriminates the originating SysML v2 behavioral AST node backing a projected User Story.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-BEHAVIOR-SOURCE]
pub(crate) enum BehaviorSource<'a> {
    PackageAction(&'a ActionDef),
    PartAction {
        part_name: &'a str,
        action: &'a ActionDef,
    },
    StandaloneInteraction(&'a InteractionDef),
}

/// Deduplicated behavioral item pairing a stripped action/interaction identifier with its `BehaviorSource`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PKG-BEHAVIOR-ITEM]
pub(crate) struct PkgBehaviorItem<'a> {
    pub(crate) clean_name: String,
    pub(crate) source: BehaviorSource<'a>,
}

/// Borrowed package, `ActionDef`, and `InteractionDef` AST handles paired 1:1 with `StoryMeta`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-AST-CTX]
pub(crate) struct StoryAstContext<'a> {
    pub(crate) pkg: &'a PackageDef,
    pub(crate) act_opt: Option<&'a ActionDef>,
    pub(crate) inter_opt: Option<&'a InteractionDef>,
}

/// Scoped package context shared across behavioral items while allocating `StoryMeta` entries.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
struct PackageStoryScanCtx<'a, 'b> {
    root_pkg_name: &'b str,
    pkg: &'a PackageDef,
    subsys_display: &'b str,
    subsys_dir: &'b str,
    part_contexts: &'b [PartContext<'a>],
}

/// Resolved lifeline endpoints, initiator actor, and reply message extracted for a single User Story behavioral item.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
struct StoryLifelineBinding {
    initiator_actor: Option<String>,
    caller_engine: String,
    target_engine: String,
    reply_msg: String,
    interaction_name: String,
}

/// Collects deduplicated `PkgBehaviorItem` entries from package-level `action_defs`, `part.actions`, and remaining standalone `interaction_defs`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-PKG-BEHAVIOR-ITEM]
fn collect_pkg_behavior_items<'a>(p: &'a PackageDef) -> Vec<PkgBehaviorItem<'a>> {
    let mut pkg_behavior_items: Vec<PkgBehaviorItem<'a>> = Vec::new();
    let mut seen_behavior_names: BTreeSet<String> = BTreeSet::new();

    for action_def in &p.action_defs {
        let clean = strip_use_case_prefix(&action_def.name).to_string();
        if !clean.is_empty() && seen_behavior_names.insert(clean.to_ascii_lowercase()) {
            pkg_behavior_items.push(PkgBehaviorItem {
                clean_name: clean,
                source: BehaviorSource::PackageAction(action_def),
            });
        }
    }
    for part in &p.part_defs {
        for action_def in &part.actions {
            let clean = strip_use_case_prefix(&action_def.name).to_string();
            if !clean.is_empty() && seen_behavior_names.insert(clean.to_ascii_lowercase()) {
                pkg_behavior_items.push(PkgBehaviorItem {
                    clean_name: clean,
                    source: BehaviorSource::PartAction {
                        part_name: &part.name,
                        action: action_def,
                    },
                });
            }
        }
    }
    let non_inter_len = pkg_behavior_items.len();
    for inter in p.interaction_defs.iter().skip(non_inter_len) {
        let clean = strip_use_case_prefix(&inter.name).to_string();
        if !clean.is_empty() && seen_behavior_names.insert(clean.to_ascii_lowercase()) {
            pkg_behavior_items.push(PkgBehaviorItem {
                clean_name: clean,
                source: BehaviorSource::StandaloneInteraction(inter),
            });
        }
    }
    pkg_behavior_items
}

/// Applies explicit `InteractionDef` lifelines and reply message overrides to `binding`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
fn apply_interaction_lifeline_overrides(
    p: &PackageDef,
    inter: &InteractionDef,
    binding: &mut StoryLifelineBinding,
) -> bool {
    let mut has_inter_reply = false;
    binding.interaction_name.clone_from(&inter.name);
    if let Some(l0) = inter.lifelines.first() {
        let raw_init = if let Some((_, type_name)) = l0.split_once(':') {
            type_name.trim()
        } else {
            l0.trim()
        };
        let clean_init = raw_init
            .trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | '[' | ']'))
            .trim();
        if !clean_init.is_empty() && !p.part_defs.iter().any(|pt| pt.name == clean_init) {
            binding.initiator_actor = Some(clean_init.to_string());
        }
    }
    if let [l0, l1, ..] = inter.lifelines.as_slice() {
        if let Some((_, type_name)) = l0.split_once(':') {
            binding.caller_engine = type_name.trim().to_string();
        }
        if let Some((_, type_name)) = l1.split_once(':') {
            binding.target_engine = type_name.trim().to_string();
        }
    }
    if let Some(m1) = inter.messages.get(1) {
        if let Some(first_word) = m1.split_whitespace().next() {
            binding.reply_msg = sanitize_id(first_word);
            has_inter_reply = true;
        }
    }
    has_inter_reply
}

/// Extracts initiator actor, caller/target engines, and reply message from an optional `InteractionDef` and `ActionDef`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
fn extract_story_lifelines(
    ctx: &PackageStoryScanCtx<'_, '_>,
    action_name: &str,
    default_engine_name: &str,
    act_opt: Option<&ActionDef>,
    inter_opt: Option<&InteractionDef>,
) -> StoryLifelineBinding {
    let p = ctx.pkg;
    let mut binding = StoryLifelineBinding {
        initiator_actor: None,
        caller_engine: sanitize_id(ctx.root_pkg_name),
        target_engine: default_engine_name.to_string(),
        reply_msg: "OperationComplete".to_string(),
        interaction_name: action_name.to_string(),
    };
    let has_inter_reply = inter_opt
        .map(|inter| apply_interaction_lifeline_overrides(p, inter, &mut binding))
        .unwrap_or(false);

    if !has_inter_reply {
        if let Some(out_p) = act_opt.and_then(|a| a.out_params.first()) {
            let candidate = if !out_p.type_name.trim().is_empty() {
                out_p.type_name.trim()
            } else {
                out_p.name.trim()
            };
            if !candidate.is_empty() {
                binding.reply_msg = sanitize_id(candidate);
            }
        }
    }

    if !p.part_defs.iter().any(|pt| pt.name == binding.caller_engine) {
        binding.caller_engine = p
            .part_defs
            .first()
            .map(|pt| pt.name.clone())
            .unwrap_or_else(|| default_engine_name.to_string());
    }
    if !p.part_defs.iter().any(|pt| pt.name == binding.target_engine) {
        binding.target_engine = p
            .part_defs
            .get(1)
            .or_else(|| p.part_defs.first())
            .map(|pt| pt.name.clone())
            .unwrap_or_else(|| default_engine_name.to_string());
    }
    if binding.caller_engine == binding.target_engine && p.part_defs.len() >= 2 {
        binding.target_engine = p
            .part_defs
            .iter()
            .find(|pt| pt.name != binding.caller_engine)
            .map(|pt| pt.name.clone())
            .unwrap_or(binding.target_engine);
    }

    binding
}

/// Builds a paired `(StoryMeta, StoryAstContext)` allocation for a single `PkgBehaviorItem`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
fn allocate_single_story<'a>(
    ctx: &PackageStoryScanCtx<'a, '_>,
    a_idx: usize,
    s_num: usize,
    item: PkgBehaviorItem<'a>,
) -> (StoryMeta, StoryAstContext<'a>) {
    let p = ctx.pkg;
    let action_name = item.clean_name;
    let (owner_part, act_opt, explicit_inter) = match item.source {
        BehaviorSource::PackageAction(act) => (None, Some(act), None),
        BehaviorSource::PartAction { part_name, action } => {
            (Some(part_name.to_string()), Some(action), None)
        }
        BehaviorSource::StandaloneInteraction(inter) => {
            let matched_act = p
                .action_defs
                .iter()
                .find(|a| strip_use_case_prefix(&a.name).eq_ignore_ascii_case(&action_name))
                .or_else(|| {
                    p.part_defs
                        .iter()
                        .flat_map(|pt| pt.actions.iter())
                        .find(|a| strip_use_case_prefix(&a.name).eq_ignore_ascii_case(&action_name))
                });
            (None, matched_act, Some(inter))
        }
    };

    let default_engine_name = owner_part.unwrap_or_else(|| select_default_package_engine(p));
    let inter_opt = explicit_inter
        .or_else(|| p.interaction_defs.get(a_idx))
        .or_else(|| p.interaction_defs.first());

    let binding =
        extract_story_lifelines(ctx, &action_name, &default_engine_name, act_opt, inter_opt);

    let default_engine_feat_num =
        lookup_part_feature_number(ctx.part_contexts, &p.name, &default_engine_name).unwrap_or(1);
    let engine_feat_num =
        lookup_part_feature_number(ctx.part_contexts, &p.name, &binding.target_engine)
            .unwrap_or(default_engine_feat_num);
    let caller_feat_num =
        lookup_part_feature_number(ctx.part_contexts, &p.name, &binding.caller_engine);

    let meta = StoryMeta {
        action_name,
        engine_name: binding.target_engine,
        caller_engine: binding.caller_engine,
        initiator_actor: binding.initiator_actor,
        reply_msg: binding.reply_msg,
        interaction_name: binding.interaction_name,
        subsys_display: ctx.subsys_display.to_string(),
        pkg_name: p.name.clone(),
        subsys_dir: ctx.subsys_dir.to_string(),
        engine_feat_num,
        caller_feat_num,
        s_num,
    };
    let ast_ctx = StoryAstContext {
        pkg: p,
        act_opt,
        inter_opt,
    };
    (meta, ast_ctx)
}

/// Allocates 1-based `StoryMeta` and `StoryAstContext` tables across all `part_packages`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
pub(crate) fn collect_story_allocations<'a>(
    root_pkg_name: &str,
    part_packages: &[&'a PackageDef],
    part_contexts: &[PartContext<'a>],
) -> (Vec<StoryMeta>, Vec<StoryAstContext<'a>>) {
    let mut story_metas: Vec<StoryMeta> = Vec::new();
    let mut story_ast_contexts: Vec<StoryAstContext<'a>> = Vec::new();

    for p in part_packages {
        let (subsys_display, subsys_dir) = resolve_package_subsystem_labels(root_pkg_name, &p.name);
        let scan_ctx = PackageStoryScanCtx {
            root_pkg_name,
            pkg: p,
            subsys_display: &subsys_display,
            subsys_dir: &subsys_dir,
            part_contexts,
        };
        let items = collect_pkg_behavior_items(p);
        for (a_idx, item) in items.into_iter().enumerate() {
            let s_num = story_metas.len().saturating_add(1);
            let (meta, ast_ctx) = allocate_single_story(&scan_ctx, a_idx, s_num, item);
            story_metas.push(meta);
            story_ast_contexts.push(ast_ctx);
        }
    }

    (story_metas, story_ast_contexts)
}

/// Disambiguates humanized User Story display titles by prefixing `[CallerEngine]` whenever multiple stories share a humanized action title.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
pub(crate) fn build_story_display_titles(story_metas: &[StoryMeta]) -> BTreeMap<usize, String> {
    let mut action_title_counts: BTreeMap<String, usize> = BTreeMap::new();
    for s in story_metas {
        let key = to_display_name(&s.action_name).to_ascii_lowercase();
        let count = action_title_counts.entry(key).or_insert(0);
        *count = count.saturating_add(1);
    }

    let mut story_display_titles: BTreeMap<usize, String> = BTreeMap::new();
    for s in story_metas {
        let base_display = to_display_name(&s.action_name);
        let key = base_display.to_ascii_lowercase();
        let count = action_title_counts.get(&key).copied().unwrap_or(0);
        let display_title = if count > 1 {
            format!("[{}] {}", s.caller_engine, base_display)
        } else {
            base_display
        };
        story_display_titles.insert(s.s_num, display_title);
    }
    story_display_titles
}

pub use super::escaping_and_dates::{
    collect_package_story_numbers, count_total_epics_inclusive, resolve_parent_epic_link,
};
