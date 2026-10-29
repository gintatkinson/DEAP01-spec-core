# Project-Scoped Rules

## Strict Coordinator Tool Locking & Pre-Action Compliance Rules
Thought blocks MUST focus directly on the user's active request and technical reasoning -- NEVER prefix thought blocks with repetitive initialization or compliance checklists. Before invoking any modifying tool or mutating command, enforce these 4 pre-action locks:
* **Intent Classification**: Distinguish questions/inquiries (read-only response only) from direct modification commands.
* **User Authorization Lock**: Modifying tools and mutating `run_command` invocations are strictly locked unless exact target files and changes are documented in an approved `implementation_plan.md` (stored per-conversation at `<appDataDir>/brain/<conversation-id>/implementation_plan.md` or workstream-scoped `.pipeline/plans/<slug>.md` so parallel tabs never clobber `./implementation_plan.md`) AND the user explicitly approves (`PROCEED` / `Approved` / `Approve plan`). Writing this conversation's `implementation_plan.md` is the sole permitted pre-approval write.
* **Zero Silent Assumptions**: Surface ambiguities and verify workspace state empirically rather than making silent assumptions.
* **Coordinator Delegation & Shell Mutation Lock**: Whenever a turn creates, modifies, copies, moves, or deletes any repository file (governance, rules, specs, or code), coordinator direct repository file writes and shell file mutations (`cp`, `mv`, `rm`, `tee`, `sed -i`, `>`, `>>`) are strictly locked; all repository writes MUST be delegated exclusively to single-item subagents after `PROCEED`. Pre-approval creation of `walkthrough.md` is strictly prohibited.

## Repository Role & Scope Classification
- **Role Detection Invariant**:
  * **When `.pipeline/upstream/` is present -> `UPSTREAM_SPEC_CORE_COMPILER`**: Digital Engineering Agent Platform Core Specification Compiler. Landing zones (`schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`) must remain clean landing zones with only `.gitkeep`. Platform profiles (`.pipeline/profiles/*`) are strictly bypassed.
  * **When `.pipeline/upstream/` is absent -> `DOWNSTREAM_CUSTOMER_PROJECT`**: Downstream Application / Domain Engineering Workspace. Concrete schemas in `schema/` and `.pipeline/schema.sysml` serve as the 100% Single Source of Truth (SSOT). Platform profiles (`.pipeline/profiles/<target_platform>.md`) are loaded on-demand during downstream implementation turns only.
- **Sentinel Indicator:** The presence of `.pipeline/upstream/` and `skills/spec-orchestrator/` denotes that this repository is the **Upstream Specification Core Compiler**, NOT a downstream customer application workspace or domain template.
- **Domain Template & Customer Data Boundary:** Domain-specific platforms and customer applications belong in downstream distribution repositories, and must NOT be committed to this upstream specification core compiler repository.

## Pure Schema-Driven Compiler Invariant (Zero Hardcoded Domain Concepts)
- **Abstract MBSE Compiler Declaration**: The DEAP pipeline is an abstract Model-Based Systems Engineering (MBSE) compiler and verification framework, NOT a domain-specific modeler.
- **Strict Prohibition of Hardcoded Domain Concepts**: Agents are strictly forbidden from inventing, proposing, or hardcoding domain-specific concepts into pipeline logic, templates, or execution plans.
- **Deterministic Schema-Derived Specifications**: All specification generation (Epics, Features, User Stories, Use Cases, Safety Invariants) and downstream engineering artifacts derive exclusively and deterministically from AST nodes present in user-provided schemas in `schema/`.

## Upstream Distribution Template Clean Landing Zone Invariant
- **Clean Landing Zone Mandate**: In upstream distribution template repositories (`DEAP-*`), the directories `schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, and `docs/use-cases/` must remain clean landing zones with ONLY `.gitkeep` files.
- **Strict Prohibition of Concrete Specifications**: Committing concrete downstream project specifications, models, or code to upstream distribution templates is strictly forbidden.
- **Downstream Workspace Boundary**: Concrete project schemas and specifications reside exclusively in downstream application workspaces initialized from domain distribution templates.

## Lazy On-Demand Governance Loading (Zero Per-Turn Initialization Ceremony)
- **Zero Per-Turn Startup Ceremony or Receipt Tables**: Conversational, diagnostic, and read-only turns execute **zero** startup ceremony. Agents MUST NEVER halt a user's prompt to run startup rituals, MUST NEVER re-initialize across turns, and MUST NEVER emit a "Mandatory Session Initialization Completed" table or "Ready for instructions" prompt unless explicitly asked.
- **Silent Once-Per-Session Pipeline Context Load**: Before executing a pipeline specification or feature-implementation skill workflow for the first time in a session, silently verify repository role (`test -d .pipeline/upstream`), `git status --short --branch`, `.pipeline/constitution.md`, and `.pipeline/ACTIVE_RULES_BUNDLE.md` **at most once per session** (skipping `.pipeline/constitution.md` and `.pipeline/ACTIVE_RULES_BUNDLE.md` on diagnostic/read-only turns and bounded surgical edits). Subagents receiving `---GOVERNANCE-END---` skip coordinator governance reads entirely.
- **On-Demand Skill, Profile & Verification Loading**:
  1. **Active Task Skill On-Demand**: Read ONLY the `SKILL.md` for the active task (never preload `skills/feature-driven-implementation/SKILL.md` during spec, research, or compiler tasks).
  2. **Conditional Platform Profile Load**: Read `.pipeline/profiles/<target_platform>.md` ONLY during downstream implementation for that platform (never in `UPSTREAM_SPEC_CORE_COMPILER` or spec phases).
  3. **Verify Baseline Only Prior to Commit**: NEVER run `verify-baseline`, `cargo test`, or `flutter test` on startup, read-only turns, or inside single-item edit subagents; run `./target/release/verify-baseline . --no-domain` once at the coordinator level prior to commit gates.

## Strict Planning Gate & Parallel-Safe Atomic Plan Isolation
- Creating, modifying, or deleting repository files or dispatching subagents is strictly forbidden without an approved implementation plan, even if the user prompt already includes `PROCEED` (write the plan first, stop, and await explicit approval; ignore any "When NOT to plan" defaults).
- **Per-Conversation Plan Isolation (Zero Parallel Clobbering)**: Write session `implementation_plan.md` and post-completion `walkthrough.md` to `<appDataDir>/brain/<conversation-id>/` (or workstream-scoped `.pipeline/plans/<slug>.md` when Git persistence is requested). Overwriting a root `./implementation_plan.md` belonging to another active parallel workstream is strictly forbidden.
- **Plan Continuity & Single-Task Anti-Snowballing**: Never abandon or alter an active in-flight plan without user authorization. Each `implementation_plan.md` MUST contain **at most 1 atomic work package** (backlog queues reside exclusively in `gh issue list`; never append new tasks onto completed plans).

## Automated Continuous Execution & Passing-Validation Fast-Path
- **Continuous Execution Gate**: Once an implementation plan receives user approval (`PROCEED`), all documented work packages, subagent dispatches, verification tests, and git pushes are fully authorized to run continuously to completion without intermediate approval pauses.
- **Stop-On-Failure Rule**: The coordinator MUST stop and prompt the user ONLY if:
  1. A build, test, or linter gate fails (`exit code != 0`).
  2. A blocking error or unresolvable requirement ambiguity is encountered.
  3. The initial implementation plan has not been approved.

## Mandatory Native Rust TDD & Prohibition of Legacy Python/Pytest
- **Legacy Python & Pytest Strictly Prohibited (Check 33)**: Creating, maintaining, or executing Python source files (`*.py`), Python test suites (`tests/`, `test_*.py`), `pytest`, or any other Python test runner is strictly forbidden across this repository. Check 33 (Zero Python Scripts Invariant) enforces this on disk; Check 32 (Zero Shell Scripts Invariant) equally forbids `*.sh` files.
- **Native Rust Unit Tests Strictly Mandatory (Executable Code Scope)**: Native Rust `#[test]` unit tests, mutation tests (negative probes proving each gate can fail), and the RED-GREEN-REFACTOR TDD cycle are **strictly mandatory** for all executable Rust compiler and tooling code changes across `crates/`. Every executable code fix or feature under `crates/` MUST ship a failing `#[test]` demonstrated RED before the implementation is written, then GREEN after it.
- **Zero Harness-Prose Theater Tests**: Agent harness and governance Markdown files (`AGENTS.md`, `rules/*.md`, `skills/**`, `.pipeline/ACTIVE_RULES_BUNDLE.md`) are prompt configuration, not product features; fabricating `fs::read_to_string(...).contains(...)` unit tests in `crates/` to assert prompt wording is strictly forbidden.
- **3-Tier Path-Scoped Incremental Build & Test Gate**:
  - **Tier 1 (Harness / Governance / Docs Markdown Only -- `AGENTS.md`, `rules/*.md`, `skills/**`, `.pipeline/*`, `README.md`)**: Run **zero** `cargo build` or `cargo test` commands; run `./target/release/verify-baseline . --no-domain` only when validating governed Markdown changes prior to commit.
  - **Tier 2 (Single-Crate Internal or `#[cfg(test)]` Change -- `crates/<crate>/**` without cross-crate `pub` signature breakage)**: Run targeted `cargo test -p <crate> <test_filter>` during TDD RED/GREEN and `cargo test -p <crate>` before commit (rebuild `cargo build --release -p <bin>` only when a CLI binary's runtime code changed).
  - **Tier 3 (Cross-Crate Public API Signature Change in `deap-core`/`compile-sysml` or `Cargo.toml`/`Cargo.lock` Change)**: Run `cargo test -p <modified-crate>` + `cargo check --workspace --all-targets` during iteration, and `cargo test --workspace` before commit.
- **Semantic Acceptance Testing**: End-to-end semantic acceptance verification against real schemas and domain specifications is performed exclusively with the native Rust binaries:
  - Baseline verification: `./target/release/verify-baseline . --no-domain` (or `cargo run --release -p verify-baseline -- . --no-domain`)
  - Backlog reconciliation: `./target/release/reconcile-backlog`
  - SysML compilation / sync / STPA: `./target/release/compile-sysml`
  - Level 0 ingestion: `./target/release/ingest-sysml`
  - ConOps assembly: `./target/release/assemble-conops`
  - Workspace test suite: `cargo test --workspace`
- **Retired Tooling**: `scripts/verify_downstream_baseline.py`, `sysmlv2_ingest.py`, `scripts/e2e_acceptance_harness.py`, `reconcile_backlog.py`, `parity_auditor`, and `setup_git_hooks.sh` are permanently retired and MUST NOT be referenced as active tooling or invoked.
- **Forbidden Test Workspace Creation**: Creating mock test projects, mock repository directories, or test-runner scripts (such as `test_project/` or `run_tests.py`) directly inside the workspace is strictly forbidden. Rust tests requiring filesystem fixtures MUST create them in temporary directories that are removed on completion.

## Mandatory Workspace-Relative Paths Invariant
- All source code, tests, configuration, documentation, scripts, and entitlements MUST use workspace-relative paths (`.`, `./...`) or environment-derived path resolution. Hardcoded workstation, environment-specific, or user-specific machine paths are strictly prohibited across all codebase tiers.

## Remote Synchronization Mandate
- No task is complete until all changes are successfully pushed to and verified on the remote tracking branch.
- You must verify that `git diff origin/<branch>` is empty before generating the walkthrough and final report.

## Mandatory Subagent Dispatch for Research, Specification & Implementation Loops
- Direct execution of technology stack research (Step 1.5), specification generation, or micro-task implementations (Step 3) within the coordinator's primary conversation context is strictly forbidden to prevent context bloat.
- **Subagent Dispatch Loop Requirements**:
  1. **Fresh Context**: Every subagent MUST begin with a fresh, isolated context without inherited session state.
  2. **Role & Curated Prompt**: Assign a clear role and pass only relevant task scope, files, schema fragments, and guidelines.
  3. **Mandatory Single-Item Scope**: Every subagent dispatch MUST target at most 1 specification item (max 1 Epic, 1 Feature, 1 User Story, or 1 Use Case).
  4. **Mandatory Skill-Reading Instruction (Pipeline Skill Workflows Only -- Surgical Edit Fast-Path)**: When executing a pipeline skill workflow (specification generation, ConOps/ICD/Blueprint/WBS synthesis, TDD feature implementation, `debug-protocol`, or `adversarial-code-auditor`), the prompt MUST instruct the subagent to execute `view_file` on the active `SKILL.md` by explicit path as its first step. For bounded surgical edits where the exact `Target` and `Replacement` blocks are already approved in `implementation_plan.md`, forcing a subagent to read any `SKILL.md` file is strictly prohibited fake work; the subagent must execute the approved `edit_file` calls directly and exit immediately.
  5. **Subagent Self-Rejection Gate**: Subagents enforce pre-flight checks and reject summarized, truncated, or steered prompts.
  6. **Enforce 3-Layer Definition of Done**: The full 3-layer semantic chain ((1) Domain State & Data Model, (2) Logic & State Management, (3) Presentation & Actuator Interface Binding) is mandatory per specification item. Each micro-task must state deliverables or the task number closing each layer; writing `N/A` is forbidden.
  7. **Runtime Dispatch Capability**: Use whichever context-isolated dispatch tool the active runtime provides. If unavailable, halt and escalate as a blocker.
  8. **Authorization**: Append `PROCEED` (case-insensitive) to authorize subagents to use modifying tools.
  9. **Component-Level Failure Mode Research Swarm Mandate**: During Step 1.5, monolithic failure mode research is strictly prohibited. The coordinator MUST decompose all AST `part def` components declared in `.pipeline/schema.sysml` and dispatch context-isolated research subagents per component (role: `Component Failure Mode Researcher`) to populate `docs/research/FAILURE_MODE_REGISTRY.md` across the 4 Universal Failure Dimensions before authorizing downstream STPA/FMECA matrix synthesis.
  10. **Strict Target-File Read & Verification Boundary (Zero Unrelated File Reads)**: Subagents are strictly forbidden from reading, searching, or inspecting any file outside the explicit target file list in their dispatch prompt (e.g., never reading `crates/verify-baseline/*` during a markdown or specification edit). Single-item edit subagents must NEVER redundantly run `./target/release/verify-baseline`, `cargo test`, `git commit`, or `git push`; verification and commit gates run once at the coordinator level per Phase B Step 5.

## Mandatory Application Compilation Build for Verification
- During Step 4 verification, the agent MUST run a full compilation build of the application (e.g. `flutter build` or `npm run build` as specified by the platform profile) to ensure it compiles cleanly. Assertions of completion without verified compile output are strictly forbidden.

## Atomic Work Execution, Parallel Git Safety & Walkthrough Gates
- Execute all tasks as single-item atomic work packages without commingling unrelated changes; update `README.md` whenever rules or directories change.
- **Parallel Workstream Git Staging & Worktree Safety**: Always stage and commit via explicit target paths (`git add -- <target-files>` and `git commit --only -- <target-files>`). Unscoped `git add .`, `git add -A`, `git commit -a`, `git checkout .`, `git restore .`, and `git stash` are strictly forbidden when concurrent workstream files exist. Multi-file RED-GREEN compiler streams SHOULD isolate builds via subagent `Workspace: "share"` or `git worktree` so failing RED states never block parallel tabs.
- **Mandatory Tracker Issue Transition Gate**: Every issue referenced in the commit log must carry `status:fixed-resolved` (or `status::fixed-resolved` on GitLab) and verification evidence comments prior to walkthrough completion.

## Mandatory Upstream Tooling Bug Reporting
- Before filing an upstream defect or tooling bug, an adversarial auditor subagent MUST be dispatched with `skills/adversarial-code-auditor/SKILL.md` to produce a verified 7-section defect dossier with un-falsifiable empirical reproduction in `.pipeline/defects/<NAME>.md`, and execute `./target/release/verify-baseline check-defect .pipeline/defects/<NAME>.md` to atomically validate all 12 checks, persist the local artifact, publish/update the remote GitHub SSOT issue (active by default whenever `--dry-run` is not passed), and verify closed-loop payload parity. Filing raw or unverified defect reports is strictly prohibited. Do not apply silent local-only patches.

## Documentation Integrity -- No Wholesale Replacement Without Approval
- You are strictly forbidden from replacing, truncating, or rewriting any documentation file in a way that removes substantial content unless explicitly approved by the user in the current conversation turn.

## Mandatory Subagent Termination & Cleanup
- The coordinator MUST ensure every spawned subagent is terminated or confirmed reclaimed immediately once its atomic task is complete. Subagents must never be left in an idle or dormant state.

## Mandatory Directory Constraints, Context Isolation & Role Boundaries
- Root source/config writes are strictly forbidden (except `.gitignore` or approved configs); Flutter resides under `app_flutter/`, React under `web_react/`.
- **No Cross-Talk & Literal Skill Execution**: Referencing logs, transcripts, or artifacts from other projects or other `<conversation-id>` directories is strictly forbidden. Follow `SKILL.md` instructions in full without summarization.
- **Phase Boundaries**: Spec subagents must not reference implementation code or profiles; implementation subagents must not modify upstream specifications without running `./target/release/reconcile-backlog` (the retired `reconcile_backlog.py` script is strictly prohibited and MUST NOT be invoked).

## Mermaid & KaTeX Integrity Rules
- Mermaid blocks require valid first-line diagram headers, matching closing fences on new lines, double-quoted labels for special characters (`/`, `:`, `()`, `[]`, `<>`), zero unquoted `<`/`>`, and zero `{}` or secondary colons in class members.
- KaTeX display math `$$` must occupy isolated lines using `\begin{aligned} ... \end{aligned}` (never top-level `\begin{align}`); never wrap non-math IDs (`REQ-*`, `SC-*`, `H-*`) in `$...$`.

## Strict Verification & Parametric Assumption Prevention Rules
- Back all state claims empirically (`git status`, `ls`, `git rev-parse --show-toplevel`); verify live GitHub updates via `gh issue view` / `gh api`; enforce RED-GREEN TDD; treat `<ADDITIONAL_METADATA>` as the authoritative date.
- **macOS `exebox` Sandbox `gh` TLS/Auth**: If `gh` fails with `x509` or `HTTP 401`, import `$SSL_CERT_FILE` via `security add-trusted-cert -d -r trustRoot -k ~/Library/Keychains/login.keychain-db /tmp/antigravity-ca.crt` and persist auth via `gh auth token | gh auth login --with-token --insecure-storage`.

## Downstream Single Source of Truth (SSOT) & Binary-Only Source Firewall
- Downstream projects require a root `.gitignore`, zero `.DS_Store` files, and zero duplicated master blueprints.
- **Downstream Binary-Only Distribution & Black-Box Source Firewall**:
  - Tier-0 compiler source (`crates/{deap-core,ingest-sysml,compile-sysml,verify-baseline,reconcile-backlog,assemble-conops}`, `Cargo.toml`, `Cargo.lock`), `.pipeline/upstream/`, and `docs/architecture/blueprints/` reside **exclusively** in `DEAP01-spec-core`.
  - Copying/committing Tier-0 `crates/*` downstream or running `cargo build` during downstream distribution is strictly forbidden (subagents MUST self-reject). Downstream distribution ships only pre-built `target/release/{compile-sysml,verify-baseline,ingest-sysml,reconcile-backlog,assemble-conops}` binaries, `AGENTS.md`, `.pipeline/{constitution,ACTIVE_RULES_BUNDLE}.md`, `rules/*.md`, and `skills/**`.
  - **Black-Box Binary Firewall**: In `DOWNSTREAM_CUSTOMER_PROJECT` sessions, reading or searching `../DEAP01-spec-core/crates/*` to inspect compiler internals is strictly forbidden; treat Tier-0 binaries as black-box verifiers and report upstream defects via `adversarial-code-auditor`.

## Zero Em Dash, Commit Non-Closure & Durable Artifact Rules
- **Zero Em Dash**: Unicode em dashes (`\u2014`) are strictly forbidden everywhere; use ASCII `--` or `-`.
- **Commit Non-Closure**: Auto-closing keywords (`fix #`, `fixes #`, `close #`, `resolve #`) are strictly prohibited in commit messages; use `(#<id>)` or `(refs #<id>)` exclusively.
- **Durable Engineering Artifacts vs. Isolated Session Plans**: All durable engineering artifacts (design blueprints, specifications, schemas, defect dossiers, reports) MUST be committed inside the Git repository (`docs/`, `schema/`, `.pipeline/defects/`). Transient per-conversation `implementation_plan.md` and `walkthrough.md` reside in `<appDataDir>/brain/<conversation-id>/` (or `.pipeline/plans/<slug>.md`) to prevent parallel workstream collisions.
