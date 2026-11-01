//! Package behavioral action and interaction deduplication, lifeline endpoint binding, and User Story title disambiguation.
//!
//! Safety & Traceability: Realises: [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
//! Contract: Collects deduplicated package `ActionDef`, `PartDef::actions`, and standalone `InteractionDef` items across ordered packages, binds caller/target lifelines to declared `PartDef` features, and disambiguates duplicate humanized User Story titles.
//! Bounds & Faults: Uses `BTreeSet`/`BTreeMap` deterministic collections, checked slice accessors (`.first`, `.get`, `.split_once`), and saturating index arithmetic (`saturating_add`) with zero panics.

use super::*;

/// Scoped package context shared across behavioral items while allocating `StoryMeta` entries.
/// Realises: [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
struct PackageStoryScanCtx<'a, 'b> {
    root_pkg_name: &'b str,
    pkg: &'a PackageDef,
    subsys_display: &'b str,
    subsys_dir: &'b str,
    part_contexts: &'b [PartContext<'a>],
}

/// Effective structural `PartDef` scope and fallback engine token for binding a single User Story.
/// Realises: [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
struct StoryStructuralScope<'a> {
    root_pkg_name: &'a str,
    pkg_name: &'a str,
    part_defs: &'a [PartDef],
    default_engine_name: String,
}

/// Resolved lifeline endpoints, initiator actor, and reply message extracted for a single User Story behavioral item.
/// Realises: [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
struct StoryLifelineBinding {
    initiator_actor: Option<String>,
    caller_engine: String,
    target_engine: String,
    reply_msg: String,
    interaction_name: String,
}

/// Normalizes `s` into a lowercase ASCII alphanumeric token for punctuation-insensitive AST matching.
/// Realises: [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
pub(crate) fn normalize_alnum_lower(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Resolves the child subpackage owning structural `PartDef`s for a parent container behavioral item when `p.part_defs` is empty.
/// Realises: [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
fn resolve_child_part_package<'a>(
    p: &'a PackageDef,
    a_idx: usize,
    action_name: &str,
    inter_opt: Option<&InteractionDef>,
) -> Option<&'a PackageDef> {
    let primary_token = inter_opt.map(|i| i.name.as_str()).unwrap_or(action_name);
    let norm_primary = normalize_alnum_lower(primary_token);
    let norm_action = normalize_alnum_lower(action_name);
    let exact_or_prefix = p
        .packages
        .iter()
        .filter(|sub| !sub.part_defs.is_empty())
        .find(|sub| {
            let norm_pkg = normalize_alnum_lower(&sub.name);
            let matches_inter = sub
                .interaction_defs
                .iter()
                .any(|si| normalize_alnum_lower(&si.name) == norm_primary);
            let matches_act = sub
                .action_defs
                .iter()
                .any(|sa| normalize_alnum_lower(strip_use_case_prefix(&sa.name)) == norm_action);
            let matches_prefix = !norm_pkg.is_empty()
                && (norm_primary.starts_with(&norm_pkg) || norm_pkg.starts_with(&norm_primary));
            matches_inter || matches_act || matches_prefix
        });
    exact_or_prefix
        .or_else(|| {
            p.packages
                .iter()
                .filter(|sub| !sub.part_defs.is_empty())
                .nth(a_idx)
        })
        .or_else(|| p.packages.iter().find(|sub| !sub.part_defs.is_empty()))
}

/// Resolves the `StoryStructuralScope` (`pkg_name`, `part_defs`, `default_engine_name`) for a behavioral item in `ctx.pkg`.
/// Realises: [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
fn resolve_story_structural_scope<'a, 'b>(
    ctx: &'b PackageStoryScanCtx<'a, 'b>,
    a_idx: usize,
    action_name: &str,
    owner_part: Option<String>,
    inter_opt: Option<&InteractionDef>,
) -> StoryStructuralScope<'b>
where
    'a: 'b,
{
    let p = ctx.pkg;
    if !p.part_defs.is_empty() {
        return StoryStructuralScope {
            root_pkg_name: ctx.root_pkg_name,
            pkg_name: &p.name,
            part_defs: &p.part_defs,
            default_engine_name: owner_part.unwrap_or_else(|| select_default_package_engine(p)),
        };
    }
    if let Some(sub_pkg) = resolve_child_part_package(p, a_idx, action_name, inter_opt) {
        return StoryStructuralScope {
            root_pkg_name: ctx.root_pkg_name,
            pkg_name: &sub_pkg.name,
            part_defs: &sub_pkg.part_defs,
            default_engine_name: owner_part
                .unwrap_or_else(|| select_default_package_engine(sub_pkg)),
        };
    }
    if let Some(first_ctx) = ctx.part_contexts.first() {
        return StoryStructuralScope {
            root_pkg_name: ctx.root_pkg_name,
            pkg_name: &first_ctx.pkg_name,
            part_defs: &[],
            default_engine_name: owner_part.unwrap_or_else(|| first_ctx.part.name.clone()),
        };
    }
    StoryStructuralScope {
        root_pkg_name: ctx.root_pkg_name,
        pkg_name: &p.name,
        part_defs: &[],
        default_engine_name: owner_part.unwrap_or_else(|| sanitize_id(&p.name)),
    }
}

/// Collects deduplicated `PkgBehaviorItem` entries from package-level `action_defs`, `part.actions`, and remaining standalone `interaction_defs`.
/// Realises: [REQ-SYNC-FORWARD-PKG-BEHAVIOR-ITEM]
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

/// Resolves the sanitized sequence reply message token from an optional `InteractionDef` or `ActionDef`.
/// Realises: [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
fn resolve_story_reply_message(
    act_opt: Option<&ActionDef>,
    inter_opt: Option<&InteractionDef>,
) -> String {
    let inter_reply = inter_opt
        .and_then(|inter| inter.messages.get(1))
        .and_then(|m1| m1.split_whitespace().next())
        .map(sanitize_id);
    let action_reply = act_opt
        .and_then(|a| a.out_params.first())
        .map(|out_p| {
            if !out_p.type_name.trim().is_empty() {
                out_p.type_name.trim()
            } else {
                out_p.name.trim()
            }
        })
        .filter(|c| !c.is_empty())
        .map(sanitize_id);
    inter_reply
        .or(action_reply)
        .unwrap_or_else(|| "OperationComplete".to_string())
}

/// Extracts initiator actor, caller/target engines, and reply message from an optional `InteractionDef` and `ActionDef`.
/// Realises: [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
fn extract_story_lifelines(
    scope: &StoryStructuralScope<'_>,
    action_name: &str,
    act_opt: Option<&ActionDef>,
    inter_opt: Option<&InteractionDef>,
) -> StoryLifelineBinding {
    let mut initiator_actor: Option<String> = None;
    let mut caller_engine = sanitize_id(scope.root_pkg_name);
    let mut target_engine = scope.default_engine_name.clone();
    let mut interaction_name = action_name.to_string();

    if let Some(inter) = inter_opt {
        interaction_name = inter.name.clone();
        if let Some(l0) = inter.lifelines.first() {
            let raw_init = if let Some((_, type_name)) = l0.split_once(':') {
                type_name.trim()
            } else {
                l0.trim()
            };
            let clean_init = raw_init
                .trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | '[' | ']'))
                .trim();
            if !clean_init.is_empty() && !scope.part_defs.iter().any(|pt| pt.name == clean_init) {
                initiator_actor = Some(clean_init.to_string());
            }
        }
        if let [l0, l1, ..] = inter.lifelines.as_slice() {
            if let Some((_, type_name)) = l0.split_once(':') {
                caller_engine = type_name.trim().to_string();
            }
            if let Some((_, type_name)) = l1.split_once(':') {
                target_engine = type_name.trim().to_string();
            }
        }
    }

    if !scope.part_defs.iter().any(|pt| pt.name == caller_engine) {
        caller_engine = scope
            .part_defs
            .first()
            .map(|pt| pt.name.clone())
            .unwrap_or_else(|| scope.default_engine_name.clone());
    }
    if !scope.part_defs.iter().any(|pt| pt.name == target_engine) {
        target_engine = scope
            .part_defs
            .get(1)
            .or_else(|| scope.part_defs.first())
            .map(|pt| pt.name.clone())
            .unwrap_or_else(|| scope.default_engine_name.clone());
    }
    if caller_engine == target_engine && scope.part_defs.len() >= 2 {
        target_engine = scope
            .part_defs
            .iter()
            .find(|pt| pt.name != caller_engine)
            .map(|pt| pt.name.clone())
            .unwrap_or(target_engine);
    }

    StoryLifelineBinding {
        initiator_actor,
        caller_engine,
        target_engine,
        reply_msg: resolve_story_reply_message(act_opt, inter_opt),
        interaction_name,
    }
}

/// Builds a paired `(StoryMeta, StoryAstContext)` allocation for a single `PkgBehaviorItem`.
/// Realises: [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
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

    let inter_opt = explicit_inter
        .or_else(|| p.interaction_defs.get(a_idx))
        .or_else(|| p.interaction_defs.first());
    let scope = resolve_story_structural_scope(ctx, a_idx, &action_name, owner_part, inter_opt);
    let binding = extract_story_lifelines(&scope, &action_name, act_opt, inter_opt);

    let default_engine_feat_num =
        lookup_part_feature_number(ctx.part_contexts, scope.pkg_name, &scope.default_engine_name)
            .unwrap_or(1);
    let engine_feat_num =
        lookup_part_feature_number(ctx.part_contexts, scope.pkg_name, &binding.target_engine)
            .unwrap_or(default_engine_feat_num);
    let caller_feat_num =
        lookup_part_feature_number(ctx.part_contexts, scope.pkg_name, &binding.caller_engine);

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
/// Realises: [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
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
/// Realises: [REQ-SYNC-FORWARD-STORY-ALLOCATOR]
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
