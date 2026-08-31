# Solution Walkthrough: Sanitization of Hardcoded Developer Paths

This document details the changes made to sanitize hardcoded developer-specific absolute paths within the repository's skill definitions, ensuring portability and correct execution across downstream workspace environments.

## 1. Overview of Changes

To allow downstream projects to install and utilize these agent skills without inheriting developer-specific absolute path configurations, we have replaced hardcoded paths with environment-agnostic placeholders.

### Modified Files

* **`skills/debug-protocol/SKILL.md`**
  - Replaced the hardcoded path `<workstation_path>/DEAP-spec-core` with `<absolute_workspace_path>`.
* **`skills/feature-driven-implementation/SKILL.md`**
  - Replaced the hardcoded path `<workstation_path>/DEAP-spec-core` with `<upstream_workspace_path>`.

---

## 2. Rationale

1. **Downstream Portability**:
   Hardcoded developer absolute paths (e.g., pointing to `<workstation_path>/DEAP-spec-core`) prevent other users or automated pipelines from running these skills directly without manual edits. Using placeholder variables like `<absolute_workspace_path>` and `<upstream_workspace_path>` allows downstream agents and tooling to dynamically resolve the appropriate path at runtime.

2. **Upstream vs. Downstream Workspace Isolation**:
   In multi-project setups (such as when working on downstream workspace integrations), the upstream-only tools (e.g., `bootstrap_downstream.py`) must be run specifically from the upstream directory (`<upstream_workspace_path>`). Using explicit, distinct placeholders prevents confusion about which workspace a path refers to.

---

## 3. Detailed Diff Walkthrough

### `skills/debug-protocol/SKILL.md`
```diff
 2. **File permissions**: Request write permission for the workspace root to ensure no write permission failures occur during edits:
-   - `<workstation_path>/DEAP-spec-core` (or local equivalent)
+   - `<absolute_workspace_path>` (or local equivalent)
```

### `skills/feature-driven-implementation/SKILL.md`
```diff
-Ensure that the downstream workspace has been bootstrapped using the upstream-only `bootstrap_downstream.py` script. Note that this is an upstream-only tool and must be executed from the upstream repository directory (`<workstation_path>/DEAP-spec-core`) targeting the downstream workspace path BEFORE the downstream agent begins work
+Ensure that the downstream workspace has been bootstrapped using the upstream-only `bootstrap_downstream.py` script. Note that this is an upstream-only tool and must be executed from the upstream repository directory (`<upstream_workspace_path>`) targeting the downstream workspace path BEFORE the downstream agent begins work
```
