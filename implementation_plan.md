# Implementation Plan: Upstream Specification & GitHub Backlog Projection Remediation (`#562`..`#690`)

## 1. Objective & Scope

Fix all broken relative/projected links, structural/behavioral AST extraction defects, forward-sync diagram/traceability bugs, and missing end-to-end acceptance gates across `.pipeline/upstream/docs/` (`EPIC-01..06`, `FEAT-01..30`, `US-01..62`, `UC-01..31`) and their projected GitHub issues (`#562` through `#690` in `gintatkinson/DEAP01-spec-core`), while strictly preserving the 1-to-1 GitHub issue mapping (`#562`..`#690`) with zero duplicate issues created.

### Verified Baseline & Issue Mapping Constraints
- **GitHub Issue Mapping (`#562`..`#690` -- MUST BE PRESERVED 1-TO-1, ZERO DUPLICATE ISSUES)**:
  - `US-01`..`US-62` -> `#562`..`#623` (62 issues; preserves the exact 62 `ActionDef` count and package order in `.pipeline/upstream/schema/model.sysml`)
  - `UC-01`..`UC-31` -> `#624`..`#654` (31 issues)
  - `FEAT-01`..`FEAT-30` -> `#655`..`#684` (30 issues)
  - `EPIC-01`..`EPIC-06` -> `#685`..`#690` (6 issues)
- **Strict Compiler Purity & Safety Header Rules (Check 34 & Check 36)**:
  - Every new or modified Rust function, struct, enum, trait, or test MUST carry a unique 7-section safety rustdoc header (`## 1. Safety Intent & Regulatory Scope` through `## 7. Realization & Traceability`).
  - Zero `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, `unimplemented!()`, `unreachable!()`, `as` casts, `Regex::new`, `unsafe`, `process::exit`, `println!`/`eprintln!`/`dbg!`, Unicode box-drawing/em-dashes, or direct string/slice indexing `s[a..b]` / `s[i]` (always use `.get(..)`).
  - In `crates/deap-core/src/sysml_ast.rs`, `test_sysml_ast_compiler_purity_and_7_section_docstrings` (`lines 8522-8650`) also strictly forbids `&[&str]` slice literals/types and `.filter_map(|...| ... .ok())`.

---

## 2. Fresh-Subagent-Per-Step Dispatch Architecture (14 Atomic Steps -> 14 Fresh Subagents `SA-01`..`SA-14`)

> [!IMPORTANT]
> **Strict 1-Step = 1-Fresh-Subagent Guarantee**:
> - Every single step below (`Step 1` through `Step 14`) spawns a **brand-new, fresh `invoke_subagent` instance** (`SA-01` through `SA-14`) with `TypeName: "self"`, `Model: "inherit"`, `Workspace: "inherit"`, and a self-contained prompt ending in `---GOVERNANCE-END---` and `PROCEED`.
> - **No subagent is ever reused across steps**, and **no code-edit step ever touches more than 1 target file**.
> - After each step's subagent returns, the coordinator verifies its completion and confirms subagent termination/reclamation via `manage_subagents(Action: "list")` before dispatching the next step's fresh subagent.

| Step | Fresh Subagent ID | Dedicated Role | Single Target File / Atomic Scope |
| :--- | :--- | :--- | :--- |
| **Step 1** | `SA-01` (Fresh Context) | `Checklist Relative Link & Stale Line Engineer` | [checklist.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/reconcile-backlog/src/checklist.rs) |
| **Step 2** | `SA-02` (Fresh Context) | `Tracker Issue Title Sync Engineer` | [tracker.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/reconcile-backlog/src/tracker.rs) |
| **Step 3** | `SA-03` (Fresh Context) | `Projected Link Bracket & Title Reconciler Engineer` | [lib.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/reconcile-backlog/src/lib.rs) |
| **Step 4** | `SA-04` (Fresh Context) | `SysML Ingestion Behavioral AST Engineer` | [markdown.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/ingest-sysml/src/translators/markdown.rs) |
| **Step 5** | `SA-05` (Fresh Context) | `Core SysML AST State Transition Parser Engineer` | [sysml_ast.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/deap-core/src/sysml_ast.rs) |
| **Step 6** | `SA-06` (Fresh Context) | `Forward-Sync Epic, Feature & Use Case Engineer` | [forward.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/compile-sysml/src/sync/forward.rs) (Epics/Features/Use Cases) |
| **Step 7** | `SA-07` (Fresh Context) | `Forward-Sync User Story Lifeline & Traceability Engineer` | [forward.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/compile-sysml/src/sync/forward.rs) (User Stories `US-01..62`) |
| **Step 8** | `SA-08` (Fresh Context) | `Spec Audit Trigger Rules Resolver Engineer` | [mod.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/verify-baseline/src/spec_audit/mod.rs) |
| **Step 9** | `SA-09` (Fresh Context) | `UML Validator State Transition Indexer Engineer` | [uml_validator.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/verify-baseline/src/spec_audit/uml_validator.rs) |
| **Step 10** | `SA-10` (Fresh Context) | `End-to-End Upstream Projection Acceptance Test Engineer` | [upstream_projection_acceptance_test.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/compile-sysml/tests/upstream_projection_acceptance_test.rs) |
| **Step 11** | `SA-11` (Fresh Context) | `Upstream SysML Schema Re-Ingestion Operator` | `.pipeline/upstream/schema/model.sysml` & `model-digest.json` |
| **Step 12** | `SA-12` (Fresh Context) | `Upstream Specification Forward-Sync & Reconciler Operator` | `.pipeline/upstream/docs/` (`EPIC-01..06`, `FEAT-01..30`, `US-01..62`, `UC-01..31`) |
| **Step 13** | `SA-13` (Fresh Context) | `Strict Spec Audit & Workspace Verification Operator` | `verify-baseline .pipeline/upstream --spec-only --strict`, `verify-baseline . --no-domain`, `cargo test --workspace` |
| **Step 14** | `SA-14` (Fresh Context) | `GitHub Issue #562..#690 Projection & Remote Sync Operator` | Live GitHub projection (`#562`..`#690`), live issue audit, commit & `git push origin main` |

---

## 3. Detailed Step-by-Step Execution Specification

### Step 1 — Fresh Subagent `SA-01`: Fix Local Relative Checklist Links & Stale Line Replacement in `checklist.rs`
- **Subagent Dispatch**: `invoke_subagent(TypeName: "self", Model: "inherit", Role: "Checklist Relative Link & Stale Line Engineer")` — brand-new isolated context, terminated immediately after Step 1 completes.
- **Single Target File**: [checklist.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/reconcile-backlog/src/checklist.rs)
- **Exact Changes**:
  1. **Local Checklist Links (`format_canonical_checklist_item` at lines `936-1008`)**:
     - Remove the branch that emits `https://github.com/{clean_repo}/blob/{branch}/{doc_path}` into local specification files.
     - Always emit workspace-relative markdown links (`let stripped = spec.rel_path.strip_prefix("docs/").unwrap_or(&spec.rel_path); format!("../{}", stripped)`) when writing local markdown files to disk (`../features/{file_name}`, `../user-stories/{file_name}`, `../use-cases/{file_name}`).
  2. **Stale Checklist Line Replacement (`reconcile_or_inject_section` at lines `1092-1174`)**:
     - When reconciling an existing section against `new_items` and `specs`, if an existing checklist line matches `specs.get(idx)` via `contains_spec_reference(trimmed, spec)`, replace the stale line with the updated canonical item from `new_items.get(idx)` (preserving `- [x]` if the existing line started with `- [x]` or `- [X]`), and drop any legacy checklist lines under managed sections (`## Child Features`, `## Linked User Stories`, `## Linked Use Cases`, `## Traceability`) that do not match any active `LoadedSpec`.
  3. **Unit Tests (TDD RED-GREEN)**:
     - Add/update unit tests in `crates/reconcile-backlog/src/checklist.rs` (with 7-section safety docstrings) verifying workspace-relative local links and stale renamed checklist line replacement.

### Step 2 — Fresh Subagent `SA-02`: Add `update_issue_title` to `TrackerClient` in `tracker.rs`
- **Subagent Dispatch**: `invoke_subagent(TypeName: "self", Model: "inherit", Role: "Tracker Issue Title Sync Engineer")` — brand-new isolated context, terminated immediately after Step 2 completes.
- **Single Target File**: [tracker.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/reconcile-backlog/src/tracker.rs)
- **Exact Changes**:
  1. **Add `update_issue_title` (`lines 449-620, 824-861, 1242-1334, 1950-2250`)**:
     - Add `fn update_issue_title(&self, issue_number: u64, title: &str) -> Result<(), String>` (with 7-section safety rustdoc headers) to `TrackerClient`, `OfflineTracker`, `GitHubTracker` (executing `gh issue edit <issue_number> --title <title>`), and `GitLabTracker` (`glab issue update <issue_number> --title <title>`).
  2. **Unit Tests (TDD RED-GREEN)**:
     - Add unit tests in `crates/reconcile-backlog/src/tracker.rs` verifying `OfflineTracker::update_issue_title` updates the stored issue title in place.

### Step 3 — Fresh Subagent `SA-03`: Fix Nested Brackets in `rewrite_projected_links` & Wire Title Sync in `lib.rs`
- **Subagent Dispatch**: `invoke_subagent(TypeName: "self", Model: "inherit", Role: "Projected Link Bracket & Title Reconciler Engineer")` — brand-new isolated context, terminated immediately after Step 3 completes.
- **Single Target File**: [lib.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/reconcile-backlog/src/lib.rs)
- **Exact Changes**:
  1. **Balanced Brackets in Markdown Link Text (`rewrite_projected_links` at lines `1111-1277`)**:
     - Ensure `[text](target)` parsing in `rewrite_projected_links` tracks balanced nested square brackets `[...]` (`depth += 1` on `[`, `depth -= 1` on `]`) so link labels containing inner brackets (such as `[Feature 01: [BP 01 ...] Name](../features/FEAT-01-...)`) are rewritten to full GitHub blob URLs when projected to GitHub issues.
  2. **Wire Title Synchronization in `run_reconciliation` (`lines 420-440`)**:
     - When `options.sync_bodies && !options.offline && !options.dry_run`, check if `!spec.frontmatter.title.trim().is_empty() && issue.title.trim() != spec.frontmatter.title.trim()` and call `tracker.update_issue_title(issue_id, spec.frontmatter.title.trim())?`.
  3. **Unit Tests (TDD RED-GREEN)**:
     - Add unit tests in `crates/reconcile-backlog/src/lib.rs` (with 7-section safety docstrings) verifying nested-bracket link projection and title synchronization during `run_reconciliation`.

### Step 4 — Fresh Subagent `SA-04`: Fix Blueprint Statechart & Action AST Extraction in `markdown.rs`
- **Subagent Dispatch**: `invoke_subagent(TypeName: "self", Model: "inherit", Role: "SysML Ingestion Behavioral AST Engineer")` — brand-new isolated context, terminated immediately after Step 4 completes.
- **Single Target File**: [markdown.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/ingest-sysml/src/translators/markdown.rs)
- **Exact Changes**:
  1. **Populate `sub_states` and `transitions` on Extracted `StateDef`s (`lines 1724-1740, 1836-1875`)**:
     - In `extract_behavioral_ast_nodes`, collect every unique state identifier (`src_id` and `tgt_id`, excluding `"Initial"` and `"Terminal"`) referenced in the `stateDiagram-v2` block and push a child `StateDef { name: state_id, ..Default::default() }` into `state_def.sub_states` in deterministic order.
     - Format transition names deterministically as `{src_id}_to_{tgt_id}_on_{trig_id}` when a trigger label is present (or `{src_id}_to_{tgt_id}` when absent), with `source: Some(src_id)`, `target: Some(tgt_id)`, and `trigger: trig_id`.
  2. **Synthesize a Deterministic Package `StateDef` When a Blueprint Lacks a `stateDiagram-v2` Fence (`lines 1875-1901`)**:
     - At the end of `extract_behavioral_ast_nodes`, if `pkg.state_defs.is_empty()`, synthesize 1 `StateDef` named `{pkg_name}_LifecycleState` whose `sub_states` (3 distinct states derived from the package's first 3 `part_defs` or `{pkg_name}`) and `transitions` (connecting `Initial` -> state 1 -> state 2 -> state 3 -> `Terminal`, triggered by the package's first `action_defs`) are populated on `pkg.state_defs`, so `BP_02` and `BP_05` have complete, subsystem-distinct `state def` blocks in `.pipeline/upstream/schema/model.sysml`.
  3. **Preserve Exact 62 `ActionDef` Count & Package Order (`lines 1800-1835`)**:
     - Strip any leading `US_AC_<digits>_` or `US_<digits>_` prefix from extracted action names while keeping the exact 62 `ActionDef`s across `BP_01..06` in their exact package order (do NOT add extra `ActionDef` items to `pkg.action_defs`).
  4. **Unit Tests (TDD RED-GREEN)**:
     - Add unit tests in `crates/ingest-sysml/src/translators/markdown.rs` verifying `sub_states` extraction and fallback `StateDef` synthesis when a blueprint lacks a `stateDiagram-v2` fence.

### Step 5 — Fresh Subagent `SA-05`: Fix `parse_state_defs` Transition Roundtrip in `sysml_ast.rs`
- **Subagent Dispatch**: `invoke_subagent(TypeName: "self", Model: "inherit", Role: "Core SysML AST State Transition Parser Engineer")` — brand-new isolated context, terminated immediately after Step 5 completes.
- **Single Target File**: [sysml_ast.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/deap-core/src/sysml_ast.rs)
- **Exact Changes**:
  1. **Parse `transition` Statements Inside `parse_state_defs` (`lines 4354-4433`)**:
     - In `parse_state_defs`, alongside `entry`, `do`, and `exit`, parse `transition` statements inside `body`:
       - Support both `transition <t_name>;` (where `<t_name>` encodes `{source}_to_{target}_on_{trigger}` or `{source}_to_{target}`) and `transition <opt_name> first <src> accept <trig> ... then <tgt>;` / `transition first <src> ... then <tgt>;`.
       - When `<t_name>` contains `"_to_"`, split on `"_to_"` and optional `"_on_"` using strictly safe `.find(...)` and `.get(...)` slicing (zero `&[&str]` slice literals/types, zero direct `s[a..b]` indexing, zero `.filter_map(... .ok())`) to recover `source: Option<String>`, `target: Option<String>`, and `trigger: Option<String>`, and push `TransitionDef { name: Some(t_name), source, target, trigger, guard: None, effect: None, doc: None }` into `transitions`.
  2. **Unit Tests (TDD RED-GREEN)**:
     - Add a unit test in `crates/deap-core/src/sysml_ast.rs` (with a complete 7-section safety docstring) verifying that `parse_state_defs` roundtrips `sub_states` and `transitions` (`source`, `target`, `trigger`) from a `state def` block.

### Step 6 — Fresh Subagent `SA-06`: Fix Epic, Feature & Use Case Forward-Sync Generation in `forward.rs`
- **Subagent Dispatch**: `invoke_subagent(TypeName: "self", Model: "inherit", Role: "Forward-Sync Epic, Feature & Use Case Engineer")` — brand-new isolated context, terminated immediately after Step 6 completes.
- **Single Target File**: [forward.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/compile-sysml/src/sync/forward.rs) (Epics, Features, and Use Cases sections: lines `200-220`, `826-1122`, `1700-1960`, `2330-2390`)
- **Exact Changes**:
  1. **Fix Broken `Specification Source` and `Source References` Links (`lines 1789-1810, 1907-1951, 2110-2135, 2358-2383`)**:
     - Check whether `schema/subsystems` exists on disk relative to the target workspace (`target_out_dir.parent().unwrap_or(&target_out_dir).join("schema/subsystems").exists()` or `target_out_dir.join("schema/subsystems").exists()`); when `schema/subsystems` does not exist on disk (as in `.pipeline/upstream` where `schema/model.sysml` is the single-file SSOT), emit `[schema/model.sysml](../../schema/model.sysml)` for `Specification Source` and reference only `schema/model.sysml` in `Source References` across all Epics, Features, User Stories, and Use Cases.
  2. **Fix Epic Statechart Generation (`generate_epic_statechart`, `lines 826-940`)**:
     - Update `generate_epic_statechart` to read `st_def.sub_states` and `st_def.transitions`:
       - Emit `stateDiagram-v2`, declare every `sub_state.name`, and emit every transition from `st_def.transitions` (`[*] --> {target}` when `source == "Initial"`, `{source} --> [*]` when `target == "Terminal"`, and `{source} --> {target} : {trigger}` otherwise) using only AST-declared `StateDef` and transition trigger symbols.
  3. **Fix Epic Class Diagram Generation (`generate_epic_class_diagram`, `lines 970-1122`)**:
     - Never emit `class {p.name}` or `class {parent_label}` when `p.name` / `parent_label` is a package name rather than a declared `PartDef` in `p.part_defs`.
     - Use ONLY `PartDef` names declared in `p.part_defs` as `class` nodes and relationship endpoints in `generate_epic_class_diagram`.
  4. **Fix Use Case Anti-Stutter (`strip_use_case_prefix`, `lines 200-218`)**:
     - Update `strip_use_case_prefix` to iteratively strip composite prefixes (`UC_AC_<digits>_`, `UC_<digits>_`, `AC_<digits>_`, `US_AC_<digits>_`, `US_<digits>_`) in a loop using safe `.get(..)` slicing so `UC_AC_01_Rustc_E0453_...` strips cleanly to `Rustc_E0453_...` (producing `UC-01-Rustc_E0453_...md` and `Use Case 01: Rustc E0453 ...`).
  5. **Remove Nested Square Brackets from Feature Link Labels (`lines 1724, 1890, 2073, 2081, 2088, 2344`)**:
     - Sanitize inner `[` and `]` inside Feature link text labels so Markdown link syntax is flat and unambiguous.

### Step 7 — Fresh Subagent `SA-07`: Fix User Story (`US-01..62`) Forward-Sync Generation in `forward.rs`
- **Subagent Dispatch**: `invoke_subagent(TypeName: "self", Model: "inherit", Role: "Forward-Sync User Story Lifeline & Traceability Engineer")` — brand-new isolated context, terminated immediately after Step 7 completes.
- **Single Target File**: [forward.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/compile-sysml/src/sync/forward.rs) (User Story section: lines `1485-1550`, `2058-2182`)
- **Exact Changes**:
  1. **Fix Broken `[EPIC-02-ConOps](../epics/EPIC-02-ConOps.md)` Link on `BP_02` User Stories (`lines 2094-2108`)**:
     - Replace `if ep_idx == 2` with `if s_meta.pkg_name == "ConOps"` so `BP_02` User Stories (`US-06..15`) link to `[EPIC-02-BP_02_Check_34_AST_Purity_Fail_Closed_IO_and_Anti_Gaming_Test_Gate_Blueprint](../epics/EPIC-02-BP_02_Check_34_AST_Purity_Fail_Closed_IO_and_Anti_Gaming_Test_Gate_Blueprint.md)`.
  2. **Fix `FEAT-01` Cross-Package Pollution & Lifeline Resolution (`lines 1485-1550, 2058-2092`)**:
     - In `StoryMeta` construction (`lines 1485-1550`), resolve `caller_engine` and `target_engine` strictly from `p.part_defs` (if an interaction lifeline type is not a declared `PartDef` in `p.part_defs`, fall back to `p.part_defs.first()` and `p.part_defs.get(1).or_else(|| p.part_defs.first())`) so every `participant` in the User Story `sequenceDiagram` is guaranteed to be a declared `PartDef` in `p.part_defs` (`index.part_names`).
     - In `req_features_list` (`lines 2070-2092`), remove the unconditional `if let Some(first_ctx) = part_contexts.first()` block that injected `FEAT-01` into every User Story across `BP_02..06`. Only link Features belonging to `s_meta.pkg_name`.
  3. **Fix User Story Anti-Stutter & Issue ID Preservation**:
     - Strip any leading `US_AC_<digits>_`, `US_<digits>_`, `UC_AC_<digits>_`, or `AC_<digits>_` prefix from action names in `StoryMeta` / `forward.rs`, and ensure `resolve_existing_issue_id` preserves `#562..#623` and `#624..#654` by `US-{s_num:02}-` / `UC-{u_num:02}-` prefix fallback.

### Step 8 — Fresh Subagent `SA-08`: Fix Ancestor Fallback Lookup for `rules/behavioral_triggers.json` in `spec_audit/mod.rs`
- **Subagent Dispatch**: `invoke_subagent(TypeName: "self", Model: "inherit", Role: "Spec Audit Trigger Rules Resolver Engineer")` — brand-new isolated context, terminated immediately after Step 8 completes.
- **Single Target File**: [mod.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/verify-baseline/src/spec_audit/mod.rs)
- **Exact Changes**:
  1. **Ancestor Directory Fallback for `rules/behavioral_triggers.json` (`lines 872-882`)**:
     - When `options.repo_root.join("rules/behavioral_triggers.json")` does not exist on disk (such as when `options.repo_root` is `.pipeline/upstream`), walk ancestor directories (`options.repo_root.ancestors()`) to locate `rules/behavioral_triggers.json` before returning an error.
  2. **Unit Tests (TDD RED-GREEN)**:
     - Add a unit test in `crates/verify-baseline/src/spec_audit/mod.rs` (with a 7-section safety docstring) verifying nested workspace resolution of `rules/behavioral_triggers.json`.

### Step 9 — Fresh Subagent `SA-09`: Index State Transition Names & Triggers in `uml_validator.rs`
- **Subagent Dispatch**: `invoke_subagent(TypeName: "self", Model: "inherit", Role: "UML Validator State Transition Indexer Engineer")` — brand-new isolated context, terminated immediately after Step 9 completes.
- **Single Target File**: [uml_validator.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/verify-baseline/src/spec_audit/uml_validator.rs)
- **Exact Changes**:
  1. **Index Transition Symbols in `SysmlSymbolIndex::index_state` (`lines 510-528`)**:
     - Ensure `index_state` inserts non-empty `trans.name`, `trans.trigger`, and `trans.effect` into `self.action_names` and recurses into `state.sub_states` so statechart transition triggers parsed from `state def` blocks are recognized by `validate_state_diagram`.
  2. **Unit Tests (TDD RED-GREEN)**:
     - Add a unit test in `crates/verify-baseline/src/spec_audit/uml_validator.rs` (with a 7-section safety docstring) verifying that a `StateDef` with `sub_states` and `TransitionDef` triggers passes `validate_state_diagram`.

### Step 10 — Fresh Subagent `SA-10`: Add End-to-End Upstream Projection Acceptance Test in `upstream_projection_acceptance_test.rs`
- **Subagent Dispatch**: `invoke_subagent(TypeName: "self", Model: "inherit", Role: "End-to-End Upstream Projection Acceptance Test Engineer")` — brand-new isolated context, terminated immediately after Step 10 completes.
- **Single Target File**: [upstream_projection_acceptance_test.rs](file:///Users/perkunas/jail/DEAP01-spec-core/crates/compile-sysml/tests/upstream_projection_acceptance_test.rs)
- **Exact Changes**:
  1. **End-to-End Acceptance Test (`test_upstream_blueprint_to_spec_and_github_projection_zero_defects`)**:
     - Create `crates/compile-sysml/tests/upstream_projection_acceptance_test.rs` (with 7-section safety docstrings and zero forbidden tokens) that runs `MarkdownTranslator` on the 6 design blueprints (`docs/designs/blueprint-01..06*.md`) in an isolated RAII temp directory, runs `forward_sync_sysml_to_specs`, runs `run_reconciliation` (offline), runs `run_spec_audit` (`--spec-only --strict` equivalent), and asserts:
       - Exact counts: 6 Epics, 30 Features, 62 User Stories, 31 Use Cases (129 specs total).
       - Zero broken relative links in any generated `.md` file (every relative markdown link resolves to an existing file on disk, and zero raw `https://github.com` URLs pollute local markdown files).
       - `project_body_for_tracker_with_prefix` rewrites relative links into valid `https://github.com/gintatkinson/DEAP01-spec-core/blob/main/.pipeline/upstream/...` URLs whose target files exist on disk.
       - Zero `spec_audit` findings across all 129 generated specifications.

### Step 11 — Fresh Subagent `SA-11`: Re-Ingest Blueprints into `.pipeline/upstream/schema/model.sysml`
- **Subagent Dispatch**: `invoke_subagent(TypeName: "self", Model: "inherit", Role: "Upstream SysML Schema Re-Ingestion Operator")` — brand-new isolated context, terminated immediately after Step 11 completes.
- **Single Target Scope**: `.pipeline/upstream/schema/model.sysml` and `.pipeline/upstream/schema/model-digest.json`
- **Execution Sequence**:
  1. Clean untracked temporary defect scratch files if present (`.pipeline/defects/defect_20261026_134947.*`, `.pipeline/upstream/.pipeline/defects/`).
  2. Build release binaries: `cargo build --release -p ingest-sysml -p compile-sysml -p reconcile-backlog -p verify-baseline`.
  3. Re-ingest the 6 design blueprints into `.pipeline/upstream/schema/model.sysml`:
     `./target/release/ingest-sysml --schema docs/designs --out .pipeline/upstream/schema/model.sysml --digest .pipeline/upstream/schema/model-digest.json`
  4. Verify that `.pipeline/upstream/schema/model.sysml` contains all 6 packages (`BP_01..06`), 6 `state def` blocks (each with child `state` and `transition` statements), and the exact 62 `action def` blocks in package order.

### Step 12 — Fresh Subagent `SA-12`: Forward-Sync `.pipeline/upstream/docs/` & Reconcile Offline Checklists
- **Subagent Dispatch**: `invoke_subagent(TypeName: "self", Model: "inherit", Role: "Upstream Specification Forward-Sync & Reconciler Operator")` — brand-new isolated context, terminated immediately after Step 12 completes.
- **Single Target Scope**: `.pipeline/upstream/docs/` (`EPIC-01..06`, `FEAT-01..30`, `US-01..62`, `UC-01..31`, `STPA_MATRIX.md`)
- **Execution Sequence**:
  1. Run forward-sync with `--force` so `resolve_existing_issue_id` preserves `#562..#690` across all 129 specs and `clean_stale_specifications` removes the old renamed `UC-*-UC_AC_*` markdown files:
     `./target/release/compile-sysml --forward-sync --schema .pipeline/upstream/schema/model.sysml --docs .pipeline/upstream/docs --out-dir .pipeline/upstream/docs --force`
  2. Reconcile local checklists & traceability matrices offline:
     `./target/release/reconcile-backlog --workspace .pipeline/upstream --offline`
  3. Verify that all 129 generated specs (`EPIC-01..06` = `#685..#690`, `FEAT-01..30` = `#655..#684`, `US-01..62` = `#562..#623`, `UC-01..31` = `#624..#654`) carry their exact 1-to-1 `Issue ID`, zero stale files remain, and zero local `.md` files contain hardcoded `https://github.com` checklist links.

### Step 13 — Fresh Subagent `SA-13`: Execute Strict Spec Audit & Workspace Baseline Verification Gates
- **Subagent Dispatch**: `invoke_subagent(TypeName: "self", Model: "inherit", Role: "Strict Spec Audit & Workspace Verification Operator")` — brand-new isolated context, terminated immediately after Step 13 completes.
- **Single Target Scope**: Verification gates on `.pipeline/upstream` and root workspace
- **Execution Sequence**:
  1. Run the strict specification audit gate on `.pipeline/upstream`:
     `./target/release/verify-baseline .pipeline/upstream --spec-only --strict` (must exit `0` with `0` errors across all 129 specification files).
  2. Run the root workspace baseline gate:
     `./target/release/verify-baseline . --no-domain` (must exit `0` with all 36 checks passing).
  3. Run the full workspace Rust test suite:
     `cargo test --workspace` (must exit `0` with zero failures).

### Step 14 — Fresh Subagent `SA-14`: Project Updated Specs to GitHub Issues `#562`..`#690`, Audit Live Issues & Push to `origin/main`
- **Subagent Dispatch**: `invoke_subagent(TypeName: "self", Model: "inherit", Role: "GitHub Issue #562..#690 Projection & Remote Sync Operator")` — brand-new isolated context, terminated immediately after Step 14 completes.
- **Single Target Scope**: GitHub issue projection (`#562`..`#690`) and `origin/main` synchronization
- **Execution Sequence**:
  1. Project updated titles, bodies, and links to GitHub issues `#562`..`#690`:
     `./target/release/reconcile-backlog --workspace .pipeline/upstream`
  2. Empirically audit live GitHub issues (`gh issue view 685`, `gh issue view 655`, `gh issue view 562`, `gh issue view 624`, `gh issue view 567`) to verify that all titles, links, diagrams, and checklists are clean and zero issues above `#690` were created.
  3. Commit all changes with neutral commit message syntax (avoiding GitHub auto-close keywords for `#562`..`#690`), push to `origin/main`, and verify `git diff origin/main` is empty.
