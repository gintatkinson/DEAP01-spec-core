<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Verification-Before-Completion

**ALWAYS enforce:** Before declaring any task, micro-task, or feature complete, you MUST provide **concrete proof of correctness**.

## What counts as proof

- Raw test output (pasted terminal output showing test results)
- Build output (pasted compiler/bundler output showing success)
- Explicit file-content verification (showing the actual content of generated/modified files)

## What does NOT count as proof

- "It works" or "tests pass" without pasted evidence
- "I verified it" without showing what was verified
- Summaries of what should have happened
- Assumptions based on previous steps succeeding

<a id="rule-verification-required"></a>
## 20. Rule: Verification Required (`rules/verification-required.md`)

**ALWAYS enforce:** Never assert state or completion without direct, empirical verification.

### Hard Constraints
- **Mandatory Quality Gates**: Run `cargo test --workspace`, headless unit/widget tests (`flutter test` excluding `integration_test/`), and `./target/release/verify-baseline` before declaring completion.
- **Remote Sync Verification**: Push all commits to the remote tracking branch and verify `git diff origin/<branch>` is empty.
- **Forensic Verification Evidence**: Document terminal test results, commit hashes, and GitHub issue transitions (`#Issue`) in issue comments and handoff summaries.
- **Canonical DEAP Taxonomy**: Organize all work strictly by Pipeline Steps (`Step 1..Step 4`), Epics (`EPIC-XX`), Features (`FEAT-XX`), User Stories (`US-XX`), Use Cases (`UC-XX`), Micro-Tasks (`Task 1..Task N`), and GitHub Issue IDs (`#Issue`). Never invent `WP-*` (Work Package) or `Phase *` labels, and never create workspace `implementation_plan.md` or `docs/reports/PHASE_*` files.
- **Opt-In Desktop Visual E2E Suites**: Do not run `flutter test integration_test/... -d macos` screenshot suites during atomic defect fixes or unit/widget tasks unless explicitly requested by the user.
- **Desktop Layout & Readability Invariants**: Multi-panel workspace tiers default to equal split proportions (`1/3 : 1/3 : 1/3`) unless specified otherwise by the deterministic Rust LUI AST, with zero `RenderFlex` overflows or truncated headers across standard desktop viewports (`>= 1920x1200`).

## Why

Agents are prone to hallucinating success. Requiring raw evidence forces ground-truth verification and makes false completions impossible to hide.
