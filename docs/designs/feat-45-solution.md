# Solution Walkthrough: Feature 45 YANG Schema Decomposition Heuristics

This document describes the design, architectural components, code realization, and testing plan for the YANG Schema Decomposition Heuristics (Feature 45).

---

## 1. Overview of Changes

To transition from naive schema parsing to a disciplined, domain-driven decomposition approach, Feature 45 introduces a series of AST-driven partitioning heuristics to govern backlog generation:

### 1.1. AST-Driven Decomposition Heuristics
- **Module Categorization**: Instead of treating all schemas equally, the pipeline now categorizes schemas into **utility modules** (which contain only helper typedefs, identities, and reusable type definitions) and **functional modules** (which model operational/state-bearing systems).
- **Utility Bypassing**: When a module is classified as a utility module, it is cataloged into the `SharedTypeRegistry` without spawning any new Epics or Features.
- **Complexity-Based Splitting**: Functional modules are split into Epics based on leaf count and tree depth constraints. Any module exceeding 40 leaves or a maximum depth of 3 triggers Epic-level partitioning.

### 1.2. DDD Bounded Context Mapping
- Each identified functional subtree is treated as a distinct **DDD Bounded Context**. 
- Subtrees are traversed to calculate their **Structural Weight (SW)**:
  `SW = leafCount + (depth * 2) + choicePenalty(5)`
- Subtrees whose Structural Weight exceeds `20.0` are isolated into separate Feature boundaries.
- If an Epic grows to exceed `15` Features, the decompiler forces an Epic split to preserve backlog readability and context isolation.

---

## 2. Code Realization Table

The following table maps the logical UML model elements of the decompiler to their respective codebase implementation paths:

| UML Element | Realization Tag | File Path | Properties & Realized Behavior |
| :--- | :--- | :--- | :--- |
| `YangDecompiler` | `@realizes UML::YangDecompiler` | [SKILL.md](skills/schema-specification-engineering/SKILL.md) | Standard parser logic adopted by the LLM subagent to ingest and categorize YANG schemas. |
| `DecompositionEngine` | `@realizes UML::DecompositionEngine` | [SKILL.md](skills/schema-specification-engineering/SKILL.md) | Core heuristic engine evaluating Bounded Contexts, Structural Weight calculations, and Epic-to-Feature boundaries. |
| `SharedTypeRegistry` | `@realizes UML::SharedTypeRegistry` | [SKILL.md](skills/schema-specification-engineering/SKILL.md) | Cataloging protocol for helper/utility data types, ensuring metadata reuse without backlog inflation. |

---

## 3. Verification & Testing Plan

### 3.1. Model Coverage Validation
To verify that the generated specifications satisfy the linter's coverage and semantic rules, run the spec verification gate:
```bash
python3 skills/spec-orchestrator/scripts/verify_model_coverage.py --spec-only
```
- **Acceptance Criteria**: The script must parse the updated feature specs under `docs/features/` and exit with code `0`, confirming that no syntax errors, unclosed Mermaid fences, or invalid types are present.

### 3.2. Automated Testing Matrix
Execute the standard test suite to verify decompiler logic:
```bash
python3 -m pytest tests/
```
- **Test cases to validate**:
  1. `test_utility_module_suppression`: Asserts that modules with only helper types are registered in `SharedTypeRegistry` and generate `0` Epics.
  2. `test_small_module_single_epic`: Asserts that modules with <= 40 leaves map to exactly 1 Epic.
  3. `test_massive_module_decomposition`: Asserts that modules with > 40 leaves or depth > 3 generate multiple Epics.
  4. `test_structural_weight_partitioning`: Asserts that subtrees exceeding weight 20 trigger separate Feature boundaries.
