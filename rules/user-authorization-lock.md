<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: User Authorization Lock & Karpathy Compliance Check

**ALWAYS enforce:** The agent must run the 4-point Karpathy and Pipeline Compliance Check in every thought block and lock all modifying tools until authorized by the keyword `PROCEED`.

## Hard constraints

- The agent is strictly forbidden from invoking any file-writing tools (`write_to_file`, `replace_file_content`, `multi_replace_file_content`) or terminal execution tools (`run_command`) unless **BOTH** of the following hold:
  1. The specific file and its exact changes are documented in an **approved implementation plan** (see `.agents/AGENTS.md` § *Strict Planning Gate*), AND
  2. The user's latest message contains the word `PROCEED` (case-insensitive), or an equivalent explicit approval such as `Approved` or `Approve plan`.
- A keyword alone is **NOT** sufficient authorization. Writing the plan and stopping to await approval is itself permitted and required — it is the one write that needs no prior plan.
- **Subagent Authorization**: To authorize spawned subagents to modify files or execute commands, the coordinator agent MUST append the keyword `PROCEED` (case-insensitive) to the end of the subagent's task prompt. Whichever tool the active runtime exposes for context-isolated subagent dispatch is locked when used to spawn modifying subagents (i.e. those with `PROCEED` in their task prompt), requiring the coordinator to verify that the user's latest message in the main chat explicitly contains the word `PROCEED` (case-insensitive) before dispatching. Concrete per-runtime tool names are listed only in `.agents/AGENTS.md` § *Mandatory Subagent Dispatch for Research, Specification & Implementation Loops*, so that a change of runtime cannot make this rule unexecutable (issue #312).
- Every agent thought block MUST begin with the 4-point Karpathy and Pipeline Compliance Check:
  * Is the user's message a question/inquiry or a direct command?
  * Has the user explicitly approved a file-write/command execution for this turn? (Yes/No)
  * Am I making any silent assumptions about the user's intent?
  * Does the active skill mandate context-isolated subagent dispatches, **or** does this turn write any repository source or specification file? (If yes to either, coordinator direct file-writing is locked).
- **Scope of point 4 (issue #312).** The delegation duty binds for all repository source and specification writes, not only during named skill phases. Governance, tooling, rule, test and documentation repair are repository writes and are therefore in scope even when no skill is active and no skill names them. Reading point 4 narrowly — "this is not skill execution, so the mandate does not apply" — is the failure recorded in #312, where the coordinator wrote every file directly for an entire session. Per § *Precedence* below, where a narrow and a broad reading are both available, the strictest applies. This is a statement of scope, not a new obligation: it makes explicit which writes `rules/role-boundary-lock.md` § *Coordinator Direct Writing & Research Lock* already covered.

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
