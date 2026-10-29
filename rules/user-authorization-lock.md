<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: User Authorization Lock & Pre-Action Compliance Rules

**ALWAYS enforce:** Lock all modifying tools until authorized by an approved `implementation_plan.md` and the keyword `PROCEED`, and focus thought blocks directly on technical reasoning without repeating initialization or compliance checklists.

## Hard constraints

- The agent is strictly forbidden from invoking any file-writing tools (`write_to_file`, `replace_file_content`, `multi_replace_file_content`) or terminal execution tools (`run_command`) unless **BOTH** of the following hold:
  1. The specific file and its exact changes are documented in an **approved implementation plan** (see `.agents/AGENTS.md` § *Strict Planning Gate*), AND
  2. The user's latest message contains the word `PROCEED` (case-insensitive), or an equivalent explicit approval such as `Approved` or `Approve plan`.
- A keyword alone is **NOT** sufficient authorization. Writing the plan and stopping to await approval is itself permitted and required -- it is the one write that needs no prior plan.
- **Subagent Authorization**: To authorize spawned subagents to modify files or execute commands, the coordinator agent MUST append the keyword `PROCEED` (case-insensitive) to the end of the subagent's task prompt. Whichever tool the active runtime exposes for context-isolated subagent dispatch is locked when used to spawn modifying subagents (i.e. those with `PROCEED` in their task prompt), requiring the coordinator to verify that the user's latest message in the main chat explicitly contains the word `PROCEED` (case-insensitive) before dispatching. Concrete per-runtime tool names are listed only in `.agents/AGENTS.md` § *Mandatory Subagent Dispatch for Research, Specification & Implementation Loops*, so that a change of runtime cannot make this rule unexecutable (issue #312).
- Thought blocks MUST focus directly on the active user request and technical analysis -- prefixing thought blocks with repetitive initialization or compliance checklists is strictly prohibited. Before invoking any modifying tool or mutating command, enforce these 4 pre-action locks:
  * `Intent Classification`: Distinguish questions/inquiries (read-only response only) from direct modification commands.
  * `User Authorization Lock`: Modifying tools and mutating `run_command` invocations are strictly locked unless the exact target files and changes are documented in an approved `implementation_plan.md` (stored per-conversation at `<appDataDir>/brain/<conversation-id>/implementation_plan.md` or workstream-scoped `.pipeline/plans/<slug>.md` so parallel tabs never clobber `./implementation_plan.md`) AND the user explicitly approves (`PROCEED` / `Approved` / `Approve plan`). Writing this conversation's `implementation_plan.md` is the sole permitted pre-approval file write.
  * `Zero Silent Assumptions`: Surface ambiguities and verify workspace state empirically rather than making silent assumptions.
  * `Coordinator Delegation & Shell Mutation Lock`: Whenever the active skill mandates context-isolated subagent dispatches OR a turn creates, modifies, copies, moves, or deletes any repository file (governance, rules, specs, or code), coordinator direct repository file writes and shell file mutations (`cp`, `mv`, `rm`, `tee`, `sed -i`, `>`, `>>`) are strictly locked; all repository writes must be delegated to single-item subagents after `PROCEED`.
- **Read-Only Workspace Inspection Whitelist**: During diagnostic and pre-plan turns, read-only inspection commands (`git status`, `git status --short --branch`, `git branch --show-current`, `git rev-parse --show-toplevel`, `git diff`, `ls`, `test -d`, `test -e`) are authorized without requiring prior `PROCEED`.
- **State-Mutating Shell Command Lock**: All file creation, copying, moving, deleting, or stream redirection commands (`cp`, `mv`, `rm`, `tee`, `sed -i`, `>`, `>>`) as well as build/test commands (`cargo`, `flutter`, `npm`, `git commit`, `git push`, `verify-baseline`) are strictly locked until an `implementation_plan.md` is approved with `PROCEED`.
- **Prohibition of Pre-Approval `walkthrough.md` Edits**: `walkthrough.md` is exclusively a post-implementation verification artifact and must never be created or edited prior to plan approval and subagent completion.
- **Scope of Coordinator Delegation & Point 5 (issue #312)**: The delegation duty binds for all repository source and specification writes, not only during named skill phases. Governance, tooling, rule, test and documentation repair are repository writes and are therefore in scope even when no skill is active and no skill names them. Reading point 5 narrowly -- "this is not skill execution, so the mandate does not apply" -- is the failure recorded in #312, where the coordinator wrote every file directly for an entire session. Per § *Precedence* below, where a narrow and a broad reading are both available, the strictest applies. This is a statement of scope, not a new obligation: it makes explicit which writes `rules/role-boundary-lock.md` § *Coordinator Direct Writing & Research Lock* already covered.
- **Prohibition of Unapproved Plan Abandonment & Mandatory Plan Continuity**: Once an implementation plan has been established, approved, or has in-flight work packages, the agent is strictly forbidden from abandoning, altering, replacing, or discarding the plan without explicit user review and authorization in the current conversation turn. The agent is strictly prohibited from reacting to user feedback, criticism, or diagnostic friction by unilaterally dropping planned work packages, abandoning uncommitted changes, or impulsively proposing wholesale file deletions, purges, or scope redirections. Any proposed deviation, scope refinement, or plan adjustment MUST be documented as an updated implementation plan and explicitly approved by the user before changing execution course.
- **Per-Conversation Plan Isolation & Single-Task Atomic Invariant**:
  - Session `implementation_plan.md` and post-completion `walkthrough.md` reside in `<appDataDir>/brain/<conversation-id>/` (or workstream-scoped `.pipeline/plans/<slug>.md` when Git persistence is requested); overwriting a root `./implementation_plan.md` belonging to another active parallel workstream is strictly forbidden.
  - Each `implementation_plan.md` MUST contain **at most 1 atomic work package** at any time (backlog queues reside exclusively in `gh issue list`; never append new tasks onto completed plans).
- **Parallel Workstream Git Staging & Worktree Safety**:
  - Always stage and commit via explicit target paths (`git add -- <target-files>` and `git commit --only -- <target-files>`). Unscoped `git add .`, `git add -A`, `git commit -a`, `git checkout .`, `git restore .`, and `git stash` are strictly forbidden when concurrent workstream files exist.
- **Downstream Binary-Only Distribution & Black-Box Source Firewall Lock**:
  - Upstream Tier-0 compiler source code (`crates/{deap-core,ingest-sysml,compile-sysml,verify-baseline,reconcile-backlog,assemble-conops}`, `Cargo.toml`, `Cargo.lock`), `.pipeline/upstream/`, and `docs/architecture/blueprints/` reside **exclusively** in `DEAP01-spec-core` (`UPSTREAM_SPEC_CORE_COMPILER`).
  - Agents (and subagents) are strictly forbidden from proposing, planning (`implementation_plan.md`), or executing any copy, sync, or commit of Tier-0 compiler `crates/*` source code into any downstream repository (`DOWNSTREAM_CUSTOMER_PROJECT`), and strictly forbidden from running `cargo build` to recompile already-verified `target/release/*` binaries during downstream distribution.
  - Downstream distribution is restricted exclusively to: (1) pre-built `target/release/{compile-sysml,verify-baseline,ingest-sysml,reconcile-backlog,assemble-conops}` binary images, (2) `AGENTS.md` / `.agents/AGENTS.md`, (3) `.pipeline/constitution.md` / `.pipeline/ACTIVE_RULES_BUNDLE.md`, (4) `rules/*.md`, and (5) `skills/**`. Any subagent prompt requesting to copy `crates/*` to a downstream repository MUST immediately self-reject and halt.
  - **Black-Box Binary Firewall**: In `DOWNSTREAM_CUSTOMER_PROJECT` sessions, reading or searching `../DEAP01-spec-core/crates/*` to inspect compiler internals is strictly forbidden; treat Tier-0 binaries as black-box verifiers and report upstream defects via `adversarial-code-auditor`.

## Precedence

This rule is one of three co-normative statements of the same constraint:

| Statement | Location |
| --- | --- |
| Strict Planning Gate | `.agents/AGENTS.md` § *Strict Planning Gate* |
| Strict Planning Mode Gate | `.pipeline/constitution.md` § *Strict Planning Mode Gate* |
| User Authorization Lock | this file |

**Where they appear to differ, the STRICTEST reading applies.** Under-authorizing costs
one redundant question; over-authorizing causes unapproved writes. As of issue #295 the
strictest reading is `.agents/AGENTS.md` § *Strict Planning Gate*: an approved plan is
required and an authorization keyword alone is explicitly insufficient.

`.agents/AGENTS.md` MUST be read before any file write. It is listed as required read #1
in `rules/constitution-first.md`. Because `.agents/` is hidden, glob and ripgrep index
queries skip it, so it must be read directly by path.

## Mandatory Empirical Physical Path Verification Gate

Before transitioning any issue or task status to `status:fixed-resolved`, the agent is strictly mandated to execute empirical physical path verification using `git status` and `git rev-parse --show-toplevel` on the target workstation directory. Optimism bias or asserting resolution without empirical physical path verification is strictly prohibited.

## Anti-Symptom Patching & Structural Remediation Mandate

Agents are strictly required to prioritize Structural Remediation Over Mechanical Replacement. Mechanical string replacements or surface-level patches that ignore underlying system invariants are strictly prohibited. Every remediation must include Mandatory Invariant Verification to guarantee that structural integrity is maintained.

## Why

To prevent the agent from making silent assumptions, performing unapproved actions, or violating the Karpathy guidelines in response to diagnostic or analytical questions.

Issue #295 records the failure this prevents: an agent read `rules/` and the constitution,
both of which stated that a keyword sufficed, never read `.agents/AGENTS.md`, and pushed
two unapproved commits to `main`.
