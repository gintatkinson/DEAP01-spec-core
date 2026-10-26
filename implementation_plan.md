# Implementation Plan: Enforce Subagent Target-File Read Lock & Eliminate Fake-Work `SKILL.md` Reads on Surgical Edits

## Objective
Eliminate subagent token waste and unauthorized file scanning by enforcing two hard constraints across [AGENTS.md](file:///Users/perkunas/jail/DEAP01-spec-core/AGENTS.md), [rules/subagent-dispatch-standards.md](file:///Users/perkunas/jail/DEAP01-spec-core/rules/subagent-dispatch-standards.md), and [.pipeline/ACTIVE_RULES_BUNDLE.md](file:///Users/perkunas/jail/DEAP01-spec-core/.pipeline/ACTIVE_RULES_BUNDLE.md):
1. **Strict Target-File Read Lock (Zero Unrelated File Reads)**: Subagents are strictly forbidden from reading, searching, or inspecting any file outside the explicit target file list in their dispatch prompt (e.g., never scanning `crates/verify-baseline/*` during a governance or specification edit), and single-item edit subagents must never redundantly run `verify-baseline`.
2. **Surgical Edit Fast-Path (No Fake-Work `SKILL.md` Reads)**: Reading a `SKILL.md` file applies strictly to pipeline skill workflows (specification synthesis, TDD feature implementation, `debug-protocol`, `adversarial-code-auditor`). When applying literal `Target`/`Replacement` edits already approved in `implementation_plan.md`, forcing a subagent to read any `SKILL.md` is prohibited fake work — the subagent must apply the exact edits to the listed target files and exit immediately.

## Target Files & Exact Changes

1. **[AGENTS.md](file:///Users/perkunas/jail/DEAP01-spec-core/AGENTS.md)** (`## Mandatory Subagent Dispatch for Research, Specification & Implementation Loops`):
   - Scope Item 4 (`Mandatory Skill-Reading Instruction`) so `view_file` on `SKILL.md` is required only for pipeline skill workflows and explicitly forbidden as fake work for surgical edits whose exact changes are already approved in `implementation_plan.md`.
   - Add Item 10 (`Strict Target-File Read & Verification Boundary`): Subagents are strictly forbidden from reading, searching, or inspecting any file outside the explicit target files listed in their prompt, and must not run `verify-baseline` inside single-item edit dispatches.
2. **[rules/subagent-dispatch-standards.md](file:///Users/perkunas/jail/DEAP01-spec-core/rules/subagent-dispatch-standards.md)** (`### 2. Mandatory Skill Direct-Path Read Directive` & new `### 10. Strict Target-File Read Lock & Surgical Edit Fast-Path`):
   - Define the **Surgical Edit Fast-Path** (no `SKILL.md` read when exact `Target`/`Replacement` edits are already approved in `implementation_plan.md`) and the **Strict Target-File Read Lock** (zero reads outside the explicit target file list; zero redundant `verify-baseline` invocations inside single-item subagents).
3. **[.pipeline/ACTIVE_RULES_BUNDLE.md](file:///Users/perkunas/jail/DEAP01-spec-core/.pipeline/ACTIVE_RULES_BUNDLE.md)** (`## 14. Rule: Subagent Dispatch Standards`):
   - Synchronize the `Surgical Edit Fast-Path` and `Strict Target-File Read Lock` bullets.

## Verification & Completion Criteria
1. Single-item subagent applies ONLY the 3 file edits above using `edit_file` (reading zero other files and zero `SKILL.md` files) and exits immediately.
2. Coordinator runs `./target/release/verify-baseline . --no-domain` once and verifies `exit code 0`.
3. Commit and push to `origin/main` and verify `git diff origin/main` is empty.
