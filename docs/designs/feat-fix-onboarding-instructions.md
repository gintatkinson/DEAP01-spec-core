# Solution Walkthrough: Onboarding Instructions Update

This document details the update applied to the onboarding instructions in the feature-driven-implementation skill and explains the technical rationale for this adjustment.

---

## 1. Code Realization Table

| Feature / Area | Source File | Lines Affected | Rationale |
| :--- | :--- | :--- | :--- |
| Workspace Feature Skill | [SKILL.md](skills/feature-driven-implementation/SKILL.md) | 42 | Replace stale reference to `bootstrap_downstream.py` with native GitHub template instructions |
| Agent System Skill Copy | [SKILL.md copy](.agents/skills/feature-driven-implementation/SKILL.md) | 42 | Replaced via shared symlink alignment |

---

## 2. Walkthrough of Changes

The onboarding instructions in the workspace's feature-driven-implementation skill guide agents and developers on how to bootstrap a downstream workspace before beginning feature implementation.

### 2.1. Feature Skill (`skills/feature-driven-implementation/SKILL.md`)
Previously, step 2 of the Backlog & Dependency Mapping process instructs:
```markdown
Ensure that the downstream workspace has been bootstrapped using the upstream-only `bootstrap_downstream.py` script. Note that this is an upstream-only tool and must be executed from the upstream repository directory (`<upstream_workspace_path>`) targeting the downstream workspace path BEFORE the downstream agent begins work (use the `--no-domain` flag with the bootstrap script if implementing a different project domain).
```
This has been updated to align with the native GitHub template onboarding architecture:
```markdown
Ensure that the downstream workspace has been bootstrapped using the native GitHub template onboarding workflow: `gh repo create <new_app_name> --template gintatkinson/DEAP-implementation-driver --public --clone`. This creates a fresh repository on GitHub and clones it locally BEFORE the downstream agent begins work.
```

### 2.2. Agent System Copy (`.agents/skills/feature-driven-implementation/SKILL.md`)
Since `.agents/skills` is a tracked symlink pointing directly to `skills`, the updates applied to `skills/feature-driven-implementation/SKILL.md` are automatically in effect for `.agents/skills/feature-driven-implementation/SKILL.md`, preventing any documentation drift.

---

## 3. Technical Rationale

1. **Eliminating Stale/Deleted References**:
   The `bootstrap_downstream.py` script was previously removed from the repository. Continuing to reference it in onboarding skills caused confusion and execution errors for agents or developers attempting to follow the onboarding guides.

2. **Native Template Integration**:
   By switching the instructions to use `gh repo create <new_app_name> --template gintatkinson/DEAP-implementation-driver --public --clone`, we leverage GitHub's native template repository capabilities. This offers a robust, officially supported, and platform-standard mechanism to bootstrap new projects from the repository template without maintaining custom scripts.

3. **Prevention of Documentation Drift**:
   Using a symlink for `.agents/skills` ensures that all agent-scoped skills remain perfectly synchronized with the workspace-scoped skills without requiring duplicate manual edits.

---

## 4. Verification

1. Verified that changes to `skills/feature-driven-implementation/SKILL.md` were correctly applied.
2. Verified that `.agents/skills/feature-driven-implementation/SKILL.md` references the same updated content due to the symlink.
3. Verified the document format and links conform to repository conventions.
