<!-- Copyright 2026. All rights reserved. -->

---
name: debug-protocol
description: "8-step Recursive Debugging Protocol for systematic bug hunting. BUG ISSUES ONLY — do NOT use for features, enhancements, epics, chores, or refactors. Use when fixing defects to follow a rigorous hypothesis-driven loop with dedicated subagents per step."
compatibility: "Works with any agent runtime. Requires GitHub CLI (`gh`) for issue management."
metadata:
  title: "Recursive Debugging Protocol (8-Step Bug Loop)"
  category: debugging
  risk: low
---

> **⚠️ THIS PROTOCOL IS FOR BUGS AND DEFECTS ONLY**
>
> If the task is a feature request, enhancement, epic, chore, refactor, or new functionality — **DO NOT USE THIS PROTOCOL.** Stop immediately and report back.
>
> **Only use this when there is a clear defect:** something is broken, incorrect, or behaving unexpectedly compared to specification.

# Recursive Debugging Protocol

## Step 0 — Verify: Is this a bug?

Before starting, confirm:
- Is there existing behavior that is wrong? (bug)
- Or is this adding new behavior that doesn't exist yet? (feature)

If bug — proceed to Step 0.1.

### Step 0.1 — Pre-flight: Unattended Setup
To prevent the user from being interrupted by endless permission prompts, the executing agent MUST immediately request the following permissions at the start of the task using the `ask_permission` tool:
1. **Command prefixes**: Request permission for the following command prefixes to enable unattended git/gh/flutter operations:
   - `git`
   - `gh`
   - `flutter`
2. **File permissions**: Request write permission for the workspace root to ensure no write permission failures occur during edits:
   - `<absolute_workspace_path>` (or local equivalent)

Once these permissions are requested and approved by the user, proceed to Step 1.

## Step 1 — Reproduction Subagent
Dispatch a subagent to: Gather complete symptom info, reproduce the bug consistently, determine scope (isolated or systemic), and check environment (version, platform). Return reproduction steps and scope report.

## Step 2 — Hypothesis Subagent
Dispatch a subagent to: Generate multiple hypotheses ranked by likelihood. Consider recent changes, data/state issues, race conditions, edge cases, interaction effects. Return a ranked list of hypotheses.

## Step 3 — Investigation Subagent
Dispatch a subagent to: Binary-search the problem space. Add strategic logging at key decision points. Trace data flow from input to output. Verify ALL assumptions — do not assume. Return evidence of what was tried and observed.

## Step 4 — Evidence Subagent
Dispatch a subagent to: Document all evidence, code snippets, logs, error messages, patterns. Track which hypotheses have been ruled out and why. Return a structured evidence dossier.

## Step 5 — Root Cause Subagent
Dispatch a subagent to: Distinguish root cause from symptoms. Apply "5 whys" to drill to the actual cause. Verify the root cause explains ALL observed symptoms. Return root cause with file:line references.

## Step 6 — Fix Subagent
Dispatch a subagent to: Design and implement the minimal fix. Consider side effects. Add regression tests. Document the fix. Stage, commit, and push all changes to the remote repository. Update the GitHub issue with root cause and fix details. Return fix summary and issue URL.

## Step 7 — Verification Subagent
Dispatch a subagent to:
1. Confirm bug is fixed using original reproduction steps from Step 1.
2. Grep the fix location (FILE_LOCATION from issue body) and confirm the fix code is present.
3. Run the full test suite and paste raw terminal output.
4. Show `git diff` of the fix commit to confirm only expected changes.
5. If all three proofs pass, comment on the GitHub issue with the evidence and apply the `status:fixed-resolved` label. Leave the issue open — `Closed` requires Product Owner validation and is unreachable by an agent (`.pipeline/constitution.md:161`).
Return: grep output, raw test output, git diff output. Do NOT return a pass/fail summary without evidence.

## Step 8 — Loop Decision
If Step 7 failed, return to Step 1. Do NOT give up after one or two failed hypotheses. If stuck, reconsider assumptions.

If the issue is a meta-issue with multiple independent sub-items (e.g. "eliminate all hardcoded data" with 14 items), treat each sub-item as one pass through Steps 1-7. After Step 7 passes for the current sub-item, return to Step 1 for the next sub-item. Do NOT stop to ask, report, or plan — just loop.

On completion of the current bug, query the repository for the next unresolved bug/defect issue using `gh issue list --label bug --search '-label:"status:fixed-resolved"'`.

> [!IMPORTANT]
> The `-label:"status:fixed-resolved"` exclusion is load-bearing, not cosmetic. Agents
> may not close issues, so a finished bug stays **open** and keeps its `bug` label. It
> also still passes the Step 0 defect gate, because it genuinely is a defect. Selecting
> on `--label bug` alone therefore reselects work that is already done, forever. The
> resolved label is what removes an item from the selection set now that closing cannot.
- If other unresolved bugs exist:
  1. Select the next highest priority or oldest bug.
  2. Skip any issues that are already assigned to someone else or explicitly marked as in-progress.
  3. Start a new Step 1-7 debugging loop on that bug.
- **Reclassification (MANDATORY — see issue #287):** If the Step 0 gate determines the selected issue is NOT a defect (feature request, enhancement, epic, chore, refactor, or documentation change), do **NOT** halt the loop. Instead:
  1. Post a comment on the issue explaining the reclassification and why the protocol does not apply.
  2. Run `gh issue edit <ID> --remove-label bug --add-label enhancement`.
  3. Continue to the next item in the backlog.

  Halting here would be unsatisfiable. The terminating condition below cannot be met while a non-defect still carries the `bug` label, and the 3-iteration skip does not rescue the loop because Step 0 halts **before iteration one** — there is no runtime symptom for the Step 1 Reproduction Subagent to reproduce. Reclassifying removes the item from the selection set, which is the only exit that both respects Step 0 and allows the loop to finish.
- If a bug cannot be resolved or reproduced after 3 full hypothesis/fix iterations (Steps 1-7), post a detailed status comment summarizing the reproduction/investigation findings on the issue, skip it, and proceed to the next unresolved bug in the backlog.
- Do NOT stop until no issue in the selection set above — open, labelled `bug`, and NOT labelled `status:fixed-resolved` — remains that passes the **Step 0 defect gate**. Two things leave the selection set: reclassification removes the `bug` label from non-defects, and reaching `Fixed / Resolved` adds `status:fixed-resolved` to genuine defects. Neither requires closing an issue.

## Persistence Rules
- **Subagent Self-Reading Mandate**: Every subagent dispatched under this protocol MUST use the file-reading tool to read `skills/debug-protocol/SKILL.md` in full as its VERY FIRST action. It must then follow the output format, evidence standards, and closing procedures defined in this file exactly. Do NOT rely on the coordinator's abbreviated step summary.
- Each step MUST use a fresh subagent — do not reuse or combine
- Do NOT skip or combine steps
- Document every attempt even if the bug isn't fully solved
- If a subagent fails to complete its step, dispatch another with more specific instructions

## Debugging Checklist
- [ ] Step 0: Confirmed this is a bug (not a feature)
- [ ] Step 1 subagent dispatched and reported
- [ ] Step 2 subagent dispatched and reported
- [ ] Step 3 subagent dispatched and reported
- [ ] Step 4 subagent dispatched and reported
- [ ] Step 5 subagent dispatched and reported
- [ ] Step 6 subagent dispatched, fix applied, changes committed and pushed, issue updated
- [ ] Step 7: Verification subagent dispatched, three proofs validated, issue marked `Fixed / Resolved` with mechanical proof
- [ ] Loop closed (bug fixed) or loop restarted (bug persists)
