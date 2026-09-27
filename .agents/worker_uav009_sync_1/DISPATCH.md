# Dispatch Instructions — worker_uav009_sync_1

Execute view_file on skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder
Target Workspace: /Users/perkunas/jail/uav-009
Original Request: /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md
Working Directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_uav009_sync_1

## Task Objective: WP-03 Git Stage, Commit & Remote Push for uav-009
1. Read `/Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md`.
2. Inspect git status in `/Users/perkunas/jail/uav-009`:
   - Identify all updated pipeline framework assets (.pipeline/, scripts/, skills/, rules/, HANDOFF.md, .gitignore).
   - Ensure customer models (`schema/avenger5_system.sysml`), compiled AST (`.pipeline/schema.sysml`), defect reports (`docs/reports/`), and 75 customer specifications are strictly preserved.
3. Stage all pipeline framework assets using `git -C /Users/perkunas/jail/uav-009 add`.
4. Commit the changes using verbatim neutral citation:
   ```bash
   git -C /Users/perkunas/jail/uav-009 commit -m "chore(pipeline): propagate upstream spec-core fixes and Check 31 SSOT parity gate (refs #378, refs #377, refs #376, refs #375, refs #372, refs #366, refs #365, refs #364, refs #362, refs #361, refs #360, refs #349, refs #286)"
   ```
5. Verify commit neutrality using the commit message validator:
   ```bash
   python3 /Users/perkunas/jail/uav-009/scripts/verify_commit_messages.py --head
   ```
6. Push to remote tracking branch on GitLab:
   ```bash
   git -C /Users/perkunas/jail/uav-009 push origin main
   ```
7. Verify remote synchronization:
   ```bash
   git -C /Users/perkunas/jail/uav-009 diff origin/main
   ```
   Confirm diff returns 0 bytes and working tree is clean.
8. Capture the HEAD commit hash:
   ```bash
   git -C /Users/perkunas/jail/uav-009 rev-parse HEAD
   ```
9. Write a comprehensive handoff report to `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_uav009_sync_1/handoff.md` detailing:
   - Git status before and after
   - Staged files
   - Commit hash and commit message verification output
   - Push output and `git diff origin/main` 0-byte verification
   - Captured commit hash for HANDOFF.md Section 2.1 matrix.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_preview_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

PROCEED
