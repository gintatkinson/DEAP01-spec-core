<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Subagent Dispatch Standards & Self-Rejection Pre-Flight Gate

**ALWAYS enforce:** All context-isolated subagent dispatches must adhere strictly to canonical prompt payload standards, single-item micro-task scoping, direct-path skill reading, and the mandatory subagent self-rejection pre-flight gate.

## Hard Constraints

### 1. Verbatim User Payload Mandate
The subagent prompt payload MUST contain the user's exact instructions verbatim. The coordinator is strictly prohibited from altering, summarizing, or wrapping the user's prompt with synthetic boilerplate.

### 2. Autonomous On-Disk Governance Initialization
Every subagent must autonomously execute `view_file` on `.pipeline/constitution.md` and `.pipeline/ACTIVE_RULES_BUNDLE.md` as its first step, without requiring prompt modification by the coordinator.

### 3. Single-Prompt Serial Dispatch Rule
Coordinators must dispatch exactly ONE subagent with ONE prompt at a time. Multi-agent concurrent dispatches are prohibited unless explicitly structured and approved by the user.

### 4. Mandatory Teamwork Preview Gate
When `/teamwork-preview` is invoked, the coordinator MUST present the full execution plan and verbatim prompt in the chat, halt, and await human approval before dispatching any subagent.

### 5. Mandatory Single-Item Micro-Task Scope
Every subagent dispatch prompt MUST target at most 1 specification item (max 1 Epic, 1 Feature, 1 User Story, or 1 Use Case) or 1 single bounded micro-task (2-5 minutes). Multi-item or batch execution prompts are strictly prohibited.

### 6. Mandatory Defect Filing Directives
Prompts launching subagents, auditors, or validators MUST explicitly instruct the subagent to file defects via both `gh issue create` (GitHub) and `glab issue create` (GitLab). Issue auto-closing keywords or issue close commands (`gh issue close`, `glab issue close`) in prompts or commit messages are strictly forbidden.

### 7. Trailing Authorization Token
The coordinator MUST append the authorization token `PROCEED` (case-insensitive) to the end of the subagent prompt payload to authorize tool execution in the subagent's isolated context.

## Subagent Pre-Flight Self-Rejection Protocol

When initialized, the subagent MUST parse and validate the prompt against the following checklist:

| Check | Requirement | Failure Action |
| --- | --- | --- |
| 1. Governance Read | Autonomously executes `view_file` on `.pipeline/constitution.md` and `.pipeline/ACTIVE_RULES_BUNDLE.md` | Halt immediately and report error |
| 2. Untruncated Payload | Zero `[...]`, `[summarized]`, or `[truncated]` synthetic markers | Emit `ERROR: Prompt rejected: prompt truncation/summarization detected` and HALT |
| 3. Single-Item Scope | Max 1 Epic, Feature, Story, Use Case, or bounded micro-task | Emit `ERROR: Prompt rejected: micro-task scope violation` and HALT |
| 4. Authorization | Contains `PROCEED` token | Emit `ERROR: Prompt rejected: missing authorization token` and HALT |

## Why

Prompt summarization and preamble degradation lead to unverified assumptions, lost quality gates, and drift from canonical engineering standards. Enforcing self-rejection at the subagent pre-flight level ensures that no subagent can be coerced or accidentally steered into bypassing repository invariants.
