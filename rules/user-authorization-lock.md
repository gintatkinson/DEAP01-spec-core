<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: User Authorization Lock & Karpathy Compliance Check

**ALWAYS enforce:** The agent must run the 5-Point Thought-Block Compliance Check in every thought block and lock all modifying tools until authorized by the keyword `PROCEED`.

## Hard constraints

- The agent is strictly forbidden from invoking any file-writing tools (`write_to_file`, `replace_file_content`, `multi_replace_file_content`) or terminal execution tools (`run_command`) unless **BOTH** of the following hold:
  1. The specific file and its exact changes are documented in an **approved implementation plan** (see `.agents/AGENTS.md` § *Strict Planning Gate*), AND
  2. The user's latest message contains the word `PROCEED` (case-insensitive), or an equivalent explicit approval such as `Approved` or `Approve plan`.
- A keyword alone is **NOT** sufficient authorization. Writing the plan and stopping to await approval is itself permitted and required -- it is the one write that needs no prior plan.
- **Subagent Authorization**: To authorize spawned subagents to modify files or execute commands, the coordinator agent MUST append the keyword `PROCEED` (case-insensitive) to the end of the subagent's task prompt. Whichever tool the active runtime exposes for context-isolated subagent dispatch is locked when used to spawn modifying subagents (i.e. those with `PROCEED` in their task prompt), requiring the coordinator to verify that the user's latest message in the main chat explicitly contains the word `PROCEED` (case-insensitive) before dispatching. Concrete per-runtime tool names are listed only in `.agents/AGENTS.md` § *Mandatory Subagent Dispatch for Research, Specification & Implementation Loops*, so that a change of runtime cannot make this rule unexecutable (issue #312).
- Every agent thought block MUST begin with the standardized 5-Point Thought-Block Compliance Check:
  * Point 1 (`Turn-0 Initialization Gate`): Have I executed Phase A of the Mandatory Session Initialization Gate (`list_dir` or direct check to detect `.pipeline/upstream/`, `view_file` in full on `.pipeline/constitution.md` and `.pipeline/ACTIVE_RULES_BUNDLE.md`, and read-only `git status --short --branch`) AND emitted the Canonical Session Initialization Receipt Table in this session, OR does my subagent prompt end with `---GOVERNANCE-END---`? (Yes/No -- If No, HALT all other output, planning, and tool calls immediately, execute Phase A right now, and emit the Mandatory Session Initialization Completed table).
  * Point 2 (`Intent Classification`): Is the user's message a question/inquiry or a direct command?
  * Point 3 (`User Authorization Lock`): Are the exact target files and changes documented in an approved `implementation_plan.md` AND does the user's latest message explicitly contain `PROCEED` (or `Approved` / `Approve plan`)? (Yes/No -- If No, all modifying tools and mutating `run_command` invocations are strictly locked; writing/updating `implementation_plan.md` after Phase A initialization is the sole permitted file write).
  * Point 4 (`Zero Silent Assumptions`): Am I making any silent assumptions about the user's intent or workspace state?
  * Point 5 (`Coordinator Delegation & Shell Mutation Lock`): Does the active skill mandate context-isolated subagent dispatches, OR does this turn create, modify, copy, move, or delete any file other than `implementation_plan.md` (including `walkthrough.md`, governance files, rules, specs, or code)? (If Yes to either, coordinator direct file-writing and shell file mutations via `cp`, `mv`, `rm`, `tee`, `sed -i`, or output redirects `>`/`>>` are strictly locked; all repository writes must be delegated to single-item subagents after `PROCEED`).
- **Turn-0 Read-Only Workspace Inspection Whitelist**: During Turn-0 initialization and diagnostic turns, read-only inspection commands (`git status`, `git status --short --branch`, `git branch --show-current`, `git rev-parse --show-toplevel`, `git diff`, `ls`, `test -d`, `test -e`) are authorized without requiring prior `PROCEED`, specifically enabling empirical population of the `Branch` and `Working Tree` fields.
- **State-Mutating Shell Command Lock**: All file creation, copying, moving, deleting, or stream redirection commands (`cp`, `mv`, `rm`, `tee`, `sed -i`, `>`, `>>`) as well as build/test commands (`cargo`, `flutter`, `npm`, `git commit`, `git push`, `verify-baseline`) are strictly locked until an `implementation_plan.md` is approved with `PROCEED`.
- **Prohibition of Pre-Approval `walkthrough.md` Edits**: `walkthrough.md` is exclusively a post-implementation verification artifact and must never be created or edited prior to plan approval and subagent completion.
- **Scope of Coordinator Delegation & Point 5 (issue #312)**: The delegation duty binds for all repository source and specification writes, not only during named skill phases. Governance, tooling, rule, test and documentation repair are repository writes and are therefore in scope even when no skill is active and no skill names them. Reading point 5 narrowly -- "this is not skill execution, so the mandate does not apply" -- is the failure recorded in #312, where the coordinator wrote every file directly for an entire session. Per § *Precedence* below, where a narrow and a broad reading are both available, the strictest applies. This is a statement of scope, not a new obligation: it makes explicit which writes `rules/role-boundary-lock.md` § *Coordinator Direct Writing & Research Lock* already covered.
- **Prohibition of Unapproved Plan Abandonment & Mandatory Plan Continuity**: Once an implementation plan has been established, approved, or has in-flight work packages, the agent is strictly forbidden from abandoning, altering, replacing, or discarding the plan without explicit user review and authorization in the current conversation turn. The agent is strictly prohibited from reacting to user feedback, criticism, or diagnostic friction by unilaterally dropping planned work packages, abandoning uncommitted changes, or impulsively proposing wholesale file deletions, purges, or scope redirections. Any proposed deviation, scope refinement, or plan adjustment MUST be documented as an updated implementation plan and explicitly approved by the user before changing execution course.
- **Single-Task Atomic `implementation_plan.md` & Anti-Snowballing Invariant**:
  - `implementation_plan.md` MUST contain **at most 1 atomic work package** at any time. Dumping multi-wave issue backlogs into `implementation_plan.md` or appending/prepending a new user task onto a previous `implementation_plan.md` is strictly forbidden; backlog queues reside exclusively in the issue tracker (`gh issue list`).
  - `Prohibition of Unapproved Plan Abandonment & Mandatory Plan Continuity` applies strictly **within the currently active in-flight atomic task** (forbidding mid-task abandonment when hitting test/linter friction), and MUST NEVER be used as an excuse to retain completed or unrelated work packages inside `implementation_plan.md` when planning a new atomic task.
- **Downstream Binary-Only Distribution & Source Firewall Lock**:
  - Upstream Tier-0 compiler source code (`crates/{deap-core,ingest-sysml,compile-sysml,verify-baseline,reconcile-backlog,assemble-conops}`, `Cargo.toml`, `Cargo.lock`), `.pipeline/upstream/`, and `docs/architecture/blueprints/` reside **exclusively** in `DEAP01-spec-core` (`UPSTREAM_SPEC_CORE_COMPILER`).
  - Agents (and subagents) are strictly forbidden from proposing, planning (`implementation_plan.md`), or executing any copy, sync, or commit of Tier-0 compiler `crates/*` source code into any downstream repository (`DOWNSTREAM_CUSTOMER_PROJECT`), and strictly forbidden from running `cargo build` to recompile already-verified `target/release/*` binaries during downstream distribution.
  - Downstream distribution is restricted exclusively to: (1) pre-built `target/release/{compile-sysml,verify-baseline,ingest-sysml,reconcile-backlog,assemble-conops}` binary images, (2) `AGENTS.md` / `.agents/AGENTS.md`, (3) `.pipeline/constitution.md` / `.pipeline/ACTIVE_RULES_BUNDLE.md`, (4) `rules/*.md`, and (5) `skills/**`. Any subagent prompt requesting to copy `crates/*` to a downstream repository MUST immediately self-reject and halt.

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
