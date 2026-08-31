# Solution Walkthrough: Context Isolation and Skill Fidelity Rules

This document details the additions made to the project-scoped rules in order to enforce strict context isolation and skill fidelity, preventing cross-talk, memory leakage, and instruction drift among agents.

---

## 1. Code Realization Table

| Feature / Area | Source File | Lines Affected | Rationale |
| :--- | :--- | :--- | :--- |
| Project-Scoped Rules | [.agents/AGENTS.md](.agents/AGENTS.md) | 61-64 | Appended strict context isolation and skill fidelity guidelines to prevent cross-talk and summarization drift. |

---

## 2. Walkthrough of Changes

The project-scoped rules document guiding agent execution has been updated to include a dedicated section on context isolation and literal skill execution.

### 2.1. Rule Appendices in `.agents/AGENTS.md`
The following section was appended to the end of the file:
```markdown
## Strict Context Isolation & Skill Fidelity (No Cross-Talk)
- **No Cross-Talk / Memory Leakage**: You are strictly forbidden from reading, scanning, or referencing logs, transcripts, artifacts, or files belonging to other projects, folders, or conversation IDs (such as `3dgs-ion`, `3dgs-phoenix`, or other network models) stored under the App Data Directory (`~/.gemini/antigravity/brain/`). You must execute tasks strictly based on the inputs and schema files present in the *active* workspace.
- **Literal Skill Execution (No Summarization)**: When adopting a skill, you must read the skill's instructions in full and adhere to them literally. You are strictly forbidden from summarizing, truncating, or using abbreviated interpretations of instructions.
```

---

## 3. Technical Rationale

1. **Strict Directory Boundaries / Preventing Cross-Talk**:
   In complex environments with multiple active projects and subagents, downstream agents could inadvertently read transcripts, logs, or artifacts belonging to unrelated workspaces (such as `3dgs-ion` or `3dgs-phoenix`). The new context isolation rule establishes explicit directory boundaries, prohibiting access to files outside the active workspace under the App Data Directory.

2. **Ensuring Skill Fidelity / Resolving Instruction Drift**:
   When adopting specialized skills, agents must execute instructions with high fidelity. Summarizing, truncating, or abbreviating skill files results in documentation and behavioral drift. The added instruction mandates literal execution, ensuring that full instructions are read and followed without omission.

---

## 4. Verification

1. Verified that the modifications were appended correctly to [.agents/AGENTS.md](.agents/AGENTS.md).
2. Verified that the walkthrough file `docs/designs/feat-fix-isolation-rules.md` conforms to repository documentation conventions.
