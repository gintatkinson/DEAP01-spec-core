# Fresh Session Handoff: Adversarial Code Auditor Issue Publication -- Analysis, Acceptance Criteria & Solution Architecture

## 1. User Problem Statement & Explicit Scope Boundary
- **Initial Symptom Reported by User**:
  > "when the adverserial auditor runs it still does not post the defect as an issue to: https://github.com/gintatkinson/DEAP01-spec-core/issues"
- **User's Explicit Scope Corrections**:
  1. **Do NOT jump into fixing code**: The user did **not** ask the agent to immediately modify code or apply a fix.
  2. **More analysis is required to understand the Acceptance Criteria**: The new session must first conduct a thorough, subagent-driven analysis of all execution paths, failure modes, and requirements to establish complete, unambiguous **Acceptance Criteria**.
  3. **Present the Architecture of the Solution first**: Before any code or skill remediation is planned or executed, the **Architecture of the Solution** and the **Acceptance Criteria** must be synthesized and presented to the user for review and alignment.

---

## 2. Workspace & Repository Context
- **Workspace Root**: `/Users/perkunas/jail/DEAP01-spec-core`
- **Repository Classification**: `UPSTREAM_SPEC_CORE_COMPILER` (`.pipeline/upstream/` and `skills/spec-orchestrator/` exist).
- **Working Tree State**: `implementation_plan.md` has been deleted per user instruction so there is zero pollution from the prior invalidated session. Only this `handoff.md` file is untracked.
- **No Cross-Talk Rule (`AGENTS.md:109-110`)**: Do NOT read or reference any transcripts, logs, or artifacts from prior conversation IDs under `<appDataDir>/brain/`.

---

## 3. Why the Previous Session Was Invalidated (Strict Anti-Patterns to Avoid)
1. **Unapproved Code-Fix Assumption (`AGENTS.md:84`)**: The previous coordinator silently assumed the user wanted immediate multi-crate code modifications instead of first analyzing the acceptance criteria and presenting the solution architecture.
2. **Coordinator Context Bloat (`AGENTS.md:63-75`)**: The previous coordinator read 30+ files directly in the primary coordinator conversation context instead of delegating codebase analysis to context-isolated `research` subagents (`invoke_subagent`).
3. **Violating the Strict Planning Gate (`AGENTS.md:33-40`)**: The coordinator modified `implementation_plan.md` during an inquiry turn without user authorization.

---

## 4. Mandatory Step-by-Step Protocol for the Fresh Session

### Step 1: Mandatory Session Initialization Gate (Turn 1)
Execute the exact initialization sequence in `AGENTS.md:22-31`:
0. **Detect Repository Role**: Verify `.pipeline/upstream/pipeline-tooling.md` exists via `view_file` -> `UPSTREAM_SPEC_CORE_COMPILER`.
1. **Read Governance Constitution**: Execute `view_file` on `.pipeline/constitution.md`.
2. **Load Active Task Skills**: Execute `view_file` on `skills/adversarial-code-auditor/SKILL.md`.
3. **Load Governance Rules**: Execute `view_file` on `.pipeline/ACTIVE_RULES_BUNDLE.md`.
4. **Skip Downstream Platform Profiles**: Do NOT read `.pipeline/profiles/*.md` during an `UPSTREAM_SPEC_CORE_COMPILER` session.
5. **Verify Baseline**: Execute `./target/release/verify-baseline . --no-domain` once.

### Step 2: Strict Planning Gate for Phase 1 (Analysis & Architecture Only -- Turn 1 Stop Point)
1. Do **NOT** read `crates/` source files in the coordinator's primary context (`AGENTS.md:63-75`) and do **NOT** plan or execute any source code edits.
2. Write a clean `implementation_plan.md` scoped **exclusively** to **Phase 1: Subagent-Driven Root-Cause Analysis, Acceptance Criteria Derivation & Solution Architecture Synthesis**:
   - **Subagent Dispatch 1 (`Adversarial Auditor Workflow & CLI Contract Researcher` -- read-only `research` subagent)**:
     - Inspect all governance, skill, and agent definitions that invoke or govern the Adversarial Code Auditor (`skills/adversarial-code-auditor/SKILL.md`, `.agents/skills/adversarial-code-auditor/SKILL.md`, `.agents/agents/adversarial_auditor/agent.md`, `AGENTS.md`, `rules/subagent-dispatch-standards.md`, `.pipeline/ACTIVE_RULES_BUNDLE.md`, `skills/spec-orchestrator/SKILL.md`, `skills/feature-driven-implementation/SKILL.md`, `skills/debug-protocol/SKILL.md`).
     - Map every documented command, flag, and instruction path an auditor subagent or coordinator follows when triggered.
   - **Subagent Dispatch 2 (`Rust Defect Publication Pipeline Researcher` -- read-only `research` subagent)**:
     - Inspect the Rust defect validation, persistence, and tracker publication pipeline (`crates/verify-baseline/src/defect_dossier.rs`, `crates/verify-baseline/src/runner.rs`, `crates/deap-core/src/diagnostics.rs`, `crates/reconcile-backlog/src/main.rs`, `crates/reconcile-backlog/src/create_issue.rs`, `crates/reconcile-backlog/src/tracker.rs`).
     - Trace all execution branches, CLI flag interactions (`--publish`, `--dry-run`, `--repo`, `--title`, `--issue`, `--label`), downstream vs. upstream workspace binary availability, `gh` CLI authentication/TLS sandbox behavior, idempotency/duplicate handling, and closed-loop verification (`verify_closed_loop_payload`).
3. **STOP** calling tools immediately after writing `implementation_plan.md` and wait for explicit user approval (`PROCEED`) to dispatch the read-only analysis subagents.

### Step 3: Subagent Execution & Synthesis of Solution Architecture + Acceptance Criteria (Post-`PROCEED`)
Once the user approves the Phase 1 analysis plan:
1. Dispatch the read-only `research` subagents with the mandatory `---GOVERNANCE-END---` preamble (`rules/subagent-dispatch-standards.md`) and immediately reclaim/terminate each subagent via `manage_subagents` once it reports back (`AGENTS.md:102-104`).
2. Synthesize and present to the user a comprehensive **Solution Architecture & Acceptance Criteria Presentation** covering:
   - **A. End-to-End Failure Mode & Gap Analysis**:
     - Why the Adversarial Code Auditor currently fails to post issues to `https://github.com/gintatkinson/DEAP01-spec-core/issues` across both upstream (`DEAP01-spec-core`) and downstream (`DEAP-*` / customer) workspaces.
     - Prompt/skill/rule contradictions vs. Rust binary CLI behavior.
   - **B. Solution Architecture (With Mermaid Architecture & Sequence Diagrams)**:
     - Component boundaries between `skills/adversarial-code-auditor/SKILL.md`, `.agents/agents/adversarial_auditor/agent.md`, `verify-baseline check-defect`, and `reconcile-backlog`.
     - Single canonical publication path vs. offline validation mode (`--dry-run`).
     - Upstream repository targeting invariant (always routing upstream compiler/tooling defects to `gintatkinson/DEAP01-spec-core` even when triggered inside a downstream repository).
     - Binary resolution strategy when running inside a downstream workspace where `./target/release/verify-baseline` may or may not exist locally.
     - Idempotency, deduplication, and closed-loop verification architecture (`gh issue view` byte-for-byte payload verification).
     - `exebox` macOS TLS / Keychain pre-flight failure handling so `gh` errors are never silently swallowed or misreported.
   - **C. Exhaustive Acceptance Criteria (Given-When-Then & Verification Matrix)**:
     - Formal Given-When-Then acceptance criteria covering every execution scenario, negative probe, and closed-loop verification gate.
3. **STOP** after presenting the Solution Architecture and Acceptance Criteria and wait for the user's architectural review and sign-off before proposing any implementation work packages.
