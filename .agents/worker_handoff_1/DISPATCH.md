# Dispatch Instructions — worker_handoff_1

Execute view_file on skills/spec-orchestrator/SKILL.md as your very first step before executing any file edits or commands, and strictly follow its formatting templates and instruction guidelines.

Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Primary Commercial Toolchain Integration Context: MATLAB / Simulink / Stateflow / Embedded Coder
Target Workspace: /Users/perkunas/jail/DEAP01-spec-core
Original Request: /Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md
Working Directory: /Users/perkunas/jail/DEAP01-spec-core/.agents/worker_handoff_1

## Task Objective: WP-07 Update Fleet Parity Matrix in HANDOFF.md & Remote Push (R7)
1. Read `/Users/perkunas/jail/DEAP01-spec-core/.agents/ORIGINAL_REQUEST.md`.
2. In `/Users/perkunas/jail/DEAP01-spec-core/HANDOFF.md`:
   - Line 10: Update customer baseline commits: `uav-009 6ac6d86` / `uav-011 fddddcd`.
   - Section 2.1 Table:
     * Update `/Users/perkunas/jail/uav-009` Local Baseline Commit to `6ac6d86` (reflecting Check 31 Dual-Schema SSOT Parity Gate verified).
     * Update `/Users/perkunas/jail/uav-011` Local Baseline Commit to `fddddcd` (reflecting Check 31 Dual-Schema SSOT Parity Gate verified).
   - Section 2.2 Subsection 2: Update `uav-009` baseline commit reference to `6ac6d86` (`chore(pipeline): propagate upstream spec-core fixes and Check 31 SSOT parity gate`).
   - Section 2.2 Subsection 3: Update `uav-011` baseline commit reference to `fddddcd` (`chore(pipeline): propagate upstream spec-core fixes and Check 31 SSOT parity gate`).
3. Stage `HANDOFF.md` in `DEAP01-spec-core`:
   ```bash
   git -C /Users/perkunas/jail/DEAP01-spec-core add HANDOFF.md
   ```
4. Commit using verbatim neutral citation:
   ```bash
   git -C /Users/perkunas/jail/DEAP01-spec-core commit -m "docs(handoff): update fleet parity baseline commit matrix (refs #372)"
   ```
5. Verify commit neutrality:
   ```bash
   python3 /Users/perkunas/jail/DEAP01-spec-core/scripts/verify_commit_messages.py --head
   ```
6. Push to GitHub remote tracking branch:
   ```bash
   git -C /Users/perkunas/jail/DEAP01-spec-core push origin main
   ```
7. Verify remote synchronization:
   ```bash
   git -C /Users/perkunas/jail/DEAP01-spec-core diff origin/main
   ```
   Confirm diff is 0 bytes and working tree is clean.
8. Capture the HEAD commit hash of `DEAP01-spec-core`:
   ```bash
   git -C /Users/perkunas/jail/DEAP01-spec-core rev-parse HEAD
   ```
9. Write a comprehensive handoff report to `/Users/perkunas/jail/DEAP01-spec-core/.agents/worker_handoff_1/handoff.md`.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_preview_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

PROCEED
