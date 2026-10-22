# Implementation Plan: Session Initialization & Governance Architecture Remediation

## 1. Problem Statement & Root Cause Analysis

Across the Digital Engineering Agent Platform (DEAP) ecosystem, autonomous coordinator agents operating in both upstream compiler (`UPSTREAM_SPEC_CORE_COMPILER`) and downstream application (`DOWNSTREAM_CUSTOMER_PROJECT`) workspaces exhibit systematic initialization failures and governance drift:

1. **The "Before Executing File Modifications" Deferral Trap**:
   `AGENTS.md` (§ `Mandatory Session Initialization Gate`) currently states: *"On session startup, before executing file modifications, the coordinator agent MUST execute..."*. Agents interpret "before executing file modifications" as authorization to defer Steps 0-3 until after an `implementation_plan.md` is approved, leaving Turn-0 responses and planning ungrounded in constitutional invariants.
2. **Coupling Read-Only Startup with Locked Execution**:
   `AGENTS.md` bundles Steps 0-3 (pure `read_file`, `list_dir`, and read-only inspection) into the same sequence as Step 5 (`./target/release/verify-baseline . --no-domain`). Because `rules/verification-required.md` strictly forbids running `verify-baseline` on startup/read-only turns and `rules/user-authorization-lock.md` locks `run_command` prior to `PROCEED`, agents defer the entire 0-5 block.
3. **4-Point vs. 5-Point Thought-Block Drift & Placement**:
   `AGENTS.md` places `Strict Coordinator Tool Locking & 4-Point Compliance Check` near the bottom of the file with 5 bullets, whereas `rules/user-authorization-lock.md` defines only 4 bullets and omits the initialization gate check. Furthermore, because it is located at the bottom of `AGENTS.md`, agents often stop reading or fail to prioritize it before taking action.
4. **Artifact & Shell Mutation Loopholes (`walkthrough.md` and `cp`/`mv`/`tee`)**:
   Agents frequently treat `walkthrough.md` as an exempt scratchpad to write during planning, or invoke shell commands like `cp`, `mv`, `tee`, or redirection (`>`, `>>`) as loopholes to circumvent file editor locks.
5. **Missing Canonical Initialization Receipt in Downstream Projects**:
   Downstream repositories fail to emit the structured `Mandatory Session Initialization Completed` status table on Turn 0 because:
   - Neither `AGENTS.md` nor `rules/constitution-first.md` formally codifies the Markdown receipt table contract.
   - `AGENTS.md` hardcodes `Repository Classification: UPSTREAM_SPEC_CORE_COMPILER` at line 4, which directly contradicts Step 0 when copied to downstream repositories where `.pipeline/upstream/` is absent.
   - `rules/user-authorization-lock.md` locks all shell commands prior to `PROCEED` without an explicit read-only inspection whitelist (`git status --short --branch`, `git rev-parse --show-toplevel`, `ls`), preventing coordinators from empirically populating the `Branch` and `Working Tree` fields in the receipt table.

---

## 2. Remediation Architecture & Invariants

To eliminate these failure modes permanently across all tiers without breaking downstream compatibility or existing Rust verification gates, this implementation plan establishes:

1. **Role-Adaptive Classification**:
   `AGENTS.md` dynamically defines `UPSTREAM_SPEC_CORE_COMPILER` (when `.pipeline/upstream/` is present) and `DOWNSTREAM_CUSTOMER_PROJECT` (when `.pipeline/upstream/` is absent), preserving required keywords for `verify-baseline` while remaining 100% accurate in both repository contexts.
2. **Two-Phase Initialization Gate**:
   - **Phase A (Unconditional Turn-0 Startup)**: Steps 0, 1, 3, and read-only git state check executed immediately on Turn 0 before any plan or prose response, followed by immediate emission of the Canonical Session Initialization Receipt Table.
   - **Phase B (Task-Scoped & Post-Approval Pre-Modification Gates)**: Step 2 (Task Skill on-demand load), Step 4 (Platform Profile load in downstream implementation phases only), and Step 5 (`verify-baseline` pre-commit gate executed only after `PROCEED` when tracked files are modified).
3. **Elevated & Synchronized 5-Point Compliance Gate**:
   Placed at the **very top of `AGENTS.md`** (immediately under `# Project-Scoped Rules`, prior to `## Repository Role & Scope Classification`), with 100% synchronization in `rules/user-authorization-lock.md` and lower section anchors preserved.
4. **Inspection Whitelist & Mutation Lock**:
   `rules/user-authorization-lock.md` and `rules/role-boundary-lock.md` explicitly whitelist read-only commands (`git status`, `git branch --show-current`, `git rev-parse --show-toplevel`, `git diff`, `ls`, `test -d`, `test -e`) on Turn 0, while explicitly locking `walkthrough.md` and shell file mutations (`cp`, `mv`, `rm`, `tee`, `sed -i`, `>`, `>>`) prior to `PROCEED`.
5. **Native Rust TDD Verification**:
   New Rust unit tests in `crates/deap-core/src/workspace.rs` validating all new structural invariants, receipt table specifications, 5-point checks, and bundle size constraints (<40,000 bytes, <750 lines).

---

## 3. Work Package Decomposition (Single-Item Subagent Execution)

Following project governance, all file modifications and Rust implementations will be executed serially by context-isolated single-item subagents once this plan is approved with `PROCEED`.

### Work Package 1: Elevate & Synchronize the 5-Point Thought-Block Gate (`AGENTS.md` & `rules/user-authorization-lock.md`)
- **Assigned Subagent Role**: `Governance Engineer -- Thought-Block Standardization`
- **Target Files**:
  - `AGENTS.md`
  - `rules/user-authorization-lock.md`
- **Detailed Specifications**:
  1. Insert `## Strict Coordinator Tool Locking & 5-Point Compliance Check` at the very top of `AGENTS.md` (lines 2-25, immediately below `# Project-Scoped Rules` and above `## Repository Role & Scope Classification`).
  2. Standardize the 5 points identically across `AGENTS.md` and `rules/user-authorization-lock.md`:
     - **Point 1 (Turn-0 Initialization Gate)**: Have I executed Phase A of the Mandatory Session Initialization Gate (`list_dir` or direct check to detect `.pipeline/upstream/`, `view_file` in full on `.pipeline/constitution.md` and `.pipeline/ACTIVE_RULES_BUNDLE.md`, and read-only `git status --short --branch`) AND emitted the Canonical Session Initialization Receipt Table in this session, OR does my subagent prompt end with `---GOVERNANCE-END---`? (Yes/No -- If No, HALT all other output, planning, and tool calls immediately, execute Phase A right now, and emit the Mandatory Session Initialization Completed table).
     - **Point 2 (Intent Classification)**: Is the user's message a question/inquiry or a direct command?
     - **Point 3 (User Authorization Lock)**: Are the exact target files and changes documented in an approved `implementation_plan.md` AND does the user's latest message explicitly contain `PROCEED` (or `Approved` / `Approve plan`)? (Yes/No -- If No, all modifying tools and mutating `run_command` invocations are strictly locked; writing/updating `implementation_plan.md` after Phase A initialization is the sole permitted file write).
     - **Point 4 (Zero Silent Assumptions)**: Am I making any silent assumptions about the user's intent or workspace state?
     - **Point 5 (Coordinator Delegation & Shell Mutation Lock)**: Does the active skill mandate context-isolated subagent dispatches, OR does this turn create, modify, copy, move, or delete any file other than `implementation_plan.md` (including `walkthrough.md`, governance files, rules, specs, or code)? (If Yes to either, coordinator direct file-writing and shell file mutations via `cp`, `mv`, `rm`, `tee`, `sed -i`, or output redirects `>`/`>>` are strictly locked; all repository writes must be delegated to single-item subagents after `PROCEED`).
  3. In `AGENTS.md`, keep the lower anchor heading `## Strict Coordinator Tool Locking & 4-Point Compliance Check` synchronized as an alias/reference pointing to the primary top-level 5-point check, ensuring backwards compatibility for any tests or tools asserting that heading string.
  4. In `rules/user-authorization-lock.md`, update the Hard Constraints to enforce the 5-point check and explicitly ban pre-approval creation or modification of `walkthrough.md`.

### Work Package 2: Make `Repository Role & Scope Classification` Role-Adaptive & Enforce Canonical Turn-0 Initialization Receipt Table (`AGENTS.md` & `rules/constitution-first.md`)
- **Assigned Subagent Role**: `Governance Engineer -- Initialization Architecture`
- **Target Files**:
  - `AGENTS.md`
  - `rules/constitution-first.md`
- **Detailed Specifications**:
  1. In `AGENTS.md` (§ `Repository Role & Scope Classification`), replace the static upstream classification with role-adaptive definitions:
     - **When `.pipeline/upstream/` is present -> `UPSTREAM_SPEC_CORE_COMPILER`**: Digital Engineering Agent Platform Core Specification Compiler. Landing zones (`schema/`, `docs/epics/`, `docs/features/`, `docs/user-stories/`, `docs/use-cases/`) must remain clean landing zones with only `.gitkeep`. Platform profiles (`.pipeline/profiles/*`) are strictly bypassed.
     - **When `.pipeline/upstream/` is absent -> `DOWNSTREAM_CUSTOMER_PROJECT`**: Downstream Application / Domain Engineering Workspace. Concrete schemas in `schema/` and `.pipeline/schema.sysml` serve as the 100% Single Source of Truth (SSOT). Platform profiles (`.pipeline/profiles/<target_platform>.md`) are loaded on-demand during downstream implementation turns only.
     - Preserve sentinel keywords (`UPSTREAM_SPEC_CORE_COMPILER`, `DOWNSTREAM_CUSTOMER_PROJECT`, `.pipeline/upstream/`, `skills/spec-orchestrator/`).
  2. In `AGENTS.md`, split `## Mandatory Session Initialization Gate` into:
     - **Phase A (Unconditional Turn-0 Read-Only Initialization -- Executed Immediately Before Any Response or Plan)**:
       - Step 0: Detect Repository Role & Scope via direct-path check for `.pipeline/upstream/`.
       - Step 1: Read `.pipeline/constitution.md` in full via `view_file`.
       - Step 2: Read `.pipeline/ACTIVE_RULES_BUNDLE.md` in full via `view_file` (with Task Skill on-demand visual slot).
       - Read-only Git state check: `git status --short --branch` and `git rev-parse --show-toplevel`.
       - Emit Canonical Session Initialization Receipt Table.
     - **Phase B (Task-Scoped & Post-Approval Pre-Modification Gates)**:
       - Load active `SKILL.md` on-demand when a task is assigned.
       - Load `.pipeline/profiles/<target_platform>.md` only in `DOWNSTREAM_CUSTOMER_PROJECT` during downstream implementation phases.
       - Baseline verification: run `./target/release/verify-baseline . --no-domain` only after `PROCEED` prior to implementation or commit gates (never on Turn-0 startup or read-only turns).
  3. In both `AGENTS.md` and `rules/constitution-first.md`, formally document the Canonical Turn-0 Initialization Receipt Table template contract:

````markdown
### Mandatory Session Initialization Completed

The coordinator initialization sequence has executed in accordance with `.pipeline/constitution.md` and `.agents/AGENTS.md`:

| Step | Check / Gate | Status | Resolution / Details |
| :--- | :--- | :--- | :--- |
| **0** | **Repository Classification** | Verified | `<UPSTREAM_SPEC_CORE_COMPILER (.pipeline/upstream/ detected) OR DOWNSTREAM_CUSTOMER_PROJECT (.pipeline/upstream/ absent)>` |
| **1** | **Governance Constitution** | Verified | Loaded `.pipeline/constitution.md` |
| **2** | **Task Skill On-Demand** | Standby | `<Awaiting active task directive OR Loaded skills/<active-skill>/SKILL.md>` |
| **3** | **Governance Rules Bundle** | Verified | Loaded `.pipeline/ACTIVE_RULES_BUNDLE.md` (`<N>` active rules) |
| **4** | **Platform Profile** | `<Bypassed OR Standby>` | `<Not applicable for upstream spec core compiler sessions OR Deferred until downstream implementation phase (.pipeline/profiles/<platform>.md)>` |
| **5** | **Baseline Gate** | Bypassed | Clean session startup; no tracked source or model files modified |

- **Branch**: `<empirical branch from git status>`
- **Working Tree**: `<Clean (git status clean) OR Dirty (N files modified/untracked)>`
- **Role Invariant**: `<Pure schema-driven abstract compiler; clean landing zones maintained. OR Pure schema-driven downstream workspace; .pipeline/schema.sysml is 100% SSOT.>`

Ready for instructions. What task or work package would you like to execute?
````

### Work Package 3: Clarify Turn-0 Read-Only Inspection Whitelist vs. Mutation Lock (`rules/user-authorization-lock.md` & `rules/role-boundary-lock.md`)
- **Assigned Subagent Role**: `Governance Engineer -- Tool Authorization Enforcement`
- **Target Files**:
  - `rules/user-authorization-lock.md`
  - `rules/role-boundary-lock.md`
- **Detailed Specifications**:
  1. In `rules/user-authorization-lock.md`:
     - Add explicit Turn-0 Read-Only Workspace Inspection Whitelist: during Turn-0 initialization and diagnostic inquiries, read-only commands (`git status`, `git status --short --branch`, `git branch --show-current`, `git rev-parse --show-toplevel`, `git diff`, `ls`, `test -d`, `test -e`) are authorized without requiring prior `PROCEED`, specifically enabling empirical population of the `Branch` and `Working Tree` fields.
     - Add explicit State-Mutating Shell Command Lock: all file creation, copying, moving, deleting, or stream redirection commands (`cp`, `mv`, `rm`, `tee`, `sed -i`, `>`, `>>`) as well as build/test commands (`cargo`, `flutter`, `npm`, `git commit`, `git push`, `verify-baseline`) are strictly locked until an `implementation_plan.md` is approved with `PROCEED`.
     - Add explicit Prohibition of Pre-Approval `walkthrough.md` writes: `walkthrough.md` is exclusively a post-implementation verification artifact and must never be created or edited prior to plan approval and subagent completion.
  2. In `rules/role-boundary-lock.md`:
     - Clarify that the coordinator direct file-writing lock applies to `walkthrough.md` and shell file mutations (`cp`, `mv`, `tee`), which cannot be used to bypass subagent delegation.

### Work Package 4: Synchronize `.pipeline/ACTIVE_RULES_BUNDLE.md` and `.agents/AGENTS.md`
- **Assigned Subagent Role**: `Governance Engineer -- Manifest Parity & Symlink Maintenance`
- **Target Files**:
  - `.pipeline/ACTIVE_RULES_BUNDLE.md`
  - `.agents/AGENTS.md` (or symlink)
- **Detailed Specifications**:
  1. Update `.pipeline/ACTIVE_RULES_BUNDLE.md` sections for:
     - Rule 4: `rules/constitution-first.md`
     - Rule 11: `rules/role-boundary-lock.md`
     - Rule 19: `rules/user-authorization-lock.md`
     ensuring 100% verbatim parity with the individual rule files under `rules/`.
  2. Verify that the total size of `.pipeline/ACTIVE_RULES_BUNDLE.md` remains strictly under 40,000 bytes and under 750 lines (monitored by `crates/deap-core/src/workspace.rs:972-979`).
  3. Check `.agents/AGENTS.md`: ensure `.agents/AGENTS.md` exists as a symbolic link pointing to `../AGENTS.md` (or valid mirrored file), ensuring Check 14 passes under any entrypoint lookup.

### Work Package 5: Native Rust TDD Enforcement in `crates/deap-core` and `crates/verify-baseline`
- **Assigned Subagent Role**: `Systems Software Engineer -- Rust TDD Specialist`
- **Target Files**:
  - `crates/deap-core/src/workspace.rs`
- **Detailed Specifications**:
  1. **RED Phase**:
     - Author new Rust unit tests in `crates/deap-core/src/workspace.rs`:
       - `test_canonical_turn_0_session_initialization_receipt_invariants`: Verifies that `AGENTS.md` and `rules/constitution-first.md` define both `UPSTREAM_SPEC_CORE_COMPILER` and `DOWNSTREAM_CUSTOMER_PROJECT` receipt specifications and the exact Markdown receipt table schema.
       - `test_5_point_thought_block_compliance_and_shell_mutation_lock`: Verifies that `AGENTS.md` and `rules/user-authorization-lock.md` specify all 5 points of the compliance check, explicitly whitelist read-only inspection commands, and lock shell mutations (`cp`, `mv`, `rm`, `tee`) and `walkthrough.md`.
       - `test_active_rules_bundle_parities_and_budget`: Verifies 100% verbatim parity between `rules/user-authorization-lock.md`, `rules/constitution-first.md`, `rules/role-boundary-lock.md` and their respective sections in `.pipeline/ACTIVE_RULES_BUNDLE.md`.
     - Execute `cargo test -p deap-core` to verify that the tests fail (RED) before governance documents are updated.
  2. **GREEN Phase**:
     - Coordinate with WP1-WP4 to apply all governance document updates.
     - Re-run `cargo test --workspace` to verify all tests pass (GREEN).
     - Run `./target/release/verify-baseline . --no-domain` to confirm all 28+ mechanical gates pass.
  3. **REFACTOR Phase**:
     - Review docstrings, ensure formatting cleanliness, and verify zero compiler warnings.

### Work Package 6: Downstream Repository Propagation Plan
- **Assigned Subagent Role**: `DevOps & Release Engineer -- Toolchain Distribution`
- **Target Files**:
  - `docs/operations/install-guide.md` (or release notes)
- **Detailed Specifications**:
  1. Document the exact downstream propagation steps for `DEAP-profile-flutter-app` (Tier 1) and `DEAP01-cuas-coa-engine` (Tier 2):
     - Synchronize updated governance assets: `AGENTS.md`, `.pipeline/constitution.md`, `.pipeline/ACTIVE_RULES_BUNDLE.md`, and `rules/*.md`.
     - Build release binaries in Tier 0: `cargo build --release --workspace`.
     - Copy the 5 precompiled release binaries (`verify-baseline`, `compile-sysml`, `ingest-sysml`, `assemble-conops`, `reconcile-backlog`) from `target/release/` to downstream `target/release/`.
     - Verify downstream execution: run `./target/release/verify-baseline . --no-domain` in downstream workspaces to confirm 0 errors.

---

## 4. Subagent Dispatch Matrix

| Work Package | Subagent Type | Role Title | Key Target Files | Trailing Authorization |
| :--- | :--- | :--- | :--- | :--- |
| **WP5 (RED)** | `self` | `Rust TDD Engineer (RED Phase)` | `crates/deap-core/src/workspace.rs` | `PROCEED` |
| **WP1** | `self` | `Governance Engineer (5-Point Check)` | `AGENTS.md`, `rules/user-authorization-lock.md` | `PROCEED` |
| **WP2** | `self` | `Governance Engineer (Init Receipt)` | `AGENTS.md`, `rules/constitution-first.md` | `PROCEED` |
| **WP3** | `self` | `Governance Engineer (Tool Whitelist)` | `rules/user-authorization-lock.md`, `rules/role-boundary-lock.md` | `PROCEED` |
| **WP4** | `self` | `Governance Engineer (Parity & Bundle)` | `.pipeline/ACTIVE_RULES_BUNDLE.md`, `.agents/AGENTS.md` | `PROCEED` |
| **WP5 (GREEN)** | `self` | `Rust TDD Engineer (GREEN Phase)` | `crates/deap-core/src/workspace.rs` | `PROCEED` |
| **WP6** | `self` | `Release Distribution Engineer` | `docs/operations/install-guide.md` | `PROCEED` |

---

## 5. Verification & Acceptance Criteria

- [ ] **Thought-Block Compliance**: The 5-point check is located at lines 2-25 of `AGENTS.md` and identically in `rules/user-authorization-lock.md`.
- [ ] **Role-Adaptive Classification**: `AGENTS.md` supports both `UPSTREAM_SPEC_CORE_COMPILER` and `DOWNSTREAM_CUSTOMER_PROJECT` without hardcoded conflict.
- [ ] **Two-Phase Initialization Gate**: Phase A (Turn-0 read-only startup) is cleanly separated from Phase B (pre-commit gate and on-demand loads).
- [ ] **Canonical Turn-0 Receipt Contract**: Mandated table template is documented and matches in both `AGENTS.md` and `rules/constitution-first.md`.
- [ ] **Inspection Whitelist**: Read-only `git status`, `git branch`, `git rev-parse`, `ls` are explicitly permitted on Turn 0 in `rules/user-authorization-lock.md`.
- [ ] **Mutation Locks**: Shell mutations (`cp`, `mv`, `rm`, `tee`, `>`, `>>`) and `walkthrough.md` edits are explicitly locked prior to `PROCEED`.
- [ ] **Rules Bundle Parity & Size Budget**: `.pipeline/ACTIVE_RULES_BUNDLE.md` matches `rules/*.md` exactly, remaining <40,000 bytes and <750 lines.
- [ ] **Native Rust Tests**: `cargo test --workspace` passes with 0 failures, including new RED-to-GREEN regression assertions.
- [ ] **Baseline Gate**: `./target/release/verify-baseline . --no-domain` passes with 0 errors.
- [ ] **Zero Em Dashes**: Zero occurrences of Unicode `\u2014` across all modified files and commit logs.
