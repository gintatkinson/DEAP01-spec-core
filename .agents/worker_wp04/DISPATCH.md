# Task Assignment: WP-04a Git Stage, Neutral Citation Commit & Remote Push
## 2026-09-27T16:08:33Z

Execute view_file on skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target Workspace: /Users/perkunas/jail/DEAP01-spec-core
Working directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp04
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder
Original Request Path: /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md
Implementation Plan Path: /Users/perkunas/jail/DEAP01-spec-core/implementation_plan.md
Worker WP02 Handoff Path: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp02/handoff.md
Worker WP03 Handoff Path: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp03/handoff.md

You are assigned Work Package WP-04a: Git Stage, Neutral Citation Commit & Remote Synchronization.

DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_preview_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Tasks:
1. In `/Users/perkunas/jail/DEAP01-spec-core`:
   Inspect `git status` to see modified files (`README.md`, `scripts/install_pipeline.sh`, `tests/test_readme_scaffolding.py`, `implementation_plan.md`).
2. Stage modified files:
   `git add README.md scripts/install_pipeline.sh tests/test_readme_scaffolding.py implementation_plan.md`
3. Commit using exact neutral citation:
   `git commit -m "docs(readme): normalize architecture tiers, fix heading sequence, and harden repository boundary (refs #371, refs #368)"`
4. Run commit neutrality validator:
   `python3 scripts/verify_commit_messages.py --head`
   Confirm exit code 0 and zero issue-closing verbs.
5. Push to GitHub remote tracking branch:
   `git push origin main`
6. Verify remote synchronization:
   `git diff origin/main` (must return exactly 0 bytes).
7. Deliver your report to `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_wp04/handoff.md`.
8. Notify orchestrator when done via send_message.

PROCEED

