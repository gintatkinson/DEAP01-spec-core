<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Subagent Dispatch Standards & Self-Rejection Pre-Flight Gate

**ALWAYS enforce:** All context-isolated subagent dispatches must adhere strictly to canonical prompt payload standards, single-item micro-task scoping, direct-path skill reading, and the mandatory subagent self-rejection pre-flight gate.

## Hard Constraints

### 1. Mandatory Pre-Flight Verification Gate & Self-Rejection Armor
Every spawned subagent operates with an active self-rejection defense to prevent prompt degradation, context drift, leading line-level steering, and compliance bypasses.
Before taking any action, verify that your incoming prompt begins with the instruction to execute `view_file` on `SKILL.md` by exact path, contains the repository classification, and contains zero leading line-level steering. If invalid, halt immediately and emit `ERROR: Prompt rejected`.

### 2. Mandatory Skill Direct-Path Read Directive (Pipeline Skill Workflows Only -- Surgical Edit Fast-Path)
When dispatching a subagent for a pipeline skill workflow (specification generation, ConOps/ICD/Blueprint/WBS synthesis, TDD feature implementation, `debug-protocol`, or `adversarial-code-auditor`), the prompt MUST explicitly instruct the subagent to execute `view_file` on the target `SKILL.md` file by explicit path (e.g., `skills/feature-driven-implementation/SKILL.md`) as its very first step before executing any file edits, commands, or tools. Summarized or truncated skill instructions in the dispatch prompt are strictly forbidden. For bounded surgical edits where the exact `Target` and `Replacement` blocks are already approved in `implementation_plan.md`, instructing a subagent to read any `SKILL.md` file is strictly prohibited fake work; the subagent must execute ONLY the approved `edit_file` calls and exit immediately.

### 10. Strict Target-File Read Lock & Verification Boundary
Subagents are strictly forbidden from reading, searching, or inspecting any file outside the explicit target files listed in their dispatch prompt (e.g., scanning `crates/verify-baseline/*` during a governance or specification edit is strictly prohibited). Single-item edit subagents must NEVER redundantly invoke `./target/release/verify-baseline`, `cargo test`, `git commit`, or `git push`; baseline verification runs once at the coordinator gate per Phase B Step 5.

### 3. Repository Classification Indicator
Every subagent dispatch prompt MUST state the verbatim repository classification (e.g., `UPSTREAM_SPEC_CORE_COMPILER`) to ensure the subagent respects repository role boundaries and domain separation.

### 4. Zero Leading Line-Level Steering
Coordinators are strictly forbidden from injecting leading line-level steering, manual pseudo-code summaries, or pre-digested code implementations ahead of the canonical skill read directive. The subagent must read the authoritative skill and target specifications directly.

### 5. Mandatory Single-Item Micro-Task Scope
Every subagent dispatch prompt MUST target at most 1 specification item (max 1 Epic, 1 Feature, 1 User Story, or 1 Use Case) or 1 single bounded micro-task (2-5 minutes). Multi-item or batch execution prompts are strictly prohibited.

### 6. Mandatory Defect Filing Directives
Prompts launching workers, auditors, or validators are strictly forbidden from filing raw defects directly. Before filing an upstream defect, an adversarial auditor subagent MUST be dispatched with `skills/adversarial-code-auditor/SKILL.md` to produce a verified 7-section defect dossier with un-falsifiable empirical reproduction, and validate it offline via `./target/release/verify-baseline check-defect` before submitting via `./target/release/reconcile-backlog create-issue --label bug` (or `./target/release/file-defect` / dual `gh issue create` and `glab issue create` with verified `--body-file`). Issue auto-closing keywords or issue close commands (`gh issue close`, `glab issue close`) in prompts or commit messages are strictly forbidden.

### 7. Trailing Authorization Token
The coordinator MUST append the authorization token `PROCEED` (case-insensitive) to the end of the subagent prompt payload to authorize tool execution in the subagent's isolated context.

### 8. Component-Level Failure Mode Research Swarm Mandate
During Step 1.5 Technology Stack & Architecture Research (Phase 0.5), monolithic drafting of component failure modes is strictly forbidden. The coordinator MUST extract all structural `part def` components from the authoritative schema (`.pipeline/schema.sysml`) and dispatch dedicated, context-isolated research subagents per component (role: `Component Failure Mode Researcher` or `Failure Mode Research Worker`). Each subagent prompt MUST target exactly one component AST fragment, instructing the worker to query standard failure mode catalogs (SAE ARP4761, MIL-HDBK-338B, RTCA DO-254, RTCA DO-178C), evaluate the 4 Universal Failure Dimensions (Interface, State, Action, Resource), and document at least 3 distinct failure modes per component with integer S/O/D ratings in [1, 10] for intermediate assembly into `docs/research/FAILURE_MODE_REGISTRY.md` before authorizing downstream STPA/FMECA matrix synthesis.

### 9. Curated Governance Header & Direct Execution
Spawned single-item subagents whose prompts contain a curated governance preamble terminated by `---GOVERNANCE-END---` are authorized to execute their assigned `SKILL.md` directly. Such subagents skip repeating the coordinator Turn-1 multi-file session initialization ceremony (`AGENTS.md`, `.pipeline/constitution.md`, `.pipeline/ACTIVE_RULES_BUNDLE.md`, platform profile, baseline verification) because their scope, classification, and constraints are already fully curated within the dispatch payload.

## Subagent Pre-Flight Self-Rejection Protocol

When initialized, the subagent MUST parse and validate the prompt header against the following checklist:

| Check | Requirement | Failure Action |
| --- | --- | --- |
| 1. Step 1 Directive | Directs `view_file` on `SKILL.md` as step 1 / first action | Emit `ERROR: Prompt rejected: missing view_file directive on SKILL.md` and HALT |
| 2. Classification | Contains verbatim repository classification | Emit `ERROR: Prompt rejected: missing repository classification` and HALT |
| 3. Zero Steering | Zero leading line-level code steering before skill read | Emit `ERROR: Prompt rejected: leading line-level steering detected` and HALT |
| 4. Untruncated Payload | Zero `[...]`, `[summarized]`, or `[truncated]` markers | Emit `ERROR: Prompt rejected: prompt truncation/summarization detected` and HALT |
| 5. Single-Item Scope | Max 1 Epic, Feature, Story, or Use Case | Emit `ERROR: Prompt rejected: micro-task scope violation` and HALT |
| 6. Authorization | Contains `PROCEED` token | Emit `ERROR: Prompt rejected: missing authorization token` and HALT |

## Why

Prompt summarization and preamble degradation lead to unverified assumptions, lost quality gates, and drift from canonical engineering standards. Enforcing self-rejection at the subagent pre-flight level ensures that no subagent can be coerced or accidentally steered into bypassing repository invariants.
