//! Level-3 behavioral outliner and deterministic identifier normalizers for SysML v2 modular code generation.
//! ## 1. Safety Intent & Regulatory Scope
//! Defines pure behavioral outlining and lexical normalization (`outliner.rs`) enforcing DO-178C and ISO 26262 per-function size and complexity ceilings.
//! ## 2. Invariants & Mathematical Properties
//! Accepts parsed `ActionDef`, `StateDef`, and `TransitionDef` AST nodes paired with quantitative `ModularityBounds` configurations without external I/O.
//! ## 3. Preconditions
//! Produces normalized `snake_case` slugs, `PascalCase` type identifiers, primitive Rust type tokens, `OutlinedActionPlan` stages, and `OutlinedStateMachinePlan` handlers.
//! ## 4. Postconditions
//! All seven functions are total and infallible, substituting canonical fallback identifiers (`"unit"`, `"Unit"`, `"Idle"`, `"tick"`, `"noop"`) on degenerate inputs.
//! ## 5. Algorithmic Complexity Bounds
//! Uses `BTreeMap` and `BTreeSet` exclusively with zero recursion, zero regular expressions, and bounded per-function cyclomatic complexity `v(G) <= 6`.
//! ## 6. Failure Modes & Fault Tolerance
//! Verified via unit and boundary mutation tests covering acronym splitting, digit prefixing, action step chunking, state ordering, and handler batch partitioning.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]
//! Implements Level-3 `ActionDef` stage decomposition and `StateDef` transition handler outlining for the `compile-sysml` modular lowering pipeline.

use std::collections::{BTreeMap, BTreeSet};

use deap_core::sysml_ast::{ActionDef, StateDef, TransitionDef};

use crate::modular_codegen::domain::{
    ModularityBounds, OutlinedActionPlan, OutlinedActionStage, OutlinedStateHandler,
    OutlinedStateMachinePlan,
};

/// Normalizes a SysML identifier into a deterministic ASCII `snake_case` module or function slug.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub fn to_snake_slug(name: &str) -> String` guarantees generated Rust module filenames and function identifiers conform to deterministic ASCII lexical rules.
/// ## 2. Invariants & Mathematical Properties
/// `pub fn to_snake_slug(name: &str) -> String` accepts any borrowed UTF-8 `&str` slice, including empty, acronym-heavy, punctuation-delimited, or digit-leading names.
/// ## 3. Preconditions
/// `pub fn to_snake_slug(name: &str) -> String` returns non-empty lowercase ASCII `snake_case` without leading, trailing, or consecutive `'_'`, prefixing leading digits with `"n_"`.
/// ## 4. Postconditions
/// `pub fn to_snake_slug(name: &str) -> String` is total and infallible, returning `"unit"` whenever the input contains zero ASCII alphanumeric characters.
/// ## 5. Algorithmic Complexity Bounds
/// `pub fn to_snake_slug(name: &str) -> String` executes a single linear `O(|name|)` peekable character scan with zero recursion and cyclomatic complexity `v(G) = 6`.
/// ## 6. Failure Modes & Fault Tolerance
/// `pub fn to_snake_slug(name: &str) -> String` is verified across `"ModularityBoundsSpec"`, `"S01"`, `"ExecuteFourLevelLoweringPipeline"`, acronyms, empty strings, and leading digits.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Provides canonical `snake_case` lexical normalization for partitioned module paths, parameters, action stages, and state handler names.
#[must_use]
pub fn to_snake_slug(name: &str) -> String {
    let mut raw = String::with_capacity(name.len().saturating_add(4));
    let mut chars = name.chars().peekable();
    let mut prev_kind = 0u8;
    while let Some(c) = chars.next() {
        if !c.is_ascii_alphanumeric() {
            raw.push('_');
            prev_kind = 0;
        } else if c.is_ascii_uppercase() {
            let next_lower = chars.peek().is_some_and(char::is_ascii_lowercase);
            if prev_kind.saturating_add(u8::from(next_lower)) >= 2 {
                raw.push('_');
            }
            raw.push(c.to_ascii_lowercase());
            prev_kind = 1;
        } else {
            raw.push(c);
            prev_kind = 2;
        }
    }
    let slug = raw.split('_').filter(|seg| !seg.is_empty()).collect::<Vec<_>>().join("_");
    if slug.is_empty() {
        "unit".to_string()
    } else if slug.starts_with(|c: char| c.is_ascii_digit()) {
        format!("n_{slug}")
    } else {
        slug
    }
}

/// Normalizes a SysML identifier into a deterministic ASCII `PascalCase` Rust type or enum variant identifier.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub fn to_pascal_ident(name: &str) -> String` ensures generated Rust structs, traits, and state enum variants carry valid, collision-stable `PascalCase` identifiers.
/// ## 2. Invariants & Mathematical Properties
/// `pub fn to_pascal_ident(name: &str) -> String` accepts any UTF-8 `&str` containing `SCREAMING_SNAKE_CASE`, `snake_case`, `camelCase`, `PascalCase`, or raw delimiters.
/// ## 3. Preconditions
/// `pub fn to_pascal_ident(name: &str) -> String` capitalizes each alphanumeric segment while preserving mixed interior casing, lowercasing all-caps segments, and prefixing leading digits with `"N"`.
/// ## 4. Postconditions
/// `pub fn to_pascal_ident(name: &str) -> String` never fails or slices byte indices directly, returning `"Unit"` when zero ASCII alphanumeric segments exist.
/// ## 5. Algorithmic Complexity Bounds
/// `pub fn to_pascal_ident(name: &str) -> String` runs in linear `O(|name|)` time and heap space over delimiter-split segments with cyclomatic complexity `v(G) = 5`.
/// ## 6. Failure Modes & Fault Tolerance
/// `pub fn to_pascal_ident(name: &str) -> String` is verified on `"IDLE"`, `"Idle"`, `"ResolvingLayers"`, `"s_01"`, empty strings, and digit-prefixed state labels.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Standardizes `PascalCase` type and state-variant identifier synthesis across Level-3 behavioral lowering and Rust emission.
#[must_use]
pub fn to_pascal_ident(name: &str) -> String {
    let mut out = String::with_capacity(name.len().saturating_add(2));
    for seg in name.split(|c: char| !c.is_ascii_alphanumeric()) {
        let mut chars = seg.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            let rest = chars.as_str();
            if seg.chars().any(|c| c.is_ascii_lowercase()) {
                out.push_str(rest);
            } else {
                out.push_str(&rest.to_ascii_lowercase());
            }
        }
    }
    if out.is_empty() {
        "Unit".to_string()
    } else if out.starts_with(|c: char| c.is_ascii_digit()) {
        format!("N{out}")
    } else {
        out
    }
}

/// Maps a SysML v2 scalar or value type name onto its canonical generated Rust primitive or owned string type token.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub fn map_sysml_type_to_rust(sysml_ty: &str) -> &'static str` enforces deterministic, narrowing-free primitive type lowering for generated parameters and attributes.
/// ## 2. Invariants & Mathematical Properties
/// `pub fn map_sysml_type_to_rust(sysml_ty: &str) -> &'static str` borrows an arbitrary `&str` type annotation that may include surrounding whitespace.
/// ## 3. Preconditions
/// `pub fn map_sysml_type_to_rust(sysml_ty: &str) -> &'static str` maps integer aliases to `"i64"`, real aliases to `"f64"`, boolean aliases to `"bool"`, and all other tokens to `"String"`.
/// ## 4. Postconditions
/// `pub fn map_sysml_type_to_rust(sysml_ty: &str) -> &'static str` is total over all UTF-8 inputs, safely defaulting unrecognized or domain-qualified types to `"String"`.
/// ## 5. Algorithmic Complexity Bounds
/// `pub fn map_sysml_type_to_rust(sysml_ty: &str) -> &'static str` executes in `O(|sysml_ty|)` trim-and-compare time with zero heap allocation and cyclomatic complexity `v(G) = 4`.
/// ## 6. Failure Modes & Fault Tolerance
/// `pub fn map_sysml_type_to_rust(sysml_ty: &str) -> &'static str` is verified across `"Integer"`, `"int"`, `"i64"`, `"usize"`, `"Real"`, `"Float"`, `"f64"`, `"Boolean"`, `"Bool"`, `"bool"`, and fallback types.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Supplies the canonical SysML-to-Rust type projection shared by action parameter lowering and structural facet emitters.
#[must_use]
pub fn map_sysml_type_to_rust(sysml_ty: &str) -> &'static str {
    let trimmed = sysml_ty.trim();
    if ["Integer", "int", "i64", "usize"].contains(&trimmed) {
        "i64"
    } else if ["Real", "Float", "f64"].contains(&trimmed) {
        "f64"
    } else if ["Boolean", "Bool", "bool"].contains(&trimmed) {
        "bool"
    } else {
        "String"
    }
}

/// Decomposes a SysML `ActionDef` into normalized typed parameters and bounded sequential `OutlinedActionStage` chunks.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub fn outline_action_def(action: &ActionDef, bounds: &ModularityBounds) -> OutlinedActionPlan` prevents monolithic action bodies by slicing steps into bounded stage helpers.
/// ## 2. Invariants & Mathematical Properties
/// `pub fn outline_action_def(action: &ActionDef, bounds: &ModularityBounds) -> OutlinedActionPlan` borrows an `ActionDef` AST node and `ModularityBounds` budget clamping `chunk_len` to at least `1`.
/// ## 3. Preconditions
/// `pub fn outline_action_def(action: &ActionDef, bounds: &ModularityBounds) -> OutlinedActionPlan` lowers `in_params` and `out_params` and emits 1-based `run_{slug}_stage_{:02}` stages or a single `"noop"` stage.
/// ## 4. Postconditions
/// `pub fn outline_action_def(action: &ActionDef, bounds: &ModularityBounds) -> OutlinedActionPlan` handles zero-step actions and zero-valued step bounds safely via `.max(1)` and `saturating_add(1)`.
/// ## 5. Algorithmic Complexity Bounds
/// `pub fn outline_action_def(action: &ActionDef, bounds: &ModularityBounds) -> OutlinedActionPlan` runs in `O(|in_params| + |out_params| + |steps|)` time with cyclomatic complexity `v(G) = 2`.
/// ## 6. Failure Modes & Fault Tolerance
/// `pub fn outline_action_def(action: &ActionDef, bounds: &ModularityBounds) -> OutlinedActionPlan` is verified on empty actions, exact-boundary actions, and multi-chunk actions exceeding `max_action_steps_per_stage`.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Constructs `OutlinedActionPlan` records attached to `PartActionsFacet` and `StandaloneAction` compilation units.
#[must_use]
pub fn outline_action_def(action: &ActionDef, bounds: &ModularityBounds) -> OutlinedActionPlan {
    let slug = to_snake_slug(&action.name);
    let chunk_len = bounds.max_action_steps_per_stage.max(1);
    let in_params = action
        .in_params
        .iter()
        .map(|p| (to_snake_slug(&p.name), map_sysml_type_to_rust(&p.type_name).to_string()))
        .collect();
    let out_params = action
        .out_params
        .iter()
        .map(|p| (to_snake_slug(&p.name), map_sysml_type_to_rust(&p.type_name).to_string()))
        .collect();
    let stages = if action.steps.is_empty() {
        vec![OutlinedActionStage {
            stage_index: 1,
            fn_name: format!("run_{slug}_stage_01"),
            steps: vec!["noop".to_string()],
        }]
    } else {
        action
            .steps
            .chunks(chunk_len)
            .enumerate()
            .map(|(idx, chunk)| {
                let ordinal = idx.saturating_add(1);
                OutlinedActionStage {
                    stage_index: ordinal,
                    fn_name: format!("run_{slug}_stage_{ordinal:02}"),
                    steps: chunk.to_vec(),
                }
            })
            .collect()
    };
    OutlinedActionPlan { action_name: slug, in_params, out_params, stages }
}

/// Extracts deduplicated `PascalCase` operational state identifiers in declaration order alongside the resolved initial state.
/// ## 1. Safety Intent & Regulatory Scope
/// `fn collect_ordered_states(state_def: &StateDef) -> (Vec<String>, String)` guarantees every generated state machine declares a non-empty, duplicate-free state set and valid entry state.
/// ## 2. Invariants & Mathematical Properties
/// `fn collect_ordered_states(state_def: &StateDef) -> (Vec<String>, String)` borrows a `StateDef` whose `sub_states` and `transitions` may reference pseudo-states `"Initial"` and `"Terminal"`.
/// ## 3. Preconditions
/// `fn collect_ordered_states(state_def: &StateDef) -> (Vec<String>, String)` preserves declaration order across `sub_states` then transition endpoints, excludes `"Initial"` and `"Terminal"`, and resolves `initial_state`.
/// ## 4. Postconditions
/// `fn collect_ordered_states(state_def: &StateDef) -> (Vec<String>, String)` synthesizes `"Idle"` when no operational states exist and falls back cleanly when no `"Initial"` transition is declared.
/// ## 5. Algorithmic Complexity Bounds
/// `fn collect_ordered_states(state_def: &StateDef) -> (Vec<String>, String)` executes in `O((S + T) log S)` time using a deterministic `BTreeSet` guard with cyclomatic complexity `v(G) = 4`.
/// ## 6. Failure Modes & Fault Tolerance
/// `fn collect_ordered_states(state_def: &StateDef) -> (Vec<String>, String)` is verified on explicit `Initial` transitions, transition-discovered states, duplicate labels, and empty `StateDef` nodes.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Supplies deterministic state-enum variant ordering and initial-state resolution to `outline_state_def`.
fn collect_ordered_states(state_def: &StateDef) -> (Vec<String>, String) {
    let mut seen = BTreeSet::from(["Initial".to_string(), "Terminal".to_string()]);
    let mut states = Vec::new();
    let candidates = state_def.sub_states.iter().map(|s| s.name.as_str()).chain(
        state_def
            .transitions
            .iter()
            .flat_map(|tx| [tx.source.as_deref(), tx.target.as_deref()].into_iter().flatten()),
    );
    for raw in candidates {
        let ident = to_pascal_ident(raw);
        if seen.insert(ident.clone()) {
            states.push(ident);
        }
    }
    if states.is_empty() {
        states.push("Idle".to_string());
    }
    let initial_state = state_def
        .transitions
        .iter()
        .filter(|tx| tx.source.as_deref() == Some("Initial"))
        .find_map(|tx| tx.target.as_deref().map(to_pascal_ident))
        .filter(|init| states.contains(init))
        .unwrap_or_else(|| states.first().cloned().unwrap_or_else(|| "Idle".to_string()));
    (states, initial_state)
}

/// Groups operational state transitions into a lexicographically ordered map from `PascalCase` source states to `(trigger, target)` pairs.
/// ## 1. Safety Intent & Regulatory Scope
/// `fn group_transitions_by_source(transitions: &[TransitionDef]) -> BTreeMap<String, Vec<(String, String)>>` partitions outgoing transitions deterministically per source state without hash randomization.
/// ## 2. Invariants & Mathematical Properties
/// `fn group_transitions_by_source(transitions: &[TransitionDef]) -> BTreeMap<String, Vec<(String, String)>>` borrows a slice of `TransitionDef` records with optional `source`, `target`, and `trigger` fields.
/// ## 3. Preconditions
/// `fn group_transitions_by_source(transitions: &[TransitionDef]) -> BTreeMap<String, Vec<(String, String)>>` omits `"Initial"` and `"Terminal"` sources, normalizes endpoints to `PascalCase`, and defaults missing triggers to `"tick"`.
/// ## 4. Postconditions
/// `fn group_transitions_by_source(transitions: &[TransitionDef]) -> BTreeMap<String, Vec<(String, String)>>` ignores incomplete transitions lacking `source` or `target` and treats blank triggers as `"tick"`.
/// ## 5. Algorithmic Complexity Bounds
/// `fn group_transitions_by_source(transitions: &[TransitionDef]) -> BTreeMap<String, Vec<(String, String)>>` runs in `O(T log S)` time over `T` transitions and `S` source states using `BTreeMap` with `v(G) = 4`.
/// ## 6. Failure Modes & Fault Tolerance
/// `fn group_transitions_by_source(transitions: &[TransitionDef]) -> BTreeMap<String, Vec<(String, String)>>` is verified for declaration-order preservation per source state, pseudo-state filtering, and `"tick"` fallback.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Prepares deterministic per-source transition tables for `outline_state_def` handler chunking.
fn group_transitions_by_source(transitions: &[TransitionDef]) -> BTreeMap<String, Vec<(String, String)>> {
    let mut grouped: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for tx in transitions {
        if let (Some(src), Some(dst)) = (tx.source.as_deref(), tx.target.as_deref()) {
            let src_ident = to_pascal_ident(src);
            if !["Initial", "Terminal"].contains(&src_ident.as_str()) {
                let trigger = tx
                    .trigger
                    .as_deref()
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .unwrap_or("tick")
                    .to_string();
                grouped.entry(src_ident).or_default().push((trigger, to_pascal_ident(dst)));
            }
        }
    }
    grouped
}

/// Lowers a SysML `StateDef` into an `OutlinedStateMachinePlan` with complexity-bounded per-source transition handlers.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub fn outline_state_def(state_def: &StateDef, bounds: &ModularityBounds) -> OutlinedStateMachinePlan` enforces `max_cyclomatic_complexity` and `max_transitions_per_handler` ceilings on state dispatchers.
/// ## 2. Invariants & Mathematical Properties
/// `pub fn outline_state_def(state_def: &StateDef, bounds: &ModularityBounds) -> OutlinedStateMachinePlan` borrows a `StateDef` and `ModularityBounds`, clamping `max_tx` to `>= 1` within `max_cyclomatic_complexity - 1`.
/// ## 3. Preconditions
/// `pub fn outline_state_def(state_def: &StateDef, bounds: &ModularityBounds) -> OutlinedStateMachinePlan` emits single `step_from_{src_slug}` handlers or batched `step_from_{src_slug}_batch_{:02}` handlers with `v(G) = 1 + len`.
/// ## 4. Postconditions
/// `pub fn outline_state_def(state_def: &StateDef, bounds: &ModularityBounds) -> OutlinedStateMachinePlan` safely handles zero-transition states and saturates arithmetic so `max_tx` never underflows to zero.
/// ## 5. Algorithmic Complexity Bounds
/// `pub fn outline_state_def(state_def: &StateDef, bounds: &ModularityBounds) -> OutlinedStateMachinePlan` iterates `BTreeMap`-ordered source groups in `O((S + T) log S)` time with cyclomatic complexity `v(G) = 4`.
/// ## 6. Failure Modes & Fault Tolerance
/// `pub fn outline_state_def(state_def: &StateDef, bounds: &ModularityBounds) -> OutlinedStateMachinePlan` is verified on single-handler states, high-fanout states split into `_batch_01`/`_batch_02`, and degenerate automata.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Produces `OutlinedStateMachinePlan` blueprints consumed by `PartStatesFacet` and `StandaloneState` Rust emitters.
#[must_use]
pub fn outline_state_def(state_def: &StateDef, bounds: &ModularityBounds) -> OutlinedStateMachinePlan {
    let (states, initial_state) = collect_ordered_states(state_def);
    let grouped = group_transitions_by_source(&state_def.transitions);
    let complexity_cap = bounds.max_cyclomatic_complexity.saturating_sub(1).max(1);
    let max_tx = bounds.max_transitions_per_handler.max(1).min(complexity_cap);
    let mut handlers = Vec::with_capacity(grouped.len());
    for (src_state, txs) in grouped {
        let src_slug = to_snake_slug(&src_state);
        if txs.len() <= max_tx {
            handlers.push(OutlinedStateHandler {
                source_state: src_state,
                fn_name: format!("step_from_{src_slug}"),
                cyclomatic_complexity: 1usize.saturating_add(txs.len()),
                transitions: txs,
            });
        } else {
            for (idx, chunk) in txs.chunks(max_tx).enumerate() {
                let batch = idx.saturating_add(1);
                handlers.push(OutlinedStateHandler {
                    source_state: src_state.clone(),
                    fn_name: format!("step_from_{src_slug}_batch_{batch:02}"),
                    cyclomatic_complexity: 1usize.saturating_add(chunk.len()),
                    transitions: chunk.to_vec(),
                });
            }
        }
    }
    OutlinedStateMachinePlan { state_def_name: to_pascal_ident(&state_def.name), states, initial_state, handlers }
}
