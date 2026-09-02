<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

---
name: feature-driven-implementation
description: "Implements Agile features using serial, subagent-driven, TDD-disciplined execution with two-stage review gates. Use when you need to implement a feature from a GitHub backlog with micro-task decomposition, RED-GREEN-REFACTOR cycles, and automated Epic resolution."
compatibility: "Requires git and configured issue tracker. Works with major agent orchestrators."
metadata:
  risk: low
  source: custom
  version: "2.0"
---

# Feature-Driven Autonomous Delivery & Closure

Use this skill to execute the end-to-end implementation lifecycle for prioritized Agile features and ensure complete automated resolution of feature issues, walkthrough updates, and parent Epics. Resolution means `Fixed / Resolved`; an agent never reaches `Closed`, which requires Product Owner validation (`.pipeline/constitution.md:161`).

This skill integrates subagent-driven development, TDD execution discipline, two-stage review gates, micro-task decomposition, systematic debugging, and verification-before-completion — ensuring that agents cannot drift, falsely report success, or skip quality gates.

## Core Mandates

1. **Serial Execution:** Strictly implement **one feature at a time**. Do not start feature N+1 until feature N is completely verified, merged, documented, and resolved.
2. **The Grill Approval:** Create an implementation plan and obtain explicit human approval BEFORE modifying any source files.
3. **Traceability:** Every issue taken to `Fixed / Resolved` MUST carry a resolution comment referencing the relative path or GitHub URL of the committed solution walkthrough.
4. **Agentic Epic Resolution:** When all constituent features of an Epic are resolved, the agent must check off the items in the local Epic markdown, update the Epic issue's body on GitHub, and mark the Epic issue `Fixed / Resolved`. The agent stops there; moving an Epic to `Closed` is the Product Owner's decision.
5. **Verification Isolation:** Verification of the implemented feature must follow the specific testing frameworks and execution models defined in the platform profile (e.g., E2E, unit, or integration tests). Refrain from using unapproved browser automation, headless engines, or scripts unless explicitly allowed by the project profile.
6. **Issue Tracker as Source of Truth:** Do not rely on local files or checklist documentation for feature definitions or backlog status as they may be contaminated or contain broken links. Always query the official issue tracker provider as the canonical source of truth.
7. **Cumulative Walkthroughs & Document Integrity:** When writing or updating living artifacts (such as implementation plans, task lists, and verification walkthroughs), you MUST NOT perform destructive overwrites. Always read the existing file first. Append or merge new details so the historical record remains fully intact.
8. **Validation Isolation & Separate Subagent Audit:** The primary agent MUST NOT self-verify database changes or documentation links without a strict checklist. Dispatch a separate **Validator Subagent** if available. In single-agent contexts, perform a strict, isolated self-audit (Step 4.5 fallback) verifying that every structural identifier and link target referenced in the walkthrough resolves and exists in the unified data repository.
9. **Test-Driven Development (TDD):** All implementation MUST follow the RED-GREEN-REFACTOR cycle. Write a failing test FIRST, verify it fails, write minimal code to pass, verify it passes, then refactor. Code written before its corresponding test must be deleted and re-implemented after the test.
10. **Micro-Task Decomposition:** Break every approved implementation plan into micro-tasks of 2-5 minutes each. Each micro-task must have: exact file paths, expected changes, a driving test, and a verification step. Never execute more than one micro-task without verification.
11. **Subagent-Driven Development:** Each micro-task SHOULD be dispatched to a fresh subagent with isolated context. The coordinator provides only the task text, relevant file contents, and project conventions — never the full session history. This prevents context drift and confirmation bias. See Step 3 for runtime-specific dispatch instructions.
12. **Two-Stage Review:** After each micro-task's implementation, two reviews MUST occur in order: (1) **Spec Compliance Review** — does the code match the approved plan and RFC/spec requirements? (2) **Code Quality Review** — is the code well-structured, typed, tested, and maintainable? Both must pass before proceeding to the next task.
13. **Verification-Before-Completion:** Before declaring any task, micro-task, or feature complete, the agent MUST provide concrete proof of correctness (raw test output, build output, or explicit file-content verification). Assertions like "it works" or "tests pass" without pasted evidence are forbidden.
14. **Inter-Task Code Review:** After each micro-task, diff the changes against the approved plan. Log deviations. Critical deviations block progress until resolved with the user. A deviation is **critical** — HALT and ask, do not log-and-continue — when any of these hold:
    - the set of files touched differs from the set named in the task;
    - the task says append or add, and delivering it requires modifying or replacing existing code;
    - the change alters the behaviour or signature of an existing symbol that has callers, including changing which error type a function returns;
    - the work expands beyond the named files to make the named files compile or pass.
    Everything else is logged and execution continues. "This is the only viable path" is not a finding — if the task as written cannot be delivered without a critical deviation, that is the report, and the plan is what changes.
15. **Subagent Research & Write Delegation:** The coordinator is strictly required to delegate all framework/dependency research tasks (Step 1.5) to a dedicated research subagent, and all codebase/specification micro-task write operations (Step 3) to dedicated implementer subagents.
16. **Closed-Loop Payload Verification Gate & Anti-Complacency Rule:** Exit code 0 is NEVER sufficient proof of success. After modifying or publishing any GitHub issue or document, the agent MUST run `gh issue view <ID>` or `gh api` to fetch the live published payload and inspect links, Mermaid headers, and syntax. Optimism bias is prohibited: agents must cite empirical output of live payload inspection before declaring completion.

---

## Step-by-Step Workflow

### Step 1: Backlog & Dependency Mapping
1. **Read the project constitution** (`.pipeline/constitution.md`) if it exists. This is the **functional layer** — domain rules, agent behavior, universal quality gates.
2. **Read the implementation profile** (`.pipeline/profiles/<target-platform>.md`) for the target platform. This provides platform-specific coding standards, testing mandates, build commands, and deployment config. If no profile exists for the target platform, halt and prompt the human to create one using the `project-constitution` skill. Ensure that the downstream workspace has been bootstrapped using the configured onboarding workflow or the Direct Copy workflow (Installation Option 2 in the README). This creates/copies a fresh repository locally BEFORE the downstream agent begins work.
3. Analyze `docs/epics/` and `docs/features/` to determine feature dependencies.
4. Map the backlog queue in order of base dependencies first.
5. Create a local tracking file (e.g., `task.md`) to manage current tasks.
6. **Dual-Track MBD Definition of Done (DoD) Scoping**:
   For any feature involving control laws, flight guidance, physical estimation, or safety statecharts, verify and enforce the Dual-Track MBD deliverables governed by `rules/dual-track-mbd-verification.md` and `docs/architecture/blueprints/SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE.md`:
   - **Track A (Native MATLAB / Simulink Synthesis)**: Programmatic model construction script (`models/scripts/build_<feature_slug>_model.m`), physical parameter dictionary (`models/matlab/<feature_slug>_params.m`), and Simulink Data Dictionary (`models/matlab/<feature_slug>_data.sldd`) configured for deterministic discrete solvers (`FixedStepDiscrete`) and Embedded Coder DO-178C C / SPARK Ada code synthesis.
   - **Track B (Headless CI Digital Twin Engine)**: Standalone, license-free discrete-time execution engine (`models/python/<feature_slug>_domain.py` and `models/python/<feature_slug>_engine.py`) executing at the exact same discrete loop rate ($dt$, e.g. 250 Hz / 0.004 s) with identical transition guards, polynomial transfer curves, and 6-DOF vehicle kinematics.
   - **Zero License Blocker Invariant**: 100% automated CI regression test harness (`tests/test_<feature_slug>_simulation.py`) executing offline without requiring MathWorks licenses.
   - **Mathematical & Discrete Equivalence Mandate**: Numerical tolerance verification ($\le 10^{-6}$) and formal simulation results report (`docs/reports/simulink_results/<FEATURE-ID>_simulation_results.md`).

### Step 1.5: Tech Stack Research (Optional)

If the feature involves unfamiliar frameworks, rapidly-evolving libraries, or new platform capabilities, the coordinator MUST dispatch a **Research Subagent** (role: `Codebase Researcher`) using whichever tool the active runtime exposes for context-isolated subagent dispatch, to investigate the codebase and target dependencies. Concrete per-runtime tool names are listed only in `.agents/AGENTS.md` § *Mandatory Subagent Dispatch for Research, Specification & Implementation Loops*, so that a change of runtime cannot make this instruction unexecutable (issue #312). If the active runtime's tool list is consulted and no such capability is found, HALT and escalate as a blocker; do not substitute direct coordinator research. The research subagent will follow these steps:

1. **Identify research targets:** What specific APIs, libraries, or patterns does this feature require?
2. **Check versions:** Verify the installed/pinned versions of relevant dependencies. Note any breaking changes between the project's current version and the latest.
3. **Document findings:** Create `research.md` on the feature branch with:
   - Framework/library versions being used
   - Relevant API documentation links
   - Known gotchas, migration notes, or deprecation warnings
   - Patterns recommended by official docs
4. **Feed into The Grill:** Reference `research.md` during the plan review to ensure the implementation plan accounts for framework realities.

> Skip this step if the feature uses well-established patterns already documented in the codebase.

### Step 2: Checkout & Plan Review ("The Grill")
1. Checkout a dedicated feature branch from the default branch resolved from configuration using the environment-configured checkout command and branch naming conventions (e.g. resolved dynamically from configuration):
2. **Platform Scoping:** The feature spec is platform-independent (functional). Apply the target platform from the implementation profile (`.pipeline/profiles/<platform>.md`, loaded in Step 1) to translate the abstract UML-mapped functional specifications into concrete platform-specific design decisions. This is where framework-specific libraries, language choices, structural patterns, and test frameworks are chosen.
3. Create/update `implementation_plan.md` outlining a **complete vertical slice** conforming to the architectural layers defined in the target platform profile (e.g., database, serialization, API, logic, or UI presentation layers):
   - **Architectural Layers:** Map the UML classes, components, operations, and attributes to their corresponding layer locations in the target codebase stack.
   - **Layout Engine Alignment:** Ensure the plan specifies CSS Container Queries (`@container`), flexbox/grid layout models, resizable split containers, and high-density standards matching `.pipeline/constitution.md` and the platform profile.
   - **Emulator Integration:** Confirm the plan contains E2E/integration tests verifying persistence transactions against a running local database emulator rather than in-memory mocks/stubs.
   - **Test Plan (TDD):** For each layer, specify the failing tests that will be written BEFORE the implementation code, using the test framework and runners specified in the platform profile.
   - **Verification Plan:** Detailed verification instructions, compiler checks, and test runner executions.
4. **Micro-Task Breakdown:** Decompose the plan into sequential micro-tasks (2-5 min each). Each task must specify:
   - Target file(s) and line ranges
   - What changes
   - The failing test that drives the change
   - How to verify completion
5. Present the plan to the user and wait for explicit approval. Enter "The Grill" — interactive review to challenge design choices, clarify ambiguities, and validate spec/RFC compliance.

### Step 3: Execution & Build (Subagent-Driven TDD Vertical Slice)

Execution follows a **per-task subagent dispatch loop**. The coordinator reads the plan once, extracts all micro-tasks, then dispatches a fresh implementer per task with two-stage review after each.

#### 3.1 Pre-Execution
1. **No Handover Trust:** Never assume previous phases or turns implemented a portion of the code correctly based on summaries. Explicitly open and check the source code files in all relevant directories.
   - **Provenance check.** If a target file already contains symbols the task assumed absent, establish where they came from **before editing them**: `git status <file>`, `git log -p -2 <file>`, `git diff <file>`. Committed code means the task was written against a stale view of the file — report that, do not silently reconcile. Uncommitted changes by another process mean a concurrent writer, which is a **HALT** under § 3.7 *Invariants* ("Never dispatch multiple implementer subagents in parallel on the same feature"); editing that file corrupts their work. Deciding which case applies is two commands and is never optional.
2. **Extract All Tasks:** Read the approved plan. Extract every micro-task with its full text, target files, driving test, and verification step. Create a tracking list (e.g., `task.md` or TodoWrite).
3. **TDD Cycle Mandate:** Mandate writing failing unit tests for both Happy Path AND all alternate/exception flows declared in specs before entering the GREEN refactoring phase.

#### 3.2 Per-Task Dispatch Loop

For each micro-task in sequence:

**A. Dispatch Implementer (Fresh Context)**

The coordinator agent is strictly locked from using modifying tools (`write_to_file`, `replace_file_content`, `multi_replace_file_content`) directly, and must delegate all write operations to a micro-task implementer subagent.

The implementer receives ONLY (Prompt Payload Template):
- The micro-task text (exact scope, target files, expected changes)
- Mandatory Subagent Task Checkboxes: Extract and format all UML Class Diagram operation signatures, attributes, and Use Case alternate flows into mandatory subagent task checkboxes in the prompt payload template.
- Relevant file contents (read and provided by the coordinator)
- Project conventions (TDD mandate, typing rules, docstring mandate, drill-down navigation rule, etc.)
- The driving test specification
- Mandatory Skill and Profile Read: Explicit prompt instruction requiring the subagent to execute view_file on both (1) the active SKILL.md file by explicit path (e.g., `skills/feature-driven-implementation/SKILL.md`) and (2) `.pipeline/profiles/<platform>.md` (e.g., `.pipeline/profiles/flutter.md`) as its very first steps.
- Preamble Integrity Sentinel: Mandatory sentinel token line (e.g. `---GOVERNANCE-END---`) terminating the governance preamble block. The subagent MUST verify the sentinel token before executing and report BLOCKED if missing due to preamble truncation.
- Mandatory Docstring & Traceability Requirement: Prompt instruction requiring that every generated class, interface, method, function, and public property MUST include full docstrings (DartDoc `///`, JSDoc `/** */`, Python `"""`), and attach UML traceability tags (`/// Realises: [SpecName/ClassName]`) to every public class header.
- **Mandatory Governance Acknowledgment**: Before executing any code writes, the subagent MUST echo back a structured summary of the governance rules received, declaring that it has read and will honor:
  (a) `.pipeline/constitution.md` Section 1.9 Zero-Mocking Live Persistence Mandate
  (b) The 3-layer LUI Definition of Done: DomainModel, ViewModel, LUI Widget Binding
  (c) The target platform profile (`.pipeline/profiles/<platform>.md`)
  (d) TDD RED-GREEN-REFACTOR cycle mandate
  The coordinator MUST validate acknowledgment receipt before the subagent proceeds to code generation. If the acknowledgment does not arrive or omits any of the four items, the coordinator MUST re-dispatch the subagent with the full preamble and log a governance delivery failure.

**Mandatory Subagent Prompt Governance Preamble**:
Every subagent prompt transmitted by the coordinator MUST begin with the complete, un-degraded Subagent Prompt Governance Preamble containing all mandatory verbatim governance markers:
1. **Skill / Constitution Reference**: `Adopt the feature-driven-implementation skill` (referencing `.pipeline/constitution.md`).
2. **First Step Mandate**: `view_file on skills/feature-driven-implementation/SKILL.md as step 1`.
3. **Zero-Mocking Mandate**: `Section 1.9 Zero-Mocking Live Persistence Mandate`.
4. **3-Layer DoD**: `3-Layer Definition of Done (DoD)`.
5. **TDD Mandate**: `RED-GREEN-REFACTOR`.
6. **Build/Test Verification Commands**: `flutter analyze (0 issues), flutter test (all pass)` (or platform profile equivalents).
7. **Preamble Integrity Sentinel Token**: `---GOVERNANCE-END---` (terminating line).

The coordinator is strictly forbidden from truncating, abbreviating, or stripping this preamble across sequential feature executions regardless of prior subagent success or failures.

**Subagent Failure Protocol**:
If a subagent returns an empty result, fails unexpectedly, or stalls:
1. **First Failure**: Upon an empty result or transient error, re-dispatch a fresh subagent with the identical, un-degraded preamble and full payload intact.
2. **Two Consecutive Failures**: If two consecutive failures occur with identical preamble payloads, HALT immediately and escalate to human operator with the failure log.
3. **Governance Invariant**: Never strip governance under any circumstances — empty result -> re-dispatch with identical preamble -> two consecutive failures -> escalate to human. Never strip governance.

The implementer MUST NOT receive the full session history or prior task context.

**Bypass Subagent Tool Lock**: The coordinator MUST append the keyword `PROCEED` (case-insensitive) to the end of the subagent prompt to authorize modifying tools for the subagent task context.

**Orchestration-Specific Dispatch**:

Configure the dispatch method dynamically based on the current agent orchestrator environment. For example:
- **Native Context-Isolated Agents**: Use native tool-based subagent calls with isolated task payloads.
- **Standard CLI/API Clients**: Execute a dedicated CLI client command or invoke subagent tool interfaces with curated context payloads.
- **Manual/Semi-Autonomous Context Reset**: If operating in environments without native subagent isolation, manually reset context by explicitly reading target files from disk and prefixing instructions to ignore prior task history.

**B. Implementer Executes TDD Cycle**
- **RED:** Write failing unit tests for both Happy Path AND all alternate/exception flows declared in specs before entering the GREEN refactoring phase. Run them and confirm they fail with the expected error.
- **GREEN:** Write the minimal code to make all tests pass. Run them. Confirm they pass.
- **REFACTOR:** Clean up the code while keeping tests green. Run tests again.
- **COMMIT:** Commit the passing micro-task with a descriptive message.
- **SELF-REVIEW:** Implementer reviews own changes before handing back.

**A.2 Output Integrity Verification**

After the subagent returns and BEFORE evaluating status, verify dispatch integrity:
1. Confirm the subagent output contains a test file (RED phase output) or explicit test code.
2. Confirm the subagent output contains implementation code (GREEN phase output) or a build/compile result.
3. If either is missing or the output is truncated (abrupt end, incomplete docstrings, or invisible dispatch failure), treat as `NEEDS_CONTEXT` and increment `subagent_dispatch_retry`.
4. **Bounded Retry Limit & Escalation Guard:** Track `subagent_dispatch_retry` for each micro-task dispatch. If `subagent_dispatch_retry` reaches 2 retries (max 2 retries on invisible/truncated/unexpected subagent output), HALT immediately and escalate to human via "The Grill" (`RETRY_LIMIT_EXCEEDED`).
5. **No Endless Payload Variations:** Endless or unauthorized payload format variations without root cause diagnosis are strictly forbidden. Do NOT vary the payload format ("maybe this format will work") without diagnosing the exact root cause. Report the exact failure mode (missing section, truncation point, preamble degradation).

**C. Handle Implementer Status**
- **DONE (integrity verified):** Proceed to two-stage review (Step 3.3).
- **DONE_WITH_CONCERNS:** Read concerns. If correctness/scope issue, address before review. If observational, note and proceed.
- **NEEDS_CONTEXT:** Coordinator provides missing context and re-dispatches. Increment `subagent_dispatch_retry`.
- **RETRY_LIMIT_EXCEEDED (HALT and escalate):** HALT immediately. Report failure mode and escalate to human via "The Grill."
- **BLOCKED:** Assess blocker: (1) context problem → provide more context, (2) task too complex → break into smaller pieces, (3) plan is wrong → escalate to human via "The Grill."

#### 3.3 Two-Stage Review Gate

After each micro-task's implementation, two reviews MUST occur **in this order**:

**Stage 1: Spec Compliance Review**
- Does the code match the approved plan exactly?
- Does it comply with the RFC/spec requirements from `docs/features/` and `docs/user-stories/`?
- Is anything missing from the spec? Is anything extra (not requested)?
- **Persistence Verification:** Assert that all persistence transactions are validated directly against a running local database emulator during local integration runs (no stubs).
- **Coupling & Leakage Audit:** Verify that no direct database SDK dependencies (e.g. `@firebase/firestore` or `cloud_firestore`) leak into UI / presentation components. All components must interact exclusively with abstract repositories.
- **Layout Engine Compliance:** Verify that split workspaces align with `logical-layout.json`, resizable splitter containers isolate reflows using CSS Container Queries to prevent unmounting state loss, and icons conform to the 16px SVG outline limits (stroke weight 1.0px–1.2px, cell padding 4px).
- **Model Integrity Check:** Verify that every constructor, `copyWith`, and `valueWriter` in the diff includes or passes through new fields.
- **Governance Adherence Check:** Verify the subagent's governance acknowledgment is present and complete. If missing or incomplete, fail the review and require re-dispatch.
- **If issues found:** Implementer fixes → re-review. Do NOT proceed to Stage 2 until Stage 1 passes.

**Stage 2: Code Quality Review**
- Is the code well-structured and idiomatic?
- Are types correct and strict (no `any` unless justified)?
- Are tests meaningful (not smoke-only)?
- Does it follow project conventions (drill-down navigation, domain-driven design, etc.)?
- **If issues found:** Implementer fixes → re-review. Do NOT proceed to next task until Stage 2 passes.

**Orchestration-Specific Review:**

Configure the review method dynamically based on the current agent orchestrator environment. For example:
- **Multi-Agent Environments**: Dispatch separate spec compliance and code quality reviewer subagents with the diff and specification documents.
- **Single-Agent Fallback**: The coordinator agent performs both reviews as explicit, sequential self-audit steps: (1) re-read the spec, diff the changes, check compliance point-by-point, and (2) re-read the code, checking design tokens, standards, and conventions. Document all findings in the local tracking checklist before proceeding.

#### 3.4 Slicing Execution Order
- Implement the vertical slice in the architectural order specified by the target platform profile (e.g., bottom-up or data-first mapping). Ensure that TDD tests are written and verified red for each component before writing the minimal passing code.

#### 3.5 Inter-Task Continuation
- After both reviews pass, mark task complete in tracking list.
- **Do not pause to ask "Should I continue?"** — execute all tasks continuously unless BLOCKED.
- If deviation from plan is critical (architectural change, missing spec compliance), STOP and return to "The Grill."

#### 3.6 Code Presence Verification
Before proceeding to Step 4, perform explicit grep or file-reading checks of all modified files to guarantee that all implementation code actually exists in the files. Do not trust summaries. Once verified, **execute Step 4 immediately without pausing for user approval.**

#### 3.7 Invariants
- Maintain strong typing, domain-driven design conventions, and strict schema compliance.
- Ensure no linting, type-checking, or compilation errors at any point.
- Never dispatch multiple implementer subagents in parallel on the same feature (conflicts).
- Never start code quality review before spec compliance is approved (wrong order).
- Never skip the re-review loop (reviewer found issues = implementer fixes = review again).
- **Cross-Cutting Field Preservation:** When adding fields to domain models, all constructors, `copyWith` methods, and `valueWriters` MUST preserve new fields across all paths. Micro-tasks MUST explicitly include regression tasks for existing constructors, `copyWith` methods, and `valueWriters`.
- **Governance Acknowledgment Invariant:** No subagent may proceed to code generation without submitting and receiving coordinator approval of a complete governance acknowledgment.

### Step 3.8: Systematic Debugging (When Tests Fail Unexpectedly)

If a test fails with an unexpected error during Step 3, follow the 4-phase debugging protocol:

1. **Reproduce:** Isolate the failure. Run the single failing test in isolation. Record exact error output.
2. **Diagnose:** Trace the root cause. Do NOT guess — read the stack trace, add targeted logging, check variable state. Identify the exact line and condition causing failure.
3. **Fix:** Apply the minimal upstream fix. Prefer single-line changes. Do not add workarounds downstream. Do not "fix" by weakening or deleting the test.
4. **Verify:** Run the full test suite (not just the fixed test) to confirm no regressions. Only proceed when all tests pass.

### Step 4: Verification & Testing
1. **Assertion-Based Automation:** When writing or updating tests, do not rely on basic smoke tests. Add explicit assertions that query return values, object states, or output trees for the presence of the new fields or data properties. For any modified domain models, mandate regression assertions on existing tests for operations on modified domain models to verify field preservation through every constructor, `copyWith`, and `valueWriter` path.
2. **Full Compilation Build:** Run local tests and run a full compilation build of the entire application (e.g. `flutter build` or `npm run build` as specified by the platform profile) to ensure it compiles without errors and is completely ready to run.
3. **Parity Auditor Gate:** Mandate running PYTHONPATH=skills/spec-orchestrator/parity_auditor/src python3 -m parity_auditor.cli --workspace . --allow-missing-specs as a blocking gate before completing any implementation task.
4. **Evidence of Completion:** Paste actual raw test output / build output as proof. Do not summarize — show the raw output.
5. Provide **precise, step-by-step human manual testing instructions** in the verification section. The instructions must guide the user on exactly what commands, scripts, or interface interactions to execute, what inputs to feed, and what specific output (e.g., payload, log entry, UI state change, database record) to inspect to verify correctness.
6. **Independent Subagent Validation Check (or Single-Agent Fallback Self-Audit):**
   - **Multi-Agent Mode:** Dispatch a separate **Validator Subagent** to read the draft walkthrough and cross-reference every referenced structural identifier and link target. The Validator subagent must independently locate these elements in the codebase to confirm they exist and match the logical specifications. Fail the validation step if there is any mismatch.
   - **Single-Agent Fallback:** The agent must step out of the implementation context and systematically audit its own draft walkthrough. Perform exact search lookups to verify that every single structural identifier and link target referenced in the walkthrough exists verbatim in the codebase. Document the results of this check explicitly before requesting user approval.
7. Apply any feedback iteratively on the feature branch. Once all tests and validations pass, **execute Step 5 immediately without pausing for user approval.**

### Step 5: Release & Closure (CRITICAL)
1. Merge the feature branch into the configured default branch resolved from configuration rules using the configured merge command template.
2. Create or update a cumulative solution walkthrough document under the configured design directory (e.g. `docs/designs/feat-<Issue_Number>-solution.md`) summarizing the changes, testing, and validations. This document must include a Code Realization Table that explicitly maps each feature and its attributes to the implemented source files, classes, methods, and functions matching the target platform's file extensions (e.g., `.tsx` for React, `.dart` for Flutter, `.cs` for C#, `.py` for Python). Do not delete or overwrite sections for previously implemented sub-features or related components. Ensure the document is a cumulative record of all changes.
   > [!IMPORTANT]
   > **DO NOT USE THE FEATURE INDEX NUMBER** in the solution filename if the tracker issue number is different. The solution filename MUST strictly use the tracker issue number (e.g., `feat-<Issue_Number>-solution.md`).
   >
   > **ZERO-TRUST COLLISION CHECK:** Before updating or creating this file, search the repository and history for the target filename to check its existing content. If it exists, read it first and append/merge the new changes rather than overwriting. If there is a filename mismatch or conflict, alert the user and resolve the naming conflict immediately.
3. **Backlog Reconciliation**: Run the backlog reconciliation script to synchronize all checklists, updated spec bodies (such as fixed Mermaid diagrams), and issue states back to GitHub: python3 skills/spec-orchestrator/scripts/reconcile_backlog.py
4. Commit and push the solution document using the configured commit command template.
5. Mark the feature issue `Fixed / Resolved` on the active issue tracker provider: apply the `status:fixed-resolved` label and embed a comment pointing to the committed solution document, dynamically constructing the URL using `meta.upstream_repository` from configuration. Leave the issue open — `Closed` requires Product Owner validation (`.pipeline/constitution.md:161`).
6. Update the local parent Epic checklist:
   - Mark the completed feature as completed (`[x]`).
   - Commit and push the updated Epic checklist file.

### Step 6: Agentic Epic Closure (CRITICAL)
1. Inspect the local Epic checklist.
2. If **all** features listed in the Epic checklist are checked off (`[x]`):
   - Update the Epic issue body on the active tracker provider with the completed task list.
   - Mark the Epic issue `Fixed / Resolved` on the active tracker provider: apply the `status:fixed-resolved` label and embed a comment indicating successful completion. Leave it open for Product Owner validation.
3. Delete the feature branch locally and remotely using the configured branch cleanup commands.

## Error Recovery
If a tool command (linter or reconciler) fails during implementation:
1. **Do not proceed** to the next step.
2. **Log the exact error** (stderr, exit code).
3. **Automated Upstream Reporting**: If the step fails and you suspect it is due to a bug in the pipeline tooling, you MUST automatically file a bug report upstream:
   - Locating the latest diagnostic payload JSON at `.pipeline/diagnostics/repro_payload_[timestamp].json`.
   - Running the command:
     `gh issue create --title "Tooling Bug: [Command] failed" --body-file [payload_path] --label "bug"`
4. Escalate to the user with the issue URL and error context.

