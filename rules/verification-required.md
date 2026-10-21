<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Verification-Before-Completion

<a id="rule-verification-required"></a>
## 20. Rule: Verification Required (`rules/verification-required.md`)

**TRIGGER SCOPE -- PRE-COMMIT / POST-EDIT GATE ONLY (NEVER ON INIT OR CONVERSATIONAL TURNS):**
- **FORBIDDEN on Initialization & Read-Only Turns**: NEVER run `cargo test`, `flutter test`, or `verify-baseline` on session startup/initialization, during read-only questions, architecture discussions, or turns where no tracked source/model files were modified.
- **WHEN TO RUN**: Execute quality gates **ONLY ONCE** at the end of an implementation task after modifying tracked code/model files and immediately before `git commit` / `git push`.

### Hard Constraints (When Modifying Tracked Code/Model Files)
- **Scoped Pre-Commit Quality Gates**:
  - Run `cargo test --workspace` ONLY when Rust (`*.rs`, `Cargo.toml`) or SysML/layout generator files were modified.
  - Run headless `flutter test` (excluding `integration_test/`) ONLY when Dart/Flutter (`app_flutter/`) or layout JSON files were modified.
  - Run `./target/release/verify-baseline` once immediately prior to committing.
- **Remote Sync Verification**: When committing changes, push to the remote tracking branch and verify `git diff origin/<branch>` is empty.
- **Canonical DEAP Taxonomy**: Organize work strictly by Pipeline Steps (`Step 1..Step 4`), Epics (`EPIC-XX`), Features (`FEAT-XX`), User Stories (`US-XX`), Use Cases (`UC-XX`), Micro-Tasks (`Task 1..Task N`), and GitHub Issue IDs (`#Issue`). Never use `WP-*` or `Phase *` labels, and never create `implementation_plan.md` or `docs/reports/PHASE_*` files.
- **Opt-In Desktop Visual E2E Suites**: Never run `flutter test integration_test/... -d macos` unless explicitly commanded by the user in the current prompt.
- **Desktop Layout & Readability Invariants**: Multi-panel workspace tiers default to equal split proportions (`1/3 : 1/3 : 1/3`) unless specified otherwise by the deterministic Rust LUI AST, with zero `RenderFlex` overflows or truncated headers across standard desktop viewports (`>= 1920x1200`).

## What counts as proof

- Raw test output (pasted terminal output showing test results)
- Build output (pasted compiler/bundler output showing success)
- Explicit file-content verification (showing the actual content of generated/modified files)

## What does NOT count as proof

- "It works" or "tests pass" without pasted evidence
- "I verified it" without showing what was verified
- Summaries of what should have happened
- Assumptions based on previous steps succeeding

## Why

Agents are prone to hallucinating success. Requiring raw evidence forces ground-truth verification and makes false completions impossible to hide.
